//! The season assembly + drivers — the Rust port of `domains.biosphere.{scenario,
//! atmosphere,soil,plants,water,consumers,season}` (Phase-7 P7.4).
//!
//! `build_season` composes the five compartment builds over one shared stock dict + a
//! flow/aux registry (the integrator stays global — one clock, one ledger, one gate).
//! `weather_resolver` builds the tiled forcing tables from the raw facts. `run_season`
//! carries the optional `reset` hook (its conservation checkpoint included) that
//! `run_perennial` drives with `annual_reset` at each year boundary.

use std::collections::{BTreeMap, HashMap};

use simcore::auxiliary::AuxProcess;
use simcore::boundary;
use simcore::conservation::assert_conserved_default;
use simcore::environment::{constant, Schedule, SourceResolver};
use simcore::error::SimError;
use simcore::events::Event;
use simcore::flow::Flow;
use simcore::integrator::EulerIntegrator;
use simcore::quantities::{Quantity, StockKind};
use simcore::registry::Registry;
use simcore::state::{State, Stock};

use super::flows::{
    Allocation, CarbonContext, Condensation, ConsumerMortality, ConsumerRespiration, Decomposition,
    Fertilization, Grazing, GrowthRespiration, Irrigation, MaintenanceRespiration,
    MicrobialRespiration, Mineralization, NitrogenSenescence, NitrogenUptake, Recycling,
    Senescence, ThermalTimeAccumulation, Transpiration, VernalizationAccumulation,
};
use super::params;
use super::stocks::*;

/// Scenario data (not crop params): plot, initial amounts, soil/atmosphere/chamber knobs.
#[derive(Debug, Clone, Copy)]
pub struct SeasonScenario {
    pub ground_area: f64,
    pub leaf_c0: f64,
    pub stem_c0: f64,
    pub root_c0: f64,
    pub storage_c0: f64,
    pub co2_atmos0: f64,
    pub ci: f64,
    pub sealed: bool,
    pub chamber_air_mol: f64,
    pub chamber_co2_mol0: f64,
    pub ci_ratio: f64,
    pub chamber_o2_mol0: f64,
    pub litter_carbon0: f64,
    pub consumer: bool,
    pub consumer_c0: f64,
    pub soil_water0: f64,
    pub water_vapor0: f64,
    pub condensate0: f64,
    pub water_source0: f64,
    pub sw_wilting: f64,
    pub sw_critical: f64,
    pub irrigation_mm_day: f64,
    pub soil_n0: f64,
    pub n_source0: f64,
    pub plant_n0: f64,
    pub sn_residual: f64,
    pub sn_critical: f64,
    pub fertilization_kg_m2_day: f64,
    pub latitude: f64,
}

/// The Phase-1 winter-wheat PP plot defaults (open field, N/water non-limiting).
pub const DEFAULT_SCENARIO: SeasonScenario = SeasonScenario {
    ground_area: 1.0,
    leaf_c0: 0.05,
    stem_c0: 0.03,
    root_c0: 0.08,
    storage_c0: 0.0,
    co2_atmos0: 0.0,
    ci: 250.0,
    sealed: false,
    chamber_air_mol: 1000.0,
    chamber_co2_mol0: 0.357,
    ci_ratio: 0.7,
    chamber_o2_mol0: 210.0,
    litter_carbon0: 0.0,
    consumer: false,
    consumer_c0: 0.01,
    soil_water0: 1000.0,
    water_vapor0: 0.0,
    condensate0: 0.0,
    water_source0: 0.0,
    sw_wilting: 20.0,
    sw_critical: 60.0,
    irrigation_mm_day: 2.0,
    soil_n0: 100.0,
    n_source0: 0.0,
    plant_n0: 0.5,
    sn_residual: 1.0,
    sn_critical: 50.0,
    fertilization_kg_m2_day: 0.0,
    latitude: 52.0,
};

