//! Template parameters + boundary-time parameter-expression evaluation — the Rust
//! mirror of Python `authoring.template` (Phase 9, Step 4b; the port of Step 3).
//!
//! A **template** scenario declares named scalar `parameters` (with defaults) that an
//! instantiation may override, and a stock `amount` / forcing `const` may be a
//! bounded-grammar **expression** over them (`param('crew_count') * 1000.0`). The
//! interpreter evaluates those expressions to literals at **build time**.
//!
//! **This is the boundary-eval cross-port surface (Step 3's deliberate amendment of
//! decision A).** The Python boundary now does `+ − ×`; the Rust boundary must compute
//! the same float. It is benign — the ops are IEEE-deterministic and decimals
//! round-trip via `f64::from_str` (correctly-rounded, like Python `float()`) — but it
//! is load-bearing, so it is a real parity surface (the graph-dump gate diffs it).
//!
//! **The grammar is reused, the context is not.** Template expressions parse with the
//! *same* [`crate::expr_parser::parse_rate_expr`] as the kinetics DSL (one parser,
//! precedence pinned once), but evaluate where no `State`/`env`/`n` exists: only
//! [`Expr::Const`] / [`Expr::ParamRef`] / [`Expr::Neg`] / [`Expr::BinOp`] are legal,
//! `param('…')` resolves against the **template-parameter** namespace, and a
//! stock/forcing/`n` reference is an [`AuthoringError`]. The `+ − ×` op-order mirrors
//! [`simcore::expr::eval_expr`] exactly, so a boundary literal is bit-identical to what
//! the engine VM would compute from the same AST — one op-order, both ports.

use std::collections::BTreeMap;

use simcore::expr::{BinaryOp, Expr};

use crate::errors::AuthoringError;
use crate::expr_parser::parse_rate_expr;
use crate::schema::NumericField;

/// Merge declared parameter defaults with an instantiation's overrides (the
/// `resolve_parameters` analogue). An override of an **undeclared** name is an error (a
/// template's parameter set is its explicit contract).
pub fn resolve_parameters(
    declared: &[(String, f64)],
    overrides: &BTreeMap<String, f64>,
) -> Result<BTreeMap<String, f64>, AuthoringError> {
    let mut params: BTreeMap<String, f64> = declared.iter().cloned().collect();
    for (name, value) in overrides {
        if !params.contains_key(name) {
            let declared_names: Vec<&String> = params.keys().collect();
            return Err(AuthoringError::new(format!(
                "override of undeclared parameter {name:?} (declared: {declared_names:?})"
            )));
        }
        params.insert(name.clone(), *value);
    }
    Ok(params)
}

/// Lower a numeric scenario field (`StockSpec.amount` / `ForcingSpec.const`) to a
/// literal `f64` (the `eval_numeric_field` analogue). A [`NumericField::Number`] passes
/// through (the all-literal form); a [`NumericField::Expr`] is a template expression
/// parsed with the bounded grammar and evaluated against `params`. `where_` labels the
/// field in any error.
pub fn eval_numeric_field(
    value: &NumericField,
    params: &BTreeMap<String, f64>,
    where_: &str,
) -> Result<f64, AuthoringError> {
    match value {
        NumericField::Number(n) => Ok(*n),
        NumericField::Expr(text) => {
            let ast = parse_rate_expr(text)?;
            eval_build_time(&ast, params, where_, text)
        }
    }
}

/// Evaluate a build-time-legal AST subtree against the template parameter map. Only
/// `Const`/`ParamRef`/`Neg`/`BinOp` are legal at build time (mirrors Python `_eval`);
/// a `StockRef`/`ForcingRef`/`StepN` or an undeclared `param('…')` is an
/// [`AuthoringError`]. The `+ − ×` op-order mirrors [`simcore::expr::eval_expr`].
fn eval_build_time(
    node: &Expr,
    params: &BTreeMap<String, f64>,
    where_: &str,
    whole: &str,
) -> Result<f64, AuthoringError> {
    match node {
        Expr::Const(value) => Ok(*value),
        Expr::ParamRef(name) => params.get(name).copied().ok_or_else(|| {
            let declared: Vec<&String> = params.keys().collect();
            AuthoringError::new(format!(
                "{where_}: expression {whole:?} references undeclared parameter {name:?} \
                 (declared: {declared:?})"
            ))
        }),
        Expr::Neg(operand) => Ok(-eval_build_time(operand, params, where_, whole)?),
        Expr::BinOp { op, left, right } => {
            // left before right, then combine — the fixed op-order the engine VM uses.
            let l = eval_build_time(left, params, where_, whole)?;
            let r = eval_build_time(right, params, where_, whole)?;
            Ok(match op {
                BinaryOp::Add => l + r,
                BinaryOp::Sub => l - r,
                BinaryOp::Mul => l * r,
            })
        }
        // StockRef / ForcingRef / StepN — legal in a kinetics *rate*, but there is no
        // State/env/n at build time.
        Expr::StockRef(_) | Expr::ForcingRef(_) | Expr::StepN => Err(AuthoringError::new(format!(
            "{where_}: template expression {whole:?} may reference only template \
             parameters (param('…')); stock/forcing/n are not available at build time"
        ))),
    }
}
