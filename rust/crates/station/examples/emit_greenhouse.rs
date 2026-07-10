//! Run the biosphere ↔ cabin `GREENHOUSE_SCENARIO` (two-rate, Euler) and emit its day-7
//! final `State` as `sim_io`-shaped JSON (Phase-7 Step 5). Compared to `greenhouse_state.json`
//! at **Tier 2** — the FvCB biosphere runs every master day. The per-sub-step conservation
//! assert inside the two-rate driver is the Tier-0 gate (a completed run is the proof).

use domains::crew::FECAL_WASTE;
use domains::params;
use simcore::integrator::EulerIntegrator;
use station::greenhouse::{
    build_greenhouse, greenhouse_bio_resolver, greenhouse_cabin_resolver, run_greenhouse,
};
use station::scenario::greenhouse_scenario;

fn main() {
    let crew = params::crew();
    let eclss = params::eclss();
    let scenario = greenhouse_scenario();
    let (state, bio_reg, cabin_reg) =
        build_greenhouse(&crew, &eclss, &scenario, true, FECAL_WASTE).expect("build_greenhouse");
    let bio_resolver = greenhouse_bio_resolver(&scenario).expect("bio_resolver");
    let cabin_resolver = greenhouse_cabin_resolver(&scenario).expect("cabin_resolver");
    let (states, rationed, events) = run_greenhouse(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(cabin_reg),
        state,
        &bio_resolver,
        &cabin_resolver,
        &scenario,
    )
    .expect("run greenhouse");

    assert_eq!(rationed, 0, "Tier-0: greenhouse rationed must be 0");
    assert!(events.is_empty(), "Tier-0: greenhouse events must be empty");

    let final_state = states.last().expect("at least one day boundary");
    print!("{}", simcore::snapshot::from_engine(final_state).to_json());
}
