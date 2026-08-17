//! Drift instrumentation — decade-scale stability metrics (slice C5 of the reference flip).
//!
//! The Rust half of what `domains/biosphere/drift.py` has done since Phase-4 Step-1: a
//! **measurement instrument** over a closed-biosphere trajectory (`&[State]`), used to
//! decide *operationally* whether the emergent limit cycle is conservation-stable over
//! decade-scale runs. "No drift" is the subtle one — the cycle is *meant* to oscillate, so
//! drift is defined along three axes:
//!
//! * **(a) Mass-conservation drift** — does `total_q` walk away from its step-0 value?
//!   [`mass_drift_trace`] / [`drift_slope`] / [`max_abs`], against the derived detector
//!   bounds [`MASS_DRIFT_ABS_BOUND`] / [`MASS_DRIFT_SLOPE_BOUND`].
//! * **(b) Limit-cycle stationarity** — per-year scalar summaries ([`year_summaries`])
//!   reach an attractor and hold it. For a period-2 cycle the *adjacent* difference is the
//!   cycle amplitude and never vanishes, so stationarity is read on **same-phase**
//!   differences ([`same_phase_diffs`]). The split is mandatory, not stylistic:
//!   [`is_stationary`] is bounded + non-amplifying and is **blind to creeping decay**
//!   (geometric decay shrinks `|d[k]|` identically to a converging cycle — the difference
//!   is the *limit*, a property of the summary level), so [`non_collapsing`] is its
//!   required companion.
//! * **(b-discrete)** — [`is_period_2`], kept apart from the scalar vector because a phase
//!   index is not a scalar you can call "non-increasing".
//!
//! ## Why this is in Rust as of slice C5
//!
//! Until C5 these folds were Python's, and the two drift goldens were the reference flip's
//! last "one run, two authors" split: Rust streamed the raw per-step series
//! (`examples/emit_drift.rs`, `station/examples/emit_sealed_energy_drift.rs`) and Python
//! folded them into the artifact. The fold is what decides what the summary *says*, so the
//! goldens were Python-authored no matter which port ran the trajectory. They are now
//! Rust's end to end. C4 (the science-gate census) needs this kit for 5 of its 15 gates.
//!
//! ## Generic over `State`, exactly as the Python is
//!
//! [`year_summaries`] takes the per-year `summary_fn` from the caller, so this module
//! imports no stock-id catalog and the *station* can use it on a thermal-node trajectory
//! (`station` already depends on `domains`) — the same layering Python has.
//!
//! ⚠ **The segmentation is inclusive of the next year's boundary state** and the period is
//! in whatever unit the caller's trajectory is indexed by — **steps** for the biosphere
//! (`steps_for(305)`), **days** for the station's master-day runs. See [`year_summaries`].

use simcore::quantities::Quantity;
use simcore::state::State;

// --- the promoted fold -------------------------------------------------------

/// Folded total of `quantity` over ALL stocks (boundaries / leak-sink included).
///
/// The absent-quantity default of `0.0` is load-bearing: it folds correctly over a
/// heterogeneous `State` (water / N / O2 pools, boundary + loss + leak sinks all
/// contribute `0.0` for a quantity they do not carry), so a *vented* leak still conserves
/// the total because the boundary sink carries the vented mass.
///
/// ⚠ Iterates `state.stocks` in `BTreeMap` order — stock-id order, the same canonical
/// order Python's `dict` preserves from insertion. The summation order is part of the
/// result at ULP scale, which is why this is a port and not a re-derivation.
pub fn total_quantity(state: &State, quantity: Quantity) -> f64 {
    state
        .stocks
        .values()
        .map(|st| st.amount * st.composition.get(&quantity).copied().unwrap_or(0.0))
        .sum()
}

// --- shared primitive --------------------------------------------------------

/// Ordinary-least-squares slope of `values` against index `0..len-1`.
///
/// The systematic-trend signature: a leak is linear in `n` (nonzero slope); bounded
/// round-off jitter is not (slope ~ machine-eps noise). Returns `0.0` for fewer than two
/// points or a degenerate (zero-variance) abscissa.
pub fn least_squares_slope(values: &[f64]) -> f64 {
    let n = values.len();
    if n < 2 {
        return 0.0;
    }
    let mean_x = (n - 1) as f64 / 2.0;
    let mean_y: f64 = values.iter().sum::<f64>() / n as f64;
    let numerator: f64 = values
        .iter()
        .enumerate()
        .map(|(i, v)| (i as f64 - mean_x) * (v - mean_y))
        .sum();
    let denominator: f64 = (0..n).map(|i| (i as f64 - mean_x).powi(2)).sum();
    if denominator != 0.0 {
        numerator / denominator
    } else {
        0.0
    }
}

