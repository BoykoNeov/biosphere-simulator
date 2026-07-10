//! Phase-8 (P8.4) **flow inspection** — the flow-level slice of the display projection.
//! Where [`crate::display`] answers "what is in the tanks," this answers "where is
//! matter/energy *moving* this step, and which flow is doing it" — the read a player uses
//! to select a stock and see the contributing flows and their legs (roadmap: *inspect
//! flows*).
//!
//! **What it evaluates.** [`inspect_flows`] binds the resolver to the current [`State`] +
//! `dt` and calls [`Flow::evaluate`](simcore::flow::Flow::evaluate) on every flow in the
//! registry's canonical id-sorted order — the **exact same** evaluation the next
//! [`EulerIntegrator::step_report`](simcore::integrator::EulerIntegrator::step_report)
//! performs for its `k1`. So the inspected legs are, by construction, precisely the
//! per-step increments the next step applies (fidelity is not approximated — it is the
//! same code path at the same state). The truthfulness teeth in the tests below confirm
//! it: the per-stock sum of the inspected legs, added to the current amount, reproduces the
//! amount after `step()`.
//!
//! **Raw per-flow legs, not post-apply deltas.** The legs are each flow's own evaluated
//! request (`dt·rate`), so `before + Σ(legs) == after` (the fidelity teeth) rests on **two**
//! well-fed assumptions, both true for every Phase-8 palette scenario:
//! * **`rationed == 0`** — no flow was scaled by the Euler backstop's proportional
//!   min-scaling. **Seam for Step 5:** once perturbations add brownout / leaks, a *rationed*
//!   flow's raw legs no longer equal what moved (arbitration scales the whole flow); the
//!   module then revisits surfacing the per-flow `scale_f`.
//! * **no extinction** — no POPULATION stock snapped to 0 this step. Extinction routes the
//!   snapped residual to a loss-sink *after* the flows apply, so that delta is absent from
//!   the legs. The palette stocks are all POOL (no extinction), so this holds.
//!
//! Neither is pre-solved here (no scaling / extinction plumbing while every palette run is
//! well-fed and POOL-only) — they are named so a later step that breaks them knows to look.
//!
//! **Single-rate only (advisor).** A two-rate (`greenhouse` / `sealed`) session steps a
//! *master day*: the biosphere flows (photosynthesis, allocation, respiration — the ones
//! that actually move carbon) live in the **slow** registry stepped once per day, while the
//! **fast** registry is only the per-second cabin chemistry. Inspecting the fast registry
//! alone would show a greenhouse player everything *except* the plant — complete-looking
//! but silently wrong. So flow inspection is scoped to single-rate sessions here
//! ([`crate::session::SimSession::inspect_flows`] returns `None` for two-rate), exactly as
//! P8.2 deferred the two-rate scalar readouts. A future step that wants two-rate inspection
//! must surface both registries as *separately-labeled rate groups* (per-day biosphere,
//! per-second cabin), never summed across.
//!
//! **Zero parity concern.** This is a display read, derived from the exact `State`; nothing
//! feeds back into stepping and no golden pins it. Plain decimal float JSON (the bit-exact
//! hex-float codec stays on [`simcore::snapshot`]).

use simcore::environment::SourceResolver;
use simcore::error::SimError;
use simcore::flow::Leg;
use simcore::registry::Registry;
use simcore::state::State;

use crate::display::{push_f64, push_json_string};

/// One flow's evaluated legs at a moment (the flow-centric row). `legs` are in the flow's
/// own emission order; a flow nets its own touches into at most one leg per stock
/// ([`FlowResult`](simcore::flow::FlowResult) invariant).
#[derive(Debug, Clone, PartialEq)]
pub struct InspectedFlow {
    /// The canonical flow id (e.g. `power.solar_charge`).
    pub id: String,
    /// The per-step legs (`amount` per dt in the stock's canonical unit; `> 0` deposits,
    /// `< 0` withdraws).
    pub legs: Vec<Leg>,
}

/// The flow-level projection of one simulation moment: every flow with its evaluated legs,
/// in canonical id order. Built by [`inspect_flows`].
#[derive(Debug, Clone, PartialEq)]
pub struct FlowInspection {
    /// The integer step count of the inspected state.
    pub n: u64,
    /// Every flow's evaluated legs, in canonical (flow-id-sorted) order.
    pub flows: Vec<InspectedFlow>,
}

