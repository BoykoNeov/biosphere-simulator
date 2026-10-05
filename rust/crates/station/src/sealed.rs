//! The sealed station — the port of `station.sealed` (P6.7 / P7.5).
//!
//! Composes **every** Phase-6 shared-stock seam over one shared stock dict + two registries
//! (biosphere-slow / everything-fast) and runs it multi-year. The fast registry holds the 5
//! cabin flows re-pointed at the biosphere gas pools (the greenhouse reverse seam) +
//! `SolarCharge`/`LoadDraw` (waste heat → `thermal.node`, the Step-1 inward move), `Lamp`
//! (waste heat → `thermal.chamber`, the plant chamber, since 2026-10-05) and `ChamberCooling`
//! (chamber → node), plus `RadiatorReject`, `WaterRecovery` (and `Harvest` iff
//! `with_harvest`); the biosphere
//! registry is `build_season` verbatim, re-sown yearly by `annual_reset` via the driver's
//! `slow_reset` hook. `with_harvest` / `close_feces` default **off** (the Tier-2 scope).
//! Tier-2 (FvCB + `T⁴`). Euler-only.

use std::collections::BTreeMap;

use domains::biosphere::stocks::{
    CARBON_POOL, DAYLENGTH_VAR, LITTER_CARBON, O2_POOL, PAR_VAR, RN_VAR, ROOTED_DEPTH, STORAGE_C,
    THERMAL_TIME,
};
use domains::biosphere::system::{annual_reset, build_season, weather_forcings, weather_shared};
use domains::biosphere::STEPS_PER_DAY;
use domains::crew::{
    CrewParams, WaterBalance, FECAL_WASTE, FOOD_INTAKE_VAR, FOOD_STORE, WATER_BALANCE,
    WATER_INTAKE_VAR, WATER_STORE,
};
use domains::eclss::{
    CO2Scrubber, Condenser, EclssParams, O2Makeup, CABIN_H2O, CO2_REMOVED, CO2_SCRUBBER, CONDENSER,
    O2_MAKEUP, O2_SUPPLY,
};
use domains::power::{
    balanced_load_w, battery_stock, daily_solar_energy, ChargeParams, LoadDraw, SolarCharge,
    BATTERY, LOAD_DRAW, LOAD_POWER_VAR, SOLAR_CHARGE, SOLAR_POWER_VAR, SOLAR_SOURCE,
};
use domains::thermal::{
    equilibrium_temperature, node_stock, RadiatorReject, ThermalParams, NODE, RADIATOR_REJECT,
    SPACE,
};
use simcore::boundary;
use simcore::environment::{constant, SourceResolver};
use simcore::error::SimError;
use simcore::events::Event;
use simcore::flow::Flow;
use simcore::integrator::EulerIntegrator;
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::{State, Stock};

use crate::chamber::{
    chamber_heat0, chamber_stock, require_step_within_response, ChamberCooling, ChamberParams,
    CHAMBER, CHAMBER_COOLING,
};
use crate::driver::{run_master_day, OwnedResetHook};
use crate::flows::{
    CrewRespiration, Harvest, HarvestParams, Lamp, LampParams, WaterRecovery, WaterRecoveryParams,
    CREW_RESPIRATION, HARVEST, LAMP, LAMP_POWER_VAR, PAR_PHOTON_ENERGY_J_PER_UMOL, WATER_RECOVERY,
};
use crate::gas_exchange::{
    gas_exchange_on_fast_step, require_one_plant_step_per_group, GasExchangeStep,
};
use crate::lighting::{lamp_light_path, lamp_net_radiation_path, LIGHT_USED};
use crate::scenario::SealedStationScenario;
use crate::stocks::{
    cabin_h2o_stock, co2_composition, food_store_stock, gas_boundary, o2_composition,
    water_store_stock,
};
use crate::water::{recovered_water_pool, BRINE, RECOVERED_WATER};

/// The constant daily-average solar supply (W) the fast Power flows read.
fn mean_solar_power(scenario: &SealedStationScenario) -> f64 {
    let ph = &scenario.power;
    daily_solar_energy(ph) / (ph.steps_per_day as f64 * ph.dt_seconds)
}

