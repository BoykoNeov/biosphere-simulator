//! Step 3c slice 2a — the separate-air station measured against its committed predictions
//! (`docs/plans/post-roadmap-room-temperature.md` §13 / §13a), as one command:
//!
//! ```text
//! cargo run --release -q -p station --example air_split
//! ```
//!
//! One season (305 days) of each case, all through the two-rate observer:
//!
//! * `shared` — the frozen sealed station (its end state is compared with the 1220-day golden
//!   only by the baseline example; here it is the comparison run);
//! * `split` cases — [`station::air_split`] at the BVAD chamber (27.66 mol) with the fan at
//!   0.1 / 0.2 / 0.4 mol/s, vapour crossing off and on; and the big-chamber control (9500 mol,
//!   10 mol/s, vapour off).
//!
//! It writes nothing and takes no decision.

use domains::biosphere::params as bio_params;
use domains::biosphere::readouts::withdrawal_demand;
use domains::biosphere::science::{
    humidity_target_kg, saturation_vapour_kg, H2O_MOLAR_MASS_KG_PER_MOL, N2_MOLAR_MASS_KG_PER_MOL,
};
use domains::biosphere::stocks::{
    CARBON_POOL, CHAMBER_INERT, CONDENSATE, LEAF_C, O2_POOL, ROOT_C, SOIL_WATER, STEM_C, STORAGE_C,
    SUBSOIL_WATER, TEMP_VAR, WATER_SOURCE, WATER_VAPOR,
};
use domains::crew::WATER_STORE;
use domains::eclss::{CABIN_CO2, CABIN_H2O, CABIN_O2};
use domains::params;
use simcore::environment::Environment;
use simcore::flow::FlowResult;
use simcore::integrator::EulerIntegrator;
use simcore::registry::Registry;
use simcore::state::State;
use station::air_split::{
    build_split_station, split_scenario, AirSplit, AIR_EXCHANGE, BVAD_CHAMBER_AIR_MOL,
    CABIN_AIR_MOL, CABIN_INERT,
};
use station::driver::{DayOrder, Side, TwoRate};
use station::gas_exchange::{gas_exchange_on_fast_step, GasExchangeStep, CARBON_BUDGET_FLOWS};
use station::params as station_params;
use station::scenario::{sealed_station_scenario, SealedStationScenario};
use station::sealed::{
    build_sealed_station, sealed_bio_resolver, sealed_fast_resolver, sealed_reset_hook,
};

const TRANSPIRATION: &str = "biosphere.transpiration";

/// What one season reads.
#[derive(Default)]
struct Reading {
    /// Σ the crop's gross CO₂ withdrawals (mol).
    co2_gross: f64,
    /// Σ chamber relative humidity at slow-step starts, and the step count.
    rh: f64,
    slow_steps: u64,
    /// Σ transpiration (kg), Σ the part that reached the air, and steps where the source
    /// bound held some back.
    transpired: f64,
    to_air: f64,
    bound_binds: u64,
    /// Σ water the fan carried chamber → cabin (kg), and fast steps where it ran the other way.
    exported: f64,
    imported_steps: u64,
    /// The cabin CO₂ (the separate cabin's, or the shared pool) at fast-step starts: min / max.
    cabin_co2_min: f64,
    cabin_co2_max: f64,
    rationed: (u64, u64),
    end: Option<State>,
    /// The largest |total gas / reference air − 1| at fast-step starts, per room (the cabin's
    /// only when it has its own air), and the season's net inert fill the fan carried into the
    /// chamber (mol).
    chamber_dev: f64,
    cabin_dev: f64,
    inert_in: f64,
    /// Shared air only (§15, 2026-10-03): the ONE room's whole vapour — the crew's
    /// `cabin_h2o` plus the plants' `water_vapor` — over saturation at the plants' temperature,
    /// at slow-step starts: its largest value and the steps it reads above 1.
    room_rh_max: f64,
    room_over_saturation: u64,
    /// The same, BEFORE the 2026-10-03 fix, from this same run: the old condenser's crew vapour
    /// is the new one less the setpoint at every step (the shift, §15 H1), and no plant value
    /// moved, so `cabin_h2o − humidity_setpoint` IS the old crew vapour.
    room_rh_max_before: f64,
    room_over_saturation_before: u64,
}

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

