//! Emit `drift_summary.json` — the 15-yr biosphere **stability signature** of both chamber
//! scenarios, folded in Rust as of the drift-summary regeneration ceremony (2026-09-06).
//!
//! ⚠ **This program's output changed KIND**, exactly as `emit_sealed_energy_drift` did in
//! C5. It used to stream the raw per-step `leaf_c` / `consumer_carbon` trajectories for a
//! Python parity gate to segment and classify — the "one run, two authors" split, since
//! the fold is what decides what the summary *says*. It now emits the artifact itself, so
//! the golden is the reference's own bytes end to end.
//!
//! ⚠ The header here used to say the conversion was blocked and must not be finished. That
//! was true and is no longer: the blocker was a Python gate
//! (`test_every_diverging_scenario_keeps_a_byte_gated_sibling`) that S6 deleted along with
//! the rest of the checker, and what it left behind was a frozen golden no tool could
//! regenerate. `domains::goldens::drift_summary` carries the fold and its whole record.

fn main() {
    print!("{}", domains::goldens::drift_summary());
}