/// The constant daily-average lamp draw (W): `lamp_power_w · photoperiod / 24`.
pub(crate) fn lighting_average_power(scenario: &SealedStationScenario) -> f64 {
    scenario.lamp_power_w * scenario.photoperiod_hours as f64 / 24.0
}

/// The on-window PAR photon flux the lamp delivers (µmol m⁻² s⁻¹).
fn sealed_lamp_par(lamp: &LampParams, scenario: &SealedStationScenario) -> f64 {
    lamp.photon_efficacy * scenario.lamp_power_w / scenario.bio.ground_area
}

/// The node's initial heat `Q_eq = C·(T_eq − T_space)` (J), set by all forced dissipation.
///
/// Op-order mirrors Python `sealed_node_heat`: charge-conversion loss `(1−η_c)·solar_avg` +
/// the 100%-dissipative `LoadDraw` (`balanced_load`) + the lamp waste heat
/// `(1−η_lamp)·lamp_avg` (the radiant η_lamp leg leaves as PAR, not to the node).
pub fn sealed_node_heat(
    charge: &ChargeParams,
    thermal_params: &ThermalParams,
    lamp: &LampParams,
    scenario: &SealedStationScenario,
) -> f64 {
    let solar_avg = mean_solar_power(scenario);
    let load_w = balanced_load_w(charge, &scenario.power);
    let lamp_avg = lighting_average_power(scenario);
    let eta_lamp = lamp.photon_efficacy * PAR_PHOTON_ENERGY_J_PER_UMOL;
    let heat_w =
        (1.0 - charge.charge_efficiency) * solar_avg + load_w + (1.0 - eta_lamp) * lamp_avg;
    let t_eq = equilibrium_temperature(thermal_params, heat_w);
    thermal_params.heat_capacity * (t_eq - thermal_params.space_temperature)
}

/// The lamp's averaged waste heat (W), `(1 − η_lamp)·lamp_avg` — the chamber's only heat
/// input in slice 2b-i (`docs/plans/post-roadmap-room-temperature.md` §23e).
pub fn lamp_waste_heat_w(lamp: &LampParams, scenario: &SealedStationScenario) -> f64 {
    let eta_lamp = lamp.photon_efficacy * PAR_PHOTON_ENERGY_J_PER_UMOL;
    (1.0 - eta_lamp) * lighting_average_power(scenario)
}

/// Assemble the fully-coupled sealed station: `(state, bio_reg, fast_reg)`, with the crop's
/// gas exchange on the cabin's minute step (the reference since 2026-10-03,
/// `docs/plans/post-roadmap-room-temperature.md` §17; [`crate::gas_exchange`]).
#[allow(clippy::too_many_arguments)]
pub fn build_sealed_station(
    charge: &ChargeParams,
    thermal_params: &ThermalParams,
    crew: &CrewParams,
    eclss: &EclssParams,
    recovery: &WaterRecoveryParams,
    lamp: &LampParams,
    harvest: &HarvestParams,
    scenario: &SealedStationScenario,
    with_harvest: bool,
    close_feces: bool,
) -> Result<(State, Registry, Registry), SimError> {
    build_sealed_station_at(
        charge,
        thermal_params,
        crew,
        eclss,
        recovery,
        lamp,
        harvest,
        scenario,
        with_harvest,
        close_feces,
        GasExchangeStep::Minute,
    )
}

