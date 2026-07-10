//! The standalone Thermal domain — the Rust port of `domains.thermal` (Phase-7 P7.3).
//!
//! ENERGY (J) only, referenced to `T_space`. Two flows: [`HeatInput`] (2-leg forced,
//! heat→heat lossless) and [`RadiatorReject`] (2-leg donor-controlled, the nonlinear
//! Stefan-Boltzmann `R = εσA(T⁴ − T_space⁴)·dt`). Thermal is **Tier-2**: the `t**4`
//! (ported op-for-op as `powf(4.0)`, per the plan) is the transcendental. The radiator
//! is the restoring force, so the resolver is a plain constant `heat_load` (no derived
//! load, unlike Power).

use std::collections::{BTreeMap, HashMap};

use simcore::boundary;
use simcore::environment::{constant, Environment, Schedule, SourceResolver};
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::quantities::{Quantity, StockKind};
use simcore::registry::Registry;
use simcore::state::{State, Stock};

/// The Stefan-Boltzmann constant σ (W·m⁻²·K⁻⁴) — a universal physical constant (CODATA
/// 2018, exact since the 2019 SI redefinition), NOT a param. Mirrors the Python module
/// constant `thermal.flows.STEFAN_BOLTZMANN`.
pub const STEFAN_BOLTZMANN: f64 = 5.670374419e-8;

// --- stock ids + forcing var + flow ids (ASCII; str sort == Rust byte sort, #15) ---
/// The Thermal domain id (only `thermal.node` carries it; reservoirs are `boundary`).
pub const THERMAL_DOMAIN: &str = "thermal";
/// The sensible-heat POOL id.
pub const NODE: &str = "thermal.node";
/// The unclamped forced-heat BOUNDARY source id.
pub const HEAT_SOURCE: &str = "boundary.heat_source";
/// The monotonic deep-space BOUNDARY sink id.
pub const SPACE: &str = "boundary.space";
/// Forcing var: instantaneous forced heat input into the node (W).
pub const HEAT_LOAD_VAR: &str = "heat_load";
/// Flow id: the forced heat input.
pub const HEAT_INPUT: &str = "thermal.heat_input";
/// Flow id: the Stefan-Boltzmann radiator.
pub const RADIATOR_REJECT: &str = "thermal.radiator_reject";

/// The radiator's thermal + radiative properties (`radiator.yaml`).
#[derive(Debug, Clone, Copy)]
pub struct ThermalParams {
    /// ε — grey-body emissivity (dimensionless), ∈ (0, 1].
    pub emissivity: f64,
    /// A — radiating area (m²), > 0.
    pub radiator_area: f64,
    /// C — node heat capacity (J/K), > 0.
    pub heat_capacity: f64,
    /// T_space — radiative sink temperature and the node reference (K), ≥ 0.
    pub space_temperature: f64,
}

/// Thermal scenario data (initial heat, forced load, step; not the radiator params).
#[derive(Debug, Clone, Copy)]
pub struct ThermalScenario {
    /// Initial sensible heat `Q = C·(T − T_space)` (J).
    pub node0: f64,
    /// Forced constant heat input (W).
    pub heat_load_w: f64,
    /// Integration step (s).
    pub dt_seconds: f64,
}

/// The standalone validation scenario (`EQUILIBRIUM_SCENARIO`): a cold node under a
/// constant load, warming to an emergent equilibrium temperature.
pub const EQUILIBRIUM_SCENARIO: ThermalScenario = ThermalScenario {
    node0: 0.0,
    heat_load_w: 3000.0,
    dt_seconds: 3600.0,
};

/// The equilibrium-run horizon (steps) — ~11 relaxation times.
pub const EQUILIBRIUM_STEPS: u64 = 720;

/// The node temperature (K), the derived readout `T = T_space + Q/C`. Op-order mirrors
/// Python `temperature`.
pub fn temperature(node_joules: f64, heat_capacity: f64, space_temperature: f64) -> f64 {
    space_temperature + node_joules / heat_capacity
}

/// The emergent steady-state node temperature `T_eq` (K), a closed form: the temperature
/// at which Stefan-Boltzmann rejection balances a forced `heat_load_w`. Op-order mirrors
/// Python `equilibrium_temperature`: `(heat_load/(εσA) + T_space⁴)^(1/4)` — `**0.25` →
/// `.powf(0.25)`, `**4` → `.powf(4.0)`, the plan's libm audit. The station derives the
/// coupled node's initial heat `Q_eq = C·(T_eq − T_space)` from this (`sealed_node_heat`).
pub fn equilibrium_temperature(params: &ThermalParams, heat_load_w: f64) -> f64 {
    let driving = heat_load_w / (params.emissivity * STEFAN_BOLTZMANN * params.radiator_area);
    (driving + params.space_temperature.powf(4.0)).powf(0.25)
}

