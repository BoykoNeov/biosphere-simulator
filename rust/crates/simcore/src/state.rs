//! Immutable state primitives: [`Stock`] and the per-step [`State`] snapshot — the
//! Rust port of `simcore.state`.
//!
//! Python freezes both dataclasses and re-fires every invariant in `__post_init__`,
//! including on `dataclasses.replace`. The Rust port mirrors that: construction goes
//! through [`Stock::new`] / [`State::new`], which validate and return `Result`, and
//! the amounts-only "replace" helpers ([`Stock::with_amount`], the `State` rebuilds
//! in the integrator) re-run [`State::new`] so the same guards re-fire on every step.
//!
//! Composition (P2.1): the empty map means "default to the 1:1 `{quantity: 1.0}`
//! map", filled in [`Stock::new`] once the stock's own quantity is known — so a stock
//! always contributes to its nominal quantity. A gas-phase stock carries several
//! (CO₂ = `{Carbon:1, Oxygen:2}`). Composition iteration order never affects a float
//! result (each quantity is an independent accumulator downstream), so the
//! `BTreeMap` container's ordering is a convenience, not a correctness dependency.

use std::collections::BTreeMap;

use crate::error::SimError;
use crate::ids::{DomainId, StockId, UnitLabel};
use crate::quantities::{Quantity, StockKind};

/// A single well-mixed compartment holding an amount of one quantity.
///
/// Frozen in spirit: updates produce a *new* `Stock` (see [`Stock::with_amount`]).
/// `composition` is always the filled map (never empty) once constructed.
#[derive(Debug, Clone, PartialEq)]
pub struct Stock {
    pub id: StockId,
    pub domain: DomainId,
    pub quantity: Quantity,
    pub unit: UnitLabel,
    pub amount: f64,
    pub kind: StockKind,
    /// POPULATION only: at/below this level the stock snaps to 0 and the residual is
    /// routed to the numerical-loss sink. Ignored for other kinds.
    pub extinction_threshold: f64,
    /// BOUNDARY source only: never throttled by arbitration min-scaling.
    pub unclamped: bool,
    /// Element composition: moles of each conserved quantity one canonical unit of
    /// this stock contributes to the ledger. Always the filled map post-construction.
    pub composition: BTreeMap<Quantity, f64>,
}

impl Stock {
    /// Construct and validate a stock (mirrors Python `Stock.__post_init__`).
    ///
    /// An empty `composition` defaults to the 1:1 `{quantity: 1.0}` map. Returns
    /// [`SimError::Validation`] on any invariant breach (non-finite amount/threshold,
    /// an unclamped non-BOUNDARY stock, a bad composition, a multi-quantity
    /// POPULATION).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: StockId,
        domain: DomainId,
        quantity: Quantity,
        unit: UnitLabel,
        amount: f64,
        kind: StockKind,
        extinction_threshold: f64,
        unclamped: bool,
        composition: BTreeMap<Quantity, f64>,
    ) -> Result<Stock, SimError> {
        if !amount.is_finite() {
            return Err(SimError::Validation(format!(
                "Stock {id:?} amount is not finite: {amount:?}"
            )));
        }
        if !extinction_threshold.is_finite() {
            return Err(SimError::Validation(format!(
                "Stock {id:?} extinction_threshold is not finite: {extinction_threshold:?}"
            )));
        }
        // `unclamped` is meaningful only for BOUNDARY sources: an unclamped
        // POOL/POPULATION would silently escape throttling and could go negative.
        if unclamped && kind != StockKind::Boundary {
            return Err(SimError::Validation(format!(
                "Stock {id:?} is unclamped but kind={}; unclamped is only valid on \
                 BOUNDARY stocks (decision #13)",
                kind.name()
            )));
        }
        // Empty (the not-supplied sentinel) → the 1:1 default.
        let composition = if composition.is_empty() {
            BTreeMap::from([(quantity, 1.0)])
        } else {
            composition
        };
        for (q, coeff) in &composition {
            if !coeff.is_finite() {
                return Err(SimError::Validation(format!(
                    "Stock {id:?} composition[{}] is not finite: {coeff:?}",
                    q.name()
                )));
            }
        }
        match composition.get(&quantity) {
            Some(c) if *c > 0.0 => {}
            other => {
                return Err(SimError::Validation(format!(
                    "Stock {id:?} composition must include its own quantity {} with a \
                     positive coeff (got {other:?})",
                    quantity.name()
                )));
            }
        }
        // A POPULATION stock must be single-quantity: extinction routes its residual
        // via one nominal quantity, so a multi-quantity POPULATION would leak mass.
        if kind == StockKind::Population
            && (composition.len() != 1 || !composition.contains_key(&quantity))
        {
            let mut keys: Vec<&str> = composition.keys().map(|q| q.name()).collect();
            keys.sort_unstable();
            return Err(SimError::Validation(format!(
                "Stock {id:?} is POPULATION but multi-quantity (composition keys \
                 {keys:?}); a POPULATION stock must be single-quantity (extinction \
                 loss-sink routing is single-quantity, P2.1)"
            )));
        }
        Ok(Stock {
            id,
            domain,
            quantity,
            unit,
            amount,
            kind,
            extinction_threshold,
            unclamped,
            composition,
        })
    }

    /// A copy with `amount` replaced (the amounts-only `dataclasses.replace`).
    ///
    /// Re-validates finiteness of the new amount — the only invariant a
    /// change-of-amount can break (kind/composition/unclamped are unchanged and were
    /// valid at construction, so re-checking them would always pass). This matches
    /// the *effect* of Python re-running `__post_init__` on `replace`.
    pub fn with_amount(&self, amount: f64) -> Result<Stock, SimError> {
        if !amount.is_finite() {
            return Err(SimError::Validation(format!(
                "Stock {:?} amount is not finite: {amount:?}",
                self.id
            )));
        }
        Ok(Stock {
            amount,
            ..self.clone()
        })
    }
}

