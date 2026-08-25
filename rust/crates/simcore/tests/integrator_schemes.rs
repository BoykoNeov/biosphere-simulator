//! The Euler / RK4 stepping spine — the Rust port of `tests/test_integrator.py`'s residue
//! (Stage-3 slice S4).
//!
//! `integrator.rs` carries two `#[cfg(test)]` unit tests (`combine` over the union of
//! stage keys, `reduce` per stock) and `laws.rs` carries the order-independence law. What
//! neither carries is the **arithmetic itself**: that one Euler step is `y + dt·rate`, that
//! RK4 is fourth-order where Euler is first, that a flow's legs are dt-linear (the
//! assumption the increment-form RK4 derivation rests on), and that all four RK4 stages
//! read the forcing at the step's own `n`. Those are this file.
//!
//! ## What is deliberately *not* here, and why that is not a gap
//!
//! * **Registration-order independence** for both schemes is
//!   `laws.rs::law_step_is_registration_order_independent_for_both_integrators`, which
//!   enumerates the permutations exhaustively rather than sampling them. Re-porting the
//!   Python case would be a second, weaker copy.
//! * **"Both integrators satisfy the `Integrator` protocol"** is unrepresentable: in Rust
//!   a type that does not implement the trait does not compile, so the Python assertion has
//!   no failing state to guard. Recorded, not silently dropped.
//!
//! Scenarios here are non-arbitrating and finite-supply on purpose — no over-draw, no
//! extinction. Those live in `extinction.rs` and the arbitration tests.

use std::collections::BTreeMap;

use simcore::boundary;
use simcore::environment::{Environment, Schedule, SourceResolver};
use simcore::error::SimError;
use simcore::flow::{assert_flow_balanced_default, Flow, FlowResult, Leg};
use simcore::integrator::{EulerIntegrator, Rk4Integrator};
use simcore::quantities::{Quantity, StockKind};
use simcore::registry::Registry;
use simcore::state::{State, Stock};

const A: &str = "bio.a";
const B: &str = "bio.b";
const DST: &str = "bio.dst";
const SINK_A: &str = "boundary.sink_a";
const SINK_B: &str = "boundary.sink_b";
const SRC: &str = "boundary.src";

/// The two schemes as a value — see `extinction.rs` for why a `simcore` test dispatches by
/// hand instead of over a trait.
#[derive(Debug, Clone, Copy)]
enum Scheme {
    Euler,
    Rk4,
}

impl Scheme {
    fn step(self, registry: Registry, state: &State, env: &SourceResolver, dt: f64) -> State {
        match self {
            Scheme::Euler => EulerIntegrator::new(registry).step(state, env, dt),
            Scheme::Rk4 => Rk4Integrator::new(registry).step(state, env, dt),
        }
        .expect("step")
    }
}

fn pool(id: &str, amount: f64, quantity: Quantity) -> Stock {
    Stock::new(
        id.to_string(),
        "bio".to_string(),
        quantity,
        quantity.canonical_unit(),
        amount,
        StockKind::Pool,
        0.0,
        false,
        BTreeMap::new(),
    )
    .expect("pool")
}

fn carbon_pool(id: &str, amount: f64) -> Stock {
    pool(id, amount, Quantity::Carbon)
}

fn stocks_of(list: Vec<Stock>) -> BTreeMap<String, Stock> {
    list.into_iter().map(|s| (s.id.clone(), s)).collect()
}

fn state_of(stocks: &BTreeMap<String, Stock>, n: u64) -> State {
    State::new(n, stocks.clone(), 0, BTreeMap::new()).expect("state")
}

/// `src -> sink` at first-order rate `rate` — dt-linear, balanced, and clear of both
/// extinction (a POOL source) and over-draw for `rate·dt < 1`.
struct DecayFlow {
    id: String,
    src: String,
    sink: String,
    rate: f64,
}

