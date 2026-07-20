//! The biosphere stock-id catalog + `ChamberWiring` — the Rust port of
//! `domains.biosphere.stocks` (Phase-7 P7.4). All ids are ASCII (str sort == byte sort,
//! #15). Byte-identical to the Python ids so every reduction stays bit-stable.

use std::collections::BTreeMap;

use simcore::error::SimError;
use simcore::quantities::{Quantity, StockKind};
use simcore::state::Stock;

// --- stock ids --------------------------------------------------------------
pub const LEAF_C: &str = "biosphere.leaf_c";
pub const STEM_C: &str = "biosphere.stem_c";
pub const ROOT_C: &str = "biosphere.root_c";
pub const STORAGE_C: &str = "biosphere.storage_c";
pub const SOIL_WATER: &str = "biosphere.soil_water";
pub const SOIL_N: &str = "biosphere.soil_n";
pub const PLANT_N: &str = "biosphere.plant_n";
pub const CO2_ATMOS: &str = "boundary.co2_atmos";
pub const CARBON_POOL: &str = "biosphere.carbon_pool";
pub const O2_POOL: &str = "biosphere.o2_pool";
pub const LITTER_CARBON: &str = "biosphere.litter_carbon";
pub const MICROBIAL_CARBON: &str = "biosphere.microbial_carbon";
pub const LITTER_N: &str = "biosphere.litter_n";
pub const WATER_VAPOR: &str = "biosphere.water_vapor";
pub const CONDENSATE: &str = "biosphere.condensate";
pub const CONSUMER_CARBON: &str = "biosphere.consumer_carbon";
pub const CO2_RESP: &str = "boundary.co2_resp";
pub const VAPOR_SINK: &str = "boundary.vapor_sink";
pub const LITTER_SINK: &str = "boundary.litter_sink";
pub const WATER_SOURCE: &str = "boundary.water_source";
pub const N_SOURCE: &str = "boundary.n_source";

// --- leaf compartment domain ids --------------------------------------------
pub const ATMOSPHERE: &str = "biosphere.atmosphere";
pub const SOIL: &str = "biosphere.soil";
pub const PLANTS: &str = "biosphere.plants";
pub const WATER: &str = "biosphere.water";
pub const CONSUMERS: &str = "biosphere.consumers";

// --- forcing var names (resolved through env.get, #16) ----------------------
pub const PAR_VAR: &str = "par";
pub const CI_VAR: &str = "ci";
pub const TEMP_VAR: &str = "temp";
pub const DAYLENGTH_VAR: &str = "daylength_s";
pub const RN_VAR: &str = "net_radiation";
pub const VPD_VAR: &str = "vpd";
pub const IRRIGATION_VAR: &str = "irrigation";
pub const FERTILIZATION_VAR: &str = "fertilization";
pub const SOIL_WATER_VAR: &str = "soil_water";
pub const CO2_POOL_VAR: &str = "co2_pool";
pub const THERMAL_TIME: &str = "thermal_time";
/// The SECOND aux accumulator (scope (B) inc. 1): cumulative vernalization days.
pub const VERNALIZATION_DAYS: &str = "vernalization_days";

/// The handful of stock ids whose identity depends on `sealed`, computed once.
#[derive(Debug, Clone)]
pub struct ChamberWiring {
    /// CARBON_POOL (sealed) | CO2_ATMOS (open).
    pub carbon_source: String,
    /// CARBON_POOL (sealed, == source) | CO2_RESP (open).
    pub resp_sink: String,
    /// O2_POOL (sealed) | None (open).
    pub o2_pool: Option<String>,
    /// LITTER_CARBON (sealed) | LITTER_SINK (open).
    pub litter_carbon_target: String,
    /// WATER_VAPOR (sealed, closed loop) | VAPOR_SINK (open).
    pub vapor_target: String,
}

/// Select the sealed-dependent cross-compartment ids (a pure id selection).
pub fn chamber_wiring(sealed: bool) -> ChamberWiring {
    if sealed {
        ChamberWiring {
            carbon_source: CARBON_POOL.to_string(),
            resp_sink: CARBON_POOL.to_string(),
            o2_pool: Some(O2_POOL.to_string()),
            litter_carbon_target: LITTER_CARBON.to_string(),
            vapor_target: WATER_VAPOR.to_string(),
        }
    } else {
        ChamberWiring {
            carbon_source: CO2_ATMOS.to_string(),
            resp_sink: CO2_RESP.to_string(),
            o2_pool: None,
            litter_carbon_target: LITTER_SINK.to_string(),
            vapor_target: VAPOR_SINK.to_string(),
        }
    }
}

/// A POPULATION CARBON organ pool (extinction-eligible, threshold 0).
pub fn organ_stock(stock_id: &str, domain: &str, amount: f64) -> Result<Stock, SimError> {
    Stock::new(
        stock_id.to_string(),
        domain.to_string(),
        Quantity::Carbon,
        Quantity::Carbon.canonical_unit(),
        amount,
        StockKind::Population,
        0.0,
        false,
        BTreeMap::new(),
    )
}

/// A single-currency POOL stock (default 1:1 composition).
pub fn pool_stock(
    stock_id: &str,
    domain: &str,
    quantity: Quantity,
    amount: f64,
) -> Result<Stock, SimError> {
    Stock::new(
        stock_id.to_string(),
        domain.to_string(),
        quantity,
        quantity.canonical_unit(),
        amount,
        StockKind::Pool,
        0.0,
        false,
        BTreeMap::new(),
    )
}
