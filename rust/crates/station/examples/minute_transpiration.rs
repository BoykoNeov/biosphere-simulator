//! Step 3c — the plants' water loss on the minute step, measured against its committed
//! predictions (`docs/plans/post-roadmap-room-temperature.md` §20), as one command:
//!
//! ```text
//! cargo run --release -q -p station --example minute_transpiration
//! ```
//!
//! One season (305 days) of the separate-air station (BVAD chamber, fan 0.2 mol/s, gas exchange
//! on the minute step), with transpiration and the chamber's condenser on the plant step (§18c's
//! runs, the control) or on the minute step; the plant chamber at the weather's temperature or
//! held at 22 °C (a plant-side forcing override, as `examples/watering.rs`); watering off or on;
//! and vapour crossing or not (§20's M6 control).
//!
//! Water stress and the root zone's fill are read on EVERY fast step (from the minute's start),
//! so a dip inside a plant window is seen whichever step transpiration runs on. Transpiration is
//! counted on whichever step runs it. It writes nothing and takes no decision.

use std::time::Instant;

use domains::biosphere::science::{soil_water_stress, transpirable_capacity};
use domains::biosphere::stocks::{
    CONDENSATE, LEAF_C, ROOTED_DEPTH, ROOT_C, SOIL_WATER, STEM_C, STORAGE_C, SUBSOIL_WATER,
    WATER_SOURCE, WATER_VAPOR,
};
use domains::crew::WATER_STORE;
use domains::eclss::CABIN_H2O;
use domains::params;
use simcore::integrator::EulerIntegrator;
use simcore::registry::Registry;
use simcore::state::State;
use station::air_split::{
    build_split_station, split_scenario, AirSplit, AIR_EXCHANGE, BVAD_CHAMBER_AIR_MOL, WATERING,
};
use station::driver::{DayOrder, Side, TwoRate};
use station::gas_exchange::GasExchangeStep;
use station::params as station_params;
use station::scenario::sealed_station_scenario;
use station::sealed::{sealed_bio_resolver, sealed_fast_resolver, sealed_reset_hook};
use station::water::BRINE;

const TRANSPIRATION: &str = "biosphere.transpiration";

fn plant_c(s: &State) -> f64 {
    [LEAF_C, STEM_C, ROOT_C, STORAGE_C]
        .iter()
        .map(|id| s.stocks[*id].amount)
        .sum()
}

fn plant_water(s: &State) -> f64 {
    [
        SOIL_WATER,
        SUBSOIL_WATER,
        CONDENSATE,
        WATER_VAPOR,
        WATER_SOURCE,
    ]
    .iter()
    .map(|id| s.stocks.get(*id).map_or(0.0, |x| x.amount))
    .sum()
}

#[derive(Clone, Copy)]
struct Case {
    warm: bool,
    water_step: GasExchangeStep,
    watering: bool,
    vapour_crosses: bool,
}

#[derive(Default)]
struct Reading {
    watered: f64,
    /// The fan's net vapour flow INTO the cabin (kg): positive = the plants export.
    exported: f64,
    transpired: f64,
    ftsw_min: f64,
    stress_min: f64,
    stressed_minutes: u64,
    minutes: u64,
    rationed: (u64, u64),
    events: usize,
}

/// The soil leg (kg withdrawn, positive) of flow `id` in `reg`, if `reg` holds it.
fn withdrawn(
    reg: &Registry,
    id: &str,
    before: &State,
    env: &dyn simcore::environment::Environment,
    dt: f64,
) -> f64 {
    reg.flows()
        .iter()
        .filter(|f| f.id() == id)
        .map(|f| {
            f.evaluate(before, env, dt)
                .expect("evaluate")
                .legs
                .iter()
                .filter(|l| l.stock == SOIL_WATER)
                .map(|l| -l.amount)
                .sum::<f64>()
        })
        .sum()
}

