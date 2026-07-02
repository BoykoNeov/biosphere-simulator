//! The Boundary domain — the Rust port of `simcore.boundary`.
//!
//! External forcing (irrigation adds water, harvest removes carbon) is unbalanced
//! against the modeled stocks, which the flow invariant (`Σ legs == 0`) forbids.
//! Resolution: model "outside" as explicit BOUNDARY reservoir stocks — a source
//! (unclamped supply), a sink (disposal), or a loss-sink (extinction's numerical-loss
//! reservoir). These constructors only *build* the stocks; they must be placed into
//! the initial `State` for the extinction deposit and the ledger to resolve them.

use std::collections::BTreeMap;

use crate::error::SimError;
use crate::ids::StockId;
use crate::quantities::{Quantity, StockKind, ASSERTED_QUANTITIES};
use crate::state::Stock;

/// The canonical Boundary domain id.
pub const BOUNDARY_DOMAIN: &str = "boundary";

/// Numerical-loss sink ids share this prefix so diagnostics and the ledger can
/// separate routed numerical-loss deltas from legitimate boundary exchange. ASCII-only
/// so Python's `str` sort matches the Rust UTF-8 byte sort (#15).
pub const LOSS_SINK_PREFIX: &str = "boundary.loss.";

/// The canonical, deterministic id for `quantity`'s numerical-loss sink.
pub fn loss_sink_id(quantity: Quantity) -> StockId {
    format!("{LOSS_SINK_PREFIX}{}", quantity.value())
}

/// True iff `stock_id` names a numerical-loss sink.
pub fn is_loss_sink(stock_id: &str) -> bool {
    stock_id.starts_with(LOSS_SINK_PREFIX)
}

fn boundary_stock(
    id: StockId,
    quantity: Quantity,
    amount: f64,
    unclamped: bool,
) -> Result<Stock, SimError> {
    Stock::new(
        id,
        BOUNDARY_DOMAIN.to_string(),
        quantity,
        quantity.canonical_unit(),
        amount,
        StockKind::Boundary,
        0.0,
        unclamped,
        BTreeMap::new(),
    )
}

/// A numerical-loss BOUNDARY reservoir for `quantity` (accumulates snapped extinction
/// residuals so the ledger balances). Never withdrawn from, so it stays clamped.
pub fn loss_sink(quantity: Quantity, amount: f64) -> Result<Stock, SimError> {
    boundary_stock(loss_sink_id(quantity), quantity, amount, false)
}

/// One loss-sink per quantity, keyed by id (ready to merge into a state). Built in
/// canonical (quantity-name) order so the result is deterministic. Defaults to
/// [`ASSERTED_QUANTITIES`].
pub fn loss_sinks(quantities: &[Quantity]) -> Result<BTreeMap<StockId, Stock>, SimError> {
    let mut ordered: Vec<Quantity> = quantities.to_vec();
    ordered.sort_by(|a, b| a.name().cmp(b.name()));
    let mut out: BTreeMap<StockId, Stock> = BTreeMap::new();
    for q in ordered {
        let s = loss_sink(q, 0.0)?;
        out.insert(s.id.clone(), s);
    }
    Ok(out)
}

/// The default loss-sink set (one per [`ASSERTED_QUANTITIES`] member).
pub fn loss_sinks_default() -> Result<BTreeMap<StockId, Stock>, SimError> {
    loss_sinks(&ASSERTED_QUANTITIES)
}

/// An "outside" supply reservoir (e.g. solar). `unclamped` defaults to `true` in
/// Python; this port makes it explicit — pass `true` for the usual never-throttled
/// supply, `false` for a finite throttleable boundary supply.
pub fn source(
    stock_id: StockId,
    quantity: Quantity,
    amount: f64,
    unclamped: bool,
) -> Result<Stock, SimError> {
    boundary_stock(stock_id, quantity, amount, unclamped)
}

/// An "outside" disposal reservoir (receives outputs; never withdrawn from). `amount`
/// is an accumulator of cumulative output (usually start at 0).
pub fn sink(stock_id: StockId, quantity: Quantity, amount: f64) -> Result<Stock, SimError> {
    boundary_stock(stock_id, quantity, amount, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loss_sink_id_is_prefixed_value() {
        assert_eq!(loss_sink_id(Quantity::Carbon), "boundary.loss.carbon");
        assert!(is_loss_sink("boundary.loss.carbon"));
        assert!(!is_loss_sink("biosphere.leaf_c"));
    }

    #[test]
    fn source_is_unclamped_boundary() {
        let s = source("boundary.solar".to_string(), Quantity::Energy, 5.0, true).unwrap();
        assert_eq!(s.kind, StockKind::Boundary);
        assert!(s.unclamped);
        assert_eq!(s.unit, "J");
    }

    #[test]
    fn loss_sinks_default_covers_all_asserted() {
        let sinks = loss_sinks_default().unwrap();
        assert_eq!(sinks.len(), ASSERTED_QUANTITIES.len());
        assert!(sinks.contains_key("boundary.loss.carbon"));
        assert!(sinks.contains_key("boundary.loss.energy"));
    }
}
