//! The big station, Step 1 — the sealed station at 200 m² of crop and four crew, measured
//! (`docs/plans/post-roadmap-big-station.md` §3). A lab: it writes nothing and takes no decision.
//!
//! ```text
//! cargo run --release -q -p station --example big_station                 # 1 m² and 200 m², 1220 days
//! cargo run --release -q -p station --example big_station -- --days N
//! cargo run --release -q -p station --example big_station -- --area A     # another crop area (m²)
//! cargo run --release -q -p station --example big_station -- --control    # the a = 1 golden check
//! cargo run --release -q -p station --example big_station -- --crew N     # N crew in the big run
//! ```
//!
//! `--crew N` (the attribution check): the big run with N crew, their food and water stores ×
//! N / 4. At `N = 4a` every m² gets the 1 m² run's CO₂, so a crop per m² back at the 1 m² run's
//! says the shortfall at four crew is the CO₂ and nothing else the sizing moved.
//!
//! ```text
//! ```
//!
//! **The sizing** ([`sized`]): everything the reference sizes per m² of crop is multiplied by the
//! area `a` — the crop's and soil's starting amounts and the plants' water tank (its start is the
//! working level, which follows the area), the lamp, the chamber's heat capacity, cooler and
//! heater, and its walls by the reference's own box rule (a `√a × √a` floor, 0.78 m tall: walls,
//! floor and roof `2a + 4·0.78·√a`, so 5.12 m² at 1 m² and 444 m² at 200 m²). The crew (four, the
//! reference), their stores, the room's air (9 500 mol) and its CO₂ and O₂ targets are left as
//! they are. At `a = 1` [`sized`] changes nothing: `--control` runs it through the reference's
//! driver and checks the committed `sealed_station_state.json` byte for byte.
//!
//! **Two stand-ins, so that neither binds** ([`outside_supply`]), recorded as stand-ins:
//! * *Outside electricity*: a lab flow, [`GridSupply`], that tops the station's battery back up
//!   to its starting charge every minute from an outside source, `boundary.grid`. The
//!   electricity drawn is what the grid gave plus what the battery lost.
//!   ⚠ Not a battery of 1e14 J (tried first): the engine's every-step conservation check allows
//!   1e-9 of the step's largest transfer, and a stock that large rounds by ~0.016 J — the 1 m²
//!   run failed on day one. The same rounding bounds every stock that keeps a running total,
//!   the boundaries included: at 200 m² the heat sent to space reaches ~3e12 J over 1220 days.
//! * *Heat rejection*: the station's radiator and its structure's heat capacity, ×a. The chamber's
//!   cooler hands its heat to the structure and can do so only while the structure is colder
//!   than the room (no heat pump is modelled); with the 1 m² radiator the structure would run
//!   far above the room and the cooler would stop. The readouts count the minutes that happens.
//!
//! **The readouts.** Closure share: the ecosystem's net CO₂ draw (the crew's exhalation less what
//! the scrubber removed less the room's own change) over the crew's exhalation, per crop cycle;
//! crop per m²; the room's CO₂ and O₂; water; electricity; the chamber's and the structure's
//! temperature, the cooler's blocked and capped minutes and the heater's capped minutes; and
//! rationing. ⚠ `rationed == 0` cannot measure starvation, so it is reported, never the verdict.

use domains::biosphere::stocks::{
    CARBON_POOL, CONDENSATE, CONSUMER_CARBON, HUMUS_CARBON, LEAF_C, LITTER_CARBON, LITTER_SINK,
    MICROBIAL_CARBON, O2_POOL, ROOT_C, SOIL_WATER, STEM_C, STEM_RESERVE_C, STORAGE_C,
    SUBSOIL_WATER, WATER_VAPOR,
};
use domains::biosphere::system::SeasonScenario;
use domains::crew::{FOOD_STORE, WATER_STORE};
use domains::eclss::{CO2_REMOVED, O2_SUPPLY};
use domains::params;
use domains::power::BATTERY;
use domains::thermal::{temperature, ThermalParams, NODE};
use simcore::environment::Environment;
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::integrator::EulerIntegrator;
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::State;
use station::chamber::{
    chamber_temperature, ChamberParams, CHAMBER, CHAMBER_COOLING, CHAMBER_HEATER,
};
use station::driver::{DayOrder, Side, TwoRate};
use station::params as sp;
use station::scenario::{sealed_station_scenario, SealedStationScenario};
use station::sealed::{
    build_sealed_station, run_sealed, sealed_bio_resolver, sealed_fast_resolver, sealed_reset_hook,
};
use station::water_reserve::working_level;

