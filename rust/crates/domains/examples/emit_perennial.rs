//! Run the perennial (annual-reset) sealed chamber and emit its final `State`
//! (Phase-7 P7.4). `PERENNIAL_CHAMBER_SCENARIO` via `run_perennial`; the horizon is set
//! by an env arg so this one example serves both the 5-yr (`perennial_chamber_state`) and
//! the 15-yr (`perennial_long_horizon_state`) goldens. Both **Tier 2**.

use domains::biosphere::{LONG_HORIZON_YEARS, PERENNIAL_CHAMBER_YEARS};

fn main() {
    print!("{}", domains::goldens::perennial_chamber(horizon()));
}

/// Horizon in years from `$1` ("long" => 15, else 5).
fn horizon() -> usize {
    match std::env::args().nth(1).as_deref() {
        Some("long") => LONG_HORIZON_YEARS,
        _ => PERENNIAL_CHAMBER_YEARS,
    }
}
