//! The lamp-lit crop's net radiation is the LAMP's (`docs/plans/post-roadmap-room-temperature.md`
//! §21, the user's ruling of 2026-10-05). Before, every lamp-lit build handed transpiration the
//! weather file's OUTDOOR net radiation: a sealed crop lost more water on sunny days outside, and
//! a dark lamp left its water loss untouched. Conservation could not see it.
//!
//! The form: `(1 − 0.23) × lamp PAR / 4.57` while lit (FAO-56's albedo on the lamp's radiant
//! PAR, McCree's photon energy), 0 in the dark.

use domains::biosphere::stocks::RN_VAR;
use domains::biosphere::system::weather_forcings;
use simcore::environment::{Environment, SourceResolver};
use simcore::state::State;
use station::greenhouse::greenhouse_bio_resolver;
use station::lighting::{lamp_par, lighting_bio_resolver};
use station::perturbations::with_lighting_failure;
use station::scenario::{greenhouse_scenario, lighting_scenario, sealed_station_scenario, Phase};
use station::sealed::{build_sealed_station, sealed_bio_resolver, sealed_fast_resolver};
use station::sowing::program_value;

const STEPS: u64 = domains::biosphere::STEPS_PER_DAY as u64;
/// The plant window at midday of `day` (lit) and the one at midnight (dark).
fn midday(day: u64) -> u64 {
    day * STEPS + STEPS / 2
}
fn midnight(day: u64) -> u64 {
    day * STEPS
}

/// The 500 µmol m⁻² s⁻¹ lamp, by hand: 0.77 × 500 / 4.57 W m⁻².
const LAMP_RN: f64 = 0.77 * 500.0 / 4.57;
/// The cold phase's dimmed 100 µmol m⁻² s⁻¹ lamp (slice 3b, §24l C5): 0.77 × 100 / 4.57.
const DIM_RN: f64 = 0.77 * 100.0 / 4.57;

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

fn rn(r: &SourceResolver, n: u64) -> f64 {
    let dt = 1.0 / STEPS as f64;
    r.bind(&state_at(n), dt).get(RN_VAR).unwrap()
}

/// A sealed resolver's net radiation on step `n`, read as the sowing clock reads it: the twin
/// of the phase of a crop sown at step 0 (slice 4 — the plain name is in no sealed resolver).
fn sealed_rn(r: &SourceResolver, n: u64) -> f64 {
    let dt = 1.0 / STEPS as f64;
    program_value(r, RN_VAR, &state_at(n), &sealed_station_scenario(), dt).unwrap()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-12 * b.abs()
}

#[test]
fn the_sealed_crops_net_radiation_is_the_lamps_lit_and_zero_dark() {
    let scn = sealed_station_scenario();
    let r = sealed_bio_resolver(&station::params::lamp(), &scn).unwrap();
    let outdoor = weather_forcings(&scn.bio, scn.years).unwrap();
    let dt = 1.0 / STEPS as f64;
    let mut differs = 0;
    // Days 0 and 40 are in the cold period (the dimmed lamp, an 8 h day still lit at midday);
    // 150 and 250 under the full lamp. Both phases asserted.
    assert_eq!(scn.phase(&state_at(midday(40))).unwrap(), Phase::Cold);
    assert_eq!(scn.phase(&state_at(midday(150))).unwrap(), Phase::Warm);
    for (day, lamp_rn) in [(0, DIM_RN), (40, DIM_RN), (150, LAMP_RN), (250, LAMP_RN)] {
        let lit = sealed_rn(&r, midday(day));
        assert!(close(lit, lamp_rn), "day {day}: lit {lit}, lamp {lamp_rn}");
        assert_eq!(
            sealed_rn(&r, midnight(day)),
            0.0,
            "day {day}: the dark window is lit"
        );
        differs += u32::from(outdoor[RN_VAR](midday(day), dt) != lit);
    }
    assert!(
        differs > 0,
        "the weather's value equals the lamp's: the pin proves nothing"
    );
}

#[test]
fn the_lighting_scenarios_net_radiation_is_its_lamps() {
    let lamp = station::params::lamp();
    let scn = lighting_scenario();
    assert_eq!(
        lamp_par(&lamp, &scn),
        500.0,
        "the hand value assumes the 500 µmol lamp"
    );
    let r = lighting_bio_resolver(&lamp, &scn, true).unwrap();
    assert!(close(rn(&r, midday(3)), LAMP_RN));
    assert_eq!(rn(&r, midnight(3)), 0.0);
    // The lamp off is dark, light and radiation both.
    let off = lighting_bio_resolver(&lamp, &scn, false).unwrap();
    assert_eq!(rn(&off, midday(3)), 0.0);
}

#[test]
fn the_sunlit_greenhouse_keeps_the_weathers_net_radiation() {
    let scn = greenhouse_scenario();
    let r = greenhouse_bio_resolver(&scn).unwrap();
    let outdoor = weather_forcings(&scn.bio, 1).unwrap();
    let dt = 1.0 / STEPS as f64;
    for n in [midday(2), midnight(2)] {
        assert_eq!(rn(&r, n).to_bits(), outdoor[RN_VAR](n, dt).to_bits());
    }
}

#[test]
fn a_lighting_failure_darkens_the_crops_net_radiation_too() {
    let scn = sealed_station_scenario();
    let bio = sealed_bio_resolver(&station::params::lamp(), &scn).unwrap();
    let fast = sealed_fast_resolver(&domains::params::charge(), &scn).unwrap();
    let (bio, _) = with_lighting_failure(bio, fast, 2 * STEPS, 5 * STEPS).unwrap();
    assert_eq!(sealed_rn(&bio, midday(3)), 0.0, "the failed lamp still radiates");
    // Day 6 is in the cold period: the lamp comes back DIMMED (slice 3b).
    assert!(
        close(sealed_rn(&bio, midday(6)), DIM_RN),
        "the lamp did not come back"
    );
}
