//! The environment source resolver — the Rust port of `simcore.environment`.
//!
//! A flow calls [`Environment::get`] and cannot tell whether the value came from a
//! forcing schedule (evaluated at `t = n*dt`, integer `n`) or a sibling domain's
//! shared stock (read from the *same* immutable snapshot the flows read, #16). That
//! indistinguishability is the point.
//!
//! Binding model: [`Environment::get`] takes only `var`, so all per-step context
//! (which snapshot, which `dt`) is bound in first. [`SourceResolver`] holds the
//! build-once wiring (two disjoint var maps); [`SourceResolver::bind`] returns a
//! lightweight [`BoundEnvironment`] borrowing one snapshot for one derivative
//! evaluation. The integrator must bind to the **same** snapshot it passes to
//! `flow.evaluate` — that is the mechanism that makes #16 hold.

use std::collections::HashMap;

use crate::error::SimError;
use crate::ids::StockId;
use crate::state::State;

/// Resolves an environment variable name to a scalar value.
pub trait Environment {
    /// Resolve `var` to a finite scalar, or an error if it is unknown / non-finite.
    fn get(&self, var: &str) -> Result<f64, SimError>;
}

/// A forcing schedule: a pure function of the integer step `n` and `dt`. Passing `n`
/// *and* `dt` (rather than a precomputed `t`) keeps the integer visible (#14).
pub type Schedule = Box<dyn Fn(u64, f64) -> f64>;

/// A forcing [`Schedule`] returning `value` at every step. `value` is validated
/// finite here, at wiring time (Python `constant`), so a bad constant fails loudly at
/// construction rather than at the first `get`.
pub fn constant(value: f64) -> Result<Schedule, SimError> {
    if !value.is_finite() {
        return Err(SimError::Validation(format!(
            "constant forcing value is not finite: {value:?}"
        )));
    }
    Ok(Box::new(move |_n, _dt| value))
}

/// Build-once env wiring: `var -> forcing schedule` XOR `var -> shared stock`. The two
/// namespaces are disjoint — an overlap is a wiring bug rejected at construction.
pub struct SourceResolver {
    forcings: HashMap<String, Schedule>,
    shared: HashMap<String, StockId>,
}

impl SourceResolver {
    /// Construct the wiring, rejecting a var wired as both forcing and shared.
    pub fn new(
        forcings: HashMap<String, Schedule>,
        shared: HashMap<String, StockId>,
    ) -> Result<SourceResolver, SimError> {
        let mut overlap: Vec<&String> = forcings.keys().filter(|k| shared.contains_key(*k)).collect();
        if !overlap.is_empty() {
            overlap.sort();
            return Err(SimError::Validation(format!(
                "env var(s) wired as both forcing and shared stock: {overlap:?} \
                 (a var is forcing xor shared, decision #16)"
            )));
        }
        Ok(SourceResolver { forcings, shared })
    }

    /// An empty resolver (no forcings, no shared vars) — the common standalone case
    /// where flows read no env.
    pub fn empty() -> SourceResolver {
        SourceResolver {
            forcings: HashMap::new(),
            shared: HashMap::new(),
        }
    }

    /// Read-only `var -> Schedule` forcing wiring.
    pub fn forcings(&self) -> &HashMap<String, Schedule> {
        &self.forcings
    }

    /// Read-only `var -> StockId` shared-stock wiring.
    pub fn shared(&self) -> &HashMap<String, StockId> {
        &self.shared
    }

    /// Consume the resolver and yield back its owned wiring `(forcings, shared)`. The
    /// inverse of [`SourceResolver::new`] and the primitive a forcing perturbation
    /// rebuilds through: a [`Schedule`] is a non-`Clone` `Box<dyn Fn>`, so Rust cannot
    /// shallow-copy the `HashMap<String, Schedule>` the way Python's `{**resolver.forcings}`
    /// copies a dict of callables. A perturbation that swaps or adds one var's schedule
    /// therefore *consumes* the resolver, mutates the owned map, and rebuilds via
    /// [`SourceResolver::new`] (which re-checks the forcing⊕shared disjointness, #16).
    pub fn into_parts(self) -> (HashMap<String, Schedule>, HashMap<String, StockId>) {
        (self.forcings, self.shared)
    }

