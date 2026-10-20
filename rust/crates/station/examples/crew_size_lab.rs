//! **The station's crew at `N` BVAD crew members, against every golden that reads it** — the lab
//! build of `docs/plans/post-roadmap-crew-size.md` §8, measured against its written predictions
//! (§6 for one crew member, §9 for four, the user's choice).
//!
//! ```text
//! cargo run --release -q -p station --example crew_size_lab              # all five, N = 4
//! cargo run --release -q -p station --example crew_size_lab -- greenhouse # by name
//! CREW_N=1 cargo run --release -q -p station --example crew_size_lab     # another count
//! ```
//!
//! Each golden's own build runs twice: as frozen, and with the cabin scenario's crew switched to
//! `N` crew members of BVAD Table 3-31 — the food rate `N × CO₂ load ÷ f_resp` from the LOADED
//! `crew.yaml` (§4: no typed copy of `f_resp`), the water `N × (humidity + urine)`. The two
//! crew-only cabins start at the scrubber's target; the 7-day crop cabins' crew water store is
//! 200 kg (§9). Every stock of the final state is compared, boundary ones included.
//!
//! ⚠ **It writes nothing and takes no decision.** The frozen side is checked against the
//! committed golden's bytes first, so a drift in this tool's copy of a build cannot pass as a
//! measurement. ⚠ The BVAD numbers below are a lab copy of `science_gates::bvad` (test-only
//! there); adoption moves them to one module both read (§4).

use std::collections::BTreeMap;

use domains::crew::CrewParams;
use domains::eclss::EclssParams;
use domains::params;
use simcore::integrator::EulerIntegrator;
use simcore::state::State;
use station::cabin::{build_cabin, cabin_resolver};
use station::greenhouse::{
    build_greenhouse, greenhouse_bio_resolver, greenhouse_cabin_resolver, run_greenhouse,
};
use station::harvest::{build_harvest, harvest_bio_resolver, harvest_cabin_resolver, run_harvest};
use station::params as station_params;
use station::run_station;
use station::scenario::{
    greenhouse_scenario, harvest_scenario, sealed_station_scenario, CabinScenario,
    CABIN_GAS_SCENARIO, CABIN_GAS_STEPS, SEALED_STATION_SEASON_DAYS, STATION_CABIN_AIR_MOL,
    WATER_RECOVERY_SCENARIO, WATER_RECOVERY_STEPS,
};
use station::sealed::{build_sealed_station, run_sealed, sealed_bio_resolver, sealed_fast_resolver};
use station::water::{build_water_recovery, water_recovery_resolver};

/// BVAD Table 3-31, per crew member per day (`docs/bvad-reference.md`).
const CO2_LOAD_KG: f64 = 1.085;
const RESP_PERSP_WATER_KG: f64 = 2.946;
const URINE_WATER_KG: f64 = 1.420;
const M_CO2: f64 = 44.009;
const SECONDS_PER_DAY: f64 = 86_400.0;

/// The 7-day crop cabins' crew water store under the switch (plan §9).
const CROP_CABIN_WATER_STORE_KG: f64 = 200.0;

/// The switched crew: `None` is the frozen scenario.
#[derive(Clone, Copy)]
struct Crew {
    food_intake_rate: f64,
    water_intake_rate: f64,
    co2_target_mol: f64,
}

impl Crew {
    fn bvad(n: f64, crew: &CrewParams, eclss: &EclssParams) -> Crew {
        Crew {
            food_intake_rate: n * CO2_LOAD_KG * 1000.0 / M_CO2 / crew.respired_carbon_fraction
                / SECONDS_PER_DAY,
            water_intake_rate: n * (RESP_PERSP_WATER_KG + URINE_WATER_KG) / SECONDS_PER_DAY,
            co2_target_mol: eclss.co2_target_mol(STATION_CABIN_AIR_MOL).expect("a target"),
        }
    }

