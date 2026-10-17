//! **The open field gets a night** — the lab form's own checks.
//!
//! Plan: `docs/plans/post-roadmap-open-field-night.md`. [`WeatherRadiation::SunPath`] spreads the
//! weather's daily net radiation over the sun's half-sine, as PAR already is, and supplies the step's
//! daylit share; [`WeatherRadiation::DailyMean`] (the day's mean on every step, wholly lit) is the
//! loader's. This file asserts the loader is untouched (N1), the sun's path keeps each day's total and
//! lights exactly when PAR does (N2, N3), and that a night step reaches the flow as a dark one.

use domains::biosphere::params;
use domains::biosphere::stocks::{LIT_FRACTION_VAR, PAR_VAR, RN_VAR, SOIL_WATER, VAPOR_SINK};
use domains::biosphere::system::{
    build_season_with, weather_forcings, weather_forcings_with, weather_resolver_with,
    WeatherRadiation, DEFAULT_SCENARIO,
};
use domains::biosphere::weather::{net_radiation, weather_facts, ALBEDO, PAR_UMOL_PER_J};
use domains::biosphere::{season_setup_composed, BIO_DT, SEASON_DAYS, STEPS_PER_DAY};
use simcore::environment::SourceResolver;
use simcore::integrator::EulerIntegrator;
use simcore::state::State;

const SEASON_STEPS: u64 = (SEASON_DAYS * STEPS_PER_DAY) as u64;

/// The loader's weather is the sun's path, to the bit (adopted 2026-10-17, plan §9 — until then N1,
/// "the loader keeps the daily mean"); the retired daily-mean form keeps its own rule: the day's mean
/// on every step, wholly lit.
#[test]
fn the_loader_is_the_sun_path_and_the_daily_mean_form_is_kept() {
    let s = DEFAULT_SCENARIO;
    let loader = weather_forcings(&s, 1).expect("loader");
    let sun = weather_forcings_with(&s, 1, WeatherRadiation::SunPath).expect("sun");
    let daily = weather_forcings_with(&s, 1, WeatherRadiation::DailyMean).expect("daily");
    let (_, rows) = weather_facts();
    let mut dark = 0;
    for n in 0..SEASON_STEPS {
        for var in [RN_VAR, LIT_FRACTION_VAR] {
            assert_eq!(
                loader[var](n, BIO_DT).to_bits(),
                sun[var](n, BIO_DT).to_bits(),
                "{var}, step {n}"
            );
        }
        if loader[RN_VAR](n, BIO_DT) <= 0.0 {
            dark += 1;
        }
        let day = (n as f64 * BIO_DT) as usize;
        let want = net_radiation(rows[day].irrad_j_m2_day);
        assert_eq!(
            daily[RN_VAR](n, BIO_DT).to_bits(),
            want.to_bits(),
            "step {n}"
        );
        assert_eq!(daily[LIT_FRACTION_VAR](n, BIO_DT), 1.0, "step {n}");
    }
    assert!(dark > 0, "the loader's open field has no night");
}

/// N2, N3: under the sun's path every day keeps its net radiation, and the net radiation, PAR and the
/// lit share are zero together; the net radiation is one fixed multiple of PAR (both are fractions of
/// the same daily shortwave); a step wholly inside the day is wholly lit; two steps a day are split.
#[test]
fn the_sun_path_keeps_each_days_net_radiation_and_lights_with_par() {
    let s = DEFAULT_SCENARIO;
    let sun = weather_forcings_with(&s, 1, WeatherRadiation::SunPath).expect("sun");
    let (_, rows) = weather_facts();
    // Rn = (1 − α)·Rs; PAR = 0.5·Rs·4.57 µmol/J — over the same window, the same shape.
    let ratio = (1.0 - ALBEDO) / (0.5 * PAR_UMOL_PER_J);
    let mut days_with_two_split = 0;
    for (day, row) in rows.iter().enumerate() {
        let mut dose = 0.0;
        let mut split = 0;
        for k in 0..STEPS_PER_DAY {
            let n = (day * STEPS_PER_DAY + k) as u64;
            let rn = sun[RN_VAR](n, BIO_DT);
            let par = sun[PAR_VAR](n, BIO_DT);
            let phi = sun[LIT_FRACTION_VAR](n, BIO_DT);
            assert!((0.0..=1.0).contains(&phi), "day {day} step {k}: φ {phi}");
            assert_eq!(
                rn > 0.0,
                par > 0.0,
                "day {day} step {k}: Rn {rn}, PAR {par}"
            );
            assert_eq!(rn > 0.0, phi > 0.0, "day {day} step {k}: Rn {rn}, φ {phi}");
            if rn > 0.0 {
                let r = rn / par;
                assert!(
                    ((r - ratio) / ratio).abs() < 1e-12,
                    "day {day} step {k}: Rn / PAR {r}, want {ratio}"
                );
            }
            if phi > 0.0 && phi < 1.0 {
                split += 1;
            }
            dose += rn * BIO_DT;
        }
        let want = net_radiation(row.irrad_j_m2_day);
        assert!(
            ((dose - want) / want).abs() < 1e-12,
            "day {day}: the steps carry {dose} W/m²·d, the day {want}"
        );
        assert!(split <= 2, "day {day}: {split} partly lit steps");
        days_with_two_split += usize::from(split == 2);
    }
    assert!(
        days_with_two_split * 20 >= rows.len() * 19,
        "only {days_with_two_split} of {} days split a sunrise and a sunset step",
        rows.len()
    );
}

