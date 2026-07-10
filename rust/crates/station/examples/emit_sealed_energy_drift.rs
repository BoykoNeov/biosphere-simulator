//! Emit the RAW per-step `thermal.node` heat series the sealed-energy drift-summary golden
//! derives from (Phase-7 Step 5). Following the biosphere `emit_drift` discipline (advisor
//! #3), all of `drift.py` stays **Python-side**: Rust runs the 15-yr single-rate Power →
//! Thermal `HEAT_CLOSURE_SCENARIO` (diurnal solar ⇒ `n` advances ⇒ the SB radiator's real
//! `T_eq` attractor) and streams the raw node `amount` trajectory; the Python parity gate
//! folds `temp = space_temp + node/C`, per-year peaks (`year_summaries`), and the
//! `is_stationary` classifier, comparing to `sealed_energy_drift_summary.json`.

use domains::params;
use domains::thermal::NODE;
use simcore::hexfloat;
use simcore::integrator::EulerIntegrator;
use simcore::state::State;
use station::run_station;
use station::scenario::{HEAT_CLOSURE_SCENARIO, SEALED_ENERGY_DAYS, SEALED_ENERGY_YEARS};
use station::system::{build_station, station_resolver};

fn main() {
    let charge = params::charge();
    let thermal = params::thermal();
    let scenario = HEAT_CLOSURE_SCENARIO;
    let (state, registry) =
        build_station(&charge, &thermal, &scenario, None).expect("build_station");
    let resolver = station_resolver(&charge, &scenario).expect("station_resolver");
    let integrator = EulerIntegrator::new(registry);
    let steps = SEALED_ENERGY_DAYS * scenario.power.steps_per_day;

    let mut node: Vec<f64> = Vec::new();
    let (_final, rationed, events) = run_station(
        &integrator,
        state,
        &resolver,
        scenario.power.dt_seconds,
        steps,
        &mut |s: &State| node.push(s.stocks[NODE].amount),
    )
    .expect("run energy drift");

    assert_eq!(rationed, 0, "Tier-0: energy drift rationed must be 0");
    assert!(
        events.is_empty(),
        "Tier-0: energy drift events must be empty"
    );

    println!("{{");
    println!("  \"horizon_years\": {SEALED_ENERGY_YEARS},");
    println!("  \"steps_per_day\": {},", scenario.power.steps_per_day);
    print!("  \"node_heat\": [");
    for (i, v) in node.iter().enumerate() {
        if i > 0 {
            print!(",");
        }
        print!("\"{}\"", hexfloat::format(*v));
    }
    println!("]");
    println!("}}");
}
