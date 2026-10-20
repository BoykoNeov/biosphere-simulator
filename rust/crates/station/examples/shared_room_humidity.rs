//! One moisture store for a shared room — the lab measurement against the committed predictions
//! (`docs/plans/post-roadmap-shared-room-humidity.md` §5), as one command:
//!
//! ```text
//! cargo run --release -q -p station --example shared_room_humidity            # one season + 7-day runs
//! cargo run --release -q -p station --example shared_room_humidity -- --full  # the 1220-day sealed run
//! cargo run --release -q -p station --example shared_room_humidity -- --greenhouses  # 7-day runs only
//! cargo run --release -q -p station --example shared_room_humidity -- --save <dir>  # adoption's bytes
//! ```
//!
//! `--save <dir>` runs the three goldens' own builds (`station::goldens`) at the room the user
//! chose on 2026-10-20 — 75 %, the sealed station's thermostat live, the greenhouses' the plant
//! window's (they have no chamber) — checks it against the committed golden's bytes (so a drift in
//! this tool's copy of a build cannot pass), writes that final state to `<dir>/<golden file>`, and
//! prints whether two stocks differ. ⚠ Until the adoption (2026-10-20, plan §12) the check ran the
//! other way: two stocks were the reference and had to reproduce the golden, and the chosen room's
//! state was saved as the prediction the regenerated goldens were `cmp`'d against.
//!
//! Each shared-room build (the sealed station, the greenhouse, the harvest greenhouse) at
//! [`RoomVapour::TwoStocks`] (the form until 2026-10-20), `OneStock { 0.75 }` (the reference
//! since, on the live thermostat for the sealed station) and `OneStock { 0.40 }`. It reads,
//! directly rather than from end stores, what each condenser catches, and the room's whole vapour
//! against saturation at the room's REAL temperature: the chamber's live heat-store temperature on
//! every minute (sealed), the weather's within-day air curve read over each minute (the
//! greenhouses). It writes nothing
//! but `--save`'s files and takes no decision.

use domains::biosphere::science::saturation_vapour_kg;
use domains::biosphere::stocks::{
    AIR_TEMP_WITHIN_DAY_VAR, CONDENSATE, LEAF_C, ROOT_C, SOIL_WATER, STEM_C, STORAGE_C,
    SUBSOIL_WATER, WATER_VAPOR,
};
use domains::biosphere::system::weather_forcings;
use domains::crew::{FECAL_WASTE, FOOD_STORE, WATER_STORE};
use domains::eclss::{CABIN_H2O, CONDENSER, HUMIDITY_CONDENSATE};
use domains::params;
use simcore::environment::{Schedule, SourceResolver};
use simcore::flow::FlowResult;
use simcore::integrator::EulerIntegrator;
use simcore::registry::Registry;
use simcore::state::State;
use station::driver::{DayOrder, OwnedResetHook, Side, TwoRate};
use station::greenhouse::{
    build_greenhouse, greenhouse_bio_resolver, greenhouse_cabin_resolver, run_greenhouse,
};
use station::harvest::{build_harvest, harvest_bio_resolver, harvest_cabin_resolver, run_harvest};
use station::params as sp;
use station::room_vapour::{RoomVapour, Thermostat, ROOM_CONDENSER};
use station::scenario::{greenhouse_scenario, harvest_scenario, sealed_station_scenario};
use station::sealed::{
    build_sealed_station, run_sealed, sealed_bio_resolver, sealed_fast_resolver, sealed_reset_hook,
};
use station::water::{BRINE, RECOVERED_WATER};

const SECONDS_PER_DAY: f64 = 86400.0;
const TRANSPIRATION: &str = "biosphere.transpiration";
const PLANT_CONDENSATION: &str = "biosphere.condensation";

/// Where the room's temperature comes from.
enum RealTemp {
    /// The chamber's heat store, every minute.
    Chamber,
    /// The weather's within-day air curve, read over each MINUTE (the plants' water reads the
    /// same curve averaged over the plant window).
    OutdoorAir(Schedule),
}

