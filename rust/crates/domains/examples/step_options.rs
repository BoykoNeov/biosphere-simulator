//! Three ways to take the biosphere's step, scored against a fine-step answer — the evidence
//! behind `docs/plans/post-roadmap-step-options.md` §6, as one command:
//!
//! ```text
//! cargo run --release -q -p domains --example step_options
//! ```
//!
//! It runs the §3 controls first and panics on any that fails, so a table is only ever printed
//! over runs that passed them. It writes nothing and takes no decision.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use domains::biosphere::params::BiosphereParams;
use domains::biosphere::science::{self, LeafAreaForm};
use domains::biosphere::stocks::{CARBON_POOL, LEAF_AREA_INDEX, LEAF_C, STORAGE_C};
use domains::biosphere::system::{
    build_season_with, consumer_chamber_scenario, perennial_chamber_scenario,
    sealed_chamber_scenario, SeasonScenario, CONSUMER_CHAMBER_YEARS, DEFAULT_SCENARIO,
    LONG_HORIZON_YEARS, PERENNIAL_CHAMBER_YEARS, SEALED_CHAMBER_YEARS,
};
use domains::biosphere::{SeasonBuild, SEASON_DAYS};
use domains::lab::biosphere_with_leaf_form;
use domains::lab::step_options::{
    build_season_implicit_uptake, run_split, run_uniform, Scope, SolveLog, SplitRule, SplitStats,
    Watched,
};
use simcore::error::SimError;
use simcore::registry::Registry;
use simcore::state::State;

/// The fine-step answer's steps per day (1/256 day ≈ 5.6 minutes).
const TRUTH_SPD: usize = 256;
/// B's depth limit: 6 halvings of a quarter day is 1/256 day, the fine-step answer's step.
const MAX_DEPTH: u32 = 6;

fn params(form: LeafAreaForm) -> BiosphereParams {
    biosphere_with_leaf_form(&[], form).expect("the frozen params load")
}

/// Every stock amount and aux value, bit for bit, in key order.
fn fingerprint(s: &State) -> Vec<u64> {
    let mut out: Vec<u64> = s.stocks.values().map(|st| st.amount.to_bits()).collect();
    out.extend(s.aux.values().map(|v| v.to_bits()));
    out
}

/// What a run looked like on the quarter-day grid every option shares.
#[derive(Default)]
struct Series {
    /// Chamber CO₂, ppm, at every quarter-day boundary (including the start).
    ppm: Vec<f64>,
    /// Storage carbon at each season's end (before any re-sow).
    yields: Vec<f64>,
    /// Peak leaf area index over the run.
    peak_lai: f64,
    /// Fingerprints at every quarter-day boundary (only when asked for).
    prints: Vec<Vec<u64>>,
}

/// An observer sampling `series` on the quarter-day grid of a run at `spd` steps per day.
fn sampler<'a>(
    series: &'a mut Series,
    scenario: &'a SeasonScenario,
    p: &'a BiosphereParams,
    spd: usize,
    keep_prints: bool,
) -> impl FnMut(&State) + 'a {
    let per_quarter = (spd / 4) as u64;
    let season = (SEASON_DAYS * spd) as u64;
    move |s: &State| {
        let lai = match s.aux.get(LEAF_AREA_INDEX) {
            Some(v) => *v,
            None => science::leaf_area_index(
                s.stocks[LEAF_C].amount,
                p.canopy.sla_per_mol_c,
                scenario.ground_area,
            ),
        };
        series.peak_lai = series.peak_lai.max(lai);
        if s.n.is_multiple_of(per_quarter) {
            if let Some(pool) = s.stocks.get(CARBON_POOL) {
                series
                    .ppm
                    .push(pool.amount / scenario.chamber_air_capacity_mol * 1.0e6);
            }
            if keep_prints {
                series.prints.push(fingerprint(s));
            }
        }
        if s.n > 0 && s.n.is_multiple_of(season) {
            series.yields.push(s.stocks[STORAGE_C].amount);
        }
    }
}

