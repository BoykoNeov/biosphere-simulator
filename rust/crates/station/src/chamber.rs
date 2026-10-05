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
//! The station starts the chamber at the steady state of the lamp alone ([`chamber_heat0`]).
//! With the walls (2b-iii) the input is the lamp less the walls' loss, which follows the
//! outdoor weather, so the nominal chamber follows it too — 295.1887–295.2034 K in the
//! reference (§23j); without walls it would be flat from step 0.
//!
//! ⚠ `dt > τ` overshoots the setpoint and oscillates **without any error**, so the station
//! build refuses it ([`require_step_within_response`]).
//!
//! # Walls and a heater (slice 2b-iii, §23i)
//!
//! [`ChamberWall`] exchanges heat with the chamber's surroundings, either way — the user's four
//! ([`ChamberSurroundings`]): the outdoor weather (the reference), a held cabin, space, or the
//! station structure. [`ChamberHeater`] is the cooler's mirror, on the battery, below the
//! setpoint. In the reference the lamp outweighs the walls, so the heater never fires there.

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
/// Flow id: the chamber's walls, chamber ↔ its surroundings.
pub const CHAMBER_WALL: &str = "station.chamber_wall";
/// Flow id: the chamber's heater, battery → chamber.
pub const CHAMBER_HEATER: &str = "station.chamber_heater";
/// The two-signed BOUNDARY the walls exchange with when the surroundings are outside the
/// station's books (the outdoor weather, a held cabin).
pub const CHAMBER_SURROUNDINGS: &str = "boundary.chamber_surroundings";
/// Fast forcing var: the outdoor air temperature (°C) the chamber's walls face — the weather
/// file's daily value, the same one the plants read.
pub const OUTDOOR_TEMP_VAR: &str = "chamber_outdoor_temp";
/// Celsius → Kelvin.
const ZERO_CELSIUS_K: f64 = 273.15;

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
    /// U — the walls' conductance per area (W/m²·K), ≥ 0.
    pub wall_conductance: f64,
    /// A — the walls' area (m²), > 0.
    pub wall_area: f64,
    /// The most heat the heater gives (W), ≥ 0. 0 is a valid, failed heater.
    pub heater_capacity: f64,
}

/// What the chamber's walls face — the user's four (§23d). Only [`Self::Outdoor`] is in the
/// reference; the other three are lab options.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChamberSurroundings {
    /// The outdoor weather's daily temperature (the reference): `UA·(T_ch − T_out)` into
    /// [`CHAMBER_SURROUNDINGS`].
    Outdoor,
    /// A cabin held at `temperature_k` (lab), hotter or cooler: `UA·(T_ch − T_cab)` into
    /// [`CHAMBER_SURROUNDINGS`].
    Cabin {
        /// The cabin's held temperature (K).
        temperature_k: f64,
    },
    /// Deep space through an insulation blanket (lab): `ε*·σ·A·(T_ch⁴ − T_space⁴)` into
    /// `boundary.space`. ⚠ ε* is **WHAT-IF** — no source (§23d).
    Space {
        /// ε* — the blanket's effective emittance, WHAT-IF.
        effective_emittance: f64,
    },
    /// The station structure, `thermal.node` (lab): `UA·(T_ch − T_node)` into the node.
    Structure,
}

/// The chamber temperature (K): `Q / C_ch`, referenced to absolute zero.
pub fn chamber_temperature(heat_joules: f64, params: &ChamberParams) -> f64 {
    heat_joules / params.heat_capacity
}

/// The chamber's starting heat (J): the setpoint plus the steady offset a constant input
/// `input_w` leaves under the first-order cooler, `C_ch·T_set + input·τ`. At `dt = τ` that
/// offset is one step's input, and the cooler (removing the whole excess each step) holds it
/// there while the input stays constant — with walls it does not (the module doc).
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

/// The walls' heat law, resolved from [`ChamberSurroundings`] at build time.
#[derive(Debug, Clone, Copy)]
enum WallLaw {
    /// `UA·(T_ch − (T_out °C + 273.15))`, `T_out` from [`OUTDOOR_TEMP_VAR`].
    Outdoor { ua: f64 },
    /// `UA·(T_ch − T)`, a fixed `T` (K).
    Held { ua: f64, temperature_k: f64 },
    /// `UA·(T_ch − T_node)`, the node read from its stock.
    Node { ua: f64, node: ThermalParams },
    /// `ε*·σ·A·(T_ch⁴ − T_space⁴)`.
    Radiative {
        emittance_area: f64,
        space_temperature: f64,
    },
}

