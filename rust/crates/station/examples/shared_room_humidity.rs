//! One moisture store for a shared room — the lab measurement against the committed predictions
//! (`docs/plans/post-roadmap-shared-room-humidity.md` §5), as one command:
//!
//! ```text
//! cargo run --release -q -p station --example shared_room_humidity            # one season + 7-day runs
//! cargo run --release -q -p station --example shared_room_humidity -- --full  # the 1220-day sealed run
//! ```
//!
//! Each shared-room build (the sealed station, the greenhouse, the harvest greenhouse) at
//! [`RoomVapour::TwoStocks`] (the reference), `OneStock { 0.75 }` and `OneStock { 0.40 }`. It reads,
//! directly rather than from end stores, what each condenser catches, and the room's whole vapour
//! against saturation at the room's REAL temperature: the chamber's live heat-store temperature on
//! every minute (sealed), the plant window's within-day air (the greenhouses). It writes nothing
//! and takes no decision.

use domains::biosphere::science::saturation_vapour_kg;
use domains::biosphere::stocks::{
    AIR_TEMP_WITHIN_DAY_VAR, CONDENSATE, LEAF_C, ROOT_C, SOIL_WATER, STEM_C, STORAGE_C,
    SUBSOIL_WATER, WATER_VAPOR,
};
use domains::crew::{FECAL_WASTE, FOOD_STORE, WATER_STORE};
use domains::eclss::{CABIN_H2O, CONDENSER, HUMIDITY_CONDENSATE};
use domains::params;
use simcore::environment::SourceResolver;
use simcore::flow::FlowResult;
use simcore::integrator::EulerIntegrator;
use simcore::registry::Registry;
use simcore::state::State;
use station::driver::{DayOrder, OwnedResetHook, Side, TwoRate};
use station::greenhouse::{build_greenhouse, greenhouse_bio_resolver, greenhouse_cabin_resolver};
use station::harvest::{build_harvest, harvest_bio_resolver, harvest_cabin_resolver};
use station::params as sp;
use station::room_vapour::{RoomVapour, Thermostat, ROOM_CONDENSER};
use station::scenario::{greenhouse_scenario, harvest_scenario, sealed_station_scenario};
use station::sealed::{
    build_sealed_station, sealed_bio_resolver, sealed_fast_resolver, sealed_reset_hook,
};
use station::water::{BRINE, RECOVERED_WATER};

const TRANSPIRATION: &str = "biosphere.transpiration";
const PLANT_CONDENSATION: &str = "biosphere.condensation";

/// Where the room's temperature comes from.
#[derive(Clone, Copy)]
enum RealTemp {
    /// The chamber's heat store, every minute.
    Chamber,
    /// The plant window's within-day air (the weather's), read at each plant step.
    WithinDayAir,
}

#[derive(Default)]
struct Reading {
    /// kg caught by the plants' `Condensation`, sent by transpiration straight to condensate,
    /// and caught by the crew's condenser (ECLSS or the room's minute one).
    plant_condenser: f64,
    transpiration_bypass: f64,
    crew_condenser: f64,
    transpired: f64,
    /// The room's whole vapour over saturation at the real temperature, on every fast step.
    fast_steps: u64,
    over: u64,
    max: f64,
    /// The same split by the room's temperature (cold < 12 °C), and on the steps whose real
    /// temperature is more than 1 °C off the plant window's (the program's switches).
    max_warm: f64,
    max_cold: f64,
    switch_steps: u64,
    max_switch: f64,
    /// The largest crop grain store in each season (between re-sows).
    grain_peaks: Vec<f64>,
    rationed: (u64, u64),
    end: Option<State>,
}

fn amount(s: &State, id: &str) -> f64 {
    s.stocks.get(id).map_or(0.0, |x| x.amount)
}

fn room_vapour_kg(s: &State) -> f64 {
    amount(s, WATER_VAPOR) + amount(s, CABIN_H2O)
}

fn positive_leg(res: &FlowResult, stock: &str) -> f64 {
    res.legs
        .iter()
        .filter(|l| l.stock == stock && l.amount > 0.0)
        .map(|l| l.amount)
        .sum()
}

fn chamber_c(s: &State) -> f64 {
    s.stocks[station::chamber::CHAMBER].amount / sp::chamber().heat_capacity - 273.15
}

