//! The station assemblies start from the aux `build_season` seeds — the omission gate.
//!
//! Each station builder takes `build_season`'s stocks and registry but assembles its OWN
//! starting aux, so a value `build_season` seeds can be silently dropped. It has happened
//! three times: `rooted_depth` (until 2026-08-12), the soil account's five values and
//! `vernalization_days` (both until 2026-10-14, `docs/plans/post-roadmap-soil-evaporation.md`
//! §17d–§19). A dropped key reads as 0 everywhere (`aux_of`, the merge's `or_insert`), so no
//! golden and no conservation check sees it; only a test that READS the key panics. This one
//! reads every key, so a key added to `build_season` later is checked here without anyone
//! remembering to.

use std::collections::BTreeMap;

use domains::biosphere::system::build_season;
use domains::params;
use station::greenhouse::build_greenhouse;
use station::harvest::build_harvest;
use station::lighting::build_lighting;
use station::params as station_params;
use station::scenario::{
    greenhouse_scenario, harvest_scenario, lighting_scenario, sealed_station_scenario,
};
use station::sealed::build_sealed_station;

/// Every key `seeded` holds is in `station` with the same value (bit-exact).
fn assert_seeds(name: &str, seeded: &BTreeMap<String, f64>, station: &BTreeMap<String, f64>) {
    for (key, value) in seeded {
        let got = station.get(key).unwrap_or_else(|| {
            panic!("{name}: `build_season` seeds `{key}` and the station build does not")
        });
        assert_eq!(
            got.to_bits(),
            value.to_bits(),
            "{name}: `{key}` starts at {got}, `build_season` seeds {value}"
        );
    }
}

#[test]
fn the_sown_station_builds_start_from_build_seasons_aux() {
    let crew = params::crew();
    let eclss = params::eclss();
    let lamp = station_params::lamp();

    let gh = greenhouse_scenario();
    let (state, _, _) =
        build_greenhouse(&crew, &eclss, &gh, true, domains::crew::FECAL_WASTE).unwrap();
    assert_seeds(
        "greenhouse",
        &build_season(&gh.bio).unwrap().0.aux,
        &state.aux,
    );

    let lit = lighting_scenario();
    let (state, _, _) = build_lighting(&lamp, &lit, true).unwrap();
    assert_seeds(
        "lighting",
        &build_season(&lit.bio).unwrap().0.aux,
        &state.aux,
    );

    let sealed = sealed_station_scenario();
    let (state, _, _) = build_sealed_station(
        &params::charge(),
        &params::thermal(),
        &crew,
        &eclss,
        &station_params::water_recovery(),
        &lamp,
        &station_params::harvest(),
        &sealed,
        false,
        false,
    )
    .unwrap();
    assert_seeds(
        "sealed_station",
        &build_season(&sealed.bio).unwrap().0.aux,
        &state.aux,
    );
}

/// Harvest starts a grown crop, so its values are its own (re-derived root zone, thermal time
/// past anthesis): only the KEY set is `build_season`'s.
#[test]
fn the_harvest_build_seeds_every_key_build_season_seeds() {
    let scenario = harvest_scenario();
    let (state, _, _) = build_harvest(
        &params::crew(),
        &params::eclss(),
        &station_params::harvest(),
        &scenario,
        true,
        true,
    )
    .unwrap();
    let seeded = build_season(&scenario.greenhouse.bio).unwrap().0.aux;
    for key in seeded.keys() {
        assert!(
            state.aux.contains_key(key),
            "harvest: `build_season` seeds `{key}` and the station build does not"
        );
    }
}
