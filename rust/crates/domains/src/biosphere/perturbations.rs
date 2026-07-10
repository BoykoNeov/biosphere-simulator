//! Phase-8 (P8.5) — the **generic** perturbation primitives, the Rust port of the
//! shared half of `domains.biosphere.perturbations`.
//!
//! A perturbation is a **scenario-layer intervention composed onto the already-assembled
//! `(state, registry, resolver)`**, never a core/domain change (the Phase-3 discipline,
//! carried cross-domain by the sibling `station::perturbations`). This module holds the
//! two seam-types that are domain-agnostic — a **forcing-schedule transform**
//! ([`window_override`] + [`with_forcing`]) and an **added boundary [`LeakFlow`]** — which
//! the station composers reuse verbatim. The station-specific pieces (`ScaledFlow`, the
//! five `with_*` composers) live in `station::perturbations`.
//!
//! **The move-pipeline shape (Rust, not Python).** Python's `with_forcing` shallow-copies
//! the resolver's dict of callables; Rust cannot — a [`Schedule`] is a non-`Clone`
//! `Box<dyn Fn>`. So every forcing perturbation here *consumes* the [`SourceResolver`]
//! (via [`SourceResolver::into_parts`]), swaps one var's schedule, and rebuilds. This is
//! safe because a perturbation is applied **before** the session takes ownership — the
//! same "compose onto assembled inputs, then run" order as Python.
//!
//! **Zero parity concern.** These are diagnostic interventions (the Phase-3 "diagnostics,
//! no golden" precedent) — Phase 7 deliberately excluded them, and no golden pins a
//! perturbed run. Determinism (a re-run is bit-identical) is the no-golden insurance.

use simcore::environment::{Environment, Schedule, SourceResolver};
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::ids::{FlowId, StockId};
use simcore::state::State;

/// The leak's windowed-activation forcing var (1.0 inside the window, 0.0 outside).
/// **Local to the perturbation harness** — never added to any domain's stock/forcing
/// catalog, so a baseline assembly never carries it and the frozen goldens are untouched
/// (the Python `LEAK_VAR` discipline).
pub const LEAK_VAR: &str = "atmospheric_leak";

/// The boundary reservoir a [`LeakFlow`] vents into (boundary-domain, so the interior's
/// closure breaks over the window but total mass — interior + this sink — is conserved).
/// Also local to the harness. The composer builds the actual stock with a composition
/// mirroring the leaked pool's, so a `{Carbon:1, Oxygen:2}` CO₂ pool vents both quantities.
pub const LEAK_SINK: &str = "boundary.leak_sink";

// --- the forcing-schedule perturbation (no new flow / no structural change) -----------

/// A [`Schedule`] forcing `value` on `[start, end)`, else the wrapped `base` — the Rust
/// port of Python `window_override`.
///
/// A pure function of the integer step `n` (#14, the legitimate forcing seam): inside the
/// window it forces `value` (e.g. PAR → 0, or a leak's activation → 1); outside it defers
/// to the wrapped `base` schedule. `base` is **owned** (moved into the closure) — Rust
/// cannot borrow-and-copy a `Box<dyn Fn>`, so the wrapper takes ownership. `dt` threads to
/// `base` unchanged.
pub fn window_override(base: Schedule, start: u64, end: u64, value: f64) -> Schedule {
    Box::new(move |n, dt| if start <= n && n < end { value } else { base(n, dt) })
}

/// Rebuild `resolver` with one forcing `var` replaced/added; the shared map preserved —
/// the Rust port of Python `with_forcing`.
///
/// Consumes the resolver ([`SourceResolver::into_parts`]), inserts `schedule` under `var`,
/// and rebuilds via [`SourceResolver::new`] (which re-checks the forcing⊕shared
/// disjointness, #16, so a new forcing var must not collide with a shared var — the return
/// is `Result` where Python's is infallible, the disjointness re-check being a Rust
/// feature not a Python one). The `shared` map is carried verbatim.
pub fn with_forcing(
    resolver: SourceResolver,
    var: &str,
    schedule: Schedule,
) -> Result<SourceResolver, SimError> {
    let (mut forcings, shared) = resolver.into_parts();
    forcings.insert(var.to_string(), schedule);
    SourceResolver::new(forcings, shared)
}

// --- the added leak flow (a boundary sink) --------------------------------------------

/// A windowed boundary leak `pool -> sink`, first-order in the pool — the Rust port of
/// Python `LeakFlow`.
///
/// The rate law is first-order donor control `k_leak · pool · active(n) · dt` where
/// `active(n) = env.get(leak_var) ∈ {0, 1}` carries the calendar (the window lives in the
/// *schedule*, not the flow body — the Step-4 discipline). Structural positivity: the draw
/// → 0 as the pool → 0, and `k_leak·dt < 1` keeps the Euler backstop unfired
/// (`rationed == 0`) with no clamp. `sink` mirrors `pool`'s element composition (built by
/// the composer), so the leg is per-quantity balanced. `flux = k·pool·active·dt` is
/// dt-linear (the RK4 increment-form contract, though the biosphere is Euler-locked).
pub struct LeakFlow {
    id: FlowId,
    priority: i64,
    pool: StockId,
    sink: StockId,
    leak_var: String,
    k_leak: f64,
}

