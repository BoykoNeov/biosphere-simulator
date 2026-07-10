//! Run the Power → biosphere `LIGHTING_SCENARIO` (two-rate, Euler) and emit its day-7 final
//! `State` as `sim_io`-shaped JSON (Phase-7 Step 5). Compared to `lighting_state.json` at
//! **Tier 2** — the lamp forces the FvCB biosphere's PAR; the biosphere runs every master day.

use simcore::integrator::EulerIntegrator;
use station::lighting::{
    build_lighting, lighting_bio_resolver, lighting_power_resolver, run_lighting,
};
use station::scenario::lighting_scenario;

fn main() {
    let lamp = station::params::lamp();
    let scenario = lighting_scenario();
    let (state, bio_reg, power_reg) =
        build_lighting(&lamp, &scenario, true).expect("build_lighting");
    let bio_resolver = lighting_bio_resolver(&lamp, &scenario, true).expect("bio_resolver");
    let power_resolver = lighting_power_resolver(&scenario).expect("power_resolver");
    let (states, rationed, events) = run_lighting(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(power_reg),
        state,
        &bio_resolver,
        &power_resolver,
        &scenario,
    )
    .expect("run lighting");

    assert_eq!(rationed, 0, "Tier-0: lighting rationed must be 0");
    assert!(events.is_empty(), "Tier-0: lighting events must be empty");

    let final_state = states.last().expect("at least one day boundary");
    print!("{}", simcore::snapshot::from_engine(final_state).to_json());
}
