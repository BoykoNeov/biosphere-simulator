//! The air-split baseline — Step 3c slice 2a's measurement BEFORE any code,
//! `docs/plans/post-roadmap-room-temperature.md` §11/§13, as one command:
//!
//! ```text
//! cargo run --release -q -p station --example air_split_baseline
//! ```
//!
//! Runs the frozen sealed station (shared air) and reads the crop's CO₂ draw in **absolute
//! moles**: per slow step (the gross withdrawal from `CARBON_POOL` the biosphere's flows make,
//! after option C's end-of-step solve) and per day, against the CO₂ held at the step's start.
//! Also the fast side's budget on the same pool: the scrubber's take (`k·CO₂·dt`, read off the
//! step's starting state) and the crew's emission (fast net + scrubber).
//!
//! Control: the end state byte-matches the committed `sealed_station_state.json`, so this loop
//! IS the reference run. It writes nothing and takes no decision.

use domains::biosphere::params as bio_params;
use domains::biosphere::readouts::withdrawal_demand;
use domains::biosphere::science::{humidity_target_kg, saturation_vapour_kg};
use domains::biosphere::stocks::{CARBON_POOL, TEMP_VAR, WATER_VAPOR};
use domains::eclss::CABIN_H2O;
use domains::goldens::{committed, compare, Verdict};
use domains::params;
use simcore::environment::Environment;
use simcore::flow::FlowResult;
use simcore::integrator::EulerIntegrator;
use simcore::state::State;
use station::driver::{DayOrder, Side, TwoRate};
use station::params as station_params;
use station::scenario::sealed_station_scenario;
use station::sealed::{
    build_sealed_station, sealed_bio_resolver, sealed_fast_resolver, sealed_reset_hook,
};

const GOLDEN: &str = "sealed_station_state.json";

/// The cabin's air (mol) and a BVAD-sized plant chamber's (§11: 0.67 m³ × 41.29 mol/m³).
const CABIN_AIR_MOL: f64 = 9500.0;
const SMALL_CHAMBER_MOL: f64 = 27.663966;

const TRANSPIRATION: &str = "biosphere.transpiration";

/// One day's CO₂ books on the shared pool (mol).
#[derive(Default, Clone)]
struct Day {
    /// Σ the crop's gross withdrawals over the day's slow steps.
    crop_gross: f64,
    /// Σ the slow side's net change on the pool (crop + soil respiration back in).
    slow_net: f64,
    /// Σ the fast side's net change on the pool.
    fast_net: f64,
    /// Σ the scrubber's take, `k·CO₂·dt` per fast step.
    scrubbed: f64,
    /// The pool at the day's slow steps' starts: min / max.
    pool_min: f64,
    pool_max: f64,
    /// Σ over the day's slow steps: temperature (°C), the shared room's relative humidity,
    /// the cabin vapour per mol of air, and the humidity target per mol of air (kg/mol).
    temp: f64,
    rh: f64,
    cabin_conc: f64,
    target_conc: f64,
    /// Σ transpiration (kg) and the part of it that reached the air.
    transpired: f64,
    to_air: f64,
    slow_steps: u64,
}

/// Grow `days` to hold day `d`.
fn day_of(days: &mut Vec<Day>, d: usize) {
    while days.len() <= d {
        days.push(Day {
            pool_min: f64::INFINITY,
            pool_max: f64::NEG_INFINITY,
            ..Day::default()
        });
    }
}

