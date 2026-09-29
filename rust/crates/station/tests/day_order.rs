//! Controls for the lab-only day order (`station::driver::TwoRate`),
//! `docs/plans/post-roadmap-intraday-gas-exchange.md` §3.
//!
//! The lab measures the interleaved order against the reference. These tests are what make
//! those measurements mean anything:
//!
//! 1. the slow-first path IS the reference day, bit for bit (else the lab measures a copy);
//! 2. with no plants, interleaving changes nothing (the cabin seeing the step counter move
//!    four times a day is measured, not assumed harmless);
//! 3. a scenario whose two sides share no stock cannot tell the orders apart;
//! 4. on a scenario whose sides do share a stock, the orders differ (not a no-op);
//! 5. an uneven split is refused.
//!
//! States are compared by their hex-float snapshot, so equal means bit-identical.

use domains::crew::FECAL_WASTE;
use domains::params;
use simcore::error::SimError;
use simcore::integrator::EulerIntegrator;
use simcore::snapshot::from_engine;
use simcore::state::State;
use station::driver::{run_master_day, DayOrder, Side, TwoRate};
use station::greenhouse::{build_greenhouse, greenhouse_bio_resolver, greenhouse_cabin_resolver};
use station::harvest::{build_harvest, harvest_bio_resolver, harvest_cabin_resolver};
use station::lighting::{build_lighting, lighting_bio_resolver, lighting_power_resolver};
use station::params as station_params;
use station::scenario::{
    greenhouse_scenario, harvest_scenario, lighting_scenario, sealed_station_scenario,
};
use station::sealed::{build_sealed_station, sealed_bio_resolver, sealed_fast_resolver};

fn snap(state: &State) -> String {
    from_engine(state).to_json()
}

fn ignore(_: Side, _: &State, _: &State) {}

/// The greenhouse, run `days` both by the reference runner and by `TwoRate` in `order`.
/// Returns `(reference final, lab final, reference rationed, lab totals)`.
fn greenhouse_both(order: DayOrder, with_plants: bool) -> (String, String, u64, u64) {
    let crew = params::crew();
    let eclss = params::eclss();
    let scenario = greenhouse_scenario();
    let build = || build_greenhouse(&crew, &eclss, &scenario, with_plants, FECAL_WASTE).unwrap();
    let bio_res = greenhouse_bio_resolver(&scenario).unwrap();
    let cabin_res = greenhouse_cabin_resolver(&scenario).unwrap();

    let (s0, bio, cabin) = build();
    let (ref_states, ref_rationed, ref_events) = run_master_day(
        &EulerIntegrator::new(bio),
        &EulerIntegrator::new(cabin),
        s0,
        &bio_res,
        &cabin_res,
        scenario.days,
        scenario.steps_per_day,
        scenario.bio_steps_per_day,
        scenario.bio_dt,
        scenario.cabin_dt,
        None,
    )
    .unwrap();

    let (s0, bio, cabin) = build();
    let (slow, fast) = (EulerIntegrator::new(bio), EulerIntegrator::new(cabin));
    let two = TwoRate {
        slow: &slow,
        fast: &fast,
        slow_resolver: &bio_res,
        fast_resolver: &cabin_res,
        steps_per_day: scenario.steps_per_day,
        slow_steps_per_day: scenario.bio_steps_per_day,
        slow_dt: scenario.bio_dt,
        fast_dt: scenario.cabin_dt,
        slow_reset: None,
    };
    let (states, totals) = two.run(order, s0, scenario.days, &mut ignore).unwrap();
    assert_eq!(states.len(), ref_states.len(), "one state per master day");
    assert_eq!(totals.events, ref_events, "events");
    (
        snap(ref_states.last().unwrap()),
        snap(states.last().unwrap()),
        ref_rationed,
        totals.slow_rationed + totals.fast_rationed,
    )
}

#[test]
fn slow_first_is_the_reference_day_bit_for_bit_on_the_greenhouse() {
    let (reference, lab, ref_rationed, lab_rationed) = greenhouse_both(DayOrder::SlowFirst, true);
    assert_eq!(lab, reference);
    assert_eq!(lab_rationed, ref_rationed);
}

