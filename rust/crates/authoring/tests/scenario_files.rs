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
use std::path::{Path, PathBuf};

use authoring::interpreter::interpret;
use authoring::schema::ScenarioSpec;
use authoring::yaml::parse_document;
use authoring::{apply_includes, load_scenario, run_scenario, BuiltScenario};

/// The repo's committed scenario directory (shared with the Python anchor tests).
fn scenarios_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../tests/authoring/scenarios")
}

fn no_overrides() -> BTreeMap<String, f64> {
    BTreeMap::new()
}

/// Interpret an inline YAML string (the reject-case + small-fixture helper). These
/// fixtures carry no `includes`, so `base_dir` (bundle resolution) is inert — pass the
/// CWD.
fn interpret_str(yaml: &str) -> Result<BuiltScenario, authoring::AuthoringError> {
    let doc = parse_document(yaml)?;
    let spec = ScenarioSpec::from_yaml(&doc)?;
    interpret(&spec, Path::new("."), &no_overrides())
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

// --- Step 6b: file composition (`includes` / bundles) ------------------------
//
// The cross-port byte-identity / graph-dump / run-match parity for the composition
// anchors lives in `tests/crossport/test_crossport.py` (it needs Python). These are the
// Rust-only "the interpreter composes and runs the anchors" gate, the includes-first
// order pin (a unit test on the merged Vec — the serialized outputs are id-sorted, so
// order is only observable here), and the compose reject cases.

#[test]
fn crew_station_single_bundle_composes_the_crew_graph() {
    // A scenario that is *only* an include of the crew species bundle builds the eight
    // frozen crew ids and runs clean (the byte-identity vs crew_state.json is the
    // cross-port gate; here we check the merge contributes the whole graph).
    let built = load_scenario(&scenarios_dir().join("crew_station.yaml"), &no_overrides())
        .expect("load crew_station");
    assert_eq!(built.state.stocks.len(), 8);
    assert!(!built.has_authored_kinetics);
    let result = run_scenario(built).expect("run crew_station");
    assert_eq!(result.total_rationed, 0);
    assert!(result.events.is_empty());
    assert_eq!(result.final_state.n, 168);
}

#[test]
fn crew_station_override_reaches_a_bundle_declared_parameter() {
    // `crew_count` is declared in the INCLUDED bundle; an override reaches it through
    // the merge (the `test_reused_bundle_scales_with_override` analogue). At 4.0 the
    // food store is 4x (the boundary-eval `param('crew_count') * 1000.0`).
    let mut overrides = BTreeMap::new();
    overrides.insert("crew_count".to_string(), 4.0);
    let built = load_scenario(&scenarios_dir().join("crew_station.yaml"), &overrides)
        .expect("load crew_station @4");
    assert_eq!(built.state.stocks["crew.food_store"].amount, 4000.0);
}

#[test]
fn station_composed_merges_two_bundles() {
    // >1 file merged into one graph (crew + battery). Disjoint domains ⇒ the run
    // completing (every-step conservation gate) IS the merge proof. The composed run
    // inherits the battery's "authored != validated" marker.
    let built = load_scenario(
        &scenarios_dir().join("station_composed.yaml"),
        &no_overrides(),
    )
    .expect("load station_composed");
    assert_eq!(built.state.stocks.len(), 10);
    assert!(
        built.has_authored_kinetics,
        "post-merge flows include the bundle's SelfDischarge kinetics"
    );
    let result = run_scenario(built).expect("run station_composed");
    assert_eq!(result.total_rationed, 0);
    assert!(result.events.is_empty());
}

#[test]
fn apply_includes_orders_includes_before_inline() {
    // Carry-forward (ii): pin includes-FIRST-then-inline on the merged Vec, *before* the
    // interpreter canonicalizes it into id-sorted maps (the serialized outputs are
    // id-sorted, so this is the only place order is observable). A port that merged
    // inline-first would fail here.
    let path = scenarios_dir().join("crew_station_inline_battery.yaml");
    let doc = parse_document(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let spec = ScenarioSpec::from_yaml(&doc).unwrap();
    let merged = apply_includes(&spec, path.parent().unwrap()).expect("apply_includes");

    let stock_ids: Vec<&str> = merged.stocks.iter().map(|s| s.id.as_str()).collect();
    // The crew bundle's eight stocks first (in bundle declaration order), then the two
    // inline battery stocks.
    assert_eq!(
        stock_ids,
        vec![
            "crew.food_store",
            "crew.water_store",
            "crew.o2_store",
            "boundary.exhaled_co2",
            "boundary.fecal_waste",
            "boundary.crew_humidity",
            "boundary.urine",
            "boundary.crew_o2_consumed",
            "power.battery",
            "boundary.waste_heat",
        ]
    );
    let flow_ids: Vec<&str> = merged.flows.iter().map(|f| f.id.as_str()).collect();
    // The three crew bundle flows first, then the inline self-discharge.
    assert_eq!(
        flow_ids,
        vec![
            "crew.oxygen_consumption",
            "crew.food_metabolism",
            "crew.water_balance",
            "power.self_discharge",
        ]
    );
    assert!(merged.includes.is_empty(), "includes emptied after merge");
}

#[test]
fn mixed_include_and_inline_equals_two_bundle_composition() {
    // The mixed anchor (crew bundle + inline battery) builds the same graph as
    // station_composed (crew bundle + battery bundle), so its final state matches.
    let mixed = run_scenario(
        load_scenario(
            &scenarios_dir().join("crew_station_inline_battery.yaml"),
            &no_overrides(),
        )
        .expect("load mixed"),
    )
    .expect("run mixed");
    let composed = run_scenario(
        load_scenario(
            &scenarios_dir().join("station_composed.yaml"),
            &no_overrides(),
        )
        .expect("load composed"),
    )
    .expect("run composed");
    for sid in ["power.battery", "boundary.waste_heat", "crew.food_store"] {
        assert_eq!(
            mixed.final_state.stocks[sid].amount, composed.final_state.stocks[sid].amount,
            "mixed include+inline diverged from two-bundle composition at {sid}"
        );
    }
}

// --- Step 6b: compose reject cases -------------------------------------------
//
// Unlike the inline-YAML rejects below, these need real bundle files on disk (a bundle
// path resolves against the scenario's directory). Written to the integration-test temp
// dir (`CARGO_TARGET_TMPDIR`, no new dev-dep), one subdir per test to avoid collisions.

/// Write `(name, text)` files into a per-test subdir of `CARGO_TARGET_TMPDIR` and return
/// the directory.
fn compose_tmp(sub: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("compose")
        .join(sub);
    std::fs::create_dir_all(&dir).unwrap();
    for (name, text) in files {
        std::fs::write(dir.join(name), text).unwrap();
    }
    dir
}

const MINI_BUNDLE: &str = "\
stocks:\n\
\x20 - id: a.pool\n    domain: a\n    quantity: energy\n    kind: pool\n    amount: 1.0e+3\n\
\x20 - id: boundary.a_sink\n    domain: boundary\n    quantity: energy\n    kind: boundary\n    amount: 0.0\n\
flows:\n  - id: a.leak\n    kinetics:\n      rate: 'stock(\"a.pool\")'\n      \
stoichiometry:\n        a.pool: -1\n        boundary.a_sink: 1\n";

const SCENARIO_HEAD: &str = "name: s\nintegrator: euler\ndt: 1.0\nsteps: 1\n";

#[test]
fn including_same_bundle_twice_is_duplicate() {
    let dir = compose_tmp(
        "same_twice",
        &[
            ("b.yaml", MINI_BUNDLE),
            (
                "s.yaml",
                &format!("{SCENARIO_HEAD}includes:\n  - b.yaml\n  - b.yaml\n"),
            ),
        ],
    );
    let err = load_scenario(&dir.join("s.yaml"), &no_overrides());
    assert!(err.is_err(), "two instances collide on every id");
    assert!(err.err().unwrap().message.contains("duplicate stock id"));
}

#[test]
fn duplicate_stock_across_include_and_inline_raises() {
    let dir = compose_tmp(
        "dup_inline",
        &[
            ("b.yaml", MINI_BUNDLE),
            (
                "s.yaml",
                &format!(
                    "{SCENARIO_HEAD}includes:\n  - b.yaml\n\
                     stocks:\n  - id: a.pool\n    domain: a\n    quantity: energy\n\
                     \x20   kind: pool\n    amount: 5.0\n"
                ),
            ),
        ],
    );
    let err = load_scenario(&dir.join("s.yaml"), &no_overrides());
    assert!(err.err().unwrap().message.contains("duplicate stock id"));
}

#[test]
fn duplicate_parameter_across_bundles_raises() {
    let dir = compose_tmp(
        "dup_param",
        &[
            ("b1.yaml", "parameters:\n  k: 1.0\n"),
            ("b2.yaml", "parameters:\n  k: 2.0\n"),
            (
                "s.yaml",
                &format!("{SCENARIO_HEAD}includes:\n  - b1.yaml\n  - b2.yaml\n"),
            ),
        ],
    );
    let err = load_scenario(&dir.join("s.yaml"), &no_overrides());
    assert!(err.err().unwrap().message.contains("duplicate parameter"));
}

#[test]
fn duplicate_forcing_across_sources_raises() {
    let dir = compose_tmp(
        "dup_forcing",
        &[
            ("b.yaml", "forcings:\n  f:\n    const: 1.0\n"),
            (
                "s.yaml",
                &format!("{SCENARIO_HEAD}includes:\n  - b.yaml\nforcings:\n  f:\n    const: 2.0\n"),
            ),
        ],
    );
    let err = load_scenario(&dir.join("s.yaml"), &no_overrides());
    assert!(err.err().unwrap().message.contains("duplicate forcing"));
}

#[test]
fn bundle_flow_param_pack_is_deferred() {
    // A parameter pack inside an included bundle is a clean AuthoringError (matching
    // Step 1 / the Rust port's packs-deferred).
    let dir = compose_tmp(
        "bundle_pack",
        &[
            (
                "b.yaml",
                "flows:\n  - id: crew.food_metabolism\n    type: crew.food_metabolism\n\
                 \x20   wiring:\n      food_store: x\n      exhaled_co2: y\n      fecal_waste: z\n\
                 \x20   params:\n      pack: some_pack.yaml\n",
            ),
            ("s.yaml", &format!("{SCENARIO_HEAD}includes:\n  - b.yaml\n")),
        ],
    );
    let err = load_scenario(&dir.join("s.yaml"), &no_overrides());
    assert!(err
        .err()
        .unwrap()
        .message
        .contains("parameter packs inside an included bundle"));
}

#[test]
fn run_config_in_bundle_is_schema_rejected() {
    // A bundle carries no run config — `steps:` is an extra key (bundle allowed-key set).
    let dir = compose_tmp(
        "bundle_runconfig",
        &[
            ("b.yaml", &format!("{MINI_BUNDLE}steps: 5\n")),
            ("s.yaml", &format!("{SCENARIO_HEAD}includes:\n  - b.yaml\n")),
        ],
    );
    assert!(load_scenario(&dir.join("s.yaml"), &no_overrides()).is_err());
}

#[test]
fn nested_include_in_bundle_is_schema_rejected() {
    // Includes are flat, one level deep — a bundle with its own `includes` is rejected.
    let dir = compose_tmp(
        "bundle_nested",
        &[
            ("inner.yaml", MINI_BUNDLE),
            ("b.yaml", &format!("{MINI_BUNDLE}includes:\n  - inner.yaml\n")),
            ("s.yaml", &format!("{SCENARIO_HEAD}includes:\n  - b.yaml\n")),
        ],
    );
    assert!(load_scenario(&dir.join("s.yaml"), &no_overrides()).is_err());
}

#[test]
fn missing_include_raises() {
    let dir = compose_tmp(
        "missing",
        &[("s.yaml", &format!("{SCENARIO_HEAD}includes:\n  - nope.yaml\n"))],
    );
    let err = load_scenario(&dir.join("s.yaml"), &no_overrides());
    assert!(err.err().unwrap().message.contains("could not be read"));
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