#[allow(clippy::too_many_arguments)]
fn season(
    scenario: &SealedStationScenario,
    state: State,
    bio: Registry,
    fast: Registry,
    chamber_air_mol: f64,
    cabin_co2_id: &str,
    days: usize,
) -> Reading {
    let charge = params::charge();
    let lamp = station_params::lamp();
    let (bio, fast) = (EulerIntegrator::new(bio), EulerIntegrator::new(fast));
    let bio_r = sealed_bio_resolver(&lamp, scenario).expect("bio resolver");
    let fast_r = sealed_fast_resolver(&charge, scenario).expect("fast resolver");
    let reset = sealed_reset_hook(scenario);
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
        cabin_co2_min: f64::INFINITY,
        cabin_co2_max: f64::NEG_INFINITY,
        ..Reading::default()
    };
    let mut observe = |side: Side, before: &State, _after: &State| match side {
        Side::Reset => {}
        Side::Slow => {
            let bound = bio_r.bind(before, scenario.bio_dt);
            let results: Vec<FlowResult> = bio
                .registry()
                .flows()
                .iter()
                .map(|f| {
                    f.evaluate(before, &bound, scenario.bio_dt)
                        .expect("evaluate")
                })
                .collect();
            let demand = withdrawal_demand(&results, &before.stocks);
            r.co2_gross += demand.get(CARBON_POOL).copied().unwrap_or(0.0);
            let temp = bound.get(TEMP_VAR).expect("temp");
            r.rh += before.stocks[WATER_VAPOR].amount / saturation_vapour_kg(temp, chamber_air_mol);
            r.slow_steps += 1;
            if !before.stocks.contains_key(CABIN_INERT) {
                let room = (before.stocks[WATER_VAPOR].amount + before.stocks[CABIN_H2O].amount)
                    / saturation_vapour_kg(temp, chamber_air_mol);
                r.room_rh_max = r.room_rh_max.max(room);
                r.room_over_saturation += u64::from(room > 1.0);
                let before_fix = (before.stocks[WATER_VAPOR].amount
                    + before.stocks[CABIN_H2O].amount
                    - params::eclss().humidity_setpoint)
                    / saturation_vapour_kg(temp, chamber_air_mol);
                r.room_rh_max_before = r.room_rh_max_before.max(before_fix);
                r.room_over_saturation_before += u64::from(before_fix > 1.0);
            }
            for (f, res) in bio.registry().flows().iter().zip(&results) {
                if f.id() == TRANSPIRATION {
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
                    r.bound_binds += u64::from(air < flux);
                }
            }
        }
        Side::Fast => {
            let total = |co2: &str, o2: &str, inert: &str, vapour: &str| {
                before.stocks[co2].amount
                    + before.stocks[o2].amount
                    + before.stocks[inert].amount / N2_MOLAR_MASS_KG_PER_MOL
                    + before.stocks[vapour].amount / H2O_MOLAR_MASS_KG_PER_MOL
            };
            let chamber = total(CARBON_POOL, O2_POOL, CHAMBER_INERT, WATER_VAPOR);
            r.chamber_dev = r.chamber_dev.max((chamber / chamber_air_mol - 1.0).abs());
            if before.stocks.contains_key(CABIN_INERT) {
                let cabin = total(CABIN_CO2, CABIN_O2, CABIN_INERT, CABIN_H2O);
                r.cabin_dev = r.cabin_dev.max((cabin / CABIN_AIR_MOL - 1.0).abs());
            }
            let c = before.stocks[cabin_co2_id].amount;
            r.cabin_co2_min = r.cabin_co2_min.min(c);
            r.cabin_co2_max = r.cabin_co2_max.max(c);
            let bound = fast_r.bind(before, scenario.cabin_dt);
            // §16: with the gas exchange on the minute step, the crop's CO₂ draw is a FAST leg.
            let budget: Vec<FlowResult> = fast
                .registry()
                .flows()
                .iter()
                .filter(|f| CARBON_BUDGET_FLOWS.contains(&f.id()))
                .map(|f| {
                    f.evaluate(before, &bound, scenario.cabin_dt)
                        .expect("budget")
                })
                .collect();
            if !budget.is_empty() {
                let demand = withdrawal_demand(&budget, &before.stocks);
                r.co2_gross += demand.get(CARBON_POOL).copied().unwrap_or(0.0);
            }
            for f in fast.registry().flows() {
                if f.id() == AIR_EXCHANGE {
                    let res = f.evaluate(before, &bound, scenario.cabin_dt).expect("fan");
                    for leg in &res.legs {
                        if leg.stock == CHAMBER_INERT {
                            r.inert_in += leg.amount / N2_MOLAR_MASS_KG_PER_MOL;
                        }
                        if leg.stock == CABIN_H2O {
                            if leg.amount > 0.0 {
                                r.exported += leg.amount;
                            } else if leg.amount < 0.0 {
                                r.imported_steps += 1;
                            }
                        }
                    }
                }
            }
        }
    };
    let (states, totals) = two
        .run(DayOrder::Interleaved, state, days, &mut observe)
        .expect("two-rate run");
    r.rationed = (totals.slow_rationed, totals.fast_rationed);
    r.end = states.last().cloned();
    r
}

fn shared(scenario: &SealedStationScenario, days: usize, gas: GasExchangeStep) -> Reading {
    let (state, bio, fast) = build_sealed_station(
        &params::charge(),
        &params::thermal(),
        &params::crew(),
        &params::eclss(),
        &station_params::water_recovery(),
        &station_params::lamp(),
        &station_params::harvest(),
        scenario,
        false,
        false,
    )
    .expect("build_sealed_station");
    let (bio, fast) = match gas {
        GasExchangeStep::PlantStep => (bio, fast),
        GasExchangeStep::Minute => gas_exchange_on_fast_step(
            &state.stocks,
            bio,
            fast,
            &domains::biosphere::system::weather_shared(&scenario.bio),
        )
        .expect("gas exchange on the minute step"),
    };
    season(
        scenario,
        state,
        bio,
        fast,
        scenario.bio.chamber_air_capacity_mol,
        CARBON_POOL,
        days,
    )
}

