//! The authored crop bed — one lamp-lit plant chamber in an authored habitat (slice 4 of
//! `docs/plans/post-roadmap-authored-plants.md`).
//!
//! **The bed is the reference's chamber, not a copy of it.** It is the sealed station's plant
//! chamber, built by the station's own builder ([`station::plant_chamber`]) from the sealed
//! station's own scenario type with an allow-listed set of fields changed ([`BedSpec`]). The
//! science is written once, in Rust; the author composes it.
//!
//! **What an author decides** (the user's answers, 2026-10-18): the re-sow calendar, the plant
//! step, the lamp, what the walls face, and the three stocks the chamber connects to — the
//! energy store its lamp and heater draw from, the heat store its cooler feeds, and (for walls
//! facing space) the radiative sink. **Every** stock of the bed may be wired by the author's
//! flows: the chamber's stocks sit at their fixed ids ([`bed_stock_ids`]), and an author flow
//! names them directly. An author may not *declare* a stock with one of those ids.
//!
//! **What an author cannot change**, by design rather than omission:
//! * the crop (`wheat` only — the chamber's plants are built from the wheat params);
//! * the chamber's air, 9500 mol — `eclss.yaml`'s humidity target hides that size (slice 1's
//!   finding), so an author O₂ regulator wired to the bed's O₂ must declare exactly that air;
//! * the cold program, the chamber's heat params, the soil.
//!
//! **How it runs:** the station's day driver ([`station::driver::advance_one_master_day`]), the
//! plants on their step, everything else — the author's flows and the chamber's — on `dt`. Euler
//! only; no `n_sub`, no `rate_class`.
//!
//! ⚠ **Authored is still not validated.** A bed composes calibrated science into arrangements
//! no one has checked; the platform promises conservation, no rationing and repeatability, and
//! nothing about whether the habitat is sensible.

use std::collections::{BTreeMap, HashMap};

use domains::biosphere::stocks::O2_POOL;
use simcore::environment::{constant, SourceResolver};
use simcore::error::SimError;
use simcore::flow::Flow;
use simcore::registry::Registry;
use simcore::state::Stock;
use station::chamber::ChamberSurroundings;
use station::driver::{OwnedResetHook, SECONDS_PER_DAY};
use station::plant_chamber::{
    assemble_chamber_habitat, chamber_fast_forcings, chamber_plants, chamber_reset_hook,
    chamber_start_stocks, ChamberPorts, ResowRule,
};
use station::scenario::SealedStationScenario;
use station::sowing::{FAST_PROGRAM_VARS, PLANT_PROGRAM_VARS};

use crate::errors::AuthoringError;
use crate::interpreter::{
    build_flow, build_stock, check_rate_preconditions, resolve_demand_controlled, slow_flow_ids,
    BuiltScenario,
};
use crate::schema::{BedResowSpec, BedSpec, BedSurroundingsSpec, ScenarioSpec};
use crate::template::eval_numeric_field;

/// The crops a bed may grow.
pub const BED_CROPS: &[&str] = &["wheat"];

/// What a bed file's run needs beyond [`BuiltScenario`]'s common fields.
pub struct BedRun {
    /// The plant side's forcing (the lamp's light and day on the cold program, the weather the
    /// season reads).
    pub plant_resolver: SourceResolver,
    /// The re-sow hook, on the author's rule.
    pub reset: OwnedResetHook,
    /// Fast (`dt`) steps per day.
    pub steps_per_day: u64,
    /// Plant steps per day.
    pub plant_steps_per_day: u64,
    /// The plant step (days).
    pub plant_dt: f64,
    /// The horizon (whole days).
    pub days: u64,
}

