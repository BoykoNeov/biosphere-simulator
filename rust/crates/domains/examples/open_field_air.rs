//! **The open field gets a night, step 2: the air** — the probe
//! (`docs/plans/post-roadmap-open-field-night.md` §7).
//!
//! Runs every weather-lit biosphere golden's build, from the loader's params, under five settings:
//! the daily mean (the reference), the sun's path alone (step 1), the within-day air alone, both
//! (the measurement asked for), and both with the curve shifted onto the day's observed mean (a
//! sensitivity row). For each it prints the water the `Transpiration` flow drew from the root zone,
//! split by lit and dark steps (net radiation > 0 or not); the dehumidifier's draw (sealed builds),
//! split the same way; the open field's own crop / soil sinks; rationing and events; and every final
//! value that moved against the daily mean, grouped water / not water.
//!
//! The draws are the flows evaluated on each observed state before its step (the final state is not
//! stepped, so not counted) — what the integrator applies when nothing is rationed, which each
//! line's `rationed` shows.
//!
//! ```text
//! cargo run --release -q -p domains --example open_field_air
//! ```

use std::collections::BTreeMap;

use domains::biosphere::params;
use domains::biosphere::science::AirTemperatureRead;
use domains::biosphere::stocks::{
    CONDENSATE, RN_VAR, SOIL_EVAP_SINK, SOIL_WATER, VAPOR_SINK, WATER_VAPOR,
};
use domains::biosphere::system::{weather_resolver_lab, WeatherAir, WeatherRadiation};
use domains::biosphere::{
    consumer_chamber_scenario, perennial_chamber_scenario, run_perennial_with, run_season,
    sealed_chamber_scenario, season_setup_with, season_steps, steps_for_years, SeasonScenario,
    BIO_DT, CONSUMER_CHAMBER_YEARS, DEFAULT_SCENARIO, PERENNIAL_CHAMBER_YEARS,
    SEALED_CHAMBER_YEARS,
};
use domains::lab::with_air_temperature;
use simcore::environment::Environment;
use simcore::quantities::Quantity;
use simcore::state::State;

#[derive(Clone, Copy)]
struct Setting {
    label: &'static str,
    radiation: WeatherRadiation,
    air: WeatherAir,
}

const SETTINGS: [Setting; 5] = [
    Setting {
        label: "daily mean",
        radiation: WeatherRadiation::DailyMean,
        air: WeatherAir::DailyMean,
    },
    Setting {
        label: "light only",
        radiation: WeatherRadiation::SunPath,
        air: WeatherAir::DailyMean,
    },
    Setting {
        label: "air only",
        radiation: WeatherRadiation::DailyMean,
        air: WeatherAir::WithinDay,
    },
    Setting {
        label: "both",
        radiation: WeatherRadiation::SunPath,
        air: WeatherAir::WithinDay,
    },
    Setting {
        label: "both, shifted",
        radiation: WeatherRadiation::SunPath,
        air: WeatherAir::WithinDayShifted,
    },
];

struct Run {
    lit_draw: f64,
    dark_draw: f64,
    lit_condensed: f64,
    dark_condensed: f64,
    rationed: u64,
    events: usize,
    end: State,
}

