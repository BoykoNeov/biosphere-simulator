//! The flow Registry — the Rust port of `simcore.registry`.
//!
//! Build-once structural config: it holds the flow set (and optional aux processes),
//! rejects duplicate ids, and exposes **canonical id-sorted iteration** — the
//! registration-order-independence guarantee (#15): shuffling the registration list
//! yields bit-identical iteration. It also derives a domain index
//! `DomainId -> {StockId}` over the initial stocks.
//!
//! The registry does not validate that legs reference known stocks — legs do not exist
//! until `evaluate` (that check is the integrator apply path's job).

use std::collections::{BTreeMap, BTreeSet};

use crate::auxiliary::AuxProcess;
use crate::error::SimError;
use crate::flow::Flow;
use crate::ids::{DomainId, StockId};
use crate::state::Stock;

/// An immutable, build-once set of flows (+ aux processes) plus a stock index.
pub struct Registry {
    flows: Vec<Box<dyn Flow>>,
    aux_processes: Vec<Box<dyn AuxProcess>>,
    domain_index: BTreeMap<DomainId, BTreeSet<StockId>>,
}

impl Registry {
    /// Construct a registry, sorting flows/aux by id and rejecting duplicate ids
    /// (Python `Registry.__init__`).
    pub fn new(
        flows: Vec<Box<dyn Flow>>,
        stocks: &BTreeMap<StockId, Stock>,
        aux_processes: Vec<Box<dyn AuxProcess>>,
    ) -> Result<Registry, SimError> {
        let flows = sort_dedup_flows(flows)?;
        let aux_processes = sort_dedup_aux(aux_processes)?;
        let mut domain_index: BTreeMap<DomainId, BTreeSet<StockId>> = BTreeMap::new();
        for stock in stocks.values() {
            domain_index
                .entry(stock.domain.clone())
                .or_default()
                .insert(stock.id.clone());
        }
        Ok(Registry {
            flows,
            aux_processes,
            domain_index,
        })
    }

    /// Convenience: a flows-only registry (no aux processes) — the Phase-0 shape.
    pub fn flows_only(
        flows: Vec<Box<dyn Flow>>,
        stocks: &BTreeMap<StockId, Stock>,
    ) -> Result<Registry, SimError> {
        Registry::new(flows, stocks, Vec::new())
    }

    /// The flows in canonical id-sorted order.
    pub fn flows(&self) -> &[Box<dyn Flow>] {
        &self.flows
    }

    /// Consume the registry and yield back its owned `(flows, aux_processes)` (in
    /// canonical id-sorted order). The inverse of [`Registry::new`] and the primitive a
    /// flow-modifying perturbation harness rebuilds through: Rust cannot clone a
    /// `Box<dyn Flow>` out of a `&Registry` (unlike Python's `list(registry.flows)`), so
    /// a perturbation that wraps or appends a flow *consumes* the registry, transforms
    /// the owned vec, and rebuilds via [`Registry::new`]. The derived `domain_index` is
    /// dropped — `new` re-derives it over the (possibly changed) stock set.
    // The `(Vec<Box<dyn Flow>>, Vec<Box<dyn AuxProcess>>)` tuple is exactly the two owned
    // fields it returns (mirroring `new`'s params); a type alias would obscure more than it
    // clarifies here.
    #[allow(clippy::type_complexity)]
    pub fn into_parts(self) -> (Vec<Box<dyn Flow>>, Vec<Box<dyn AuxProcess>>) {
        (self.flows, self.aux_processes)
    }

    /// The aux processes in canonical id-sorted order (empty if none).
    pub fn aux_processes(&self) -> &[Box<dyn AuxProcess>] {
        &self.aux_processes
    }

    /// Read-only `DomainId -> {StockId}` over the initial stocks.
    pub fn domain_index(&self) -> &BTreeMap<DomainId, BTreeSet<StockId>> {
        &self.domain_index
    }

    /// The number of flows.
    pub fn len(&self) -> usize {
        self.flows.len()
    }

