//! Run the standalone Power `BOUNDED_SOC_SCENARIO` in the Rust port and emit its 7-day
//! final `State` as `sim_io`-shaped JSON (Phase-7 Step 3). Compared to `power_state.json`
//! at **Tier 2 (measured band)** — the half-sine `solar_schedule` (`sin`) is the
//! transcendental. The derived `balanced_load_w` is *re-computed* here (ported, not
//! smuggled from the golden).

use domains::power::{build_power, power_resolver, BOUNDED_SOC_DAYS, BOUNDED_SOC_SCENARIO};
use domains::{params, run};
use simcore::integrator::EulerIntegrator;

fn main() {
    let charge = params::charge();
    let scenario = BOUNDED_SOC_SCENARIO;
    let (state, registry) = build_power(&charge, &scenario, None).expect("build_power");
    let resolver = power_resolver(&charge, &scenario).expect("power_resolver");
    let integrator = EulerIntegrator::new(registry);
    let steps = BOUNDED_SOC_DAYS * scenario.steps_per_day;
    let (final_state, rationed, events) =
        run(&integrator, state, &resolver, scenario.dt_seconds, steps).expect("run power");

    assert_eq!(rationed, 0, "Tier-0: power rationed must be 0 (well-fed sizing)");
    assert!(events.is_empty(), "Tier-0: power events must be empty");

    print!("{}", simcore::snapshot::from_engine(&final_state).to_json());
}
