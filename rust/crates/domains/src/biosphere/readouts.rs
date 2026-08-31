//! The **readouts**: one biosphere run reduced to the scalars a claim is read off.
//!
//! # Why this is a library module and not a test fixture
//!
//! Until 2026-08-27 all of this lived inside `#[cfg(test)] mod runs` / `mod folds` in
//! [`super::science_gates`], where each science gate reads it. That was the right home while
//! the only consumer was a gate — and it is why no non-test binary could obtain a *margin*:
//! [`super::science_gates::GATES`] is public at ordinary compile time, but it carries each
//! `bound` as a human-readable string, not an evaluator, so the numbers behind the claims were
//! unreachable from outside the test harness.
//!
//! The value-switch harness (`docs/plans/post-roadmap-value-switch-harness.md`, §5R) needs
//! exactly those numbers, under substituted params. Three routes were priced; this is the
//! narrow one. **What moved is the fixture, not the census.** Every gate declaration is
//! untouched, the `science_gates!` macro still emits the row and the assertion as one thing,
//! and `mod runs` / `mod folds` survive there as thin `#[cfg(test)]` shims that supply the
//! frozen params. Re-deriving these quantities in the harness instead was rejected outright:
//! *a rule with two copies has one that is stale*, and a margin computed twice is that rule's
//! worst case.
//!
//! # ⚠ Every function here is explicit about its params, and that is the point
//!
//! [`trajectory`] takes a `&BiosphereParams` and [`Trajectory`] **keeps the one it was run
//! with**, so a fold can never be applied to a run with somebody else's coefficients. The
//! caching that makes the gates cheap lives in `science_gates`, keyed by scenario — which is
//! safe there because the frozen params are the only ones it ever passes. ⚠ A cache keyed by
//! scenario alone on *this* side would hand a substituted run the frozen trajectory and the
//! harness would print "no change": §7 of the plan with a new cause.

use crate::biosphere::params::{BiosphereParams, MOLAR_MASS_CARBON_KG_PER_MOL};
use crate::biosphere::science::leaf_area_index;
use crate::biosphere::stocks::{CARBON_POOL, CONSUMER_CARBON, LEAF_C, STEM_C, STORAGE_C};
use crate::biosphere::system::sealed_chamber_scenario;
use crate::biosphere::{
    build_season_with, run_perennial, run_season, season_setup_composed, season_steps, steps_for,
    steps_for_years, SeasonBuild, SeasonScenario, BIO_DT, SEASON_DAYS,
};
use simcore::error::SimError;
use simcore::state::State;

/// One trajectory, reduced to the scalar series the gates fold.
///
/// ⚠⚠ **Pre-reduction is the station's precedent, and it opens a hole Python does not
/// have.** `station/examples/emit_sealed_energy_drift.rs` already folds a per-step
/// temperature series rather than materializing 109,801 `State`s, and `year_summaries`
/// is generic precisely so it can. But that fold computes `n_years = (len - 1) / year`,
/// so an observer emitting `steps` samples instead of `steps + 1` yields **14** annual
/// summaries instead of 15 — and every gate still passes, because `non_collapsing`
/// over 14 years passes exactly as well as over 15. Python never needed a guard for
/// that; the pre-reduction is what creates it. Hence [`Trajectory::years`] and the
/// count assertion in every decade gate.
pub struct Trajectory {
    /// The scenario this run was built from.
    pub scenario: SeasonScenario,
    /// The params this run was built from — frozen, or substituted by `domains::lab`.
    ///
    /// ⚠ Carried rather than assumed so that a fold reads *this run's* coefficients. A fold
    /// reaching for `params::canopy()` instead would silently read the frozen value and
    /// report a substituted run's quantity as if nothing had moved.
    pub params: BiosphereParams,
    /// `biosphere.leaf_c` per step (initial state included).
    pub leaf_c: Vec<f64>,
    /// `biosphere.stem_c` per step.
    pub stem_c: Vec<f64>,
    /// `biosphere.storage_c` per step.
    pub storage_c: Vec<f64>,
    /// `biosphere.carbon_pool` per step — the chamber atmosphere.
    pub carbon_pool: Vec<f64>,
    /// `biosphere.consumer_carbon` per step, or empty where the stock is absent.
    pub consumer_c: Vec<f64>,
    /// Arbitration firings over the whole run. A band is a claim about a *well-fed*
    /// run; a rationed run's trace is not the model's answer.
    pub rationed: u64,
    /// Extinction events over the whole run.
    pub events: usize,
    /// Seasons run — what the annual summary count must equal.
    pub years: usize,
}