fn setup(radiation: WeatherRadiation) -> (State, EulerIntegrator, SourceResolver) {
    let s = DEFAULT_SCENARIO;
    // Step 1's form under the daily-mean air it was measured with, named rather than the loader's.
    let p = domains::lab::with_air_temperature(
        params::biosphere(),
        domains::biosphere::science::AirTemperatureRead::DailyMean,
    );
    let (state, integrator, _) =
        season_setup_composed(&s, 1, &p, &|s, p| build_season_with(s, p)).expect("setup");
    (
        state,
        integrator,
        weather_resolver_with(&s, 1, radiation).expect("weather"),
    )
}

fn run(radiation: WeatherRadiation, days: usize) -> State {
    let (mut state, integrator, resolver) = setup(radiation);
    for _ in 0..days * STEPS_PER_DAY {
        state = integrator
            .step_report(&state, &resolver, BIO_DT)
            .expect("step")
            .state;
    }
    state
}

/// The reach: grown 200 days, the open field's `Transpiration` draws less soil water at midnight
/// under the sun's path (a dark step, closed stomata, no radiation) than under the daily mean, and
/// more at noon (the day's radiation concentrated into the daylight).
#[test]
fn a_night_step_reaches_the_flow_as_a_dark_one() {
    let state = run(WeatherRadiation::DailyMean, 200);
    let draw = |radiation: WeatherRadiation, n: u64| -> f64 {
        let (_, integrator, resolver) = setup(radiation);
        // The same crop, read at step `n` of its day.
        let mut at = state.clone();
        at.n = n;
        let env = resolver.bind(&at, BIO_DT);
        let flow = integrator
            .registry()
            .flows()
            .iter()
            .find(|f| f.type_name() == "Transpiration")
            .expect("one transpiration flow");
        -flow
            .evaluate(&at, &env, BIO_DT)
            .expect("flow")
            .legs
            .iter()
            .filter(|l| l.stock == SOIL_WATER)
            .map(|l| l.amount)
            .sum::<f64>()
    };
    let midnight = (200 * STEPS_PER_DAY) as u64;
    let noon = midnight + (STEPS_PER_DAY / 2) as u64;
    let (night_sun, night_mean) = (
        draw(WeatherRadiation::SunPath, midnight),
        draw(WeatherRadiation::DailyMean, midnight),
    );
    let (noon_sun, noon_mean) = (
        draw(WeatherRadiation::SunPath, noon),
        draw(WeatherRadiation::DailyMean, noon),
    );
    assert!(
        night_sun > 0.0 && night_sun < 0.5 * night_mean,
        "midnight: the sun's path draws {night_sun}, the daily mean {night_mean}"
    );
    assert!(
        noon_sun > noon_mean,
        "noon: the sun's path draws {noon_sun}, the daily mean {noon_mean}"
    );
}

/// A season under the sun's path runs (conservation is asserted every step) and moves the water.
#[test]
fn a_season_under_the_sun_path_runs_and_moves_the_water() {
    let (sun, mean) = (
        run(WeatherRadiation::SunPath, 60),
        run(WeatherRadiation::DailyMean, 60),
    );
    assert_ne!(
        sun.stocks[VAPOR_SINK].amount.to_bits(),
        mean.stocks[VAPOR_SINK].amount.to_bits(),
        "the crop's water did not move"
    );
}
