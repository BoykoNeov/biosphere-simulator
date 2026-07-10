//! The Phase-8 (P8.2) **display projection** — the derived, human-facing readouts the
//! Godot dashboard renders. This is the layer one step out from the frozen
//! [`simcore::observation`] surface: it groups the observation per-domain and computes the
//! aggregates and domain-specific scalars a UI wants (per-quantity totals, node
//! temperature, battery state-of-charge, the last step's conservation residual).
//!
//! **Zero parity concern.** Everything here is a pure function of the exact [`State`] (plus
//! per-scenario constants that don't live in state — see [`DisplayContext`]); nothing feeds
//! back into stepping and no golden pins it. It is developed freely, with normal decimal
//! float formatting (the parity-critical hex-float codec stays on
//! [`simcore::snapshot`]). The one discipline kept is honesty of labels — see the SOC note
//! on [`BatteryReadout`].
//!
//! **Why the derived readouts need a context (advisor).** Temperature `T = T_space + Q/C`
//! and SOC% need constants that are deliberately *not* in `State`: the node's heat capacity
//! and space-reference temperature are radiator params, and the battery's full-charge
//! reference is sizing/scenario data (CLAUDE.md: "capacity is NOT a param"). The
//! **shared-stock** highlight is likewise a *construction-time fact of the assembly* — the
//! station layer explicitly wires specific stocks to be cross-domain (the cabin CO₂/O₂
//! pools; the coupled `thermal.node`) — not something recoverable from a `Stock` (each
//! carries one `domain`) or the flow graph (a [`Flow`](simcore::flow::Flow) exposes no
//! static stock refs). So the caller *declares* all three per scenario in a
//! [`DisplayContext`], reading them off the palette it already built.

use std::collections::BTreeMap;

use simcore::ids::{DomainId, StockId};
use simcore::observation::{observe, StockObservation};
use simcore::quantities::Quantity;
use simcore::state::State;

use domains::thermal::temperature;

/// The node-temperature readout inputs for a scenario that has a thermal node (`None` for
/// scenarios that don't). All three come from the scenario's [`ThermalParams`] +
/// stock id — reused verbatim by [`temperature`].
///
/// [`ThermalParams`]: domains::thermal::ThermalParams
#[derive(Debug, Clone)]
pub struct ThermalReadout {
    /// The sensible-heat POOL id (`thermal.node`).
    pub node_id: StockId,
    /// C — node heat capacity (J/K).
    pub heat_capacity: f64,
    /// T_space — the node reference / radiative-sink temperature (K).
    pub space_temperature: f64,
}

/// The battery state-of-charge readout inputs for a scenario that has a battery.
///
/// **SOC is "% of initial charge," not "% of capacity" — an honest label (advisor).**
/// A POOL battery has no upper clamp, and capacity is scenario/sizing data absent from
/// `State`, so the only reference the display can cite exactly is the initial charge
/// `battery0`. Under diurnal charging the SOC therefore swings *above* 100% when the
/// battery charges past its start — which is correct for this reference, not a bug.
#[derive(Debug, Clone)]
pub struct BatteryReadout {
    /// The battery POOL id (`power.battery`).
    pub battery_id: StockId,
    /// The initial charge `battery0` (J) — the 100% reference.
    pub initial_charge: f64,
}

/// Per-scenario constants the display layer needs that are absent from [`State`]. The
/// bridge fills this from the same palette entry it built the session with. All fields are
/// optional / empty for a scenario that lacks the corresponding feature (`cabin_gas` has
/// no node and no battery; `station` has both but no shared cabin gas pools).
#[derive(Debug, Clone, Default)]
pub struct DisplayContext {
    /// The node-temperature readout, if this scenario has a thermal node.
    pub thermal: Option<ThermalReadout>,
    /// The battery SOC readout, if this scenario has a battery.
    pub battery: Option<BatteryReadout>,
    /// Stock ids the assembly wires to be cross-domain (highlighted in the UI). Declared,
    /// not inferred — see the module docs.
    pub shared_stock_ids: Vec<StockId>,
}

/// The total amount of one conserved [`Quantity`] summed across every stock (a health
/// readout: for a closed run it is the conserved invariant, constant to round-off).
#[derive(Debug, Clone, PartialEq)]
pub struct QuantityTotal {
    pub quantity: Quantity,
    pub total: f64,
}