/// ENERGY flow `chamber ↔ surroundings` — the chamber's walls (2-leg, either sign: a positive
/// flux leaves the chamber).
pub struct ChamberWall {
    id: String,
    chamber: String,
    surroundings: String,
    law: WallLaw,
    params: ChamberParams,
}

impl ChamberWall {
    /// Construct the walls for `surroundings`; `surroundings_stock` is where their heat goes
    /// ([`CHAMBER_SURROUNDINGS`], `boundary.space` or `thermal.node`, by the caller's wiring).
    pub fn new(
        id: String,
        chamber: String,
        surroundings_stock: String,
        surroundings: ChamberSurroundings,
        params: ChamberParams,
        node: ThermalParams,
    ) -> Self {
        let ua = params.wall_conductance * params.wall_area;
        let law = match surroundings {
            ChamberSurroundings::Outdoor => WallLaw::Outdoor { ua },
            ChamberSurroundings::Cabin { temperature_k } => WallLaw::Held { ua, temperature_k },
            ChamberSurroundings::Structure => WallLaw::Node { ua, node },
            ChamberSurroundings::Space {
                effective_emittance,
            } => WallLaw::Radiative {
                emittance_area: effective_emittance * params.wall_area,
                space_temperature: node.space_temperature,
            },
        };
        ChamberWall {
            id,
            chamber,
            surroundings: surroundings_stock,
            law,
            params,
        }
    }
}

impl Flow for ChamberWall {
    fn type_name(&self) -> &'static str {
        "ChamberWall"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let t_ch = chamber_temperature(donor_amount(snapshot, &self.chamber)?, &self.params);
        let watts = match self.law {
            WallLaw::Outdoor { ua } => ua * (t_ch - (env.get(OUTDOOR_TEMP_VAR)? + ZERO_CELSIUS_K)),
            WallLaw::Held { ua, temperature_k } => ua * (t_ch - temperature_k),
            WallLaw::Node { ua, node } => {
                let q = donor_amount(snapshot, &self.surroundings)?;
                ua * (t_ch - temperature(q, node.heat_capacity, node.space_temperature))
            }
            WallLaw::Radiative {
                emittance_area,
                space_temperature,
            } => {
                emittance_area
                    * domains::thermal::STEFAN_BOLTZMANN
                    * (t_ch.powf(4.0) - space_temperature.powf(4.0))
            }
        };
        let lost = watts * dt;
        FlowResult::new(vec![
            Leg::new(self.chamber.clone(), -lost)?,
            Leg::new(self.surroundings.clone(), lost)?,
        ])
    }
}

/// The heat the heater gives over one step (J): the cooler's mirror, zero at or above the
/// setpoint.
fn heating(chamber_joules: f64, params: &ChamberParams, dt: f64) -> f64 {
    let deficit = params.heat_capacity * params.setpoint - chamber_joules;
    if deficit <= 0.0 {
        return 0.0;
    }
    (deficit * (dt / params.response_time)).min(params.heater_capacity * dt)
}

/// ENERGY flow `battery → chamber` — the chamber's resistive heater (2-leg): every joule drawn
/// is heat.
pub struct ChamberHeater {
    id: String,
    battery: String,
    chamber: String,
    params: ChamberParams,
}

impl ChamberHeater {
    /// Construct a `ChamberHeater` drawing on `battery`.
    pub fn new(id: String, battery: String, chamber: String, params: ChamberParams) -> Self {
        ChamberHeater {
            id,
            battery,
            chamber,
            params,
        }
    }
}