impl Trajectory {
    /// Samples per season, in **steps** (the unit the trajectory is indexed by).
    pub fn year(&self) -> usize {
        steps_for(SEASON_DAYS)
    }
}

/// Run `scenario` for `years` seasons against `p`, keeping the per-step series.
///
/// `perennial` selects the annual reset. ⚠ Each frozen scenario must be driven **the way its
/// own golden drives it** — `sealed_chamber` through `run_season` with no re-sow, the chambers
/// through `run_perennial`. Driving them uniformly is how the sealed chamber once acquired a
/// compensation-point crossing it does not have.
pub fn trajectory(
    scenario: SeasonScenario,
    years: usize,
    perennial: bool,
    p: &BiosphereParams,
) -> Trajectory {
    trajectory_composed(scenario, years, perennial, p, &build_season_with)
}

/// [`trajectory`] against a caller-supplied **build** — the mechanism-switch seam's readouts.
///
/// The science half of the harness needs the same folds under a *composed* registry, and
/// there is exactly one reason this is a parameter rather than a second function: the
/// observer body below is where [`Trajectory`]'s whole contract lives — which stocks are
/// sampled, the empty-series hazard, the `steps + 1` count assertion. A second copy of it for
/// composed runs would be the two-assembly-bodies defect the mechanism lab was built to
/// close, one layer up. ⚠ So [`trajectory`] is this function at the frozen build, not its
/// sibling: there is no path by which the two can drift.
pub fn trajectory_composed(
    scenario: SeasonScenario,
    years: usize,
    perennial: bool,
    p: &BiosphereParams,
    build: SeasonBuild<'_>,
) -> Trajectory {
    try_trajectory_composed(scenario, years, perennial, p, build).expect("trajectory")
}

/// Why a composed trajectory could not be produced — and the distinction is the whole point.
///
/// ⚠ A caller composing mechanisms needs to tell these two apart, and nothing else in the
/// tree needed to before: [`Setup`](TrajectoryError::Setup) is a bad **request**, wrong under
/// every scenario, and must stop a comparison; [`Run`](TrajectoryError::Run) is a fact about
/// *this* scenario under *this* mechanism set, and is a result worth printing. Collapsing them
/// would mean either aborting a whole report because one chamber died, or swallowing a
/// malformed request as if it were a scientific outcome.
#[derive(Debug)]
pub enum TrajectoryError {
    /// The season could not be assembled — the composition is malformed.
    Setup(SimError),
    /// The season was assembled and the run did not survive it.
    ///
    /// ⚠ This is the ordinary outcome of knocking out a load-bearing process, not an edge
    /// case: remove root water uptake and the perennial chambers raise at the annual reset
    /// (*"seed bank too small to re-sow"*), because the crop never stored enough carbon to
    /// sow the next season. Measured 2026-08-31, on the first mechanism column ever built.
    Run(SimError),
}

impl std::fmt::Display for TrajectoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrajectoryError::Setup(e) => write!(f, "the season could not be assembled: {e}"),
            TrajectoryError::Run(e) => write!(f, "the run did not survive the season: {e}"),
        }
    }
}

