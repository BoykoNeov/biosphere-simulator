//! Integration tests for the Phase-9 Step-4b Rust scenario-file interpreter.
//!
//! Reads the *committed* anchor scenario files (shared with the Python side) through
//! the whole `authoring` pipeline (closed-subset YAML reader → schema → interpreter →
//! run) and checks the structural + run invariants. The **cross-port** byte-identity /
//! graph-dump parity lives in `tests/crossport/test_crossport.py` (it needs Python);
//! this file is the Rust-only "the interpreter builds and runs the anchors" gate, plus
//! the reject cases (a malformed / illegal scenario is an `AuthoringError`, never a
//! silent mis-build — the reject→both-error side of Tier-0 parse-parity).

use std::collections::BTreeMap;
use std::path::PathBuf;

use authoring::interpreter::interpret;
use authoring::schema::ScenarioSpec;
use authoring::yaml::parse_document;
use authoring::{load_scenario, run_scenario, BuiltScenario};

/// The repo's committed scenario directory (shared with the Python anchor tests).
fn scenarios_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../tests/authoring/scenarios")
}

fn no_overrides() -> BTreeMap<String, f64> {
    BTreeMap::new()
}

/// Interpret an inline YAML string (the reject-case + small-fixture helper).
fn interpret_str(yaml: &str) -> Result<BuiltScenario, authoring::AuthoringError> {
    let doc = parse_document(yaml)?;
    let spec = ScenarioSpec::from_yaml(&doc)?;
    interpret(&spec, &no_overrides())
}

#[test]
fn crew_mission_interprets_and_runs_clean() {
    let built = load_scenario(&scenarios_dir().join("crew_mission.yaml"), &no_overrides())
        .expect("load crew_mission");
    assert_eq!(built.integrator, "euler");
    assert_eq!(built.steps, 168);
    assert!(!built.has_authored_kinetics);
    // Eight stocks (3 stores + 5 boundary sinks), three crew flows (id-sorted).
    assert_eq!(built.state.stocks.len(), 8);
    let flow_ids: Vec<&str> = built.registry.flows().iter().map(|f| f.id()).collect();
    assert_eq!(
        flow_ids,
        vec![
            "crew.food_metabolism",
            "crew.oxygen_consumption",
            "crew.water_balance",
        ]
    );
    let result = run_scenario(built).expect("run crew_mission");
    assert_eq!(result.total_rationed, 0);
    assert!(result.events.is_empty());
    assert_eq!(result.final_state.n, 168);
}

#[test]
fn self_discharge_dsl_builds_an_authored_flow() {
    let built = load_scenario(
        &scenarios_dir().join("self_discharge_dsl.yaml"),
        &no_overrides(),
    )
    .expect("load self_discharge_dsl");
    assert!(built.has_authored_kinetics, "kinetics flow marks the scenario");
    let result = run_scenario(built).expect("run self_discharge_dsl");
    assert_eq!(result.total_rationed, 0);
    assert!(result.events.is_empty());
}

#[test]
fn template_default_and_override_scale_the_store() {
    // Default crew_count = 1.0 → food_store = 1000.0.
    let base = load_scenario(
        &scenarios_dir().join("crew_habitat_template.yaml"),
        &no_overrides(),
    )
    .expect("load template");
    assert_eq!(base.state.stocks["crew.food_store"].amount, 1000.0);

    // crew_count = 4.0 → food_store = 4000.0 (the boundary-eval `param * const`).
    let mut overrides = BTreeMap::new();
    overrides.insert("crew_count".to_string(), 4.0);
    let scaled = load_scenario(
        &scenarios_dir().join("crew_habitat_template.yaml"),
        &overrides,
    )
    .expect("load template @4");
    assert_eq!(scaled.state.stocks["crew.food_store"].amount, 4000.0);
    // The forcing scaled too (a constant schedule at n=0).
    let intake = scaled.resolver.forcings()["crew_food_intake"](0, 0.0);
    assert_eq!(intake, 4.0 * 5.0e-4);
}

#[test]
fn override_of_undeclared_parameter_is_rejected() {
    let mut overrides = BTreeMap::new();
    overrides.insert("no_such_param".to_string(), 2.0);
    let err = load_scenario(
        &scenarios_dir().join("crew_habitat_template.yaml"),
        &overrides,
    );
    assert!(err.is_err());
}

/// An inline donor-controlled `kinetics` scenario (the SelfDischarge shape) at the
/// given integrator — exercises the `run.rs` integrator dispatch (the anchor files all
/// pin `euler`, so this is the sole coverage of the `rk4` arm).
fn self_discharge_inline(integrator: &str) -> String {
    format!(
        "name: t\nintegrator: {integrator}\ndt: 3600.0\nsteps: 24\n\
         stocks:\n\
         \x20 - id: p\n    domain: power\n    quantity: energy\n    kind: pool\n    amount: 10000000.0\n\
         \x20 - id: h\n    domain: boundary\n    quantity: energy\n    kind: boundary\n    amount: 0.0\n\
         flows:\n  - id: sd\n    kinetics:\n      rate: 'param(\"self_discharge_rate\") * stock(\"p\")'\n      \
         stoichiometry:\n        p: -1\n        h: 1\n    params: self_discharge\n"
    )
}