/// A complete, plain-data display read of one simulation moment — everything the dashboard
/// renders, computed Rust-side. Built by [`project`].
#[derive(Debug, Clone)]
pub struct DisplayProjection {
    /// The integer step count (master days for a two-rate session).
    pub n: u64,
    /// Cumulative flows scaled by the Euler backstop (a healthy run reads 0).
    pub rationed: u64,
    /// Number of extinction events emitted so far.
    pub events: usize,
    /// The largest absolute conservation residual over the last step (`None` before the
    /// first step) — a "nothing leaked" indicator, ~1e-15 on a healthy run.
    pub max_residual: Option<f64>,
    /// Observations grouped by domain, in canonical domain- then id-sorted order.
    pub domains: BTreeMap<DomainId, Vec<StockObservation>>,
    /// The declared cross-domain stock ids to highlight.
    pub shared_stock_ids: Vec<StockId>,
    /// Total per conserved quantity across all stocks, in quantity-name-sorted order.
    pub totals: Vec<QuantityTotal>,
    /// Node temperature `T = T_space + Q/C` (K), if the scenario has a thermal node.
    pub temperature_k: Option<f64>,
    /// Battery SOC as a percentage **of initial charge** (see [`BatteryReadout`]), if the
    /// scenario has a battery.
    pub soc_percent_of_initial: Option<f64>,
}

/// Group an [`Observation`](simcore::observation::Observation)'s stocks by their `domain`.
/// The observation is already id-sorted, so each group preserves canonical id order, and
/// the `BTreeMap` keys are domain-sorted.
pub fn group_by_domain(state: &State) -> BTreeMap<DomainId, Vec<StockObservation>> {
    let mut grouped: BTreeMap<DomainId, Vec<StockObservation>> = BTreeMap::new();
    for so in observe(state).stocks {
        grouped.entry(so.domain.clone()).or_default().push(so);
    }
    grouped
}

/// Total per conserved quantity across every stock, folding each stock's composition
/// (`amount · coeff`) so a gas-phase stock (CO₂ = `{carbon:1, oxygen:2}`) contributes to
/// each element. Accumulated in id-sorted order (the `BTreeMap` iteration order); returned
/// in quantity-name-sorted order.
pub fn per_quantity_totals(state: &State) -> Vec<QuantityTotal> {
    let mut totals: BTreeMap<Quantity, f64> = BTreeMap::new();
    for stock in state.stocks.values() {
        for (q, coeff) in &stock.composition {
            *totals.entry(*q).or_insert(0.0) += stock.amount * coeff;
        }
    }
    let mut out: Vec<QuantityTotal> = totals
        .into_iter()
        .map(|(quantity, total)| QuantityTotal { quantity, total })
        .collect();
    out.sort_by(|a, b| a.quantity.name().cmp(b.quantity.name()));
    out
}

/// Build the full [`DisplayProjection`] for `state` under `ctx`. The session-level scalars
/// (`rationed`, `events`, `max_residual`) are passed in because they are step-history, not
/// projections of one `State` — the caller reads them off the [`SimSession`](crate::session::SimSession).
pub fn project(
    state: &State,
    ctx: &DisplayContext,
    rationed: u64,
    events: usize,
    max_residual: Option<f64>,
) -> DisplayProjection {
    let temperature_k = ctx.thermal.as_ref().and_then(|t| {
        state
            .stocks
            .get(&t.node_id)
            .map(|node| temperature(node.amount, t.heat_capacity, t.space_temperature))
    });
    let soc_percent_of_initial = ctx.battery.as_ref().and_then(|b| {
        state
            .stocks
            .get(&b.battery_id)
            .map(|bat| 100.0 * bat.amount / b.initial_charge)
    });
    DisplayProjection {
        n: state.n,
        rationed,
        events,
        max_residual,
        domains: group_by_domain(state),
        shared_stock_ids: ctx.shared_stock_ids.clone(),
        totals: per_quantity_totals(state),
        temperature_k,
        soc_percent_of_initial,
    }
}