    /// The switched cabin: rates replaced; `crew_only` cabins start at the target, the crop
    /// cabins' water store is resized when `crop_store` is given.
    fn cabin(&self, base: CabinScenario, crew_only: bool, crop_store: Option<f64>) -> CabinScenario {
        CabinScenario {
            food_intake_rate: self.food_intake_rate,
            water_intake_rate: self.water_intake_rate,
            cabin_co2_0: if crew_only { self.co2_target_mol } else { base.cabin_co2_0 },
            water_store0: crop_store.unwrap_or(base.water_store0),
            ..base
        }
    }
}

/// One side of a run: the final state, every day's state when the build keeps them, and
/// whether the run stayed clean.
struct Run {
    last: State,
    days: Vec<State>,
    rationed: u64,
    events: usize,
}

type Builder = fn(Option<Crew>) -> Run;

fn snapshot(state: &State) -> String {
    simcore::snapshot::from_engine(state).to_json()
}

fn cabin_gas_run(sw: Option<Crew>) -> Run {
    let crew = params::crew();
    let eclss = params::eclss();
    let scenario = sw.map_or(CABIN_GAS_SCENARIO, |c| c.cabin(CABIN_GAS_SCENARIO, true, None));
    let (state, registry) = build_cabin(&crew, &eclss, &scenario).expect("build_cabin");
    let resolver = cabin_resolver(&scenario).expect("cabin_resolver");
    let (last, rationed, events) = run_station(
        &EulerIntegrator::new(registry),
        state,
        &resolver,
        scenario.dt_seconds,
        CABIN_GAS_STEPS,
        &mut |_: &State| {},
    )
    .expect("run cabin");
    Run { last, days: Vec::new(), rationed, events: events.len() }
}

fn water_recovery_run(sw: Option<Crew>) -> Run {
    let crew = params::crew();
    let eclss = params::eclss();
    let recovery = station_params::water_recovery();
    let scenario =
        sw.map_or(WATER_RECOVERY_SCENARIO, |c| c.cabin(WATER_RECOVERY_SCENARIO, true, None));
    let (state, registry) =
        build_water_recovery(&crew, &eclss, &recovery, &scenario).expect("build_water_recovery");
    let resolver = water_recovery_resolver(&scenario).expect("water_recovery_resolver");
    let (last, rationed, events) = run_station(
        &EulerIntegrator::new(registry),
        state,
        &resolver,
        scenario.dt_seconds,
        WATER_RECOVERY_STEPS,
        &mut |_: &State| {},
    )
    .expect("run water_recovery");
    Run { last, days: Vec::new(), rationed, events: events.len() }
}

fn greenhouse_run(sw: Option<Crew>) -> Run {
    let crew = params::crew();
    let eclss = params::eclss();
    let mut scenario = greenhouse_scenario();
    if let Some(c) = sw {
        scenario.cabin = c.cabin(scenario.cabin, false, Some(CROP_CABIN_WATER_STORE_KG));
    }
    let (state, bio_reg, cabin_reg) =
        build_greenhouse(&crew, &eclss, &scenario, true, domains::crew::FECAL_WASTE)
            .expect("build_greenhouse");
    let bio = greenhouse_bio_resolver(&scenario).expect("bio_resolver");
    let cabin = greenhouse_cabin_resolver(&scenario).expect("cabin_resolver");
    let (states, rationed, events) = run_greenhouse(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(cabin_reg),
        state,
        &bio,
        &cabin,
        &scenario,
    )
    .expect("run greenhouse");
    let last = states.last().expect("a day").clone();
    Run { last, days: states, rationed, events: events.len() }
}

fn harvest_run(sw: Option<Crew>) -> Run {
    let crew = params::crew();
    let eclss = params::eclss();
    let harvest = station_params::harvest();
    let mut scenario = harvest_scenario();
    if let Some(c) = sw {
        scenario.greenhouse.cabin =
            c.cabin(scenario.greenhouse.cabin, false, Some(CROP_CABIN_WATER_STORE_KG));
    }
    let (state, bio_reg, cabin_reg) =
        build_harvest(&crew, &eclss, &harvest, &scenario, true, true).expect("build_harvest");
    let bio = harvest_bio_resolver(&scenario).expect("bio_resolver");
    let cabin = harvest_cabin_resolver(&scenario).expect("cabin_resolver");
    let (states, rationed, events) = run_harvest(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(cabin_reg),
        state,
        &bio,
        &cabin,
        &scenario,
    )
    .expect("run harvest");
    let last = states.last().expect("a day").clone();
    Run { last, days: states, rationed, events: events.len() }
}

