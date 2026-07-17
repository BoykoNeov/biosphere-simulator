//! The multi-rate authoring surface — the Rust mirror (post-roadmap Step 6).
//!
//! Python is the reference (`tests/test_authoring_multirate_partition.py`,
//! `tests/test_authoring_rate_precondition.py`); this file exists so the two ports cannot
//! silently disagree about **which scenarios the platform refuses to build** and **which
//! step each rate class is judged at**.
//!
//! The Rust mirror carries three facts Python's does not, all consequences of the port's
//! own scope:
//!
//! * **Parameter packs are deferred here**, and that is *load-bearing* for the
//!   precondition — see `pack_deferral_is_what_makes_the_frozen_rate_read_sound`.
//! * **`k` is read from the frozen constant**, not off the built flow (a `Box<dyn Flow>`
//!   exposes no params accessor).
//! * **The routing branch has no injection seam.** Python monkeypatches `multirate_step`
//!   to raise; Rust cannot. It is pinned *behaviorally* instead, via aux — see
//!   `a_single_rate_scenario_never_touches_the_driver`.

use std::collections::BTreeMap;
use std::path::Path;

use authoring::interpreter::{effective_step, interpret, interpret_allowing_unsafe_step};
use authoring::schema::ScenarioSpec;
use authoring::yaml::parse_document;
use authoring::{run_scenario, BuiltScenario, RATE_CLASSES};
use simcore::auxiliary::AuxProcess;
use simcore::environment::{Environment, SourceResolver};
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::quantities::{Quantity, StockKind};
use simcore::registry::Registry;
use simcore::state::{State, Stock};

fn no_overrides() -> BTreeMap<String, f64> {
    BTreeMap::new()
}

fn build(yaml: &str) -> Result<BuiltScenario, authoring::AuthoringError> {
    let doc = parse_document(yaml)?;
    let spec = ScenarioSpec::from_yaml(&doc)?;
    interpret(&spec, Path::new("."), &no_overrides())
}

fn build_allowing_unsafe(yaml: &str) -> Result<BuiltScenario, authoring::AuthoringError> {
    let doc = parse_document(yaml)?;
    let spec = ScenarioSpec::from_yaml(&doc)?;
    interpret_allowing_unsafe_step(&spec, Path::new("."), &no_overrides())
}

/// A minimal ECLSS cabin: the scrubber (donor-controlled, `k_scrub = 1e-3`) plus the
/// condenser (`k_cond = 5e-4`, half the scrubber's — the only ECLSS flow that may legally
/// be slow at `dt=3600`). `dt`/`n_sub`/the two rate classes are the knobs under test.
fn cabin_yaml(dt: &str, n_sub: &str, scrub_class: &str, cond_class: &str) -> String {
    format!(
        "name: multirate_probe\n\
         integrator: euler\n\
         dt: {dt}\n\
         steps: 4\n\
         n_sub: {n_sub}\n\
         stocks:\n\
         \x20 - id: eclss.cabin_co2\n    domain: eclss\n    quantity: carbon\n    kind: pool\n    amount: 10.0\n\
         \x20 - id: eclss.co2_removed\n    domain: boundary\n    quantity: carbon\n    kind: boundary\n    amount: 0.0\n\
         \x20 - id: eclss.cabin_h2o\n    domain: eclss\n    quantity: water\n    kind: pool\n    amount: 10.0\n\
         \x20 - id: eclss.humidity_condensate\n    domain: boundary\n    quantity: water\n    kind: boundary\n    amount: 0.0\n\
         flows:\n\
         \x20 - id: eclss.co2_scrubber\n    type: eclss.co2_scrubber\n    params: eclss\n    rate_class: {scrub_class}\n\
         \x20\x20\x20 wiring:\n      cabin_co2: eclss.cabin_co2\n      co2_removed: eclss.co2_removed\n\
         \x20 - id: eclss.condenser\n    type: eclss.condenser\n    params: eclss\n    rate_class: {cond_class}\n\
         \x20\x20\x20 wiring:\n      cabin_h2o: eclss.cabin_h2o\n      humidity_condensate: eclss.humidity_condensate\n"
    )
}

// ---------------------------------------------------------------------------
// The legality matrix — which partitions build at all.
// ---------------------------------------------------------------------------

