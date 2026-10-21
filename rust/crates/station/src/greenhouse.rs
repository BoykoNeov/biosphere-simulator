//! The biosphere ↔ cabin greenhouse — the port of `station.greenhouse` (P6.3 / P7.5).
//!
//! The frozen sealed biosphere's gas exchange meets the Step-2 cabin, so plants + soil
//! microbes + crew all breathe **one** cabin-air stock. The seam is **reversed**: the
//! biosphere's `CARBON_POOL` (`{C:1,O:2}`) / `O2_POOL` (`{O:2}`) stay the shared cabin air,
//! and the CABIN's five all-parameterised flows are re-pointed at those ids (`build_season`
//! reused wholesale). Runs under the two-rate [`crate::driver::run_master_day`] (biosphere
//! slow ×`bio_steps_per_day`, each step followed by its share of the cabin's fast
//! ×`steps_per_day`). Tier-2 (the FvCB biosphere in the graph).

use std::collections::BTreeMap;

use domains::biosphere::stocks::{
    CARBON_POOL, O2_POOL, ROOTED_DEPTH, SOIL_WATER, THERMAL_TIME, VERNALIZATION_DAYS,
};
use domains::biosphere::params::BiosphereParams;
use domains::biosphere::system::{
    build_season_with, soil_surface_start, weather_resolver, weather_shared,
};
use domains::crew::URINE;
use domains::crew::{CrewParams, FECAL_WASTE, FOOD_INTAKE_VAR, WATER_INTAKE_VAR};
use domains::eclss::{EclssParams, CO2_REMOVED, HUMIDITY_CONDENSATE, O2_SUPPLY};
use simcore::boundary;
use simcore::environment::{constant, SourceResolver};
use simcore::error::SimError;
use simcore::events::Event;
use simcore::flow::Flow;
use simcore::integrator::EulerIntegrator;
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::{State, Stock};

use crate::cabin::{build_cabin_flows, eclss_condenser};
use crate::driver::run_master_day;
use crate::gas_exchange::{
    gas_exchange_on_fast_step, require_one_plant_step_per_group, GasExchangeStep,
};
use crate::room_vapour::{record_room_air, room_condenser, Thermostat};
use crate::scenario::GreenhouseScenario;
use crate::stocks::{
    cabin_h2o_stock, co2_composition, food_store_stock, gas_boundary, o2_composition,
    water_store_stock,
};

/// The greenhouse's cabin-only stocks (added to the biosphere stocks over the shared dict).
///
/// `fecal_waste_target == FECAL_WASTE` ⇒ include the `FECAL_WASTE` boundary sink (the open
/// loop); a non-default target (Step 6 `LITTER_CARBON`) omits it (no shadow sink).
pub(crate) fn greenhouse_cabin_stocks(
    scenario: &GreenhouseScenario,
    fecal_waste_target: &str,
) -> Result<Vec<Stock>, SimError> {
    let mut seq = vec![
        food_store_stock(scenario.cabin.food_store0)?,
        water_store_stock(scenario.cabin.water_store0)?,
    ];
    if scenario.room_vapour.has_cabin_vapour() {
        seq.push(cabin_h2o_stock(scenario.cabin.cabin_h2o_0)?);
    }
    seq.extend([
        gas_boundary(O2_SUPPLY, Quantity::Oxygen, o2_composition(), true)?,
        gas_boundary(CO2_REMOVED, Quantity::Carbon, co2_composition(), false)?,
        boundary::sink(HUMIDITY_CONDENSATE.to_string(), Quantity::Water, 0.0)?,
        boundary::sink(URINE.to_string(), Quantity::Water, 0.0)?,
    ]);
    if fecal_waste_target == FECAL_WASTE {
        seq.push(boundary::sink(
            FECAL_WASTE.to_string(),
            Quantity::Carbon,
            0.0,
        )?);
    }
    Ok(seq)
}

/// The greenhouse plants' params: the reference crop's, with the shared room's setting under one
/// vapour store ([`crate::room_vapour::RoomVapour::plant_params`]).
pub(crate) fn greenhouse_plant_params(scenario: &GreenhouseScenario) -> BiosphereParams {
    scenario
        .room_vapour
        .plant_params(domains::biosphere::params::biosphere())
}

