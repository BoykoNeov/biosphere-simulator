//! Phase-8 (P8.6) — the **fixed-palette component builder**: the player composes a station
//! from a bounded, code-defined set of parts instead of only picking a whole pre-built
//! scenario.
//!
//! Confirmed decision #1 (USER-CONFIRMED): "build systems" = *place & connect from a fixed,
//! code-defined component palette* (add a battery, a radiator) — **not** declarative
//! authoring (Phase 9). Registry construction stays Rust-side and bounded: each
//! [`Component`] is a **thin delegate to the existing frozen constructors** (a [`Radiator`]
//! is the *same* [`RadiatorReject::new`] with the *same* ids/params [`crate::system::build_station`]
//! uses), and [`assemble`] wires them by the Phase-6 mechanism — *choosing which stock id
//! each flow points at* (finding #1). Components connect where they name a shared canonical
//! stock: Power's dissipation legs land in [`NODE`] when a [`Radiator`] is present, else in
//! the [`WASTE_HEAT`] boundary sink.
//!
//! [`Component::PowerPlant`] · [`Component::Radiator`] · [`Component::SelfDischarge`]
//!
//! # A second byte-identity anchor (the self-discharge part, advisor)
//!
//! `assemble(&[PowerPlant, SelfDischarge], ctx)` (no radiator ⇒ dissipation to
//! [`WASTE_HEAT`]) reproduces the frozen standalone
//! [`domains::power::build_power`]`(charge, BOUNDED_SOC, Some(sd))` **bit-for-bit** — the
//! three-flow leaky microgrid. Compared Rust-vs-Rust (bit-exact by construction on one
//! libm), it chains transitively to the frozen `power_self_discharge` golden via the
//! Step-3 crossport test (`assemble == build_power == golden`), a cleaner statement than a
//! cross-libm band comparison against the Tier-2 golden file.
//!
//! # What makes this step legal under the freeze (the byte-identity anchor, advisor)
//!
//! `assemble(&[PowerPlant, Radiator], ctx)` reproduces [`crate::system::build_station`]
//! **bit-for-bit** — same stocks, same flows, same resolver, so the same trajectory
//! (`tests/builder_parity.rs`). That equivalence proves the builder is a **pure refactor** of
//! existing construction into composable pieces: "no new science" is *proven, not asserted*
//! (the analogue of Step-0's session-parity teeth and Step-5's `health = 1` bit-identity).
//! Because the delegation is exact, the frozen goldens are untouched — the old
//! `build_station` path is unchanged, this is purely additive.
//!
//! # Two IC modes, never a general re-derivation engine (advisor)
//!
//! - **Reproduce path** (`{PowerPlant, Radiator}`): the node's initial heat is the scenario's
//!   exact derived equilibrium ([`crate::system::equilibrium_node_heat`]) → byte-identity.
//! - **Free-composition path**: simple, defensible defaults (a `Radiator` with no `PowerPlant`
//!   starts the node cold at `Q = 0`; a `PowerPlant` with no `Radiator` dissipates to the
//!   boundary). A player who omits the radiator gets a node that heats unbounded — a
//!   **legitimate, conservation-closed *failure* outcome** the Phase-8 exit criterion wants
//!   ("observe failure/stability"), not a bug. There is deliberately **no** per-composition IC
//!   re-derivation engine — that is exactly where new science / unvalidated regimes would leak
//!   in, and where byte-identity would become unprovable.
//!
//! # Not a frozen scenario
//!
//! A composed station is a **runtime object** — not in the freeze manifest, no golden. The
//! frozen reference set (the 20 goldens) does not grow. The engine's per-step conservation
//! assert makes even a novel composition safe: a violation surfaces as a runtime error to the
//! player, never a silent fix.
//!
//! **Scope (P8.6):** single-rate only (`PowerPlant` + `Radiator` — the Power → Thermal
//! station, the one assembly with the temperature / SOC readouts that make composition
//! visible). Two-rate composition (cabin gas / greenhouse / sealed — resolver-pair merge +
//! master-day + reset hook) is deferred exactly as P8.4 deferred two-rate flow inspection.

use std::collections::BTreeMap;

use domains::power::{
    battery_stock, power_resolver, LoadDraw, SelfDischarge, SolarCharge, BATTERY, LOAD_DRAW,
    SELF_DISCHARGE, SOLAR_CHARGE, SOLAR_SOURCE, WASTE_HEAT,
};
use domains::thermal::{node_stock, RadiatorReject, NODE, RADIATOR_REJECT, SPACE};

use simcore::boundary;
use simcore::environment::SourceResolver;
use simcore::error::SimError;
use simcore::flow::Flow;
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::{State, Stock};

