//! Integrator strategies: Euler and RK4 — the Rust port of `simcore.integrator`.
//!
//! Each full step: take an immutable snapshot, evaluate every flow against it in
//! canonical id-order, **arbitrate**, reduce legs per-stock in canonical order,
//! combine the scheme's evaluations, **apply once** (`n -> n+1`), run the extinction
//! pass, then assert the every-step conservation gate.
//!
//! Increment-form contract: `Flow::evaluate` returns `dt·rate` (not a bare rate), so:
//! * Euler:  `y_{n+1} = y_n + f(y_n)`                            (one evaluation)
//! * RK4:    `k1=f(y_n)`, `k2=f(y_n+½k1)`, `k3=f(y_n+½k2)`, `k4=f(y_n+k3)`, then
//!   `y_{n+1}=y_n+(k1+2k2+2k3+k4)/6` (four evaluations).
//!
//! Because every `k_i` already carries `dt`, the ⅙-combine reproduces classical RK4
//! exactly (holds only if `rate` is dt-independent). Arbitration asymmetry: Euler
//! scales over-drawing flows and reports the firing count; RK4 hard-errors on a needed
//! `scale_f < 1` ([`crate::arbitration::check_no_overdraw`]). Extinction, aux, and the
//! conservation gate are each in exactly one place so neither scheme can skip them.
//!
//! **Every arithmetic grouping below mirrors the Python source character-for-character**
//! (float `+`/`*` are commutative but not associative, so op order is load-bearing for
//! the cross-port bit-exact gate).

use std::collections::{BTreeMap, BTreeSet};

use crate::arbitration;
use crate::boundary::loss_sink_id;
use crate::conservation::assert_conserved_default;
use crate::environment::SourceResolver;
use crate::error::SimError;
use crate::events::{Event, ExtinctionEvent};
use crate::flow::FlowResult;
use crate::ids::StockId;
use crate::quantities::StockKind;
use crate::registry::Registry;
use crate::state::{State, Stock};

/// One step's result plus its diagnostics (the functional side-channel).
///
/// `state` is the produced `State` (`n -> n+1`). `events` are the step's extinctions
/// in canonical stock-id order. `rationed` is the number of flows scaled by the Euler
/// backstop this step (always 0 for RK4). A golden run sums `rationed` and asserts `== 0`.
#[derive(Debug, Clone, PartialEq)]
pub struct StepReport {
    pub state: State,
    pub events: Vec<Event>,
    pub rationed: u64,
}

/// A concrete integrator's amounts-only advance (the multi-rate building block).
/// `substep` is like `step_report` but keeps `State.n`, runs arbitration + extinction,
/// and does **not** advance aux or assert conservation — the multi-rate driver owns
/// the single `n -> n+1` commit and the composite gate.
pub trait Substepper {
    /// Amounts-only advance, keeping `n` (see the trait docs).
    fn substep(&self, state: &State, env: &SourceResolver, dt: f64) -> Result<StepReport, SimError>;
}

// --------------------------------------------------------------------------- //
// Shared step machinery (module-free-functions, the `_BaseIntegrator` spine).  //
// --------------------------------------------------------------------------- //

/// Evaluate every flow against `stage_state`, in canonical id-order. Binds `env` to
/// the **same** `stage_state` handed to `flow.evaluate` (the #16 seam).
fn evaluate_all(
    registry: &Registry,
    stage_state: &State,
    env: &SourceResolver,
    dt: f64,
) -> Result<Vec<FlowResult>, SimError> {
    let bound = env.bind(stage_state, dt);
    let mut out: Vec<FlowResult> = Vec::with_capacity(registry.len());
    for flow in registry.flows() {
        out.push(flow.evaluate(stage_state, &bound, dt)?);
    }
    Ok(out)
}

/// Per-stock delta map: sum legs over flows in canonical (input) order, legs in leg
/// order. Per-key accumulation happens in flow-order × leg-order (the `BTreeMap` is
/// storage only — this ordered traversal is what pins the float sum, #15).
fn reduce(results: &[FlowResult]) -> BTreeMap<StockId, f64> {
    let mut deltas: BTreeMap<StockId, f64> = BTreeMap::new();
    for result in results {
        for leg in &result.legs {
            *deltas.entry(leg.stock.clone()).or_insert(0.0) += leg.amount;
        }
    }
    deltas
}

