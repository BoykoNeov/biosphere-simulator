//! **A scorecard row against a real chamber** — the 2026-09-29 review's Step 6, slice 3. LAB-ONLY.
//!
//! Plan: `docs/plans/post-roadmap-real-world-checks.md` §7–§8. The trial: NASA TM 102788
//! (Wheeler & Sager 1990), a 20 m² stand of Yecora Rojo spring wheat in nutrient solution in the
//! Biomass Production Chamber, 20 h light, CO₂ held at 1000 ppm, January–April 1989. Every number
//! from it below carries its page.
//!
//! The model: the frozen wheat in the OPEN build (whose leaf CO₂ is a fixed forcing — a held
//! chamber's counterpart) with vernalization off, every weather-derived forcing replaced by the
//! trial's conditions. Model day 0 is TM day 3, when the lamps came on (p. 5).
//!
//! ⚠ **This is a diagnostic, not a gate.** It asserts only its own instrument — the books, the
//! replaced weather, the trial's non-limiting water and nitrogen, the unit arithmetic — and prints
//! the scorecard. No ratio is pinned: a pinned ratio would make every science change re-pin it.
//! The ratios live in the plan, tied to a commit.
//!
//! ⚠ **Totals are set by the starting seedling, not the science.** The frozen seedling is 0.16
//! mol C m⁻² with no plant count; the trial sowed ~1500 plants m⁻² (p. 9), so its canopy closed by
//! day ~25. Season totals (U2, C1, B1, W2) are labelled so; the closed-canopy rates (U1, N1, N2, W1)
//! are the comparable rows.

use std::collections::BTreeMap;

use domains::biosphere::light_path::top_hat_window_mean;
use domains::biosphere::params::{self, BiosphereParams};
use domains::biosphere::science;
use domains::biosphere::stocks::{
    CI_VAR, CO2_ATMOS, CO2_RESP, DAYLENGTH_VAR, FERTILIZATION_VAR, IRRIGATION_VAR, LEAF_C, PAR_VAR,
    PLANT_N, RN_VAR, ROOTED_DEPTH, ROOT_C, SOIL_WATER, STEM_C, STEM_RESERVE_C, STORAGE_C,
    LITTER_SINK, TEMP_VAR, VPD_VAR,
};
use domains::biosphere::system::{build_season, build_season_with, weather_forcings, weather_shared};
use domains::biosphere::weather::saturation_vapor_pressure;
use domains::biosphere::{SeasonScenario, BIO_DT, DEFAULT_SCENARIO, STEPS_PER_DAY};
use simcore::environment::{constant, Schedule, SourceResolver};
use simcore::integrator::EulerIntegrator;
use simcore::registry::Registry;
use simcore::state::State;
use station::lighting::lamp_net_radiation;

/// TM day of the model's step 0: planted on day 0, 72 h dark, then the lamps (pp. 4–5).
const TM_DAY0: usize = 3;
/// The last TM day: lights off on day 86 (p. 6).
const TM_LAST_DAY: usize = 86;
const DAYS: usize = TM_LAST_DAY - TM_DAY0;
/// 20 h light / 4 h dark (p. 5).
const PHOTOPERIOD_S: f64 = 20.0 * 3600.0;
/// PPF before / after the day-28 dimming (p. 11: "from 695 to 480").
const PAR_EARLY: f64 = 695.0;
const PAR_LATE: f64 = 480.0;
const DIMMED_ON_TM_DAY: usize = 28;
/// 20 °C constant to day 34, then 20 °C light / 16 °C dark (p. 5).
const NIGHTS_COOL_FROM_TM_DAY: usize = 34;
const T_LIGHT: f64 = 20.0;
const T_DARK: f64 = 16.0;
/// Relative humidity, light period (p. 5: 81 % ± 4).
const RH: f64 = 0.81;
/// CO₂ in the light: mean 1160 ppm (the row); the 1000 ppm setpoint (the sensitivity) (p. 5).
const CA_ROW: f64 = 1160.0;
const CA_SETPOINT: f64 = 1000.0;
/// µmol m⁻² s⁻¹ from mol per step over the model's ground area.
const STEP_SECONDS: f64 = BIO_DT * 86_400.0;

fn tm_day(n: u64) -> usize {
    TM_DAY0 + n as usize / STEPS_PER_DAY
}

