//! Run the coupled crew ↔ ECLSS `CABIN_GAS_SCENARIO` and emit its final `State` as
//! `sim_io`-shaped JSON (Phase-7 Step 5). Compared to `cabin_gas_state.json` at **Tier 1
//! (bit-exact)** — the cabin loop is transcendental-free (forced/linear crew respiration +
//! first-order ECLSS controls; no biosphere, no `sin`/`pow`), the strongest cross-port gate.

use domains::params;
use simcore::integrator::EulerIntegrator;
use station::cabin::{build_cabin, cabin_resolver};
use station::run_station;
use station::scenario::{CABIN_GAS_SCENARIO, CABIN_GAS_STEPS};

fn main() {
    let crew = params::crew();
    let eclss = params::eclss();
    let scenario = CABIN_GAS_SCENARIO;
    let (state, registry) = build_cabin(&crew, &eclss, &scenario).expect("build_cabin");
    let resolver = cabin_resolver(&scenario).expect("cabin_resolver");
    let integrator = EulerIntegrator::new(registry);
    let mut noop = |_: &simcore::state::State| {};
    let (final_state, rationed, events) = run_station(
        &integrator,
        state,
        &resolver,
        scenario.dt_seconds,
        CABIN_GAS_STEPS,
        &mut noop,
    )
    .expect("run cabin");

    assert_eq!(
        rationed, 0,
        "Tier-0: cabin rationed must be 0 (well-fed sizing)"
    );
    assert!(events.is_empty(), "Tier-0: cabin events must be empty");

    print!("{}", simcore::snapshot::from_engine(&final_state).to_json());
}