/// The sealed station's scenario with the bed's allow-listed fields applied — the one place an
/// authored value enters the chamber.
pub fn bed_scenario(
    bed: &BedSpec,
    dt: f64,
    steps: u64,
) -> Result<(SealedStationScenario, u64), AuthoringError> {
    if !BED_CROPS.contains(&bed.crop.as_str()) {
        return Err(AuthoringError::new(format!(
            "bed.crop: {:?} is not a crop a bed can grow (known: {BED_CROPS:?}); the chamber's \
             plants are built from the wheat params",
            bed.crop
        )));
    }
    if !(dt.is_finite() && dt > 0.0) {
        return Err(AuthoringError::new(format!(
            "a bed file's dt is its fast step in seconds and must be > 0, got {dt}"
        )));
    }
    let steps_per_day = (SECONDS_PER_DAY / dt).round();
    if steps_per_day < 1.0 || steps_per_day * dt != SECONDS_PER_DAY {
        return Err(AuthoringError::new(format!(
            "a bed file's dt ({dt} s) must divide a day ({SECONDS_PER_DAY} s) into whole steps"
        )));
    }
    let steps_per_day = steps_per_day as u64;
    if steps == 0 || !steps.is_multiple_of(steps_per_day) {
        return Err(AuthoringError::new(format!(
            "a bed file's steps ({steps}) must be whole days of {steps_per_day} fast steps"
        )));
    }
    let days = steps / steps_per_day;
    // Any count the day driver can interleave with the fast steps (it refuses the rest). ⚠ Until
    // 2026-10-18 a power of two only: at 24 the lamp's lit share over a step rounded to
    // 1.0000000000000004 and the station's guard stopped the run; the lamp window is capped at its
    // on value since (`docs/plans/post-roadmap-authored-plants.md` §12).
    if bed.plant_steps_per_day == 0 {
        return Err(AuthoringError::new(
            "bed.plant_steps_per_day must be at least 1. 16 is the step the plant science is \
             measured at"
                .to_string(),
        ));
    }
    let mut scenario = station::scenario::sealed_station_scenario();
    if let Some(w) = bed.lamp_power_w {
        if !(w.is_finite() && w > 0.0) {
            return Err(AuthoringError::new(format!(
                "bed.lamp_power_w must be a finite number > 0, got {w}"
            )));
        }
        scenario.lamp_power_w = w;
    }
    if let Some(h) = bed.photoperiod_hours {
        if !(1..=24).contains(&h) {
            return Err(AuthoringError::new(format!(
                "bed.photoperiod_hours must be 1..=24, got {h}"
            )));
        }
        scenario.photoperiod_hours = h;
    }
    scenario.cabin_dt = dt;
    scenario.steps_per_day = steps_per_day;
    scenario.bio_steps_per_day = bed.plant_steps_per_day;
    scenario.bio_dt = 1.0 / bed.plant_steps_per_day as f64;
    // The weather is tiled over whole seasons: enough of them that the table never runs out
    // (it clamps at its last day, which would silently repeat one day for the rest).
    let season = scenario.season_days as u64;
    scenario.years = days.div_ceil(season) as usize;
    Ok((scenario, days))
}

/// The bed's interior stocks at the start — the ids an author's flows may wire to and an
/// author's stocks may not take: the season's (plants, soil, the chamber's air and water, the
/// loss sink), the chamber's heat store, and the walls' boundary when they face the outdoors or
/// a held cabin. Built by the station's own builders, so the list cannot drift from the bed.
pub fn bed_stock_ids(
    scenario: &SealedStationScenario,
    surroundings: ChamberSurroundings,
) -> Result<BTreeMap<String, Stock>, SimError> {
    let mut stocks = chamber_plants(scenario)?.stocks;
    for stock in chamber_start_stocks(scenario, &station::params::chamber(), surroundings)? {
        stocks.insert(stock.id.clone(), stock);
    }
    Ok(stocks)
}

fn surroundings_of(bed: &BedSpec) -> Result<ChamberSurroundings, AuthoringError> {
    let positive = |v: f64, what: &str| -> Result<f64, AuthoringError> {
        if v.is_finite() && v > 0.0 {
            Ok(v)
        } else {
            Err(AuthoringError::new(format!(
                "bed.surroundings: {what} must be a finite number > 0, got {v}"
            )))
        }
    };
    let surroundings = match bed.surroundings {
        BedSurroundingsSpec::Outdoor => ChamberSurroundings::Outdoor,
        BedSurroundingsSpec::Structure => ChamberSurroundings::Structure,
        BedSurroundingsSpec::Space(e) => ChamberSurroundings::Space {
            effective_emittance: positive(e, "space's effective emittance")?,
        },
        BedSurroundingsSpec::Cabin(t) => ChamberSurroundings::Cabin {
            temperature_k: positive(t, "cabin's temperature (K)")?,
        },
    };
    let wants_space = matches!(surroundings, ChamberSurroundings::Space { .. });
    if wants_space != bed.space.is_some() {
        return Err(AuthoringError::new(if wants_space {
            "bed.space: walls facing space need the author's radiative sink (an energy stock)"
                .to_string()
        } else {
            "bed.space is used only by surroundings {space: …}".to_string()
        }));
    }
    Ok(surroundings)
}

