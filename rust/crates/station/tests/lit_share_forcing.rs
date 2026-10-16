//! **The lamp's lit share** — the `lit_fraction` forcing every lamp-lit build supplies for the lab
//! lit-share night rule (`docs/plans/post-roadmap-lit-share-stomata.md` §1a, predictions L2–L3).
//!
//! It is the lamp's own top-hat at unit light, so on every plant step the crop's net radiation is
//! the lit share times the lamp-on value, and dark exactly when the share is 0.

use domains::biosphere::stocks::{LIT_FRACTION_VAR, RN_VAR};
use simcore::environment::{Environment, SourceResolver};
use simcore::state::State;
use station::lighting::{lamp_net_radiation, lamp_par, lighting_bio_resolver};
use station::scenario::{lighting_scenario, sealed_station_scenario, Phase};
use station::sealed::{build_sealed_station, sealed_bio_resolver};
use station::sowing::program_value;

const STEPS: u64 = domains::biosphere::STEPS_PER_DAY as u64;
const DT: f64 = 1.0 / STEPS as f64;

fn state_at(n: u64) -> State {
    let (s, _, _) = build_sealed_station(
        &domains::params::charge(),
        &domains::params::thermal(),
        &domains::params::crew(),
        &domains::params::eclss(),
        &station::params::water_recovery(),
        &station::params::lamp(),
        &station::params::harvest(),
        &sealed_station_scenario(),
        false,
        false,
    )
    .unwrap();
    State::new(n, s.stocks, s.rng_seed, s.aux).unwrap()
}

/// One day's (φ, Rn) by plant step, through `read`.
fn day(first: u64, read: &dyn Fn(&str, u64) -> f64) -> Vec<(f64, f64)> {
    (first..first + STEPS)
        .map(|n| (read(LIT_FRACTION_VAR, n), read(RN_VAR, n)))
        .collect()
}

/// L2 on one day: `Rn == φ · rn_on` (to rounding) and `Rn == 0 ⇔ φ == 0`; returns the partly
/// lit steps' shares.
fn check_day(steps: &[(f64, f64)], rn_on: f64, what: &str) -> Vec<f64> {
    let mut partial = Vec::new();
    for (i, &(phi, rn)) in steps.iter().enumerate() {
        assert!((0.0..=1.0).contains(&phi), "{what} step {i}: φ {phi}");
        assert_eq!(rn == 0.0, phi == 0.0, "{what} step {i}: Rn {rn}, φ {phi}");
        assert!(
            (rn - phi * rn_on).abs() <= 1e-12 * rn_on,
            "{what} step {i}: Rn {rn} against φ·Rn_on {}",
            phi * rn_on
        );
        if phi > 0.0 && phi < 1.0 {
            partial.push(phi);
        }
    }
    partial
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-12
}

/// The sealed station, read as the sowing clock reads it (the phase's twin), on a cold day and a
/// warm one: the warm 16-h day has two steps lit a third (03:00–04:30, 19:30–21:00), the cold
/// 8-h day two lit two thirds (07:30–09:00, 15:00–16:30) — L3.
#[test]
fn the_sealed_stations_lit_share_is_its_light_in_both_phases() {
    let scn = sealed_station_scenario();
    assert_eq!(
        (scn.photoperiod_hours, scn.cold.photoperiod_hours),
        (16, 8),
        "the hand shares below assume a 16-h warm day and an 8-h cold one"
    );
    let lamp = station::params::lamp();
    let r = sealed_bio_resolver(&lamp, &scn).unwrap();
    let read = |var: &str, n: u64| program_value(&r, var, &state_at(n), &scn, DT).unwrap();
    for (d, phase, on_par, want) in [
        (40, Phase::Cold, scn.cold.par, 2.0 / 3.0),
        (
            150,
            Phase::Warm,
            scn.full_lamp_par(lamp.photon_efficacy),
            1.0 / 3.0,
        ),
    ] {
        assert_eq!(scn.phase(&state_at(d * STEPS + STEPS / 2)).unwrap(), phase);
        let partial = check_day(
            &day(d * STEPS, &read),
            lamp_net_radiation(on_par),
            &format!("{phase:?}"),
        );
        assert_eq!(partial.len(), 2, "{phase:?}: partly lit steps {partial:?}");
        assert!(
            partial.iter().all(|&p| close(p, want)),
            "{phase:?}: {partial:?} against {want}"
        );
    }
}

/// The lighting build, lamp on and off: the same identity, and no lamp means no lit share.
#[test]
fn the_lighting_builds_lit_share_is_its_light() {
    let lamp = station::params::lamp();
    let scn = lighting_scenario();
    assert_eq!(
        scn.photoperiod_hours, 16,
        "the hand share assumes a 16-h day"
    );
    let on: SourceResolver = lighting_bio_resolver(&lamp, &scn, true).unwrap();
    let read_on = |var: &str, n: u64| on.bind(&state_at(n), DT).get(var).unwrap();
    let partial = check_day(
        &day(3 * STEPS, &read_on),
        lamp_net_radiation(lamp_par(&lamp, &scn)),
        "lighting",
    );
    assert_eq!(partial.len(), 2, "partly lit steps {partial:?}");
    assert!(partial.iter().all(|&p| close(p, 1.0 / 3.0)), "{partial:?}");
    let off = lighting_bio_resolver(&lamp, &scn, false).unwrap();
    for n in 3 * STEPS..4 * STEPS {
        let st = state_at(n);
        let env = off.bind(&st, DT);
        assert_eq!(env.get(LIT_FRACTION_VAR).unwrap(), 0.0);
        assert_eq!(env.get(RN_VAR).unwrap(), 0.0);
    }
}
