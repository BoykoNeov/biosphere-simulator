//! The drift-instrument kit, checked against the reference it was ported from (slice C5).
//!
//! ⚠ The fixtures below are **the same numbers `tests/test_drift.py` uses**, on purpose.
//! These folds decide two frozen goldens and — from C4 — five of the fifteen science
//! gates, so the question a Rust test has to answer is not "is this self-consistent" but
//! "does it agree with the reference". Sharing the fixtures makes a behavioural divergence
//! show up as a failing assertion instead of a passing test that checks something else.
//!
//! An *integration* test rather than a `#[cfg(test)]` module, deliberately: it exercises
//! the kit through the public API, which is exactly how C4's gates and
//! `station/examples/emit_sealed_energy_drift.rs` reach it.

use domains::biosphere::drift::{
    drift_slope, is_period_2, is_stationary, least_squares_slope, mass_drift_trace, max_abs,
    non_collapsing, same_phase_diffs, total_quantity, year_summaries, MASS_DRIFT_ABS_BOUND,
    MASS_DRIFT_SLOPE_BOUND,
};
use simcore::ids::StockId;
use simcore::quantities::Quantity;
use simcore::quantities::StockKind;
use simcore::state::{State, Stock};
use std::collections::BTreeMap;

fn stock(id: &str, quantity: Quantity, amount: f64, comp: &[(Quantity, f64)]) -> Stock {
    Stock::new(
        id.to_string(),
        "test".into(),
        quantity,
        "mol".into(),
        amount,
        StockKind::Pool,
        0.0,
        false,
        comp.iter().copied().collect::<BTreeMap<_, _>>(),
    )
    .expect("stock")
}

fn state_of(n: u64, stocks: Vec<Stock>) -> State {
    let map: BTreeMap<StockId, Stock> = stocks.into_iter().map(|s| (s.id.clone(), s)).collect();
    State::new(n, map, 0, BTreeMap::new()).expect("state")
}

fn carbon(id: &str, amount: f64) -> Stock {
    stock(id, Quantity::Carbon, amount, &[(Quantity::Carbon, 1.0)])
}

// --- the promoted fold -------------------------------------------------------

#[test]
fn total_quantity_folds_composition_over_heterogeneous_stocks() {
    // A stock carrying none of the folded quantity contributes 0.0 — the default that
    // makes a *vented* leak still conserve, because the boundary sink holds the mass.
    let s = state_of(
        0,
        vec![
            carbon("c", 2.0),
            stock("w", Quantity::Water, 5.0, &[(Quantity::Water, 1.0)]),
            stock(
                "sugar",
                Quantity::Carbon,
                3.0,
                &[(Quantity::Carbon, 6.0), (Quantity::Oxygen, 6.0)],
            ),
        ],
    );
    assert_eq!(total_quantity(&s, Quantity::Carbon), 2.0 + 3.0 * 6.0);
    assert_eq!(total_quantity(&s, Quantity::Water), 5.0);
    assert_eq!(total_quantity(&s, Quantity::Oxygen), 18.0);
    assert_eq!(total_quantity(&s, Quantity::Nitrogen), 0.0);
}

// --- the shared primitive ----------------------------------------------------

#[test]
fn least_squares_slope_recovers_a_known_slope() {
    let values: Vec<f64> = (0..10).map(|i| 3.0 + 2.0 * i as f64).collect();
    assert!((least_squares_slope(&values) - 2.0).abs() < 1e-12);
    assert!(least_squares_slope(&[7.0; 6]).abs() < 1e-15);
}

#[test]
fn least_squares_slope_degenerate_inputs() {
    assert_eq!(least_squares_slope(&[]), 0.0);
    assert_eq!(least_squares_slope(&[42.0]), 0.0);
}

// --- axis (a): mass-conservation drift ---------------------------------------

#[test]
fn max_abs_reads_magnitude_and_defaults_to_zero() {
    assert_eq!(max_abs(&[1.0, -5.0, 3.0]), 5.0);
    assert_eq!(max_abs(&[]), 0.0);
}

