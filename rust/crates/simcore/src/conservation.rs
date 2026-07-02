//! Conservation ledger + the every-step balance gate — the Rust port of
//! `simcore.conservation`.
//!
//! Over the augmented system (modeled stocks + boundary reservoirs), per asserted
//! quantity the total mass across all stocks is unchanged step-to-step, within
//! tolerance: `residual_q = Σ (after[s] − before[s]) ≈ 0`. This module reasons about
//! **state deltas, not flows** (it does not reuse `flow::assert_flow_balanced`), so it
//! has its own composition fold; it shares only the tolerance constants and the
//! `Conservation` error variant.
//!
//! Per-stock deltas are accumulated within each `StockKind` partition in **sorted
//! stock-id order** (the `BTreeMap` iteration order) — float sums are non-associative,
//! so this is what makes the ledger bit-identical under registration shuffle (#15).

use std::collections::HashMap;

use crate::error::SimError;
use crate::quantities::{Quantity, StockKind, ASSERTED_QUANTITIES, BALANCE_ATOL, BALANCE_RTOL};
use crate::state::State;

/// Per-quantity conservation accounting for one step. `residual == boundary_delta +
/// stored_delta` (≡ total-mass Δ) always holds by construction.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantityLedger {
    pub quantity: Quantity,
    pub boundary_delta: f64,
    pub stored_delta: f64,
    pub residual: f64,
}

/// Per-quantity ledger for the step `before → after`, in name-sorted quantity order.
///
/// Returns [`SimError::Validation`] if `before`/`after` do not share the same stock-id
/// key set (Phase 0 never adds or removes stocks mid-run, so a mismatch is a bug).
pub fn compute_ledger(before: &State, after: &State) -> Result<Vec<QuantityLedger>, SimError> {
    if !same_key_set(before, after) {
        let before_only: Vec<&String> = before
            .stocks
            .keys()
            .filter(|k| !after.stocks.contains_key(*k))
            .collect();
        let after_only: Vec<&String> = after
            .stocks
            .keys()
            .filter(|k| !before.stocks.contains_key(*k))
            .collect();
        return Err(SimError::Validation(format!(
            "conservation ledger requires before/after to share the same stock ids; \
             Phase 0 never adds/removes stocks mid-run (before-only={before_only:?}, \
             after-only={after_only:?})"
        )));
    }
    let mut boundary: HashMap<Quantity, f64> = HashMap::new();
    let mut stored: HashMap<Quantity, f64> = HashMap::new();
    // BTreeMap iterates sorted by id — the canonical accumulation order (#15).
    for (sid, b) in &before.stocks {
        let delta = after.stocks[sid].amount - b.amount;
        let bucket = if b.kind == StockKind::Boundary {
            &mut boundary
        } else {
            &mut stored
        };
        for (q, coeff) in &b.composition {
            *bucket.entry(*q).or_insert(0.0) += delta * coeff;
        }
    }
    // Quantities present in either partition, in name-sorted order.
    let mut quantities: Vec<Quantity> = boundary
        .keys()
        .chain(stored.keys())
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    quantities.sort_by(|a, b| a.name().cmp(b.name()));
    Ok(quantities
        .into_iter()
        .map(|q| {
            let bd = boundary.get(&q).copied().unwrap_or(0.0);
            let sd = stored.get(&q).copied().unwrap_or(0.0);
            QuantityLedger {
                quantity: q,
                boundary_delta: bd,
                stored_delta: sd,
                residual: bd + sd,
            }
        })
        .collect())
}

fn same_key_set(before: &State, after: &State) -> bool {
    before.stocks.len() == after.stocks.len()
        && before.stocks.keys().all(|k| after.stocks.contains_key(k))
}

/// Raise [`SimError::Conservation`] if any asserted quantity's mass is not conserved
/// across the step (the every-step engine gate). `scale` is the transfer magnitude
/// (`max |per-stock Δ·coeff|`) for that quantity; `tol = atol + rtol*scale`.
pub fn assert_conserved(
    before: &State,
    after: &State,
    atol: f64,
    rtol: f64,
) -> Result<(), SimError> {
    let ledger: HashMap<Quantity, QuantityLedger> = compute_ledger(before, after)?
        .into_iter()
        .map(|ql| (ql.quantity, ql))
        .collect();
    let mut scale: HashMap<Quantity, f64> = HashMap::new();
    for (sid, b) in &before.stocks {
        let d = after.stocks[sid].amount - b.amount;
        for (q, coeff) in &b.composition {
            let acc = scale.entry(*q).or_insert(0.0);
            *acc = acc.max((d * coeff).abs());
        }
    }
    for quantity in ASSERTED_QUANTITIES {
        let ql = match ledger.get(&quantity) {
            Some(ql) => ql,
            None => continue, // quantity absent from the state — trivially conserved
        };
        let tol = atol + rtol * scale.get(&quantity).copied().unwrap_or(0.0);
        if ql.residual.abs() > tol {
            return Err(SimError::Conservation(format!(
                "conservation violated for {}: residual {:?} exceeds tolerance {tol:?} \
                 (boundary_delta={:?}, stored_delta={:?})",
                quantity.name(),
                ql.residual,
                ql.boundary_delta,
                ql.stored_delta
            )));
        }
    }
    Ok(())
}

