//! Step 3c slice 2a — the separate-air station (LAB-ONLY; no golden, no `src/` change to the
//! reference). Plan: `docs/plans/post-roadmap-room-temperature.md` §13 / §13a, where each
//! prediction was committed before the code. The season-level grade is the example
//! `cargo run --release -q -p station --example air_split`; these pins hold the same claims
//! over a shorter run so the suite stays cheap.

use domains::biosphere::stocks::{
    CONDENSATE, LEAF_C, ROOT_C, SOIL_WATER, STEM_C, STORAGE_C, SUBSOIL_WATER, WATER_SOURCE,
    WATER_VAPOR,
};
use domains::crew::WATER_STORE;
use domains::params;
use simcore::integrator::EulerIntegrator;
use simcore::state::State;
use station::air_split::{build_split_station, split_scenario, AirSplit, BVAD_CHAMBER_AIR_MOL};
use station::driver::run_master_day;
use station::gas_exchange::GasExchangeStep;
use station::params as station_params;
use station::scenario::{sealed_station_scenario, SealedStationScenario};
use station::sealed::{
    build_sealed_station, sealed_bio_resolver, sealed_fast_resolver, sealed_reset_hook,
};

/// Long enough for the crop to be well past seedling (the starvation is visible by day 30).
const DAYS: usize = 90;

fn plant_c(s: &State) -> f64 {
    [LEAF_C, STEM_C, ROOT_C, STORAGE_C]
        .iter()
        .map(|id| s.stocks[*id].amount)
        .sum()
}

/// The plants' own water loop: soil, subsoil, condensate, vapour, the recycled source.
fn plant_water(s: &State) -> f64 {
    [
        SOIL_WATER,
        SUBSOIL_WATER,
        CONDENSATE,
        WATER_VAPOR,
        WATER_SOURCE,
    ]
    .iter()
    .map(|id| s.stocks.get(*id).map_or(0.0, |x| x.amount))
    .sum()
}

fn run(
    scenario: &SealedStationScenario,
    built: (
        State,
        simcore::registry::Registry,
        simcore::registry::Registry,
    ),
) -> State {
    let (state, bio, fast) = built;
    let lamp = station_params::lamp();
    let bio_r = sealed_bio_resolver(&lamp, scenario).expect("bio resolver");
    let fast_r = sealed_fast_resolver(&params::charge(), scenario).expect("fast resolver");
    let reset = sealed_reset_hook(scenario);
    let (states, rationed, events) = run_master_day(
        &EulerIntegrator::new(bio),
        &EulerIntegrator::new(fast),
        state,
        &bio_r,
        &fast_r,
        DAYS,
        scenario.steps_per_day,
        scenario.bio_steps_per_day,
        scenario.bio_dt,
        scenario.cabin_dt,
        Some(&*reset),
    )
    .expect("run");
    assert_eq!(rationed, 0, "the backstop fired");
    assert!(events.is_empty(), "events: {events:?}");
    states.last().expect("a day").clone()
}

fn shared() -> State {
    let scenario = sealed_station_scenario();
    let built = build_sealed_station(
        &params::charge(),
        &params::thermal(),
        &params::crew(),
        &params::eclss(),
        &station_params::water_recovery(),
        &station_params::lamp(),
        &station_params::harvest(),
        &scenario,
        false,
        false,
    )
    .expect("build_sealed_station");
    run(&scenario, built)
}

fn split(chamber_air_mol: f64, fan_mol_per_s: f64, vapour_crosses: bool) -> State {
    split_with(
        chamber_air_mol,
        fan_mol_per_s,
        vapour_crosses,
        GasExchangeStep::PlantStep,
    )
}

fn split_with(
    chamber_air_mol: f64,
    fan_mol_per_s: f64,
    vapour_crosses: bool,
    gas_exchange: GasExchangeStep,
) -> State {
    let scenario = sealed_station_scenario();
    let s = AirSplit {
        chamber_air_mol,
        fan_mol_per_s,
        vapour_crosses,
        gas_exchange,
        watering: false,
        transpiration: GasExchangeStep::PlantStep,
    };
    let built = build_split_station(
        &params::charge(),
        &params::thermal(),
        &params::crew(),
        &params::eclss(),
        &station_params::water_recovery(),
        &station_params::lamp(),
        &station_params::harvest(),
        &scenario,
        &s,
    )
    .expect("build_split_station");
    run(&split_scenario(&scenario, &s), built)
}

