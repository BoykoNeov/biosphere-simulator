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

pub mod crew;
pub mod eclss;
pub mod params;
pub mod power;
pub mod thermal;

use simcore::environment::SourceResolver;
use simcore::error::SimError;
use simcore::events::Event;
use simcore::integrator::EulerIntegrator;
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