struct Run {
    label: String,
    series: Series,
    rationed: u64,
    /// Full flow-set evaluations per simulated day.
    evals_per_day: f64,
    /// Extra allocation-only evaluations per step (C's solve).
    alloc_evals_per_step: f64,
    secs: f64,
    end: State,
}

fn days(years: usize) -> f64 {
    (SEASON_DAYS * years) as f64
}

#[allow(clippy::too_many_arguments)]
fn uniform(
    label: &str,
    s: &SeasonScenario,
    years: usize,
    perennial: bool,
    p: &BiosphereParams,
    build: SeasonBuild<'_>,
    spd: usize,
    keep_prints: bool,
) -> Run {
    let mut series = Series::default();
    let t0 = Instant::now();
    let (end, rationed) = {
        let mut obs = sampler(&mut series, s, p, spd, keep_prints);
        run_uniform(s, years, perennial, p, build, spd, &mut obs).expect("uniform run")
    };
    Run {
        label: label.to_string(),
        series,
        rationed,
        evals_per_day: spd as f64,
        alloc_evals_per_step: 0.0,
        secs: t0.elapsed().as_secs_f64(),
        end,
    }
}

#[allow(clippy::too_many_arguments)]
fn split(
    label: &str,
    s: &SeasonScenario,
    years: usize,
    perennial: bool,
    p: &BiosphereParams,
    rule: SplitRule,
    keep_prints: bool,
) -> (Run, SplitStats) {
    let mut series = Series::default();
    let t0 = Instant::now();
    let (end, stats) = {
        let mut obs = sampler(&mut series, s, p, 4, keep_prints);
        run_split(s, years, perennial, p, &build_season_with, 4, rule, &mut obs).expect("split run")
    };
    let run = Run {
        label: label.to_string(),
        series,
        rationed: stats.rationed,
        evals_per_day: (stats.leaf_steps + stats.probes) as f64 / days(years),
        alloc_evals_per_step: 0.0,
        secs: t0.elapsed().as_secs_f64(),
        end,
    };
    (run, stats)
}

#[allow(clippy::too_many_arguments)]
fn implicit(
    label: &str,
    s: &SeasonScenario,
    years: usize,
    perennial: bool,
    p: &BiosphereParams,
    spd: usize,
    scope: Scope,
    checked: bool,
) -> (Run, SolveLog) {
    let log = Rc::new(RefCell::new(SolveLog::default()));
    let build_log = Rc::clone(&log);
    let build = move |sc: &SeasonScenario, pp: &BiosphereParams| -> Result<(State, Registry), SimError> {
        build_season_implicit_uptake(sc, pp, scope, checked, Rc::clone(&build_log))
    };
    let mut run = uniform(label, s, years, perennial, p, &build, spd, false);
    let log = log.borrow().clone();
    run.alloc_evals_per_step = log.evaluations as f64 / (SEASON_DAYS * years * spd) as f64;
    (run, log)
}

/// `(max |Δ|, mean |Δ|, mean signed Δ)` of two ppm series on the shared grid.
fn ppm_err(a: &[f64], truth: &[f64]) -> (f64, f64, f64) {
    assert_eq!(a.len(), truth.len(), "the quarter-day grids differ in length");
    let (mut max, mut sum, mut signed) = (0.0f64, 0.0, 0.0);
    for (x, t) in a.iter().zip(truth) {
        let d = x - t;
        max = max.max(d.abs());
        sum += d.abs();
        signed += d;
    }
    let n = a.len() as f64;
    (max, sum / n, signed / n)
}

/// The season's relative yield error of largest magnitude, SIGNED (+ = above the truth).
fn yield_err(a: &[f64], truth: &[f64]) -> f64 {
    assert_eq!(a.len(), truth.len());
    a.iter()
        .zip(truth)
        .map(|(x, t)| (x - t) / t)
        .fold(0.0, |m: f64, e: f64| if e.abs() > m.abs() { e } else { m })
}

