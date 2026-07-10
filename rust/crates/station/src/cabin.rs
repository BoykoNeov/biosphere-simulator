//! The crew ↔ ECLSS cabin gas loop — the port of `station.cabin` (P6.2 / P7.5).
//!
//! Couples three quantities (CARBON / OXYGEN / WATER) at the shared cabin-air stocks, with
//! **CO₂ as a composition `{C:1,O:2}` stock** — which is what makes OXYGEN *close* over
//! the augmented loop `o2_supply → cabin_o2 → cabin_co2 → co2_removed`. The ECLSS forced
//! `CrewMetabolism` stand-in + its `metabolic_*` reservoirs and the crew `o2_store` /
//! `OxygenConsumption` are dropped; the real crew breathes cabin air via [`CrewRespiration`]
//! and adds humidity via crew `WaterBalance`. **Tier-1 bit-exact** — transcendental-free,
//! the strongest cross-port gate.

use std::collections::BTreeMap;

use domains::crew::{
    CrewParams, WaterBalance, FECAL_WASTE, FOOD_INTAKE_VAR, FOOD_STORE, URINE, WATER_BALANCE,
    WATER_INTAKE_VAR, WATER_STORE,
};
use domains::eclss::{
    CO2Scrubber, Condenser, EclssParams, O2Makeup, CABIN_CO2, CABIN_O2, CO2_REMOVED, CO2_SCRUBBER,
    CONDENSER, HUMIDITY_CONDENSATE, O2_MAKEUP, O2_SUPPLY,
};
use simcore::boundary;
use simcore::environment::{constant, SourceResolver};
use simcore::error::SimError;
use simcore::flow::Flow;
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::{State, Stock};

use crate::flows::{CrewRespiration, CREW_RESPIRATION};
use crate::scenario::{CabinScenario, CABIN_GAS_SCENARIO};
use crate::stocks::{
    cabin_h2o_stock, co2_composition, food_store_stock, gas_boundary, gas_pool, o2_composition,
    water_store_stock,
};

/// The cabin stocks (10) — inline gas/composition + crew stores + boundary reservoirs.
fn cabin_stocks(scenario: &CabinScenario) -> Result<BTreeMap<String, Stock>, SimError> {
    let seq = [
        gas_pool(
            CABIN_O2,
            Quantity::Oxygen,
            scenario.cabin_o2_0,
            o2_composition(),
        )?,
        gas_pool(
            CABIN_CO2,
            Quantity::Carbon,
            scenario.cabin_co2_0,
            co2_composition(),
        )?,
        cabin_h2o_stock(scenario.cabin_h2o_0)?,
        food_store_stock(scenario.food_store0)?,
        water_store_stock(scenario.water_store0)?,
        gas_boundary(O2_SUPPLY, Quantity::Oxygen, o2_composition(), true)?,
        gas_boundary(CO2_REMOVED, Quantity::Carbon, co2_composition(), false)?,
        boundary::sink(HUMIDITY_CONDENSATE.to_string(), Quantity::Water, 0.0)?,
        boundary::sink(FECAL_WASTE.to_string(), Quantity::Carbon, 0.0)?,
        boundary::sink(URINE.to_string(), Quantity::Water, 0.0)?,
    ];
    let mut stocks: BTreeMap<String, Stock> = BTreeMap::new();
    for s in seq {
        stocks.insert(s.id.clone(), s);
    }
    Ok(stocks)
}

/// The five coupled cabin flows (`CrewRespiration` + crew `WaterBalance` + the three ECLSS
/// control loops), over the cabin's own gas ids.
pub(crate) fn build_cabin_flows(
    crew: &CrewParams,
    eclss: &EclssParams,
    cabin_co2: &str,
    cabin_o2: &str,
    fecal_waste: &str,
) -> Vec<Box<dyn Flow>> {
    vec![
        Box::new(CrewRespiration::new(
            CREW_RESPIRATION.to_string(),
            FOOD_STORE.to_string(),
            cabin_co2.to_string(),
            cabin_o2.to_string(),
            fecal_waste.to_string(),
            crew.respired_carbon_fraction,
        )),
        Box::new(WaterBalance::new(
            WATER_BALANCE.to_string(),
            WATER_STORE.to_string(),
            domains::eclss::CABIN_H2O.to_string(), // the seam: crew humidity → the cabin
            URINE.to_string(),
            *crew,
        )),
        Box::new(CO2Scrubber::new(
            CO2_SCRUBBER.to_string(),
            cabin_co2.to_string(),
            CO2_REMOVED.to_string(),
            *eclss,
        )),
        Box::new(Condenser::new(
            CONDENSER.to_string(),
            domains::eclss::CABIN_H2O.to_string(),
            HUMIDITY_CONDENSATE.to_string(),
            *eclss,
        )),
        Box::new(O2Makeup::new(
            O2_MAKEUP.to_string(),
            O2_SUPPLY.to_string(),
            cabin_o2.to_string(),
            *eclss,
        )),
    ]
}

/// Assemble the coupled Crew ↔ ECLSS cabin's initial `State` + flow `Registry`.
pub fn build_cabin(
    crew: &CrewParams,
    eclss: &EclssParams,
    scenario: &CabinScenario,
) -> Result<(State, Registry), SimError> {
    let stocks = cabin_stocks(scenario)?;
    let state = State::new(0, stocks.clone(), 0, BTreeMap::new())?;
    let flows = build_cabin_flows(crew, eclss, CABIN_CO2, CABIN_O2, FECAL_WASTE);
    let registry = Registry::flows_only(flows, &stocks)?;
    Ok((state, registry))
}

/// The merged cabin forcing: the two constant crew intake rates (O₂ intake is derived via
/// RQ = 1 inside `CrewRespiration`, so it is not a forcing var).
pub fn cabin_resolver(scenario: &CabinScenario) -> Result<SourceResolver, SimError> {
    let mut forcings = std::collections::HashMap::new();
    forcings.insert(
        FOOD_INTAKE_VAR.to_string(),
        constant(scenario.food_intake_rate)?,
    );
    forcings.insert(
        WATER_INTAKE_VAR.to_string(),
        constant(scenario.water_intake_rate)?,
    );
    SourceResolver::new(forcings, std::collections::HashMap::new())
}

/// The default cabin-gas resolver (the module's canonical scenario).
pub fn default_cabin_resolver() -> Result<SourceResolver, SimError> {
    cabin_resolver(&CABIN_GAS_SCENARIO)
}
