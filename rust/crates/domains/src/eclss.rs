//! The standalone ECLSS domain — the Rust port of `domains.eclss` (Phase-7 P7.3).
//!
//! The first **multi-quantity** sibling: three single-quantity cabin POOLs (OXYGEN /
//! CARBON / WATER). Four flows: the forced [`CrewMetabolism`] (6 legs across 3
//! quantities) + three donor-/demand-controlled control loops ([`CO2Scrubber`],
//! [`Condenser`], [`O2Makeup`]). All arithmetic is `+ - * /` (no transcendental), so
//! ECLSS is **Tier-1 bit-exact** — which makes the op-order in each `evaluate` (the
//! `(k · stock) · dt` grouping, the `(setpoint − cabin_o2)` demand term) load-bearing.

use std::collections::{BTreeMap, HashMap};

use simcore::boundary;
use simcore::environment::{constant, Environment, Schedule, SourceResolver};
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::quantities::{Quantity, StockKind};
use simcore::registry::Registry;
use simcore::state::{State, Stock};

// --- stock ids + forcing vars + flow ids (ASCII; str sort == Rust byte sort, #15) ---
/// The ECLSS domain id (only the three `cabin_*` pools carry it; rest are `boundary`).
pub const ECLSS_DOMAIN: &str = "eclss";
/// Breathable-O₂ cabin POOL id (OXYGEN, mol).
pub const CABIN_O2: &str = "eclss.cabin_o2";
/// Metabolic-CO₂ cabin POOL id (CARBON, mol).
pub const CABIN_CO2: &str = "eclss.cabin_co2";
/// Cabin-humidity POOL id (WATER, kg).
pub const CABIN_H2O: &str = "eclss.cabin_h2o";
/// Unclamped O₂-makeup tank BOUNDARY source id.
pub const O2_SUPPLY: &str = "boundary.o2_supply";
/// Monotonic scrubber-product BOUNDARY sink id.
pub const CO2_REMOVED: &str = "boundary.co2_removed";
/// Monotonic condenser-product BOUNDARY sink id.
pub const HUMIDITY_CONDENSATE: &str = "boundary.humidity_condensate";
/// Crew-seam O₂ sink id.
pub const METABOLIC_O2_SINK: &str = "boundary.metabolic_o2_sink";
/// Crew-seam CO₂ source id.
pub const METABOLIC_CO2_SOURCE: &str = "boundary.metabolic_co2_source";
/// Crew-seam H₂O source id.
pub const METABOLIC_H2O_SOURCE: &str = "boundary.metabolic_h2o_source";
/// Forcing var: crew O₂ intake out of the cabin (mol/s).
pub const O2_CONSUMPTION_VAR: &str = "o2_consumption";
/// Forcing var: crew CO₂ exhaled into the cabin (mol/s).
pub const CO2_PRODUCTION_VAR: &str = "co2_production";
/// Forcing var: crew humidity into the cabin (kg/s).
pub const H2O_PRODUCTION_VAR: &str = "h2o_production";
/// Flow id: the forced multi-quantity crew seam.
pub const CREW_METABOLISM: &str = "eclss.crew_metabolism";
/// Flow id: the CO₂ scrubber.
pub const CO2_SCRUBBER: &str = "eclss.co2_scrubber";
/// Flow id: the humidity condenser.
pub const CONDENSER: &str = "eclss.condenser";
/// Flow id: the O₂ regulator.
pub const O2_MAKEUP: &str = "eclss.o2_makeup";

/// The ECLSS control-loop coefficients (`eclss.yaml`).
#[derive(Debug, Clone, Copy)]
pub struct EclssParams {
    /// k_scrub — first-order CO₂ removal rate (1/s), > 0.
    pub co2_scrub_rate: f64,
    /// k_cond — first-order humidity removal rate (1/s), > 0.
    pub condense_rate: f64,
    /// k_makeup — proportional O₂-regulator gain (1/s), > 0.
    pub o2_makeup_gain: f64,
    /// o2_setpoint — target cabin O₂ inventory (mol), > 0.
    pub o2_setpoint: f64,
}

/// ECLSS scenario data (initial cabin inventories, forced crew rates, step).
#[derive(Debug, Clone, Copy)]
pub struct EclssScenario {
    /// Initial cabin O₂ (mol) — starts at the setpoint.
    pub cabin_o2_0: f64,
    /// Initial cabin CO₂ (mol).
    pub cabin_co2_0: f64,
    /// Initial cabin H₂O (kg).
    pub cabin_h2o_0: f64,
    /// Forced crew O₂ consumption (mol/s).
    pub o2_consumption_rate: f64,
    /// Forced crew CO₂ production (mol/s).
    pub co2_production_rate: f64,
    /// Forced crew humidity production (kg/s).
    pub h2o_production_rate: f64,
    /// Integration step (s).
    pub dt_seconds: f64,
}

