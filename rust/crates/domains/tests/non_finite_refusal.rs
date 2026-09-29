//! A flow that computes a NaN or an infinity **fails the step**; it does not reach the
//! conservation check.
//!
//! Why this is pinned, and why here: the engine's per-step conservation check compares
//! `residual.abs() > tol`, and a NaN residual passes that comparison silently. A 2026-09-29
//! record called that a live hole needing an engine unfreeze. It is not live: a value enters a
//! step only through a leg, a stock amount, an aux value or a forcing, and each is refused when
//! non-finite before the check runs:
//!
//! * a flow's leg: `Leg::new` (**pinned here** — nothing pinned it);
//! * a stock amount: `Stock::new` (`simcore/src/state.rs` unit test) and `Stock::with_amount`,
//!   the per-step write the integrator uses (**pinned here** — the `simcore` test calls only
//!   `Stock::new`);
//! * an aux value: `State::new` (`simcore/tests/aux_channel.rs`);
//! * a forcing: `SourceResolver::bind` (`simcore/tests/environment_wiring.rs`).
//!
//! ⚠ **Measured, both layers.** With `Leg::new`'s check disabled, a NaN flow is still refused
//! one layer down, when `with_amount` writes the new pool. With **both** disabled, the step
//! completes, the pool reads NaN, and the conservation check passes it — so the blind spot is
//! real and is covered twice, not absent.
//!
//! ⚠ **Which test sees which removal.** The step test cannot see `with_amount`'s check go
//! alone, because `Leg::new` refuses the value first; and it sees `Leg::new`'s go alone only
//! incidentally, because RK4 then reports the infinity as an over-draw. So each layer has its
//! own direct test below, and the step test pins the combined outcome.
//!
//! ⚠ The one theoretical way past both is overflow — two finite amounts near ±1.8e308 whose
//! difference is infinite — which no scenario comes within 300 orders of magnitude of.
//!
//! ⚠ It lives in `domains`, not `simcore`, because the engine core is frozen with no unfreeze
//! path (`docs/biosphere-reference.md`, the unfreeze discipline, step 2); a test file there is
//! still a diff under `simcore/`. `docs/log/what-if-experiments.md` has the record.

use std::collections::{BTreeMap, HashMap};

use simcore::boundary;
use simcore::environment::{Environment, SourceResolver};
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::integrator::{EulerIntegrator, Rk4Integrator};
use simcore::quantities::{Quantity, StockKind};
use simcore::registry::Registry;
use simcore::state::{State, Stock};

const POOL: &str = "sim.pool";
const SINK: &str = "sim.snk";

/// Moves `amount · factor` out of the pool, where `factor` is a what-if that went wrong.
struct Degenerate(f64);
impl Flow for Degenerate {
    fn type_name(&self) -> &'static str {
        "Degenerate"
    }
    fn id(&self) -> &str {
        "sim.degenerate"
    }
    fn evaluate(&self, s: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let flux = s.stocks[POOL].amount * self.0 * dt;
        FlowResult::new(vec![
            Leg::new(POOL.to_string(), -flux)?,
            Leg::new(SINK.to_string(), flux)?,
        ])
    }
}

fn stocks() -> BTreeMap<String, Stock> {
    let pool = Stock::new(
        POOL.to_string(),
        "sim".to_string(),
        Quantity::Carbon,
        "mol".to_string(),
        10.0,
        StockKind::Pool,
        0.0,
        false,
        BTreeMap::new(),
    )
    .unwrap();
    let sink = boundary::sink(SINK.to_string(), Quantity::Carbon, 0.0).unwrap();
    BTreeMap::from([(POOL.to_string(), pool), (sink.id.clone(), sink)])
}

fn is_refused(r: Result<State, SimError>, bad: f64) {
    match r {
        Err(SimError::Validation(msg)) => assert!(msg.contains("not finite"), "{bad:?}: {msg}"),
        Ok(s) => panic!(
            "a {bad:?} flow completed a step (pool now {:?}) — the conservation check cannot \
             see a NaN, so the refusals upstream of it are all that stops a quiet run",
            s.stocks[POOL].amount
        ),
        Err(other) => panic!("wrong error for {bad:?}: {other:?}"),
    }
}

/// The first layer by itself: a leg cannot hold a non-finite amount.
#[test]
fn a_leg_rejects_a_non_finite_amount() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        match Leg::new(POOL.to_string(), bad) {
            Err(SimError::Validation(msg)) => assert!(msg.contains("not finite"), "{bad:?}: {msg}"),
            other => panic!("Leg::new accepted or misreported {bad:?}: {other:?}"),
        }
    }
}

/// The second layer by itself: the per-step amount write refuses a non-finite value.
#[test]
fn a_stock_update_rejects_a_non_finite_amount() {
    let pool = &stocks()[POOL];
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        match pool.with_amount(bad) {
            Err(SimError::Validation(msg)) => assert!(msg.contains("not finite"), "{bad:?}: {msg}"),
            other => panic!("with_amount accepted or misreported {bad:?}: {other:?}"),
        }
    }
}

#[test]
fn a_flow_that_computes_a_non_finite_amount_fails_the_step() {
    let env = SourceResolver::new(HashMap::new(), HashMap::new()).unwrap();
    let state = State::new(0, stocks(), 0, BTreeMap::new()).unwrap();
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let reg = || Registry::flows_only(vec![Box::new(Degenerate(bad))], &stocks()).unwrap();
        is_refused(EulerIntegrator::new(reg()).step(&state, &env, 0.25), bad);
        is_refused(Rk4Integrator::new(reg()).step(&state, &env, 0.25), bad);
    }
}

/// The control: the same flow with a finite factor runs, so the refusal above is about the
/// value and not about the test's wiring.
#[test]
fn the_same_flow_with_a_finite_factor_runs() {
    let env = SourceResolver::new(HashMap::new(), HashMap::new()).unwrap();
    let state = State::new(0, stocks(), 0, BTreeMap::new()).unwrap();
    let reg = Registry::flows_only(vec![Box::new(Degenerate(0.1))], &stocks()).unwrap();
    let next = EulerIntegrator::new(reg).step(&state, &env, 0.25).unwrap();
    assert_eq!(next.stocks[POOL].amount, 10.0 - 10.0 * 0.1 * 0.25);
}