// --- axis (a): mass-conservation drift ---------------------------------------

/// `[total_q(n) - total_q(0)]` over the trajectory — the accumulation trace.
///
/// For a structurally-balanced flow set `total_q` is conserved up to float round-off, so
/// this trace is expected to be bounded-oscillating / sqrt(N) noise, NOT linearly
/// trending; a systematic leak shows up as a nonzero [`drift_slope`].
pub fn mass_drift_trace(states: &[State], quantity: Quantity) -> Vec<f64> {
    let base = total_quantity(&states[0], quantity);
    states
        .iter()
        .map(|s| total_quantity(s, quantity) - base)
        .collect()
}

/// Least-squares slope of a mass-drift trace vs step index (the leak signature).
pub fn drift_slope(trace: &[f64]) -> f64 {
    least_squares_slope(trace)
}

/// `max|trace|` — the interpretable conservation bound (vs the structural ceiling).
pub fn max_abs(trace: &[f64]) -> f64 {
    trace.iter().fold(0.0_f64, |acc, x| acc.max(x.abs()))
}

// Detector bounds for axis (a) — DERIVED from the Step-1 probe measurement, NOT
// hand-tuned. A regression-guard threshold derived from round-off is part of the *test*,
// not a model coefficient, so it lives here as a documented constant with provenance —
// not in a param YAML.
//
// PROVENANCE (re-derivable — carried over from `drift.py` verbatim, because a bound
// nobody can reproduce is exactly what this block exists to prevent):
//   Procedure : run PERENNIAL_CHAMBER_SCENARIO and CONSUMER_CHAMBER_SCENARIO under BOTH
//               Euler and RK4, dt = 1.0, to a 15-year horizon (steps = 15 * 305 = 4575),
//               and measure per quantity q in {CARBON, OXYGEN, NITROGEN, WATER}:
//                 max|d_q| = max_abs(mass_drift_trace(states, q))
//                 slope_q  = drift_slope(mass_drift_trace(states, q))
//               ⚠ dt = 1.0 and the 4575 are the values these bounds were DERIVED at and
//               are left as the historical record of that derivation. The step moved to
//               BIO_DT = 1/4 on 2026-08-14 (the same horizon is 18300 steps) and the
//               bounds were NOT re-derived — they were re-run and still hold with orders
//               to spare. A re-derivation would tighten them; saying so is cheaper than a
//               bound nobody can reproduce.
//   Observed  : worst case over all (scenario x integrator x q) at 15 yr —
//                 max|d_q|  ~ 3.3e-12  (WATER; CARBON ~1e-14, the tightest)
//                 |slope_q| ~ 7.3e-16  (WATER) — machine-eps noise, no trend
//               vs the structural ceiling N * BALANCE_ATOL ~ 4.6e-6 (~6-9 orders looser).
//   Re-confirmed at 328 yr (100,040 steps, Euler, both closed scenarios):
//                 max|d_q|  ~ 7.4e-11  (WATER)   |slope_q| ~ 7.5e-16 (flat)
//               The SLOPE stayed flat over a 22x-longer run while max|d_q| grew at that
//               machine-eps slope — deterministic round-off in this fold's summation
//               order, NOT a leak.
/// ABS bound: ~300x the 15-yr round-off floor (3.3e-12), ~13x the 328-yr floor (7.4e-11).
pub const MASS_DRIFT_ABS_BOUND: f64 = 1e-9;
/// SLOPE bound: ~4 orders above the measured round-off slope, ~2 orders below a
/// 1e-9/step leak's slope.
pub const MASS_DRIFT_SLOPE_BOUND: f64 = 1e-11;

// --- axis (b): limit-cycle stationarity --------------------------------------

/// One scalar per year via `summary_fn` over each year's segment.
///
/// ⚠ Two things here are load-bearing and both are off-by-one shaped:
///
/// * the segment is `states[y*year ..= (y+1)*year]` — **inclusive of the next year's
///   boundary state**, exactly as the perennial / ledger slicing does;
/// * `n_years = (len - 1) / year`, because a trajectory of `steps` steps carries
///   `steps + 1` states (the initial one included).
///
/// ⚠ `year` is a period in **the unit the trajectory is indexed by**, which is not always
/// days: the biosphere's `run_season` trajectory is per-STEP (`steps_for(305)` = 1220 at
/// `dt = 1/4`), while the station's `run_master_day` trajectory is per-DAY. Passing days
/// where steps are meant is the trap `post-roadmap-step-unfreeze.md` §1 records.
/// ⚠ Generic over the element type, not fixed to `State`, and that is the faithful port:
/// Python's is duck-typed over any sequence and only *happens* to be handed a
/// `list[State]`. The station folds a pre-reduced per-step temperature series with it
/// (`examples/emit_sealed_energy_drift.rs`) rather than materializing 109,801 `State`s to
/// satisfy a signature.
pub fn year_summaries<T, F>(items: &[T], year: usize, summary_fn: F) -> Vec<f64>
where
    F: Fn(&[T]) -> f64,
{
    // ⚠ `saturating_sub`, not `- 1`. Python's `(len(states) - 1) // year` on an EMPTY
    // trajectory is `-1 // year == -1`, and `range(-1)` is empty, so it returns `[]`. The
    // literal Rust transcription underflows `usize` — a debug panic and, in release, a
    // near-`usize::MAX` year count. Found by porting `test_drift.py`'s own
    // `test_year_summaries_handles_short_trajectories` rather than by reading the code.
    let n_years = items.len().saturating_sub(1) / year;
    (0..n_years)
        .map(|y| summary_fn(&items[y * year..=(y + 1) * year]))
        .collect()
}

