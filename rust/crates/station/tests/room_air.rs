//! **A room is not outdoors** — the station's half of the open field's night
//! (`docs/plans/post-roadmap-open-field-night.md` §7e, §9).
//!
//! Since 2026-10-17 the loader's weather supplies the air's within-day cycle and the loader's water
//! reads it. A lamp-lit room has its own temperature, so its water must read that, never the outdoor
//! curve. Two guards make it so, and both are pinned here because the loader now supplies the names
//! they keep out: the room resolvers drop the two within-day names (a reader left on them fails at its
//! first step rather than reading the outdoors — the silent leak the flipped tree found), and
//! `room_params` differs from the loader's params in exactly the air read.

use domains::biosphere::params;
use domains::biosphere::science::AirTemperatureRead;
use domains::biosphere::stocks::{AIR_TEMP_WITHIN_DAY_VAR, VPD_WITHIN_DAY_VAR};
use domains::biosphere::system::weather_forcings;
use station::lighting::{lighting_bio_resolver, room_params};
use station::params as station_params;
use station::scenario::{lighting_scenario, sealed_station_scenario};
use station::sealed::sealed_bio_resolver;

const OUTDOOR_AIR: [&str; 2] = [AIR_TEMP_WITHIN_DAY_VAR, VPD_WITHIN_DAY_VAR];

/// The loader's weather supplies the names (so the drop below is load-bearing, not a no-op), and
/// neither room resolver passes them on.
#[test]
fn the_rooms_drop_the_outdoor_air() {
    let lighting = lighting_scenario();
    let weather = weather_forcings(&lighting.bio, 1).expect("weather");
    for name in OUTDOOR_AIR {
        assert!(weather.contains_key(name), "the loader's weather no longer supplies {name}");
    }
    let lamp = station_params::lamp();
    for with_lamp in [true, false] {
        let (forcings, _) = lighting_bio_resolver(&lamp, &lighting, with_lamp)
            .expect("lighting resolver")
            .into_parts();
        for name in OUTDOOR_AIR {
            assert!(!forcings.contains_key(name), "the lighting room passes on {name}");
        }
    }
    let (forcings, _) = sealed_bio_resolver(&lamp, &sealed_station_scenario())
        .expect("sealed resolver")
        .into_parts();
    for name in OUTDOOR_AIR {
        assert!(!forcings.contains_key(name), "the sealed room passes on {name}");
    }
}

/// A room's params are the loader's but for the air its water reads: the room's own `temp`.
#[test]
fn a_room_reads_its_own_air_and_nothing_else_differs() {
    let loader = params::biosphere();
    assert_eq!(loader.water.air_temperature, AirTemperatureRead::WithinDay);
    let room = room_params();
    assert_eq!(room.water.air_temperature, AirTemperatureRead::DailyMean);
    let mut aligned = loader.clone();
    aligned.water.air_temperature = AirTemperatureRead::DailyMean;
    assert_eq!(format!("{room:?}"), format!("{aligned:?}"));
}
