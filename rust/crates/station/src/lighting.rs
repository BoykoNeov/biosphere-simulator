//! The Power → biosphere grow lamp — the port of `station.lighting` (P6.5 / P7.5).
//!
//! The phase's **one non-shared-stock coupling** (#16): Power and the biosphere share *no*
//! stock; the whole interface is the **lamp-draw schedule**, which drives both the [`Lamp`]
//! flow (the ENERGY it withdraws from `power.battery`) and the biosphere's `par`,
//! `net_radiation` and `daylength_s` **forcings** (this module computes `PAR =
//! photon_efficacy·lamp_power/ground_area`, the crop's net radiation from that PAR
//! ([`lamp_net_radiation`], since 2026-10-05), and `daylength_s = photoperiod·3600`). The lamp
//! draws a constant **daily-average** power (`substep` freezes `n`, so a within-day top-hat is
//! not an `n`-schedule; daily energy is exact). The `waste_heat` leg lands in `boundary.waste_heat`
//! (the inward move to `thermal.node` is deferred to the sealed station). Tier-2 (FvCB).

use std::collections::BTreeMap;

use domains::biosphere::stocks::{
    AIR_TEMP_WITHIN_DAY_VAR, DAYLENGTH_VAR, LIT_FRACTION_VAR, PAR_VAR, RN_VAR, ROOTED_DEPTH,
    SOIL_WATER, TEMP_VAR, THERMAL_TIME, VERNALIZATION_DAYS, VPD_WITHIN_DAY_VAR,
};
use domains::biosphere::system::{
    build_season_with, soil_surface_start, weather_forcings, weather_shared,
};
use domains::biosphere::{light_path, weather};
use domains::power::{battery_stock, BATTERY, WASTE_HEAT};
use simcore::boundary;
use simcore::environment::{constant, Schedule, SourceResolver};
use simcore::error::SimError;
use simcore::events::Event;
use simcore::flow::Flow;
use simcore::integrator::EulerIntegrator;
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::{State, Stock};

use crate::driver::run_master_day;
use crate::flows::{Lamp, LampParams, LAMP, LAMP_POWER_VAR, PAR_PHOTON_ENERGY_J_PER_UMOL};
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

/// The loader's biosphere params for a lamp-lit ROOM: its water reads the room's own temperature
/// (`temp`, the habitat's or the chamber's), never the weather's within-day air — a room is not
/// outdoors (`docs/plans/post-roadmap-open-field-night.md` §7b, §9). Differs from the loader's in
/// that one field since the loader took the within-day air (2026-10-17); `tests/room_air.rs` pins it.
pub fn room_params() -> domains::biosphere::params::BiosphereParams {
    room_form(domains::biosphere::params::biosphere())
}

/// `base` set for a lamp-lit ROOM ([`room_params`]'s one field) — any crop's params, so a room's
/// potato reads the room as its wheat does (2026-10-18, `docs/plans/post-roadmap-authored-plants.md`
/// §17). A crop's raw params would read the weather's within-day air, which a room drops.
pub fn room_form(
    mut base: domains::biosphere::params::BiosphereParams,
) -> domains::biosphere::params::BiosphereParams {
    base.water.air_temperature = domains::biosphere::science::AirTemperatureRead::DailyMean;
    base
}

