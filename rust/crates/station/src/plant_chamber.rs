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

use std::collections::{BTreeMap, BTreeSet, HashMap};

use domains::biosphere::system::{annual_reset_with, build_season_with, weather_shared};
use domains::thermal::ThermalParams;
use simcore::boundary;
use simcore::environment::Schedule;
use simcore::error::SimError;
use simcore::flow::Flow;
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::{State, Stock};

use crate::chamber::{
    chamber_heat0, chamber_stock, require_step_within_response, ChamberCooling, ChamberHeater,
    ChamberParams, ChamberSurroundings, ChamberWall, CHAMBER, CHAMBER_COOLING, CHAMBER_HEATER,
    CHAMBER_SURROUNDINGS, CHAMBER_WALL, OUTDOOR_TEMP_VAR,
};
use crate::driver::OwnedResetHook;
use crate::flows::{Lamp, LampParams, LAMP, LAMP_POWER_VAR};
use crate::gas_exchange::{gas_exchange_on_fast_step, require_one_plant_step_per_group};
use crate::latent_heat::ChamberLatentHeat;
use crate::scenario::{Phase, SealedStationScenario};
use crate::sealed::switched_lamp_draw_for;
use crate::sowing::{insert_twins, sow_now, SOWN_STEP};

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

/// The chamber's plants: the season built from the crop's params for a lamp-lit room
/// ([`SealedStationScenario::plant_params`]), its
/// stocks, its plant registry and its starting state values — **the builder's own**, plus the
/// cold program's sowing clock (slice 4: the crop is sown at step 0).
///
/// ⚠ The starting values were hand-assembled by the sealed builder until 2026-10-18, beside the
/// builder's own (ROOTED_DEPTH was once missing from that copy, and the crop started at depth 0).
/// Measured equal on the reference params before the copy was deleted; under the lab leaf form
/// the builder's carry the seedling's leaf area, which the copy did not.
pub fn chamber_plants(scenario: &SealedStationScenario) -> Result<ChamberPlants, SimError> {
    let (bio_state, registry) = build_season_with(&scenario.bio, &scenario.plant_params())?;
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

/// The chamber's own fast-step forcings: the lamp's draw on the cold program, switched with its
/// light ([`switched_lamp_draw_for`], as phase twins), the outdoor temperature the walls face
/// ([`crate::sealed::outdoor_temperature`]), and the setpoint program
/// ([`crate::sealed::chamber_setpoint_twins`]). The sealed station adds its crew's and power's.
///
/// ⚠ The lamp's efficacy is `lamp.yaml`'s, as the plant side's PAR is built from the
/// `LampParams` its caller passes: a caller passing a modified efficacy there would see cold PAR
/// and cold draw disagree (none does).
pub fn chamber_fast_forcings(
    scenario: &SealedStationScenario,
) -> Result<HashMap<String, Schedule>, SimError> {
    let mut forcings = HashMap::new();
    insert_twins(
        &mut forcings,
        LAMP_POWER_VAR,
        switched_lamp_draw_for(scenario, Phase::Cold)?,
        switched_lamp_draw_for(scenario, Phase::Warm)?,
    );
    forcings.insert(
        OUTDOOR_TEMP_VAR.to_string(),
        crate::sealed::outdoor_temperature(scenario)?,
    );
    crate::sealed::chamber_setpoint_twins(&mut forcings, scenario, &crate::params::chamber())?;
    Ok(forcings)
}

/// When a chamber's crop is re-sown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResowRule {
    /// At the first master-day start on which the crop has matured (development stage 2) — the
    /// sealed station's rule since Step 3c slice 4 stage 2.
    Maturity,
    /// Every `n` master days after the last sowing (an author's calendar).
    EveryDays(u64),
    /// Never: one crop, whatever becomes of it.
    Never,
}

/// The chamber's re-sow hook under `rule`, owned so a caller-driven session can hold it. A re-sow
/// is `annual_reset_with` the params the season was built from (`plant_params`: a run built from
/// an override is never re-sown with the frozen values — `tests/param_funnel.rs`'s hazard) and
/// the sowing clock set to the state's own step ([`crate::sowing::sow_now`]).
pub fn chamber_reset_hook(scenario: &SealedStationScenario, rule: ResowRule) -> OwnedResetHook {
    let bio = scenario.bio;
    let pheno = scenario.pheno();
    let bio_steps_per_day = scenario.bio_steps_per_day;
    // `annual_reset_with` and the params the season was built from: identical to `annual_reset`
    // on every frozen build, and it also re-sows the lab forms' state values, which the plain
    // reset refuses (`docs/plans/post-roadmap-soil-evaporation.md` §10).
    let params = scenario.plant_params();
    Box::new(
        move |n: u64, current: &State| -> Result<Option<State>, SimError> {
            let due = match rule {
                ResowRule::Maturity => crate::sealed::is_mature(current, &pheno)?,
                ResowRule::EveryDays(days) => {
                    let since = n.saturating_sub(crate::sowing::sown_step(current)?);
                    since > 0 && since % (days * bio_steps_per_day) == 0
                }
                ResowRule::Never => false,
            };
            if due {
                Ok(Some(sow_now(annual_reset_with(current, &bio, &params)?)?))
            } else {
                Ok(None)
            }
        },
    )
}

