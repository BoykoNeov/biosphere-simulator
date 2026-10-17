//! **The open field gets a night, step 2: the air** — the lab form's own checks.
//!
//! Plan: `docs/plans/post-roadmap-open-field-night.md` §7. [`WeatherAir::WithinDay`] puts the air
//! temperature on Teh's (2006, §2.7) curve through the day's observed minimum and maximum, and the
//! open field's deficit on it; [`AirTemperatureRead::WithinDay`] makes every water reader take it and
//! nothing else. This file asserts the loader is untouched (A1), the curve's own shape (A2), and that
//! the reader reaches the water: a build whose within-day data is the daily mean runs to the bit as
//! the reference does.

use domains::biosphere::params;
use domains::biosphere::science::AirTemperatureRead;
use domains::biosphere::stocks::{
    AIR_TEMP_WITHIN_DAY_VAR, CONDENSATE, TEMP_VAR, VAPOR_SINK, VPD_VAR, VPD_WITHIN_DAY_VAR,
};
use domains::biosphere::system::{
    build_season_with, weather_forcings, weather_forcings_lab, weather_shared, WeatherAir,
    WeatherRadiation, DEFAULT_SCENARIO,
};
use domains::biosphere::weather::{
    air_temperature_day_mean, air_temperature_window_mean, daylength_seconds, weather_facts,
};
use domains::biosphere::{
    sealed_chamber_scenario, season_setup_composed, SeasonScenario, BIO_DT, STEPS_PER_DAY,
};
use domains::lab::with_air_temperature;
use simcore::environment::{Schedule, SourceResolver};
use simcore::state::State;
use std::collections::HashMap;

/// A1: the loader's weather carries no within-day air, and the loader's water reads the daily mean.
#[test]
fn the_loader_keeps_the_daily_mean_air() {
    let loader = weather_forcings(&DEFAULT_SCENARIO, 1).expect("loader");
    assert!(!loader.contains_key(AIR_TEMP_WITHIN_DAY_VAR));
    assert!(!loader.contains_key(VPD_WITHIN_DAY_VAR));
    assert_eq!(
        params::biosphere().water.air_temperature,
        AirTemperatureRead::DailyMean
    );
}

/// A2: on every day of the fixture, the step windows tile the day to the curve's own 24-h mean; the
/// curve is at `TMIN` 1.5 h after sunrise and at `TMAX` 1.5 h after solar noon, never outside the
/// observed range, and continuous (no jump at sunrise + 1.5 h, sunset, or across its own midnight).
#[test]
fn the_curve_is_tehs_through_the_days_extremes() {
    let (latitude, rows) = weather_facts();
    let at = |hour: f64, tmin: f64, tmax: f64, dl: f64| {
        // A 1-second window: the curve's value there, to its slope × 0.5 s.
        air_temperature_window_mean(hour / 24.0, 1.0 / 86400.0, tmin, tmax, dl).expect("a day")
    };
    for (i, row) in rows.iter().enumerate() {
        let (tmin, tmax) = (row.tmin_c, row.tmax_c);
        let dl = daylength_seconds(latitude, row.day_of_year);
        let day_mean = air_temperature_day_mean(tmin, tmax, dl).expect("a day");
        let tiled: f64 = (0..STEPS_PER_DAY)
            .map(|k| {
                air_temperature_window_mean(k as f64 * BIO_DT, BIO_DT, tmin, tmax, dl)
                    .expect("a window")
            })
            .sum::<f64>()
            / STEPS_PER_DAY as f64;
        assert!(
            (tiled - day_mean).abs() < 1e-12,
            "day {i}: {tiled} vs {day_mean}"
        );
        let (sunrise, sunset) = (12.0 - 0.5 * dl / 3600.0, 12.0 + 0.5 * dl / 3600.0);
        let tol = 1e-3;
        assert!(
            (at(sunrise + 1.5, tmin, tmax, dl) - tmin).abs() < tol,
            "day {i}: not TMIN"
        );
        assert!(
            (at(13.5, tmin, tmax, dl) - tmax).abs() < tol,
            "day {i}: not TMAX"
        );
        for hour in [sunrise + 1.5, sunset] {
            let (before, after) = (
                at(hour - 1.0 / 3600.0, tmin, tmax, dl),
                at(hour, tmin, tmax, dl),
            );
            assert!((before - after).abs() < 1e-2, "day {i}: a jump at {hour} h");
        }
        let (first, last) = (
            at(0.0, tmin, tmax, dl),
            at(24.0 - 1.0 / 3600.0, tmin, tmax, dl),
        );
        assert!(
            (first - last).abs() < 1e-2,
            "day {i}: the day does not close at midnight"
        );
        for k in 0..STEPS_PER_DAY {
            let w = air_temperature_window_mean(k as f64 * BIO_DT, BIO_DT, tmin, tmax, dl)
                .expect("a window");
            assert!(
                w >= tmin - 1e-9 && w <= tmax + 1e-9,
                "day {i} step {k}: {w}"
            );
        }
    }
}

