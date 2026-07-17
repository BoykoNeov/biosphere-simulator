//! Run an interpreted scenario (single-rate, no-reset) — the Rust mirror of Python
//! `authoring.run` (Phase 9, Step 4b).
//!
//! Mirrors the standalone `run_crew` / `run_power` drivers: step the requested
//! integrator `steps` times, returning the **final** state, the summed
//! arbitration-backstop firings, and any extinction events (the Rust `domains::run`
//! final-state discipline — the goldens pin the final `State`). **No reset hook** — the
//! master-day driver + phenology/annual-reset stay deferred.
//!
//! **Multi-rate (post-roadmap Step 6).** A scenario that declared a coupling cadence
//! (`BuiltScenario::is_multirate`) is driven by [`simcore::multirate::multirate_step`];
//! **everything else takes the pre-multi-rate loop over the whole `registry`, verbatim**.
//! Do not "simplify" that branch away: `n_sub=1` with an empty slow set reproduces the
//! single-rate trajectory bit-for-bit, so routing every scenario through the driver would
//! pass every test *today* while silently resting all 25 goldens on that identity holding
//! forever. Note this is a distinct axis from the **two-rate master-day** driver named
//! above — that is the biosphere's phenology cadence, not the rate-class partition.
//!
//! **The knob does not make a step safe**, and this harness is not where that is caught:
//! the build-time `k·h < 1` precondition (`crate::interpreter::check_rate_preconditions`)
//! is the direct closer. A `RationedError` here is the *donor-controlled* half; the
//! demand-controlled `eclss.o2_makeup` is invisible to the backstop at any `dt`. Neither
//! gate subsumes the other.
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
use simcore::integrator::{EulerIntegrator, Rk4Integrator, Substepper};
use simcore::multirate::{multirate_step, Split};
use simcore::state::State;

use crate::errors::AuthoringError;
use crate::interpreter::BuiltScenario;

/// The operator-splitting scheme, **pinned to Strang** — not author-visible.
///
/// Strang is the core's own default and carries the higher nominal order. **The
/// justification is order/safety, not performance**: Lie is actually *cheaper* on the slow
/// set (one slow evaluation per master step vs Strang's two — the missing factor of two
/// behind the multi-rate plan's measured 30× rather than the predicted 60×). Strang
/// additionally steps the slow set at `dt/2`, which is *safer* for the slow set's own
/// `k·dt`, and our Euler flows collapse Strang to 1st order anyway, so exposing the knob
/// would buy an author no order they can use. Lie is documented in `simcore` as
/// "fallback / comparison" — a **study** tool, not an authoring choice.
///
/// **`interpreter::SLOW_STEP_DIVISOR` (the precondition's `dt/2`) tracks this constant**,
/// and the coupling is asserted in this module's tests rather than left as a comment:
/// under Lie the slow set steps at the full `dt`, which would make that check too
/// permissive by exactly 2×, silently, in the unsafe direction.
///
/// **Public deliberately, though nothing outside this crate drives it.** It is exposed so
/// the coupling above can be *asserted from a test* rather than trusted: it is a fact
/// about the harness that the precondition depends on, not an implementation detail.
pub const SPLIT: Split = Split::Strang;

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
    let n_sub = built.n_sub;
    let multirate = built.is_multirate();
    let result = run_scenario_allowing_rationing(built)?;
    if result.total_rationed > 0 {
        return Err(AuthoringError::rationed(rationed_message(
            result.total_rationed,
            dt,
            steps,
            n_sub,
            multirate,
        )));
    }
    Ok(result)
}

