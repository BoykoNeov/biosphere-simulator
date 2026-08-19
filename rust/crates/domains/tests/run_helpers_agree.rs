//! `run_trajectory` and `run` agree — the claim S3 introduced and nothing else asserts.
//!
//! # ⚠⚠ Why this file exists
//!
//! Stage-3 slice S3 added [`run_trajectory`] *additively*, deliberately leaving
//! [`domains::run`] untouched because `goldens.rs` calls it and `goldens.rs` writes the
//! frozen bytes. That decision was justified by predicting a zero golden diff, and
//! `tests/golden_regression.rs` confirmed it — but **that check only ever exercises
//! `run`**. Meanwhile all 68 sibling run-level tests go through `run_trajectory`.
//!
//! So the tree ended up with two step loops, one certified by the frozen bytes and one
//! carrying the behavioural coverage, and **nothing tying them together**. Today they
//! agree. The exposure is forward: an edit to either silently desynchronizes them, the
//! goldens stay green, and 68 tests certify a path the frozen bytes do not cover. That is
//! the slice's own subject — a gate measuring a different subject than the one it is read
//! as covering — one level up from where §5w diagnoses it.
//!
//! Found on review, not by a test, which is the §5u pattern again.
//!
//! Euler only: `run` takes `&EulerIntegrator` concretely, so there is no second arm to
//! compare. The four scenarios are the frozen sibling ones, so this is the same work the
//! goldens do, asked of the other loop.

use domains::crew::{build_crew, crew_resolver, MISSION_DAYS, MISSION_SCENARIO};
use domains::eclss::{build_eclss, eclss_resolver, STEADY_STATE_SCENARIO, STEADY_STATE_STEPS};
use domains::params;
use domains::power::{
    build_power, power_resolver, BOUNDED_SOC_DAYS, BOUNDED_SOC_SCENARIO, SELF_DISCHARGE_DAYS,
};
use domains::thermal::{
    build_thermal, thermal_resolver, EQUILIBRIUM_SCENARIO, EQUILIBRIUM_STEPS,
};
use domains::{run, run_trajectory};
use simcore::error::SimError;
use simcore::integrator::EulerIntegrator;
use simcore::registry::Registry;
use simcore::state::State;

/// Run `steps` steps of the same system through both helpers and assert they agree.
///
/// `build` is called twice on purpose — `State` and `Registry` are consumed by a run, and
/// building afresh is also what makes this a comparison of two *runs* rather than of one
/// run against a copy of itself.
fn both_agree<B>(what: &str, build: B, dt: f64, steps: u64)
where
    B: Fn() -> Result<((State, Registry), simcore::environment::SourceResolver), SimError>,
{
    let ((state_a, reg_a), resolver_a) = build().expect("build a");
    let (final_a, rationed_a, events_a) =
        run(&EulerIntegrator::new(reg_a), state_a, &resolver_a, dt, steps).expect("run");

    let ((state_b, reg_b), resolver_b) = build().expect("build b");
    let (trajectory, rationed_b, events_b) = run_trajectory(
        &EulerIntegrator::new(reg_b),
        state_b,
        &resolver_b,
        dt,
        steps,
    )
    .expect("run_trajectory");

    assert_eq!(
        trajectory.len() as u64,
        steps + 1,
        "{what}: the trajectory must include the initial state"
    );
    let final_b = &trajectory[trajectory.len() - 1];
    assert_eq!(
        final_a.n, final_b.n,
        "{what}: the two loops disagree on the step count"
    );
    for (id, a) in &final_a.stocks {
        let b = final_b
            .stocks
            .get(id)
            .unwrap_or_else(|| panic!("{what}: {id} is missing from the trajectory's final state"));
        assert_eq!(
            a.amount.to_bits(),
            b.amount.to_bits(),
            "{what}: {id} differs between `run` ({:?}) and `run_trajectory` ({:?})",
            a.amount,
            b.amount
        );
    }
    // The whole `State`, not only the amounts — `n`, `rng_seed` and `aux` included.
    assert_eq!(&final_a, final_b, "{what}: the final States differ");
    assert_eq!(
        (rationed_a, events_a.len()),
        (rationed_b, events_b.len()),
        "{what}: the two loops disagree on the diagnostics"
    );
}

#[test]
fn power_agrees_between_run_and_run_trajectory() {
    both_agree(
        "power/bounded_soc",
        || {
            let charge = params::charge();
            Ok((
                build_power(&charge, &BOUNDED_SOC_SCENARIO, None)?,
                power_resolver(&charge, &BOUNDED_SOC_SCENARIO)?,
            ))
        },
        BOUNDED_SOC_SCENARIO.dt_seconds,
        BOUNDED_SOC_DAYS * BOUNDED_SOC_SCENARIO.steps_per_day,
    );
}

#[test]
fn power_self_discharge_agrees_between_run_and_run_trajectory() {
    // The donor-controlled arm: the only sibling scenario whose flows read a stock AND
    // whose golden is a separate file, so it is the one where a desynchronized loop could
    // most plausibly diverge without the forced arms noticing.
    both_agree(
        "power/self_discharge",
        || {
            let charge = params::charge();
            Ok((
                build_power(&charge, &BOUNDED_SOC_SCENARIO, Some(params::self_discharge()))?,
                power_resolver(&charge, &BOUNDED_SOC_SCENARIO)?,
            ))
        },
        BOUNDED_SOC_SCENARIO.dt_seconds,
        SELF_DISCHARGE_DAYS * BOUNDED_SOC_SCENARIO.steps_per_day,
    );
}

#[test]
fn thermal_agrees_between_run_and_run_trajectory() {
    both_agree(
        "thermal/equilibrium",
        || {
            let p = params::thermal();
            Ok((
                build_thermal(&p, &EQUILIBRIUM_SCENARIO)?,
                thermal_resolver(&EQUILIBRIUM_SCENARIO)?,
            ))
        },
        EQUILIBRIUM_SCENARIO.dt_seconds,
        EQUILIBRIUM_STEPS,
    );
}

#[test]
fn eclss_agrees_between_run_and_run_trajectory() {
    both_agree(
        "eclss/steady_state",
        || {
            let p = params::eclss();
            Ok((
                build_eclss(&p, &STEADY_STATE_SCENARIO)?,
                eclss_resolver(&STEADY_STATE_SCENARIO)?,
            ))
        },
        STEADY_STATE_SCENARIO.dt_seconds,
        STEADY_STATE_STEPS,
    );
}

#[test]
fn crew_agrees_between_run_and_run_trajectory() {
    both_agree(
        "crew/mission",
        || {
            let p = params::crew();
            Ok((
                build_crew(&p, &MISSION_SCENARIO)?,
                crew_resolver(&MISSION_SCENARIO)?,
            ))
        },
        MISSION_SCENARIO.dt_seconds,
        MISSION_DAYS * MISSION_SCENARIO.steps_per_day,
    );
}