#[test]
fn a_scenario_with_no_multirate_keys_is_single_rate() {
    // The default path: `rate_class` defaults to fast + `n_sub` defaults to 1 ⇒ an EMPTY
    // slow set ⇒ the bit-exact identity path. This is what makes every pre-multi-rate
    // scenario (and all 25 goldens) hold *by construction* rather than by re-baselining.
    let built = build(&cabin_yaml("60.0", "1", "fast", "fast")).expect("builds");
    assert!(!built.is_multirate(), "no keys ⇒ not multi-rate");
    assert_eq!(built.n_sub, 1);
    assert!(
        built.slow_registry.flows().is_empty(),
        "the slow set must be empty"
    );
    assert_eq!(
        built.fast_registry.flows().len(),
        built.registry.flows().len(),
        "with an empty slow set the fast registry holds every flow"
    );
}

#[test]
fn n_sub_gt_1_with_an_empty_slow_set_is_legal() {
    // This row LOOKS like a misconfiguration ("multi-rate with nothing slow") — and that
    // intuition would have deleted the phase's headline. It is *uniform sub-stepping*:
    // the export cadence decoupled from the solver step, which is exactly the
    // configuration the measured payoff runs on (master dt=3600, n_sub=60 lands on the
    // same value as single-rate dt=60 while exporting 60x less often).
    let built = build(&cabin_yaml("3600.0", "60", "fast", "fast")).expect("builds");
    assert!(built.is_multirate());
    assert!(built.slow_registry.flows().is_empty());
}

#[test]
fn n_sub_1_with_a_non_empty_slow_set_is_refused() {
    // It buys NO rate separation and no perf win — yet it is not inert: it moves the
    // answer, via the slow set's own two-half-step discretization ((1-k*dt/2)^2 !=
    // (1-k*dt)) AND, the dominant effect, the fast flows reading slow-updated stocks
    // mid-step. A misconfiguration, refused rather than honoured.
    let err = match build(&cabin_yaml("60.0", "1", "fast", "slow")) {
        Ok(_) => panic!("n_sub=1 with a slow set must be refused"),
        Err(e) => e,
    };
    assert!(
        err.message.contains("n_sub=1"),
        "the offending combination must be named: {}",
        err.message
    );
    // The remedy must say BOTH ways out — and the second is the one an author misses:
    // "the same graph, single-rate" is not `n_sub=1`, it is dropping the rate_class keys.
    assert!(
        err.message.contains("rate_class"),
        "the remedy must name dropping the rate_class key(s): {}",
        err.message
    );
}

#[test]
fn an_unknown_rate_class_is_refused() {
    let err = match build(&cabin_yaml("60.0", "2", "fast", "medium")) {
        Ok(_) => panic!("an unknown rate class must be refused"),
        Err(e) => e,
    };
    assert!(err.message.contains("medium"), "{}", err.message);
}

#[test]
fn the_rate_class_vocabulary_is_closed_at_two() {
    // Not a style assertion: `multirate_step` takes exactly TWO `Substepper`s (decision
    // N3 — the driver takes the two pre-built integrators and does not infer the
    // partition), so a third class cannot appear without a `simcore` change. If this goes
    // red, the core's own signature moved.
    assert_eq!(RATE_CLASSES, &["fast", "slow"]);
}

// ---------------------------------------------------------------------------
// The effective step — the Step-5 finding, mirrored.
// ---------------------------------------------------------------------------

#[test]
fn the_effective_step_is_per_rate_class_not_dt_over_n_sub() {
    // THE FINDING the multi-rate plan's own formula got wrong. Three cases, and the slow
    // one is the trap: it ignores `n_sub` entirely.
    assert_eq!(
        effective_step(3600.0, 60, false, false),
        3600.0,
        "single-rate: the whole registry steps at the master dt"
    );
    assert_eq!(
        effective_step(3600.0, 60, true, false),
        60.0,
        "multi-rate fast: dt/n_sub — the point of the cadence knob"
    );
    assert_eq!(
        effective_step(3600.0, 60, true, true),
        1800.0,
        "multi-rate SLOW: dt/2 (Strang's half-step), NOT dt/n_sub = 60"
    );
}

