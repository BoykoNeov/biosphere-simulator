//! Flows: structured, atomic, stoichiometric transfers — the Rust port of
//! `simcore.flow`.
//!
//! A [`Flow`] exposes only [`Flow::evaluate`]; legs exist **only after** evaluation
//! against a snapshot, so balance is an *evaluation-time* property of a [`FlowResult`]
//! (per conserved quantity the legs sum to ~0), checked by [`assert_flow_balanced`].
//! Referential integrity (a leg naming a real stock) is the integrator apply path's
//! job, not checked here — a leg on an unknown stock surfaces as [`SimError::Reference`]
//! in [`per_quantity_residual`] / [`assert_flow_balanced`], mirroring Python's
//! `KeyError`.
//!
//! ENERGY is asserted like the mass quantities (it joined the conserved set in
//! Phase 5); [`per_quantity_residual`] reports every quantity present and
//! [`assert_flow_balanced`] iterates all of [`ASSERTED_QUANTITIES`].

use std::collections::{BTreeMap, BTreeSet};

use crate::environment::Environment;
use crate::error::SimError;
use crate::ids::{DomainId, StockId};
use crate::quantities::{Quantity, ASSERTED_QUANTITIES, BALANCE_ATOL, BALANCE_RTOL};
use crate::state::{State, Stock};

/// One stock touched by a flow. `amount` is per dt in the stock's canonical unit:
/// `> 0` deposits, `< 0` withdraws.
#[derive(Debug, Clone, PartialEq)]
pub struct Leg {
    pub stock: StockId,
    pub amount: f64,
}

impl Leg {
    /// Construct a leg, rejecting a non-finite amount (Python `Leg.__post_init__`).
    pub fn new(stock: StockId, amount: f64) -> Result<Leg, SimError> {
        if !amount.is_finite() {
            return Err(SimError::Validation(format!(
                "Leg on {stock:?} amount is not finite: {amount:?}"
            )));
        }
        Ok(Leg { stock, amount })
    }
}

/// The requested transfer from one `evaluate` against a snapshot. At most one leg per
/// stock — a flow nets its own touches into a single leg.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowResult {
    pub legs: Vec<Leg>,
}

impl FlowResult {
    /// Construct a result, rejecting a duplicate leg on any stock (Python
    /// `FlowResult.__post_init__`). An empty `legs` is a valid no-op.
    pub fn new(legs: Vec<Leg>) -> Result<FlowResult, SimError> {
        let mut seen: BTreeSet<&StockId> = BTreeSet::new();
        for leg in &legs {
            if !seen.insert(&leg.stock) {
                return Err(SimError::Validation(format!(
                    "FlowResult has a duplicate leg for stock {:?}; a flow must net \
                     its own touches on a stock into one leg",
                    leg.stock
                )));
            }
        }
        Ok(FlowResult { legs })
    }

    /// The empty (no-op) result.
    pub fn empty() -> FlowResult {
        FlowResult { legs: Vec::new() }
    }
}

/// A pure, deterministic stoichiometric transfer (Python's `Flow` Protocol).
///
/// `evaluate` reads the snapshot/env only, never mutates, and returns the *per-step
/// increment* (`dt·rate`, increment-form — `rate` must be independent of `dt`, which
/// is what lets RK4's ⅙-combine reproduce classical RK4). `priority` is carried for
/// declared-controller policies but is unused under the proportional default;
/// canonical order is always id-sorted, never priority-sorted.
pub trait Flow {
    /// The canonical flow id (ASCII — its sort order is the reduction order).
    fn id(&self) -> &str;

    /// Declared priority (unused under the proportional default; defaults to 0).
    fn priority(&self) -> i64 {
        0
    }

    /// Evaluate the flow against `snapshot`/`env`, returning its per-step legs.
    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError>;
}

/// Net leg sum per `Quantity` present in `result` — the diagnostic the balance gate
/// composes. Legs are folded in **canonical order (sorted by stock id)**, so the sum
/// is bit-identical regardless of leg construction order (each quantity accumulates
/// in sorted-leg order). A 1:1 stock folds `amount·1.0` to its single quantity.
///
/// A leg naming an unknown stock returns [`SimError::Reference`] (Python `KeyError`).
pub fn per_quantity_residual(
    result: &FlowResult,
    stocks: &BTreeMap<StockId, Stock>,
) -> Result<BTreeMap<Quantity, f64>, SimError> {
    let mut order: Vec<&Leg> = result.legs.iter().collect();
    order.sort_by(|a, b| a.stock.cmp(&b.stock));
    let mut residual: BTreeMap<Quantity, f64> = BTreeMap::new();
    for leg in order {
        let stock = stocks.get(&leg.stock).ok_or_else(|| {
            SimError::Reference(format!("unknown stock {:?} in flow leg", leg.stock))
        })?;
        for (quantity, coeff) in &stock.composition {
            let acc = residual.entry(*quantity).or_insert(0.0);
            *acc += leg.amount * coeff;
        }
    }
    Ok(residual)
}