/// Evaluate every flow in `registry` against `state`/`resolver`/`dt` and collect its legs —
/// the flow-level display read. Mirrors the integrator's private `evaluate_all`: binds the
/// resolver to the **same** `state` + `dt` and iterates [`Registry::flows`] in canonical
/// order, so the result is exactly the next Euler step's `k1`.
///
/// Errors only if a flow's `evaluate` errors (a missing env var / non-finite forcing) —
/// which cannot happen for a session that steps successfully, since this is the same
/// evaluation the next `step()` runs.
pub fn inspect_flows(
    registry: &Registry,
    state: &State,
    resolver: &SourceResolver,
    dt: f64,
) -> Result<FlowInspection, SimError> {
    let bound = resolver.bind(state, dt);
    let mut flows = Vec::with_capacity(registry.len());
    for flow in registry.flows() {
        let result = flow.evaluate(state, &bound, dt)?;
        flows.push(InspectedFlow {
            id: flow.id().to_string(),
            legs: result.legs,
        });
    }
    Ok(FlowInspection { n: state.n, flows })
}

impl FlowInspection {
    /// The flows that touch `stock`, each with its signed leg amount — the "select a stock,
    /// see the contributing flows" query. Flows are returned in canonical id order (the
    /// order they appear in [`FlowInspection::flows`]); each flow contributes at most one
    /// entry (a flow nets its own touches on a stock into one leg).
    pub fn flows_touching(&self, stock: &str) -> Vec<(&str, f64)> {
        let mut out = Vec::new();
        for flow in &self.flows {
            for leg in &flow.legs {
                if leg.stock == stock {
                    out.push((flow.id.as_str(), leg.amount));
                }
            }
        }
        out
    }

    /// Serialize to plain-float JSON for the Godot flow-inspection panel:
    /// `{"n":..,"flows":[{"id":"power.solar_charge","legs":[{"stock":"..","amount":..}]}]}`.
    /// Plain decimal floats (this is a zero-parity display read; the hex-float codec stays
    /// on [`simcore::snapshot`]). Reuses the crate-local JSON emit helpers from
    /// [`crate::display`].
    pub fn to_json(&self) -> String {
        let mut out = String::from("{");
        out.push_str(&format!("\"n\":{},", self.n));
        out.push_str("\"flows\":[");
        for (i, flow) in self.flows.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push('{');
            push_json_string(&mut out, "id");
            out.push(':');
            push_json_string(&mut out, &flow.id);
            out.push(',');
            push_json_string(&mut out, "legs");
            out.push_str(":[");
            for (j, leg) in flow.legs.iter().enumerate() {
                if j > 0 {
                    out.push(',');
                }
                out.push('{');
                push_json_string(&mut out, "stock");
                out.push(':');
                push_json_string(&mut out, &leg.stock);
                out.push(',');
                push_json_string(&mut out, "amount");
                out.push(':');
                push_f64(&mut out, leg.amount);
                out.push('}');
            }
            out.push(']');
            out.push('}');
        }
        out.push_str("]}");
        out
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use simcore::environment::{constant, Environment, SourceResolver};
    use simcore::flow::{Flow, FlowResult};
    use simcore::integrator::EulerIntegrator;
    use simcore::quantities::{Quantity, StockKind};
    use simcore::registry::Registry;
    use simcore::state::{State, Stock};

    use super::*;

    /// A forced inflow `source -> pool` reading a rate off `env` (the SolarCharge shape,
    /// minus the loss leg): a boundary source is drawn, the pool deposited.
    struct ForcedInflow;
    impl Flow for ForcedInflow {
        fn id(&self) -> &str {
            "test.inflow"
        }
        fn evaluate(
            &self,
            _snapshot: &State,
            env: &dyn Environment,
            dt: f64,
        ) -> Result<FlowResult, SimError> {
            let rate = env.get("inflow_w")?;
            let amount = rate * dt;
            FlowResult::new(vec![
                Leg::new("boundary.source".to_string(), -amount)?,
                Leg::new("d.pool".to_string(), amount)?,
            ])
        }
    }

    /// A donor-controlled leak `pool -> sink` (`k·pool·dt`) — a second flow that also
    /// touches `d.pool`, so `flows_touching("d.pool")` returns two entries and the reduce
    /// sums two legs on one stock.
    struct Leak;
    impl Flow for Leak {
        fn id(&self) -> &str {
            "test.leak"
        }
        fn evaluate(
            &self,
            snapshot: &State,
            _env: &dyn Environment,
            dt: f64,
        ) -> Result<FlowResult, SimError> {
            let pool = snapshot.stocks["d.pool"].amount;
            let leak = 0.01 * pool * dt;
            FlowResult::new(vec![
                Leg::new("d.pool".to_string(), -leak)?,
                Leg::new("boundary.sink".to_string(), leak)?,
            ])
        }
    }

