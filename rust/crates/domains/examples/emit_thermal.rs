//! Run the standalone Thermal `EQUILIBRIUM_SCENARIO` in the Rust port and emit its final
//! `State` (Phase-7 Step 3). Compared to `thermal_state.json` at **Tier 2 (measured
//! band)** — the Stefan-Boltzmann `RadiatorReject` computes `(T⁴ − T_space⁴)` via
//! `powf(4.0)` every step (the nonlinear attractor, the plan's first real libm audit).

use domains::thermal::{build_thermal, thermal_resolver, EQUILIBRIUM_SCENARIO, EQUILIBRIUM_STEPS};
use domains::{params, run};
use simcore::integrator::EulerIntegrator;

fn main() {
    let params = params::thermal();
    let scenario = EQUILIBRIUM_SCENARIO;
    let (state, registry) = build_thermal(&params, &scenario).expect("build_thermal");
    let resolver = thermal_resolver(&scenario).expect("thermal_resolver");
    let integrator = EulerIntegrator::new(registry);
    let (final_state, rationed, events) = run(
        &integrator,
        state,
        &resolver,
        scenario.dt_seconds,
        EQUILIBRIUM_STEPS,
    )
    .expect("run thermal");

    assert_eq!(rationed, 0, "Tier-0: thermal rationed must be 0 (τ >> dt sizing)");
    assert!(events.is_empty(), "Tier-0: thermal events must be empty");

    print!("{}", simcore::snapshot::from_engine(&final_state).to_json());
}
