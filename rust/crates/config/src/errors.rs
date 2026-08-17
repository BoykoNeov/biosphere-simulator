//! The param-boundary error type (reference flip, slice C1).
//!
//! Mirrors Python `config.errors.ConfigError` / `UnitValidationError`: every failure
//! decidable from a param file alone — an unreadable file, a malformed YAML line, a
//! missing or unexpected key, a value that is not a number, a unit that is not the one
//! the quantity is declared in, a value outside its documented bound — surfaces as one
//! [`ConfigError`].
//!
//! As on the authoring boundary, **the message text is not a parity target.** What the
//! two ports must agree on is *accept vs reject*, not the wording.

use std::fmt;

/// A param-boundary failure: a file that cannot be read, parsed, bound or believed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError {
    /// The human-readable message (not parity-pinned).
    pub message: String,
}

impl ConfigError {
    /// Construct a [`ConfigError`] from anything string-like.
    pub fn new(message: impl Into<String>) -> ConfigError {
        ConfigError {
            message: message.into(),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ConfigError {}
