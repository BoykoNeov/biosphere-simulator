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
    Drainage, Fertilization, Grazing, GrowthRespiration, HumusDecomposition, HumusNitrogenRelease,
    Irrigation, LitterNitrogenTransfer, MaintenanceRespiration, MicrobialNitrogenRelease,
    MicrobialRespiration, NitrogenSenescence, NitrogenUptake, Recycling, RootDepthExtension,
    RootZoneCapture, Senescence, StemRemobilization, ThermalTimeAccumulation, Transpiration,
    VernalizationAccumulation,
};
use super::light_path;
use super::params;
use super::science;
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
    /// WSSG — the threshold FRACTION of transpirable soil water below which growth
    /// and transpiration decline ([F] Table 15.1, wheat 0.30). ⚠ Replaced the absolute
    /// `sw_wilting`/`sw_critical` kg band on 2026-08-12: see the Python
    /// `SeasonScenario.wssg` and docs/plans/post-roadmap-soil-water-rebasing.md.
    pub wssg: f64,
    /// mm/day **AVAILABLE** — a capacity, not a rate (the flow is demand-driven,
    /// [F] Eqn 14.8). A zero is still a hard off.
    pub irrigation_mm_day: f64,
    pub soil_n0: f64,
    pub n_source0: f64,
    pub plant_n0: f64,
    pub sn_residual: f64,
    pub sn_critical: f64,
    /// The reference soil layer the soil-N pool is DECLARED to be, for the root-zone
    /// access gate. Scenario/soil data, like `sn_residual`/`sn_critical`. DESIGN, not
    /// cited - see the Python `SeasonScenario.soil_layer_depth`.
    pub soil_layer_depth: f64,
    /// `WSTORG` - extractable water present BELOW the current rooted depth (kg). The
    /// default is `soil_depth * soil_extractable_water * 1000 * ground_area`, i.e. the
    /// profile at the drained upper limit. Must be > 0 for roots to grow ([F] Box 14.1
    /// `If WSTORG = 0 Then GRTD = 0`). See the Python `SeasonScenario.subsoil_water0`.
    pub subsoil_water0: f64,
    /// MAI, the moisture availability index ([F] 14.25-14.28), 0..1. Both stores are
    /// `depth * EXTR * rho * A * MAI`; the port carries it as data exactly as Python
    /// does, with the identities pinned rather than computed.
    pub soil_moisture_index: f64,
    /// DRAINF ([F] Eqn 14.11 + Table 14.2). **The valve**: 0.0 shuts drainage off.
    pub drainage_factor: f64,
    /// `EXTR` - volumetric extractable soil water (m^3/m^3). [F] Ch. 13: "approximately
    /// 0.13 mm mm-1 (Ratliff et al., 1983; Ritchie et al., 1999)". Soil data, not a crop
    /// param.
    pub soil_extractable_water: f64,
    /// `SOLDEP` - physical soil depth (m); caps rooted depth alongside the crop's own
    /// `max_rooted_depth` ([F] Box 14.1; [E] Listing 7 L33 takes the shallowest).
    pub soil_depth: f64,
    /// `DEPORT` at emergence (m). [F] Ch. 14: "normally between 150 to 400 mm"; 0.15 is
    /// the cautious bottom of that range. Replaces an uncited 0.0.
    pub rooted_depth0: f64,
    pub fertilization_kg_m2_day: f64,
    pub latitude: f64,
    /// Whether the crop requires vernalization (a cold cue) to leave the vegetative phase
    /// — scope (B) inc. 1's `phenology.py` seam, mirrored (`plants.py::build_plants`).
    /// `true` for the frozen winter wheat (every full literal / spread keeps it → goldens
    /// byte-identical); a DAY-NEUTRAL crop sets it `false` so thermal time advances at the
    /// plain degree-day rate (no `VernalizationAccumulation`, no `verfun` gate).
    pub vernalization: bool,
    /// Whether the crop's development is photoperiod-sensitive (long-day wheat) — the
    /// companion modifier. `true` for the frozen winter wheat; a DAY-NEUTRAL crop sets it
    /// `false` so flowering ignores daylength.
    pub photoperiod: bool,
    /// `WSSD` — the drought development-response COEFFICIENT ([F] Eqn 15.8, Table 15.1
    /// wheat row = 0.40). `None` means "[F] gives no coefficient for this crop", which is
    /// POTATO's case: Table 15.1 has no potato row and populates `WSSD` for only two of
    /// its ten crops. Switched by a VALUE rather than a bool (unlike the two modifiers
    /// above) for exactly that reason. See `SeasonScenario.wssd` and
    /// docs/plans/post-roadmap-water-stress-curves.md.
    pub wssd: Option<f64>,
    /// Whether the crop holds a share of its stem growth apart as shielded starch and
    /// remobilizes it into the grain between anthesis and maturity. `true` for the frozen
    /// winter wheat — the first flag here whose default MOVES the goldens rather than
    /// preserving them, because the mechanism is the reference science and not an option
    /// bolted beside it. A BOOL and not a value (unlike `wssd`): the three numbers are
    /// crop params, and a value-only switch would hand every second species wheat's
    /// tabulated 0.40 by default. POTATO sets it `false` — [E] Table 7 gives potato a
    /// RANGE ("0.2-0.4") where wheat gets a single 0.4, and picking inside someone else's
    /// range is our number wearing their name. See the Python `SeasonScenario`.
    pub stem_reserves: bool,
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
    soil_water0: 19.5,
    water_vapor0: 0.0,
    condensate0: 0.0,
    water_source0: 0.0,
    wssg: 0.30,
    wssd: Some(0.40),
    irrigation_mm_day: 8.0,
    soil_n0: 100.0,
    n_source0: 0.0,
    plant_n0: 0.000243294816,
    sn_residual: 1.0,
    sn_critical: 50.0,
    soil_layer_depth: 0.30,
    subsoil_water0: 175.5,
    soil_moisture_index: 1.0,
    drainage_factor: 0.3,
    soil_extractable_water: 0.13,
    soil_depth: 1.5,
    rooted_depth0: 0.15,
    fertilization_kg_m2_day: 0.0,
    latitude: 52.0,
    vernalization: true,
    photoperiod: true,
    stem_reserves: true,
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
///
/// ⚠ `litter_carbon0` RE-SIZED 3.0 → 3.5 (2026-08-12) because the stem-reserve build
/// abolished the phenomenon this scenario exists to show: the crop fixes more carbon, so
/// it releases more O₂ (PQ = 1) exactly where the trough forms, and the pool bottomed at
/// 5.08 % of its fill against a ≥ 95 %-depletion contract. Mirrors the Python
/// `SEALED_CHAMBER_SCENARIO`, which carries the sweep behind the value; the port has NO
/// reference authority and this is a mirroring of that decision, not a re-taking of it.
pub fn sealed_chamber_scenario() -> SeasonScenario {
    SeasonScenario {
        sealed: true,
        chamber_o2_mol0: 2.0,
        litter_carbon0: 3.5,
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
        soil_water_var: SOIL_WATER_VAR.to_string(),
        wssg: scenario.wssg,
        rooted_depth_aux: ROOTED_DEPTH.to_string(),
        soil_extractable_water: scenario.soil_extractable_water,
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
        // The below-root store: water present in the profile but out of the roots'
        // reach. Unconditional (open field and sealed chamber alike); in the sealed
        // chamber it joins the closed water loop's conserved total, crossing no boundary.
        pool_stock(
            SUBSOIL_WATER,
            SOIL,
            Quantity::Water,
            scenario.subsoil_water0,
        )?,
        pool_stock(SOIL_N, SOIL, Quantity::Nitrogen, scenario.soil_n0)?,
        boundary::source(
            N_SOURCE.to_string(),
            Quantity::Nitrogen,
            scenario.n_source0,
            true,
        )?,
    ];
    let mut flows: Vec<Box<dyn Flow>> = vec![
        Box::new(Fertilization {
            id: "biosphere.fertilization".to_string(),
            n_source: N_SOURCE.to_string(),
            soil_n: SOIL_N.to_string(),
            fertilization_var: FERTILIZATION_VAR.to_string(),
            ground_area: scenario.ground_area,
        }),
        // DRAIN ([F] Eqns 14.11 + 14.12): water above what the current root zone can
        // transpire drains BELOW it, into `subsoil_water` - not out of the system, which
        // is [F]'s own destination. ⚠ Inert on every frozen scenario (demand-driven
        // irrigation never over-fills), so no golden protects it; its pins construct an
        // over-filled zone on purpose.
        Box::new(Drainage {
            id: "biosphere.drainage".to_string(),
            soil_water: SOIL_WATER.to_string(),
            subsoil_water: SUBSOIL_WATER.to_string(),
            drainage_factor: scenario.drainage_factor,
            rooted_depth_aux: ROOTED_DEPTH.to_string(),
            soil_extractable_water: scenario.soil_extractable_water,
            ground_area: scenario.ground_area,
        }),
        // EWAT ([F] Eqn 14.10): the deepening root zone captures the water of the soil
        // it has just explored.
        Box::new(RootZoneCapture {
            id: "biosphere.root_zone_capture".to_string(),
            subsoil_water: SUBSOIL_WATER.to_string(),
            soil_water: SOIL_WATER.to_string(),
            rooted_depth_aux: ROOTED_DEPTH.to_string(),
            thermal_time_aux: THERMAL_TIME.to_string(),
            temp_var: TEMP_VAR.to_string(),
            params: p.rootd,
            photo: p.photo,
            pheno: p.pheno,
            wssg: scenario.wssg,
            soil_depth: scenario.soil_depth,
            soil_extractable_water: scenario.soil_extractable_water,
            ground_area: scenario.ground_area,
        }),
    ];
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
            rooted_depth_aux: ROOTED_DEPTH.to_string(),
            soil_extractable_water: scenario.soil_extractable_water,
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
        stocks.push(pool_stock(MICROBIAL_N, SOIL, Quantity::Nitrogen, 0.0)?);
        // CENTURY slow SOM + its N counterpart (the humification split). Both POOLs,
        // both start empty: humus takes no fresh plant input, it is FORMED.
        stocks.push(pool_stock(HUMUS_CARBON, SOIL, Quantity::Carbon, 0.0)?);
        stocks.push(pool_stock(HUMUS_N, SOIL, Quantity::Nitrogen, 0.0)?);
        flows.push(Box::new(Decomposition {
            id: "biosphere.decomposition".to_string(),
            litter_carbon: LITTER_CARBON.to_string(),
            microbial_carbon: MICROBIAL_CARBON.to_string(),
            co2_pool: CARBON_POOL.to_string(),
            o2_pool: O2_POOL.to_string(),
            decomposition_rate: p.decomp.decomposition_rate,
            litter_respired_fraction: p.humi.litter_respired_fraction,
            o2_half_saturation: p.micro.o2_half_saturation,
            air_mol: scenario.chamber_air_mol,
        }));
        flows.push(Box::new(MicrobialRespiration {
            id: "biosphere.microbial_respiration".to_string(),
            microbial_carbon: MICROBIAL_CARBON.to_string(),
            humus_carbon: HUMUS_CARBON.to_string(),
            co2_pool: CARBON_POOL.to_string(),
            o2_pool: O2_POOL.to_string(),
            microbial_respiration_rate: p.micro.microbial_respiration_rate,
            active_stabilization_co2_fraction: p.humi.active_stabilization_co2_fraction,
            o2_half_saturation: p.micro.o2_half_saturation,
            air_mol: scenario.chamber_air_mol,
        }));
        flows.push(Box::new(HumusDecomposition {
            id: "biosphere.humus_decomposition".to_string(),
            humus_carbon: HUMUS_CARBON.to_string(),
            microbial_carbon: MICROBIAL_CARBON.to_string(),
            co2_pool: CARBON_POOL.to_string(),
            o2_pool: O2_POOL.to_string(),
            slow_decomposition_rate: p.humi.slow_decomposition_rate,
            slow_respired_fraction: p.humi.slow_respired_fraction,
            o2_half_saturation: p.micro.o2_half_saturation,
            air_mol: scenario.chamber_air_mol,
        }));
        // The microbe-mediated N return leg: each carried by the carbon its decomposer
        // sibling already moved, so neither carries a rate of its own.
        flows.push(Box::new(LitterNitrogenTransfer {
            id: "biosphere.litter_n_transfer".to_string(),
            litter_n: LITTER_N.to_string(),
            microbial_n: MICROBIAL_N.to_string(),
            soil_n: SOIL_N.to_string(),
            litter_carbon: LITTER_CARBON.to_string(),
            o2_pool: O2_POOL.to_string(),
            decomposition_rate: p.decomp.decomposition_rate,
            litter_respired_fraction: p.humi.litter_respired_fraction,
            o2_half_saturation: p.micro.o2_half_saturation,
            air_mol: scenario.chamber_air_mol,
        }));
        flows.push(Box::new(MicrobialNitrogenRelease {
            id: "biosphere.microbial_n_release".to_string(),
            microbial_n: MICROBIAL_N.to_string(),
            soil_n: SOIL_N.to_string(),
            humus_n: HUMUS_N.to_string(),
            microbial_carbon: MICROBIAL_CARBON.to_string(),
            o2_pool: O2_POOL.to_string(),
            microbial_respiration_rate: p.micro.microbial_respiration_rate,
            active_stabilization_co2_fraction: p.humi.active_stabilization_co2_fraction,
            o2_half_saturation: p.micro.o2_half_saturation,
            air_mol: scenario.chamber_air_mol,
        }));
        flows.push(Box::new(HumusNitrogenRelease {
            id: "biosphere.humus_n_release".to_string(),
            humus_n: HUMUS_N.to_string(),
            soil_n: SOIL_N.to_string(),
            microbial_n: MICROBIAL_N.to_string(),
            humus_carbon: HUMUS_CARBON.to_string(),
            o2_pool: O2_POOL.to_string(),
            slow_decomposition_rate: p.humi.slow_decomposition_rate,
            slow_respired_fraction: p.humi.slow_respired_fraction,
            o2_half_saturation: p.micro.o2_half_saturation,
            air_mol: scenario.chamber_air_mol,
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
    if scenario.stem_reserves {
        // Starts EMPTY, with no scenario field: the reserve is formed out of stem
        // GROWTH, so a newly sown crop has had none, and a settable initial amount would
        // be a number no source gives.
        stocks.push(pool_stock(STEM_RESERVE_C, PLANTS, Quantity::Carbon, 0.0)?);
    }
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
            // The FORMATION half of stem reserves. Inert (`None` / 0.0 / 0.0) when the
            // crop has no reserve, so its legs are byte-for-byte what they were.
            stem_reserve_c: if scenario.stem_reserves {
                Some(STEM_RESERVE_C.to_string())
            } else {
                None
            },
            fstr: if scenario.stem_reserves {
                p.stem_reserve.remobilizable_fraction
            } else {
                0.0
            },
            // The SAME number the drain below stops on, read from the same params, so the
            // two halves cannot disagree about where the mechanism ends.
            reserve_cessation_dvs: if scenario.stem_reserves {
                p.stem_reserve.cessation_dvs
            } else {
                0.0
            },
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
            rooted_depth_aux: ROOTED_DEPTH.to_string(),
            soil_extractable_water: scenario.soil_extractable_water,
            wssg: scenario.wssg,
        }),
        Box::new(NitrogenUptake {
            id: "biosphere.nitrogen_uptake".to_string(),
            soil_n: SOIL_N.to_string(),
            plant_n: PLANT_N.to_string(),
            // Demand-deficit uptake reads biomass: Greenwood's W excludes fibrous roots
            // (leaf+stem+storage), the deficit applies to f_N's own denominator.
            leaf_c: LEAF_C.to_string(),
            stem_c: STEM_C.to_string(),
            root_c: ROOT_C.to_string(),
            storage_c: STORAGE_C.to_string(),
            max_uptake_capacity: p.nitro.max_uptake_capacity,
            n_target_coefficient: p.nitro.n_target_coefficient,
            n_target_exponent: p.nitro.n_target_exponent,
            n_target_w_plateau: p.nitro.n_target_w_plateau,
            dm_kg_per_mol_c: p.nitro.dm_kg_per_mol_c,
            ground_area: scenario.ground_area,
            sn_residual: scenario.sn_residual,
            sn_critical: scenario.sn_critical,
            rooted_depth_aux: ROOTED_DEPTH.to_string(),
            soil_layer_depth: scenario.soil_layer_depth,
        }),
    ];
    if scenario.stem_reserves {
        // The DRAIN half: the shielded starch moves into the grain between anthesis and
        // maturity (the fill above shares that upper bound, from the same params).
        // Unconditional on `sealed` — this is plant-internal, so it needs no chamber
        // wiring and behaves identically in the open field and in a closed chamber.
        flows.push(Box::new(StemRemobilization {
            id: "biosphere.stem_remobilization".to_string(),
            stem_reserve_c: STEM_RESERVE_C.to_string(),
            storage_c: STORAGE_C.to_string(),
            thermal_time_aux: THERMAL_TIME.to_string(),
            pheno: p.pheno,
            params: p.stem_reserve,
        }));
    }
    if scenario.sealed {
        flows.push(Box::new(NitrogenSenescence {
            id: "biosphere.nitrogen_senescence".to_string(),
            plant_n: PLANT_N.to_string(),
            litter_n: LITTER_N.to_string(),
            // N shedding is DRIVEN by carbon senescence, so this flow carries the same
            // rdr_* rates the Senescence flow above does — one physical event, two legs.
            leaf_c: LEAF_C.to_string(),
            stem_c: STEM_C.to_string(),
            root_c: ROOT_C.to_string(),
            rdr_leaf: p.senesc.rdr_leaf,
            rdr_stem: p.senesc.rdr_stem,
            rdr_root: p.senesc.rdr_root,
            n_residual_per_mol_c: p.nitro.n_residual_per_mol_c,
        }));
    }
    // Two accumulators (scope (B) inc. 1): vernalization days accrue from temperature,
    // and thermal time accrues *gated by them* (and by daylength) through the vegetative
    // phase. Mirrors domains/biosphere/plants.py.
    //
    // Both modifiers are OPTIONAL and gated by the scenario (the `phenology.py` seam). The
    // frozen winter wheat keeps both (defaults `true` → the aux vector below is unchanged →
    // goldens byte-identical). A DAY-NEUTRAL crop turns both off: no
    // `VernalizationAccumulation` is built, and `ThermalTimeAccumulation` carries neither
    // modifier, so thermal time advances at the plain degree-day rate (byte-for-byte, per
    // `plants.py`).
    // The THIRD accumulator (post-roadmap root functional coupling): rooted depth, which
    // gates NitrogenUptake's supply term. Unconditional - every crop roots. Mirrors the
    // Python build order so the aux reduction sees the same canonical id sort.
    let root_depth = RootDepthExtension {
        id: "biosphere.rooted_depth".to_string(),
        accumulator: ROOTED_DEPTH.to_string(),
        thermal_time_aux: THERMAL_TIME.to_string(),
        temp_var: TEMP_VAR.to_string(),
        soil_water: SOIL_WATER.to_string(),
        subsoil_water: SUBSOIL_WATER.to_string(),
        params: p.rootd,
        photo: p.photo,
        pheno: p.pheno,
        wssg: scenario.wssg,
        soil_extractable_water: scenario.soil_extractable_water,
        ground_area: scenario.ground_area,
        soil_depth: scenario.soil_depth,
    };
    let mut aux: Vec<Box<dyn AuxProcess>> = vec![Box::new(ThermalTimeAccumulation {
        id: "biosphere.thermal_time".to_string(),
        accumulator: THERMAL_TIME.to_string(),
        temp_var: TEMP_VAR.to_string(),
        t_base: p.pheno.t_base,
        t_cap: p.pheno.t_cap,
        tsum_anthesis: p.pheno.tsum_anthesis,
        tsum_maturity: p.pheno.tsum_maturity,
        vernalization: scenario.vernalization.then_some(p.vern),
        vernalization_accumulator: scenario
            .vernalization
            .then(|| VERNALIZATION_DAYS.to_string()),
        photoperiod: scenario.photoperiod.then_some(p.photoperiod),
        daylength_var: scenario.photoperiod.then(|| DAYLENGTH_VAR.to_string()),
        // The THIRD modifier. Same optionality, driven by a value rather than a bool.
        drought: scenario.wssd.map(|wssd| params::DroughtDevelopmentParams {
            wssd,
            wssg: scenario.wssg,
            soil_extractable_water: scenario.soil_extractable_water,
            ground_area: scenario.ground_area,
        }),
        drought_soil_water: scenario.wssd.map(|_| SOIL_WATER.to_string()),
        drought_rooted_depth_aux: scenario.wssd.map(|_| ROOTED_DEPTH.to_string()),
    })];
    aux.push(Box::new(root_depth));
    if scenario.vernalization {
        aux.push(Box::new(VernalizationAccumulation {
            id: "biosphere.vernalization_days".to_string(),
            accumulator: VERNALIZATION_DAYS.to_string(),
            temp_var: TEMP_VAR.to_string(),
            params: p.vern,
        }));
    }
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
            // The CITED sowing depth, not 0 ([F] Ch. 14 makes DEPORT-at-emergence an
            // input); mirrors the Python `season.build_season`.
            (ROOTED_DEPTH.to_string(), scenario.rooted_depth0),
        ]),
    )?;
    let registry = Registry::new(flows, &stocks, aux)?;
    Ok((state, registry))
}