/// The O₂-poor sealed chamber (Phase-2 capstone). Run 3 years via `run_season`.
pub const SEALED_CHAMBER_YEARS: usize = 3;
/// The perennial (re-sown) sealed chamber. Run 5 years via `run_perennial`.
pub const PERENNIAL_CHAMBER_YEARS: usize = 5;
/// The consumer chamber. Run 5 years via `run_perennial`.
pub const CONSUMER_CHAMBER_YEARS: usize = 5;
/// The decade-scale horizon (Phase-4 long-horizon goldens).
pub const LONG_HORIZON_YEARS: usize = 15;

/// The O₂-poor sealed chamber scenario (`SEALED_CHAMBER_SCENARIO`).
pub fn sealed_chamber_scenario() -> SeasonScenario {
    SeasonScenario {
        sealed: true,
        chamber_o2_mol0: 2.0,
        litter_carbon0: 3.0,
        ..DEFAULT_SCENARIO
    }
}

/// The perennial (annual-reset) chamber scenario (`PERENNIAL_CHAMBER_SCENARIO`).
pub fn perennial_chamber_scenario() -> SeasonScenario {
    SeasonScenario {
        sealed: true,
        litter_carbon0: 3.0,
        ..DEFAULT_SCENARIO
    }
}

/// The minimal-consumer chamber scenario (`CONSUMER_CHAMBER_SCENARIO`).
///
/// Chamber ENLARGED 2x (post-roadmap scope (B) increment 1): the vernalization +
/// photoperiod sciences give a ~5x larger plant, and the herbivore raises carbon
/// throughput, so the original 0.357 mol / 1000 mol air over-drew the CO2 pool. All three
/// gas quantities scale by the same factor so Ci0 (250) and x_O2 (0.21) both stay
/// invariant. SEALED and PERENNIAL keep their frozen sizing. Mirrors the Python
/// `CONSUMER_CHAMBER_SCENARIO`; see docs/plans/post-roadmap-oracle-match.md.
pub fn consumer_chamber_scenario() -> SeasonScenario {
    SeasonScenario {
        sealed: true,
        litter_carbon0: 3.0,
        consumer: true,
        chamber_air_mol: 2000.0,
        chamber_co2_mol0: 0.714,
        chamber_o2_mol0: 420.0,
        ..DEFAULT_SCENARIO
    }
}

/// One compartment's contribution (stocks, flows, aux, shared-map entries).
struct CompartmentBuild {
    stocks: Vec<Stock>,
    flows: Vec<Box<dyn Flow>>,
    aux: Vec<Box<dyn AuxProcess>>,
}

impl CompartmentBuild {
    fn empty() -> Self {
        CompartmentBuild {
            stocks: Vec::new(),
            flows: Vec::new(),
            aux: Vec::new(),
        }
    }
}

/// A composition stock (the CO₂/O₂ chamber pools) built directly.
fn composition_pool(
    id: &str,
    domain: &str,
    quantity: Quantity,
    amount: f64,
    composition: BTreeMap<Quantity, f64>,
) -> Result<Stock, SimError> {
    Stock::new(
        id.to_string(),
        domain.to_string(),
        quantity,
        quantity.canonical_unit(),
        amount,
        StockKind::Pool,
        0.0,
        false,
        composition,
    )
}

fn carbon_context(scenario: &SeasonScenario, p: &params::BiosphereParams) -> CarbonContext {
    CarbonContext {
        leaf_c: LEAF_C.to_string(),
        stem_c: STEM_C.to_string(),
        root_c: ROOT_C.to_string(),
        par_var: PAR_VAR.to_string(),
        ci_var: CI_VAR.to_string(),
        temp_var: TEMP_VAR.to_string(),
        daylength_var: DAYLENGTH_VAR.to_string(),
        soil_water_var: SOIL_WATER_VAR.to_string(),
        sw_wilting: scenario.sw_wilting,
        sw_critical: scenario.sw_critical,
        plant_n: PLANT_N.to_string(),
        photo: p.photo,
        canopy: p.canopy,
        resp: p.resp,
        nitro: p.nitro,
        ground_area: scenario.ground_area,
        co2_pool_var: if scenario.sealed {
            Some(CO2_POOL_VAR.to_string())
        } else {
            None
        },
        chamber_air_mol: if scenario.sealed {
            Some(scenario.chamber_air_mol)
        } else {
            None
        },
        ci_ratio: if scenario.sealed {
            Some(scenario.ci_ratio)
        } else {
            None
        },
    }
}

