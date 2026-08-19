//! Native Rust port of the frozen Python Phase-5 siblings (Phase 7, P7.3).
//!
//! The four standalone domains — [`power`], [`thermal`], [`eclss`], [`crew`] — each
//! port their Python twin's stocks, flows, scenario, and system-builder onto the
//! [`simcore`] engine. There is **no new science**: every flow's arithmetic mirrors
//! the Python `evaluate` character-for-character (float `+`/`*` are not associative,
//! so op-order is load-bearing for the bit-exact Tier-1 gate on crew/eclss), and the
//! coefficients arrive from the frozen Python loaders via [`params`] (a generated
//! hex-float file, not a re-parsed YAML).
//!
//! Cross-port tiers (see `docs/plans/phase-7-native-core.md` / `tests/crossport/`):
//! **crew** and **eclss** are transcendental-free ⇒ Tier-1 bit-exact; **power** (the
//! half-sine solar schedule, `sin`) and **thermal** (the `T⁴` Stefan-Boltzmann
//! radiator, `powf`) are Tier-2, validated against a *measured* relative band. The
//! Tier-0 structural invariants (`rationed == 0`, `events == ()`, conservation every
//! step) are asserted in Rust by [`run`] + the emit examples (conservation is enforced
//! inside `step_report`, so a completed run is itself the proof).

pub mod biosphere;
pub mod crew;
pub mod eclss;
pub mod freeze_manifest;
pub mod goldens;
pub mod params;
pub mod power;
pub mod thermal;

use simcore::environment::SourceResolver;
use simcore::error::SimError;
use simcore::events::Event;
use simcore::integrator::{EulerIntegrator, Rk4Integrator, StepReport};
use simcore::state::State;

/// Step `steps` times under Euler, returning `(final_state, total_rationed, events)`.
///
/// The shared `run_power` / `run_thermal` / `run_eclss` / `run_crew` analogue (none of
/// the four siblings has a reset hook). Unlike the Python `run_*` it keeps only the
/// **final** state — the goldens pin the final `State`, and the intra-run Tier-0
/// invariants surface through the returned `total_rationed` / `events` (the emit
/// examples assert both are zero/empty). The every-step conservation gate runs inside
/// [`EulerIntegrator::step_report`], so a completed run proves the ledger balanced
/// every step (the "conservation every step in Rust" Tier-0 leg).
pub fn run(
    integrator: &EulerIntegrator,
    initial: State,
    resolver: &SourceResolver,
    dt: f64,
    steps: u64,
) -> Result<(State, u64, Vec<Event>), SimError> {
    let mut state = initial;
    let mut total_rationed = 0u64;
    let mut events: Vec<Event> = Vec::new();
    for _ in 0..steps {
        let report = integrator.step_report(&state, resolver, dt)?;
        state = report.state;
        total_rationed += report.rationed;
        events.extend(report.events);
    }
    Ok((state, total_rationed, events))
}

/// One full step with diagnostics — the one thing the two integrators share that no
/// public `simcore` trait carries.
///
/// # Why this trait is here rather than in `simcore`
///
/// `step_report` is an **inherent** method on `EulerIntegrator` and `Rk4Integrator`, and
/// `simcore`'s own `Scheme` trait — which does abstract over them — is private. The one
/// public trait over both, `Substepper`, is not a substitute: by its own documentation it
/// keeps `n`, skips aux, and **does not assert conservation**, which is exactly the
/// property the run-level sibling gates check.
///
/// So the choice (Stage-3 slice S3, `docs/plans/post-roadmap-reference-flip.md` §5v) was
/// between making `Scheme` public and declaring a two-impl trait in the crate that needs
/// it. Widening a frozen `simcore` API to serve test ergonomics is unfreeze-adjacent and
/// is not what S3 is for, so the trait lives here.
pub trait StepIntegrator {
    /// Advance one step, returning the new state plus the rationing count and events.
    fn step_report(
        &self,
        state: &State,
        env: &SourceResolver,
        dt: f64,
    ) -> Result<StepReport, SimError>;
}

impl StepIntegrator for EulerIntegrator {
    fn step_report(
        &self,
        state: &State,
        env: &SourceResolver,
        dt: f64,
    ) -> Result<StepReport, SimError> {
        EulerIntegrator::step_report(self, state, env, dt)
    }
}

impl StepIntegrator for Rk4Integrator {
    fn step_report(
        &self,
        state: &State,
        env: &SourceResolver,
        dt: f64,
    ) -> Result<StepReport, SimError> {
        Rk4Integrator::step_report(self, state, env, dt)
    }
}

/// Step `steps` times, keeping **every** state — `(trajectory, total_rationed, events)`.
///
/// The Python `run_power` / `run_thermal` / `run_eclss` / `run_crew` analogue in full:
/// `trajectory` includes the initial state, so its length is `steps + 1`. [`run`] keeps
/// only the final state because that is all the goldens pin; the sibling validation asks
/// trajectory-shaped questions instead — conserved *every step*, sinks monotonic, SOC
/// returning at each day boundary, two runs contracting geometrically — and none of them
/// is answerable from a final state.
///
/// ⚠ **Additive, and deliberately not a change to [`run`].** `goldens.rs` calls `run`, and
/// `goldens.rs` produces the frozen golden bytes; re-expressing `run` in terms of this
/// function would put an allocation and a different accumulation on the path that writes
/// them. The predicted golden diff for S3 is **zero**, and `tests/golden_regression.rs` is
/// what proves it rather than this comment asserting it.
///
/// Generic over [`StepIntegrator`] because several of the sibling gates run the *same*
/// scenario under both schemes — Power and Crew to assert RK4 ≡ Euler bit-for-bit (every
/// flow forced ⇒ `k1 = k2 = k3 = k4`), Thermal and ECLSS to assert the opposite, that the
/// identity is *broken* once a flow reads a stock.
pub fn run_trajectory<I: StepIntegrator + ?Sized>(
    integrator: &I,
    initial: State,
    resolver: &SourceResolver,
    dt: f64,
    steps: u64,
) -> Result<(Vec<State>, u64, Vec<Event>), SimError> {
    let mut trajectory: Vec<State> = Vec::with_capacity(steps as usize + 1);
    let mut state = initial;
    let mut total_rationed = 0u64;
    let mut events: Vec<Event> = Vec::new();
    trajectory.push(state.clone());
    for _ in 0..steps {
        let report = integrator.step_report(&state, resolver, dt)?;
        state = report.state;
        total_rationed += report.rationed;
        events.extend(report.events);
        trajectory.push(state.clone());
    }
    Ok((trajectory, total_rationed, events))
}