use crate::scenario::StationScenario;
use crate::system::equilibrium_node_heat;

/// A fixed-palette component — a thin descriptor delegating to the frozen domain
/// constructors. The player selects a set of these; [`assemble`] turns the set into a runnable
/// `(State, Registry, resolver)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Component {
    /// The power microgrid: the [`BATTERY`] POOL + the unclamped [`SOLAR_SOURCE`] +
    /// [`SolarCharge`] (solar → battery, dissipating charge loss as heat) + [`LoadDraw`]
    /// (battery → heat). Its two dissipation legs land in the thermal node when a
    /// [`Component::Radiator`] is present, else in the [`WASTE_HEAT`] boundary sink.
    PowerPlant,
    /// The thermal-management radiator: the [`NODE`] POOL (the heat accumulator the power
    /// plant dissipates into) + the deep-space [`SPACE`] sink + [`RadiatorReject`] (the
    /// Stefan-Boltzmann node → space rejection). Omit it and the node has no way to shed heat.
    Radiator,
    /// The battery's first-order self-discharge leak ([`SelfDischarge`], `battery → heat`).
    /// **Requires a [`Component::PowerPlant`]** (it reads the battery stock); its heat leg
    /// follows the same dissipation target as the power plant's (the node with a radiator,
    /// else [`WASTE_HEAT`]). Adding it makes the SOC decay below the daily-balanced baseline
    /// (the P5.5 "it earned its keep" behaviour — the first restoring force).
    SelfDischarge,
}

impl Component {
    /// The stable string id the FFI / palette UI uses to name this component.
    pub fn id(self) -> &'static str {
        match self {
            Component::PowerPlant => "power_plant",
            Component::Radiator => "radiator",
            Component::SelfDischarge => "self_discharge",
        }
    }

    /// Parse a palette id back to a [`Component`] (the inverse of [`Component::id`]).
    pub fn from_id(id: &str) -> Option<Component> {
        match id {
            "power_plant" => Some(Component::PowerPlant),
            "radiator" => Some(Component::Radiator),
            "self_discharge" => Some(Component::SelfDischarge),
            _ => None,
        }
    }
}

/// The shared build inputs a composition draws on — the frozen params + the station
/// sub-scenario (`battery0`, the Power sub-scenario driving the diurnal forcing). The
/// reproduce path uses `HEAT_CLOSURE`'s exact values so `assemble` matches `build_station`.
#[derive(Debug, Clone, Copy)]
pub struct BuildContext {
    /// The charge params (`charge_efficiency`) — feeds `SolarCharge` and the node IC.
    pub charge: domains::power::ChargeParams,
    /// The thermal params (ε / area / heat capacity / space temperature) — feeds the radiator
    /// and the node IC.
    pub thermal: domains::thermal::ThermalParams,
    /// The self-discharge rate k — feeds the optional [`Component::SelfDischarge`] leak.
    /// Always supplied (like `charge`/`thermal`); consulted only when that component is chosen.
    pub self_discharge: domains::power::SelfDischargeParams,
    /// The station sub-scenario: `power.battery0`, the diurnal Power scenario, the dt.
    pub scenario: StationScenario,
}

