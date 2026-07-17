//! Native Rust port of the frozen Python `authoring` boundary (Phase 9).
//!
//! The Python `src/authoring` package is the declarative-scenario boundary layer:
//! schema → interpreter → run, plus the rate-expression parser. This crate mirrors it.
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

pub mod compose;
pub mod errors;
pub mod expr_parser;
pub mod flow_registry;
pub mod graph_dump;
pub mod interpreter;
pub mod run;
pub mod schema;
pub mod sexpr;
pub mod template;
pub mod yaml;

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
