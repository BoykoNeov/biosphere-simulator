//! Run the standalone ECLSS `STEADY_STATE_SCENARIO` in the Rust port and emit its final
//! `State` as `sim_io`-shaped JSON (Phase-7 Step 3). Compared to `eclss_state.json` at
//! **Tier 1 (bit-exact)** — ECLSS is transcendental-free (linear control loops), so the
//! op-order of each `(k · stock) · dt` / `(setpoint − cabin_o2)` is load-bearing.

use domains::eclss::{build_eclss, eclss_resolver, STEADY_STATE_SCENARIO, STEADY_STATE_STEPS};
use domains::{params, run};
use simcore::integrator::EulerIntegrator;

fn main() {
    let params = params::eclss();
    let scenario = STEADY_STATE_SCENARIO;
    let (state, registry) = build_eclss(&params, &scenario).expect("build_eclss");
    let resolver = eclss_resolver(&scenario).expect("eclss_resolver");
    let integrator = EulerIntegrator::new(registry);
    let (final_state, rationed, events) = run(
        &integrator,
        state,
        &resolver,
        scenario.dt_seconds,
        STEADY_STATE_STEPS,
    )
    .expect("run eclss");

    assert_eq!(rationed, 0, "Tier-0: eclss rationed must be 0");
    assert!(events.is_empty(), "Tier-0: eclss events must be empty");

    print!("{}", simcore::snapshot::from_engine(&final_state).to_json());
}
