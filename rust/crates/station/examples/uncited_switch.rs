//! The **station-side value switch** for the uncited equipment numbers — Step 5 of the
//! 2026-09-29 review (`docs/plans/post-roadmap-uncited-triage.md`).
//!
//! ```text
//! cargo run --release -q -p station --example uncited_switch            # every row
//! cargo run --release -q -p station --example uncited_switch -- radiator_area
//! ```
//!
//! `domains`' `value_switch` reaches the fifteen biosphere param files only. The equipment
//! values — the cabin's scrubber, condenser and O₂ regulator, the radiator, the battery's
//! self-discharge, water recovery and harvest — are `DESIGN` numbers in the sibling and
//! station files, and every one is handed to its builder **as a value**, so a copy of the
//! struct with one field scaled is the whole substitution: the same builders, resolvers and
//! runners the goldens use, one field different.
//!
//! Each row runs on a build that READS its field:
//!
//! * the eclss and water-recovery fields on the **sealed station** (the multi-year
//!   five-domain build, ~90 s a run in release);
//! * `harvest_rate` on the **harvest** golden's build (`HARVEST_SCENARIO`, 7 days). ⚠ Not on
//!   the sealed station: its golden builds with `with_harvest = false`, so no flow there reads
//!   the field — this tool's first run measured it there and read 0.000 % everywhere;
//! * the radiator fields on the **15-year heat-closure** run, read as the one station gate
//!   reads it (the annual peak node temperature, `non_collapsing(floor = 100 K)`);
//! * `self_discharge_rate` on the standalone **power** self-discharge run.
//!
//! Every row runs at ×0.9 and ×1.1. A row whose param file records an error far larger than
//! that — the scrubber ~8–30× fast, water recovery ~2–3 orders fast, the harvest ~65–78× fast
//! — also runs at the recorded size, so a value cannot read as inert at ±10 % while carrying a
//! thousandfold error nobody measured.
//!
//! ⚠ **It writes nothing and takes no decision.** No param file, golden or manifest is read
//! for writing. A run that rations, reverses or fails is printed as that result, never
//! hidden behind a panic — the goldens' runners assert well-fedness, this does not.
//!
//! ⚠ **The chamber's five DESIGN values are not here.** When this tool was written the build
//! read `params::chamber()` inside itself, so there was no value to hand in. Since 2026-10-21
//! they are an input (`SealedStationScenario::chamber`, `docs/plans/post-roadmap-big-station.md`),
//! so they could be added as rows; not done.

use std::collections::BTreeMap;

use domains::biosphere::drift::year_summaries;
use domains::eclss::EclssParams;
use domains::params;
use domains::power::{build_power, power_resolver, BOUNDED_SOC_SCENARIO, SELF_DISCHARGE_DAYS};
use domains::thermal::{ThermalParams, NODE};
use simcore::integrator::EulerIntegrator;
use simcore::quantities::StockKind;
use simcore::state::State;
use station::flows::{HarvestParams, WaterRecoveryParams};
use station::harvest::{build_harvest, harvest_bio_resolver, harvest_cabin_resolver, run_harvest};
use station::params as station_params;
use station::scenario::{
    harvest_scenario, sealed_station_scenario, HEAT_CLOSURE_SCENARIO, SEALED_ENERGY_DAYS,
    SEALED_STATION_SEASON_DAYS,
};
use station::sealed::{build_sealed_station, run_sealed, sealed_bio_resolver, sealed_fast_resolver};
use station::system::{build_station, station_resolver};

/// The perturbation every row runs at, as multiples of its frozen value.
const FACTORS: [f64; 2] = [0.9, 1.1];

/// Every sealed-station input this tool varies, as values.
#[derive(Clone)]
struct SealedInputs {
    eclss: EclssParams,
    recovery: WaterRecoveryParams,
}

