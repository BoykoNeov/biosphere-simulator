//! Run the biomass/food `HARVEST_SCENARIO` (two-rate, Euler; `with_harvest=True`,
//! `close_feces=True` — the closed trophic ring) and emit its day-7 final `State` as
//! `sim_io`-shaped JSON (Phase-7 Step 5). Compared to `harvest_state.json` at **Tier 2** —
//! built on the FvCB greenhouse. `thermal_time0` starts the plant past anthesis (grain-filling).

use domains::params;
use simcore::integrator::EulerIntegrator;
use station::harvest::{build_harvest, harvest_bio_resolver, harvest_cabin_resolver, run_harvest};
use station::scenario::harvest_scenario;

fn main() {
    let crew = params::crew();
    let eclss = params::eclss();
    let harvest = station::params::harvest();
    let scenario = harvest_scenario();
    let (state, bio_reg, cabin_reg) =
        build_harvest(&crew, &eclss, &harvest, &scenario, true, true).expect("build_harvest");
    let bio_resolver = harvest_bio_resolver(&scenario).expect("bio_resolver");
    let cabin_resolver = harvest_cabin_resolver(&scenario).expect("cabin_resolver");
    let (states, rationed, events) = run_harvest(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(cabin_reg),
        state,
        &bio_resolver,
        &cabin_resolver,
        &scenario,
    )
    .expect("run harvest");

    assert_eq!(rationed, 0, "Tier-0: harvest rationed must be 0");
    assert!(events.is_empty(), "Tier-0: harvest events must be empty");

    let final_state = states.last().expect("at least one day boundary");
    print!("{}", simcore::snapshot::from_engine(final_state).to_json());
}
