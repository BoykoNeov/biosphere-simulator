//! **LAB-ONLY** — one moisture store for a room the crew and the plants share
//! (`docs/plans/post-roadmap-shared-room-humidity.md`).
//!
//! Three station builds put the crew and the plants in one room (`greenhouse`, `harvest`,
//! `sealed_station`): the crew breathes the plants' CO₂ and O₂ pools. Their water vapour is not
//! shared in the reference ([`RoomVapour::TwoStocks`]): the crew's breath goes to `cabin_h2o`, held
//! at 40 % of a stored 22 °C saturation by the ECLSS condenser, and the plants' transpiration to
//! `water_vapor`, held at 75 % of saturation at the plants' temperature. Counted together the room
//! is above saturation on almost every step (plan §1).
//!
//! [`RoomVapour::OneStock`] gives the room one vapour stock, the plants' `water_vapor`, and both
//! condensers one target: `setting` × saturation at the temperature the plants' water reads
//! ([`water_air_vars`]). The crew's condenser takes the plants' frozen law, everything above the
//! target ([`Condensation`]), on the minute step, instead of the ECLSS first-order law: with that
//! law's standing excess the plants' condenser would sweep about a third of the crew's water into
//! the soil (plan §2, F3). Its output still goes where the crew's condensate went.
//!
//! The two condensers read one thermostat: the minute one reads the plant window's recorded value
//! of the plants' water temperature ([`OnFastStep`]), so it needs the minute gas exchange's
//! recorder, plus [`WithinDayAirRecorder`] in a build whose water reads the within-day air.
//!
//! ⚠ Not exact: the minute's crew breath left after the minute condenser, and the target falling
//! between plant steps, reach the plants' condenser (plan §2, F3a).

use std::collections::{BTreeMap, HashMap};

use domains::biosphere::flows::Condensation;
use domains::biosphere::params::BiosphereParams;
use domains::biosphere::stocks::{AIR_TEMP_WITHIN_DAY_VAR, TEMP_VAR, WATER_VAPOR};
use domains::biosphere::system::water_air_vars;
use domains::eclss::CABIN_H2O;
use simcore::auxiliary::AuxProcess;
use simcore::environment::Environment;
use simcore::error::SimError;
use simcore::flow::Flow;
use simcore::ids::StockId;
use simcore::registry::Registry;
use simcore::state::{State, Stock};

use crate::gas_exchange::{window_key, OnFastStep, PLANT_WINDOW_RECORDER};

/// The shared room's water vapour: two stocks (the reference) or one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoomVapour {
    /// The crew's `cabin_h2o` and the plants' `water_vapor`, each held by its own condenser.
    TwoStocks,
    /// One stock, `water_vapor`, held at `setting` (a fraction of saturation) by both condensers;
    /// the minute one reads `thermostat`.
    OneStock {
        setting: f64,
        thermostat: Thermostat,
    },
}

/// The temperature the room's minute condenser holds its target at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Thermostat {
    /// The plant window's recorded value of the plants' water temperature: ONE thermostat with
    /// the plants' condenser. Lags the room when it changes inside a plant step (plan §8).
    Window,
    /// The chamber's live heat-store temperature, every minute (a build with a chamber only).
    LiveChamber,
}

impl RoomVapour {
    /// The plants' params in this room: under [`RoomVapour::OneStock`] the room's setting is the
    /// plants' `humidity_setpoint`, the one copy every reader takes — transpiration's split, the
    /// plants' condenser, and [`room_condenser`].
    pub fn plant_params(self, mut p: BiosphereParams) -> BiosphereParams {
        if let RoomVapour::OneStock { setting, .. } = self {
            p.water.humidity_setpoint = setting;
        }
        p
    }

    /// The stock the crew's breath goes to.
    pub fn crew_vapour(self) -> &'static str {
        match self {
            RoomVapour::TwoStocks => CABIN_H2O,
            RoomVapour::OneStock { .. } => WATER_VAPOR,
        }
    }

    /// Does the room build the crew's own vapour stock?
    pub fn has_cabin_vapour(self) -> bool {
        self == RoomVapour::TwoStocks
    }

    /// The minute condenser's thermostat, or `None` for two stocks.
    pub fn thermostat(self) -> Option<Thermostat> {
        match self {
            RoomVapour::TwoStocks => None,
            RoomVapour::OneStock { thermostat, .. } => Some(thermostat),
        }
    }
}

