//! Arbitration backstop: single-pass min-scaling — the Rust port of
//! `simcore.arbitration`.
//!
//! A rare numerical guard, not the ecological mechanism. It catches explicit-
//! integration overshoot (a flow withdrawing more from a stock than it holds at the
//! start of the step). The demand sum is accumulated in **canonical flow-id order**
//! (the order the registry yields `results`), so the float result is bit-identical
//! under registration shuffle (#15).
//!
//! Euler/RK4 asymmetry: [`min_scaling`] (Euler) scales the offending whole flows and
//! counts firings; [`check_no_overdraw`] (RK4+) makes a needed `scale_f < 1` a hard
//! [`SimError::Arbitration`] — positivity under higher-order schemes must come from
//! the kinetics, since the single-evaluation conservation-safety proof does not carry
//! to a weighted sum of clamped stage derivatives.

use std::collections::HashMap;

use crate::error::SimError;
use crate::flow::{FlowResult, Leg};
use crate::ids::StockId;
use crate::state::Stock;

/// Per-flow scale factors aligned with `results` (canonical-order demand sum).
///
/// `results` MUST already be in canonical flow-id order — the per-stock `demand_s`
/// sum is accumulated in that order, which is what makes the float result stable under
/// registration shuffle (#15). A leg naming an unknown stock returns
/// [`SimError::Reference`].
fn scale_factors(
    results: &[FlowResult],
    stocks: &HashMap<&StockId, &Stock>,
) -> Result<Vec<f64>, SimError> {
    // demand_s over clamped stocks, summed in the given canonical order (#15).
    let mut demand: HashMap<StockId, f64> = HashMap::new();
    for result in results {
        for leg in &result.legs {
            if leg.amount < 0.0 {
                let stock = lookup(stocks, &leg.stock)?;
                if !stock.unclamped {
                    let acc = demand.entry(leg.stock.clone()).or_insert(0.0);
                    // Python: demand.get(..., 0.0) - leg.amount  (leg.amount < 0).
                    *acc -= leg.amount;
                }
            }
        }
    }
    // scale_s: per-stock, order-independent (each is a self-contained min of one ratio).
    let mut scale_s: HashMap<StockId, f64> = HashMap::new();
    for (sid, d) in &demand {
        let stock = lookup(stocks, sid)?;
        let s = if *d > 0.0 {
            1.0_f64.min(stock.amount / d)
        } else {
            1.0
        };
        scale_s.insert(sid.clone(), s);
    }
    let mut factors: Vec<f64> = Vec::with_capacity(results.len());
    for result in results {
        let mut f = 1.0_f64;
        for leg in &result.legs {
            if leg.amount < 0.0 {
                if let Some(s) = scale_s.get(&leg.stock) {
                    f = f.min(*s);
                }
            }
        }
        factors.push(f);
    }
    Ok(factors)
}

fn lookup<'a>(
    stocks: &HashMap<&StockId, &'a Stock>,
    sid: &StockId,
) -> Result<&'a Stock, SimError> {
    stocks
        .get(sid)
        .copied()
        .ok_or_else(|| SimError::Reference(format!("unknown stock {sid:?} in arbitration")))
}

/// Build the id→stock view arbitration needs (avoids re-borrowing the owned map).
fn view(stocks: &std::collections::BTreeMap<StockId, Stock>) -> HashMap<&StockId, &Stock> {
    stocks.iter().collect()
}