/// A forcing schedule reading a precomputed per-day table (clamped at the end) — the
/// `season._table` analogue.
///
/// Indexed by **elapsed physical days**, `int(n * dt)`, not by the step count. Every
/// other forcing in this tree is a pure function of the integer `n`, which is fine for an
/// analytic schedule — a half-sine sampled twice as often is the same half-sine — and
/// wrong for a *table*: indexed by `n` at `dt = 1/4` it would hand the crop the whole
/// season in the first quarter-year and then clamp on the final day for the rest.
///
/// At `dt = 1.0` this is `values[min(n, last)]` exactly, so no golden moves across the
/// change. The port mirrors the rule, not the rationale — the Python reference in
/// `domains/biosphere/season.py::_table` is the authority.
/// The within-day PAR forcing: the sinusoidal day, averaged over the step window.
///
/// Mirrors `season._sine_light_path`. Where `table_schedule` holds a daily observation
/// fixed across the sub-steps within the day, this **shapes** it — and its night steps
/// return exactly 0, which is what lets maintenance respiration's biomass-burning branch
/// run at all.
fn sine_light_path(daytime_mean_par: Vec<f64>, daylength_s: Vec<f64>) -> Schedule {
    let last = daytime_mean_par.len() - 1;
    Box::new(move |n, dt| {
        let t = n as f64 * dt;
        let day = (t as usize).min(last);
        // The window is guaranteed inside one day by the step dividing the day; a build
        // that broke that would surface as an error in the Python reference, and here the
        // schedule signature has nowhere to put one, so it is clamped to 0 the same way a
        // dark window is. (The Python side raises; the divergence is unreachable while
        // `BIO_DT` divides the day, which its own module asserts.)
        light_path::half_sine_window_mean(
            t - t.trunc(),
            dt,
            daytime_mean_par[day],
            daylength_s[day],
        )
        .unwrap_or(0.0)
    })
}