/// Per-name aux increment: one Euler evaluation at `snapshot`. Aux processes iterate in
/// canonical `AuxId` order; each process's names are summed in sorted-name order; the
/// cross-process per-name sum runs in process order. Empty when there are no processes.
fn aux_increments(
    registry: &Registry,
    snapshot: &State,
    env: &SourceResolver,
    dt: f64,
) -> Result<BTreeMap<String, f64>, SimError> {
    let mut increments: BTreeMap<String, f64> = BTreeMap::new();
    if registry.aux_processes().is_empty() {
        return Ok(increments);
    }
    let bound = env.bind(snapshot, dt);
    for proc in registry.aux_processes() {
        // proc.evaluate returns a BTreeMap → already sorted-by-name (Python does
        // `for name in sorted(result)`), so this iteration matches the reference.
        let result = proc.evaluate(snapshot, &bound, dt)?;
        for (name, inc) in &result {
            *increments.entry(name.clone()).or_insert(0.0) += *inc;
        }
    }
    Ok(increments)
}

/// `state.aux` with each named accumulator advanced by its increment. Empty increments
/// ⇒ `state.aux` unchanged (the no-aux fast path). Each name is one independent addition.
fn advanced_aux(state: &State, increments: &BTreeMap<String, f64>) -> BTreeMap<String, f64> {
    if increments.is_empty() {
        return state.aux.clone();
    }
    let mut merged = state.aux.clone();
    for (name, inc) in increments {
        *merged.entry(name.clone()).or_insert(0.0) += *inc;
    }
    merged
}

/// `state.stocks` with each named stock's amount shifted by `factor*delta`. Owns
/// referential integrity: a delta on a stock absent from `state.stocks` returns
/// [`SimError::Reference`] (Python `KeyError`). Each stock's shift is independent.
fn shifted_stocks(
    state: &State,
    deltas: &BTreeMap<StockId, f64>,
    factor: f64,
) -> Result<BTreeMap<StockId, Stock>, SimError> {
    let mut stocks = state.stocks.clone();
    for (sid, delta) in deltas {
        let stock = stocks.get(sid).ok_or_else(|| {
            SimError::Reference(format!(
                "flow produced a leg on unknown stock {sid:?}; referential integrity is \
                 checked in the integrator apply path (step 5/6)"
            ))
        })?;
        // Character-for-character with Python: stock.amount + factor * deltas[sid].
        let new = stock.with_amount(stock.amount + factor * delta)?;
        stocks.insert(sid.clone(), new);
    }
    Ok(stocks)
}

/// An RK4 stage state: amounts shifted by `factor*deltas`, keeping `n` (and aux, seed).
fn perturb(
    state: &State,
    deltas: &BTreeMap<StockId, f64>,
    factor: f64,
) -> Result<State, SimError> {
    State::new(
        state.n,
        shifted_stocks(state, deltas, factor)?,
        state.rng_seed,
        state.aux.clone(),
    )
}

/// Write the step result: amounts shifted by `deltas`, aux advanced, `n -> n+1`.
fn apply(
    state: &State,
    deltas: &BTreeMap<StockId, f64>,
    aux_incs: &BTreeMap<String, f64>,
) -> Result<State, SimError> {
    State::new(
        state.n + 1,
        shifted_stocks(state, deltas, 1.0)?,
        state.rng_seed,
        advanced_aux(state, aux_incs),
    )
}

/// RK4 ⅙-weighted combine over the **union** of stage keys (missing ⇒ 0). Iterating
/// only one stage's keys would drop a stock a state-gated flow touched at a perturbed
/// stage but not at `y_n`.
fn combine(
    k1: &BTreeMap<StockId, f64>,
    k2: &BTreeMap<StockId, f64>,
    k3: &BTreeMap<StockId, f64>,
    k4: &BTreeMap<StockId, f64>,
) -> BTreeMap<StockId, f64> {
    let keys: BTreeSet<&StockId> = k1.keys().chain(k2.keys()).chain(k3.keys()).chain(k4.keys()).collect();
    let g = |m: &BTreeMap<StockId, f64>, s: &StockId| m.get(s).copied().unwrap_or(0.0);
    let mut out: BTreeMap<StockId, f64> = BTreeMap::new();
    for s in keys {
        // Character-for-character with Python:
        // (k1 + 2.0*k2 + 2.0*k3 + k4) / 6.0
        let v = (g(k1, s) + 2.0 * g(k2, s) + 2.0 * g(k3, s) + g(k4, s)) / 6.0;
        out.insert(s.clone(), v);
    }
    out
}

/// One RK4 derivative evaluation: evaluate → hard-error guard → reduce.
fn rk4_stage(
    registry: &Registry,
    stage_state: &State,
    env: &SourceResolver,
    dt: f64,
) -> Result<BTreeMap<StockId, f64>, SimError> {
    let results = evaluate_all(registry, stage_state, env, dt)?;
    arbitration::check_no_overdraw(&results, &stage_state.stocks)?;
    Ok(reduce(&results))
}