    /// Whether the registry has no flows.
    pub fn is_empty(&self) -> bool {
        self.flows.is_empty()
    }
}

fn sort_dedup_flows(mut flows: Vec<Box<dyn Flow>>) -> Result<Vec<Box<dyn Flow>>, SimError> {
    flows.sort_by(|a, b| a.id().cmp(b.id()));
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for flow in &flows {
        if !seen.insert(flow.id().to_string()) {
            return Err(SimError::Validation(format!(
                "Registry has a duplicate FlowId {:?}",
                flow.id()
            )));
        }
    }
    Ok(flows)
}

fn sort_dedup_aux(
    mut aux: Vec<Box<dyn AuxProcess>>,
) -> Result<Vec<Box<dyn AuxProcess>>, SimError> {
    aux.sort_by(|a, b| a.id().cmp(b.id()));
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for proc in &aux {
        if !seen.insert(proc.id().to_string()) {
            return Err(SimError::Validation(format!(
                "Registry has a duplicate AuxId {:?}",
                proc.id()
            )));
        }
    }
    Ok(aux)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::Environment;
    use crate::flow::{FlowResult, Leg};
    use crate::quantities::{Quantity, StockKind};
    use crate::state::State;

    struct NoopFlow(&'static str);
    impl Flow for NoopFlow {
        fn id(&self) -> &str {
            self.0
        }
        fn evaluate(
            &self,
            _snapshot: &State,
            _env: &dyn Environment,
            _dt: f64,
        ) -> Result<FlowResult, SimError> {
            Ok(FlowResult::new(vec![Leg::new("x".to_string(), 0.0).unwrap()]).unwrap())
        }
    }

    fn stock(id: &str) -> Stock {
        Stock::new(
            id.to_string(),
            "biosphere".to_string(),
            Quantity::Carbon,
            "mol".to_string(),
            1.0,
            StockKind::Pool,
            0.0,
            false,
            BTreeMap::new(),
        )
        .unwrap()
    }

    #[test]
    fn flows_iterate_id_sorted_regardless_of_registration_order() {
        let stocks = BTreeMap::from([("x".to_string(), stock("x"))]);
        let reg = Registry::flows_only(
            vec![
                Box::new(NoopFlow("z.flow")),
                Box::new(NoopFlow("a.flow")),
                Box::new(NoopFlow("m.flow")),
            ],
            &stocks,
        )
        .unwrap();
        let ids: Vec<&str> = reg.flows().iter().map(|f| f.id()).collect();
        assert_eq!(ids, ["a.flow", "m.flow", "z.flow"]);
    }

    #[test]
    fn into_parts_yields_owned_flows_for_rebuild() {
        // The perturbation-harness primitive: decompose a registry into its owned flows,
        // and rebuild via `new` (Rust cannot clone a Box<dyn Flow> out of a &Registry).
        let stocks = BTreeMap::from([("x".to_string(), stock("x"))]);
        let reg = Registry::flows_only(
            vec![Box::new(NoopFlow("z.flow")), Box::new(NoopFlow("a.flow"))],
            &stocks,
        )
        .unwrap();
        let (mut flows, aux) = reg.into_parts();
        assert_eq!(flows.iter().map(|f| f.id()).collect::<Vec<_>>(), ["a.flow", "z.flow"]);
        assert!(aux.is_empty());
        // Rebuild with an extra flow — the harness pattern (append, re-sort).
        flows.push(Box::new(NoopFlow("m.flow")));
        let rebuilt = Registry::new(flows, &stocks, aux).unwrap();
        let ids: Vec<&str> = rebuilt.flows().iter().map(|f| f.id()).collect();
        assert_eq!(ids, ["a.flow", "m.flow", "z.flow"]);
    }

    #[test]
    fn duplicate_flow_id_is_rejected() {
        let stocks = BTreeMap::from([("x".to_string(), stock("x"))]);
        let e = Registry::flows_only(
            vec![Box::new(NoopFlow("dup")), Box::new(NoopFlow("dup"))],
            &stocks,
        );
        assert!(matches!(e, Err(SimError::Validation(_))));
    }
}