fn table_schedule(values: Vec<f64>) -> Schedule {
    let last = values.len() - 1;
    Box::new(move |n, dt| values[((n as f64 * dt) as usize).min(last)])
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
    forcings.insert(
        PAR_VAR.to_string(),
        sine_light_path(f.par.clone(), f.daylength.clone()),
    );
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
    // The stem's shielded starch dies with the stem that held it. It is NOT part of the
    // seedling: the reserve is formed out of stem growth, so a newly sown crop has had
    // none. Absent for a crop without the mechanism — the stock is not built at all.
    let held_reserve = stocks
        .get(STEM_RESERVE_C)
        .map(|st| st.amount)
        .unwrap_or(0.0);
    if let Some(st) = stocks.get(STEM_RESERVE_C) {
        let zeroed = st.with_amount(0.0)?;
        stocks.insert(STEM_RESERVE_C.to_string(), zeroed);
    }
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
    // The balancing residual — carbon in, carbon out, computed rather than formulated
    // (the senescence/maintenance idiom), so the reserve's inclusion cannot leak.
    let litter_gain = old_veg + grain + held_reserve - seedling_total;
    let new_litter = stocks[LITTER_CARBON].amount + litter_gain;
    stocks.insert(
        LITTER_CARBON.to_string(),
        stocks[LITTER_CARBON].with_amount(new_litter)?,
    );
    // The NITROGEN half (post-roadmap: the N-cycle form gap). This used to be carbon-only,
    // leaving `plant_n` as an N windfall for the seedling; with coupled shedding that is
    // incoherent, so the seed keeps the parent's tissue concentration and the remainder dies
    // to litter — the balancing-residual idiom, so NITROGEN is conserved exactly.
    let old_plant_n = stocks[PLANT_N].amount;
    let conc_old = if old_veg > 0.0 {
        old_plant_n / old_veg
    } else {
        0.0
    };
    let seedling_n = conc_old * seedling_total;
    stocks.insert(
        PLANT_N.to_string(),
        stocks[PLANT_N].with_amount(seedling_n)?,
    );
    let new_litter_n = stocks[LITTER_N].amount + (old_plant_n - seedling_n);
    stocks.insert(
        LITTER_N.to_string(),
        stocks[LITTER_N].with_amount(new_litter_n)?,
    );
    let mut aux = state.aux.clone();
    aux.insert(THERMAL_TIME.to_string(), 0.0);
    // A re-sown crop must re-vernalize: the cold requirement is per-cycle, so the second
    // accumulator resets alongside the first (both are outside the conservation gate).
    aux.insert(VERNALIZATION_DAYS.to_string(), 0.0);
    // A re-sown crop starts with the SOWING root system: rooted depth is a property of
    // the standing crop, not of the soil, so it resets with the other per-cycle
    // accumulators - to `rooted_depth0`, which [F] Ch. 14 makes an input.
    let old_depth = aux.get(ROOTED_DEPTH).copied().unwrap_or(0.0);
    aux.insert(ROOTED_DEPTH.to_string(), scenario.rooted_depth0);
    // THE WATER HALF OF THE RE-SOW. The root zone just shrank, so the abandoned share of
    // its water is once again BELOW the root zone and returns to `subsoil_water`.
    // ⚠ THIS RULE IS OURS - [F] is single-season and silent. Without it every re-sow
    // would ratchet more of the profile permanently into the root zone. It calls
    // `science::resow_water_return` rather than restating the arithmetic: a Python test
    // helper hand-copied this block once and kept the old rule when it changed, which is
    // the same hazard a port has by construction.
    let returned =
        science::resow_water_return(stocks[SOIL_WATER].amount, old_depth, scenario.rooted_depth0);
    if returned > 0.0 {
        let held = stocks[SOIL_WATER].amount;
        stocks.insert(
            SOIL_WATER.to_string(),
            stocks[SOIL_WATER].with_amount(held - returned)?,
        );
        let below = stocks[SUBSOIL_WATER].amount + returned;
        stocks.insert(
            SUBSOIL_WATER.to_string(),
            stocks[SUBSOIL_WATER].with_amount(below)?,
        );
    }
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
    use super::super::{run_perennial_final, run_season_final, steps_for_years};
    use super::*;

    /// `Drainage` reads no forcing at all (state only), so its pins need no resolver.
    struct NoEnv;
    impl simcore::environment::Environment for NoEnv {
        fn get(&self, var: &str) -> Result<f64, SimError> {
            panic!("Drainage must read no forcing, but asked for {var:?}")
        }
    }

    /// A minimal two-stock water State at a given rooted depth, for the flow-level pins.
    fn water_state(soil: f64, subsoil: f64, depth: f64) -> State {
        let mut stocks = std::collections::BTreeMap::new();
        stocks.insert(
            SOIL_WATER.to_string(),
            pool_stock(SOIL_WATER, SOIL, Quantity::Water, soil).unwrap(),
        );
        stocks.insert(
            SUBSOIL_WATER.to_string(),
            pool_stock(SUBSOIL_WATER, SOIL, Quantity::Water, subsoil).unwrap(),
        );
        State::new(
            0,
            stocks,
            0,
            std::collections::BTreeMap::from([(ROOTED_DEPTH.to_string(), depth)]),
        )
        .unwrap()
    }

    fn leg_amount(result: &simcore::flow::FlowResult, stock: &str) -> f64 {
        result
            .legs
            .iter()
            .find(|l| l.stock == stock)
            .map(|l| l.amount)
            .unwrap_or(0.0)
    }

    /// DRAINAGE, WHICH NO GOLDEN AND NO SCENARIO CAN SEE.
    ///
    /// ⚠ `Drainage` is **bit-identically inert on the entire frozen roster** — with
    /// irrigation demand-driven ([F] Eqn 14.8) the root zone is never over-filled, so
    /// `DRAINF` 0.3 and 0.0 produce identical states everywhere. That is physically
    /// correct and it means deleting the flow outright would leave every golden, the
    /// cross-port comparison and the rest of `cargo test` green. The last build measured
    /// FIVE separate mutations of the capture that the Rust suite could not see; the
    /// lesson taken then was that a port's pins have to CONSTRUCT the conditions its
    /// scenarios never reach, so these do.
    #[test]
    fn drainage_relieves_an_overfilled_zone_and_only_an_overfilled_one() {
        let extr = 0.13;
        let area = 2.0; // non-unit on purpose: a dropped area factor is invisible at 1.0
        let depth = 0.5;
        let capacity = science::transpirable_capacity(depth, extr, area);
        let flow = Drainage {
            id: "biosphere.drainage".to_string(),
            soil_water: SOIL_WATER.to_string(),
            subsoil_water: SUBSOIL_WATER.to_string(),
            drainage_factor: 0.3,
            rooted_depth_aux: ROOTED_DEPTH.to_string(),
            soil_extractable_water: extr,
            ground_area: area,
        };
        let env = NoEnv;

        // (a) At or below capacity: nothing moves. This is the branch every frozen
        // scenario takes, which is exactly why it proves nothing on its own.
        for held in [0.0, capacity * 0.5, capacity] {
            let st = water_state(held, 0.0, depth);
            let legs = flow.evaluate(&st, &env, 1.0).unwrap();
            assert_eq!(leg_amount(&legs, SOIL_WATER), 0.0, "drained below capacity");
            assert_eq!(leg_amount(&legs, SUBSOIL_WATER), 0.0);
        }

        // (b) Over capacity: a DRAINF share of the EXCESS, not of the whole store —
        // the mutation a "drain 30 % of the water" misreading would produce.
        let held = capacity + 50.0;
        let st = water_state(held, 7.0, depth);
        let legs = flow.evaluate(&st, &env, 1.0).unwrap();
        let moved = leg_amount(&legs, SUBSOIL_WATER);
        assert!((moved - 50.0 * 0.3).abs() < 1e-12, "moved {moved}, want 15");
        assert!(
            (leg_amount(&legs, SOIL_WATER) + moved).abs() < 1e-12,
            "unbalanced"
        );
        // ...and it is genuinely different from draining a share of the whole store.
        assert!(
            moved < 0.3 * held * 0.5,
            "the excess subtraction was dropped"
        );

        // (c) The AREA factor is load-bearing: the same water in the same depth over a
        // 1 m2 plot is NOT over capacity at all, so the flow must do nothing there.
        let unit = Drainage {
            ground_area: 1.0,
            ..flow
        };
        let legs_unit = unit.evaluate(&st, &env, 1.0).unwrap();
        assert!(
            leg_amount(&legs_unit, SUBSOIL_WATER) > moved,
            "the area factor is being ignored"
        );

        // (d) The VALVE: DRAINF = 0 is a hard off on the same over-filled state.
        let shut = Drainage {
            drainage_factor: 0.0,
            ground_area: area,
            ..unit
        };
        assert_eq!(
            leg_amount(&shut.evaluate(&st, &env, 1.0).unwrap(), SUBSOIL_WATER),
            0.0,
            "DRAINF = 0 must shut the valve exactly"
        );
    }

    /// The donor clamp, on the only input that can reach it: `DRAINF > 1`.
    ///
    /// Unreachable from any scenario (all declare 0..1), and the arbitration backstop
    /// must not be what catches it — every golden asserts that backstop fires zero times.
    #[test]
    fn drainage_never_overdraws_its_donor() {
        let extr = 0.13;
        let flow = Drainage {
            id: "biosphere.drainage".to_string(),
            soil_water: SOIL_WATER.to_string(),
            subsoil_water: SUBSOIL_WATER.to_string(),
            drainage_factor: 5.0,
            rooted_depth_aux: ROOTED_DEPTH.to_string(),
            soil_extractable_water: extr,
            ground_area: 1.0,
        };
        let env = NoEnv;
        // A near-zero root zone makes almost the whole store "excess", and 5x that
        // exceeds the store — the clamp is the only thing standing in the way.
        let st = water_state(40.0, 0.0, 1e-6);
        let legs = flow.evaluate(&st, &env, 1.0).unwrap();
        let moved = leg_amount(&legs, SUBSOIL_WATER);
        assert!(moved <= 40.0, "overdrew the donor: {moved}");
        assert!(
            (moved - 40.0).abs() < 1e-9,
            "the clamp should bind here: {moved}"
        );
    }

    /// THE CAPTURE'S GEOMETRY AT ITS CALL SITE, on a NON-UNIT ground area.
    ///
    /// Every frozen scenario has `ground_area = 1.0`, so a caller that drops the area
    /// factor entirely computes the identical number and is invisible to every golden,
    /// to the rest of `cargo test`, and to the cross-port comparison. Measured: that
    /// mutant left the whole Rust suite green before this test existed. Mirrors the
    /// Python anti-drift pin
    /// `test_soil_layers.py::test_capture_and_depth_use_the_same_gated_rate`.
    #[test]
    fn capture_scales_with_ground_area_at_its_call_sites() {
        let scenario = SeasonScenario {
            ground_area: 2.0,
            // Everything extensive scales with the plot, or this is a different soil.
            soil_water0: 39.0,
            subsoil_water0: 351.0,
            soil_n0: 200.0,
            plant_n0: 2.0 * DEFAULT_SCENARIO.plant_n0,
            leaf_c0: 0.10,
            stem_c0: 0.06,
            root_c0: 0.16,
            ..DEFAULT_SCENARIO
        };
        let (state, integrator, resolver) = super::super::season_setup(&scenario, 1).unwrap();
        let mut seen: Vec<(f64, f64)> = Vec::new();
        let mut observe = |s: &State| {
            seen.push((s.aux[ROOTED_DEPTH], s.stocks[SUBSOIL_WATER].amount));
        };
        let steps = super::super::steps_for_years(1);
        run_season(
            &integrator,
            state,
            &resolver,
            super::super::BIO_DT,
            steps,
            None,
            &mut observe,
        )
        .expect("deep-rooting season");
        let mut checked = 0usize;
        for pair in seen.windows(2) {
            let (d0, w0) = pair[0];
            let (d1, w1) = pair[1];
            let gained = d1 - d0;
            if gained <= 0.0 {
                continue;
            }
            let want = science::captured_water(
                gained,
                scenario.soil_extractable_water,
                scenario.ground_area,
            );
            let got = w0 - w1;
            assert!(
                (got - want).abs() <= 1e-9 * want.abs(),
                "capture {got} != geometry {want}"
            );
            checked += 1;
        }
        assert!(
            checked > 30,
            "the capture must actually run ({checked} steps)"
        );
    }

    /// The donor clamp ([F] Eqn 14.10's `min`) and the re-sow return, on a plot whose
    /// below-root store RUNS OUT.
    ///
    /// Neither is reachable from a frozen scenario: the default store (195 kg) never
    /// empties, and the frozen plots are all 1 m2 so the re-sow return's area factor is
    /// invisible. Both mutants (dropping the clamp; using a unit area in the return)
    /// left the Rust suite green before this test existed. The Python side reaches the
    /// clamp through its own near-empty-store pin.
    #[test]
    fn the_clamp_and_the_resow_return_hold_on_an_emptying_store() {
        let scenario = SeasonScenario {
            ground_area: 2.0,
            soil_water0: 2000.0,
            // Far less than the ~299 kg a 2 m2 plot's roots would capture, so the store
            // empties mid-season and the clamp is the only thing standing between the
            // flow and an overdraw.
            subsoil_water0: 40.0,
            // ⚠ THE VALVE IS SHUT HERE, DELIBERATELY (2026-08-12). This test's subject
            // is the CAPTURE donor clamp and the re-sow return; its 2000 kg root zone is
            // an over-fill on purpose, and with drainage live that over-fill would pour
            // straight into `subsoil_water` and the store would never empty (measured:
            // it holds at its initial 40). `drainage_factor = 0.0` is [F]'s own way to
            // say "no drainage" — a parameter, not a test-only branch — so the isolation
            // costs no special-casing.
            drainage_factor: 0.0,
            soil_n0: 200.0,
            plant_n0: 2.0 * DEFAULT_SCENARIO.plant_n0,
            leaf_c0: 0.10,
            stem_c0: 0.06,
            root_c0: 0.16,
            litter_carbon0: 6.0,
            sealed: true,
            ..DEFAULT_SCENARIO
        };
        let (state, integrator, resolver) = super::super::season_setup(&scenario, 2).unwrap();
        let mut lowest = f64::INFINITY;
        let mut returned_at_reset: Vec<f64> = Vec::new();
        let mut fraction_at_reset: Vec<(f64, f64, f64, f64)> = Vec::new();
        let mut prev: Option<(f64, f64, f64)> = None;
        let mut observe = |s: &State| {
            let below = s.stocks[SUBSOIL_WATER].amount;
            let held = s.stocks[SOIL_WATER].amount;
            let depth = s.aux[ROOTED_DEPTH];
            if below < lowest {
                lowest = below;
            }
            if let Some((prev_depth, prev_below, prev_held)) = prev {
                // A reset is the only place depth falls. Record BOTH sides of it: the
                // rule is a redistribution, so what is checked is that nothing is lost
                // and that FTSW comes through unchanged.
                if depth < prev_depth {
                    returned_at_reset.push(below - prev_below);
                    fraction_at_reset.push((prev_held, prev_depth, held, depth));
                }
            }
            prev = Some((depth, below, held));
        };
        let steps = super::super::steps_for_years(2);
        let (_, rationed, _) = run_perennial(
            &integrator,
            state,
            &scenario,
            &resolver,
            super::super::BIO_DT,
            steps,
            super::super::season_steps(),
            &mut observe,
        )
        .expect("emptying-store chamber");
        // THE CLAMP: the store is drained to exactly empty, never past it, and the
        // arbitration backstop never has to rescue the overdraw.
        assert_eq!(rationed, 0, "the clamp must keep the backstop out of it");
        assert!(lowest >= 0.0, "the store went negative: {lowest}");
        assert!(lowest < 1e-12, "the store must actually empty: {lowest}");
        // THE RETURN, with its area factor: one re-sow, giving back the abandoned
        // column. ⚠ Note WHICH depth that is — the store empties mid-season, so the
        // `WSTORG = 0` gate stalls the roots at ~0.30 m rather than the 1.3 m cap, and
        // the return is correspondingly ~40 kg rather than ~299. The cycle closes on
        // what was actually captured, which is the property that matters.
        // The driver's trace cannot isolate the reset: `run_perennial` emits the state
        // AFTER the step that follows a reset, so a day of flows is already folded in
        // (measured: the two sides differ by ~3 kg, which is one day's transpiration and
        // capture, not a leak). So the reset's water rule is pinned by calling
        // `annual_reset` DIRECTLY below, and the driver run keeps the properties it can
        // actually see: the clamp, the emptying store, and `rationed == 0`.
        assert_eq!(returned_at_reset.len(), 1);
        assert!(returned_at_reset[0] > 0.0, "the re-sow returned nothing");
        assert!(!fraction_at_reset.is_empty());
    }

    /// `WSFD` at the ACCUMULATOR, on a hand-built stressed state with an EXACT expected
    /// value — the two things the whole-season pin below provably cannot see.
    ///
    /// ⚠ Found by mutation, and worth the second test: the season-level pin asserts only
    /// DIRECTION (accelerated > off) and a bound, so it stays green when `WSFD` is given
    /// the wrong threshold (0.4 instead of `wssg` = 0.30 — still accelerates, just by the
    /// wrong amount) and when the multiply is moved inside the `DVS < 1` branch (most of
    /// a stressed season is vegetative, so the season still accelerates). A direction
    /// assertion is not a value assertion, and neither is a bound.
    ///
    /// Mirrors `tests/test_phenology.py::test_thermal_time_aux_drought_is_NOT_gated_off
    /// _at_anthesis` and `..._reads_the_same_ftsw_as_the_other_consumers`.
    #[test]
    fn wsfd_uses_wssg_and_is_not_gated_off_at_anthesis() {
        let extr = 0.13;
        let area = 3.0; // non-unit: a dropped/hardcoded area factor is invisible at 1.0
        let depth = 0.15;
        let wssg = 0.30;
        let ttsw = science::transpirable_capacity(depth, extr, area);
        let pheno = params::PhenologyParams {
            t_base: 0.0,
            t_cap: 30.0,
            tsum_anthesis: 1100.0,
            tsum_maturity: 750.0,
        };
        let proc = ThermalTimeAccumulation {
            id: "test.thermal_time".to_string(),
            accumulator: THERMAL_TIME.to_string(),
            temp_var: TEMP_VAR.to_string(),
            t_base: pheno.t_base,
            t_cap: pheno.t_cap,
            tsum_anthesis: pheno.tsum_anthesis,
            tsum_maturity: pheno.tsum_maturity,
            vernalization: None,
            vernalization_accumulator: None,
            photoperiod: None,
            daylength_var: None,
            drought: Some(params::DroughtDevelopmentParams {
                wssd: 0.40,
                wssg,
                soil_extractable_water: extr,
                ground_area: area,
            }),
            drought_soil_water: Some(SOIL_WATER.to_string()),
            drought_rooted_depth_aux: Some(ROOTED_DEPTH.to_string()),
        };
        struct WarmEnv;
        impl simcore::environment::Environment for WarmEnv {
            fn get(&self, _var: &str) -> Result<f64, SimError> {
                Ok(18.0)
            }
        }
        // FTSW = 0.15 = half of wssg, so WSFG = 0.5 and WSFD = (1 - 0.5)*0.4 + 1 = 1.2.
        // With a WRONG threshold of 0.4 this would read WSFG = 0.375 and WSFD = 1.25.
        let at = |thermal_time: f64| -> f64 {
            let mut stocks = std::collections::BTreeMap::new();
            stocks.insert(
                SOIL_WATER.to_string(),
                pool_stock(SOIL_WATER, SOIL, Quantity::Water, 0.15 * ttsw).unwrap(),
            );
            let state = State::new(
                0,
                stocks,
                0,
                std::collections::BTreeMap::from([
                    (ROOTED_DEPTH.to_string(), depth),
                    (THERMAL_TIME.to_string(), thermal_time),
                ]),
            )
            .unwrap();
            proc.evaluate(&state, &WarmEnv, 1.0).unwrap()[THERMAL_TIME]
        };
        // Vegetative...
        assert_eq!(at(0.0), 18.0 * 1.2);
        // ...and PAST ANTHESIS, where its two neighbours are gated off but [F] Box 16.2
        // keeps applying this one (it gates on `CTU > tuEMR` only).
        assert_eq!(at(pheno.tsum_anthesis + 1.0), 18.0 * 1.2);
    }

    /// `WSFD`'s WIRING, on a CONSTRUCTED water-limited run — the pin the pure-function
    /// tests in `science.rs` cannot make.
    ///
    /// ⚠ **Every scenario in the Rust roster holds `WSFG == 1`**, so drought acceleration
    /// is bit-identically inert across the whole suite: dropping the `rate *=
    /// self.drought_factor(..)` multiply, or moving it inside the vegetative branch,
    /// leaves every golden, every parity run and every session test green. Measured, not
    /// assumed. This test manufactures the missing condition with a low
    /// `soil_moisture_index` (the same field `water_biting` uses on the Python side) and
    /// asserts the run's thermal time actually MOVES with `wssd`, in both directions.
    #[test]
    fn drought_acceleration_is_wired_into_the_accumulator_and_no_scenario_shows_it() {
        // The Python `WATER_BITING_SCENARIO` declaration, which the Rust roster has no
        // equivalent of — the whole reason this condition has to be manufactured here.
        // FTSW starts at MAI and stays far below wssg = 0.30 all season.
        let dry = SeasonScenario {
            sealed: true,
            litter_carbon0: 3.0,
            soil_moisture_index: 0.05,
            soil_water0: 0.975,
            subsoil_water0: 8.775,
            ..DEFAULT_SCENARIO
        };
        let thermal_time_after = |wssd: Option<f64>| -> f64 {
            let scenario = SeasonScenario { wssd, ..dry };
            let (last, rationed, _) =
                super::super::run_season_final(&scenario, 1).expect("dry run");
            assert_eq!(rationed, 0);
            last.aux[THERMAL_TIME]
        };
        let off = thermal_time_after(None);
        let accelerated = thermal_time_after(Some(0.40));
        let delayed = thermal_time_after(Some(-0.40));
        // Drought HASTENS development (Table 15.2), so the season accumulates MORE
        // thermal time; a negative coefficient is [F]'s provision for the species it
        // delays, and must move the other way.
        assert!(
            accelerated > off,
            "wssd = 0.40 must accelerate: {accelerated} vs {off}"
        );
        assert!(delayed < off, "wssd = -0.40 must delay: {delayed} vs {off}");
        // ...and the effect is bounded by 1 + wssd, never runaway.
        assert!(
            accelerated <= off * 1.40,
            "WSFD exceeded its 1 + WSSD bound"
        );
        // The identity, on a WET run: unstressed, the coefficient changes nothing at all.
        let wet = |wssd: Option<f64>| -> f64 {
            let scenario = SeasonScenario {
                wssd,
                ..DEFAULT_SCENARIO
            };
            let (last, _, _) = super::super::run_season_final(&scenario, 1).expect("wet run");
            last.aux[THERMAL_TIME]
        };
        assert_eq!(
            wet(Some(0.40)),
            wet(None),
            "WSFD must be BIT-identical where water does not limit"
        );
    }

    /// The re-sow water rule, called directly — the abandoned FRACTION of what the root
    /// zone held ([F] is silent here; the rule is ours).
    ///
    /// ⚠ **THIS REPLACED A PIN ON `captured_water(abandoned)`** — the abandoned column at
    /// the DRAINED UPPER LIMIT — on 2026-08-12. That form is right only for a full zone
    /// and exceeds the whole store once the store is geometric, at which point its clamp
    /// fired on every re-sow and handed the entire root zone to the subsoil (measured:
    /// the 4-year sealed station then made no grain at all). Two properties are asserted
    /// rather than the formula restated, because restating a formula in a second place is
    /// exactly how the Python side's test helper kept the old rule after it changed.
    #[test]
    fn the_resow_returns_the_abandoned_fraction_and_preserves_ftsw() {
        let scenario = SeasonScenario {
            ground_area: 2.0,
            sealed: true,
            ..DEFAULT_SCENARIO
        };
        let (state, _integrator, _resolver) = super::super::season_setup(&scenario, 1).unwrap();
        // A grown-in root zone, drawn down to a fraction of what it can hold.
        let grown_depth = 1.3;
        let capacity = science::transpirable_capacity(
            grown_depth,
            scenario.soil_extractable_water,
            scenario.ground_area,
        );
        let held = capacity * 0.6;
        let mut stocks = state.stocks.clone();
        stocks.insert(
            SOIL_WATER.to_string(),
            stocks[SOIL_WATER].with_amount(held).unwrap(),
        );
        // Enough grain to satisfy the seed-bank precondition.
        let seed = scenario.leaf_c0 + scenario.stem_c0 + scenario.root_c0 + 1.0;
        stocks.insert(
            STORAGE_C.to_string(),
            stocks[STORAGE_C].with_amount(seed).unwrap(),
        );
        let below0 = stocks[SUBSOIL_WATER].amount;
        let mut aux = state.aux.clone();
        aux.insert(ROOTED_DEPTH.to_string(), grown_depth);
        let before = State::new(state.n, stocks, state.rng_seed, aux).unwrap();

        let after = annual_reset(&before, &scenario).expect("re-sow");
        let returned = after.stocks[SUBSOIL_WATER].amount - below0;
        let lost = held - after.stocks[SOIL_WATER].amount;

        // (a) A REDISTRIBUTION: what the root zone lost, the subsoil gained, exactly.
        assert!(
            (lost - returned).abs() <= 1e-12 * lost,
            "leaked: {lost} vs {returned}"
        );
        // (b) The FRACTION rule, against the shared helper the reset must be calling.
        let want = science::resow_water_return(held, grown_depth, scenario.rooted_depth0);
        assert!(
            (returned - want).abs() <= 1e-12 * want,
            "returned {returned}, fraction rule {want}"
        );
        // (c) FTSW comes through UNCHANGED — the property the fraction rule exists for,
        // and the one the column-at-DUL form did not have.
        let ftsw_before = science::fraction_transpirable(held, capacity);
        let ftsw_after = science::fraction_transpirable(
            after.stocks[SOIL_WATER].amount,
            science::transpirable_capacity(
                after.aux[ROOTED_DEPTH],
                scenario.soil_extractable_water,
                scenario.ground_area,
            ),
        );
        assert!(
            (ftsw_before - ftsw_after).abs() <= 1e-12,
            "FTSW moved across the re-sow: {ftsw_before} -> {ftsw_after}"
        );
        assert!(
            (ftsw_before - 0.6).abs() < 1e-12,
            "the fixture is not 0.6 FTSW"
        );
        // (d) MUTATION GUARD: the old column-at-DUL form would have returned MORE than
        // the zone held here, so the two rules are not merely different in principle.
        let old_form = science::captured_water(
            grown_depth - scenario.rooted_depth0,
            scenario.soil_extractable_water,
            scenario.ground_area,
        );
        assert!(
            old_form > held,
            "the fixture does not distinguish the two rules"
        );
    }

    /// The open season runs the whole hard core to completion under the every-step
    /// conservation gate (a completed run is the proof), with the Tier-0 invariants.
    #[test]
    fn open_season_runs_well_fed_and_conserved() {
        let (final_state, rationed, events) =
            run_season_final(&DEFAULT_SCENARIO, 1).expect("open season");
        assert_eq!(final_state.n as usize, steps_for_years(1));
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
        assert_eq!(
            final_state.n as usize,
            steps_for_years(SEALED_CHAMBER_YEARS)
        );
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
            steps_for_years(PERENNIAL_CHAMBER_YEARS)
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