/// Prediction 3: at the BVAD chamber the crop starves by the slow step — well under half of
/// shared air's growth (measured over a season: 0.144 of shared plant carbon).
#[test]
fn a_bvad_sized_chamber_starves_the_crop_of_co2() {
    let base = plant_c(&shared());
    let small = plant_c(&split(BVAD_CHAMBER_AIR_MOL, 0.2, false));
    assert!(
        small < 0.5 * base,
        "plant carbon {small} is not under half of shared air's {base}"
    );
}

/// The fan-rate control: at 0.1 and 0.4 mol/s (both inside the stable window) the crop grows
/// within 5 % — the starvation is the slow step's, not a fan too weak to keep up.
#[test]
fn the_starvation_does_not_depend_on_the_fan_rate() {
    let slow = plant_c(&split(BVAD_CHAMBER_AIR_MOL, 0.1, false));
    let fast = plant_c(&split(BVAD_CHAMBER_AIR_MOL, 0.4, false));
    assert!(
        (slow / fast - 1.0).abs() < 0.05,
        "fan 0.1 → {slow}, fan 0.4 → {fast}"
    );
}

/// Prediction 5, the control: a chamber the cabin's own size with a fast fan grows the crop
/// within 5 % of shared air — so the split machinery itself does not starve anything.
#[test]
fn a_cabin_sized_chamber_recovers_shared_air() {
    let base = plant_c(&shared());
    let big = plant_c(&split(9500.0, 10.0, false));
    assert!(
        (big / base - 1.0).abs() < 0.05,
        "big chamber {big} vs shared {base}"
    );
}

/// W1 / W2: with vapour held back the plants' water loop is exactly closed; with vapour
/// crossing, water moves between the plants' books and the crew's. A REDISTRIBUTION —
/// conservation alone cannot see it.
///
/// ⚠ **The direction is a fact about the two rooms' air, and it reversed on 2026-10-03.** Until
/// then the cabin sat at ≈ 1.5 % RH and the plants lost water to the crew. With the cabin's
/// condenser holding BVAD's 40 % at 22 °C (1.95e-4 kg of vapour per mol of air), the cabin is
/// WETTER per mol than a chamber at 75 % of the weather's colder saturation, so over these 90
/// days the crew's water flows INTO the plants (`docs/plans/post-roadmap-room-temperature.md`
/// §15, H5). It should flip back once the chamber is held at 22 °C (75 % vs 40 % at one
/// temperature); this pin will then go red, and that red is the expected one.
#[test]
fn vapour_crossing_moves_water_between_the_plants_and_the_crew() {
    let base = shared();
    let off = split(BVAD_CHAMBER_AIR_MOL, 0.2, false);
    let on = split(BVAD_CHAMBER_AIR_MOL, 0.2, true);
    // A total folded from five stocks, so rounding in the last place is allowed, not more.
    assert!(
        (plant_water(&off) - plant_water(&base)).abs() < 1e-9,
        "vapour off: the plants' water loop {} should hold the shared one's {}",
        plant_water(&off),
        plant_water(&base)
    );
    let plants_gained = plant_water(&on) - plant_water(&off);
    let crew_lost = off.stocks[WATER_STORE].amount - on.stocks[WATER_STORE].amount;
    assert!(
        plants_gained > 1.0,
        "vapour on: the plants gained only {plants_gained} kg from the cabin"
    );
    assert!(
        crew_lost > 0.0,
        "vapour on: the crew's store lost {crew_lost} kg"
    );
    assert!(
        crew_lost <= plants_gained,
        "the cabin's own vapour and the recovery buffer make up the rest, never more \
         ({crew_lost} > {plants_gained})"
    );
}

