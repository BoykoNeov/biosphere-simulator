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

use domains::biosphere::science;
use domains::biosphere::stocks::{
    CARBON_POOL, LITTER_CARBON, O2_POOL, ROOTED_DEPTH, SOIL_WATER, STORAGE_C, SUBSOIL_WATER,
    THERMAL_TIME,
};
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
    // ⚠ THE INJECTED DEPTH DRAGS THE WATER STORES WITH IT (2026-08-12). Injecting a
    // 1.3 m root system while the stocks still hold the SOWING zone's water is incoherent
    // once the store is geometric: the biosphere seeds `soil_water0` for a 0.15 m zone,
    // and that inside a 1.3 m zone is `FTSW = 0.115` — a grain-filling crop declared at a
    // third of its growth threshold on day 0. Re-derived here from the same [F] Eqns
    // 14.26-14.28 the scenario defaults use, exactly as Python does.
    let bio = &scenario.greenhouse.bio;
    let atsw = science::captured_water(
        scenario.rooted_depth0,
        bio.soil_extractable_water,
        bio.ground_area,
    ) * bio.soil_moisture_index;
    let wstorg = science::captured_water(
        bio.soil_depth - scenario.rooted_depth0,
        bio.soil_extractable_water,
        bio.ground_area,
    ) * bio.soil_moisture_index;
    let mut stocks = gh_state.stocks.clone();
    let sw = stocks[SOIL_WATER].with_amount(atsw)?;
    stocks.insert(SOIL_WATER.to_string(), sw);
    let sub = stocks[SUBSOIL_WATER].with_amount(wstorg)?;
    stocks.insert(SUBSOIL_WATER.to_string(), sub);
    let state = State::new(
        gh_state.n,
        stocks,
        gh_state.rng_seed,
        BTreeMap::from([
            (THERMAL_TIME.to_string(), scenario.thermal_time0),
            // Goes with thermal_time0: a crop started past anthesis has finished rooting.
            (ROOTED_DEPTH.to_string(), scenario.rooted_depth0),
        ]),
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
        gh.bio_steps_per_day,
        gh.bio_dt,
        gh.cabin_dt,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params as station_params;
    use crate::scenario::harvest_scenario;
    use domains::biosphere::science;
    use domains::params;

    /// The station's past-anthesis injection keeps DEPTH and WATER together — the place
    /// this gap actually bit.
    ///
    /// `build_harvest` overrides `rooted_depth0` to 1.3 m on top of a greenhouse built for
    /// the 0.15 m sowing zone. Before 2026-08-12 it inherited that zone's water: 19.5 kg
    /// inside a 169 kg capacity, `FTSW = 0.115`, grain 79 % low — a grain-filling crop
    /// declared at a third of its growth threshold on day 0. Both stores are now re-derived
    /// from the injected depth, and this asserts the resulting STATE rather than the code
    /// path, so a future refactor cannot quietly drop it.
    ///
    /// ⚠ This is the concrete case behind
    /// `scenario.rs`'s `every_scenarios_water_stores_are_geometric`: that census covers
    /// declared scenarios, and the harvest injection is a store rewritten at BUILD time,
    /// which no scenario-level census can see.
    /// Mirrors `tests/test_soil_layers.py::test_the_harvest_injection_keeps_depth_and_
    /// water_together`.
    #[test]
    fn the_harvest_injection_keeps_depth_and_water_together() {
        let crew = params::crew();
        let eclss = params::eclss();
        let harvest_params = station_params::harvest();
        let scenario = harvest_scenario();
        let (state, _bio, _cabin) =
            build_harvest(&crew, &eclss, &harvest_params, &scenario, true, true)
                .expect("build_harvest");
        let bio = &scenario.greenhouse.bio;
        let depth = state.aux[ROOTED_DEPTH];
        assert_eq!(depth, scenario.rooted_depth0);

        let capacity = science::captured_water(depth, bio.soil_extractable_water, bio.ground_area);
        let held = state.stocks[SOIL_WATER].amount;
        assert!(
            (held - capacity * bio.soil_moisture_index).abs() <= 1e-12 * capacity,
            "the injected crop's root zone holds {held}, not its own geometry {capacity}"
        );
        // ...which is to say it starts at its DECLARED FTSW, not at 0.115.
        let ftsw = held / capacity;
        assert!(
            (ftsw - bio.soil_moisture_index).abs() <= 1e-12,
            "the injected crop starts at FTSW {ftsw}, not the declared MAI {}",
            bio.soil_moisture_index
        );
        assert!(
            ftsw > 0.5,
            "the 0.115 regression is back: a grain-filling crop below its growth threshold"
        );
        // The below-root store follows the same re-derivation, or the profile stops being
        // a partition and the injection creates or destroys water.
        let below = state.stocks[SUBSOIL_WATER].amount;
        let want = science::captured_water(
            bio.soil_depth - depth,
            bio.soil_extractable_water,
            bio.ground_area,
        ) * bio.soil_moisture_index;
        assert!(
            (below - want).abs() <= 1e-12 * want,
            "the below-root store {below} is not (SOLDEP - DEPORT) geometry {want}"
        );
        // ⚠ NON-VACUITY: the injection must actually have MOVED the stores. If the
        // greenhouse's own `soil_water0` happened to equal the injected value, every
        // assertion above would hold on a build that had dropped the re-derivation.
        assert_ne!(
            bio.soil_water0, held,
            "the injected depth did not change the store, so this test proves nothing"
        );
    }
}
