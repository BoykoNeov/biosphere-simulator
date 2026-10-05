//! The plant chamber's heat store, Step 3c slice 2b (`docs/plans/post-roadmap-room-temperature.md`
//! §23e–§23g). The lamp's whole draw — its waste heat (2b-i) and its light (2b-ii) — heats
//! `thermal.chamber`, whose cooler hands it on to the node.
//!
//! Nominally the chamber holds its steady state. This file checks the side the reference run
//! cannot show: the room **failing** (2b-i's P9, 2b-ii's Q7). With the cooler dead, the chamber
//! warms at the lamp's draw over its heat capacity and the node — losing that input — relaxes
//! toward the colder equilibrium of the remaining dissipation.
//!
//! ⚠ **Since slice 3a the plants read the chamber** (`docs/plans/post-roadmap-room-temperature.md`
//! §24h), so P9's "the crop is untouched" INVERTED, by design: it is now this file's liveness
//! for the plants reading the room — the overheated crop's leaf, stem and root fall below the
//! nominal crop's over day 1. With the walls off a dead cooler drives the chamber without bound
//! (thousands of kelvin by day 60, outside every plant formula's range), so the failed run's
//! rationing is recorded as found, not asserted zero. **The cold period is opted out**
//! (`cold.days = 0`): this file's subject is the cooler of a warm chamber.
//!
//! ⚠ **The walls (2b-iii) are switched off in every run here** (scaled by 0), so this file keeps
//! checking the cooler alone: with walls the nominal chamber follows the weather and a dead
//! cooler's chamber climbs toward its walls' equilibrium, not linearly (§23j). The walls and
//! the heater have their own file, `tests/chamber_walls.rs`.

use domains::biosphere::perturbations::{window_override, with_forcing};
use domains::thermal::{equilibrium_temperature, ThermalParams, NODE};
use simcore::environment::constant;
use simcore::flow::Flow;
use simcore::integrator::EulerIntegrator;
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::State;
use station::chamber::{chamber_temperature, CHAMBER, CHAMBER_COOLING, CHAMBER_WALL};
use station::driver::run_master_day;
use station::perturbations::ScaledFlow;
use station::scenario::{sealed_station_scenario, ColdProgram, SealedStationScenario};
use station::sealed::{
    build_sealed_station, full_lamp_heat_input_w, sealed_bio_resolver, sealed_fast_resolver,
    sealed_node_heat, sealed_reset_hook,
};

const DAYS: usize = 60;
const COOLER_HEALTH: &str = "test.chamber_cooler_health";
/// The chamber's walls (2b-iii), switched OFF in every run here (see the module doc).
const WALL_HEALTH: &str = "test.chamber_wall_health";

/// The sealed scenario without the cold period: the chamber held warm from day 0.
fn warm_scenario() -> SealedStationScenario {
    let s = sealed_station_scenario();
    SealedStationScenario {
        cold: ColdProgram { days: 0, ..s.cold },
        ..s
    }
}

