//! The engine error type — the Rust port of Python's four distinct raise sites.
//!
//! Python `simcore` raises four kinds of failure, each with a deliberately
//! separate type so callers (and the every-step gate) can match the *specific*
//! failure. This enum carries the same four, one variant each:
//!
//! * [`SimError::Conservation`] — `flow.ConservationError`: a flow or the every-step
//!   ledger failed per-quantity balance within tolerance. An engine bug, not a
//!   recoverable condition, but a *catchable* one (the imbalanced-flow gate).
//! * [`SimError::Arbitration`] — `arbitration.ArbitrationError`: a flow needs
//!   `scale_f < 1` under RK4+, where min-scaling does not apply — positivity must
//!   come from the kinetics.
//! * [`SimError::Validation`] — Python's `ValueError`: a construction invariant
//!   (non-finite amount, an unclamped non-BOUNDARY stock, a bad composition, …) or a
//!   structural precondition (`n_sub >= 1`, disjoint forcing/shared wiring, a
//!   before/after stock-id-set mismatch in the ledger).
//! * [`SimError::Reference`] — Python's `KeyError`: referential integrity — a leg or
//!   env var or extinction loss-sink names a stock/var absent from the state/wiring.
//!
//! Keeping them distinct lets the Rust cross-port tests assert *which* failure fired,
//! exactly as the Python tests match the exception type.

/// A simulation engine error. See the module docs for the Python correspondence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimError {
    /// Per-quantity balance failed (Python `ConservationError`).
    Conservation(String),
    /// A needed `scale_f < 1` under RK4+ (Python `ArbitrationError`).
    Arbitration(String),
    /// A construction/structural invariant was violated (Python `ValueError`).
    Validation(String),
    /// Referential integrity: an unknown stock/var/loss-sink (Python `KeyError`).
    Reference(String),
}

impl std::fmt::Display for SimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimError::Conservation(m) => write!(f, "ConservationError: {m}"),
            SimError::Arbitration(m) => write!(f, "ArbitrationError: {m}"),
            SimError::Validation(m) => write!(f, "ValueError: {m}"),
            SimError::Reference(m) => write!(f, "KeyError: {m}"),
        }
    }
}

impl std::error::Error for SimError {}
