//! Run the standalone Crew `MISSION_SCENARIO` in the Rust port and emit its 7-day
//! final `State` as `sim_io`-shaped JSON (Phase-7 Step 3).
//!
//! Unlike the Step-0 `simcore/examples/emit_crew.rs` (which hand-built the golden's
//! own values to test the *interchange*), this **computes** crew_state from the ported
//! engine — the real cross-port validation. `tests/crossport/test_crossport.py` runs
//! this and compares the output to `crew_state.json` at **Tier 1 (bit-exact)**.
//!
//! Tier-0 invariants are asserted here in Rust: `rationed == 0`, `events == ()`, and —
//! implicitly — conservation every step (the run would have errored inside
//! `step_report` otherwise).

use domains::crew::{build_crew, crew_resolver, MISSION_DAYS, MISSION_SCENARIO};
use domains::{params, run};
use simcore::integrator::EulerIntegrator;

fn main() {
    let params = params::crew();
    let scenario = MISSION_SCENARIO;
    let (state, registry) = build_crew(&params, &scenario).expect("build_crew");
    let resolver = crew_resolver(&scenario).expect("crew_resolver");
    let integrator = EulerIntegrator::new(registry);
    let steps = MISSION_DAYS * scenario.steps_per_day;
    let (final_state, rationed, events) =
        run(&integrator, state, &resolver, scenario.dt_seconds, steps).expect("run crew");

    assert_eq!(rationed, 0, "Tier-0: crew rationed must be 0 (well-fed sizing)");
    assert!(events.is_empty(), "Tier-0: crew events must be empty (no POPULATION stock)");

    print!("{}", simcore::snapshot::from_engine(&final_state).to_json());
}