/// What one run produced: whether it was clean, and the stocks it ended on.
struct Outcome {
    /// `None` when the run completed; the error text when it did not.
    failed: Option<String>,
    rationed: u64,
    events: usize,
    /// Final amounts of every non-boundary stock, by id.
    finals: BTreeMap<String, f64>,
    /// The run-minimum of every non-boundary stock over the recorded states.
    minima: BTreeMap<String, f64>,
    /// A named scalar the row's gate reads, when there is one.
    gate: Option<(&'static str, f64)>,
}

fn non_boundary(state: &State) -> impl Iterator<Item = (&String, f64)> {
    state
        .stocks
        .iter()
        .filter(|(_, s)| s.kind != StockKind::Boundary)
        .map(|(id, s)| (id, s.amount))
}

fn outcome_of(states: &[State], rationed: u64, events: usize) -> Outcome {
    let last = states.last().expect("at least one recorded state");
    let finals = non_boundary(last).map(|(k, v)| (k.clone(), v)).collect();
    let mut minima: BTreeMap<String, f64> = BTreeMap::new();
    for s in states {
        for (k, v) in non_boundary(s) {
            let m = minima.entry(k.clone()).or_insert(v);
            *m = m.min(v);
        }
    }
    Outcome { failed: None, rationed, events, finals, minima, gate: None }
}

fn failed(e: impl std::fmt::Debug) -> Outcome {
    Outcome {
        failed: Some(format!("{e:?}")),
        rationed: 0,
        events: 0,
        finals: BTreeMap::new(),
        minima: BTreeMap::new(),
        gate: None,
    }
}

fn run_sealed_with(inputs: &SealedInputs) -> Outcome {
    let charge = params::charge();
    let thermal = params::thermal();
    let crew = params::crew();
    let lamp = station_params::lamp();
    let scenario = sealed_station_scenario();
    let built = build_sealed_station(
        &charge,
        &thermal,
        &crew,
        &inputs.eclss,
        &inputs.recovery,
        &lamp,
        &station_params::harvest(),
        &scenario,
        false,
        false,
    );
    let (state, bio_reg, fast_reg) = match built {
        Ok(b) => b,
        Err(e) => return failed(e),
    };
    let bio = sealed_bio_resolver(&lamp, &scenario).expect("sealed_bio_resolver");
    let fast = sealed_fast_resolver(&charge, &crew, &scenario).expect("sealed_fast_resolver");
    match run_sealed(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(fast_reg),
        state,
        &bio,
        &fast,
        &scenario,
    ) {
        Ok((states, rationed, events)) => outcome_of(&states, rationed, events.len()),
        Err(e) => failed(e),
    }
}

/// The harvest golden's run (`goldens::harvest`), with one harvest value different.
fn run_harvest_with(harvest: &HarvestParams) -> Outcome {
    let crew = params::crew();
    let eclss = params::eclss();
    let scenario = harvest_scenario();
    let (state, bio_reg, cabin_reg) =
        match build_harvest(&crew, &eclss, harvest, &scenario, true, true) {
            Ok(b) => b,
            Err(e) => return failed(e),
        };
    let bio = harvest_bio_resolver(&scenario).expect("harvest_bio_resolver");
    let cabin = harvest_cabin_resolver(&crew, &scenario).expect("harvest_cabin_resolver");
    match run_harvest(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(cabin_reg),
        state,
        &bio,
        &cabin,
        &scenario,
    ) {
        Ok((states, rationed, events)) => outcome_of(&states, rationed, events.len()),
        Err(e) => failed(e),
    }
}

/// The heat-closure run as `science_gates::runs::node_peak_temps` takes it, with the gate's
/// own quantity: the lowest annual peak node temperature (floor 100 K).
fn run_heat_closure_with(thermal: &ThermalParams) -> Outcome {
    let charge = params::charge();
    let scenario = HEAT_CLOSURE_SCENARIO;
    let (state, registry) = match build_station(&charge, thermal, &scenario, None) {
        Ok(b) => b,
        Err(e) => return failed(e),
    };
    let resolver = station_resolver(&charge, &scenario).expect("station_resolver");
    let integrator = EulerIntegrator::new(registry);
    let mut node_temp: Vec<f64> = Vec::new();
    let mut last: Option<State> = None;
    let run = station::run_station(
        &integrator,
        state,
        &resolver,
        scenario.power.dt_seconds,
        SEALED_ENERGY_DAYS * scenario.power.steps_per_day,
        &mut |s: &State| {
            node_temp.push(thermal.space_temperature + s.stocks[NODE].amount / thermal.heat_capacity);
        },
    );
    match run {
        Ok((final_state, rationed, events)) => {
            last.replace(final_state);
            let steps_per_year =
                scenario.power.steps_per_day as usize * SEALED_STATION_SEASON_DAYS;
            let peaks = year_summaries(&node_temp, steps_per_year, |seg: &[f64]| {
                seg.iter().fold(f64::NEG_INFINITY, |a, &t| a.max(t))
            });
            let low = peaks.iter().fold(f64::INFINITY, |a, &t| a.min(t));
            let mut o = outcome_of(&[last.expect("final state")], rationed, events.len());
            o.gate = Some(("lowest annual peak node T (K), floor 100", low));
            o
        }
        Err(e) => failed(e),
    }
}

fn run_self_discharge_with(rate: f64) -> Outcome {
    let charge = params::charge();
    let mut sd = params::self_discharge();
    sd.self_discharge_rate = rate;
    let scenario = BOUNDED_SOC_SCENARIO;
    let (state, registry) = match build_power(&charge, &scenario, Some(sd)) {
        Ok(b) => b,
        Err(e) => return failed(e),
    };
    let resolver = power_resolver(&charge, &scenario).expect("power_resolver");
    let integrator = EulerIntegrator::new(registry);
    match domains::run(
        &integrator,
        state,
        &resolver,
        scenario.dt_seconds,
        SELF_DISCHARGE_DAYS * scenario.steps_per_day,
    ) {
        Ok((final_state, rationed, events)) => outcome_of(&[final_state], rationed, events.len()),
        Err(e) => failed(e),
    }
}

/// One row: a field, the frozen value, and how to run a scaled copy.
struct Row {
    /// Rows sharing a build share one baseline run (the sealed station is ~90 s).
    build: &'static str,
    file: &'static str,
    field: &'static str,
    frozen: f64,
    /// Further multiples at the error the param file itself records, with that record.
    recorded: &'static [(f64, &'static str)],
    run: Box<dyn Fn(f64) -> Outcome>,
}

