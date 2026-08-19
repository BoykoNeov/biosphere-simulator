//! The standalone Power **self-discharge** validation (Stage-3 slice S3).
//!
//! Subject: `tests/test_power_self_discharge.py`'s 11 cases. `SelfDischarge` is the opt-in
//! third Power flow and the first one that reads a **stock**, which is what makes this
//! file different from `power_run.rs` rather than more of it:
//!
//! * **The contraction test is the magnitude-independent proof of a restoring force.** Two
//!   runs differing *only* in `battery0` under identical forcing: the forced terms cancel
//!   in the difference, so under Euler `d_n = d_0·(1 − k·dt)^n` exactly — a geometric
//!   contraction that measures `k` back out. A forced-only system keeps `d_n` *constant*,
//!   which is the contrast the next test asserts, and together they *distinguish* a
//!   donor-controlled flow from a forced one. It holds at the realistic Li-ion rate
//!   (~2.6 %/month) where the SOC barely bends: the property is proved by the algebra, not
//!   by a visible convergence, so no rate had to be inflated to make it show.
//! * **RK4 ≢ Euler.** The forced-only bit-identity that `power_run.rs` asserts is *broken*
//!   by the state-dependent leak, so the integrator cross-check here is a real tolerance
//!   agreement rather than an algebraic identity.
//!
//! Helpers are duplicated from the sibling `*_run.rs` files by the same house rule stated
//! in `power_run.rs`.

use std::collections::BTreeMap;

use domains::params;
use domains::power::{
    build_power, power_resolver, PowerScenario, BATTERY, BOUNDED_SOC_SCENARIO, SELF_DISCHARGE_DAYS,
    SOLAR_SOURCE, WASTE_HEAT,
};
use domains::{run_trajectory, StepIntegrator};
use simcore::conservation::compute_ledger;
use simcore::events::Event;
use simcore::integrator::{EulerIntegrator, Rk4Integrator};
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::State;

const STEPS: u64 = SELF_DISCHARGE_DAYS * BOUNDED_SOC_SCENARIO.steps_per_day;
const DT: f64 = BOUNDED_SOC_SCENARIO.dt_seconds;

/// The per-step contraction factor's argument, `k·dt`.
fn k_dt() -> f64 {
    params::self_discharge().self_discharge_rate * DT
}

fn run<F>(scenario: &PowerScenario, leaky: bool, make: F) -> (Vec<State>, u64, Vec<Event>)
where
    F: FnOnce(Registry) -> Box<dyn StepIntegrator>,
{
    let charge = params::charge();
    let sd = if leaky {
        Some(params::self_discharge())
    } else {
        None
    };
    let (state, registry) = build_power(&charge, scenario, sd).expect("build");
    let resolver = power_resolver(&charge, scenario).expect("resolver");
    let integrator = make(registry);
    let steps = SELF_DISCHARGE_DAYS * scenario.steps_per_day;
    run_trajectory(
        integrator.as_ref(),
        state,
        &resolver,
        scenario.dt_seconds,
        steps,
    )
    .expect("run")
}

fn leaky_run(scenario: &PowerScenario) -> (Vec<State>, u64, Vec<Event>) {
    run(scenario, true, |r| Box::new(EulerIntegrator::new(r)))
}

fn soc(states: &[State]) -> Vec<f64> {
    states.iter().map(|s| s.stocks[BATTERY].amount).collect()
}

/// `BOUNDED_SOC_SCENARIO` with `battery0` offset — the second arm of every contrast here.
fn offset_by(delta: f64) -> PowerScenario {
    PowerScenario {
        battery0: BOUNDED_SOC_SCENARIO.battery0 + delta,
        ..BOUNDED_SOC_SCENARIO
    }
}

fn bits(state: &State) -> BTreeMap<&str, u64> {
    state
        .stocks
        .iter()
        .map(|(id, s)| (id.as_str(), s.amount.to_bits()))
        .collect()
}

