//! **The water the larger crop moves** — the readout `docs/plans/post-roadmap-co2-level.md` §9
//! left owed before the 0.13 kPa level can be adopted: the sealed station's transpiration total
//! over its four years, and a day series of the soil and condensate stores to test the
//! "watering pulses at a different phase" reading of the final day.
//!
//! ```text
//! cargo run --release -q -p station --example co2_water_lab
//! cargo run --release -q -p station --example co2_water_lab -- --series <path.csv>
//! ```
//!
//! The sealed station runs twice, as frozen and with the scrubber switched to remove only the
//! CO₂ above 0.13 kPa (`domains::lab::with_scrubber_above_target`, the cabin started at that
//! target, as `co2_target_lab` does). It runs through the lab's two-rate driver in the
//! reference's interleaved order, which is the reference day operation for operation; the
//! frozen side's final state is checked against the committed golden's bytes before anything
//! is read. Every flow of both sides is evaluated on each step's start state, and what it takes
//! out of each of the chamber's four water stores is summed by flow, by year.
//!
//! ⚠ **It writes nothing and takes no decision.** `--series` writes the day series to the path
//! given, and only there.

use std::collections::BTreeMap;

use domains::biosphere::stocks::{CONDENSATE, SOIL_EVAP_TODAY, SOIL_WATER, SUBSOIL_WATER, WATER_VAPOR};
use domains::eclss::EclssParams;
use domains::lab::{with_scrubber_above_target, ABOVE_TARGET_SCRUB_RATE, CO2_TARGET_PA};
use domains::params;
use simcore::integrator::EulerIntegrator;
use simcore::state::State;
use station::driver::{DayOrder, Side, TwoRate};
use station::params as station_params;
use station::scenario::{sealed_station_scenario, SEALED_STATION_SEASON_DAYS, STATION_CABIN_AIR_MOL};
use station::sealed::{
    build_sealed_station, sealed_bio_resolver, sealed_fast_resolver, sealed_reset_hook,
};

/// The chamber's water stores, as the flows' legs name them.
const WATER: [&str; 4] = [SOIL_WATER, SUBSOIL_WATER, WATER_VAPOR, CONDENSATE];
const RECYCLING: &str = "biosphere.recycling";