/// The chamber's height (m) in `chamber.yaml`'s wall-area rule: BVAD's shoot + root zone.
const CHAMBER_HEIGHT: f64 = 0.78;
/// The outside supply [`GridSupply`] draws from.
const GRID: &str = "boundary.grid";
/// The lab's outside-electricity flow id.
const GRID_SUPPLY: &str = "lab.grid_supply";

/// ENERGY `boundary.grid → power.battery`: tops the battery back up to `level` (J) each step,
/// from the step's starting charge. A lab stand-in for a grid connection (module doc).
struct GridSupply {
    level: f64,
}

impl Flow for GridSupply {
    fn type_name(&self) -> &'static str {
        "GridSupply"
    }

    fn id(&self) -> &str {
        GRID_SUPPLY
    }

    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        _dt: f64,
    ) -> Result<FlowResult, SimError> {
        let deficit = (self.level - amount(snapshot, BATTERY)).max(0.0);
        FlowResult::new(vec![
            Leg::new(GRID.to_string(), -deficit)?,
            Leg::new(BATTERY.to_string(), deficit)?,
        ])
    }
}

fn amount(s: &State, id: &str) -> f64 {
    s.stocks.get(id).map_or(0.0, |x| x.amount)
}

fn plant_c(s: &State) -> f64 {
    [LEAF_C, STEM_C, ROOT_C, STORAGE_C]
        .iter()
        .map(|id| amount(s, id))
        .sum()
}

/// The carbon held in the room outside its air: the crop (with its stem reserve), the soil's
/// litter, microbes and humus, the consumers, and what the litter sink has taken.
fn held_c(s: &State) -> (f64, f64) {
    let soil = [LITTER_CARBON, MICROBIAL_CARBON, HUMUS_CARBON, CONSUMER_CARBON, LITTER_SINK]
        .iter()
        .map(|id| amount(s, id))
        .sum();
    (plant_c(s) + amount(s, STEM_RESERVE_C), soil)
}

fn plant_water(s: &State) -> f64 {
    [SOIL_WATER, SUBSOIL_WATER, CONDENSATE, WATER_VAPOR]
        .iter()
        .map(|id| amount(s, id))
        .sum()
}

/// The water stocks whose flows the run tallies.
const WATER_STOCKS: [&str; 5] = [SOIL_WATER, SUBSOIL_WATER, WATER_VAPOR, CONDENSATE, WATER_STORE];

