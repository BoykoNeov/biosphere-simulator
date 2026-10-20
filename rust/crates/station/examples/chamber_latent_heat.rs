//! The heat the chamber's water carries — an estimate (`docs/plans/post-roadmap-chamber-latent-heat.md`).
//!
//! The model gives evaporation no heat, so this measures the missing term from the model's own water
//! flows: the sealed station over its full horizon through the lab two-rate runner, every flow
//! evaluated on its step's own start (the draw census's route). Per plant step it takes the
//! transpiration flow's draw on the root zone (the crop's and the soil's together in a sealed build;
//! the soil's share from `soil_evap_today`), turns it into latent power with the model's λ, and sets
//! it against the chamber's heat legs over the minutes that follow (lamp, walls, heater, cooler).
//!
//! Controls: `rationed == 0`; each step's legs re-applied give the `after` state bit for bit; the end
//! state is byte-exact to the tree's own `sealed_station` golden. Writes nothing.
//!
//! Run from `rust/`: `cargo run --release -q -p station --example chamber_latent_heat`.
//!
//! `-- --averaged`: the room's lamp heat at its daily average on every minute (LAB-ONLY since the
//! switched lamp was adopted, 2026-10-15, `docs/plans/post-roadmap-room-lamp-heat.md`,
//! `station::sealed::with_averaged_lamp_heat`, with the chamber's start restored by
//! `with_averaged_chamber_start`) — how the records before that date were made. The golden control
//! then reports DIFFERS against today's golden by design; against the pre-adoption golden (`06a064e`)
//! it is byte-exact (measured 2026-10-15). The room's temperature and its cooler and heater are printed
//! for both.
//!
//! The evaporation's heat is booked in the room (`docs/plans/post-roadmap-latent-heat-books.md`; built
//! lab-only behind `--latent`, ADOPTED 2026-10-15); `-- --no-latent` is the lab switch back
//! (`station::latent_heat::without_latent_heat`), whose golden control reports DIFFERS by design. The
//! probe adds the booked heat's identity per window (heat drawn over a window = λ × the window's moved water), the
//! heater-saturated minutes, the minutes cooler + latent exceed the cooler's capacity, the minutes heat
//! is booked to a node not colder than the room, the battery's minimum and the simple form's vapour
//! bound.
//!
//! `-- --surroundings=structure|cabin|space`: the chamber's walls face a lab surroundings instead of the
//! outdoor weather (`ChamberSurroundings`; cabin at 18 °C, space at ε* = 0.01, WHAT-IF). The golden
//! control then reports DIFFERS by design.

use std::collections::BTreeMap;

use domains::biosphere::stocks::{
    CONDENSATE, RN_VAR, SOIL_DRY_DAYS, SOIL_EVAP_TODAY, SOIL_WATER, WATER_VAPOR,
};
use domains::goldens::{committed, compare, Verdict};
use simcore::arbitration;
use simcore::environment::{Environment, SourceResolver};
use simcore::flow::FlowResult;
use simcore::integrator::EulerIntegrator;
use simcore::registry::Registry;
use simcore::state::State;
use station::chamber::{
    chamber_temperature, CHAMBER, CHAMBER_COOLING, CHAMBER_HEATER, CHAMBER_SETPOINT_VAR,
    CHAMBER_WALL, OUTDOOR_TEMP_VAR,
};
use station::driver::{DayOrder, Side, TwoRate};
use station::flows::LAMP;
use station::scenario::{sealed_station_scenario, Phase};
use station::sealed::{
    build_sealed_station_in, sealed_bio_resolver, sealed_fast_resolver, sealed_reset_hook,
    with_averaged_chamber_start, with_averaged_lamp_heat,
};
use station::chamber::ChamberSurroundings;
use station::gas_exchange::GasExchangeStep;
use station::latent_heat::{without_latent_heat, CHAMBER_LATENT_HEAT};
use station::sowing::{sown_step, twin};

/// λ, J/kg — the model's own (`science.rs`, `LATENT_HEAT_VAPORIZATION`, private there).
const LAMBDA: f64 = 2.45e6;
const SECONDS_PER_DAY: f64 = 86400.0;
const TRANSPIRATION: &str = "biosphere.transpiration";
const RECYCLING: &str = "biosphere.recycling";
const CONDENSATION: &str = "biosphere.condensation";

