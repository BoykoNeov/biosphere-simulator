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
//! `station::sealed::with_averaged_lamp_heat`) — how the records before that date were made. The
//! golden control then reports DIFFERS by design; the room's temperature and its cooler and heater are
//! printed for both.

use std::collections::BTreeMap;

use domains::biosphere::stocks::{CONDENSATE, RN_VAR, SOIL_EVAP_TODAY, SOIL_WATER, WATER_VAPOR};
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
    build_sealed_station, sealed_bio_resolver, sealed_fast_resolver, sealed_reset_hook,
    with_averaged_lamp_heat,
};
use station::sowing::{sown_step, twin};

/// λ, J/kg — the model's own (`science.rs`, `LATENT_HEAT_VAPORIZATION`, private there).
const LAMBDA: f64 = 2.45e6;
const SECONDS_PER_DAY: f64 = 86400.0;
const TRANSPIRATION: &str = "biosphere.transpiration";
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
    let (state, bio, fast) = build_sealed_station(
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
    )
    .expect("build_sealed_station");
    let (bio, fast) = (EulerIntegrator::new(bio), EulerIntegrator::new(fast));
    let bio_r = sealed_bio_resolver(&lamp, &scenario).expect("bio resolver");
    let averaged = std::env::args().any(|a| a == "--averaged");
    let chamber = station::params::chamber();
    let fast_r = sealed_fast_resolver(&charge, &scenario).expect("fast resolver");
    let fast_r = if averaged {
        with_averaged_lamp_heat(fast_r, &scenario).expect("averaged")
    } else {
        fast_r
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
        room: Room {
            outdoor_max_c: f64::NEG_INFINITY,
            ..Room::default()
        },
        ..Probe::default()
    };
    let mut observe = |side: Side, before: &State, after: &State| match side {
        Side::Reset => {}
        Side::Slow => {
            p.close();
            let results = p.evaluate(two.slow.registry(), two.slow_resolver, two.slow_dt, before, after);
            let mut evap = 0.0;
            for (f, r) in two.slow.registry().flows().iter().zip(&results) {
                if f.id() == TRANSPIRATION {
                    evap += -leg_on(r, SOIL_WATER);
                    p.transpiration_seen += 1;
                }
            }
            let (t0, t1) = (before.aux[SOIL_EVAP_TODAY], after.aux[SOIL_EVAP_TODAY]);
            let soil = if t1 >= t0 { t1 - t0 } else { t1 };
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
        }
        Side::Fast => {
            let results = p.evaluate(two.fast.registry(), two.fast_resolver, two.fast_dt, before, after);
            for (f, r) in two.fast.registry().flows().iter().zip(&results) {
                let e = leg_on(r, CHAMBER);
                match f.id() {
                    LAMP => p.w.lamp_j += e,
                    CHAMBER_WALL => p.w.wall_in_j += e.max(0.0),
                    CHAMBER_HEATER => {
                        p.w.heater_j += e;
                        if e > 0.0 {
                            let key = if p.w.cold { "cold" } else { "warm" };
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
            let key = if p.w.cold { "cold" } else { "warm" };
            let bound = two.fast_resolver.bind(before, two.fast_dt);
            let phase = if p.w.cold { Phase::Cold } else { Phase::Warm };
            let set = bound.get(&twin(CHAMBER_SETPOINT_VAR, phase)).expect("setpoint");
            let outdoor = bound.get(OUTDOOR_TEMP_VAR).expect("outdoor");
            p.room.outdoor_max_c = p.room.outdoor_max_c.max(outdoor);
            if outdoor + 273.15 > set {
                p.room.outdoor_above_minutes += 1;
            }
            let dev = chamber_temperature(after.stocks[CHAMBER].amount, &chamber) - set;
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
}