/// Assemble a **single-rate** station from a set of palette components (Phase-8 P8.6).
///
/// Unions each component's stocks, collects its flows (the [`Registry`] sorts by id, so the
/// component order is irrelevant — the perturbations.rs `into_parts` precedent), and merges
/// its forcing vars into one resolver. Power's dissipation legs are wired to [`NODE`] when a
/// [`Component::Radiator`] is in the set, else to a freshly-added [`WASTE_HEAT`] sink.
///
/// Returns the initial `State`, the flow `Registry`, and the merged `SourceResolver` — ready
/// for [`crate::session::SimSession::single_rate`]. Errors on an empty set or a duplicated
/// component (the bounded-palette discipline: each part appears at most once).
pub fn assemble(
    components: &[Component],
    ctx: &BuildContext,
) -> Result<(State, Registry, SourceResolver), SimError> {
    if components.is_empty() {
        return Err(SimError::Validation(
            "builder: a composition needs at least one component".to_string(),
        ));
    }
    // Bounded palette: reject a duplicated component rather than silently merging two
    // conflicting copies of the same stock id.
    for (i, c) in components.iter().enumerate() {
        if components[i + 1..].contains(c) {
            return Err(SimError::Validation(format!(
                "builder: component {:?} listed more than once",
                c.id()
            )));
        }
    }
    let has_power = components.contains(&Component::PowerPlant);
    let has_radiator = components.contains(&Component::Radiator);
    let has_self_discharge = components.contains(&Component::SelfDischarge);

    // Dependency discipline: the self-discharge leak reads the battery stock, so it is
    // meaningless without the power plant that owns it. Reject early with a named part rather
    // than letting the registry surface a bare "unknown stock" referential error.
    if has_self_discharge && !has_power {
        return Err(SimError::Validation(
            "builder: component \"self_discharge\" requires \"power_plant\" (it drains the battery)"
                .to_string(),
        ));
    }

    // The Phase-6 coupling choice, made once: dissipation lands in the node if a radiator is
    // present to shed it, else in the boundary heat sink.
    let dissipation_target = if has_radiator { NODE } else { WASTE_HEAT };

    let mut stocks: BTreeMap<String, Stock> = BTreeMap::new();
    let mut flows: Vec<Box<dyn Flow>> = Vec::new();
    let mut forcings: std::collections::HashMap<String, simcore::environment::Schedule> =
        std::collections::HashMap::new();

    if has_power {
        let ph = &ctx.scenario.power;
        insert(&mut stocks, battery_stock(ph.battery0)?);
        insert(
            &mut stocks,
            boundary::source(SOLAR_SOURCE.to_string(), Quantity::Energy, 0.0, true)?,
        );
        // Only materialize the boundary heat sink when the dissipation has nowhere else to go
        // (no radiator ⇒ no node). With a radiator, the legs point at NODE and WASTE_HEAT must
        // stay absent — that absence is what keeps the `{PowerPlant, Radiator}` stock set
        // exactly `build_station`'s (byte-identity).
        if !has_radiator {
            insert(
                &mut stocks,
                boundary::sink(WASTE_HEAT.to_string(), Quantity::Energy, 0.0)?,
            );
        }
        flows.push(Box::new(SolarCharge::new(
            SOLAR_CHARGE.to_string(),
            SOLAR_SOURCE.to_string(),
            BATTERY.to_string(),
            dissipation_target.to_string(),
            ctx.charge,
        )));
        flows.push(Box::new(LoadDraw::new(
            LOAD_DRAW.to_string(),
            BATTERY.to_string(),
            dissipation_target.to_string(),
        )));
        // The opt-in first-order leak — its heat follows the same dissipation target. With no
        // radiator this reproduces `build_power(.., Some(sd))` bit-for-bit (byte-identity
        // anchor 2); with a radiator the leak simply feeds the node too.
        if has_self_discharge {
            flows.push(Box::new(SelfDischarge::new(
                SELF_DISCHARGE.to_string(),
                BATTERY.to_string(),
                dissipation_target.to_string(),
                ctx.self_discharge,
            )));
        }
        // The Power forcing (diurnal solar half-sine + the derived daily-balanced load).
        let (power_forcings, _shared) = power_resolver(&ctx.charge, ph)?.into_parts();
        forcings.extend(power_forcings);
    }

    if has_radiator {
        // Reproduce path: with a power plant present, the node begins at the dissipation-set
        // equilibrium (byte-identity with `build_station`). Free-composition path: no power
        // input ⇒ start the node cold at Q = 0 (a defensible default, never a re-derivation).
        let node0 = if has_power {
            equilibrium_node_heat(&ctx.charge, &ctx.thermal, &ctx.scenario)
        } else {
            0.0
        };
        insert(&mut stocks, node_stock(node0)?);
        insert(
            &mut stocks,
            boundary::sink(SPACE.to_string(), Quantity::Energy, 0.0)?,
        );
        flows.push(Box::new(RadiatorReject::new(
            RADIATOR_REJECT.to_string(),
            NODE.to_string(),
            SPACE.to_string(),
            ctx.thermal,
        )));
    }

    let state = State::new(0, stocks.clone(), 0, BTreeMap::new())?;
    let registry = Registry::flows_only(flows, &stocks)?;
    let resolver = SourceResolver::new(forcings, std::collections::HashMap::new())?;
    Ok((state, registry, resolver))
}

