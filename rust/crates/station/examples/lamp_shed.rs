//! **LAB-ONLY** — the numbers behind `tests/lamp_shed.rs` (Step 3a, option B).
//!
//! WHAT-IF — would the crop feel a power shortage if the lamp were shed at a reserve: 24 h of
//! the life-support load, a round number chosen, not sourced. Every figure printed is about
//! that assumption. Plan: `docs/plans/post-roadmap-light-from-delivered-power.md`.
//!
//! From `rust/`: `cargo run --release -q -p station --example lamp_shed`

use domains::biosphere::stocks::{LEAF_C, ROOT_C, STEM_C, STORAGE_C};
use domains::power::BATTERY;
use simcore::integrator::EulerIntegrator;
use simcore::state::State;
use station::driver::run_master_day;
use station::lamp_shed::{rewire_for_shedding, run_shedding, what_if_reserve};
use station::lighting::LIGHT_USED;
use station::perturbations::with_brownout;
use station::scenario::{sealed_station_scenario, SealedStationScenario};
use station::sealed::{build_sealed_station, sealed_bio_resolver, sealed_fast_resolver};

const DAYS: usize = 8;
const STEPS: u64 = domains::biosphere::STEPS_PER_DAY as u64;

fn crop(state: &State) -> f64 {
    [LEAF_C, STEM_C, ROOT_C, STORAGE_C]
        .iter()
        .map(|s| state.stocks[*s].amount)
        .sum()
}

fn run(scn: &SealedStationScenario, blackout: bool, shed: bool) -> (Vec<State>, u64, Vec<f64>) {
    let (state, bio_reg, fast_reg) = build_sealed_station(
        &domains::params::charge(),
        &domains::params::thermal(),
        &domains::params::crew(),
        &domains::params::eclss(),
        &station::params::water_recovery(),
        &station::params::lamp(),
        &station::params::harvest(),
        scn,
        false,
        false,
    )
    .unwrap();
    let bio = sealed_bio_resolver(&station::params::lamp(), scn).unwrap();
    let mut fast = sealed_fast_resolver(&domains::params::charge(), scn).unwrap();
    if blackout {
        fast = with_brownout(fast, 2 * STEPS, 5 * STEPS, 0.0).unwrap();
    }
    if !shed {
        let (states, rationed, _) = run_master_day(
            &EulerIntegrator::new(bio_reg),
            &EulerIntegrator::new(fast_reg),
            state,
            &bio,
            &fast,
            DAYS,
            scn.steps_per_day,
            scn.bio_steps_per_day,
            scn.bio_dt,
            scn.cabin_dt,
            None,
        )
        .unwrap();
        return (states, rationed, Vec::new());
    }
    let reserve = what_if_reserve(&domains::params::charge(), scn);
    let (state, bio_reg, fast_reg) =
        rewire_for_shedding(state, bio_reg, fast_reg, reserve).unwrap();
    let (states, rationed, _, log) = run_shedding(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(fast_reg),
        state,
        &bio,
        &fast,
        scn,
        DAYS,
    )
    .unwrap();
    (states, rationed, log.delivery)
}

fn main() {
    let scn = SealedStationScenario {
        years: 1,
        season_days: 305,
        battery0: 1.5e8,
        ..sealed_station_scenario()
    };
    let reserve = what_if_reserve(&domains::params::charge(), &scn);
    println!("WHAT-IF reserve: 24 h of life support = {reserve:.4e} J; battery0 = 1.5e8 J");
    println!("blackout: solar x0 over days 2-5; horizon {DAYS} days\n");
    let calm_plain = run(&scn, false, false);
    println!(
        "{:<26} {:>12} {:>12} {:>12} {:>9} {:>14}",
        "run", "crop mol C", "vs calm", "battery J", "rationed", "lamp light J"
    );
    for (name, blackout, shed) in [
        ("plain, calm", false, false),
        ("plain, blackout", true, false),
        ("lab shed, calm", false, true),
        ("lab shed, blackout", true, true),
    ] {
        let (states, rationed, delivery) = run(&scn, blackout, shed);
        let last = states.last().unwrap();
        let rel = crop(last) / crop(calm_plain.0.last().unwrap()) - 1.0;
        println!(
            "{:<26} {:>12.6} {:>+11.3}% {:>12.4e} {:>9} {:>14.4e}",
            name,
            crop(last),
            100.0 * rel,
            last.stocks[BATTERY].amount,
            rationed,
            last.stocks[LIGHT_USED].amount
        );
        if let Some(first) = delivery.iter().position(|&x| x < 1.0) {
            let groups = delivery.len() / DAYS;
            println!(
                "    first shed in group {first} (day {:.3}); share then {:.4}; groups dark to the end: {}",
                first as f64 / groups as f64,
                delivery[first],
                delivery[first + 1..].iter().all(|&x| x == 0.0)
            );
        }
        let min_battery = states
            .iter()
            .map(|s| s.stocks[BATTERY].amount)
            .fold(f64::INFINITY, f64::min);
        println!("    lowest end-of-day battery {min_battery:.4e} J");
    }
}