fn main() {
    let charge = params::charge();
    let eclss = params::eclss();
    let lamp = station_params::lamp();
    let scenario = sealed_station_scenario();
    let (state, bio, fast) = build_sealed_station(
        &charge,
        &params::thermal(),
        &params::crew(),
        &eclss,
        &station_params::water_recovery(),
        &lamp,
        &station_params::harvest(),
        &scenario,
        false,
        false,
    )
    .expect("build_sealed_station");
    let (bio, fast) = (EulerIntegrator::new(bio), EulerIntegrator::new(fast));
    let bio_r = sealed_bio_resolver(&lamp, &scenario).expect("sealed_bio_resolver");
    let fast_r = sealed_fast_resolver(&charge, &scenario).expect("sealed_fast_resolver");
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

    let slow_per_day = scenario.bio_steps_per_day;
    let fast_per_day = scenario.steps_per_day;
    let k_scrub = eclss.co2_scrub_rate;
    let fast_dt = scenario.cabin_dt;

    let mut days: Vec<Day> = Vec::new();
    let mut slow_k: u64 = 0;
    let mut fast_k: u64 = 0;
    // Worst single slow step: (gross mol, pool at start, day).
    let mut worst = (0.0_f64, 0.0_f64, 0.0_f64);
    let mut steps_drawing: u64 = 0;
    let mut gross_sum = 0.0_f64;
    let setpoint = bio_params::biosphere().water.humidity_setpoint;
    // Water: the worst one-step transpiration against a small chamber's humidity target,
    // and how often the cabin's air is the wetter of the two (per mol of air).
    let mut worst_water = (0.0_f64, 0.0_f64, 0.0_f64); // (flux kg, small target kg, day)
    let mut ratio_sum = 0.0_f64;
    let mut ratio_n: u64 = 0;
    let mut cabin_wetter: u64 = 0;
    let mut chamber_wetter: u64 = 0;

    let mut observe = |side: Side, before: &State, after: &State| match side {
        Side::Reset => {}
        Side::Slow => {
            let d = (slow_k / slow_per_day) as usize;
            day_of(&mut days, d);
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
            let gross = demand.get(CARBON_POOL).copied().unwrap_or(0.0);
            let pool = before.stocks[CARBON_POOL].amount;
            let row = &mut days[d];
            row.crop_gross += gross;
            row.slow_net += after.stocks[CARBON_POOL].amount - pool;
            row.pool_min = row.pool_min.min(pool);
            row.pool_max = row.pool_max.max(pool);
            if gross > 0.0 {
                steps_drawing += 1;
                gross_sum += gross;
            }
            if gross > worst.0 {
                worst = (gross, pool, slow_k as f64 / slow_per_day as f64);
            }
            // Water.
            let temp = bound.get(TEMP_VAR).expect("temp");
            let vapour = before.stocks[WATER_VAPOR].amount;
            let cabin = before.stocks[CABIN_H2O].amount;
            let (mut flux, mut air) = (0.0, 0.0);
            for (f, r) in bio.registry().flows().iter().zip(&results) {
                if f.id() == TRANSPIRATION {
                    for leg in &r.legs {
                        if leg.amount < 0.0 {
                            flux -= leg.amount;
                        } else if leg.stock == WATER_VAPOR {
                            air += leg.amount;
                        }
                    }
                }
            }
            let target_conc = humidity_target_kg(temp, 1.0, setpoint);
            let cabin_conc = cabin / CABIN_AIR_MOL;
            row.temp += temp;
            row.rh += vapour / saturation_vapour_kg(temp, CABIN_AIR_MOL);
            row.cabin_conc += cabin_conc;
            row.target_conc += target_conc;
            row.transpired += flux;
            row.to_air += air;
            row.slow_steps += 1;
            if cabin_conc > target_conc {
                cabin_wetter += 1;
            } else {
                chamber_wetter += 1;
            }
            let small_target = humidity_target_kg(temp, SMALL_CHAMBER_MOL, setpoint);
            if flux > 0.0 {
                ratio_sum += flux / small_target;
                ratio_n += 1;
            }
            if flux > worst_water.0 {
                worst_water = (flux, small_target, slow_k as f64 / slow_per_day as f64);
            }
            slow_k += 1;
        }
        Side::Fast => {
            let d = (fast_k / fast_per_day) as usize;
            day_of(&mut days, d);
            let pool = before.stocks[CARBON_POOL].amount;
            let row = &mut days[d];
            row.fast_net += after.stocks[CARBON_POOL].amount - pool;
            row.scrubbed += k_scrub * pool * fast_dt;
            fast_k += 1;
        }
    };

    let (states, totals) = two
        .run(DayOrder::Interleaved, state, scenario.days(), &mut observe)
        .expect("two-rate run");

    // Control: this loop is the reference run.
    let numerics = station::goldens::all()
        .into_iter()
        .find(|g| g.name == GOLDEN)
        .expect("golden on the roster")
        .numerics;
    let snap = simcore::snapshot::from_engine(states.last().expect("a day")).to_json();
    let verdict = match compare(&snap, &committed(GOLDEN), numerics) {
        Verdict::ByteExact => "byte-exact".to_string(),
        Verdict::StructurallyEqual => "structurally equal (off-platform)".to_string(),
        Verdict::Differs(why) => panic!("control FAILED — {GOLDEN} differs: {why}"),
    };
    println!("control: end state vs {GOLDEN}: {verdict}");
    println!(
        "rationed: slow {} fast {}; days {}",
        totals.slow_rationed,
        totals.fast_rationed,
        days.len()
    );

    println!(
        "\nworst single slow step: crop drew {:.6} mol of CO2 from a pool of {:.6} mol \
         (ratio {:.4}) on day {:.4}",
        worst.0,
        worst.1,
        worst.0 / worst.1,
        worst.2
    );
    println!(
        "mean draw over the {steps_drawing} slow steps that drew: {:.6} mol",
        gross_sum / steps_drawing as f64
    );

    // Daily figures: peak crop day, and the crew/scrubber budget.
    let peak = days
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.crop_gross.total_cmp(&b.1.crop_gross))
        .expect("a day");
    let n = days.len() as f64;
    let mean_gross: f64 = days.iter().map(|d| d.crop_gross).sum::<f64>() / n;
    let mean_crew: f64 = days.iter().map(|d| d.fast_net + d.scrubbed).sum::<f64>() / n;
    let mean_scrub: f64 = days.iter().map(|d| d.scrubbed).sum::<f64>() / n;
    let pool_min = days
        .iter()
        .map(|d| d.pool_min)
        .fold(f64::INFINITY, f64::min);
    let pool_max = days
        .iter()
        .map(|d| d.pool_max)
        .fold(f64::NEG_INFINITY, f64::max);
    println!(
        "\npeak crop day {}: gross draw {:.6} mol/day (slow net {:.6}, pool {:.4}–{:.4})",
        peak.0, peak.1.crop_gross, peak.1.slow_net, peak.1.pool_min, peak.1.pool_max
    );
    println!("mean over the run: crop gross {mean_gross:.6} mol/day; crew emission {mean_crew:.6} mol/day; scrubbed {mean_scrub:.6} mol/day");
    println!("CO2 pool at slow-step starts over the run: {pool_min:.6} – {pool_max:.6} mol");

    println!(
        "\nwater: worst one-step transpiration {:.6} kg against a small chamber's humidity \
         target {:.6} kg (ratio {:.2}) on day {:.4}; mean ratio over {ratio_n} transpiring \
         steps {:.2}",
        worst_water.0,
        worst_water.1,
        worst_water.0 / worst_water.1,
        worst_water.2,
        ratio_sum / ratio_n as f64
    );
    println!(
        "water: the cabin's air is wetter than a chamber at its humidity target on \
         {cabin_wetter} slow steps, drier on {chamber_wetter}"
    );

    println!("\nday   temp   rh_shared  cabin_kg/mol  target_kg/mol  transp_kg  to_air_kg");
    for (i, d) in days.iter().enumerate().step_by(10) {
        let k = d.slow_steps as f64;
        println!(
            "{i:4}  {:5.2}  {:8.4}  {:12.4e}  {:13.4e}  {:9.5}  {:9.5}",
            d.temp / k,
            d.rh / k,
            d.cabin_conc / k,
            d.target_conc / k,
            d.transpired,
            d.to_air
        );
    }

    // A season profile: every 10th day.
    println!("\nday   crop_gross  slow_net   crew_emit  scrubbed   pool_min  pool_max");
    for (i, d) in days.iter().enumerate().step_by(10) {
        println!(
            "{i:4}  {:9.5}  {:9.5}  {:9.5}  {:9.5}  {:8.4}  {:8.4}",
            d.crop_gross,
            d.slow_net,
            d.fast_net + d.scrubbed,
            d.scrubbed,
            d.pool_min,
            d.pool_max
        );
    }
}
