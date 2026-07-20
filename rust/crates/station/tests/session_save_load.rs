//! Phase-8 Step-7 (P8.7) save/load parity teeth: a session **saved at step A**, its
//! state serialized (`to_json`) and reloaded (`from_json`), and stepped **B more**,
//! is **bit-identical** to a session that ran straight through to `A + B` without ever
//! saving. That equivalence *is* the save/load guarantee — provable here with no Godot
//! toolchain (the genuine cross-boundary FileAccess round-trip is Step 7's smoke).
//!
//! Why it holds by construction: the fixed-palette registry is rebuilt deterministically
//! from the recipe, the loader restores the exact `State` (stocks + aux + `n` + seed via
//! the hex-float codec), and the core RNG is `(seed, key, n)`-keyed — there is no
//! sequential RNG state to lose (the module-doc determinism corollary). So a resume needs
//! `(recipe, State)` and nothing else.
//!
//! States are compared by their exact hex-float `sim_io` JSON, so a match is bit-exact.
//! Coverage: single-rate (`cabin_gas`, Tier-1), two-rate with an accumulated aux
//! (`greenhouse` — proves the biosphere aux/phenology survives the round-trip; since scope
//! (B) increment 1 the save point is past day 14, where `vernalization_days` first
//! accrues, because thermal_time is arrested at warm sowing), and an `#[ignore]`d sealed
//! resume that crosses a season boundary (the reset-adopt branch).

use domains::crew::FECAL_WASTE;
use domains::params;
use simcore::integrator::EulerIntegrator;
use simcore::snapshot::{from_engine, from_json};
use simcore::state::State;
use station::cabin::{build_cabin, cabin_resolver};
use station::greenhouse::{build_greenhouse, greenhouse_bio_resolver, greenhouse_cabin_resolver};
use station::params as station_params;
use station::scenario::{greenhouse_scenario, sealed_station_scenario, CABIN_GAS_SCENARIO};
use station::sealed::{
    build_sealed_station, sealed_bio_resolver, sealed_fast_resolver, sealed_reset_hook,
};
use station::session::SimSession;

/// Exact hex-float snapshot of a state — a string match is a bit-for-bit match.
fn snap(state: &State) -> String {
    from_engine(state).to_json()
}

/// The headline gate: `build()` produces a *fresh* session for the same recipe. Running
/// it straight to `total`, versus saving its state at `save_at` (through `to_json` →
/// `from_json`), loading it into a fresh session, and stepping the remaining `total -
/// save_at`, yields **bit-identical** final states.
fn assert_resume_parity(build: impl Fn() -> SimSession, save_at: u64, total: u64) {
    // Reference: never saved.
    let mut straight = build();
    straight.step_n(total).unwrap();
    let straight_final = snap(straight.state());

    // Save at `save_at`: serialize the mid-run state exactly as the game would.
    let mut saver = build();
    saver.step_n(save_at).unwrap();
    let saved_text = snap(saver.state());

    // The loader is a faithful inverse: the reloaded state is bit-identical to the saved one.
    let loaded = from_json(&saved_text).expect("from_json accepts a saved snapshot");
    assert_eq!(snap(&loaded), saved_text, "loader round-trip is bit-exact");

    // Resume: a fresh session (registries rebuilt from the recipe), load, step the rest.
    let mut resume = build();
    resume.load_state(loaded).unwrap();
    assert_eq!(resume.n(), save_at, "loaded state carries n");
    resume.step_n(total - save_at).unwrap();

    assert_eq!(resume.n(), total, "resumed to the full horizon");
    assert_eq!(
        snap(resume.state()),
        straight_final,
        "resume-after-load is bit-identical to a never-saved straight run"
    );
    // Non-vacuous: the state actually advanced (a no-op run would pass trivially).
    let mut fresh = build();
    fresh.step_n(save_at).unwrap();
    assert_ne!(
        snap(fresh.state()),
        straight_final,
        "the run is non-trivial"
    );
}

/// Single-rate `cabin_gas` (Tier-1, transcendental-free): save at 120, resume to 300.
#[test]
fn cabin_gas_resume_after_save_is_bit_identical() {
    let build = || {
        let crew = params::crew();
        let eclss = params::eclss();
        let scenario = CABIN_GAS_SCENARIO;
        let (state, registry) = build_cabin(&crew, &eclss, &scenario).unwrap();
        let resolver = cabin_resolver(&scenario).unwrap();
        SimSession::single_rate(
            EulerIntegrator::new(registry),
            state,
            resolver,
            scenario.dt_seconds,
        )
    };
    assert_resume_parity(build, 120, 300);
}