fn lai_err(a: f64, truth: f64) -> f64 {
    (a - truth) / truth
}

fn print_score(runs: &[&Run], truth: &Run) {
    println!(
        "  {:<26} {:>9} {:>9} {:>9} {:>8} {:>8} {:>9} {:>9} {:>6} {:>7}",
        "option", "maxΔppm", "meanΔppm", "biasppm", "yield%", "LAI%", "evals/d", "alloc/st", "rat.", "secs"
    );
    for r in runs {
        let (max, mean, bias) = ppm_err(&r.series.ppm, &truth.series.ppm);
        println!(
            "  {:<26} {:>9.3} {:>9.4} {:>+9.4} {:>+8.3} {:>+8.3} {:>9.2} {:>9.2} {:>6} {:>7.2}",
            r.label,
            max,
            mean,
            bias,
            100.0 * yield_err(&r.series.yields, &truth.series.yields),
            100.0 * lai_err(r.series.peak_lai, truth.series.peak_lai),
            r.evals_per_day,
            r.alloc_evals_per_step,
            r.rationed,
            r.secs,
        );
    }
}

fn main() {
    let jar = sealed_chamber_scenario();
    let jar_years = SEALED_CHAMBER_YEARS;
    let frozen = params(LeafAreaForm::Derived);
    let leafy = params(LeafAreaForm::NodeEnvelope);

    // ------------------------------------------------------------------------------- //
    // The diagnostic the first run forced: is the quarter-day error about the chamber  //
    // pool at all? The open field has no pool, so none of it can be.                   //
    // ------------------------------------------------------------------------------- //
    println!("== diagnostic: the open field (no chamber pool), plain Euler against 1/{TRUTH_SPD} ==");
    let open_runs: Vec<Run> = [4usize, 8, 16, 32, 64, 128, TRUTH_SPD]
        .iter()
        .map(|&spd| uniform(&format!("euler 1/{spd}"), &DEFAULT_SCENARIO, 1, false, &frozen, &build_season_with, spd, false))
        .collect();
    let open_truth = open_runs.last().expect("truth");
    println!("  {:<14} {:>10} {:>10} {:>6}", "step", "yield%", "LAI%", "rat.");
    for r in &open_runs {
        println!(
            "  {:<14} {:>+10.5} {:>+10.5} {:>6}",
            r.label,
            100.0 * yield_err(&r.series.yields, &open_truth.series.yields),
            100.0 * lai_err(r.series.peak_lai, open_truth.series.peak_lai),
            r.rationed
        );
    }
    println!(
        "  truth: yield {:?} mol C, peak LAI {:.4}",
        open_truth.series.yields.iter().map(|y| format!("{y:.4}")).collect::<Vec<_>>(),
        open_truth.series.peak_lai
    );
    if std::env::args().any(|a| a == "--open-field-only") {
        return;
    }

    // ------------------------------------------------------------------------------- //
    // Controls 2 and 3: B never splitting is the reference; B always splitting is the  //
    // finer uniform run, at every coarse step — on the jar AND a re-sowing chamber.     //
    // ------------------------------------------------------------------------------- //
    println!("== controls 2 + 3: the split driver against uniform runs ==");
    let perennial = perennial_chamber_scenario();
    for (name, s, years, per) in [
        ("sealed_chamber", jar, jar_years, false),
        ("perennial_chamber", perennial, PERENNIAL_CHAMBER_YEARS, true),
    ] {
        let reference = uniform("euler 1/4", &s, years, per, &frozen, &build_season_with, 4, true);
        let never = SplitRule { theta: f64::INFINITY, watched: Watched::Every, max_depth: MAX_DEPTH };
        let (b_inf, st) = split("B never", &s, years, per, &frozen, never, true);
        assert_eq!(st.probes, 0, "θ = ∞ must not probe");
        assert_eq!(
            fingerprint(&b_inf.end),
            fingerprint(&reference.end),
            "{name}: B at θ = ∞ is not the reference run"
        );
        assert_eq!(b_inf.series.prints, reference.series.prints, "{name}: B at θ = ∞ drifts mid-run");
        println!("  {name}: B at θ = ∞ is bit-identical to Euler 1/4 at all {} quarter-days", reference.series.prints.len());
        for depth in [1u32, 2] {
            let spd = 4usize << depth;
            let fine = uniform("euler fine", &s, years, per, &frozen, &build_season_with, spd, true);
            let always = SplitRule { theta: -1.0, watched: Watched::Every, max_depth: depth };
            let (b, st) = split("B always", &s, years, per, &frozen, always, true);
            assert_eq!(st.split_steps, st.coarse_steps, "a forced split must split every step");
            assert_eq!(
                b.series.prints, fine.series.prints,
                "{name}: B forced {depth} deep is not Euler at 1/{spd} — the counter renumbering is wrong"
            );
            assert_eq!(fingerprint(&b.end), fingerprint(&fine.end));
            println!(
                "  {name}: B forced {depth} deep is bit-identical to Euler 1/{spd} at all {} quarter-days",
                fine.series.prints.len()
            );
        }
    }

    // ------------------------------------------------------------------------------- //
    // Control 4: C on the open field is the reference run.                             //
    // ------------------------------------------------------------------------------- //
    println!("\n== control 4: C on the open field ==");
    let open = uniform("euler 1/4", &DEFAULT_SCENARIO, 1, false, &frozen, &build_season_with, 4, false);
    let (open_c, log) = implicit("C", &DEFAULT_SCENARIO, 1, false, &frozen, 4, Scope::WithInflows, true);
    assert_eq!(log.solves, 0, "the open field has no pool to solve against");
    assert_eq!(fingerprint(&open_c.end), fingerprint(&open.end), "C moved the open field");
    println!("  open field: C is bit-identical to Euler 1/4 (0 solves)");

    // ------------------------------------------------------------------------------- //
    // The two jar cases: convergence (control 1), then the scorecard.                  //
    // ------------------------------------------------------------------------------- //
    for (case, p) in [("sealed jar, frozen science", &frozen), ("sealed jar, lab leaf form", &leafy)] {
        println!("\n==================== {case} ({jar_years} seasons, no re-sow) ====================");
        let mut euler: Vec<Run> = Vec::new();
        for spd in [4usize, 8, 16, 32, 64, 128, TRUTH_SPD] {
            euler.push(uniform(&format!("euler 1/{spd}"), &jar, jar_years, false, p, &build_season_with, spd, false));
        }
        println!("\n-- control 1: does plain Euler converge? (difference to the next step size down) --");
        println!("  {:<16} {:>10} {:>8} {:>10} {:>8} {:>10} {:>8} {:>6}", "step", "maxΔppm", "ratio", "yield%", "ratio", "LAI%", "ratio", "rat.");
        let mut prev: Option<(f64, f64, f64)> = None;
        for w in euler.windows(2) {
            let (a, b) = (&w[0], &w[1]);
            let d = (
                ppm_err(&a.series.ppm, &b.series.ppm).0,
                (100.0 * yield_err(&a.series.yields, &b.series.yields)).abs(),
                (100.0 * lai_err(a.series.peak_lai, b.series.peak_lai)).abs(),
            );
            let ratio = |x: f64, y: f64| if y > 0.0 { x / y } else { f64::NAN };
            let (r1, r2, r3) = match prev {
                Some(q) => (ratio(q.0, d.0), ratio(q.1, d.1), ratio(q.2, d.2)),
                None => (f64::NAN, f64::NAN, f64::NAN),
            };
            println!(
                "  {:<16} {:>10.4} {:>8.2} {:>10.5} {:>8.2} {:>10.5} {:>8.2} {:>6}",
                format!("{} vs {}", a.label.trim_start_matches("euler "), b.label.trim_start_matches("euler ")),
                d.0, r1, d.1, r2, d.2, r3, a.rationed
            );
            prev = Some(d);
        }
        let truth = euler.last().expect("truth");
        assert_eq!(truth.rationed, 0, "the fine-step answer rationed");
        println!(
            "  truth (1/{TRUTH_SPD}): rationed {}, CO₂ {:.1}–{:.1} ppm, yields {:?}, peak LAI {:.4}",
            truth.rationed,
            truth.series.ppm.iter().cloned().fold(f64::INFINITY, f64::min),
            truth.series.ppm.iter().cloned().fold(0.0, f64::max),
            truth.series.yields.iter().map(|y| format!("{y:.3}")).collect::<Vec<_>>(),
            truth.series.peak_lai
        );

        // B.
        let mut b_runs: Vec<(Run, SplitStats)> = Vec::new();
        for (label, watched, theta) in [
            ("B co2 θ=0.5", Watched::ChamberCo2, 0.5),
            ("B co2 θ=0.25", Watched::ChamberCo2, 0.25),
            ("B co2 θ=0.1", Watched::ChamberCo2, 0.1),
            ("B every θ=0.25", Watched::Every, 0.25),
        ] {
            let rule = SplitRule { theta, watched, max_depth: MAX_DEPTH };
            b_runs.push(split(label, &jar, jar_years, false, p, rule, false));
        }

        // C, checked (controls 6 + 7) and unchecked (timing) — identical by construction, asserted.
        let (c4, log4) = implicit("C 1/4", &jar, jar_years, false, p, 4, Scope::CropOnly, true);
        let (c4u, _) = implicit("C 1/4", &jar, jar_years, false, p, 4, Scope::CropOnly, false);
        assert_eq!(fingerprint(&c4.end), fingerprint(&c4u.end), "the scope check moved a number");
        let (c8, log8) = implicit("C 1/8", &jar, jar_years, false, p, 8, Scope::CropOnly, true);
        let (c64, log64) = implicit("C 1/64", &jar, jar_years, false, p, 64, Scope::CropOnly, false);
        let (r4, logr4) = implicit("C+returns 1/4", &jar, jar_years, false, p, 4, Scope::WithInflows, true);
        let (r8, logr8) = implicit("C+returns 1/8", &jar, jar_years, false, p, 8, Scope::WithInflows, true);
        let (r64, logr64) =
            implicit("C+returns 1/64", &jar, jar_years, false, p, 64, Scope::WithInflows, false);

        println!("\n-- controls 6 + 7: C's solve and scope --");
        for (l, log) in [
            ("C 1/4", &log4),
            ("C 1/8", &log8),
            ("C 1/64", &log64),
            ("C+returns 1/4", &logr4),
            ("C+returns 1/8", &logr8),
            ("C+returns 1/64", &logr64),
        ] {
            println!(
                "  {l}: {} solves ({} idle, {} beyond the start pool), max residual {:.3e} mol, {:.1} allocation evals per solve, {} scope checks, differing: {:?}",
                log.solves,
                log.idle,
                log.beyond_start,
                log.max_residual,
                log.evaluations as f64 / log.solves.max(1) as f64,
                log.scope_checks,
                log.differing
            );
        }

        // Control 5: C and Euler share one limit.
        let e64 = &euler[4];
        let at_quarter = ppm_err(&c4u.series.ppm, &euler[0].series.ppm).0;
        let at_fine = ppm_err(&c64.series.ppm, &e64.series.ppm).0;
        println!(
            "\n-- control 5: C and Euler share a limit: max |C − Euler| = {at_quarter:.4} ppm at 1/4, {at_fine:.4} ppm at 1/64 --"
        );
        assert!(at_fine < 0.25 * at_quarter, "C and Euler do not approach one answer");
        let r_quarter = ppm_err(&r4.series.ppm, &euler[0].series.ppm).0;
        let r_fine = ppm_err(&r64.series.ppm, &e64.series.ppm).0;
        println!(
            "-- control 5, C+returns: max |C+returns − Euler| = {r_quarter:.4} ppm at 1/4, {r_fine:.4} ppm at 1/64 --"
        );
        assert!(r_fine < 0.25 * r_quarter, "C+returns and Euler do not approach one answer");

        println!("\n-- the scorecard, against Euler 1/{TRUTH_SPD} (quarter-day grid) --");
        let mut shown: Vec<&Run> = vec![&euler[0], &euler[1], &euler[2]];
        for (r, _) in &b_runs {
            shown.push(r);
        }
        shown.extend([&c4u, &c8, &c64, &r4, &r8, &r64, &euler[4]]);
        print_score(&shown, truth);
        println!("\n  B's splitting:");
        for (r, st) in &b_runs {
            println!(
                "    {:<18} split {:>5} of {} coarse steps, deepest {}, bottomed out {}, probes {}, steps {}",
                r.label, st.split_steps, st.coarse_steps, st.deepest, st.bottomed, st.probes, st.leaf_steps
            );
        }
    }

    // ------------------------------------------------------------------------------- //
    // Which frozen results would move (frozen science, 1/4 day).                       //
    // ------------------------------------------------------------------------------- //
    println!("\n==================== would a frozen result move? (frozen science) ====================");
    let consumer = consumer_chamber_scenario();
    let roster: Vec<(&str, SeasonScenario, usize, bool)> = vec![
        ("open_season", DEFAULT_SCENARIO, 1, false),
        ("sealed_chamber", jar, jar_years, false),
        ("perennial_chamber", perennial, PERENNIAL_CHAMBER_YEARS, true),
        ("consumer_chamber", consumer, CONSUMER_CHAMBER_YEARS, true),
        ("perennial_long_horizon", perennial, LONG_HORIZON_YEARS, true),
        ("consumer_long_horizon", consumer, LONG_HORIZON_YEARS, true),
    ];
    println!(
        "  {:<24} {:>14} {:>14} {:>14} {:>16} {:>18} {:>18}",
        "run", "B co2 0.5", "B co2 0.25", "B co2 0.1", "B every 0.25", "C moves?", "C+ret moves?"
    );
    for (name, s, years, per) in roster {
        let reference = uniform("ref", &s, years, per, &frozen, &build_season_with, 4, false);
        let mut cells: Vec<String> = Vec::new();
        for (watched, theta) in [
            (Watched::ChamberCo2, 0.5),
            (Watched::ChamberCo2, 0.25),
            (Watched::ChamberCo2, 0.1),
            (Watched::Every, 0.25),
        ] {
            let rule = SplitRule { theta, watched, max_depth: MAX_DEPTH };
            let (r, st) = split("B", &s, years, per, &frozen, rule, false);
            let moved = fingerprint(&r.end) != fingerprint(&reference.end);
            assert_eq!(moved, st.split_steps > 0, "{name}: a split run's movement disagrees with its split count");
            cells.push(format!("{} splits", st.split_steps));
        }
        for scope in [Scope::CropOnly, Scope::WithInflows] {
            let (c, log) = implicit("C", &s, years, per, &frozen, 4, scope, false);
            let differing: usize = fingerprint(&c.end)
                .iter()
                .zip(fingerprint(&reference.end))
                .filter(|(a, b)| **a != *b)
                .count();
            cells.push(if log.solves == 0 {
                "no pool".to_string()
            } else {
                format!("{differing} values, {} rat.", c.rationed)
            });
        }
        println!(
            "  {:<24} {:>14} {:>14} {:>14} {:>16} {:>18} {:>18}",
            name, cells[0], cells[1], cells[2], cells[3], cells[4], cells[5]
        );
    }
}