#[test]
fn the_slow_sets_step_ignores_n_sub_entirely() {
    // Strang splits the slow set into two dt/2 halves *regardless* of how finely the fast
    // set sub-steps. The plan's `dt/n_sub` formula would report 4 different numbers here.
    for n_sub in [2u32, 60, 600, 1000] {
        assert_eq!(
            effective_step(3600.0, n_sub, true, true),
            1800.0,
            "the slow step must be dt/2 at every n_sub (n_sub={n_sub})"
        );
    }
}

#[test]
fn a_slow_flow_is_judged_at_dt_over_2_not_the_plans_formula() {
    // The measured false-PASS, as a build verdict. The scrubber classed SLOW at master
    // dt=3600 / n_sub=60: the plan's formula reports k*h = 1e-3 * 60 = 0.06 and PASSES;
    // the truth is 1e-3 * 1800 = 1.8, and the run rations 24 times and empties cabin_co2
    // to exactly 0.0. A false PASS in the unsafe direction is worse than no check,
    // because it reads as a guarantee.
    let err = match build(&cabin_yaml("3600.0", "60", "slow", "fast")) {
        Ok(_) => panic!("a slow flow at k*dt/2 = 1.8 must be refused, not passed at 0.06"),
        Err(e) => e,
    };
    assert!(
        err.message.contains("1.8"),
        "the TRUE k*h (1.8) must be named, not the formula's 0.06: {}",
        err.message
    );
    // The remedy must NOT say "raise n_sub" for a slow flow — an author who followed that
    // would raise it, watch nothing change, and conclude the check is broken.
    assert!(
        err.message.contains("REGARDLESS of n_sub"),
        "the slow remedy must warn that n_sub will not help: {}",
        err.message
    );
}

#[test]
fn the_same_flow_at_the_same_dt_is_fine_as_fast() {
    // Only `rate_class` moved: 1.8 -> 0.06. The rate class DECIDES the verdict, which is
    // the whole reason the effective step must be per-class.
    build(&cabin_yaml("3600.0", "60", "fast", "fast")).expect("the scrubber is safe as fast");
}

#[test]
fn the_condenser_is_the_only_eclss_flow_that_may_be_slow_at_dt_3600() {
    // k_cond = 5e-4 is exactly half the scrubber's, so 5e-4 * 1800 = 0.9 < 1 — it clears
    // the bound where the scrubber (1.8) does not. Same graph, same dt, same n_sub; only
    // `k` differs, and it decides membership of the slow set.
    let built =
        build(&cabin_yaml("3600.0", "60", "fast", "slow")).expect("the condenser may be slow");
    assert!(built.is_multirate());
    assert_eq!(
        built.slow_registry.flows().len(),
        1,
        "exactly the condenser is slow"
    );
    assert_eq!(built.slow_registry.flows()[0].id(), "eclss.condenser");
}

#[test]
fn the_bound_is_strictly_less_than_one() {
    // `k*h >= 1` is refused: at k*h == 1.0 exactly the update map is DEADBEAT
    // (x -> x - 1.0*x = 0 in one step). k_scrub = 1e-3 ⇒ dt = 1000 is exactly 1.0.
    match build(&cabin_yaml("1000.0", "1", "fast", "fast")) {
        Ok(_) => panic!("k*h == 1.0 exactly must be refused (the bound is < 1, not <= 1)"),
        Err(e) => assert!(e.message.contains("co2_scrub_rate"), "{}", e.message),
    }
    build(&cabin_yaml("999.0", "1", "fast", "fast")).expect("k*h just under 1.0 builds");
}

// ---------------------------------------------------------------------------
// The couplings that must not drift silently.
// ---------------------------------------------------------------------------

#[test]
fn the_slow_step_tracks_the_split_actually_used() {
    // `interpreter::SLOW_STEP_DIVISOR`'s dt/2 is true ONLY because the harness pins
    // Strang. Under **Lie** the slow set steps at the full `dt`, which would make the
    // precondition too permissive by exactly 2x — silently, in the unsafe direction.
    // `interpreter` cannot import `run::SPLIT` (`run` imports `interpreter`), so the
    // coupling is asserted rather than commented. If author-visible `split` is ever
    // exposed, this goes red HERE — which is the correct place to be stopped.
    //
    // Read via the public driver contract: a Strang master step runs the slow set at
    // dt/2, twice. That is also the fact behind Step 4's measured 30x (not 60x) saving.
    assert_eq!(
        authoring::SPLIT,
        simcore::multirate::Split::Strang,
        "the slow set's dt/2 divisor assumes Strang; under Lie it is dt and the \
         precondition would be 2x too permissive"
    );
}

