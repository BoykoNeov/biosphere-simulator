//! The **uncited-value triage** for the biosphere — Step 5 of the 2026-09-29 review
//! (`docs/plans/post-roadmap-uncited-triage.md`).
//!
//! ```text
//! cargo run --release -q -p domains --example uncited_triage            # every row
//! cargo run --release -q -p domains --example uncited_triage -- vcmax q10
//! cargo run --release -q -p domains --example uncited_triage -- --recorded
//! ```
//!
//! One row per uncited biosphere value (the plan's roster rule), each run twice: ±10 % of the
//! frozen value, or ±1 °C for a temperature, where ±10 % of a Celsius number is not a size
//! at all (`t_base` is 0). The partition table runs as its four organ shares, each scaled at
//! every knot with the other three compensated (`lab::partition`). Two readings per run:
//!
//! 1. **The gated readouts** — the value switch's own [`SPECS`], 15-year rows included — as a
//!    relative move, plus *margin used* for the three whose bound is a plain number
//!    (`5.0 < peak LAI < 8.0`, `peak W < 14.4248`, the fixed point `> 0.55`).
//! 2. **Every final stock** of the four frozen season runs, largest relative moves first.
//!    The gated readouts are carbon and CO₂ only, so a water or nitrogen value can move
//!    nothing there and still move the soil; this column is what tells the two apart.
//!
//! 3. **At the recorded range** (`RECORDED`, or `-- --recorded` alone): the values whose own
//!    `source:` records a range or an error wider than ±10 %, run again at that size.
//!
//! ⚠ **It writes nothing and takes no decision.** Each substitution lives in memory for one
//! run, as in `value_switch`. A row whose frozen value is typed wrong here is refused before it
//! runs: the substitution at the frozen value must rebuild the frozen params bit for bit.
//!
//! ⚠ **±10 % ranks sensitivity, not error.** A value whose recorded range is wider than ±10 %
//! (Vcmax 80–120, r_a 30–100) can sit low here and still be the larger unknown — the plan's
//! table prints each row's recorded range beside its reading for exactly that reason.

use std::collections::BTreeMap;

use domains::biosphere::params::BiosphereParams;
use domains::biosphere::{
    consumer_chamber_scenario, perennial_chamber_scenario, run_perennial_with, run_season,
    sealed_chamber_scenario, season_setup_with, season_steps, steps_for_years, SeasonScenario,
    BIO_DT, CONSUMER_CHAMBER_YEARS, DEFAULT_SCENARIO, PERENNIAL_CHAMBER_YEARS,
    SEALED_CHAMBER_YEARS,
};
use domains::lab::partition::{biosphere_with_share, Knot};
use domains::lab::report::{measure, Column, SPECS};
use domains::lab::{biosphere_with, Substitution};
use simcore::quantities::StockKind;
use simcore::state::State;

/// How a row is perturbed.
#[derive(Clone, Copy)]
enum Step {
    /// ×0.9 and ×1.1.
    Relative,
    /// −1 and +1 in the value's own unit (°C).
    Celsius,
}

/// One uncited value: where it lives, its frozen value, its class, and how to step it.
struct Row {
    file: &'static str,
    field: &'static str,
    frozen: f64,
    class: &'static str,
    step: Step,
}

const fn r(file: &'static str, field: &'static str, frozen: f64, class: &'static str) -> Row {
    Row { file, field, frozen, class, step: Step::Relative }
}

const fn c(file: &'static str, field: &'static str, frozen: f64, class: &'static str) -> Row {
    Row { file, field, frozen, class, step: Step::Celsius }
}

/// The roster's scalar biosphere rows (the partition table is the four `ORGAN_ROWS`).
const ROWS: &[Row] = &[
    r("photosynthesis.yaml", "vcmax", 100.0, "TODO(cite)"),
    r("photosynthesis.yaml", "jmax", 180.0, "TODO(cite)"),
    r("photosynthesis.yaml", "quantum_yield", 0.3, "TODO(cite)"),
    r("photosynthesis.yaml", "theta", 0.7, "TODO(cite)"),
    c("photosynthesis.yaml", "t_min", 0.0, "TODO(cite)"),
    c("photosynthesis.yaml", "t_opt_lo", 15.0, "TODO(cite)"),
    c("photosynthesis.yaml", "t_opt_hi", 25.0, "TODO(cite)"),
    c("photosynthesis.yaml", "t_max", 35.0, "TODO(cite)"),
    r("respiration.yaml", "maintenance_coef", 0.02, "TODO(cite)"),
    r("respiration.yaml", "q10", 2.0, "TODO(cite)"),
    c("respiration.yaml", "t_ref", 25.0, "TODO(cite)"),
    r("respiration.yaml", "growth_efficiency", 0.75, "TODO(cite)"),
    r("respiration.yaml", "o2_half_saturation", 0.0001, "TODO(cite)"),
    c("phenology.yaml", "t_base", 0.0, "TODO(cite)"),
    c("phenology.yaml", "t_cap", 30.0, "TODO(cite)"),
    r("transpiration.yaml", "aerodynamic_resistance", 50.0, "TODO(cite)"),
    r("transpiration.yaml", "surface_resistance", 70.0, "TODO(cite)"),
    r("nitrogen.yaml", "max_uptake_capacity", 0.0015, "TODO(cite)"),
    r("herbivory.yaml", "o2_half_saturation", 0.0001, "TODO(cite)"),
    r("senescence.yaml", "rdr_leaf", 0.02, "TODO(cite)"),
    r("senescence.yaml", "rdr_stem", 0.005, "TODO(cite)"),
    r("senescence.yaml", "rdr_root", 0.01, "TODO(cite)"),
    r("herbivory.yaml", "grazing_rate", 0.004, "DESIGN"),
    r("herbivory.yaml", "respiration_rate", 0.02, "DESIGN"),
    r("herbivory.yaml", "mortality_rate", 0.01, "DESIGN"),
    r("water_cycle.yaml", "recycling_rate", 0.5, "DESIGN"),
    r("decomposition.yaml", "decomposition_rate", 0.011, "CALIBRATED"),
    r("microbial_respiration.yaml", "microbial_respiration_rate", 0.016, "CALIBRATED"),
    r("stem_reserves.yaml", "trigger_dvs", 1.0, "OURS"),
    r("stem_reserves.yaml", "remobilization_rate", 0.1, "UNCITED (book, bare)"),
];