/// The build itself: each room is charged at its OWN reference air, so the chamber's gases
/// and inert fill sum to its size and the cabin's to 9500 mol — the field every biosphere flow
/// reads the chamber's partial pressures and humidity target from. (The growth pins above
/// cannot see a chamber left at the cabin's size: its crop starves either way.)
#[test]
fn each_room_is_charged_at_its_own_size() {
    use domains::biosphere::science::N2_MOLAR_MASS_KG_PER_MOL;
    use domains::biosphere::stocks::{CARBON_POOL, CHAMBER_INERT, O2_POOL};
    use station::air_split::{cabin_gases, CABIN_AIR_MOL};
    let scenario = sealed_station_scenario();
    let s = AirSplit {
        chamber_air_mol: BVAD_CHAMBER_AIR_MOL,
        fan_mol_per_s: 0.2,
        vapour_crosses: false,
        gas_exchange: GasExchangeStep::PlantStep,
        watering: false,
        transpiration: GasExchangeStep::PlantStep,
    };
    let (state, _, _) = build_split_station(
        &params::charge(),
        &params::thermal(),
        &params::crew(),
        &params::eclss(),
        &station_params::water_recovery(),
        &station_params::lamp(),
        &station_params::harvest(),
        &scenario,
        &s,
    )
    .expect("build_split_station");
    let chamber = state.stocks[CARBON_POOL].amount
        + state.stocks[O2_POOL].amount
        + state.stocks[CHAMBER_INERT].amount / N2_MOLAR_MASS_KG_PER_MOL;
    assert!(
        (chamber / BVAD_CHAMBER_AIR_MOL - 1.0).abs() < 1e-12,
        "the chamber holds {chamber} mol of air, not {BVAD_CHAMBER_AIR_MOL}"
    );
    let (co2, o2, inert) = cabin_gases(&state).expect("cabin gases");
    let cabin = co2 + o2 + inert / N2_MOLAR_MASS_KG_PER_MOL;
    assert!(
        (cabin / CABIN_AIR_MOL - 1.0).abs() < 1e-12,
        "the cabin holds {cabin} mol of air, not {CABIN_AIR_MOL}"
    );
    // Both rooms start at the same composition.
    assert!(
        (state.stocks[CARBON_POOL].amount / BVAD_CHAMBER_AIR_MOL / (co2 / CABIN_AIR_MOL) - 1.0)
            .abs()
            < 1e-12
    );
}

/// §16 G2 + G3 — with the crop's gas exchange on the MINUTE step, the BVAD chamber no longer
/// starves, and the fan rate now matters. Measured at 90 days (2026-10-03): Q 0.1 / 0.2 / 0.4 →
/// 0.892 / 0.939 / 0.963 of shared air's plant carbon, against 0.144 at any rate on the plant
/// step (`the_starvation_does_not_depend_on_the_fan_rate`). Equal draws at different fan rates
/// would mean the plant step is still the limit — the failure this pin exists to catch.
#[test]
fn on_the_minute_step_the_chamber_is_fed_and_the_fan_rate_matters() {
    let base = plant_c(&shared());
    let m = GasExchangeStep::Minute;
    let [slow, mid, fast] =
        [0.1, 0.2, 0.4].map(|q| plant_c(&split_with(BVAD_CHAMBER_AIR_MOL, q, false, m)));
    assert!(mid / base > 0.8, "fan 0.2: {mid} vs shared {base}");
    assert!(
        slow < mid && mid < fast,
        "fan 0.1 / 0.2 / 0.4 → {slow} / {mid} / {fast}"
    );
    assert!(
        slow / fast < 0.97,
        "fan 0.1 → {slow}, fan 0.4 → {fast}: the fan should matter"
    );
}

/// §16 G4 — the control on the minute step: a cabin-sized chamber with a fast fan grows within
/// 5 % of shared air (measured 1.008 at 90 days).
#[test]
fn on_the_minute_step_a_cabin_sized_chamber_recovers_shared_air() {
    let base = plant_c(&shared());
    let big = plant_c(&split_with(9500.0, 10.0, false, GasExchangeStep::Minute));
    assert!(
        (big / base - 1.0).abs() < 0.05,
        "big chamber {big} vs shared {base}"
    );
}

