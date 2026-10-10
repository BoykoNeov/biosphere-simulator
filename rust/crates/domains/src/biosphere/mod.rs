//! Native Rust port of the frozen Python biosphere (Phase-7 P7.4).
//!
//! Mirrors `domains.biosphere`: the clean-room crop science (FvCB photosynthesis,
//! Penman–Monteith transpiration, thermal-time phenology, allocation, the coupled carbon
//! budget, nitrogen, the water cycle, decomposition/mineralization, the minimal consumer),
//! the compartment builders, and `run_season`/`annual_reset_with`/`run_perennial_with`. Every rate
//! law and flow `evaluate` mirrors the Python arithmetic character-for-character and every
//! `math.*` op-for-op (`exp` to `.exp()`, `sqrt` to `.sqrt()`, `q10**e` to `.powf(e)`), so
//! the cross-port deviation is bounded by last-ULP libm differences (all 7 biosphere
//! goldens are Tier 2; the biosphere is Euler-locked by its freeze — no RK4 cross-check).

// ⚠ TEST-ONLY, deliberately. Unlike `params::param_files` -- which the manifest writer
// calls at runtime -- the claim census has no runtime consumer, and its gate READS THE
// SOURCE DIRECTORY FROM DISK. A spine module that does file I/O in a shipped build is
// exactly what `biosphere_spine_purity` exists to prevent; `#[cfg(test)]` keeps the
// filesystem out of the engine and the census where it belongs, beside the suite it
// measures.
#[cfg(test)]
pub mod claim_census;
pub mod compartments;
pub mod drift;
pub mod flows;
pub mod light_path;
pub mod params;
pub mod perturbations;
pub mod readouts;
pub mod science;
pub mod science_gates;
pub mod stocks;
pub mod system;
pub mod weather;

use simcore::environment::SourceResolver;
use simcore::error::SimError;
use simcore::integrator::EulerIntegrator;
use simcore::registry::Registry;
use simcore::state::State;

pub use system::{
    annual_reset_with, build_season, build_season_with, consumer_chamber_scenario,
    perennial_chamber_scenario, potato_scenario, run_perennial_with, run_season,
    sealed_chamber_scenario, weather_resolver, SeasonScenario, CONSUMER_CHAMBER_YEARS,
    DEFAULT_SCENARIO, LONG_HORIZON_YEARS, PERENNIAL_CHAMBER_YEARS, SEALED_CHAMBER_YEARS,
};

/// The biosphere's integration step, in days: **1/16** since 2026-09-30.
///
/// The reference's own choice since the flip (Python, whose `step.py` this once mirrored, is
/// retired). History: `1` until 2026-08-14 (`docs/plans/post-roadmap-step-unfreeze.md`), `1/4`
/// until 2026-09-30, then `1/16` on the user's call after the step options were priced against
/// a 1/256-day answer (`docs/plans/post-roadmap-step-sixteenth.md`). A power of two, so
/// `n · dt` stays exact, and it divides the station's 1440 cabin minutes into 90 per plant step.
pub const BIO_DT: f64 = 0.0625;

/// Integration steps per physical day; `BIO_DT · STEPS_PER_DAY == 1`.
pub const STEPS_PER_DAY: usize = 16;

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
    // ⚠ Via `build_season`, NOT via `season_setup_with(.., &params::biosphere())`. Both
    // spell the same run; only this one leaves the whole spine with a **single** production
    // param load, which is the property `tests/param_funnel.rs` gates. A second frozen-params
    // call here would cost nothing today and turn that gate into a roster of allowed sites.
    let (state, registry) = build_season(scenario)?;
    let resolver = weather_resolver(scenario, weather_years)?;
    Ok((state, EulerIntegrator::new(registry), resolver))
}

/// [`season_setup`] against caller-supplied params — the value-switch seam's pass-through.
///
/// ⚠ **The weather is deliberately NOT part of the seam.** `weather_resolver` reads the
/// committed fixture and the scenario's own fields; a param substitution must not be able to
/// change the forcing, or an A/B table stops being about the coefficient. See
/// [`system::build_season_with`] for the seam's own note.
pub fn season_setup_with(
    scenario: &SeasonScenario,
    weather_years: usize,
    p: &params::BiosphereParams,
) -> Result<(State, EulerIntegrator, SourceResolver), SimError> {
    season_setup_composed(scenario, weather_years, p, &system::build_season_with)
}

/// How a season's `(State, Registry)` is obtained — [`system::build_season_with`], or a
/// `domains::lab::mechanism` composition of it.
///
/// ⚠ The **science** seam's pass-through, and it is deliberately a build rather than a
/// registry: a caller handing in a pre-built pair could hand in one assembled from a
/// different scenario than the resolver is bound to, and nothing here could tell. Taking the
/// build keeps the scenario the single input it already is.
pub type SeasonBuild<'a> =
    &'a dyn Fn(&SeasonScenario, &params::BiosphereParams) -> Result<(State, Registry), SimError>;

/// [`season_setup_with`] against a caller-supplied **build** — the mechanism-switch seam.
///
/// ⚠ The weather note on [`season_setup_with`] applies here unchanged and more strongly: a
/// mechanism swap must not be able to move the forcing either, or an A/B table stops being
/// about the process. `weather_resolver` reads the committed fixture and the scenario, and
/// neither is reachable from `build`.
pub fn season_setup_composed(
    scenario: &SeasonScenario,
    weather_years: usize,
    p: &params::BiosphereParams,
    build: SeasonBuild<'_>,
) -> Result<(State, EulerIntegrator, SourceResolver), SimError> {
    let (state, registry) = build(scenario, p)?;
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

/// Run `scenario` built from `p` with [`annual_reset_with`] every season, final `State` only.
///
/// ⚠ Takes `p` rather than loading the frozen params: the re-sow must see the params the season
/// was built from, and a load here would be a second production site in the spine, the thing
/// `tests/param_funnel.rs` gates (`docs/plans/post-roadmap-soil-evaporation.md` §16d).
pub fn run_perennial_final(
    scenario: &SeasonScenario,
    weather_years: usize,
    p: &params::BiosphereParams,
) -> Result<(State, u64, Vec<simcore::events::Event>), SimError> {
    let (state, integrator, resolver) = season_setup_with(scenario, weather_years, p)?;
    let steps = steps_for_years(weather_years);
    let mut noop = |_: &State| {};
    run_perennial_with(
        &integrator,
        state,
        scenario,
        p,
        &resolver,
        BIO_DT,
        steps,
        season_steps(),
        &mut noop,
    )
}
