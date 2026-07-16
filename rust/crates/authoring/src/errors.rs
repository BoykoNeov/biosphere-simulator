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

/// What kind of authoring failure this is — the Rust stand-in for Python's *two* error
/// classes (`AuthoringError` / `RationedError`).
///
/// Python can express the split as separate types because `run_scenario` raises; Rust
/// returns one `Result`, and widening its error into an enum would churn 61 construction
/// sites for no behavioral gain. So the distinction lives here instead, and it is a real
/// one — see [`ErrorKind::Rationed`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// Decidable from the scenario file alone, before any step runs (the original and
    /// still overwhelmingly common case): unknown flow type, bad wiring, a duplicate id.
    Structural,
    /// The Euler arbitration backstop fired during the run — the author's `dt` is too
    /// large for some flow's frozen rate constant. **Not structural**: the same file at a
    /// smaller `dt` is fine, and this is only knowable by running. Mirrors Python
    /// `authoring.errors.RationedError`; see that class for why a conserving, completed,
    /// non-raising run can still have asphyxiated its crew.
    Rationed,
}

/// An authoring failure surfaced by the authoring boundary. Usually decidable at
/// parse/interpret time ([`ErrorKind::Structural`]); [`ErrorKind::Rationed`] is the one
/// runtime verdict this layer reaches on its own (engine-side conservation failures stay
/// [`simcore::error::SimError`] until `run_scenario` converts them at its boundary).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoringError {
    /// The human-readable message (not parity-pinned).
    pub message: String,
    /// Which failure surface this is. Match on it rather than sniffing `message` — the
    /// text is explicitly not a parity target and may be reworded freely.
    pub kind: ErrorKind,
}

impl AuthoringError {
    /// Construct a [`ErrorKind::Structural`] [`AuthoringError`] from anything
    /// string-like. (The default: every pre-existing call site means this one.)
    pub fn new(message: impl Into<String>) -> AuthoringError {
        AuthoringError {
            message: message.into(),
            kind: ErrorKind::Structural,
        }
    }

    /// Construct a [`ErrorKind::Rationed`] [`AuthoringError`] — the run completed but
    /// needed the backstop, so its `dt` is wrong.
    pub fn rationed(message: impl Into<String>) -> AuthoringError {
        AuthoringError {
            message: message.into(),
            kind: ErrorKind::Rationed,
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
