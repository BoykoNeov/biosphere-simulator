//! The step-options lab's constructional controls (`docs/plans/post-roadmap-step-options.md`
//! §3), kept as tests so a later edit to the drivers cannot quietly turn them into a different
//! run. The scorecard itself is `examples/step_options.rs`; these are the checks whose answers
//! arithmetic fixes in advance, on one season to stay cheap.

use std::cell::RefCell;
use std::rc::Rc;

use domains::biosphere::params::BiosphereParams;
use domains::biosphere::science::LeafAreaForm;
use domains::biosphere::system::{
    build_season_with, perennial_chamber_scenario, sealed_chamber_scenario, SeasonScenario,
    DEFAULT_SCENARIO,
};
use domains::lab::biosphere_with_leaf_form;
use domains::lab::step_options::{
    build_season_implicit_uptake, run_split, run_uniform, Scope, SolveLog, SplitRule, Watched,
};
use simcore::error::SimError;
use simcore::registry::Registry;
use simcore::state::State;

fn frozen() -> BiosphereParams {
    biosphere_with_leaf_form(&[], LeafAreaForm::Derived).expect("params")
}

fn fingerprint(s: &State) -> Vec<u64> {
    let mut out: Vec<u64> = s.stocks.values().map(|st| st.amount.to_bits()).collect();
    out.extend(s.aux.values().map(|v| v.to_bits()));
    out
}

/// Fingerprints every `every` steps of a uniform run.
fn uniform_prints(s: &SeasonScenario, years: usize, perennial: bool, spd: usize, every: u64) -> Vec<Vec<u64>> {
    let p = frozen();
    let mut prints = Vec::new();
    let mut obs = |st: &State| {
        if st.n.is_multiple_of(every) {
            prints.push(fingerprint(st));
        }
    };
    run_uniform(s, years, perennial, &p, &build_season_with, spd, &mut obs).expect("uniform");
    prints
}

fn split_prints(s: &SeasonScenario, years: usize, perennial: bool, rule: SplitRule) -> (Vec<Vec<u64>>, u64, u64) {
    let p = frozen();
    let mut prints = Vec::new();
    let mut obs = |st: &State| prints.push(fingerprint(st));
    let (_, stats) =
        run_split(s, years, perennial, &p, &build_season_with, 4, rule, &mut obs).expect("split");
    (prints, stats.split_steps, stats.probes)
}

#[test]
fn b_never_splitting_is_the_reference_run() {
    let jar = sealed_chamber_scenario();
    let rule = SplitRule { theta: f64::INFINITY, watched: Watched::Every, max_depth: 6 };
    let (b, splits, probes) = split_prints(&jar, 1, false, rule);
    assert_eq!((splits, probes), (0, 0));
    assert_eq!(b, uniform_prints(&jar, 1, false, 4, 1));
}

/// The only check that proves the counter renumbering: a split step's halves must see the
/// same forcing times, advance the same phenology counters, and land where a uniform eighth-day
/// run lands — bit for bit, at every coarse step, across a re-sow.
#[test]
fn b_always_splitting_once_is_the_eighth_day_run_across_a_resow() {
    let chamber = perennial_chamber_scenario();
    let rule = SplitRule { theta: -1.0, watched: Watched::Every, max_depth: 1 };
    let (b, splits, _) = split_prints(&chamber, 2, true, rule);
    assert_eq!(splits as usize, b.len() - 1, "every coarse step must split");
    assert_eq!(b, uniform_prints(&chamber, 2, true, 8, 2));
}

#[test]
fn b_refuses_rather_than_rations() {
    // The lab leaf form draws 1.15 of the jar's CO₂ on one quarter-day step. With no room to
    // split, B must refuse — not hand the step to the backstop.
    let jar = sealed_chamber_scenario();
    let p = biosphere_with_leaf_form(&[], LeafAreaForm::NodeEnvelope).expect("params");
    let rule = SplitRule { theta: 0.5, watched: Watched::ChamberCo2, max_depth: 0 };
    let mut obs = |_: &State| {};
    let err = run_split(&jar, 1, false, &p, &build_season_with, 4, rule, &mut obs)
        .expect_err("B must refuse an overdraw it cannot split away");
    assert!(err.to_string().contains("refuses"), "{err}");
}

fn implicit_run(s: &SeasonScenario, scope: Scope, checked: bool) -> (State, SolveLog) {
    let p = frozen();
    let log = Rc::new(RefCell::new(SolveLog::default()));
    let build_log = Rc::clone(&log);
    let build = move |sc: &SeasonScenario, pp: &BiosphereParams| -> Result<(State, Registry), SimError> {
        build_season_implicit_uptake(sc, pp, scope, checked, Rc::clone(&build_log))
    };
    let mut obs = |_: &State| {};
    let (end, rationed) = run_uniform(s, 1, false, &p, &build, 4, &mut obs).expect("C run");
    assert_eq!(rationed, 0);
    let log = log.borrow().clone();
    (end, log)
}

#[test]
fn c_leaves_the_open_field_alone() {
    let (c, log) = implicit_run(&DEFAULT_SCENARIO, Scope::WithInflows, true);
    assert_eq!(log.solves, 0);
    let p = frozen();
    let mut obs = |_: &State| {};
    let (reference, _) =
        run_uniform(&DEFAULT_SCENARIO, 1, false, &p, &build_season_with, 4, &mut obs).expect("ref");
    assert_eq!(fingerprint(&c), fingerprint(&reference));
}

/// C must solve on every step, satisfy its own equation, touch nothing but the crop's uptake —
/// and actually MOVE the jar: a pool override that the inner flow never read would pass the
/// residual check trivially (`U = U(C₀)` is its own fixed point) and leave the run unchanged.
#[test]
fn c_solves_stays_in_scope_and_moves_the_jar() {
    let jar = sealed_chamber_scenario();
    let (c, log) = implicit_run(&jar, Scope::CropOnly, true);
    assert_eq!(log.solves, 1220);
    assert_eq!(log.scope_checks, log.solves);
    assert!(log.differing.is_empty(), "{:?}", log.differing);
    assert!(log.idle < log.solves, "the crop never took anything");
    let p = frozen();
    let mut obs = |_: &State| {};
    let (reference, _) = run_uniform(&jar, 1, false, &p, &build_season_with, 4, &mut obs).expect("ref");
    assert_ne!(fingerprint(&c), fingerprint(&reference), "C left the jar bit-identical");
}

/// The with-returns scope reads the crop's whole budget against one end-of-step pool. Its
/// explicit remainder must not read the pool (no witness differs), it must never ration on the
/// jar, and it must move the jar differently from the crop-only scope — or the returns were
/// never counted.
#[test]
fn c_with_returns_solves_in_scope_and_differs_from_crop_only() {
    let jar = sealed_chamber_scenario();
    let (with_returns, log) = implicit_run(&jar, Scope::WithInflows, true);
    assert_eq!(log.solves, 1220);
    assert!(log.differing.is_empty(), "{:?}", log.differing);
    assert_eq!(log.beyond_start, 0);
    let (crop_only, _) = implicit_run(&jar, Scope::CropOnly, false);
    assert_ne!(fingerprint(&with_returns), fingerprint(&crop_only));
}
