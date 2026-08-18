//! Run the minimal-consumer sealed chamber and emit its final `State` (Phase-7 P7.4).
//! `CONSUMER_CHAMBER_SCENARIO` via `run_perennial`; the horizon (`$1`) selects the 5-yr
//! (`consumer_chamber_state`) or 15-yr (`consumer_long_horizon_state`) golden. Both
//! **Tier 2** — the FvCB biosphere + the herbivory sub-loop.

use domains::biosphere::{CONSUMER_CHAMBER_YEARS, LONG_HORIZON_YEARS};

fn main() {
    print!("{}", domains::goldens::consumer_chamber(horizon()));
}

/// Horizon in years from `$1` ("long" => 15, else 5).
fn horizon() -> usize {
    match std::env::args().nth(1).as_deref() {
        Some("long") => LONG_HORIZON_YEARS,
        _ => CONSUMER_CHAMBER_YEARS,
    }
}
