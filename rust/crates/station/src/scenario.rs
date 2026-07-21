//! Station scenario data — the Rust port of `station.scenario` (Phase-7 P7.5).
//!
//! Each station scenario **references** the already-validated sibling / biosphere scenarios
//! and adds only the cross-domain wiring / timing choices (the Python
//! `StationScenario`-wrapping-`PowerScenario` rhythm). Only the fields the eight coupled
//! goldens exercise are ported.

use domains::biosphere::{SeasonScenario, DEFAULT_SCENARIO, LONG_HORIZON_YEARS};
use domains::power::{PowerScenario, BOUNDED_SOC_SCENARIO};

// --- Step 1 (P6.1): the Power → Thermal heat-closure station --------------------------

/// Station run data: which sibling scenarios the coupled station is assembled from.
#[derive(Debug, Clone, Copy)]
pub struct StationScenario {
    /// The Power sub-scenario driving the station (its dissipation feeds the node).
    pub power: PowerScenario,
}

/// `HEAT_CLOSURE_SCENARIO`: Power's daily-balanced microgrid feeding the Thermal node.
pub const HEAT_CLOSURE_SCENARIO: StationScenario = StationScenario {
    power: BOUNDED_SOC_SCENARIO,
};

/// The golden / bounded-node horizon (days).
pub const HEAT_CLOSURE_DAYS: u64 = 7;

// --- Step 2 (P6.2): the crew ↔ ECLSS cabin gas loop ----------------------------------

/// Step-2 gas-loop run data (the crew↔ECLSS cabin-air coupling).
#[derive(Debug, Clone, Copy)]
pub struct CabinScenario {
    /// Initial cabin O₂ (mol) — starts AT the ECLSS setpoint.
    pub cabin_o2_0: f64,
    /// Initial cabin CO₂ (mol).
    pub cabin_co2_0: f64,
    /// Initial cabin H₂O (kg).
    pub cabin_h2o_0: f64,
    /// Finite provisioned food store (mol carbon).
    pub food_store0: f64,
    /// Finite provisioned water store (kg).
    pub water_store0: f64,
    /// Forced food-carbon intake rate (mol/s).
    pub food_intake_rate: f64,
    /// Forced water intake rate (kg/s).
    pub water_intake_rate: f64,
    /// Integration step (s) — 60 s, ECLSS's binding `k_scrub·dt < 1`.
    pub dt_seconds: f64,
}

/// `CABIN_GAS_SCENARIO`: the crew respiring into / breathing from the ECLSS cabin.
pub const CABIN_GAS_SCENARIO: CabinScenario = CabinScenario {
    cabin_o2_0: 10.0,
    cabin_co2_0: 0.0,
    cabin_h2o_0: 0.0,
    food_store0: 1000.0,
    water_store0: 20.0,
    food_intake_rate: 4.0e-3,
    water_intake_rate: 5.0e-5,
    dt_seconds: 60.0,
};

/// The cabin steady-state-run horizon (steps).
pub const CABIN_GAS_STEPS: u64 = 900;

// --- Step 4 (P6.4): the crew water-recovery loop -------------------------------------

/// `WATER_RECOVERY_SCENARIO`: reuses the Step-2 cabin sizing verbatim.
pub const WATER_RECOVERY_SCENARIO: CabinScenario = CABIN_GAS_SCENARIO;

/// The water-recovery horizon (steps) — reuses `CABIN_GAS_STEPS`.
pub const WATER_RECOVERY_STEPS: u64 = CABIN_GAS_STEPS;

// --- Step 3 (P6.3): the biosphere ↔ cabin greenhouse ---------------------------------

/// The greenhouse biosphere: a sealed chamber whose CO₂/O₂ pools ARE the cabin air,
/// cabin-sized fills (`GREENHOUSE_BIO_SCENARIO`).
pub fn greenhouse_bio_scenario() -> SeasonScenario {
    SeasonScenario {
        sealed: true,
        chamber_o2_mol0: 10.0,
        chamber_co2_mol0: 3.796,
        chamber_air_mol: 9500.0,
        litter_carbon0: 0.0,
        ..DEFAULT_SCENARIO
    }
}

/// The greenhouse crew stores, re-sized for the multi-day horizon
/// (`GREENHOUSE_CABIN_SCENARIO`).
pub const GREENHOUSE_CABIN_SCENARIO: CabinScenario = CabinScenario {
    food_store0: 4000.0,
    water_store0: 50.0,
    ..CABIN_GAS_SCENARIO
};

/// Step-3 biosphere ↔ cabin run data (plants + crew share the cabin air).
#[derive(Debug, Clone, Copy)]
pub struct GreenhouseScenario {
    /// The sealed biosphere whose gas pools are the cabin air.
    pub bio: SeasonScenario,
    /// The crew stores + intake rates + initial humidity.
    pub cabin: CabinScenario,
    /// Horizon in master steps (days).
    pub days: usize,
    /// Cabin sub-steps per biosphere day (1440 = 86400/60).
    pub steps_per_day: u64,
    /// The cabin sub-step dt (s).
    pub cabin_dt: f64,
    /// The biosphere structural step (day).
    pub bio_dt: f64,
}

