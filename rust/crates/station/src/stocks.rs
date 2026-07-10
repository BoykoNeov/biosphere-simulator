//! Shared composition-carrying stock builders for the cabin gas seams (P7.5).
//!
//! The ECLSS `cabin_*` / `boundary` constructors build **single-quantity** pools; the
//! coupled cabin loop needs the gas-phase composition (`{C:1,O:2}` / `{O:2}`) or OXYGEN
//! fails to close (the scrubber removes 2 O per CO₂, the makeup adds 2 O per O₂). The
//! Python assembly builds these gas stocks inline because `boundary.py` / `eclss.stocks`
//! take no composition arg and extending them is a core change; the Rust port centralizes
//! the same inline builders here (used by [`crate::cabin`] / [`crate::water`] /
//! [`crate::greenhouse`] / [`crate::sealed`]) rather than re-declaring them per module.

use std::collections::BTreeMap;

use domains::crew::CREW_DOMAIN;
use domains::eclss::ECLSS_DOMAIN;
use simcore::boundary::BOUNDARY_DOMAIN;
use simcore::error::SimError;
use simcore::quantities::{Quantity, StockKind};
use simcore::state::Stock;

/// CO₂ composition `{CARBON:1, OXYGEN:2}` (biosphere convention: OXYGEN is O-atoms).
pub fn co2_composition() -> BTreeMap<Quantity, f64> {
    BTreeMap::from([(Quantity::Carbon, 1.0), (Quantity::Oxygen, 2.0)])
}

/// O₂ composition `{OXYGEN:2}`.
pub fn o2_composition() -> BTreeMap<Quantity, f64> {
    BTreeMap::from([(Quantity::Oxygen, 2.0)])
}

/// A composition-carrying cabin POOL (`cabin_o2` / `cabin_co2`), ECLSS domain.
pub fn gas_pool(
    id: &str,
    quantity: Quantity,
    amount: f64,
    composition: BTreeMap<Quantity, f64>,
) -> Result<Stock, SimError> {
    Stock::new(
        id.to_string(),
        ECLSS_DOMAIN.to_string(),
        quantity,
        quantity.canonical_unit(),
        amount,
        StockKind::Pool,
        0.0,
        false,
        composition,
    )
}

/// A composition-carrying BOUNDARY reservoir (`co2_removed` sink / `o2_supply` source).
pub fn gas_boundary(
    id: &str,
    quantity: Quantity,
    composition: BTreeMap<Quantity, f64>,
    unclamped: bool,
) -> Result<Stock, SimError> {
    Stock::new(
        id.to_string(),
        BOUNDARY_DOMAIN.to_string(),
        quantity,
        quantity.canonical_unit(),
        0.0,
        StockKind::Boundary,
        0.0,
        unclamped,
        composition,
    )
}

/// A single-quantity POOL in the given domain (`cabin_h2o` / crew stores).
pub fn simple_pool(
    id: &str,
    domain: &str,
    quantity: Quantity,
    amount: f64,
) -> Result<Stock, SimError> {
    Stock::new(
        id.to_string(),
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

/// The crew `food_store` POOL (CARBON, mol).
pub fn food_store_stock(amount: f64) -> Result<Stock, SimError> {
    simple_pool(
        domains::crew::FOOD_STORE,
        CREW_DOMAIN,
        Quantity::Carbon,
        amount,
    )
}

/// The crew `water_store` POOL (WATER, kg).
pub fn water_store_stock(amount: f64) -> Result<Stock, SimError> {
    simple_pool(
        domains::crew::WATER_STORE,
        CREW_DOMAIN,
        Quantity::Water,
        amount,
    )
}

/// The ECLSS `cabin_h2o` POOL (WATER, kg).
pub fn cabin_h2o_stock(amount: f64) -> Result<Stock, SimError> {
    simple_pool(
        domains::eclss::CABIN_H2O,
        ECLSS_DOMAIN,
        Quantity::Water,
        amount,
    )
}
