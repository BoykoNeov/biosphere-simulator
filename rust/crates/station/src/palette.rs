//! The fixed, code-defined **scenario palette** (Phase-8) — the gdext-free catalogue of
//! named sessions the front-end and the headless CLI both build from.
//!
//! # Why this lives in `station`, not `godot_bridge` (the by-construction exit claim)
//!
//! The Phase-8 exit criterion is that *"the **exact same** simulation runs headless."* That
//! is only true **by construction** if every entry point — the Godot cdylib and the headless
//! CLI (`src/bin/sim.rs`) — builds its session through **one shared builder**. So the
//! named-scenario dispatch lives here, in the engine-side `station` crate (no `gdext`), and:
//!
//! - [`godot_bridge`]'s `build_session` is a thin wrapper that calls [`build_scenario`] and
//!   adds nothing but the Godot plumbing;
//! - the headless `sim` bin calls [`build_scenario`] directly.
//!
//! Neither re-implements the wiring, so they cannot drift — the same extraction discipline
//! as Phase-8 Step 0 ([`crate::driver::advance_one_master_day`],
//! [`crate::sealed::sealed_reset_hook`]) and Phase-7 (`crew::carbon_split` made `pub`).
//!
//! Each builder returns the [`SimSession`] **and** its [`DisplayContext`] — the CLI simply
//! ignores the context (it only emits the bit-exact snapshot), while the bridge uses it for
//! the display projection. One function, both callers: the shared-builder guarantee.

use simcore::error::SimError;
use simcore::integrator::EulerIntegrator;

use domains::crew::FECAL_WASTE;

use crate::display::{BatteryReadout, DisplayContext, ThermalReadout};
use crate::session::SimSession;

/// Build the owned [`SimSession`] **and its [`DisplayContext`]** for a fixed-palette
/// scenario id (confirmed decision #1: "build systems" = a fixed, code-defined palette;
/// registry construction stays in Rust). The four palette entries:
///
/// - `"cabin_gas"` — the crew ↔ ECLSS cabin-air loop (single-rate, Tier-1; the cross-boundary
///   parity tripwire, no thermal/battery scalars);
/// - `"station"` — Power → Thermal heat closure (single-rate; the one entry with a real node
///   temperature and battery SOC);
/// - `"greenhouse"` — biosphere ↔ cabin (two-rate, `reset = None`, 7-day);
/// - `"sealed"` — the full multi-year re-sown station (two-rate, the "fast-forward decades"
///   entry).
///
/// An unknown id is a loud [`SimError::Validation`].
pub fn build_scenario(scenario_id: &str) -> Result<(SimSession, DisplayContext), SimError> {
    match scenario_id {
        "cabin_gas" => build_cabin_gas(),
        "station" => build_station(),
        "greenhouse" => build_greenhouse_session(),
        "sealed" => build_sealed_session(),
        other => Err(SimError::Validation(format!(
            "station::palette: unknown scenario id {other:?} (palette: \"cabin_gas\", \
             \"station\", \"greenhouse\", \"sealed\")"
        ))),
    }
}

/// The coupled crew ↔ ECLSS `CABIN_GAS_SCENARIO` as a single-rate session — mirrors the
/// [`crate::run_station`] setup (and `tests/session_parity.rs`) exactly. No thermal node
/// or battery (both readouts project to `None`); the shared stocks are the three cabin-air
/// pools the crew breathes and ECLSS regulates (a construction-time fact of the assembly).
fn build_cabin_gas() -> Result<(SimSession, DisplayContext), SimError> {
    let crew = domains::params::crew();
    let eclss = domains::params::eclss();
    let scenario = crate::scenario::CABIN_GAS_SCENARIO;
    let (state, registry) = crate::cabin::build_cabin(&crew, &eclss, &scenario)?;
    let resolver = crate::cabin::cabin_resolver(&scenario)?;
    let session = SimSession::single_rate(
        EulerIntegrator::new(registry),
        state,
        resolver,
        scenario.dt_seconds,
    );
    let ctx = DisplayContext {
        thermal: None,
        battery: None,
        shared_stock_ids: vec![
            domains::eclss::CABIN_O2.to_string(),
            domains::eclss::CABIN_CO2.to_string(),
            domains::eclss::CABIN_H2O.to_string(),
        ],
    };
    Ok((session, ctx))
}

