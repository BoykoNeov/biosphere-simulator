//! Why the quarter-day step loses the jar's harvest: the step size and the light's time
//! resolution, separated — the evidence behind `docs/plans/post-roadmap-step-cause.md` §5, as
//! one command:
//!
//! ```text
//! cargo run --release -q -p domains --example step_cause
//! ```
//!
//! It runs the §3 controls first and panics on any that fails, so the four-cell table is only
//! ever printed over runs that passed them. It writes nothing and takes no decision.

use std::time::Instant;

use domains::biosphere::params::BiosphereParams;
use domains::biosphere::science::{self, LeafAreaForm};
use domains::biosphere::stocks::{CARBON_POOL, LEAF_C, ROOT_C, STEM_C, STORAGE_C};
use domains::biosphere::system::{
    build_season_with, sealed_chamber_scenario, SeasonScenario, DEFAULT_SCENARIO,
    SEALED_CHAMBER_YEARS,
};
use domains::biosphere::SEASON_DAYS;
use domains::lab::biosphere_with_leaf_form;
use domains::lab::step_cause::{
    build_season_light_quadrature, dose_partition_worst, par_readers, run_uniform_blocked_light,
    BUDGET_FLOWS,
};
use domains::lab::step_options::run_uniform;
use simcore::error::SimError;
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::State;

/// The fine-step answer's steps per day (1/256 day ≈ 5.6 minutes), as in the step options.
const TRUTH_SPD: usize = 256;

/// Every stock amount and aux value, bit for bit, in key order.
fn fingerprint(s: &State) -> Vec<u64> {
    let mut out: Vec<u64> = s.stocks.values().map(|st| st.amount.to_bits()).collect();
    out.extend(s.aux.values().map(|v| v.to_bits()));
    out
}

/// What a run looked like on the quarter-day grid every cell shares.
#[derive(Default)]
struct Series {
    /// Chamber CO₂, ppm, at every quarter-day boundary (empty without a pool).
    ppm: Vec<f64>,
    /// Storage carbon at each season's end.
    yields: Vec<f64>,
    /// Peak leaf area index over the run.
    peak_lai: f64,
    /// Leaf + stem + root carbon at the step of peak leaf area.
    veg_at_peak: f64,
    /// Fingerprints at every quarter-day boundary.
    prints: Vec<Vec<u64>>,
}