/// The greenhouse's five cabin flows over the shared room: the crew breathes the plants' gas
/// pools, and its breath goes to the room's vapour as [`GreenhouseScenario::room_vapour`] says.
pub(crate) fn shared_room_cabin_flows(
    crew: &CrewParams,
    eclss: &EclssParams,
    scenario: &GreenhouseScenario,
    fecal_waste_target: &str,
) -> Vec<Box<dyn Flow>> {
    let room = scenario.room_vapour;
    let condenser = if room.has_cabin_vapour() {
        eclss_condenser(eclss, room.crew_vapour(), HUMIDITY_CONDENSATE)
    } else {
        room_condenser(
            &greenhouse_plant_params(scenario),
            scenario.bio.chamber_air_capacity_mol,
            HUMIDITY_CONDENSATE,
        )
    };
    build_cabin_flows(
        crew,
        eclss,
        CARBON_POOL,
        O2_POOL,
        fecal_waste_target,
        room.crew_vapour(),
        condenser,
    )
}

/// Assemble the greenhouse: `(state, bio_reg, cabin_reg)` (biosphere ↔ cabin).
///
/// The biosphere registry is `build_season`'s output verbatim (over the biosphere stocks —
/// mirroring Python, which reuses `full_bio_registry`); the cabin registry is the five
/// cabin flows re-pointed at the biosphere gas pools, over the merged stock dict. The two
/// stock-id sets are asserted disjoint. `with_plants = false` gives the no-plant baseline
/// (empty biosphere registry) — not used by the golden, kept for the "it bit" contrast.
///
/// With plants, the crop's gas exchange is taken on the cabin's minute step (the reference
/// since 2026-10-03, `docs/plans/post-roadmap-room-temperature.md` §17;
/// [`crate::gas_exchange`]).
pub fn build_greenhouse(
    crew: &CrewParams,
    eclss: &EclssParams,
    scenario: &GreenhouseScenario,
    with_plants: bool,
    fecal_waste_target: &str,
) -> Result<(State, Registry, Registry), SimError> {
    build_greenhouse_at(
        crew,
        eclss,
        scenario,
        with_plants,
        fecal_waste_target,
        GasExchangeStep::Minute,
    )
}

/// [`build_greenhouse`] with the crop's gas exchange on the step `gas` names
/// (`GasExchangeStep::PlantStep` is the form until 2026-10-03, kept for the lab's records).
pub fn build_greenhouse_at(
    crew: &CrewParams,
    eclss: &EclssParams,
    scenario: &GreenhouseScenario,
    with_plants: bool,
    fecal_waste_target: &str,
    gas: GasExchangeStep,
) -> Result<(State, Registry, Registry), SimError> {
    require_room_readable(scenario, with_plants && gas == GasExchangeStep::Minute)?;
    build_greenhouse_any_room(crew, eclss, scenario, with_plants, fecal_waste_target, gas)
}

/// Refuse a one-store room whose run would not record the plant window's temperature, which its
/// condenser reads (`readable`: the plants run, with their gas exchange on the minute step).
pub(crate) fn require_room_readable(
    scenario: &GreenhouseScenario,
    readable: bool,
) -> Result<(), SimError> {
    if scenario.room_vapour.thermostat() == Some(Thermostat::LiveChamber) {
        return Err(SimError::Validation(
            "a greenhouse has no chamber heat store for its room's condenser to read live: \
             its thermostat is the plant window's (a live read of the outdoor air curve is \
             possible but not built)"
                .into(),
        ));
    }
    if scenario.room_vapour.has_cabin_vapour() || readable {
        return Ok(());
    }
    Err(SimError::Validation(
        "one vapour store for a shared room needs the plants, with their gas exchange on the \
         minute step: its condenser reads the plant window's temperature"
            .into(),
    ))
}