/// `GREENHOUSE_SCENARIO`: the frozen sealed biosphere breathing the crew's cabin air.
pub fn greenhouse_scenario() -> GreenhouseScenario {
    GreenhouseScenario {
        bio: greenhouse_bio_scenario(),
        cabin: GREENHOUSE_CABIN_SCENARIO,
        days: 7,
        steps_per_day: 1440,
        cabin_dt: 60.0,
        bio_dt: 1.0,
    }
}

// --- Step 5 (P6.5): Power → biosphere lighting ---------------------------------------

/// The lighting biosphere: a plain sealed self-contained chamber (`LIGHTING_BIO_SCENARIO`).
pub fn lighting_bio_scenario() -> SeasonScenario {
    SeasonScenario {
        sealed: true,
        ..DEFAULT_SCENARIO
    }
}

/// Step-5 Power → biosphere lighting run data (the lamp carries energy into biology).
#[derive(Debug, Clone, Copy)]
pub struct LightingScenario {
    /// The sealed self-contained biosphere the lamp lights.
    pub bio: SeasonScenario,
    /// Initial provisioned battery energy (J).
    pub battery0: f64,
    /// The lamp's on-window electrical power (W).
    pub lamp_power_w: f64,
    /// Photoperiod (integer hours of lamp-on per day).
    pub photoperiod_hours: u64,
    /// Horizon in master steps (days).
    pub days: usize,
    /// Power sub-steps per biosphere day (24).
    pub steps_per_day: u64,
    /// The Power sub-step dt (s).
    pub power_dt: f64,
    /// The biosphere structural step (day).
    pub bio_dt: f64,
    /// Optional constant habitat air temperature (°C). `None` (the default / frozen
    /// `lighting_scenario()`) keeps the weather-table temperature untouched — the frozen
    /// golden is byte-identical. `Some(t)` overrides `TEMP_VAR` with `constant(t)` in
    /// [`crate::lighting::lighting_bio_resolver`] (beside the lamp's PAR / daylength
    /// overrides) — a warm, lamp-lit habitat: the lamp supplies light, the environment
    /// supplies warmth. Authored content, not a controlled-environment chamber (VPD / net
    /// radiation stay weather-driven — the deferred refinement the lighting docstring
    /// names).
    pub habitat_temp_c: Option<f64>,
}

/// The day-neutral crop the lamp-lit habitat needs (scope (B) day-neutral crop): the
/// sealed self-contained chamber with BOTH phenology gates removed (`vernalization =
/// false`, `photoperiod = false`) + `litter_carbon0 = 3` decomposer fuel. Reuses the same
/// cited winter-wheat params — a day-neutral wheat is winter-wheat physiology with the
/// gates removed, not a new param file (the Python `DAY_NEUTRAL_SCENARIO` analogue).
pub fn day_neutral_lighting_bio_scenario() -> SeasonScenario {
    SeasonScenario {
        sealed: true,
        litter_carbon0: 3.0,
        vernalization: false,
        photoperiod: false,
        ..DEFAULT_SCENARIO
    }
}

/// `LIGHTING_SCENARIO`: the sealed biosphere lit by a battery-powered grow lamp.
pub fn lighting_scenario() -> LightingScenario {
    LightingScenario {
        bio: lighting_bio_scenario(),
        battery0: 2.0e8,
        lamp_power_w: 200.0,
        photoperiod_hours: 16,
        days: 7,
        steps_per_day: 24,
        power_dt: 3600.0,
        bio_dt: 1.0,
        habitat_temp_c: None,
    }
}

/// A warm, lamp-lit habitat growing the DAY-NEUTRAL crop — the lamp-lit-habitat *product*
/// form the day-neutral crop was made for (`docs/plans/post-roadmap-day-neutral-crop.md`,
/// "Lamp-lit `LightingScenario` wiring"). At 20 °C the frozen winter wheat is permanently
/// arrested (no cold cue ⇒ `verfun ≡ 0`), while this day-neutral crop develops on thermal
/// time under the lamp. Authored content (conservation + determinism, no golden — "authored
/// ≠ validated"): the battery is sized well-fed over the longer development horizon (the
/// lamp draws `1.152e7 J/day`).
pub fn day_neutral_lighting_scenario() -> LightingScenario {
    LightingScenario {
        bio: day_neutral_lighting_bio_scenario(),
        battery0: 3.0e9,
        lamp_power_w: 200.0,
        photoperiod_hours: 16,
        days: 120,
        steps_per_day: 24,
        power_dt: 3600.0,
        bio_dt: 1.0,
        habitat_temp_c: Some(20.0),
    }
}

// --- Step 6 (P6.6): the biomass/food loop --------------------------------------------

/// The harvest horizon (master days).
pub const HARVEST_DAYS: usize = 7;

/// Step-6 biomass/food-loop run data (grain → crew `food_store`).
#[derive(Debug, Clone, Copy)]
pub struct HarvestScenario {
    /// The Step-3 greenhouse this is built on (only the Step-6 horizon named).
    pub greenhouse: GreenhouseScenario,
    /// The biosphere `thermal_time` accumulator at t=0 (°C·day), past anthesis.
    pub thermal_time0: f64,
}