/// The plant registry of a WATERED separate-air build, its resolver, and its state.
fn watered_build() -> (
    State,
    simcore::registry::Registry,
    simcore::environment::SourceResolver,
) {
    let scenario = sealed_station_scenario();
    let s = AirSplit {
        chamber_air_mol: BVAD_CHAMBER_AIR_MOL,
        fan_mol_per_s: 0.2,
        vapour_crosses: true,
        gas_exchange: GasExchangeStep::Minute,
        watering: true,
        transpiration: GasExchangeStep::PlantStep,
    };
    let (state, bio, _) = build_split_station(
        &params::charge(),
        &params::thermal(),
        &params::crew(),
        &params::eclss(),
        &station_params::water_recovery(),
        &station_params::lamp(),
        &station_params::harvest(),
        &scenario,
        &s,
    )
    .expect("build_split_station");
    let resized = split_scenario(&scenario, &s);
    let r = sealed_bio_resolver(&station_params::lamp(), &resized).expect("bio resolver");
    (state, bio, r)
}

fn with_soil_water(state: &State, kg: f64) -> State {
    let mut stocks = state.stocks.clone();
    let sw = stocks[SOIL_WATER].with_amount(kg).unwrap();
    stocks.insert(SOIL_WATER.to_string(), sw);
    State::new(state.n, stocks, state.rng_seed, state.aux.clone()).unwrap()
}

/// The root zone's transpirable capacity at this state's rooted depth (kg).
fn ttsw(state: &State) -> f64 {
    let b = sealed_station_scenario().bio;
    domains::biosphere::science::transpirable_capacity(
        state.aux[domains::biosphere::stocks::ROOTED_DEPTH],
        b.soil_extractable_water,
        b.ground_area,
    )
}

/// §18b T2 — watering FIRES on a root zone below FAO-56's trigger (FTSW < 0.45): water moves
/// from the crew's store to the soil, kilogram for kilogram, until the zone is back over the
/// trigger, and then it stops. The zone is set low on purpose: no season run gets there
/// (their lowest fill is 0.66).
#[test]
fn watering_fires_below_the_trigger_and_stops_above_it() {
    let (state, bio, r) = watered_build();
    let plant_dt = sealed_station_scenario().bio_dt;
    let flow = bio
        .flows()
        .iter()
        .find(|f| f.id() == station::air_split::WATERING)
        .expect("the watering flow");
    let cap = ttsw(&state);
    let mut s = with_soil_water(&state, 0.30 * cap);
    let mut steps = 0;
    loop {
        let res = flow.evaluate(&s, &r.bind(&s, plant_dt), plant_dt).unwrap();
        let fill = s.stocks[SOIL_WATER].amount / cap;
        if res.legs.is_empty() {
            assert!(fill >= 0.45, "it stopped at fill {fill}, below the trigger");
            break;
        }
        assert!(fill < 0.45, "it watered at fill {fill}, above the trigger");
        let to_soil: f64 = res
            .legs
            .iter()
            .filter(|l| l.stock == SOIL_WATER)
            .map(|l| l.amount)
            .sum();
        let from_store: f64 = res
            .legs
            .iter()
            .filter(|l| l.stock == WATER_STORE)
            .map(|l| l.amount)
            .sum();
        assert!(
            to_soil > 0.0 && to_soil == -from_store,
            "{to_soil} vs {from_store}"
        );
        s = with_soil_water(&s, s.stocks[SOIL_WATER].amount + to_soil);
        steps += 1;
        assert!(steps < 100, "never stopped");
    }
    assert!(
        steps > 1,
        "a single step refilled it, so the stop was not tested"
    );
}

/// §18b T3 — above the trigger the watering gives EXACTLY nothing: no legs at all, at a fill
/// just over 0.45 and at the full zone a season starts with.
#[test]
fn watering_gives_nothing_above_the_trigger() {
    let (state, bio, r) = watered_build();
    let plant_dt = sealed_station_scenario().bio_dt;
    let flow = bio
        .flows()
        .iter()
        .find(|f| f.id() == station::air_split::WATERING)
        .expect("the watering flow");
    for fill in [0.46, 0.66, 1.0] {
        let s = with_soil_water(&state, fill * ttsw(&state));
        let res = flow.evaluate(&s, &r.bind(&s, plant_dt), plant_dt).unwrap();
        assert!(res.legs.is_empty(), "fill {fill}: {:?}", res.legs.len());
    }
}
