//! The observation surface: `observe(&State) -> Observation` — the Rust port of the
//! frozen `simcore.observation` (Phase-7's *conscious deferral*, revived in Phase-8 P8.2
//! as the foundation of the Godot display projection).
//!
//! `observe` is the **consumer-facing read** of a simulation — "what is in the tanks at
//! step `n`" — decoupled from [`State`]'s engine-internal shape so a UI / telemetry /
//! Godot front-end never has to reach into `State.stocks` and know about the RNG seed,
//! arbitration flags, or extinction thresholds.
//!
//! **It is a projection, not an aggregate.** [`StockObservation`] re-exposes only the
//! *observable* subset of each [`Stock`] and deliberately **drops engine internals**
//! (`rng_seed`, `extinction_threshold`, `unclamped`) and `kind`. The rule (mirrored from
//! the Python module): *if you cannot say what a consumer observes with a field, it is an
//! engine internal and does not belong.* Each kept field passes that test — `id` (which
//! tank), `domain` (grouping axis), `quantity` (what substance), `unit` (the amount's
//! canonical label), `amount` (the measurement). `kind` is dropped because its observable
//! half (BOUNDARY-vs-modeled) is redundant with `domain` and its unique half
//! (POOL-vs-POPULATION) is engine-behavioral (extinction eligibility) — an API-freeze
//! decision. **No aggregates** live here (no totals / per-domain rollups): those are the
//! Phase-8 *display* layer's job ([`crate`] stays the frozen surface; see
//! `station::display`), exactly as the Python `observation.py` keeps them out.
//!
//! **Plain-data, hashable.** Both types are plain structs of stdlib primitives + the core
//! enums, so an [`Observation`] is fully `==`-comparable and `Hash` — unlike [`State`]. and
//! it carries stocks in **canonical id-sorted order** (#15), so equal states observe equal
//! regardless of `State.stocks` insertion order (here free, since `State.stocks` is a
//! `BTreeMap` — but the port keeps the explicit sort so the guarantee is stated, not
//! incidental).

use crate::ids::{DomainId, StockId, UnitLabel};
use crate::quantities::Quantity;
use crate::state::State;

/// The observable subset of one [`Stock`] (engine-internal fields dropped).
///
/// See the module docs for why each field is observable and why
/// `extinction_threshold` / `unclamped` / `kind` are not.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StockObservation {
    pub id: StockId,
    pub domain: DomainId,
    pub quantity: Quantity,
    pub unit: UnitLabel,
    /// The measured amount. Held as raw bits for `Eq`/`Hash` (float has neither); read
    /// it back with [`StockObservation::amount`].
    amount_bits: u64,
}

impl StockObservation {
    /// The measured amount (in `unit`) — a bit-exact copy of the source `Stock.amount`.
    pub fn amount(&self) -> f64 {
        f64::from_bits(self.amount_bits)
    }
}

/// A plain-data, hashable read of a [`State`] (decoupled from its internals).
///
/// `n` is the integer step count (wall time is `n*dt`, but `dt` is not part of state —
/// decision #14). `stocks` are the per-stock observations in canonical id-sorted order
/// (#15).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Observation {
    pub n: u64,
    pub stocks: Vec<StockObservation>,
}

