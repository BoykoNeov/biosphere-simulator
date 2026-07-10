//! The biomass/food loop — the port of `station.harvest` (P6.6 / P7.5).
//!
//! Built on the Step-3 greenhouse. Adds **one** station-owned flow, [`Harvest`]
//! (`storage_c → food_store`, donor-controlled), to the cabin / fast registry — the CARBON
//! twin of Step-4 `WaterRecovery`, making the crew's finite `food_store` **regenerative**.
//! The reproductive plant precondition is met by injecting the biosphere `thermal_time` aux
//! **past anthesis** at `State` construction (a station-level injection — `SeasonScenario`
//! is untouched). Seam 2 (`close_feces`) re-points `CrewRespiration`'s fecal carbon into
//! `LITTER_CARBON`, closing the trophic CARBON ring. Two-rate, Euler-only. Tier-2 (FvCB).

use std::collections::BTreeMap;

use domains::biosphere::stocks::{CARBON_POOL, LITTER_CARBON, O2_POOL, STORAGE_C, THERMAL_TIME};
use domains::crew::{CrewParams, FECAL_WASTE, FOOD_STORE};
use domains::eclss::EclssParams;
use simcore::environment::SourceResolver;
use simcore::error::SimError;
use simcore::events::Event;
use simcore::integrator::EulerIntegrator;
use simcore::registry::Registry;
use simcore::state::State;

use crate::cabin::build_cabin_flows;
use crate::driver::run_master_day;
use crate::flows::{Harvest, HarvestParams, HARVEST};
use crate::greenhouse::{build_greenhouse, greenhouse_bio_resolver, greenhouse_cabin_resolver};
use crate::scenario::HarvestScenario;

/// Assemble the harvest greenhouse: `(state, bio_reg, cabin_reg)`.
///
/// Reuses [`build_greenhouse`] (the sealed biosphere ↔ cabin gas loop), then: (1) starts the
/// biosphere `thermal_time` aux at `scenario.thermal_time0` (past anthesis ⇒ grain-filling);
/// (2) appends the [`Harvest`] flow to the cabin / fast registry (`with_harvest`); and (3)
/// `close_feces` re-points fecal carbon into `LITTER_CARBON` (omitting the `FECAL_WASTE`
/// sink). The bio/cabin **flow-id** sets are asserted disjoint.
pub fn build_harvest(
    crew: &CrewParams,
    eclss: &EclssParams,
    harvest: &HarvestParams,
    scenario: &HarvestScenario,
    with_harvest: bool,
    close_feces: bool,
) -> Result<(State, Registry, Registry), SimError> {
    let fecal_target = if close_feces {
        LITTER_CARBON
    } else {
        FECAL_WASTE
    };
    let (gh_state, bio_reg, _gh_cabin_reg) =
        build_greenhouse(crew, eclss, &scenario.greenhouse, true, fecal_target)?;

    // (1) Start the biosphere phenology past anthesis (a grain-filling plant) — a
    // station-level aux injection over the greenhouse State's stocks.
    let state = State::new(
        gh_state.n,
        gh_state.stocks.clone(),
        gh_state.rng_seed,
        BTreeMap::from([(THERMAL_TIME.to_string(), scenario.thermal_time0)]),
    )?;

    // (2) Rebuild the cabin flows (the Rust Registry does not lend out owned flows) and
    // append Harvest — mirroring Python's `list(cabin_reg.flows) + [Harvest(...)]`.
    let mut cabin_flows = build_cabin_flows(crew, eclss, CARBON_POOL, O2_POOL, fecal_target);
    if with_harvest {
        cabin_flows.push(Box::new(Harvest::new(
            HARVEST.to_string(),
            STORAGE_C.to_string(),
            FOOD_STORE.to_string(),
            *harvest,
        )));
    }
    let cabin_reg = Registry::flows_only(cabin_flows, &state.stocks)?;

    assert_flow_ids_disjoint(&bio_reg, &cabin_reg)?;
    Ok((state, bio_reg, cabin_reg))
}

/// Guard: the biosphere-slow and cabin-fast registries share no `FlowId`.
fn assert_flow_ids_disjoint(bio_reg: &Registry, cabin_reg: &Registry) -> Result<(), SimError> {
    let bio_ids: std::collections::BTreeSet<&str> =
        bio_reg.flows().iter().map(|f| f.id()).collect();
    for flow in cabin_reg.flows() {
        if bio_ids.contains(flow.id()) {
            return Err(SimError::Validation(format!(
                "harvest flow-id collision between the biosphere and the cabin registries: \
                 {:?} (the two flow sets the driver steps together must be disjoint)",
                flow.id()
            )));
        }
    }
    Ok(())
}

/// The biosphere forcing resolver — the greenhouse's, over the embedded scenario.
pub fn harvest_bio_resolver(scenario: &HarvestScenario) -> Result<SourceResolver, SimError> {
    greenhouse_bio_resolver(&scenario.greenhouse)
}

/// The cabin forcing resolver — the greenhouse's two constant crew intake rates.
pub fn harvest_cabin_resolver(scenario: &HarvestScenario) -> Result<SourceResolver, SimError> {
    greenhouse_cabin_resolver(&scenario.greenhouse)
}

/// The two-rate driver: one day per step (biosphere-slow / cabin-fast).
pub fn run_harvest(
    bio_integrator: &EulerIntegrator,
    cabin_integrator: &EulerIntegrator,
    state: State,
    bio_resolver: &SourceResolver,
    cabin_resolver: &SourceResolver,
    scenario: &HarvestScenario,
) -> Result<(Vec<State>, u64, Vec<Event>), SimError> {
    let gh = &scenario.greenhouse;
    run_master_day(
        bio_integrator,
        cabin_integrator,
        state,
        bio_resolver,
        cabin_resolver,
        gh.days,
        gh.steps_per_day,
        gh.bio_dt,
        gh.cabin_dt,
        None,
    )
}
