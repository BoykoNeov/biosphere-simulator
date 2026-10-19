//! **The above-target scrubber against every golden that reads it** — the lab build of
//! `docs/plans/post-roadmap-co2-level.md` §9, measured against its written predictions.
//!
//! ```text
//! cargo run --release -q -p station --example co2_target_lab              # all six
//! cargo run --release -q -p station --example co2_target_lab -- greenhouse # by name
//! ```
//!
//! Each golden's own build runs twice: as frozen, and with the scrubber switched to remove only
//! the CO₂ above 0.13 kPa of the cabin's air (the lab switch `with_scrubber_above_target` before the adoption). The
//! crop-sharing builds start the cabin at the new target, as the frozen ones start at their own
//! fixed point; the crew-only cabins keep their frozen start of 0. Every stock of the final
//! state is compared, boundary ones included: bit-identical stocks are counted, and moved ones
//! are listed largest first.
//!
//! ⚠ **It writes nothing and takes no decision.** The frozen side is checked against the
//! committed golden's bytes first, so a drift in this tool's copy of a build cannot pass as a
//! measurement.
//!
//! ⚠ **Since the adoption (2026-10-19) the two sides have swapped.** The frozen side is now the
//! above-target scrubber (the loader's), and the switched side is the pre-adoption first-order
//! form at 1e-3 (`domains::lab::with_first_order_scrubber`), its crop-sharing cabins started at
//! that form's fixed point, 3.796 mol. The moves it prints are therefore the plan §9 table's
//! with the sign reversed. Before the adoption, the switched states were saved and the
//! regenerated goldens compared against them byte for byte (plan §10).

use std::collections::BTreeMap;

use domains::eclss::{build_eclss, eclss_resolver, EclssParams, STEADY_STATE_SCENARIO, STEADY_STATE_STEPS};
use domains::lab::{with_first_order_scrubber, FIRST_ORDER_SCRUB_RATE};
use domains::params;
use simcore::integrator::EulerIntegrator;
use simcore::state::State;
use station::cabin::{build_cabin, cabin_resolver};
use station::greenhouse::{
    build_greenhouse, greenhouse_bio_resolver, greenhouse_cabin_resolver, run_greenhouse,
};
use station::harvest::{build_harvest, harvest_bio_resolver, harvest_cabin_resolver, run_harvest};
use station::params as station_params;
use station::run_station;
use station::scenario::{
    greenhouse_scenario, harvest_scenario, sealed_station_scenario, CABIN_GAS_SCENARIO,
    CABIN_GAS_STEPS, STATION_CABIN_AIR_MOL, WATER_RECOVERY_SCENARIO, WATER_RECOVERY_STEPS,
};
use station::sealed::{build_sealed_station, run_sealed, sealed_bio_resolver, sealed_fast_resolver};
use station::water::{build_water_recovery, water_recovery_resolver};

/// One side of a run: the final state, and whether the run stayed clean.
struct Run {
    last: State,
    rationed: u64,
    events: usize,
}

type Builder = fn(&EclssParams, Option<f64>) -> Run;

fn snapshot(state: &State) -> String {
    simcore::snapshot::from_engine(state).to_json()
}

fn eclss_run(p: &EclssParams, _start: Option<f64>) -> Run {
    let scenario = STEADY_STATE_SCENARIO;
    let (state, registry) = build_eclss(p, &scenario).expect("build_eclss");
    let resolver = eclss_resolver(&scenario).expect("eclss_resolver");
    let (last, rationed, events) = domains::run(
        &EulerIntegrator::new(registry),
        state,
        &resolver,
        scenario.dt_seconds,
        STEADY_STATE_STEPS,
    )
    .expect("run eclss");
    Run { last, rationed, events: events.len() }
}

fn cabin_gas_run(p: &EclssParams, _start: Option<f64>) -> Run {
    let crew = params::crew();
    let scenario = CABIN_GAS_SCENARIO;
    let (state, registry) = build_cabin(&crew, p, &scenario).expect("build_cabin");
    let resolver = cabin_resolver(&scenario).expect("cabin_resolver");
    let (last, rationed, events) = run_station(
        &EulerIntegrator::new(registry),
        state,
        &resolver,
        scenario.dt_seconds,
        CABIN_GAS_STEPS,
        &mut |_: &State| {},
    )
    .expect("run cabin");
    Run { last, rationed, events: events.len() }
}

fn water_recovery_run(p: &EclssParams, _start: Option<f64>) -> Run {
    let crew = params::crew();
    let recovery = station_params::water_recovery();
    let scenario = WATER_RECOVERY_SCENARIO;
    let (state, registry) =
        build_water_recovery(&crew, p, &recovery, &scenario).expect("build_water_recovery");
    let resolver = water_recovery_resolver(&scenario).expect("water_recovery_resolver");
    let (last, rationed, events) = run_station(
        &EulerIntegrator::new(registry),
        state,
        &resolver,
        scenario.dt_seconds,
        WATER_RECOVERY_STEPS,
        &mut |_: &State| {},
    )
    .expect("run water_recovery");
    Run { last, rationed, events: events.len() }
}

