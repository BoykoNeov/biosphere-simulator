//! The lamp-lit plant chamber, as parts a habitat assembles — the sealed station's chamber,
//! extracted so an authored crop bed builds the SAME chamber (slice 3 of
//! `docs/plans/post-roadmap-authored-plants.md`).
//!
//! Until 2026-10-18 the chamber existed only inside [`crate::sealed::build_sealed_station_unread`],
//! wired to the station's battery and thermal node by name. Here it is the same pieces with its
//! outward connections as [`ChamberPorts`]: the sealed station passes its battery, node and
//! `boundary.space`, and an authored bed passes the author's own stocks. Everything inside the
//! chamber — the season, its air and soil, the chamber's heat store, the lamp program, the
//! latent-heat books, the gas exchange on the fast step, the last wrappers — is the reference's,
//! not a copy of it.
//!
//! **The proof it is the same chamber:** the sealed station is rebuilt on these parts and every
//! golden is byte-identical (`regen_goldens` 0 of 20).

use std::collections::BTreeMap;

use domains::biosphere::system::build_season_with;
use domains::thermal::ThermalParams;
use simcore::boundary;
use simcore::error::SimError;
use simcore::flow::Flow;
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::Stock;

use crate::chamber::{
    chamber_heat0, chamber_stock, ChamberCooling, ChamberHeater, ChamberParams,
    ChamberSurroundings, ChamberWall, CHAMBER, CHAMBER_COOLING, CHAMBER_HEATER,
    CHAMBER_SURROUNDINGS, CHAMBER_WALL,
};
use crate::flows::{Lamp, LampParams, LAMP};
use crate::latent_heat::ChamberLatentHeat;
use crate::scenario::SealedStationScenario;
use crate::sealed::switched_lamp_draw_for;
use crate::sowing::SOWN_STEP;

/// Where a plant chamber connects to the habitat around it. Each is a stock id of the habitat's.
#[derive(Debug, Clone, Copy)]
pub struct ChamberPorts<'a> {
    /// The energy store the lamp and the chamber's heater draw from (the station's battery).
    pub power: &'a str,
    /// The heat store the chamber's cooler and its latent-heat coil hand heat on to (the
    /// station's thermal node). Also the walls' far side under [`ChamberSurroundings::Structure`].
    pub heat_sink: &'a str,
    /// The walls' far side under [`ChamberSurroundings::Space`] (the station's `boundary.space`).
    pub space: &'a str,
}

/// The chamber's own fast flows, in the sealed station's order: the lamp, the cooler, the walls,
/// the heater. The latent-heat coil is [`chamber_latent_heat_flow`], kept apart because the
/// station's registry lists it last.
pub fn chamber_flows(
    lamp: &LampParams,
    chamber: &ChamberParams,
    thermal_params: &ThermalParams,
    surroundings: ChamberSurroundings,
    ports: ChamberPorts<'_>,
) -> Vec<Box<dyn Flow>> {
    vec![
        Box::new(Lamp::new(
            LAMP.to_string(),
            ports.power.to_string(),
            CHAMBER.to_string(), // the light is absorbed in the chamber it lights (2b-ii)…
            CHAMBER.to_string(), // …and the waste heat lands there too (2b-i)
            *lamp,
        )),
        Box::new(ChamberCooling::new(
            CHAMBER_COOLING.to_string(),
            CHAMBER.to_string(),
            ports.heat_sink.to_string(), // the cooler hands the lamp heat on
            *chamber,
            *thermal_params,
        )),
        Box::new(ChamberWall::new(
            CHAMBER_WALL.to_string(),
            CHAMBER.to_string(),
            match surroundings {
                ChamberSurroundings::Outdoor | ChamberSurroundings::Cabin { .. } => {
                    CHAMBER_SURROUNDINGS.to_string()
                }
                ChamberSurroundings::Space { .. } => ports.space.to_string(),
                ChamberSurroundings::Structure => ports.heat_sink.to_string(),
            },
            surroundings,
            *chamber,
            *thermal_params,
        )),
        Box::new(ChamberHeater::new(
            CHAMBER_HEATER.to_string(),
            ports.power.to_string(),
            CHAMBER.to_string(),
            *chamber,
        )),
    ]
}