/// `(flow id, store)` → kg, per year: drawn from the store (`out`) or delivered to it (`into`).
type Census = BTreeMap<(String, &'static str), Vec<f64>>;

struct Run {
    states: Vec<State>,
    out: Census,
    into: Census,
    /// The soil's evaporation inside `biosphere.transpiration` (kg per year), read off the
    /// flow's own daily counter (`SOIL_EVAP_TODAY`): the flow draws crop and soil as one leg.
    soil_evap: Vec<f64>,
    /// Plant steps on which the condensate-to-soil watering fired, per day.
    watering_steps: Vec<u32>,
    /// The re-sow hook's signed change to each store, summed over the run (kg).
    reset_moved: BTreeMap<&'static str, f64>,
    rationed: (u64, u64),
    events: usize,
}

fn run(eclss: &EclssParams, start: Option<f64>) -> Run {
    let charge = params::charge();
    let lamp = station_params::lamp();
    let mut scenario = sealed_station_scenario();
    if let Some(n) = start {
        scenario.bio.chamber_co2_mol0 = n;
    }
    let (state, bio_reg, fast_reg) = build_sealed_station(
        &charge,
        &params::thermal(),
        &params::crew(),
        eclss,
        &station_params::water_recovery(),
        &lamp,
        &station_params::harvest(),
        &scenario,
        false,
        false,
    )
    .expect("build_sealed_station");
    let bio_r = sealed_bio_resolver(&lamp, &scenario).expect("bio resolver");
    let fast_r = sealed_fast_resolver(&charge, &scenario).expect("fast resolver");
    let reset = sealed_reset_hook(&scenario);
    let (bio, fast) = (EulerIntegrator::new(bio_reg), EulerIntegrator::new(fast_reg));
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
    let days = scenario.days();
    let years = days.div_ceil(SEALED_STATION_SEASON_DAYS);
    let (mut out, mut into) = (Census::new(), Census::new());
    let mut soil_evap = vec![0.0; years];
    let mut watering_steps = vec![0u32; days];
    let mut reset_moved: BTreeMap<&'static str, f64> = WATER.iter().map(|w| (*w, 0.0)).collect();
    let mut day = 0usize;
    let (slow_per_day, mut slow_seen) = (scenario.bio_steps_per_day as usize, 0usize);
    let evap_today = |s: &State| s.aux.get(SOIL_EVAP_TODAY).copied().unwrap_or(0.0);
    let mut observe = |side: Side, before: &State, after: &State| {
        let (registry, env, dt) = match side {
            Side::Reset => {
                for id in WATER {
                    *reset_moved.get_mut(id).expect("a store") +=
                        after.stocks[id].amount - before.stocks[id].amount;
                }
                return;
            }
            Side::Slow => {
                day = slow_seen / slow_per_day;
                slow_seen += 1;
                // The counter restarts on the day's first plant step (`SoilAccount`).
                soil_evap[day / SEALED_STATION_SEASON_DAYS] +=
                    if before.n.is_multiple_of(scenario.bio_steps_per_day) {
                        evap_today(after)
                    } else {
                        evap_today(after) - evap_today(before)
                    };
                (bio.registry(), bio_r.bind(before, scenario.bio_dt), scenario.bio_dt)
            }
            Side::Fast => (fast.registry(), fast_r.bind(before, scenario.cabin_dt), scenario.cabin_dt),
        };
        let year = day / SEALED_STATION_SEASON_DAYS;
        for flow in registry.flows() {
            let res = flow.evaluate(before, &env, dt).expect("flow evaluates");
            for leg in &res.legs {
                let Some(store) = WATER.iter().find(|w| **w == leg.stock) else {
                    continue;
                };
                let book = if leg.amount < 0.0 { &mut out } else { &mut into };
                book.entry((flow.id().to_string(), *store)).or_insert_with(|| vec![0.0; years])[year] +=
                    leg.amount.abs();
                if leg.amount < 0.0 && flow.id() == RECYCLING && matches!(side, Side::Slow) {
                    watering_steps[day] += 1;
                }
            }
        }
    };
    let (states, totals) = two
        .run(DayOrder::Interleaved, state, days, &mut observe)
        .expect("two-rate run");
    Run {
        states,
        out,
        into,
        soil_evap,
        watering_steps,
        reset_moved,
        rationed: (totals.slow_rationed, totals.fast_rationed),
        events: totals.events.len(),
    }
}

fn store_name(id: &str) -> &str {
    id.rsplit('.').next().unwrap_or(id)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let series = args.iter().position(|a| a == "--series").and_then(|i| args.get(i + 1));
    let frozen = params::eclss();
    let lab = with_scrubber_above_target(frozen, CO2_TARGET_PA, ABOVE_TARGET_SCRUB_RATE);
    let target = lab.co2_target_mol(STATION_CABIN_AIR_MOL).expect("a target");

    let base = run(&frozen, None);
    let golden = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/golden/sealed_station_state.json");
    let committed = std::fs::read_to_string(golden).expect("golden file");
    let last = base.states.last().expect("a day");
    // A tool that does not run the golden's build measures nothing: stop before any reading.
    assert!(
        simcore::snapshot::from_engine(last).to_json() == committed,
        "the frozen side does not reproduce sealed_station_state.json byte for byte"
    );
    let switched = run(&lab, Some(target));

    println!("sealed station, 4 years: frozen scrubber vs above-target {CO2_TARGET_PA} Pa (k = {ABOVE_TARGET_SCRUB_RATE:e} /s)");
    println!("frozen side reproduces the golden bytes: true (asserted)");
    for (name, r) in [("frozen", &base), ("switched", &switched)] {
        println!(
            "{name}: rationed (slow, fast) {:?}  events {}  re-sow hook's water moves {:?}",
            r.rationed, r.events, r.reset_moved
        );
    }

    // Closure: the census, re-evaluated on each step's start state, against what the run did.
    println!("\n== closure per store: in − out + re-sow vs final − initial (kg)");
    for (name, r) in [("frozen", &base), ("switched", &switched)] {
        for id in WATER {
            let sum = |c: &Census| c.iter().filter(|(k, _)| k.1 == id).map(|(_, v)| v.iter().sum::<f64>()).sum::<f64>();
            let booked = sum(&r.into) - sum(&r.out) + r.reset_moved[id];
            let actual = r.states.last().expect("a day").stocks[id].amount - r.states[0].stocks[id].amount;
            println!(
                "{name:<9} {:<14} booked {booked:>14.6}  actual {actual:>14.6}  leftover {:+.3e}",
                store_name(id),
                booked - actual
            );
        }
    }

    for (title, pick) in [("drawn from", 0), ("delivered to", 1)] {
        println!("\n== water {title} each chamber store, by flow (kg per year; frozen -> switched)");
        let (bc, sc) = if pick == 0 { (&base.out, &switched.out) } else { (&base.into, &switched.into) };
        let keys: std::collections::BTreeSet<_> = bc.keys().chain(sc.keys()).cloned().collect();
        for k in keys {
            let zero = vec![0.0; base.soil_evap.len()];
            let b = bc.get(&k).unwrap_or(&zero);
            let s = sc.get(&k).unwrap_or(&zero);
            let (bt, st): (f64, f64) = (b.iter().sum(), s.iter().sum());
            if bt == 0.0 && st == 0.0 {
                continue;
            }
            println!("{:<34} {title} {:<14} total {bt:>10.2} -> {st:>10.2} ({:+.2} %)", k.0, store_name(k.1), 100.0 * (st / bt - 1.0));
            for (y, (bv, sv)) in b.iter().zip(s).enumerate() {
                println!("{:<60} year {}: {bv:>10.2} -> {sv:>10.2} ({:+.2} %)", "", y + 1, 100.0 * (sv / bv - 1.0));
            }
        }
    }

    println!("\n== biosphere.transpiration split: the soil's evaporation (its daily counter) and the crop's rest (kg)");
    let transpired = |r: &Run| r.out[&("biosphere.transpiration".to_string(), SOIL_WATER)].clone();
    let (bt, st) = (transpired(&base), transpired(&switched));
    for y in 0..bt.len() {
        let (be, se) = (base.soil_evap[y], switched.soil_evap[y]);
        println!(
            "   year {}: soil {be:>9.2} -> {se:>9.2} ({:+.2} %)   crop {:>9.2} -> {:>9.2} ({:+.2} %)",
            y + 1,
            100.0 * (se / be - 1.0),
            bt[y] - be,
            st[y] - se,
            100.0 * ((st[y] - se) / (bt[y] - be) - 1.0)
        );
    }
    let (be, se): (f64, f64) = (base.soil_evap.iter().sum(), switched.soil_evap.iter().sum());
    let (bta, sta): (f64, f64) = (bt.iter().sum(), st.iter().sum());
    println!(
        "   4 years: soil {be:.2} -> {se:.2} ({:+.2} %)   crop {:.2} -> {:.2} ({:+.2} %)",
        100.0 * (se / be - 1.0),
        bta - be,
        sta - se,
        100.0 * ((sta - se) / (bta - be) - 1.0)
    );

    println!("\n== watering (condensate -> soil) events and the stores' day series");
    let days = base.watering_steps.len();
    for (name, r) in [("frozen", &base), ("switched", &switched)] {
        let fired: Vec<usize> = (0..days).filter(|d| r.watering_steps[*d] > 0).collect();
        let gaps: Vec<usize> = fired.windows(2).map(|w| w[1] - w[0]).collect();
        let mean_gap = gaps.iter().sum::<usize>() as f64 / gaps.len().max(1) as f64;
        let since = fired.last().map_or(days, |d| days - 1 - d);
        println!(
            "{name}: watering on {} days, mean gap {mean_gap:.2} d, min {} max {}; last on day {:?} ({} d before the final day)",
            fired.len(),
            gaps.iter().min().unwrap_or(&0),
            gaps.iter().max().unwrap_or(&0),
            fired.last(),
            since
        );
        for (y, chunk) in r.states[1..].chunks(SEALED_STATION_SEASON_DAYS).enumerate() {
            let mean = |id: &str| chunk.iter().map(|s| s.stocks[id].amount).sum::<f64>() / chunk.len() as f64;
            let total = |s: &State| WATER.iter().map(|id| s.stocks[*id].amount).sum::<f64>();
            let (lo, hi) = chunk.iter().map(total).fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), v| (a.min(v), b.max(v)));
            println!(
                "   year {}: mean soil {:.2}, subsoil {:.2}, condensate {:.2}, vapour {:.4}; chamber water min/max {lo:.6}/{hi:.6}",
                y + 1,
                mean(SOIL_WATER),
                mean(SUBSOIL_WATER),
                mean(CONDENSATE),
                mean(WATER_VAPOR)
            );
        }
    }

    if let Some(path) = series {
        let mut out = String::from("day,frozen_soil,frozen_subsoil,frozen_condensate,frozen_watering_steps,switched_soil,switched_subsoil,switched_condensate,switched_watering_steps\n");
        for d in 0..days {
            let (b, s) = (&base.states[d + 1], &switched.states[d + 1]);
            out.push_str(&format!(
                "{d},{},{},{},{},{},{},{},{}\n",
                b.stocks[SOIL_WATER].amount,
                b.stocks[SUBSOIL_WATER].amount,
                b.stocks[CONDENSATE].amount,
                base.watering_steps[d],
                s.stocks[SOIL_WATER].amount,
                s.stocks[SUBSOIL_WATER].amount,
                s.stocks[CONDENSATE].amount,
                switched.watering_steps[d]
            ));
        }
        std::fs::write(path, out).expect("write the series");
        println!("\nday series written to {path}");
    }
}