/// Two-rate `greenhouse`: save at day 2 (after `thermal_time` has accumulated), resume to
/// day 4. The biosphere aux/phenology is part of the saved `State` (the snapshot's `aux`
/// map), so a bit-identical resume proves the round-trip preserves it — the reason a
/// two-rate case matters over the single-rate one.
#[test]
fn greenhouse_resume_after_save_preserves_aux_phenology() {
    let build = || {
        let crew = params::crew();
        let eclss = params::eclss();
        let scenario = greenhouse_scenario();
        let (state, bio, cabin) =
            build_greenhouse(&crew, &eclss, &scenario, true, FECAL_WASTE).unwrap();
        SimSession::two_rate(
            EulerIntegrator::new(bio),
            EulerIntegrator::new(cabin),
            state,
            greenhouse_bio_resolver(&scenario).unwrap(),
            greenhouse_cabin_resolver(&scenario).unwrap(),
            scenario.steps_per_day,
            scenario.bio_dt,
            scenario.cabin_dt,
            None,
        )
        .unwrap()
    };
    // Pre-flight: some aux accumulator is non-zero by the save point (else the aux claim
    // is vacuous). ⚠ Save point moved 2 → 16 by post-roadmap scope (B) increment 1: with
    // vernalization + photoperiod, thermal_time is ARRESTED at sowing (verfun = 0 until
    // the cold requirement is met) and it is too warm in early October to vernalize, so
    // BOTH accumulators are legitimately 0 for the first ~14 days. `vernalization_days`
    // first accrues on day 14 (temperature drops below the 12 °C ceiling). Saving at day
    // 16 makes the guard meaningful again — and now it proves the SECOND accumulator
    // round-trips, a strictly stronger check than the old single-accumulator one.
    let mut probe = build();
    probe.step_n(16).unwrap();
    assert!(
        probe.state().aux.values().any(|&v| v != 0.0),
        "greenhouse should have accumulated vernalization_days by day 16"
    );
    assert_resume_parity(build, 16, 18);
}

/// `load_state` rejects a state whose stock-id set does not match the session's registry —
/// a save from a different recipe, or a corrupt file. Cross-load a `station` state into a
/// `cabin_gas` session (disjoint stock sets) and expect a loud `Validation` error.
#[test]
fn load_state_rejects_a_mismatched_stock_set() {
    let crew = params::crew();
    let eclss = params::eclss();
    let (cabin_state, cabin_reg) = build_cabin(&crew, &eclss, &CABIN_GAS_SCENARIO).unwrap();
    let mut cabin_session = SimSession::single_rate(
        EulerIntegrator::new(cabin_reg),
        cabin_state,
        cabin_resolver(&CABIN_GAS_SCENARIO).unwrap(),
        CABIN_GAS_SCENARIO.dt_seconds,
    );

    // A station (Power → Thermal) state — a completely different stock-id set.
    let charge = params::charge();
    let thermal = params::thermal();
    let scenario = station::scenario::HEAT_CLOSURE_SCENARIO;
    let (station_state, _reg) =
        station::system::build_station(&charge, &thermal, &scenario, None).unwrap();

    let err = cabin_session.load_state(station_state).unwrap_err();
    assert!(
        matches!(err, simcore::error::SimError::Validation(_)),
        "cross-recipe load must be a Validation error, got {err:?}"
    );
}

/// The sealed station resumed **across a season boundary** — save one day before the reset,
/// resume through it. This is the genuinely-new combination (save/load + the biosphere
/// re-sow), transitively covered by the `#[ignore]`d full-horizon session-parity test but
/// pinned here on its own. ~440 K sub-steps; run with `cargo test -- --ignored`.
#[test]
#[ignore = "crosses a sealed season boundary (~440 K sub-steps); run with --ignored"]
fn sealed_resume_across_a_season_boundary_is_bit_identical() {
    let scenario = sealed_station_scenario();
    let boundary = scenario.season_days as u64;
    let build = || {
        let charge = params::charge();
        let thermal = params::thermal();
        let crew = params::crew();
        let eclss = params::eclss();
        let recovery = station_params::water_recovery();
        let lamp = station_params::lamp();
        let harvest = station_params::harvest();
        let scenario = sealed_station_scenario();
        let (state, bio, fast) = build_sealed_station(
            &charge, &thermal, &crew, &eclss, &recovery, &lamp, &harvest, &scenario, false, false,
        )
        .unwrap();
        SimSession::two_rate(
            EulerIntegrator::new(bio),
            EulerIntegrator::new(fast),
            state,
            sealed_bio_resolver(&lamp, &scenario).unwrap(),
            sealed_fast_resolver(&charge, &scenario).unwrap(),
            scenario.steps_per_day,
            scenario.bio_dt,
            scenario.cabin_dt,
            Some(sealed_reset_hook(&scenario)),
        )
        .unwrap()
    };
    // Save one day before the boundary; resume two days past it (the reset fires in between).
    assert_resume_parity(build, boundary - 1, boundary + 2);
}
