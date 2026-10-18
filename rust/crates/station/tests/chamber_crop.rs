//! The plant chamber's crop is ONE value (`SealedStationScenario::crop`), and every place the
//! chamber was wheat by name follows it: the season's params, the re-sow's params, the maturity
//! threshold, the season's crop switches and the cold program
//! (`docs/plans/post-roadmap-authored-plants.md` §17).
//!
//! ⚠ A chamber that switched only some of the five builds, conserves, never rations and repeats —
//! every check a run has would pass on it. So each one is checked against the crop's own value
//! here, and the wheat side is checked to be the reference's exactly (the switch moved no golden).
//!
//! The structs carry no `PartialEq`; they are compared by their `Debug` text, which prints every
//! field (and every float to its round-trip digits).

use domains::biosphere::stocks::{STORAGE_C, THERMAL_TIME};
use domains::biosphere::{annual_reset_with, build_season_with, potato_switches};
use station::lighting::{room_form, room_params};
use station::plant_chamber::{chamber_plants, chamber_reset_hook, ResowRule};
use station::scenario::{sealed_station_scenario, Crop, Phase, SealedStationScenario};
use station::sowing::{sow_now, SOWN_STEP};

fn same<T: std::fmt::Debug>(a: &T, b: &T) -> bool {
    format!("{a:?}") == format!("{b:?}")
}

fn potato() -> SealedStationScenario {
    sealed_station_scenario().with_crop(Crop::Potato)
}

/// The reference chamber is wheat, and its params and phenology are the ones it was built from
/// before the crop became a field: the room form of the loader's, and `params::phenology()`.
#[test]
fn the_reference_chamber_is_wheat_as_it_was() {
    let s = sealed_station_scenario();
    assert_eq!(s.crop, Crop::Wheat);
    assert!(same(&s.plant_params(), &room_params()));
    assert!(same(&s.pheno(), &domains::biosphere::params::phenology()));
    assert!(
        same(&s.with_crop(Crop::Wheat), &s),
        "Crop::Wheat must leave the scenario as it is"
    );
}

/// A potato chamber's params, phenology, switches and cold are each the potato's — and nothing
/// else of the scenario moved.
#[test]
fn a_potato_chamber_takes_all_five_from_the_crop() {
    let wheat = sealed_station_scenario();
    let s = potato();
    assert_eq!(s.crop, Crop::Potato);
    assert!(same(
        &s.plant_params(),
        &room_form(domains::biosphere::params::potato())
    ));
    assert!(same(&s.pheno(), &domains::biosphere::params::potato().pheno));
    assert!(
        !same(&s.pheno(), &wheat.pheno()),
        "the two crops' phenology must differ, or the pin above is vacuous"
    );
    // The switches: the field potato's own, on the chamber's plot.
    assert!(same(&s.bio, &potato_switches(wheat.bio)));
    assert!(!s.bio.vernalization && !s.bio.photoperiod && !s.bio.stem_reserves);
    assert!(s.bio.wssd.is_none());
    // The cold: none, from the sowing on — the warm setpoint and the full lamp on day 0.
    assert_eq!(s.cold.days, 0);
    assert_eq!(s.phase_since_sowing(0), Phase::Warm);
    assert_eq!(s.lamp_power(s.phase_since_sowing(0), 1.0), s.lamp_power_w);
    // Nothing else moved.
    let mut back = s;
    back.crop = wheat.crop;
    back.bio = wheat.bio;
    back.cold.days = wheat.cold.days;
    assert!(same(&back, &wheat));
}

/// The chamber's plants are the season built from the potato's room params — the builder's own
/// stocks and starting values, plus the sowing clock — and differ from the wheat chamber's.
#[test]
fn a_potato_chamber_builds_its_season_from_the_potato() {
    let s = potato();
    let plants = chamber_plants(&s).expect("plants");
    let (direct, _) = build_season_with(&s.bio, &s.plant_params()).expect("season");
    let mut aux = direct.aux.clone();
    aux.insert(SOWN_STEP.to_string(), 0.0);
    assert!(same(&plants.aux, &aux));
    assert!(same(&plants.stocks, &direct.stocks));
    let wheat = chamber_plants(&sealed_station_scenario()).expect("wheat plants");
    assert!(
        !same(&plants.aux, &wheat.aux),
        "a potato chamber starts with the wheat chamber's values"
    );
}

/// The re-sow: at the POTATO's development stage 2 and not one ulp of thermal time before, and the
/// re-sown state is reset with the potato's params — which a reset with the wheat's is not (the
/// seedling's leaf area shades the soil, and the two crops' specific leaf areas differ).
#[test]
fn a_potato_chamber_resows_at_its_own_maturity_with_its_own_params() {
    let s = potato();
    let pheno = domains::biosphere::params::potato().pheno;
    // The first thermal time at which the development stage reads 2. ⚠ Not the sum of the two
    // sums: the potato's are not whole numbers, and the stage rounds to 2 a few ulps below it.
    let stage = |tt: f64| {
        domains::biosphere::science::development_stage(tt, pheno.tsum_anthesis, pheno.tsum_maturity)
    };
    let mut mature_tt = pheno.tsum_anthesis + pheno.tsum_maturity;
    assert!(stage(mature_tt) >= 2.0);
    while stage(f64::from_bits(mature_tt.to_bits() - 1)) >= 2.0 {
        mature_tt = f64::from_bits(mature_tt.to_bits() - 1);
    }
    let wheat_pheno = domains::biosphere::params::phenology();
    assert!(
        mature_tt < wheat_pheno.tsum_anthesis + wheat_pheno.tsum_maturity,
        "the potato must mature first, or 'not before' does not tell the crops apart"
    );
    let plants = chamber_plants(&s).expect("plants");
    let n = 200 * s.bio_steps_per_day;
    let with_tt = |tt: f64| {
        let mut aux = plants.aux.clone();
        aux.insert(THERMAL_TIME.to_string(), tt);
        let mut stocks = plants.stocks.clone();
        // A matured crop has tubers to re-sow from.
        stocks.insert(
            STORAGE_C.to_string(),
            stocks[STORAGE_C].with_amount(10.0).unwrap(),
        );
        simcore::state::State::new(n, stocks, 0, aux).expect("state")
    };
    let hook = chamber_reset_hook(&s, ResowRule::Maturity);
    let before = with_tt(f64::from_bits(mature_tt.to_bits() - 1));
    assert!(hook(n, &before).unwrap().is_none(), "re-sown before maturity");
    let at = with_tt(mature_tt);
    let resown = hook(n, &at).unwrap().expect("not re-sown at maturity");
    let expected = sow_now(annual_reset_with(&at, &s.bio, &s.plant_params()).unwrap()).unwrap();
    assert!(same(&resown.aux, &expected.aux));
    let with_wheat = sow_now(annual_reset_with(&at, &s.bio, &room_params()).unwrap()).unwrap();
    assert!(
        !same(&resown.aux, &with_wheat.aux),
        "a reset with the wheat's params must differ, or the pin above is vacuous"
    );
}
