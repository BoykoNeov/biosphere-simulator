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
//!
//! Step 1 (P7.1) adds `rng` — the counter-based splitmix64 generator, the first
//! ported engine component and a pure-integer Tier-1 bit-exact target.
//!
//! Step 2 (P7.2) ports the **whole engine**: identifiers and quantities, the immutable
//! `State`/`Stock` primitives, flows + balance, the arbitration backstop, the
//! conservation ledger + every-step gate, the Euler/RK4 integrator (extinction, aux,
//! `substep`), the registry, the environment resolver, the boundary reservoirs,
//! events, aux processes, and the multi-rate driver. It is validated by a synthetic
//! transcendental-free scenario gated bit-exact (Tier 1) under Euler, RK4, and
//! multi-rate against a Python-generated trajectory (`tests/engine_vectors.rs`).
//!
//! `observation` (Phase-8 P8.2) revives the frozen `simcore.observe` surface — a
//! consumer-facing projection deferred through Phase 7 (no golden is an `Observation`,
//! and the cross-port gate compares `State` snapshots directly). It is display-only and
//! carries no aggregates; the derived Godot readouts (per-domain totals, temperature,
//! SOC) live one layer out in `station::display`, keeping this module the faithful port.

pub mod arbitration;
pub mod auxiliary;
pub mod boundary;
pub mod conservation;
pub mod environment;
pub mod error;
pub mod events;
pub mod flow;
pub mod hexfloat;
pub mod ids;
pub mod integrator;
pub mod json;
pub mod multirate;
pub mod observation;
pub mod quantities;
pub mod registry;
pub mod rng;
pub mod snapshot;
pub mod state;