fn tally(t: &mut std::collections::BTreeMap<(String, &'static str), f64>, id: &str, res: &FlowResult) {
    for stock in WATER_STOCKS {
        let v = leg(res, stock);
        if v != 0.0 {
            *t.entry((id.to_string(), stock)).or_insert(0.0) += v;
        }
    }
}

fn leg(res: &FlowResult, stock: &str) -> f64 {
    res.legs
        .iter()
        .filter(|l| l.stock == stock)
        .map(|l| l.amount)
        .sum()
}

/// The crop and its soil at `a` m²: every extensive starting amount × a, and the plants' tank.
/// The room's air and its gases and vapour are the room's, not the crop's: left alone.
fn crop_at(bio: &SeasonScenario, a: f64) -> SeasonScenario {
    let mut t = *bio;
    t.ground_area *= a;
    for f in [
        &mut t.leaf_c0,
        &mut t.stem_c0,
        &mut t.root_c0,
        &mut t.storage_c0,
        &mut t.co2_atmos0,
        &mut t.litter_carbon0,
        &mut t.consumer_c0,
        &mut t.soil_water0,
        &mut t.subsoil_water0,
        &mut t.water_source0,
        &mut t.condensate0,
        &mut t.soil_n0,
        &mut t.n_source0,
        &mut t.plant_n0,
        &mut t.sn_residual,
        &mut t.sn_critical,
    ] {
        *f *= a;
    }
    t
}

/// The chamber at `a` m²: heat capacity, cooler and heater × a; walls by the box rule.
fn chamber_at(c: &ChamberParams, a: f64) -> ChamberParams {
    let box_walls = |area: f64| 2.0 * area + 4.0 * CHAMBER_HEIGHT * area.sqrt();
    ChamberParams {
        heat_capacity: c.heat_capacity * a,
        cooling_capacity: c.cooling_capacity * a,
        heater_capacity: c.heater_capacity * a,
        wall_area: c.wall_area * (box_walls(a) / box_walls(1.0)),
        ..*c
    }
}

/// The sealed station sized for `a` m² of crop ([`crop_at`], [`chamber_at`], the lamp). The
/// identity at `a = 1`.
fn sized(a: f64) -> SealedStationScenario {
    let s = sealed_station_scenario();
    SealedStationScenario {
        bio: crop_at(&s.bio, a),
        lamp_power_w: s.lamp_power_w * a,
        chamber: chamber_at(&s.chamber, a),
        ..s
    }
}

/// The heat-rejection stand-in (module doc): the radiator and the structure × a.
fn outside_heat(a: f64) -> ThermalParams {
    let t = params::thermal();
    ThermalParams {
        radiator_area: t.radiator_area * a,
        heat_capacity: t.heat_capacity * a,
        ..t
    }
}

/// The station built on the grid: the sealed build plus `boundary.grid` and [`GridSupply`],
/// holding the battery at its starting charge.
fn on_grid(
    sc: &SealedStationScenario,
    thermal: &ThermalParams,
) -> (
    State,
    simcore::registry::Registry,
    simcore::registry::Registry,
) {
    let (state, bio, fast) = build(sc, thermal);
    let mut stocks = state.stocks.clone();
    let grid = simcore::boundary::source(GRID.to_string(), Quantity::Energy, 0.0, true)
        .expect("grid");
    stocks.insert(grid.id.clone(), grid);
    let state = State::new(state.n, stocks, state.rng_seed, state.aux.clone()).expect("state");
    let (mut flows, aux) = fast.into_parts();
    flows.push(Box::new(GridSupply {
        level: sc.battery0,
    }));
    let fast = Registry::new(flows, &state.stocks, aux).expect("fast registry");
    (state, bio, fast)
}

fn build(
    sc: &SealedStationScenario,
    thermal: &ThermalParams,
) -> (
    State,
    simcore::registry::Registry,
    simcore::registry::Registry,
) {
    build_sealed_station(
        &params::charge(),
        thermal,
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

/// `--control`: [`sized`]`(1.0)` through the reference's own driver reproduces the committed
/// golden, byte for byte.
fn control() {
    let sc = sized(1.0);
    let (state, bio, fast) = build(&sc, &params::thermal());
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
    let ours = simcore::snapshot::from_engine(states.last().expect("a day")).to_json();
    let committed = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../data/golden/sealed_station_state.json"),
    )
    .expect("golden file");
    println!(
        "control: sized(1.0) reproduces sealed_station_state.json byte for byte: {} \
         (rationed {rationed}, events {})",
        ours == committed,
        events.len()
    );
}

/// One crop cycle's readouts.
struct Cycle {
    days: usize,
    exhaled: f64,
    scrubbed: f64,
    pool_change: f64,
    best_day_share: f64,
    grain_max: f64,
    plant_c_max: f64,
    /// Change over the cycle of the crop's and the soil's carbon ([`held_c`]).
    crop_change: f64,
    soil_change: f64,
    soil_end: f64,
}

#[derive(Default)]
struct Minutes {
    chamber_min: f64,
    chamber_max: f64,
    node_min: f64,
    node_max: f64,
    co2_min: f64,
    co2_max: f64,
    o2_min: f64,
    o2_max: f64,
    cooler_blocked: u64,
    cooler_capped: u64,
    heater_capped: u64,
    above_setpoint: u64,
    store_min: f64,
    /// Every flow's legs on the water stocks ([`WATER_STOCKS`]), summed over the run (kg).
    water: std::collections::BTreeMap<(String, &'static str), f64>,
}

struct Run {
    a: f64,
    cycles: Vec<Cycle>,
    m: Minutes,
    first: State,
    last: State,
    rationed: (u64, u64),
    days: usize,
    crew: f64,
}

fn run(a: f64, crew: Option<f64>, days: usize) -> Run {
    let (mut sc, thermal) = (sized(a), outside_heat(a));
    if let Some(n) = crew {
        let k = n / sc.cabin.crew_count;
        sc.cabin.crew_count = n;
        sc.cabin.food_store0 *= k;
        sc.cabin.water_store0 *= k;
    }
    let chamber = sc.chamber;
    let f_resp = params::crew().respired_carbon_fraction;
    let (state, bio, fast) = on_grid(&sc, &thermal);
    let first = state.clone();
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
    let dt = sc.cabin_dt;
    let mut m = Minutes {
        chamber_min: f64::INFINITY,
        chamber_max: f64::NEG_INFINITY,
        node_min: f64::INFINITY,
        node_max: f64::NEG_INFINITY,
        co2_min: f64::INFINITY,
        co2_max: f64::NEG_INFINITY,
        o2_min: f64::INFINITY,
        o2_max: f64::NEG_INFINITY,
        store_min: f64::INFINITY,
        ..Minutes::default()
    };
    // The crop's cycle ends at each re-sow: the day index of each reset.
    let mut fast_steps: u64 = 0;
    let mut reset_days: Vec<usize> = Vec::new();
    let mut observe = |side: Side, before: &State, after: &State| match side {
        Side::Reset => reset_days.push((fast_steps / sc.steps_per_day) as usize),
        Side::Slow => {
            let env = bio_r.bind(before, sc.bio_dt);
            for f in bio.registry().flows() {
                let res = f.evaluate(before, &env, sc.bio_dt).expect("plant flow");
                tally(&mut m.water, f.id(), &res);
            }
        }
        Side::Fast => {
            fast_steps += 1;
            let t_ch = chamber_temperature(amount(after, CHAMBER), &chamber);
            let t_node = temperature(
                amount(after, NODE),
                thermal.heat_capacity,
                thermal.space_temperature,
            );
            m.chamber_min = m.chamber_min.min(t_ch);
            m.chamber_max = m.chamber_max.max(t_ch);
            m.node_min = m.node_min.min(t_node);
            m.node_max = m.node_max.max(t_node);
            m.co2_min = m.co2_min.min(amount(after, CARBON_POOL));
            m.co2_max = m.co2_max.max(amount(after, CARBON_POOL));
            m.o2_min = m.o2_min.min(amount(after, O2_POOL));
            m.o2_max = m.o2_max.max(amount(after, O2_POOL));
            m.store_min = m.store_min.min(amount(after, WATER_STORE));
            let env = fast_r.bind(before, dt);
            let t_before = chamber_temperature(amount(before, CHAMBER), &chamber);
            let node_before = temperature(
                amount(before, NODE),
                thermal.heat_capacity,
                thermal.space_temperature,
            );
            let phase = sc.phase(before).expect("the sowing clock");
            let setpoint = sc.chamber_setpoint(phase, chamber.warm_setpoint);
            if t_before > setpoint + 0.5 {
                m.above_setpoint += 1;
                if node_before >= t_before {
                    m.cooler_blocked += 1;
                }
            }
            for f in fast.registry().flows() {
                let cap = match f.id() {
                    CHAMBER_COOLING => chamber.cooling_capacity,
                    CHAMBER_HEATER => chamber.heater_capacity,
                    _ => {
                        let res = f.evaluate(before, &env, dt).expect("fast flow");
                        tally(&mut m.water, f.id(), &res);
                        continue;
                    }
                };
                let res = f.evaluate(before, &env, dt).expect("chamber flow");
                if leg(&res, CHAMBER).abs() >= 0.999 * cap * dt {
                    if f.id() == CHAMBER_COOLING {
                        m.cooler_capped += 1;
                    } else {
                        m.heater_capped += 1;
                    }
                }
            }
        }
    };
    let (states, totals) = two
        .run(DayOrder::Interleaved, state, days, &mut observe)
        .expect("two-rate run");
    // Day-end states, with the start in front: day d runs from `ends[d]` to `ends[d + 1]`.
    let mut ends = vec![first.clone()];
    ends.extend(states.iter().cloned());
    let ends = &ends[ends.len() - days - 1..];
    let mut bounds = vec![0];
    bounds.extend(reset_days.iter().copied().filter(|&d| d > 0 && d < days));
    bounds.push(days);
    bounds.dedup();
    let cycles = bounds
        .windows(2)
        .map(|w| {
            let (s, e) = (&ends[w[0]], &ends[w[1]]);
            let exhaled = (amount(s, FOOD_STORE) - amount(e, FOOD_STORE)) * f_resp;
            let mut best: f64 = f64::NEG_INFINITY;
            let mut grain: f64 = 0.0;
            let mut plant: f64 = 0.0;
            for d in w[0]..w[1] {
                let (x, y) = (&ends[d], &ends[d + 1]);
                let ex = (amount(x, FOOD_STORE) - amount(y, FOOD_STORE)) * f_resp;
                let sc = amount(y, CO2_REMOVED) - amount(x, CO2_REMOVED);
                let dp = amount(y, CARBON_POOL) - amount(x, CARBON_POOL);
                best = best.max((ex - sc - dp) / ex);
                grain = grain.max(amount(y, STORAGE_C));
                plant = plant.max(plant_c(y));
            }
            Cycle {
                days: w[1] - w[0],
                exhaled,
                scrubbed: amount(e, CO2_REMOVED) - amount(s, CO2_REMOVED),
                pool_change: amount(e, CARBON_POOL) - amount(s, CARBON_POOL),
                best_day_share: best,
                grain_max: grain,
                plant_c_max: plant,
                crop_change: held_c(e).0 - held_c(s).0,
                soil_change: held_c(e).1 - held_c(s).1,
                soil_end: held_c(e).1,
            }
        })
        .collect();
    Run {
        a,
        cycles,
        m,
        first,
        last: states.last().expect("a day").clone(),
        rationed: (totals.slow_rationed, totals.fast_rationed),
        days,
        crew: sc.cabin.crew_count,
    }
}

fn report(r: &Run) {
    let a = r.a;
    let (s, e) = (&r.first, &r.last);
    let m = &r.m;
    let air = sealed_station_scenario().bio.chamber_air_capacity_mol;
    println!(
        "\n=== {a} m², {} crew, {} days; rationed (plant, minute) {:?} ===",
        r.crew, r.days, r.rationed
    );
    println!("per crop cycle (closure share = ecosystem's net CO₂ draw / crew exhalation):");
    for (i, c) in r.cycles.iter().enumerate() {
        let net = c.exhaled - c.scrubbed - c.pool_change;
        println!(
            "  cycle {i}: {:4} days; exhaled {:9.1} mol ({:.2}/day), scrubbed {:9.1}; closure share \
             {:5.1} % (best day {:5.1} %); grain max {:.3} mol C/m², plant C max {:.3} mol C/m²",
            c.days,
            c.exhaled,
            c.exhaled / c.days as f64,
            c.scrubbed,
            100.0 * net / c.exhaled,
            100.0 * c.best_day_share,
            c.grain_max / a,
            c.plant_c_max / a
        );
        println!(
            "           net draw {:9.1} mol C = crop {:+9.1} + soil {:+9.1} (soil ends {:9.1};              residual {:+.3}); peak crop / exhaled {:5.1} %",
            net,
            c.crop_change,
            c.soil_change,
            c.soil_end,
            net - c.crop_change - c.soil_change,
            100.0 * c.plant_c_max / c.exhaled
        );
    }
    let (ex, scr, dp) = r.cycles.iter().fold((0.0, 0.0, 0.0), |(x, y, z), c| {
        (x + c.exhaled, y + c.scrubbed, z + c.pool_change)
    });
    println!("  whole run: closure share {:.1} %", 100.0 * (ex - scr - dp) / ex);
    println!(
        "room CO₂ {:.2}–{:.2} mol ({:.0}–{:.0} ppm of {air} mol); O₂ {:.1}–{:.1} mol ({:.2}–{:.2} %)",
        m.co2_min,
        m.co2_max,
        1e6 * m.co2_min / air,
        1e6 * m.co2_max / air,
        m.o2_min,
        m.o2_max,
        100.0 * m.o2_min / air,
        100.0 * m.o2_max / air
    );
    println!(
        "O₂ supply change {:+.1} mol (negative: made up from the tank; positive: surplus returned)",
        amount(e, O2_SUPPLY) - amount(s, O2_SUPPLY)
    );
    println!(
        "water: crew store {:.1} → {:.1} kg (min {:.1}); plants' water {:.1} → {:.1} kg",
        amount(s, WATER_STORE),
        amount(e, WATER_STORE),
        m.store_min,
        plant_water(s),
        plant_water(e)
    );
    println!("water moved, by flow and stock (kg over the run; + into the stock):");
    for ((id, stock), v) in &m.water {
        println!(
            "  {id:36} {stock:26} {v:+14.1} ({:+.2} kg/day)",
            v / r.days as f64
        );
    }
    let drawn = amount(s, BATTERY) - amount(e, BATTERY) + amount(s, GRID) - amount(e, GRID);
    println!(
        "electricity drawn {:.3e} J = {:.1} kWh/day",
        drawn,
        drawn / 3.6e6 / r.days as f64
    );
    println!(
        "chamber {:.2}–{:.2} °C; structure {:.1}–{:.1} K; minutes above setpoint+0.5 K {}, of them \
         cooler blocked (structure not colder) {}; cooler at cap {}; heater at cap {}",
        m.chamber_min - 273.15,
        m.chamber_max - 273.15,
        m.node_min,
        m.node_max,
        m.above_setpoint,
        m.cooler_blocked,
        m.cooler_capped,
        m.heater_capped
    );
    println!(
        "end: plant C {:.3} mol/m²; food store {:.1} mol; tank {:.1} kg; scrubbed total {:.1} mol",
        plant_c(e) / a,
        amount(e, FOOD_STORE),
        amount(e, CONDENSATE),
        amount(e, CO2_REMOVED) - amount(s, CO2_REMOVED)
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |name: &str| {
        args.iter()
            .position(|x| x == name)
            .and_then(|i| args.get(i + 1))
    };
    if args.iter().any(|x| x == "--control") {
        control();
        return;
    }
    let sc = sealed_station_scenario();
    let days = arg("--days").map_or(sc.days(), |d| d.parse().expect("--days N"));
    let area: f64 = arg("--area").map_or(200.0, |x| x.parse().expect("--area A"));
    let crew: Option<f64> = arg("--crew").map(|x| x.parse().expect("--crew N"));
    let big = sized(area);
    println!(
        "sizing at {area} m²: lamp {} W, chamber C {:.3e} J/K, cooler {} W, heater {} W, walls \
         {:.1} m², tank start {:.1} kg (working level {:.1} kg); crew {}",
        big.lamp_power_w,
        big.chamber.heat_capacity,
        big.chamber.cooling_capacity,
        big.chamber.heater_capacity,
        big.chamber.wall_area,
        big.bio.condensate0,
        working_level(&big.bio, &big.plant_params()),
        big.cabin.crew_count
    );
    let (one, many) = std::thread::scope(|s| {
        let one = s.spawn(|| run(1.0, None, days));
        let many = s.spawn(|| run(area, crew, days));
        (one.join().expect("1 m²"), many.join().expect("big"))
    });
    report(&one);
    report(&many);
}
