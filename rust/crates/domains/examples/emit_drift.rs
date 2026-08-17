//! Emit the RAW per-step biosphere series the drift-summary golden derives from
//! (Phase-7 P7.4). Rust emits the per-step `leaf_c` (perennial + consumer) and
//! `consumer_carbon` (consumer) trajectories over the 15-yr runs; the Python parity gate
//! folds them into per-year summaries (`year_summaries`) and the period class
//! (`is_period_2`) and compares to `drift_summary.json`. So this example reproduces NO
//! segmentation — it only runs the two `run_perennial` trajectories and streams the raw
//! stock amounts.
//!
//! ⚠ **The REASON for that changed in slice C5, even though the behaviour did not.** This
//! file used to say the plan (advisor #3) keeps `drift.py` **Python-side**. That is no
//! longer true: `domains::biosphere::drift` now carries the whole fold kit, and the
//! station's `emit_sealed_energy_drift` was converted to emit its summary directly.
//!
//! What blocks the same conversion HERE is not the absence of a Rust fold — it is a
//! measurement. Folding the Rust series moves **4 of `drift_summary.json`'s 45 values**
//! (≤7 ULP: the consumer trajectory diverges by 1 ULP at step 4095 and the contracting
//! attractor damps it back to a bit-identical final state by year 15). Python would then
//! need tolerance-gating, i.e. an entry on `golden_platform.PYTHON_DIVERGES` — and
//! `test_every_diverging_scenario_keeps_a_byte_gated_sibling` goes red, because this
//! example serves exactly one golden and so has no byte-gated sibling under that gate's
//! emitter-program key. Widening that key from inside the slice that needs it widened is
//! the co-adaptation this repo refuses, so the authorship move is **deferred to its own
//! ceremony**. See §5h of `docs/plans/post-roadmap-reference-flip.md`.
//!
//! ⚠ So the raw-series shape below is now a **deliberate holdover with a named blocker**,
//! not the plan's standing design. Do not "finish the job" by converting it without
//! resolving the gate first.

use domains::biosphere::stocks::{CONSUMER_CARBON, LEAF_C};
use domains::biosphere::{
    consumer_chamber_scenario, perennial_chamber_scenario, run_perennial, season_setup,
    season_steps, steps_for_years, BIO_DT, LONG_HORIZON_YEARS, SEASON_DAYS,
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
    let steps = steps_for_years(years);

    let perennial = perennial_chamber_scenario();
    let (p_state, p_integ, p_res) = season_setup(&perennial, years).expect("perennial setup");
    let mut perennial_leaf: Vec<f64> = Vec::new();
    run_perennial(
        &p_integ,
        p_state,
        &perennial,
        &p_res,
        BIO_DT,
        steps,
        season_steps(),
        &mut |s: &State| perennial_leaf.push(s.stocks[LEAF_C].amount),
    )
    .expect("run perennial");

    let consumer = consumer_chamber_scenario();
    let (c_state, c_integ, c_res) = season_setup(&consumer, years).expect("consumer setup");
    let mut consumer_leaf: Vec<f64> = Vec::new();
    let mut consumer_carbon: Vec<f64> = Vec::new();
    run_perennial(
        &c_integ,
        c_state,
        &consumer,
        &c_res,
        BIO_DT,
        steps,
        season_steps(),
        &mut |s: &State| {
            consumer_leaf.push(s.stocks[LEAF_C].amount);
            consumer_carbon.push(s.stocks[CONSUMER_CARBON].amount);
        },
    )
    .expect("run consumer");

    println!("{{");
    println!("  \"horizon_years\": {years},");
    // ⚠ In physical DAYS, deliberately. The Python parity gate converts it with its own
    // `steps_for` to segment these step-indexed trajectories; emitting steps here would
    // double-convert and slice the 15-yr run into 60 quarter-years.
    println!("  \"season_days\": {SEASON_DAYS},");
    emit_array("perennial_leaf", &perennial_leaf, false);
    emit_array("consumer_leaf", &consumer_leaf, false);
    emit_array("consumer_carbon", &consumer_carbon, true);
    println!("}}");
}