/// [`build_sealed_station`] with the crop's gas exchange on the step `gas` names.
/// `GasExchangeStep::PlantStep` is the form until 2026-10-03, kept for the lab instruments that
/// reproduce records made with it (and for the separate-air build's own base).
#[allow(clippy::too_many_arguments)]
pub fn build_sealed_station_at(
    charge: &ChargeParams,
    thermal_params: &ThermalParams,
    crew: &CrewParams,
    eclss: &EclssParams,
    recovery: &WaterRecoveryParams,
    lamp: &LampParams,
    harvest: &HarvestParams,
    scenario: &SealedStationScenario,
    with_harvest: bool,
    close_feces: bool,
    gas: GasExchangeStep,
) -> Result<(State, Registry, Registry), SimError> {
    // --- biosphere (slow) — build_season verbatim ---
    let (bio_state, bio_reg) = build_season(&scenario.bio)?;
    let bio_stocks = bio_state.stocks.clone();

    // --- fast-domain stocks (cabin + crew + Power/Thermal), biosphere-disjoint ---
    let fecal_target = if close_feces {
        LITTER_CARBON
    } else {
        FECAL_WASTE
    };
    let node0 = sealed_node_heat(charge, thermal_params, lamp, scenario);
    let chamber = crate::params::chamber();
    require_step_within_response(scenario.cabin_dt, &chamber)?;
    let mut fast_seq = vec![
        food_store_stock(scenario.cabin.food_store0)?,
        water_store_stock(scenario.cabin.water_store0)?,
        cabin_h2o_stock(scenario.cabin.cabin_h2o_0)?,
        gas_boundary(O2_SUPPLY, Quantity::Oxygen, o2_composition(), true)?,
        gas_boundary(CO2_REMOVED, Quantity::Carbon, co2_composition(), false)?,
        recovered_water_pool()?,
        boundary::sink(BRINE.to_string(), Quantity::Water, 0.0)?,
    ];
    if !close_feces {
        fast_seq.push(boundary::sink(
            FECAL_WASTE.to_string(),
            Quantity::Carbon,
            0.0,
        )?);
    }
    fast_seq.push(battery_stock(scenario.battery0)?);
    fast_seq.push(boundary::source(
        SOLAR_SOURCE.to_string(),
        Quantity::Energy,
        0.0,
        true,
    )?);
    fast_seq.push(node_stock(node0)?);
    fast_seq.push(chamber_stock(chamber_heat0(
        &chamber,
        lamp_waste_heat_w(lamp, scenario),
    ))?);
    fast_seq.push(boundary::sink(SPACE.to_string(), Quantity::Energy, 0.0)?);
    fast_seq.push(boundary::sink(
        LIGHT_USED.to_string(),
        Quantity::Energy,
        0.0,
    )?);

    let mut fast_stocks: BTreeMap<String, Stock> = BTreeMap::new();
    for s in fast_seq {
        fast_stocks.insert(s.id.clone(), s);
    }
    for id in fast_stocks.keys() {
        if bio_stocks.contains_key(id) {
            return Err(SimError::Validation(format!(
                "sealed-station stock-id collision between the biosphere and the fast \
                 domain: {id:?} (the two stock sets must be disjoint)"
            )));
        }
    }
    let mut stocks = bio_stocks;
    for (id, s) in fast_stocks {
        stocks.insert(id, s);
    }
    let state = State::new(
        0,
        stocks.clone(),
        0,
        BTreeMap::from([
            (THERMAL_TIME.to_string(), 0.0),
            // ⚠ ROOTED_DEPTH is LOAD-BEARING and was MISSING from all three station
            // assemblies (both ports) until 2026-08-12. `build_season` has seeded it
            // with the cited sowing depth since the root-depth build; the station
            // builders assembled their own aux and silently started the crop at depth 0.
            // Invisible while the depth gate was inert; fatal once stress divides by
            // `TTSW = depth * EXTR * rho * A`.
            (ROOTED_DEPTH.to_string(), scenario.bio.rooted_depth0),
        ]),
    )?;

    // --- fast flows (the crew breathes the biosphere's own gas pools: shared air) ---
    let fast_flows = sealed_fast_flows(
        charge,
        thermal_params,
        crew,
        eclss,
        recovery,
        lamp,
        harvest,
        &chamber,
        CabinAir {
            co2: CARBON_POOL, // the greenhouse seam: crew exhales into the bio CO₂
            o2: O2_POOL,      // the greenhouse seam: crew breathes the bio O₂
        },
        fecal_target,
        with_harvest,
    );
    let fast_reg = Registry::flows_only(fast_flows, &stocks)?;

    assert_flow_ids_disjoint(&bio_reg, &fast_reg)?;
    let (bio_reg, fast_reg) = match gas {
        GasExchangeStep::PlantStep => (bio_reg, fast_reg),
        GasExchangeStep::Minute => {
            require_one_plant_step_per_group(scenario.steps_per_day, scenario.bio_steps_per_day)?;
            gas_exchange_on_fast_step(&stocks, bio_reg, fast_reg, &weather_shared(&scenario.bio))?
        }
    };
    Ok((state, bio_reg, fast_reg))
}

