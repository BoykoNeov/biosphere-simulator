//! Emit `sealed_energy_drift_summary.json` — the sealed station's 15-yr energy **stability
//! signature**, folded in Rust as of slice C5 of the reference flip.
//!
//! ⚠ **This program's output changed KIND in C5.** Through Phase-7 Step 5 it streamed the
//! raw per-step `thermal.node` heat series and the *Python* gate folded it (`temp =
//! space_temp + node/C`, per-year peaks, the `is_stationary` classifier) into the golden —
//! the "one run, two authors" split, since the fold is what decides what the summary
//! *says*. `domains::biosphere::drift` now carries that kit, so this example emits the
//! artifact itself and the golden is Rust's end to end.
//!
//! The run is unchanged: the 15-yr single-rate Power → Thermal `HEAT_CLOSURE_SCENARIO`
//! (diurnal solar ⇒ `n` advances ⇒ the SB radiator's real `T_eq` attractor).
//!
//! ⚠ The fold lives in `domains::biosphere::drift` and is used here by the *station* —
//! deliberately, and mirroring Python, where `tests/test_regression_sealed_station.py`
//! imports `year_summaries` / `same_phase_diffs` / `is_stationary` from
//! `domains.biosphere.drift`. The module is an instrument, generic over `State`; the
//! caller supplies the per-year `summary_fn`, so nothing biosphere-specific comes with it.

use domains::biosphere::drift::{is_stationary, same_phase_diffs, year_summaries};
use domains::params;
use domains::thermal::NODE;
use simcore::hexfloat;
use simcore::integrator::EulerIntegrator;
use simcore::state::State;
use station::run_station;
use station::scenario::{
    HEAT_CLOSURE_SCENARIO, SEALED_ENERGY_DAYS, SEALED_ENERGY_YEARS, SEALED_STATION_SEASON_DAYS,
};
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

    // ⚠ The node TEMPERATURE is reduced in the observer, not after the run. Python's
    // `peak_temp` computes `space_temp + node/C` per state and then takes the `max` over
    // the segment, so reducing per step and folding `max` afterwards is the same sequence
    // of IEEE operations — and it avoids materializing 109,801 `State`s to fold over.
    let mut node_temp: Vec<f64> = Vec::new();
    let (_final, rationed, events) = run_station(
        &integrator,
        state,
        &resolver,
        scenario.power.dt_seconds,
        steps,
        &mut |s: &State| {
            node_temp
                .push(thermal.space_temperature + s.stocks[NODE].amount / thermal.heat_capacity)
        },
    )
    .expect("run energy drift");

    assert_eq!(rationed, 0, "Tier-0: energy drift rationed must be 0");
    assert!(
        events.is_empty(),
        "Tier-0: energy drift events must be empty"
    );

    // ⚠ In DAYS times steps-per-day, i.e. the trajectory's own index unit — `run_station`
    // appends one state per POWER step. The biosphere's `run_season` trajectory is indexed
    // differently (see `year_summaries`); passing one unit where the other is meant is the
    // trap `docs/plans/post-roadmap-step-unfreeze.md` §1 records.
    let steps_per_year = scenario.power.steps_per_day as usize * SEALED_STATION_SEASON_DAYS;
    let peaks = year_summaries(&node_temp, steps_per_year, |segment: &[f64]| {
        segment.iter().fold(f64::NEG_INFINITY, |acc, &t| acc.max(t))
    });
    // Period 1, not 2: Power carries no seasonal forcing, so the node's attractor is a
    // fixed point and consecutive years ARE the same phase.
    let stationary = is_stationary(&same_phase_diffs(&peaks, 1), 0.1, 1e-3, 0);

    // Canonical JSON: `indent=2, sort_keys=True` + a trailing newline, matching
    // `sim_io.dumps` and every other committed golden. The keys below are in sorted order
    // BY HAND — this is a `println!`, not a serializer, so the ordering is an invariant of
    // this function and the byte-compare against the committed golden is what holds it.
    println!("{{");
    println!("  \"horizon_years\": {SEALED_ENERGY_YEARS},");
    println!("  \"is_stationary\": {stationary},");
    println!("  \"node_peak_temp_k\": [");
    for (i, v) in peaks.iter().enumerate() {
        let comma = if i + 1 < peaks.len() { "," } else { "" };
        println!("    \"{}\"{}", hexfloat::format(*v), comma);
    }
    println!("  ]");
    println!("}}");
}
