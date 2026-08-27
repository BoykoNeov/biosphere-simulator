//! The value-switch seam, end to end: a substituted param reaches the **run**.
//!
//! `lab.rs` proves a substitution reaches the params object and `param_funnel.rs` proves the
//! params object is the only thing a run reads. This file closes the loop by running the
//! season both ways, which is the only test that can fail if either of those two is right
//! about its own half and wrong about the join.
//!
//! ## Both directions, deliberately
//!
//! A one-direction test here would be worth little. *"A substituted run differs"* passes if
//! the lab silently returned garbage; *"a baseline run matches"* passes if the seam ignored
//! the substitution entirely. This tree's recorded lesson is that a no-op diff needs a
//! two-direction control (`memory/manifest-reanchored-mixed-authority`), so:
//!
//! * with no substitutions the run is **bit-identical** to the ordinary frozen entry point;
//! * with one substitution it **differs**, and it differs in the direction the science says.

use domains::biosphere::params::BiosphereParams;
use domains::biosphere::stocks::LEAF_C;
use domains::biosphere::{
    run_season, season_setup, season_setup_with, steps_for_years, SeasonScenario, BIO_DT,
    DEFAULT_SCENARIO,
};
use domains::lab::{biosphere_with, Substitution};
use simcore::state::State;

/// The open field's leaf-carbon series, run against `p`.
fn leaf_series(scenario: &SeasonScenario, p: &BiosphereParams) -> Vec<f64> {
    let (state, integrator, resolver) = season_setup_with(scenario, 1, p).expect("setup");
    collect(state, integrator, resolver)
}

/// The same series through the ordinary frozen entry point — no `_with` anywhere.
fn frozen_leaf_series(scenario: &SeasonScenario) -> Vec<f64> {
    let (state, integrator, resolver) = season_setup(scenario, 1).expect("setup");
    collect(state, integrator, resolver)
}

fn collect(
    state: State,
    integrator: simcore::integrator::EulerIntegrator,
    resolver: simcore::environment::SourceResolver,
) -> Vec<f64> {
    let steps = steps_for_years(1);
    let mut series = Vec::with_capacity(steps + 1);
    {
        let mut observe = |s: &State| series.push(s.stocks[LEAF_C].amount);
        let (_final, rationed, events) = run_season(
            &integrator,
            state,
            &resolver,
            BIO_DT,
            steps,
            None,
            &mut observe,
        )
        .expect("season");
        assert_eq!(rationed, 0, "an A/B run must be well-fed to be comparable");
        assert!(events.is_empty(), "unexpected extinction: {events:?}");
    }
    series
}

/// Direction 1: the seam changes nothing when nothing is substituted.
#[test]
fn an_unsubstituted_run_is_bit_identical_to_the_frozen_path() {
    let frozen = frozen_leaf_series(&DEFAULT_SCENARIO);
    let through_seam = leaf_series(&DEFAULT_SCENARIO, &biosphere_with(&[]).expect("empty"));
    assert_eq!(frozen.len(), through_seam.len());
    for (i, (a, b)) in frozen.iter().zip(&through_seam).enumerate() {
        assert_eq!(a.to_bits(), b.to_bits(), "step {i}: {a:?} vs {b:?}");
    }
}

/// Direction 2: a substitution reaches the run — and the empty diff above was not the
/// seam quietly ignoring its argument.
///
/// ⚠ The assertion is on the **direction**, not on a value. A larger canopy extinction
/// coefficient intercepts more light per unit leaf area, so the season's peak leaf carbon
/// rises. Pinning the number would freeze an experimental result into a test, which is
/// exactly what this harness exists to avoid: it regenerates evidence, it does not enshrine
/// it. `extinction_coef` is still 0.6 in the tree and this test does not move it.
#[test]
fn a_substituted_run_differs_and_in_the_direction_the_science_says() {
    let base = frozen_leaf_series(&DEFAULT_SCENARIO);
    let higher_k = leaf_series(
        &DEFAULT_SCENARIO,
        &biosphere_with(&[Substitution::new("canopy.yaml", "extinction_coef", 0.65)])
            .expect("substitution"),
    );
    assert!(
        base.iter()
            .zip(&higher_k)
            .any(|(a, b)| a.to_bits() != b.to_bits()),
        "the substituted run is bit-identical to the baseline — the override never reached \
         the run"
    );
    let peak = |s: &[f64]| s.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    assert!(
        peak(&higher_k) > peak(&base),
        "peak leaf carbon fell at the higher extinction coefficient: {} vs {}",
        peak(&higher_k),
        peak(&base)
    );
}

/// ⚠ The one that would have caught a lab returning a *different* baseline. The two
/// substitutions above and below bracket the frozen value, so the frozen run must sit
/// between them — a property no single comparison can see.
#[test]
fn the_frozen_run_sits_between_a_lower_and_a_higher_substitution() {
    let peak = |s: &[f64]| s.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let at = |k: f64| {
        peak(&leaf_series(
            &DEFAULT_SCENARIO,
            &biosphere_with(&[Substitution::new("canopy.yaml", "extinction_coef", k)])
                .expect("substitution"),
        ))
    };
    let base = peak(&frozen_leaf_series(&DEFAULT_SCENARIO));
    let (low, high) = (at(0.55), at(0.65));
    assert!(
        low < base && base < high,
        "frozen {base} is not between k=0.55 ({low}) and k=0.65 ({high})"
    );
}
