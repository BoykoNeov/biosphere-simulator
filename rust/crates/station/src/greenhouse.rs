//! The biosphere ↔ cabin greenhouse — the port of `station.greenhouse` (P6.3 / P7.5).
//!
//! The frozen sealed biosphere's gas exchange meets the Step-2 cabin, so plants + soil
//! microbes + crew all breathe **one** cabin-air stock. The seam is **reversed**: the
//! biosphere's `CARBON_POOL` (`{C:1,O:2}`) / `O2_POOL` (`{O:2}`) stay the shared cabin air,
//! and the CABIN's five all-parameterised flows are re-pointed at those ids (`build_season`
//! reused wholesale). Runs under the two-rate [`crate::driver::run_master_day`] (biosphere
//! slow once/day, cabin fast ×`steps_per_day`). Tier-2 (the FvCB biosphere in the graph).

use std::collections::BTreeMap;

use domains::biosphere::stocks::{CARBON_POOL, O2_POOL, THERMAL_TIME};
use domains::biosphere::system::{build_season, weather_resolver};
use domains::crew::URINE;
use domains::crew::{CrewParams, FECAL_WASTE, FOOD_INTAKE_VAR, WATER_INTAKE_VAR};
use domains::eclss::{EclssParams, CO2_REMOVED, HUMIDITY_CONDENSATE, O2_SUPPLY};
use simcore::boundary;
use simcore::environment::{constant, SourceResolver};
use simcore::error::SimError;
use simcore::events::Event;
use simcore::integrator::EulerIntegrator;
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::{State, Stock};

use crate::cabin::build_cabin_flows;
use crate::driver::run_master_day;
use crate::scenario::GreenhouseScenario;
use crate::stocks::{
    cabin_h2o_stock, co2_composition, food_store_stock, gas_boundary, o2_composition,
    water_store_stock,
};

/// The greenhouse's cabin-only stocks (added to the biosphere stocks over the shared dict).
///
/// `fecal_waste_target == FECAL_WASTE` ⇒ include the `FECAL_WASTE` boundary sink (the open
/// loop); a non-default target (Step 6 `LITTER_CARBON`) omits it (no shadow sink).
pub(crate) fn greenhouse_cabin_stocks(
    scenario: &GreenhouseScenario,
    fecal_waste_target: &str,
) -> Result<Vec<Stock>, SimError> {
    let mut seq = vec![
        food_store_stock(scenario.cabin.food_store0)?,
        water_store_stock(scenario.cabin.water_store0)?,
        cabin_h2o_stock(scenario.cabin.cabin_h2o_0)?,
        gas_boundary(O2_SUPPLY, Quantity::Oxygen, o2_composition(), true)?,
        gas_boundary(CO2_REMOVED, Quantity::Carbon, co2_composition(), false)?,
        boundary::sink(HUMIDITY_CONDENSATE.to_string(), Quantity::Water, 0.0)?,
        boundary::sink(URINE.to_string(), Quantity::Water, 0.0)?,
    ];
    if fecal_waste_target == FECAL_WASTE {
        seq.push(boundary::sink(
            FECAL_WASTE.to_string(),
            Quantity::Carbon,
            0.0,
        )?);
    }
    Ok(seq)
}

/// Assemble the greenhouse: `(state, bio_reg, cabin_reg)` (biosphere ↔ cabin).
///
/// The biosphere registry is `build_season`'s output verbatim (over the biosphere stocks —
/// mirroring Python, which reuses `full_bio_registry`); the cabin registry is the five
/// cabin flows re-pointed at the biosphere gas pools, over the merged stock dict. The two
/// stock-id sets are asserted disjoint. `with_plants = false` gives the no-plant baseline
/// (empty biosphere registry) — not used by the golden, kept for the "it bit" contrast.
pub fn build_greenhouse(
    crew: &CrewParams,
    eclss: &EclssParams,
    scenario: &GreenhouseScenario,
    with_plants: bool,
    fecal_waste_target: &str,
) -> Result<(State, Registry, Registry), SimError> {
    let (bio_state, full_bio_registry) = build_season(&scenario.bio)?;
    let bio_stocks = bio_state.stocks.clone();

    let cabin_seq = greenhouse_cabin_stocks(scenario, fecal_waste_target)?;
    let mut cabin_stocks: BTreeMap<String, Stock> = BTreeMap::new();
    for s in cabin_seq {
        cabin_stocks.insert(s.id.clone(), s);
    }
    for id in cabin_stocks.keys() {
        if bio_stocks.contains_key(id) {
            return Err(SimError::Validation(format!(
                "greenhouse stock-id collision between the biosphere and the cabin: {id:?} \
                 (the two stock sets must be disjoint)"
            )));
        }
    }
    let mut stocks = bio_stocks;
    for (id, s) in cabin_stocks {
        stocks.insert(id, s);
    }
    let state = State::new(
        0,
        stocks.clone(),
        0,
        BTreeMap::from([(THERMAL_TIME.to_string(), 0.0)]),
    )?;

    let cabin_flows = build_cabin_flows(crew, eclss, CARBON_POOL, O2_POOL, fecal_waste_target);
    let cabin_registry = Registry::flows_only(cabin_flows, &stocks)?;
    let bio_registry = if with_plants {
        full_bio_registry
    } else {
        Registry::flows_only(Vec::new(), &stocks)?
    };
    Ok((state, bio_registry, cabin_registry))
}

/// The biosphere forcing resolver — the frozen `weather_resolver`, reused as-is (the reverse
/// seam keeps `CO2_POOL_VAR → CARBON_POOL`, now the cabin air).
pub fn greenhouse_bio_resolver(scenario: &GreenhouseScenario) -> Result<SourceResolver, SimError> {
    // The weather is tiled over the master-day horizon (`days` = whole days here, ≤ 1 year).
    weather_resolver(&scenario.bio, 1)
}

/// The cabin forcing resolver — the two constant crew intake rates.
pub fn greenhouse_cabin_resolver(
    scenario: &GreenhouseScenario,
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
    SourceResolver::new(forcings, std::collections::HashMap::new())
}

/// The two-rate driver: one day per master step (biosphere-slow / cabin-fast).
pub fn run_greenhouse(
    bio_integrator: &EulerIntegrator,
    cabin_integrator: &EulerIntegrator,
    state: State,
    bio_resolver: &SourceResolver,
    cabin_resolver: &SourceResolver,
    scenario: &GreenhouseScenario,
) -> Result<(Vec<State>, u64, Vec<Event>), SimError> {
    run_master_day(
        bio_integrator,
        cabin_integrator,
        state,
        bio_resolver,
        cabin_resolver,
        scenario.days,
        scenario.steps_per_day,
        scenario.bio_dt,
        scenario.cabin_dt,
        None,
    )
}