/// Step-algorithm #6: snap below-threshold POPULATION stocks to 0, routing the snapped
/// residual to the quantity's numerical-loss sink so the ledger balances. Stocks are
/// scanned and loss deposits applied in canonical (sorted) id order.
fn extinction_pass(state: &State) -> Result<(State, Vec<ExtinctionEvent>), SimError> {
    let mut events: Vec<ExtinctionEvent> = Vec::new();
    let mut snapped: BTreeMap<StockId, Stock> = BTreeMap::new();
    let mut loss_deltas: BTreeMap<StockId, f64> = BTreeMap::new();
    for (sid, stock) in &state.stocks {
        if stock.kind != StockKind::Population {
            continue;
        }
        if stock.amount < stock.extinction_threshold && stock.amount != 0.0 {
            let residual = stock.amount;
            snapped.insert(sid.clone(), stock.with_amount(0.0)?);
            let ls_id = loss_sink_id(stock.quantity);
            *loss_deltas.entry(ls_id).or_insert(0.0) += residual;
            events.push(ExtinctionEvent {
                n: state.n,
                stock: sid.clone(),
                quantity: stock.quantity,
                residual,
            });
        }
    }
    if snapped.is_empty() {
        return Ok((state.clone(), Vec::new()));
    }
    let mut new_stocks = state.stocks.clone();
    for (sid, s) in snapped {
        new_stocks.insert(sid, s);
    }
    for (ls_id, delta) in &loss_deltas {
        let ls = new_stocks.get(ls_id).ok_or_else(|| {
            SimError::Reference(format!(
                "extinction routes a residual to loss-sink {ls_id:?} but it is absent \
                 from State.stocks; the initial state must include the boundary \
                 loss-sinks (decision #6 / referential integrity)"
            ))
        })?;
        let updated = ls.with_amount(ls.amount + delta)?;
        new_stocks.insert(ls_id.clone(), updated);
    }
    let next = State::new(state.n, new_stocks, state.rng_seed, state.aux.clone())?;
    Ok((next, events))
}

/// Shared post-apply tail: extinction pass → conservation gate → `StepReport`.
fn finalize(before: &State, applied: &State, rationed: u64) -> Result<StepReport, SimError> {
    let (next, events) = extinction_pass(applied)?;
    assert_conserved_default(before, &next)?;
    Ok(StepReport {
        state: next,
        events,
        rationed,
    })
}

/// Multi-rate sub-step tail: extinction pass only — **no** conservation gate (asserted
/// once at the composite master-step boundary).
fn finalize_substep(advanced: &State, rationed: u64) -> Result<StepReport, SimError> {
    let (next, events) = extinction_pass(advanced)?;
    Ok(StepReport {
        state: next,
        events,
        rationed,
    })
}

// --------------------------------------------------------------------------- //
// The scheme abstraction: `_deltas` differs; everything else is shared.        //
// --------------------------------------------------------------------------- //

/// The per-scheme combined delta map + rationing-firing count — the one piece that
/// differs between Euler and RK4. Everything else (`step_report`/`substep`) is shared.
trait Scheme {
    fn registry(&self) -> &Registry;
    fn deltas(
        &self,
        state: &State,
        env: &SourceResolver,
        dt: f64,
    ) -> Result<(BTreeMap<StockId, f64>, u64), SimError>;
}

fn do_step_report<S: Scheme + ?Sized>(
    s: &S,
    state: &State,
    env: &SourceResolver,
    dt: f64,
) -> Result<StepReport, SimError> {
    let (deltas, rationed) = s.deltas(state, env, dt)?;
    let aux_incs = aux_increments(s.registry(), state, env, dt)?;
    let applied = apply(state, &deltas, &aux_incs)?;
    finalize(state, &applied, rationed)
}

fn do_substep<S: Scheme + ?Sized>(
    s: &S,
    state: &State,
    env: &SourceResolver,
    dt: f64,
) -> Result<StepReport, SimError> {
    let (deltas, rationed) = s.deltas(state, env, dt)?;
    let advanced = perturb(state, &deltas, 1.0)?;
    finalize_substep(&advanced, rationed)
}

/// Explicit Euler: one derivative evaluation per step, with the min-scaling backstop.
pub struct EulerIntegrator {
    registry: Registry,
}

impl EulerIntegrator {
    /// Construct an Euler integrator over `registry` (model structure).
    pub fn new(registry: Registry) -> Self {
        EulerIntegrator { registry }
    }

    /// The flow registry this integrator steps.
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// One full step, returning just the produced `State` (the frozen-API surface).
    pub fn step(&self, state: &State, env: &SourceResolver, dt: f64) -> Result<State, SimError> {
        Ok(self.step_report(state, env, dt)?.state)
    }

    /// One full step with diagnostics (events + rationing-firing count).
    pub fn step_report(
        &self,
        state: &State,
        env: &SourceResolver,
        dt: f64,
    ) -> Result<StepReport, SimError> {
        do_step_report(self, state, env, dt)
    }
}

