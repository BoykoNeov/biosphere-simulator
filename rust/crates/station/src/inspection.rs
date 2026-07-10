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
//! **Raw legs (requested) + a per-flow `scale` (rationing), so the view stays truthful under
//! failure.** Each [`InspectedFlow`] keeps its flow's own evaluated request
//! ([`InspectedFlow::legs`], `dt·rate` — what it *wants* to move) **and** the fraction the
//! Euler backstop would actually apply ([`InspectedFlow::scale`]). The amount that moves is
//! `leg · scale`, so the identity is `before + Σ(scale·leg) == after`
//! ([`FlowInspection::applied_delta`]) — it holds even when a flow is **rationed**. This
//! discharges the P8.4 seam: Phase-8 Step 5 added perturbations (a deep brownout empties the
//! battery ⇒ `LoadDraw` rations on the single-rate `station` palette entry), so `scale < 1`
//! is now reachable through the palette. Raw legs are deliberately kept un-scaled (not
//! pre-multiplied) so a "requested vs delivered" panel can show the shortfall — the signal a
//! failure-observation game wants; the rationing `scale` is the annotation that makes it
//! honest.
//!
//! One assumption remains, and it stays true here: **no extinction** — a POPULATION stock
//! snapping to 0 routes its residual to a loss-sink *after* the flows apply, so that delta
//! would be absent from the legs. The single-rate palette (`cabin_gas`, `station`) is
//! POOL-only (no extinction), and two-rate inspection returns `None` (below), so this cannot
//! bite; it is named so a future step that surfaces a POPULATION stock through inspection
//! knows to look.
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

/// One flow's evaluated legs at a moment (the flow-centric row). `legs` are the flow's
/// **requested** transfer in its own emission order; a flow nets its own touches into at
/// most one leg per stock ([`FlowResult`](simcore::flow::FlowResult) invariant). `scale`
/// is the fraction the Euler backstop would actually apply this step (see the field doc) —
/// `legs · scale` is what *moves*.
#[derive(Debug, Clone, PartialEq)]
pub struct InspectedFlow {
    /// The canonical flow id (e.g. `power.solar_charge`).
    pub id: String,
    /// The per-step **requested** legs (`amount` per dt in the stock's canonical unit;
    /// `> 0` deposits, `< 0` withdraws). These are the flow's own evaluated request — what
    /// it *wants* to move. Under rationing the amount actually applied is `leg · scale`;
    /// the raw request is kept (not pre-scaled) so a "requested vs delivered" panel can show
    /// the shortfall — the signal a failure-observation game wants.
    pub legs: Vec<Leg>,
    /// The per-flow scale factor the Euler min-scaling backstop would apply to this whole
    /// flow this step: `1.0` when the flow is unthrottled, `< 1.0` when it over-draws a
    /// clamped stock (the same factor [`min_scaling`](simcore::arbitration::min_scaling)
    /// folds into the applied legs). So the amount that actually moves for each leg is
    /// `leg.amount · scale`. On a well-fed run this is always `1.0`; a deep brownout that
    /// empties the battery drives `LoadDraw`'s scale below 1 — the reachable rationing case.
    pub scale: f64,
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
    let mut results = Vec::with_capacity(registry.len());
    let mut ids = Vec::with_capacity(registry.len());
    for flow in registry.flows() {
        results.push(flow.evaluate(state, &bound, dt)?);
        ids.push(flow.id().to_string());
    }
    // The same per-flow scale the next Euler step's min-scaling backstop applies — computed
    // over the *same* evaluated results and start-of-step stocks. `1.0` unless a flow
    // over-draws a clamped stock (the reachable rationing case; a display read, never fed
    // back into stepping).
    let factors = simcore::arbitration::scale_factors(&results, &state.stocks)?;
    let flows = ids
        .into_iter()
        .zip(results)
        .zip(factors)
        .map(|((id, result), scale)| InspectedFlow {
            id,
            legs: result.legs,
            scale,
        })
        .collect();
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

    /// The net amount actually applied to `stock` this step — `Σ (scale · leg.amount)` over
    /// the flows touching it, in canonical order. Unlike summing [`flows_touching`] (which
    /// reports the *requested* legs), this folds in each flow's rationing `scale`, so
    /// `before + applied_delta(stock) == after` holds **even when a flow is rationed** —
    /// this is the same quantity the integrator's `reduce(min_scaling(...))` applies. On a
    /// well-fed run (`scale == 1` everywhere) it equals the raw sum. This is what discharges
    /// the P8.4 "raw legs == applied" seam once perturbations can drive `scale < 1`.
    pub fn applied_delta(&self, stock: &str) -> f64 {
        let mut acc = 0.0;
        for flow in &self.flows {
            for leg in &flow.legs {
                if leg.stock == stock {
                    acc += flow.scale * leg.amount;
                }
            }
        }
        acc
    }

