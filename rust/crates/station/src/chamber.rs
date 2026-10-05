//! The plant growth chamber's heat store and its cooler — Step 3c, slice 2b
//! (`docs/plans/post-roadmap-room-temperature.md` §23; the user's form B, "a held room that can
//! fail", 2026-10-01).
//!
//! The grow lamp's heat goes into [`CHAMBER`], a sensible-heat store read as `T = Q / C_ch`
//! (absolute Kelvin, referenced to 0 K — **not** to `T_space` like `thermal.node`), and
//! [`ChamberCooling`] moves it on to the station's thermal node: first-order toward the
//! setpoint, capped at a capacity, and only into a node that is colder than the chamber (the
//! second law — no heat pump is modelled). **The plants do not read this temperature yet**
//! (slice 3).
//!
//! # The controller's offset is a property of the form, not an error
//!
//! The cooler acts on the step's starting state, so the chamber settles one step's input
//! above the setpoint when `dt = τ` (deadbeat), and `input·τ / C_ch` above it in general.
//! The station starts the chamber at that steady state ([`chamber_heat0`]), so a nominal run
//! is flat from step 0.
//!
//! ⚠ `dt > τ` overshoots the setpoint and oscillates **without any error**, so the station
//! build refuses it ([`require_step_within_response`]).

use std::collections::BTreeMap;

use domains::thermal::{temperature, ThermalParams, THERMAL_DOMAIN};
use simcore::environment::Environment;
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::quantities::{Quantity, StockKind};
use simcore::state::{State, Stock};

/// The chamber's sensible-heat POOL (ENERGY, J), `T = Q / C_ch`.
pub const CHAMBER: &str = "thermal.chamber";
/// Flow id: the chamber's cooler, chamber → thermal node.
pub const CHAMBER_COOLING: &str = "station.chamber_cooling";

/// The chamber's heat coefficients (`chamber.yaml`).
#[derive(Debug, Clone, Copy)]
pub struct ChamberParams {
    /// C_ch — the chamber's lumped heat capacity (J/K), > 0.
    pub heat_capacity: f64,
    /// The most heat the cooler moves (W), ≥ 0.
    pub cooling_capacity: f64,
    /// τ — the cooler's first-order time constant (s), > 0.
    pub response_time: f64,
    /// The chamber's held temperature (K), > 0.
    pub setpoint: f64,
}

/// The chamber temperature (K): `Q / C_ch`, referenced to absolute zero.
pub fn chamber_temperature(heat_joules: f64, params: &ChamberParams) -> f64 {
    heat_joules / params.heat_capacity
}

/// The chamber's starting heat (J): the setpoint plus the steady offset a constant input
/// `input_w` leaves under the first-order cooler, `C_ch·T_set + input·τ`. At `dt = τ` that
/// offset is one step's input, and the cooler (removing the whole excess each step) holds it
/// there.
pub fn chamber_heat0(params: &ChamberParams, input_w: f64) -> f64 {
    params.heat_capacity * params.setpoint + input_w * params.response_time
}

/// The station build's guard: the fast step must not exceed the cooler's time constant.
pub fn require_step_within_response(dt: f64, params: &ChamberParams) -> Result<(), SimError> {
    if dt > params.response_time {
        return Err(SimError::Validation(format!(
            "the chamber cooler's response time {} s is shorter than the step {dt} s: \
             a first-order cooler with dt > τ overshoots its setpoint and oscillates",
            params.response_time
        )));
    }
    Ok(())
}

/// The chamber's sensible-heat POOL, starting at `amount` (J).
pub fn chamber_stock(amount: f64) -> Result<Stock, SimError> {
    Stock::new(
        CHAMBER.to_string(),
        THERMAL_DOMAIN.to_string(),
        Quantity::Energy,
        Quantity::Energy.canonical_unit(),
        amount,
        StockKind::Pool,
        0.0,
        false,
        BTreeMap::new(),
    )
}

fn donor_amount(snapshot: &State, id: &str) -> Result<f64, SimError> {
    snapshot
        .stocks
        .get(id)
        .map(|s| s.amount)
        .ok_or_else(|| SimError::Reference(format!("flow reads unknown stock {id:?}")))
}

/// The heat the cooler removes over one step (J). Zero at or below the setpoint, and zero
/// unless the node is colder than the chamber.
fn cooling(
    chamber_joules: f64,
    node_joules: f64,
    params: &ChamberParams,
    node: &ThermalParams,
    dt: f64,
) -> f64 {
    let excess = chamber_joules - params.heat_capacity * params.setpoint;
    let t_chamber = chamber_temperature(chamber_joules, params);
    let t_node = temperature(node_joules, node.heat_capacity, node.space_temperature);
    if excess <= 0.0 || t_node >= t_chamber {
        return 0.0;
    }
    (excess * (dt / params.response_time)).min(params.cooling_capacity * dt)
}

/// ENERGY flow `chamber → thermal.node` — the chamber's cooler (2-leg, donor-controlled).
pub struct ChamberCooling {
    id: String,
    chamber: String,
    node: String,
    params: ChamberParams,
    node_params: ThermalParams,
}

impl ChamberCooling {
    /// Construct a `ChamberCooling`; `node_params` reads the node's temperature for the
    /// second-law gate.
    pub fn new(
        id: String,
        chamber: String,
        node: String,
        params: ChamberParams,
        node_params: ThermalParams,
    ) -> Self {
        ChamberCooling {
            id,
            chamber,
            node,
            params,
            node_params,
        }
    }
}