/// A2: the shifted variant's day mean is the observed mean; the unshifted one is the curve's own, and
/// the deficit is `e_s(T) − e_a` at the step's temperature, held at the day's vapour pressure.
#[test]
fn the_shifted_variant_keeps_the_observed_mean() {
    let s = DEFAULT_SCENARIO;
    let (latitude, rows) = weather_facts();
    let shifted = weather_forcings_lab(
        &s,
        1,
        WeatherRadiation::DailyMean,
        WeatherAir::WithinDayShifted,
    )
    .expect("shifted");
    let plain = weather_forcings_lab(&s, 1, WeatherRadiation::DailyMean, WeatherAir::WithinDay)
        .expect("plain");
    for (day, row) in rows.iter().enumerate() {
        let steps = (day * STEPS_PER_DAY) as u64..((day + 1) * STEPS_PER_DAY) as u64;
        let mean = |f: &HashMap<String, Schedule>| {
            steps
                .clone()
                .map(|n| f[AIR_TEMP_WITHIN_DAY_VAR](n, BIO_DT))
                .sum::<f64>()
                / STEPS_PER_DAY as f64
        };
        assert!((mean(&shifted) - row.temp_c).abs() < 1e-9, "day {day}");
        let dl = daylength_seconds(latitude, row.day_of_year);
        let own = air_temperature_day_mean(row.tmin_c, row.tmax_c, dl).expect("a day");
        assert!((mean(&plain) - own).abs() < 1e-9, "day {day}");
        for n in steps.clone() {
            let t = plain[AIR_TEMP_WITHIN_DAY_VAR](n, BIO_DT);
            let want = (domains::biosphere::weather::saturation_vapor_pressure(t)
                - row.vap_hpa * 100.0)
                .max(0.0);
            assert_eq!(
                plain[VPD_WITHIN_DAY_VAR](n, BIO_DT).to_bits(),
                want.to_bits()
            );
        }
    }
}

fn run(
    scenario: &SeasonScenario,
    read: AirTemperatureRead,
    forcings: HashMap<String, Schedule>,
    days: usize,
) -> Result<State, simcore::error::SimError> {
    let p = with_air_temperature(params::biosphere(), read);
    let (mut state, integrator, _) =
        season_setup_composed(scenario, 1, &p, &|s, p| build_season_with(s, p))?;
    let resolver = SourceResolver::new(forcings, weather_shared(scenario))?;
    for _ in 0..days * STEPS_PER_DAY {
        state = integrator.step_report(&state, &resolver, BIO_DT)?.state;
    }
    Ok(state)
}

/// The weather's daily mean, light and air both named — never the loader's, so the comparisons
/// below stay comparisons whichever form the loader takes.
fn daily_mean(s: &SeasonScenario) -> HashMap<String, Schedule> {
    weather_forcings_lab(s, 1, WeatherRadiation::DailyMean, WeatherAir::DailyMean).expect("weather")
}

/// The reader without the data is an error at the first step, not a silent daily mean.
#[test]
fn the_reader_without_the_data_fails() {
    for s in [DEFAULT_SCENARIO, sealed_chamber_scenario()] {
        let daily = daily_mean(&s);
        assert!(run(&s, AirTemperatureRead::WithinDay, daily, 1).is_err());
    }
}

/// The switch is only a re-wiring, on the open field and a sealed jar over 30 days: with the
/// within-day data replaced by the daily mean's own forcings, the reader runs **to the bit** as the
/// reference; with the real curve, the water moves. (Which flows read which name is the next test's:
/// a reader left on `temp` would pass this one, its two names holding the same values here.)
#[test]
fn the_reader_reaches_the_water_and_only_the_water() {
    for s in [DEFAULT_SCENARIO, sealed_chamber_scenario()] {
        let reference =
            run(&s, AirTemperatureRead::DailyMean, daily_mean(&s), 30).expect("reference");
        let mut aliased =
            weather_forcings_lab(&s, 1, WeatherRadiation::DailyMean, WeatherAir::WithinDay)
                .unwrap();
        let mut daily = daily_mean(&s);
        aliased.insert(
            AIR_TEMP_WITHIN_DAY_VAR.to_string(),
            daily.remove(TEMP_VAR).unwrap(),
        );
        aliased.insert(
            VPD_WITHIN_DAY_VAR.to_string(),
            daily.remove(VPD_VAR).unwrap(),
        );
        let same = run(&s, AirTemperatureRead::WithinDay, aliased, 30).expect("aliased");
        for (id, stock) in &reference.stocks {
            assert_eq!(
                stock.amount.to_bits(),
                same.stocks[id].amount.to_bits(),
                "{id} moved with the daily mean aliased in"
            );
        }
        assert_eq!(reference.aux, same.aux);
        let curve = run(
            &s,
            AirTemperatureRead::WithinDay,
            weather_forcings_lab(&s, 1, WeatherRadiation::DailyMean, WeatherAir::WithinDay)
                .unwrap(),
            30,
        )
        .expect("curve");
        let water = if s.sealed { CONDENSATE } else { VAPOR_SINK };
        assert_ne!(
            reference.stocks[water].amount.to_bits(),
            curve.stocks[water].amount.to_bits(),
            "{water} did not move under the curve"
        );
    }
}

