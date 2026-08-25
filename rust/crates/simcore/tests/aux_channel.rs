//! The non-conserved auxiliary channel — the Rust port of `tests/test_aux.py` (Stage-3
//! slice S4).
//!
//! `auxiliary.rs` has **no tests of its own**, and the only Rust test that touched aux at
//! all was `laws.rs`'s order-independence law. So the numerics of the channel — that it
//! advances by exactly one explicit-Euler evaluation per step under *both* schemes, that a
//! flow reading it sees a within-step constant, that it is outside the conservation gate
//! without weakening it, and that a sub-step leaves it alone — were asserted nowhere.
//!
//! ## The placement claim is the one that matters
//!
//! Aux is advanced once per **master** step, never per RK4 stage and never per multi-rate
//! sub-operation. Advancing it in the shared `substep` path would advance it `n_sub`× per
//! master step; advancing it per RK4 stage would advance it 4×. Both bugs conserve mass
//! perfectly — aux carries no conserved quantity — so *nothing else in the suite would go
//! red*. [`a_substep_leaves_aux_untouched_while_a_full_step_advances_it_once`] and
//! [`all_four_rk4_stages_see_the_step_entry_aux`] are the two gates on that.
//!
//! ## Deliberately not here
//!
//! * The aux sum's registration-order independence **and** its associativity pin are
//!   `laws.rs::law_aux_accumulator_sum_is_registration_order_independent`, which enumerates
//!   all six permutations and pins the canonical value at `0.0`. Re-porting would be a
//!   weaker copy of both.
//! * Four Python cases assert Python-level immutability: that `state.aux["x"] = 1.0` raises,
//!   that the mapping is read-only, that `State` detaches from the caller's dict, and that
//!   `AuxProcess` is `runtime_checkable`. In Rust a `State`'s map is owned and moved in, and
//!   a trait is checked at compile time — there is no failing state to guard. Recorded
//!   rather than dropped silently.

use std::collections::BTreeMap;
use std::sync::Mutex;

use simcore::auxiliary::AuxProcess;
use simcore::boundary;
use simcore::conservation::{assert_conserved_default, compute_ledger};
use simcore::environment::{constant, Environment, Schedule, SourceResolver};
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::integrator::{EulerIntegrator, Rk4Integrator, StepReport, Substepper};
use simcore::quantities::{Quantity, StockKind};
use simcore::registry::Registry;
use simcore::state::{State, Stock};

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

/// Constant-rate accumulator: the increment is `rate·dt` (increment form, like a flow leg).
struct ConstRateAux {
    id: String,
    name: String,
    rate: f64,
}

impl AuxProcess for ConstRateAux {
    fn type_name(&self) -> &'static str {
        "ConstRateAux"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        _snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<BTreeMap<String, f64>, SimError> {
        Ok(BTreeMap::from([(self.name.clone(), self.rate * dt)]))
    }
}

/// Thermal-time-like: the increment is `env.get(var)·dt`, so it reads forcing through the
/// same #16 seam a flow does.
struct ForcedAux {
    id: String,
    name: String,
    var: String,
}

impl AuxProcess for ForcedAux {
    fn type_name(&self) -> &'static str {
        "ForcedAux"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        _snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<BTreeMap<String, f64>, SimError> {
        Ok(BTreeMap::from([(self.name.clone(), env.get(&self.var)? * dt)]))
    }
}

/// A no-op *balanced* flow (empty legs) that records the aux value it saw. `evaluate` is
/// called once per RK4 stage, so the recorded sequence is the instrument for the
/// within-step-constant claim.
struct AuxRecordingFlow {
    id: String,
    name: String,
    seen: Mutex<Vec<f64>>,
}

impl Flow for AuxRecordingFlow {
    fn type_name(&self) -> &'static str {
        "AuxRecordingFlow"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, snapshot: &State, _env: &dyn Environment, _dt: f64) -> Result<FlowResult, SimError> {
        self.seen
            .lock()
            .expect("recorder")
            .push(snapshot.aux.get(&self.name).copied().unwrap_or(0.0));
        Ok(FlowResult::empty())
    }
}

