//! The crew water-recovery loop — the port of `station.water` (P6.4 / P7.5).
//!
//! Built one assembly below the greenhouse (on the Step-2 cabin): the two WATER disposal
//! sinks (`humidity_condensate` / `urine`) are re-pointed into a `recovered_water` buffer
//! POOL, and a station-owned [`WaterRecovery`] flow returns the recovered fraction η_w to
//! `water_store`, venting `(1−η_w)` to `brine`. So the crew's water becomes **regenerative**
//! (net drain drops from the full intake to `(1−η_w)·intake`). Everything CARBON / OXYGEN is
//! identical to the cabin — Step 4 touches only WATER. **Tier-1 bit-exact**
//! (donor-controlled `WaterRecovery` is still only `*`/`+`/`-`/`/`).

use std::collections::BTreeMap;

use domains::crew::{
    CrewParams, WaterBalance, FECAL_WASTE, FOOD_STORE, WATER_BALANCE, WATER_STORE,
};
use domains::eclss::{
    CO2Scrubber, Condenser, EclssParams, O2Makeup, CABIN_CO2, CABIN_H2O, CABIN_O2, CO2_REMOVED,
    CO2_SCRUBBER, CONDENSER, ECLSS_DOMAIN, O2_MAKEUP, O2_SUPPLY,
};
use simcore::boundary;
use simcore::environment::SourceResolver;
use simcore::error::SimError;
use simcore::flow::Flow;
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::{State, Stock};

use crate::cabin::cabin_resolver;
use crate::flows::{
    CrewRespiration, WaterRecovery, WaterRecoveryParams, CREW_RESPIRATION, WATER_RECOVERY,
};
use crate::scenario::{CabinScenario, WATER_RECOVERY_SCENARIO};
use crate::stocks::{
    cabin_h2o_stock, co2_composition, food_store_stock, gas_boundary, gas_pool, o2_composition,
    simple_pool, water_store_stock,
};

/// The recovered-water buffer POOL id (the crew analogue of the biosphere's `condensate`).
pub const RECOVERED_WATER: &str = "eclss.recovered_water";
/// The unrecoverable-water boundary sink id (the `(1−η_w)` brine leg).
pub const BRINE: &str = "boundary.brine";

/// The `recovered_water` buffer POOL (WATER, ECLSS domain; starts empty).
pub fn recovered_water_pool() -> Result<Stock, SimError> {
    simple_pool(RECOVERED_WATER, ECLSS_DOMAIN, Quantity::Water, 0.0)
}

/// Assemble the coupled crew water-recovery cabin (`State` + flow `Registry`).
pub fn build_water_recovery(
    crew: &CrewParams,
    eclss: &EclssParams,
    recovery: &WaterRecoveryParams,
    scenario: &CabinScenario,
) -> Result<(State, Registry), SimError> {
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
        boundary::sink(FECAL_WASTE.to_string(), Quantity::Carbon, 0.0)?,
        recovered_water_pool()?,
        boundary::sink(BRINE.to_string(), Quantity::Water, 0.0)?,
    ];
    let mut stocks: BTreeMap<String, Stock> = BTreeMap::new();
    for s in seq {
        stocks.insert(s.id.clone(), s);
    }
    let state = State::new(0, stocks.clone(), 0, BTreeMap::new())?;
    let flows: Vec<Box<dyn Flow>> = vec![
        Box::new(CrewRespiration::new(
            CREW_RESPIRATION.to_string(),
            FOOD_STORE.to_string(),
            CABIN_CO2.to_string(),
            CABIN_O2.to_string(),
            FECAL_WASTE.to_string(),
            crew.respired_carbon_fraction,
        )),
        Box::new(WaterBalance::new(
            WATER_BALANCE.to_string(),
            WATER_STORE.to_string(),
            CABIN_H2O.to_string(),
            RECOVERED_WATER.to_string(), // the seam: urine → the recovery buffer
            *crew,
        )),
        Box::new(CO2Scrubber::new(
            CO2_SCRUBBER.to_string(),
            CABIN_CO2.to_string(),
            CO2_REMOVED.to_string(),
            *eclss,
        )),
        Box::new(Condenser::new(
            CONDENSER.to_string(),
            CABIN_H2O.to_string(),
            RECOVERED_WATER.to_string(), // the seam: condensate → the buffer
            *eclss,
        )),
        Box::new(O2Makeup::new(
            O2_MAKEUP.to_string(),
            O2_SUPPLY.to_string(),
            CABIN_O2.to_string(),
            *eclss,
        )),
        Box::new(WaterRecovery::new(
            WATER_RECOVERY.to_string(),
            RECOVERED_WATER.to_string(),
            WATER_STORE.to_string(), // the seam: recovered water → the store
            BRINE.to_string(),
            *recovery,
        )),
    ];
    let registry = Registry::flows_only(flows, &stocks)?;
    Ok((state, registry))
}

/// The forcing — the two constant crew intake rates (reused from the cabin).
pub fn water_recovery_resolver(scenario: &CabinScenario) -> Result<SourceResolver, SimError> {
    cabin_resolver(scenario)
}

/// The default water-recovery resolver (the module's canonical scenario).
pub fn default_water_recovery_resolver() -> Result<SourceResolver, SimError> {
    water_recovery_resolver(&WATER_RECOVERY_SCENARIO)
}