/// Values whose own `source:` records a range or an error wider than ±10 %: each runs again at
/// the recorded size, with the record it comes from. Read off the param files 2026-10-19.
const RECORDED: &[(&str, &str, &[f64], &str)] = &[
    ("photosynthesis.yaml", "vcmax", &[80.0, 120.0], "literature-typical C3, 80-120"),
    ("photosynthesis.yaml", "jmax", &[150.0, 200.0], "1.5-2 x Vcmax"),
    ("respiration.yaml", "maintenance_coef", &[0.015, 0.03], "0.015-0.03 at reference T"),
    ("respiration.yaml", "growth_efficiency", &[0.7, 0.85], "Yg 0.7-0.85"),
    ("transpiration.yaml", "aerodynamic_resistance", &[30.0, 100.0], "r_a 30-100 s/m"),
    ("nitrogen.yaml", "max_uptake_capacity", &[0.00025], "~6x the fastest reported accumulation"),
    ("senescence.yaml", "rdr_leaf", &[0.012], "the cited table's own late value, 0.012 at DS 1.8+"),
];

/// The partition table's four organ shares (`allocation.yaml`, TODO(cite)).
const ORGAN_ROWS: [&str; 4] = ["fl", "fs", "fr", "fo"];

/// The three gated readouts whose bound is one plain number: (spec index, bound, kind).
/// `upper` means the value must stay BELOW the bound.
fn numeric_bound(scenario: &str, quantity: &str) -> Option<(f64, bool)> {
    match (scenario, quantity) {
        ("open_season", q) if q.starts_with("peak LAI") => Some((8.0, true)),
        ("open_season", q) if q.starts_with("peak W") => Some((14.4248, true)),
        ("perennial_long_horizon", q) if q.starts_with("converged peak-leaf") => Some((0.55, false)),
        _ => None,
    }
}

fn rel(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        if a == 0.0 { 0.0 } else { f64::INFINITY }
    } else {
        (a - b) / b.abs()
    }
}

/// The four frozen season runs the final-stock column reads.
fn seasons() -> Vec<(&'static str, SeasonScenario, usize, bool)> {
    vec![
        ("open_season", DEFAULT_SCENARIO, 1, false),
        ("sealed_chamber", sealed_chamber_scenario(), SEALED_CHAMBER_YEARS, false),
        ("perennial_chamber", perennial_chamber_scenario(), PERENNIAL_CHAMBER_YEARS, true),
        ("consumer_chamber", consumer_chamber_scenario(), CONSUMER_CHAMBER_YEARS, true),
    ]
}

/// Final non-boundary stocks of every season run, keyed `scenario/stock`; `Err` on a run
/// that did not finish (printed, never hidden).
fn finals(p: &BiosphereParams) -> Result<BTreeMap<String, f64>, String> {
    let mut out = BTreeMap::new();
    for (name, scenario, years, perennial) in seasons() {
        let (state, integrator, resolver) =
            season_setup_with(&scenario, years, p).map_err(|e| format!("{name}: {e:?}"))?;
        let steps = steps_for_years(years);
        let mut noop = |_: &State| {};
        let run = if perennial {
            run_perennial_with(
                &integrator, state, &scenario, p, &resolver, BIO_DT, steps, season_steps(),
                &mut noop,
            )
        } else {
            run_season(&integrator, state, &resolver, BIO_DT, steps, None, &mut noop)
        };
        let (last, rationed, _events) = run.map_err(|e| format!("{name}: {e:?}"))?;
        if rationed != 0 {
            return Err(format!("{name}: rationed {rationed}"));
        }
        for (id, s) in &last.stocks {
            if s.kind != StockKind::Boundary {
                out.insert(format!("{name}/{id}"), s.amount);
            }
        }
    }
    Ok(out)
}