// --- k·dt < 1 : the structural-positivity precondition --------------------------------
#[test]
fn self_discharge_step_factor_below_one() {
    // The donor-controlled draw self-limits only while k·dt < 1 (the herbivory rate·dt < 1
    // discipline). At the realistic rate this is ~3.6e-5 — deeply structural.
    let kdt = k_dt();
    assert!(0.0 < kdt && kdt < 1.0, "k·dt = {kdt}");
}

// --- THE keep-earning proof: the exact geometric contraction ---------------------------
#[test]
fn self_discharge_contracts_geometrically() {
    // Two runs differing ONLY in battery0 under identical forcing. The forced terms cancel
    // in the difference, so d_n = d_0·(1 − k·dt)^n exactly (to round-off) under Euler.
    let (a, _, _) = leaky_run(&BOUNDED_SOC_SCENARIO);
    let (b, _, _) = leaky_run(&offset_by(1.0e6));
    let (sa, sb) = (soc(&a), soc(&b));
    let d0 = sb[0] - sa[0];
    assert!((d0 - 1.0e6).abs() < 1e-6, "d0 = {d0}");
    let decay = 1.0 - k_dt();
    for n in 0..=(STEPS as usize) {
        let predicted = d0 * decay.powi(n as i32);
        let actual = sb[n] - sa[n];
        assert!(
            (actual - predicted).abs() <= 1e-9 * predicted.abs(),
            "n = {n}: {actual} != {predicted}"
        );
    }
    // And the difference genuinely SHRANK — a contraction, not a flat line.
    assert!((sb[STEPS as usize] - sa[STEPS as usize]).abs() < d0.abs());
}

#[test]
fn forced_only_difference_is_constant() {
    // The contrast that makes the contraction meaningful: with self-discharge OFF the two
    // forced flows have no restoring force, so the battery0 offset propagates undecayed —
    // d_n == d_0 for every n. Donor-control is exactly what turns this constant into a
    // geometric contraction.
    let (a, _, _) = run(&BOUNDED_SOC_SCENARIO, false, |r| {
        Box::new(EulerIntegrator::new(r))
    });
    let (b, _, _) = run(&offset_by(1.0e6), false, |r| {
        Box::new(EulerIntegrator::new(r))
    });
    let (sa, sb) = (soc(&a), soc(&b));
    for n in 0..=(STEPS as usize) {
        assert!(
            (sb[n] - sa[n] - 1.0e6).abs() <= 1e-3,
            "n = {n}: {}",
            sb[n] - sa[n]
        );
    }
}

// --- isolation: the leak is the sole driver of departure from the balanced baseline ----
#[test]
fn self_discharge_departs_the_balanced_baseline() {
    // The forced part is daily-balanced, so WITHOUT the leak the SOC returns to battery0
    // at each day boundary. WITH it the SOC decays monotonically below battery0 — the leak
    // isolated as the sole conservation-preserving drift.
    let (leaky, _, _) = leaky_run(&BOUNDED_SOC_SCENARIO);
    let (balanced, _, _) = run(&BOUNDED_SOC_SCENARIO, false, |r| {
        Box::new(EulerIntegrator::new(r))
    });
    let spd = BOUNDED_SOC_SCENARIO.steps_per_day as usize;
    let b0 = BOUNDED_SOC_SCENARIO.battery0;
    let days = SELF_DISCHARGE_DAYS as usize;
    for d in 0..=days {
        let amount = balanced[d * spd].stocks[BATTERY].amount;
        assert!((amount - b0).abs() <= 1e-6, "baseline day {d}: {amount}");
    }
    let day_soc: Vec<f64> = (0..=days)
        .map(|d| leaky[d * spd].stocks[BATTERY].amount)
        .collect();
    for pair in day_soc.windows(2) {
        assert!(pair[0] > pair[1], "{:?} !> {:?}", pair[0], pair[1]);
    }
    // It "bit": the departure is well above round-off.
    assert!(b0 - day_soc[days] > 1.0);
}

