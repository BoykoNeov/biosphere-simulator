//! Emit `sealed_energy_drift_summary.json` — the sealed station's 15-yr energy **stability
//! signature**, folded in Rust as of slice C5 of the reference flip.
//!
//! ⚠ **This program's output changed KIND in C5.** Through Phase-7 Step 5 it streamed the
//! raw per-step `thermal.node` heat series and the *Python* gate folded it (`temp =
//! space_temp + node/C`, per-year peaks, the `is_stationary` classifier) into the golden —
//! the "one run, two authors" split, since the fold is what decides what the summary
//! *says*. `domains::biosphere::drift` now carries that kit, so this example emits the
//! artifact itself and the golden is Rust's end to end.
//!
//! The run is unchanged: the 15-yr single-rate Power → Thermal `HEAT_CLOSURE_SCENARIO`
//! (diurnal solar ⇒ `n` advances ⇒ the SB radiator's real `T_eq` attractor).
//!
//! ⚠ The fold lives in `domains::biosphere::drift` and is used here by the *station* —
//! deliberately, and mirroring Python, where `tests/test_regression_sealed_station.py`
//! imports `year_summaries` / `same_phase_diffs` / `is_stationary` from
//! `domains.biosphere.drift`. The module is an instrument, generic over `State`; the
//! caller supplies the per-year `summary_fn`, so nothing biosphere-specific comes with it.

fn main() {
    print!("{}", station::goldens::sealed_energy_drift());
}