#[allow(clippy::too_many_arguments)]
fn observed_run(
    state: State,
    bio: Registry,
    fast: Registry,
    bio_r: &SourceResolver,
    fast_r: &SourceResolver,
    reset: Option<&OwnedResetHook>,
    (steps_per_day, slow_steps_per_day, slow_dt, fast_dt): (u64, u64, f64, f64),
    air_mol: f64,
    real: RealTemp,
    crew_dest: &str,
    days: usize,
) -> Reading {
    let (bio, fast) = (EulerIntegrator::new(bio), EulerIntegrator::new(fast));
    let two = TwoRate {
        slow: &bio,
        fast: &fast,
        slow_resolver: bio_r,
        fast_resolver: fast_r,
        steps_per_day,
        slow_steps_per_day,
        slow_dt,
        fast_dt,
        slow_reset: reset.map(|r| &**r),
    };
    let mut r = Reading {
        grain_peaks: vec![0.0],
        ..Reading::default()
    };
    let mut window_temp = f64::NAN;
    let mut observe = |side: Side, before: &State, after: &State| match side {
        Side::Reset => r.grain_peaks.push(0.0),
        Side::Slow => {
            let bound = bio_r.bind(before, slow_dt);
            window_temp = match real {
                RealTemp::Chamber => chamber_c(before),
                RealTemp::WithinDayAir => {
                    use simcore::environment::Environment;
                    bound.get(AIR_TEMP_WITHIN_DAY_VAR).expect("within-day air")
                }
            };
            for f in bio.registry().flows() {
                if f.id() == PLANT_CONDENSATION || f.id() == TRANSPIRATION {
                    let res = f.evaluate(before, &bound, slow_dt).expect("plant flow");
                    if f.id() == PLANT_CONDENSATION {
                        r.plant_condenser += positive_leg(&res, CONDENSATE);
                    } else {
                        r.transpiration_bypass += positive_leg(&res, CONDENSATE);
                        r.transpired -= res
                            .legs
                            .iter()
                            .filter(|l| l.stock == SOIL_WATER)
                            .map(|l| l.amount)
                            .sum::<f64>();
                    }
                }
            }
            let peak = r.grain_peaks.last_mut().expect("a season");
            *peak = peak.max(amount(after, STORAGE_C));
        }
        Side::Fast => {
            let t = match real {
                RealTemp::Chamber => chamber_c(before),
                RealTemp::WithinDayAir => window_temp,
            };
            let ratio = room_vapour_kg(before) / saturation_vapour_kg(t, air_mol);
            r.fast_steps += 1;
            r.over += u64::from(ratio > 1.0);
            r.max = r.max.max(ratio);
            if t < 12.0 {
                r.max_cold = r.max_cold.max(ratio);
            } else {
                r.max_warm = r.max_warm.max(ratio);
            }
            if (t - window_temp).abs() > 1.0 {
                r.switch_steps += 1;
                r.max_switch = r.max_switch.max(ratio);
            }
            let bound = fast_r.bind(before, fast_dt);
            for f in fast.registry().flows() {
                if f.id() == CONDENSER || f.id() == ROOM_CONDENSER {
                    let res = f.evaluate(before, &bound, fast_dt).expect("crew condenser");
                    r.crew_condenser += positive_leg(&res, crew_dest);
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

const fn one(setting: f64, thermostat: Thermostat) -> RoomVapour {
    RoomVapour::OneStock {
        setting,
        thermostat,
    }
}

/// The window thermostat, both settings: every build.
const ROOMS: [(&str, RoomVapour); 3] = [
    ("two stocks", RoomVapour::TwoStocks),
    ("one @ 0.75", one(0.75, Thermostat::Window)),
    ("one @ 0.40", one(0.40, Thermostat::Window)),
];

/// The live chamber thermostat (plan §8): the sealed station only, which has a chamber.
const LIVE_ROOMS: [(&str, RoomVapour); 2] = [
    ("live @ 0.75", one(0.75, Thermostat::LiveChamber)),
    ("live @ 0.40", one(0.40, Thermostat::LiveChamber)),
];

fn sealed(room: RoomVapour, days: usize) -> Reading {
    let sc = sealed_station_scenario();
    let sc = station::scenario::SealedStationScenario {
        room_vapour: room,
        ..sc
    };
    let (state, bio, fast) = build_sealed_station(
        &params::charge(),
        &params::thermal(),
        &params::crew(),
        &params::eclss(),
        &sp::water_recovery(),
        &sp::lamp(),
        &sp::harvest(),
        &sc,
        false,
        false,
    )
    .expect("build sealed");
    let bio_r = sealed_bio_resolver(&sp::lamp(), &sc).expect("bio resolver");
    let fast_r = sealed_fast_resolver(&params::charge(), &params::crew(), &sc).expect("fast");
    let reset = sealed_reset_hook(&sc);
    observed_run(
        state,
        bio,
        fast,
        &bio_r,
        &fast_r,
        Some(&reset),
        (
            sc.steps_per_day,
            sc.bio_steps_per_day,
            sc.bio_dt,
            sc.cabin_dt,
        ),
        sc.bio.chamber_air_capacity_mol,
        RealTemp::Chamber,
        RECOVERED_WATER,
        days,
    )
}

fn greenhouse(room: RoomVapour) -> Reading {
    let sc = station::scenario::GreenhouseScenario {
        room_vapour: room,
        ..greenhouse_scenario()
    };
    let crew = params::crew();
    let (state, bio, fast) =
        build_greenhouse(&crew, &params::eclss(), &sc, true, FECAL_WASTE).expect("build");
    let bio_r = greenhouse_bio_resolver(&sc).expect("bio resolver");
    let fast_r = greenhouse_cabin_resolver(&crew, &sc).expect("cabin resolver");
    observed_run(
        state,
        bio,
        fast,
        &bio_r,
        &fast_r,
        None,
        (
            sc.steps_per_day,
            sc.bio_steps_per_day,
            sc.bio_dt,
            sc.cabin_dt,
        ),
        sc.bio.chamber_air_capacity_mol,
        RealTemp::WithinDayAir,
        HUMIDITY_CONDENSATE,
        sc.days,
    )
}

fn harvest(room: RoomVapour) -> Reading {
    let mut sc = harvest_scenario();
    sc.greenhouse.room_vapour = room;
    let crew = params::crew();
    // As the golden: harvest on, feces closed into the litter.
    let (state, bio, fast) =
        build_harvest(&crew, &params::eclss(), &sp::harvest(), &sc, true, true).expect("build");
    let bio_r = harvest_bio_resolver(&sc).expect("bio resolver");
    let fast_r = harvest_cabin_resolver(&crew, &sc).expect("cabin resolver");
    let g = &sc.greenhouse;
    observed_run(
        state,
        bio,
        fast,
        &bio_r,
        &fast_r,
        None,
        (g.steps_per_day, g.bio_steps_per_day, g.bio_dt, g.cabin_dt),
        g.bio.chamber_air_capacity_mol,
        RealTemp::WithinDayAir,
        HUMIDITY_CONDENSATE,
        g.days,
    )
}

fn plant_c(s: &State) -> f64 {
    [LEAF_C, STEM_C, ROOT_C, STORAGE_C]
        .iter()
        .map(|id| amount(s, id))
        .sum()
}

fn report(build: &str, readings: &[(&str, Reading)]) {
    println!("\n=== {build} ===");
    let base = readings[0].1.end.as_ref().expect("end");
    for (name, r) in readings {
        let e = r.end.as_ref().expect("end");
        println!(
            "{name:11} rationed {:?} | caught: plants' condenser {:.3} kg, transpiration→condensate \
             {:.3} kg, crew condenser {:.3} kg | transpired {:.3} kg",
            r.rationed, r.plant_condenser, r.transpiration_bypass, r.crew_condenser, r.transpired
        );
        println!(
            "{:11} room ÷ real saturation: max {:.4} (warm {:.4}, cold {:.4}), over 1 on {} of {} \
             minutes; off-window minutes {} (max {:.4})",
            "", r.max, r.max_warm, r.max_cold, r.over, r.fast_steps, r.switch_steps, r.max_switch
        );
        let peaks: Vec<String> = r.grain_peaks.iter().map(|g| format!("{g:.3}")).collect();
        println!("{:11} grain peaks per season [{}]", "", peaks.join(", "));
        for id in [
            WATER_STORE,
            RECOVERED_WATER,
            BRINE,
            HUMIDITY_CONDENSATE,
            CONDENSATE,
            SOIL_WATER,
            SUBSOIL_WATER,
            WATER_VAPOR,
            CABIN_H2O,
            FOOD_STORE,
        ] {
            if !e.stocks.contains_key(id) && !base.stocks.contains_key(id) {
                continue;
            }
            let (v, b) = (amount(e, id), amount(base, id));
            println!(
                "{:11}   {id:24} {v:16.6} ({:+.6}, {:+.4} %)",
                "",
                v - b,
                if b != 0.0 {
                    100.0 * (v - b) / b
                } else {
                    f64::NAN
                }
            );
        }
        let (pc, pb) = (plant_c(e), plant_c(base));
        println!(
            "{:11}   plant carbon {pc:.6} mol ({:+.4} %)",
            "",
            100.0 * (pc - pb) / pb
        );
        for id in ["thermal.node", "power.battery", station::chamber::CHAMBER] {
            if let (Some(v), Some(b)) = (e.stocks.get(id), base.stocks.get(id)) {
                println!(
                    "{:11}   {id:24} {:+.3e} relative",
                    "",
                    (v.amount - b.amount) / b.amount
                );
            }
        }
    }
}

fn main() {
    let full = std::env::args().any(|a| a == "--full");
    let sc = sealed_station_scenario();
    let days = if full {
        sc.years as usize * sc.season_days
    } else {
        sc.season_days
    };
    let sealed_runs: Vec<(&str, Reading)> = ROOMS
        .iter()
        .chain(LIVE_ROOMS.iter())
        .map(|(n, room)| (*n, sealed(*room, days)))
        .collect();
    report(&format!("sealed station, {days} days"), &sealed_runs);
    if full {
        return;
    }
    let gh: Vec<(&str, Reading)> = ROOMS
        .iter()
        .map(|(n, room)| (*n, greenhouse(*room)))
        .collect();
    report("greenhouse, 7 days", &gh);
    let hv: Vec<(&str, Reading)> = ROOMS.iter().map(|(n, room)| (*n, harvest(*room))).collect();
    report("harvest, 7 days", &hv);
}