fn split(scenario: &SealedStationScenario, s: AirSplit, days: usize) -> Reading {
    let (state, bio, fast) = build_split_station(
        &params::charge(),
        &params::thermal(),
        &params::crew(),
        &params::eclss(),
        &station_params::water_recovery(),
        &station_params::lamp(),
        &station_params::harvest(),
        scenario,
        &s,
    )
    .expect("build_split_station");
    let resized = split_scenario(scenario, &s);
    season(
        &resized,
        state,
        bio,
        fast,
        s.chamber_air_mol,
        CABIN_CO2,
        days,
    )
}

fn print(name: &str, r: &Reading, base: &Reading) {
    let end = r.end.as_ref().expect("end");
    let base_end = base.end.as_ref().expect("end");
    println!(
        "{name:<22} co2 {:8.4} mol ({:6.3} of shared) | plant C {:7.3} ({:6.3}) | RH {:.4} | \
         transp {:8.3} kg ({:5.3}) to_air {:7.3} bound {:5} | export {:7.3} kg (back {}) | \
         plant water {:8.3} crew water {:9.3} | cabin CO2 {:.6}–{:.6} | rationed {:?} |          total/ref − 1: chamber {:.3e} cabin {:.3e}, inert into chamber {:.4} mol",
        r.co2_gross,
        r.co2_gross / base.co2_gross,
        plant_c(end),
        plant_c(end) / plant_c(base_end),
        r.rh / r.slow_steps as f64,
        r.transpired,
        r.transpired / base.transpired,
        r.to_air,
        r.bound_binds,
        r.exported,
        r.imported_steps,
        plant_water(end),
        end.stocks[WATER_STORE].amount,
        r.cabin_co2_min,
        r.cabin_co2_max,
        r.rationed,
        r.chamber_dev,
        r.cabin_dev,
        r.inert_in,
    );
}

fn main() {
    let scenario = sealed_station_scenario();
    let days = scenario.season_days;
    let setpoint = bio_params::biosphere().water.humidity_setpoint;
    println!(
        "one season ({days} days); humidity setpoint {setpoint}; a 27.66-mol chamber's target at \
         16 °C {:.6} kg",
        humidity_target_kg(16.0, BVAD_CHAMBER_AIR_MOL, setpoint)
    );
    let base = shared(&scenario, days, GasExchangeStep::PlantStep);
    print("shared", &base, &base);
    println!(
        "shared room, crew + plant vapour over saturation at the plants' temperature: max {:.4}, \
         above 1 on {} of {} slow steps (before the fix: max {:.4}, above 1 on {})",
        base.room_rh_max,
        base.room_over_saturation,
        base.slow_steps,
        base.room_rh_max_before,
        base.room_over_saturation_before
    );
    let minute = shared(&scenario, days, GasExchangeStep::Minute);
    print("shared, minute gas", &minute, &base);
    let (plant, min) = (GasExchangeStep::PlantStep, GasExchangeStep::Minute);
    let cases = [
        (
            "split Q0.2 vapour-off",
            BVAD_CHAMBER_AIR_MOL,
            0.2,
            false,
            plant,
        ),
        (
            "split Q0.1 vapour-off",
            BVAD_CHAMBER_AIR_MOL,
            0.1,
            false,
            plant,
        ),
        (
            "split Q0.4 vapour-off",
            BVAD_CHAMBER_AIR_MOL,
            0.4,
            false,
            plant,
        ),
        (
            "split Q0.2 vapour-on",
            BVAD_CHAMBER_AIR_MOL,
            0.2,
            true,
            plant,
        ),
        ("big 9500 Q10 vap-off", 9500.0, 10.0, false, plant),
        (
            "min split Q0.2 v-off",
            BVAD_CHAMBER_AIR_MOL,
            0.2,
            false,
            min,
        ),
        (
            "min split Q0.1 v-off",
            BVAD_CHAMBER_AIR_MOL,
            0.1,
            false,
            min,
        ),
        (
            "min split Q0.4 v-off",
            BVAD_CHAMBER_AIR_MOL,
            0.4,
            false,
            min,
        ),
        ("min split Q0.2 v-on", BVAD_CHAMBER_AIR_MOL, 0.2, true, min),
        ("min big 9500 Q10", 9500.0, 10.0, false, min),
    ];
    for (name, cap, q, vapour, gas) in cases {
        let t0 = std::time::Instant::now();
        let r = split(
            &scenario,
            AirSplit {
                chamber_air_mol: cap,
                fan_mol_per_s: q,
                vapour_crosses: vapour,
                gas_exchange: gas,
            },
            days,
        );
        print(name, &r, &base);
        println!(
            "    ({name}: {:.1} s with the observer)",
            t0.elapsed().as_secs_f64()
        );
    }
}
