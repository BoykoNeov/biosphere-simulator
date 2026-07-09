//! Emit the RAW per-step biosphere series the drift-summary golden derives from
//! (Phase-7 P7.4). The plan (advisor #3) keeps `drift.py` **Python-side**: Rust emits the
//! per-step `leaf_c` (perennial + consumer) and `consumer_carbon` (consumer) trajectories
//! over the 15-yr runs; the Python parity gate folds them into per-year summaries
//! (`year_summaries`) and the period class (`is_period_2`) and compares to
//! `drift_summary.json`. So this example reproduces NO segmentation — it only runs the two
//! `run_perennial` trajectories and streams the raw stock amounts.

use domains::biosphere::stocks::{CONSUMER_CARBON, LEAF_C};
use domains::biosphere::{
    consumer_chamber_scenario, perennial_chamber_scenario, run_perennial, season_setup,
    steps_for, LONG_HORIZON_YEARS, SEASON_DAYS,
};
use simcore::hexfloat;
use simcore::state::State;

fn emit_array(name: &str, values: &[f64], last: bool) {
    print!("  \"{name}\": [");
    for (i, v) in values.iter().enumerate() {
        if i > 0 {
            print!(",");
        }
        print!("\"{}\"", hexfloat::format(*v));
    }
    println!("]{}", if last { "" } else { "," });
}

fn main() {
    let years = LONG_HORIZON_YEARS;
    let steps = steps_for(years);

    let perennial = perennial_chamber_scenario();
    let (p_state, p_integ, p_res) = season_setup(&perennial, years).expect("perennial setup");
    let mut perennial_leaf: Vec<f64> = Vec::new();
    run_perennial(
        &p_integ, p_state, &perennial, &p_res, 1.0, steps, SEASON_DAYS,
        &mut |s: &State| perennial_leaf.push(s.stocks[LEAF_C].amount),
    )
    .expect("run perennial");

    let consumer = consumer_chamber_scenario();
    let (c_state, c_integ, c_res) = season_setup(&consumer, years).expect("consumer setup");
    let mut consumer_leaf: Vec<f64> = Vec::new();
    let mut consumer_carbon: Vec<f64> = Vec::new();
    run_perennial(
        &c_integ, c_state, &consumer, &c_res, 1.0, steps, SEASON_DAYS,
        &mut |s: &State| {
            consumer_leaf.push(s.stocks[LEAF_C].amount);
            consumer_carbon.push(s.stocks[CONSUMER_CARBON].amount);
        },
    )
    .expect("run consumer");

    println!("{{");
    println!("  \"horizon_years\": {years},");
    println!("  \"season_days\": {SEASON_DAYS},");
    emit_array("perennial_leaf", &perennial_leaf, false);
    emit_array("consumer_leaf", &consumer_leaf, false);
    emit_array("consumer_carbon", &consumer_carbon, true);
    println!("}}");
}
