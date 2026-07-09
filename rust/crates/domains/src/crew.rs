//! The standalone Crew domain — the Rust port of `domains.crew` (Phase-7 P7.3).
//!
//! The first net-consumer / open-loop sibling: three finite provisioned POOLs (CARBON /
//! WATER / OXYGEN) drawn down by three **forced** metabolic flows. Two of them split
//! ([`FoodMetabolism`], [`WaterBalance`]) via the `SolarCharge` η-split idiom on a mass
//! quantity. All arithmetic is `+ - * /` (no transcendental) and every flow is forced,
//! so Crew is **Tier-1 bit-exact** — the split op-order (`f · q`, `(1 − f) · q`, NOT
//! `q − f·q`) is the load-bearing detail (advisor's Tier-1 tripwire).

use std::collections::{BTreeMap, HashMap};

use simcore::boundary;
use simcore::environment::{constant, Environment, Schedule, SourceResolver};
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::quantities::{Quantity, StockKind};
use simcore::registry::Registry;
use simcore::state::{State, Stock};

// --- stock ids + forcing vars + flow ids (ASCII; str sort == Rust byte sort, #15) ---
/// The Crew domain id (only the three `crew.*` stores carry it; rest are `boundary`).
pub const CREW_DOMAIN: &str = "crew";
/// Provisioned-food-carbon POOL id (CARBON, mol).
pub const FOOD_STORE: &str = "crew.food_store";
/// Provisioned-potable-water POOL id (WATER, kg).
pub const WATER_STORE: &str = "crew.water_store";
/// Provisioned-O₂ POOL id (OXYGEN, mol).
pub const O2_STORE: &str = "crew.o2_store";
/// Monotonic respired-CO₂ BOUNDARY sink id.
pub const EXHALED_CO2: &str = "boundary.exhaled_co2";
/// Monotonic egested-carbon BOUNDARY sink id.
pub const FECAL_WASTE: &str = "boundary.fecal_waste";
/// Monotonic respiration+perspiration BOUNDARY sink id.
pub const CREW_HUMIDITY: &str = "boundary.crew_humidity";
/// Monotonic urine BOUNDARY sink id.
pub const URINE: &str = "boundary.urine";
/// Monotonic metabolic-O₂ BOUNDARY sink id.
pub const CREW_O2_CONSUMED: &str = "boundary.crew_o2_consumed";
/// Forcing var: crew O₂ drawn from the O₂ store (mol/s).
pub const O2_INTAKE_VAR: &str = "crew_o2_intake";
/// Forcing var: food carbon drawn from the food store (mol/s).
pub const FOOD_INTAKE_VAR: &str = "crew_food_intake";
/// Forcing var: water drawn from the water store (kg/s).
pub const WATER_INTAKE_VAR: &str = "crew_water_intake";
/// Flow id: O₂ consumption.
pub const OXYGEN_CONSUMPTION: &str = "crew.oxygen_consumption";
/// Flow id: food metabolism (the CARBON split).
pub const FOOD_METABOLISM: &str = "crew.food_metabolism";
/// Flow id: water balance (the WATER split).
pub const WATER_BALANCE: &str = "crew.water_balance";

/// The Crew metabolic-split fractions (`crew.yaml`).
#[derive(Debug, Clone, Copy)]
pub struct CrewParams {
    /// f_resp — fraction of ingested food carbon respired as CO₂ ∈ [0, 1].
    pub respired_carbon_fraction: f64,
    /// f_insensible — fraction of water intake leaving as humidity ∈ [0, 1].
    pub insensible_water_fraction: f64,
}

/// Crew scenario data (initial store inventories, forced intake rates, step).
#[derive(Debug, Clone, Copy)]
pub struct CrewScenario {
    /// Initial provisioned food carbon (mol).
    pub food_store0: f64,
    /// Initial provisioned potable water (kg).
    pub water_store0: f64,
    /// Initial provisioned O₂ (mol).
    pub o2_store0: f64,
    /// Forced crew O₂ intake (mol/s).
    pub o2_intake_rate: f64,
    /// Forced crew food-carbon intake (mol/s).
    pub food_intake_rate: f64,
    /// Forced crew water intake (kg/s).
    pub water_intake_rate: f64,
    /// Integration step (s).
    pub dt_seconds: f64,
    /// Steps per 24 h day.
    pub steps_per_day: u64,
}

/// The standalone validation scenario (`MISSION_SCENARIO`): a provisioned mission whose
/// stores deplete monotonically but stay well-fed over the horizon.
pub const MISSION_SCENARIO: CrewScenario = CrewScenario {
    food_store0: 1000.0,
    water_store0: 60.0,
    o2_store0: 2000.0,
    o2_intake_rate: 1.0e-3,
    food_intake_rate: 5.0e-4,
    water_intake_rate: 3.0e-5,
    dt_seconds: 3600.0,
    steps_per_day: 24,
};

/// The mission length in days (168 hourly steps).
pub const MISSION_DAYS: u64 = 7;

/// Split ingested food carbon into `(respired_co2, egested_feces)` (mol). Op-order
/// mirrors Python `carbon_split`: `respired = f·food`, `feces = (1 − f)·food`.
fn carbon_split(food_mol: f64, respired_carbon_fraction: f64) -> (f64, f64) {
    let respired = respired_carbon_fraction * food_mol;
    let feces = (1.0 - respired_carbon_fraction) * food_mol;
    (respired, feces)
}