fn build_atmosphere(
    scenario: &SeasonScenario,
    p: &params::BiosphereParams,
) -> Result<CompartmentBuild, SimError> {
    if scenario.sealed {
        let stocks = vec![
            composition_pool(
                CARBON_POOL,
                ATMOSPHERE,
                Quantity::Carbon,
                scenario.chamber_co2_mol0,
                BTreeMap::from([(Quantity::Carbon, 1.0), (Quantity::Oxygen, 2.0)]),
            )?,
            composition_pool(
                O2_POOL,
                ATMOSPHERE,
                Quantity::Oxygen,
                scenario.chamber_o2_mol0,
                BTreeMap::from([(Quantity::Oxygen, 2.0)]),
            )?,
            pool_stock(
                WATER_VAPOR,
                ATMOSPHERE,
                Quantity::Water,
                scenario.water_vapor0,
            )?,
        ];
        let flows: Vec<Box<dyn Flow>> = vec![Box::new(Condensation {
            id: "biosphere.condensation".to_string(),
            water_vapor: WATER_VAPOR.to_string(),
            condensate: CONDENSATE.to_string(),
            condensation_rate: p.water.condensation_rate,
        })];
        Ok(CompartmentBuild {
            stocks,
            flows,
            aux: Vec::new(),
        })
    } else {
        let stocks = vec![
            boundary::source(
                CO2_ATMOS.to_string(),
                Quantity::Carbon,
                scenario.co2_atmos0,
                true,
            )?,
            boundary::sink(CO2_RESP.to_string(), Quantity::Carbon, 0.0)?,
        ];
        Ok(CompartmentBuild {
            stocks,
            flows: Vec::new(),
            aux: Vec::new(),
        })
    }
}

fn build_soil(
    scenario: &SeasonScenario,
    p: &params::BiosphereParams,
) -> Result<CompartmentBuild, SimError> {
    let mut stocks = vec![
        pool_stock(SOIL_WATER, SOIL, Quantity::Water, scenario.soil_water0)?,
        pool_stock(SOIL_N, SOIL, Quantity::Nitrogen, scenario.soil_n0)?,
        boundary::source(
            N_SOURCE.to_string(),
            Quantity::Nitrogen,
            scenario.n_source0,
            true,
        )?,
    ];
    let mut flows: Vec<Box<dyn Flow>> = vec![Box::new(Fertilization {
        id: "biosphere.fertilization".to_string(),
        n_source: N_SOURCE.to_string(),
        soil_n: SOIL_N.to_string(),
        fertilization_var: FERTILIZATION_VAR.to_string(),
        ground_area: scenario.ground_area,
    })];
    if !scenario.sealed {
        stocks.push(boundary::source(
            WATER_SOURCE.to_string(),
            Quantity::Water,
            scenario.water_source0,
            true,
        )?);
        flows.push(Box::new(Irrigation {
            id: "biosphere.irrigation".to_string(),
            water_source: WATER_SOURCE.to_string(),
            soil_water: SOIL_WATER.to_string(),
            irrigation_var: IRRIGATION_VAR.to_string(),
            ground_area: scenario.ground_area,
        }));
    }
    if scenario.sealed {
        stocks.push(pool_stock(
            LITTER_CARBON,
            SOIL,
            Quantity::Carbon,
            scenario.litter_carbon0,
        )?);
        stocks.push(organ_stock(MICROBIAL_CARBON, SOIL, 0.0)?);
        stocks.push(pool_stock(LITTER_N, SOIL, Quantity::Nitrogen, 0.0)?);
        flows.push(Box::new(Decomposition {
            id: "biosphere.decomposition".to_string(),
            litter_carbon: LITTER_CARBON.to_string(),
            microbial_carbon: MICROBIAL_CARBON.to_string(),
            decomposition_rate: p.decomp.decomposition_rate,
        }));
        flows.push(Box::new(MicrobialRespiration {
            id: "biosphere.microbial_respiration".to_string(),
            microbial_carbon: MICROBIAL_CARBON.to_string(),
            co2_pool: CARBON_POOL.to_string(),
            o2_pool: O2_POOL.to_string(),
            microbial_respiration_rate: p.micro.microbial_respiration_rate,
            o2_half_saturation: p.micro.o2_half_saturation,
            air_mol: scenario.chamber_air_mol,
        }));
        flows.push(Box::new(Mineralization {
            id: "biosphere.mineralization".to_string(),
            litter_n: LITTER_N.to_string(),
            soil_n: SOIL_N.to_string(),
            mineralization_rate: p.miner.mineralization_rate,
        }));
    }
    Ok(CompartmentBuild {
        stocks,
        flows,
        aux: Vec::new(),
    })
}