fn rows() -> Vec<Row> {
    let base = SealedInputs {
        eclss: params::eclss(),
        recovery: station_params::water_recovery(),
    };
    let thermal = params::thermal();
    let mut rows: Vec<Row> = Vec::new();
    macro_rules! sealed_row {
        ($file:literal, $group:ident . $field:ident, $recorded:expr) => {{
            let b = base.clone();
            rows.push(Row {
                build: "sealed_station",
                file: $file,
                field: stringify!($field),
                frozen: base.$group.$field,
                recorded: $recorded,
                run: Box::new(move |v| {
                    let mut i = b.clone();
                    i.$group.$field = v;
                    run_sealed_with(&i)
                }),
            });
        }};
    }
    sealed_row!(
        "eclss.yaml",
        eclss.co2_scrub_rate,
        &[
            (0.125, "ISS CDRA's 144-min bed cycle, ~8x our tau"),
            (1.0 / 30.0, "the record's '1-1.5 orders fast', upper end"),
        ]
    );
    sealed_row!("eclss.yaml", eclss.condense_rate, &[]);
    sealed_row!("eclss.yaml", eclss.o2_makeup_gain, &[]);
    sealed_row!(
        "water_recovery.yaml",
        recovery.recovery_rate,
        &[(0.01, "the record's '2-3 orders slower', low end"), (0.001, "its high end")]
    );
    sealed_row!("water_recovery.yaml", recovery.recovery_efficiency, &[]);
    let harvest = station_params::harvest();
    rows.push(Row {
        build: "harvest",
        file: "harvest.yaml",
        field: "harvest_rate",
        frozen: harvest.harvest_rate,
        recorded: &[(1.0 / 70.0, "the record's '65-78x fast' (one batch at 75-90 dAP)")],
        run: Box::new(move |v| {
            let mut h = harvest;
            h.harvest_rate = v;
            run_harvest_with(&h)
        }),
    });
    macro_rules! thermal_row {
        ($field:ident) => {{
            let t = thermal.clone();
            rows.push(Row {
                build: "heat_closure",
                file: "radiator.yaml",
                field: stringify!($field),
                frozen: thermal.$field,
                recorded: &[],
                run: Box::new(move |v| {
                    let mut p = t.clone();
                    p.$field = v;
                    run_heat_closure_with(&p)
                }),
            });
        }};
    }
    thermal_row!(radiator_area);
    thermal_row!(heat_capacity);
    rows.push(Row {
        build: "power_self_discharge",
        file: "self_discharge.yaml",
        field: "self_discharge_rate",
        frozen: params::self_discharge().self_discharge_rate,
        recorded: &[],
        run: Box::new(run_self_discharge_with),
    });
    rows
}