    fn boundary(id: &str) -> Stock {
        // An unclamped BOUNDARY reservoir (source/sink), like SolarCharge's endpoints.
        Stock::new(
            id.to_string(),
            "boundary".to_string(),
            Quantity::Energy,
            "J".to_string(),
            0.0,
            StockKind::Boundary,
            0.0,
            true,
            BTreeMap::new(),
        )
        .unwrap()
    }

    fn pool(id: &str, amount: f64) -> Stock {
        Stock::new(
            id.to_string(),
            "d".to_string(),
            Quantity::Energy,
            "J".to_string(),
            amount,
            StockKind::Pool,
            0.0,
            false,
            BTreeMap::new(),
        )
        .unwrap()
    }

    fn setup() -> (Registry, State, SourceResolver, f64) {
        let stocks = BTreeMap::from([
            ("boundary.source".to_string(), boundary("boundary.source")),
            ("boundary.sink".to_string(), boundary("boundary.sink")),
            ("d.pool".to_string(), pool("d.pool", 100.0)),
        ]);
        let state = State::new(5, stocks.clone(), 0, BTreeMap::new()).unwrap();
        let registry =
            Registry::flows_only(vec![Box::new(ForcedInflow), Box::new(Leak)], &stocks).unwrap();
        let mut forcings = std::collections::HashMap::new();
        forcings.insert("inflow_w".to_string(), constant(3.0).unwrap());
        let resolver = SourceResolver::new(forcings, std::collections::HashMap::new()).unwrap();
        (registry, state, resolver, 2.0)
    }

    #[test]
    fn inspects_every_flow_in_canonical_order() {
        let (registry, state, resolver, dt) = setup();
        let insp = inspect_flows(&registry, &state, &resolver, dt).unwrap();
        assert_eq!(insp.n, 5);
        let ids: Vec<&str> = insp.flows.iter().map(|f| f.id.as_str()).collect();
        assert_eq!(ids, ["test.inflow", "test.leak"]); // id-sorted
        // ForcedInflow: rate 3 * dt 2 = 6 into d.pool, -6 from boundary.source.
        assert_eq!(insp.flows[0].legs.len(), 2);
    }

    #[test]
    fn flows_touching_finds_every_contributor() {
        let (registry, state, resolver, dt) = setup();
        let insp = inspect_flows(&registry, &state, &resolver, dt).unwrap();
        // Both flows touch d.pool: inflow deposits +6, leak withdraws -0.01*100*2 = -2.
        let touching = insp.flows_touching("d.pool");
        assert_eq!(touching.len(), 2);
        assert_eq!(touching[0], ("test.inflow", 6.0));
        assert_eq!(touching[1], ("test.leak", -2.0));
        // A stock only one flow touches.
        assert_eq!(insp.flows_touching("boundary.source"), [("test.inflow", -6.0)]);
        // An untouched id → empty.
        assert!(insp.flows_touching("nope").is_empty());
    }

    /// The truthfulness teeth (advisor #2): the inspected legs are exactly what the next
    /// step applies. Inspect at the current state, then `step_report`, and confirm that for
    /// every stock `before + Σ(inspected legs) == after` — i.e. the inspection did not lie
    /// about where matter/energy moves. Exact here (well-fed, `rationed == 0`, so raw legs
    /// == applied deltas, and the fold order matches the integrator's `reduce`).
    #[test]
    fn inspected_legs_reconstruct_the_applied_step_delta() {
        let (registry, state, resolver, dt) = setup();
        let insp = inspect_flows(&registry, &state, &resolver, dt).unwrap();

        let integrator = EulerIntegrator::new(registry);
        let report = integrator.step_report(&state, &resolver, dt).unwrap();
        assert_eq!(report.rationed, 0, "well-fed: raw legs must equal applied");

        for (sid, before) in &state.stocks {
            let sum: f64 = insp.flows_touching(sid).iter().map(|(_, a)| a).sum();
            let after = report.state.stocks[sid].amount;
            assert!(
                (before.amount + sum - after).abs() <= 1e-12 * after.abs() + 1e-12,
                "flow inspection lied about {sid}: before {} + Σlegs {sum} != after {after}",
                before.amount,
            );
        }
    }

    #[test]
    fn json_is_wellformed_and_carries_the_legs() {
        let (registry, state, resolver, dt) = setup();
        let json = inspect_flows(&registry, &state, &resolver, dt).unwrap().to_json();
        assert!(json.starts_with('{') && json.ends_with('}'));
        assert!(json.contains("\"n\":5"));
        assert!(json.contains("\"id\":\"test.inflow\""));
        assert!(json.contains("\"stock\":\"d.pool\""));
        assert!(json.contains("\"amount\":6")); // the +6 inflow leg
    }
}