#[test]
fn pack_deferral_is_what_makes_the_frozen_rate_read_sound() {
    // **The Rust-only coupling, and it is a safety one.** Python reads `k` off the
    // *built flow*, because a param PACK may have inflated it — that is Step 5's
    // load-bearing reason the check is at build time at all. Rust reads the FROZEN
    // constant (`frozen_rate_value`), which is only correct while packs cannot exist
    // here.
    //
    // If packs are ever added to the Rust port, `frozen_rate_value` silently becomes a
    // **false PASS in the unsafe direction** — it would report the frozen k while the
    // flow ran the pack's inflated one. That is exactly the shape Step 5 caught in the
    // plan's own dt/n_sub formula, and it would be invisible: `eclss.o2_makeup` is
    // demand-controlled, so the run-time backstop cannot see the oscillation either.
    //
    // So this pin is the tripwire: the day a pack builds here, this goes red and whoever
    // did it must fix `frozen_rate_value` to read the resolved value first.
    let yaml = "name: pack_probe\n\
                integrator: euler\n\
                dt: 60.0\n\
                steps: 1\n\
                stocks:\n\
                \x20 - id: eclss.cabin_co2\n    domain: eclss\n    quantity: carbon\n    kind: pool\n    amount: 10.0\n\
                \x20 - id: eclss.co2_removed\n    domain: boundary\n    quantity: carbon\n    kind: boundary\n    amount: 0.0\n\
                flows:\n\
                \x20 - id: eclss.co2_scrubber\n    type: eclss.co2_scrubber\n\
                \x20\x20\x20 wiring:\n      cabin_co2: eclss.cabin_co2\n      co2_removed: eclss.co2_removed\n\
                \x20\x20\x20 params:\n      pack: eclss_hot_makeup.yaml\n";
    let err = match build(yaml) {
        Ok(_) => panic!(
            "a param pack must still be REFUSED in the Rust port — `frozen_rate_value` \
             reads the frozen k and would not see a pack's inflated one"
        ),
        Err(e) => e,
    };
    assert!(
        err.message.contains("pack"),
        "the refusal must name packs: {}",
        err.message
    );
}

// ---------------------------------------------------------------------------
// The routing branch, pinned behaviorally (Rust has no monkeypatch).
// ---------------------------------------------------------------------------

const AUX_INC: f64 = 0.5;

/// A trivial forced flow (no rate constant ⇒ nothing for the precondition to judge).
struct Trickle;
impl Flow for Trickle {
    fn id(&self) -> &str {
        "sim.trickle"
    }
    fn evaluate(
        &self,
        _s: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let flux = 0.01 * dt;
        FlowResult::new(vec![
            Leg::new("sim.a".to_string(), -flux)?,
            Leg::new("boundary.snk".to_string(), flux)?,
        ])
    }
}

/// An accumulator: the thing `multirate_step` deliberately never advances.
struct Ticker;
impl AuxProcess for Ticker {
    fn id(&self) -> &str {
        "sim.ticker"
    }
    fn evaluate(
        &self,
        _s: &State,
        _env: &dyn Environment,
        _dt: f64,
    ) -> Result<BTreeMap<String, f64>, SimError> {
        Ok(BTreeMap::from([("ticks".to_string(), AUX_INC)]))
    }
}

fn aux_stocks() -> BTreeMap<String, Stock> {
    let mk = |id: &str, kind: StockKind, amount: f64| {
        Stock::new(
            id.to_string(),
            if kind == StockKind::Boundary {
                "boundary".to_string()
            } else {
                "sim".to_string()
            },
            Quantity::Carbon,
            "mol".to_string(),
            amount,
            kind,
            0.0,
            false,
            BTreeMap::new(),
        )
        .unwrap()
    };
    BTreeMap::from([
        ("sim.a".to_string(), mk("sim.a", StockKind::Pool, 100.0)),
        (
            "boundary.snk".to_string(),
            mk("boundary.snk", StockKind::Boundary, 0.0),
        ),
    ])
}