/// Euler backstop: scale the over-drawing whole flows; return `(scaled, fired)`.
///
/// `fired` counts the flows scaled this step (one per flow with `scale_f < 1`) — the
/// rationing-firing diagnostic the golden gate asserts `== 0` on a well-fed run. A flow
/// with `scale_f == 1` is returned unchanged. The returned list stays in the input's
/// canonical order.
pub fn min_scaling(
    results: &[FlowResult],
    stocks: &std::collections::BTreeMap<StockId, Stock>,
) -> Result<(Vec<FlowResult>, u64), SimError> {
    let v = view(stocks);
    let factors = scale_factors(results, &v)?;
    let mut scaled: Vec<FlowResult> = Vec::with_capacity(results.len());
    let mut fired: u64 = 0;
    for (result, f) in results.iter().zip(factors.iter()) {
        if *f < 1.0 {
            fired += 1;
            let mut legs: Vec<Leg> = Vec::with_capacity(result.legs.len());
            for leg in &result.legs {
                legs.push(Leg::new(leg.stock.clone(), leg.amount * f)?);
            }
            scaled.push(FlowResult::new(legs)?);
        } else {
            scaled.push(result.clone());
        }
    }
    Ok((scaled, fired))
}

/// RK4+ backstop: return [`SimError::Arbitration`] if any flow needs `scale_f < 1`.
pub fn check_no_overdraw(
    results: &[FlowResult],
    stocks: &std::collections::BTreeMap<StockId, Stock>,
) -> Result<(), SimError> {
    let v = view(stocks);
    for (i, f) in scale_factors(results, &v)?.iter().enumerate() {
        if *f < 1.0 {
            return Err(SimError::Arbitration(format!(
                "flow #{i} (canonical order) would over-draw a stock (scale_f={f:?} < 1) \
                 under a higher-order scheme; min-scaling is Euler-only — positivity \
                 under RK4+ must come from the kinetics, not the backstop (too-large dt \
                 or mis-scaled kinetics)"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quantities::{Quantity, StockKind};
    use std::collections::BTreeMap;

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
    fn no_overdraw_leaves_flows_unscaled() {
        let stocks = BTreeMap::from([("a".to_string(), pool("a", 100.0))]);
        let results = vec![FlowResult::new(vec![Leg::new("a".to_string(), -10.0).unwrap()]).unwrap()];
        let (scaled, fired) = min_scaling(&results, &stocks).unwrap();
        assert_eq!(fired, 0);
        assert_eq!(scaled[0].legs[0].amount, -10.0);
        assert!(check_no_overdraw(&results, &stocks).is_ok());
    }

    #[test]
    fn overdraw_scales_and_counts() {
        // Two flows each want 60 from a stock holding 100 → demand 120, scale 100/120.
        let stocks = BTreeMap::from([
            ("a".to_string(), pool("a", 100.0)),
            ("snk".to_string(), pool("snk", 0.0)),
        ]);
        let f1 = FlowResult::new(vec![
            Leg::new("a".to_string(), -60.0).unwrap(),
            Leg::new("snk".to_string(), 60.0).unwrap(),
        ])
        .unwrap();
        let f2 = f1.clone();
        let results = vec![f1, f2];
        let (scaled, fired) = min_scaling(&results, &stocks).unwrap();
        assert_eq!(fired, 2);
        let scale = 100.0_f64 / 120.0;
        assert_eq!(scaled[0].legs[0].amount, -60.0 * scale);
        // RK4 refuses to scale — it hard-errors instead.
        assert!(matches!(
            check_no_overdraw(&results, &stocks),
            Err(SimError::Arbitration(_))
        ));
    }

    #[test]
    fn unclamped_source_is_never_throttled() {
        let src = Stock::new(
            "src".to_string(),
            "boundary".to_string(),
            Quantity::Carbon,
            "mol".to_string(),
            0.0,
            StockKind::Boundary,
            0.0,
            true,
            BTreeMap::new(),
        )
        .unwrap();
        let stocks = BTreeMap::from([("src".to_string(), src)]);
        // A huge withdrawal from an unclamped source imposes no constraint.
        let results = vec![FlowResult::new(vec![Leg::new("src".to_string(), -1e9).unwrap()]).unwrap()];
        let (_, fired) = min_scaling(&results, &stocks).unwrap();
        assert_eq!(fired, 0);
    }
}
