//! **LAB-ONLY** — the lamp shed at a reserve, and a crop that feels it (Step 3a, option B).
//!
//! Plan: `docs/plans/post-roadmap-light-from-delivered-power.md`. WHAT-IF — would the crop
//! feel a power shortage if the lamp were shed at a reserve: 24 h of the life-support load,
//! a round number chosen, not sourced. The results here are about that assumption, not about
//! real stations.
//!
//! The controls come first: the lab wiring with nothing shed must be the plain sealed run,
//! bit for bit, or every difference below would be the wiring's.

use std::sync::OnceLock;

use domains::biosphere::stocks::{LEAF_C, ROOT_C, STEM_C, STORAGE_C};
use domains::power::BATTERY;
use simcore::environment::SourceResolver;
use simcore::integrator::EulerIntegrator;
use simcore::registry::Registry;
use simcore::state::State;
use station::driver::run_master_day;
use station::lamp_shed::{
    rewire_for_shedding, run_shedding, what_if_reserve, with_lamp_power_cut, ShedLog,
    LAMP_DELIVERY_AUX,
};
use station::lighting::LIGHT_USED;
use station::perturbations::with_brownout;
use station::scenario::{sealed_station_scenario, SealedStationScenario};
use station::sealed::{build_sealed_station, sealed_bio_resolver, sealed_fast_resolver};

const DAYS: usize = 8;
const STEPS: u64 = domains::biosphere::STEPS_PER_DAY as u64;
/// A battery small enough for a three-day blackout to reach the reserve, large enough that
/// the lamp alone does not reach it in eight days (it drains 1.152e7 J/day, unpaid — §6).
const SMALL_BATTERY: f64 = 1.5e8;
const BLACKOUT: (u64, u64) = (2 * STEPS, 5 * STEPS);

fn scenario(battery0: f64) -> SealedStationScenario {
    SealedStationScenario {
        years: 1,
        season_days: 305,
        battery0,
        ..sealed_station_scenario()
    }
}

fn build(scn: &SealedStationScenario) -> (State, Registry, Registry) {
    build_sealed_station(
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
    .unwrap()
}

fn resolvers(scn: &SealedStationScenario, blackout: bool) -> (SourceResolver, SourceResolver) {
    let bio = sealed_bio_resolver(&station::params::lamp(), scn).unwrap();
    let mut fast = sealed_fast_resolver(&domains::params::charge(), scn).unwrap();
    if blackout {
        fast = with_brownout(fast, BLACKOUT.0, BLACKOUT.1, 0.0).unwrap();
    }
    (bio, fast)
}

struct Run {
    states: Vec<State>,
    rationed: u64,
    log: ShedLog,
}

fn plain(scn: &SealedStationScenario, fast: &SourceResolver, bio: &SourceResolver) -> Run {
    let (state, bio_reg, fast_reg) = build(scn);
    let (states, rationed, events) = run_master_day(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(fast_reg),
        state,
        bio,
        fast,
        DAYS,
        scn.steps_per_day,
        scn.bio_steps_per_day,
        scn.bio_dt,
        scn.cabin_dt,
        None,
    )
    .unwrap();
    assert!(events.is_empty());
    Run {
        states,
        rationed,
        log: ShedLog::default(),
    }
}

fn lab(
    scn: &SealedStationScenario,
    fast: &SourceResolver,
    bio: &SourceResolver,
    reserve_j: f64,
) -> Run {
    let (state, bio_reg, fast_reg) = build(scn);
    let (state, bio_reg, fast_reg) =
        rewire_for_shedding(state, bio_reg, fast_reg, reserve_j).unwrap();
    let (states, rationed, events, log) = run_shedding(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(fast_reg),
        state,
        bio,
        fast,
        scn,
        DAYS,
    )
    .unwrap();
    assert!(events.is_empty());
    Run {
        states,
        rationed,
        log,
    }
}

fn reserve(scn: &SealedStationScenario) -> f64 {
    what_if_reserve(&domains::params::charge(), scn)
}

fn crop(state: &State) -> f64 {
    [LEAF_C, STEM_C, ROOT_C, STORAGE_C]
        .iter()
        .map(|s| state.stocks[*s].amount)
        .sum()
}

/// The lab state with its bookkeeping slot removed — what the plain run carries.
fn without_slot(state: &State) -> State {
    let mut s = state.clone();
    s.aux.remove(LAMP_DELIVERY_AUX);
    s
}

fn assert_same_run(lab: &Run, plain: &Run) {
    assert_eq!(lab.rationed, plain.rationed);
    assert_eq!(lab.states.len(), plain.states.len());
    for (day, (l, p)) in lab.states.iter().zip(&plain.states).enumerate() {
        assert_eq!(&without_slot(l), p, "day {day} differs");
    }
}

fn small_lab_baseline() -> &'static Run {
    static RUN: OnceLock<Run> = OnceLock::new();
    RUN.get_or_init(|| {
        let scn = scenario(SMALL_BATTERY);
        let (bio, fast) = resolvers(&scn, false);
        lab(&scn, &fast, &bio, reserve(&scn))
    })
}

