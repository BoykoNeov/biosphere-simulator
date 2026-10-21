//! The plant chamber's heat coefficients are an INPUT of the sealed build
//! (`SealedStationScenario::chamber`, 2026-10-21; `docs/plans/post-roadmap-big-station.md`): a
//! chamber can be sized, and every reader takes the scenario's value — none re-reads
//! `chamber.yaml`.
//!
//! A reader that still read the file would see the wrong room. The chamber's temperature is its
//! heat over its heat capacity, so a chamber given four times the capacity, read anywhere at the
//! file's, reads four times as hot: ~1 180 K to the plants or the room's condenser, or a start
//! charged at a quarter of the setpoint. The checks are that a 4× chamber holds the same
//! temperature as the reference one and grows the same crop with the same air, and that a moved
//! setpoint is the temperature the room settles at.

use domains::biosphere::stocks::{LEAF_C, ROOT_C, STEM_C, WATER_VAPOR};
use simcore::integrator::EulerIntegrator;
use simcore::state::State;
use station::chamber::{chamber_temperature, ChamberParams, CHAMBER};
use station::scenario::{sealed_station_scenario, ColdProgram, SealedStationScenario};
use station::sealed::{build_sealed_station, run_sealed, sealed_bio_resolver, sealed_fast_resolver};

const DAYS: usize = 3;

/// The sealed station held warm from day 0 (no cold period), with `chamber`.
fn scenario(chamber: ChamberParams) -> SealedStationScenario {
    let s = sealed_station_scenario();
    SealedStationScenario {
        cold: ColdProgram { days: 0, ..s.cold },
        years: 1,
        season_days: DAYS,
        chamber,
        ..s
    }
}

/// The day-end states of `DAYS` master days.
fn run(sc: &SealedStationScenario) -> Vec<State> {
    let charge = domains::params::charge();
    let crew = domains::params::crew();
    let lamp = station::params::lamp();
    let (state, bio, fast) = build_sealed_station(
        &charge,
        &domains::params::thermal(),
        &crew,
        &domains::params::eclss(),
        &station::params::water_recovery(),
        &lamp,
        &station::params::harvest(),
        sc,
        false,
        false,
    )
    .expect("build");
    let bio_r = sealed_bio_resolver(&lamp, sc).expect("bio resolver");
    let fast_r = sealed_fast_resolver(&charge, &crew, sc).expect("fast resolver");
    let (states, _, _) = run_sealed(
        &EulerIntegrator::new(bio),
        &EulerIntegrator::new(fast),
        state,
        &bio_r,
        &fast_r,
        sc,
    )
    .expect("run");
    states
}

fn temperature(s: &State, chamber: &ChamberParams) -> f64 {
    chamber_temperature(s.stocks[CHAMBER].amount, chamber)
}

fn plant_c(s: &State) -> f64 {
    [LEAF_C, STEM_C, ROOT_C]
        .iter()
        .map(|id| s.stocks[*id].amount)
        .sum()
}

/// Four times the heat capacity: the same room, read at the scenario's capacity everywhere.
#[test]
fn a_chamber_given_as_input_reaches_every_reader() {
    let reference = station::params::chamber();
    let big = ChamberParams {
        heat_capacity: 4.0 * reference.heat_capacity,
        ..reference
    };
    let (a, b) = (run(&scenario(reference)), run(&scenario(big)));
    assert_eq!(a.len(), b.len());
    assert!(a.len() > DAYS, "the run reached its horizon: {} states", a.len());
    for (day, (sa, sb)) in a.iter().zip(&b).enumerate() {
        let (ta, tb) = (temperature(sa, &reference), temperature(sb, &big));
        assert!(
            (ta - tb).abs() < 0.5,
            "state {day}: the 4× chamber holds {tb} K, the reference {ta} K"
        );
        let (ca, cb) = (plant_c(sa), plant_c(sb));
        assert!(
            (ca - cb).abs() <= 0.01 * ca,
            "state {day}: the crop is {cb} mol C in the 4× chamber, {ca} in the reference"
        );
        let (va, vb) = (sa.stocks[WATER_VAPOR].amount, sb.stocks[WATER_VAPOR].amount);
        assert!(
            (va - vb).abs() <= 0.05 * va,
            "state {day}: the room holds {vb} kg of vapour in the 4× chamber, {va} in the reference"
        );
    }
}

/// A setpoint 3 K warmer is what the room holds: the cooler and the heater read the scenario's
/// setpoint, not the file's.
#[test]
fn a_moved_setpoint_is_the_temperature_the_room_holds() {
    let reference = station::params::chamber();
    let warmer = ChamberParams {
        warm_setpoint: reference.warm_setpoint + 3.0,
        ..reference
    };
    let states = run(&scenario(warmer));
    let end = states.last().expect("states");
    let t = temperature(end, &warmer);
    assert!(
        (t - warmer.warm_setpoint).abs() < 0.5,
        "the room ends at {t} K against a setpoint of {} K",
        warmer.warm_setpoint
    );
}

/// The lab's averaged-lamp start ([`station::sealed::with_averaged_chamber_start`]) charges the
/// chamber at the scenario's capacity: a 4× chamber starts at its setpoint, not at a quarter of it.
#[test]
fn the_averaged_start_charges_the_given_chamber() {
    let reference = station::params::chamber();
    let big = ChamberParams {
        heat_capacity: 4.0 * reference.heat_capacity,
        ..reference
    };
    let sc = scenario(big);
    let (state, _, _) = build_sealed_station(
        &domains::params::charge(),
        &domains::params::thermal(),
        &domains::params::crew(),
        &domains::params::eclss(),
        &station::params::water_recovery(),
        &station::params::lamp(),
        &station::params::harvest(),
        &sc,
        false,
        false,
    )
    .expect("build");
    let state = station::sealed::with_averaged_chamber_start(state, &sc).expect("start");
    let t = temperature(&state, &big);
    assert!(
        (t - big.warm_setpoint).abs() < 1.0,
        "the averaged start is {t} K against a setpoint of {} K",
        big.warm_setpoint
    );
}

/// The shared room's size reaches its regulators: in a room of twice the air the CO₂ scrubber and
/// the O₂ makeup hold twice the gas (their targets are partial pressures, taken on the scenario's
/// `chamber_air_capacity_mol` since 2026-10-21 — a `const` beside it until then).
#[test]
fn the_room_air_reaches_the_regulators() {
    let base = scenario(station::params::chamber());
    let double = base.with_chamber_air(2.0 * base.bio.chamber_air_capacity_mol);
    let (a, b) = (run(&base), run(&double));
    let (ea, eb) = (a.last().expect("states"), b.last().expect("states"));
    for id in [
        domains::biosphere::stocks::CARBON_POOL,
        domains::biosphere::stocks::O2_POOL,
    ] {
        let ratio = eb.stocks[id].amount / ea.stocks[id].amount;
        assert!(
            (ratio - 2.0).abs() < 0.05,
            "{id}: the doubled room holds {ratio}× the reference's"
        );
    }
}
