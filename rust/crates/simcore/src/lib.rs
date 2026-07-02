//! Native Rust port of the frozen Python `simcore` (Phase 7, P7.0 onward).
//!
//! Step 0 (P7.0) lands only the cross-port *interchange*: a C99 hex-float codec
//! (`hexfloat`) and a serialize-only state snapshot emitter (`snapshot`) whose
//! output the Python `sim_io.loads` reads unchanged. The simulation engine, the
//! RNG, and the domain flows arrive in later steps; there is deliberately no
//! invariant logic here — the Python `State`/`Stock` constructors re-fire every
//! invariant on load, so a faithful byte stream is all the Rust side owes.
//!
//! Cross-port contract (see `docs/plans/phase-7-native-core.md`): compare *parsed
//! f64 values*, never JSON bytes. The Rust emitter need only produce valid JSON
//! that round-trips through Python `loads`; it is not required to match Python's
//! `json.dumps` string spelling byte-for-byte.

pub mod hexfloat;
pub mod snapshot;
