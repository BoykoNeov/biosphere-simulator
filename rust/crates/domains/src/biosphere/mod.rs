//! Native Rust port of the frozen Python biosphere (Phase-7 P7.4).
//!
//! Mirrors `domains.biosphere`: the clean-room crop science (FvCB photosynthesis,
//! Penman–Monteith transpiration, thermal-time phenology, allocation, the coupled carbon
//! budget, nitrogen, the water cycle, decomposition/mineralization, the minimal consumer),
//! the compartment builders, and `run_season`/`annual_reset`/`run_perennial`. Every rate
//! law and flow `evaluate` mirrors the Python arithmetic character-for-character and every
//! `math.*` op-for-op (`exp` to `.exp()`, `sqrt` to `.sqrt()`, `q10**e` to `.powf(e)`), so
//! the cross-port deviation is bounded by last-ULP libm differences (all 7 biosphere
//! goldens are Tier 2; the biosphere is Euler-locked by its freeze — no RK4 cross-check).

pub mod flows;
pub mod params;
pub mod perturbations;
pub mod science;
pub mod stocks;
pub mod system;
pub mod weather;

use simcore::environment::SourceResolver;
use simcore::error::SimError;
use simcore::integrator::EulerIntegrator;
use simcore::state::State;

pub use system::{
    annual_reset, build_season, consumer_chamber_scenario, perennial_chamber_scenario,
    run_perennial, run_season, sealed_chamber_scenario, weather_resolver, SeasonScenario,
    CONSUMER_CHAMBER_YEARS, DEFAULT_SCENARIO, LONG_HORIZON_YEARS, PERENNIAL_CHAMBER_YEARS,
    SEALED_CHAMBER_YEARS,
};

/// Steps in a `weather_years`-tiled run: `weather_years · SEASON_DAYS`.
pub fn steps_for(weather_years: usize) -> usize {
    SEASON_DAYS * weather_years
}

/// The committed weather fixture's season length (305 days).
pub const SEASON_DAYS: usize = 305;

/// Build the Euler integrator + tiled resolver for `scenario` over `weather_years`.
pub fn season_setup(
    scenario: &SeasonScenario,
    weather_years: usize,
) -> Result<(State, EulerIntegrator, SourceResolver), SimError> {
    let (state, registry) = build_season(scenario)?;
    let resolver = weather_resolver(scenario, weather_years)?;
    Ok((state, EulerIntegrator::new(registry), resolver))
}

/// Run `scenario` for `weather_years` tiled seasons (no reset), final `State` only.
pub fn run_season_final(
    scenario: &SeasonScenario,
    weather_years: usize,
) -> Result<(State, u64, Vec<simcore::events::Event>), SimError> {
    let (state, integrator, resolver) = season_setup(scenario, weather_years)?;
    let steps = steps_for(weather_years);
    let mut noop = |_: &State| {};
    run_season(&integrator, state, &resolver, 1.0, steps, None, &mut noop)
}

/// Run `scenario` with `annual_reset` every `SEASON_DAYS`, final `State` only.
pub fn run_perennial_final(
    scenario: &SeasonScenario,
    weather_years: usize,
) -> Result<(State, u64, Vec<simcore::events::Event>), SimError> {
    let (state, integrator, resolver) = season_setup(scenario, weather_years)?;
    let steps = steps_for(weather_years);
    let mut noop = |_: &State| {};
    run_perennial(
        &integrator, state, scenario, &resolver, 1.0, steps, SEASON_DAYS, &mut noop,
    )
}