/// `src -> boundary sink` first-order decay (dt-linear, balanced carbon).
struct Decay {
    id: String,
    src: String,
    sink: String,
    rate: f64,
}

impl Flow for Decay {
    fn type_name(&self) -> &'static str {
        "Decay"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, snapshot: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let moved = self.rate * snapshot.stocks[&self.src].amount * dt;
        FlowResult::new(vec![
            Leg::new(self.src.clone(), -moved)?,
            Leg::new(self.sink.clone(), moved)?,
        ])
    }
}

fn pool(id: &str, amount: f64) -> Stock {
    Stock::new(
        id.to_string(),
        "bio".to_string(),
        Quantity::Carbon,
        Quantity::Carbon.canonical_unit(),
        amount,
        StockKind::Pool,
        0.0,
        false,
        BTreeMap::new(),
    )
    .expect("pool")
}

fn empty_state(aux: BTreeMap<String, f64>) -> State {
    State::new(0, BTreeMap::new(), 0, aux).expect("state")
}

// --------------------------------------------------------------------------- //
// Accumulation under both schemes                                              //
// --------------------------------------------------------------------------- //

/// A constant-rate process integrates by explicit Euler to `rate·n·dt` under **both**
/// schemes — the assertion that aux advances exactly once per step and is never sub-staged
/// through RK4. Under RK4 a per-stage advance would give four times this.
fn constant_rate_accumulates(scheme: Scheme) {
    let (rate, dt, steps) = (2.5, 0.5, 7);
    let mut state = empty_state(BTreeMap::new());
    let env = SourceResolver::empty();
    for _ in 0..steps {
        let registry = Registry::new(
            Vec::new(),
            &BTreeMap::new(),
            vec![Box::new(ConstRateAux {
                id: "tt".to_string(),
                name: "tt".to_string(),
                rate,
            })],
        )
        .expect("registry");
        state = scheme.step(registry, &state, &env, dt);
    }
    assert_eq!(state.n, steps);
    assert_eq!(
        state.aux["tt"],
        rate * dt * steps as f64,
        "{scheme:?}: aux advanced the wrong number of times"
    );
}

#[test]
fn a_constant_rate_process_accumulates_to_rate_times_n_dt_under_euler() {
    constant_rate_accumulates(Scheme::Euler);
}

#[test]
fn a_constant_rate_process_accumulates_to_rate_times_n_dt_under_rk4() {
    constant_rate_accumulates(Scheme::Rk4);
}

/// An aux process resolves forcing through the bound environment — the same seam a flow
/// uses, which is what lets a thermal-time accumulator exist at all.
#[test]
fn a_forced_aux_process_reads_the_environment() {
    let (temp, dt, steps) = (18.0, 1.0, 4);
    let mut forcings: std::collections::HashMap<String, Schedule> = std::collections::HashMap::new();
    forcings.insert("temp".to_string(), constant(temp).unwrap());
    let resolver = SourceResolver::new(forcings, std::collections::HashMap::new()).unwrap();
    let mut state = empty_state(BTreeMap::new());
    for _ in 0..steps {
        let registry = Registry::new(
            Vec::new(),
            &BTreeMap::new(),
            vec![Box::new(ForcedAux {
                id: "tt".to_string(),
                name: "thermal_time".to_string(),
                var: "temp".to_string(),
            })],
        )
        .expect("registry");
        state = EulerIntegrator::new(registry)
            .step(&state, &resolver, dt)
            .expect("step");
    }
    assert_eq!(state.aux["thermal_time"], temp * dt * steps as f64);
}