fn run(scenario: &SeasonScenario, years: usize, perennial: bool, setting: Setting) -> Run {
    let read = match setting.air {
        WeatherAir::DailyMean => AirTemperatureRead::DailyMean,
        _ => AirTemperatureRead::WithinDay,
    };
    let p = with_air_temperature(params::biosphere(), read);
    let (state, integrator, _) = season_setup_with(scenario, years, &p).expect("setup");
    let resolver =
        weather_resolver_lab(scenario, years, setting.radiation, setting.air).expect("weather");
    let steps = steps_for_years(years);
    let flows = integrator.registry().flows();
    let transpiration = flows
        .iter()
        .find(|f| f.type_name() == "Transpiration")
        .expect("one transpiration flow");
    let condensation = flows.iter().find(|f| f.type_name() == "Condensation");
    let (mut lit_draw, mut dark_draw, mut lit_condensed, mut dark_condensed) = (0.0, 0.0, 0.0, 0.0);
    let mut seen = 0usize;
    let mut observe = |s: &State| {
        seen += 1;
        if seen > steps {
            return; // the final state is not stepped
        }
        let env = resolver.bind(s, BIO_DT);
        let lit = env.get(RN_VAR).expect("rn") > 0.0;
        let drawn: f64 = -transpiration
            .evaluate(s, &env, BIO_DT)
            .expect("transpiration")
            .legs
            .iter()
            .filter(|l| l.stock == SOIL_WATER)
            .map(|l| l.amount)
            .sum::<f64>();
        let condensed: f64 = condensation.map_or(0.0, |f| {
            f.evaluate(s, &env, BIO_DT)
                .expect("condensation")
                .legs
                .iter()
                .filter(|l| l.stock == CONDENSATE)
                .map(|l| l.amount)
                .sum()
        });
        if lit {
            lit_draw += drawn;
            lit_condensed += condensed;
        } else {
            dark_draw += drawn;
            dark_condensed += condensed;
        }
    };
    let (end, rationed, events) = if perennial {
        run_perennial_with(
            &integrator,
            state,
            scenario,
            &p,
            &resolver,
            BIO_DT,
            steps,
            season_steps(),
            &mut observe,
        )
    } else {
        run_season(&integrator, state, &resolver, BIO_DT, steps, None, &mut observe)
    }
    .expect("run");
    Run {
        lit_draw,
        dark_draw,
        lit_condensed,
        dark_condensed,
        rationed,
        events: events.len(),
        end,
    }
}

fn values(s: &State) -> BTreeMap<String, (f64, bool)> {
    s.stocks
        .iter()
        .map(|(id, st)| (id.clone(), (st.amount, st.quantity == Quantity::Water)))
        .chain(s.aux.iter().map(|(k, v)| (format!("aux:{k}"), (*v, false))))
        .collect()
}

fn main() {
    let cases: [(&str, SeasonScenario, usize, bool); 4] = [
        ("open field (season_euler)", DEFAULT_SCENARIO, 1, false),
        ("sealed chamber", sealed_chamber_scenario(), SEALED_CHAMBER_YEARS, false),
        ("perennial chamber", perennial_chamber_scenario(), PERENNIAL_CHAMBER_YEARS, true),
        ("consumer chamber", consumer_chamber_scenario(), CONSUMER_CHAMBER_YEARS, true),
    ];
    for (name, scenario, years, perennial) in cases {
        println!("== {name}, {years} yr ==");
        let runs: Vec<Run> =
            SETTINGS.iter().map(|s| run(&scenario, years, perennial, *s)).collect();
        let base = &runs[0];
        let base_total = base.lit_draw + base.dark_draw;
        for (setting, r) in SETTINGS.iter().zip(&runs) {
            let total = r.lit_draw + r.dark_draw;
            println!(
                "  {:>13}: drawn {total:9.3} kg ({:+6.2} %; lit {:.3}, dark {:.3}); condensed lit {:.3}, \
                 dark {:.3} kg; rationed {}, events {}",
                setting.label,
                100.0 * (total - base_total) / base_total,
                r.lit_draw,
                r.dark_draw,
                r.lit_condensed,
                r.dark_condensed,
                r.rationed,
                r.events,
            );
            if !scenario.sealed {
                println!(
                    "  {:>13}  crop {:.3} kg, soil {:.3} kg",
                    "",
                    r.end.stocks[VAPOR_SINK].amount,
                    r.end.stocks[SOIL_EVAP_SINK].amount
                );
            } else {
                println!(
                    "  {:>13}  air vapour at the end {:.6} kg",
                    "",
                    r.end.stocks[WATER_VAPOR].amount
                );
            }
        }
        let a = values(&base.end);
        for (setting, r) in SETTINGS.iter().zip(&runs).skip(1) {
            let b = values(&r.end);
            let (mut water, mut other) = (0, Vec::new());
            for (key, (va, is_water)) in &a {
                let vb = b[key].0;
                if va.to_bits() != vb.to_bits() {
                    if *is_water {
                        water += 1;
                    } else {
                        let rel = if *va != 0.0 { (vb - va) / va.abs() } else { f64::NAN };
                        other.push(format!("{key} {rel:+.3e}"));
                    }
                }
            }
            println!(
                "  {:>13} vs daily mean: {water} water values moved; not water: {}",
                setting.label,
                if other.is_empty() { "none".to_string() } else { other.join(", ") }
            );
        }
        println!();
    }
}
