//! The parked leaf mechanism, re-measured on today's tree — the evidence behind
//! `docs/plans/post-roadmap-leaf-rust-remeasure.md` §4, as one command:
//!
//! ```text
//! cargo run --release -q -p domains --example leaf_remeasure            # Euler table
//! cargo run --release -q -p domains --example leaf_remeasure -- --rk4   # + RK4 decade
//! ```
//!
//! Every row is run twice — the frozen `LeafAreaForm::Derived` and the lab
//! `LeafAreaForm::NodeEnvelope` — each scenario driven the way its own golden drives it. It
//! writes nothing and takes no decision.
//!
//! ⚠ **`WSFL` is counted here because nothing else can see it.** The leaf drought factor fires
//! only below `FTSW = 0.40`; the two scenarios that exercised it (`n_limited`,
//! `water_biting`) were retired by C6. A zero in that column is a statement about the ROSTER'S
//! COVERAGE, not about drought.

use domains::biosphere::params::BiosphereParams;
use domains::biosphere::readouts::{
    leaf_thickness_ratio, min_compensation_ratio, peak_lai, peak_w, trajectory,
};
use domains::biosphere::science::{self, LeafAreaForm};
use domains::biosphere::stocks::{ROOTED_DEPTH, SOIL_WATER, THERMAL_TIME};
use domains::biosphere::system::{
    annual_reset_with, build_season_with, consumer_chamber_scenario, perennial_chamber_scenario,
    run_perennial_with, run_season, sealed_chamber_scenario, weather_resolver, SeasonScenario,
    CONSUMER_CHAMBER_YEARS, DEFAULT_SCENARIO, LONG_HORIZON_YEARS, PERENNIAL_CHAMBER_YEARS,
    SEALED_CHAMBER_YEARS,
};
use domains::biosphere::{season_steps, steps_for_years, BIO_DT};
use domains::lab::biosphere_with_leaf_form;
use simcore::integrator::{EulerIntegrator, Rk4Integrator};
use simcore::state::State;

/// `(name, scenario, years, perennial)` — the six runs `lab::report` reads, driven the same way.
fn runs() -> Vec<(&'static str, SeasonScenario, usize, bool)> {
    vec![
        ("open_season", DEFAULT_SCENARIO, 1, false),
        ("sealed_chamber", sealed_chamber_scenario(), SEALED_CHAMBER_YEARS, false),
        ("perennial_chamber", perennial_chamber_scenario(), PERENNIAL_CHAMBER_YEARS, true),
        ("consumer_chamber", consumer_chamber_scenario(), CONSUMER_CHAMBER_YEARS, true),
        ("perennial_long_horizon", perennial_chamber_scenario(), LONG_HORIZON_YEARS, true),
        ("consumer_long_horizon", consumer_chamber_scenario(), LONG_HORIZON_YEARS, true),
    ]
}

fn params(form: LeafAreaForm) -> BiosphereParams {
    biosphere_with_leaf_form(&[], form).expect("the frozen params load")
}

/// `(steps with WSFL < 1, of them before TLM, total steps)` — the leaf drought factor's reach.
fn wsfl_reach(scenario: &SeasonScenario, years: usize, perennial: bool, p: &BiosphereParams) -> (usize, usize, usize) {
    let (state, registry) = build_season_with(scenario, p).expect("build");
    let resolver = weather_resolver(scenario, years).expect("resolver");
    let integrator = EulerIntegrator::new(registry);
    let (mut below, mut below_pre_tlm, mut total) = (0, 0, 0);
    let mut observe = |s: &State| {
        let wsfl = science::soil_water_stress(
            s.stocks[SOIL_WATER].amount,
            s.aux[ROOTED_DEPTH],
            scenario.soil_extractable_water,
            scenario.ground_area,
            science::LEAF_WSSL,
        );
        total += 1;
        if wsfl < 1.0 {
            below += 1;
            if s.aux[THERMAL_TIME] < science::LEAF_TU_TLM {
                below_pre_tlm += 1;
            }
        }
    };
    let steps = steps_for_years(years);
    if perennial {
        run_perennial_with(&integrator, state, scenario, p, &resolver, BIO_DT, steps, season_steps(), &mut observe)
            .expect("run");
    } else {
        run_season(&integrator, state, &resolver, BIO_DT, steps, None, &mut observe).expect("run");
    }
    (below, below_pre_tlm, total)
}

