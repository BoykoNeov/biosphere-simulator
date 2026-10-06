//! **A drought with an end** — the 2026-09-29 review's Step 6, slice 1. LAB-ONLY.
//!
//! Plan: `docs/plans/post-roadmap-real-world-checks.md` §2–§4. The default wheat, one season,
//! with its watering cut to 0 over days 220–260 and back to the default after. The window lives
//! in the forcing (`perturbations::window_override` on `IRRIGATION_VAR`), the perturbation
//! harness's discipline: no new flow, no new parameter, no scenario field, no golden.
//!
//! **Why it exists.** No frozen scenario ever stresses the crop. Two test-only ones do (the
//! deep-water run and the manufactured dry run in `system.rs`), but both are chronic: the water
//! never comes back, so whether every drought factor returns to exactly 1 had never been run, and
//! the lab leaf form's own drought factor had never fired (`log/leaf-rust-remeasure.md` finding 5).
//!
//! **What it reads.** Every drought factor in the crop is the same root-zone water fraction
//! (`FTSW`, `science::soil_water_stress`) against its own threshold: growth and transpiration at
//! `wssg` (0.30), development speed through the growth factor, the lab leaf form at `LEAF_WSSL`
//! (0.40). So the test reads `FTSW` off every emitted state and compares it to the thresholds.
//!
//! ⚠ **The drought is the model's, not a measured one.** Transpiration is the full-cover
//! Penman–Monteith rate times the soil-water factor and does not read leaf area, so the soil
//! dries at a full canopy's pace whatever the canopy is. The window sits around flowering, where
//! the canopy is near its largest and that gap distorts least. A stated limit, not a fix.

use domains::biosphere::params::{self, BiosphereParams};
use domains::biosphere::perturbations::window_override;
use domains::biosphere::science::{self, LeafAreaForm};
use domains::biosphere::stocks::{IRRIGATION_VAR, ROOTED_DEPTH, SOIL_WATER, STORAGE_C, THERMAL_TIME};
use domains::biosphere::system::{run_season, SeasonScenario, DEFAULT_SCENARIO};
use domains::biosphere::{season_setup_with, steps_for_years, BIO_DT, STEPS_PER_DAY};
use domains::lab::biosphere_with_leaf_form;
use simcore::environment::SourceResolver;
use simcore::state::State;

/// The window's first day with no watering.
const CUT_FROM_DAY: usize = 220;
/// The first day watered again.
const CUT_TO_DAY: usize = 260;
/// The watering inside the window (mm/day); the hard off `Irrigation` documents.
const CUT_TO: f64 = 0.0;

const SCENARIO: SeasonScenario = DEFAULT_SCENARIO;

/// One season, every emitted state, with or without the window. Asserts the books on the way.
fn trace(p: &BiosphereParams, windowed: bool) -> Vec<State> {
    let (state, integrator, resolver) = season_setup_with(&SCENARIO, 1, p).expect("setup");
    let resolver = if windowed {
        let (mut forcings, shared) = resolver.into_parts();
        let base = forcings.remove(IRRIGATION_VAR).expect("the season waters its crop");
        let start = (CUT_FROM_DAY * STEPS_PER_DAY) as u64;
        let end = (CUT_TO_DAY * STEPS_PER_DAY) as u64;
        forcings.insert(
            IRRIGATION_VAR.to_string(),
            window_override(base, start, end, CUT_TO),
        );
        SourceResolver::new(forcings, shared).expect("rewired")
    } else {
        resolver
    };
    let mut seen = Vec::new();
    let mut observe = |s: &State| seen.push(s.clone());
    let (_, rationed, events) = run_season(
        &integrator,
        state,
        &resolver,
        BIO_DT,
        steps_for_years(1),
        None,
        &mut observe,
    )
    .expect("season");
    assert_eq!(rationed, 0, "the backstop must stay out of a drought instrument");
    assert!(events.is_empty(), "unexpected extinction: {events:?}");
    seen
}

/// `FTSW` on one state — the read every drought factor shares.
fn ftsw(s: &State) -> f64 {
    let depth = s.aux.get(ROOTED_DEPTH).copied().unwrap_or(0.0);
    let capacity = science::transpirable_capacity(
        depth,
        SCENARIO.soil_extractable_water,
        SCENARIO.ground_area,
    );
    science::fraction_transpirable(s.stocks[SOIL_WATER].amount, capacity)
}

fn dvs(s: &State) -> f64 {
    let ph = params::phenology();
    science::development_stage(s.aux[THERMAL_TIME], ph.tsum_anthesis, ph.tsum_maturity)
}

fn day(n: usize) -> usize {
    n / STEPS_PER_DAY
}

/// The first state index at or past a development stage.
fn first_at(states: &[State], stage: f64) -> usize {
    states
        .iter()
        .position(|s| dvs(s) >= stage)
        .expect("the season reaches the stage")
}

/// The indices of the states where a factor at `threshold` sits below 1.
fn stressed(states: &[State], threshold: f64) -> Vec<usize> {
    states
        .iter()
        .enumerate()
        .filter(|(_, s)| science::water_stress_factor(ftsw(s), threshold) < 1.0)
        .map(|(n, _)| n)
        .collect()
}