/// The lamp's lit share of step `n` — the top-hat the station's lamp path uses.
fn lit(n: u64, dt: f64) -> f64 {
    let t = n as f64 * dt;
    top_hat_window_mean(t - t.trunc(), dt, 1.0, PHOTOPERIOD_S).expect("a valid window")
}

fn par_on(n: u64) -> f64 {
    if tm_day(n) < DIMMED_ON_TM_DAY {
        PAR_EARLY
    } else {
        PAR_LATE
    }
}

fn temp(n: u64, dt: f64) -> f64 {
    if tm_day(n) < NIGHTS_COOL_FROM_TM_DAY {
        T_LIGHT
    } else {
        T_DARK + (T_LIGHT - T_DARK) * lit(n, dt)
    }
}

fn scenario(ca: f64) -> SeasonScenario {
    SeasonScenario {
        vernalization: false,
        // The frozen sealed chamber's leaf-to-air ratio, applied to the trial's held air.
        ci: DEFAULT_SCENARIO.ci_ratio * ca,
        ..DEFAULT_SCENARIO
    }
}

/// The trial's chamber as forcings, and the keys it replaced. `rn_scale` multiplies the lamp's
/// net radiation — 1 for the row, other values only in the water-gap WHAT-IF (plan §10).
fn chamber_resolver(s: &SeasonScenario, rn_scale: f64) -> (SourceResolver, Vec<String>) {
    let mut forcings = weather_forcings(s, 1).expect("weather forcings");
    let replaced: Vec<(&str, Schedule)> = vec![
        (PAR_VAR, Box::new(|n, dt| par_on(n) * lit(n, dt))),
        (
            RN_VAR,
            Box::new(move |n, dt| rn_scale * lamp_net_radiation(par_on(n)) * lit(n, dt)),
        ),
        (DAYLENGTH_VAR, constant(PHOTOPERIOD_S).unwrap()),
        (TEMP_VAR, Box::new(temp)),
        (
            VPD_VAR,
            Box::new(|n, dt| saturation_vapor_pressure(temp(n, dt)) * (1.0 - RH)),
        ),
    ];
    let mut keys = Vec::new();
    for (k, f) in replaced {
        assert!(forcings.insert(k.to_string(), f).is_some(), "{k} was not a weather forcing");
        keys.push(k.to_string());
    }
    let resolver = SourceResolver::new(forcings, weather_shared(s)).expect("resolver");
    (resolver, keys)
}

/// Per-step books of one run.
struct Run {
    states: Vec<State>,
    /// Net crop CO₂ exchange per step, mol (positive = into the air).
    ///
    /// ⚠ The open build keeps the air in TWO boundary stocks: `co2_atmos` supplies the carbon and
    /// `co2_resp` receives the respired CO₂ (even the share of maintenance paid straight out of the
    /// day's assimilate is a draw from one and a return to the other). The trial's analyser saw
    /// their sum. The first draft booked `co2_atmos` alone, read the night as exactly 0, and was
    /// caught by that zero before any score was taken (plan §9).
    co2: Vec<f64>,
    /// The same, per flow and boundary stock, summed over the run.
    co2_by_flow: BTreeMap<String, f64>,
    /// Transpiration per step, kg.
    water: Vec<f64>,
}

fn run(ca: f64) -> Run {
    let s = scenario(ca);
    run_built(&s, build_season(&s).expect("build"), 1.0)
}

/// The water-gap WHAT-IF (plan §10): other params and/or a scaled lamp net radiation.
fn run_what_if(p: &BiosphereParams, rn_scale: f64) -> Run {
    let s = scenario(CA_ROW);
    run_built(&s, build_season_with(&s, p).expect("build"), rn_scale)
}