/// Under RK4 the recording flow is evaluated four times, once per stage, and every read
/// equals the **step-entry** aux value: stage states keep aux and only stock amounts perturb.
/// Aux advances after the stages, at the commit. A second step then reads the *new* entry
/// value four times — which is what distinguishes "constant within a step" from "frozen".
#[test]
fn all_four_rk4_stages_see_the_step_entry_aux() {
    let (rate, dt, v0) = (3.0, 0.5, 10.0);
    let name = "tt";
    let flow = std::sync::Arc::new(AuxRecordingFlow {
        id: "rec".to_string(),
        name: name.to_string(),
        seen: Mutex::new(Vec::new()),
    });
    let make_registry = || {
        Registry::new(
            vec![Box::new(RecorderHandle(flow.clone()))],
            &BTreeMap::new(),
            vec![Box::new(ConstRateAux {
                id: "tt".to_string(),
                name: name.to_string(),
                rate,
            })],
        )
        .expect("registry")
    };

    let state = empty_state(BTreeMap::from([(name.to_string(), v0)]));
    let next = Rk4Integrator::new(make_registry())
        .step(&state, &SourceResolver::empty(), dt)
        .expect("step");

    assert_eq!(
        *flow.seen.lock().unwrap(),
        vec![v0, v0, v0, v0],
        "four stage reads, all at the step-entry value"
    );
    assert_eq!(next.aux[name], v0 + rate * dt, "aux advanced exactly once");

    flow.seen.lock().unwrap().clear();
    Rk4Integrator::new(make_registry())
        .step(&next, &SourceResolver::empty(), dt)
        .expect("second step");
    assert_eq!(
        *flow.seen.lock().unwrap(),
        vec![v0 + rate * dt; 4],
        "the second step's four reads are at the new entry value"
    );
}

/// A shared-ownership wrapper so the recorder can be read after the registry consumed it.
/// `Registry::new` takes `Vec<Box<dyn Flow>>` by value, and the Python original kept its
/// handle simply by holding the same object — this is that, spelled for Rust.
struct RecorderHandle(std::sync::Arc<AuxRecordingFlow>);

impl Flow for RecorderHandle {
    fn type_name(&self) -> &'static str {
        // The wrapper returns the *wrapped* name here because it is a test instrument, not
        // a modelled flow: nothing derives a freeze-manifest `flow_set` from this file.
        self.0.type_name()
    }
    fn id(&self) -> &str {
        self.0.id()
    }
    fn evaluate(&self, snapshot: &State, env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        self.0.evaluate(snapshot, env, dt)
    }
}

// --------------------------------------------------------------------------- //
// Aux is outside the conservation gate — without weakening it                  //
// --------------------------------------------------------------------------- //

/// Identical stocks, different aux: conserves trivially, because the ledger reasons over
/// stocks only.
#[test]
fn an_aux_only_change_conserves() {
    let stocks = BTreeMap::from([("bio.c".to_string(), pool("bio.c", 3.0))]);
    let before = State::new(0, stocks.clone(), 0, BTreeMap::from([("tt".to_string(), 0.0)])).unwrap();
    let after = State::new(1, stocks, 0, BTreeMap::from([("tt".to_string(), 99.0)])).unwrap();
    assert_conserved_default(&before, &after).expect("an aux-only delta conserves");
}

#[test]
fn aux_does_not_appear_in_the_ledger() {
    let stocks = BTreeMap::from([("bio.c".to_string(), pool("bio.c", 3.0))]);
    let before = State::new(0, stocks.clone(), 0, BTreeMap::from([("tt".to_string(), 0.0)])).unwrap();
    let after = State::new(1, stocks, 0, BTreeMap::from([("tt".to_string(), 99.0)])).unwrap();
    let ledger = compute_ledger(&before, &after).expect("ledger");
    assert!(
        ledger.iter().all(|q| q.residual == 0.0),
        "aux carries no conserved-quantity surface: {ledger:?}"
    );
}

