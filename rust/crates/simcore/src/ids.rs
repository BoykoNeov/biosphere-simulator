//! Stable, canonical-sortable identifiers — the Rust port of `simcore.ids`.
//!
//! Python uses `NewType` wrappers over `str` (distinct under the checker, plain
//! strings at runtime). Rust has no zero-cost `NewType`, and the whole port relies
//! on ids being *canonically sortable* the same way Python sorts them — so the port
//! uses plain `String` type aliases and depends on the invariant that ids are
//! **ASCII**, where a Rust `String` (UTF-8 byte) ordering agrees with Python's `str`
//! ordering exactly (decision #15). The alias names document intent at call sites.
//!
//! `UnitLabel` is the canonical-unit *label* (`"mol"`, `"kg"`, `"J"`); the core
//! stores only the label, never a dimensioned quantity (dimensional validation lives
//! in the outer Python `config` loader, never ported — decision #9).

/// A stock identifier (e.g. `"biosphere.leaf_c"`).
pub type StockId = String;
/// A domain namespace identifier (e.g. `"boundary"`).
pub type DomainId = String;
/// A flow identifier (e.g. `"power.solar_charge"`).
pub type FlowId = String;
/// A canonical-unit label (`"mol"`, `"kg"`, `"J"`).
pub type UnitLabel = String;
/// An aux-process identifier (distinct from the accumulator *names* it writes).
pub type AuxId = String;
