//! Phase-8 Step-0 parity teeth: caller-driven [`SimSession`] stepping is **bit-identical**
//! to the Phase-7 run-to-completion runners. This equivalence *is* the "the exact same
//! simulation runs headless" guarantee — provable here with no Godot toolchain. (The
//! genuine *cross-boundary* check, through the actual `gdext` cdylib Godot loads, is a
//! separate first-class deliverable in Steps 1 and 8: an intra-process test cannot see
//! FP-environment divergence such as FTZ/DAZ flags a game engine may set per-thread.)
//!
//! States are compared by their exact hex-float `sim_io` JSON (the golden discipline), so
//! a match is bit-exact — a single last-ULP difference would change the spelling and fail.
//! Coverage: the single-rate seam (`cabin_gas`, Tier-1 transcendental-free), and both
//! two-rate assemblies + both reset branches (greenhouse with `reset = None`, sealed with
//! the real `sealed_reset_hook`).

use domains::crew::FECAL_WASTE;
use domains::params;
use simcore::integrator::EulerIntegrator;
use simcore::snapshot::from_engine;
use simcore::state::State;
use station::cabin::{build_cabin, cabin_resolver};
use station::driver::run_master_day;
use station::greenhouse::{
    build_greenhouse, greenhouse_bio_resolver, greenhouse_cabin_resolver, run_greenhouse,
};
use station::params as station_params;
use station::run_station;
use station::scenario::{
    greenhouse_scenario, sealed_station_scenario, CABIN_GAS_SCENARIO, CABIN_GAS_STEPS,
};
use station::sealed::{
    build_sealed_station, sealed_bio_resolver, sealed_fast_resolver, sealed_reset_hook,
};
use station::session::SimSession;

/// Exact hex-float snapshot of a state (n + seed + aux + stocks) — a string match is a
/// bit-for-bit match.
fn snap(state: &State) -> String {
    from_engine(state).to_json()
}

/// Single-rate: `N × session.step()` == `run_station(N)`, bit-exact. The cabin loop is
/// transcendental-free (Tier-1) — the cleanest tripwire for an op-reordering bug in the
/// owned-loop → caller-driven inversion.
#[test]
fn single_rate_session_matches_run_station_bit_exact() {
    let crew = params::crew();
    let eclss = params::eclss();
    let scenario = CABIN_GAS_SCENARIO;

    // Reference: run-to-completion.
    let (ref_state, ref_reg) = build_cabin(&crew, &eclss, &scenario).unwrap();
    let ref_resolver = cabin_resolver(&scenario).unwrap();
    let ref_integrator = EulerIntegrator::new(ref_reg);
    let mut noop = |_: &State| {};
    let (ref_final, ref_rationed, ref_events) = run_station(
        &ref_integrator,
        ref_state,
        &ref_resolver,
        scenario.dt_seconds,
        CABIN_GAS_STEPS,
        &mut noop,
    )
    .unwrap();

    // Session: caller-driven, same build, stepped the same count.
    let (sess_state, sess_reg) = build_cabin(&crew, &eclss, &scenario).unwrap();
    let sess_resolver = cabin_resolver(&scenario).unwrap();
    let mut session = SimSession::single_rate(
        EulerIntegrator::new(sess_reg),
        sess_state,
        sess_resolver,
        scenario.dt_seconds,
    );
    session.step_n(CABIN_GAS_STEPS).unwrap();

    assert_eq!(session.n(), CABIN_GAS_STEPS, "step count");
    assert_eq!(snap(session.state()), snap(&ref_final), "state bit-identical");
    assert_eq!(session.total_rationed(), ref_rationed, "rationed");
    assert_eq!(session.events(), ref_events.as_slice(), "events");
    // Non-vacuous: the run is well-fed (else the comparison would pass trivially).
    assert_eq!(ref_rationed, 0);
    assert!(ref_events.is_empty());
}

