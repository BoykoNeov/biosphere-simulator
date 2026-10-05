//! The plant chamber's heat store, Step 3c slice 2b-i (`docs/plans/post-roadmap-room-temperature.md`
//! §23e). The lamp's waste heat now passes through `thermal.chamber`, whose cooler hands it on
//! to the node. The plants do not read the chamber yet.
//!
//! Nominally the chamber holds its steady state (P2) and the reference run did not move a single
//! stock (graded in §23f). This file checks the side the reference run cannot show: the room
//! **failing** (P9). With the cooler dead, the chamber warms at the lamp's waste heat over its
//! heat capacity, the node — losing that input — relaxes toward the colder equilibrium of the
//! remaining dissipation, and the crop, which reads no chamber temperature, is untouched.

use domains::biosphere::perturbations::{window_override, with_forcing};
use domains::thermal::{equilibrium_temperature, ThermalParams, NODE};
use simcore::environment::constant;
use simcore::flow::Flow;
use simcore::integrator::EulerIntegrator;
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::State;
use station::chamber::{chamber_temperature, CHAMBER, CHAMBER_COOLING};
use station::driver::run_master_day;
use station::perturbations::ScaledFlow;
use station::scenario::sealed_station_scenario;
use station::sealed::{
    build_sealed_station, lamp_waste_heat_w, sealed_bio_resolver, sealed_fast_resolver,
    sealed_node_heat, sealed_reset_hook,
};

const DAYS: usize = 60;
const COOLER_HEALTH: &str = "test.chamber_cooler_health";

/// Run the sealed station `DAYS` master days, the chamber's cooler at `health` throughout.
fn run(health: f64) -> Vec<State> {
    let scenario = sealed_station_scenario();
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
            } else {
                f
            }
        })
        .collect();
    assert_eq!(
        wrapped, 1,
        "the sealed build wires exactly one chamber cooler"
    );
    let fast_reg = Registry::new(flows, &state.stocks, aux).unwrap();
    let fast_resolver = with_forcing(
        sealed_fast_resolver(&charge, &scenario).unwrap(),
        COOLER_HEALTH,
        window_override(constant(1.0).unwrap(), 0, u64::MAX, health),
    )
    .unwrap();
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
    assert_eq!(rationed, 0);
    assert!(events.is_empty());
    states
}

fn node_t(s: &State, th: &ThermalParams) -> f64 {
    th.space_temperature + s.stocks[NODE].amount / th.heat_capacity
}

#[test]
fn a_dead_cooler_heats_the_chamber_cools_the_node_and_leaves_the_crop_alone() {
    let th = domains::params::thermal();
    let ch = station::params::chamber();
    let scenario = sealed_station_scenario();
    let lamp = station::params::lamp();
    let nominal = run(1.0);
    let failed = run(0.0);

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
    let w = lamp_waste_heat_w(&lamp, &scenario);
    let per_day = w * 86_400.0 / ch.heat_capacity;
    assert!(
        (per_day - 34.787).abs() < 1e-3,
        "the hand rate 1.449 K/h: {per_day}"
    );
    let t1 = chamber_temperature(failed[1].stocks[CHAMBER].amount, &ch);
    assert!(
        ((t1 - t0) - per_day).abs() <= 1e-9 * per_day,
        "day-1 rise {} vs {per_day}",
        t1 - t0
    );

    // The node, losing the waste heat, relaxes toward the colder closed-form equilibrium.
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
    assert!(
        (last - t_eq_failed).abs() < 0.15,
        "after {DAYS} days (4.7 time constants) the node is at {last}, not near {t_eq_failed}"
    );
    // Nominal node holds.
    let nominal_end = node_t(nominal.last().unwrap(), &th);
    assert!((nominal_end - t_eq_nominal).abs() < 1e-6, "{nominal_end}");

    // The crop reads no chamber temperature: every non-energy stock and every aux value is
    // byte-identical between the two runs, on every day.
    let mut compared = 0;
    for (a, b) in nominal.iter().zip(&failed) {
        assert_eq!(a.aux, b.aux, "day {}: aux differs", a.n);
        for (id, sa) in &a.stocks {
            if sa.quantity == Quantity::Energy {
                continue;
            }
            assert_eq!(
                sa.amount.to_bits(),
                b.stocks[id].amount.to_bits(),
                "day {}: {id} moved with the cooler",
                a.n
            );
            compared += 1;
        }
    }
    assert!(compared > 30 * DAYS, "compared only {compared} stock-days");
}
