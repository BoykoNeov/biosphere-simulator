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

use std::collections::{BTreeMap, HashMap};

use domains::biosphere::flows::Irrigation;
use domains::biosphere::science::N2_MOLAR_MASS_KG_PER_MOL;
use domains::biosphere::science::{fraction_transpirable, transpirable_capacity};
use domains::biosphere::stocks::{
    CARBON_POOL, CHAMBER_INERT, IRRIGATION_VAR, O2_POOL, RN_VAR, ROOTED_DEPTH, SOIL_WATER,
    TEMP_VAR, WATER_VAPOR,
};
use domains::biosphere::system::weather_shared;
use domains::crew::{CrewParams, FECAL_WASTE, WATER_STORE};
use domains::eclss::{EclssParams, CABIN_CO2, CABIN_H2O, CABIN_O2, ECLSS_DOMAIN};
use domains::power::ChargeParams;
use domains::thermal::ThermalParams;
use simcore::auxiliary::AuxProcess;
use simcore::environment::Environment;
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::ids::StockId;
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::{State, Stock};

use crate::flows::{HarvestParams, LampParams, WaterRecoveryParams};
use crate::gas_exchange::{
    gas_exchange_on_fast_step, require_one_plant_step_per_group, window_key, GasExchangeStep,
    OnFastStep, PLANT_WINDOW_RECORDER,
};
use crate::scenario::SealedStationScenario;
use crate::sealed::{build_sealed_station_unread, sealed_fast_flows, CabinAir};
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
    /// Whether the plants are watered from the crew's store (§18b): the open field's own
    /// refill (`Irrigation`, [F] Eqn 14.8), its source re-pointed at `crew.water_store`, gated
    /// by FAO-56's depletion trigger ([`TriggeredWatering`]).
    pub watering: bool,
    /// Which step the plants' water loss is taken on (§20): `Minute` moves `Transpiration` and
    /// the chamber's `Condensation` onto the fast step together ([`water_on_fast_step`]). Needs
    /// `gas_exchange: Minute`, whose recorder holds the window's temperature.
    pub transpiration: GasExchangeStep,
}

/// The chamber's water-loss pair (§20): transpiration's split between air and condensate counts
/// the condenser's same-step draw, so the two move together or not at all.
pub const WATER_LOSS_FLOWS: [&str; 2] = ["biosphere.transpiration", "biosphere.condensation"];

/// The forcings the water-loss pair reads, as recorded by the plant step: temperature (by
/// [`PlantWindowRecorder`](crate::gas_exchange::PlantWindowRecorder)) and net radiation (by [`NetRadiationRecorder`]).
pub const WATER_LOSS_WINDOW: [&str; 2] = [TEMP_VAR, RN_VAR];

/// The lab recorder's aux-process id.
pub const NET_RADIATION_RECORDER: &str = "station.plant_window_rn";

/// **LAB-ONLY** — the plant-step aux process that records the window's net radiation under
/// [`window_key`]`(RN_VAR)`, as [`PlantWindowRecorder`](crate::gas_exchange::PlantWindowRecorder) records PAR and temperature. A recorder of
/// its own so the reference's aux keys do not change; it records net radiation ONLY, since the
/// aux channel is additive and a second writer of the temperature key would double it.
pub struct NetRadiationRecorder;

impl AuxProcess for NetRadiationRecorder {
    fn type_name(&self) -> &'static str {
        "NetRadiationRecorder"
    }

    fn id(&self) -> &str {
        NET_RADIATION_RECORDER
    }

    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        _dt: f64,
    ) -> Result<BTreeMap<String, f64>, SimError> {
        let key = window_key(RN_VAR);
        let old = snapshot.aux.get(&key).copied().unwrap_or(0.0);
        Ok(BTreeMap::from([(key, env.get(RN_VAR)? - old)]))
    }
}

/// Move the plants' water loss onto the fast step of a build whose gas exchange is already there
/// (§20): [`WATER_LOSS_FLOWS`] out of the plant registry and wrapped into the fast one, reading
/// [`WATER_LOSS_WINDOW`]; [`NetRadiationRecorder`] added to the plant step. An error unless the
/// plant step already carries [`PlantWindowRecorder`](crate::gas_exchange::PlantWindowRecorder) and both flows are found.
pub fn water_on_fast_step(
    stocks: &BTreeMap<StockId, Stock>,
    bio_reg: Registry,
    fast_reg: Registry,
    shared: &HashMap<String, StockId>,
) -> Result<(Registry, Registry), SimError> {
    let (flows, mut bio_aux) = bio_reg.into_parts();
    if !bio_aux.iter().any(|a| a.id() == PLANT_WINDOW_RECORDER) {
        return Err(SimError::Validation(
            "water loss on the fast step reads the window's temperature, which only the minute \
             gas exchange's recorder holds: set gas_exchange to Minute"
                .into(),
        ));
    }
    let (moved, rest): (Vec<_>, Vec<_>) = flows
        .into_iter()
        .partition(|f| WATER_LOSS_FLOWS.contains(&f.id()));
    if moved.len() != WATER_LOSS_FLOWS.len() {
        let found: Vec<&str> = moved.iter().map(|f| f.id()).collect();
        return Err(SimError::Validation(format!(
            "water loss on the fast step: the plant registry holds {found:?} of \
             {WATER_LOSS_FLOWS:?}; they move together or not at all"
        )));
    }
    bio_aux.push(Box::new(NetRadiationRecorder));
    let (mut fast_flows, fast_aux) = fast_reg.into_parts();
    for inner in moved {
        fast_flows.push(Box::new(OnFastStep::reading(
            inner,
            shared.clone(),
            &WATER_LOSS_WINDOW,
        )));
    }
    Ok((
        Registry::new(rest, stocks, bio_aux)?,
        Registry::new(fast_flows, stocks, fast_aux)?,
    ))
}