/// Two-rate, `reset = None` (the greenhouse seam): `days × session.step()` ==
/// `run_greenhouse(days)`, bit-exact. Exercises the slow biosphere step + 1440 fast
/// sub-steps/day through the shared [`station::driver::advance_one_master_day`].
#[test]
fn two_rate_greenhouse_session_matches_run_greenhouse_bit_exact() {
    let crew = params::crew();
    let eclss = params::eclss();
    let scenario = greenhouse_scenario();

    // Reference.
    let (ref_state, ref_bio, ref_cabin) =
        build_greenhouse(&crew, &eclss, &scenario, true, FECAL_WASTE).unwrap();
    let (ref_final, ref_rationed, ref_events) = run_greenhouse(
        &EulerIntegrator::new(ref_bio),
        &EulerIntegrator::new(ref_cabin),
        ref_state,
        &greenhouse_bio_resolver(&scenario).unwrap(),
        &greenhouse_cabin_resolver(&scenario).unwrap(),
        &scenario,
    )
    .unwrap();
    let ref_last = ref_final.last().unwrap();

    // Session (reset = None, mirroring run_greenhouse's driver call).
    let (sess_state, sess_bio, sess_cabin) =
        build_greenhouse(&crew, &eclss, &scenario, true, FECAL_WASTE).unwrap();
    let mut session = SimSession::two_rate(
        EulerIntegrator::new(sess_bio),
        EulerIntegrator::new(sess_cabin),
        sess_state,
        greenhouse_bio_resolver(&scenario).unwrap(),
        greenhouse_cabin_resolver(&scenario).unwrap(),
        scenario.steps_per_day,
        scenario.bio_dt,
        scenario.cabin_dt,
        None,
    )
    .unwrap();
    session.step_n(scenario.days as u64).unwrap();

    assert_eq!(session.n(), scenario.days as u64, "master-day count");
    assert_eq!(snap(session.state()), snap(ref_last), "state bit-identical");
    assert_eq!(session.total_rationed(), ref_rationed, "rationed");
    assert_eq!(session.events(), ref_events.as_slice(), "events");
}

/// Two-rate, `reset = Some(sealed_reset_hook)` (the sealed seam): a short horizon
/// (3 master days, no season boundary crossed) driven through the *actual sealed pieces* —
/// `run_master_day` with the real hook vs a session built with the same hook. This proves
/// the `Some(reset)` wiring is stored and passed bit-identically; the reset-*adopt* branch
/// is not session-specific code (it lives in the shared `advance_one_master_day` +
/// `sealed_reset_hook`, both exercised by the full sealed golden), and the ignored
/// `full_horizon` test below drives it end-to-end.
#[test]
fn two_rate_sealed_session_matches_run_master_day_bit_exact() {
    parity_sealed(3);
}

/// The literal advisor statement `run_sealed(days) == days × session.step()` over the full
/// multi-year horizon (crosses season boundaries → the reset-adopt branch fires). ~1.3 M
/// sub-steps twice; run with `cargo test -- --ignored`.
#[test]
#[ignore = "full multi-year horizon (~2.6 M sub-steps); run with --ignored"]
fn two_rate_sealed_session_matches_full_horizon_bit_exact() {
    let scenario = sealed_station_scenario();
    parity_sealed(scenario.days() as u64);
}

/// Drive the sealed station `days` master days both ways and assert bit-exact parity.
fn parity_sealed(days: u64) {
    let charge = params::charge();
    let thermal = params::thermal();
    let crew = params::crew();
    let eclss = params::eclss();
    let recovery = station_params::water_recovery();
    let lamp = station_params::lamp();
    let harvest = station_params::harvest();
    let scenario = sealed_station_scenario();

    // Reference: the sealed pieces driven `days` days via run_master_day with the real
    // owned reset hook (exactly run_sealed's construction, but with a caller-set horizon).
    let (ref_state, ref_bio, ref_fast) = build_sealed_station(
        &charge, &thermal, &crew, &eclss, &recovery, &lamp, &harvest, &scenario, false, false,
    )
    .unwrap();
    let ref_reset = sealed_reset_hook(&scenario);
    let (ref_states, ref_rationed, ref_events) = run_master_day(
        &EulerIntegrator::new(ref_bio),
        &EulerIntegrator::new(ref_fast),
        ref_state,
        &sealed_bio_resolver(&lamp, &scenario).unwrap(),
        &sealed_fast_resolver(&charge, &scenario).unwrap(),
        days as usize,
        scenario.steps_per_day,
        scenario.bio_dt,
        scenario.cabin_dt,
        Some(&*ref_reset),
    )
    .unwrap();
    let ref_last = ref_states.last().unwrap();

    // Session: same pieces, same owned reset hook, stepped `days`.
    let (sess_state, sess_bio, sess_fast) = build_sealed_station(
        &charge, &thermal, &crew, &eclss, &recovery, &lamp, &harvest, &scenario, false, false,
    )
    .unwrap();
    let mut session = SimSession::two_rate(
        EulerIntegrator::new(sess_bio),
        EulerIntegrator::new(sess_fast),
        sess_state,
        sealed_bio_resolver(&lamp, &scenario).unwrap(),
        sealed_fast_resolver(&charge, &scenario).unwrap(),
        scenario.steps_per_day,
        scenario.bio_dt,
        scenario.cabin_dt,
        Some(sealed_reset_hook(&scenario)),
    )
    .unwrap();
    session.step_n(days).unwrap();

    assert_eq!(session.n(), days, "master-day count");
    assert_eq!(snap(session.state()), snap(ref_last), "state bit-identical");
    assert_eq!(session.total_rationed(), ref_rationed, "rationed");
    assert_eq!(session.events(), ref_events.as_slice(), "events");
}