fn rel(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        if a == 0.0 { 0.0 } else { f64::INFINITY }
    } else {
        (a - b) / b.abs()
    }
}

/// The three largest relative moves of `field` (finals or minima) against the baseline.
fn top_movers(
    base: &BTreeMap<String, f64>,
    run: &BTreeMap<String, f64>,
) -> Vec<(String, f64)> {
    let mut moves: Vec<(String, f64)> = base
        .iter()
        .filter_map(|(k, &b)| run.get(k).map(|&r| (k.clone(), rel(r, b))))
        .collect();
    moves.sort_by(|x, y| y.1.abs().total_cmp(&x.1.abs()).then_with(|| x.0.cmp(&y.0)));
    moves.truncate(3);
    moves
}

fn describe(o: &Outcome) -> String {
    match &o.failed {
        Some(e) => format!("FAILED: {e}"),
        None => format!("rationed {} events {}", o.rationed, o.events),
    }
}

fn main() {
    let only: Vec<String> = std::env::args().skip(1).collect();
    let rows: Vec<Row> = rows()
        .into_iter()
        .filter(|r| only.is_empty() || only.iter().any(|o| o == r.field))
        .collect();
    if rows.is_empty() {
        eprintln!("uncited_switch: no row matches {only:?}");
        std::process::exit(2);
    }
    println!("station uncited-value switch — MEASURED QUANTITIES, ±10 % of each frozen value,");
    println!("and the error the param file records where that is larger.");
    println!("Moves are relative to the same build at the frozen value; boundary stocks excluded.\n");
    let mut baselines: BTreeMap<&'static str, Outcome> = BTreeMap::new();
    for row in &rows {
        let base = baselines.entry(row.build).or_insert_with(|| (row.run)(row.frozen));
        println!("{} / {} = {:e}   on {}, baseline: {}", row.file, row.field, row.frozen, row.build, describe(base));
        if let Some((name, v)) = base.gate {
            println!("    gate quantity at the frozen value: {name} = {v:.6}");
        }
        let steps = FACTORS.iter().map(|&f| (f, "")).chain(row.recorded.iter().copied());
        for (f, why) in steps {
            let o = (row.run)(row.frozen * f);
            let why = if why.is_empty() { String::new() } else { format!(" (recorded: {why})") };
            println!("  ×{f:.4}{why}: {}", describe(&o));
            if o.failed.is_some() || base.failed.is_some() {
                continue;
            }
            if let (Some((_, b)), Some((_, v))) = (base.gate, o.gate) {
                let used = 1.0 - (v - 100.0) / (b - 100.0);
                println!("    gate quantity {v:.6} ({:+.3} %; margin above the 100 K floor used {:+.3} %)",
                    100.0 * rel(v, b), 100.0 * used);
            }
            for (label, b, r) in [("final", &base.finals, &o.finals), ("run-min", &base.minima, &o.minima)] {
                let movers: Vec<String> = top_movers(b, r)
                    .into_iter()
                    .map(|(k, m)| format!("{k} {:+.3} %", 100.0 * m))
                    .collect();
                println!("    {label:>7}: {}", movers.join(", "));
            }
        }
        println!();
    }
}