#[test]
fn mass_drift_trace_is_relative_to_step_zero() {
    let states = vec![
        state_of(0, vec![carbon("c", 10.0)]),
        state_of(1, vec![carbon("c", 10.5)]),
        state_of(2, vec![carbon("c", 9.5)]),
    ];
    assert_eq!(
        mass_drift_trace(&states, Quantity::Carbon),
        vec![0.0, 0.5, -0.5]
    );
}

#[test]
fn detector_discriminates_leak_from_roundoff() {
    // The bounds have teeth: a 1e-9/step leak trips both detectors, and round-off jitter
    // at the measured scale trips neither. This is what keeps MASS_DRIFT_*_BOUND honest —
    // a bound that no input can cross is not a detector.
    let leak: Vec<f64> = (0..4575).map(|n| 1e-9 * n as f64).collect();
    assert!(max_abs(&leak) > MASS_DRIFT_ABS_BOUND);
    assert!(drift_slope(&leak).abs() > MASS_DRIFT_SLOPE_BOUND);

    let jitter: Vec<f64> = (0..4575)
        .map(|n| if n % 2 == 0 { 3.0e-12 } else { -3.0e-12 })
        .collect();
    assert!(max_abs(&jitter) < MASS_DRIFT_ABS_BOUND);
    assert!(drift_slope(&jitter).abs() < MASS_DRIFT_SLOPE_BOUND);
}

// --- axis (b): the segmentation ----------------------------------------------

#[test]
fn year_summaries_segments_like_the_perennial_tests() {
    // year=3, 7 states (amounts 0..6) → 2 years; segment y spans [y*3 ..= (y+1)*3], i.e.
    // a full year PLUS the next year-boundary state. The inclusive end is load-bearing:
    // drop it and every per-year peak in both drift goldens changes value.
    let states: Vec<State> = (0..7)
        .map(|i| state_of(i, vec![carbon("x", i as f64)]))
        .collect();
    let peaks = year_summaries(&states, 3, |seg: &[State]| {
        seg.iter()
            .fold(f64::NEG_INFINITY, |acc, s| acc.max(s.stocks["x"].amount))
    });
    assert_eq!(peaks, vec![3.0, 6.0]);
}

#[test]
fn year_summaries_handles_short_trajectories() {
    // ⚠ The `usize` underflow guard, and the reason it exists. Python returns `[]` for
    // both because `(len - 1) // year` is `-1` and `range(-1)` is empty; the literal Rust
    // transcription of that expression underflows — a debug panic, and in release a
    // near-`usize::MAX` year count. Ported from `test_drift.py`'s own edge case.
    let none: [f64; 0] = [];
    assert!(year_summaries(&none, 3, |s: &[f64]| s.len() as f64).is_empty());
    assert!(year_summaries(&[1.0], 3, |s: &[f64]| s.len() as f64).is_empty());
}

#[test]
fn same_phase_diffs_matches_the_reference() {
    assert_eq!(
        same_phase_diffs(&[1.0, 2.0, 3.0, 4.0, 5.0], 2),
        vec![2.0, 2.0, 2.0]
    );
    assert!(same_phase_diffs(&[10.0, 20.0], 2).is_empty());
}

// --- axis (b): the stationarity / collapse split ------------------------------

#[test]
fn is_stationary_passes_settled_and_converging_cycles() {
    let settled = [1.0, 2.0, 1.0, 2.0, 1.0, 2.0]; // same-phase diffs all 0
    assert!(is_stationary(&same_phase_diffs(&settled, 2), 0.1, 0.01, 0));

    // The lock does NOT require a reached attractor: a still-converging cycle passes.
    let converging = [1.0, 3.0, 1.4, 2.6, 1.7, 2.3, 1.9, 2.1];
    assert!(is_stationary(
        &same_phase_diffs(&converging, 2),
        1.0,
        0.01,
        0
    ));
}

#[test]
fn is_stationary_fails_an_amplifying_cycle() {
    let amplifying = [1.0, 2.0, 0.8, 2.3, 0.5, 2.7, 0.1, 3.2];
    assert!(!is_stationary(
        &same_phase_diffs(&amplifying, 2),
        10.0,
        0.001,
        0
    ));
}

