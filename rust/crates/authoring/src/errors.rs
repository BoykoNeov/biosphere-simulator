//! The authoring-boundary error type (Phase 9, Step 4b).
//!
//! Mirrors the Python `authoring.errors.AuthoringError`: every failure decidable from
//! the scenario file alone (a malformed YAML line, an unknown flow type, wiring that
//! does not match the flow type's fields, an unbalanced authored stoichiometry, a
//! reference to an undeclared param) surfaces as one [`AuthoringError`]. It carries a
//! human message; **the message text is NOT a parse-parity target** (Tier-0 is
//! accept→same-graph, reject→both-error — the Step-4a discipline, one level up to
//! files). Engine-side failures (a mis-wire that only surfaces at the every-step
//! conservation gate) stay [`simcore::error::SimError`], raised from inside the run.

use std::fmt;

use crate::expr_parser::ParseError;

/// A scenario-authoring failure decidable at parse/interpret time (never a runtime
/// conservation failure — that is [`simcore::error::SimError`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoringError {
    /// The human-readable message (not parity-pinned).
    pub message: String,
}

impl AuthoringError {
    /// Construct an [`AuthoringError`] from anything string-like.
    pub fn new(message: impl Into<String>) -> AuthoringError {
        AuthoringError {
            message: message.into(),
        }
    }
}

impl fmt::Display for AuthoringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AuthoringError {}

impl From<ParseError> for AuthoringError {
    /// A rate/template expression parse failure is an authoring failure (both ports
    /// reject; the message is not pinned).
    fn from(err: ParseError) -> AuthoringError {
        AuthoringError::new(err.to_string())
    }
}

impl From<simcore::error::SimError> for AuthoringError {
    /// A frozen-constructor validation failure (an impossible stock, an unknown
    /// quantity/kind) surfaces during interpretation as an authoring failure — the
    /// authored file asked for something the engine rejects. (Runtime conservation
    /// failures are raised from the *run*, not here, and stay `SimError`.)
    fn from(err: simcore::error::SimError) -> AuthoringError {
        AuthoringError::new(err.to_string())
    }
}