fn build_plants(
    scenario: &SeasonScenario,
    p: &params::BiosphereParams,
) -> Result<CompartmentBuild, SimError> {
    let wiring = chamber_wiring(scenario.sealed);
    let ctx = carbon_context(scenario, p);
    let mut stocks = vec![
        organ_stock(LEAF_C, PLANTS, scenario.leaf_c0)?,
        organ_stock(STEM_C, PLANTS, scenario.stem_c0)?,
        organ_stock(ROOT_C, PLANTS, scenario.root_c0)?,
        organ_stock(STORAGE_C, PLANTS, scenario.storage_c0)?,
        pool_stock(PLANT_N, PLANTS, Quantity::Nitrogen, scenario.plant_n0)?,
    ];
    if !scenario.sealed {
        stocks.push(boundary::sink(
            VAPOR_SINK.to_string(),
            Quantity::Water,
            0.0,
        )?);
        stocks.push(boundary::sink(
            LITTER_SINK.to_string(),
            Quantity::Carbon,
            0.0,
        )?);
    }
    let mut flows: Vec<Box<dyn Flow>> = vec![
        Box::new(Allocation {
            id: "biosphere.allocation".to_string(),
            ctx: ctx.clone(),
            co2_atmos: wiring.carbon_source.clone(),
            storage_c: STORAGE_C.to_string(),
            thermal_time_aux: THERMAL_TIME.to_string(),
            pheno: p.pheno,
            table: p.alloc.table.clone(),
            o2_pool: wiring.o2_pool.clone(),
        }),
        Box::new(GrowthRespiration {
            id: "biosphere.growth_respiration".to_string(),
            ctx: ctx.clone(),
            co2_atmos: wiring.carbon_source.clone(),
            co2_resp: wiring.resp_sink.clone(),
        }),
        Box::new(MaintenanceRespiration {
            id: "biosphere.maintenance_respiration".to_string(),
            ctx: ctx.clone(),
            co2_atmos: wiring.carbon_source.clone(),
            co2_resp: wiring.resp_sink.clone(),
            o2_pool: wiring.o2_pool.clone(),
            air_mol: if scenario.sealed {
                Some(scenario.chamber_air_mol)
            } else {
                None
            },
        }),
        Box::new(Senescence {
            id: "biosphere.senescence".to_string(),
            leaf_c: LEAF_C.to_string(),
            stem_c: STEM_C.to_string(),
            root_c: ROOT_C.to_string(),
            litter_sink: wiring.litter_carbon_target.clone(),
            rdr_leaf: p.senesc.rdr_leaf,
            rdr_stem: p.senesc.rdr_stem,
            rdr_root: p.senesc.rdr_root,
        }),
        Box::new(Transpiration {
            id: "biosphere.transpiration".to_string(),
            soil_water: SOIL_WATER.to_string(),
            vapor_sink: wiring.vapor_target.clone(),
            rn_var: RN_VAR.to_string(),
            vpd_var: VPD_VAR.to_string(),
            temp_var: TEMP_VAR.to_string(),
            aerodynamic_resistance: p.transp.aerodynamic_resistance,
            surface_resistance: p.transp.surface_resistance,
            ground_area: scenario.ground_area,
            sw_wilting: scenario.sw_wilting,
            sw_critical: scenario.sw_critical,
        }),
        Box::new(NitrogenUptake {
            id: "biosphere.nitrogen_uptake".to_string(),
            soil_n: SOIL_N.to_string(),
            plant_n: PLANT_N.to_string(),
            max_uptake_capacity: p.nitro.max_uptake_capacity,
            ground_area: scenario.ground_area,
            sn_residual: scenario.sn_residual,
            sn_critical: scenario.sn_critical,
        }),
    ];
    if scenario.sealed {
        flows.push(Box::new(NitrogenSenescence {
            id: "biosphere.nitrogen_senescence".to_string(),
            plant_n: PLANT_N.to_string(),
            litter_n: LITTER_N.to_string(),
            n_senescence_rate: p.miner.n_senescence_rate,
        }));
    }
    // Two accumulators (scope (B) inc. 1): vernalization days accrue from temperature,
    // and thermal time accrues *gated by them* (and by daylength) through the vegetative
    // phase. Mirrors domains/biosphere/plants.py.
    let aux: Vec<Box<dyn AuxProcess>> = vec![
        Box::new(ThermalTimeAccumulation {
            id: "biosphere.thermal_time".to_string(),
            accumulator: THERMAL_TIME.to_string(),
            temp_var: TEMP_VAR.to_string(),
            t_base: p.pheno.t_base,
            t_cap: p.pheno.t_cap,
            tsum_anthesis: p.pheno.tsum_anthesis,
            tsum_maturity: p.pheno.tsum_maturity,
            vernalization: Some(p.vern),
            vernalization_accumulator: Some(VERNALIZATION_DAYS.to_string()),
            photoperiod: Some(p.photoperiod),
            daylength_var: Some(DAYLENGTH_VAR.to_string()),
        }),
        Box::new(VernalizationAccumulation {
            id: "biosphere.vernalization_days".to_string(),
            accumulator: VERNALIZATION_DAYS.to_string(),
            temp_var: TEMP_VAR.to_string(),
            params: p.vern,
        }),
    ];
    Ok(CompartmentBuild { stocks, flows, aux })
}