#[test]
fn decay_is_diff_blind_and_only_the_floor_catches_it() {
    // THE MANDATORY SPLIT, ported verbatim. A cycle creeping toward extinction has
    // SHRINKING same-phase diffs — mathematically indistinguishable from a cycle
    // converging to a finite attractor by the diffs alone, since the only difference is
    // the *limit* — so it PASSES `is_stationary`. Only the level check catches it. Both
    // are asserted so the test records WHICH detector owns extinction.
    let decaying = [2.0, 1.0, 0.5, 0.25, 0.125, 0.0625, 0.03, 0.015];
    assert!(is_stationary(&same_phase_diffs(&decaying, 2), 2.0, 0.0, 0));
    assert!(!non_collapsing(&decaying, 0.1));
}

#[test]
fn non_collapsing_floor() {
    assert!(non_collapsing(&[0.5, 0.6, 0.55, 0.6], 0.1));
    assert!(!non_collapsing(&[0.5, 0.2, 0.05, 0.6], 0.1)); // one dip below the floor
}

// --- axis (b-discrete): the period-2 structural check -------------------------

#[test]
fn is_period_2_structural() {
    assert!(is_period_2(&[1.0, 2.0, 1.0, 2.0, 1.0], 0, 1e-3));
    assert!(is_period_2(&[0.18, 0.26, 0.18, 0.25, 0.18], 0, 1e-3)); // the perennial
    assert!(!is_period_2(&[1.0, 2.0, 3.0, 4.0], 0, 1e-3)); // monotone → period-1
    assert!(!is_period_2(&[5.0, 5.0, 5.0], 0, 1e-3)); // flat → no phase
    assert!(!is_period_2(&[1.0, 2.0], 0, 1e-3)); // too short to establish alternation
}

#[test]
fn is_period_2_respects_the_transient() {
    assert!(is_period_2(&[0.0, 9.0, 1.0, 2.0, 1.0, 2.0, 1.0], 2, 1e-3));
}

#[test]
fn is_period_2_rejects_damped_oscillation_to_a_fixed_point() {
    // THE GUARD. A damped oscillation converging to a FIXED POINT rings — its adjacent
    // diffs alternate in sign through the transient — but the branch gap collapses to ~0.
    // A naive sign-alternation check false-passes it; the sustained-gap floor rejects it.
    // This is the real consumer chamber: the herbivore damps the producer cycle, so it is
    // period-1, and `drift_summary.json` records `is_period_2: false` for it.
    let damped = [1.0, 3.0, 1.6, 2.6, 1.9, 2.3, 1.99, 2.05, 2.0, 2.0, 2.0, 2.0];
    assert!(!is_period_2(&damped, 6, 1e-3));
    // ...while the SAME shape, sustaining its amplitude, IS period-2:
    let sustained = [1.0, 3.0, 1.6, 2.6, 1.9, 2.3, 1.0, 3.0, 1.0, 3.0, 1.0, 3.0];
    assert!(is_period_2(&sustained, 6, 1e-3));
}

// --- the transient boundary, gated on BOTH classifiers ------------------------
//
// ⚠ Neither port had a case for `transient >= len` before slice C5, and C4's gates call
// both of these with non-zero transients (`_TRANSIENT = 3`, `_PERIOD_TRANSIENT = 8`), so
// the boundary is on a live path. Python and Rust reach the same answers by different
// routes — Python slices to an empty tail and falls through, Rust returns early — which is
// exactly the shape that hides a divergence until something calls it.

#[test]
fn is_stationary_is_vacuously_true_past_the_transient() {
    // Python: `diffs[transient:]` is `[]` and `if not tail: return True`.
    assert!(is_stationary(&[1.0, 2.0], 0.0, 0.0, 2));
    assert!(is_stationary(&[1.0, 2.0], 0.0, 0.0, 5));
    assert!(is_stationary(&[], 0.0, 0.0, 0));
}

#[test]
fn is_period_2_is_false_past_the_transient() {
    // Python: the empty/short tail fails `len(tail) < 3` and returns False. ⚠ Note the
    // asymmetry with `is_stationary` above — "no data" means TRUE for the amplitude
    // detector and FALSE for the structural one, because one asserts an absence of drift
    // and the other asserts the presence of a cycle.
    assert!(!is_period_2(&[1.0, 2.0, 1.0], 3, 1e-3));
    assert!(!is_period_2(&[1.0, 2.0, 1.0], 9, 1e-3));
    assert!(!is_period_2(&[], 0, 1e-3));
}