/// The minute condenser's flow id.
pub const ROOM_CONDENSER: &str = "station.room_condenser";

/// The within-day air recorder's aux-process id.
pub const WITHIN_DAY_AIR_RECORDER: &str = "station.plant_window_air";

const DAILY_MEAN_AIR: [&str; 1] = [TEMP_VAR];
const WITHIN_DAY_AIR: [&str; 1] = [AIR_TEMP_WITHIN_DAY_VAR];

/// The plant window's forcings the minute condenser reads, for params `p`.
fn condenser_window(p: &BiosphereParams) -> &'static [&'static str] {
    if water_air_vars(p).0 == TEMP_VAR {
        &DAILY_MEAN_AIR
    } else {
        &WITHIN_DAY_AIR
    }
}

/// The crew's condenser in a one-store room: the plants' [`Condensation`] law, on the minute
/// step, at the plants' setting (`p`, already through [`RoomVapour::plant_params`]), into `dest`
/// (where the crew's condensate goes in this build). Under [`Thermostat::Window`] it reads the
/// plants' water temperature for the window; under [`Thermostat::LiveChamber`] the chamber's.
pub fn room_condenser(
    p: &BiosphereParams,
    air_capacity_mol: f64,
    dest: &str,
    thermostat: Thermostat,
) -> Box<dyn Flow> {
    if thermostat == Thermostat::LiveChamber {
        return crate::chamber::reads_chamber(
            Box::new(Condensation {
                id: ROOM_CONDENSER.to_string(),
                water_vapor: WATER_VAPOR.to_string(),
                condensate: dest.to_string(),
                temp_var: TEMP_VAR.to_string(),
                air_capacity_mol,
                humidity_setpoint: p.water.humidity_setpoint,
            }),
            &crate::params::chamber(),
        );
    }
    let window = condenser_window(p);
    Box::new(OnFastStep::reading(
        Box::new(Condensation {
            id: ROOM_CONDENSER.to_string(),
            water_vapor: WATER_VAPOR.to_string(),
            condensate: dest.to_string(),
            temp_var: window[0].to_string(),
            air_capacity_mol,
            humidity_setpoint: p.water.humidity_setpoint,
        }),
        HashMap::new(),
        window,
    ))
}

/// The plant-step aux process that records the within-day air for its window, under
/// [`window_key`]`(AIR_TEMP_WITHIN_DAY_VAR)`, as [`crate::gas_exchange::PlantWindowRecorder`]
/// records the daily mean. Its own recorder, since the aux channel is additive and a second
/// writer of one key would double it.
pub struct WithinDayAirRecorder;

impl AuxProcess for WithinDayAirRecorder {
    fn type_name(&self) -> &'static str {
        "WithinDayAirRecorder"
    }

    fn id(&self) -> &str {
        WITHIN_DAY_AIR_RECORDER
    }

    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        _dt: f64,
    ) -> Result<BTreeMap<String, f64>, SimError> {
        let key = window_key(AIR_TEMP_WITHIN_DAY_VAR);
        let old = snapshot.aux.get(&key).copied().unwrap_or(0.0);
        Ok(BTreeMap::from([(
            key,
            env.get(AIR_TEMP_WITHIN_DAY_VAR)? - old,
        )]))
    }
}

/// The plant registry of a one-store room, made ready for [`room_condenser`]: an error unless the
/// minute gas exchange's recorder is there (it records the daily mean), and
/// [`WithinDayAirRecorder`] added when the plants' water reads the within-day air. A two-store
/// room's registry is returned as it is.
pub fn record_room_air(
    room: RoomVapour,
    p: &BiosphereParams,
    bio_reg: Registry,
    stocks: &BTreeMap<StockId, Stock>,
) -> Result<Registry, SimError> {
    if room.has_cabin_vapour() {
        return Ok(bio_reg);
    }
    let (flows, mut aux) = bio_reg.into_parts();
    if !aux.iter().any(|a| a.id() == PLANT_WINDOW_RECORDER) {
        return Err(SimError::Validation(
            "one vapour store for a shared room reads the plant window's temperature, which only \
             the minute gas exchange's recorder holds: take the gas exchange on the minute step"
                .into(),
        ));
    }
    if condenser_window(p)[0] == AIR_TEMP_WITHIN_DAY_VAR {
        aux.push(Box::new(WithinDayAirRecorder));
    }
    Registry::new(flows, stocks, aux)
}
