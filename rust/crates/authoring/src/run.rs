//! Run an interpreted scenario (single-rate, no-reset) — the Rust mirror of Python
//! `authoring.run` (Phase 9, Step 4b).
//!
//! Mirrors the standalone `run_crew` / `run_power` drivers: step the requested
//! integrator `steps` times, returning the **final** state, the summed
//! arbitration-backstop firings, and any extinction events (the Rust `domains::run`
//! final-state discipline — the goldens pin the final `State`). **No reset hook** and
//! **single-rate only** — the two-rate master-day driver + phenology/annual-reset are a
//! later Phase-9 step, exactly as the Rust side layered them after the single-rate
//! session.
//!
//! The every-step conservation gate runs inside `integrator.step_report`, so a
//! completed run is itself proof the authored graph balanced every step — an authored
//! mis-wiring surfaces here as a [`SimError::Conservation`] (the safety property).
//!
//! **Rationing is a hard error here (post-roadmap).** Conservation is *not* sufficient:
//! the `dt` hazard produces a run that balances every step and still asphyxiates the
//! cabin, because the backstop clamps the over-draw at zero rather than going negative.
//! So this harness gates on `total_rationed == 0` too — the second half of "the authored
//! graph is sound". See Python `authoring.errors.RationedError`, the reference for this.

use simcore::error::SimError;
use simcore::events::Event;
use simcore::integrator::{EulerIntegrator, Rk4Integrator};
use simcore::state::State;

use crate::errors::AuthoringError;
use crate::interpreter::BuiltScenario;

/// The result of a run: the final `State`, the total arbitration-backstop firings, and
/// any extinction events.
pub struct RunResult {
    pub final_state: State,
    pub total_rationed: u64,
    pub events: Vec<Event>,
}

/// Step `built` to completion under its requested integrator. An unknown integrator
/// name is an [`AuthoringError`]; a conservation failure (a mis-wire) is the
/// [`SimError`] the every-step gate raises.
///
/// **Rationing is a hard error** ([`crate::errors::ErrorKind::Rationed`]): if the Euler backstop fired
/// at all, the authored `dt` is too large for some flow's frozen rate constant, and the
/// trajectory is not returned. Mirrors Python `authoring.run.run_scenario`'s default.
/// Use [`run_scenario_allowing_rationing`] to inspect such a run instead of failing.
pub fn run_scenario(built: BuiltScenario) -> Result<RunResult, AuthoringError> {
    let dt = built.dt;
    let steps = built.steps;
    let result = run_scenario_allowing_rationing(built)?;
    if result.total_rationed > 0 {
        return Err(AuthoringError::rationed(format!(
            "the arbitration backstop fired {} time(s) at dt={dt:?} over {steps} \
             step(s). On an authored graph this means dt is too large for some flow's \
             frozen rate constant: the over-draw was clamped at zero, so the run still \
             conserved every quantity and still finished — but a clamped stock is an \
             emptied one (a cabin with no oxygen conserves mass perfectly). Reduce dt \
             (ECLSS's frozen rates want dt <= ~60 s) and re-run; see 'The dt \
             constraint' in docs/authoring-reference.md. To inspect the rationed run \
             instead of failing, use run_scenario_allowing_rationing.",
            result.total_rationed
        )));
    }
    Ok(result)
}

/// [`run_scenario`] without the rationing gate — returns the trajectory even if the
/// backstop fired. The Rust stand-in for Python's `allow_rationing=True` kwarg (Rust has
/// no default arguments, and widening `run_scenario`'s signature would churn every
/// caller for a flag that should be rare).
///
/// For **deliberately studying** a rationed run, not for making a scenario "work": a
/// rationed authored run is one whose `dt` is wrong, and the stocks it clamped at zero
/// are stocks it emptied.
pub fn run_scenario_allowing_rationing(
    built: BuiltScenario,
) -> Result<RunResult, AuthoringError> {
    let steps = built.steps;
    let dt = built.dt;
    let resolver = built.resolver;
    let state = built.state;
    let result = match built.integrator.as_str() {
        "euler" => step_euler(EulerIntegrator::new(built.registry), state, &resolver, dt, steps),
        "rk4" => step_rk4(Rk4Integrator::new(built.registry), state, &resolver, dt, steps),
        other => {
            return Err(AuthoringError::new(format!(
                "unknown integrator {other:?} (known: [\"euler\", \"rk4\"])"
            )))
        }
    };
    result.map_err(AuthoringError::from)
}

fn step_euler(
    integrator: EulerIntegrator,
    initial: State,
    resolver: &simcore::environment::SourceResolver,
    dt: f64,
    steps: u64,
) -> Result<RunResult, SimError> {
    let mut state = initial;
    let mut total_rationed = 0u64;
    let mut events: Vec<Event> = Vec::new();
    for _ in 0..steps {
        let report = integrator.step_report(&state, resolver, dt)?;
        state = report.state;
        total_rationed += report.rationed;
        events.extend(report.events);
    }
    Ok(RunResult {
        final_state: state,
        total_rationed,
        events,
    })
}

fn step_rk4(
    integrator: Rk4Integrator,
    initial: State,
    resolver: &simcore::environment::SourceResolver,
    dt: f64,
    steps: u64,
) -> Result<RunResult, SimError> {
    let mut state = initial;
    let mut total_rationed = 0u64;
    let mut events: Vec<Event> = Vec::new();
    for _ in 0..steps {
        let report = integrator.step_report(&state, resolver, dt)?;
        state = report.state;
        total_rationed += report.rationed;
        events.extend(report.events);
    }
    Ok(RunResult {
        final_state: state,
        total_rationed,
        events,
    })
}
