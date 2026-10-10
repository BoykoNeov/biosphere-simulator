//! Step 3c — watering the plants from the crew's supply, measured against its committed
//! predictions (`docs/plans/post-roadmap-room-temperature.md` §18), as one command:
//!
//! ```text
//! cargo run --release -q -p station --example watering
//! ```
//!
//! One season (305 days) of the separate-air station (BVAD chamber, fan 0.2 mol/s, vapour
//! crossing, gas exchange on the minute step), four ways: the plant chamber at the weather's
//! temperature or held at 22 °C (a plant-side forcing override, as `warm_room_arrest` does —
//! the decided setpoint, not yet a room with heat books), each with watering off and on.
//!
//! It writes nothing and takes no decision.

use domains::biosphere::science::soil_water_stress;
use domains::biosphere::stocks::{
    CONDENSATE, LEAF_C, ROOTED_DEPTH, ROOT_C, SOIL_WATER, STEM_C, STORAGE_C, SUBSOIL_WATER,
    TOP_SOIL_WATER, WATER_SOURCE, WATER_VAPOR,
};
use domains::crew::WATER_STORE;
use domains::eclss::CABIN_H2O;
use domains::params;
use simcore::integrator::EulerIntegrator;
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

#[derive(Default)]
struct Reading {
    watered: f64,
    exported: f64,
    stress_min: f64,
    /// The root zone's lowest fill, `ATSW / TTSW` (FTSW), at plant-step starts.
    ftsw_min: f64,
    stressed_steps: u64,
    /// Σ transpiration (kg), Σ the part that reached the chamber's air, and the plant steps
    /// where the chamber's headroom held some back (the cap R2 names).
    transpired: f64,
    to_air: f64,
    capped_steps: u64,
    plant_steps: u64,
    /// Plant steps whose top layer held no more than Stage I's wet threshold, so the soil's
    /// evaporation could not restart (the soil-surface account, `post-roadmap-soil-evaporation.md` §17).
    top_dry_steps: u64,
    rationed: (u64, u64),
    events: usize,
}

fn season(warm: bool, watering: bool) -> (State, State, Reading) {
    // The warm case (since slice 3a): the chamber with no cold period, held at 22 °C from day 0
    // (22.05 °C as the plants read it). Until 3a it was a plant-side 22 °C forcing, now refused.
    let mut base = sealed_station_scenario();
    if warm {
        base.cold.days = 0;
    }
    let split = AirSplit {
        chamber_air_mol: BVAD_CHAMBER_AIR_MOL,
        fan_mol_per_s: 0.2,
        vapour_crosses: true,
        gas_exchange: GasExchangeStep::Minute,
        watering,
        transpiration: GasExchangeStep::PlantStep,
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
    let top_wet = domains::biosphere::params::biosphere().transp.soil.top_layer_wet * b.ground_area;
    let mut observe = |side: Side, before: &State, _after: &State| match side {
        Side::Reset => {}
        Side::Slow => {
            let f = soil_water_stress(
                before.stocks[SOIL_WATER].amount,
                before.aux.get(ROOTED_DEPTH).copied().unwrap_or(0.0),
                b.soil_extractable_water,
                b.ground_area,
                b.wssg,
            );
            let ttsw = domains::biosphere::science::transpirable_capacity(
                before.aux.get(ROOTED_DEPTH).copied().unwrap_or(0.0),
                b.soil_extractable_water,
                b.ground_area,
            );
            r.ftsw_min = r.ftsw_min.min(before.stocks[SOIL_WATER].amount / ttsw);
            r.stress_min = r.stress_min.min(f);
            r.stressed_steps += u64::from(f < 1.0);
            let env = bio_r.bind(before, scenario.bio_dt);
            r.plant_steps += 1;
            r.top_dry_steps +=
                u64::from(before.aux.get(TOP_SOIL_WATER).copied().unwrap_or(0.0) <= top_wet);
            for flow in bio.registry().flows() {
                if flow.id() == TRANSPIRATION {
                    let res = flow
                        .evaluate(before, &env, scenario.bio_dt)
                        .expect("transpiration");
                    let (mut flux, mut air) = (0.0, 0.0);
                    for leg in &res.legs {
                        if leg.amount < 0.0 {
                            flux -= leg.amount;
                        } else if leg.stock == WATER_VAPOR {
                            air += leg.amount;
                        }
                    }
                    r.transpired += flux;
                    r.to_air += air;
                    r.capped_steps += u64::from(air < flux);
                }
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
            let env = fast_r.bind(before, scenario.cabin_dt);
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
    println!("one season; separate air, BVAD chamber, fan 0.2 mol/s, vapour crossing, minute gas exchange");
    let mut ends: Vec<String> = Vec::new();
    for (warm, watering) in [(false, false), (false, true), (true, false), (true, true)] {
        let (s0, end, r) = season(warm, watering);
        ends.push(simcore::snapshot::from_engine(&end).to_json());
        let name = format!(
            "{} chamber, watering {}",
            if warm { "22 °C" } else { "cold prog" },
            if watering { "on " } else { "off" }
        );
        println!(
            "{name:<30} | watered {:8.3} kg | fan export {:8.3} kg | plant water {:8.3} → {:8.3} \
             (soil {:7.3}→{:7.3}, subsoil {:7.3}→{:7.3}) | crew store {:+9.3} kg, brine {:+7.3} kg \
             | transp {:8.3} kg, to air {:6.3} kg, capped on {} of {} plant steps              | top layer dry on {} | FTSW min {:.4} | stress min {:.4} on {} steps | plant C {:7.3} | rationed {:?}, events {}",
            r.watered,
            r.exported,
            plant_water(&s0),
            plant_water(&end),
            s0.stocks[SOIL_WATER].amount,
            end.stocks[SOIL_WATER].amount,
            s0.stocks[SUBSOIL_WATER].amount,
            end.stocks[SUBSOIL_WATER].amount,
            end.stocks[WATER_STORE].amount - s0.stocks[WATER_STORE].amount,
            end.stocks[BRINE].amount - s0.stocks[BRINE].amount,
            r.transpired,
            r.to_air,
            r.capped_steps,
            r.plant_steps,
            r.top_dry_steps,
            r.ftsw_min,
            r.stress_min,
            r.stressed_steps,
            plant_c(&end),
            r.rationed,
            r.events,
        );
    }
    println!(
        "end state, watering on vs off: weather chamber {}, 22 °C chamber {}",
        if ends[0] == ends[1] {
            "byte-identical"
        } else {
            "DIFFERS"
        },
        if ends[2] == ends[3] {
            "byte-identical"
        } else {
            "DIFFERS"
        },
    );
}