#[test]
fn slow_first_is_the_reference_day_bit_for_bit_on_the_harvest_ring() {
    let crew = params::crew();
    let eclss = params::eclss();
    let hp = station_params::harvest();
    let scenario = harvest_scenario();
    let gh = &scenario.greenhouse;
    let bio_res = harvest_bio_resolver(&scenario).unwrap();
    let cabin_res = harvest_cabin_resolver(&scenario).unwrap();
    let build = || build_harvest(&crew, &eclss, &hp, &scenario, true, true).unwrap();

    let (s0, bio, cabin) = build();
    let (ref_states, ref_rationed, _) = run_master_day(
        &EulerIntegrator::new(bio),
        &EulerIntegrator::new(cabin),
        s0,
        &bio_res,
        &cabin_res,
        gh.days,
        gh.steps_per_day,
        gh.bio_steps_per_day,
        gh.bio_dt,
        gh.cabin_dt,
        None,
    )
    .unwrap();

    let (s0, bio, cabin) = build();
    let (slow, fast) = (EulerIntegrator::new(bio), EulerIntegrator::new(cabin));
    let two = TwoRate {
        slow: &slow,
        fast: &fast,
        slow_resolver: &bio_res,
        fast_resolver: &cabin_res,
        steps_per_day: gh.steps_per_day,
        slow_steps_per_day: gh.bio_steps_per_day,
        slow_dt: gh.bio_dt,
        fast_dt: gh.cabin_dt,
        slow_reset: None,
    };
    let (states, totals) = two
        .run(DayOrder::SlowFirst, s0, gh.days, &mut ignore)
        .unwrap();
    assert_eq!(
        snap(states.last().unwrap()),
        snap(ref_states.last().unwrap())
    );
    assert_eq!(totals.slow_rationed + totals.fast_rationed, ref_rationed);
}

/// The sealed station for 3 days with a re-sow hook that fires at the start of day 2 and
/// adopts the state it is handed. The real hook cannot fire inside 3 days (it needs a
/// season's seed bank); this one exercises the adopt branch and its observer call in both
/// runners. The full horizon with the real hook is compared in the lab example.
#[test]
fn slow_first_is_the_reference_day_bit_for_bit_across_a_resow() {
    let charge = params::charge();
    let thermal = params::thermal();
    let crew = params::crew();
    let eclss = params::eclss();
    let recovery = station_params::water_recovery();
    let lamp = station_params::lamp();
    let hp = station_params::harvest();
    let scenario = sealed_station_scenario();
    let days = 3;
    let fire_at = 2 * scenario.bio_steps_per_day;
    let hook = move |n: u64, s: &State| -> Result<Option<State>, SimError> {
        Ok((n == fire_at).then(|| s.clone()))
    };
    let build = || {
        build_sealed_station(
            &charge, &thermal, &crew, &eclss, &recovery, &lamp, &hp, &scenario, false, false,
        )
        .unwrap()
    };
    let bio_res = sealed_bio_resolver(&lamp, &scenario).unwrap();
    let fast_res = sealed_fast_resolver(&charge, &scenario).unwrap();

    let (s0, bio, fast) = build();
    let (ref_states, ref_rationed, _) = run_master_day(
        &EulerIntegrator::new(bio),
        &EulerIntegrator::new(fast),
        s0,
        &bio_res,
        &fast_res,
        days,
        scenario.steps_per_day,
        scenario.bio_steps_per_day,
        scenario.bio_dt,
        scenario.cabin_dt,
        Some(&hook),
    )
    .unwrap();

    let (s0, bio, fast) = build();
    let (slow, fast) = (EulerIntegrator::new(bio), EulerIntegrator::new(fast));
    let two = TwoRate {
        slow: &slow,
        fast: &fast,
        slow_resolver: &bio_res,
        fast_resolver: &fast_res,
        steps_per_day: scenario.steps_per_day,
        slow_steps_per_day: scenario.bio_steps_per_day,
        slow_dt: scenario.bio_dt,
        fast_dt: scenario.cabin_dt,
        slow_reset: Some(&hook),
    };
    let mut seen = [0u64; 3];
    let mut count = |side: Side, _: &State, _: &State| {
        seen[side as usize] += 1;
    };
    let (states, totals) = two.run(DayOrder::SlowFirst, s0, days, &mut count).unwrap();
    assert_eq!(
        snap(states.last().unwrap()),
        snap(ref_states.last().unwrap())
    );
    assert_eq!(totals.slow_rationed + totals.fast_rationed, ref_rationed);
    // Every step was observed exactly once, and the re-sow once.
    assert_eq!(seen[Side::Reset as usize], 1, "re-sow observed once");
    assert_eq!(
        seen[Side::Slow as usize],
        days as u64 * scenario.bio_steps_per_day
    );
    assert_eq!(
        seen[Side::Fast as usize],
        days as u64 * scenario.steps_per_day
    );
}