/// The coupled Power → Thermal `HEAT_CLOSURE_SCENARIO` as a single-rate session — mirrors
/// the [`crate::system::build_station`] setup (and `examples/emit_station.rs`) exactly.
/// This is the palette entry with a real node temperature and battery SOC: the display
/// context carries the thermal params (`T = T_space + Q/C`) and the initial battery charge
/// (the SOC reference), and highlights `thermal.node` — the stock Power dissipates into and
/// Thermal radiates from (cross-domain by construction, the Step-1 seam).
fn build_station() -> Result<(SimSession, DisplayContext), SimError> {
    let charge = domains::params::charge();
    let thermal = domains::params::thermal();
    let scenario = crate::scenario::HEAT_CLOSURE_SCENARIO;
    let (state, registry) = crate::system::build_station(&charge, &thermal, &scenario, None)?;
    let resolver = crate::system::station_resolver(&charge, &scenario)?;
    let session = SimSession::single_rate(
        EulerIntegrator::new(registry),
        state,
        resolver,
        scenario.power.dt_seconds,
    );
    let ctx = DisplayContext {
        thermal: Some(ThermalReadout {
            node_id: domains::thermal::NODE.to_string(),
            heat_capacity: thermal.heat_capacity,
            space_temperature: thermal.space_temperature,
        }),
        battery: Some(BatteryReadout {
            battery_id: domains::power::BATTERY.to_string(),
            initial_charge: scenario.power.battery0,
        }),
        shared_stock_ids: vec![domains::thermal::NODE.to_string()],
    };
    Ok((session, ctx))
}

/// The biosphere ↔ cabin `greenhouse` as a **two-rate** session — the palette's first
/// two-rate entry, and the reason time controls run off the render thread: each
/// [`SimSession::step`] is one **master day** = one slow biosphere step + `steps_per_day`
/// (1440) fast cabin sub-steps. Mirrors [`crate::greenhouse::run_greenhouse`]'s setup (and
/// `tests/session_parity.rs`) with `reset = None`. The shared stocks are the biosphere
/// carbon/O₂ pools the cabin flows are re-pointed at (a construction-time fact of the
/// reversed greenhouse seam). No thermal node or battery, so both scalars project to `None`.
fn build_greenhouse_session() -> Result<(SimSession, DisplayContext), SimError> {
    let crew = domains::params::crew();
    let eclss = domains::params::eclss();
    let scenario = crate::scenario::greenhouse_scenario();
    let (state, bio_registry, cabin_registry) =
        crate::greenhouse::build_greenhouse(&crew, &eclss, &scenario, true, FECAL_WASTE)?;
    let session = SimSession::two_rate(
        EulerIntegrator::new(bio_registry),
        EulerIntegrator::new(cabin_registry),
        state,
        crate::greenhouse::greenhouse_bio_resolver(&scenario)?,
        crate::greenhouse::greenhouse_cabin_resolver(&scenario)?,
        scenario.steps_per_day,
        scenario.bio_dt,
        scenario.cabin_dt,
        None,
    )?;
    let ctx = DisplayContext {
        thermal: None,
        battery: None,
        shared_stock_ids: vec![
            domains::biosphere::stocks::CARBON_POOL.to_string(),
            domains::biosphere::stocks::O2_POOL.to_string(),
        ],
    };
    Ok((session, ctx))
}