/// Interpret a scenario with a `bed:` — the bed path of [`crate::interpreter::interpret`].
pub(crate) fn interpret_bed(
    spec: &ScenarioSpec,
    params: &BTreeMap<String, f64>,
    allow_unsafe_step: bool,
) -> Result<BuiltScenario, AuthoringError> {
    let bed = spec
        .bed
        .as_ref()
        .expect("interpret_bed is called with a bed");
    if spec.integrator != "euler" {
        return Err(AuthoringError::new(format!(
            "a bed file runs on the station's day driver, which is Euler only; got integrator \
             {:?}",
            spec.integrator
        )));
    }
    let slow_ids = slow_flow_ids(spec)?;
    if spec.n_sub != 1 || !slow_ids.is_empty() {
        return Err(AuthoringError::new(
            "a bed file declares its steps through dt (fast) and bed.plant_steps_per_day; \
             n_sub and rate_class: slow do not apply"
                .to_string(),
        ));
    }
    let (scenario, days) = bed_scenario(bed, spec.dt, spec.steps)?;
    let surroundings = surroundings_of(bed)?;
    if !allow_unsafe_step {
        check_rate_preconditions(spec, &slow_ids)?;
    }

    // The bed's stocks first: the author's flows may wire to any of them.
    let bed_stocks = bed_stock_ids(&scenario, surroundings)?;
    let mut author_stocks: Vec<Stock> = Vec::new();
    let mut all_stocks = bed_stocks.clone();
    for stock_spec in &spec.stocks {
        let stock = build_stock(stock_spec, params)?;
        if bed_stocks.contains_key(&stock.id) {
            return Err(AuthoringError::new(format!(
                "stock {:?} is the crop bed's own: wire your flows to it, do not declare it",
                stock.id
            )));
        }
        if all_stocks.contains_key(&stock.id) {
            return Err(AuthoringError::new(format!(
                "duplicate stock id {:?}",
                stock.id
            )));
        }
        all_stocks.insert(stock.id.clone(), stock.clone());
        author_stocks.push(stock);
    }

    // An O₂ regulator serving the bed's air must aim at THAT air: the leaf and the regulator
    // must agree on the room's size.
    let bed_air = scenario.bio.chamber_air_capacity_mol;
    for flow_spec in &spec.flows {
        let serves_bed = flow_spec.type_.as_deref() == Some("eclss.o2_makeup")
            && flow_spec
                .wiring
                .iter()
                .any(|(field, stock)| field == "cabin_o2" && stock == O2_POOL);
        if !serves_bed {
            continue;
        }
        let declared = flow_spec
            .cabin_air_mol
            .as_ref()
            .map(|f| eval_numeric_field(f, params, "cabin_air_mol"))
            .transpose()?;
        if declared != Some(bed_air) {
            return Err(AuthoringError::new(format!(
                "flow {:?} regulates the crop bed's O₂ ({O2_POOL:?}), so its cabin_air_mol must \
                 be the bed's air, {bed_air} mol; got {declared:?}",
                flow_spec.id
            )));
        }
    }

    let mut author_flows: Vec<Box<dyn Flow>> = Vec::new();
    for flow_spec in &spec.flows {
        author_flows.push(build_flow(flow_spec, &all_stocks, params)?);
    }

    // The fast side's forcing: the chamber's own, then the author's constants — which may not
    // take a name the chamber or its program reads.
    let mut fast_forcings = chamber_fast_forcings(&scenario).map_err(AuthoringError::from)?;
    let reserved: Vec<String> = fast_forcings
        .keys()
        .cloned()
        .chain(FAST_PROGRAM_VARS.iter().map(|v| v.to_string()))
        .chain(PLANT_PROGRAM_VARS.iter().map(|v| v.to_string()))
        .collect();
    for (name, forcing) in &spec.forcings {
        if reserved.contains(name) {
            return Err(AuthoringError::new(format!(
                "forcing {name:?} is a name the crop bed's chamber reads; choose another"
            )));
        }
        let value =
            eval_numeric_field(&forcing.const_, params, &format!("forcing {name:?} const"))?;
        fast_forcings.insert(name.clone(), constant(value)?);
    }
    let fast_resolver = SourceResolver::new(fast_forcings, HashMap::new())?;
    let lamp = station::params::lamp();
    let plant_resolver = station::sealed::sealed_bio_resolver(&lamp, &scenario)?;

    let ports = ChamberPorts {
        power: &bed.power,
        heat_sink: &bed.heat_sink,
        space: bed.space.as_deref().unwrap_or(""),
    };
    let habitat = assemble_chamber_habitat(
        &scenario,
        &lamp,
        &domains::params::thermal(),
        surroundings,
        ports,
        author_stocks,
        author_flows,
        spec.rng_seed,
    )?;
    let rule = match bed.resow {
        BedResowSpec::Maturity => ResowRule::Maturity,
        BedResowSpec::Never => ResowRule::Never,
        BedResowSpec::EveryDays(0) => {
            return Err(AuthoringError::new(
                "bed.resow: every_days must be >= 1".to_string(),
            ))
        }
        BedResowSpec::EveryDays(n) => ResowRule::EveryDays(n),
    };
    let empty = Registry::flows_only(Vec::new(), &habitat.state.stocks)?;
    Ok(BuiltScenario {
        name: spec.name.clone(),
        state: habitat.state,
        registry: empty,
        slow_registry: habitat.plant_registry,
        fast_registry: habitat.fast_registry,
        resolver: fast_resolver,
        integrator: spec.integrator.clone(),
        dt: spec.dt,
        steps: spec.steps,
        n_sub: 1,
        has_authored_kinetics: spec.flows.iter().any(|f| f.kinetics.is_some()),
        demand_controlled: resolve_demand_controlled(spec, params)?,
        bed: Some(BedRun {
            plant_resolver,
            reset: chamber_reset_hook(&scenario, rule),
            steps_per_day: scenario.steps_per_day,
            plant_steps_per_day: scenario.bio_steps_per_day,
            plant_dt: scenario.bio_dt,
            days,
        }),
    })
}
