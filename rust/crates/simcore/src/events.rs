//! Simulation events — the Rust port of `simcore.events`.
//!
//! Events are the diagnostic record of a *non-flow* state change the engine makes
//! during a step. Phase 0 has exactly one: extinction. Events are returned from the
//! integrator in a `StepReport` (never carried in `State`), so the core stays pure and
//! re-runnable.

use crate::ids::StockId;
use crate::quantities::Quantity;

/// A POPULATION stock went extinct: its amount fell below `extinction_threshold`
/// (and was not already exactly 0), so it was snapped to 0 and `residual` mass routed
/// to the quantity's numerical-loss sink so the ledger still balances.
///
/// * `n` — the step count of the *post-apply* state (the `n` the integrator produces).
/// * `stock` — the POPULATION stock that went extinct.
/// * `quantity` — its conserved quantity.
/// * `residual` — the snapped amount routed to the loss-sink (the pre-snap amount).
#[derive(Debug, Clone, PartialEq)]
pub struct ExtinctionEvent {
    pub n: u64,
    pub stock: StockId,
    pub quantity: Quantity,
    pub residual: f64,
}

/// The Phase-0 event union is a single type. Widen to an enum when more event kinds
/// appear; everything annotating `Event` (notably `StepReport.events`) broadens with it.
pub type Event = ExtinctionEvent;