/// The full sealed station as a **two-rate** session — the multi-year, re-sown scenario that
/// *is* "fast-forward decades." Mirrors [`crate::sealed::run_sealed`]'s construction (and the
/// sealed branch of `tests/session_parity.rs`): every Phase-6 seam over one shared stock dict
/// and two registries, with the real `sealed_reset_hook` re-sowing the biosphere each season.
/// It carries a real node temperature and battery SOC (Power → Thermal is inside), and highlights
/// the cross-domain shared stocks (`thermal.node` + the biosphere carbon/O₂ pools the cabin
/// breathes). Each master day is 1440 fast sub-steps, and a decade is thousands of master days
/// — the palette entry the off-render-thread fast-forward exists for.
fn build_sealed_session() -> Result<(SimSession, DisplayContext), SimError> {
    let charge = domains::params::charge();
    let thermal = domains::params::thermal();
    let crew = domains::params::crew();
    let eclss = domains::params::eclss();
    let recovery = crate::params::water_recovery();
    let lamp = crate::params::lamp();
    let harvest = crate::params::harvest();
    let scenario = crate::scenario::sealed_station_scenario();
    let (state, bio_registry, fast_registry) = crate::sealed::build_sealed_station(
        &charge, &thermal, &crew, &eclss, &recovery, &lamp, &harvest, &scenario, false, false,
    )?;
    let session = SimSession::two_rate(
        EulerIntegrator::new(bio_registry),
        EulerIntegrator::new(fast_registry),
        state,
        crate::sealed::sealed_bio_resolver(&lamp, &scenario)?,
        crate::sealed::sealed_fast_resolver(&charge, &scenario)?,
        scenario.steps_per_day,
        scenario.bio_dt,
        scenario.cabin_dt,
        Some(crate::sealed::sealed_reset_hook(&scenario)),
    )?;
    let ctx = DisplayContext {
        thermal: Some(ThermalReadout {
            node_id: domains::thermal::NODE.to_string(),
            heat_capacity: thermal.heat_capacity,
            space_temperature: thermal.space_temperature,
        }),
        battery: Some(BatteryReadout {
            battery_id: domains::power::BATTERY.to_string(),
            initial_charge: scenario.battery0,
        }),
        shared_stock_ids: vec![
            domains::thermal::NODE.to_string(),
            domains::biosphere::stocks::CARBON_POOL.to_string(),
            domains::biosphere::stocks::O2_POOL.to_string(),
        ],
    };
    Ok((session, ctx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_scenario_knows_the_palette_and_rejects_unknown() {
        assert!(build_scenario("cabin_gas").is_ok());
        assert!(build_scenario("station").is_ok());
        assert!(build_scenario("greenhouse").is_ok());
        assert!(build_scenario("sealed").is_ok());
        match build_scenario("no_such_scenario") {
            Err(SimError::Validation(_)) => {}
            Err(other) => panic!("expected Validation error, got {other:?}"),
            Ok(_) => panic!("unknown scenario id must not build"),
        }
    }

    /// A single-rate palette entry built through the shared builder steps the frozen horizon
    /// well-fed — the same Tier-0 payload the emit example asserts (`rationed == 0`, no events).
    #[test]
    fn cabin_gas_steps_well_fed() {
        let (mut session, ctx) = build_scenario("cabin_gas").unwrap();
        session.step_n(crate::scenario::CABIN_GAS_STEPS).unwrap();
        assert_eq!(session.n(), crate::scenario::CABIN_GAS_STEPS);
        assert_eq!(session.total_rationed(), 0);
        assert!(session.events().is_empty());
        // The display context declares the three cabin-air pools as the shared stocks and no
        // thermal/battery scalars (a construction-time fact of the cabin assembly).
        assert!(ctx.thermal.is_none() && ctx.battery.is_none());
        assert_eq!(ctx.shared_stock_ids.len(), 3);
    }

    /// The `station` entry carries the thermal + battery readouts and highlights `thermal.node`.
    #[test]
    fn station_carries_thermal_and_battery_context() {
        let (mut session, ctx) = build_scenario("station").unwrap();
        session.step_n(crate::scenario::HEAT_CLOSURE_DAYS * 24).unwrap();
        assert_eq!(session.total_rationed(), 0);
        assert!(ctx.thermal.is_some() && ctx.battery.is_some());
        assert_eq!(ctx.shared_stock_ids, vec![domains::thermal::NODE.to_string()]);
    }

    /// The two-rate `sealed` entry builds with the real re-sow hook and steps a few master days.
    #[test]
    fn sealed_steps_a_few_master_days() {
        let (mut session, _ctx) = build_scenario("sealed").unwrap();
        session.step_n(3).unwrap();
        assert_eq!(session.n(), 3);
        assert_eq!(session.total_rationed(), 0);
        assert!(session.events().is_empty());
    }
}