#[test]
fn a_run_resumed_from_a_saved_day_is_the_continuous_run_bit_for_bit() {
    // The share lives in `State` so that a saved day carries it. Split an 8-day run whose
    // lamp is cut over days 2–5 at day 5 — the last group of day 5 is the first lit one
    // after the cut, so a day-end state that held a stale share would resume dark.
    let scn = scenario(sealed_station_scenario().battery0);
    let (bio, _) = resolvers(&scn, false);
    let cut = || {
        with_lamp_power_cut(
            sealed_fast_resolver(&domains::params::charge(), &scn).unwrap(),
            BLACKOUT.0,
            BLACKOUT.1,
        )
        .unwrap()
    };
    let whole = lab(&scn, &cut(), &bio, reserve(&scn));
    // ⚠ Comparing the resumed crop alone cannot see a stale share: a day starts at midnight,
    // inside the lamp's dark hours, so the first plant step multiplies zero light. So each
    // saved day must carry the share of its own last group.
    let groups = whole.log.delivery.len() / DAYS;
    for day in 1..=DAYS {
        assert_eq!(
            whole.states[day].aux[LAMP_DELIVERY_AUX],
            whole.log.delivery[day * groups - 1],
            "the day-{day} state carries a stale share"
        );
    }
    let (_, bio_reg, fast_reg) = build(&scn);
    let saved = whole.states[5].clone();
    let (state, bio_reg, fast_reg) =
        rewire_for_shedding(saved, bio_reg, fast_reg, reserve(&scn)).unwrap();
    let (resumed, _, _, _) = run_shedding(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(fast_reg),
        state,
        &bio,
        &cut(),
        &scn,
        DAYS - 5,
    )
    .unwrap();
    assert_eq!(resumed.last().unwrap(), whole.states.last().unwrap());
}

fn small_lab_blackout() -> &'static Run {
    static RUN: OnceLock<Run> = OnceLock::new();
    RUN.get_or_init(|| {
        let scn = scenario(SMALL_BATTERY);
        let (bio, fast) = resolvers(&scn, true);
        lab(&scn, &fast, &bio, reserve(&scn))
    })
}

// --- controls -------------------------------------------------------------------------

#[test]
fn rule_off_the_lab_wiring_is_the_plain_sealed_run_bit_for_bit() {
    let scn = scenario(sealed_station_scenario().battery0);
    let (bio, fast) = resolvers(&scn, false);
    let lab = lab(&scn, &fast, &bio, f64::NEG_INFINITY);
    assert!(lab.log.delivery.iter().all(|&d| d == 1.0));
    assert_same_run(&lab, &plain(&scn, &fast, &bio));
}