fn sampler<'a>(
    series: &'a mut Series,
    scenario: &'a SeasonScenario,
    p: &'a BiosphereParams,
    spd: usize,
) -> impl FnMut(&State) + 'a {
    let per_quarter = (spd / 4) as u64;
    let season = (SEASON_DAYS * spd) as u64;
    move |s: &State| {
        let lai = science::leaf_area_index(
            s.stocks[LEAF_C].amount,
            p.canopy.sla_per_mol_c,
            scenario.ground_area,
        );
        if lai > series.peak_lai {
            series.peak_lai = lai;
            series.veg_at_peak =
                s.stocks[LEAF_C].amount + s.stocks[STEM_C].amount + s.stocks[ROOT_C].amount;
        }
        if s.n.is_multiple_of(per_quarter) {
            if let Some(pool) = s.stocks.get(CARBON_POOL) {
                series.ppm.push(pool.amount / scenario.chamber_air_capacity_mol * 1.0e6);
            }
            series.prints.push(fingerprint(s));
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
    /// Budget-flow evaluations per step (3 plain; 3·k under E2).
    budget_evals_per_step: u64,
    steps_per_day: usize,
    secs: f64,
    end: State,
}

/// Plain Euler (`light_spd = None`) or E1 (`Some(block)`), at `spd`.
fn euler(
    label: &str,
    s: &SeasonScenario,
    years: usize,
    p: &BiosphereParams,
    spd: usize,
    light_spd: Option<usize>,
) -> Run {
    let mut series = Series::default();
    let t0 = Instant::now();
    let (end, rationed) = {
        let mut obs = sampler(&mut series, s, p, spd);
        match light_spd {
            None => run_uniform(s, years, false, p, &build_season_with, spd, &mut obs),
            Some(block) => run_uniform_blocked_light(
                s, years, false, p, &build_season_with, spd, block, &mut obs,
            ),
        }
        .expect("run")
    };
    Run {
        label: label.to_string(),
        series,
        rationed,
        budget_evals_per_step: 3,
        steps_per_day: spd,
        secs: t0.elapsed().as_secs_f64(),
        end,
    }
}

/// E2: the quarter-day step with the budget flows' light integrated over `k` sub-windows.
fn quadrature(label: &str, s: &SeasonScenario, years: usize, p: &BiosphereParams, k: u64) -> Run {
    let build = move |sc: &SeasonScenario, pp: &BiosphereParams| -> Result<(State, Registry), SimError> {
        build_season_light_quadrature(sc, pp, years, k)
    };
    let mut series = Series::default();
    let t0 = Instant::now();
    let (end, rationed) = {
        let mut obs = sampler(&mut series, s, p, 4);
        run_uniform(s, years, false, p, &build, 4, &mut obs).expect("quadrature run")
    };
    Run {
        label: label.to_string(),
        series,
        rationed,
        budget_evals_per_step: 3 * k,
        steps_per_day: 4,
        secs: t0.elapsed().as_secs_f64(),
        end,
    }
}

fn rel(x: f64, t: f64) -> f64 {
    100.0 * (x - t) / t
}

/// Final-harvest error, %, signed.
fn harvest_pct(r: &Run, truth: &Run) -> f64 {
    rel(*r.series.yields.last().unwrap(), *truth.series.yields.last().unwrap())
}

fn ppm_err(a: &[f64], truth: &[f64]) -> (f64, f64, f64) {
    if a.is_empty() {
        return (f64::NAN, f64::NAN, f64::NAN);
    }
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

fn print_table(runs: &[&Run], truth: &Run) {
    println!(
        "  {:<30} {:>8} {:>8} {:>8} {:>9} {:>9} {:>9} {:>8} {:>9} {:>5} {:>7}",
        "cell", "harv%", "d305%", "LAI%", "veg@pk%", "maxΔppm", "meanΔppm", "steps/d", "budget/st", "rat.", "secs"
    );
    for r in runs {
        let (max, mean, _) = ppm_err(&r.series.ppm, &truth.series.ppm);
        println!(
            "  {:<30} {:>+8.3} {:>+8.3} {:>+8.3} {:>+9.3} {:>9.3} {:>9.4} {:>8} {:>9} {:>5} {:>7.2}",
            r.label,
            harvest_pct(r, truth),
            rel(r.series.yields[0], truth.series.yields[0]),
            rel(r.series.peak_lai, truth.series.peak_lai),
            rel(r.series.veg_at_peak, truth.series.veg_at_peak),
            max,
            mean,
            r.steps_per_day,
            r.budget_evals_per_step,
            r.rationed,
            r.secs,
        );
    }
}

fn main() {
    let frozen: BiosphereParams =
        biosphere_with_leaf_form(&[], LeafAreaForm::Derived).expect("the frozen params load");
    let jar = sealed_chamber_scenario();
    let cases: [(&str, SeasonScenario, usize); 2] = [
        ("open field", DEFAULT_SCENARIO, 1),
        ("sealed jar, frozen science", jar, SEALED_CHAMBER_YEARS),
    ];

    // ------------------------------------------------------------------------------- //
    // Controls 3 and 4: the sub-windows partition the dose; only the budget reads PAR. //
    // ------------------------------------------------------------------------------- //
    println!("== controls 3 + 4: dose partition and PAR readers ==");
    let want: std::collections::BTreeSet<String> = BUDGET_FLOWS.iter().map(|s| s.to_string()).collect();
    for (name, s, years) in &cases {
        for k in [4u64, 16, 64] {
            let worst = dose_partition_worst(s, *years, k).expect("dose");
            assert!(worst <= 1e-12, "{name}: k = {k} sub-windows miss the window's dose by {worst:e}");
            println!("  {name}: k = {k:>2}, worst relative dose gap {worst:.2e}");
        }
        let (readers, lit) = par_readers(s, &frozen).expect("par readers");
        assert!(lit > 0, "{name}: no lit step — the reader probe is vacuous");
        // Nothing OUTSIDE the wrapped set may read PAR. Not every wrapped flow has to: in a
        // sealed chamber growth respiration's source and sink are one pool and it returns
        // empty before reading the budget (plan §3 said "exactly those three"; the jar
        // corrected it on the first run).
        assert!(readers.is_subset(&want), "{name}: PAR is read outside the three budget flows: {readers:?}");
        assert!(readers.contains(BUDGET_FLOWS[0]), "{name}: the allocation never read PAR — the probe is blind");
        println!("  {name}: PAR read by {readers:?} over {lit} lit steps (all inside the wrapped set)");
    }

    // ------------------------------------------------------------------------------- //
    // Controls 1 and 2: E1 at a block as fine as the step, E2 at k = 1 — both the plain //
    // run, bit for bit, at every quarter-day.                                          //
    // ------------------------------------------------------------------------------- //
    println!("\n== controls 1 + 2: the instruments at their identity settings ==");
    for (name, s, years) in &cases {
        for spd in [4usize, TRUTH_SPD] {
            let plain = euler("plain", s, *years, &frozen, spd, None);
            let e1 = euler("E1 identity", s, *years, &frozen, spd, Some(spd));
            assert_eq!(e1.series.prints, plain.series.prints, "{name}: E1 with a step-sized block moved the run at 1/{spd}");
            println!("  {name}: E1 with light blocks of 1/{spd} = plain Euler 1/{spd} at all {} quarter-days", plain.series.prints.len());
            if spd == 4 {
                let e2 = quadrature("E2 k=1", s, *years, &frozen, 1);
                assert_eq!(e2.series.prints, plain.series.prints, "{name}: E2 at k = 1 moved the run");
                println!("  {name}: E2 at k = 1 = plain Euler 1/4 at all {} quarter-days", plain.series.prints.len());
            }
        }
    }

    // ------------------------------------------------------------------------------- //
    // The four cells, plus the light and quadrature sweeps and A at 1/16 for price.    //
    // ------------------------------------------------------------------------------- //
    for (name, s, years) in &cases {
        println!("\n==================== {name} ({years} season(s)), against Euler 1/{TRUTH_SPD} ====================");
        let truth = euler(&format!("answer: step 1/{TRUTH_SPD}, light 1/{TRUTH_SPD}"), s, *years, &frozen, TRUTH_SPD, None);
        assert_eq!(truth.rationed, 0, "the fine-step answer rationed");
        let shipped = euler("shipped: step 1/4, light 1/4", s, *years, &frozen, 4, None);
        let mut e1: Vec<Run> = Vec::new();
        for block in [4usize, 8, 16, 64] {
            e1.push(euler(&format!("E1: step 1/{TRUTH_SPD}, light 1/{block}"), s, *years, &frozen, TRUTH_SPD, Some(block)));
        }
        let mut e2: Vec<Run> = Vec::new();
        for k in [4u64, 16, 64] {
            e2.push(quadrature(&format!("E2: step 1/4, light 1/{}", 4 * k), s, *years, &frozen, k));
        }
        let a16 = euler("A: step 1/16, light 1/16", s, *years, &frozen, 16, None);
        let mut rows: Vec<&Run> = vec![&shipped];
        rows.extend(e1.iter());
        rows.extend(e2.iter());
        rows.push(&a16);
        rows.push(&truth);
        print_table(&rows, &truth);
        println!(
            "  answer: final harvest {:.4} mol C, day-305 storage {:.4}, peak LAI {:.4}, veg C at peak {:.4}",
            truth.series.yields.last().unwrap(),
            truth.series.yields[0],
            truth.series.peak_lai,
            truth.series.veg_at_peak
        );
        let (sh, e1h, e2h) = (harvest_pct(&shipped, &truth), harvest_pct(&e1[0], &truth), harvest_pct(&e2[2], &truth));
        println!(
            "  four cells, final harvest %: shipped {sh:+.3}, E1 (light 1/4) {e1h:+.3}, E2 (k = 64) {e2h:+.3}, answer 0 \
             → interaction shipped − E1 − E2 = {:+.3} points",
            sh - e1h - e2h
        );
        let (sl, e1l, e2l) = (
            rel(shipped.series.peak_lai, truth.series.peak_lai),
            rel(e1[0].series.peak_lai, truth.series.peak_lai),
            rel(e2[2].series.peak_lai, truth.series.peak_lai),
        );
        println!(
            "  four cells, peak LAI %: shipped {sl:+.3}, E1 {e1l:+.3}, E2 {e2l:+.3} → interaction {:+.3} points",
            sl - e1l - e2l
        );
        // Where the missing harvest sits at the end: every stock that differs from the answer.
        println!("  end-of-run stocks that differ from the answer (Δ = cell − answer):");
        println!("    {:<34} {:>14} {:>12} {:>12} {:>12}", "stock", "answer", "Δ shipped", "Δ E1 1/4", "Δ E2 k=64");
        for (id, st) in &truth.end.stocks {
            let d = |r: &Run| r.end.stocks[id].amount - st.amount;
            let (a, b, c) = (d(&shipped), d(&e1[0]), d(&e2[2]));
            if a.abs().max(b.abs()).max(c.abs()) > 1e-6 * st.amount.abs().max(1e-3) {
                println!("    {id:<34} {:>14.6} {a:>+12.6} {b:>+12.6} {c:>+12.6}", st.amount);
            }
        }
        // The water stores: does E1 (fine step) end on the answer's water and E2 (quarter-day
        // step) on the shipped run's — bit for bit, not just to the printed digits?
        let water: Vec<&String> = truth
            .end
            .stocks
            .iter()
            .filter(|(_, st)| st.quantity == Quantity::Water)
            .map(|(id, _)| id)
            .collect();
        assert!(!water.is_empty(), "no water stores to compare");
        let gap = |x: &Run, y: &Run| -> (bool, f64) {
            let mut same = true;
            let mut worst = 0.0f64;
            for id in &water {
                let (u, v) = (x.end.stocks[*id].amount, y.end.stocks[*id].amount);
                same &= u.to_bits() == v.to_bits();
                worst = worst.max((u - v).abs() / v.abs().max(1e-300));
            }
            (same, worst)
        };
        let (e1_same, e1_worst) = gap(&e1[0], &truth);
        let (e2_same, e2_worst) = gap(&e2[2], &shipped);
        println!(
            "  water stores ({}): E1 vs answer bit-identical {e1_same} (worst rel {e1_worst:.1e}); E2 vs shipped bit-identical {e2_same} (worst rel {e2_worst:.1e})",
            water.len()
        );
    }
}
