//! Run the single-rate `station` under a **deep brownout** (the Phase-8 P8.5 energy failure
//! cascade) and emit its 12-day final `State` as `sim_io` hex-float JSON — the **headless
//! reference** the cross-boundary perturbation smoke (`godot/perturbation_smoke.gd`) is
//! compared against byte-for-byte (the "the FFI didn't corrupt determinism" proof, on a
//! perturbed run this time).
//!
//! The blackout (`factor = 0` over days `[2, 8)`) empties the battery so `LoadDraw` cannot be
//! met → `rationed > 0` (asserted — the failure cascade fired; the Euler backstop conserves
//! as it rations). This is the **same build path** the bridge's
//! `build_perturbed("station", "brownout", …)` takes: `build_station` + `station_resolver` +
//! `with_brownout`, then `run_station(N)` — which equals `N × session.step()` (session
//! parity), so the final snapshot is byte-identical to the Godot cdylib run.

use domains::params;
use simcore::integrator::EulerIntegrator;
use station::perturbations::with_brownout;
use station::run_station;
use station::scenario::HEAT_CLOSURE_SCENARIO;
use station::system::{build_station, station_resolver};

const DAYS: u64 = 12; // must match godot/perturbation_smoke.gd

fn main() {
    let charge = params::charge();
    let thermal = params::thermal();
    let scenario = HEAT_CLOSURE_SCENARIO;
    let spd = scenario.power.steps_per_day;
    // The smoke hardcodes SPD = 24 (start=48, end=192, steps=288); guard the invariant so a
    // scenario change fails here loudly rather than silently desyncing the two sides.
    assert_eq!(spd, 24, "perturbation_smoke.gd assumes steps_per_day == 24");

    let (state, registry) =
        build_station(&charge, &thermal, &scenario, None).expect("build_station");
    let resolver = station_resolver(&charge, &scenario).expect("station_resolver");
    // Deep blackout on days [2, 8).
    let resolver = with_brownout(resolver, 2 * spd, 8 * spd, 0.0).expect("with_brownout");
    let integrator = EulerIntegrator::new(registry);
    let mut noop = |_: &simcore::state::State| {};
    let (final_state, rationed, events) = run_station(
        &integrator,
        state,
        &resolver,
        scenario.power.dt_seconds,
        DAYS * spd,
        &mut noop,
    )
    .expect("run station");

    assert!(rationed > 0, "deep brownout must ration (the failure cascade)");
    assert!(events.is_empty(), "no POPULATION stock ⇒ no extinction events");

    print!("{}", simcore::snapshot::from_engine(&final_state).to_json());
}
