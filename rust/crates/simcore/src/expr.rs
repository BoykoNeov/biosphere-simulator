//! The bounded kinetics expression VM — the Rust port of `simcore.expr`
//! (Phase 9, Step 4a / decision A / decision D).
//!
//! This mirrors the **one deliberate, one-time extension** of the frozen Python
//! `simcore` core (the `expr.py` added in Phase-9 Step 2). Like the integrator, it
//! is a single frozen engine primitive: a plain-data AST + a pure evaluator +
//! [`DeclarativeFlow`] (rate-expr × stoichiometry, balanced by construction). It is
//! purely additive — no existing Rust `simcore` module changes — so every frozen
//! golden stays byte-identical.
//!
//! Why it lives in the core (not the boundary): an authored flow's rate expression is
//! evaluated **per step, inside the integrator** (once per Euler step, per stage under
//! RK4), so the evaluator must be deterministic like every other flow. *Parsing* a
//! scenario file's text into this AST is a one-time boundary act and stays in the
//! `authoring` crate (decision A). Only AST→`f64` evaluation is core.
//!
//! The grammar is **bounded, closed, and deterministic** (decision D) — a fixed,
//! finite set of primitives, no user functions / recursion / loops / I/O. Step 2
//! shipped the unambiguous arithmetic core, and Step 4a ports exactly that, no more:
//!
//!   * literals ([`Expr::Const`]);
//!   * reads: a stock amount by id ([`Expr::StockRef`]), a param by name
//!     ([`Expr::ParamRef`]), a forcing by name ([`Expr::ForcingRef`], resolved
//!     through [`Environment::get`] — #16), and the integer step `n` ([`Expr::StepN`]);
//!   * binary `+ - *` ([`Expr::BinOp`]) and unary `-` ([`Expr::Neg`]).
//!
//! **Deferred (do NOT complete the grammar).** Division and the closed function set
//! (`exp ln pow sqrt abs min max clamp monod` + bounded conditionals) are deliberately
//! *not* here — each carries a cross-port semantic choice (`x/0` is Python-raise vs
//! Rust-`inf`; `monod`/`clamp`/`ifpos` definitions) that a real frozen flow must force.
//! The [`BinaryOp`] enum literally cannot represent `/`, so an unimplemented op can
//! never silently evaluate — the analogue of the Python parser/evaluator's explicit
//! rejection (the "helpful dev adds all the ops" trap, structurally foreclosed).
//!
//! **No `dt` token, by construction.** The rate expression is the *instantaneous* rate
//! (`dt`-independent); [`DeclarativeFlow`] supplies the single `× dt` that makes the
//! per-step increment. So RK4-order-safety is structural for every authored flow. `n`
//! stays readable (`dt`-independent, safe).
//!
//! The AST is plain data (an `Expr` tree of `f64`/`String`/nested nodes), so this is a
//! mechanical mirror of the Python module and cross-port parity is tolerance-gated
//! exactly like every other transcendental (Phase-7 3-tier contract): a
//! transcendental-free authored flow is Tier-1 bit-exact.

use std::collections::BTreeMap;

use crate::environment::Environment;
use crate::error::SimError;
use crate::flow::{Flow, FlowResult, Leg};
use crate::ids::{FlowId, StockId};
use crate::state::State;

/// The closed binary-operator set Step 2 ships (unambiguous IEEE arithmetic).
///
/// An enum, not a `char`/`String`: `/` and the function set are *deferred*, and an
/// enum with only these three variants makes an unsupported op **unrepresentable** —
/// the Rust type system subsumes the Python evaluator's `raise ValueError` fallback,
/// the same "the check moved to compile time" pattern as the RNG `TypeError` guards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
}

impl BinaryOp {
    /// The canonical symbol (`"+"` / `"-"` / `"*"`) — the parse-parity spelling.
    pub fn symbol(self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
        }
    }
}