/// [`trajectory_composed`], with the two failure modes separated rather than panicked on.
pub fn try_trajectory_composed(
    scenario: SeasonScenario,
    years: usize,
    perennial: bool,
    p: &BiosphereParams,
    build: SeasonBuild<'_>,
) -> Result<Trajectory, TrajectoryError> {
    let (state, integrator, resolver) =
        season_setup_composed(&scenario, years, p, build).map_err(TrajectoryError::Setup)?;
    let steps = steps_for_years(years);
    let mut t = Trajectory {
        scenario,
        params: p.clone(),
        leaf_c: Vec::with_capacity(steps + 1),
        stem_c: Vec::with_capacity(steps + 1),
        storage_c: Vec::with_capacity(steps + 1),
        carbon_pool: Vec::with_capacity(steps + 1),
        consumer_c: Vec::new(),
        rationed: 0,
        events: 0,
        years,
    };
    {
        let mut observe = |s: &State| {
            t.leaf_c.push(s.stocks[LEAF_C].amount);
            t.stem_c.push(s.stocks[STEM_C].amount);
            t.storage_c.push(s.stocks[STORAGE_C].amount);
            // ⚠ Both of these are present only in some scenarios, and the empty
            // series that leaves is a SILENT-PASS hazard, not a convenience: an open
            // field has no `biosphere.carbon_pool` at all (unsealed runs draw on the
            // boundary atmosphere), and only the consumer chambers carry a herbivore.
            // A fold over an empty series returns the identity — `min` returns
            // +infinity, which is happily "above the compensation point". The folds
            // that read them assert non-emptiness for exactly that reason.
            //
            // ⚠ A **mechanism** composition cannot reach that hazard, and the
            // science-switch plan's slice 4 said it could. Stock presence is decided
            // by `build_season_with`'s compartments and a composition only rewrites
            // the flow list, so every series here is exactly as long as the frozen
            // run's. What a swap CAN do is leave a series **constant** — remove a
            // stock's only writer and `min_ppm` returns the initial charge, which is
            // finite, plausible, and reads as comfortably above the floor. That is
            // the worse failure, because `+inf` is conspicuous; the guard for it is
            // `lab::report`'s constancy check, not this assertion.
            if let Some(stock) = s.stocks.get(CARBON_POOL) {
                t.carbon_pool.push(stock.amount);
            }
            if let Some(stock) = s.stocks.get(CONSUMER_CARBON) {
                t.consumer_c.push(stock.amount);
            }
        };
        let outcome = if perennial {
            run_perennial(
                &integrator,
                state,
                &t.scenario,
                &resolver,
                BIO_DT,
                steps,
                season_steps(),
                &mut observe,
            )
        } else {
            run_season(
                &integrator,
                state,
                &resolver,
                BIO_DT,
                steps,
                None,
                &mut observe,
            )
        };
        let (_final, rationed, events) = outcome.map_err(TrajectoryError::Run)?;
        t.rationed = rationed;
        t.events = events.len();
    }
    // The observer contract this whole module's arithmetic rests on: `run_season`
    // calls it on the initial state AND each produced state. Asserted rather than
    // trusted — see the `Trajectory` note.
    //
    // ⚠ Still an assertion and not a `TrajectoryError`: a short sample count is a
    // broken observer, which is a defect in this file, not an outcome a composed run
    // can produce. A run that ends early raises above instead.
    assert_eq!(t.leaf_c.len(), steps + 1, "observer sample count");
    Ok(t)
}

// ---------------------------------------------------------------------------------
// The folds the gates share.
// ---------------------------------------------------------------------------------

/// kg C / kg DM — Greenwood's basis (`nitrogen.yaml` / `canopy.yaml`, cited).
pub const CARBON_FRACTION: f64 = 0.45;

/// The CO₂ compensation point in **chamber** ppm, from `p`.
///
/// `Γ*` is the compensation point in the *intercellular* air; the gate is on the
/// *ambient* air, and the two are related by the C3 set point `Ci = ci_ratio · Ca`
/// the sealed carbon budget already uses. So the ambient floor is `Γ*/ci_ratio`.
/// Computed, never typed — which is why the recorded 61.07 needs its own tripwire.
pub fn floor_ppm(p: &BiosphereParams) -> f64 {
    p.photo.gamma_star / sealed_chamber_scenario().ci_ratio
}

/// Minimum chamber CO₂ (ppm) over the whole trajectory.
pub fn min_ppm(t: &Trajectory) -> f64 {
    // ⚠ Not defensive tidying: an empty series folds to +infinity, which passes
    // `min > floor` vacuously. An unsealed scenario reaching this fold is a wiring
    // error that must be loud, not a band that quietly holds.
    assert!(
        !t.carbon_pool.is_empty(),
        "min_ppm on a run with no chamber carbon pool"
    );
    let air = t.scenario.chamber_air_mol;
    t.carbon_pool
        .iter()
        .map(|c| c / air * 1e6)
        .fold(f64::INFINITY, f64::min)
}

