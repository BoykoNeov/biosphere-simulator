//! The Power → biosphere grow lamp — the port of `station.lighting` (P6.5 / P7.5).
//!
//! The phase's **one non-shared-stock coupling** (#16): Power and the biosphere share *no*
//! stock; the whole interface is the **lamp-draw schedule**, which drives both the [`Lamp`]
//! flow (the ENERGY it withdraws from `power.battery`) and the biosphere's `par` /
//! `daylength_s` **forcings** (this module computes `PAR = photon_efficacy·lamp_power/
//! ground_area` and `daylength_s = photoperiod·3600`). The lamp draws a constant
//! **daily-average** power (`substep` freezes `n`, so a within-day top-hat is not an
//! `n`-schedule; daily energy is exact). The `waste_heat` leg lands in `boundary.waste_heat`
//! (the inward move to `thermal.node` is deferred to the sealed station). Tier-2 (FvCB).

use std::collections::BTreeMap;

use domains::biosphere::stocks::{DAYLENGTH_VAR, PAR_VAR, THERMAL_TIME};
use domains::biosphere::system::{build_season, weather_forcings, weather_shared};
use domains::power::{battery_stock, BATTERY, WASTE_HEAT};
use simcore::boundary;
use simcore::environment::{constant, SourceResolver};
use simcore::error::SimError;
use simcore::events::Event;
use simcore::flow::Flow;
use simcore::integrator::EulerIntegrator;
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::{State, Stock};

use crate::driver::run_master_day;
use crate::flows::{Lamp, LampParams, LAMP, LAMP_POWER_VAR};
use crate::scenario::LightingScenario;

/// The radiant-PAR-energy boundary sink id (the `η_lamp` leg of the Lamp flow).
pub const LIGHT_USED: &str = "boundary.light_used";

/// The on-window PAR photon flux the lamp delivers (µmol m⁻² s⁻¹): `photon_efficacy ·
/// lamp_power_w / ground_area` — the biosphere's `par` forcing under the lamp.
pub fn lamp_par(lamp: &LampParams, scenario: &LightingScenario) -> f64 {
    lamp.photon_efficacy * scenario.lamp_power_w / scenario.bio.ground_area
}

/// The constant daily-average lamp electrical power (W): `lamp_power_w · photoperiod / 24`.
pub fn lamp_average_power(scenario: &LightingScenario) -> f64 {
    scenario.lamp_power_w * scenario.photoperiod_hours as f64 / 24.0
}

/// Assemble the lighting station: `(state, bio_registry, power_registry)`.
///
/// The biosphere registry is `build_season`'s output verbatim; the Power registry is the
/// single [`Lamp`] flow over the biosphere stocks ∪ the three Power ENERGY stocks (battery,
/// `light_used`, `waste_heat`). `with_lamp = false` gives the dark baseline (empty Power
/// registry) — not used by the golden.
pub fn build_lighting(
    lamp: &LampParams,
    scenario: &LightingScenario,
    with_lamp: bool,
) -> Result<(State, Registry, Registry), SimError> {
    let (bio_state, bio_registry) = build_season(&scenario.bio)?;
    let mut stocks = bio_state.stocks.clone();

    let power_seq = [
        battery_stock(scenario.battery0)?,
        boundary::sink(LIGHT_USED.to_string(), Quantity::Energy, 0.0)?,
        boundary::sink(WASTE_HEAT.to_string(), Quantity::Energy, 0.0)?,
    ];
    for s in &power_seq {
        if stocks.contains_key(&s.id) {
            return Err(SimError::Validation(format!(
                "lighting stock-id collision between the biosphere and Power: {:?} (Power \
                 and the biosphere share NO stock, only the lamp-draw schedule)",
                s.id
            )));
        }
    }
    let power_stocks: Vec<Stock> = power_seq.to_vec();
    for s in power_stocks {
        stocks.insert(s.id.clone(), s);
    }
    let state = State::new(
        0,
        stocks.clone(),
        0,
        BTreeMap::from([(THERMAL_TIME.to_string(), 0.0)]),
    )?;

    let power_flows: Vec<Box<dyn Flow>> = if with_lamp {
        vec![Box::new(Lamp::new(
            LAMP.to_string(),
            BATTERY.to_string(),
            LIGHT_USED.to_string(),
            WASTE_HEAT.to_string(),
            *lamp,
        ))]
    } else {
        Vec::new()
    };
    let power_registry = Registry::flows_only(power_flows, &stocks)?;
    Ok((state, bio_registry, power_registry))
}

/// The biosphere forcing resolver: weather-driven, with `PAR` + `daylength` from the lamp.
///
/// Rebuilds the weather forcing table ([`weather_forcings`]) and overrides two entries —
/// `PAR_VAR` → [`lamp_par`] (0 when `with_lamp = false`) and `DAYLENGTH_VAR` →
/// `photoperiod·3600` — then reassembles the resolver with the sealed shared map. (The
/// `Box<dyn Fn>` schedules of a built resolver are not `Clone`, so the map is regenerated
/// rather than copied — the Python `dict(base.forcings)` analogue.)
pub fn lighting_bio_resolver(
    lamp: &LampParams,
    scenario: &LightingScenario,
    with_lamp: bool,
) -> Result<SourceResolver, SimError> {
    let mut forcings = weather_forcings(&scenario.bio, 1)?;
    let par = if with_lamp {
        lamp_par(lamp, scenario)
    } else {
        0.0
    };
    forcings.insert(PAR_VAR.to_string(), constant(par)?);
    forcings.insert(
        DAYLENGTH_VAR.to_string(),
        constant(scenario.photoperiod_hours as f64 * 3600.0)?,
    );
    SourceResolver::new(forcings, weather_shared(&scenario.bio))
}

/// The Power forcing: the constant daily-average `lamp_power` draw.
pub fn lighting_power_resolver(scenario: &LightingScenario) -> Result<SourceResolver, SimError> {
    let mut forcings = std::collections::HashMap::new();
    forcings.insert(
        LAMP_POWER_VAR.to_string(),
        constant(lamp_average_power(scenario))?,
    );
    SourceResolver::new(forcings, std::collections::HashMap::new())
}

/// The two-rate driver: one master day = biosphere-slow (once) + Power-fast (×24).
pub fn run_lighting(
    bio_integrator: &EulerIntegrator,
    power_integrator: &EulerIntegrator,
    state: State,
    bio_resolver: &SourceResolver,
    power_resolver: &SourceResolver,
    scenario: &LightingScenario,
) -> Result<(Vec<State>, u64, Vec<Event>), SimError> {
    run_master_day(
        bio_integrator,
        power_integrator,
        state,
        bio_resolver,
        power_resolver,
        scenario.days,
        scenario.steps_per_day,
        scenario.bio_dt,
        scenario.power_dt,
        None,
    )
}
