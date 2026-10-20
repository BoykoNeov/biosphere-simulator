//! One water reserve for the sealed station
//! (`docs/plans/post-roadmap-shared-room-humidity.md` §13): the crew's clean-water tank serves the
//! plants too.
//!
//! The crew and the plants share one room's air, so some of the crew's breath reaches the plants'
//! condenser (plan §2, F3a) and collects in the plants' tank (`biosphere.condensate`). With a
//! reserve ([`WaterReserve::CrewStore`]) the plants' tank keeps one full watering
//! ([`working_level`]); above it, [`TankDrain`] sends the extra to the crew's recycler on the minute
//! step. When a watering needs more than the tank holds, [`ReserveTopUp`] draws the rest from the
//! crew's tank on the plant step. Nothing else in the plants' water moves: the tank still fills from
//! transpiration and the plants' condenser, and the plants' own watering (`EventRecycling`) still
//! draws on it first.

use std::collections::BTreeMap;

use domains::biosphere::flows::{EventRecycling, Recycling};
use domains::biosphere::params::BiosphereParams;
use domains::biosphere::science::{transpirable_capacity, WateringForm};
use domains::biosphere::stocks::{CONDENSATE, ROOTED_DEPTH, SOIL_WATER, SUBSOIL_WATER};
use domains::biosphere::system::{soil_surface_account_with, SeasonScenario, SOIL_SURFACE_ACCOUNT};
use domains::crew::WATER_STORE;
use simcore::environment::Environment;
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::ids::StockId;
use simcore::registry::Registry;
use simcore::state::{State, Stock};

use crate::water::RECOVERED_WATER;

/// Where the sealed station's plants get water their own tank cannot give, and where their
/// extra goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaterReserve {
    /// The plants' tank alone: short waterings stay short, the extra stays in the tank — the form
    /// until 2026-10-20 (plan §13), kept for the lab's records and the separate-air lab, which
    /// waters from the crew's store by its own rule.
    Off,
    /// The crew's clean-water tank is the station's reserve: the plants' tank keeps one full
    /// watering, drains the rest to the crew's recycler, and is topped up from the crew's tank.
    CrewStore,
}

/// The top-up's flow id.
pub const RESERVE_TOP_UP: &str = "station.reserve_top_up";
/// The drain's flow id.
pub const TANK_DRAIN: &str = "station.plant_tank_drain";

fn amount(snapshot: &State, id: &str) -> f64 {
    snapshot.stocks.get(id).map_or(0.0, |s| s.amount)
}

/// What the plants' tank keeps (kg): one full watering of the deepest root zone the crop can
/// reach, from the trigger to full — `depletion_fraction × transpirable_capacity(min(max rooted
/// depth, soil depth)) / drip_efficiency`. A DESIGN value derived from existing settings, no new
/// number (plan §13: 103.3 kg for the reference wheat).
pub fn working_level(bio: &SeasonScenario, p: &BiosphereParams) -> f64 {
    let depth = p.rootd.max_rooted_depth.min(bio.soil_depth);
    p.water.depletion_fraction
        * transpirable_capacity(depth, bio.soil_extractable_water, bio.ground_area)
        / p.water.drip_efficiency
}

/// WATER `water_store → soil_water + subsoil_water`, on the plant step: the part of a watering the
/// plants' tank cannot give. `full` is the plants' own watering law ([`EventRecycling`]) pointed at
/// the crew's tank, which is never short, so its draw is the watering's whole gross; the top-up is
/// that gross less what the plants' tank holds, split as the watering splits. Together with the
/// plants' own watering it delivers exactly the whole watering; with the tank full it gives no legs.
pub struct ReserveTopUp {
    full: EventRecycling,
    tank: String,
}

impl Flow for ReserveTopUp {
    fn type_name(&self) -> &'static str {
        "ReserveTopUp"
    }
    fn id(&self) -> &str {
        RESERVE_TOP_UP
    }
    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let full = self.full.evaluate(snapshot, env, dt)?;
        let gross: f64 = -full
            .legs
            .iter()
            .filter(|l| l.stock == self.full.inner.condensate)
            .map(|l| l.amount)
            .sum::<f64>();
        let short = gross - amount(snapshot, &self.tank).min(gross);
        if short <= 0.0 {
            return Ok(FlowResult::empty());
        }
        let net = short * self.full.application_efficiency;
        // `full`'s reservoir is the crew's tank.
        FlowResult::new(vec![
            Leg::new(self.full.inner.condensate.clone(), -short)?,
            Leg::new(self.full.inner.soil_water.clone(), net)?,
            Leg::new(self.full.subsoil_water.clone(), short - net)?,
        ])
    }
}