/// A hand-built aux-bearing scenario. `interpret` can never produce one (it never wires
/// `aux_processes` — the authoring layer cannot express aux at all), which is precisely
/// why the aux tripwire lives in `run` rather than `interpret`: here it is *reachable and
/// testable*; there it would be unreachable and untestable both.
fn aux_scenario(n_sub: u32) -> BuiltScenario {
    let stocks = aux_stocks();
    let registry = Registry::new(vec![Box::new(Trickle)], &stocks, vec![Box::new(Ticker)]).unwrap();
    let fast = Registry::new(vec![Box::new(Trickle)], &stocks, vec![Box::new(Ticker)]).unwrap();
    let slow = Registry::flows_only(vec![], &stocks).unwrap();
    BuiltScenario {
        name: "aux_probe".to_string(),
        state: State::new(0, stocks, 0, BTreeMap::new()).unwrap(),
        registry,
        slow_registry: slow,
        fast_registry: fast,
        resolver: SourceResolver::new(Default::default(), Default::default()).unwrap(),
        integrator: "euler".to_string(),
        dt: 1.0,
        steps: 3,
        n_sub,
        has_authored_kinetics: false,
    }
}

#[test]
fn a_single_rate_scenario_never_touches_the_driver() {
    // **THE BRANCH PIN.** Python monkeypatches `multirate_step` to raise; Rust has no
    // such seam, so this pins the branch *behaviorally* — and the observable is aux.
    //
    // `step_report` advances `State.aux`; `multirate_step` deliberately NEVER does
    // (decision P2). So if the single-rate branch ever leaked into the driver, the ticks
    // below would freeze at 0.0 and this test goes red.
    //
    // Why the branch needs a pin of its own: `n_sub=1` + an empty slow set reproduces the
    // single-rate trajectory bit-for-bit, so routing EVERYTHING through the driver would
    // keep every golden green — while silently resting all 25 on that identity holding
    // forever. The leak would surface years later as a `simcore` change moving 25 files
    // at once with no cause attached.
    let result = run_scenario(aux_scenario(1)).expect("a single-rate aux graph runs");
    assert_eq!(
        result.final_state.aux.get("ticks").copied(),
        Some(AUX_INC * 3.0),
        "aux must have advanced once per step — if it is 0/absent, the single-rate path \
         leaked into multirate_step, which never advances aux"
    );
}

#[test]
fn a_multirate_scenario_with_aux_is_refused_rather_than_silently_frozen() {
    // The aux tripwire. `multirate_step` never advances aux (P2), and the conservation
    // gate structurally CANNOT see the freeze (aux is non-conserved by definition), so
    // the failure mode this prevents is a run that balances, completes, reports clean —
    // and has a frozen accumulator.
    //
    // Unreachable from any authored file today (`interpret` never wires aux); it is a
    // tripwire for the phase that makes the biosphere — the one aux-bearing domain —
    // authorable.
    let err = match run_scenario(aux_scenario(4)) {
        Ok(_) => panic!("a multi-rate aux graph must be refused, not silently frozen"),
        Err(e) => e,
    };
    assert!(
        err.message.contains("aux"),
        "the message must name aux: {}",
        err.message
    );
    assert!(
        err.message.contains("sim.ticker"),
        "the offending process must be named: {}",
        err.message
    );
}

#[test]
fn the_rationed_message_names_n_sub_only_on_the_multirate_path() {
    // "Increase n_sub" is honest on the multi-rate path and WRONG on the single-rate one:
    // there is no n_sub to raise, and naming it sends an author hunting for a key their
    // scenario does not have. The hatch is needed to *reach* a rationed run at all now.
    let built = build_allowing_unsafe(&cabin_yaml("3600.0", "1", "fast", "fast"))
        .expect("the hatch opens the build");
    let err = match run_scenario(built) {
        Ok(_) => panic!("an unsafe single-rate run must ration"),
        Err(e) => e,
    };
    assert_eq!(err.kind, authoring::ErrorKind::Rationed);
    assert!(
        !err.message.contains("n_sub"),
        "the single-rate remedy must NOT name n_sub: {}",
        err.message
    );
}