/// The bounded-grammar rate-expression AST (plain data — immutable, `PartialEq`).
///
/// Every node is a value type, so the whole tree is comparable and ports mechanically.
/// `Neg`/`BinOp` box their operands because the type is recursive.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A numeric literal (the parser has already coerced it to `f64`).
    Const(f64),
    /// A read of a stock's current amount, by id, from the evaluation snapshot —
    /// read **directly** (`snapshot.stocks[id].amount`), the donor-controlled idiom
    /// (`SelfDischarge` reads `battery` this way), *not* via `env`.
    StockRef(StockId),
    /// A read of a flow param by name (from the flow's own param map).
    ParamRef(String),
    /// A read of a forcing var by name, resolved through [`Environment::get`] (#16) —
    /// indistinguishable at evaluation time from a coupled sibling's shared stock.
    ForcingRef(String),
    /// A read of the integer step count `n` (as an `f64`); `dt`-independent.
    StepN,
    /// Unary negation `- operand`.
    Neg(Box<Expr>),
    /// A binary arithmetic op `left <op> right`. `left` is evaluated before `right`
    /// and then combined — a fixed op-order the Python port mirrors, so the result is
    /// bit-identical within a build.
    BinOp {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

/// Evaluate `node` to an `f64` against a snapshot/env/param context.
///
/// Pure and deterministic in its inputs. Reference resolution mirrors the frozen flows
/// and the Python `eval_expr`: [`Expr::StockRef`] reads the snapshot directly (#16
/// donor read), [`Expr::ForcingRef`] goes through [`Environment::get`] (#16
/// forcing/shared read), [`Expr::ParamRef`] reads the flow's param map, [`Expr::StepN`]
/// reads `snapshot.n`. A `StockRef`/`ParamRef` at a missing id returns
/// [`SimError::Reference`] (Python raises `KeyError`); referential integrity is
/// validated at *build* time by the interpreter, so these are belt-and-suspenders.
///
/// `dt` is intentionally absent from the signature: the rate grammar has no `dt` token,
/// so a rate expression cannot depend on `dt`.
pub fn eval_expr(
    node: &Expr,
    snapshot: &State,
    env: &dyn Environment,
    params: &BTreeMap<String, f64>,
) -> Result<f64, SimError> {
    match node {
        Expr::Const(value) => Ok(*value),
        Expr::StockRef(stock) => snapshot
            .stocks
            .get(stock)
            .map(|s| s.amount)
            .ok_or_else(|| {
                SimError::Reference(format!("rate expression reads unknown stock {stock:?}"))
            }),
        Expr::ParamRef(name) => params.get(name).copied().ok_or_else(|| {
            SimError::Reference(format!("rate expression reads unknown param {name:?}"))
        }),
        Expr::ForcingRef(name) => env.get(name),
        Expr::StepN => Ok(snapshot.n as f64),
        Expr::Neg(operand) => Ok(-eval_expr(operand, snapshot, env, params)?),
        Expr::BinOp { op, left, right } => {
            // left before right, then combine — the fixed op-order Python mirrors.
            let l = eval_expr(left, snapshot, env, params)?;
            let r = eval_expr(right, snapshot, env, params)?;
            Ok(match op {
                BinaryOp::Add => l + r,
                BinaryOp::Sub => l - r,
                BinaryOp::Mul => l * r,
            })
        }
    }
}

/// An authored [`Flow`]: an instantaneous *rate* expression × a fixed stoichiometry —
/// the Rust port of Python's `DeclarativeFlow`.
///
/// **Balanced by construction (decision C).** The flow emits one leg per
/// `(stock, coeff)` pair, all sharing the single scalar `increment = rate·dt`, so per
/// conserved quantity `Σ legs = rate·dt · Σ(coeff · composition)` — which is `0` for
/// *any* rate value **iff** the stoichiometry's coefficient vector balances. The
/// author picks only the scalar rate and the (integer/rational) coefficients; they
/// cannot vary the per-leg magnitude independently. The interpreter validates the
/// coefficient vector against the stock compositions at build time; the every-step
/// conservation gate is then a redundant backstop.
///
/// **Increment-form (RK4-order-safe).** `rate` is the `dt`-independent instantaneous
/// rate; [`Flow::evaluate`] forms `increment = rate · dt` and each leg is
/// `coeff · increment` — the standard flux→`× dt` split every frozen flow uses, so
/// RK4's ⅙-combine reproduces classical RK4 exactly. The rate has no `dt` token
/// (grammar-enforced), so `dt`-linearity is structural.
///
/// `params` is stored as a sorted `(name, value)` list (the frozen-flow idiom) and
/// exposed to the evaluator as a `BTreeMap`; `stoichiometry` is `(stock, coeff)` in
/// author order (leg order does not affect the trajectory — each leg lands on a
/// distinct stock and the integrator sums per stock).
#[derive(Debug, Clone, PartialEq)]
pub struct DeclarativeFlow {
    id: FlowId,
    priority: i64,
    rate: Expr,
    stoichiometry: Vec<(StockId, f64)>,
    params: Vec<(String, f64)>,
}

impl DeclarativeFlow {
    /// Construct a `DeclarativeFlow`. `params` is stored sorted by name (the
    /// frozen-flow idiom + a stable canonical form); `stoichiometry` keeps author
    /// order (leg order is inert).
    pub fn new(
        id: FlowId,
        priority: i64,
        rate: Expr,
        stoichiometry: Vec<(StockId, f64)>,
        params: Vec<(String, f64)>,
    ) -> DeclarativeFlow {
        let mut params = params;
        params.sort_by(|a, b| a.0.cmp(&b.0));
        DeclarativeFlow {
            id,
            priority,
            rate,
            stoichiometry,
            params,
        }
    }

    fn param_map(&self) -> BTreeMap<String, f64> {
        self.params.iter().cloned().collect()
    }
}

impl Flow for DeclarativeFlow {
    fn id(&self) -> &str {
        &self.id
    }

    fn priority(&self) -> i64 {
        self.priority
    }

    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let rate = eval_expr(&self.rate, snapshot, env, &self.param_map())?;
        let increment = rate * dt;
        let mut legs = Vec::with_capacity(self.stoichiometry.len());
        for (stock, coeff) in &self.stoichiometry {
            legs.push(Leg::new(stock.clone(), coeff * increment)?);
        }
        FlowResult::new(legs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::{constant, SourceResolver};
    use crate::flow::assert_flow_balanced_default;
    use crate::quantities::{Quantity, StockKind};
    use crate::state::Stock;

    const BATTERY: &str = "power.battery";
    const WASTE_HEAT: &str = "boundary.waste_heat";

    fn energy_stock(id: &str, amount: f64, kind: StockKind) -> Stock {
        Stock::new(
            id.to_string(),
            "power".to_string(),
            Quantity::Energy,
            "J".to_string(),
            amount,
            kind,
            0.0,
            false,
            BTreeMap::new(),
        )
        .unwrap()
    }

    fn state(battery: f64, n: u64) -> State {
        let mut stocks = BTreeMap::new();
        stocks.insert(BATTERY.to_string(), energy_stock(BATTERY, battery, StockKind::Pool));
        stocks.insert(
            WASTE_HEAT.to_string(),
            energy_stock(WASTE_HEAT, 0.0, StockKind::Boundary),
        );
        State::new(n, stocks, 0, BTreeMap::new()).unwrap()
    }

    fn eval(node: &Expr, battery: f64, n: u64, params: &[(&str, f64)]) -> f64 {
        let st = state(battery, n);
        let resolver = SourceResolver::empty();
        let env = resolver.bind(&st, 0.0);
        let pmap: BTreeMap<String, f64> =
            params.iter().map(|(k, v)| (k.to_string(), *v)).collect();
        eval_expr(node, &st, &env, &pmap).unwrap()
    }

    // --- leaf reads --------------------------------------------------------
    #[test]
    fn const_evaluates_to_its_value() {
        assert_eq!(eval(&Expr::Const(2.5), 5.0, 3, &[]), 2.5);
    }

    #[test]
    fn stock_ref_reads_the_snapshot_amount() {
        assert_eq!(eval(&Expr::StockRef(BATTERY.to_string()), 7.25, 3, &[]), 7.25);
    }

    #[test]
    fn param_ref_reads_the_param_map() {
        assert_eq!(
            eval(&Expr::ParamRef("k".to_string()), 5.0, 3, &[("k", 1.0e-8)]),
            1.0e-8
        );
    }

    #[test]
    fn forcing_ref_reads_through_env() {
        // A forcing var resolves through env.get (#16); build a resolver that has it.
        let st = state(5.0, 3);
        let mut forcings = std::collections::HashMap::new();
        forcings.insert("load_power".to_string(), constant(42.0).unwrap());
        let resolver = SourceResolver::new(forcings, std::collections::HashMap::new()).unwrap();
        let env = resolver.bind(&st, 0.0);
        let pmap = BTreeMap::new();
        let value = eval_expr(&Expr::ForcingRef("load_power".to_string()), &st, &env, &pmap);
        assert_eq!(value.unwrap(), 42.0);
    }

    #[test]
    fn step_n_reads_n_as_float() {
        assert_eq!(eval(&Expr::StepN, 5.0, 9, &[]), 9.0);
    }

    #[test]
    fn missing_param_is_a_reference_error() {
        // Belt-and-suspenders (the interpreter validates refs at build time): the raw
        // VM surfaces a missing param as SimError::Reference, not a wrong answer.
        let st = state(5.0, 3);
        let resolver = SourceResolver::empty();
        let env = resolver.bind(&st, 0.0);
        let pmap = BTreeMap::new();
        let err = eval_expr(&Expr::ParamRef("absent".to_string()), &st, &env, &pmap);
        assert!(matches!(err, Err(SimError::Reference(_))));
    }

    // --- operators, op-for-op ---------------------------------------------
    fn binop(op: BinaryOp, left: Expr, right: Expr) -> Expr {
        Expr::BinOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    #[test]
    fn addition() {
        assert_eq!(
            eval(&binop(BinaryOp::Add, Expr::Const(0.1), Expr::Const(0.2)), 5.0, 3, &[]),
            0.1 + 0.2
        );
    }

    #[test]
    fn subtraction() {
        assert_eq!(
            eval(&binop(BinaryOp::Sub, Expr::Const(1.0), Expr::Const(0.3)), 5.0, 3, &[]),
            1.0 - 0.3
        );
    }

    #[test]
    fn multiplication() {
        let expr = binop(
            BinaryOp::Mul,
            Expr::Const(1.0e-8),
            Expr::StockRef(BATTERY.to_string()),
        );
        assert_eq!(eval(&expr, 1.0e7, 3, &[]), 1.0e-8 * 1.0e7);
    }

    #[test]
    fn unary_negation() {
        assert_eq!(
            eval(&Expr::Neg(Box::new(Expr::StockRef(BATTERY.to_string()))), 3.0, 3, &[]),
            -3.0
        );
    }

    #[test]
    fn negation_is_exact_for_zero() {
        // -x sign-of-zero: what makes DeclarativeFlow's leg formula bit-identical to a
        // frozen `-leak`; assert the VM's Neg produces -0.0.
        let v = eval(&Expr::Neg(Box::new(Expr::Const(0.0))), 5.0, 3, &[]);
        assert_eq!(v.signum(), -1.0);
        assert!(v == 0.0 && v.is_sign_negative());
    }

    #[test]
    fn left_associativity_of_subtraction() {
        // (10 - 3) - 2, not 10 - (3 - 2): left combined before right.
        let expr = binop(
            BinaryOp::Sub,
            binop(BinaryOp::Sub, Expr::Const(10.0), Expr::Const(3.0)),
            Expr::Const(2.0),
        );
        assert_eq!(eval(&expr, 5.0, 3, &[]), (10.0 - 3.0) - 2.0);
    }

    #[test]
    fn nested_rate_expression_op_for_op() {
        // k * (battery - n): a demand-shaped rate, in the fixed op-order.
        let expr = binop(
            BinaryOp::Mul,
            Expr::ParamRef("k".to_string()),
            binop(BinaryOp::Sub, Expr::StockRef(BATTERY.to_string()), Expr::StepN),
        );
        assert_eq!(eval(&expr, 8.0, 2, &[("k", 0.5)]), 0.5 * (8.0 - 2.0));
    }

    // --- DeclarativeFlow ---------------------------------------------------
    fn self_discharge_flow(k: f64) -> DeclarativeFlow {
        // rate k·battery, −1/+1 ENERGY split (the SelfDischarge re-expression).
        DeclarativeFlow::new(
            "power.self_discharge".to_string(),
            0,
            binop(
                BinaryOp::Mul,
                Expr::ParamRef("k".to_string()),
                Expr::StockRef(BATTERY.to_string()),
            ),
            vec![(BATTERY.to_string(), -1.0), (WASTE_HEAT.to_string(), 1.0)],
            vec![("k".to_string(), k)],
        )
    }

    fn evaluate_flow(flow: &DeclarativeFlow, battery: f64, dt: f64) -> FlowResult {
        let st = state(battery, 0);
        let resolver = SourceResolver::empty();
        let env = resolver.bind(&st, dt);
        flow.evaluate(&st, &env, dt).unwrap()
    }

    #[test]
    fn declarative_flow_legs_and_increment_form() {
        let flow = self_discharge_flow(1.0e-8);
        let result = evaluate_flow(&flow, 1.0e7, 3600.0);
        let leak = (1.0e-8 * 1.0e7) * 3600.0;
        // Legs in author (stoichiometry) order; each on a distinct stock.
        let stocks: Vec<&str> = result.legs.iter().map(|l| l.stock.as_str()).collect();
        assert_eq!(stocks, vec![BATTERY, WASTE_HEAT]);
        let battery_leg = result.legs.iter().find(|l| l.stock == BATTERY).unwrap();
        let heat_leg = result.legs.iter().find(|l| l.stock == WASTE_HEAT).unwrap();
        assert_eq!(battery_leg.amount, -leak);
        assert_eq!(heat_leg.amount, leak);
    }

    #[test]
    fn declarative_flow_is_energy_balanced() {
        let flow = self_discharge_flow(1.0e-8);
        let st = state(1.0e7, 0);
        let result = evaluate_flow(&flow, 1.0e7, 3600.0);
        // Errors if unbalanced; the −1/+1 split balances exactly.
        assert_flow_balanced_default(&result, &st.stocks).unwrap();
    }

    #[test]
    fn declarative_flow_is_dt_linear() {
        let flow = self_discharge_flow(1.0e-8);
        let small = evaluate_flow(&flow, 1.0e7, 1.0);
        let big = evaluate_flow(&flow, 1.0e7, 10.0);
        let small_b = small.legs.iter().find(|l| l.stock == BATTERY).unwrap().amount;
        let big_b = big.legs.iter().find(|l| l.stock == BATTERY).unwrap().amount;
        // The rate is dt-independent, so the increment scales exactly linearly in dt.
        assert_eq!(big_b, small_b * 10.0);
    }

    #[test]
    fn declarative_flow_zero_battery_is_no_op() {
        let flow = self_discharge_flow(1.0e-8);
        let result = evaluate_flow(&flow, 0.0, 3600.0);
        for leg in &result.legs {
            assert_eq!(leg.amount, 0.0);
        }
    }
}