fn build_water(
    scenario: &SeasonScenario,
    p: &params::BiosphereParams,
) -> Result<CompartmentBuild, SimError> {
    if !scenario.sealed {
        return Ok(CompartmentBuild::empty());
    }
    let stocks = vec![pool_stock(
        CONDENSATE,
        WATER,
        Quantity::Water,
        scenario.condensate0,
    )?];
    let flows: Vec<Box<dyn Flow>> = vec![Box::new(Recycling {
        id: "biosphere.recycling".to_string(),
        condensate: CONDENSATE.to_string(),
        soil_water: SOIL_WATER.to_string(),
        recycling_rate: p.water.recycling_rate,
    })];
    Ok(CompartmentBuild {
        stocks,
        flows,
        aux: Vec::new(),
    })
}

fn build_consumers(
    scenario: &SeasonScenario,
    p: &params::BiosphereParams,
) -> Result<CompartmentBuild, SimError> {
    if !(scenario.sealed && scenario.consumer) {
        return Ok(CompartmentBuild::empty());
    }
    let stocks = vec![organ_stock(
        CONSUMER_CARBON,
        CONSUMERS,
        scenario.consumer_c0,
    )?];
    let flows: Vec<Box<dyn Flow>> = vec![
        Box::new(Grazing {
            id: "biosphere.grazing".to_string(),
            leaf_c: LEAF_C.to_string(),
            consumer_carbon: CONSUMER_CARBON.to_string(),
            grazing_rate: p.herb.grazing_rate,
        }),
        Box::new(ConsumerRespiration {
            id: "biosphere.consumer_respiration".to_string(),
            consumer_carbon: CONSUMER_CARBON.to_string(),
            co2_pool: CARBON_POOL.to_string(),
            o2_pool: O2_POOL.to_string(),
            respiration_rate: p.herb.respiration_rate,
            o2_half_saturation: p.herb.o2_half_saturation,
            air_mol: scenario.chamber_air_mol,
        }),
        Box::new(ConsumerMortality {
            id: "biosphere.consumer_mortality".to_string(),
            consumer_carbon: CONSUMER_CARBON.to_string(),
            litter_carbon: LITTER_CARBON.to_string(),
            mortality_rate: p.herb.mortality_rate,
        }),
    ];
    Ok(CompartmentBuild {
        stocks,
        flows,
        aux: Vec::new(),
    })
}

