//! Phase-8 Step-6 (P8.6) byte-identity anchor — **what makes the fixed-palette builder legal
//! under the freeze**.
//!
//! [`station::builder::assemble`] composes a station from palette components; if the
//! `{PowerPlant, Radiator}` composition is **bit-for-bit** identical to the frozen
//! [`station::system::build_station`] — same stocks, same flows, same resolver, so the same
//! trajectory under stepping — then the builder is a **pure refactor** of existing
//! construction into composable pieces, and "no new science" is *proven, not asserted* (the
//! analogue of Step-0's session-parity teeth and Step-5's `health = 1` bit-identity).
//!
//! States are compared by their exact hex-float `sim_io` JSON (the golden discipline), so a
//! match is bit-exact — a single last-ULP difference would change the spelling and fail. This
//! runs in `cargo test`, no Godot toolchain — the go/no-go spike for the whole step.

use domains::params;
use domains::power::{
    build_power, power_resolver, BATTERY, BOUNDED_SOC_SCENARIO, SELF_DISCHARGE_DAYS,
};
use simcore::integrator::EulerIntegrator;
use simcore::snapshot::from_engine;
use simcore::state::State;
use station::builder::{assemble, BuildContext, Component};
use station::run_station;
use station::scenario::{StationScenario, HEAT_CLOSURE_DAYS, HEAT_CLOSURE_SCENARIO};
use station::system::{build_station, station_resolver};

use domains::thermal::NODE;

/// Exact hex-float snapshot of a state (n + seed + aux + stocks) — a string match is a
/// bit-for-bit match.
fn snap(state: &State) -> String {
    from_engine(state).to_json()
}

fn context(scenario: StationScenario) -> BuildContext {
    BuildContext {
        charge: params::charge(),
        thermal: params::thermal(),
        self_discharge: params::self_discharge(),
        scenario,
    }
}

/// `assemble([PowerPlant, Radiator])` == `build_station()`, stepped bit-exact over the
/// heat-closure horizon. The composition order is deliberately *reversed* from
/// `build_station`'s internal order (Radiator listed first) to prove the [`Registry`]'s
/// sort-by-id makes composition order irrelevant — a real property, not an artifact of
/// listing the components in the same order the reference builds them.
#[test]
fn station_composition_matches_build_station_bit_exact() {
    let scenario = HEAT_CLOSURE_SCENARIO;
    let steps = HEAT_CLOSURE_DAYS * 24; // 24 diurnal steps/day, the observation test's horizon

    // Reference: the frozen hand-built Power → Thermal station.
    let charge = params::charge();
    let thermal = params::thermal();
    let (ref_state, ref_reg) = build_station(&charge, &thermal, &scenario, None).unwrap();
    let ref_resolver = station_resolver(&charge, &scenario).unwrap();
    let mut noop = |_: &State| {};
    let (ref_final, ref_rationed, ref_events) = run_station(
        &EulerIntegrator::new(ref_reg),
        ref_state,
        &ref_resolver,
        scenario.power.dt_seconds,
        steps,
        &mut noop,
    )
    .unwrap();

    // Composition: assembled from the palette, Radiator-first to exercise order-independence.
    let (comp_state, comp_reg, comp_resolver) =
        assemble(&[Component::Radiator, Component::PowerPlant], &context(scenario)).unwrap();
    let (comp_final, comp_rationed, comp_events) = run_station(
        &EulerIntegrator::new(comp_reg),
        comp_state,
        &comp_resolver,
        scenario.power.dt_seconds,
        steps,
        &mut noop,
    )
    .unwrap();

    assert_eq!(snap(&comp_final), snap(&ref_final), "composed state bit-identical");
    assert_eq!(comp_rationed, ref_rationed, "rationed");
    assert_eq!(comp_events, ref_events, "events");
    // Non-vacuous: the station is well-fed and the node reached a real equilibrium (else the
    // comparison could pass on a trivially-empty run).
    assert_eq!(ref_rationed, 0);
    assert!(ref_events.is_empty());
    assert!(comp_final.stocks["thermal.node"].amount > 0.0, "node carries real heat");
}