/// Split water intake into `(humidity, urine)` (kg). Op-order mirrors Python
/// `water_split`: `humidity = f·water`, `urine = (1 − f)·water`.
fn water_split(water_kg: f64, insensible_water_fraction: f64) -> (f64, f64) {
    let humidity = insensible_water_fraction * water_kg;
    let urine = (1.0 - insensible_water_fraction) * water_kg;
    (humidity, urine)
}

/// OXYGEN flow `crew.o2_store → boundary.crew_o2_consumed` — forced O₂ intake (2-leg).
pub struct OxygenConsumption {
    id: String,
    o2_store: String,
    o2_consumed: String,
}

impl Flow for OxygenConsumption {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        _snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let q = env.get(O2_INTAKE_VAR)? * dt;
        FlowResult::new(vec![
            Leg::new(self.o2_store.clone(), -q)?,
            Leg::new(self.o2_consumed.clone(), q)?,
        ])
    }
}

/// CARBON flow — food carbon split into respired CO₂ + egested feces (forced, 3-leg).
pub struct FoodMetabolism {
    id: String,
    food_store: String,
    exhaled_co2: String,
    fecal_waste: String,
    params: CrewParams,
}

impl Flow for FoodMetabolism {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        _snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let q = env.get(FOOD_INTAKE_VAR)? * dt;
        let (respired, feces) = carbon_split(q, self.params.respired_carbon_fraction);
        FlowResult::new(vec![
            Leg::new(self.food_store.clone(), -q)?,
            Leg::new(self.exhaled_co2.clone(), respired)?,
            Leg::new(self.fecal_waste.clone(), feces)?,
        ])
    }
}

/// WATER flow — water intake split into humidity + urine (forced, 3-leg).
pub struct WaterBalance {
    id: String,
    water_store: String,
    crew_humidity: String,
    urine: String,
    params: CrewParams,
}

impl Flow for WaterBalance {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        _snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let q = env.get(WATER_INTAKE_VAR)? * dt;
        let (humidity, urine) = water_split(q, self.params.insensible_water_fraction);
        FlowResult::new(vec![
            Leg::new(self.water_store.clone(), -q)?,
            Leg::new(self.crew_humidity.clone(), humidity)?,
            Leg::new(self.urine.clone(), urine)?,
        ])
    }
}

/// A finite, depleting provisioned-consumable POOL (its 1:1 default composition).
fn store(stock_id: &str, quantity: Quantity, amount: f64) -> Result<Stock, SimError> {
    Stock::new(
        stock_id.to_string(),
        CREW_DOMAIN.to_string(),
        quantity,
        quantity.canonical_unit(),
        amount,
        StockKind::Pool,
        0.0,
        false,
        BTreeMap::new(),
    )
}

/// Assemble the standalone Crew system's initial `State` and flow `Registry`.
pub fn build_crew(
    params: &CrewParams,
    scenario: &CrewScenario,
) -> Result<(State, Registry), SimError> {
    let food_store = store(FOOD_STORE, Quantity::Carbon, scenario.food_store0)?;
    let water_store = store(WATER_STORE, Quantity::Water, scenario.water_store0)?;
    let o2_store = store(O2_STORE, Quantity::Oxygen, scenario.o2_store0)?;
    let exhaled_co2 = boundary::sink(EXHALED_CO2.to_string(), Quantity::Carbon, 0.0)?;
    let fecal_waste = boundary::sink(FECAL_WASTE.to_string(), Quantity::Carbon, 0.0)?;
    let crew_humidity = boundary::sink(CREW_HUMIDITY.to_string(), Quantity::Water, 0.0)?;
    let urine = boundary::sink(URINE.to_string(), Quantity::Water, 0.0)?;
    let o2_consumed = boundary::sink(CREW_O2_CONSUMED.to_string(), Quantity::Oxygen, 0.0)?;
    let mut stocks: BTreeMap<String, Stock> = BTreeMap::new();
    for s in [
        food_store,
        water_store,
        o2_store,
        exhaled_co2,
        fecal_waste,
        crew_humidity,
        urine,
        o2_consumed,
    ] {
        stocks.insert(s.id.clone(), s);
    }
    let state = State::new(0, stocks.clone(), 0, BTreeMap::new())?;
    let flows: Vec<Box<dyn Flow>> = vec![
        Box::new(OxygenConsumption {
            id: OXYGEN_CONSUMPTION.to_string(),
            o2_store: O2_STORE.to_string(),
            o2_consumed: CREW_O2_CONSUMED.to_string(),
        }),
        Box::new(FoodMetabolism {
            id: FOOD_METABOLISM.to_string(),
            food_store: FOOD_STORE.to_string(),
            exhaled_co2: EXHALED_CO2.to_string(),
            fecal_waste: FECAL_WASTE.to_string(),
            params: *params,
        }),
        Box::new(WaterBalance {
            id: WATER_BALANCE.to_string(),
            water_store: WATER_STORE.to_string(),
            crew_humidity: CREW_HUMIDITY.to_string(),
            urine: URINE.to_string(),
            params: *params,
        }),
    ];
    let registry = Registry::flows_only(flows, &stocks)?;
    Ok((state, registry))
}

/// The forcing: the three constant crew intake rates.
pub fn crew_resolver(scenario: &CrewScenario) -> Result<SourceResolver, SimError> {
    let mut forcings: HashMap<String, Schedule> = HashMap::new();
    forcings.insert(O2_INTAKE_VAR.to_string(), constant(scenario.o2_intake_rate)?);
    forcings.insert(
        FOOD_INTAKE_VAR.to_string(),
        constant(scenario.food_intake_rate)?,
    );
    forcings.insert(
        WATER_INTAKE_VAR.to_string(),
        constant(scenario.water_intake_rate)?,
    );
    SourceResolver::new(forcings, HashMap::new())
}
