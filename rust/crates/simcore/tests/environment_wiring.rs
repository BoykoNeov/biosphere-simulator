//! The environment source resolver — the Rust port of `tests/test_environment.py`'s residue
//! (Stage-3 slice S4).
//!
//! `environment.rs` carries three `#[cfg(test)]` unit tests (overlap rejected, `into_parts`
//! round-trip, both branches resolve) and `laws.rs` carries the law that a forcing value
//! depends only on `n` and `dt`. The plan's census put the residue at **15 of 18 subjects**,
//! and the ones missing were the load-bearing half: the non-finite rejections at both ends,
//! the rebinding behaviour that makes decision #16 true rather than merely stated, and the
//! indistinguishability of the two branches — which is the whole point of the type.
//!
//! ## The one subject that does not port
//!
//! Three Python cases assert that `BoundEnvironment` satisfies the `Environment` *Protocol*
//! and that an object without `get` does not. In Rust a type either implements the trait or
//! the program does not compile, so there is no failing state to guard and no test to write.
//! Recorded rather than dropped silently.

use std::collections::{BTreeMap, HashMap};

use simcore::environment::{constant, Environment, Schedule, SourceResolver};
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::quantities::{Quantity, StockKind};
use simcore::state::{State, Stock};

const LIGHT_STOCK: &str = "boundary.light";
const SINK: &str = "boundary.sink";

/// A boundary reservoir a shared env var can resolve to.
fn light_stock(amount: f64) -> Stock {
    Stock::new(
        LIGHT_STOCK.to_string(),
        "boundary".to_string(),
        Quantity::Energy,
        Quantity::Energy.canonical_unit(),
        amount,
        StockKind::Boundary,
        0.0,
        true,
        BTreeMap::new(),
    )
    .expect("light stock")
}

fn state_at(amount: f64, n: u64) -> State {
    let stocks = BTreeMap::from([(LIGHT_STOCK.to_string(), light_stock(amount))]);
    State::new(n, stocks, 0, BTreeMap::new()).expect("state")
}

fn forcing_only(var: &str, schedule: Schedule) -> SourceResolver {
    let mut forcings: HashMap<String, Schedule> = HashMap::new();
    forcings.insert(var.to_string(), schedule);
    SourceResolver::new(forcings, HashMap::new()).expect("resolver")
}

fn shared_only(var: &str, stock: &str) -> SourceResolver {
    SourceResolver::new(
        HashMap::new(),
        HashMap::from([(var.to_string(), stock.to_string())]),
    )
    .expect("resolver")
}

/// A pure flow whose legs depend only on `env.get(var)` — so its result is a direct witness
/// to what the resolver returned, which is what makes the indistinguishability case an
/// observation rather than an inspection.
struct ReadsEnvFlow {
    id: String,
    var: String,
    sink: String,
}

impl Flow for ReadsEnvFlow {
    fn type_name(&self) -> &'static str {
        "ReadsEnvFlow"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, _snapshot: &State, env: &dyn Environment, _dt: f64) -> Result<FlowResult, SimError> {
        let value = env.get(&self.var)?;
        FlowResult::new(vec![Leg::new(self.sink.clone(), value)?])
    }
}

// --------------------------------------------------------------------------- //
// The constant schedule                                                        //
// --------------------------------------------------------------------------- //

#[test]
fn a_constant_schedule_returns_its_value_for_any_n_and_dt() {
    let schedule = constant(7.5).unwrap();
    assert_eq!(schedule(0, 1.0), 7.5);
    assert_eq!(schedule(123, 0.25), 7.5);
}

/// A bad constant fails at **wiring time**, not at the first `get` — so a mis-specified
/// scenario dies at construction with the var in hand rather than mid-run. All three
/// non-finite shapes, because `is_finite` is one call and a hand-rolled check might not be.
#[test]
fn a_non_finite_constant_is_rejected_at_construction() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        match constant(bad) {
            Err(SimError::Validation(msg)) => {
                assert!(msg.contains("not finite"), "{bad:?}: {msg}")
            }
            other => panic!("constant({bad:?}) was accepted: {:?}", other.is_ok()),
        }
    }
}

// --------------------------------------------------------------------------- //
// The forcing branch — evaluated at t = n·dt, integer n (#14)                   //
// --------------------------------------------------------------------------- //

/// A schedule returning `n·dt` witnesses both halves at once: that it sees the integer step
/// count, and that time is *evaluated* at `n·dt` rather than accumulated across steps.
#[test]
fn the_forcing_branch_receives_the_integer_n_and_the_dt() {
    let resolver = forcing_only("t", Box::new(|n, dt| n as f64 * dt));
    assert_eq!(resolver.bind(&state_at(0.0, 0), 0.5).get("t").unwrap(), 0.0);
    assert_eq!(resolver.bind(&state_at(0.0, 3), 0.5).get("t").unwrap(), 1.5);
    assert!((resolver.bind(&state_at(0.0, 10), 0.1).get("t").unwrap() - 1.0).abs() < 1e-12);
}

#[test]
fn a_constant_forcing_is_fixed_across_n() {
    let resolver = forcing_only("solar", constant(42.0).unwrap());
    assert_eq!(resolver.bind(&state_at(0.0, 0), 1.0).get("solar").unwrap(), 42.0);
    assert_eq!(resolver.bind(&state_at(0.0, 999), 1.0).get("solar").unwrap(), 42.0);
}

