//! Canonical S-expression rendering of a rate-expression AST — the **parse-parity
//! diff mechanism** (Phase 9, Step 4a).
//!
//! Parse-parity compares the AST a rate string lowers to in each port. Rather than
//! reflect over private node fields, both ports serialize the AST to one canonical
//! text form and the cross-port harness diffs *strings* — the engine-vectors
//! discipline (a committed vector file of `input <TAB> expected_sexpr`, generated
//! Python-side, re-derived and asserted Rust-side).
//!
//! The format is unambiguous and captures structure, associativity (nesting), and
//! *exact* constants:
//!
//! ```text
//! (const <float.hex>)      # bit-exact literal (via the hex-float codec)
//! (stock "power.battery")  # a StockRef by id
//! (param "k")              # a ParamRef by name
//! (forcing "load_power")   # a ForcingRef by name
//! n                        # StepN
//! (neg <e>)                # unary minus
//! (+ <l> <r>)              # BinOp; op ∈ + - *
//! ```
//!
//! `Const` renders through [`simcore::hexfloat::format`] (not a decimal spelling) so a
//! literal's parity is **bit-exact** — the value the parser produced, not a rounded
//! re-print. The Python generator emits the identical spelling via `float.hex()`, which
//! the hex-float codec reproduces digit-for-digit (proven in Phase-7 Step 0).

use simcore::expr::Expr;
use simcore::hexfloat;

/// Render `expr` to its canonical S-expression string (see the module docs).
pub fn render_sexpr(expr: &Expr) -> String {
    match expr {
        Expr::Const(v) => format!("(const {})", hexfloat::format(*v)),
        Expr::StockRef(id) => format!("(stock {})", quote(id)),
        Expr::ParamRef(name) => format!("(param {})", quote(name)),
        Expr::ForcingRef(name) => format!("(forcing {})", quote(name)),
        Expr::StepN => "n".to_string(),
        Expr::Neg(operand) => format!("(neg {})", render_sexpr(operand)),
        Expr::BinOp { op, left, right } => {
            format!("({} {} {})", op.symbol(), render_sexpr(left), render_sexpr(right))
        }
    }
}

/// Double-quote an id/name for the canonical form. Ids are simple ASCII (dotted
/// namespaces), so no escaping is needed — matching the parser's no-escape string rule.
fn quote(s: &str) -> String {
    format!("\"{s}\"")
}