impl LeakFlow {
    /// Construct a leak `pool -> sink`, gated by `leak_var`, rate `k_leak` (per second).
    pub fn new(
        id: FlowId,
        priority: i64,
        pool: StockId,
        sink: StockId,
        leak_var: String,
        k_leak: f64,
    ) -> Self {
        LeakFlow {
            id,
            priority,
            pool,
            sink,
            leak_var,
            k_leak,
        }
    }
}

impl Flow for LeakFlow {
    fn id(&self) -> &str {
        &self.id
    }

    fn priority(&self) -> i64 {
        self.priority
    }

    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let active = env.get(&self.leak_var)?;
        // Character-for-character with Python: k_leak * pool.amount * active * dt.
        let amount = self.k_leak * snapshot.stocks[&self.pool].amount * active * dt;
        FlowResult::new(vec![
            Leg::new(self.pool.clone(), -amount)?,
            Leg::new(self.sink.clone(), amount)?,
        ])
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};

    use simcore::environment::constant;
    use simcore::quantities::{Quantity, StockKind};
    use simcore::state::Stock;

    use super::*;

    fn pool(id: &str, amount: f64) -> Stock {
        Stock::new(
            id.to_string(),
            "d".to_string(),
            Quantity::Carbon,
            "mol".to_string(),
            amount,
            StockKind::Pool,
            0.0,
            false,
            BTreeMap::new(),
        )
        .unwrap()
    }

    #[test]
    fn window_override_forces_inside_defers_outside() {
        let base = constant(3.0).unwrap();
        let sched = window_override(base, 2, 5, 0.0);
        assert_eq!(sched(1, 1.0), 3.0); // before → base
        assert_eq!(sched(2, 1.0), 0.0); // start (inclusive) → value
        assert_eq!(sched(4, 1.0), 0.0); // inside → value
        assert_eq!(sched(5, 1.0), 3.0); // end (exclusive) → base
    }

    #[test]
    fn with_forcing_swaps_one_var_preserves_the_rest() {
        let mut forcings: HashMap<String, Schedule> = HashMap::new();
        forcings.insert("a".to_string(), constant(1.0).unwrap());
        forcings.insert("b".to_string(), constant(2.0).unwrap());
        let resolver = SourceResolver::new(forcings, HashMap::new()).unwrap();
        let resolver = with_forcing(resolver, "a", constant(9.0).unwrap()).unwrap();
        let state = State::new(0, BTreeMap::new(), 0, BTreeMap::new()).unwrap();
        let bound = resolver.bind(&state, 1.0);
        assert_eq!(bound.get("a").unwrap(), 9.0); // swapped
        assert_eq!(bound.get("b").unwrap(), 2.0); // preserved
    }

    #[test]
    fn leak_flow_is_first_order_gated_and_balanced() {
        let stocks = BTreeMap::from([
            ("p".to_string(), pool("p", 50.0)),
            (LEAK_SINK.to_string(), pool(LEAK_SINK, 0.0)),
        ]);
        let state = State::new(3, stocks, 0, BTreeMap::new()).unwrap();
        let leak = LeakFlow::new(
            "test.leak".to_string(),
            0,
            "p".to_string(),
            LEAK_SINK.to_string(),
            LEAK_VAR.to_string(),
            1.0e-3,
        );
        // active = 1 → the leak flows.
        let mut on: HashMap<String, Schedule> = HashMap::new();
        on.insert(LEAK_VAR.to_string(), constant(1.0).unwrap());
        let res_on = SourceResolver::new(on, HashMap::new()).unwrap();
        let out = leak
            .evaluate(&state, &res_on.bind(&state, 60.0), 60.0)
            .unwrap();
        // k·pool·active·dt = 1e-3·50·1·60 = 3.0 out of the pool, into the sink (balanced).
        assert_eq!(out.legs[0], Leg::new("p".to_string(), -3.0).unwrap());
        assert_eq!(out.legs[1], Leg::new(LEAK_SINK.to_string(), 3.0).unwrap());
        // active = 0 (outside the window) → the leak is inert.
        let mut off: HashMap<String, Schedule> = HashMap::new();
        off.insert(LEAK_VAR.to_string(), constant(0.0).unwrap());
        let res_off = SourceResolver::new(off, HashMap::new()).unwrap();
        let inert = leak
            .evaluate(&state, &res_off.bind(&state, 60.0), 60.0)
            .unwrap();
        assert_eq!(inert.legs[0].amount, 0.0);
        assert_eq!(inert.legs[1].amount, 0.0);
    }
}