impl Flow for ChamberCooling {
    fn type_name(&self) -> &'static str {
        "ChamberCooling"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let removed = cooling(
            donor_amount(snapshot, &self.chamber)?,
            donor_amount(snapshot, &self.node)?,
            &self.params,
            &self.node_params,
            dt,
        );
        FlowResult::new(vec![
            Leg::new(self.chamber.clone(), -removed)?,
            Leg::new(self.node.clone(), removed)?,
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domains::thermal::NODE;
    use simcore::environment::SourceResolver;
    use simcore::flow::assert_flow_balanced_default;
    use std::collections::HashMap;

    const P: ChamberParams = ChamberParams {
        heat_capacity: 1.5e5,
        cooling_capacity: 200.0,
        response_time: 60.0,
        setpoint: 295.15,
    };
    const NODE_P: ThermalParams = ThermalParams {
        emissivity: 0.85,
        radiator_area: 10.0,
        heat_capacity: 1.0e7,
        space_temperature: 2.7,
    };

    /// The node's stored heat at `t` kelvin (referenced to `T_space`).
    fn node_at(t: f64) -> f64 {
        NODE_P.heat_capacity * (t - NODE_P.space_temperature)
    }

    fn state(chamber: f64, node: f64) -> State {
        let mut stocks = BTreeMap::new();
        for s in [
            chamber_stock(chamber).expect("chamber"),
            domains::thermal::node_stock(node).expect("node"),
        ] {
            stocks.insert(s.id.clone(), s);
        }
        State::new(0, stocks, 0, BTreeMap::new()).expect("state")
    }

    fn legs(chamber: f64, node: f64, dt: f64, params: ChamberParams) -> (f64, f64) {
        let flow = ChamberCooling::new(
            CHAMBER_COOLING.to_string(),
            CHAMBER.to_string(),
            NODE.to_string(),
            params,
            NODE_P,
        );
        let s = state(chamber, node);
        let r = SourceResolver::new(HashMap::new(), HashMap::new()).expect("resolver");
        let env = r.bind(&s, dt);
        let result = flow.evaluate(&s, &env, dt).expect("evaluate");
        assert_flow_balanced_default(&result, &s.stocks).expect("balanced");
        let get = |id: &str| {
            result
                .legs
                .iter()
                .find(|l| l.stock == id)
                .expect("leg")
                .amount
        };
        (get(CHAMBER), get(NODE))
    }

    #[test]
    fn temperature_is_heat_over_capacity_from_absolute_zero() {
        assert_eq!(chamber_temperature(P.heat_capacity * 295.15, &P), 295.15);
    }

    #[test]
    fn deadbeat_at_dt_equal_tau_removes_the_whole_excess() {
        let excess = 3600.0;
        let (ch, node) = legs(
            P.heat_capacity * P.setpoint + excess,
            node_at(170.0),
            60.0,
            P,
        );
        assert!((-ch - excess).abs() <= 1e-6, "{ch}");
        assert_eq!(node, -ch);
    }

    #[test]
    fn first_order_below_deadbeat() {
        // dt = τ/2 removes half the excess.
        let excess = 3600.0;
        let (ch, _) = legs(
            P.heat_capacity * P.setpoint + excess,
            node_at(170.0),
            30.0,
            P,
        );
        assert!((-ch - excess / 2.0).abs() <= 1e-6, "{ch}");
    }

    #[test]
    fn capacity_caps_the_removal() {
        let (ch, _) = legs(
            P.heat_capacity * (P.setpoint + 10.0),
            node_at(170.0),
            60.0,
            P,
        );
        assert_eq!(ch, -P.cooling_capacity * 60.0);
    }

    #[test]
    fn nothing_moves_at_or_below_the_setpoint() {
        for q in [
            P.heat_capacity * P.setpoint,
            P.heat_capacity * (P.setpoint - 1.0),
        ] {
            assert_eq!(legs(q, node_at(170.0), 60.0, P), (0.0, 0.0));
        }
    }

    #[test]
    fn a_failed_cooler_moves_nothing() {
        let failed = ChamberParams {
            cooling_capacity: 0.0,
            ..P
        };
        let (ch, _) = legs(
            P.heat_capacity * (P.setpoint + 1.0),
            node_at(170.0),
            60.0,
            failed,
        );
        assert_eq!(ch, 0.0);
    }

    /// ⚠ The second-law gate across the two REFERENCE POINTS: the chamber is read from 0 K,
    /// the node from `T_space`. Just below the chamber's temperature heat moves; at and just
    /// above it, none does.
    #[test]
    fn heat_moves_only_into_a_colder_node() {
        let t_ch = P.setpoint + 0.5;
        let q = P.heat_capacity * t_ch;
        let (below, _) = legs(q, node_at(t_ch - 1e-6), 60.0, P);
        assert!(
            below < 0.0,
            "a node just below the chamber takes heat: {below}"
        );
        for t_node in [t_ch, t_ch + 1e-6, t_ch + 50.0] {
            assert_eq!(
                legs(q, node_at(t_node), 60.0, P).0,
                0.0,
                "node at {t_node} K"
            );
        }
    }

    #[test]
    fn the_steady_start_is_setpoint_plus_input_times_tau() {
        let q0 = chamber_heat0(&P, 60.0);
        assert_eq!(q0, P.heat_capacity * P.setpoint + 60.0 * P.response_time);
    }

    #[test]
    fn a_step_longer_than_the_response_time_is_refused() {
        assert!(require_step_within_response(60.0, &P).is_ok());
        assert!(require_step_within_response(60.000001, &P).is_err());
    }
}
