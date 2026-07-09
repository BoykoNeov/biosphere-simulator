//! Run the standalone Power `SELF_DISCHARGE` build (the two forced flows + the opt-in
//! donor-controlled `SelfDischarge`) over 14 days and emit its final `State` (Phase-7
//! Step 3). Compared to `power_self_discharge_state.json` at **Tier 2 (measured band)**
//! — it reuses `BOUNDED_SOC_SCENARIO`'s half-sine solar (inherits `sin`); the leak leg
//! itself is linear.

use domains::power::{
    build_power, power_resolver, BOUNDED_SOC_SCENARIO, SELF_DISCHARGE_DAYS,
};
use domains::{params, run};
use simcore::integrator::EulerIntegrator;

fn main() {
    let charge = params::charge();
    let self_discharge = params::self_discharge();
    let scenario = BOUNDED_SOC_SCENARIO;
    let (state, registry) =
        build_power(&charge, &scenario, Some(self_discharge)).expect("build_power");
    let resolver = power_resolver(&charge, &scenario).expect("power_resolver");
    let integrator = EulerIntegrator::new(registry);
    let steps = SELF_DISCHARGE_DAYS * scenario.steps_per_day;
    let (final_state, rationed, events) =
        run(&integrator, state, &resolver, scenario.dt_seconds, steps).expect("run power");

    assert_eq!(rationed, 0, "Tier-0: power self-discharge rationed must be 0");
    assert!(events.is_empty(), "Tier-0: power self-discharge events must be empty");

    print!("{}", simcore::snapshot::from_engine(&final_state).to_json());
}