/// One plant step and the minutes after it (the interleaved day runs the plant step first).
#[derive(Default, Clone)]
struct Window {
    open: bool,
    dark: bool,
    cold: bool,
    /// Water off the root zone through the transpiration flow (kg), and the soil's share.
    evap_kg: f64,
    soil_kg: f64,
    /// Latent power over the window (W): λ·evap over the plant step's seconds.
    latent_w: f64,
    /// Heat legs on the chamber over the window's minutes (J).
    lamp_j: f64,
    wall_in_j: f64,
    heater_j: f64,
    seconds: f64,
    /// The chamber air's water vapour at the plant step's start (kg) — whether the night air holds
    /// its humidity setting (`docs/plans/post-roadmap-night-stomata.md`).
    vapour_kg: f64,
    /// Heat the latent flow drew from the chamber over the window's minutes (J, ≥ 0).
    booked_j: f64,
    /// Σ (room temperature − setpoint) over the window's minutes (K·min).
    dev_sum: f64,
}

/// Run-wide sums for one class of window.
#[derive(Default)]
struct Class {
    windows: u64,
    evap_kg: f64,
    soil_kg: f64,
    seconds: f64,
    lamp_j: f64,
    heat_in_j: f64,
    latent_j: f64,
    unfunded: u64,
    unfunded_j: f64,
    vapour_kg: f64,
    dev_sum: f64,
}

impl Class {
    fn add(&mut self, w: &Window) {
        let latent = w.latent_w * w.seconds;
        let heat_in = w.lamp_j + w.wall_in_j + w.heater_j;
        self.windows += 1;
        self.evap_kg += w.evap_kg;
        self.soil_kg += w.soil_kg;
        self.seconds += w.seconds;
        self.lamp_j += w.lamp_j;
        self.heat_in_j += heat_in;
        self.latent_j += latent;
        self.vapour_kg += w.vapour_kg;
        self.dev_sum += w.dev_sum;
        if latent > heat_in {
            self.unfunded += 1;
            self.unfunded_j += latent - heat_in;
        }
    }
}

#[derive(Default)]
struct Probe {
    w: Window,
    classes: BTreeMap<&'static str, Class>,
    /// Σ legs by (flow, stock) over the run, for the water stocks and the chamber.
    legs: BTreeMap<(String, String), f64>,
    /// Heater-on minutes, and the latent power held over them (J), by phase.
    heater_minutes: BTreeMap<&'static str, (u64, f64, f64)>,
    min_crop_kg: f64,
    transpiration_seen: u64,
    mismatched: u64,
    first_mismatch: Option<String>,
    firings: u64,
    room: Room,
    /// The step count at each re-sow (the season reset).
    resets: Vec<u64>,
    /// The crop at each re-sow, before the reset: grain, leaf, stem, root (mol C).
    harvests: Vec<[f64; 4]>,
    /// Thermal time at the start of the current day, and at each re-sow (overshoot past maturity,
    /// the last day's gain), °C·d.
    tt_day_start: f64,
    maturity_margins: Vec<(f64, f64)>,
    latent: LatentBooks,
    /// The soil's evaporation against its watering: trigger events (plant steps whose recycling
    /// moved water), and plant steps and soil evaporation (kg) by stage — a wet surface
    /// (`soil_dry_days == 0` at the step's start) and a drying one.
    watering_events: u64,
    watered_kg: f64,
    wet_steps: u64,
    dry_steps: u64,
    soil_wet_kg: f64,
    soil_dry_kg: f64,
}

/// The booked latent heat's own checks, and the heater's and battery's limits.
#[derive(Default)]
struct LatentBooks {
    /// Windows checked, and the largest |booked − λ·evap| (J, and relative) over them.
    windows: u64,
    max_rel_err: f64,
    max_abs_err_j: f64,
    booked_j: f64,
    /// Minutes the heater gave its whole capacity, by phase.
    heater_saturated: BTreeMap<&'static str, u64>,
    /// Minutes cooler + latent exceeded the cooler's capacity, by phase.
    over_coil: BTreeMap<&'static str, u64>,
    /// Minutes heat was booked to a node not colder than the room.
    node_not_colder: u64,
    battery_min_j: f64,
    vapour_min_kg: f64,
    vapour_max_kg: f64,
    /// The first window's evaporation (kg) and whether it was dark.
    first_window: Option<(f64, bool)>,
}

