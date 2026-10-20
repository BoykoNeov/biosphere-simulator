//! One water reserve for the station — the lab measurement
//! (`docs/plans/post-roadmap-shared-room-humidity.md` §13):
//!
//! ```text
//! cargo run --release -q -p station --example water_reserve            # the 1220-day sealed run
//! cargo run --release -q -p station --example water_reserve -- --days N
//! cargo run --release -q -p station --example water_reserve -- --save <dir>  # adoption's bytes
//! ```
//!
//! The sealed station at [`WaterReserve::Off`] (the form until 2026-10-20) and
//! [`WaterReserve::CrewStore`], crop cycle by crop cycle. For each it reads what the top-up and the
//! drain move, evaluated on each step's own snapshot. For `Off`, which has neither, it reads what
//! they WOULD move: the tank's shortfall against each watering's gross (`deficit / ea`), and each
//! rise of the tank above one full watering — estimates, since nothing acts on them. Then the
//! largest watering, the deepest roots, and the plants' tank and total water at each cycle's end.
//!
//! `--save <dir>` runs `station::goldens::sealed_station`'s build at `CrewStore`, after checking
//! that the scenario's own reserve reproduces the committed golden (so a drift in this tool's copy
//! of the build cannot pass), and writes the final state to `<dir>/sealed_station_state.json`.
//! It writes nothing else and takes no decision.

use domains::biosphere::science::transpirable_capacity;
use domains::biosphere::stocks::{
    CONDENSATE, LEAF_C, ROOTED_DEPTH, ROOT_C, SOIL_WATER, STEM_C, STORAGE_C, SUBSOIL_WATER,
    WATER_VAPOR,
};
use domains::crew::WATER_STORE;
use domains::params;
use simcore::flow::FlowResult;
use simcore::integrator::EulerIntegrator;
use simcore::state::State;
use station::driver::{DayOrder, Side, TwoRate};
use station::params as sp;
use station::scenario::{sealed_station_scenario, SealedStationScenario};
use station::sealed::{
    build_sealed_station, run_sealed, sealed_bio_resolver, sealed_fast_resolver, sealed_reset_hook,
};
use station::water::{BRINE, RECOVERED_WATER};
use station::water_reserve::{working_level, WaterReserve, RESERVE_TOP_UP, TANK_DRAIN};

fn amount(s: &State, id: &str) -> f64 {
    s.stocks.get(id).map_or(0.0, |x| x.amount)
}

/// The plants' water: root zone, below it, their tank and the room's vapour.
fn plant_water(s: &State) -> f64 {
    [SOIL_WATER, SUBSOIL_WATER, CONDENSATE, WATER_VAPOR]
        .iter()
        .map(|id| amount(s, id))
        .sum()
}

fn plant_c(s: &State) -> f64 {
    [LEAF_C, STEM_C, ROOT_C, STORAGE_C]
        .iter()
        .map(|id| amount(s, id))
        .sum()
}

fn leg(res: &FlowResult, stock: &str) -> f64 {
    res.legs
        .iter()
        .filter(|l| l.stock == stock)
        .map(|l| l.amount)
        .sum()
}

fn scenario(reserve: WaterReserve) -> SealedStationScenario {
    SealedStationScenario {
        water_reserve: reserve,
        ..sealed_station_scenario()
    }
}

fn build(
    sc: &SealedStationScenario,
) -> (
    State,
    simcore::registry::Registry,
    simcore::registry::Registry,
) {
    build_sealed_station(
        &params::charge(),
        &params::thermal(),
        &params::crew(),
        &params::eclss(),
        &sp::water_recovery(),
        &sp::lamp(),
        &sp::harvest(),
        sc,
        false,
        false,
    )
    .expect("build sealed")
}

#[derive(Default)]
struct Cycle {
    triggers: u64,
    short: u64,
    /// Applied (`CrewStore`) or estimated (`Off`).
    top_up: f64,
    drain: f64,
    largest: f64,
    deepest: f64,
    tank_max: f64,
    tank_min: f64,
    grain_max: f64,
}