fn compartments(
    scenario: &SeasonScenario,
    p: &params::BiosphereParams,
) -> Result<Vec<CompartmentBuild>, SimError> {
    Ok(vec![
        build_atmosphere(scenario, p)?,
        build_soil(scenario, p)?,
        build_plants(scenario, p)?,
        build_water(scenario, p)?,
        build_consumers(scenario, p)?,
    ])
}

/// Assemble the season's initial `State` and the flow + aux `Registry`.
pub fn build_season(scenario: &SeasonScenario) -> Result<(State, Registry), SimError> {
    let p = params::biosphere();
    let builds = compartments(scenario, &p)?;
    let mut stocks: BTreeMap<String, Stock> = BTreeMap::new();
    for build in &builds {
        for stock in &build.stocks {
            stocks.insert(stock.id.clone(), stock.clone());
        }
    }
    // Only POPULATION carbon organs are extinction-eligible ⇒ only the carbon loss-sink.
    for (id, s) in boundary::loss_sinks(&[Quantity::Carbon])? {
        stocks.insert(id, s);
    }
    let mut flows: Vec<Box<dyn Flow>> = Vec::new();
    let mut aux: Vec<Box<dyn AuxProcess>> = Vec::new();
    for build in builds {
        flows.extend(build.flows);
        aux.extend(build.aux);
    }
    let state = State::new(
        0,
        stocks.clone(),
        0,
        BTreeMap::from([
            (THERMAL_TIME.to_string(), 0.0),
            (VERNALIZATION_DAYS.to_string(), 0.0),
        ]),
    )?;
    let registry = Registry::new(flows, &stocks, aux)?;
    Ok((state, registry))
}

/// A forcing schedule reading a precomputed per-day table (clamped at the end) — the
/// `season._table` analogue.
fn table_schedule(values: Vec<f64>) -> Schedule {
    let last = values.len() - 1;
    Box::new(move |n, _dt| values[(n as usize).min(last)])
}

/// The weather forcing table (per-var schedules), tiling the raw facts over `years`.
///
/// Factored out of [`weather_resolver`] so the station lighting / sealed seams can rebuild
/// the resolver with `PAR`/`daylength` overridden by the lamp (a built `SourceResolver`'s
/// `Box<dyn Fn>` schedules are not `Clone`, so they cannot be copied out of an existing
/// resolver — the override must reconstruct the map). The Python analogue is the
/// `dict(base.forcings)` copy; here we regenerate the same table.
pub fn weather_forcings(
    scenario: &SeasonScenario,
    years: usize,
) -> Result<HashMap<String, Schedule>, SimError> {
    let (latitude, rows) = super::weather::weather_facts();
    let f = super::weather::season_forcing(latitude, &rows, years);
    let mut forcings: HashMap<String, Schedule> = HashMap::new();
    forcings.insert(TEMP_VAR.to_string(), table_schedule(f.temp));
    forcings.insert(PAR_VAR.to_string(), table_schedule(f.par));
    forcings.insert(DAYLENGTH_VAR.to_string(), table_schedule(f.daylength));
    forcings.insert(RN_VAR.to_string(), table_schedule(f.net_radiation));
    forcings.insert(VPD_VAR.to_string(), table_schedule(f.vpd));
    forcings.insert(CI_VAR.to_string(), constant(scenario.ci)?);
    forcings.insert(
        IRRIGATION_VAR.to_string(),
        constant(scenario.irrigation_mm_day)?,
    );
    forcings.insert(
        FERTILIZATION_VAR.to_string(),
        constant(scenario.fertilization_kg_m2_day)?,
    );
    Ok(forcings)
}