fn greenhouse_run(p: &EclssParams, start: Option<f64>) -> Run {
    let crew = params::crew();
    let mut scenario = greenhouse_scenario();
    if let Some(n) = start {
        scenario.bio.chamber_co2_mol0 = n;
    }
    let (state, bio_reg, cabin_reg) =
        build_greenhouse(&crew, p, &scenario, true, domains::crew::FECAL_WASTE)
            .expect("build_greenhouse");
    let bio = greenhouse_bio_resolver(&scenario).expect("bio_resolver");
    let cabin = greenhouse_cabin_resolver(&scenario).expect("cabin_resolver");
    let (states, rationed, events) = run_greenhouse(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(cabin_reg),
        state,
        &bio,
        &cabin,
        &scenario,
    )
    .expect("run greenhouse");
    Run { last: states.last().expect("a day").clone(), rationed, events: events.len() }
}

fn harvest_run(p: &EclssParams, start: Option<f64>) -> Run {
    let crew = params::crew();
    let harvest = station_params::harvest();
    let mut scenario = harvest_scenario();
    if let Some(n) = start {
        scenario.greenhouse.bio.chamber_co2_mol0 = n;
    }
    let (state, bio_reg, cabin_reg) =
        build_harvest(&crew, p, &harvest, &scenario, true, true).expect("build_harvest");
    let bio = harvest_bio_resolver(&scenario).expect("bio_resolver");
    let cabin = harvest_cabin_resolver(&scenario).expect("cabin_resolver");
    let (states, rationed, events) = run_harvest(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(cabin_reg),
        state,
        &bio,
        &cabin,
        &scenario,
    )
    .expect("run harvest");
    Run { last: states.last().expect("a day").clone(), rationed, events: events.len() }
}

fn sealed_station_run(p: &EclssParams, start: Option<f64>) -> Run {
    let charge = params::charge();
    let thermal = params::thermal();
    let crew = params::crew();
    let lamp = station_params::lamp();
    let mut scenario = sealed_station_scenario();
    if let Some(n) = start {
        scenario.bio.chamber_co2_mol0 = n;
    }
    let (state, bio_reg, fast_reg) = build_sealed_station(
        &charge,
        &thermal,
        &crew,
        p,
        &station_params::water_recovery(),
        &lamp,
        &station_params::harvest(),
        &scenario,
        false,
        false,
    )
    .expect("build_sealed_station");
    let bio = sealed_bio_resolver(&lamp, &scenario).expect("sealed_bio_resolver");
    let fast = sealed_fast_resolver(&charge, &scenario).expect("sealed_fast_resolver");
    let (states, rationed, events) = run_sealed(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(fast_reg),
        state,
        &bio,
        &fast,
        &scenario,
    )
    .expect("run sealed station");
    Run { last: states.last().expect("a day").clone(), rationed, events: events.len() }
}

/// (golden file, builder, whether the crop shares the cabin — so the start moves).
fn roster() -> Vec<(&'static str, Builder, bool)> {
    vec![
        ("eclss_state.json", eclss_run as Builder, false),
        ("cabin_gas_state.json", cabin_gas_run, false),
        ("water_recovery_state.json", water_recovery_run, false),
        ("greenhouse_state.json", greenhouse_run, true),
        ("harvest_state.json", harvest_run, true),
        ("sealed_station_state.json", sealed_station_run, true),
    ]
}

fn amounts(s: &State) -> BTreeMap<String, f64> {
    s.stocks.iter().map(|(k, v)| (k.clone(), v.amount)).collect()
}

fn main() {
    let only: Vec<String> = std::env::args().skip(1).collect();
    let frozen = params::eclss();
    let lab = with_first_order_scrubber(frozen, FIRST_ORDER_SCRUB_RATE);
    let target = frozen.co2_target_mol(STATION_CABIN_AIR_MOL).expect("a target");
    // The first-order form's fixed point, where the crop-sharing cabins started before.
    let old_start = 3.796;
    let golden_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/golden");
    println!(
        "frozen: above-target scrubber, {} Pa, k = {:e} /s, target {target:.6} mol of the \
         station's {STATION_CABIN_AIR_MOL} mol; switched: first-order, k = \
         {FIRST_ORDER_SCRUB_RATE:e} /s, crop cabins started at {old_start}\n",
        frozen.co2_partial_pressure, frozen.co2_scrub_rate
    );
    for (name, run, shares_crop) in roster() {
        if !only.is_empty() && !only.iter().any(|o| name.contains(o.as_str())) {
            continue;
        }
        let base = run(&frozen, None);
        let committed = std::fs::read_to_string(golden_dir.join(name)).expect("golden file");
        let reproduces = snapshot(&base.last) == committed;
        let switched = run(&lab, shares_crop.then_some(old_start));
        let (b, s) = (amounts(&base.last), amounts(&switched.last));
        let mut moved: Vec<(String, f64, f64, f64)> = b
            .iter()
            .filter(|(k, v)| s[*k].to_bits() != v.to_bits())
            .map(|(k, &v)| {
                let n = s[k];
                let rel = if v == 0.0 { f64::INFINITY } else { n / v - 1.0 };
                (k.clone(), v, n, rel)
            })
            .collect();
        moved.sort_by(|x, y| y.3.abs().total_cmp(&x.3.abs()));
        println!(
            "== {name}: frozen side reproduces the golden bytes: {reproduces}; \
             switched rationed {} events {} (frozen {} / {})",
            switched.rationed, switched.events, base.rationed, base.events
        );
        println!("   {} of {} stocks bit-identical, {} moved", b.len() - moved.len(), b.len(), moved.len());
        for (k, v, n, rel) in &moved {
            println!("   {k:<44} {v:>16.6} -> {n:>16.6}  ({:+.4} %)", 100.0 * rel);
        }
        println!();
    }
}
