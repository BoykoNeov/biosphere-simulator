//! Multi-rate sub-stepping: operator splitting over disjoint flow sets — the Rust port
//! of `simcore.multirate`.
//!
//! One master step of size `dt` advances the integer clock by exactly 1; a *fast* flow
//! set sub-steps `n_sub` times inside that master step while a *slow* flow set steps at
//! the master rate. `slow`/`fast` are two integrators over disjoint flow registries
//! that share one stock dict (the driver takes the two pre-built substeppers; it does
//! not infer the partition). Every sub-operation is an amounts-only `substep` (keeps
//! `n`); the single `n -> n+1` commit and the composite conservation gate are owned
//! here — sub-steps skip the per-operation assert, so an unbalanced sub-delta trips
//! this boundary gate.
//!
//! Aux is deliberately **not advanced** across a master step (`substep` leaves it
//! untouched; advancing it per sub-op would advance it `n_sub`× — wrong).

use crate::conservation::assert_conserved_default;
use crate::environment::SourceResolver;
use crate::error::SimError;
use crate::events::Event;
use crate::integrator::{StepReport, Substepper};
use crate::state::State;

/// The operator-splitting scheme for one master step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Split {
    /// Symmetric `slow(dt/2)` → `fast` → `slow(dt/2)`; 2nd-order iff both operators are
    /// RK4 (the default, gated path).
    Strang,
    /// `slow(dt)` → `fast`; 1st-order regardless of sub-integrator (fallback/comparison).
    Lie,
}

/// One master multi-rate step: advance `State.n` by exactly 1.
///
/// The fast set sub-steps `n_sub` times at `dt/n_sub`; the slow set steps at the master
/// rate (split into two `dt/2` halves under Strang). `n_sub` must be `>= 1`
/// ([`SimError::Validation`] otherwise). With `n_sub == 1` and an empty slow registry,
/// a Strang master step reproduces the single-rate `step` bit-for-bit.
pub fn multirate_step(
    slow: &dyn Substepper,
    fast: &dyn Substepper,
    state: &State,
    env: &SourceResolver,
    dt: f64,
    n_sub: u32,
    split: Split,
) -> Result<StepReport, SimError> {
    if n_sub < 1 {
        return Err(SimError::Validation(format!("n_sub must be >= 1, got {n_sub}")));
    }

    // A flat ordered op-list — the splitting sequence made self-documenting. Each entry
    // is one amounts-only sub-operation at its own step size. `dt / n_sub` mirrors
    // Python's int→float promotion.
    let fast_h = dt / (n_sub as f64);
    let mut ops: Vec<(&dyn Substepper, f64)> = Vec::new();
    match split {
        Split::Strang => {
            ops.push((slow, dt / 2.0));
            for _ in 0..n_sub {
                ops.push((fast, fast_h));
            }
            ops.push((slow, dt / 2.0));
        }
        Split::Lie => {
            ops.push((slow, dt));
            for _ in 0..n_sub {
                ops.push((fast, fast_h));
            }
        }
    }

    let before = state;
    let mut events: Vec<Event> = Vec::new();
    let mut rationed: u64 = 0;
    let mut cur = state.clone();
    for (stepper, h) in ops {
        let report = stepper.substep(&cur, env, h)?;
        cur = report.state;
        events.extend(report.events);
        rationed += report.rationed;
    }

    // The single master-step commit: n -> n+1 once, over the post-split amounts.
    let committed = State::new(before.n + 1, cur.stocks, cur.rng_seed, cur.aux)?;
    // The composite conservation gate, asserted once here over the whole master step
    // (sub-steps skipped it — this is the load-bearing tripwire).
    assert_conserved_default(before, &committed)?;
    // Re-stamp events to the produced state's n (sub-steps keep n at before.n).
    let stamped: Vec<Event> = events
        .into_iter()
        .map(|mut e| {
            e.n = committed.n;
            e
        })
        .collect();
    Ok(StepReport {
        state: committed,
        events: stamped,
        rationed,
    })
}