/// The weather shared-stock map (#16): `soil_water` always; the sealed chamber's
/// `co2_pool → CARBON_POOL`. Factored out alongside [`weather_forcings`] for the same
/// resolver-rebuild reason.
pub fn weather_shared(scenario: &SeasonScenario) -> HashMap<String, String> {
    let mut shared: HashMap<String, String> = HashMap::new();
    shared.insert(SOIL_WATER_VAR.to_string(), SOIL_WATER.to_string());
    if scenario.sealed {
        shared.insert(CO2_POOL_VAR.to_string(), CARBON_POOL.to_string());
    }
    shared
}

/// Build the forcing resolver, tiling the raw weather facts over `years`.
pub fn weather_resolver(
    scenario: &SeasonScenario,
    years: usize,
) -> Result<SourceResolver, SimError> {
    SourceResolver::new(weather_forcings(scenario, years)?, weather_shared(scenario))
}

/// The annual phenology reset / re-sow (P3.4) — a pure, carbon-conserving transform.
pub fn annual_reset(state: &State, scenario: &SeasonScenario) -> Result<State, SimError> {
    let seedling_total = scenario.leaf_c0 + scenario.stem_c0 + scenario.root_c0;
    let mut stocks = state.stocks.clone();
    let grain = stocks[STORAGE_C].amount;
    if grain < seedling_total {
        return Err(SimError::Validation(format!(
            "annual_reset: seed bank too small to re-sow — storage_c {grain:?} < seedling {seedling_total:?}"
        )));
    }
    let old_veg = stocks[LEAF_C].amount + stocks[STEM_C].amount + stocks[ROOT_C].amount;
    stocks.insert(
        LEAF_C.to_string(),
        stocks[LEAF_C].with_amount(scenario.leaf_c0)?,
    );
    stocks.insert(
        STEM_C.to_string(),
        stocks[STEM_C].with_amount(scenario.stem_c0)?,
    );
    stocks.insert(
        ROOT_C.to_string(),
        stocks[ROOT_C].with_amount(scenario.root_c0)?,
    );
    stocks.insert(STORAGE_C.to_string(), stocks[STORAGE_C].with_amount(0.0)?);
    let litter_gain = old_veg + grain - seedling_total; // the balancing residual
    let new_litter = stocks[LITTER_CARBON].amount + litter_gain;
    stocks.insert(
        LITTER_CARBON.to_string(),
        stocks[LITTER_CARBON].with_amount(new_litter)?,
    );
    let mut aux = state.aux.clone();
    aux.insert(THERMAL_TIME.to_string(), 0.0);
    // A re-sown crop must re-vernalize: the cold requirement is per-cycle, so the second
    // accumulator resets alongside the first (both are outside the conservation gate).
    aux.insert(VERNALIZATION_DAYS.to_string(), 0.0);
    State::new(state.n, stocks, state.rng_seed, aux)
}

/// A schedule-agnostic reset hook `(n, state) -> Ok(Some(new_state))` on a reset boundary
/// (checked by the conservation gate then adopted) or `Ok(None)` otherwise.
pub type ResetHook<'a> = &'a dyn Fn(u64, &State) -> Result<Option<State>, SimError>;

/// Step `steps` times, calling `observer` on the initial state and each produced state.
/// `reset` (if given) is consulted before each step; a returned `Some(state)` is checked
/// with the conservation gate then adopted (the `run_season` reset checkpoint).
pub fn run_season(
    integrator: &EulerIntegrator,
    initial: State,
    resolver: &SourceResolver,
    dt: f64,
    steps: usize,
    reset: Option<ResetHook<'_>>,
    observer: &mut dyn FnMut(&State),
) -> Result<(State, u64, Vec<Event>), SimError> {
    let mut state = initial;
    let mut total_rationed = 0u64;
    let mut events: Vec<Event> = Vec::new();
    observer(&state);
    for _ in 0..steps {
        if let Some(reset_fn) = reset {
            if let Some(reset_state) = reset_fn(state.n, &state)? {
                assert_conserved_default(&state, &reset_state)?;
                state = reset_state;
            }
        }
        let report = integrator.step_report(&state, resolver, dt)?;
        state = report.state;
        observer(&state);
        total_rationed += report.rationed;
        events.extend(report.events);
    }
    Ok((state, total_rationed, events))
}