/// One measured variant: the gated column and the final stocks.
struct Measured {
    label: String,
    gated: Column,
    finals: Result<BTreeMap<String, f64>, String>,
}

fn measured(label: String, p: &BiosphereParams) -> Measured {
    Measured { gated: measure(&label, p, true), finals: finals(p), label }
}

fn print_variant(base: &Measured, v: &Measured) {
    println!("  {}", v.label);
    for (i, b) in &base.gated.values {
        let spec = &SPECS[*i];
        let Some((_, x)) = v.gated.values.iter().find(|(j, _)| j == i) else {
            continue;
        };
        let mut line = format!(
            "    {:<24} {:<46} {:>12.6} {:+9.3} %",
            spec.scenario,
            spec.quantity,
            x,
            100.0 * rel(*x, *b)
        );
        if let Some((bound, upper)) = numeric_bound(spec.scenario, spec.quantity) {
            let (m0, m1) = if upper { (bound - b, bound - x) } else { (b - bound, x - bound) };
            line.push_str(&format!("   margin used {:+.2} %", 100.0 * (1.0 - m1 / m0)));
        }
        println!("{line}");
    }
    for (i, why) in v.gated.failed.iter().chain(v.gated.not_applicable.iter()) {
        println!("    {:<24} {:<46} NOT MEASURED: {why}", SPECS[*i].scenario, SPECS[*i].quantity);
    }
    match (&base.finals, &v.finals) {
        (Ok(b), Ok(x)) => {
            let mut moves: Vec<(String, f64)> = b
                .iter()
                .filter_map(|(k, &bv)| x.get(k).map(|&xv| (k.clone(), rel(xv, bv))))
                .collect();
            moves.sort_by(|p, q| q.1.abs().total_cmp(&p.1.abs()).then_with(|| p.0.cmp(&q.0)));
            let top: Vec<String> = moves
                .iter()
                .take(4)
                .map(|(k, m)| format!("{k} {:+.3} %", 100.0 * m))
                .collect();
            println!("    final stocks, largest moves: {}", top.join(", "));
        }
        (_, Err(e)) => println!("    final stocks: RUN FAILED — {e}"),
        (Err(e), _) => println!("    final stocks: BASELINE FAILED — {e}"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // `--recorded` runs only the recorded-range section (`RECORDED`).
    let recorded_only = args.iter().any(|a| a == "--recorded");
    let only: Vec<String> = args.into_iter().filter(|a| a != "--recorded").collect();
    let wanted = |name: &str| only.is_empty() || only.iter().any(|o| o == name);
    let frozen = biosphere_with(&[]).expect("frozen params");
    let base = measured("frozen".to_string(), &frozen);
    println!("biosphere uncited-value triage — MEASURED QUANTITIES, not verdicts.");
    println!("Moves are relative to the frozen run. 15-year rows included.\n");

    for row in ROWS.iter().filter(|r| !recorded_only && wanted(r.field)) {
        // ⚠ The control: the substitution at the typed frozen value must BE the frozen params.
        let at_frozen = biosphere_with(&[Substitution::new(row.file, row.field, row.frozen)])
            .unwrap_or_else(|e| panic!("{}:{} = {}: {e}", row.file, row.field, row.frozen));
        assert_eq!(
            format!("{at_frozen:?}"),
            format!("{frozen:?}"),
            "{}:{}'s typed frozen value {} is not the committed one",
            row.file,
            row.field,
            row.frozen
        );
        println!("{}:{} = {}  [{}]", row.file, row.field, row.frozen, row.class);
        let values = match row.step {
            Step::Relative => [row.frozen * 0.9, row.frozen * 1.1],
            Step::Celsius => [row.frozen - 1.0, row.frozen + 1.0],
        };
        for v in values {
            let label = format!("{}={v}", row.field);
            match biosphere_with(&[Substitution::new(row.file, row.field, v)]) {
                Ok(p) => print_variant(&base, &measured(label, &p)),
                Err(e) => println!("  {label}: REFUSED by the loader — {e}"),
            }
        }
        println!();
    }
    for organ in ORGAN_ROWS.iter().filter(|o| !recorded_only && wanted(o)) {
        println!("allocation.yaml:partition_table {organ} (every knot, others compensated)  [TODO(cite)]");
        for f in [0.9, 1.1] {
            let label = format!("{organ}×{f}");
            match biosphere_with_share(organ, Knot::Every, f) {
                Ok(p) => print_variant(&base, &measured(label, &p)),
                Err(e) => println!("  {label}: REFUSED — {e}"),
            }
        }
        println!();
    }
    for (file, field, values, record) in RECORDED.iter().filter(|r| wanted(r.1)) {
        println!("{file}:{field} AT ITS RECORDED RANGE ({record})");
        for &v in values.iter() {
            let label = format!("{field}={v}");
            match biosphere_with(&[Substitution::new(file, field, v)]) {
                Ok(p) => print_variant(&base, &measured(label, &p)),
                Err(e) => println!("  {label}: REFUSED by the loader — {e}"),
            }
        }
        println!();
    }
}