/// Run the sealed station `DAYS` master days, the chamber's cooler at `health` throughout:
/// `(states, rationed)`.
fn run(health: f64) -> (Vec<State>, u64) {
    let scenario = warm_scenario();
    let charge = domains::params::charge();
    let lamp = station::params::lamp();
    let (state, bio_reg, fast_reg) = build_sealed_station(
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
    .unwrap();
    let (flows, aux) = fast_reg.into_parts();
    let mut wrapped = 0;
    let flows: Vec<Box<dyn Flow>> = flows
        .into_iter()
        .map(|f| {
            if f.id() == CHAMBER_COOLING {
                wrapped += 1;
                Box::new(ScaledFlow::new(f, COOLER_HEALTH.to_string())) as Box<dyn Flow>
            } else if f.id() == CHAMBER_WALL {
                wrapped += 10;
                Box::new(ScaledFlow::new(f, WALL_HEALTH.to_string())) as Box<dyn Flow>
            } else {
                f
            }
        })
        .collect();
    assert_eq!(
        wrapped, 11,
        "the sealed build wires exactly one chamber cooler and one set of walls"
    );
    let fast_reg = Registry::new(flows, &state.stocks, aux).unwrap();
    let fast_resolver = with_forcing(
        sealed_fast_resolver(&charge, &scenario).unwrap(),
        COOLER_HEALTH,
        window_override(constant(1.0).unwrap(), 0, u64::MAX, health),
    )
    .unwrap();
    let fast_resolver = with_forcing(fast_resolver, WALL_HEALTH, constant(0.0).unwrap()).unwrap();
    let bio_resolver = sealed_bio_resolver(&lamp, &scenario).unwrap();
    let reset = sealed_reset_hook(&scenario);
    let (states, rationed, events) = run_master_day(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(fast_reg),
        state,
        &bio_resolver,
        &fast_resolver,
        DAYS,
        scenario.steps_per_day,
        scenario.bio_steps_per_day,
        scenario.bio_dt,
        scenario.cabin_dt,
        Some(&*reset),
    )
    .unwrap();
    assert!(events.is_empty());
    (states, rationed)
}

/// Leaf + stem + root carbon (mol).
fn veg(s: &State) -> f64 {
    use domains::biosphere::stocks::{LEAF_C, ROOT_C, STEM_C};
    s.stocks[LEAF_C].amount + s.stocks[STEM_C].amount + s.stocks[ROOT_C].amount
}

fn node_t(s: &State, th: &ThermalParams) -> f64 {
    th.space_temperature + s.stocks[NODE].amount / th.heat_capacity
}

#[test]
fn a_dead_cooler_heats_the_chamber_cools_the_node_and_overheats_the_crop() {
    let th = domains::params::thermal();
    let ch = station::params::chamber();
    let scenario = warm_scenario();
    let lamp = station::params::lamp();
    let (nominal, nominal_rationed) = run(1.0);
    let (failed, failed_rationed) = run(0.0);
    assert_eq!(nominal_rationed, 0, "the nominal run rationed");
    // §24h's prediction: maintenance doubles every 10 K and soon asks for more than the organs
    // hold, so the backstop fires. Recorded, not a target.
    eprintln!("the failed run rationed {failed_rationed} times over {DAYS} days");
    assert!(
        failed_rationed > 0,
        "a crop in a chamber thousands of kelvin hot never rationed"
    );

    // Nominal: the chamber holds its steady state on every day (P2).
    let t0 = chamber_temperature(nominal[0].stocks[CHAMBER].amount, &ch);
    for s in &nominal {
        let t = chamber_temperature(s.stocks[CHAMBER].amount, &ch);
        assert!(
            (t - t0).abs() <= 1e-9,
            "nominal chamber drifted: {t} vs {t0}"
        );
    }

    // Failed: it warms at w / C_ch, exactly linear (the lamp's draw is the daily average).
    let w = full_lamp_heat_input_w(&scenario);
    let per_day = w * 86_400.0 / ch.heat_capacity;
    assert!(
        (per_day - 76.8).abs() < 1e-9,
        "the hand rate 3.2 K/h (133.33 W over 1.5e5 J/K): {per_day}"
    );
    let t1 = chamber_temperature(failed[1].stocks[CHAMBER].amount, &ch);
    assert!(
        ((t1 - t0) - per_day).abs() <= 1e-9 * per_day,
        "day-1 rise {} vs {per_day}",
        t1 - t0
    );

    // The node, losing the lamp's heat, relaxes toward the colder closed-form equilibrium.
    let t_eq_nominal = node_t(&nominal[0], &th);
    let heat_now = sealed_node_heat(&domains::params::charge(), &th, &lamp, &scenario)
        / th.heat_capacity
        + th.space_temperature;
    assert_eq!(t_eq_nominal, heat_now, "the node starts at its equilibrium");
    let dissipation = th.emissivity
        * domains::thermal::STEFAN_BOLTZMANN
        * th.radiator_area
        * (t_eq_nominal.powf(4.0) - th.space_temperature.powf(4.0));
    let t_eq_failed = equilibrium_temperature(&th, dissipation - w);
    assert!((t_eq_failed - 160.308).abs() < 1e-3, "{t_eq_failed}");
    let mut last = t_eq_nominal;
    for s in failed.iter().skip(1) {
        let t = node_t(s, &th);
        assert!(
            t < last,
            "the node must fall monotonically: {t} after {last}"
        );
        last = t;
    }
    // How close, derived rather than banded: for T ≥ T_eq, T⁴ − T_eq⁴ ≥ 4·T_eq³·(T − T_eq), so
    // the gap shrinks at least as fast as `exp(−t/τ_eq)`, τ_eq = C / (4εσA·T_eq³) — the
    // relaxation time AT the target, the slowest on the way down. And it never overshoots.
    // ⚠ A fixed 0.15 K band stood here in 2b-i, sized for that slice's 7.1 K starting gap; 2b-ii
    // starts the node 14.65 K above the target and the band was not re-derived (§23h, Q7).
    let tau_eq = th.heat_capacity
        / (4.0
            * th.emissivity
            * domains::thermal::STEFAN_BOLTZMANN
            * th.radiator_area
            * t_eq_failed.powf(3.0));
    let bound = (t_eq_nominal - t_eq_failed) * (-(DAYS as f64) * 86_400.0 / tau_eq).exp();
    assert!(
        last > t_eq_failed && last - t_eq_failed <= bound,
        "after {DAYS} days the node is {} K above {t_eq_failed}; the bound is {bound} K",
        last - t_eq_failed
    );
    // Nominal node holds.
    let nominal_end = node_t(nominal.last().unwrap(), &th);
    assert!((nominal_end - t_eq_nominal).abs() < 1e-6, "{nominal_end}");

    // The crop reads the chamber (slice 3a): identical at the start, and over day 1 — the
    // chamber 76.8 K above nominal by its end — the overheated crop burns more than it fixes,
    // so its leaf + stem + root fall below the nominal crop's.
    for (id, sa) in &nominal[0].stocks {
        if sa.quantity != Quantity::Energy {
            assert_eq!(
                sa.amount.to_bits(),
                failed[0].stocks[id].amount.to_bits(),
                "{id}"
            );
        }
    }
    let (v_nominal, v_failed) = (veg(&nominal[1]), veg(&failed[1]));
    eprintln!("day 1: leaf + stem + root nominal {v_nominal}, overheated {v_failed}");
    assert!(
        v_failed < v_nominal,
        "the overheated crop's leaf + stem + root {v_failed} is not below the nominal {v_nominal}"
    );
}