/// The `ErrorKind::Rationed` text — **the remedy is conditional on the rate class**, and
/// that is not cosmetic.
///
/// "Increase `n_sub`" is honest on the multi-rate path and **wrong** on the single-rate
/// one: there is no `n_sub` to raise, and naming it sends an author hunting for a key
/// their scenario does not have. The multi-rate variant also reports the **effective
/// sub-step** `dt/n_sub` rather than the master `dt` — which is no longer the step any
/// flow was integrated at, so quoting it would name the one number that is *not* the
/// cause. It further warns that the slow set steps at `dt/2` **regardless of `n_sub`**
/// under Strang, so raising `n_sub` will not rescue a slow flow's over-draw.
fn rationed_message(rationed: u64, dt: f64, steps: u64, n_sub: u32, multirate: bool) -> String {
    let (where_, remedy) = if multirate {
        (
            format!(
                "an effective sub-step of dt/n_sub = {:?} (master dt={dt:?}, n_sub={n_sub})",
                dt / f64::from(n_sub)
            ),
            "Increase n_sub (the fast set then sub-steps more finely, leaving the master \
             export cadence untouched), or reduce dt. NOTE: a 'rate_class: slow' flow \
             steps at dt/2 regardless of n_sub under Strang splitting, so if a slow flow \
             is the one over-drawing, raising n_sub will NOT help it — re-class it fast, \
             or reduce dt."
                .to_string(),
        )
    } else {
        (
            format!("dt={dt:?}"),
            "Reduce dt and re-run; each flow type's constraint is tabulated under 'The dt \
             constraint' in docs/authoring-reference.md (e.g. ECLSS's frozen rates want dt \
             <= ~60 s)."
                .to_string(),
        )
    };
    format!(
        "the arbitration backstop fired {rationed} time(s) at {where_} over {steps} \
         step(s). On an authored graph this means the step is too large for some flow's \
         frozen rate constant: the over-draw was clamped at zero, so the run still \
         conserved every quantity and still finished — but a clamped stock is an emptied \
         one (a cabin with no oxygen conserves mass perfectly). {remedy} To inspect the \
         rationed run instead of failing, use run_scenario_allowing_rationing."
    )
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
    // The branch, not the identity, is what preserves the goldens. A declared cadence
    // goes to `multirate_step`; **everything else takes the pre-multi-rate loop over the
    // whole `registry`, verbatim**. That is not an optimization: `n_sub=1` with an empty
    // slow set reproduces the single-rate trajectory bit-for-bit, so routing everything
    // through the driver would also work — and would silently rest every golden on that
    // identity holding forever.
    if built.is_multirate() {
        check_no_aux(&built)?;
        return run_multirate(built);
    }
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

/// Refuse to drive an aux-bearing graph through `multirate_step` — the **aux tripwire**.
///
/// `step_report` advances `State.aux`; `multirate_step` deliberately **never** does
/// (decision P2: advancing it per sub-op would advance it `n_sub`×). So routing an
/// aux-bearing graph through the driver would freeze every accumulator **silently**, and
/// the conservation gate structurally cannot see it (aux is non-conserved by definition).
///
/// **It cannot fire from any authored file today** — the interpreter never wires
/// `aux_processes`, so the authoring layer cannot express aux at all. It is a tripwire for
/// the phase that makes the biosphere (the one aux-bearing domain) authorable. Living
/// here rather than in `interpret` is also what makes it *testable*: a hand-built
/// `BuiltScenario` can reach it, where in `interpret` it would be unreachable and
/// untestable both. It is an `AuthoringError` despite firing at run time because it is
/// decidable from the graph's structure alone, and is raised before any step runs.
///
/// **Not in `multirate_step`**: `simcore` is frozen and this is a *consumer*. **Not on
/// the single-rate path**: `step_report` handles aux correctly there, and refusing it
/// would ban a working shape.
fn check_no_aux(built: &BuiltScenario) -> Result<(), AuthoringError> {
    let aux = built.registry.aux_processes();
    if !aux.is_empty() {
        let names: Vec<&str> = aux.iter().map(|a| a.id()).collect();
        return Err(AuthoringError::new(format!(
            "scenario {:?} declares a multi-rate cadence (n_sub={}) but its graph carries \
             aux process(es) {names:?}. simcore::multirate::multirate_step never advances \
             State.aux (decision P2, 'Aux x multi-rate is out of scope'), so every \
             accumulator would silently freeze — and the conservation gate cannot see it, \
             because aux is non-conserved by definition. Run this scenario single-rate \
             (drop n_sub and any 'rate_class: slow'), where step_report advances aux \
             correctly.",
            built.name, built.n_sub
        )));
    }
    Ok(())
}

/// Drive [`multirate_step`] once per master step over the interpreter's partition (the
/// `_run_multirate` analogue).
///
/// Both sub-integrators are built from the scenario's single `integrator`:
/// `multirate_step` would accept `slow=rk4, fast=euler`, but a per-rate-class integrator
/// is **deferred by name**. `multirate_step` aggregates `rationed` across its sub-ops, so
/// the count this returns is directly comparable to the single-rate path's.
fn run_multirate(built: BuiltScenario) -> Result<RunResult, AuthoringError> {
    let steps = built.steps;
    let dt = built.dt;
    let n_sub = built.n_sub;
    let resolver = built.resolver;
    let mut state = built.state;

    let (slow, fast): (Box<dyn Substepper>, Box<dyn Substepper>) = match built.integrator.as_str() {
        "euler" => (
            Box::new(EulerIntegrator::new(built.slow_registry)),
            Box::new(EulerIntegrator::new(built.fast_registry)),
        ),
        "rk4" => (
            Box::new(Rk4Integrator::new(built.slow_registry)),
            Box::new(Rk4Integrator::new(built.fast_registry)),
        ),
        other => {
            return Err(AuthoringError::new(format!(
                "unknown integrator {other:?} (known: [\"euler\", \"rk4\"])"
            )))
        }
    };

    let mut total_rationed = 0u64;
    let mut events: Vec<Event> = Vec::new();
    for _ in 0..steps {
        let report = multirate_step(
            slow.as_ref(),
            fast.as_ref(),
            &state,
            &resolver,
            dt,
            n_sub,
            SPLIT,
        )
        .map_err(AuthoringError::from)?;
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