/// Records every forcing name asked of it.
struct Recording<'a> {
    inner: &'a dyn simcore::environment::Environment,
    asked: std::cell::RefCell<Vec<String>>,
}

impl simcore::environment::Environment for Recording<'_> {
    fn get(&self, var: &str) -> Result<f64, simcore::error::SimError> {
        self.asked.borrow_mut().push(var.to_string());
        self.inner.get(var)
    }
}

/// **The water and only the water**, by what each flow and aux process asks for, on the open field
/// and a sealed jar grown 150 days, under the reader with the real curve: every flow with a water leg
/// reads the within-day air and never `temp` — except the roots' capture, whose pace is growth's — and
/// so does the soil's water account; no other flow and no other aux process (development, roots —
/// growth) asks for either within-day name.
#[test]
fn the_water_reads_the_within_day_air_and_nothing_else_does() {
    for s in [DEFAULT_SCENARIO, sealed_chamber_scenario()] {
        let state = run(&s, AirTemperatureRead::DailyMean, daily_mean(&s), 150)
            .expect("grown");
        let p = with_air_temperature(params::biosphere(), AirTemperatureRead::WithinDay);
        let (_, integrator, _) =
            season_setup_composed(&s, 1, &p, &|s, p| build_season_with(s, p)).expect("setup");
        let resolver = SourceResolver::new(
            weather_forcings_lab(&s, 1, WeatherRadiation::SunPath, WeatherAir::WithinDay).unwrap(),
            weather_shared(&s),
        )
        .unwrap();
        let mut at = state.clone();
        at.n = (150 * STEPS_PER_DAY + STEPS_PER_DAY / 2) as u64; // noon
        let env = resolver.bind(&at, BIO_DT);
        let within = [AIR_TEMP_WITHIN_DAY_VAR, VPD_WITHIN_DAY_VAR];
        let mut water_readers = 0;
        for flow in integrator.registry().flows() {
            let rec = Recording { inner: &env, asked: Default::default() };
            let legs = flow.evaluate(&at, &rec, BIO_DT).expect("flow").legs;
            let asked = rec.asked.into_inner();
            let water = legs.iter().any(|l| {
                at.stocks[&l.stock].quantity == simcore::quantities::Quantity::Water
            });
            let reads = |v: &str| asked.iter().any(|a| a == v);
            // The one water flow paced by growth: new roots capture the water of the soil they
            // reach, and how fast they deepen reads the daily mean, as all growth does.
            let growth_paced = flow.id() == "biosphere.root_zone_capture";
            if water && !growth_paced {
                assert!(!reads(TEMP_VAR), "{}: a water flow reads `temp`", flow.id());
                if reads(AIR_TEMP_WITHIN_DAY_VAR) {
                    water_readers += 1;
                }
            } else {
                for v in within {
                    assert!(!reads(v), "{}: a flow with no water leg reads {v}", flow.id());
                }
            }
        }
        // The crop's transpiration; and in the jar its dehumidifier too.
        assert_eq!(water_readers, if s.sealed { 2 } else { 1 }, "{}", s.sealed);
        // The soil's water account (its top layer and evaporation clocks) is a water reader; every
        // other aux process is growth's.
        let mut soil_account = 0;
        for aux in integrator.registry().aux_processes() {
            let rec = Recording { inner: &env, asked: Default::default() };
            aux.evaluate(&at, &rec, BIO_DT).expect("aux");
            let asked = rec.asked.into_inner();
            let reads = |v: &str| asked.iter().any(|a| a == v);
            if aux.id() == "biosphere.soil_surface" {
                soil_account += 1;
                assert!(reads(AIR_TEMP_WITHIN_DAY_VAR) && !reads(TEMP_VAR), "{}", aux.id());
            } else {
                for v in within {
                    assert!(!reads(v), "{}: an aux process reads {v}", aux.id());
                }
            }
        }
        assert_eq!(soil_account, 1);
    }
}