    /// Serialize to plain-float JSON for the Godot flow-inspection panel:
    /// `{"n":..,"flows":[{"id":"power.solar_charge","scale":1.0,"legs":[{"stock":"..","amount":..}]}]}`.
    /// `scale` is the per-flow rationing factor (`1.0` unthrottled, `< 1.0` rationed) so the
    /// panel can show requested (`legs`) vs delivered (`legs · scale`). Plain decimal floats
    /// (this is a zero-parity display read; the hex-float codec stays on
    /// [`simcore::snapshot`]). Reuses the crate-local JSON emit helpers from [`crate::display`].
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
            push_json_string(&mut out, "scale");
            out.push(':');
            push_f64(&mut out, flow.scale);
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

    /// A flow that unconditionally withdraws 8 from `d.pool` into `boundary.sink` — used to
    /// force the Euler backstop to ration (two of these over a pool of 10).
    struct BigDraw(&'static str);
    impl Flow for BigDraw {
        fn id(&self) -> &str {
            self.0
        }
        fn evaluate(
            &self,
            _snapshot: &State,
            _env: &dyn Environment,
            _dt: f64,
        ) -> Result<FlowResult, SimError> {
            FlowResult::new(vec![
                Leg::new("d.pool".to_string(), -8.0)?,
                Leg::new("boundary.sink".to_string(), 8.0)?,
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
        assert_eq!(report.rationed, 0, "well-fed: every scale == 1");
        assert!(insp.flows.iter().all(|f| f.scale == 1.0));

        for (sid, before) in &state.stocks {
            let applied = insp.applied_delta(sid);
            let after = report.state.stocks[sid].amount;
            assert!(
                (before.amount + applied - after).abs() <= 1e-12 * after.abs() + 1e-12,
                "flow inspection lied about {sid}: before {} + Δ {applied} != after {after}",
                before.amount,
            );
        }
    }

    /// The seam discharged (advisor): under rationing (`scale < 1`) the raw legs no longer
    /// equal what moved, but `before + Σ(scale·leg) == after` still holds. Build a flow that
    /// over-draws a clamped pool so the Euler backstop scales it, and confirm the identity
    /// via [`applied_delta`], plus that the raw-leg sum would *disagree* (proving the scale is
    /// load-bearing, not decorative).
    #[test]
    fn inspection_is_truthful_when_a_flow_is_rationed() {
        // A pool holding 10, two flows each requesting -8 → demand 16 > 10, so both scale to
        // 10/16 = 0.625.
        let stocks = BTreeMap::from([
            ("boundary.sink".to_string(), boundary("boundary.sink")),
            ("d.pool".to_string(), pool("d.pool", 10.0)),
        ]);
        let state = State::new(0, stocks.clone(), 0, BTreeMap::new()).unwrap();
        let registry = Registry::flows_only(
            vec![Box::new(BigDraw("test.a")), Box::new(BigDraw("test.b"))],
            &stocks,
        )
        .unwrap();
        let resolver = SourceResolver::empty();
        let dt = 1.0;

        let insp = inspect_flows(&registry, &state, &resolver, dt).unwrap();
        assert!(
            insp.flows.iter().all(|f| (f.scale - 0.625).abs() < 1e-12),
            "both flows should ration to 10/16"
        );

        let integrator = EulerIntegrator::new(registry);
        let report = integrator.step_report(&state, &resolver, dt).unwrap();
        assert_eq!(report.rationed, 2, "both flows rationed");

        // applied_delta (scale-aware) reconstructs the step exactly...
        for (sid, before) in &state.stocks {
            let after = report.state.stocks[sid].amount;
            assert!(
                (before.amount + insp.applied_delta(sid) - after).abs() <= 1e-12,
                "scale-aware identity failed for {sid}"
            );
        }
        // ...while the raw-leg sum would over-report the pool draw (the seam the scale fixes):
        // raw = -16, applied = -10.
        let raw: f64 = insp.flows_touching("d.pool").iter().map(|(_, a)| a).sum();
        assert_eq!(raw, -16.0);
        assert_eq!(insp.applied_delta("d.pool"), -10.0);
    }

    #[test]
    fn json_is_wellformed_and_carries_the_legs_and_scale() {
        let (registry, state, resolver, dt) = setup();
        let json = inspect_flows(&registry, &state, &resolver, dt).unwrap().to_json();
        assert!(json.starts_with('{') && json.ends_with('}'));
        assert!(json.contains("\"n\":5"));
        assert!(json.contains("\"id\":\"test.inflow\""));
        assert!(json.contains("\"scale\":1")); // well-fed → unthrottled
        assert!(json.contains("\"stock\":\"d.pool\""));
        assert!(json.contains("\"amount\":6")); // the +6 inflow leg
    }
}
