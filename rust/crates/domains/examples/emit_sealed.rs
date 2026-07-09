//! Run the O₂-poor sealed chamber (`SEALED_CHAMBER_SCENARIO`, 3 yr, `run_season`) in the
//! Rust port and emit its final `State` (Phase-7 P7.4). Compared to
//! `sealed_chamber_state.json` at **Tier 2** — the closed biosphere (FvCB + the
//! decomposer gas loop + f_O2 self-limitation).

use domains::biosphere::{run_season, sealed_chamber_scenario, season_setup, steps_for, SEALED_CHAMBER_YEARS};
use simcore::state::State;

fn main() {
    let wy = SEALED_CHAMBER_YEARS;
    let scenario = sealed_chamber_scenario();
    let (state, integrator, resolver) = season_setup(&scenario, wy).expect("season_setup");
    let steps = steps_for(wy);
    let mut noop = |_: &State| {};
    let (final_state, rationed, events) =
        run_season(&integrator, state, &resolver, 1.0, steps, None, &mut noop).expect("run_season");

    assert_eq!(rationed, 0, "Tier-0: sealed rationed must be 0 (f_O2 self-limits)");
    assert!(events.is_empty(), "Tier-0: sealed events must be empty");

    print!("{}", simcore::snapshot::from_engine(&final_state).to_json());
}
