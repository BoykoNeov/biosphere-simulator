//! Run the minimal-consumer sealed chamber and emit its final `State` (Phase-7 P7.4).
//! `CONSUMER_CHAMBER_SCENARIO` via `run_perennial`; the horizon (`$1`) selects the 5-yr
//! (`consumer_chamber_state`) or 15-yr (`consumer_long_horizon_state`) golden. Both
//! **Tier 2** — the FvCB biosphere + the herbivory sub-loop.

use domains::biosphere::{consumer_chamber_scenario, run_perennial_final, CONSUMER_CHAMBER_YEARS, LONG_HORIZON_YEARS};

fn main() {
    let years = match std::env::args().nth(1).as_deref() {
        Some("long") => LONG_HORIZON_YEARS,
        _ => CONSUMER_CHAMBER_YEARS,
    };
    let scenario = consumer_chamber_scenario();
    let (final_state, rationed, events) =
        run_perennial_final(&scenario, years).expect("run_perennial");
    assert_eq!(rationed, 0, "Tier-0: consumer rationed must be 0");
    assert!(events.is_empty(), "Tier-0: consumer events must be empty");
    print!("{}", simcore::snapshot::from_engine(&final_state).to_json());
}