impl Flow for DecayFlow {
    fn type_name(&self) -> &'static str {
        "DecayFlow"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, snapshot: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let amount = self.rate * snapshot.stocks[&self.src].amount * dt;
        FlowResult::new(vec![
            Leg::new(self.src.clone(), -amount)?,
            Leg::new(self.sink.clone(), amount)?,
        ])
    }
}

/// `src -> dst` moving a fixed fraction of `src` per step (dt-linear).
struct TransferFlow {
    id: String,
    src: String,
    dst: String,
    frac: f64,
}

impl Flow for TransferFlow {
    fn type_name(&self) -> &'static str {
        "TransferFlow"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, snapshot: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let amount = self.frac * snapshot.stocks[&self.src].amount * dt;
        FlowResult::new(vec![
            Leg::new(self.src.clone(), -amount)?,
            Leg::new(self.dst.clone(), amount)?,
        ])
    }
}

/// Active only while `gate`'s amount is below `threshold`, moving `rate·dt`. When inactive
/// it returns **empty** legs, so the stocks it touches are *absent* from that stage's `k` —
/// which is what makes it a probe of the union-of-keys combine.
struct GatedFlow {
    id: String,
    gate: String,
    threshold: f64,
    src: String,
    sink: String,
    rate: f64,
}

impl Flow for GatedFlow {
    fn type_name(&self) -> &'static str {
        "GatedFlow"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, snapshot: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        if snapshot.stocks[&self.gate].amount < self.threshold {
            let x = self.rate * dt;
            return FlowResult::new(vec![
                Leg::new(self.src.clone(), -x)?,
                Leg::new(self.sink.clone(), x)?,
            ]);
        }
        Ok(FlowResult::empty())
    }
}

/// Deposits `env.get(var)·dt` into `dst` from a boundary `src` (balanced).
struct ForcingDepositFlow {
    id: String,
    var: String,
    src: String,
    dst: String,
}

impl Flow for ForcingDepositFlow {
    fn type_name(&self) -> &'static str {
        "ForcingDepositFlow"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, _snapshot: &State, env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let v = env.get(&self.var)? * dt;
        FlowResult::new(vec![
            Leg::new(self.src.clone(), -v)?,
            Leg::new(self.dst.clone(), v)?,
        ])
    }
}

/// A flow whose leg names a stock that does not exist — the apply path's referential
/// integrity probe.
struct GhostFlow;

impl Flow for GhostFlow {
    fn type_name(&self) -> &'static str {
        "GhostFlow"
    }
    fn id(&self) -> &str {
        "ghost"
    }
    fn evaluate(&self, _snapshot: &State, _env: &dyn Environment, _dt: f64) -> Result<FlowResult, SimError> {
        FlowResult::new(vec![Leg::new("bio.ghost".to_string(), 1.0)?])
    }
}

// --------------------------------------------------------------------------- //
// Euler: the formula itself                                                    //
// --------------------------------------------------------------------------- //

/// One explicit-Euler step is exactly `y + dt·rate(y)` — asserted on the nose, not within a
/// tolerance, because at one step there is nothing to accumulate.
#[test]
fn one_euler_step_is_y_plus_dt_times_the_rate() {
    let (a0, rate, dt) = (10.0, 0.3, 0.5);
    let stocks = stocks_of(vec![
        carbon_pool(A, a0),
        boundary::sink(SINK_A.to_string(), Quantity::Carbon, 0.0).unwrap(),
    ]);
    let state = state_of(&stocks, 0);
    let flow = DecayFlow {
        id: "decay".to_string(),
        src: A.to_string(),
        sink: SINK_A.to_string(),
        rate,
    };

    let next = EulerIntegrator::new(Registry::flows_only(vec![Box::new(flow)], &stocks).unwrap())
        .step(&state, &SourceResolver::empty(), dt)
        .expect("step");

    let moved = rate * a0 * dt;
    assert_eq!(next.n, 1);
    assert_eq!(next.stocks[A].amount, a0 - moved);
    assert_eq!(next.stocks[SINK_A].amount, moved);
}

// --------------------------------------------------------------------------- //
// RK4: accuracy and order                                                      //
// --------------------------------------------------------------------------- //