/// The standalone validation scenario (`STEADY_STATE_SCENARIO`): a clean cabin under a
/// constant crew load, each species relaxing to an emergent steady state.
pub const STEADY_STATE_SCENARIO: EclssScenario = EclssScenario {
    cabin_o2_0: 10.0,
    cabin_co2_0: 0.0,
    cabin_h2o_0: 0.0,
    o2_consumption_rate: 0.004,
    co2_production_rate: 0.003,
    h2o_production_rate: 2.0e-5,
    dt_seconds: 60.0,
};

/// The steady-state-run horizon (steps) — ~27 slowest-loop time constants.
pub const STEADY_STATE_STEPS: u64 = 900;

/// Instantaneous CO₂ scrub rate `k_scrub · cabin_co2` (mol/s). Op-order mirrors Python.
fn scrub_flux(cabin_co2: f64, co2_scrub_rate: f64) -> f64 {
    co2_scrub_rate * cabin_co2
}

/// Instantaneous humidity-condensation rate `k_cond · cabin_h2o` (kg/s).
fn condense_flux(cabin_h2o: f64, condense_rate: f64) -> f64 {
    condense_rate * cabin_h2o
}

/// Instantaneous O₂-makeup rate `k_makeup · (o2_setpoint − cabin_o2)` (mol/s). The
/// `(setpoint − cabin_o2)` demand term is bit-exact-load-bearing (Tier-1).
fn makeup_flux(cabin_o2: f64, o2_makeup_gain: f64, o2_setpoint: f64) -> f64 {
    o2_makeup_gain * (o2_setpoint - cabin_o2)
}

/// Read a donor stock's amount (Python `snapshot.stocks[id].amount` / `KeyError`).
fn donor_amount(snapshot: &State, id: &str) -> Result<f64, SimError> {
    snapshot
        .stocks
        .get(id)
        .map(|s| s.amount)
        .ok_or_else(|| SimError::Reference(format!("flow reads unknown stock {id:?}")))
}

/// Forced multi-quantity flow — the crew/Phase-6 seam (6 legs across 3 quantities).
pub struct CrewMetabolism {
    id: String,
    cabin_o2: String,
    cabin_co2: String,
    cabin_h2o: String,
    metabolic_o2_sink: String,
    metabolic_co2_source: String,
    metabolic_h2o_source: String,
}

impl Flow for CrewMetabolism {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        _snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let o2 = env.get(O2_CONSUMPTION_VAR)? * dt;
        let co2 = env.get(CO2_PRODUCTION_VAR)? * dt;
        let h2o = env.get(H2O_PRODUCTION_VAR)? * dt;
        FlowResult::new(vec![
            // OXYGEN: crew consumes O₂ out of the cabin.
            Leg::new(self.cabin_o2.clone(), -o2)?,
            Leg::new(self.metabolic_o2_sink.clone(), o2)?,
            // CARBON: crew exhales CO₂ into the cabin.
            Leg::new(self.metabolic_co2_source.clone(), -co2)?,
            Leg::new(self.cabin_co2.clone(), co2)?,
            // WATER: crew adds humidity to the cabin.
            Leg::new(self.metabolic_h2o_source.clone(), -h2o)?,
            Leg::new(self.cabin_h2o.clone(), h2o)?,
        ])
    }
}

/// CARBON flow `cabin_co2 → boundary.co2_removed` — the first-order scrubber (2-leg).
pub struct CO2Scrubber {
    id: String,
    cabin_co2: String,
    co2_removed: String,
    params: EclssParams,
}

impl Flow for CO2Scrubber {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        // Op-order mirrors Python: scrub_flux(cabin_co2) * dt = (k_scrub * cabin_co2) * dt.
        let removed = scrub_flux(
            donor_amount(snapshot, &self.cabin_co2)?,
            self.params.co2_scrub_rate,
        ) * dt;
        FlowResult::new(vec![
            Leg::new(self.cabin_co2.clone(), -removed)?,
            Leg::new(self.co2_removed.clone(), removed)?,
        ])
    }
}

/// WATER flow `cabin_h2o → boundary.humidity_condensate` — the condenser (2-leg).
pub struct Condenser {
    id: String,
    cabin_h2o: String,
    humidity_condensate: String,
    params: EclssParams,
}

impl Flow for Condenser {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let condensed = condense_flux(
            donor_amount(snapshot, &self.cabin_h2o)?,
            self.params.condense_rate,
        ) * dt;
        FlowResult::new(vec![
            Leg::new(self.cabin_h2o.clone(), -condensed)?,
            Leg::new(self.humidity_condensate.clone(), condensed)?,
        ])
    }
}

/// OXYGEN flow `boundary.o2_supply → cabin_o2` — the demand-controlled regulator (2-leg).
pub struct O2Makeup {
    id: String,
    o2_supply: String,
    cabin_o2: String,
    params: EclssParams,
}