#[test]
fn the_control_holds_every_drought_factor_at_exactly_one() {
    let control = trace(&params::biosphere(), false);
    let lowest = control.iter().map(ftsw).fold(f64::MAX, f64::min);
    eprintln!("control: lowest FTSW {lowest:.4}");
    assert!(stressed(&control, SCENARIO.wssg).is_empty());
    assert!(stressed(&control, science::LEAF_WSSL).is_empty());
}

#[test]
fn the_windowed_drought_fires_inside_its_window_and_never_before_it() {
    let dry = trace(&params::biosphere(), true);
    let growth = stressed(&dry, SCENARIO.wssg);
    let leaf = stressed(&dry, science::LEAF_WSSL);
    let lowest = dry.iter().map(ftsw).fold(f64::MAX, f64::min);
    eprintln!(
        "windowed: below 0.40 from day {:?}, below 0.30 from day {:?} to day {:?} ({} states), lowest FTSW {lowest:.4}",
        leaf.first().map(|&n| day(n)),
        growth.first().map(|&n| day(n)),
        growth.last().map(|&n| day(n)),
        growth.len(),
    );
    assert!(!growth.is_empty(), "the window must stress the crop");
    assert!(
        day(growth[0]) >= CUT_FROM_DAY && day(growth[0]) < CUT_TO_DAY,
        "stress must begin inside the window, began on day {}",
        day(growth[0])
    );
    assert!(day(leaf[0]) >= CUT_FROM_DAY, "no stress before the cut");
}

#[test]
fn every_drought_factor_is_back_at_exactly_one_before_maturity_and_stays_there() {
    let dry = trace(&params::biosphere(), true);
    let maturity = first_at(&dry, 2.0);
    let growth = stressed(&dry, SCENARIO.wssg);
    let leaf = stressed(&dry, science::LEAF_WSSL);
    let last = *growth.last().expect("the window stresses the crop");
    eprintln!(
        "windowed: last stressed day {} (0.30), {:?} (0.40); maturity day {}",
        day(last),
        leaf.last().map(|&n| day(n)),
        day(maturity)
    );
    assert!(
        last < maturity && *leaf.last().unwrap() < maturity,
        "the crop must recover before it matures"
    );
    assert!(dry[maturity..]
        .iter()
        .all(|s| science::water_stress_factor(ftsw(s), science::LEAF_WSSL) == 1.0));
}

#[test]
fn the_drought_hastens_development_and_costs_grain() {
    let p = params::biosphere();
    let control = trace(&p, false);
    let dry = trace(&p, true);
    let peak_factor = dry
        .iter()
        .map(|s| {
            let f = science::water_stress_factor(ftsw(s), SCENARIO.wssg);
            science::drought_development_factor(f, SCENARIO.wssd.expect("wheat has a WSSD"))
        })
        .fold(1.0, f64::max);
    let (ca, cm) = (first_at(&control, 1.0), first_at(&control, 2.0));
    let (da, dm) = (first_at(&dry, 1.0), first_at(&dry, 2.0));
    let grain_ratio = dry[dm].stocks[STORAGE_C].amount / control[cm].stocks[STORAGE_C].amount;
    eprintln!(
        "development factor peak {peak_factor:.4}; anthesis day {} vs {}; maturity day {} vs {}; \
         grain at maturity {:.4} vs {:.4} (ratio {grain_ratio:.4}); end thermal time {:.2} vs {:.2}",
        day(da),
        day(ca),
        day(dm),
        day(cm),
        dry[dm].stocks[STORAGE_C].amount,
        control[cm].stocks[STORAGE_C].amount,
        dry.last().unwrap().aux[THERMAL_TIME],
        control.last().unwrap().aux[THERMAL_TIME],
    );
    assert!(peak_factor > 1.0 && peak_factor <= 1.0 + SCENARIO.wssd.unwrap());
    assert!(dry.last().unwrap().aux[THERMAL_TIME] > control.last().unwrap().aux[THERMAL_TIME]);
    assert!(dm <= cm, "drought must not delay maturity");
    assert!(grain_ratio < 1.0, "a drought at flowering must cost grain");
}

#[test]
fn the_windowed_run_is_bit_identical_on_a_rerun() {
    let p = params::biosphere();
    let (a, b) = (trace(&p, true), trace(&p, true));
    let bits = |s: &State| -> Vec<u64> {
        s.stocks
            .values()
            .map(|k| k.amount.to_bits())
            .chain(s.aux.values().map(|v| v.to_bits()))
            .collect()
    };
    assert_eq!(a.len(), b.len());
    assert!(a.iter().zip(&b).all(|(x, y)| bits(x) == bits(y)));
}

/// `log/leaf-rust-remeasure.md` finding 5: the lab leaf form's drought factor had fired on 0
/// steps of any run. On this window it fires before flowering, when leaf expansion runs.
#[test]
fn the_lab_leaf_form_feels_the_window_before_flowering() {
    let p = biosphere_with_leaf_form(&[], LeafAreaForm::NodeEnvelope).expect("frozen params load");
    let dry = trace(&p, true);
    let anthesis = first_at(&dry, 1.0);
    let before = stressed(&dry, science::LEAF_WSSL)
        .into_iter()
        .filter(|&n| n < anthesis)
        .count();
    eprintln!(
        "lab leaf form: {before} pre-flowering states below 0.40; anthesis day {}",
        day(anthesis)
    );
    assert!(before > 0);
}
