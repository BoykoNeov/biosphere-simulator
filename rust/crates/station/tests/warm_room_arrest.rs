//! **LAB-ONLY** — the frozen winter wheat in a warm held room never develops (Step 3c,
//! slice 1).
//!
//! Plan: `docs/plans/post-roadmap-room-temperature.md` §10, slice 1, whose predictions were
//! committed before this file. The sealed station is run with the plants' room held at the
//! chosen 22 °C setpoint and everything else unchanged. (22 °C is BVAD Table 4-73's crew
//! *cabin* nominal; the plants sit in their own chamber, so it applies by analogy only.)
//! The cited vernalization window ends at 12 °C, so no chill-day ever accrues, the
//! vernalization factor stays exactly 0, and development never starts.
//!
//! This is the control for slice 3: it shows that the cold period, not the warm room, is
//! what lets the crop develop. No golden, no freeze.
//!
//! ⚠ **Re-expressed in slice 3a (§24h).** The plants read the chamber now, and a plant-side
//! temperature forcing is refused, so the warm room is **the chamber with no cold period**
//! (`cold.days = 0`): held at 22 °C from day 0 (22.05 °C as the plants read it — the chamber
//! settles one step's input above its setpoint). The plain run is the reference, cold period
//! and all.

use std::sync::OnceLock;

use domains::biosphere::stocks::{
    LEAF_C, ROOT_C, STEM_C, STORAGE_C, THERMAL_TIME, VERNALIZATION_DAYS,
};
use simcore::integrator::EulerIntegrator;
use simcore::state::State;
use station::chamber::{chamber_temperature, CHAMBER};
use station::scenario::{sealed_station_scenario, ColdProgram, SealedStationScenario};
use station::sealed::{
    build_sealed_station, run_sealed, sealed_bio_resolver, sealed_fast_resolver,
};
use station::sowing::sown_step;

/// The decided setpoint (note §10, decision 4) — `chamber.yaml`'s warm one.
const ROOM_C: f64 = 22.0;

fn scenario(years: usize) -> SealedStationScenario {
    SealedStationScenario {
        years,
        ..sealed_station_scenario()
    }
}

/// The plain sealed run, or the same run with no cold period: the chamber held at [`ROOM_C`].
fn run(scn: &SealedStationScenario, warm: bool) -> Result<Vec<State>, simcore::error::SimError> {
    let scn = &if warm {
        SealedStationScenario {
            cold: ColdProgram {
                days: 0,
                ..scn.cold
            },
            ..*scn
        }
    } else {
        *scn
    };
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
    )?;
    let bio = sealed_bio_resolver(&station::params::lamp(), scn)?;
    let fast = sealed_fast_resolver(&domains::params::charge(), scn)?;
    let (states, _rationed, _events) = run_sealed(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(fast_reg),
        state,
        &bio,
        &fast,
        scn,
    )?;
    Ok(states)
}

/// One season each way, computed once and shared by the tests below.
fn season() -> &'static (Vec<State>, Vec<State>) {
    static RUNS: OnceLock<(Vec<State>, Vec<State>)> = OnceLock::new();
    RUNS.get_or_init(|| {
        let scn = scenario(1);
        (run(&scn, false).unwrap(), run(&scn, true).unwrap())
    })
}

fn veg(state: &State) -> f64 {
    state.stocks[LEAF_C].amount + state.stocks[STEM_C].amount + state.stocks[ROOT_C].amount
}

/// The instrument can see development: the plain run vernalizes, develops and fills grain.
/// Without this, the zeros below could be a reading that is always zero.
///
/// ⚠ Read over the season, not on its last day (slice 4 stage 2, §25g): the plain crop is
/// re-sown when it matures, so day 305 holds a third crop 27 days old, with no grain yet.
#[test]
fn control_the_plain_season_develops_and_fills_grain() {
    let plain = &season().0;
    assert!(plain.iter().any(|s| s.aux.get(VERNALIZATION_DAYS).is_some_and(|v| *v > 0.0)));
    assert!(plain.iter().any(|s| s.aux[THERMAL_TIME] > 0.0));
    assert!(plain.iter().any(|s| s.stocks[STORAGE_C].amount > 0.0));
    // …and it is re-sown, so the warm run's never-re-sown sowing below is not a constant.
    assert!(plain.iter().any(|s| sown_step(s).unwrap() > 0));
}

/// The warm run's room is the chamber held at [`ROOM_C`] — read off the chamber, every day.
#[test]
fn the_warm_room_is_the_chamber_held_at_22c() {
    let ch = station::params::chamber();
    for (day, s) in season().1.iter().enumerate() {
        let t = chamber_temperature(s.stocks[CHAMBER].amount, &ch) - 273.15;
        assert!(
            (t - ROOM_C).abs() < 0.1,
            "day {day}: the chamber is at {t} °C"
        );
    }
}

/// Prediction 1: no chill-day, so no thermal time, at any day of the warm season.
///
/// The sown state (day 0) carries no `vernalization_days` entry: `build_sealed_station`
/// seeds only `thermal_time` and `rooted_depth`. The development flow reads a missing entry
/// as 0, so an absent entry here reads as 0 too — on day 0 only.
#[test]
fn warm_room_accrues_no_chill_day_and_no_development() {
    for (day, s) in season().1.iter().enumerate() {
        let chill = match s.aux.get(VERNALIZATION_DAYS) {
            Some(v) => *v,
            None if day == 0 => 0.0,
            None => panic!("no chill-day accumulator on day {day}"),
        };
        assert_eq!(chill, 0.0, "chill-days on day {day}");
        assert_eq!(s.aux[THERMAL_TIME], 0.0, "thermal time on day {day}");
    }
}

/// Prediction 2: no grain ever, while the vegetative crop keeps growing.
#[test]
fn warm_room_grows_leaf_stem_root_and_no_grain() {
    let warm = &season().1;
    for (day, s) in warm.iter().enumerate() {
        assert_eq!(s.stocks[STORAGE_C].amount, 0.0, "grain on day {day}");
    }
    let scn = scenario(1);
    let seedling = scn.bio.leaf_c0 + scn.bio.stem_c0 + scn.bio.root_c0;
    let end = veg(warm.last().unwrap());
    assert!(
        end > seedling,
        "vegetative carbon {end} did not grow past the seedling {seedling}"
    );
}

/// Prediction 3, RESTATED by slice 4 stage 2 (§25f): the warm crop never matures, so it is never
/// re-sown — over two seasons its sowing stays at step 0 and it holds no grain.
///
/// Until stage 2 the re-sow was on a 305-day calendar, and this test asserted that the first
/// re-sow refused for want of seed ("seed bank too small to re-sow — storage_c 0.0"). With the
/// re-sow on maturity that refusal is unreachable from a warm room; the refusal itself stays
/// covered where it lives (`domains`, `reset_crop`).
#[test]
fn warm_room_is_never_resown_because_it_never_matures() {
    let warm = run(&scenario(2), true).expect("a crop that never matures is never re-sown");
    assert_eq!(warm.len(), 2 * 305 + 1);
    for (day, s) in warm.iter().enumerate() {
        assert_eq!(sown_step(s).unwrap(), 0, "re-sown by day {day}");
        assert_eq!(s.stocks[STORAGE_C].amount, 0.0, "grain on day {day}");
    }
}