fn run_built(s: &SeasonScenario, built: (State, Registry), rn_scale: f64) -> Run {
    let (state0, registry) = built;
    let integrator = EulerIntegrator::new(registry);
    let (resolver, _) = chamber_resolver(s, rn_scale);
    let steps = DAYS * STEPS_PER_DAY;
    let mut state = state0;
    let mut out = Run {
        states: vec![state.clone()],
        co2: Vec::with_capacity(steps),
        co2_by_flow: BTreeMap::new(),
        water: Vec::with_capacity(steps),
    };
    for _ in 0..steps {
        let env = resolver.bind(&state, BIO_DT);
        let (mut co2, mut water) = (0.0, 0.0);
        for flow in integrator.registry().flows() {
            let result = flow.evaluate(&state, &env, BIO_DT).expect("flow");
            for leg in &result.legs {
                if leg.stock == CO2_ATMOS || leg.stock == CO2_RESP {
                    co2 += leg.amount;
                    *out.co2_by_flow.entry(format!("{} -> {}", flow.id(), leg.stock)).or_insert(0.0) += leg.amount;
                }
                if flow.type_name() == "Transpiration" && leg.stock == SOIL_WATER {
                    water -= leg.amount;
                }
            }
        }
        let report = integrator.step_report(&state, &resolver, BIO_DT).expect("step");
        assert_eq!(report.rationed, 0, "the backstop must stay out of a scorecard");
        assert!(report.events.is_empty(), "unexpected events: {:?}", report.events);
        let air = |s: &State| s.stocks[CO2_ATMOS].amount + s.stocks[CO2_RESP].amount;
        let observed = air(&report.state) - air(&state);
        assert!(
            (observed - co2).abs() <= 1e-9 * observed.abs().max(1e-6),
            "the CO₂ books do not close on step {}: booked {co2}, observed {observed}",
            state.n
        );
        out.co2.push(co2);
        out.water.push(water);
        state = report.state;
        out.states.push(state.clone());
    }
    out
}

fn umol_m2_s(mol_per_step: f64) -> f64 {
    mol_per_step / (DEFAULT_SCENARIO.ground_area * STEP_SECONDS) * 1e6
}

/// Step indices of one TM day where the lamp is fully on (`want = 1`) or fully off (`want = 0`).
fn steps_of(day: usize, want: f64) -> impl Iterator<Item = usize> {
    let first = (day - TM_DAY0) * STEPS_PER_DAY;
    (first..first + STEPS_PER_DAY).filter(move |&n| lit(n as u64, BIO_DT) == want)
}

fn mean(xs: impl Iterator<Item = f64>) -> f64 {
    let (s, c) = xs.fold((0.0, 0usize), |(s, c), x| (s + x, c + 1));
    s / c as f64
}

fn night_resp(r: &Run, day: usize) -> f64 {
    mean(steps_of(day, 0.0).map(|n| umol_m2_s(r.co2[n])))
}

fn day_uptake_peak(r: &Run, day: usize) -> f64 {
    steps_of(day, 1.0)
        .map(|n| -umol_m2_s(r.co2[n]))
        .fold(f64::MIN, f64::max)
}

fn daily_water(r: &Run, day: usize) -> f64 {
    let first = (day - TM_DAY0) * STEPS_PER_DAY;
    r.water[first..first + STEPS_PER_DAY].iter().sum::<f64>() / DEFAULT_SCENARIO.ground_area
}

/// Net carbon fixed over TM days `from..=to`, mol C m⁻².
fn net_fixed(r: &Run, from: usize, to: usize) -> f64 {
    let a = (from - TM_DAY0) * STEPS_PER_DAY;
    let b = (to + 1 - TM_DAY0) * STEPS_PER_DAY;
    -r.co2[a..b].iter().sum::<f64>() / DEFAULT_SCENARIO.ground_area
}

fn plant_c(s: &State) -> f64 {
    [LEAF_C, STEM_C, ROOT_C, STORAGE_C, STEM_RESERVE_C]
        .iter()
        .map(|k| s.stocks.get(*k).map_or(0.0, |x| x.amount))
        .sum()
}

#[test]
fn the_unit_arithmetic_reproduces_the_tms_own() {
    // p. 6: 1 ppm = 4692 µmol in 112 600 L at 20 °C; abstract: 440 ppm per 4-h night = 7.2.
    let night: f64 = 440.0 * 4692.0 / 20.0 / (4.0 * 3600.0);
    assert!((night - 7.2).abs() < 0.05, "{night}");
    // p. 10: 15 µmol m⁻² s⁻¹ over 20 m² for a 20-h day = 21.6 mol.
    let day_mol = 15.0 * 20.0 * PHOTOPERIOD_S / 1e6;
    assert!((day_mol - 21.6).abs() < 1e-9, "{day_mol}");
    // And this file's own converter: 1 µmol m⁻² s⁻¹ held for one step on 1 m².
    assert!((umol_m2_s(STEP_SECONDS * 1e-6) - 1.0).abs() < 1e-12);
}