impl Scheme for EulerIntegrator {
    fn registry(&self) -> &Registry {
        &self.registry
    }
    fn deltas(
        &self,
        state: &State,
        env: &SourceResolver,
        dt: f64,
    ) -> Result<(BTreeMap<StockId, f64>, u64), SimError> {
        let results = evaluate_all(&self.registry, state, env, dt)?;
        let (scaled, rationed) = arbitration::min_scaling(&results, &state.stocks)?;
        Ok((reduce(&scaled), rationed))
    }
}

impl Substepper for EulerIntegrator {
    fn substep(&self, state: &State, env: &SourceResolver, dt: f64) -> Result<StepReport, SimError> {
        do_substep(self, state, env, dt)
    }
}

/// Classical 4th-order Runge–Kutta in increment form (four evaluations/step). Stage
/// states keep the step's `n`; a needed `scale_f < 1` at any stage is a hard
/// [`SimError::Arbitration`]. `StepReport.rationed` is always 0 here.
pub struct Rk4Integrator {
    registry: Registry,
}

impl Rk4Integrator {
    /// Construct an RK4 integrator over `registry`.
    pub fn new(registry: Registry) -> Self {
        Rk4Integrator { registry }
    }

    /// The flow registry this integrator steps.
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// One full step, returning just the produced `State`.
    pub fn step(&self, state: &State, env: &SourceResolver, dt: f64) -> Result<State, SimError> {
        Ok(self.step_report(state, env, dt)?.state)
    }

    /// One full step with diagnostics.
    pub fn step_report(
        &self,
        state: &State,
        env: &SourceResolver,
        dt: f64,
    ) -> Result<StepReport, SimError> {
        do_step_report(self, state, env, dt)
    }
}

impl Scheme for Rk4Integrator {
    fn registry(&self) -> &Registry {
        &self.registry
    }
    fn deltas(
        &self,
        state: &State,
        env: &SourceResolver,
        dt: f64,
    ) -> Result<(BTreeMap<StockId, f64>, u64), SimError> {
        let reg = &self.registry;
        let k1 = rk4_stage(reg, state, env, dt)?;
        let k2 = rk4_stage(reg, &perturb(state, &k1, 0.5)?, env, dt)?;
        let k3 = rk4_stage(reg, &perturb(state, &k2, 0.5)?, env, dt)?;
        let k4 = rk4_stage(reg, &perturb(state, &k3, 1.0)?, env, dt)?;
        Ok((combine(&k1, &k2, &k3, &k4), 0))
    }
}

impl Substepper for Rk4Integrator {
    fn substep(&self, state: &State, env: &SourceResolver, dt: f64) -> Result<StepReport, SimError> {
        do_substep(self, state, env, dt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `combine`'s "missing key ⇒ 0.0" union branch is **dead** in the synthetic
    /// trajectory (every stage emits the same stock set), yet the plan names it as a
    /// Step-2 thing to pin. Exercise it directly with disjoint-key stages: a stock
    /// present only in k2, another only in k4, one shared across all four.
    #[test]
    fn combine_folds_over_the_union_of_stage_keys() {
        let k1 = BTreeMap::from([("shared".to_string(), 6.0)]);
        let k2 = BTreeMap::from([("shared".to_string(), 6.0), ("only_k2".to_string(), 3.0)]);
        let k3 = BTreeMap::from([("shared".to_string(), 6.0)]);
        let k4 = BTreeMap::from([("shared".to_string(), 6.0), ("only_k4".to_string(), 12.0)]);
        let out = combine(&k1, &k2, &k3, &k4);
        // shared: (6 + 2*6 + 2*6 + 6)/6 = 36/6 = 6.
        assert_eq!(out["shared"], 6.0);
        // only_k2: (0 + 2*3 + 0 + 0)/6 = 1.0.
        assert_eq!(out["only_k2"], 1.0);
        // only_k4: (0 + 0 + 0 + 12)/6 = 2.0.
        assert_eq!(out["only_k4"], 2.0);
        // The union has all three keys.
        assert_eq!(out.len(), 3);
    }

    /// `reduce` sums per stock in flow-order × leg-order (the container is storage).
    #[test]
    fn reduce_accumulates_per_stock() {
        let r1 = FlowResult::new(vec![
            crate::flow::Leg::new("a".to_string(), -2.0).unwrap(),
            crate::flow::Leg::new("b".to_string(), 2.0).unwrap(),
        ])
        .unwrap();
        let r2 = FlowResult::new(vec![crate::flow::Leg::new("a".to_string(), 0.5).unwrap()]).unwrap();
        let deltas = reduce(&[r1, r2]);
        assert_eq!(deltas["a"], -1.5);
        assert_eq!(deltas["b"], 2.0);
    }
}
