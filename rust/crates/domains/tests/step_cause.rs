//! The step-cause lab's constructional controls (`docs/plans/post-roadmap-step-cause.md` §3),
//! kept as tests so a later edit to the instruments cannot quietly turn them into a different
//! run. The four-cell table itself is `examples/step_cause.rs`; these are the checks whose
//! answers arithmetic fixes in advance, on one season to stay cheap.

use std::collections::BTreeSet;

use domains::biosphere::params::BiosphereParams;
use domains::biosphere::science::LeafAreaForm;
use domains::biosphere::stocks::PAR_VAR;
use domains::biosphere::system::{
    build_season_with, sealed_chamber_scenario, weather_resolver, SeasonScenario,
    DEFAULT_SCENARIO,
};
use domains::lab::biosphere_with_leaf_form;
use domains::lab::step_cause::{
    blocked_light_resolver, build_season_light_quadrature, par_readers, par_schedule,
    run_uniform_blocked_light, LightQuadrature, BUDGET_FLOWS,
};
use domains::lab::step_options::run_uniform;
use simcore::environment::Environment;
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
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

fn plain_end(s: &SeasonScenario) -> Vec<u64> {
    let p = frozen();
    let (end, _) = run_uniform(s, 1, false, &p, &build_season_with, 4, &mut |_| {}).expect("plain");
    fingerprint(&end)
}

#[test]
fn e1_with_a_step_sized_light_block_is_the_plain_run() {
    let p = frozen();
    for s in [sealed_chamber_scenario(), DEFAULT_SCENARIO] {
        let (end, _) =
            run_uniform_blocked_light(&s, 1, false, &p, &build_season_with, 4, 4, &mut |_| {})
                .expect("E1");
        assert_eq!(fingerprint(&end), plain_end(&s), "the rebuilt resolver is not the stock one");
    }
}

#[test]
fn e1_hands_every_fine_step_its_coarse_window() {
    // A step of 1/8 day under quarter-day light blocks: both eighths of quarter-day m read the
    // stock resolver's quarter-day value at m — and at least one day's two eighths would have
    // read different light without the block (non-vacuity).
    let jar = sealed_chamber_scenario();
    let (state, _) = build_season_with(&jar, &frozen()).expect("build");
    let stock = weather_resolver(&jar, 1).expect("stock");
    let blocked = blocked_light_resolver(&jar, 1, 4).expect("blocked");
    let at = |n: u64| State::new(n, state.stocks.clone(), state.rng_seed, state.aux.clone()).expect("state");
    let mut differs = false;
    for m in 0..(4 * 30) {
        let coarse = stock.bind(&at(m), 0.25).get(PAR_VAR).expect("par");
        for half in [2 * m, 2 * m + 1] {
            let got = blocked.bind(&at(half), 0.125).get(PAR_VAR).expect("par");
            assert_eq!(got.to_bits(), coarse.to_bits(), "eighth {half} did not read quarter-day {m}");
            let unblocked = stock.bind(&at(half), 0.125).get(PAR_VAR).expect("par");
            differs |= unblocked != coarse;
        }
    }
    assert!(differs, "the block changed nothing on 30 days — the check is vacuous");
}

#[test]
fn e2_at_one_sub_window_is_the_plain_run() {
    let p = frozen();
    for s in [sealed_chamber_scenario(), DEFAULT_SCENARIO] {
        let build = |sc: &SeasonScenario, pp: &BiosphereParams| -> Result<(State, Registry), SimError> {
            build_season_light_quadrature(sc, pp, 1, 1)
        };
        let (end, _) = run_uniform(&s, 1, false, &p, &build, 4, &mut |_| {}).expect("E2");
        assert_eq!(fingerprint(&end), plain_end(&s), "E2 at k = 1 moved the run");
    }
    // Non-vacuity: at k = 4 it does move the jar.
    let jar = sealed_chamber_scenario();
    let build4 = |sc: &SeasonScenario, pp: &BiosphereParams| -> Result<(State, Registry), SimError> {
        build_season_light_quadrature(sc, pp, 1, 4)
    };
    let (end, _) = run_uniform(&jar, 1, false, &p, &build4, 4, &mut |_| {}).expect("E2 k=4");
    assert_ne!(fingerprint(&end), plain_end(&jar), "E2 at k = 4 changed nothing");
}

/// A probe flow whose one leg is the PAR it was handed (times `dt`), so the wrapper's average
/// can be read back directly.
struct ParLeg;

impl Flow for ParLeg {
    fn type_name(&self) -> &'static str {
        "ParLeg"
    }
    fn id(&self) -> &str {
        "probe.par_leg"
    }
    fn evaluate(&self, _: &State, env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        FlowResult::new(vec![Leg::new("probe.par".to_string(), env.get(PAR_VAR)? * dt)?])
    }
}

#[test]
fn the_quadrature_reads_the_windows_a_finer_step_would() {
    // The mean of the k sub-window lights is the step's own window mean (the window means
    // partition one integral) — which holds only if sub-window j of step n is read at the
    // renumbered counter n·k + j with step dt/k. And the sub-windows really differ, or the
    // average is trivially the whole.
    let jar = sealed_chamber_scenario();
    let (state, _) = build_season_with(&jar, &frozen()).expect("build");
    let stock = weather_resolver(&jar, 1).expect("stock");
    let par = par_schedule(&jar, 1).expect("par");
    for k in [2u64, 4, 16] {
        let wrapped = LightQuadrature::new(Box::new(ParLeg), par_schedule(&jar, 1).expect("par"), k);
        let mut uneven = false;
        for n in 0..(4 * 30) {
            let s = State::new(n, state.stocks.clone(), state.rng_seed, state.aux.clone()).expect("state");
            let env = stock.bind(&s, 0.25);
            let whole = env.get(PAR_VAR).expect("par") * 0.25;
            let got = wrapped.evaluate(&s, &env, 0.25).expect("evaluate").legs[0].amount;
            assert!(
                (got - whole).abs() <= 1e-12 * whole.max(1e-300),
                "k = {k}, step {n}: sub-window mean {got} is not the window's {whole}"
            );
            let first = par(n * k, 0.25 / k as f64);
            uneven |= (1..k).any(|j| par(n * k + j, 0.25 / k as f64) != first);
        }
        assert!(uneven, "k = {k}: every sub-window read the same light — the check is vacuous");
    }
}

#[test]
fn nothing_outside_the_budget_flows_reads_the_light() {
    let want: BTreeSet<String> = BUDGET_FLOWS.iter().map(|s| s.to_string()).collect();
    for s in [sealed_chamber_scenario(), DEFAULT_SCENARIO] {
        let (readers, lit) = par_readers(&s, &frozen()).expect("readers");
        assert!(lit > 0);
        assert!(readers.is_subset(&want), "PAR read outside the wrapped set: {readers:?}");
        assert!(readers.contains(BUDGET_FLOWS[0]));
    }
}