/// The default-tolerance [`assert_conserved`] (using [`BALANCE_ATOL`] / [`BALANCE_RTOL`]).
pub fn assert_conserved_default(before: &State, after: &State) -> Result<(), SimError> {
    assert_conserved(before, after, BALANCE_ATOL, BALANCE_RTOL)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Stock;
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

    fn state(a: f64, b: f64) -> State {
        State::new(
            0,
            BTreeMap::from([("a".to_string(), pool("a", a)), ("b".to_string(), pool("b", b))]),
            0,
            BTreeMap::new(),
        )
        .unwrap()
    }

    #[test]
    fn balanced_transfer_conserves() {
        // a: 100 → 90, b: 0 → 10 — CARBON conserved.
        assert!(assert_conserved_default(&state(100.0, 0.0), &state(90.0, 10.0)).is_ok());
    }

    #[test]
    fn imbalanced_step_is_caught() {
        // a: 100 → 90, b: 0 → 5 — 5 mol CARBON vanished.
        let e = assert_conserved_default(&state(100.0, 0.0), &state(90.0, 5.0));
        assert!(matches!(e, Err(SimError::Conservation(_))));
    }

    fn co2(id: &str, amount: f64) -> Stock {
        // CO₂ = {Carbon:1, Oxygen:2} — the multi-quantity composition the ledger's
        // own `delta * coeff` fold must honour (cabin_gas / any biosphere CO₂ pool).
        Stock::new(
            id.to_string(),
            "d".to_string(),
            Quantity::Carbon,
            "mol".to_string(),
            amount,
            StockKind::Pool,
            0.0,
            false,
            BTreeMap::from([(Quantity::Carbon, 1.0), (Quantity::Oxygen, 2.0)]),
        )
        .unwrap()
    }

    #[test]
    fn compute_ledger_folds_composition_coeff() {
        // A CO₂ pool drops by 1 mol (nothing else changes). The OXYGEN delta must be
        // −2.0 (Δamount·coeff), not −1.0 — this is the exact `delta` vs `delta*coeff`
        // transcription a single-quantity scenario cannot catch (coeff = 1.0 hides it).
        let before = State::new(
            0,
            BTreeMap::from([("co2.a".to_string(), co2("co2.a", 10.0))]),
            0,
            BTreeMap::new(),
        )
        .unwrap();
        let after = State::new(
            0,
            BTreeMap::from([("co2.a".to_string(), co2("co2.a", 9.0))]),
            0,
            BTreeMap::new(),
        )
        .unwrap();
        let ledger: HashMap<Quantity, QuantityLedger> = compute_ledger(&before, &after)
            .unwrap()
            .into_iter()
            .map(|ql| (ql.quantity, ql))
            .collect();
        assert_eq!(ledger[&Quantity::Carbon].stored_delta, -1.0);
        assert_eq!(ledger[&Quantity::Oxygen].stored_delta, -2.0, "coeff-2 fold");
    }

    #[test]
    fn multi_quantity_balanced_transfer_conserves() {
        // co2.a → co2.b of 1 mol conserves both CARBON and OXYGEN (each side folds ·2
        // for oxygen), so the gate passes.
        let before = State::new(
            0,
            BTreeMap::from([
                ("co2.a".to_string(), co2("co2.a", 10.0)),
                ("co2.b".to_string(), co2("co2.b", 0.0)),
            ]),
            0,
            BTreeMap::new(),
        )
        .unwrap();
        let after = State::new(
            0,
            BTreeMap::from([
                ("co2.a".to_string(), co2("co2.a", 9.0)),
                ("co2.b".to_string(), co2("co2.b", 1.0)),
            ]),
            0,
            BTreeMap::new(),
        )
        .unwrap();
        assert!(assert_conserved_default(&before, &after).is_ok());
    }

    #[test]
    fn multi_quantity_imbalanced_step_is_caught() {
        // co2.a drops 1 mol but co2.b rises only 0.4 — both CARBON and OXYGEN break
        // (OXYGEN residual = −2 + 0.8 = −1.2), exercising the assert path over a
        // composition stock.
        let before = State::new(
            0,
            BTreeMap::from([
                ("co2.a".to_string(), co2("co2.a", 10.0)),
                ("co2.b".to_string(), co2("co2.b", 0.0)),
            ]),
            0,
            BTreeMap::new(),
        )
        .unwrap();
        let after = State::new(
            0,
            BTreeMap::from([
                ("co2.a".to_string(), co2("co2.a", 9.0)),
                ("co2.b".to_string(), co2("co2.b", 0.4)),
            ]),
            0,
            BTreeMap::new(),
        )
        .unwrap();
        assert!(matches!(
            assert_conserved_default(&before, &after),
            Err(SimError::Conservation(_))
        ));
    }

    #[test]
    fn keyset_mismatch_is_validation_error() {
        let before = state(1.0, 1.0);
        let after = State::new(
            0,
            BTreeMap::from([("a".to_string(), pool("a", 1.0))]),
            0,
            BTreeMap::new(),
        )
        .unwrap();
        assert!(matches!(
            compute_ledger(&before, &after),
            Err(SimError::Validation(_))
        ));
    }
}