impl DisplayProjection {
    /// Serialize to plain-float JSON for the Godot dashboard (GDScript `JSON.parse_string`).
    ///
    /// Deliberately **not** the hex-float codec: this is a human-facing, zero-parity
    /// readout, so normal decimal formatting (Rust's shortest round-tripping `Display`) is
    /// correct — the bit-exact snapshot path stays on [`simcore::snapshot`]. Optional
    /// scalars emit JSON `null` when absent. Zero-dep hand-written emit, mirroring the
    /// snapshot emitter's stdlib-only discipline.
    pub fn to_json(&self) -> String {
        let mut out = String::from("{");
        out.push_str(&format!("\"n\":{},", self.n));
        out.push_str(&format!("\"rationed\":{},", self.rationed));
        out.push_str(&format!("\"events\":{},", self.events));
        out.push_str("\"max_residual\":");
        push_opt_f64(&mut out, self.max_residual);
        out.push(',');
        out.push_str("\"temperature_k\":");
        push_opt_f64(&mut out, self.temperature_k);
        out.push(',');
        out.push_str("\"soc_percent_of_initial\":");
        push_opt_f64(&mut out, self.soc_percent_of_initial);
        out.push(',');

        // shared_stock_ids: ["a","b"]
        out.push_str("\"shared_stock_ids\":[");
        for (i, id) in self.shared_stock_ids.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            push_json_string(&mut out, id);
        }
        out.push_str("],");

        // totals: {"carbon":123.4, ...} (lowercase quantity value keys)
        out.push_str("\"totals\":{");
        for (i, qt) in self.totals.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            push_json_string(&mut out, qt.quantity.value());
            out.push(':');
            push_f64(&mut out, qt.total);
        }
        out.push_str("},");

        // domains: {"crew":[{stock}, ...], ...}
        out.push_str("\"domains\":{");
        for (i, (domain, stocks)) in self.domains.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            push_json_string(&mut out, domain);
            out.push_str(":[");
            for (j, so) in stocks.iter().enumerate() {
                if j > 0 {
                    out.push(',');
                }
                out.push('{');
                push_json_string(&mut out, "id");
                out.push(':');
                push_json_string(&mut out, &so.id);
                out.push(',');
                push_json_string(&mut out, "quantity");
                out.push(':');
                push_json_string(&mut out, so.quantity.value());
                out.push(',');
                push_json_string(&mut out, "unit");
                out.push(':');
                push_json_string(&mut out, &so.unit);
                out.push(',');
                push_json_string(&mut out, "amount");
                out.push(':');
                push_f64(&mut out, so.amount());
                out.push('}');
            }
            out.push(']');
        }
        out.push_str("}}");
        out
    }
}

/// Append a finite `f64` as a JSON number (Rust's shortest round-tripping decimal).
/// Shared with [`crate::inspection`] (the flow-level slice of the same display projection).
pub(crate) fn push_f64(out: &mut String, v: f64) {
    // Every value routed here is finite (amounts/totals from a validated State; derived
    // temperature/SOC of finite inputs). Guard defensively anyway — JSON has no NaN/Inf.
    if v.is_finite() {
        out.push_str(&format!("{v}"));
    } else {
        out.push_str("null");
    }
}

/// Append an `Option<f64>` as a JSON number or `null`.
fn push_opt_f64(out: &mut String, v: Option<f64>) {
    match v {
        Some(x) => push_f64(out, x),
        None => out.push_str("null"),
    }
}

