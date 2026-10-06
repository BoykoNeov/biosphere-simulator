//! Step 3c slice 4 stage 2's lab twin (`docs/plans/post-roadmap-room-temperature.md` §25f/§25g):
//! the sealed station over its full horizon with the crop re-sown on maturity, against the
//! calendar re-sow's `sealed_station` golden — read from git at [`CALENDAR_COMMIT`] (slice 4
//! stage 1, whose stocks are the calendar run's to the bit), because since stage 2 the working
//! tree's golden IS this run. Needs `git` and the repository's history. Prints the readouts the §25f
//! predictions name: the re-sow days and the development stage at each, the grain each crop
//! re-sows from, the cold days, and the end state (crop, battery, node, gas regulators).
//!
//! Run from `rust/`: `cargo run --release -q -p station --example resow_on_maturity`.

use domains::biosphere::science::development_stage;
use domains::biosphere::stocks::{STORAGE_C, THERMAL_TIME, VERNALIZATION_DAYS};
use domains::eclss::{CO2_REMOVED, O2_SUPPLY};
use domains::power::BATTERY;
use domains::thermal::NODE;
use simcore::integrator::EulerIntegrator;
use simcore::state::State;
use station::scenario::{sealed_station_scenario, Phase};
use station::sealed::{build_sealed_station, run_sealed, sealed_bio_resolver, sealed_fast_resolver};
use station::sowing::sown_step;

/// The last commit whose `sealed_station` golden is the CALENDAR re-sow's (slice 4 stage 1: its
/// one added aux key aside, byte-identical to the calendar reference).
const CALENDAR_COMMIT: &str = "9941eba";

fn main() {
    let scenario = sealed_station_scenario();
    let charge = domains::params::charge();
    let thermal = domains::params::thermal();
    let lamp = station::params::lamp();
    let pheno = scenario.pheno;
    let (state, bio_reg, fast_reg) = build_sealed_station(
        &charge,
        &thermal,
        &domains::params::crew(),
        &domains::params::eclss(),
        &station::params::water_recovery(),
        &lamp,
        &station::params::harvest(),
        &scenario,
        false,
        false,
    )
    .unwrap();
    let (states, rationed, events) = run_sealed(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(fast_reg),
        state,
        &sealed_bio_resolver(&lamp, &scenario).unwrap(),
        &sealed_fast_resolver(&charge, &scenario).unwrap(),
        &scenario,
    )
    .unwrap();
    let per_day = scenario.bio_steps_per_day;
    let dvs = |s: &State| development_stage(s.aux[THERMAL_TIME], pheno.tsum_anthesis, pheno.tsum_maturity);
    println!("days {}  rationed {rationed}  events {}", states.len() - 1, events.len());

    // Re-sows: `states[d]` is the start of day d; a re-sow at day d's start shows in states[d+1].
    println!("re-sows (start of day; DVS on that day-start and the one before; grain re-sown from):");
    let mut cold_days = 0;
    for d in 0..states.len() - 1 {
        let (before, after) = (&states[d], &states[d + 1]);
        let sown = sown_step(after).unwrap();
        if sown != sown_step(before).unwrap() {
            assert_eq!(sown, d as u64 * per_day, "a re-sow off a day start");
            println!(
                "  day {d:>4}  DVS {:.4} (day before {:.4})  grain {:.4} mol C",
                dvs(before),
                if d > 0 { dvs(&states[d - 1]) } else { f64::NAN },
                before.stocks[STORAGE_C].amount
            );
        }
        let since = d as u64 * per_day - sown;
        if scenario.phase_since_sowing(since) == Phase::Cold {
            cold_days += 1;
        }
    }
    println!("cold days: {cold_days}");

    let shown = std::process::Command::new("git")
        .args(["-C", env!("CARGO_MANIFEST_DIR"), "show"])
        .arg(format!("{CALENDAR_COMMIT}:rust/data/golden/sealed_station_state.json"))
        .output()
        .expect("git");
    assert!(shown.status.success(), "git show {CALENDAR_COMMIT}: the history is needed");
    let old = simcore::snapshot::from_json(&String::from_utf8(shown.stdout).unwrap()).unwrap();
    let new = states.last().unwrap();
    let node_k = |s: &State| s.stocks[NODE].amount / thermal.heat_capacity + thermal.space_temperature;
    println!("end state           golden (calendar)        re-sow on maturity");
    for (name, f) in [
        ("storage_c", Box::new(|s: &State| s.stocks[STORAGE_C].amount) as Box<dyn Fn(&State) -> f64>),
        ("thermal_time", Box::new(|s: &State| s.aux[THERMAL_TIME])),
        ("vernalization_days", Box::new(|s: &State| s.aux[VERNALIZATION_DAYS])),
        ("battery J", Box::new(|s: &State| s.stocks[BATTERY].amount)),
        ("node K", Box::new(node_k)),
        ("co2_removed mol", Box::new(|s: &State| s.stocks[CO2_REMOVED].amount)),
        ("o2_supply mol", Box::new(|s: &State| s.stocks[O2_SUPPLY].amount)),
    ] {
        println!("  {name:<18} {:>22.10e} {:>22.10e}", f(&old), f(new));
    }
    println!("  DVS at the end      {:.4} (days since sowing {})", dvs(new), (new.n - sown_step(new).unwrap()) / per_day);
}
