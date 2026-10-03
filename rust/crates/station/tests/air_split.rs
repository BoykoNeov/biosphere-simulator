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
    let scenario = sealed_station_scenario();
    let s = AirSplit {
        chamber_air_mol,
        fan_mol_per_s,
        vapour_crosses,
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
/// crossing, the loop loses water and the crew's store gains it (one way: the cabin is far
/// drier). A REDISTRIBUTION — conservation alone cannot see it.
#[test]
fn vapour_crossing_drains_the_plants_water_into_the_crews() {
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
    let lost = plant_water(&off) - plant_water(&on);
    let gained = on.stocks[WATER_STORE].amount - off.stocks[WATER_STORE].amount;
    assert!(lost > 1.0, "vapour on: the plants lost only {lost} kg");
    assert!(
        gained > 0.0,
        "vapour on: the crew's store gained {gained} kg"
    );
    assert!(
        gained <= lost,
        "the crew cannot gain more than the plants lost ({gained} > {lost})"
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