/// Aux being outside the gate must not *weaken* it: an unbalanced stock change — carbon from
/// nothing — still raises, whatever aux did in the same step.
#[test]
fn an_unbalanced_stock_change_still_trips_the_gate_despite_aux() {
    let before = State::new(
        0,
        BTreeMap::from([("bio.c".to_string(), pool("bio.c", 3.0))]),
        0,
        BTreeMap::from([("tt".to_string(), 0.0)]),
    )
    .unwrap();
    let after = State::new(
        1,
        BTreeMap::from([("bio.c".to_string(), pool("bio.c", 5.0))]), // +2 carbon, no counterparty
        0,
        BTreeMap::from([("tt".to_string(), 1.0)]),
    )
    .unwrap();

    match assert_conserved_default(&before, &after) {
        Err(SimError::Conservation(msg)) => assert!(msg.contains("CARBON"), "{msg}"),
        other => panic!("the gate did not trip: {other:?}"),
    }
}

/// A model that advances aux **and** moves real mass passes the always-on every-step gate
/// for fifty steps — aux advancing does not perturb the gate, and the gate does not stop aux.
#[test]
fn stepping_with_both_aux_and_mass_flow_conserves_every_step() {
    let stocks = BTreeMap::from([
        ("bio.c".to_string(), pool("bio.c", 4.0)),
        (
            "boundary.sink".to_string(),
            boundary::sink("boundary.sink".to_string(), Quantity::Carbon, 0.0).unwrap(),
        ),
    ]);
    let mut state = State::new(0, stocks.clone(), 0, BTreeMap::new()).unwrap();
    for _ in 0..50 {
        let registry = Registry::new(
            vec![Box::new(Decay {
                id: "decay".to_string(),
                src: "bio.c".to_string(),
                sink: "boundary.sink".to_string(),
                rate: 0.25,
            })],
            &stocks,
            vec![Box::new(ConstRateAux {
                id: "tt".to_string(),
                name: "tt".to_string(),
                rate: 2.0,
            })],
        )
        .expect("registry");
        // `step` returns Err if any step's gate trips, so completing the loop is the claim.
        state = EulerIntegrator::new(registry)
            .step(&state, &SourceResolver::empty(), 1.0)
            .expect("a step broke conservation");
    }
    assert_eq!(state.n, 50);
    assert_eq!(state.aux["tt"], 2.0 * 50.0, "aux accumulated alongside the mass flow");
}

// --------------------------------------------------------------------------- //
// The multi-rate placement guard                                               //
// --------------------------------------------------------------------------- //

/// The pinned placement: `substep` — the multi-rate primitive — keeps aux *and* keeps `n`;
/// only a full step advances either. Advancing aux in the shared sub-step path would advance
/// it `n_sub`× per master step while conserving mass perfectly, so this is the only gate
/// that can see that bug.
#[test]
fn a_substep_leaves_aux_untouched_while_a_full_step_advances_it_once() {
    let (rate, dt, v0) = (4.0, 0.5, 1.0);
    let make = || {
        EulerIntegrator::new(
            Registry::new(
                Vec::new(),
                &BTreeMap::new(),
                vec![Box::new(ConstRateAux {
                    id: "tt".to_string(),
                    name: "tt".to_string(),
                    rate,
                })],
            )
            .expect("registry"),
        )
    };
    let state = empty_state(BTreeMap::from([("tt".to_string(), v0)]));
    let env = SourceResolver::empty();

    let sub: StepReport = make().substep(&state, &env, dt).expect("substep");
    assert_eq!(sub.state.aux["tt"], v0, "substep left aux untouched");
    assert_eq!(sub.state.n, state.n, "substep kept n");

    let full = make().step_report(&state, &env, dt).expect("step");
    assert_eq!(full.state.aux["tt"], v0 + rate * dt, "a full step advanced aux once");
    assert_eq!(full.state.n, state.n + 1);
}

// --------------------------------------------------------------------------- //
// Names, and the State / Registry primitives                                   //
// --------------------------------------------------------------------------- //

