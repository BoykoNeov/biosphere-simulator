//! Native Rust port of the frozen Python `authoring` boundary (Phase 9).
//!
//! The Python `src/authoring` package was the declarative-scenario boundary layer:
//! schema → interpreter → run, plus the rate-expression parser. This crate was written to
//! mirror it, and since S6 deleted it this crate is the only one.
//!
//! ⚠ **History, not authority** (2026-10-19). This crate was ported from Python and its comments
//! still name the code it came from (`test_*.py` files, Python modules and classes). S6 deleted
//! all of it at `4f7168e` (2026-08-27); read it at `4f7168e^`. Since the reference flip
//! (2026-08-16) this crate is the reference: a comment saying it was "ported from" or "mirrors"
//! Python records where the code came from, not a copy it must keep matching.
//!
//! **Step 4a** ported the rate-grammar parser (the sole Tier-0 parse-parity surface) +
//! the canonical S-expr renderer, at **zero YAML dependency** — the parse-parity +
//! trajectory-parity crux is provable from rate *strings*, no file.
//!
//! **Step 4b** adds runtime scenario-**file** parsing (decision E, USER-CONFIRMED
//! hand-rolled over a vetted crate): the closed-subset YAML reader ([`yaml`]), the
//! schema ([`schema`]), the interpreter ([`interpreter`]) calling the frozen
//! constructors (+ Step-3 template boundary-eval, [`template`]), and the run harness
//! ([`run`]). File-level parse-parity is the byte-identity of the interpreted run vs the
//! frozen golden, plus the canonical structural [`graph_dump`]. **Parameter packs are
//! deferred in the Rust port** (see [`flow_registry`] / [`interpreter`]).
//!
//! Purity: this is a boundary crate; it depends only on `simcore` (the engine, the AST,
//! the hex-float codec) and `domains` (the frozen crew flows, the Option-C param
//! constants), exactly as the Python `authoring` package imports `simcore` + `domains`.

pub mod bed;
pub mod compose;
pub mod errors;
pub mod expr_parser;
pub mod flow_registry;
pub mod freeze_manifest;
pub mod graph_dump;
pub mod interpreter;
pub mod run;
pub mod schema;
pub mod sexpr;
pub mod surface;
pub mod template;

/// The closed-subset YAML reader.
///
/// ⚠ **Re-export, not a module here** (reference flip, slice C1): the reader moved to
/// the `config` crate because `domains` and `station` load param files too and may not
/// depend on `authoring` (that would invert the boundary layering). The path
/// `authoring::yaml::…` is unchanged, and there is deliberately **one** reader —
/// a second would be slice 5's lesson repeated (*a policy with two implementations has
/// one that is stale*).
pub use config::yaml;

pub use compose::apply_includes;
pub use errors::{AuthoringError, ErrorKind};
pub use expr_parser::{parse_rate_expr, render_rate_expr, ParseError};
pub use graph_dump::render_graph_dump;
pub use interpreter::{
    effective_step, interpret, interpret_allowing_unsafe_step, load_scenario,
    load_scenario_allowing_unsafe_step, BuiltScenario, RATE_CLASSES,
};
pub use run::{run_scenario, run_scenario_allowing_rationing, RunResult, SPLIT};
pub use schema::ScenarioSpec;
pub use sexpr::render_sexpr;
