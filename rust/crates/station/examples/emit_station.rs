//! Run the coupled Power → Thermal `HEAT_CLOSURE_SCENARIO` (single-rate, Euler) and emit
//! its 7-day final `State` as `sim_io`-shaped JSON (Phase-7 Step 5). Compared to
//! `station_state.json` at **Tier 2 (measured band)** — Power's half-sine (`sin`) dissipates
//! into Thermal's `T⁴` radiator, both transcendentals in one graph. The node starts at the
//! dissipation-set equilibrium (`node0 = None ⇒ equilibrium_node_heat`).

use domains::params;
use simcore::integrator::EulerIntegrator;
use station::run_station;
use station::scenario::{HEAT_CLOSURE_DAYS, HEAT_CLOSURE_SCENARIO};
use station::system::{build_station, station_resolver};

fn main() {
    let charge = params::charge();
    let thermal = params::thermal();
    let scenario = HEAT_CLOSURE_SCENARIO;
    let (state, registry) =
        build_station(&charge, &thermal, &scenario, None).expect("build_station");
    let resolver = station_resolver(&charge, &scenario).expect("station_resolver");
    let integrator = EulerIntegrator::new(registry);
    let steps = HEAT_CLOSURE_DAYS * scenario.power.steps_per_day;
    let mut noop = |_: &simcore::state::State| {};
    let (final_state, rationed, events) = run_station(
        &integrator,
        state,
        &resolver,
        scenario.power.dt_seconds,
        steps,
        &mut noop,
    )
    .expect("run station");

    assert_eq!(
        rationed, 0,
        "Tier-0: station rationed must be 0 (well-fed sizing)"
    );
    assert!(
        events.is_empty(),
        "Tier-0: station events must be empty (no POPULATION stock)"
    );

    print!("{}", simcore::snapshot::from_engine(&final_state).to_json());
}
