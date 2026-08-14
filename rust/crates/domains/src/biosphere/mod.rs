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
pub mod light_path;
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

/// The biosphere's integration step, in days — mirrors `domains.biosphere.step.BIO_DT`.
///
/// ⚠ **The port has no reference authority.** This is a mirror of the Python constant,
/// not an independent choice; the reason it is `1/4` and not `1` lives in
/// `src/domains/biosphere/step.py` and `docs/plans/post-roadmap-step-unfreeze.md`. If the
/// two ever disagree, Python is right by definition.
pub const BIO_DT: f64 = 0.25;

/// Integration steps per physical day — mirrors `domains.biosphere.step.STEPS_PER_DAY`.
pub const STEPS_PER_DAY: usize = 4;

/// Integration steps in `days` physical days — the exact analogue of the Python
/// `domains.biosphere.step.steps_for`, **taking the same unit**.
///
/// The weather table itself stays one row per physical **day** at any step
/// (`table_schedule` indexes `int(n · dt)`), so it must never be tiled to match
/// `STEPS_PER_DAY`.
pub fn steps_for(days: usize) -> usize {
    days * STEPS_PER_DAY
}

/// Steps in a `weather_years`-tiled run.
///
/// ⚠ **Takes YEARS, not days** — hence the name. This was called `steps_for` until
/// 2026-08-14, when it took years while the Python `steps_for` took days: the same name
/// meaning two different units across the two ports, which is exactly the conflation this
/// step ceremony existed to remove. Renamed rather than left to the next reader who sees
/// `steps_for` on both sides and assumes one meaning.
pub fn steps_for_years(weather_years: usize) -> usize {
    steps_for(SEASON_DAYS * weather_years)
}

/// The reset period for a perennial run, in **steps** (one season).
pub fn season_steps() -> usize {
    steps_for(SEASON_DAYS)
}

/// The committed weather fixture's season length (305 physical days).
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
    let steps = steps_for_years(weather_years);
    let mut noop = |_: &State| {};
    run_season(
        &integrator,
        state,
        &resolver,
        BIO_DT,
        steps,
        None,
        &mut noop,
    )
}

/// Run `scenario` with `annual_reset` every season, final `State` only.
pub fn run_perennial_final(
    scenario: &SeasonScenario,
    weather_years: usize,
) -> Result<(State, u64, Vec<simcore::events::Event>), SimError> {
    let (state, integrator, resolver) = season_setup(scenario, weather_years)?;
    let steps = steps_for_years(weather_years);
    let mut noop = |_: &State| {};
    run_perennial(
        &integrator,
        state,
        scenario,
        &resolver,
        BIO_DT,
        steps,
        season_steps(),
        &mut noop,
    )
}
