//! **The heat evaporation takes from the room** — the lab switch's own checks.
//!
//! Plan: `docs/plans/post-roadmap-latent-heat-books.md` (§1, §4 Q1). `station::latent_heat::with_latent_heat`
//! records each plant window's evaporation and draws λ × it from the chamber over the window's minutes.
//! These tests hold the drawn heat to the moved water, window by window, and hold the switch's
//! refusals.

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
    with_latent_heat, CHAMBER_LATENT_HEAT, EVAPORATION_RATE_AUX, EVAPORATION_RECORDER,
    TRANSPIRATION,
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
    let (state, bio, fast) = with_latent_heat(state, bio, fast).expect("latent");
    let (bio, fast) = (EulerIntegrator::new(bio), EulerIntegrator::new(fast));
    let bio_r = sealed_bio_resolver(&lamp, &scenario).expect("bio resolver");
    let fast_r = sealed_fast_resolver(&domains::params::charge(), &scenario).expect("fast");
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
    let (_, totals) = two
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
}

/// The switch adds one recorder and one fast flow, seeds the rate, and leaves transpiration on the
/// plant step once, under its own type name; the build without it carries none of them.
#[test]
fn the_switch_adds_exactly_its_own_pieces() {
    let (state, bio, fast) = build();
    let count = |reg: &Registry, id: &str| reg.flows().iter().filter(|f| f.id() == id).count();
    assert_eq!(count(&fast, CHAMBER_LATENT_HEAT), 0);
    assert!(!state.aux.contains_key(EVAPORATION_RATE_AUX));
    assert!(!bio.aux_processes().iter().any(|a| a.id() == EVAPORATION_RECORDER));
    let (n_bio, n_fast, n_aux) = (bio.flows().len(), fast.flows().len(), bio.aux_processes().len());

    let (state, bio, fast) = with_latent_heat(state, bio, fast).expect("latent");
    assert_eq!(bio.flows().len(), n_bio);
    assert_eq!(fast.flows().len(), n_fast + 1);
    assert_eq!(bio.aux_processes().len(), n_aux + 1);
    assert_eq!(count(&bio, TRANSPIRATION), 1);
    assert_eq!(count(&fast, TRANSPIRATION), 0);
    let t = bio.flows().iter().find(|f| f.id() == TRANSPIRATION).unwrap();
    assert_eq!(t.type_name(), "Transpiration");
    assert_eq!(state.aux[EVAPORATION_RATE_AUX], 0.0);
}

/// The water moved onto the minute (the lab's `water_on_fast_step`) and the latent heat refuse
/// each other in either order: the recorder would read a plant step the water had left.
#[test]
fn water_on_the_minute_and_the_latent_heat_refuse_each_other() {
    let shared = weather_shared(&sealed_station_scenario().bio);

    let (state, bio, fast) = build();
    let (bio, fast) = water_on_fast_step(&state.stocks, bio, fast, &shared).expect("move");
    assert!(with_latent_heat(state, bio, fast).is_err());

    let (state, bio, fast) = build();
    let (state, bio, fast) = with_latent_heat(state, bio, fast).expect("latent");
    assert!(water_on_fast_step(&state.stocks, bio, fast, &shared).is_err());
}

/// The switch wires a fresh build only.
#[test]
fn the_switch_refuses_a_running_state() {
    let (state, bio, fast) = build();
    let moved = State::new(5, state.stocks.clone(), state.rng_seed, state.aux.clone()).unwrap();
    assert!(with_latent_heat(moved, bio, fast).is_err());
}