/// RK4 over the run, re-sowing like `run_perennial_with`. `Ok(rationed)` or the engine's error.
fn rk4(scenario: &SeasonScenario, years: usize, perennial: bool, p: &BiosphereParams) -> Result<u64, String> {
    let (mut state, registry) = build_season_with(scenario, p).map_err(|e| e.to_string())?;
    let resolver = weather_resolver(scenario, years).map_err(|e| e.to_string())?;
    let integrator = Rk4Integrator::new(registry);
    let year = season_steps() as u64;
    let mut rationed = 0;
    for _ in 0..steps_for_years(years) {
        if perennial && state.n > 0 && state.n.is_multiple_of(year) {
            state = annual_reset_with(&state, scenario, p).map_err(|e| format!("reset: {e}"))?;
        }
        let report = integrator
            .step_report(&state, &resolver, BIO_DT)
            .map_err(|e| format!("step {}: {e}", state.n))?;
        rationed += report.rationed;
        state = report.state;
    }
    Ok(rationed)
}

fn main() {
    let with_rk4 = std::env::args().any(|a| a == "--rk4");
    let frozen = params(LeafAreaForm::Derived);
    let lab = params(LeafAreaForm::NodeEnvelope);
    println!(
        "{:<24} {:>10} {:>10} {:>8} | {:>9} {:>9} | {:>9} {:>9} | {:>6} {:>6} | {:>15} | WSFL<1 (pre-TLM) / steps",
        "run", "LAI frz", "LAI lab", "ratio", "W frz", "W lab", "CO2r frz", "CO2r lab", "rat f", "rat l", "thick min..max"
    );
    for (name, scenario, years, perennial) in runs() {
        let a = trajectory(scenario, years, perennial, &frozen);
        let b = trajectory(scenario, years, perennial, &lab);
        let (la, lb) = (peak_lai(&a), peak_lai(&b));
        let co2 = |t| if scenario.sealed { format!("{:>9.4}", min_compensation_ratio(t)) } else { format!("{:>9}", "-") };
        let thick = leaf_thickness_ratio(&b)
            .map(|(lo, hi)| format!("{lo:.4}..{hi:.4}"))
            .unwrap_or_else(|| "-".to_string());
        let (below, pre, total) = wsfl_reach(&scenario, years, perennial, &lab);
        println!(
            "{name:<24} {la:>10.4} {lb:>10.4} {:>8.4} | {:>9.4} {:>9.4} | {} {} | {:>6} {:>6} | {thick:>15} | {below} ({pre}) / {total}",
            lb / la,
            peak_w(&a),
            peak_w(&b),
            co2(&a),
            co2(&b),
            a.rationed,
            b.rationed,
        );
    }
    // Where the thinnest and thickest readings fall, and how much leaf carbon grew over the
    // step that produced them — the envelope reads leaf carbon at STEP ENTRY, so the sampled
    // ratio can leave it by one step's growth. `(step, day-of-season, ratio, leaf growth over
    // that step as a fraction)`.
    println!("\nthickness extremes under the lab form: (step, day of season, state/derived, leaf-C growth that step)");
    for (name, scenario, years, perennial) in runs().into_iter().take(4) {
        let t = trajectory(scenario, years, perennial, &lab);
        let sla = lab.canopy.sla_per_mol_c;
        let ratio = |i: usize| t.leaf_area_state[i] / science::leaf_area_index(t.leaf_c[i], sla, scenario.ground_area);
        let growth = |i: usize| if i == 0 { 0.0 } else { t.leaf_c[i] / t.leaf_c[i - 1] - 1.0 };
        let idx: Vec<usize> = (0..t.leaf_c.len()).filter(|i| t.leaf_c[*i] > 0.0).collect();
        let lo = *idx.iter().min_by(|a, b| ratio(**a).total_cmp(&ratio(**b))).expect("steps");
        let hi = *idx.iter().max_by(|a, b| ratio(**a).total_cmp(&ratio(**b))).expect("steps");
        let day = |i: usize| (i % season_steps()) as f64 * BIO_DT;
        println!(
            "{name:<24} min ({lo}, {:.2}, {:.4}, {:+.4})  max ({hi}, {:.2}, {:.4}, {:+.4})",
            day(lo), ratio(lo), growth(lo), day(hi), ratio(hi), growth(hi)
        );
    }
    if with_rk4 {
        println!("\nRK4 at dt = {BIO_DT}: rationed firings, or the engine's error");
        for (name, scenario, years, perennial) in runs() {
            println!("{name:<24} frozen {:?}", rk4(&scenario, years, perennial, &frozen));
            println!("{name:<24} lab    {:?}", rk4(&scenario, years, perennial, &lab));
        }
    }
}