#[test]
fn two_processes_write_two_distinct_names() {
    let registry = Registry::new(
        Vec::new(),
        &BTreeMap::new(),
        vec![
            Box::new(ConstRateAux {
                id: "a".to_string(),
                name: "alpha".to_string(),
                rate: 1.0,
            }),
            Box::new(ConstRateAux {
                id: "b".to_string(),
                name: "beta".to_string(),
                rate: 2.0,
            }),
        ],
    )
    .expect("registry");
    let out = EulerIntegrator::new(registry)
        .step(&empty_state(BTreeMap::new()), &SourceResolver::empty(), 0.5)
        .expect("step");
    assert_eq!(
        out.aux,
        BTreeMap::from([("alpha".to_string(), 0.5), ("beta".to_string(), 1.0)])
    );
}

#[test]
fn a_state_defaults_to_an_empty_aux_map() {
    assert!(empty_state(BTreeMap::new()).aux.is_empty());
}

/// Aux values are validated finite at construction, exactly like a stock amount — otherwise
/// a NaN would propagate through every later step silently.
#[test]
fn a_non_finite_aux_value_is_rejected_at_construction() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let built = State::new(
            0,
            BTreeMap::new(),
            0,
            BTreeMap::from([("tt".to_string(), bad)]),
        );
        match built {
            Err(SimError::Validation(msg)) => assert!(msg.contains("not finite"), "{bad:?}: {msg}"),
            Ok(_) => panic!("State accepted a {bad:?} aux value"),
            Err(other) => panic!("wrong error for {bad:?}: {other:?}"),
        }
    }
}

/// The validation is `is_finite` only, matching `Stock.amount`: `-0.0` is finite and its sign
/// survives. Signed zero is not pedantry here — it is the difference the hex-float goldens
/// record, so a validator that normalised it would move frozen bytes.
#[test]
fn a_signed_zero_aux_value_keeps_its_sign() {
    let state = empty_state(BTreeMap::from([("z".to_string(), -0.0)]));
    assert!(state.aux["z"].is_sign_negative());
}

#[test]
fn the_registry_rejects_a_duplicate_aux_id() {
    let built = Registry::new(
        Vec::new(),
        &BTreeMap::new(),
        vec![
            Box::new(ConstRateAux {
                id: "dup".to_string(),
                name: "a".to_string(),
                rate: 1.0,
            }),
            Box::new(ConstRateAux {
                id: "dup".to_string(),
                name: "b".to_string(),
                rate: 2.0,
            }),
        ],
    );
    match built {
        Err(SimError::Validation(msg)) => assert!(msg.contains("duplicate"), "{msg}"),
        other => panic!("a duplicate AuxId was accepted: {:?}", other.is_ok()),
    }
}

/// The registry sorts aux processes by id, and that order **is** the reduction order — the
/// thing `laws.rs`'s order-independence law depends on being true.
#[test]
fn the_registry_holds_aux_processes_in_canonical_id_order() {
    let registry = Registry::new(
        Vec::new(),
        &BTreeMap::new(),
        vec![
            Box::new(ConstRateAux {
                id: "c".to_string(),
                name: "x".to_string(),
                rate: 1.0,
            }),
            Box::new(ConstRateAux {
                id: "a".to_string(),
                name: "x".to_string(),
                rate: 1.0,
            }),
            Box::new(ConstRateAux {
                id: "b".to_string(),
                name: "x".to_string(),
                rate: 1.0,
            }),
        ],
    )
    .expect("registry");
    let ids: Vec<&str> = registry.aux_processes().iter().map(|p| p.id()).collect();
    assert_eq!(ids, ["a", "b", "c"]);
}

#[test]
fn a_registry_built_without_aux_has_none() {
    let registry = Registry::flows_only(Vec::new(), &BTreeMap::new()).expect("registry");
    assert!(registry.aux_processes().is_empty());
}