/// The room's temperature against its setpoint, and its cooler, per phase ("cold" / "warm").
#[derive(Default)]
struct Room {
    /// (minutes, min T − setpoint, max T − setpoint, minutes > 0.1 K above, cooler-at-capacity minutes,
    /// cooler J removed) by phase.
    by_phase: BTreeMap<&'static str, (u64, f64, f64, u64, u64, f64)>,
    /// The warmest outdoor air the walls faced (°C), and the minutes it was above the setpoint.
    outdoor_max_c: f64,
    outdoor_above_minutes: u64,
}
impl Probe {
    fn close(&mut self) {
        if !self.w.open {
            return;
        }
        let key = match (self.w.cold, self.w.dark) {
            (false, false) => "warm lit",
            (false, true) => "warm dark",
            (true, false) => "cold lit",
            (true, true) => "cold dark",
        };
        let w = self.w.clone();
        self.classes.entry(key).or_default().add(&w);
        if self.latent.first_window.is_none() {
            self.latent.first_window = Some((w.evap_kg, w.dark));
        }
        if w.booked_j > 0.0 || w.evap_kg > 0.0 {
            let want = LAMBDA * w.evap_kg;
            let err = (w.booked_j - want).abs();
            self.latent.windows += 1;
            self.latent.max_abs_err_j = self.latent.max_abs_err_j.max(err);
            if want > 0.0 {
                self.latent.max_rel_err = self.latent.max_rel_err.max(err / want);
            }
        }
        self.w = Window::default();
    }

    /// Evaluate every flow on `before`, record the legs, and check the step re-applies bit for bit.
    fn evaluate(
        &mut self,
        registry: &Registry,
        resolver: &SourceResolver,
        dt: f64,
        before: &State,
        after: &State,
    ) -> Vec<FlowResult> {
        let bound = resolver.bind(before, dt);
        let results: Vec<FlowResult> = registry
            .flows()
            .iter()
            .map(|f| f.evaluate(before, &bound, dt).expect("evaluate"))
            .collect();
        for (f, r) in registry.flows().iter().zip(&results) {
            for leg in &r.legs {
                let s = leg.stock.as_str();
                if [SOIL_WATER, WATER_VAPOR, CONDENSATE, CHAMBER].contains(&s) {
                    *self
                        .legs
                        .entry((f.id().to_string(), s.to_string()))
                        .or_insert(0.0) += leg.amount;
                }
            }
        }
        let (scaled, fired) = arbitration::min_scaling(&results, &before.stocks).expect("scaling");
        self.firings += fired;
        let mut delta: BTreeMap<&str, f64> = BTreeMap::new();
        for r in &scaled {
            for leg in &r.legs {
                *delta.entry(leg.stock.as_str()).or_insert(0.0) += leg.amount;
            }
        }
        for (id, stock) in &before.stocks {
            let expected = match delta.get(id.as_str()) {
                Some(d) => stock.amount + 1.0 * d,
                None => stock.amount,
            };
            if expected.to_bits() != after.stocks[id].amount.to_bits() {
                self.mismatched += 1;
                self.first_mismatch
                    .get_or_insert_with(|| format!("step {} ({id})", before.n));
                break;
            }
        }
        results
    }
}

fn leg_on(r: &FlowResult, stock: &str) -> f64 {
    r.legs.iter().filter(|l| l.stock == stock).map(|l| l.amount).sum()
}