#[test]
fn interleaving_without_plants_changes_nothing() {
    // The cabin now sees the step counter move four times a day. With no plant flows the
    // slow side only advances the counter, so any difference is the cabin reading it.
    let (reference, lab, _, _) = greenhouse_both(DayOrder::Interleaved, false);
    assert_eq!(lab, reference);
}

#[test]
fn interleaving_moves_a_scenario_whose_sides_share_a_stock() {
    // Control on the control above: the order is not a no-op where the crop and the cabin
    // share the air.
    let (reference, lab, _, _) = greenhouse_both(DayOrder::Interleaved, true);
    assert_ne!(lab, reference);
}

#[test]
fn interleaving_cannot_move_a_scenario_whose_sides_share_no_stock() {
    // The lighting seam: lamp (Power) and crop share no stock, only the lamp schedule, a
    // forcing. Each stock is written by one side only, so the order cannot reach the bits.
    let lamp = station_params::lamp();
    let scenario = lighting_scenario();
    let bio_res = lighting_bio_resolver(&lamp, &scenario, true).unwrap();
    let power_res = lighting_power_resolver(&scenario).unwrap();
    let run = |order: DayOrder| {
        let (s0, bio, power) = build_lighting(&lamp, &scenario, true).unwrap();
        let (slow, fast) = (EulerIntegrator::new(bio), EulerIntegrator::new(power));
        let two = TwoRate {
            slow: &slow,
            fast: &fast,
            slow_resolver: &bio_res,
            fast_resolver: &power_res,
            steps_per_day: scenario.steps_per_day,
            slow_steps_per_day: scenario.bio_steps_per_day,
            slow_dt: scenario.bio_dt,
            fast_dt: scenario.power_dt,
            slow_reset: None,
        };
        let (states, _) = two.run(order, s0, scenario.days, &mut ignore).unwrap();
        snap(states.last().unwrap())
    };
    assert_eq!(run(DayOrder::Interleaved), run(DayOrder::SlowFirst));
}

#[test]
fn an_uneven_split_is_refused_and_the_reference_guards_hold() {
    let crew = params::crew();
    let eclss = params::eclss();
    let scenario = greenhouse_scenario();
    let (_, bio, cabin) = build_greenhouse(&crew, &eclss, &scenario, true, FECAL_WASTE).unwrap();
    let bio_res = greenhouse_bio_resolver(&scenario).unwrap();
    let cabin_res = greenhouse_cabin_resolver(&scenario).unwrap();
    let (slow, fast) = (EulerIntegrator::new(bio), EulerIntegrator::new(cabin));
    let with = |steps_per_day: u64, slow_steps_per_day: u64, slow_dt: f64, fast_dt: f64| TwoRate {
        slow: &slow,
        fast: &fast,
        slow_resolver: &bio_res,
        fast_resolver: &cabin_res,
        steps_per_day,
        slow_steps_per_day,
        slow_dt,
        fast_dt,
        slow_reset: None,
    };
    // 1440 minutes over 7 slow steps does not split evenly: interleaved refuses, the
    // reference order (which needs no split) does not.
    let uneven = with(1440, 7, 1.0 / 7.0, 60.0);
    let msg = uneven
        .validate(DayOrder::Interleaved)
        .unwrap_err()
        .to_string();
    assert!(msg.contains("divide evenly"), "{msg}");
    assert!(uneven.validate(DayOrder::SlowFirst).is_ok());
    // advance_day refuses on its own, not only through run().
    let (s0, _, _) = build_greenhouse(&crew, &eclss, &scenario, true, FECAL_WASTE).unwrap();
    let mut totals = Default::default();
    assert!(uneven
        .advance_day(DayOrder::Interleaved, &s0, &mut totals, &mut ignore)
        .is_err());
    // The reference runner's two day-length guards, both orders.
    for order in [DayOrder::SlowFirst, DayOrder::Interleaved] {
        assert!(
            with(1440, 4, 0.25, 30.0).validate(order).is_err(),
            "fast day"
        );
        assert!(
            with(1440, 4, 0.5, 60.0).validate(order).is_err(),
            "slow day"
        );
        assert!(with(1440, 4, 0.25, 60.0).validate(order).is_ok());
    }
}
