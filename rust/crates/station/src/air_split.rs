//! **LAB-ONLY** — the crew and the plants in two air spaces, joined by a fan (Step 3c slice 2a,
//! `docs/plans/post-roadmap-room-temperature.md` §11 and §13).
//!
//! Nothing in the reference calls anything here: not a runner, not the session, not the
//! bridge, not a golden. The frozen sealed station keeps **shared air** — the crew breathes the
//! biosphere's own CO₂ and O₂ pools. This module builds the other topology the user asked for
//! (2026-10-03): the cabin gets its own CO₂, O₂ and inert fill at the cabin's
//! [`CABIN_AIR_MOL`]; the crew, the CO₂ scrubber and the O₂ makeup act on those (the fast flow
//! list is the frozen one, re-pointed — [`crate::sealed::sealed_fast_flows`], one copy); the
//! plants keep the biosphere's pools in a chamber of their own size; and [`AirExchange`] moves
//! air between the two. One temperature, no heat (slice 2a only).
//!
//! # The fan's form
//!
//! The fan moves `Q` mol of air per second each way (equal volumes at the same temperature and
//! pressure). Every species `i` crosses at `Q · dt · (a_i,cabin / cap_cabin − a_i,chamber /
//! cap_chamber)`, where `a` is the stock's amount and `cap` the room's reference air (mol) —
//! the amount per mol of air **at reference pressure**, i.e. a concentration, NOT a live mole
//! fraction. ⚠ Do not "simplify" this to `a_i / Σ a`: at fixed composition a mole fraction
//! does not move when a room loses pressure, so a breach on one side would never draw air from
//! the other (the atmosphere build's own trap, `docs/log/atmosphere.md`). The **inert fill**
//! crosses too, which is what lets pressure even out. The form is linear in each amount, so it
//! holds in any stock unit (mol of C, mol of O₂, kg of N₂, kg of water) without conversion.
//!
//! `Q` is **DESIGN** — no inter-room ventilation rate was found (§11): BVAD Table 4-73's
//! "Ventilation" row is an air speed. The plant chamber's size is BVAD Table 4-88's 0.67 m³
//! of shoot zone per m² of crop (page image owed), an upper bound on free air.

use domains::biosphere::science::N2_MOLAR_MASS_KG_PER_MOL;
use domains::biosphere::stocks::{CARBON_POOL, CHAMBER_INERT, O2_POOL, WATER_VAPOR};
use domains::biosphere::system::weather_shared;
use domains::crew::{CrewParams, FECAL_WASTE};
use domains::eclss::{EclssParams, CABIN_CO2, CABIN_H2O, CABIN_O2, ECLSS_DOMAIN};
use domains::power::ChargeParams;
use domains::thermal::ThermalParams;
use simcore::environment::Environment;
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::State;

use crate::flows::{HarvestParams, LampParams, WaterRecoveryParams};
use crate::gas_exchange::{
    gas_exchange_on_fast_step, require_one_plant_step_per_group, GasExchangeStep,
};
use crate::scenario::SealedStationScenario;
use crate::sealed::{build_sealed_station_at, sealed_fast_flows, CabinAir};
use crate::stocks::{co2_composition, gas_pool, o2_composition, simple_pool};

/// The cabin's reference air (mol) — the frozen sealed station's `chamber_air_capacity_mol`,
/// kept so `eclss.yaml`'s absolute `o2_setpoint` (0.21 × 9500) stays true for the cabin.
pub const CABIN_AIR_MOL: f64 = 9500.0;

/// The cabin's own inert fill (N₂ + Ar lumped), kg of N₂ — the cabin twin of
/// `biosphere.chamber_inert`.
pub const CABIN_INERT: &str = "eclss.cabin_inert";

/// The fan's flow id.
pub const AIR_EXCHANGE: &str = "station.air_exchange";

/// The plant chamber's reference air (mol) at BVAD Table 4-88's 0.67 m³ per m² of crop, for
/// the sealed station's 1 m²: `0.67 · P/(R·T)` at 22 °C and 101.325 kPa (41.29 mol/m³).
pub const BVAD_CHAMBER_AIR_MOL: f64 = 27.663966;

/// The separate-air setup: the plant chamber's size, the fan, and whether vapour crosses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AirSplit {
    /// The plant chamber's reference air (mol).
    pub chamber_air_mol: f64,
    /// The fan's air exchange, mol of air per second each way (DESIGN).
    pub fan_mol_per_s: f64,
    /// Whether water vapour crosses with the air (§13a's W4 control turns it off).
    pub vapour_crosses: bool,
    /// Which step the crop's gas exchange is taken on (§16): the plant step starves a small
    /// chamber by construction, the minute step lets the fan refill it as the crop draws.
    pub gas_exchange: GasExchangeStep,
}

/// The fan: every species crosses by its concentration difference ([module docs](self)).
pub struct AirExchange {
    pub id: String,
    /// `(cabin stock, chamber stock)` pairs, in a fixed order.
    pub pairs: Vec<(String, String)>,
    pub cabin_air_mol: f64,
    pub chamber_air_mol: f64,
    pub fan_mol_per_s: f64,
}

impl Flow for AirExchange {
    fn type_name(&self) -> &'static str {
        "AirExchange"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let mut legs = Vec::with_capacity(2 * self.pairs.len());
        for (cabin, chamber) in &self.pairs {
            let a_cabin = amount(snapshot, cabin)?;
            let a_chamber = amount(snapshot, chamber)?;
            let to_chamber = self.fan_mol_per_s
                * dt
                * (a_cabin / self.cabin_air_mol - a_chamber / self.chamber_air_mol);
            legs.push(Leg::new(cabin.clone(), -to_chamber)?);
            legs.push(Leg::new(chamber.clone(), to_chamber)?);
        }
        FlowResult::new(legs)
    }
}

