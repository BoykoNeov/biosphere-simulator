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

use authoring::interpreter::{interpret, interpret_allowing_unsafe_step};
use authoring::schema::ScenarioSpec;
use authoring::yaml::parse_document;
use authoring::{
    apply_includes, load_scenario, run_scenario, run_scenario_allowing_rationing, BuiltScenario,
};

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

/// [`interpret_str`] with the build-time rate precondition disabled — for fixtures whose
/// whole subject is the unsafe step (the `dt`-hazard block below).
fn interpret_str_allowing_unsafe_step(yaml: &str) -> Result<BuiltScenario, authoring::AuthoringError> {
    let doc = parse_document(yaml)?;
    let spec = ScenarioSpec::from_yaml(&doc)?;
    interpret_allowing_unsafe_step(&spec, Path::new("."), &no_overrides())
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

// --- Step 6c: multi-instance id-namespacing (the SAME bundle included twice) -----

/// Interpret an inline scenario whose bundle paths resolve against the committed
/// scenarios dir (so an inline scenario can `include` the real `battery.domain.yaml`).
fn interpret_in_scenarios(yaml: &str) -> Result<BuiltScenario, authoring::AuthoringError> {
    let doc = parse_document(yaml).expect("parse inline scenario");
    let spec = ScenarioSpec::from_yaml(&doc).expect("bind inline scenario");
    interpret(&spec, &scenarios_dir(), &no_overrides())
}

/// The single-battery oracle: the same bundle included ONCE (bare), same horizon — each
/// namespaced half of `two_batteries` must reproduce this bit-for-bit.
fn single_battery_final_amount() -> f64 {
    let built = interpret_in_scenarios(
        "name: one_battery\nintegrator: euler\ndt: 3600.0\nsteps: 168\n\
         includes:\n  - bundles/battery.domain.yaml\n",
    )
    .expect("interpret single battery");
    let result = run_scenario(built).expect("run single battery");
    result.final_state.stocks["power.battery"].amount
}

#[test]
fn namespaced_same_bundle_twice_runs_and_conserves() {
    // The battery domain included twice under distinct prefixes: namespacing rewrites
    // every id so the two instances do not collide (the bare double-include DOES collide).
    // Disjoint after prefixing ⇒ ENERGY conserves every step, the run completing is the
    // proof; both instances mark it authored (their SelfDischarge kinetics).
    let built = load_scenario(&scenarios_dir().join("two_batteries.yaml"), &no_overrides())
        .expect("load two_batteries");
    assert!(built.has_authored_kinetics);
    let mut ids: Vec<&str> = built.state.stocks.keys().map(|s| s.as_str()).collect();
    ids.sort_unstable();
    assert_eq!(
        ids,
        vec![
            "bat_a.boundary.waste_heat",
            "bat_a.power.battery",
            "bat_b.boundary.waste_heat",
            "bat_b.power.battery",
        ]
    );
    let result = run_scenario(built).expect("run two_batteries");
    assert_eq!(result.total_rationed, 0);
    assert!(result.events.is_empty());
}

#[test]
fn namespaced_instances_each_match_single_battery() {
    // Projection faithfulness: each namespaced half is bit-for-bit a single-battery run
    // over the same horizon — namespacing changed only ids, not dynamics. The two
    // identical instances also equal each other exactly.
    let result = run_scenario(
        load_scenario(&scenarios_dir().join("two_batteries.yaml"), &no_overrides())
            .expect("load two_batteries"),
    )
    .expect("run two_batteries");
    let oracle = single_battery_final_amount();
    let a = result.final_state.stocks["bat_a.power.battery"].amount;
    let b = result.final_state.stocks["bat_b.power.battery"].amount;
    assert_eq!(a, oracle);
    assert_eq!(b, oracle);
}

#[test]
fn apply_includes_namespaces_ids_and_rate_refs() {
    use simcore::expr::Expr;

    let path = scenarios_dir().join("two_batteries.yaml");
    let doc = parse_document(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let spec = ScenarioSpec::from_yaml(&doc).unwrap();
    let merged = apply_includes(&spec, path.parent().unwrap()).expect("apply_includes");

    let stock_ids: Vec<&str> = merged.stocks.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(
        stock_ids,
        vec![
            "bat_a.power.battery",
            "bat_a.boundary.waste_heat",
            "bat_b.power.battery",
            "bat_b.boundary.waste_heat",
        ]
    );
    let flow_ids: Vec<&str> = merged.flows.iter().map(|f| f.id.as_str()).collect();
    assert_eq!(
        flow_ids,
        vec!["bat_a.power.self_discharge", "bat_b.power.self_discharge"]
    );
    // The load-bearing part: the kinetics rate's `stock(...)` ref is rewritten to the
    // prefixed id (a structural AST rewrite, re-emitted). Stoichiometry keys too.
    let flow_a = merged
        .flows
        .iter()
        .find(|f| f.id == "bat_a.power.self_discharge")
        .unwrap();
    let kin = flow_a.kinetics.as_ref().unwrap();
    let stoich_keys: Vec<&str> = kin.stoichiometry.iter().map(|(s, _)| s.as_str()).collect();
    assert!(stoich_keys.contains(&"bat_a.power.battery"));
    assert!(stoich_keys.contains(&"bat_a.boundary.waste_heat"));
    let rate = authoring::parse_rate_expr(&kin.rate).unwrap();
    match rate {
        Expr::BinOp { right, .. } => match *right {
            Expr::StockRef(id) => assert_eq!(id, "bat_a.power.battery"),
            other => panic!("rate rhs not a StockRef: {other:?}"),
        },
        other => panic!("rate not a BinOp: {other:?}"),
    }
}

#[test]
fn prefixed_forcing_bound_bundle_fails_loudly() {
    // The crew-forcing blocker, LOCKED cross-port (the load-bearing Step-6c scope claim):
    // the frozen crew flows read intake forcings by a hardcoded name, so namespacing the
    // forcing keys orphans that lookup. The build is clean (namespacing is well-formed),
    // but the run must FAIL LOUDLY — `Environment::get` returns `SimError::Reference` at
    // step 1 — never a silent, still-conserving wrong run.
    let built = interpret_in_scenarios(
        "name: s\nintegrator: euler\ndt: 3600.0\nsteps: 1\n\
         includes:\n  - bundle: bundles/crew.domain.yaml\n    prefix: crewA\n",
    )
    .expect("a prefixed crew include builds cleanly");
    assert!(
        run_scenario(built).is_err(),
        "a prefixed forcing-bound (crew) bundle must fail loudly, not run silently wrong"
    );
}

#[test]
fn same_bundle_twice_same_prefix_is_duplicate() {
    // Namespacing enables multi-instance, but two instances under the SAME prefix still
    // collide (the prefix must distinguish them) — the collision guard stays honest.
    let err = interpret_in_scenarios(
        "name: s\nintegrator: euler\ndt: 3600.0\nsteps: 1\n\
         includes:\n  - bundle: bundles/battery.domain.yaml\n    prefix: dup\n\
         \x20 - bundle: bundles/battery.domain.yaml\n    prefix: dup\n",
    );
    match err {
        Err(e) => assert!(
            e.message.contains("duplicate stock id"),
            "unexpected error: {}",
            e.message
        ),
        Ok(_) => panic!("expected a duplicate-stock-id error for same-prefix includes"),
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

// ---------------------------------------------------------------------------
// The `dt` hazard gate (post-roadmap) — the Rust mirror of Python's
// tests/test_authoring_dt_hazard.py. Python is the reference; this file exists so the
// two ports cannot silently disagree about whether a rationed authored run is an error.
//
// The hazard itself: every frozen rate constant was sized against the `dt` of its own
// frozen scenario, but an author picks `dt`. At dt = 3600 `eclss.co2_scrubber`'s k*dt is
// 3.6 — it demands 3.6x the whole CO2 pool in one step. The backstop clamps it, so the
// run conserves every quantity and completes normally *with an airless cabin*. Nothing
// was raised; the only signal was the rationed count, which callers discard.
// ---------------------------------------------------------------------------

/// Read the committed ECLSS anchor and retarget its `dt`/`steps`, mirroring the Python
/// test's "mutate a parsed copy of the one committed file" approach — so the *only*
/// difference that matters stays unmissable, and neither port ships a near-duplicate
/// hazard fixture that could drift from the anchor.
///
/// **The study hatch is passed deliberately** (multi-rate Step 6, mirroring Python's
/// `_build_at`). Step 5 added a **build-time** `k·h < 1` precondition, which refuses
/// `dt = 3600` before a single step runs — so the hazard is now caught **twice, at two
/// different stages**. Disabling the build check is what keeps the tests below pointed at
/// the *run-time* `Rationed` verdict they were written to pin, instead of silently
/// becoming duplicate tests of the build check. The new stage has its own test:
/// `the_build_now_refuses_the_unsafe_dt_before_any_step_runs`.
fn eclss_anchor_at(dt: &str, steps: &str) -> BuiltScenario {
    interpret_str_allowing_unsafe_step(&eclss_anchor_yaml_at(dt, steps))
        .expect("interpret the retargeted anchor")
}

/// The retargeted anchor's YAML text — split out from [`eclss_anchor_at`] so the
/// build-refusal test can drive the **author's default path** (no hatch) over the exact
/// same bytes the run-time tests use.
fn eclss_anchor_yaml_at(dt: &str, steps: &str) -> String {
    let text = std::fs::read_to_string(scenarios_dir().join("eclss_cabin.yaml"))
        .expect("read eclss_cabin.yaml");
    let retargeted = text
        .replace("dt: 60.0", &format!("dt: {dt}"))
        .replace("steps: 900", &format!("steps: {steps}"));
    assert!(
        retargeted.contains(&format!("dt: {dt}")),
        "the anchor's `dt: 60.0` line moved — this helper is silently a no-op now"
    );
    retargeted
}

#[test]
fn at_the_frozen_dt_the_backstop_never_fires() {
    // The control: same graph, same frozen params, the sized dt. Without this, the
    // assertion below would not implicate dt.
    let result = run_scenario(eclss_anchor_at("60.0", "900")).expect("run at the sized dt");
    assert_eq!(result.total_rationed, 0);
    assert!(result.events.is_empty());
}

#[test]
fn the_build_now_refuses_the_unsafe_dt_before_any_step_runs() {
    // The stage that did not exist when this file was written (multi-rate Step 5, now
    // mirrored). The author's natural call is `interpret` WITHOUT the study hatch, and it
    // refuses: k_scrub * 3600 = 3.6 >= 1 is decidable from the params + dt alone, so
    // there is no reason to make an author spend a long run to learn it.
    //
    // This is why every other test in this block passes the hatch: the run-time gate they
    // pin is now UNREACHABLE by an author's default path. Both gates are kept because
    // they catch different populations — the build check sees the demand-controlled
    // `eclss.o2_makeup` that rationing structurally cannot see at any dt; the run-time
    // one sees the state-dependent over-draws (crew_metabolism's forced draw) that no
    // build check can decide. **Neither subsumes the other.**
    //
    // Matched rather than `expect_err`: `BuiltScenario` is deliberately not `Debug` (the
    // `RunResult` precedent — deriving it on a reference type to prettify a test
    // assertion is the tail wagging the dog).
    let err = match interpret_str(&eclss_anchor_yaml_at("3600.0", "15")) {
        Ok(_) => panic!("an unsafe k*h must be refused at build on the author's default path"),
        Err(e) => e,
    };

    // Structural, NOT Rationed — this verdict IS decidable from the file alone, which is
    // the whole difference between the two stages. (Contrast the run-time gate below,
    // where the identical file at dt=60 is perfectly valid.)
    assert_eq!(err.kind, authoring::ErrorKind::Structural);
    assert!(
        err.message.contains("co2_scrub_rate"),
        "the offending param must be named: {}",
        err.message
    );
    assert!(
        err.message.contains("3.6"),
        "the k*h value must be named: {}",
        err.message
    );
    // The hatch must be discoverable from the message — but by its RUST name. Python's is
    // the kwarg `allow_unsafe_step=True`; Rust has no default arguments, so the hatch is
    // a separate function (the `run_scenario_allowing_rationing` idiom). The message text
    // is explicitly not a parity target (`crate::errors`), so each port names its own API.
    assert!(
        err.message.contains("interpret_allowing_unsafe_step"),
        "the study hatch must be discoverable: {}",
        err.message
    );
}

#[test]
fn at_an_unsafe_dt_the_run_is_rejected_not_returned() {
    // THE GATE. The author's natural call refuses to hand back the trajectory.
    // (Matched rather than `expect_err`: `RunResult` is deliberately not `Debug`, and
    // deriving it on the reference type to prettify a test assertion is the tail wagging
    // the dog.)
    let err = match run_scenario(eclss_anchor_at("3600.0", "15")) {
        Ok(_) => panic!("a rationed authored run must be an error, not a result"),
        Err(e) => e,
    };

    // Rationed, NOT Structural — the same file at dt = 60 is perfectly valid, so this
    // failure is not decidable from the file's structure. Matching the kind (rather than
    // sniffing the message) is the point of ErrorKind existing.
    assert_eq!(err.kind, authoring::ErrorKind::Rationed);
    assert!(err.message.contains("37"), "count must be named: {}", err.message);
    assert!(err.message.contains("3600"), "dt must be named: {}", err.message);
}

#[test]
fn the_underlying_hazard_is_unchanged_only_its_silence_was_fixed() {
    // THE FINDING, PRESERVED — and the Rust half of the cross-port claim that both ports
    // ration *identically* (37), not merely that both refuse. The escape hatch shows the
    // physics is untouched: we made the failure loud, we did not make the scenario work.
    let result = run_scenario_allowing_rationing(eclss_anchor_at("3600.0", "15"))
        .expect("the escape hatch returns the rationed run");
    assert_eq!(result.total_rationed, 37, "the hazard's mechanism moved — docs are stale");
    assert!(result.events.is_empty());

    // The cabin is airless: clamped at zero by the backstop (hence ~0 from below by
    // roundoff, never properly negative — which is exactly why nothing raised before,
    // and why the every-step conservation gate could never have caught this).
    let cabin_o2 = result.final_state.stocks["eclss.cabin_o2"].amount;
    assert!(cabin_o2.abs() < 1e-9, "expected an emptied cabin, got {cabin_o2}");
}