#[test]
fn rule_on_with_the_frozen_battery_nothing_sheds_and_nothing_moves() {
    // The promotion prediction: the frozen battery never reaches the reserve inside this
    // horizon, so the share is the literal 1.0 throughout and the run is the plain one.
    let scn = scenario(sealed_station_scenario().battery0);
    let (bio, fast) = resolvers(&scn, false);
    let lab = lab(&scn, &fast, &bio, reserve(&scn));
    assert!(lab.log.delivery.iter().all(|&d| d == 1.0));
    assert_same_run(&lab, &plain(&scn, &fast, &bio));
}

#[test]
fn the_small_battery_alone_does_not_reach_the_reserve() {
    let run = small_lab_baseline();
    assert!(run.log.delivery.iter().all(|&d| d == 1.0));
    assert_eq!(run.rationed, 0);
    let scn = scenario(SMALL_BATTERY);
    assert!(run.states.last().unwrap().stocks[BATTERY].amount > reserve(&scn));
}

// --- the defect, as the reference has it --------------------------------------------------

#[test]
fn the_plain_crop_does_not_feel_a_blackout() {
    // Today: the blackout drains the battery, and the crop is bit-identical to the run
    // without it, because its light is the lamp's nameplate.
    let scn = scenario(SMALL_BATTERY);
    let (bio, calm) = resolvers(&scn, false);
    let (_, dark) = resolvers(&scn, true);
    let calm = plain(&scn, &calm, &bio);
    let dark = plain(&scn, &dark, &bio);
    let (c, d) = (calm.states.last().unwrap(), dark.states.last().unwrap());
    assert!(d.stocks[BATTERY].amount < c.stocks[BATTERY].amount);
    assert_eq!(crop(d).to_bits(), crop(c).to_bits());
}

// --- the lab answer ---------------------------------------------------------------------

#[test]
fn a_blackout_sheds_the_lamp_and_the_crop_feels_it() {
    let calm = small_lab_baseline();
    let dark = small_lab_blackout();
    let (c, d) = (calm.states.last().unwrap(), dark.states.last().unwrap());
    // The lamp was shed: some group delivered less than its nominal power…
    assert!(dark.log.delivery.iter().any(|&x| x < 1.0));
    assert!(d.stocks[LIGHT_USED].amount < c.stocks[LIGHT_USED].amount);
    // …life support was never short (the reserve held, the backstop never fired)…
    assert_eq!(dark.rationed, 0);
    // …and the crop grew less.
    assert!(crop(d) < crop(c));
}

#[test]
fn a_shed_lamp_on_the_sealed_station_never_comes_back() {
    // §6: with the lamp off, this station's solar pays exactly its life support, so the
    // battery never charges back over the reserve once the blackout has passed.
    let dark = small_lab_blackout();
    let first_off = dark.log.delivery.iter().position(|&x| x < 1.0).unwrap();
    assert!(dark.log.delivery[first_off + 1..].iter().all(|&x| x == 0.0));
}

#[test]
fn cutting_only_the_lamps_power_darkens_the_lab_crop() {
    // The lighting-failure perturbation cuts the crop's light and the lamp's draw by hand,
    // together. The lab crop reads what the lamp drew, so the draw alone is enough.
    let scn = scenario(sealed_station_scenario().battery0);
    let (bio, fast) = resolvers(&scn, false);
    let cut = with_lamp_power_cut(
        sealed_fast_resolver(&domains::params::charge(), &scn).unwrap(),
        BLACKOUT.0,
        BLACKOUT.1,
    )
    .unwrap();
    let lit = lab(&scn, &fast, &bio, reserve(&scn));
    let off = lab(&scn, &cut, &bio, reserve(&scn));
    assert!(off.log.delivery.contains(&0.0));
    assert!(crop(off.states.last().unwrap()) < crop(lit.states.last().unwrap()));
    // The plain build cannot see it: its crop is lit by the nameplate.
    let plain_cut = plain(&scn, &cut, &bio);
    let plain_lit = plain(&scn, &fast, &bio);
    assert_eq!(
        crop(plain_cut.states.last().unwrap()).to_bits(),
        crop(plain_lit.states.last().unwrap()).to_bits()
    );
}