fn sealed_station_run(sw: Option<Crew>) -> Run {
    let charge = params::charge();
    let thermal = params::thermal();
    let crew = params::crew();
    let eclss = params::eclss();
    let lamp = station_params::lamp();
    let mut scenario = sealed_station_scenario();
    if let Some(c) = sw {
        scenario.cabin = c.cabin(scenario.cabin, false, None);
    }
    let (state, bio_reg, fast_reg) = build_sealed_station(
        &charge,
        &thermal,
        &crew,
        &eclss,
        &station_params::water_recovery(),
        &lamp,
        &station_params::harvest(),
        &scenario,
        false,
        false,
    )
    .expect("build_sealed_station");
    let bio = sealed_bio_resolver(&lamp, &scenario).expect("sealed_bio_resolver");
    let fast = sealed_fast_resolver(&charge, &scenario).expect("sealed_fast_resolver");
    let (states, rationed, events) = run_sealed(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(fast_reg),
        state,
        &bio,
        &fast,
        &scenario,
    )
    .expect("run sealed station");
    let last = states.last().expect("a day").clone();
    Run { last, days: states, rationed, events: events.len() }
}

fn roster() -> Vec<(&'static str, Builder)> {
    vec![
        ("cabin_gas_state.json", cabin_gas_run as Builder),
        ("water_recovery_state.json", water_recovery_run),
        ("greenhouse_state.json", greenhouse_run),
        ("harvest_state.json", harvest_run),
        ("sealed_station_state.json", sealed_station_run),
    ]
}

fn amounts(s: &State) -> BTreeMap<String, f64> {
    s.stocks.iter().map(|(k, v)| (k.clone(), v.amount)).collect()
}

fn get(s: &State, id: &str) -> f64 {
    s.stocks.get(id).map_or(0.0, |x| x.amount)
}

/// The crop-side readings the golden's final state does not hold (plan §6): each season's
/// grain peak, the days whose end finds the cabin below the scrubber's target, and the
/// three-way soil-water total at the end.
fn crop_readings(name: &str, base: &Run, switched: &Run, target: f64) {
    if base.days.is_empty() {
        return;
    }
    let below = |r: &Run| r.days.iter().filter(|s| get(s, "biosphere.carbon_pool") < target).count();
    println!(
        "   days ending below the target ({target:.6} mol): frozen {} / switched {} of {}",
        below(base),
        below(switched),
        switched.days.len()
    );
    let water = |s: &State| {
        get(s, "biosphere.soil_water") + get(s, "biosphere.subsoil_water")
            + get(s, "biosphere.condensate")
    };
    let (wb, ws) = (water(&base.last), water(&switched.last));
    println!(
        "   soil + subsoil + condensate water: {wb:.6} -> {ws:.6} kg ({:+.4} %)",
        100.0 * (ws / wb - 1.0)
    );
    if name.starts_with("sealed_station") {
        // Per 305-day year, as `co2_level_price` read it: a year holds about two crops (the
        // re-sow on maturity comes every ~140 days), so this is the larger of their peaks.
        let peaks = |r: &Run| -> Vec<f64> {
            r.days
                .as_chunks::<SEALED_STATION_SEASON_DAYS>()
                .0
                .iter()
                .map(|c| c.iter().map(|s| get(s, "biosphere.storage_c")).fold(0.0, f64::max))
                .collect()
        };
        for (y, (b, s)) in peaks(base).iter().zip(peaks(switched)).enumerate() {
            println!(
                "   year {} grain peak: {b:.6} -> {s:.6} mol C ({:+.4} %)",
                y + 1,
                100.0 * (s / b - 1.0)
            );
        }
        // The re-sow on maturity restarts the lamp's program, so a shifted maturity day moves
        // the battery: the days whose end finds the grain harvested after a day holding some.
        let resows = |r: &Run| -> Vec<usize> {
            r.days
                .windows(2)
                .enumerate()
                .filter(|(_, w)| {
                    get(&w[0], "biosphere.storage_c") > 0.0 && get(&w[1], "biosphere.storage_c") == 0.0
                })
                .map(|(i, _)| i + 1)
                .collect()
        };
        println!("   re-sow days: frozen {:?} / switched {:?}", resows(base), resows(switched));
        // The final day's crop is `age` days from its sowing; the leaf at that same age after
        // every earlier sowing says whether the final leaf's small move is the crop's stage.
        let last_day = base.days.len() - 1;
        let sowings = resows(base);
        if let Some(&final_sow) = sowings.last() {
            let age = last_day - final_sow;
            for &d in &sowings {
                let day = d + age;
                if day < base.days.len() && day < switched.days.len() {
                    let (b, s) = (
                        get(&base.days[day], "biosphere.leaf_c"),
                        get(&switched.days[day], "biosphere.leaf_c"),
                    );
                    println!(
                        "   leaf at age {age} d (day {day}): {b:.6} -> {s:.6} ({:+.4} %)",
                        100.0 * (s / b - 1.0)
                    );
                }
            }
        }
    }
}