/// Insert a stock keyed by its own id (the `build_station` idiom).
fn insert(stocks: &mut BTreeMap<String, Stock>, stock: Stock) {
    stocks.insert(stock.id.clone(), stock);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::HEAT_CLOSURE_SCENARIO;

    fn ctx() -> BuildContext {
        BuildContext {
            charge: domains::params::charge(),
            thermal: domains::params::thermal(),
            self_discharge: domains::params::self_discharge(),
            scenario: HEAT_CLOSURE_SCENARIO,
        }
    }

    #[test]
    fn component_id_roundtrips() {
        for c in [Component::PowerPlant, Component::Radiator, Component::SelfDischarge] {
            assert_eq!(Component::from_id(c.id()), Some(c));
        }
        assert_eq!(Component::from_id("no_such"), None);
    }

    #[test]
    fn station_composition_has_build_stations_stock_and_flow_sets() {
        // The structural half of the byte-identity claim (the trajectory half is in
        // tests/builder_parity.rs): {PowerPlant, Radiator} yields exactly build_station's
        // four stocks and three flows, and NO boundary.waste_heat (dissipation → the node).
        let (state, registry, _res) =
            assemble(&[Component::PowerPlant, Component::Radiator], &ctx()).unwrap();
        let mut ids: Vec<&str> = state.stocks.keys().map(|s| s.as_str()).collect();
        ids.sort_unstable();
        let mut want_stocks = vec![BATTERY, SOLAR_SOURCE, NODE, SPACE];
        want_stocks.sort_unstable();
        assert_eq!(ids, want_stocks);
        assert!(!state.stocks.contains_key(WASTE_HEAT), "no shadow heat sink with a radiator");
        let mut flow_ids: Vec<&str> = registry.flows().iter().map(|f| f.id()).collect();
        flow_ids.sort_unstable();
        let mut want_flows = vec![LOAD_DRAW, SOLAR_CHARGE, RADIATOR_REJECT];
        want_flows.sort_unstable();
        assert_eq!(flow_ids, want_flows);
    }

    #[test]
    fn power_plant_alone_dissipates_to_the_boundary_heat_sink() {
        // Free-composition: no radiator ⇒ the two dissipation legs land in boundary.waste_heat
        // (materialized here), and there is no node/space.
        let (state, registry, _res) = assemble(&[Component::PowerPlant], &ctx()).unwrap();
        assert!(state.stocks.contains_key(WASTE_HEAT));
        assert!(!state.stocks.contains_key(NODE));
        assert!(!state.stocks.contains_key(SPACE));
        // Both Power flows point their heat leg at the boundary sink.
        assert_eq!(registry.flows().len(), 2);
    }

    #[test]
    fn radiator_alone_starts_the_node_cold() {
        // Free-composition: a radiator with no power input begins at Q = 0 (the defensible
        // default — never equilibrium_node_heat, which would need the absent power plant).
        let (state, _registry, _res) = assemble(&[Component::Radiator], &ctx()).unwrap();
        assert_eq!(state.stocks[NODE].amount, 0.0);
        assert!(!state.stocks.contains_key(BATTERY));
    }

    #[test]
    fn empty_and_duplicate_compositions_are_rejected() {
        assert!(matches!(assemble(&[], &ctx()), Err(SimError::Validation(_))));
        assert!(matches!(
            assemble(&[Component::PowerPlant, Component::PowerPlant], &ctx()),
            Err(SimError::Validation(_))
        ));
    }

    #[test]
    fn self_discharge_adds_a_third_leaky_flow_over_the_power_plant() {
        // {PowerPlant, SelfDischarge} (no radiator) is the three-flow leaky microgrid: the two
        // forced Power flows + the donor-controlled leak, all dissipating to boundary.waste_heat.
        let (state, registry, _res) =
            assemble(&[Component::PowerPlant, Component::SelfDischarge], &ctx()).unwrap();
        assert!(state.stocks.contains_key(WASTE_HEAT));
        assert!(!state.stocks.contains_key(NODE));
        let mut flow_ids: Vec<&str> = registry.flows().iter().map(|f| f.id()).collect();
        flow_ids.sort_unstable();
        let mut want = vec![SOLAR_CHARGE, LOAD_DRAW, SELF_DISCHARGE];
        want.sort_unstable();
        assert_eq!(flow_ids, want);
    }

    #[test]
    fn self_discharge_requires_a_power_plant() {
        // The leak reads the battery — meaningless without the plant that owns it. Rejected
        // early (a named part), not left to surface as a registry referential error.
        assert!(matches!(
            assemble(&[Component::SelfDischarge], &ctx()),
            Err(SimError::Validation(_))
        ));
        assert!(matches!(
            assemble(&[Component::SelfDischarge, Component::Radiator], &ctx()),
            Err(SimError::Validation(_))
        ));
    }

    #[test]
    fn self_discharge_with_a_radiator_feeds_the_node() {
        // {PowerPlant, Radiator, SelfDischarge}: all three dissipation flows land in the node
        // (no boundary.waste_heat), the leak included.
        let (state, registry, _res) = assemble(
            &[Component::PowerPlant, Component::Radiator, Component::SelfDischarge],
            &ctx(),
        )
        .unwrap();
        assert!(!state.stocks.contains_key(WASTE_HEAT), "no shadow sink with a radiator");
        assert!(state.stocks.contains_key(NODE));
        assert_eq!(registry.flows().len(), 4); // SolarCharge, LoadDraw, SelfDischarge, RadiatorReject
    }
}