impl Flow for O2Makeup {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        // Op-order mirrors Python: makeup_flux(cabin_o2) * dt.
        let supplied = makeup_flux(
            donor_amount(snapshot, &self.cabin_o2)?,
            self.params.o2_makeup_gain,
            self.params.o2_setpoint,
        ) * dt;
        FlowResult::new(vec![
            Leg::new(self.o2_supply.clone(), -supplied)?,
            Leg::new(self.cabin_o2.clone(), supplied)?,
        ])
    }
}

/// A single-quantity cabin-air POOL (its 1:1 default composition).
fn cabin_pool(stock_id: &str, quantity: Quantity, amount: f64) -> Result<Stock, SimError> {
    Stock::new(
        stock_id.to_string(),
        ECLSS_DOMAIN.to_string(),
        quantity,
        quantity.canonical_unit(),
        amount,
        StockKind::Pool,
        0.0,
        false,
        BTreeMap::new(),
    )
}

/// Assemble the standalone ECLSS system's initial `State` and flow `Registry`.
pub fn build_eclss(
    params: &EclssParams,
    scenario: &EclssScenario,
) -> Result<(State, Registry), SimError> {
    let cabin_o2 = cabin_pool(CABIN_O2, Quantity::Oxygen, scenario.cabin_o2_0)?;
    let cabin_co2 = cabin_pool(CABIN_CO2, Quantity::Carbon, scenario.cabin_co2_0)?;
    let cabin_h2o = cabin_pool(CABIN_H2O, Quantity::Water, scenario.cabin_h2o_0)?;
    let o2_supply = boundary::source(O2_SUPPLY.to_string(), Quantity::Oxygen, 0.0, true)?;
    let co2_removed = boundary::sink(CO2_REMOVED.to_string(), Quantity::Carbon, 0.0)?;
    let humidity_condensate =
        boundary::sink(HUMIDITY_CONDENSATE.to_string(), Quantity::Water, 0.0)?;
    let metabolic_o2_sink = boundary::sink(METABOLIC_O2_SINK.to_string(), Quantity::Oxygen, 0.0)?;
    let metabolic_co2_source =
        boundary::source(METABOLIC_CO2_SOURCE.to_string(), Quantity::Carbon, 0.0, true)?;
    let metabolic_h2o_source =
        boundary::source(METABOLIC_H2O_SOURCE.to_string(), Quantity::Water, 0.0, true)?;
    let mut stocks: BTreeMap<String, Stock> = BTreeMap::new();
    for s in [
        cabin_o2,
        cabin_co2,
        cabin_h2o,
        o2_supply,
        co2_removed,
        humidity_condensate,
        metabolic_o2_sink,
        metabolic_co2_source,
        metabolic_h2o_source,
    ] {
        stocks.insert(s.id.clone(), s);
    }
    let state = State::new(0, stocks.clone(), 0, BTreeMap::new())?;
    let flows: Vec<Box<dyn Flow>> = vec![
        Box::new(CrewMetabolism {
            id: CREW_METABOLISM.to_string(),
            cabin_o2: CABIN_O2.to_string(),
            cabin_co2: CABIN_CO2.to_string(),
            cabin_h2o: CABIN_H2O.to_string(),
            metabolic_o2_sink: METABOLIC_O2_SINK.to_string(),
            metabolic_co2_source: METABOLIC_CO2_SOURCE.to_string(),
            metabolic_h2o_source: METABOLIC_H2O_SOURCE.to_string(),
        }),
        Box::new(CO2Scrubber {
            id: CO2_SCRUBBER.to_string(),
            cabin_co2: CABIN_CO2.to_string(),
            co2_removed: CO2_REMOVED.to_string(),
            params: *params,
        }),
        Box::new(Condenser {
            id: CONDENSER.to_string(),
            cabin_h2o: CABIN_H2O.to_string(),
            humidity_condensate: HUMIDITY_CONDENSATE.to_string(),
            params: *params,
        }),
        Box::new(O2Makeup {
            id: O2_MAKEUP.to_string(),
            o2_supply: O2_SUPPLY.to_string(),
            cabin_o2: CABIN_O2.to_string(),
            params: *params,
        }),
    ];
    let registry = Registry::flows_only(flows, &stocks)?;
    Ok((state, registry))
}

/// The forcing: the three constant crew metabolic rates.
pub fn eclss_resolver(scenario: &EclssScenario) -> Result<SourceResolver, SimError> {
    let mut forcings: HashMap<String, Schedule> = HashMap::new();
    forcings.insert(
        O2_CONSUMPTION_VAR.to_string(),
        constant(scenario.o2_consumption_rate)?,
    );
    forcings.insert(
        CO2_PRODUCTION_VAR.to_string(),
        constant(scenario.co2_production_rate)?,
    );
    forcings.insert(
        H2O_PRODUCTION_VAR.to_string(),
        constant(scenario.h2o_production_rate)?,
    );
    SourceResolver::new(forcings, HashMap::new())
}