fn main() {
    let scenario = sealed_station_scenario();
    let charge = domains::params::charge();
    let lamp = station::params::lamp();
    let surroundings = match std::env::args()
        .find_map(|a| a.strip_prefix("--surroundings=").map(str::to_string))
        .as_deref()
    {
        None | Some("outdoor") => ChamberSurroundings::Outdoor,
        Some("structure") => ChamberSurroundings::Structure,
        Some("cabin") => ChamberSurroundings::Cabin {
            temperature_k: 291.15,
        },
        Some("space") => ChamberSurroundings::Space {
            effective_emittance: 0.01,
        },
        Some(other) => panic!("unknown surroundings {other:?}"),
    };
    let (state, bio, fast) = build_sealed_station_in(
        &charge,
        &domains::params::thermal(),
        &domains::params::crew(),
        &domains::params::eclss(),
        &station::params::water_recovery(),
        &lamp,
        &station::params::harvest(),
        &scenario,
        false,
        false,
        GasExchangeStep::Minute,
        surroundings,
    )
    .expect("build_sealed_station");
    let latent = !std::env::args().any(|a| a == "--no-latent");
    let (state, bio, fast) = if latent {
        (state, bio, fast)
    } else {
        without_latent_heat(state, bio, fast).expect("no latent")
    };
    let (bio, fast) = (EulerIntegrator::new(bio), EulerIntegrator::new(fast));
    let bio_r = sealed_bio_resolver(&lamp, &scenario).expect("bio resolver");
    let averaged = std::env::args().any(|a| a == "--averaged");
    let chamber = station::params::chamber();
    let thermal = domains::params::thermal();
    let fast_r = sealed_fast_resolver(&charge, &domains::params::crew(), &scenario).expect("fast resolver");
    let (state, fast_r) = if averaged {
        (
            with_averaged_chamber_start(state, &scenario).expect("averaged start"),
            with_averaged_lamp_heat(fast_r, &scenario).expect("averaged"),
        )
    } else {
        (state, fast_r)
    };
    let reset = sealed_reset_hook(&scenario);
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
    let vapour0 = state.stocks[WATER_VAPOR].amount;
    let mut p = Probe {
        min_crop_kg: f64::INFINITY,
        latent: LatentBooks {
            battery_min_j: f64::INFINITY,
            vapour_min_kg: f64::INFINITY,
            vapour_max_kg: f64::NEG_INFINITY,
            ..LatentBooks::default()
        },
        room: Room {
            outdoor_max_c: f64::NEG_INFINITY,
            ..Room::default()
        },
        ..Probe::default()
    };
    let mut observe = |side: Side, before: &State, after: &State| match side {
        Side::Reset => {
            use domains::biosphere::stocks::{LEAF_C, ROOT_C, STEM_C, STORAGE_C};
            p.resets.push(after.n);
            let pheno = scenario.pheno();
            let tt = before.aux[domains::biosphere::stocks::THERMAL_TIME];
            p.maturity_margins.push((
                tt - (pheno.tsum_anthesis + pheno.tsum_maturity),
                tt - p.tt_day_start,
            ));
            p.harvests.push([STORAGE_C, LEAF_C, STEM_C, ROOT_C].map(|k| before.stocks[k].amount));
        }
        Side::Slow => {
            p.close();
            if before.n.is_multiple_of(scenario.bio_steps_per_day) {
                p.tt_day_start = before.aux[domains::biosphere::stocks::THERMAL_TIME];
            }
            let results = p.evaluate(two.slow.registry(), two.slow_resolver, two.slow_dt, before, after);
            let mut evap = 0.0;
            for (f, r) in two.slow.registry().flows().iter().zip(&results) {
                if f.id() == TRANSPIRATION {
                    evap += -leg_on(r, SOIL_WATER);
                    p.transpiration_seen += 1;
                }
                if f.id() == RECYCLING && leg_on(r, SOIL_WATER) > 0.0 {
                    p.watering_events += 1;
                    p.watered_kg += leg_on(r, SOIL_WATER);
                }
            }
            let (t0, t1) = (before.aux[SOIL_EVAP_TODAY], after.aux[SOIL_EVAP_TODAY]);
            let soil = if t1 >= t0 { t1 - t0 } else { t1 };
            if before.aux[SOIL_DRY_DAYS] == 0.0 {
                p.wet_steps += 1;
                p.soil_wet_kg += soil;
            } else {
                p.dry_steps += 1;
                p.soil_dry_kg += soil;
            }
            p.min_crop_kg = p.min_crop_kg.min(evap - soil);
            let since = before.n - sown_step(before).expect("sown");
            let phase = scenario.phase_since_sowing(since);
            // The lamp's net radiation is carried as its two phase twins (`crate::sowing`).
            let rn = two
                .slow_resolver
                .bind(before, two.slow_dt)
                .get(&twin(RN_VAR, phase))
                .expect("rn");
            p.w = Window {
                open: true,
                dark: rn == 0.0,
                cold: phase == Phase::Cold,
                evap_kg: evap,
                soil_kg: soil,
                latent_w: LAMBDA * evap / (two.slow_dt * SECONDS_PER_DAY),
                vapour_kg: before.stocks[WATER_VAPOR].amount,
                ..Window::default()
            };
            let v = before.stocks[WATER_VAPOR].amount;
            p.latent.vapour_min_kg = p.latent.vapour_min_kg.min(v);
            p.latent.vapour_max_kg = p.latent.vapour_max_kg.max(v);
        }
        Side::Fast => {
            let results = p.evaluate(two.fast.registry(), two.fast_resolver, two.fast_dt, before, after);
            let key = if p.w.cold { "cold" } else { "warm" };
            let mut latent_out = 0.0;
            for (f, r) in two.fast.registry().flows().iter().zip(&results) {
                let e = leg_on(r, CHAMBER);
                match f.id() {
                    CHAMBER_LATENT_HEAT => {
                        latent_out = -e;
                        p.w.booked_j -= e;
                        p.latent.booked_j -= e;
                    }
                    LAMP => p.w.lamp_j += e,
                    CHAMBER_WALL => p.w.wall_in_j += e.max(0.0),
                    CHAMBER_HEATER => {
                        p.w.heater_j += e;
                        if e >= chamber.heater_capacity * two.fast_dt * (1.0 - 1e-12) {
                            *p.latent.heater_saturated.entry(key).or_default() += 1;
                        }
                        if e > 0.0 {
                            let h = p.heater_minutes.entry(key).or_default();
                            h.0 += 1;
                            h.1 += p.w.latent_w * two.fast_dt;
                            h.2 += e;
                        }
                    }
                    _ => {}
                }
            }
            p.w.seconds += two.fast_dt;
            let t_node = domains::thermal::temperature(
                before.stocks[domains::thermal::NODE].amount,
                thermal.heat_capacity,
                thermal.space_temperature,
            );
            if latent_out > 0.0
                && t_node >= chamber_temperature(before.stocks[CHAMBER].amount, &chamber)
            {
                p.latent.node_not_colder += 1;
            }
            p.latent.battery_min_j = p
                .latent
                .battery_min_j
                .min(after.stocks[domains::power::BATTERY].amount);
            let bound = two.fast_resolver.bind(before, two.fast_dt);
            let phase = if p.w.cold { Phase::Cold } else { Phase::Warm };
            let set = bound.get(&twin(CHAMBER_SETPOINT_VAR, phase)).expect("setpoint");
            let outdoor = bound.get(OUTDOOR_TEMP_VAR).expect("outdoor");
            p.room.outdoor_max_c = p.room.outdoor_max_c.max(outdoor);
            if outdoor + 273.15 > set {
                p.room.outdoor_above_minutes += 1;
            }
            let dev = chamber_temperature(after.stocks[CHAMBER].amount, &chamber) - set;
            p.w.dev_sum += dev;
            let cooled = -results
                .iter()
                .zip(two.fast.registry().flows())
                .filter(|(_, f)| f.id() == CHAMBER_COOLING)
                .map(|(r, _)| leg_on(r, CHAMBER))
                .sum::<f64>();
            let r = p
                .room
                .by_phase
                .entry(key)
                .or_insert((0, f64::INFINITY, f64::NEG_INFINITY, 0, 0, 0.0));
            r.0 += 1;
            r.1 = r.1.min(dev);
            r.2 = r.2.max(dev);
            if dev > 0.1 {
                r.3 += 1;
            }
            if cooled >= chamber.cooling_capacity * two.fast_dt * (1.0 - 1e-12) {
                r.4 += 1;
            }
            r.5 += cooled;
            if cooled + latent_out > chamber.cooling_capacity * two.fast_dt {
                *p.latent.over_coil.entry(key).or_default() += 1;
            }
        }
    };
    let (states, totals) = two
        .run(DayOrder::Interleaved, state, scenario.days(), &mut observe)
        .expect("two-rate run");
    p.close();
    let end = states.last().expect("a day");

    // --- controls ---
    let golden = "sealed_station_state.json";
    let numerics = station::goldens::all()
        .into_iter()
        .find(|g| g.name == golden)
        .expect("on the roster")
        .numerics;
    let verdict = match compare(
        &simcore::snapshot::from_engine(end).to_json(),
        &committed(golden),
        numerics,
    ) {
        Verdict::ByteExact => "byte-exact".to_string(),
        Verdict::StructurallyEqual => "structurally equal (off-platform)".to_string(),
        Verdict::Differs(why) => format!("DIFFERS — {why}"),
    };
    // Which value the sealed flow reads is the tree's (the golden control pins the tree); both
    // fields print, since the commit before the adoption carries the chamber's 7.5 read by nothing.
    let t = domains::biosphere::params::biosphere().transp;
    println!(
        "lamp heat: {}",
        if averaged { "daily average (lab; the reference until 2026-10-15)" } else { "switched with the light (the reference)" }
    );
    println!(
        "sealed station, {} days; transpiration.yaml: open-field ra {} s/m, chamber ra {} s/m",
        states.len() - 1,
        t.aerodynamic_resistance,
        t.chamber_aerodynamic_resistance
    );
    println!(
        "controls: rationed slow {} fast {} (probe {}); legs re-applied: {} mismatches{}; golden: {verdict}; transpiration evaluated {} times; min crop share {:.3e} kg",
        totals.slow_rationed,
        totals.fast_rationed,
        p.firings,
        p.mismatched,
        p.first_mismatch.map(|m| format!(" (first {m})")).unwrap_or_default(),
        p.transpiration_seen,
        p.min_crop_kg
    );

    // --- water books ---
    println!(
        "\nsoil against watering: {} trigger events, {:.4} kg into the root zone; plant steps wet {} / \
         drying {}; soil evaporation wet {:.4} kg / drying {:.4} kg",
        p.watering_events, p.watered_kg, p.wet_steps, p.dry_steps, p.soil_wet_kg, p.soil_dry_kg
    );
    println!("\nlegs over the run (kg for water, J for the chamber), by flow:");
    for ((flow, stock), v) in &p.legs {
        println!("  {flow:<40} {stock:<26} {v:>16.6e}");
    }
    let get = |f: &str, s: &str| p.legs.get(&(f.to_string(), s.to_string())).copied().unwrap_or(0.0);
    let evaporated = -get(TRANSPIRATION, SOIL_WATER);
    let condensed = get(TRANSPIRATION, CONDENSATE) + get(CONDENSATION, CONDENSATE);
    let d_vapour = end.stocks[WATER_VAPOR].amount - vapour0;
    let soil_out: f64 = p
        .legs
        .iter()
        .filter(|((_, s), v)| s == SOIL_WATER && **v < 0.0)
        .map(|(_, v)| -v)
        .sum();
    println!(
        "\nevaporated {evaporated:.4} kg; condensed {condensed:.4} kg; Δvapour {d_vapour:.3e} kg; residual {:.3e} kg",
        evaporated - condensed - d_vapour
    );
    println!("every withdrawal from the root zone: {soil_out:.4} kg (evaporation is {:.2} % of it)", 100.0 * evaporated / soil_out);

    // --- the estimate ---
    let run_s = (states.len() - 1) as f64 * SECONDS_PER_DAY;
    let all_soil: f64 = p.classes.values().map(|c| c.soil_kg).sum();
    let all_evap: f64 = p.classes.values().map(|c| c.evap_kg).sum();
    println!(
        "\nmean latent power over the run: {:.2} W; soil's share of evaporation {:.2} %",
        LAMBDA * all_evap / run_s,
        100.0 * all_soil / all_evap
    );
    println!("by window class (a window = one plant step and its minutes):");
    println!("  class        windows  crop W   soil W   lamp W   heat-in W  latent/lamp  unfunded (windows, MJ)");
    for (k, c) in &p.classes {
        let crop_w = LAMBDA * (c.evap_kg - c.soil_kg) / c.seconds;
        let soil_w = LAMBDA * c.soil_kg / c.seconds;
        println!(
            "  {k:<11} {:>8} {crop_w:>8.2} {soil_w:>8.2} {:>8.2} {:>10.2} {:>12.3}  {:>6} ({:.2})",
            c.windows,
            c.lamp_j / c.seconds,
            c.heat_in_j / c.seconds,
            if c.lamp_j > 0.0 { c.latent_j / c.lamp_j } else { f64::NAN },
            c.unfunded,
            c.unfunded_j / 1e6
        );
    }
    let crop_rate = |k: &str| {
        p.classes
            .get(k)
            .map(|c| (c.evap_kg - c.soil_kg) / c.seconds)
            .unwrap_or(f64::NAN)
    };
    println!(
        "crop transpiration rate, dark / lit: warm {:.3}, cold {:.3}",
        crop_rate("warm dark") / crop_rate("warm lit"),
        crop_rate("cold dark") / crop_rate("cold lit")
    );
    println!("chamber air's water vapour, mean at the plant step's start (kg):");
    for (k, c) in &p.classes {
        println!("  {k:<11} {:.6}", c.vapour_kg / c.windows as f64);
    }
    println!("heater-on minutes and the latent power held over them:");
    for (k, (n, latent_j, heater_j)) in &p.heater_minutes {
        let s = *n as f64 * two.fast_dt;
        println!(
            "  {k:<5} {n:>9} min  heater {:.2} W mean, {:.2} MJ total; latent {:.2} W mean, {:.2} MJ total",
            heater_j / s,
            heater_j / 1e6,
            latent_j / s,
            latent_j / 1e6
        );
    }

    // --- the room (docs/plans/post-roadmap-room-lamp-heat.md) ---
    println!(
        "
the room against its setpoint (minutes; T − setpoint min / max K; minutes > 0.1 K above; cooler at capacity; cooler MJ):"
    );
    for (k, (n, lo, hi, above, sat, cooled)) in &p.room.by_phase {
        println!(
            "  {k:<5} {n:>9} {lo:>+9.4} {hi:>+9.4} {above:>9} {sat:>9} {:>10.2}",
            cooled / 1e6
        );
    }
    println!(
        "outdoor air: warmest {:.2} °C; minutes above the setpoint {}",
        p.room.outdoor_max_c, p.room.outdoor_above_minutes
    );
    use domains::biosphere::stocks::{LEAF_C, ROOT_C, STEM_C, STORAGE_C};
    use domains::power::BATTERY;
    println!("end state:");
    for k in [LEAF_C, STEM_C, ROOT_C, STORAGE_C, SOIL_WATER, CONDENSATE, BATTERY, CHAMBER, domains::thermal::NODE] {
        println!("  {k:<32} {:.10e}", end.stocks[k].amount);
    }
    println!("  harvested (food store)           {:.10e}", end.stocks.get("crew.food_store").map_or(f64::NAN, |s| s.amount));

    // --- the booked latent heat (docs/plans/post-roadmap-latent-heat-books.md) ---
    println!(
        "re-sown at day {:?}",
        p.resets.iter().map(|n| *n as f64 / scenario.bio_steps_per_day as f64).collect::<Vec<_>>()
    );
    println!("maturity at each re-sow: thermal time past the threshold (°C·d), the last day's gain (°C·d), the overshoot as a fraction of that day");
    for (over, gain) in &p.maturity_margins {
        println!("  {over:>9.4} {gain:>9.4} {:>7.4}", over / gain);
    }
    println!("the room's mean temperature against its setpoint by window class (K):");
    for (k, c) in &p.classes {
        println!("  {k:<11} {:+.5}", c.dev_sum * two.fast_dt / c.seconds);
    }
    println!("the crop at each maturity (mol C): grain, leaf, stem, root");
    for h in &p.harvests {
        println!("  {:.6e} {:.6e} {:.6e} {:.6e}", h[0], h[1], h[2], h[3]);
    }
    let l = &p.latent;
    println!(
        "\nlatent heat booked in the room: {}; walls face {surroundings:?}",
        if latent { "yes (the reference)" } else { "no (lab, --no-latent)" }
    );
    println!(
        "  booked {:.6} GJ; λ × evaporated {:.6} GJ; window identity over {} windows: max |booked − λE| {:.3e} J, max relative {:.3e}",
        l.booked_j / 1e9,
        LAMBDA * evaporated / 1e9,
        l.windows,
        l.max_abs_err_j,
        l.max_rel_err
    );
    if let Some((kg, dark)) = l.first_window {
        println!(
            "  first window: {kg:.6e} kg evaporated ({}), latent {:.4} W over it",
            if dark { "dark" } else { "lit" },
            LAMBDA * kg / (two.slow_dt * SECONDS_PER_DAY)
        );
    }
    let heater_total: f64 = p.heater_minutes.values().map(|h| h.2).sum();
    println!("  heater total {:.4} GJ", heater_total / 1e9);
    println!("  heater at full capacity (minutes): {:?}", l.heater_saturated);
    println!("  cooler + latent above the cooler's capacity (minutes): {:?}", l.over_coil);
    println!("  minutes heat was booked to a node not colder than the room: {}", l.node_not_colder);
    println!("  battery minimum {:.6e} J", l.battery_min_j);
    println!(
        "  the simple form's bound: chamber air vapour {:.4}–{:.4} kg at plant-step starts → λ × range {:.3} MJ; λ × Δvapour over the run {:.3} MJ",
        l.vapour_min_kg,
        l.vapour_max_kg,
        LAMBDA * (l.vapour_max_kg - l.vapour_min_kg) / 1e6,
        LAMBDA * d_vapour / 1e6
    );
}