/// The weather's within-day air curve, as the greenhouses' plant resolver builds it.
fn outdoor_air(bio: &domains::biosphere::SeasonScenario) -> RealTemp {
    let mut forcings = weather_forcings(bio, 1).expect("weather");
    RealTemp::OutdoorAir(
        forcings
            .remove(AIR_TEMP_WITHIN_DAY_VAR)
            .expect("within-day air"),
    )
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
    /// One store only: minutes whose room is above the setting plus one minute's crew breath
    /// (the ceiling a live thermostat predicts), and how many of those had the room cooling
    /// since the previous minute.
    above_ceiling: u64,
    above_ceiling_cooling: u64,
    /// The battery's lowest charge on any fast step (J), where the build has one.
    battery_min: f64,
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
    real: &RealTemp,
    crew_dest: &str,
    days: usize,
    // One store: (the room's setting, the crew's breath per fast step, kg).
    ceiling: Option<(f64, f64)>,
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
        battery_min: f64::INFINITY,
        ..Reading::default()
    };
    let mut window_temp = f64::NAN;
    let mut prev_t = f64::NAN;
    // The minute's index on the fast clock: the window's plant step × minutes per window + k.
    let per_window = steps_per_day / slow_steps_per_day;
    let (mut window_n, mut k) = (0u64, 0u64);
    let mut observe = |side: Side, before: &State, after: &State| match side {
        Side::Reset => r.grain_peaks.push(0.0),
        Side::Slow => {
            let bound = bio_r.bind(before, slow_dt);
            window_temp = match real {
                RealTemp::Chamber => chamber_c(before),
                RealTemp::OutdoorAir(_) => {
                    use simcore::environment::Environment;
                    bound.get(AIR_TEMP_WITHIN_DAY_VAR).expect("within-day air")
                }
            };
            (window_n, k) = (before.n, 0);
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
                RealTemp::OutdoorAir(air) => {
                    air(window_n * per_window + k, fast_dt / SECONDS_PER_DAY)
                }
            };
            k += 1;
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
            if let Some((setting, breath)) = ceiling {
                let sat = saturation_vapour_kg(t, air_mol);
                if room_vapour_kg(before) > setting * sat + breath {
                    r.above_ceiling += 1;
                    r.above_ceiling_cooling += u64::from(t < prev_t);
                }
            }
            prev_t = t;
            if let Some(b) = after.stocks.get(domains::power::BATTERY) {
                r.battery_min = r.battery_min.min(b.amount);
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
        &RealTemp::Chamber,
        RECOVERED_WATER,
        days,
        ceiling(room, sc.cabin, sc.cabin_dt),
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
        &outdoor_air(&sc.bio),
        HUMIDITY_CONDENSATE,
        sc.days,
        ceiling(room, sc.cabin, sc.cabin_dt),
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
        &outdoor_air(&g.bio),
        HUMIDITY_CONDENSATE,
        g.days,
        ceiling(room, g.cabin, g.cabin_dt),
    )
}