/// A habitat assembled around one plant chamber ([`assemble_chamber_habitat`]).
pub struct ChamberHabitat {
    /// The starting state: the plants' stocks, the chamber's, the habitat's; the plants' values.
    pub state: State,
    /// The plant registry, on the plant step.
    pub plant_registry: Registry,
    /// The fast registry: the habitat's flows, the chamber's, the crop's gas exchange.
    pub fast_registry: Registry,
}

/// The chamber of `scenario` wired to a habitat through `ports`, with `habitat_stocks` and
/// `habitat_flows` beside it, in the sealed station's sequence: the latent-heat books, the gas
/// exchange on the fast step, then the last wrappers (the plants read the chamber; both
/// registries on the sowing clock). The habitat's stock and flow ids must be disjoint from the
/// chamber's; each port must name an energy stock of the habitat (`space` only under
/// [`ChamberSurroundings::Space`]).
///
/// ⚠ The sealed station does not call this: its fast registry interleaves its radiator and
/// harvest between the chamber's flows and the coil, and is kept byte-for-byte in that order.
/// The SEQUENCE is the same, and its parts are the same functions.
#[allow(clippy::too_many_arguments)]
pub fn assemble_chamber_habitat(
    scenario: &SealedStationScenario,
    lamp: &LampParams,
    thermal_params: &ThermalParams,
    surroundings: ChamberSurroundings,
    ports: ChamberPorts<'_>,
    habitat_stocks: Vec<Stock>,
    habitat_flows: Vec<Box<dyn Flow>>,
    rng_seed: u64,
) -> Result<ChamberHabitat, SimError> {
    let chamber = crate::params::chamber();
    require_step_within_response(scenario.cabin_dt, &chamber)?;
    let plants = chamber_plants(scenario)?;
    let mut stocks = plants.stocks;
    for stock in chamber_start_stocks(scenario, &chamber, surroundings)? {
        stocks.insert(stock.id.clone(), stock);
    }
    for stock in habitat_stocks {
        if stocks.contains_key(&stock.id) {
            return Err(SimError::Validation(format!(
                "stock id {:?} is the plant chamber's own; a habitat may wire to it but not \
                 declare it",
                stock.id
            )));
        }
        stocks.insert(stock.id.clone(), stock);
    }
    let space_used = matches!(surroundings, ChamberSurroundings::Space { .. });
    for (port, id, used) in [
        ("power", ports.power, true),
        ("heat_sink", ports.heat_sink, true),
        ("space", ports.space, space_used),
    ] {
        if !used {
            continue;
        }
        let stock = stocks.get(id).ok_or_else(|| {
            SimError::Validation(format!(
                "the chamber's {port} port names {id:?}, which is no stock of the habitat"
            ))
        })?;
        if stock.quantity != Quantity::Energy {
            return Err(SimError::Validation(format!(
                "the chamber's {port} port must be an energy stock; {id:?} holds {:?}",
                stock.quantity
            )));
        }
    }
    let state = State::new(0, stocks.clone(), rng_seed, plants.aux)?;
    let (state, plant_registry) = crate::latent_heat::book_latent_heat(state, plants.registry)?;
    let mut seen: BTreeSet<String> = plant_registry
        .flows()
        .iter()
        .map(|f| f.id().to_string())
        .collect();
    let mut fast_flows = habitat_flows;
    fast_flows.extend(chamber_flows(
        lamp,
        &chamber,
        thermal_params,
        surroundings,
        ports,
    ));
    fast_flows.push(chamber_latent_heat_flow(ports));
    for flow in &fast_flows {
        if !seen.insert(flow.id().to_string()) {
            return Err(SimError::Validation(format!(
                "flow id {:?} appears twice in the habitat around the plant chamber",
                flow.id()
            )));
        }
    }
    let fast_registry = Registry::flows_only(fast_flows, &stocks)?;
    require_one_plant_step_per_group(scenario.steps_per_day, scenario.bio_steps_per_day)?;
    let (plant_registry, fast_registry) = gas_exchange_on_fast_step(
        &stocks,
        plant_registry,
        fast_registry,
        &weather_shared(&scenario.bio),
    )?;
    let (plant_registry, fast_registry) =
        crate::sealed::wrap_last(plant_registry, fast_registry, &stocks, scenario)?;
    Ok(ChamberHabitat {
        state,
        plant_registry,
        fast_registry,
    })
}