/// Project `state` to a plain-data [`Observation`] (the consumer read surface).
///
/// Re-exposes only the observable subset of each stock (dropping the RNG seed, the
/// extinction/arbitration controls, and `kind`) with stocks in **canonical id-sorted
/// order** (#15). `State.stocks` is a `BTreeMap`, so its `.values()` already yield
/// id-sorted stocks; the collect preserves that.
pub fn observe(state: &State) -> Observation {
    let stocks = state
        .stocks
        .values()
        .map(|stock| StockObservation {
            id: stock.id.clone(),
            domain: stock.domain.clone(),
            quantity: stock.quantity,
            unit: stock.unit.clone(),
            amount_bits: stock.amount.to_bits(),
        })
        .collect();
    Observation {
        n: state.n,
        stocks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quantities::StockKind;
    use crate::state::Stock;
    use std::collections::BTreeMap;

    fn pool(id: &str, amount: f64) -> Stock {
        Stock::new(
            id.to_string(),
            "bio".to_string(),
            Quantity::Carbon,
            Quantity::Carbon.canonical_unit(),
            amount,
            StockKind::Pool,
            0.0,
            false,
            BTreeMap::new(),
        )
        .unwrap()
    }

    fn pop(id: &str, amount: f64) -> Stock {
        // A non-default extinction_threshold: the projection must NOT leak it.
        Stock::new(
            id.to_string(),
            "bio".to_string(),
            Quantity::Carbon,
            Quantity::Carbon.canonical_unit(),
            amount,
            StockKind::Population,
            5.0,
            false,
            BTreeMap::new(),
        )
        .unwrap()
    }

    fn boundary(id: &str, amount: f64) -> Stock {
        // An unclamped BOUNDARY source: proves `unclamped` is dropped too.
        Stock::new(
            id.to_string(),
            "boundary".to_string(),
            Quantity::Energy,
            Quantity::Energy.canonical_unit(),
            amount,
            StockKind::Boundary,
            0.0,
            true,
            BTreeMap::new(),
        )
        .unwrap()
    }

    fn state_of(n: u64, stocks: Vec<Stock>, rng_seed: u64) -> State {
        State::new(
            n,
            stocks.into_iter().map(|s| (s.id.clone(), s)).collect(),
            rng_seed,
            BTreeMap::new(),
        )
        .unwrap()
    }

    #[test]
    fn copies_observable_fields_faithfully() {
        let state = state_of(
            7,
            vec![
                pool("bio.atmospheric_c", 1000.0),
                pop("bio.plant_c", 100.0),
                boundary("boundary.light", 1.0),
            ],
            42,
        );
        let obs = observe(&state);
        assert_eq!(obs.n, 7);
        for so in &obs.stocks {
            let src = &state.stocks[&so.id];
            assert_eq!(so.id, src.id);
            assert_eq!(so.domain, src.domain);
            assert_eq!(so.quantity, src.quantity);
            assert_eq!(so.unit, src.unit);
            assert_eq!(so.amount(), src.amount); // exact: a plain copy
        }
    }

    #[test]
    fn observe_empty_state() {
        let obs = observe(&state_of(0, vec![], 0));
        assert_eq!(obs.n, 0);
        assert!(obs.stocks.is_empty());
    }

    #[test]
    fn preserves_exact_amount() {
        // -0.0 keeps its sign, subnormals / max double survive (a bit-exact copy).
        for amount in [0.0, -0.0, 5e-324, f64::MAX] {
            let obs = observe(&state_of(0, vec![pool("bio.c", amount)], 0));
            assert_eq!(obs.stocks[0].amount().to_bits(), amount.to_bits());
        }
    }

    #[test]
    fn emits_stocks_in_canonical_id_order() {
        let obs = observe(&state_of(
            3,
            vec![
                pool("bio.atmospheric_c", 1.0),
                pop("bio.plant_c", 1.0),
                boundary("boundary.light", 1.0),
            ],
            0,
        ));
        let ids: Vec<&str> = obs.stocks.iter().map(|s| s.id.as_str()).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted);
    }

    #[test]
    fn insertion_order_independent() {
        let stocks = vec![
            pool("bio.atmospheric_c", 1.0),
            pop("bio.plant_c", 2.0),
            boundary("boundary.light", 3.0),
        ];
        let mut reversed = stocks.clone();
        reversed.reverse();
        let forward = observe(&state_of(2, stocks, 0));
        let backward = observe(&state_of(2, reversed, 0));
        assert_eq!(forward, backward);
        // Hashable + equal-hash for equal observations (the plain-data property).
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut ha = DefaultHasher::new();
        let mut hb = DefaultHasher::new();
        forward.hash(&mut ha);
        backward.hash(&mut hb);
        assert_eq!(ha.finish(), hb.finish());
    }

    #[test]
    fn rng_seed_does_not_change_observation() {
        // Two states differing only in the RNG seed observe equal (seed is dropped).
        let a = observe(&state_of(1, vec![pool("bio.c", 1.0)], 1));
        let b = observe(&state_of(1, vec![pool("bio.c", 1.0)], 999));
        assert_eq!(a, b);
    }

    #[test]
    fn n_participates_in_equality() {
        let a = observe(&state_of(3, vec![pool("bio.c", 1.0)], 0));
        let b = observe(&state_of(4, vec![pool("bio.c", 1.0)], 0));
        assert_ne!(a, b);
    }
}