/// Stock amounts are validated finite at construction, so a forcing schedule is the **only**
/// way NaN or Inf can enter a derivative evaluation. That is why the guard sits on this
/// branch and nowhere else, and why removing it would be invisible to every other test.
#[test]
fn a_forcing_schedule_returning_a_non_finite_value_is_rejected_at_resolve_time() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let resolver = forcing_only("bad", Box::new(move |_n, _dt| bad));
        match resolver.bind(&state_at(0.0, 0), 1.0).get("bad") {
            Err(SimError::Validation(msg)) => {
                assert!(msg.contains("non-finite"), "{bad:?}: {msg}")
            }
            other => panic!("a {bad:?} forcing resolved: {other:?}"),
        }
    }
}

// --------------------------------------------------------------------------- //
// The shared-stock branch — reads the bound snapshot (#16)                      //
// --------------------------------------------------------------------------- //

#[test]
fn the_shared_branch_reads_the_bound_snapshots_amount() {
    let resolver = shared_only("light", LIGHT_STOCK);
    assert_eq!(resolver.bind(&state_at(5.0, 0), 1.0).get("light").unwrap(), 5.0);
}

/// The crux of #16: the bound view reads the *current* snapshot and never a cached value.
/// Rebinding to a snapshot holding a different amount must change the answer — an
/// implementation that cached at construction would pass every single-binding case above.
#[test]
fn rebinding_the_shared_branch_reflects_the_new_snapshot() {
    let resolver = shared_only("light", LIGHT_STOCK);
    assert_eq!(resolver.bind(&state_at(5.0, 0), 1.0).get("light").unwrap(), 5.0);
    assert_eq!(resolver.bind(&state_at(8.0, 0), 1.0).get("light").unwrap(), 8.0);
}

/// Referential integrity is resolve-time by design, consistent with how a flow leg naming a
/// missing stock behaves.
#[test]
fn a_shared_var_pointing_at_a_missing_stock_is_a_reference_error() {
    let resolver = shared_only("ghost", "boundary.does_not_exist");
    assert!(matches!(
        resolver.bind(&state_at(1.0, 0), 1.0).get("ghost"),
        Err(SimError::Reference(_))
    ));
}

// --------------------------------------------------------------------------- //
// The headline: a reader cannot tell which branch answered                     //
// --------------------------------------------------------------------------- //

/// A flow reading `env.get("x")` produces an **equal** result whether `x` is a constant
/// forcing holding V or a stock holding V — across several `n`, so the case also pins that
/// constant-forcing and static-stock stay equivalent as steps advance. A single-binding
/// comparison would pass even if the forcing branch ignored `n` entirely.
#[test]
fn a_forcing_and_a_shared_stock_are_indistinguishable_across_n() {
    let value = 3.25;
    let flow = ReadsEnvFlow {
        id: "reads_x".to_string(),
        var: "x".to_string(),
        sink: SINK.to_string(),
    };
    let forced_resolver = forcing_only("x", constant(value).unwrap());
    let shared_resolver = shared_only("x", LIGHT_STOCK);

    for n in [0u64, 1, 7, 1000] {
        let empty = state_at(0.0, n);
        let forced = flow
            .evaluate(&empty, &forced_resolver.bind(&empty, 1.0), 1.0)
            .unwrap();
        let snap = state_at(value, n);
        let coupled = flow
            .evaluate(&snap, &shared_resolver.bind(&snap, 1.0), 1.0)
            .unwrap();

        let legs = |r: &FlowResult| -> Vec<(String, u64)> {
            r.legs.iter().map(|l| (l.stock.clone(), l.amount.to_bits())).collect()
        };
        assert_eq!(legs(&forced), legs(&coupled), "n={n}: the two branches diverge");
    }
}

/// One resolver wiring some vars as forcing and others as a shared stock, dispatched
/// correctly from a single bind — the shape every real scenario uses.
#[test]
fn one_resolver_dispatches_both_branches_from_one_bind() {
    let mut forcings: HashMap<String, Schedule> = HashMap::new();
    forcings.insert("solar".to_string(), constant(42.0).unwrap());
    let resolver = SourceResolver::new(
        forcings,
        HashMap::from([("light".to_string(), LIGHT_STOCK.to_string())]),
    )
    .unwrap();

    let state = state_at(5.0, 7);
    let bound = resolver.bind(&state, 1.0);
    assert_eq!(bound.get("solar").unwrap(), 42.0);
    assert_eq!(bound.get("light").unwrap(), 5.0);
}

// --------------------------------------------------------------------------- //
// Wiring: the empty and unknown cases                                          //
// --------------------------------------------------------------------------- //

/// An empty resolver is valid — the common standalone case where no flow reads env — and
/// resolves nothing. Valid-but-inert is a state worth pinning: it is what a scenario that
/// forgot its wiring looks like, and it must fail at the first `get` rather than return 0.
#[test]
fn an_empty_resolver_is_valid_and_resolves_nothing() {
    let resolver = SourceResolver::empty();
    match resolver.bind(&state_at(0.0, 0), 1.0).get("anything") {
        Err(SimError::Reference(msg)) => assert!(msg.contains("unknown env var"), "{msg}"),
        other => panic!("an empty resolver answered: {other:?}"),
    }
}

#[test]
fn an_unwired_var_is_a_reference_error_even_when_others_are_wired() {
    let resolver = forcing_only("solar", constant(1.0).unwrap());
    match resolver.bind(&state_at(0.0, 0), 1.0).get("missing") {
        Err(SimError::Reference(msg)) => assert!(msg.contains("unknown env var"), "{msg}"),
        other => panic!("an unwired var resolved: {other:?}"),
    }
}
