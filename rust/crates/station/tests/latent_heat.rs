//! **The heat evaporation takes from the room** — its own checks.
//!
//! Plan: `docs/plans/post-roadmap-latent-heat-books.md` (§1, §4 Q1, §8). Every sealed build records each
//! plant window's evaporation and draws λ × it from the chamber over the window's minutes (built lab-only,
//! adopted 2026-10-15); `station::latent_heat::without_latent_heat` is the lab switch back. These tests
//! hold the drawn heat to the moved water, window by window, and hold the switch and the refusals.

use domains::biosphere::science::LATENT_HEAT_VAPORIZATION as LAMBDA;
use domains::biosphere::stocks::SOIL_WATER;
use domains::biosphere::system::weather_shared;
use simcore::flow::FlowResult;
use simcore::integrator::EulerIntegrator;
use simcore::registry::Registry;
use simcore::state::State;
use station::air_split::water_on_fast_step;
use station::chamber::CHAMBER;
use station::driver::{DayOrder, Side, TwoRate};
use station::latent_heat::{
    without_latent_heat, CHAMBER_LATENT_HEAT, EVAPORATED_TOTAL_AUX, EVAPORATION_RATE_AUX,
    EVAPORATION_RECORDER, TRANSPIRATION,
};
use station::scenario::sealed_station_scenario;
use station::sealed::{
    build_sealed_station, sealed_bio_resolver, sealed_fast_resolver, sealed_reset_hook,
};

