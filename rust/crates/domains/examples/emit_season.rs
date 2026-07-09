//! Run the open-field `DEFAULT_SCENARIO` season in the Rust port and emit its final
//! `State` (Phase-7 P7.4). Compared to `season_euler_state.json` at **Tier 2** — the
//! FvCB / Penman–Monteith / weather transcendental surface. Euler-daily, 1 season.

use domains::biosphere::{run_season, season_setup, steps_for, DEFAULT_SCENARIO};
use simcore::state::State;

fn main() {
    let weather_years = 1;
    let (state, integrator, resolver) =
        season_setup(&DEFAULT_SCENARIO, weather_years).expect("season_setup");
    let steps = steps_for(weather_years);
    let mut noop = |_: &State| {};
    let (final_state, rationed, events) =
        run_season(&integrator, state, &resolver, 1.0, steps, None, &mut noop).expect("run_season");

    assert_eq!(rationed, 0, "Tier-0: open season rationed must be 0");
    assert!(events.is_empty(), "Tier-0: open season events must be empty");

    print!("{}", simcore::snapshot::from_engine(&final_state).to_json());
}
