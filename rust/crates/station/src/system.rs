//! The Power → Thermal heat-closure station — the port of `station.system` (P6.1 / P7.5).
//!
//! The first cross-domain seam: Power's dissipation legs (`SolarCharge`/`LoadDraw`) are
//! re-pointed from `boundary.waste_heat` into `thermal.node` (by passing `NODE`'s id where
//! the Power flows took `waste_heat`), Thermal's forced `HeatInput` stand-in is dropped
//! (Power's dissipation IS the input now), and `RadiatorReject` rejects the real load to
//! deep space — so `boundary.waste_heat` / `boundary.heat_source` are **absent** (the
//! redirection is structural). Single-quantity (ENERGY); the node's initial heat is DERIVED
//! from Power's actual dissipation ([`equilibrium_node_heat`]), so the run begins at the
//! attractor. Tier-2 (Power's half-sine `sin` + the `T⁴` radiator).

use std::collections::BTreeMap;

use domains::power::{
    balanced_load_w, battery_stock, daily_solar_energy, power_resolver, ChargeParams, LoadDraw,
    SolarCharge, BATTERY, LOAD_DRAW, SOLAR_CHARGE, SOLAR_SOURCE,
};
use domains::thermal::{
    equilibrium_temperature, node_stock, RadiatorReject, ThermalParams, NODE, RADIATOR_REJECT,
    SPACE,
};
use simcore::boundary;
use simcore::environment::SourceResolver;
use simcore::error::SimError;
use simcore::flow::Flow;
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::{State, Stock};

use crate::scenario::{StationScenario, HEAT_CLOSURE_SCENARIO};

/// The time-average power (W) Power dissipates into the node over one day. Op-order mirrors
/// Python `mean_dissipated_power`: `(charge_loss_per_day + load_per_day) / day_seconds`.
pub fn mean_dissipated_power(charge: &ChargeParams, scenario: &StationScenario) -> f64 {
    let ph = &scenario.power;
    let day_seconds = ph.steps_per_day as f64 * ph.dt_seconds;
    let charge_loss_per_day = (1.0 - charge.charge_efficiency) * daily_solar_energy(ph);
    let load_per_day = balanced_load_w(charge, ph) * day_seconds;
    (charge_loss_per_day + load_per_day) / day_seconds
}

/// The node's predicted equilibrium temperature `T_eq` (K), set by dissipation — Thermal's
/// closed form with the forced load replaced by [`mean_dissipated_power`].
pub fn predicted_equilibrium_temperature(
    charge: &ChargeParams,
    thermal_params: &ThermalParams,
    scenario: &StationScenario,
) -> f64 {
    equilibrium_temperature(thermal_params, mean_dissipated_power(charge, scenario))
}

/// The node's predicted equilibrium sensible heat `Q_eq = C·(T_eq − T_space)` (J) — the
/// initial node heat `build_station` uses by default.
pub fn equilibrium_node_heat(
    charge: &ChargeParams,
    thermal_params: &ThermalParams,
    scenario: &StationScenario,
) -> f64 {
    let t_eq = predicted_equilibrium_temperature(charge, thermal_params, scenario);
    thermal_params.heat_capacity * (t_eq - thermal_params.space_temperature)
}

/// Assemble the coupled Power → Thermal station's initial `State` + flow `Registry`.
///
/// Four stocks (battery POOL, unclamped `solar_source`, `thermal.node` POOL, `space` sink)
/// and three ENERGY-balanced flows (`SolarCharge`/`LoadDraw` with `waste_heat = NODE`,
/// `RadiatorReject`). `node0 = None` ⇒ [`equilibrium_node_heat`]. No loss-sinks (no
/// POPULATION stock).
pub fn build_station(
    charge: &ChargeParams,
    thermal_params: &ThermalParams,
    scenario: &StationScenario,
    node0: Option<f64>,
) -> Result<(State, Registry), SimError> {
    let ph = &scenario.power;
    let node_heat =
        node0.unwrap_or_else(|| equilibrium_node_heat(charge, thermal_params, scenario));
    let battery = battery_stock(ph.battery0)?;
    let solar_source = boundary::source(SOLAR_SOURCE.to_string(), Quantity::Energy, 0.0, true)?;
    let node = node_stock(node_heat)?;
    let space = boundary::sink(SPACE.to_string(), Quantity::Energy, 0.0)?;
    let mut stocks: BTreeMap<String, Stock> = BTreeMap::new();
    for s in [battery, solar_source, node, space] {
        stocks.insert(s.id.clone(), s);
    }
    let state = State::new(0, stocks.clone(), 0, BTreeMap::new())?;
    let flows: Vec<Box<dyn Flow>> = vec![
        Box::new(SolarCharge::new(
            SOLAR_CHARGE.to_string(),
            SOLAR_SOURCE.to_string(),
            BATTERY.to_string(),
            NODE.to_string(), // the seam: dissipation lands in the thermal node
            *charge,
        )),
        Box::new(LoadDraw::new(
            LOAD_DRAW.to_string(),
            BATTERY.to_string(),
            NODE.to_string(),
        )),
        Box::new(RadiatorReject::new(
            RADIATOR_REJECT.to_string(),
            NODE.to_string(),
            SPACE.to_string(),
            *thermal_params,
        )),
    ];
    let registry = Registry::flows_only(flows, &stocks)?;
    Ok((state, registry))
}

/// The merged station forcing — for Step 1, exactly Power's diurnal resolver (Thermal's
/// radiator reads the node stock, not `env`, so it contributes no forcing).
pub fn station_resolver(
    charge: &ChargeParams,
    scenario: &StationScenario,
) -> Result<SourceResolver, SimError> {
    power_resolver(charge, &scenario.power)
}

/// The default heat-closure station resolver (the module's canonical scenario).
pub fn default_station_resolver(charge: &ChargeParams) -> Result<SourceResolver, SimError> {
    station_resolver(charge, &HEAT_CLOSURE_SCENARIO)
}