/// Byte-identity anchor 2 (advisor): `assemble([PowerPlant, SelfDischarge])` (no radiator ⇒
/// dissipation to `boundary.waste_heat`) reproduces the frozen standalone
/// `build_power(charge, BOUNDED_SOC, Some(sd))` **bit-for-bit** — the three-flow leaky
/// microgrid. Rust-vs-Rust (bit-exact on one libm); it chains to the frozen
/// `power_self_discharge` golden via the Step-3 crossport test. The non-vacuity check folds in
/// the P5.5 "it earned its keep" signal: the leaky SOC must depart from the two-flow
/// (no-self-discharge) baseline, else an inert leak would still pass the equality.
#[test]
fn self_discharge_composition_matches_standalone_build_power_bit_exact() {
    let scenario = HEAT_CLOSURE_SCENARIO; // .power == BOUNDED_SOC_SCENARIO
    let steps = SELF_DISCHARGE_DAYS * BOUNDED_SOC_SCENARIO.steps_per_day;
    let charge = params::charge();
    let sd = params::self_discharge();
    let mut noop = |_: &State| {};

    // Reference: the frozen standalone three-flow leaky Power build.
    let (ref_state, ref_reg) = build_power(&charge, &BOUNDED_SOC_SCENARIO, Some(sd)).unwrap();
    let ref_resolver = power_resolver(&charge, &BOUNDED_SOC_SCENARIO).unwrap();
    let (ref_final, ref_rationed, ref_events) = run_station(
        &EulerIntegrator::new(ref_reg),
        ref_state,
        &ref_resolver,
        BOUNDED_SOC_SCENARIO.dt_seconds,
        steps,
        &mut noop,
    )
    .unwrap();

    // Composition: the palette build of the same leaky microgrid.
    let (comp_state, comp_reg, comp_resolver) =
        assemble(&[Component::PowerPlant, Component::SelfDischarge], &context(scenario)).unwrap();
    let (comp_final, comp_rationed, comp_events) = run_station(
        &EulerIntegrator::new(comp_reg),
        comp_state,
        &comp_resolver,
        BOUNDED_SOC_SCENARIO.dt_seconds,
        steps,
        &mut noop,
    )
    .unwrap();

    assert_eq!(snap(&comp_final), snap(&ref_final), "leaky composition bit-identical");
    assert_eq!(comp_rationed, ref_rationed);
    assert_eq!(comp_events, ref_events);
    assert_eq!(ref_rationed, 0);
    assert!(ref_events.is_empty());

    // Non-vacuity: the leak actually bit — the SOC departs from the two-flow baseline (which
    // is daily-balanced and returns near battery0). Same horizon, no self-discharge component.
    let (base_state, base_reg, base_resolver) =
        assemble(&[Component::PowerPlant], &context(scenario)).unwrap();
    let (base_final, base_rationed, _) = run_station(
        &EulerIntegrator::new(base_reg),
        base_state,
        &base_resolver,
        BOUNDED_SOC_SCENARIO.dt_seconds,
        steps,
        &mut noop,
    )
    .unwrap();
    assert_eq!(base_rationed, 0);
    assert!(
        comp_final.stocks[BATTERY].amount < base_final.stocks[BATTERY].amount,
        "the leak drains the battery below the no-self-discharge baseline"
    );
}

/// The flagship 3-part composition `{PowerPlant, Radiator, SelfDischarge}` — the "Leaky
/// station" preset the palette UI ships — is the only runtime config the palette enables with
/// **no** byte-identity anchor (the leak feeding the node alongside the radiator is neither
/// `build_station` nor `build_power(+sd)`). This test *runs* it (advisor): it steps well-fed
/// with a bounded node, and — turning the decomposition argument into a positive assertion —
/// its battery trajectory is **bit-identical** to `{PowerPlant, SelfDischarge}`, because every
/// battery leg (SolarCharge/LoadDraw/SelfDischarge) is independent of which id the *heat* leg
/// points at (node vs waste_heat). So the 3-part run is the composition of two
/// separately-anchored behaviours — the battery of anchor 2 and the node of anchor 1 plus a
/// tiny `k·battery` leak the T⁴ radiator sheds.
#[test]
fn three_part_leaky_station_steps_well_fed_with_a_bounded_node() {
    let scenario = HEAT_CLOSURE_SCENARIO; // .power == BOUNDED_SOC_SCENARIO
    let steps = SELF_DISCHARGE_DAYS * BOUNDED_SOC_SCENARIO.steps_per_day;
    let dt = BOUNDED_SOC_SCENARIO.dt_seconds;
    let mut noop = |_: &State| {};

    // The 3-part composition: plant + radiator + leak, all heat into the node.
    let (s3, r3, res3) = assemble(
        &[Component::PowerPlant, Component::Radiator, Component::SelfDischarge],
        &context(scenario),
    )
    .unwrap();
    let (final3, rationed3, events3) =
        run_station(&EulerIntegrator::new(r3), s3, &res3, dt, steps, &mut noop).unwrap();

    // Tier-0: well-fed and stable — a bounded, positive node (the leak adds a little heat the
    // radiator sheds; it does not run away).
    assert_eq!(rationed3, 0, "3-part composition is well-fed");
    assert!(events3.is_empty());
    let node = final3.stocks[NODE].amount;
    assert!(node.is_finite() && node > 0.0, "node carries bounded, positive heat: {node}");

    // The decomposition, made empirical: the battery trajectory equals {PowerPlant,
    // SelfDischarge}'s bit-for-bit — the heat-leg target (node vs waste_heat) never touches a
    // battery leg, and rationed==0 means no arbitration scaling perturbs it.
    let (s_sd, r_sd, res_sd) =
        assemble(&[Component::PowerPlant, Component::SelfDischarge], &context(scenario)).unwrap();
    let (final_sd, rationed_sd, _) =
        run_station(&EulerIntegrator::new(r_sd), s_sd, &res_sd, dt, steps, &mut noop).unwrap();
    assert_eq!(rationed_sd, 0);
    assert_eq!(
        final3.stocks[BATTERY].amount,
        final_sd.stocks[BATTERY].amount,
        "the leak's battery draw is independent of where its heat leg points",
    );
}

/// The initial states already match before any stepping — the strongest form of the anchor
/// (identical `State` at n=0 ⇒ identical trajectory under the same registry + resolver).
#[test]
fn station_composition_matches_build_station_at_construction() {
    let scenario = HEAT_CLOSURE_SCENARIO;
    let charge = params::charge();
    let thermal = params::thermal();
    let (ref_state, _ref_reg) = build_station(&charge, &thermal, &scenario, None).unwrap();
    let (comp_state, _comp_reg, _comp_resolver) =
        assemble(&[Component::PowerPlant, Component::Radiator], &context(scenario)).unwrap();
    assert_eq!(snap(&comp_state), snap(&ref_state), "initial state bit-identical");
}