struct Run {
    cycles: Vec<Cycle>,
    ends: Vec<State>,
    rationed: (u64, u64),
}

fn run(reserve: WaterReserve, days: usize) -> Run {
    let sc = scenario(reserve);
    let p = sc.plant_params();
    let (extr, area) = (sc.bio.soil_extractable_water, sc.bio.ground_area);
    let (ea, trigger) = (p.water.drip_efficiency, 1.0 - p.water.depletion_fraction);
    let level = working_level(&sc.bio, &p);
    let (state, bio, fast) = build(&sc);
    let bio_r = sealed_bio_resolver(&sp::lamp(), &sc).expect("bio resolver");
    let fast_r = sealed_fast_resolver(&params::charge(), &params::crew(), &sc).expect("fast");
    let reset = sealed_reset_hook(&sc);
    let (bio, fast) = (EulerIntegrator::new(bio), EulerIntegrator::new(fast));
    let two = TwoRate {
        slow: &bio,
        fast: &fast,
        slow_resolver: &bio_r,
        fast_resolver: &fast_r,
        steps_per_day: sc.steps_per_day,
        slow_steps_per_day: sc.bio_steps_per_day,
        slow_dt: sc.bio_dt,
        fast_dt: sc.cabin_dt,
        slow_reset: Some(&*reset),
    };
    let fresh = || Cycle {
        tank_min: f64::INFINITY,
        ..Cycle::default()
    };
    let mut cycles = vec![fresh()];
    let mut ends = Vec::new();
    let mut observe = |side: Side, before: &State, after: &State| {
        if side == Side::Reset {
            ends.push(before.clone());
            cycles.push(fresh());
            return;
        }
        let c = cycles.last_mut().expect("a cycle");
        let (tb, ta) = (amount(before, CONDENSATE), amount(after, CONDENSATE));
        c.tank_max = c.tank_max.max(ta);
        c.tank_min = c.tank_min.min(ta);
        if side == Side::Fast {
            match reserve {
                WaterReserve::Off => {
                    c.drain += ((ta - level).max(0.0) - (tb - level).max(0.0)).max(0.0);
                }
                WaterReserve::CrewStore => {
                    let env = fast_r.bind(before, sc.cabin_dt);
                    for f in fast.registry().flows() {
                        if f.id() == TANK_DRAIN {
                            let res = f.evaluate(before, &env, sc.cabin_dt).expect("drain");
                            c.drain += leg(&res, RECOVERED_WATER);
                        }
                    }
                }
            }
            return;
        }
        c.grain_max = c.grain_max.max(amount(after, STORAGE_C));
        let rooted = before.aux.get(ROOTED_DEPTH).copied().unwrap_or(0.0);
        c.deepest = c.deepest.max(rooted);
        let capacity = transpirable_capacity(rooted, extr, area);
        let held = amount(before, SOIL_WATER);
        if capacity > 0.0 && held / capacity <= trigger {
            let gross = (capacity - held).max(0.0) / ea;
            c.triggers += 1;
            c.largest = c.largest.max(gross);
            if gross > tb {
                c.short += 1;
                if reserve == WaterReserve::Off {
                    c.top_up += gross - tb;
                }
            }
        }
        if reserve == WaterReserve::CrewStore {
            let env = bio_r.bind(before, sc.bio_dt);
            for f in bio.registry().flows() {
                if f.id() == RESERVE_TOP_UP {
                    let res = f.evaluate(before, &env, sc.bio_dt).expect("top-up");
                    c.top_up -= leg(&res, WATER_STORE);
                }
            }
        }
    };
    let (states, totals) = two
        .run(DayOrder::Interleaved, state, days, &mut observe)
        .expect("two-rate run");
    ends.push(states.last().expect("a day").clone());
    Run {
        cycles,
        ends,
        rationed: (totals.slow_rationed, totals.fast_rationed),
    }
}