/// Drops the weather's within-day air from a ROOM's forcing table: a room's water reads the room's
/// own temperature ([`room_params`]), so a reader left on the outdoor curve fails at its first step
/// instead of reading the outdoors (`docs/plans/post-roadmap-open-field-night.md` §7e). Load-bearing
/// since the loader's weather supplies the names (2026-10-17); `tests/room_air.rs` pins it.
pub fn drop_outdoor_air(
    forcings: &mut std::collections::HashMap<String, simcore::environment::Schedule>,
) {
    forcings.remove(AIR_TEMP_WITHIN_DAY_VAR);
    forcings.remove(VPD_WITHIN_DAY_VAR);
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
    let (bio_state, bio_registry) = build_season_with(&scenario.bio, &room_params())?;
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
    // The soil account's five starting values, as `build_season` seeds them — the same
    // omission as ROOTED_DEPTH below, a second time (§17d of the soil-evaporation plan).
    let soil_start = soil_surface_start(
        &scenario.bio,
        &domains::biosphere::params::biosphere(),
        stocks[SOIL_WATER].amount,
        scenario.bio.rooted_depth0,
    );
    let state = State::new(
        0,
        stocks.clone(),
        0,
        BTreeMap::from_iter(soil_start.into_iter().chain([
            (THERMAL_TIME.to_string(), 0.0),
            // Seeded at 0 as `build_season` seeds it; a missing key read as 0 everywhere, so
            // this moved no byte (soil-evaporation plan §19). `tests/starting_aux.rs` gates it.
            (VERNALIZATION_DAYS.to_string(), 0.0),
            // ⚠ ROOTED_DEPTH is LOAD-BEARING and was MISSING from all three station
            // assemblies (both ports) until 2026-08-12. `build_season` has seeded it
            // with the cited sowing depth since the root-depth build; the station
            // builders assembled their own aux and silently started the crop at depth 0.
            // Invisible while the depth gate was inert; fatal once stress divides by
            // `TTSW = depth * EXTR * rho * A`.
            (ROOTED_DEPTH.to_string(), scenario.bio.rooted_depth0),
        ])),
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

/// The lamp's PAR forcing: a within-day top-hat centred on midday.
///
/// Mirrors `station.lighting._lamp_light_path`. Shared by this module and `sealed`, the
/// two seams whose crop is lit by a lamp rather than by the sun. The lamp's ENERGY half follows
/// it window for window since 2026-10-15 ([`switched_lamp_draw`]). ⚠ Until then it drew the daily
/// average, on the reasoning that `substep` freezes `n`, so no within-day shape is expressible on
/// the fast side — true only while a plant step was a day; at 1/16 day `n` names the window.
pub(crate) fn lamp_light_path(on_par: f64, photoperiod_s: f64) -> Schedule {
    Box::new(move |n, dt| {
        let t = n as f64 * dt;
        light_path::top_hat_window_mean(t - t.trunc(), dt, on_par, photoperiod_s).unwrap_or(0.0)
    })
}

/// The lamp-lit crop's net radiation (W m⁻²) while the lamp is on: the lamp's radiant PAR,
/// `on_par · PAR_PHOTON_ENERGY_J_PER_UMOL` (the conversion the lamp's own energy split uses), net
/// of FAO-56's albedo ([`weather::net_shortwave`]).
///
/// ⚠ Before 2026-10-05 a lamp-lit crop's transpiration read the weather file's OUTDOOR net
/// radiation: it lost more water on sunny days outside, and a lamp blackout left its water loss
/// untouched (`docs/plans/post-roadmap-room-temperature.md` §21). Left out, as outdoors: net
/// long-wave, the lamp's waste heat, and the canopy's size.
pub fn lamp_net_radiation(on_par: f64) -> f64 {
    weather::net_shortwave(on_par * PAR_PHOTON_ENERGY_J_PER_UMOL)
}

/// The lamp-lit crop's net-radiation forcing: [`lamp_net_radiation`] on the lamp's own top-hat,
/// so it is 0 in the dark and follows the lamp window for window.
pub(crate) fn lamp_net_radiation_path(on_par: f64, photoperiod_s: f64) -> Schedule {
    lamp_light_path(lamp_net_radiation(on_par), photoperiod_s)
}

/// The share of each plant step the lamp is on: the lamp's own top-hat at unit light, so it is the
/// light's shape exactly — `net_radiation == φ · lamp_net_radiation(on_par)` on every step. Read
/// by the loader's `NightRule::LitShare` since 2026-10-16
/// (`docs/plans/post-roadmap-lit-share-stomata.md` §6).
pub(crate) fn lamp_lit_fraction_path(photoperiod_s: f64) -> Schedule {
    lamp_light_path(1.0, photoperiod_s)
}

/// The lamp's DRAW switched with its light (`docs/plans/post-roadmap-room-lamp-heat.md`; the
/// reference since 2026-10-15, §8): `on_w` on the plant step's top-hat, read on the FAST side,
/// where until then the lamp drew the daily average on every minute.
///
/// ⚠ **The windows are the group's, behind `n`.** The interleaved day runs a group's
/// `slow_per_group` plant steps (moving `n` past them) and then its fast steps, which lie in those
/// plant windows, `n − slow_per_group .. n − 1` ([`crate::driver::day_groups`]; the plant step
/// records its window, [`crate::gas_exchange`]). Keyed on `n` the room would heat a group before
/// the light came on. The draw is the mean of those windows' top-hats — exactly window `n − 1` when
/// a group is one plant step (the sealed station's 90 minutes); the group's mean when it is more
/// (the lighting seam's 3 power hours over 2 plant windows, where `n` cannot tell the hours apart).
/// Each window is read a whole day later (`+ bio_steps_per_day`): the same time of day, and no
/// underflow at small `n`. A day's energy is `on_w · photoperiod_s`, the daily average's, to
/// rounding.
pub fn switched_lamp_draw(
    on_w: f64,
    photoperiod_s: f64,
    bio_dt: f64,
    bio_steps_per_day: u64,
    slow_per_group: u64,
) -> Schedule {
    Box::new(move |n, _fast_dt| {
        let sum: f64 = (1..=slow_per_group)
            .map(|back| {
                let t = (n + bio_steps_per_day - back) as f64 * bio_dt;
                light_path::top_hat_window_mean(t - t.trunc(), bio_dt, on_w, photoperiod_s)
                    .unwrap_or(0.0)
            })
            .sum();
        sum / slow_per_group as f64
    })
}

/// The biosphere forcing resolver: weather-driven, with `PAR`, net radiation and `daylength`
/// from the lamp.
///
/// Rebuilds the weather forcing table ([`weather_forcings`]) and overrides three entries —
/// `PAR_VAR` → [`lamp_par`] (0 when `with_lamp = false`), `RN_VAR` → [`lamp_net_radiation`] of
/// that PAR (the weather's outdoor value before 2026-10-05), and `DAYLENGTH_VAR` →
/// `photoperiod·3600` — then reassembles the resolver with the sealed shared map. (The
/// `Box<dyn Fn>` schedules of a built resolver are not `Clone`, so the map is regenerated
/// rather than copied — the Python `dict(base.forcings)` analogue.)
pub fn lighting_bio_resolver(
    lamp: &LampParams,
    scenario: &LightingScenario,
    with_lamp: bool,
) -> Result<SourceResolver, SimError> {
    let mut forcings = weather_forcings(&scenario.bio, 1)?;
    drop_outdoor_air(&mut forcings);
    let par = if with_lamp {
        lamp_par(lamp, scenario)
    } else {
        0.0
    };
    let photoperiod_s = scenario.photoperiod_hours as f64 * 3600.0;
    forcings.insert(PAR_VAR.to_string(), lamp_light_path(par, photoperiod_s));
    forcings.insert(
        RN_VAR.to_string(),
        lamp_net_radiation_path(par, photoperiod_s),
    );
    forcings.insert(DAYLENGTH_VAR.to_string(), constant(photoperiod_s)?);
    // The lamp's lit share (read by the loader's lit-share night rule); no lamp, no lit share.
    forcings.insert(
        LIT_FRACTION_VAR.to_string(),
        if with_lamp {
            lamp_lit_fraction_path(photoperiod_s)
        } else {
            constant(0.0)?
        },
    );
    // A warm lamp-lit habitat overrides the weather-table temperature with a constant
    // (the lamp supplies light, the environment supplies warmth) — beside the PAR /
    // daylength overrides, in this non-frozen station module (the embedded `weather_facts`
    // cold table is untouched). `None` leaves the weather temperature in place, so the
    // frozen lighting golden is byte-identical.
    if let Some(temp_c) = scenario.habitat_temp_c {
        forcings.insert(TEMP_VAR.to_string(), constant(temp_c)?);
    }
    SourceResolver::new(forcings, weather_shared(&scenario.bio))
}

/// The Power forcing: the lamp's `lamp_power` draw switched with its light
/// ([`switched_lamp_draw`], the group's mean over its plant windows) — since 2026-10-15
/// (`docs/plans/post-roadmap-room-lamp-heat.md` §8); the constant daily average
/// ([`lamp_average_power`]) until then. A day's energy is the same either way.
pub fn lighting_power_resolver(scenario: &LightingScenario) -> Result<SourceResolver, SimError> {
    let groups = crate::driver::day_groups(scenario.steps_per_day, scenario.bio_steps_per_day)?;
    let mut forcings = std::collections::HashMap::new();
    forcings.insert(
        LAMP_POWER_VAR.to_string(),
        switched_lamp_draw(
            scenario.lamp_power_w,
            scenario.photoperiod_hours as f64 * 3600.0,
            scenario.bio_dt,
            scenario.bio_steps_per_day,
            groups.slow,
        ),
    );
    SourceResolver::new(forcings, std::collections::HashMap::new())
}

/// LAB-ONLY: [`lighting_power_resolver`] with the lamp's constant DAILY AVERAGE
/// ([`lamp_average_power`]) — the reference until 2026-10-15, for instruments whose subject
/// needs a forcing that is flat within the day (the retired day order sees one `n` per day, so
/// under the switched draw all its power hours read the day's last, dark, group).
pub fn lighting_power_resolver_averaged(
    scenario: &LightingScenario,
) -> Result<SourceResolver, SimError> {
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
        scenario.bio_steps_per_day,
        scenario.bio_dt,
        scenario.power_dt,
        None,
    )
}
