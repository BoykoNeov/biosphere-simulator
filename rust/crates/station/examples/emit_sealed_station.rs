//! Run the fully-coupled sealed station `SEALED_STATION_SCENARIO` (two-rate, Euler,
//! `with_harvest=False` / `close_feces=False` — the Tier-2 scope) over the multi-year
//! horizon and emit its day-boundary final `State` as `sim_io`-shaped JSON (Phase-7 Step 5).
//! Compared to `sealed_station_state.json` at **Tier 2**. The ~1.3 M-sub-step run's real
//! payload is the per-sub-step conservation assert inside the driver (the Tier-0 gate): a
//! completed run is itself proof the combined ledger balanced every sub-step over the full
//! five-domain assembly.

use domains::params;
use simcore::integrator::EulerIntegrator;
use station::params as station_params;
use station::scenario::sealed_station_scenario;
use station::sealed::{
    build_sealed_station, run_sealed, sealed_bio_resolver, sealed_fast_resolver,
};

fn main() {
    let charge = params::charge();
    let thermal = params::thermal();
    let crew = params::crew();
    let eclss = params::eclss();
    let recovery = station_params::water_recovery();
    let lamp = station_params::lamp();
    let harvest = station_params::harvest();
    let scenario = sealed_station_scenario();

    let (state, bio_reg, fast_reg) = build_sealed_station(
        &charge, &thermal, &crew, &eclss, &recovery, &lamp, &harvest, &scenario, false, false,
    )
    .expect("build_sealed_station");
    let bio_resolver = sealed_bio_resolver(&lamp, &scenario).expect("sealed_bio_resolver");
    let fast_resolver = sealed_fast_resolver(&charge, &scenario).expect("sealed_fast_resolver");

    let (states, rationed, events) = run_sealed(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(fast_reg),
        state,
        &bio_resolver,
        &fast_resolver,
        &scenario,
    )
    .expect("run sealed station");

    assert_eq!(rationed, 0, "Tier-0: sealed station rationed must be 0");
    assert!(
        events.is_empty(),
        "Tier-0: sealed station events must be empty"
    );

    let final_state = states.last().expect("at least one day boundary");
    print!("{}", simcore::snapshot::from_engine(final_state).to_json());
}