/// The heat the plants' evaporation takes from the chamber, handed to the habitat's heat store
/// (2026-10-15, `docs/plans/post-roadmap-latent-heat-books.md`). The plant side's recorder is
/// booked by [`crate::latent_heat::book_latent_heat`].
pub fn chamber_latent_heat_flow(ports: ChamberPorts<'_>) -> Box<dyn Flow> {
    Box::new(ChamberLatentHeat::new(
        crate::latent_heat::CHAMBER_LATENT_HEAT.to_string(),
        CHAMBER.to_string(),
        ports.heat_sink.to_string(),
    ))
}

/// The chamber's own stocks at the start: its heat store, at the steady state of the program's
/// setpoint and the lamp's first-window draw on the sowing day, and — when the walls face the
/// outdoors or a held cabin — the two-signed boundary they exchange with.
pub fn chamber_start_stocks(
    scenario: &SealedStationScenario,
    chamber: &ChamberParams,
    surroundings: ChamberSurroundings,
) -> Result<Vec<Stock>, SimError> {
    // Day 0's setpoint and lamp: the program's on the sowing day (the cold phase's in the
    // reference; §24c, "start values"). The crop is sown at step 0 (`SOWN_STEP`). The lamp is
    // its draw over the first fast group (window 0, dark: 0 W) since the lamp's heat switched
    // with its light (2026-10-15, `docs/plans/post-roadmap-room-lamp-heat.md` §8); the daily
    // average until then.
    let sowing_phase = scenario.phase_since_sowing(0);
    let first_group_n =
        crate::driver::day_groups(scenario.steps_per_day, scenario.bio_steps_per_day)?.slow;
    let mut stocks = vec![chamber_stock(chamber_heat0(
        chamber,
        scenario.chamber_setpoint(sowing_phase, chamber.warm_setpoint),
        switched_lamp_draw_for(scenario, sowing_phase)?(first_group_n, scenario.cabin_dt),
    ))?];
    if matches!(
        surroundings,
        ChamberSurroundings::Outdoor | ChamberSurroundings::Cabin { .. }
    ) {
        // Two-signed: a hotter outside sends heat in.
        stocks.push(boundary::source(
            CHAMBER_SURROUNDINGS.to_string(),
            Quantity::Energy,
            0.0,
            true,
        )?);
    }
    Ok(stocks)
}

/// The chamber's plants: the season built for a lamp-lit room (`lighting::room_params`), its
/// stocks, its plant registry and its starting state values — **the builder's own**, plus the
/// cold program's sowing clock (slice 4: the crop is sown at step 0).
///
/// ⚠ The starting values were hand-assembled by the sealed builder until 2026-10-18, beside the
/// builder's own (ROOTED_DEPTH was once missing from that copy, and the crop started at depth 0).
/// Measured equal on the reference params before the copy was deleted; under the lab leaf form
/// the builder's carry the seedling's leaf area, which the copy did not.
pub fn chamber_plants(scenario: &SealedStationScenario) -> Result<ChamberPlants, SimError> {
    let (bio_state, registry) = build_season_with(&scenario.bio, &crate::lighting::room_params())?;
    let mut aux = bio_state.aux.clone();
    // The cold program's clock: seeded here, not defaulted at the reader — the omission
    // `vernalization_days` had until §19.
    aux.insert(SOWN_STEP.to_string(), 0.0);
    Ok(ChamberPlants {
        stocks: bio_state.stocks,
        aux,
        registry,
    })
}

/// The chamber's plants as [`chamber_plants`] builds them.
pub struct ChamberPlants {
    /// The season's stocks (plants, soil, the chamber's air and water, the loss sink).
    pub stocks: BTreeMap<String, Stock>,
    /// The starting state values: the season builder's own, plus the sowing clock.
    pub aux: BTreeMap<String, f64>,
    /// The plant registry, as built (not yet booked, moved or wrapped).
    pub registry: Registry,
}