/// Peak leaf area index over the whole trajectory.
pub fn peak_lai(t: &Trajectory) -> f64 {
    let sla = t.params.canopy.sla_per_mol_c;
    t.leaf_c
        .iter()
        .map(|c| leaf_area_index(*c, sla, t.scenario.ground_area))
        .fold(f64::NEG_INFINITY, f64::max)
}

/// mol C → t DM/ha on Greenwood's basis (1 kg/m² == 10 t/ha).
pub fn t_per_ha(mol_c: f64, ground_area: f64) -> f64 {
    ((mol_c * MOLAR_MASS_CARBON_KG_PER_MOL / CARBON_FRACTION) / ground_area) * 10.0
}

/// Peak whole-plant mass **excluding fibrous roots** (t/ha) — Greenwood's W.
pub fn peak_w(t: &Trajectory) -> f64 {
    (0..t.leaf_c.len())
        .map(|i| {
            t_per_ha(
                t.leaf_c[i] + t.stem_c[i] + t.storage_c[i],
                t.scenario.ground_area,
            )
        })
        .fold(f64::NEG_INFINITY, f64::max)
}

/// Per-year maximum of a per-step series (the `_peak_leaf` / segment-max fold).
pub fn segment_max(seg: &[f64]) -> f64 {
    seg.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))
}

/// Per-year minimum of a per-step series (the `_min_carbon_pool` fold).
pub fn segment_min(seg: &[f64]) -> f64 {
    seg.iter().fold(f64::INFINITY, |a, &b| a.min(b))
}

/// Per-year LAST value of a per-step series (the `_year_end_consumer` fold).
pub fn segment_last(seg: &[f64]) -> f64 {
    *seg.last().expect("non-empty year segment")
}

/// `max` over a slice — the scale the relative stationarity bounds are taken against.
pub fn scale_of(values: &[f64]) -> f64 {
    segment_max(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biosphere::params;
    use crate::biosphere::DEFAULT_SCENARIO;

    fn stub(params: BiosphereParams) -> Trajectory {
        Trajectory {
            scenario: DEFAULT_SCENARIO,
            params,
            leaf_c: vec![0.0, 1.0, 0.5],
            stem_c: vec![0.0, 1.0, 0.5],
            storage_c: vec![0.0, 0.0, 0.0],
            carbon_pool: Vec::new(),
            consumer_c: Vec::new(),
            rationed: 0,
            events: 0,
            years: 1,
        }
    }

    /// ⚠⚠ **This test exists because the obvious one is blind to the defect it is about.**
    /// Mutating [`peak_lai`] to read `params::canopy()` instead of the trajectory's own params
    /// was measured against the whole end-to-end suite: `tests/value_switch_run.rs` stayed
    /// GREEN, because it substitutes `extinction_coef` and this fold reads
    /// `specific_leaf_area`. Only `tests/param_funnel.rs` caught it, and only because the
    /// mutation happened to add a production param load — a version that took the frozen value
    /// through some other route would have been invisible to the entire tree.
    ///
    /// So the property is asserted directly: two trajectories with the same series and
    /// different params must fold differently.
    #[test]
    fn a_fold_reads_the_trajectorys_own_params_not_the_frozen_ones() {
        let frozen = params::biosphere();
        let at_frozen = peak_lai(&stub(frozen.clone()));
        let mut doubled = frozen.clone();
        doubled.canopy.sla_per_mol_c *= 2.0;
        let at_doubled = peak_lai(&stub(doubled));
        assert!(
            at_doubled > at_frozen,
            "peak_lai ignored the trajectory's params: {at_doubled} vs {at_frozen}"
        );

        // The same, for the compensation-point floor, which reads `photo.gamma_star`.
        let mut warmer = frozen.clone();
        warmer.photo.gamma_star *= 1.5;
        assert!(floor_ppm(&warmer) > floor_ppm(&frozen));
    }

    /// The empty-series hazard the observer note names: an unsealed run has no chamber pool,
    /// and a silent `+inf` would pass "above the compensation point" vacuously.
    #[test]
    fn a_chamber_fold_on_an_unsealed_run_is_loud() {
        let t = stub(params::biosphere());
        assert!(t.carbon_pool.is_empty(), "the premise is gone");
        assert!(std::panic::catch_unwind(|| min_ppm(&t)).is_err());
    }
}