#[test]
fn every_weather_forcing_is_the_chambers() {
    let s = scenario(CA_ROW);
    let weather: Vec<String> = weather_forcings(&s, 1).unwrap().into_keys().collect();
    let (_, replaced) = chamber_resolver(&s, 1.0);
    let scenario_constants = [CI_VAR, IRRIGATION_VAR, FERTILIZATION_VAR];
    for k in &weather {
        assert!(
            replaced.contains(k) || scenario_constants.contains(&k.as_str()),
            "the outdoor weather still supplies `{k}`"
        );
    }
}

#[test]
fn the_crop_is_never_short_of_water_or_nitrogen_as_in_nutrient_solution() {
    let r = run(CA_ROW);
    let nitro = params::biosphere().nitro;
    let (mut min_ftsw, mut min_fn) = (f64::MAX, f64::MAX);
    for s in &r.states {
        let depth = s.aux.get(ROOTED_DEPTH).copied().unwrap_or(0.0);
        let cap = science::transpirable_capacity(
            depth,
            DEFAULT_SCENARIO.soil_extractable_water,
            DEFAULT_SCENARIO.ground_area,
        );
        min_ftsw = min_ftsw.min(science::fraction_transpirable(s.stocks[SOIL_WATER].amount, cap));
        let biomass = [LEAF_C, STEM_C, ROOT_C].iter().map(|k| s.stocks[*k].amount).sum();
        min_fn = min_fn.min(science::nitrogen_stress_factor(
            s.stocks[PLANT_N].amount,
            biomass,
            nitro.n_residual_per_mol_c,
            nitro.n_critical_per_mol_c,
        ));
    }
    eprintln!("conditions: lowest FTSW {min_ftsw:.4}, lowest nitrogen factor {min_fn:.4}");
    assert!(min_ftsw >= DEFAULT_SCENARIO.wssg, "water limited: FTSW {min_ftsw}");
    assert_eq!(min_fn, 1.0, "nitrogen limited");
}

/// The instrument sees both halves of the day: uptake on every fully lit step after the
/// seedling's first day, and an efflux on every fully dark step. The first draft read 0 at night.
#[test]
fn the_books_see_the_night_and_the_day() {
    let r = run(CA_ROW);
    for d in TM_DAY0..TM_LAST_DAY {
        assert!(steps_of(d, 0.0).count() > 0 && steps_of(d, 1.0).count() > 0);
        assert!(night_resp(&r, d) > 0.0, "no night respiration on TM day {d}");
        assert!(day_uptake_peak(&r, d) > 0.0, "no daytime uptake on TM day {d}");
    }
}

#[test]
fn the_row_is_bit_identical_on_a_rerun() {
    let (a, b) = (run(CA_ROW), run(CA_ROW));
    assert!(a.co2.iter().zip(&b.co2).all(|(x, y)| x.to_bits() == y.to_bits()));
    assert!(a.water.iter().zip(&b.water).all(|(x, y)| x.to_bits() == y.to_bits()));
}