fn report(name: &str, r: &Run, base: Option<&Run>) {
    println!("\n=== {name}: rationed {:?} ===", r.rationed);
    for (i, (c, e)) in r.cycles.iter().zip(&r.ends).enumerate() {
        println!(
            "cycle {i}: triggers {} ({} short of the tank); top-up {:.3} kg; drain {:.3} kg; \
             largest watering {:.3} kg; deepest roots {:.3} m; tank {:.3}–{:.3} kg; grain max {:.4}",
            c.triggers,
            c.short,
            c.top_up,
            c.drain,
            c.largest,
            c.deepest,
            c.tank_min,
            c.tank_max,
            c.grain_max
        );
        println!(
            "         end: tank {:.3}, plants' water {:.3} (soil {:.3}, subsoil {:.3}), crew store \
             {:.3}, recovery {:.3}, brine {:.3}, plant C {:.4}",
            amount(e, CONDENSATE),
            plant_water(e),
            amount(e, SOIL_WATER),
            amount(e, SUBSOIL_WATER),
            amount(e, WATER_STORE),
            amount(e, RECOVERED_WATER),
            amount(e, BRINE),
            plant_c(e)
        );
    }
    let (top, drain): (f64, f64) = r
        .cycles
        .iter()
        .fold((0.0, 0.0), |(t, d), c| (t + c.top_up, d + c.drain));
    println!("total: top-up {top:.3} kg, drain {drain:.3} kg");
    if let Some(b) = base {
        let (e, be) = (r.ends.last().expect("end"), b.ends.last().expect("end"));
        for id in [
            WATER_STORE,
            BRINE,
            CONDENSATE,
            SOIL_WATER,
            SUBSOIL_WATER,
            STORAGE_C,
        ] {
            let (v, w) = (amount(e, id), amount(be, id));
            println!("  end {id:24} {v:14.4} ({:+.4})", v - w);
        }
        let (v, w) = (plant_water(e), plant_water(be));
        println!("  end {:24} {v:14.4} ({:+.4})", "plants' water", v - w);
        let (v, w) = (plant_c(e), plant_c(be));
        println!(
            "  end {:24} {v:14.4} ({:+.4} %)",
            "plant carbon",
            100.0 * (v - w) / w
        );
    }
}

/// `station::goldens::sealed_station`'s run at `reserve`, to its snapshot.
fn golden(reserve: WaterReserve) -> String {
    let sc = scenario(reserve);
    let (state, bio, fast) = build(&sc);
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
    simcore::snapshot::from_engine(states.last().expect("a day")).to_json()
}

fn save(dir: &str) {
    let name = "sealed_station_state.json";
    let committed = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../data/golden")
            .join(name),
    )
    .expect("golden file");
    let reference = sealed_station_scenario().water_reserve;
    assert!(
        golden(reference) == committed,
        "{name}: the reference reserve ({reference:?}) does not reproduce the golden"
    );
    let chosen = golden(WaterReserve::CrewStore);
    std::fs::write(std::path::Path::new(dir).join(name), &chosen).expect("save");
    println!(
        "== {name}: CrewStore saved; reproduces the golden: {}",
        chosen == committed
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--save") {
        save(args.get(i + 1).expect("--save <dir>"));
        return;
    }
    let sc = sealed_station_scenario();
    let days = args
        .iter()
        .position(|a| a == "--days")
        .and_then(|i| args.get(i + 1))
        .map_or(sc.years * sc.season_days, |d| d.parse().expect("--days N"));
    println!(
        "one full watering (the tank's level): {:.3} kg",
        working_level(&sc.bio, &sc.plant_params())
    );
    let off = run(WaterReserve::Off, days);
    report(
        &format!("Off, {days} days (top-up and drain ESTIMATED)"),
        &off,
        None,
    );
    let on = run(WaterReserve::CrewStore, days);
    report(
        &format!("CrewStore, {days} days (against Off)"),
        &on,
        Some(&off),
    );
}