fn main() {
    let only: Vec<String> = std::env::args().skip(1).collect();
    let n: f64 = std::env::var("CREW_N").ok().map_or(4.0, |v| v.parse().expect("CREW_N a number"));
    let crew = params::crew();
    let eclss = params::eclss();
    let sw = Crew::bvad(n, &crew, &eclss);
    let golden_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/golden");
    println!(
        "frozen: food {:e} mol C/s, water {:e} kg/s; switched: {n} BVAD crew members, food {:e} \
         mol C/s (f_resp {} loaded), water {:e} kg/s; crew-only cabins start at {:.6} mol; crop \
         cabins' water store {CROP_CABIN_WATER_STORE_KG} kg\n",
        CABIN_GAS_SCENARIO.food_intake_rate,
        CABIN_GAS_SCENARIO.water_intake_rate,
        sw.food_intake_rate,
        crew.respired_carbon_fraction,
        sw.water_intake_rate,
        sw.co2_target_mol
    );
    for (name, run) in roster() {
        if !only.is_empty() && !only.iter().any(|o| name.contains(o.as_str())) {
            continue;
        }
        let base = run(None);
        let committed = std::fs::read_to_string(golden_dir.join(name)).expect("golden file");
        let reproduces = snapshot(&base.last) == committed;
        let switched = run(Some(sw));
        let (b, s) = (amounts(&base.last), amounts(&switched.last));
        let mut moved: Vec<(String, f64, f64, f64)> = b
            .iter()
            .filter(|(k, v)| s[*k].to_bits() != v.to_bits())
            .map(|(k, &v)| {
                let x = s[k];
                let rel = if v == 0.0 { f64::INFINITY } else { x / v - 1.0 };
                (k.clone(), v, x, rel)
            })
            .collect();
        moved.sort_by(|x, y| y.3.abs().total_cmp(&x.3.abs()));
        println!(
            "== {name}: frozen side reproduces the golden bytes: {reproduces}; \
             switched rationed {} events {} (frozen {} / {})",
            switched.rationed, switched.events, base.rationed, base.events
        );
        println!("   {} of {} stocks bit-identical, {} moved", b.len() - moved.len(), b.len(), moved.len());
        for (k, v, x, rel) in &moved {
            println!("   {k:<44} {v:>18.6} -> {x:>18.6}  ({:+.4} %)", 100.0 * rel);
        }
        crop_readings(name, &base, &switched, sw.co2_target_mol);
        println!();
    }
}