/// The scorecard itself: printed, never asserted (`--nocapture` to read it).
#[test]
fn scorecard() {
    let r = run(CA_ROW);
    let (peak, peak_day) = (TM_DAY0..TM_LAST_DAY)
        .map(|d| (day_uptake_peak(&r, d), d))
        .fold((f64::MIN, 0), |a, b| if b.0 > a.0 { b } else { a });
    let night_20 = night_resp(&r, 20);
    let night_mean = mean((TM_DAY0..TM_LAST_DAY).map(|d| night_resp(&r, d)));
    let n2 = night_resp(&r, NIGHTS_COOL_FROM_TM_DAY - 1) / night_resp(&r, NIGHTS_COOL_FROM_TM_DAY);
    let w1 = mean((25..=80).map(|d| daily_water(&r, d)));
    let w2 = mean((TM_DAY0..TM_LAST_DAY).map(|d| daily_water(&r, d)));
    let u2 = mean((10..=84).flat_map(|d| steps_of(d, 1.0)).map(|n| -umol_m2_s(r.co2[n])));
    let c1 = net_fixed(&r, 10, 84);
    let end = r.states.last().unwrap();
    let b1 = plant_c(end) / DEFAULT_SCENARIO.ground_area;
    // TM p. 10 says only "about 40 kg of total biomass" — whether roots are in it is not stated.
    let b1_shoot = (plant_c(end) - end.stocks[ROOT_C].amount) / DEFAULT_SCENARIO.ground_area;
    let c1_setpoint = net_fixed(&run(CA_SETPOINT), 10, 84);

    let rows: [(&str, f64, f64, &str); 10] = [
        ("U1 peak daytime net uptake (µmol m⁻² s⁻¹)", peak, 27.0, "rate"),
        ("N1 night respiration, TM day 20", night_20, 13.0, "rate"),
        ("N1 night respiration, season mean", night_mean, 7.2, "rate"),
        ("N2 night resp. 20 °C / 16 °C (day 33 / 34)", n2, 1.65, "rate"),
        ("W1 transpiration, days 25–80 (L m⁻² d⁻¹)", w1, 6.0, "rate"),
        ("U2 mean daytime net uptake, days 10–84", u2, 15.0, "total"),
        ("C1 net C fixed, days 10–84 (mol C m⁻²)", c1, 73.1, "total"),
        ("B1 plant C at day 86, roots in (mol C m⁻²)", b1, 66.7, "total"),
        ("B1 plant C at day 86, roots out", b1_shoot, 66.7, "total"),
        ("W2 mean transpiration (L m⁻² d⁻¹)", w2, 4.5, "TOTAL (leaf-blind)"),
    ];
    eprintln!("\nTM 102788 scorecard (ci = {:.0} ppm):", scenario(CA_ROW).ci);
    for (name, model, tm, kind) in rows {
        eprintln!("  {name:<48} model {model:>9.4}  TM {tm:>6.2}  ratio {:>7.4}  [{kind}]", model / tm);
    }
    eprintln!("  U1 peak on TM day {peak_day}");
    eprintln!(
        "  S1 C1 at ci {:.0} / ci {:.0}: {:.4}",
        scenario(CA_ROW).ci,
        scenario(CA_SETPOINT).ci,
        c1 / c1_setpoint
    );
    let ph = params::phenology();
    let tt = end.aux[domains::biosphere::stocks::THERMAL_TIME];
    eprintln!(
        "  end: DVS {:.3}; leaf {:.3} stem {:.3} root {:.3} grain {:.3} reserve {:.3} mol C",
        science::development_stage(tt, ph.tsum_anthesis, ph.tsum_maturity),
        end.stocks[LEAF_C].amount,
        end.stocks[STEM_C].amount,
        end.stocks[ROOT_C].amount,
        end.stocks[STORAGE_C].amount,
        end.stocks.get(STEM_RESERVE_C).map_or(0.0, |x| x.amount),
    );
    eprintln!("  CO₂ by flow over the run (mol): {:?}", r.co2_by_flow);
    // Where the fixed carbon went: the plant, or tissue shed to the open build's litter sink.
    let start = &r.states[0];
    let fixed_all = -r.co2.iter().sum::<f64>();
    let litter = end.stocks[LITTER_SINK].amount - start.stocks[LITTER_SINK].amount;
    let grew = plant_c(end) - plant_c(start);
    eprintln!(
        "  carbon over the whole run (mol): fixed {fixed_all:.3} = plant gain {grew:.3} + shed to \
         litter {litter:.3} + rest {:.2e}",
        fixed_all - grew - litter
    );
    for d in [5, 10, 15, 20, 25, 30, 40, 50, 60, 70, 80] {
        eprintln!(
            "  TM day {d:>2}: peak uptake {:>7.3}  night resp {:>7.3}  water {:>6.3}  plant C {:>8.3}",
            day_uptake_peak(&r, d),
            night_resp(&r, d),
            daily_water(&r, d),
            plant_c(&r.states[(d - TM_DAY0) * STEPS_PER_DAY])
        );
    }
}

// --- the water gap, split (plan §10) — WHAT-IF, lab-only, pinning nothing ----------------------

/// TM days 25–80: the full-cover window the W1 row averages.
const W1_DAYS: std::ops::RangeInclusive<usize> = 25..=80;