/// Integrate `ẏ = -λy` to `t_end` under `scheme` and return the absolute error against the
/// closed form `y0·e^{−λt}`.
fn decay_error(scheme: Scheme, dt: f64, lam: f64, t_end: f64) -> f64 {
    let a0 = 1.0;
    let steps = (t_end / dt).round() as u64;
    let stocks = stocks_of(vec![
        carbon_pool(A, a0),
        boundary::sink(SINK_A.to_string(), Quantity::Carbon, 0.0).unwrap(),
    ]);
    let mut state = state_of(&stocks, 0);
    let env = SourceResolver::empty();
    for _ in 0..steps {
        let flow = DecayFlow {
            id: "decay".to_string(),
            src: A.to_string(),
            sink: SINK_A.to_string(),
            rate: lam,
        };
        let registry = Registry::flows_only(vec![Box::new(flow)], &stocks).unwrap();
        state = scheme.step(registry, &state, &env, dt);
    }
    (state.stocks[A].amount - a0 * (-lam * t_end).exp()).abs()
}

#[test]
fn rk4_is_far_more_accurate_than_euler_at_the_same_dt() {
    let (lam, t_end, dt) = (1.0, 1.0, 0.1);
    let rk4 = decay_error(Scheme::Rk4, dt, lam, t_end);
    let euler = decay_error(Scheme::Euler, dt, lam, t_end);
    assert!(rk4 < euler / 1000.0, "rk4={rk4:e} euler={euler:e}");
}

/// The order claim, read off a halving of `dt`: a first-order method's error roughly
/// halves, a fourth-order method's drops by roughly 16×. Bands rather than points, because
/// the asymptotic constant is not the subject.
#[test]
fn rk4_is_fourth_order_where_euler_is_first_order() {
    let (lam, t_end) = (1.0, 1.0);
    let euler_ratio =
        decay_error(Scheme::Euler, 0.02, lam, t_end) / decay_error(Scheme::Euler, 0.01, lam, t_end);
    let rk4_ratio =
        decay_error(Scheme::Rk4, 0.02, lam, t_end) / decay_error(Scheme::Rk4, 0.01, lam, t_end);
    assert!(
        (1.8..2.2).contains(&euler_ratio),
        "euler halving ratio {euler_ratio} is not first-order"
    );
    assert!(
        (10.0..20.0).contains(&rk4_ratio),
        "rk4 halving ratio {rk4_ratio} is not fourth-order"
    );
}

/// The dt-linearity contract the increment-form RK4 derivation rests on: `evaluate(y, dt)`
/// must equal `dt·rate(y)` with `rate` independent of `dt`, so doubling `dt` doubles every
/// leg. A flow that violated this would make the ⅙-combine silently wrong rather than loud.
#[test]
fn flow_legs_scale_linearly_with_dt() {
    let stocks = stocks_of(vec![
        carbon_pool(A, 10.0),
        boundary::sink(SINK_A.to_string(), Quantity::Carbon, 0.0).unwrap(),
    ]);
    let state = state_of(&stocks, 0);
    let flow = DecayFlow {
        id: "decay".to_string(),
        src: A.to_string(),
        sink: SINK_A.to_string(),
        rate: 0.3,
    };
    let env = SourceResolver::empty();
    let bound = env.bind(&state, 0.1);

    let legs1: BTreeMap<String, f64> = flow
        .evaluate(&state, &bound, 0.1)
        .unwrap()
        .legs
        .iter()
        .map(|l| (l.stock.clone(), l.amount))
        .collect();
    let legs2: BTreeMap<String, f64> = flow
        .evaluate(&state, &bound, 0.2)
        .unwrap()
        .legs
        .iter()
        .map(|l| (l.stock.clone(), l.amount))
        .collect();

    assert_eq!(
        legs1.keys().collect::<Vec<_>>(),
        legs2.keys().collect::<Vec<_>>()
    );
    for (sid, amount) in &legs1 {
        assert!(
            (legs2[sid] - 2.0 * amount).abs() < 1e-15,
            "{sid}: {} is not 2x {amount}",
            legs2[sid]
        );
    }
}