#[test]
fn rk4_integrator_runs_and_differs_from_euler() {
    // The rk4 dispatch arm runs clean...
    let rk4 = run_scenario(interpret_str(&self_discharge_inline("rk4")).expect("build rk4"))
        .expect("run rk4");
    assert_eq!(rk4.total_rationed, 0);
    assert!(rk4.events.is_empty());
    // ...and a donor-controlled flow makes RK4 ≢ Euler (the "it exercised the stages"
    // signal — a broken rk4 arm that silently ran Euler would fail this).
    let euler = run_scenario(interpret_str(&self_discharge_inline("euler")).expect("build euler"))
        .expect("run euler");
    assert_ne!(
        rk4.final_state.stocks["p"].amount, euler.final_state.stocks["p"].amount,
        "donor-controlled flow: RK4 must differ from Euler"
    );
}

#[test]
fn unknown_integrator_is_rejected() {
    let built = interpret_str(&self_discharge_inline("midpoint")).expect("build");
    assert!(run_scenario(built).is_err());
}

// --- reject cases (reject → both-error; message not pinned) ------------------
//
// NOTE (advisor): reject-parity is a Rust-SAFETY property here, not a cross-port parity
// surface (as it was for 4a rate strings). The Rust port deliberately rejects a
// *superset* of what Python accepts — a `{pack: …}` reference and a non-zero frozen-flow
// priority both SUCCEED in Python but ERROR in Rust (the deferred surfaces below). So
// these prove "the Rust interpreter never silently mis-builds", not "both ports agree on
// rejection". The accept→same-graph parity is carried by the anchor byte-identity /
// graph-dump / run-match gates in `tests/crossport/test_crossport.py`.

const STOCKS_HEADER: &str = "name: t\nintegrator: euler\ndt: 1.0\nsteps: 1\n";

#[test]
fn unknown_flow_type_is_rejected() {
    let yaml = format!(
        "{STOCKS_HEADER}\
         stocks:\n  - id: a\n    domain: d\n    quantity: carbon\n    kind: pool\n    amount: 1.0\n\
         flows:\n  - id: f\n    type: no.such_flow\n"
    );
    assert!(interpret_str(&yaml).is_err());
}

#[test]
fn type_and_kinetics_both_given_is_rejected() {
    let yaml = format!(
        "{STOCKS_HEADER}\
         stocks:\n  - id: a\n    domain: d\n    quantity: carbon\n    kind: pool\n    amount: 1.0\n\
         flows:\n  - id: f\n    type: crew.oxygen_consumption\n    kinetics:\n      rate: n\n      \
         stoichiometry:\n        a: 1\n"
    );
    assert!(interpret_str(&yaml).is_err());
}

#[test]
fn unbalanced_authored_stoichiometry_is_rejected() {
    // Two same-quantity legs both -1 → Σ = -2 ≠ 0 (decision C, build-time balance).
    let yaml = format!(
        "{STOCKS_HEADER}\
         stocks:\n\
         \x20 - id: a\n    domain: d\n    quantity: carbon\n    kind: pool\n    amount: 1.0\n\
         \x20 - id: b\n    domain: d\n    quantity: carbon\n    kind: boundary\n    amount: 0.0\n\
         flows:\n  - id: f\n    kinetics:\n      rate: n\n      stoichiometry:\n        a: -1\n        b: -1\n"
    );
    let err = interpret_str(&yaml);
    assert!(err.is_err(), "unbalanced stoichiometry must be rejected");
}

#[test]
fn kinetics_reference_to_unknown_stock_is_rejected() {
    let yaml = format!(
        "{STOCKS_HEADER}\
         stocks:\n\
         \x20 - id: a\n    domain: d\n    quantity: carbon\n    kind: pool\n    amount: 1.0\n\
         \x20 - id: b\n    domain: d\n    quantity: carbon\n    kind: boundary\n    amount: 0.0\n\
         flows:\n  - id: f\n    kinetics:\n      rate: 'stock(\"ghost\")'\n      \
         stoichiometry:\n        a: -1\n        b: 1\n"
    );
    assert!(interpret_str(&yaml).is_err());
}

#[test]
fn non_zero_priority_on_a_frozen_type_is_rejected() {
    // The Rust-port deferral: the frozen crew constructors carry no priority field.
    let yaml = format!(
        "{STOCKS_HEADER}\
         stocks:\n\
         \x20 - id: s\n    domain: crew\n    quantity: oxygen\n    kind: pool\n    amount: 1.0\n\
         \x20 - id: k\n    domain: boundary\n    quantity: oxygen\n    kind: boundary\n    amount: 0.0\n\
         flows:\n  - id: f\n    type: crew.oxygen_consumption\n    priority: 5\n    \
         wiring:\n      o2_store: s\n      o2_consumed: k\n"
    );
    assert!(interpret_str(&yaml).is_err());
}

#[test]
fn param_pack_on_a_frozen_type_is_deferred() {
    // Packs are deferred in the Rust port → a clean error, not a silent build.
    let yaml = format!(
        "{STOCKS_HEADER}\
         stocks:\n\
         \x20 - id: fs\n    domain: crew\n    quantity: carbon\n    kind: pool\n    amount: 1.0\n\
         \x20 - id: co2\n    domain: boundary\n    quantity: carbon\n    kind: boundary\n    amount: 0.0\n\
         \x20 - id: fw\n    domain: boundary\n    quantity: carbon\n    kind: boundary\n    amount: 0.0\n\
         flows:\n  - id: f\n    type: crew.food_metabolism\n    \
         wiring:\n      food_store: fs\n      exhaled_co2: co2\n      fecal_waste: fw\n    \
         params:\n      pack: some_pack.yaml\n"
    );
    assert!(interpret_str(&yaml).is_err());
}