fn season(case: Case) -> (State, State, Reading) {
    // The warm case (since slice 3a): the chamber with no cold period, held at 22 °C from day 0
    // (22.05 °C as the plants read it). Until 3a it was a plant-side 22 °C forcing, now refused.
    let mut base = sealed_station_scenario();
    if case.warm {
        base.cold.days = 0;
    }
    let split = AirSplit {
        chamber_air_mol: BVAD_CHAMBER_AIR_MOL,
        fan_mol_per_s: 0.2,
        vapour_crosses: case.vapour_crosses,
        gas_exchange: GasExchangeStep::Minute,
        watering: case.watering,
        transpiration: case.water_step,
    };
    let (state, bio, fast) = build_split_station(
        &params::charge(),
        &params::thermal(),
        &params::crew(),
        &params::eclss(),
        &station_params::water_recovery(),
        &station_params::lamp(),
        &station_params::harvest(),
        &base,
        &split,
    )
    .expect("build_split_station");
    let scenario = split_scenario(&base, &split);
    let start = state.clone();
    let bio_r = sealed_bio_resolver(&station_params::lamp(), &scenario).expect("bio");
    let fast_r = sealed_fast_resolver(&params::charge(), &scenario).expect("fast");
    let reset = sealed_reset_hook(&scenario);
    let (bio, fast) = (EulerIntegrator::new(bio), EulerIntegrator::new(fast));
    let two = TwoRate {
        slow: &bio,
        fast: &fast,
        slow_resolver: &bio_r,
        fast_resolver: &fast_r,
        steps_per_day: scenario.steps_per_day,
        slow_steps_per_day: scenario.bio_steps_per_day,
        slow_dt: scenario.bio_dt,
        fast_dt: scenario.cabin_dt,
        slow_reset: Some(&*reset),
    };
    let mut r = Reading {
        stress_min: 1.0,
        ftsw_min: f64::INFINITY,
        ..Reading::default()
    };
    let b = scenario.bio;
    let mut observe = |side: Side, before: &State, _after: &State| match side {
        Side::Reset => {}
        Side::Slow => {
            let env = bio_r.bind(before, scenario.bio_dt);
            r.transpired += withdrawn(bio.registry(), TRANSPIRATION, before, &env, scenario.bio_dt);
            for flow in bio.registry().flows() {
                if flow.id() == WATERING {
                    let res = flow
                        .evaluate(before, &env, scenario.bio_dt)
                        .expect("watering");
                    r.watered += res
                        .legs
                        .iter()
                        .filter(|l| l.stock == SOIL_WATER)
                        .map(|l| l.amount)
                        .sum::<f64>();
                }
            }
        }
        Side::Fast => {
            let depth = before.aux.get(ROOTED_DEPTH).copied().unwrap_or(0.0);
            let soil = before.stocks[SOIL_WATER].amount;
            let f = soil_water_stress(soil, depth, b.soil_extractable_water, b.ground_area, b.wssg);
            let ttsw = transpirable_capacity(depth, b.soil_extractable_water, b.ground_area);
            r.ftsw_min = r.ftsw_min.min(soil / ttsw);
            r.stress_min = r.stress_min.min(f);
            r.stressed_minutes += u64::from(f < 1.0);
            r.minutes += 1;
            let env = fast_r.bind(before, scenario.cabin_dt);
            r.transpired += withdrawn(
                fast.registry(),
                TRANSPIRATION,
                before,
                &env,
                scenario.cabin_dt,
            );
            for flow in fast.registry().flows() {
                if flow.id() == AIR_EXCHANGE {
                    let res = flow.evaluate(before, &env, scenario.cabin_dt).expect("fan");
                    r.exported += res
                        .legs
                        .iter()
                        .filter(|l| l.stock == CABIN_H2O)
                        .map(|l| l.amount)
                        .sum::<f64>();
                }
            }
        }
    };
    let (states, totals) = two
        .run(
            DayOrder::Interleaved,
            state,
            scenario.season_days,
            &mut observe,
        )
        .expect("two-rate run");
    r.rationed = (totals.slow_rationed, totals.fast_rationed);
    r.events = totals.events.len();
    (start, states.last().expect("a day").clone(), r)
}

fn main() {
    println!(
        "one season; separate air, BVAD chamber, fan 0.2 mol/s, minute gas exchange; stress and \
         FTSW read every minute"
    );
    let cases = [
        (
            "weather | plant step | off",
            Case {
                warm: false,
                water_step: GasExchangeStep::PlantStep,
                watering: false,
                vapour_crosses: true,
            },
        ),
        (
            "weather | minute     | off",
            Case {
                warm: false,
                water_step: GasExchangeStep::Minute,
                watering: false,
                vapour_crosses: true,
            },
        ),
        (
            "weather | minute     | on ",
            Case {
                warm: false,
                water_step: GasExchangeStep::Minute,
                watering: true,
                vapour_crosses: true,
            },
        ),
        (
            "22 °C   | plant step | off",
            Case {
                warm: true,
                water_step: GasExchangeStep::PlantStep,
                watering: false,
                vapour_crosses: true,
            },
        ),
        (
            "22 °C   | minute     | off",
            Case {
                warm: true,
                water_step: GasExchangeStep::Minute,
                watering: false,
                vapour_crosses: true,
            },
        ),
        (
            "22 °C   | minute     | on ",
            Case {
                warm: true,
                water_step: GasExchangeStep::Minute,
                watering: true,
                vapour_crosses: true,
            },
        ),
        (
            "weather | plant step | off | NO vapour crossing",
            Case {
                warm: false,
                water_step: GasExchangeStep::PlantStep,
                watering: false,
                vapour_crosses: false,
            },
        ),
        (
            "weather | minute     | off | NO vapour crossing",
            Case {
                warm: false,
                water_step: GasExchangeStep::Minute,
                watering: false,
                vapour_crosses: false,
            },
        ),
    ];
    for (name, case) in cases {
        let t0 = Instant::now();
        let (s0, end, r) = season(case);
        let secs = t0.elapsed().as_secs_f64();
        println!(
            "{name:<48} | watered {:8.3} kg | fan to cabin {:+9.3} kg | transpired {:8.2} kg | plant \
             water {:8.3} → {:9.4} (soil {:7.3}, subsoil {:7.3}, condensate {:6.3}, vapour {:.4}) | crew \
             store {:+9.3} kg, brine {:+8.3} kg | FTSW min {:.4} | stress min {:.4} on {}/{} min \
             ({:.1} %) | plant C {:7.3} | rationed {:?}, events {} | {:.1} s",
            r.watered,
            r.exported,
            r.transpired,
            plant_water(&s0),
            plant_water(&end),
            end.stocks[SOIL_WATER].amount,
            end.stocks[SUBSOIL_WATER].amount,
            end.stocks[CONDENSATE].amount,
            end.stocks[WATER_VAPOR].amount,
            end.stocks[WATER_STORE].amount - s0.stocks[WATER_STORE].amount,
            end.stocks[BRINE].amount - s0.stocks[BRINE].amount,
            r.ftsw_min,
            r.stress_min,
            r.stressed_minutes,
            r.minutes,
            100.0 * r.stressed_minutes as f64 / r.minutes as f64,
            plant_c(&end),
            r.rationed,
            r.events,
            secs,
        );
    }
}