/// `HARVEST_SCENARIO`: the reproductive greenhouse plant filling grain the harvest drains.
pub fn harvest_scenario() -> HarvestScenario {
    HarvestScenario {
        greenhouse: GreenhouseScenario {
            days: HARVEST_DAYS,
            ..greenhouse_scenario()
        },
        thermal_time0: 1300.0,
    }
}

// --- Step 7 (P6.7): the sealed station -----------------------------------------------

/// The season length / re-sow period (days).
pub const SEALED_STATION_SEASON_DAYS: usize = 305;

/// The Tier-2 combined-ledger horizon (whole seasons). 4 since the scope-B decomposer
/// calibration (2026-07-21): the enlarged soil-pool equilibria lengthen the year-1
/// soil-establishment spin-up, so the Python biomass gate needs 2 settled post-spin-up
/// diffs. Mirrors Python `station.scenario.SEALED_STATION_YEARS`; the golden is horizon 4.
pub const SEALED_STATION_YEARS: usize = 4;

/// The Tier-1 energy-decade horizon (years) — reuses the biosphere's decade horizon.
pub const SEALED_ENERGY_YEARS: usize = LONG_HORIZON_YEARS;

/// The Tier-1 energy-decade horizon (days).
pub const SEALED_ENERGY_DAYS: u64 = (SEALED_ENERGY_YEARS * SEALED_STATION_SEASON_DAYS) as u64;

/// The Phase-8 (P8.8) sealed **cross-boundary parity** horizon (master days) — a handful of
/// days past one season so the smoke exercises the re-sow (`slow_reset`) adopt branch across
/// the Godot FFI boundary, without paying the full multi-year run through headless Godot (the
/// full-horizon parity is proven intra-process in `tests/session_parity.rs` + the frozen
/// `sealed_station_state.json` golden). Shared by `examples/emit_sealed_resume.rs` and
/// `godot/sealed_smoke.gd`.
pub const SEALED_RESUME_DAYS: u64 = SEALED_STATION_SEASON_DAYS as u64 + 5;

/// The Tier-2 perennial-capable sealed biosphere (greenhouse chamber + `litter_carbon0=3`).
pub fn sealed_station_bio_scenario() -> SeasonScenario {
    SeasonScenario {
        litter_carbon0: 3.0,
        ..greenhouse_bio_scenario()
    }
}

/// The Tier-2 crew stores, sized well-fed over the multi-year horizon.
pub const SEALED_STATION_CABIN_SCENARIO: CabinScenario = CabinScenario {
    food_store0: 5.0e5,
    water_store0: 2.0e4,
    ..CABIN_GAS_SCENARIO
};

/// The Tier-2 Power sub-scenario: the microgrid re-timed to the cabin's fast rate.
pub const SEALED_STATION_POWER_SCENARIO: PowerScenario = PowerScenario {
    dt_seconds: 60.0,
    steps_per_day: 1440,
    ..BOUNDED_SOC_SCENARIO
};

/// Step-7 sealed-station run data (the fully-coupled multi-year station).
#[derive(Debug, Clone, Copy)]
pub struct SealedStationScenario {
    /// The perennial sealed biosphere (greenhouse gas seam + lamp light), re-sown yearly.
    pub bio: SeasonScenario,
    /// The crew stores (multi-year sized) + intake rates + initial cabin humidity.
    pub cabin: CabinScenario,
    /// The Power microgrid at the fast rate (constant daily-average solar/load).
    pub power: PowerScenario,
    /// The grow-lamp on-window power (W).
    pub lamp_power_w: f64,
    /// Photoperiod (integer hours).
    pub photoperiod_hours: u64,
    /// The provisioned battery (J).
    pub battery0: f64,
    /// The Tier-2 horizon: whole seasons.
    pub years: usize,
    /// The season / re-sow period (days).
    pub season_days: usize,
    /// The cabin/Power sub-steps per biosphere day (1440).
    pub steps_per_day: u64,
    /// The cabin sub-step dt (s).
    pub cabin_dt: f64,
    /// The biosphere structural step (day).
    pub bio_dt: f64,
}

impl SealedStationScenario {
    /// The Tier-2 master-day horizon (`years · season_days`).
    pub fn days(&self) -> usize {
        self.years * self.season_days
    }
}

/// `SEALED_STATION_SCENARIO`: the fully-coupled sealed station over multiple annual cycles.
pub fn sealed_station_scenario() -> SealedStationScenario {
    SealedStationScenario {
        bio: sealed_station_bio_scenario(),
        cabin: SEALED_STATION_CABIN_SCENARIO,
        power: SEALED_STATION_POWER_SCENARIO,
        lamp_power_w: 200.0,
        photoperiod_hours: 16,
        battery0: 2.0e10,
        years: SEALED_STATION_YEARS,
        season_days: SEALED_STATION_SEASON_DAYS,
        steps_per_day: 1440,
        cabin_dt: 60.0,
        bio_dt: 1.0,
    }
}