// --------------------------------------------------------------------------- //
// The combine/apply arithmetic conserves                                       //
// --------------------------------------------------------------------------- //

/// Each `k_i` is a sum of balanced legs and the ⅙-combine is linear, so the *realized*
/// per-step delta must balance too. Asserted at test level with the balance helper — the
/// same claim the runtime gate makes, stated where a failure names this arithmetic.
fn applied_delta_conserves(scheme: Scheme) {
    let stocks = stocks_of(vec![
        carbon_pool(A, 100.0),
        carbon_pool(B, 10.0),
        boundary::sink(SINK_A.to_string(), Quantity::Carbon, 0.0).unwrap(),
    ]);
    let state = state_of(&stocks, 0);
    let registry = Registry::flows_only(
        vec![
            Box::new(TransferFlow {
                id: "transfer".to_string(),
                src: A.to_string(),
                dst: B.to_string(),
                frac: 0.1,
            }),
            Box::new(DecayFlow {
                id: "harvest".to_string(),
                src: B.to_string(),
                sink: SINK_A.to_string(),
                rate: 0.2,
            }),
        ],
        &stocks,
    )
    .unwrap();

    let next = scheme.step(registry, &state, &SourceResolver::empty(), 0.5);

    let legs: Vec<Leg> = state
        .stocks
        .keys()
        .map(|sid| Leg::new(sid.clone(), next.stocks[sid].amount - state.stocks[sid].amount).unwrap())
        .collect();
    assert_flow_balanced_default(&FlowResult::new(legs).unwrap(), &state.stocks)
        .unwrap_or_else(|e| panic!("{scheme:?}: the applied delta does not balance: {e}"));
}

#[test]
fn the_applied_euler_delta_conserves_mass() {
    applied_delta_conserves(Scheme::Euler);
}

#[test]
fn the_applied_rk4_delta_conserves_mass() {
    applied_delta_conserves(Scheme::Rk4);
}

/// The union-of-keys combine, at run level. `A` decays fast enough to cross the gate
/// threshold *within* the half-step perturbations: the gated flow is inactive at `y_n` (so
/// `B` is absent from `k1`) but active at stages 2–4. A combine that iterated only `k1`'s
/// keys would drop `B` entirely — and, unlike `integrator.rs`'s unit test on `combine`, this
/// case reaches that path through a real step, so it also pins that the stage states are
/// perturbed enough to get there.
#[test]
fn the_rk4_combine_includes_a_stock_touched_only_at_a_perturbed_stage() {
    let stocks = stocks_of(vec![
        carbon_pool(A, 10.0),
        carbon_pool(B, 10.0),
        boundary::sink(SINK_A.to_string(), Quantity::Carbon, 0.0).unwrap(),
        boundary::sink(SINK_B.to_string(), Quantity::Carbon, 0.0).unwrap(),
    ]);
    let state = state_of(&stocks, 0);
    let registry = Registry::flows_only(
        vec![
            Box::new(DecayFlow {
                id: "decay_a".to_string(),
                src: A.to_string(),
                sink: SINK_A.to_string(),
                rate: 0.2,
            }),
            Box::new(GatedFlow {
                id: "gated".to_string(),
                gate: A.to_string(),
                threshold: 9.5,
                src: B.to_string(),
                sink: SINK_B.to_string(),
                rate: 1.0,
            }),
        ],
        &stocks,
    )
    .unwrap();

    let next = Rk4Integrator::new(registry)
        .step(&state, &SourceResolver::empty(), 1.0)
        .expect("step");

    assert!(
        next.stocks[B].amount < 10.0 - 1e-9,
        "B was touched only at perturbed stages and must still have moved"
    );
    assert!(
        (next.stocks[SINK_B].amount - (10.0 - next.stocks[B].amount)).abs() < 1e-12,
        "and that movement conserves against its own sink"
    );
}