fn build() -> (State, Registry, Registry) {
    build_sealed_station(
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
    .expect("build")
}

fn leg_on(r: &FlowResult, stock: &str) -> f64 {
    r.legs
        .iter()
        .filter(|l| l.stock == stock)
        .map(|l| l.amount)
        .sum()
}

/// Q1: over every window, the heat the latent flow draws from the chamber is λ × the water the
/// window's transpiration flow takes off the root zone — the same flow, evaluated on the plant
/// step's own start. Four cold days (the soil evaporates under the dim lamp) and the first warm
/// days would both do; the cold period's are cheap.
#[test]
fn the_heat_drawn_over_a_window_is_lambda_times_its_water() {
    let scenario = sealed_station_scenario();
    let lamp = station::params::lamp();
    let (state, bio, fast) = build();
    let (bio, fast) = (EulerIntegrator::new(bio), EulerIntegrator::new(fast));
    let bio_r = sealed_bio_resolver(&lamp, &scenario).expect("bio resolver");
    let fast_r = sealed_fast_resolver(&domains::params::charge(), &domains::params::crew(), &scenario).expect("fast");
    let reset = sealed_reset_hook(&scenario);
    let two = TwoRate {
        slow: &bio,
        fast: &fast,
        slow_resolver: &bio_r,
        fast_resolver: &fast_r,
        steps_per_day: scenario.steps_per_day,
        slow_steps_per_day: scenario.bio_steps_per_day,
        slow_dt: scenario.bio_dt,
        fast_dt: scenario.cabin_dt,
        slow_reset: Some(&*reset),
    };
    // (water moved, heat drawn) per window.
    let mut windows: Vec<(f64, f64)> = Vec::new();
    let mut observe = |side: Side, before: &State, _after: &State| match side {
        Side::Reset => {}
        Side::Slow => {
            let env = bio_r.bind(before, scenario.bio_dt);
            let f = bio
                .registry()
                .flows()
                .iter()
                .find(|f| f.id() == TRANSPIRATION)
                .expect("transpiration on the plant step");
            let r = f.evaluate(before, &env, scenario.bio_dt).expect("evaluate");
            windows.push((-leg_on(&r, SOIL_WATER), 0.0));
        }
        Side::Fast => {
            let env = fast_r.bind(before, scenario.cabin_dt);
            let f = fast
                .registry()
                .flows()
                .iter()
                .find(|f| f.id() == CHAMBER_LATENT_HEAT)
                .expect("the latent flow on the fast step");
            let r = f.evaluate(before, &env, scenario.cabin_dt).expect("evaluate");
            windows.last_mut().expect("a plant step opens the window").1 -= leg_on(&r, CHAMBER);
        }
    };
    let (states, totals) = two
        .run(DayOrder::Interleaved, state, 4, &mut observe)
        .expect("run");
    assert_eq!((totals.slow_rationed, totals.fast_rationed), (0, 0));
    assert_eq!(windows.len() as u64, 4 * scenario.bio_steps_per_day);
    let mut wet = 0;
    for (k, (kg, heat)) in windows.iter().enumerate() {
        let want = LAMBDA * kg;
        assert!(
            (heat - want).abs() <= 1e-12 * want.max(1.0),
            "window {k}: drew {heat} J for {kg} kg (λE {want} J)"
        );
        if *kg > 0.0 {
            wet += 1;
        }
    }
    assert!(wet > 0, "no window evaporated: the identity was never tested");
    // The ledger: at a day end every recorded window's heat has been drawn, so the heat booked to
    // date is λ × the running total.
    let drawn: f64 = windows.iter().map(|(_, h)| h).sum();
    let ledger = LAMBDA * states.last().unwrap().aux[EVAPORATED_TOTAL_AUX];
    assert!(
        (drawn - ledger).abs() <= 1e-12 * ledger,
        "drew {drawn} J; the ledger says {ledger} J"
    );
}

/// The reference build carries the books — one recorder, one fast flow, the rate seeded, transpiration
/// on the plant step once under its own type name — and the lab switch takes exactly those out, and
/// refuses a build that does not carry them.
#[test]
fn the_build_carries_the_books_and_the_switch_takes_exactly_them_out() {
    let (state, bio, fast) = build();
    let count = |reg: &Registry, id: &str| reg.flows().iter().filter(|f| f.id() == id).count();
    let recorders = |reg: &Registry| {
        reg.aux_processes()
            .iter()
            .filter(|a| a.id() == EVAPORATION_RECORDER)
            .count()
    };
    assert_eq!(count(&fast, CHAMBER_LATENT_HEAT), 1);
    assert_eq!(recorders(&bio), 1);
    assert_eq!(state.aux[EVAPORATION_RATE_AUX], 0.0);
    assert_eq!(state.aux[EVAPORATED_TOTAL_AUX], 0.0);
    assert_eq!(count(&bio, TRANSPIRATION), 1);
    assert_eq!(count(&fast, TRANSPIRATION), 0);
    let t = bio.flows().iter().find(|f| f.id() == TRANSPIRATION).unwrap();
    assert_eq!(t.type_name(), "Transpiration");
    let (n_bio, n_fast, n_aux) = (bio.flows().len(), fast.flows().len(), bio.aux_processes().len());

    let (state, bio, fast) = without_latent_heat(state, bio, fast).expect("off");
    assert_eq!(bio.flows().len(), n_bio);
    assert_eq!(fast.flows().len(), n_fast - 1);
    assert_eq!(bio.aux_processes().len(), n_aux - 1);
    assert_eq!(count(&fast, CHAMBER_LATENT_HEAT), 0);
    assert_eq!(recorders(&bio), 0);
    assert!(!state.aux.contains_key(EVAPORATION_RATE_AUX));
    assert!(!state.aux.contains_key(EVAPORATED_TOTAL_AUX));
    assert!(without_latent_heat(state, bio, fast).is_err(), "switched off twice");
}

/// The water moved onto the minute (the lab's `water_on_fast_step`) refuses a build carrying the books —
/// the recorder would read a plant step the water had left — and moves once they are switched off (the
/// minute-water option opts out, §8).
#[test]
fn water_on_the_minute_needs_the_books_switched_off() {
    let shared = weather_shared(&sealed_station_scenario().bio);

    let (state, bio, fast) = build();
    assert!(water_on_fast_step(&state.stocks, bio, fast, &shared).is_err());

    let (state, bio, fast) = build();
    let (state, bio, fast) = without_latent_heat(state, bio, fast).expect("off");
    assert!(water_on_fast_step(&state.stocks, bio, fast, &shared).is_ok());
}

/// A save from before the books (no rate key) resumes at a group boundary: the plant step that opens
/// the window records the rate before any minute reads it (§8a).
#[test]
fn a_save_without_the_rate_key_resumes_at_a_group_boundary() {
    let scenario = sealed_station_scenario();
    let lamp = station::params::lamp();
    let (mut state, bio, fast) = build();
    state.aux.remove(EVAPORATION_RATE_AUX);
    state.aux.remove(EVAPORATED_TOTAL_AUX);
    let (bio, fast) = (EulerIntegrator::new(bio), EulerIntegrator::new(fast));
    let bio_r = sealed_bio_resolver(&lamp, &scenario).expect("bio resolver");
    let fast_r = sealed_fast_resolver(&domains::params::charge(), &domains::params::crew(), &scenario).expect("fast");
    let reset = sealed_reset_hook(&scenario);
    let two = TwoRate {
        slow: &bio,
        fast: &fast,
        slow_resolver: &bio_r,
        fast_resolver: &fast_r,
        steps_per_day: scenario.steps_per_day,
        slow_steps_per_day: scenario.bio_steps_per_day,
        slow_dt: scenario.bio_dt,
        fast_dt: scenario.cabin_dt,
        slow_reset: Some(&*reset),
    };
    let (states, _) = two
        .run(DayOrder::Interleaved, state, 1, &mut |_, _, _| {})
        .expect("a keyless save resumes");
    let end = states.last().unwrap();
    assert!(end.aux.contains_key(EVAPORATION_RATE_AUX));
    assert!(end.aux[EVAPORATED_TOTAL_AUX] > 0.0);
}

/// The retired slow-first day refuses the books: every minute would draw the day's last window's
/// rate, and the heat drawn would part from the ledger without an error. With them switched off it
/// runs (a plant-step gas exchange, so the gas refusal does not answer first). At two vapour
/// stocks: a plant-step build cannot hold the one-store shared room (the reference since
/// 2026-10-20), whose condenser reads the minute step's recorded window.
#[test]
fn the_slow_first_day_refuses_the_books() {
    let scenario = station::scenario::SealedStationScenario {
        room_vapour: station::room_vapour::RoomVapour::TwoStocks,
        ..sealed_station_scenario()
    };
    let lamp = station::params::lamp();
    let build_plant_step = || {
        station::sealed::build_sealed_station_at(
            &domains::params::charge(),
            &domains::params::thermal(),
            &domains::params::crew(),
            &domains::params::eclss(),
            &station::params::water_recovery(),
            &lamp,
            &station::params::harvest(),
            &scenario,
            false,
            false,
            station::gas_exchange::GasExchangeStep::PlantStep,
        )
        .expect("build")
    };
    let bio_r = sealed_bio_resolver(&lamp, &scenario).expect("bio resolver");
    let fast_r = sealed_fast_resolver(&domains::params::charge(), &domains::params::crew(), &scenario).expect("fast");
    for books in [true, false] {
        let (state, bio, fast) = build_plant_step();
        let (_, bio, fast) = if books {
            (state, bio, fast)
        } else {
            without_latent_heat(state, bio, fast).expect("off")
        };
        let (bio, fast) = (EulerIntegrator::new(bio), EulerIntegrator::new(fast));
        let two = TwoRate {
            slow: &bio,
            fast: &fast,
            slow_resolver: &bio_r,
            fast_resolver: &fast_r,
            steps_per_day: scenario.steps_per_day,
            slow_steps_per_day: scenario.bio_steps_per_day,
            slow_dt: scenario.bio_dt,
            fast_dt: scenario.cabin_dt,
            slow_reset: None,
        };
        assert_eq!(
            two.validate(DayOrder::SlowFirst).is_err(),
            books,
            "books {books}"
        );
    }
}