fn amount(state: &State, id: &str) -> Result<f64, SimError> {
    state
        .stocks
        .get(id)
        .map(|s| s.amount)
        .ok_or_else(|| SimError::Reference(format!("the fan reads {id:?}, which is not a stock")))
}

/// The sealed-station scenario with the plants' chamber resized to `chamber_air_mol`, charged
/// at the cabin's composition (CO₂ and O₂ scaled by `chamber_air_mol / CABIN_AIR_MOL`).
pub fn split_scenario(scenario: &SealedStationScenario, split: &AirSplit) -> SealedStationScenario {
    let scale = split.chamber_air_mol / scenario.bio.chamber_air_capacity_mol;
    let mut s = *scenario;
    s.bio.chamber_air_capacity_mol = split.chamber_air_mol;
    s.bio.chamber_co2_mol0 = scenario.bio.chamber_co2_mol0 * scale;
    s.bio.chamber_o2_mol0 = scenario.bio.chamber_o2_mol0 * scale;
    s
}

/// Build the separate-air station: `(state, bio_reg, fast_reg)`, from `scenario` (the
/// shared-air one; the chamber is resized here by [`split_scenario`]). Run it with the
/// RESIZED scenario's resolvers and re-sow hook. `with_harvest` / `close_feces` off, as the
/// frozen Tier-2 scope.
#[allow(clippy::too_many_arguments)]
pub fn build_split_station(
    charge: &ChargeParams,
    thermal_params: &ThermalParams,
    crew: &CrewParams,
    eclss: &EclssParams,
    recovery: &WaterRecoveryParams,
    lamp: &LampParams,
    harvest: &HarvestParams,
    scenario: &SealedStationScenario,
    split: &AirSplit,
) -> Result<(State, Registry, Registry), SimError> {
    if scenario.bio.chamber_air_capacity_mol != CABIN_AIR_MOL {
        return Err(SimError::Validation(format!(
            "build_split_station: the cabin keeps the shared room's air ({CABIN_AIR_MOL} mol) so \
             the O₂ setpoint stays true; this scenario's room is {} mol",
            scenario.bio.chamber_air_capacity_mol
        )));
    }
    let resized = split_scenario(scenario, split);
    // ⚠ The PLANT-STEP base, always: this build discards the shared fast registry and keeps the
    // plant one, so a minute-step base would throw the crop's carbon budget away with it. The
    // minute step is applied below, onto THIS build's own fast registry.
    let (state, bio_reg, _shared_fast) = build_sealed_station_at(
        charge,
        thermal_params,
        crew,
        eclss,
        recovery,
        lamp,
        harvest,
        &resized,
        false,
        false,
        GasExchangeStep::PlantStep,
    )?;

    // The cabin's own air, charged with the shared room's starting gases.
    let co2 = scenario.bio.chamber_co2_mol0;
    let o2 = scenario.bio.chamber_o2_mol0;
    let inert_mol = CABIN_AIR_MOL - co2 - o2;
    let mut stocks = state.stocks;
    for s in [
        gas_pool(CABIN_CO2, Quantity::Carbon, co2, co2_composition())?,
        gas_pool(CABIN_O2, Quantity::Oxygen, o2, o2_composition())?,
        simple_pool(
            CABIN_INERT,
            ECLSS_DOMAIN,
            Quantity::Nitrogen,
            inert_mol * N2_MOLAR_MASS_KG_PER_MOL,
        )?,
    ] {
        if stocks.insert(s.id.clone(), s).is_some() {
            return Err(SimError::Validation(
                "build_split_station: a cabin gas stock already exists".into(),
            ));
        }
    }
    let state = State::new(state.n, stocks, state.rng_seed, state.aux)?;

    let mut pairs = vec![
        (CABIN_CO2.to_string(), CARBON_POOL.to_string()),
        (CABIN_O2.to_string(), O2_POOL.to_string()),
        (CABIN_INERT.to_string(), CHAMBER_INERT.to_string()),
    ];
    if split.vapour_crosses {
        pairs.push((CABIN_H2O.to_string(), WATER_VAPOR.to_string()));
    }
    let mut fast_flows = sealed_fast_flows(
        charge,
        thermal_params,
        crew,
        eclss,
        recovery,
        lamp,
        harvest,
        CabinAir {
            co2: CABIN_CO2,
            o2: CABIN_O2,
        },
        FECAL_WASTE,
        false,
    );
    fast_flows.push(Box::new(AirExchange {
        id: AIR_EXCHANGE.to_string(),
        pairs,
        cabin_air_mol: CABIN_AIR_MOL,
        chamber_air_mol: split.chamber_air_mol,
        fan_mol_per_s: split.fan_mol_per_s,
    }));
    let fast_reg = Registry::flows_only(fast_flows, &state.stocks)?;
    match split.gas_exchange {
        GasExchangeStep::PlantStep => Ok((state, bio_reg, fast_reg)),
        GasExchangeStep::Minute => {
            require_one_plant_step_per_group(resized.steps_per_day, resized.bio_steps_per_day)?;
            let (bio_reg, fast_reg) = gas_exchange_on_fast_step(
                &state.stocks,
                bio_reg,
                fast_reg,
                &weather_shared(&resized.bio),
            )?;
            Ok((state, bio_reg, fast_reg))
        }
    }
}

/// The cabin's gas books, for readers: `(co2 mol, o2 mol, inert kg)`.
pub fn cabin_gases(state: &State) -> Result<(f64, f64, f64), SimError> {
    Ok((
        amount(state, CABIN_CO2)?,
        amount(state, CABIN_O2)?,
        amount(state, CABIN_INERT)?,
    ))
}