impl Flow for ChamberHeater {
    fn type_name(&self) -> &'static str {
        "ChamberHeater"
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
        let given = heating(donor_amount(snapshot, &self.chamber)?, &self.params, dt);
        FlowResult::new(vec![
            Leg::new(self.battery.clone(), -given)?,
            Leg::new(self.chamber.clone(), given)?,
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
        wall_conductance: 0.30,
        wall_area: 5.12,
        heater_capacity: 200.0,
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

    // --- the walls (2b-iii) --------------------------------------------------------------

    /// The chamber's leg of one wall step (J; positive = heat lost), with the outdoor forcing
    /// at `outdoor_c` and every stock the four laws read present.
    fn wall_loss(surroundings: ChamberSurroundings, t_ch: f64, node_t: f64, outdoor_c: f64) -> f64 {
        let sink = match surroundings {
            ChamberSurroundings::Structure => NODE,
            _ => CHAMBER_SURROUNDINGS,
        };
        let flow = ChamberWall::new(
            CHAMBER_WALL.to_string(),
            CHAMBER.to_string(),
            sink.to_string(),
            surroundings,
            P,
            NODE_P,
        );
        let mut s = state(P.heat_capacity * t_ch, node_at(node_t));
        let b = simcore::boundary::source(
            CHAMBER_SURROUNDINGS.to_string(),
            Quantity::Energy,
            0.0,
            true,
        )
        .expect("boundary");
        s.stocks.insert(b.id.clone(), b);
        let mut forcings: HashMap<String, simcore::environment::Schedule> = HashMap::new();
        forcings.insert(
            OUTDOOR_TEMP_VAR.to_string(),
            simcore::environment::constant(outdoor_c).expect("constant"),
        );
        let r = SourceResolver::new(forcings, HashMap::new()).expect("resolver");
        let env = r.bind(&s, 60.0);
        let result = flow.evaluate(&s, &env, 60.0).expect("evaluate");
        assert_flow_balanced_default(&result, &s.stocks).expect("balanced");
        -result
            .legs
            .iter()
            .find(|l| l.stock == CHAMBER)
            .expect("leg")
            .amount
    }

    const UA: f64 = 0.30 * 5.12;

    #[test]
    fn the_outdoor_wall_reads_celsius_and_loses_ua_delta_t() {
        // 22 °C chamber, 2 °C outside: 20 K across UA = 1.536 W/K, for 60 s.
        let lost = wall_loss(ChamberSurroundings::Outdoor, 295.15, 170.0, 2.0);
        assert!((lost - UA * 20.0 * 60.0).abs() < 1e-9, "{lost}");
        // At the chamber's own temperature, nothing.
        let none = wall_loss(ChamberSurroundings::Outdoor, 295.15, 170.0, 22.0);
        assert!(none.abs() < 1e-9, "{none}");
        // A hotter outside sends heat in.
        assert!(wall_loss(ChamberSurroundings::Outdoor, 295.15, 170.0, 30.0) < 0.0);
    }

    #[test]
    fn a_held_cabin_pulls_or_pushes() {
        let cold = ChamberSurroundings::Cabin {
            temperature_k: 291.15,
        };
        let hot = ChamberSurroundings::Cabin {
            temperature_k: 300.15,
        };
        assert!((wall_loss(cold, 295.15, 170.0, 0.0) - UA * 4.0 * 60.0).abs() < 1e-9);
        assert!((wall_loss(hot, 295.15, 170.0, 0.0) + UA * 5.0 * 60.0).abs() < 1e-9);
    }

    #[test]
    fn the_structure_wall_reads_the_node() {
        // The node at 175 K (from T_space), the chamber at 295.15 K (from 0 K).
        let lost = wall_loss(ChamberSurroundings::Structure, 295.15, 175.0, 0.0);
        assert!((lost - UA * 120.15 * 60.0).abs() < 1e-6, "{lost}");
    }

    /// L3 — space through a blanket, ε* = 0.01 (WHAT-IF): 22.05 W at 295.2 K.
    #[test]
    fn the_space_wall_radiates() {
        let space = ChamberSurroundings::Space {
            effective_emittance: 0.01,
        };
        let watts = wall_loss(space, 295.2, 170.0, 0.0) / 60.0;
        assert!((watts - 22.0469).abs() < 1e-3, "{watts}");
    }

    // --- the heater (2b-iii) --------------------------------------------------------------

    #[test]
    fn the_heater_mirrors_the_cooler_below_the_setpoint_only() {
        let s_set = P.heat_capacity * P.setpoint;
        assert_eq!(heating(s_set, &P, 60.0), 0.0);
        assert_eq!(heating(s_set + 1.0, &P, 60.0), 0.0);
        assert!((heating(s_set - 3600.0, &P, 60.0) - 3600.0).abs() < 1e-6);
        assert!((heating(s_set - 3600.0, &P, 30.0) - 1800.0).abs() < 1e-6);
        // Capped at its capacity; a failed heater gives nothing.
        assert_eq!(heating(s_set - 1e6, &P, 60.0), P.heater_capacity * 60.0);
        let failed = ChamberParams {
            heater_capacity: 0.0,
            ..P
        };
        assert_eq!(heating(s_set - 1e6, &failed, 60.0), 0.0);
    }
}