/// Append a JSON string literal, escaping the characters JSON requires (mirrors the
/// snapshot emitter — our ids/units contain none of these in practice). Shared with
/// [`crate::inspection`].
pub(crate) fn push_json_string(out: &mut String, s: &str) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;
    use simcore::quantities::StockKind;
    use simcore::state::Stock;

    fn stock(id: &str, domain: &str, q: Quantity, amount: f64) -> Stock {
        Stock::new(
            id.to_string(),
            domain.to_string(),
            q,
            q.canonical_unit(),
            amount,
            StockKind::Pool,
            0.0,
            false,
            BTreeMap::new(),
        )
        .unwrap()
    }

    fn co2(id: &str, domain: &str, amount: f64) -> Stock {
        Stock::new(
            id.to_string(),
            domain.to_string(),
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

    fn state_of(stocks: Vec<Stock>) -> State {
        State::new(
            0,
            stocks.into_iter().map(|s| (s.id.clone(), s)).collect(),
            0,
            BTreeMap::new(),
        )
        .unwrap()
    }

    #[test]
    fn groups_stocks_by_domain() {
        let state = state_of(vec![
            stock("crew.food_store", "crew", Quantity::Carbon, 1.0),
            stock("eclss.cabin_o2", "eclss", Quantity::Oxygen, 2.0),
            stock("crew.water_store", "crew", Quantity::Water, 3.0),
        ]);
        let grouped = group_by_domain(&state);
        assert_eq!(grouped["crew"].len(), 2);
        assert_eq!(grouped["eclss"].len(), 1);
        // canonical id order within a group
        assert_eq!(grouped["crew"][0].id, "crew.food_store");
        assert_eq!(grouped["crew"][1].id, "crew.water_store");
    }

    #[test]
    fn totals_fold_composition_coeff() {
        // A CO₂ pool of 5 mol contributes 5 CARBON and 10 OXYGEN (coeff 2).
        let totals = per_quantity_totals(&state_of(vec![co2("eclss.cabin_co2", "eclss", 5.0)]));
        let carbon = totals.iter().find(|t| t.quantity == Quantity::Carbon).unwrap();
        let oxygen = totals.iter().find(|t| t.quantity == Quantity::Oxygen).unwrap();
        assert_eq!(carbon.total, 5.0);
        assert_eq!(oxygen.total, 10.0);
    }

    #[test]
    fn temperature_uses_thermal_context() {
        // Q = 100 J, C = 10 J/K, T_space = 4 K → T = 4 + 10 = 14 K.
        let state = state_of(vec![stock("thermal.node", "thermal", Quantity::Energy, 100.0)]);
        let ctx = DisplayContext {
            thermal: Some(ThermalReadout {
                node_id: "thermal.node".to_string(),
                heat_capacity: 10.0,
                space_temperature: 4.0,
            }),
            ..Default::default()
        };
        let proj = project(&state, &ctx, 0, 0, None);
        assert_eq!(proj.temperature_k, Some(14.0));
        assert_eq!(proj.soc_percent_of_initial, None); // no battery context
    }

    #[test]
    fn soc_is_percent_of_initial_and_can_exceed_100() {
        let state = state_of(vec![stock("power.battery", "power", Quantity::Energy, 120.0)]);
        let ctx = DisplayContext {
            battery: Some(BatteryReadout {
                battery_id: "power.battery".to_string(),
                initial_charge: 100.0,
            }),
            ..Default::default()
        };
        let proj = project(&state, &ctx, 0, 0, None);
        assert_eq!(proj.soc_percent_of_initial, Some(120.0)); // charged past start
    }

    #[test]
    fn absent_features_project_to_none() {
        // cabin_gas-shaped state: no node, no battery — both scalars None.
        let state = state_of(vec![stock("eclss.cabin_o2", "eclss", Quantity::Oxygen, 8.0)]);
        let ctx = DisplayContext::default();
        let proj = project(&state, &ctx, 3, 0, Some(1e-15));
        assert!(proj.temperature_k.is_none());
        assert!(proj.soc_percent_of_initial.is_none());
        assert_eq!(proj.rationed, 3);
        assert_eq!(proj.max_residual, Some(1e-15));
    }

    #[test]
    fn json_is_wellformed_and_carries_the_readouts() {
        let state = state_of(vec![
            stock("power.battery", "power", Quantity::Energy, 50.0),
            stock("thermal.node", "thermal", Quantity::Energy, 100.0),
        ]);
        let ctx = DisplayContext {
            thermal: Some(ThermalReadout {
                node_id: "thermal.node".to_string(),
                heat_capacity: 10.0,
                space_temperature: 4.0,
            }),
            battery: Some(BatteryReadout {
                battery_id: "power.battery".to_string(),
                initial_charge: 100.0,
            }),
            shared_stock_ids: vec!["thermal.node".to_string()],
        };
        let json = project(&state, &ctx, 0, 0, Some(1e-16)).to_json();
        assert!(json.starts_with('{') && json.ends_with('}'));
        assert!(json.contains("\"temperature_k\":14"));
        assert!(json.contains("\"soc_percent_of_initial\":50"));
        assert!(json.contains("\"shared_stock_ids\":[\"thermal.node\"]"));
        assert!(json.contains("\"power\":["));
        assert!(json.contains("\"energy\":150")); // total ENERGY across both stocks
    }

    #[test]
    fn none_scalars_emit_json_null() {
        let json = project(&state_of(vec![]), &DisplayContext::default(), 0, 0, None).to_json();
        assert!(json.contains("\"temperature_k\":null"));
        assert!(json.contains("\"soc_percent_of_initial\":null"));
        assert!(json.contains("\"max_residual\":null"));
    }
}