/// Forcing is piecewise-constant *within* a step: the schedule returns `n`, RK4 stage states
/// keep the step's `n`, so all four stages read the same value. An implementation that
/// advanced `n` per stage — or read `n+1` — would deposit something else.
#[test]
fn rk4_reads_the_forcing_at_the_steps_own_n_for_all_four_stages() {
    let (dt, start_n) = (0.5, 5u64);
    let stocks = stocks_of(vec![
        boundary::source(SRC.to_string(), Quantity::Energy, 1000.0, true).unwrap(),
        pool(DST, 0.0, Quantity::Energy),
    ]);
    let state = state_of(&stocks, start_n);
    let mut forcings: std::collections::HashMap<String, Schedule> = std::collections::HashMap::new();
    forcings.insert("f".to_string(), Box::new(|n, _dt| n as f64));
    let resolver = SourceResolver::new(forcings, std::collections::HashMap::new()).unwrap();
    let registry = Registry::flows_only(
        vec![Box::new(ForcingDepositFlow {
            id: "deposit".to_string(),
            var: "f".to_string(),
            src: SRC.to_string(),
            dst: DST.to_string(),
        })],
        &stocks,
    )
    .unwrap();

    let next = Rk4Integrator::new(registry)
        .step(&state, &resolver, dt)
        .expect("step");

    assert_eq!(next.n, start_n + 1);
    assert!(
        (next.stocks[DST].amount - (start_n as f64) * dt).abs() < 1e-12,
        "deposited {} rather than n*dt",
        next.stocks[DST].amount
    );
}

/// A leg naming a stock the state does not hold is referential integrity, caught in the
/// apply path rather than silently created.
#[test]
fn a_leg_to_an_unknown_stock_is_a_reference_error() {
    let stocks = stocks_of(vec![carbon_pool(A, 1.0)]);
    let state = state_of(&stocks, 0);
    let registry = Registry::flows_only(vec![Box::new(GhostFlow)], &stocks).unwrap();

    let err = EulerIntegrator::new(registry)
        .step(&state, &SourceResolver::empty(), 1.0)
        .expect_err("a ghost leg must fail");

    match err {
        SimError::Reference(msg) => assert!(msg.contains("unknown stock"), "{msg}"),
        other => panic!("expected a reference error, got {other:?}"),
    }
}

/// Two identical steps from one state produce bit-identical amounts. Weaker than
/// `laws.rs`'s order-independence law and kept anyway: it is the claim that a step has no
/// hidden state at all, which order-independence does not imply.
#[test]
fn stepping_the_same_state_twice_is_bit_identical() {
    let stocks = stocks_of(vec![
        carbon_pool(A, 100.0),
        carbon_pool(B, 10.0),
        carbon_pool(DST, 1.0),
        boundary::sink(SINK_A.to_string(), Quantity::Carbon, 0.0).unwrap(),
    ]);
    let state = state_of(&stocks, 0);
    let make = || {
        Registry::flows_only(
            vec![
                Box::new(TransferFlow {
                    id: "f_transfer".to_string(),
                    src: A.to_string(),
                    dst: B.to_string(),
                    frac: 0.1,
                }),
                Box::new(DecayFlow {
                    id: "a_decay".to_string(),
                    src: B.to_string(),
                    sink: SINK_A.to_string(),
                    rate: 0.2,
                }),
                Box::new(TransferFlow {
                    id: "m_transfer".to_string(),
                    src: A.to_string(),
                    dst: DST.to_string(),
                    frac: 0.05,
                }),
            ],
            &stocks,
        )
        .unwrap()
    };

    let first = Rk4Integrator::new(make())
        .step(&state, &SourceResolver::empty(), 0.5)
        .unwrap();
    let second = Rk4Integrator::new(make())
        .step(&state, &SourceResolver::empty(), 0.5)
        .unwrap();

    let bits = |s: &State| -> BTreeMap<String, u64> {
        s.stocks
            .iter()
            .map(|(id, st)| (id.clone(), st.amount.to_bits()))
            .collect()
    };
    assert_eq!(bits(&first), bits(&second));
}