/// W1 from the forcings alone: Penman–Monteith on every step's lamp net radiation, VPD and
/// temperature, at the given resistances, with no soil-water factor (the conditions test holds
/// FTSW above `wssg`, where that factor is exactly 1).
fn w1_by_hand(ra: f64, rs: f64, rn_scale: f64) -> f64 {
    mean(W1_DAYS.map(|d| {
        let first = ((d - TM_DAY0) * STEPS_PER_DAY) as u64;
        (first..first + STEPS_PER_DAY as u64)
            .map(|n| {
                let t = temp(n, BIO_DT);
                let rn = rn_scale * lamp_net_radiation(par_on(n)) * lit(n, BIO_DT);
                let vpd = saturation_vapor_pressure(t) * (1.0 - RH);
                science::penman_monteith_transpiration(rn, vpd, t, ra, rs) * BIO_DT
            })
            .sum::<f64>()
    }))
}

fn transpiration_params() -> (f64, f64) {
    let p = params::biosphere();
    (p.transp.aerodynamic_resistance, p.transp.surface_resistance)
}

/// A1 — airflow alone cannot close the gap: with the air resistance near 0, full-cover water use
/// tops out below the trial's ~6 L m⁻² d⁻¹ at the frozen surface resistance.
#[test]
fn airflow_alone_cannot_reach_the_trials_water_use() {
    let (ra, rs) = transpiration_params();
    let frozen = w1_by_hand(ra, rs, 1.0);
    let limit = w1_by_hand(1e-6, rs, 1.0);
    eprintln!("A1: W1 at the frozen ra {ra} = {frozen:.4}; as ra -> 0: {limit:.4} L m⁻² d⁻¹ (TM ~6.0)");
    assert!(limit < 6.0, "infinite airflow would close the gap: {limit}");
}

/// R1 — a WHAT-IF air resistance reaches `Transpiration`: the run's W1 equals Penman–Monteith on
/// the same forcings at the substituted value.
#[test]
fn a_what_if_air_resistance_reaches_the_water_flow() {
    let (_, rs) = transpiration_params();
    let p = domains::lab::biosphere_what_if(&[domains::lab::Substitution::new(
        "transpiration.yaml",
        "aerodynamic_resistance",
        10.0,
    )])
    .expect("what-if params");
    let r = run_what_if(&p, 1.0);
    let model = mean(W1_DAYS.map(|d| daily_water(&r, d)));
    let hand = w1_by_hand(10.0, rs, 1.0);
    eprintln!("R1 (WHAT-IF ra = 10): W1 model {model:.6}, by hand {hand:.6}");
    assert!((model - hand).abs() <= 1e-9 * hand, "{model} vs {hand}");
}

/// R2 — a WHAT-IF lamp heating moves water only: every CO₂ step and the end crop bit-identical to
/// the row, the root zone never below `wssg`.
#[test]
fn a_what_if_lamp_heating_moves_water_and_nothing_else() {
    let scale = 4.2;
    let base = run(CA_ROW);
    let hot = run_what_if(&params::biosphere(), scale);
    let min_ftsw = hot
        .states
        .iter()
        .map(|s| {
            let depth = s.aux.get(ROOTED_DEPTH).copied().unwrap_or(0.0);
            let cap = science::transpirable_capacity(
                depth,
                DEFAULT_SCENARIO.soil_extractable_water,
                DEFAULT_SCENARIO.ground_area,
            );
            science::fraction_transpirable(s.stocks[SOIL_WATER].amount, cap)
        })
        .fold(f64::MAX, f64::min);
    let w1 = mean(W1_DAYS.map(|d| daily_water(&hot, d)));
    eprintln!("R2 (WHAT-IF net radiation x{scale}): W1 {w1:.4} L m⁻² d⁻¹; lowest FTSW {min_ftsw:.4}");
    assert!(min_ftsw >= DEFAULT_SCENARIO.wssg, "the soil dried: {min_ftsw}");
    assert!(base.co2.iter().zip(&hot.co2).all(|(a, b)| a.to_bits() == b.to_bits()));
    let (a, b) = (base.states.last().unwrap(), hot.states.last().unwrap());
    for k in [LEAF_C, STEM_C, ROOT_C, STORAGE_C, STEM_RESERVE_C, PLANT_N] {
        assert_eq!(a.stocks[k].amount.to_bits(), b.stocks[k].amount.to_bits(), "{k} moved");
    }
}