/// [`build_greenhouse_at`] without [`require_room_readable`]: for the harvest build, which takes
/// this plant-step base, discards its cabin registry and moves the gas exchange itself.
pub(crate) fn build_greenhouse_any_room(
    crew: &CrewParams,
    eclss: &EclssParams,
    scenario: &GreenhouseScenario,
    with_plants: bool,
    fecal_waste_target: &str,
    gas: GasExchangeStep,
) -> Result<(State, Registry, Registry), SimError> {
    let plant_params = greenhouse_plant_params(scenario);
    let (bio_state, full_bio_registry) = build_season_with(&scenario.bio, &plant_params)?;
    let bio_stocks = bio_state.stocks.clone();

    let cabin_seq = greenhouse_cabin_stocks(scenario, fecal_waste_target)?;
    let mut cabin_stocks: BTreeMap<String, Stock> = BTreeMap::new();
    for s in cabin_seq {
        cabin_stocks.insert(s.id.clone(), s);
    }
    for id in cabin_stocks.keys() {
        if bio_stocks.contains_key(id) {
            return Err(SimError::Validation(format!(
                "greenhouse stock-id collision between the biosphere and the cabin: {id:?} \
                 (the two stock sets must be disjoint)"
            )));
        }
    }
    let mut stocks = bio_stocks;
    for (id, s) in cabin_stocks {
        stocks.insert(id, s);
    }
    // The soil account's five starting values, as `build_season` seeds them — the same
    // omission as ROOTED_DEPTH below, a second time (§17d of the soil-evaporation plan).
    let soil_start = soil_surface_start(
        &scenario.bio,
        &plant_params,
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

    let cabin_flows = shared_room_cabin_flows(crew, eclss, scenario, fecal_waste_target);
    let cabin_registry = Registry::flows_only(cabin_flows, &stocks)?;
    if !with_plants {
        return Ok((
            state,
            Registry::flows_only(Vec::new(), &stocks)?,
            cabin_registry,
        ));
    }
    match gas {
        GasExchangeStep::PlantStep => Ok((state, full_bio_registry, cabin_registry)),
        GasExchangeStep::Minute => {
            require_one_plant_step_per_group(scenario.steps_per_day, scenario.bio_steps_per_day)?;
            let (bio_registry, cabin_registry) = gas_exchange_on_fast_step(
                &stocks,
                full_bio_registry,
                cabin_registry,
                &weather_shared(&scenario.bio),
            )?;
            let bio_registry =
                record_room_air(scenario.room_vapour, &plant_params, bio_registry, &stocks)?;
            Ok((state, bio_registry, cabin_registry))
        }
    }
}

/// The biosphere forcing resolver — the frozen `weather_resolver`, reused as-is (the reverse
/// seam keeps `CO2_POOL_VAR → CARBON_POOL`, now the cabin air).
pub fn greenhouse_bio_resolver(scenario: &GreenhouseScenario) -> Result<SourceResolver, SimError> {
    // The weather is tiled over the master-day horizon (`days` = whole days here, ≤ 1 year).
    weather_resolver(&scenario.bio, 1)
}

/// The cabin forcing resolver — the two constant crew intake rates.
pub fn greenhouse_cabin_resolver(
    crew: &CrewParams,
    scenario: &GreenhouseScenario,
) -> Result<SourceResolver, SimError> {
    let mut forcings = std::collections::HashMap::new();
    forcings.insert(
        FOOD_INTAKE_VAR.to_string(),
        constant(scenario.cabin.food_intake_rate(crew))?,
    );
    forcings.insert(
        WATER_INTAKE_VAR.to_string(),
        constant(scenario.cabin.water_intake_rate())?,
    );
    SourceResolver::new(forcings, std::collections::HashMap::new())
}

/// The two-rate driver: one day per master step (biosphere-slow / cabin-fast).
pub fn run_greenhouse(
    bio_integrator: &EulerIntegrator,
    cabin_integrator: &EulerIntegrator,
    state: State,
    bio_resolver: &SourceResolver,
    cabin_resolver: &SourceResolver,
    scenario: &GreenhouseScenario,
) -> Result<(Vec<State>, u64, Vec<Event>), SimError> {
    run_master_day(
        bio_integrator,
        cabin_integrator,
        state,
        bio_resolver,
        cabin_resolver,
        scenario.days,
        scenario.steps_per_day,
        scenario.bio_steps_per_day,
        scenario.bio_dt,
        scenario.cabin_dt,
        None,
    )
}