/// `[summary(k) - summary(k-period)]` — the same-branch difference of the cycle.
///
/// For a period-2 cycle the *adjacent* difference is the cycle amplitude (it does not
/// vanish); the same-phase difference *does* vanish once the branch settles, so it is the
/// right stationarity signal.
pub fn same_phase_diffs(summaries: &[f64], period: usize) -> Vec<f64> {
    (period..summaries.len())
        .map(|k| summaries[k] - summaries[k - period])
        .collect()
}

/// Bounded + non-amplifying past `transient` — the amplitude-drift detector.
///
/// `bounded` = `max|diff| <= bound` (against the summary scale). `non_amplifying` =
/// least-squares slope of `|diff|` `<= slope_tol` — a *trend* test, not strict pairwise
/// monotonicity, which with ~4-8 diffs and float noise would be flaky.
///
/// Catches *amplifying* drift and passes a settled or still-converging cycle. It is
/// **blind** to creeping decay toward extinction — see [`non_collapsing`], which is a
/// required companion and not a stylistic one.
pub fn is_stationary(diffs: &[f64], bound: f64, slope_tol: f64, transient: usize) -> bool {
    if transient >= diffs.len() {
        return true;
    }
    let tail = &diffs[transient..];
    let abs_tail: Vec<f64> = tail.iter().map(|d| d.abs()).collect();
    let bounded = abs_tail.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)) <= bound;
    let non_amplifying = least_squares_slope(&abs_tail) <= slope_tol;
    bounded && non_amplifying
}

/// The extinction detector: every per-year summary stays at or above `floor`.
///
/// Mandatory companion to [`is_stationary`]: a cycle decaying toward extinction has
/// shrinking same-phase diffs (diff-blind), so only a *level* check on the summaries
/// themselves catches it.
pub fn non_collapsing(summaries: &[f64], floor: f64) -> bool {
    summaries.iter().all(|&s| s >= floor)
}

// --- axis (b-discrete): the period-2 structural check ------------------------

/// Discrete structural check: a *sustained* period-2 cycle past `transient`.
///
/// A genuine period-2 limit cycle has the odd/even years on **opposite branches**, so
/// every adjacent difference is (1) a real jump and (2) alternating in sign. Both must
/// hold across the whole post-`transient` tail:
///
/// * **alternation** — the structural period-2 signature;
/// * **a sustained branch gap** — EVERY adjacent `|diff|` exceeds `min_rel_gap * scale`
///   (`scale` = `max|summary|` over the tail). This is the load-bearing guard: a *damped*
///   oscillation converging to a **fixed point** rings — its adjacent diffs alternate
///   during the transient — but the gap collapses to ~0. Without the gap floor that
///   ringing is misread as period-2 (the consumer chamber is period-1: the herbivore damps
///   the producer cycle). So pick a `transient` that reaches the settled tail.
pub fn is_period_2(summaries: &[f64], transient: usize, min_rel_gap: f64) -> bool {
    if transient >= summaries.len() {
        return false;
    }
    let tail = &summaries[transient..];
    if tail.len() < 3 {
        return false;
    }
    let adjacent: Vec<f64> = (0..tail.len() - 1).map(|k| tail[k + 1] - tail[k]).collect();
    let scale = tail.iter().fold(0.0_f64, |acc, s| acc.max(s.abs()));
    let floor = min_rel_gap * scale;
    // A collapsed adjacent gap means the branches merged → a fixed point, not a cycle.
    if adjacent.iter().any(|d| d.abs() <= floor) {
        return false;
    }
    let rises: Vec<bool> = adjacent.iter().map(|d| *d > 0.0).collect();
    (0..rises.len().saturating_sub(1)).all(|i| rises[i] != rises[i + 1])
}