// --- the closure payload is preserved (the leak is a balanced 2-leg transfer) ----------
#[test]
fn self_discharge_energy_conserved_every_step() {
    // Adding a donor-controlled flow does not break energy closure: the leak's
    // −leak + leak = 0 keeps the augmented ENERGY ledger balanced every step.
    let (states, _, _) = leaky_run(&BOUNDED_SOC_SCENARIO);
    for pair in states.windows(2) {
        let ledger = compute_ledger(&pair[0], &pair[1]).expect("ledger");
        let energy = ledger
            .iter()
            .find(|q| q.quantity == Quantity::Energy)
            .expect("ENERGY is present");
        assert!(energy.residual.abs() <= 1e-6, "{:?}", energy.residual);
    }
}

#[test]
fn self_discharge_energy_total_is_invariant() {
    // Integral form: the total never leaves battery0 — the leak moves joules from battery
    // to heat, it does not destroy them.
    let (states, _, _) = leaky_run(&BOUNDED_SOC_SCENARIO);
    for s in &states {
        let total =
            s.stocks[SOLAR_SOURCE].amount + s.stocks[BATTERY].amount + s.stocks[WASTE_HEAT].amount;
        assert!(
            (total - BOUNDED_SOC_SCENARIO.battery0).abs() <= 1e-4,
            "{total}"
        );
    }
}

#[test]
fn self_discharge_waste_heat_monotonic() {
    // waste_heat now also receives the leak, and still only ever receives.
    let (states, _, _) = leaky_run(&BOUNDED_SOC_SCENARIO);
    let heat: Vec<f64> = states.iter().map(|s| s.stocks[WASTE_HEAT].amount).collect();
    for pair in heat.windows(2) {
        assert!(pair[0] <= pair[1]);
    }
    assert!(heat[heat.len() - 1] > heat[0]);
    assert!(heat[0] > -1.0);
}

// --- rationed == 0 / no events ---------------------------------------------------------
#[test]
fn self_discharge_never_rations() {
    // The battery stays well-fed (the leak decays it ~1 % over the horizon), so the Euler
    // backstop never fires. ⚠ This leans on LoadDraw's well-fed *sizing*: the
    // self-discharge LEG is separately structural (k·dt < 1), but that is not what keeps
    // the run unrationed here, and the distinction is worth keeping visible.
    let (_, rationed, _) = leaky_run(&BOUNDED_SOC_SCENARIO);
    assert_eq!(rationed, 0);
}

#[test]
fn self_discharge_no_events() {
    let (_, _, events) = leaky_run(&BOUNDED_SOC_SCENARIO);
    assert!(events.is_empty(), "{events:?}");
}

// --- the broken bit-identity: RK4 agrees with Euler only to TOLERANCE now --------------
#[test]
fn self_discharge_breaks_rk4_euler_bit_identity() {
    // The state-dependent leak means the RK4 stage derivatives are no longer identical
    // (k1 ≠ k2), so RK4 ≢ Euler bit-for-bit — unlike the forced-only run in
    // `power_run.rs`, whose bit-identity must stay. They agree to O(dt²); the tiny leak
    // makes that gap small but nonzero.
    let (e, _, _) = leaky_run(&BOUNDED_SOC_SCENARIO);
    let (r, _, _) = run(&BOUNDED_SOC_SCENARIO, true, |reg| {
        Box::new(Rk4Integrator::new(reg))
    });
    let e_final = e[e.len() - 1].stocks[BATTERY].amount;
    let r_final = r[r.len() - 1].stocks[BATTERY].amount;
    assert_ne!(
        r_final.to_bits(),
        e_final.to_bits(),
        "the identity did not break"
    );
    assert!(
        (r_final - e_final).abs() <= 1e-5 * e_final.abs(),
        "{r_final} vs {e_final}"
    );
}

// --- determinism ------------------------------------------------------------------------
#[test]
fn self_discharge_is_deterministic() {
    let (a, ra, ea) = leaky_run(&BOUNDED_SOC_SCENARIO);
    let (b, rb, eb) = leaky_run(&BOUNDED_SOC_SCENARIO);
    assert_eq!(bits(&b[b.len() - 1]), bits(&a[a.len() - 1]));
    assert_eq!((rb, eb.len()), (ra, ea.len()));
}