/// The instantaneous radiated power `ε·σ·A·(T⁴ − T_space⁴)` (W). Op-order mirrors Python
/// `radiated_power`; `t**4` → `t.powf(4.0)` op-for-op (Tier-2, the plan's libm audit).
fn radiated_power(node_joules: f64, params: &ThermalParams) -> f64 {
    let t = temperature(node_joules, params.heat_capacity, params.space_temperature);
    params.emissivity
        * STEFAN_BOLTZMANN
        * params.radiator_area
        * (t.powf(4.0) - params.space_temperature.powf(4.0))
}

/// Read a donor stock's amount (Python `snapshot.stocks[id].amount` / `KeyError`).
fn donor_amount(snapshot: &State, id: &str) -> Result<f64, SimError> {
    snapshot
        .stocks
        .get(id)
        .map(|s| s.amount)
        .ok_or_else(|| SimError::Reference(format!("flow reads unknown stock {id:?}")))
}

/// ENERGY flow `heat_source → node` — the forced heat input (2-leg, lossless).
pub struct HeatInput {
    id: String,
    heat_source: String,
    node: String,
}

impl Flow for HeatInput {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        _snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let supply = env.get(HEAT_LOAD_VAR)? * dt;
        FlowResult::new(vec![
            Leg::new(self.heat_source.clone(), -supply)?,
            Leg::new(self.node.clone(), supply)?,
        ])
    }
}

/// ENERGY flow `node → boundary.space` — the Stefan-Boltzmann radiator (2-leg, nonlinear).
pub struct RadiatorReject {
    id: String,
    node: String,
    space: String,
    params: ThermalParams,
}

impl RadiatorReject {
    /// Construct a `RadiatorReject` with the given ids — the station reuses it verbatim
    /// (`node`/`space` unchanged) to reject the coupled Power/lamp dissipation load.
    pub fn new(id: String, node: String, space: String, params: ThermalParams) -> Self {
        RadiatorReject {
            id,
            node,
            space,
            params,
        }
    }
}

impl Flow for RadiatorReject {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        // Op-order mirrors Python: radiated_power(node) * dt.
        let rejected = radiated_power(donor_amount(snapshot, &self.node)?, &self.params) * dt;
        FlowResult::new(vec![
            Leg::new(self.node.clone(), -rejected)?,
            Leg::new(self.space.clone(), rejected)?,
        ])
    }
}

/// The sensible-heat POOL `thermal.node` (ENERGY, J), referenced to T_space.
pub fn node_stock(amount: f64) -> Result<Stock, SimError> {
    Stock::new(
        NODE.to_string(),
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

/// Assemble the standalone Thermal system's initial `State` and flow `Registry`.
pub fn build_thermal(
    params: &ThermalParams,
    scenario: &ThermalScenario,
) -> Result<(State, Registry), SimError> {
    let node = node_stock(scenario.node0)?;
    let heat_source = boundary::source(HEAT_SOURCE.to_string(), Quantity::Energy, 0.0, true)?;
    let space = boundary::sink(SPACE.to_string(), Quantity::Energy, 0.0)?;
    let mut stocks: BTreeMap<String, Stock> = BTreeMap::new();
    for s in [node, heat_source, space] {
        stocks.insert(s.id.clone(), s);
    }
    let state = State::new(0, stocks.clone(), 0, BTreeMap::new())?;
    let flows: Vec<Box<dyn Flow>> = vec![
        Box::new(HeatInput {
            id: HEAT_INPUT.to_string(),
            heat_source: HEAT_SOURCE.to_string(),
            node: NODE.to_string(),
        }),
        Box::new(RadiatorReject {
            id: RADIATOR_REJECT.to_string(),
            node: NODE.to_string(),
            space: SPACE.to_string(),
            params: *params,
        }),
    ];
    let registry = Registry::flows_only(flows, &stocks)?;
    Ok((state, registry))
}

/// The forcing: a constant `heat_load` (W) into the node (the radiator is the balance).
pub fn thermal_resolver(scenario: &ThermalScenario) -> Result<SourceResolver, SimError> {
    let mut forcings: HashMap<String, Schedule> = HashMap::new();
    forcings.insert(HEAT_LOAD_VAR.to_string(), constant(scenario.heat_load_w)?);
    SourceResolver::new(forcings, HashMap::new())
}