/// `run_season` with `annual_reset` applied every `year` steps (P3.4).
#[allow(clippy::too_many_arguments)]
pub fn run_perennial(
    integrator: &EulerIntegrator,
    initial: State,
    scenario: &SeasonScenario,
    resolver: &SourceResolver,
    dt: f64,
    steps: usize,
    year: usize,
    observer: &mut dyn FnMut(&State),
) -> Result<(State, u64, Vec<Event>), SimError> {
    let year_u = year as u64;
    let reset = move |n: u64, current: &State| -> Result<Option<State>, SimError> {
        // Python: `n > 0 and n % year == 0` (is_multiple_of is true at n=0, hence the guard).
        if n > 0 && n.is_multiple_of(year_u) {
            Ok(Some(annual_reset(current, scenario)?))
        } else {
            Ok(None)
        }
    };
    run_season(
        integrator,
        initial,
        resolver,
        dt,
        steps,
        Some(&reset),
        observer,
    )
}

#[cfg(test)]
mod tests {
    use super::super::{run_perennial_final, run_season_final, SEASON_DAYS};
    use super::*;

    /// The open season runs the whole hard core to completion under the every-step
    /// conservation gate (a completed run is the proof), with the Tier-0 invariants.
    #[test]
    fn open_season_runs_well_fed_and_conserved() {
        let (final_state, rationed, events) =
            run_season_final(&DEFAULT_SCENARIO, 1).expect("open season");
        assert_eq!(final_state.n as usize, SEASON_DAYS);
        assert_eq!(rationed, 0, "open season must be well-fed");
        assert!(events.is_empty(), "open season must be extinction-free");
        // A live plant assimilated carbon: leaf_c stays finite and positive.
        assert!(final_state.stocks[LEAF_C].amount > 0.0);
    }

    /// The sealed chamber closes the gas/water/decomposer loops; O₂ depletes but f_O2
    /// self-limits so the run stays well-fed (rationed == 0) — the Phase-2 capstone.
    #[test]
    fn sealed_chamber_runs_well_fed() {
        let (final_state, rationed, events) =
            run_season_final(&sealed_chamber_scenario(), SEALED_CHAMBER_YEARS).expect("sealed");
        assert_eq!(final_state.n as usize, SEASON_DAYS * SEALED_CHAMBER_YEARS);
        assert_eq!(rationed, 0);
        assert!(events.is_empty());
        // O₂ depleted well below its initial 2.0 mol fill (the depletion mechanism).
        assert!(final_state.stocks[O2_POOL].amount < 2.0);
    }

    /// The perennial chamber re-sows every year via annual_reset (its conservation
    /// checkpoint fires), sustaining a multi-year cycle, and stays genuinely closed.
    #[test]
    fn perennial_chamber_resows_and_stays_closed() {
        let (final_state, rationed, events) =
            run_perennial_final(&perennial_chamber_scenario(), PERENNIAL_CHAMBER_YEARS)
                .expect("perennial");
        assert_eq!(
            final_state.n as usize,
            SEASON_DAYS * PERENNIAL_CHAMBER_YEARS
        );
        assert_eq!(rationed, 0);
        assert!(
            events.is_empty(),
            "death routes to litter, never the loss-sink"
        );
        // Genuinely closed: the carbon loss-sink stays exactly 0.
        assert_eq!(
            final_state.stocks["boundary.loss.carbon"].amount, 0.0,
            "perennial run must be genuinely closed"
        );
    }

    /// A too-small seed bank makes annual_reset refuse to conjure carbon (the closure
    /// caveat): a fresh chamber has grain 0 < the seedling, so a reset at n=0+year errors.
    #[test]
    fn annual_reset_rejects_an_empty_seed_bank() {
        let scenario = perennial_chamber_scenario();
        let (state, _) = build_season(&scenario).unwrap();
        // storage_c starts at 0 < seedling_total, so a reset must error.
        let err = annual_reset(&state, &scenario);
        assert!(matches!(err, Err(SimError::Validation(_))));
    }
}
