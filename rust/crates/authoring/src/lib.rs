//! Native Rust port of the frozen Python `authoring` boundary (Phase 9, Step 4a).
//!
//! The Python `src/authoring` package is the declarative-scenario boundary layer:
//! schema → interpreter → run, plus the rate-expression parser. Step 4a ports **only
//! the parser** — the sole Tier-0 parse-parity surface — because the genuinely-new
//! cross-port surface of this phase (rate-grammar parse-parity + authored-flow
//! trajectory parity) is provable with no YAML dependency: the trajectory anchor builds
//! a [`simcore::expr::DeclarativeFlow`] directly from a parsed rate string.
//!
//! Runtime scenario-file parsing (the YAML/schema/interpreter, decision E) is Step
//! 4b/5, where the crate/hand-rolled YAML choice actually gets resolved; keeping it out
//! of here is why the crux needs no new dependency.
//!
//! - [`expr_parser`] — text → [`simcore::expr::Expr`] AST (recursive-descent, pinned
//!   precedence/associativity, the deferred grammar rejected as in Python).
//! - [`sexpr`] — the canonical S-expression renderer the parse-parity gate diffs.

pub mod expr_parser;
pub mod sexpr;

pub use expr_parser::{parse_rate_expr, ParseError};
pub use sexpr::render_sexpr;