/// The two gas pools the crew, the CO₂ scrubber and the O₂ makeup act on. In the frozen
/// sealed station they ARE the biosphere's (`CARBON_POOL` / `O2_POOL`, shared air); the lab
/// separate-air build ([`crate::air_split`]) points them at the cabin's own.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CabinAir<'a> {
    pub co2: &'a str,
    pub o2: &'a str,
}

/// The sealed station's fast registry, in its one canonical order. Extracted so the lab
/// separate-air build re-points the cabin gas without a second copy of this list.
#[allow(clippy::too_many_arguments)]
pub(crate) fn sealed_fast_flows(
    charge: &ChargeParams,
    thermal_params: &ThermalParams,
    crew: &CrewParams,
    eclss: &EclssParams,
    recovery: &WaterRecoveryParams,
    lamp: &LampParams,
    harvest: &HarvestParams,
    chamber: &ChamberParams,
    air: CabinAir<'_>,
    fecal_target: &str,
    with_harvest: bool,
) -> Vec<Box<dyn Flow>> {
    let mut fast_flows: Vec<Box<dyn Flow>> = vec![
        Box::new(CrewRespiration::new(
            CREW_RESPIRATION.to_string(),
            FOOD_STORE.to_string(),
            air.co2.to_string(),
            air.o2.to_string(),
            fecal_target.to_string(),
            crew.respired_carbon_fraction,
        )),
        Box::new(WaterBalance::new(
            WATER_BALANCE.to_string(),
            WATER_STORE.to_string(),
            CABIN_H2O.to_string(),
            RECOVERED_WATER.to_string(), // the Step-4 seam: urine → the recovery buffer
            *crew,
        )),
        Box::new(CO2Scrubber::new(
            CO2_SCRUBBER.to_string(),
            air.co2.to_string(),
            CO2_REMOVED.to_string(),
            *eclss,
        )),
        Box::new(Condenser::new(
            CONDENSER.to_string(),
            CABIN_H2O.to_string(),
            RECOVERED_WATER.to_string(), // the Step-4 seam: condensate → the buffer
            *eclss,
        )),
        Box::new(O2Makeup::new(
            O2_MAKEUP.to_string(),
            O2_SUPPLY.to_string(),
            air.o2.to_string(),
            *eclss,
        )),
        Box::new(WaterRecovery::new(
            WATER_RECOVERY.to_string(),
            RECOVERED_WATER.to_string(),
            WATER_STORE.to_string(), // the Step-4 seam: recovered water → the store
            BRINE.to_string(),
            *recovery,
        )),
        Box::new(SolarCharge::new(
            SOLAR_CHARGE.to_string(),
            SOLAR_SOURCE.to_string(),
            BATTERY.to_string(),
            NODE.to_string(), // the Step-1 inward seam: dissipation → the thermal node
            *charge,
        )),
        Box::new(LoadDraw::new(
            LOAD_DRAW.to_string(),
            BATTERY.to_string(),
            NODE.to_string(),
        )),
        Box::new(Lamp::new(
            LAMP.to_string(),
            BATTERY.to_string(),
            LIGHT_USED.to_string(),
            CHAMBER.to_string(), // lamp heat → the plant chamber (Step 3c slice 2b-i)
            *lamp,
        )),
        Box::new(ChamberCooling::new(
            CHAMBER_COOLING.to_string(),
            CHAMBER.to_string(),
            NODE.to_string(), // the chamber's cooler hands the lamp heat on to the node
            *chamber,
            *thermal_params,
        )),
        Box::new(RadiatorReject::new(
            RADIATOR_REJECT.to_string(),
            NODE.to_string(),
            SPACE.to_string(),
            *thermal_params,
        )),
    ];
    if with_harvest {
        fast_flows.push(Box::new(Harvest::new(
            HARVEST.to_string(),
            STORAGE_C.to_string(),
            FOOD_STORE.to_string(),
            *harvest,
        )));
    }
    fast_flows
}

/// Guard: the biosphere-slow and fast registries share no `FlowId`.
fn assert_flow_ids_disjoint(bio_reg: &Registry, fast_reg: &Registry) -> Result<(), SimError> {
    let bio_ids: std::collections::BTreeSet<&str> =
        bio_reg.flows().iter().map(|f| f.id()).collect();
    for flow in fast_reg.flows() {
        if bio_ids.contains(flow.id()) {
            return Err(SimError::Validation(format!(
                "sealed-station flow-id collision between the biosphere and the fast \
                 registries: {:?} (the two flow sets the driver steps together must be \
                 disjoint)",
                flow.id()
            )));
        }
    }
    Ok(())
}