/// The top-up for `bio` at `p` — ONE builder, so the soil account's instance is built exactly as
/// the registered flow is. An error unless the plants water on the depletion trigger (the
/// reference's form), the only one a top-up of a short watering has a meaning for.
pub fn reserve_top_up(bio: &SeasonScenario, p: &BiosphereParams) -> Result<ReserveTopUp, SimError> {
    if p.water.watering != WateringForm::Fao56Trigger {
        return Err(SimError::Validation(
            "the water reserve tops up a short watering, and these plants do not water on the \
             depletion trigger"
                .into(),
        ));
    }
    Ok(ReserveTopUp {
        full: EventRecycling {
            inner: Recycling {
                id: RESERVE_TOP_UP.to_string(),
                condensate: WATER_STORE.to_string(),
                soil_water: SOIL_WATER.to_string(),
                recycling_rate: p.water.recycling_rate,
            },
            trigger_ftsw: 1.0 - p.water.depletion_fraction,
            application_efficiency: p.water.drip_efficiency,
            subsoil_water: SUBSOIL_WATER.to_string(),
            rooted_depth_aux: ROOTED_DEPTH.to_string(),
            soil_extractable_water: bio.soil_extractable_water,
            ground_area: bio.ground_area,
        },
        tank: CONDENSATE.to_string(),
    })
}

/// WATER `condensate → recovered_water`, on the minute step: everything in the plants' tank above
/// `level`, to the crew's recycler (which returns 90 % of it to the crew's tank).
pub struct TankDrain {
    level: f64,
}

impl Flow for TankDrain {
    fn type_name(&self) -> &'static str {
        "TankDrain"
    }
    fn id(&self) -> &str {
        TANK_DRAIN
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        _dt: f64,
    ) -> Result<FlowResult, SimError> {
        let extra = amount(snapshot, CONDENSATE) - self.level;
        if extra <= 0.0 {
            return Ok(FlowResult::empty());
        }
        FlowResult::new(vec![
            Leg::new(CONDENSATE.to_string(), -extra)?,
            Leg::new(RECOVERED_WATER.to_string(), extra)?,
        ])
    }
}

/// The two registries with the reserve: the top-up added to the plant step and counted by the soil
/// account (replaced in its own slot, as the separate-air watering's is), the drain added to the
/// minute step. [`WaterReserve::Off`] returns both as they are. ⚠ Apply before `wrap_last`, which
/// rewraps the plant step's aux processes to read the chamber.
pub fn with_water_reserve(
    reserve: WaterReserve,
    bio: &SeasonScenario,
    p: &BiosphereParams,
    bio_reg: Registry,
    fast_reg: Registry,
    stocks: &BTreeMap<StockId, Stock>,
) -> Result<(Registry, Registry), SimError> {
    if reserve == WaterReserve::Off {
        return Ok((bio_reg, fast_reg));
    }
    for id in [CONDENSATE, WATER_STORE, RECOVERED_WATER] {
        if !stocks.contains_key(id) {
            return Err(SimError::Validation(format!(
                "the water reserve needs {id:?}, which this build does not keep"
            )));
        }
    }
    let (mut flows, mut aux) = bio_reg.into_parts();
    flows.push(Box::new(reserve_top_up(bio, p)?));
    if let Some(account) =
        soil_surface_account_with(bio, p, vec![Box::new(reserve_top_up(bio, p)?)])
    {
        let slot = aux
            .iter()
            .position(|a| a.id() == SOIL_SURFACE_ACCOUNT)
            .ok_or_else(|| {
                SimError::Validation(format!(
                    "the water reserve: the soil form is on but the season carries no \
                     {SOIL_SURFACE_ACCOUNT:?} account to count the top-up"
                ))
            })?;
        aux[slot] = account;
    }
    let (mut fast_flows, fast_aux) = fast_reg.into_parts();
    fast_flows.push(Box::new(TankDrain {
        level: working_level(bio, p),
    }));
    Ok((
        Registry::new(flows, stocks, aux)?,
        Registry::new(fast_flows, stocks, fast_aux)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_reference_wheat_keeps_one_watering_of_its_deepest_roots() {
        let sc = crate::scenario::sealed_station_scenario();
        let level = working_level(&sc.bio, &sc.plant_params());
        // 0.55 × 1.3 m × 0.13 × 1000 kg m⁻³ × 1 m² / 0.9 (plan §13).
        assert!(
            (level - 0.55 * 1.3 * 0.13 * 1000.0 / 0.9).abs() < 1e-9,
            "{level}"
        );
    }
}
