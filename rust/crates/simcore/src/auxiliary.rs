//! The non-conserved auxiliary channel — the Rust port of `simcore.auxiliary`.
//!
//! An [`AuxProcess`] is the parallel of a `Flow`: it evaluates against the step-entry
//! snapshot and returns per-accumulator-name increments in the same increment form
//! (`dt·rate`). But unlike a flow it is single-valued and **un-balanced** — there is
//! no conserved counterparty, so there is no balance check, and the integrator advances
//! aux **outside** the conservation gate.
//!
//! Numerics: aux is advanced by **one explicit-Euler evaluation at the step-entry
//! snapshot**, independent of the stock scheme, and is **never sub-staged through
//! RK4** (RK4 stage states keep aux, only stock amounts perturb) — so a flow that
//! *reads* aux sees a within-step constant.

use std::collections::BTreeMap;

use crate::environment::Environment;
use crate::error::SimError;
use crate::state::State;

/// A pure, deterministic rate for one or more non-conserved accumulators.
///
/// `evaluate` returns a map from accumulator **name** to its per-step increment
/// (`dt·rate`). There is no balance check — aux is non-conserved by definition. `id`
/// identifies the *process* (for dedup and canonical iteration order), not an
/// accumulator name (a process may write several names, and several processes may
/// write one shared name — the integrator sums those contributions).
pub trait AuxProcess {
    /// The canonical process id (ASCII — its sort order is the reduction order).
    fn id(&self) -> &str;

    /// Evaluate against `snapshot`/`env`, returning per-name increments.
    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<BTreeMap<String, f64>, SimError>;
}

/// A convenience read of an aux-process id (mirrors [`crate::flow::flow_id`]). The id
/// is a [`crate::ids::AuxId`] at the type level (a plain `String` alias).
pub fn aux_id(proc: &dyn AuxProcess) -> &str {
    proc.id()
}