/// Printed, never asserted (plan §10e): the canopy-scale surface resistance FAO-56 gives for the
/// model's own mean leaf area — Eq. 5 with Box 5's `LAI_active = 0.5 LAI` and `rl` ≈ 100 s m⁻¹,
/// read first-hand on 2026-10-07 (`docs/plans/post-roadmap-canopy-resistance.md` §2; FAO scopes it
/// to dense full cover, which LAI 6.2 is) through the one shared `science::canopy_surface_resistance`
/// — and the W1 it would give at the frozen air resistance. The frozen `rs` = 70 is uncited
/// (`TODO(cite)`); it equals FAO-56's grass reference, i.e. `rl = 100` at LAI 2.88.
#[test]
fn the_surface_resistance_a_full_canopy_would_have() {
    let r = run(CA_ROW);
    let sla = params::biosphere().canopy.sla_per_mol_c;
    let lai = mean(W1_DAYS.map(|d| {
        let s = &r.states[(d - TM_DAY0) * STEPS_PER_DAY];
        science::leaf_area_index(s.stocks[LEAF_C].amount, sla, DEFAULT_SCENARIO.ground_area)
    }));
    let (ra, rs) = transpiration_params();
    let tp = params::biosphere().transp;
    let rs_fao = science::canopy_surface_resistance(
        science::SurfaceResistanceForm::FaoFullCover,
        rs,
        lai,
        tp.leaf_stomatal_resistance,
        tp.threshold_lai,
    );
    eprintln!(
        "§10e: mean LAI over TM days 25–80 {lai:.3}; FAO-form rs {rs_fao:.2} s m⁻¹ (frozen {rs}); \
         W1 at ra {ra}: {:.4} (frozen rs: {:.4}); at that rs as ra -> 0: {:.4} (TM ~6.0)",
        w1_by_hand(ra, rs_fao, 1.0),
        w1_by_hand(ra, rs, 1.0),
        w1_by_hand(1e-6, rs_fao, 1.0)
    );
}

/// Printed, never asserted (`docs/plans/post-roadmap-canopy-resistance.md` §5, P8–P9): the row's
/// water use under the lab canopy-resistance forms, as a level (W1) and as a SHAPE — each day's
/// use over its own mean for TM days 25–80 — against TM 102788 Fig. 10 (p. 27) "water added",
/// read off the page image (±5–10 L day⁻¹ on ~85; stand of 20 m²): day 5 ~18, 8 ~40, 11 ~60,
/// 14 ~100, 26 ~110, then ~70–90 to day 80, ~57 at day 83.
#[test]
fn the_rows_water_curve_under_the_canopy_resistance_forms() {
    use domains::biosphere::science::SurfaceResistanceForm as F;
    let days = [5usize, 8, 11, 14, 20, 26, 40, 60, 80];
    let base = run(CA_ROW);
    let dark: f64 = (0..base.water.len())
        .filter(|&n| lit(n as u64, BIO_DT) == 0.0)
        .map(|n| base.water[n])
        .sum();
    let partly_dark: f64 = (0..base.water.len())
        .filter(|&n| {
            let l = lit(n as u64, BIO_DT);
            l > 0.0 && l < 1.0
        })
        .map(|n| base.water[n])
        .sum();
    let total: f64 = base.water.iter().sum();
    println!(
        "P9: under the frozen form, fully dark steps carry {:.4} of the row's water; partly lit steps {:.4}",
        dark / total,
        partly_dark / total
    );
    let sla = params::biosphere().canopy.sla_per_mol_c;
    for form in [F::Constant, F::FaoFullCover, F::SzeiczLong] {
        let r = run_what_if(&domains::lab::biosphere_with_rs_form(&[], form).unwrap(), 1.0);
        let w1 = mean(W1_DAYS.map(|d| daily_water(&r, d)));
        let shape: Vec<String> = days
            .iter()
            .map(|&d| format!("{d}:{:.2}", daily_water(&r, d) / w1))
            .collect();
        let lai: Vec<String> = days
            .iter()
            .map(|&d| {
                let s = &r.states[(d - TM_DAY0) * STEPS_PER_DAY];
                format!("{:.2}", science::leaf_area_index(s.stocks[LEAF_C].amount, sla, 1.0))
            })
            .collect();
        println!("P8 {form:<13?} W1 {w1:.4} L m⁻² d⁻¹; day/W1: {}", shape.join(" "));
        println!("   LAI on those days: {}", lai.join(" "));
    }
    println!("TM Fig. 10 water added / ~85 (its days 25–80 level): 5:0.21 8:0.47 11:0.71 14:1.18 26:1.29 40–80: 0.82–1.06 83:0.67");
}
