//! Run the crew water-recovery `WATER_RECOVERY_SCENARIO` and emit its final `State` as
//! `sim_io`-shaped JSON (Phase-7 Step 5). Compared to `water_recovery_state.json` at
//! **Tier 1 (bit-exact)** — the donor-controlled `WaterRecovery` is still only `*`/`+`/`-`/
//! `/` atop the transcendental-free cabin (no biosphere).

use domains::params;
use simcore::integrator::EulerIntegrator;
use station::run_station;
use station::scenario::{WATER_RECOVERY_SCENARIO, WATER_RECOVERY_STEPS};
use station::water::{build_water_recovery, water_recovery_resolver};

fn main() {
    let crew = params::crew();
    let eclss = params::eclss();
    let recovery = station::params::water_recovery();
    let scenario = WATER_RECOVERY_SCENARIO;
    let (state, registry) =
        build_water_recovery(&crew, &eclss, &recovery, &scenario).expect("build_water_recovery");
    let resolver = water_recovery_resolver(&scenario).expect("water_recovery_resolver");
    let integrator = EulerIntegrator::new(registry);
    let mut noop = |_: &simcore::state::State| {};
    let (final_state, rationed, events) = run_station(
        &integrator,
        state,
        &resolver,
        scenario.dt_seconds,
        WATER_RECOVERY_STEPS,
        &mut noop,
    )
    .expect("run water_recovery");

    assert_eq!(rationed, 0, "Tier-0: water-recovery rationed must be 0");
    assert!(
        events.is_empty(),
        "Tier-0: water-recovery events must be empty"
    );

    print!("{}", simcore::snapshot::from_engine(&final_state).to_json());
}