/// The watering flow's id (§18).
pub const WATERING: &str = "station.watering";

/// FAO-56 (Allen et al. 1998, FAO Irrigation and Drainage Paper 56) **Table 22**, "Ranges of
/// maximum effective rooting depth (Zr), and soil water depletion fraction for no stress (p)":
/// spring and winter wheat, **p = 0.55** (`RAW = p TAW`). The watering trigger is `1 − p`.
pub const FAO56_WHEAT_DEPLETION_FRACTION: f64 = 0.55;

/// Watering on a depletion trigger (§18b): the inner refill runs only on a plant step whose root
/// zone has fallen below `trigger_ftsw` of its transpirable capacity, and gives NO legs
/// otherwise.
///
/// ⚠ Why the gate exists: §18a measured the bare refill — the field's rule — watering plants
/// that were not short. It refilled the root zone every step the crop's own transpiration dipped
/// it, ahead of the chamber's recycled condensate, and the plants' loop gained 30–45 kg of the
/// crew's water a season. A trigger delivers nothing while the room's own loop keeps up.
/// ⚠ No latch: with the refill capped per step, the zone is HELD at the trigger, not refilled to
/// the top.
pub struct TriggeredWatering {
    /// The refill: `Irrigation`, sourced from the crew's store.
    pub inner: Irrigation,
    /// The fill `FTSW = ATSW / TTSW` below which it waters (`1 − p`).
    pub trigger_ftsw: f64,
}

impl Flow for TriggeredWatering {
    fn type_name(&self) -> &'static str {
        "TriggeredWatering"
    }

    fn id(&self) -> &str {
        self.inner.id()
    }

    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let i = &self.inner;
        let capacity = transpirable_capacity(
            snapshot
                .aux
                .get(&i.rooted_depth_aux)
                .copied()
                .unwrap_or(0.0),
            i.soil_extractable_water,
            i.ground_area,
        );
        let ftsw = fraction_transpirable(amount(snapshot, &i.soil_water)?, capacity);
        if ftsw < self.trigger_ftsw {
            i.evaluate(snapshot, env, dt)
        } else {
            Ok(FlowResult::empty())
        }
    }
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
    // ⚠ And UNREAD (slice 3a, §24b): the plants are wrapped to read the chamber at the very end,
    // after this build's own moves — a flow wrapped before it is moved meets its window's `temp`
    // and is refused.
    let (state, bio_reg, _shared_fast) = build_sealed_station_unread(
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
        crate::chamber::ChamberSurroundings::Outdoor,
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

    // §18: watered from the crew's supply — the field's top-up, on the plant step (its rate is
    // per day, and it reads the rooted depth the plant step advances).
    let bio_reg = if split.watering {
        let (mut flows, aux) = bio_reg.into_parts();
        flows.push(Box::new(TriggeredWatering {
            inner: Irrigation {
                id: WATERING.to_string(),
                water_source: WATER_STORE.to_string(),
                soil_water: SOIL_WATER.to_string(),
                irrigation_var: IRRIGATION_VAR.to_string(),
                ground_area: resized.bio.ground_area,
                rooted_depth_aux: ROOTED_DEPTH.to_string(),
                soil_extractable_water: resized.bio.soil_extractable_water,
            },
            trigger_ftsw: 1.0 - FAO56_WHEAT_DEPLETION_FRACTION,
        }));
        Registry::new(flows, &state.stocks, aux)?
    } else {
        bio_reg
    };

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
        &crate::params::chamber(),
        crate::chamber::ChamberSurroundings::Outdoor,
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
    let shared = weather_shared(&resized.bio);
    let (bio_reg, fast_reg) = match split.gas_exchange {
        GasExchangeStep::PlantStep => (bio_reg, fast_reg),
        GasExchangeStep::Minute => {
            require_one_plant_step_per_group(resized.steps_per_day, resized.bio_steps_per_day)?;
            gas_exchange_on_fast_step(&state.stocks, bio_reg, fast_reg, &shared)?
        }
    };
    let (bio_reg, fast_reg) = match split.transpiration {
        GasExchangeStep::PlantStep => (bio_reg, fast_reg),
        GasExchangeStep::Minute => water_on_fast_step(&state.stocks, bio_reg, fast_reg, &shared)?,
    };
    let (bio_reg, fast_reg) = crate::sealed::wrap_last(bio_reg, fast_reg, &state.stocks, &resized)?;
    Ok((state, bio_reg, fast_reg))
}

/// The cabin's gas books, for readers: `(co2 mol, o2 mol, inert kg)`.
pub fn cabin_gases(state: &State) -> Result<(f64, f64, f64), SimError> {
    Ok((
        amount(state, CABIN_CO2)?,
        amount(state, CABIN_O2)?,
        amount(state, CABIN_INERT)?,
    ))
}