    /// Bind to one snapshot + `dt` for a single derivative evaluation. The bound view
    /// is lightweight — it holds references, copies nothing.
    pub fn bind<'a>(&'a self, snapshot: &'a State, dt: f64) -> BoundEnvironment<'a> {
        BoundEnvironment {
            resolver: self,
            snapshot,
            dt,
        }
    }
}

/// An [`Environment`] bound to one snapshot + `dt` for a single evaluation. Both
/// branches draw on the one bound snapshot, so forcing-time and shared reads are
/// mutually consistent; the caller cannot tell which branch answered.
pub struct BoundEnvironment<'a> {
    resolver: &'a SourceResolver,
    snapshot: &'a State,
    dt: f64,
}

impl Environment for BoundEnvironment<'_> {
    fn get(&self, var: &str) -> Result<f64, SimError> {
        if let Some(schedule) = self.resolver.forcings.get(var) {
            let value = schedule(self.snapshot.n, self.dt);
            // Only a forcing schedule can introduce NaN/Inf (stock amounts are
            // already finite), so guard it here.
            if !value.is_finite() {
                return Err(SimError::Validation(format!(
                    "forcing schedule for env var {var:?} returned non-finite value: {value:?}"
                )));
            }
            return Ok(value);
        }
        if let Some(sid) = self.resolver.shared.get(var) {
            // Reads the bound snapshot (#16). A missing stock is referential
            // integrity, resolve-time by design.
            return self
                .snapshot
                .stocks
                .get(sid)
                .map(|s| s.amount)
                .ok_or_else(|| {
                    SimError::Reference(format!(
                        "shared env var {var:?} points at missing stock {sid:?}"
                    ))
                });
        }
        Err(SimError::Reference(format!(
            "unknown env var {var:?} (wired as neither forcing nor shared stock)"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quantities::{Quantity, StockKind};
    use crate::state::Stock;
    use std::collections::BTreeMap;

    fn state_with(stock_id: &str, amount: f64) -> State {
        let s = Stock::new(
            stock_id.to_string(),
            "d".to_string(),
            Quantity::Carbon,
            "mol".to_string(),
            amount,
            StockKind::Pool,
            0.0,
            false,
            BTreeMap::new(),
        )
        .unwrap();
        State::new(3, BTreeMap::from([(stock_id.to_string(), s)]), 0, BTreeMap::new()).unwrap()
    }

    #[test]
    fn overlap_is_rejected() {
        let mut forcings: HashMap<String, Schedule> = HashMap::new();
        forcings.insert("x".to_string(), constant(1.0).unwrap());
        let shared = HashMap::from([("x".to_string(), "s".to_string())]);
        assert!(matches!(
            SourceResolver::new(forcings, shared),
            Err(SimError::Validation(_))
        ));
    }

    #[test]
    fn into_parts_yields_owned_wiring_for_rebuild() {
        // The forcing-perturbation primitive: decompose the resolver into its owned maps,
        // swap one schedule, rebuild (a Schedule is a non-Clone Box<dyn Fn>, so the maps
        // must be *moved* out, not borrowed-and-copied).
        let mut forcings: HashMap<String, Schedule> = HashMap::new();
        forcings.insert("f".to_string(), constant(2.5).unwrap());
        let shared = HashMap::from([("g".to_string(), "s".to_string())]);
        let resolver = SourceResolver::new(forcings, shared).unwrap();

        let (mut forcings, shared) = resolver.into_parts();
        forcings.insert("f".to_string(), constant(9.0).unwrap()); // swap the schedule
        let rebuilt = SourceResolver::new(forcings, shared).unwrap();

        let state = state_with("s", 7.0);
        let bound = rebuilt.bind(&state, 1.0);
        assert_eq!(bound.get("f").unwrap(), 9.0); // the swapped value
        assert_eq!(bound.get("g").unwrap(), 7.0); // shared preserved
    }

    #[test]
    fn forcing_and_shared_resolve() {
        let mut forcings: HashMap<String, Schedule> = HashMap::new();
        forcings.insert("f".to_string(), constant(2.5).unwrap());
        let shared = HashMap::from([("g".to_string(), "s".to_string())]);
        let resolver = SourceResolver::new(forcings, shared).unwrap();
        let state = state_with("s", 7.0);
        let bound = resolver.bind(&state, 1.0);
        assert_eq!(bound.get("f").unwrap(), 2.5);
        assert_eq!(bound.get("g").unwrap(), 7.0);
        assert!(matches!(bound.get("h"), Err(SimError::Reference(_))));
    }
}