/// The biosphere forcing: weather-driven, with `PAR`, net radiation (since 2026-10-05; the
/// weather's outdoor value before) and `daylength` from the lamp. The `weather` is tiled over
/// `scenario.years` seasons (so `_table` never end-clamps).
pub fn sealed_bio_resolver(
    lamp: &LampParams,
    scenario: &SealedStationScenario,
) -> Result<SourceResolver, SimError> {
    let mut forcings = weather_forcings(&scenario.bio, scenario.years)?;
    let photoperiod_s = scenario.photoperiod_hours as f64 * 3600.0;
    let par = sealed_lamp_par(lamp, scenario);
    forcings.insert(PAR_VAR.to_string(), lamp_light_path(par, photoperiod_s));
    // The crop's net radiation is the lamp's too, not the weather file's outdoor value (§21).
    forcings.insert(
        RN_VAR.to_string(),
        lamp_net_radiation_path(par, photoperiod_s),
    );
    forcings.insert(DAYLENGTH_VAR.to_string(), constant(photoperiod_s)?);
    SourceResolver::new(forcings, weather_shared(&scenario.bio))
}

/// The fast-domain forcing: crew intakes + lamp draw + constant solar/load.
pub fn sealed_fast_resolver(
    charge: &ChargeParams,
    scenario: &SealedStationScenario,
) -> Result<SourceResolver, SimError> {
    let mut forcings = std::collections::HashMap::new();
    forcings.insert(
        FOOD_INTAKE_VAR.to_string(),
        constant(scenario.cabin.food_intake_rate)?,
    );
    forcings.insert(
        WATER_INTAKE_VAR.to_string(),
        constant(scenario.cabin.water_intake_rate)?,
    );
    forcings.insert(
        LAMP_POWER_VAR.to_string(),
        constant(lighting_average_power(scenario))?,
    );
    forcings.insert(
        SOLAR_POWER_VAR.to_string(),
        constant(mean_solar_power(scenario))?,
    );
    forcings.insert(
        LOAD_POWER_VAR.to_string(),
        constant(balanced_load_w(charge, &scenario.power))?,
    );
    SourceResolver::new(forcings, std::collections::HashMap::new())
}

/// The sealed station's annual re-sow hook, **owned** (boxed) so a caller-driven
/// [`crate::session::SimSession`] can hold it. `annual_reset` fires on each season
/// boundary; `run_sealed` and the two-rate session build it via this same function so both
/// step the identical re-sow logic (the Phase-8 parity discipline).
///
/// ⚠ The period is in **steps**, not days — `n` is the slow domain's step count. Mirrors
/// the Python `season_steps = steps_for(scenario.season_days)`.
pub fn sealed_reset_hook(scenario: &SealedStationScenario) -> OwnedResetHook {
    let season_steps = (scenario.season_days * STEPS_PER_DAY) as u64;
    let bio = scenario.bio;
    Box::new(
        move |n: u64, current: &State| -> Result<Option<State>, SimError> {
            if n > 0 && n.is_multiple_of(season_steps) {
                Ok(Some(annual_reset(current, &bio)?))
            } else {
                Ok(None)
            }
        },
    )
}

/// The two-rate driver over the multi-year horizon, with the annual re-sow hook.
pub fn run_sealed(
    bio_integrator: &EulerIntegrator,
    fast_integrator: &EulerIntegrator,
    state: State,
    bio_resolver: &SourceResolver,
    fast_resolver: &SourceResolver,
    scenario: &SealedStationScenario,
) -> Result<(Vec<State>, u64, Vec<Event>), SimError> {
    let reset = sealed_reset_hook(scenario);
    run_master_day(
        bio_integrator,
        fast_integrator,
        state,
        bio_resolver,
        fast_resolver,
        scenario.days(),
        scenario.steps_per_day,
        scenario.bio_steps_per_day,
        scenario.bio_dt,
        scenario.cabin_dt,
        Some(&*reset),
    )
}