/// One immutable simulation snapshot.
///
/// Time is the integer step count `n` (`t = n*dt`, evaluated, never accumulated).
/// `aux` is the non-conserved auxiliary channel — scalar accumulators advanced by the
/// integrator outside the conservation gate.
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub n: u64,
    pub stocks: BTreeMap<StockId, Stock>,
    pub rng_seed: u64,
    pub aux: BTreeMap<String, f64>,
}

impl State {
    /// Construct and validate a snapshot (mirrors Python `State.__post_init__`).
    ///
    /// `n < 0` is unrepresentable (`u64`), subsuming Python's `n >= 0` guard. Each
    /// stock's map key must equal its own id, and every aux value must be finite.
    pub fn new(
        n: u64,
        stocks: BTreeMap<StockId, Stock>,
        rng_seed: u64,
        aux: BTreeMap<String, f64>,
    ) -> Result<State, SimError> {
        for (key, stock) in &stocks {
            if *key != stock.id {
                return Err(SimError::Validation(format!(
                    "State.stocks key {key:?} != stock.id {:?}",
                    stock.id
                )));
            }
        }
        for (name, value) in &aux {
            if !value.is_finite() {
                return Err(SimError::Validation(format!(
                    "State.aux[{name:?}] is not finite: {value:?}"
                )));
            }
        }
        Ok(State {
            n,
            stocks,
            rng_seed,
            aux,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool(id: &str, amount: f64) -> Stock {
        Stock::new(
            id.to_string(),
            "d".to_string(),
            Quantity::Carbon,
            "mol".to_string(),
            amount,
            StockKind::Pool,
            0.0,
            false,
            BTreeMap::new(),
        )
        .unwrap()
    }

    #[test]
    fn empty_composition_defaults_to_1to1() {
        let s = pool("a", 1.0);
        assert_eq!(s.composition, BTreeMap::from([(Quantity::Carbon, 1.0)]));
    }

    #[test]
    fn rejects_non_finite_amount() {
        let e = Stock::new(
            "a".to_string(),
            "d".to_string(),
            Quantity::Carbon,
            "mol".to_string(),
            f64::NAN,
            StockKind::Pool,
            0.0,
            false,
            BTreeMap::new(),
        );
        assert!(matches!(e, Err(SimError::Validation(_))));
    }

    #[test]
    fn rejects_unclamped_pool() {
        let e = Stock::new(
            "a".to_string(),
            "d".to_string(),
            Quantity::Carbon,
            "mol".to_string(),
            1.0,
            StockKind::Pool,
            0.0,
            true,
            BTreeMap::new(),
        );
        assert!(matches!(e, Err(SimError::Validation(_))));
    }

    #[test]
    fn rejects_multi_quantity_population() {
        let e = Stock::new(
            "p".to_string(),
            "d".to_string(),
            Quantity::Carbon,
            "mol".to_string(),
            1.0,
            StockKind::Population,
            0.0,
            false,
            BTreeMap::from([(Quantity::Carbon, 1.0), (Quantity::Oxygen, 2.0)]),
        );
        assert!(matches!(e, Err(SimError::Validation(_))));
    }

    #[test]
    fn state_rejects_key_id_mismatch() {
        let s = pool("a", 1.0);
        let stocks = BTreeMap::from([("wrong".to_string(), s)]);
        let e = State::new(0, stocks, 0, BTreeMap::new());
        assert!(matches!(e, Err(SimError::Validation(_))));
    }
}