/// Raise [`SimError::Conservation`] if any asserted quantity fails to balance:
/// `abs(residual) > atol + rtol * scale`, where `scale` is the transfer magnitude
/// (`max |leg.amount·coeff|`) for that quantity. Quantities are checked in name-sorted
/// order so the first reported failure is deterministic.
pub fn assert_flow_balanced(
    result: &FlowResult,
    stocks: &BTreeMap<StockId, Stock>,
    atol: f64,
    rtol: f64,
) -> Result<(), SimError> {
    let residual = per_quantity_residual(result, stocks)?;
    // scale: max |leg.amount·coeff| per quantity (leg order irrelevant — it is a max).
    let mut scale: BTreeMap<Quantity, f64> = BTreeMap::new();
    for leg in &result.legs {
        let stock = stocks.get(&leg.stock).ok_or_else(|| {
            SimError::Reference(format!("unknown stock {:?} in flow leg", leg.stock))
        })?;
        for (quantity, coeff) in &stock.composition {
            let acc = scale.entry(*quantity).or_insert(0.0);
            *acc = acc.max((leg.amount * coeff).abs());
        }
    }
    for quantity in ASSERTED_QUANTITIES {
        let r = residual.get(&quantity).copied().unwrap_or(0.0);
        let tol = atol + rtol * scale.get(&quantity).copied().unwrap_or(0.0);
        if r.abs() > tol {
            return Err(SimError::Conservation(format!(
                "flow not balanced for {}: residual {r:?} exceeds tolerance {tol:?}",
                quantity.name()
            )));
        }
    }
    Ok(())
}

/// The default-tolerance [`assert_flow_balanced`] (using [`BALANCE_ATOL`] /
/// [`BALANCE_RTOL`]).
pub fn assert_flow_balanced_default(
    result: &FlowResult,
    stocks: &BTreeMap<StockId, Stock>,
) -> Result<(), SimError> {
    assert_flow_balanced(result, stocks, BALANCE_ATOL, BALANCE_RTOL)
}

/// The domains the evaluated `result` touches (`> 1` ⇒ cross-domain). A leg naming an
/// unknown stock returns [`SimError::Reference`].
pub fn domains_touched(
    result: &FlowResult,
    stocks: &BTreeMap<StockId, Stock>,
) -> Result<BTreeSet<DomainId>, SimError> {
    let mut domains: BTreeSet<DomainId> = BTreeSet::new();
    for leg in &result.legs {
        let stock = stocks.get(&leg.stock).ok_or_else(|| {
            SimError::Reference(format!("unknown stock {:?} in flow leg", leg.stock))
        })?;
        domains.insert(stock.domain.clone());
    }
    Ok(domains)
}

/// A read-only view of a flow id + its priority — a convenience for tests/callers.
pub fn flow_id(flow: &dyn Flow) -> &str {
    flow.id()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quantities::StockKind;

    fn co2_stock(id: &str) -> Stock {
        // A gas-phase composition stock: CO₂ = {Carbon:1, Oxygen:2}.
        Stock::new(
            id.to_string(),
            "d".to_string(),
            Quantity::Carbon,
            "mol".to_string(),
            10.0,
            StockKind::Pool,
            0.0,
            false,
            BTreeMap::from([(Quantity::Carbon, 1.0), (Quantity::Oxygen, 2.0)]),
        )
        .unwrap()
    }

    fn o2_stock(id: &str) -> Stock {
        Stock::new(
            id.to_string(),
            "d".to_string(),
            Quantity::Oxygen,
            "mol".to_string(),
            10.0,
            StockKind::Pool,
            0.0,
            false,
            BTreeMap::from([(Quantity::Oxygen, 2.0)]),
        )
        .unwrap()
    }

    #[test]
    fn rejects_duplicate_leg() {
        let e = FlowResult::new(vec![
            Leg::new("a".to_string(), 1.0).unwrap(),
            Leg::new("a".to_string(), -1.0).unwrap(),
        ]);
        assert!(matches!(e, Err(SimError::Validation(_))));
    }

    #[test]
    fn composition_fold_books_both_quantities() {
        // Withdraw 1 CO₂ unit, deposit into another CO₂ pool → balanced for both
        // CARBON and OXYGEN via the composition fold.
        let stocks = BTreeMap::from([
            ("co2.a".to_string(), co2_stock("co2.a")),
            ("co2.b".to_string(), co2_stock("co2.b")),
        ]);
        let result = FlowResult::new(vec![
            Leg::new("co2.a".to_string(), -1.0).unwrap(),
            Leg::new("co2.b".to_string(), 1.0).unwrap(),
        ])
        .unwrap();
        let res = per_quantity_residual(&result, &stocks).unwrap();
        assert_eq!(res.get(&Quantity::Carbon), Some(&0.0));
        assert_eq!(res.get(&Quantity::Oxygen), Some(&0.0));
        assert!(assert_flow_balanced_default(&result, &stocks).is_ok());
    }

    #[test]
    fn imbalanced_flow_is_caught() {
        let stocks = BTreeMap::from([
            ("o2.a".to_string(), o2_stock("o2.a")),
            ("o2.b".to_string(), o2_stock("o2.b")),
        ]);
        // Withdraw 1 unit, deposit 0.5 — OXYGEN residual = 2*(-1)+2*(0.5) = -1.0.
        let result = FlowResult::new(vec![
            Leg::new("o2.a".to_string(), -1.0).unwrap(),
            Leg::new("o2.b".to_string(), 0.5).unwrap(),
        ])
        .unwrap();
        let e = assert_flow_balanced_default(&result, &stocks);
        assert!(matches!(e, Err(SimError::Conservation(_))));
    }

    #[test]
    fn unknown_stock_is_reference_error() {
        let stocks = BTreeMap::from([("o2.a".to_string(), o2_stock("o2.a"))]);
        let result = FlowResult::new(vec![Leg::new("missing".to_string(), 1.0).unwrap()]).unwrap();
        assert!(matches!(
            per_quantity_residual(&result, &stocks),
            Err(SimError::Reference(_))
        ));
    }
}