/// A one-store room's live ceiling inputs: its setting and the crew's breath per fast step.
fn ceiling(
    room: RoomVapour,
    cabin: station::scenario::CabinScenario,
    cabin_dt: f64,
) -> Option<(f64, f64)> {
    match room {
        RoomVapour::TwoStocks => None,
        RoomVapour::OneStock { setting, .. } => Some((
            setting,
            cabin.water_intake_rate() * params::crew().insensible_water_fraction * cabin_dt,
        )),
    }
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
        println!(
            "{:11} above setting + one minute's breath: {} minutes ({} with the room cooling since \
             the minute before); battery min {:.6e} J",
            "", r.above_ceiling, r.above_ceiling_cooling, r.battery_min
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

fn snapshot(state: &State) -> String {
    simcore::snapshot::from_engine(state).to_json()
}

/// `station::goldens::greenhouse`, at `room`.
fn greenhouse_golden(room: RoomVapour) -> String {
    let (crew, eclss) = (params::crew(), params::eclss());
    let sc = station::scenario::GreenhouseScenario {
        room_vapour: room,
        ..greenhouse_scenario()
    };
    let (state, bio, cabin) =
        build_greenhouse(&crew, &eclss, &sc, true, FECAL_WASTE).expect("build");
    let bio_r = greenhouse_bio_resolver(&sc).expect("bio resolver");
    let cabin_r = greenhouse_cabin_resolver(&crew, &sc).expect("cabin resolver");
    let (states, rationed, events) = run_greenhouse(
        &EulerIntegrator::new(bio),
        &EulerIntegrator::new(cabin),
        state,
        &bio_r,
        &cabin_r,
        &sc,
    )
    .expect("run greenhouse");
    assert!(
        rationed == 0 && events.is_empty(),
        "greenhouse: rationed or events"
    );
    snapshot(states.last().expect("a day"))
}

/// `station::goldens::harvest`, at `room`.
fn harvest_golden(room: RoomVapour) -> String {
    let (crew, eclss) = (params::crew(), params::eclss());
    let mut sc = harvest_scenario();
    sc.greenhouse.room_vapour = room;
    let (state, bio, cabin) =
        build_harvest(&crew, &eclss, &sp::harvest(), &sc, true, true).expect("build");
    let bio_r = harvest_bio_resolver(&sc).expect("bio resolver");
    let cabin_r = harvest_cabin_resolver(&crew, &sc).expect("cabin resolver");
    let (states, rationed, events) = run_harvest(
        &EulerIntegrator::new(bio),
        &EulerIntegrator::new(cabin),
        state,
        &bio_r,
        &cabin_r,
        &sc,
    )
    .expect("run harvest");
    assert!(
        rationed == 0 && events.is_empty(),
        "harvest: rationed or events"
    );
    snapshot(states.last().expect("a day"))
}

/// `station::goldens::sealed_station`, at `room`.
fn sealed_golden(room: RoomVapour) -> String {
    let sc = station::scenario::SealedStationScenario {
        room_vapour: room,
        ..sealed_station_scenario()
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
    let (states, rationed, events) = run_sealed(
        &EulerIntegrator::new(bio),
        &EulerIntegrator::new(fast),
        state,
        &bio_r,
        &fast_r,
        &sc,
    )
    .expect("run sealed");
    assert!(
        rationed == 0 && events.is_empty(),
        "sealed: rationed or events"
    );
    snapshot(states.last().expect("a day"))
}

/// One golden's own build and run at a room, to its snapshot.
type GoldenRun = fn(RoomVapour) -> String;

/// `--save <dir>`: each golden at the chosen room (checked against the committed bytes), saved,
/// and at two stocks for the contrast.
fn save(dir: &str) {
    let golden_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/golden");
    let window = one(0.75, Thermostat::Window);
    let live = one(0.75, Thermostat::LiveChamber);
    let runs: [(&str, GoldenRun, RoomVapour); 3] = [
        ("greenhouse_state.json", greenhouse_golden, window),
        ("harvest_state.json", harvest_golden, window),
        ("sealed_station_state.json", sealed_golden, live),
    ];
    for (name, run, chosen) in runs {
        let committed = std::fs::read_to_string(golden_dir.join(name)).expect("golden file");
        let adopted = run(chosen);
        let reproduces = adopted == committed;
        assert!(
            reproduces,
            "{name}: the chosen room ({chosen:?}) does not reproduce the golden"
        );
        std::fs::write(std::path::Path::new(dir).join(name), &adopted).expect("save");
        println!(
            "== {name}: {chosen:?} reproduces the golden: {reproduces}, saved; two stocks differ \
             from it: {}",
            run(RoomVapour::TwoStocks) != committed
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--save") {
        save(args.get(i + 1).expect("--save <dir>"));
        return;
    }
    let full = std::env::args().any(|a| a == "--full");
    let greenhouses_only = std::env::args().any(|a| a == "--greenhouses");
    let sc = sealed_station_scenario();
    let days = if full {
        sc.years as usize * sc.season_days
    } else {
        sc.season_days
    };
    if !greenhouses_only {
        let sealed_runs: Vec<(&str, Reading)> = ROOMS
            .iter()
            .chain(LIVE_ROOMS.iter())
            .map(|(n, room)| (*n, sealed(*room, days)))
            .collect();
        report(&format!("sealed station, {days} days"), &sealed_runs);
    }
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
