//! **The open field gets a night** — the probe (`docs/plans/post-roadmap-open-field-night.md` §2).
//!
//! Runs every weather-lit biosphere golden's build under both [`WeatherRadiation`] forms, from the
//! loader's params, and prints: the water the `Transpiration` flow drew from the root zone (crop and
//! soil together), split by whether the step's net radiation was > 0 (lit) or not (dark); the open
//! field's own crop / soil sinks; rationing and events; and every final stock and aux value that the
//! form moved, with its relative change.
//!
//! The draw is the flow evaluated on each observed state before its step (the final state is not
//! stepped, so not counted; a re-sow step is read on the state before its re-sow) — what the integrator applies when nothing is rationed, which each line's
//! `rationed` shows.
//!
//! ```text
//! cargo run --release -q -p domains --example open_field_night
//! ```

use std::collections::BTreeMap;

use domains::biosphere::params;
use domains::biosphere::stocks::{RN_VAR, SOIL_EVAP_SINK, SOIL_WATER, TOP_SOIL_WATER, VAPOR_SINK};
use domains::biosphere::system::{weather_resolver_with, WeatherRadiation};
use domains::biosphere::{
    consumer_chamber_scenario, perennial_chamber_scenario, run_perennial_with, run_season,
    sealed_chamber_scenario, season_setup_with, season_steps, steps_for_years, SeasonScenario,
    BIO_DT, CONSUMER_CHAMBER_YEARS, DEFAULT_SCENARIO, PERENNIAL_CHAMBER_YEARS,
    SEALED_CHAMBER_YEARS,
};
use simcore::environment::Environment;
use simcore::state::State;

struct Run {
    lit_draw: f64,
    dark_draw: f64,
    dark_steps: u64,
    /// The open field's soil evaporation (its own sink's leg): all of it, and on the steps it took
    /// the whole top layer (the layer's room bound it), with their count.
    soil: f64,
    soil_emptying: f64,
    emptying_steps: u64,
    rationed: u64,
    events: usize,
    end: State,
}

fn run(
    scenario: &SeasonScenario,
    years: usize,
    perennial: bool,
    radiation: WeatherRadiation,
) -> Run {
    let p = params::biosphere();
    let (state, integrator, _) = season_setup_with(scenario, years, &p).expect("setup");
    let resolver = weather_resolver_with(scenario, years, radiation).expect("weather");
    let steps = steps_for_years(years);
    let flow = integrator
        .registry()
        .flows()
        .iter()
        .find(|f| f.type_name() == "Transpiration")
        .expect("one transpiration flow");
    let (mut lit_draw, mut dark_draw, mut dark_steps, mut seen) = (0.0, 0.0, 0u64, 0usize);
    let (mut soil, mut soil_emptying, mut emptying_steps) = (0.0, 0.0, 0u64);
    let mut observe = |s: &State| {
        seen += 1;
        if seen > steps {
            return; // the final state is not stepped
        }
        let env = resolver.bind(s, BIO_DT);
        let legs = flow.evaluate(s, &env, BIO_DT).expect("flow").legs;
        let drawn: f64 = -legs
            .iter()
            .filter(|l| l.stock == SOIL_WATER)
            .map(|l| l.amount)
            .sum::<f64>();
        let to_soil_sink: f64 = legs
            .iter()
            .filter(|l| l.stock == SOIL_EVAP_SINK)
            .map(|l| l.amount)
            .sum();
        soil += to_soil_sink;
        let top = s.aux.get(TOP_SOIL_WATER).copied().unwrap_or(0.0);
        if to_soil_sink > 0.0 && to_soil_sink >= top * (1.0 - 1e-12) {
            soil_emptying += to_soil_sink;
            emptying_steps += 1;
        }
        if env.get(RN_VAR).expect("rn") > 0.0 {
            lit_draw += drawn;
        } else {
            dark_draw += drawn;
            dark_steps += 1;
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
        run_season(
            &integrator,
            state,
            &resolver,
            BIO_DT,
            steps,
            None,
            &mut observe,
        )
    }
    .expect("run");
    Run {
        lit_draw,
        dark_draw,
        dark_steps,
        soil,
        soil_emptying,
        emptying_steps,
        rationed,
        events: events.len(),
        end,
    }
}

fn values(s: &State) -> BTreeMap<String, f64> {
    s.stocks
        .iter()
        .map(|(id, st)| (id.clone(), st.amount))
        .chain(s.aux.iter().map(|(k, v)| (format!("aux:{k}"), *v)))
        .collect()
}

fn main() {
    let cases: [(&str, SeasonScenario, usize, bool); 4] = [
        ("open field (season_euler)", DEFAULT_SCENARIO, 1, false),
        (
            "sealed chamber",
            sealed_chamber_scenario(),
            SEALED_CHAMBER_YEARS,
            false,
        ),
        (
            "perennial chamber",
            perennial_chamber_scenario(),
            PERENNIAL_CHAMBER_YEARS,
            true,
        ),
        (
            "consumer chamber",
            consumer_chamber_scenario(),
            CONSUMER_CHAMBER_YEARS,
            true,
        ),
    ];
    for (name, scenario, years, perennial) in cases {
        let mean = run(&scenario, years, perennial, WeatherRadiation::DailyMean);
        let sun = run(&scenario, years, perennial, WeatherRadiation::SunPath);
        println!("== {name}, {years} yr ==");
        for (label, r) in [("daily mean", &mean), ("sun path", &sun)] {
            let total = r.lit_draw + r.dark_draw;
            println!(
                "  {label:>10}: drawn {total:.4} kg (lit {:.4}, dark {:.4} over {} steps; dark share {:.1} %); rationed {}, events {}",
                r.lit_draw,
                r.dark_draw,
                r.dark_steps,
                100.0 * r.dark_draw / total,
                r.rationed,
                r.events,
            );
        }
        let (tm, ts) = (mean.lit_draw + mean.dark_draw, sun.lit_draw + sun.dark_draw);
        println!(
            "  drawn: {:+.4} kg ({:+.2} %)",
            ts - tm,
            100.0 * (ts - tm) / tm
        );
        if !scenario.sealed {
            for (label, r) in [("daily mean", &mean), ("sun path", &sun)] {
                println!(
                    "  {label:>10}: soil evaporation {:.4} kg, of it {:.4} kg on {} steps that emptied the top layer",
                    r.soil, r.soil_emptying, r.emptying_steps
                );
            }
            for id in [VAPOR_SINK, SOIL_EVAP_SINK] {
                let (a, b) = (mean.end.stocks[id].amount, sun.end.stocks[id].amount);
                println!("  {id}: {a:.4} -> {b:.4} ({:+.2} %)", 100.0 * (b - a) / a);
            }
        }
        let (a, b) = (values(&mean.end), values(&sun.end));
        let mut moved = 0;
        for (key, va) in &a {
            let vb = b[key];
            if va.to_bits() != vb.to_bits() {
                moved += 1;
                let rel = if *va != 0.0 {
                    (vb - va) / va.abs()
                } else {
                    f64::NAN
                };
                println!("    {key}: {va:.6e} -> {vb:.6e} ({rel:+.3e})");
            }
        }
        println!("  {moved} of {} end values moved\n", a.len());
    }
}
