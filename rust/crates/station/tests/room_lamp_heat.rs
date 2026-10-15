//! **The room's lamp heat switched with the light** — the lab switch's own checks.
//!
//! Plan: `docs/plans/post-roadmap-room-lamp-heat.md` (§4 L5). LAB-ONLY: the reference's fast-side
//! `lamp_power` is the daily average; `station::sealed::with_switched_lamp_heat` swaps in the lamp's
//! draw on the plant step's own top-hat. These tests hold the swap to the plants' light: the same
//! window, the same daily energy.

use domains::biosphere::stocks::PAR_VAR;
use simcore::environment::{Environment, SourceResolver};
use simcore::state::State;
use station::flows::LAMP_POWER_VAR;
use station::scenario::{sealed_station_scenario, Phase, SealedStationScenario};
use station::sealed::{
    build_sealed_station, sealed_bio_resolver, sealed_fast_resolver, with_switched_lamp_heat,
};
use station::sowing::twin;

fn setup() -> (SealedStationScenario, State, SourceResolver, SourceResolver) {
    let scenario = sealed_station_scenario();
    let charge = domains::params::charge();
    let lamp = station::params::lamp();
    let (state, _, _) = build_sealed_station(
        &charge,
        &domains::params::thermal(),
        &domains::params::crew(),
        &domains::params::eclss(),
        &station::params::water_recovery(),
        &lamp,
        &station::params::harvest(),
        &scenario,
        true,
        true,
    )
    .expect("build");
    let bio = sealed_bio_resolver(&lamp, &scenario).expect("bio resolver");
    let fast = with_switched_lamp_heat(
        sealed_fast_resolver(&charge, &scenario).expect("fast resolver"),
        &scenario,
    )
    .expect("switched");
    (scenario, state, bio, fast)
}

fn at(state: &State, n: u64) -> State {
    State::new(n, state.stocks.clone(), state.rng_seed, state.aux.clone()).expect("state")
}

/// L5, the window: the fast minutes after plant step `k` (run with `n = k + 1`) draw the lamp's
/// on-power times the lit share of plant window `k` — the share the plants' PAR has there.
#[test]
fn the_lamp_draws_on_the_plants_own_window() {
    let (scenario, state, bio, fast) = setup();
    let efficacy = station::params::lamp().photon_efficacy;
    let spd = scenario.bio_steps_per_day;
    for phase in Phase::BOTH {
        let on_w = scenario.lamp_power(phase, efficacy);
        let on_par = scenario.lamp_par(phase, efficacy);
        let (mut lit, mut dark) = (0, 0);
        // Two days, from day 3, so the window `n − 1` crosses a day boundary.
        for k in 3 * spd..5 * spd {
            let share_par = bio
                .bind(&at(&state, k), scenario.bio_dt)
                .get(&twin(PAR_VAR, phase))
                .expect("par")
                / on_par;
            let share_w = fast
                .bind(&at(&state, k + 1), scenario.cabin_dt)
                .get(&twin(LAMP_POWER_VAR, phase))
                .expect("lamp power")
                / on_w;
            assert!(
                (share_w - share_par).abs() <= 1e-12,
                "{phase:?} window {k}: lamp {share_w} against light {share_par}"
            );
            if share_par == 0.0 {
                dark += 1;
            } else {
                lit += 1;
            }
        }
        assert!(lit > 0 && dark > 0, "{phase:?}: {lit} lit, {dark} dark");
    }
}

/// L5, the dose: one day of the switched draw is the daily average's energy, to rounding, in both
/// phases — the switch moves the lamp's heat within the day, never the day's total.
#[test]
fn a_day_of_the_switched_draw_is_the_averages_energy() {
    let (scenario, state, _, fast) = setup();
    let efficacy = station::params::lamp().photon_efficacy;
    let spd = scenario.bio_steps_per_day;
    let window_s = scenario.bio_dt * 86_400.0;
    for phase in Phase::BOTH {
        let day: f64 = (1..=spd)
            .map(|n| {
                fast.bind(&at(&state, 10 * spd + n), scenario.cabin_dt)
                    .get(&twin(LAMP_POWER_VAR, phase))
                    .expect("lamp power")
                    * window_s
            })
            .sum();
        let want = scenario.lamp_average_power(phase, efficacy) * 86_400.0;
        assert!(
            ((day - want) / want).abs() <= 1e-12,
            "{phase:?}: a day's draw {day} J against the average's {want} J"
        );
    }
}

/// The swap refuses a resolver that does not carry the twins (a plain or already-wrapped one),
/// rather than adding a second lamp schedule beside it.
#[test]
fn the_switch_refuses_a_resolver_without_the_twins() {
    let scenario = sealed_station_scenario();
    let empty = SourceResolver::new(Default::default(), Default::default()).expect("empty");
    assert!(with_switched_lamp_heat(empty, &scenario).is_err());
}

/// The lighting seam's power hours come in groups of 3 over 2 plant windows; `n` cannot tell the
/// hours apart, so the switched draw is the group's mean. Its energy over each group equals the
/// plants' light over the group's two windows, and a day's equals the average's.
#[test]
fn the_lighting_seams_groups_draw_the_light_of_their_windows() {
    use station::lighting::{
        build_lighting, lamp_average_power, lamp_par, lighting_bio_resolver,
        lighting_power_resolver_switched,
    };
    let scenario = station::scenario::lighting_scenario();
    let lamp = station::params::lamp();
    let (state, _, _) = build_lighting(&lamp, &scenario, true).expect("build");
    let bio = lighting_bio_resolver(&lamp, &scenario, true).expect("bio");
    let power = lighting_power_resolver_switched(&scenario).expect("power");
    let groups =
        station::driver::day_groups(scenario.steps_per_day, scenario.bio_steps_per_day).unwrap();
    assert_eq!(
        (groups.slow, groups.fast),
        (2, 3),
        "the grouping this test documents moved"
    );
    let on_par = lamp_par(&lamp, &scenario);
    let window_s = scenario.bio_dt * 86_400.0;
    let spd = scenario.bio_steps_per_day;
    let mut day = 0.0;
    for g in 0..groups.count {
        let n = 2 * spd + (g + 1) * groups.slow;
        let drawn = power
            .bind(&at(&state, n), scenario.power_dt)
            .get(LAMP_POWER_VAR)
            .unwrap()
            * scenario.power_dt
            * groups.fast as f64;
        let light: f64 = (n - groups.slow..n)
            .map(|k| {
                bio.bind(&at(&state, k), scenario.bio_dt)
                    .get(PAR_VAR)
                    .unwrap()
                    / on_par
            })
            .sum::<f64>()
            * scenario.lamp_power_w
            * window_s;
        assert!(
            (drawn - light).abs() <= 1e-9 * scenario.lamp_power_w * window_s,
            "group {g}: lamp {drawn} J against light {light} J"
        );
        day += drawn;
    }
    let want = lamp_average_power(&scenario) * 86_400.0;
    assert!(((day - want) / want).abs() <= 1e-12, "{day} vs {want}");
}
