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
            shade_rate: p.senesc.shade_rate,
            lai_threshold: p.senesc.lai_threshold,
            sla_per_mol_c: p.canopy.sla_per_mol_c,
            ground_area: scenario.ground_area,
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
            shade_rate: p.senesc.shade_rate,
            lai_threshold: p.senesc.lai_threshold,
            sla_per_mol_c: p.canopy.sla_per_mol_c,
            ground_area: scenario.ground_area,
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
    use super::super::{
        run_perennial_final, run_season_final, season_setup, steps_for_years, BIO_DT,
    };
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

    /// `f_N`'s WIRING, on a CONSTRUCTED nitrogen-limited run — the successor to the
    /// Python `n_limited` scenario, retired by C6 of the reference flip (2026-08-18).
    ///
    /// ⚠ **Every scenario in the Rust roster holds `f_N == 1` for the whole season**,
    /// so the nitrogen limiter is bit-identically inert across the suite: dropping the
    /// `f_water * f_n` multiply in `flows.rs` leaves every golden, every parity run and
    /// every session test green. Measured, not assumed — the wet half at the end of this
    /// test is that measurement, and it is why the condition has to be manufactured.
    ///
    /// The declaration copied here is the retired `N_LIMITED_SCENARIO`: a tiny sowing
    /// reserve (`plant_n0`) whose concentration sits inside the `f_N` band, over a soil
    /// below `sn_residual` so uptake is off. The bite is therefore PURE DILUTION — a
    /// fixed reserve spread through growing biomass — which is what made the scenario a
    /// clean single-limiter experiment. It carries the four claims of
    /// `tests/test_n_limited.py`: the bite is real and sustained, never N-dead, never
    /// rations, and is strictly worse than an otherwise-identical N-replete run.
    #[test]
    fn nitrogen_limitation_is_wired_into_assimilation_and_no_scenario_shows_it() {
        let nitro = params::nitrogen();
        let starved = SeasonScenario {
            plant_n0: 6e-5,
            soil_n0: 0.5,
            ..DEFAULT_SCENARIO
        };
        let vegetative =
            |s: &State| s.stocks[LEAF_C].amount + s.stocks[STEM_C].amount + s.stocks[ROOT_C].amount;
        let f_n = |s: &State| {
            science::nitrogen_stress_factor(
                s.stocks[PLANT_N].amount,
                vegetative(s),
                nitro.n_residual_per_mol_c,
                nitro.n_critical_per_mol_c,
            )
        };
        let trace = |scenario: &SeasonScenario| -> (Vec<State>, u64, Vec<Event>) {
            let (state, integrator, resolver) = season_setup(scenario, 1).expect("setup");
            let mut states: Vec<State> = Vec::new();
            let (_last, rationed, events) = run_season(
                &integrator,
                state,
                &resolver,
                BIO_DT,
                steps_for_years(1),
                None,
                &mut |s: &State| states.push(s.clone()),
            )
            .expect("run");
            (states, rationed, events)
        };

        let (states, rationed, events) = trace(&starved);
        let factors: Vec<f64> = states.iter().map(f_n).collect();
        let min_f = factors.iter().copied().fold(f64::INFINITY, f64::min);

        // A REAL bite, not float noise, and sustained rather than a one-step blip.
        assert!(min_f < 0.9, "f_N never bit: min {min_f}");
        assert!(
            factors.iter().filter(|f| **f < 0.9).count() > 30,
            "the bite was not sustained"
        );
        // Stressed, never N-DEAD: the limiter throttles growth, it does not kill.
        assert!(
            factors.iter().all(|f| *f > 0.0 && *f <= 1.0),
            "f_N left (0, 1]"
        );
        // Pure dilution: uptake is shut off, so the reserve is CONSTANT and the whole
        // fall in `f_N` is the growing biomass diluting it.
        for s in &states {
            assert_eq!(
                science::soil_n_availability(
                    s.stocks[SOIL_N].amount,
                    starved.sn_residual,
                    starved.sn_critical,
                ),
                0.0,
                "uptake was not shut off"
            );
            assert_eq!(s.stocks[PLANT_N].amount, starved.plant_n0);
        }
        // The limiter reduces DRAWS; it must never push the Euler backstop or kill the
        // crop (`rationed == 0` / `events == ()` were the scenario's own invariants).
        assert_eq!(rationed, 0);
        assert!(events.is_empty(), "{events:?}");

        // The cascade, direction only: against an otherwise-IDENTICAL N-replete run —
        // one field changed — the starved run reaches a LOWER peak vegetative biomass.
        let replete = SeasonScenario {
            plant_n0: 0.5,
            ..starved
        };
        let (replete_states, _, _) = trace(&replete);
        let peak = |ss: &[State]| ss.iter().map(vegetative).fold(f64::NEG_INFINITY, f64::max);
        assert!(
            replete_states.iter().all(|s| f_n(s) == 1.0),
            "the baseline is not genuinely N-replete"
        );
        assert!(
            peak(&states) < peak(&replete_states),
            "the limiter did not cost the crop anything"
        );

        // ...and THE MEASUREMENT this test's warning rests on: the frozen roster's own
        // declaration never leaves `f_N == 1`, so nothing else here can catch the wiring.
        let (frozen_states, _, _) = trace(&DEFAULT_SCENARIO);
        assert!(
            frozen_states.iter().all(|s| f_n(s) == 1.0),
            "a frozen scenario DOES reach the ramp — this warning is stale"
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

    /// ⚠ **The WIRING declines the drought modifier when no `WSSD` is cited** -
    /// the pin every other `WSFD` test provably cannot make.
    ///
    /// All the others construct `ThermalTimeAccumulation` directly, so every one of them
    /// stays green if `build_plants` wires the modifier UNCONDITIONALLY and a crop with
    /// no cited coefficient silently inherits wheat's 0.40. On the Python side that exact
    /// break passed the whole file; here it would additionally pass every golden, because
    /// no scenario in the Rust roster is water-limited.
    ///
    /// This walks the aux vector `build_plants` actually produces and EVALUATES it, which
    /// is a stronger claim than reading the struct fields back: the Python test asserts
    /// `proc.drought is None`, this asserts that a bone-dry root zone changes nothing.
    ///
    /// ⚠ The plot is deliberately OFF-DEFAULT (3.5 m2, EXTR 0.09, `wssg` 0.42,
    /// `wssd` 0.17). Every scenario in the tree has `ground_area == 1.0` and the
    /// reference EXTR, so a hardcoded 1.0 or a swapped threshold in the wiring is
    /// invisible on the defaults - the same blindness the soil-layers build recorded.
    /// Mirrors `test_the_wiring_declines_wssd_for_potato_not_just_the_scenario_field`
    /// and `test_thermal_time_aux_without_drought_is_the_plain_rate`.
    ///
    /// ⚠ **Potato has no Rust successor and that is a GAP, not a decision.** The
    /// Python test names `POTATO_SCENARIO` because [F] Table 15.1 has no potato row; the
    /// Rust roster has no potato build at all (`params.rs` records its stage 2 as
    /// deferred), so the crop-specific half of that claim cannot be ported. What is
    /// portable is the RULE - `wssd: None` declines the modifier - and that is what this
    /// asserts.
    #[test]
    fn the_wiring_declines_the_drought_modifier_when_no_wssd_is_cited() {
        const AREA: f64 = 3.5;
        const EXTR: f64 = 0.09;
        const WSSG: f64 = 0.42;
        const WSSD: f64 = 0.17;
        const DEPTH: f64 = 0.20;

        // Hand geometry: TTSW = 0.20 m x 0.09 x 1000 kg/m3 x 3.5 m2 = 63.0 kg, and
        // holding half of `wssg` worth of it puts FTSW at 0.21 - exactly half the
        // threshold - so WSFG = 0.5 and WSFD = (1 - 0.5) x 0.17 + 1 = 1.085.
        let ttsw = DEPTH * EXTR * 1000.0 * AREA;
        assert_eq!(ttsw, 63.0, "the hand geometry must be the tree's geometry");
        let held = 0.5 * WSSG * ttsw;
        let expected_accelerated = 18.0 * 1.085;

        struct WarmEnv;
        impl simcore::environment::Environment for WarmEnv {
            fn get(&self, _var: &str) -> Result<f64, SimError> {
                Ok(18.0)
            }
        }

        let p = params::biosphere();
        let increment = |wssd: Option<f64>| -> f64 {
            let scenario = SeasonScenario {
                wssd,
                ground_area: AREA,
                soil_extractable_water: EXTR,
                wssg: WSSG,
                ..DEFAULT_SCENARIO
            };
            let build = build_plants(&scenario, &p).expect("build_plants");
            let procs: Vec<&Box<dyn AuxProcess>> = build
                .aux
                .iter()
                .filter(|a| a.type_name() == "ThermalTimeAccumulation")
                .collect();
            assert_eq!(procs.len(), 1, "exactly one thermal-time accumulator");
            let mut stocks = std::collections::BTreeMap::new();
            stocks.insert(
                SOIL_WATER.to_string(),
                pool_stock(SOIL_WATER, SOIL, Quantity::Water, held).expect("soil water"),
            );
            // PAST ANTHESIS on purpose: it gates the two vegetative modifiers off, so
            // this reads WSFD alone rather than WSFD times an unvernalized zero.
            let state = State::new(
                0,
                stocks,
                0,
                std::collections::BTreeMap::from([
                    (ROOTED_DEPTH.to_string(), DEPTH),
                    (THERMAL_TIME.to_string(), p.pheno.tsum_anthesis + 1.0),
                ]),
            )
            .expect("stressed snapshot");
            procs[0].evaluate(&state, &WarmEnv, 1.0).expect("evaluate")[THERMAL_TIME]
        };

        // Cited: the modifier is wired, reads THIS plot's geometry and THIS threshold.
        let accelerated = increment(Some(WSSD));
        assert!(
            (accelerated - expected_accelerated).abs() <= 1.0e-12 * expected_accelerated,
            "wired WSFD must give {expected_accelerated}, got {accelerated}"
        );
        // Not cited: the same bone-dry root zone changes nothing at all.
        assert_eq!(
            increment(None),
            18.0,
            "an uncited WSSD must leave the plain degree-day rate byte-for-byte"
        );
    }

    // -----------------------------------------------------------------------------
    // S5 batch C, the season-level water and root-depth claims.
    //
    // ⚠ Three Python scenarios have no Rust roster entry — `DEEP_WATER`, `DROUGHT` and the
    // retired `WATER_BITING` — so the diagnostics below DECLARE their subject inline, the
    // shape `nitrogen_limitation_is_wired_into_assimilation_and_no_scenario_shows_it`
    // already uses in this file. They are diagnostics, not reference scenarios: adding
    // them to the production roster would put a new SeasonScenario in front of the freeze
    // manifest for no reference gain.
    // -----------------------------------------------------------------------------

    /// The Python `DEEP_WATER_SCENARIO`: the default stratified profile with the supply
    /// cut to 1 mm/day — deliberately below the measured 5.7744 kg/day peak demand, so
    /// what the roots can REACH decides the season.
    ///
    /// ⚠ The capacity is 1.0 because a sweep found it: at 2 mm/day and above the subsoil
    /// is irrelevant, at 0 the season is physically unwinnable (a 1.3 m root system over
    /// 1 m² can ever reach 169 kg against a 582 kg season demand). Choosing the operating
    /// point that exposes a mechanism is legitimate for a DIAGNOSTIC — provided the sweep
    /// is recorded, which it is, in the Python scenario's own comment. What would NOT be
    /// legitimate is quoting the effect size as a property of the model rather than of
    /// this scenario at this capacity.
    fn deep_water_scenario() -> SeasonScenario {
        SeasonScenario {
            irrigation_mm_day: 1.0,
            ..DEFAULT_SCENARIO
        }
    }

    /// Run a scenario, returning every emitted state.
    fn trace_season(scenario: &SeasonScenario, years: usize) -> Vec<State> {
        let (state, integrator, resolver) = super::super::season_setup(scenario, years).unwrap();
        let mut seen: Vec<State> = Vec::new();
        let mut observe = |s: &State| seen.push(s.clone());
        let (_, rationed, events) = run_season(
            &integrator,
            state,
            &resolver,
            super::super::BIO_DT,
            super::super::steps_for_years(years),
            None,
            &mut observe,
        )
        .expect("season");
        assert_eq!(rationed, 0, "the backstop must stay out of a water diagnostic");
        assert!(events.is_empty(), "unexpected extinction: {events:?}");
        seen
    }

    /// The same season with exactly ONE flow removed from the registry.
    ///
    /// ⚠ **THE CONTROL THIS CLAIM NEEDS, AND THE ONE THE PYTHON FILE RECORDS AS HAVING
    /// BEEN DESTROYED ONCE ALREADY.** The obvious control — `soil_extractable_water = 0`
    /// — was justified as "removes the transfer, leaves rooted depth to grow exactly as
    /// in the subject". That held while `EXTR` appeared in ONE place; since the geometry
    /// re-basing it appears in TWO, the transfer *and* `TTSW`, so a zero makes every
    /// stress reading `FTSW = 0` and kills the crop outright rather than isolating the
    /// transfer. A control that changes more than it claims is worse than no control.
    /// Dropping the flow from the registry changes exactly one thing.
    ///
    /// No production seam was added for this: `compartments` is module-private and this
    /// test module is inside the module, so the flow list is reachable before
    /// `Registry::new` closes over it.
    fn trace_without_flow(scenario: &SeasonScenario, drop_id: &str) -> Vec<State> {
        let p = params::biosphere();
        let builds = compartments(scenario, &p).expect("compartments");
        let mut stocks: BTreeMap<String, Stock> = BTreeMap::new();
        for build in &builds {
            for stock in &build.stocks {
                stocks.insert(stock.id.clone(), stock.clone());
            }
        }
        for (id, s) in boundary::loss_sinks(&[Quantity::Carbon]).expect("loss sinks") {
            stocks.insert(id, s);
        }
        let mut flows: Vec<Box<dyn Flow>> = Vec::new();
        let mut aux: Vec<Box<dyn AuxProcess>> = Vec::new();
        let mut dropped = 0usize;
        for build in builds {
            for flow in build.flows {
                if flow.id() == drop_id {
                    dropped += 1;
                } else {
                    flows.push(flow);
                }
            }
            aux.extend(build.aux);
        }
        assert_eq!(dropped, 1, "{drop_id} is not in this scenario's registry");
        let state = State::new(
            0,
            stocks.clone(),
            0,
            BTreeMap::from([
                (THERMAL_TIME.to_string(), 0.0),
                (VERNALIZATION_DAYS.to_string(), 0.0),
                (ROOTED_DEPTH.to_string(), scenario.rooted_depth0),
            ]),
        )
        .expect("state");
        let registry = Registry::new(flows, &stocks, aux).expect("trimmed registry");
        let integrator = EulerIntegrator::new(registry);
        let resolver = super::super::weather_resolver(scenario, 1).expect("resolver");
        let mut seen: Vec<State> = Vec::new();
        let mut observe = |s: &State| seen.push(s.clone());
        let (_, rationed, _) = run_season(
            &integrator,
            state,
            &resolver,
            super::super::BIO_DT,
            super::super::steps_for_years(1),
            None,
            &mut observe,
        )
        .expect("control season");
        assert_eq!(rationed, 0);
        seen
    }

    /// Rooted depth follows [E]'s law over a whole season: monotone, gated, and stopped at
    /// the cited cap with a bounded overshoot.
    ///
    /// The overshoot bound is the documented consequence of cutting the RATE at the cap
    /// rather than clamping the increment — clamping would break the aux channel's
    /// dt-independence contract, so one step's unstressed extension is allowed through and
    /// nothing more. ⚠ The GATING half is the part that distinguishes `0.018 m/day` from
    /// `0.018 m/day × f_water × f_temp`: without it the crop would reach the cap in
    /// exactly 73 days, and the two are otherwise indistinguishable from a trajectory.
    /// Mirrors `test_depth_follows_es_law_and_stops_at_the_cited_cap` and
    /// `test_extension_is_temperature_and_water_gated_not_a_flat_rate`.
    #[test]
    fn rooted_depth_follows_the_gated_extension_law_and_stops_at_the_cited_cap() {
        let rootd = params::root_depth();
        let depths: Vec<f64> = trace_season(&DEFAULT_SCENARIO, 1)
            .iter()
            .map(|s| s.aux[ROOTED_DEPTH])
            .collect();
        // A sown crop starts at the CITED emergence depth, not at 0 ([F] Ch. 14: "It is
        // normally between 150 to 400 mm").
        assert_eq!(depths[0], DEFAULT_SCENARIO.rooted_depth0);
        assert!((0.15..=0.40).contains(&DEFAULT_SCENARIO.rooted_depth0));
        // Monotone: roots only deepen.
        assert!(
            depths.windows(2).all(|w| w[1] >= w[0]),
            "rooted depth went backwards"
        );
        // The cap binds, and the overshoot is at most ONE step's unstressed extension.
        let deepest = depths.iter().copied().fold(f64::MIN, f64::max);
        assert!(deepest >= rootd.max_rooted_depth, "the cap never bound: {deepest}");
        assert!(
            deepest <= rootd.max_rooted_depth + rootd.max_extension_rate,
            "overshoot beyond one step: {deepest}"
        );
        // No step extends faster than the unstressed maximum — f_water, f_temp <= 1.
        // ⚠ In STEPS, not days: the engine runs at dt = 1/4, so the per-step ceiling is a
        // QUARTER of the daily rate. A test that compared against the daily rate would
        // pass on a build that had dropped `dt` from the accumulator entirely.
        let per_step = rootd.max_extension_rate * super::super::BIO_DT;
        let steps: Vec<f64> = depths.windows(2).map(|w| w[1] - w[0]).collect();
        let fastest = steps.iter().copied().fold(f64::MIN, f64::max);
        assert!(fastest <= per_step + 1e-15, "a step of {fastest} > {per_step}");
        // ...and the two response factors really are applied: many steps are strictly
        // slower than the maximum without being zero.
        let throttled = steps
            .iter()
            .filter(|s| **s > 0.0 && **s < per_step - 1e-12)
            .count();
        assert!(
            throttled > 30,
            "the temperature and water gates are not being applied ({throttled} throttled steps)"
        );
    }

    /// [E] p. 136: "Root growth generally stops around flowering" — exercised DIRECTLY,
    /// because for the frozen winter wheat the 1.3 m cap binds first (~day 140 against
    /// anthesis ~day 255) so a full run cannot tell the two cut-offs apart.
    ///
    /// The fixture lifts BOTH the crop cap and the soil cap to 99 m, leaving the flowering
    /// stop as the only thing that can bind, and evaluates the accumulator on two
    /// hand-built states either side of anthesis.
    /// Mirrors `test_root_growth_stops_at_flowering`.
    #[test]
    fn root_growth_stops_at_flowering_when_neither_cap_can_bind() {
        struct WarmEnv;
        impl simcore::environment::Environment for WarmEnv {
            fn get(&self, _var: &str) -> Result<f64, SimError> {
                Ok(20.0) // comfortably inside the optimum plateau
            }
        }
        let pheno = params::phenology();
        let proc = RootDepthExtension {
            id: "test.rooted_depth".to_string(),
            accumulator: ROOTED_DEPTH.to_string(),
            thermal_time_aux: THERMAL_TIME.to_string(),
            temp_var: TEMP_VAR.to_string(),
            soil_water: SOIL_WATER.to_string(),
            subsoil_water: SUBSOIL_WATER.to_string(),
            params: params::RootDepthParams {
                max_extension_rate: 0.018,
                max_rooted_depth: 99.0,
            },
            photo: params::photosynthesis(),
            pheno,
            wssg: DEFAULT_SCENARIO.wssg,
            soil_depth: 99.0, // the SOIL cap lifted too
            soil_extractable_water: DEFAULT_SCENARIO.soil_extractable_water,
            ground_area: DEFAULT_SCENARIO.ground_area,
        };
        let (base, _) = build_season(&DEFAULT_SCENARIO).expect("season");
        let at = |thermal_time: f64| -> State {
            State::new(
                0,
                base.stocks.clone(),
                0,
                BTreeMap::from([
                    (THERMAL_TIME.to_string(), thermal_time),
                    (ROOTED_DEPTH.to_string(), 0.5),
                ]),
            )
            .expect("hand-built state")
        };
        let vegetative = proc.evaluate(&at(0.0), &WarmEnv, 1.0).expect("vegetative")
            [ROOTED_DEPTH];
        // Past `tsum_anthesis` the crop is at DVS >= 1 and extension is off.
        let flowering = proc
            .evaluate(&at(pheno.tsum_anthesis + 1.0), &WarmEnv, 1.0)
            .expect("flowering")[ROOTED_DEPTH];
        assert!(vegetative > 0.0, "the vegetative crop must still be rooting");
        assert_eq!(flowering, 0.0, "root growth must stop at flowering");
        // ⚠ ON the boundary, not merely past it: `is_vegetative` tests DVS < 1, so
        // exactly at anthesis the stop must already have fired. The port's own hazard —
        // the Python file does not make this distinction — and the same boundary batch B
        // pinned for the anthesis gate.
        let exactly = proc
            .evaluate(&at(pheno.tsum_anthesis), &WarmEnv, 1.0)
            .expect("at anthesis")[ROOTED_DEPTH];
        assert_eq!(exactly, 0.0, "the stop must fire ON the anthesis boundary");
    }

    /// A dry below-root store stops extension for the WHOLE SEASON, and a wet one lets the
    /// roots reach the crop's cap.
    ///
    /// [F] Box 14.1's `If WSTORG = 0 Then GRTD = 0` — roots do not extend into dry soil.
    /// This is what makes a scenario's `subsoil_water0` load-bearing rather than
    /// decorative, and it is the season-level companion to the equation-level pin in
    /// `science.rs`. The dry declaration is the Python `DROUGHT_SCENARIO`'s, which the
    /// Rust roster has no entry for.
    /// Mirrors `test_a_dry_subsoil_stops_root_extension`.
    #[test]
    fn a_dry_below_root_store_stops_extension_for_the_whole_season() {
        let dry = SeasonScenario {
            subsoil_water0: 0.0,
            ..DEFAULT_SCENARIO
        };
        let depths: Vec<f64> = trace_season(&dry, 1)
            .iter()
            .map(|s| s.aux[ROOTED_DEPTH])
            .collect();
        assert!(
            depths.iter().all(|d| *d == DEFAULT_SCENARIO.rooted_depth0),
            "a crop over dry soil rooted anyway: max {}",
            depths.iter().copied().fold(f64::MIN, f64::max)
        );
        // ⚠ Non-vacuity, and it is the whole point: the SAME season with water below
        // reaches the crop's cap. Without this half the test passes on a build where
        // extension is broken outright.
        let wet: Vec<f64> = trace_season(&DEFAULT_SCENARIO, 1)
            .iter()
            .map(|s| s.aux[ROOTED_DEPTH])
            .collect();
        assert!(wet.iter().copied().fold(f64::MIN, f64::max) > 1.3);
        // The DROUGHT scenario's own construction: a dry layer under a root zone that is
        // still at the drained upper limit. The leanness is the STRATIFICATION, not a dry
        // bed — and it survives only because the sowing depth is a cited nonzero (at depth
        // 0 the root-zone access fraction is 0 and nitrogen uptake would be off).
        assert_eq!(dry.soil_water0, DEFAULT_SCENARIO.soil_water0);
        assert!(dry.rooted_depth0 > 0.0);
    }

    /// A re-sown crop starts with the SOWING root system, not with the old one and not
    /// with none.
    ///
    /// Rooted depth is a property of the standing crop, not of the soil, so it resets with
    /// the other per-cycle accumulators. No golden can see it (measured bit-identical
    /// either way on the Python side), and the chambers re-sow many times.
    /// ⚠ The recorded value just after a reset is NOT exactly `rooted_depth0`:
    /// `annual_reset` sets the accumulator and the same step then applies one extension
    /// increment before the state is snapshotted. So the bound is one unstressed STEP
    /// (`rate · dt`), not one day — pinning the value exactly would be pinning the reset's
    /// position within the step.
    /// Mirrors `test_a_resown_crop_starts_with_the_sowing_root_system`.
    #[test]
    fn a_resown_crop_starts_with_the_sowing_root_system() {
        let scenario = perennial_chamber_scenario();
        let (state, integrator, resolver) = super::super::season_setup(&scenario, 3).unwrap();
        let mut depths: Vec<f64> = Vec::new();
        let mut observe = |s: &State| depths.push(s.aux[ROOTED_DEPTH]);
        let (_, rationed, _) = run_perennial(
            &integrator,
            state,
            &scenario,
            &resolver,
            super::super::BIO_DT,
            super::super::steps_for_years(3),
            super::super::season_steps(),
            &mut observe,
        )
        .expect("perennial");
        assert_eq!(rationed, 0);
        let drops: Vec<usize> = depths
            .windows(2)
            .enumerate()
            .filter(|(_, w)| w[1] < w[0])
            .map(|(i, _)| i)
            .collect();
        assert!(
            !drops.is_empty(),
            "rooted depth never reset — a re-sown crop kept the old root system"
        );
        let sown = scenario.rooted_depth0;
        let per_step = params::root_depth().max_extension_rate * super::super::BIO_DT;
        for i in drops {
            assert!(
                depths[i + 1] >= sown && depths[i + 1] <= sown + per_step,
                "post-reset depth {} is not the sowing root system",
                depths[i + 1]
            );
            // ...and it is a genuine reset, not a small dip.
            assert!(depths[i + 1] < depths[i] / 2.0);
        }
    }

    /// The root-zone access gate THROTTLES nitrogen supply — and is bit-identically inert
    /// on the frozen reference, which is a measured fact rather than a disclaimer.
    ///
    /// Both halves are needed and they say opposite-looking things. The gate does real
    /// work: against a reference layer far deeper than the crop can reach, `FROOT1` stays
    /// small all season and the crop takes up strictly less nitrogen than the same run
    /// with the gate wide open. And on the FROZEN scenario it changes nothing, because
    /// uptake is demand-bound there and the gate only shrinks supply — so if the second
    /// half ever reddens, the frozen crop has become supply-bound and `soil_layer_depth`
    /// (a DESIGN value) has silently become load-bearing.
    /// Mirrors `test_the_gate_scales_uptake_capacity` and
    /// `test_the_gate_is_inert_on_the_frozen_reference_and_that_is_recorded`.
    #[test]
    fn the_root_zone_gate_throttles_uptake_and_is_inert_on_the_frozen_reference() {
        let final_n = |layer: f64| -> f64 {
            let scenario = SeasonScenario {
                soil_layer_depth: layer,
                ..DEFAULT_SCENARIO
            };
            super::super::run_season_final(&scenario, 1)
                .expect("season")
                .0
                .stocks[PLANT_N]
                .amount
        };
        // A 50 m reference layer keeps FROOT1 tiny all season; 0.01 m saturates it after
        // one step.
        let gated = final_n(50.0);
        let open = final_n(0.01);
        assert!(
            gated < open,
            "the root-zone gate does not restrict nitrogen uptake ({gated} vs {open})"
        );
        // INERT ON THE FROZEN REFERENCE, to the BIT: the default layer and a layer so
        // shallow the gate is wide open from step 1 give the same number.
        let frozen = super::super::run_season_final(&DEFAULT_SCENARIO, 1)
            .expect("frozen season")
            .0
            .stocks[PLANT_N]
            .amount;
        assert_eq!(
            frozen.to_bits(),
            final_n(0.0001).to_bits(),
            "the frozen crop has become supply-bound; soil_layer_depth is now load-bearing"
        );
    }

    /// THE HEADLINE WATER CLAIM: reaching the below-root store is what saves a crop whose
    /// supply is deliberately below its demand — measured against a control that removes
    /// ONLY the transfer.
    ///
    /// ⚠ The effect size is a property of THIS scenario at THIS irrigation capacity, not
    /// of the model, and the bound is two-sided so a slide in either direction is caught
    /// rather than absorbed. The Python side has watched this ratio move twice for reasons
    /// unrelated to what it measures (WSFD in 2026-08-12, the depth-resolved canopy in
    /// 2026-08-15), each time with nothing red because the bound had slack — which is why
    /// it is re-measured on this port rather than copied across.
    /// Mirrors `test_reaching_the_subsoil_is_what_saves_the_deep_water_crop`.
    #[test]
    fn reaching_the_below_root_store_is_what_saves_the_deep_water_crop() {
        let scenario = deep_water_scenario();
        let subject = trace_season(&scenario, 1);
        let control = trace_without_flow(&scenario, "biosphere.root_zone_capture");
        let peak = |states: &[State]| -> f64 {
            states
                .iter()
                .map(|s| s.stocks[LEAF_C].amount)
                .fold(f64::MIN, f64::max)
        };
        // SAME ROOT SYSTEM IN BOTH — that is what makes this a water measurement rather
        // than a rooting one. The control's roots still grow: `subsoil_water` is full, it
        // is simply never drawn from.
        let deepest = |states: &[State]| -> f64 {
            states
                .iter()
                .map(|s| s.aux[ROOTED_DEPTH])
                .fold(f64::MIN, f64::max)
        };
        assert!(
            (deepest(&subject) - deepest(&control)).abs() < 0.02,
            "the two runs grew different root systems: {} vs {}",
            deepest(&subject),
            deepest(&control)
        );
        // The rescue itself, an order of magnitude in the canopy and several times the
        // grain. Measured on this port; see the plan record for the values.
        let leaf_ratio = peak(&subject) / peak(&control);
        let grain_ratio = subject.last().unwrap().stocks[STORAGE_C].amount
            / control.last().unwrap().stocks[STORAGE_C].amount;
        // ⚠ MEASURED ON THIS PORT: leaf 9.5775x, grain 5.8281x. Two-sided on the
        // measurement, and the bands are the Python file's own current ones — which its
        // last two re-measurements landed on independently. That agreement is a
        // cross-port reading, not a copied literal: the numbers were produced here first
        // and then found to sit inside bounds Python had already re-pinned twice.
        assert!(
            (9.0..10.5).contains(&leaf_ratio),
            "the canopy rescue moved: {leaf_ratio}"
        );
        assert!(
            (5.5..6.2).contains(&grain_ratio),
            "the grain rescue moved: {grain_ratio}"
        );
        assert!(
            subject.last().unwrap().stocks[STORAGE_C].amount > 2.5,
            "the subject did not fill grain, so the ratio is about two failures"
        );
    }

    /// The two controls for the claim above are NOT interchangeable, and the difference is
    /// the honest result rather than a nuisance.
    ///
    /// The naive control (`subsoil_water0 = 0`) removes the water AND freezes rooted depth
    /// through the `WSTORG = 0` gate, so it also moves the depth-gated nitrogen supply.
    /// The clean control removes only the transfer. This project has had a causal claim
    /// ("one cause, two symptoms") come back at 39 % before, so the attribution is
    /// measured rather than asserted — and pinned so nobody "simplifies" the headline test
    /// back onto the cheaper control.
    /// Mirrors `test_the_deep_water_effect_is_water_and_not_the_nitrogen_gate`.
    #[test]
    fn the_clean_and_naive_deep_water_controls_are_not_interchangeable() {
        let scenario = deep_water_scenario();
        let clean = trace_without_flow(&scenario, "biosphere.root_zone_capture");
        let naive = trace_season(
            &SeasonScenario {
                subsoil_water0: 0.0,
                ..scenario
            },
            1,
        );
        let last_clean = clean.last().unwrap();
        let last_naive = naive.last().unwrap();
        // The naive control freezes depth at sowing; the clean one lets it grow.
        assert_eq!(last_naive.aux[ROOTED_DEPTH], scenario.rooted_depth0);
        assert!(last_clean.aux[ROOTED_DEPTH] > 1.0);
        // So they are different experiments, and they do not agree.
        assert_ne!(
            last_clean.stocks[LEAF_C].amount,
            last_naive.stocks[LEAF_C].amount
        );
    }

    /// The three cycle flows carry ONLY water, in the ring order soil → atmosphere →
    /// water → soil.
    ///
    /// ⚠ THE WIRING CHECK THE LEDGER IDENTITY CANNOT MAKE: both sides of a conservation
    /// identity move together under a mislabel, so a flow plumbed to the wrong pool still
    /// balances. Evaluated against the registry's REAL flow objects on a synthetic
    /// all-positive state, so a builder mis-wiring is caught here. The before-battery is
    /// the reason: making `Recycling` read `soil_water` instead of `condensate` — still
    /// perfectly balanced — reddened nine tests, none of them about the ring.
    ///
    /// ⚠ The rooted-depth aux is LOAD-BEARING in this fixture. Stress divides by
    /// `TTSW = depth · EXTR · ρ · A`, so a synthetic state without it has zero capacity,
    /// zero FTSW, and a transpiration flow that returns nothing at all — a silent pass for
    /// a test whose whole subject is that these flows carry water.
    /// Mirrors `test_three_cycle_flows_carry_only_water_in_ring_order`.
    #[test]
    fn the_three_cycle_flows_carry_only_water_in_ring_order() {
        let scenario = sealed_chamber_scenario();
        let (base, registry) = build_season(&scenario).expect("sealed season");
        // A 1.3 m root zone holding 1000 kg is FTSW 4.7, far above wssg — f_water = 1 —
        // and both ring pools are nonzero so every flux is positive.
        let mut stocks = base.stocks.clone();
        for (id, amount) in [
            (SOIL_WATER, 1000.0),
            (WATER_VAPOR, 5.0),
            (CONDENSATE, 5.0),
        ] {
            stocks.insert(id.to_string(), stocks[id].with_amount(amount).unwrap());
        }
        let mut aux = base.aux.clone();
        aux.insert(ROOTED_DEPTH.to_string(), 1.3);
        let state = State::new(base.n, stocks, base.rng_seed, aux).expect("ring state");
        let resolver = super::super::weather_resolver(&scenario, 1).expect("resolver");
        let env = resolver.bind(&state, 1.0);

        let expected: [(&str, &str, &str); 3] = [
            ("biosphere.transpiration", SOIL_WATER, WATER_VAPOR),
            ("biosphere.condensation", WATER_VAPOR, CONDENSATE),
            ("biosphere.recycling", CONDENSATE, SOIL_WATER),
        ];
        for (id, want_source, want_sink) in expected {
            let flow = registry
                .flows()
                .iter()
                .find(|f| f.id() == id)
                .unwrap_or_else(|| panic!("{id} is not in the sealed registry"));
            let result = flow.evaluate(&state, &env, 1.0).expect("evaluate");
            // WATER only: every touched stock carries WATER and nothing else.
            for leg in &result.legs {
                let stock = &state.stocks[&leg.stock];
                assert_eq!(stock.quantity, Quantity::Water, "{id} touched {}", leg.stock);
                assert!(
                    stock.composition.is_empty()
                        || stock.composition.keys().all(|q| *q == Quantity::Water),
                    "{id} touched a multi-quantity stock"
                );
            }
            simcore::flow::assert_flow_balanced_default(&result, &state.stocks)
                .unwrap_or_else(|e| panic!("{id} is unbalanced: {e:?}"));
            // Exactly one source and one sink, and they are the ring's.
            let sources: Vec<&str> = result
                .legs
                .iter()
                .filter(|l| l.amount < 0.0)
                .map(|l| l.stock.as_str())
                .collect();
            let sinks: Vec<&str> = result
                .legs
                .iter()
                .filter(|l| l.amount > 0.0)
                .map(|l| l.stock.as_str())
                .collect();
            assert_eq!(sources, vec![want_source], "{id} draws from the wrong pool");
            assert_eq!(sinks, vec![want_sink], "{id} delivers to the wrong pool");
        }
    }

    /// The sealed chamber's water is conserved over ALL FOUR stores, and the ring is
    /// genuinely running.
    ///
    /// ⚠ `subsoil_water` is a term here, and NOT because the ring grew a fourth leg. The
    /// below-root store is in-system soil water that `RootZoneCapture` moves into
    /// `soil_water` as the roots reach it; it crosses no boundary, so a sealed chamber's
    /// water is conserved over four stocks rather than three. Omitting it would make a
    /// conserved transfer look like a leak — and including it without the non-vacuity
    /// half below would make a frozen ring look conserved.
    /// Mirrors `test_sealed_closed_water_loop_is_conserved`,
    /// `test_sealed_water_cycle_is_active_and_distributes` and
    /// `test_sealed_water_cycle_never_rations`.
    #[test]
    fn the_sealed_chamber_conserves_water_over_all_four_stores() {
        let scenario = sealed_chamber_scenario();
        let (state, integrator, resolver) =
            super::super::season_setup(&scenario, SEALED_CHAMBER_YEARS).unwrap();
        let total = |s: &State| -> f64 {
            s.stocks[SOIL_WATER].amount
                + s.stocks[SUBSOIL_WATER].amount
                + s.stocks[WATER_VAPOR].amount
                + s.stocks[CONDENSATE].amount
        };
        let mut totals: Vec<f64> = Vec::new();
        let mut peak_vapor = 0.0f64;
        let mut peak_condensate = 0.0f64;
        let mut first: Option<(f64, f64)> = None;
        let mut observe = |s: &State| {
            totals.push(total(s));
            peak_vapor = peak_vapor.max(s.stocks[WATER_VAPOR].amount);
            peak_condensate = peak_condensate.max(s.stocks[CONDENSATE].amount);
            if first.is_none() {
                first = Some((s.stocks[WATER_VAPOR].amount, s.stocks[CONDENSATE].amount));
            }
        };
        let (_, rationed, _) = run_season(
            &integrator,
            state,
            &resolver,
            super::super::BIO_DT,
            super::super::steps_for_years(SEALED_CHAMBER_YEARS),
            None,
            &mut observe,
        )
        .expect("sealed season");
        // Structural positivity through the ring: each first-order draw self-limits
        // against the start-of-step pool, so the Euler backstop never fires.
        assert_eq!(rationed, 0, "the closed water ring needed the backstop");
        // CONSERVED. `soil_water` is O(1e2-1e3) kg, so the band is absolute rather than
        // relative — the float-subtraction noise of the large pool.
        let first_total = totals[0];
        for t in &totals {
            assert!(
                (t - first_total).abs() < 1e-7,
                "water leaked: {t} vs {first_total}"
            );
        }
        // NON-VACUITY: the ring genuinely moves water. Both pools start at zero and build
        // from transpiration, so a frozen cycle would be conserved and useless.
        assert_eq!(first, Some((0.0, 0.0)));
        assert!(peak_vapor > 1e-3, "the vapour pool never filled: {peak_vapor}");
        assert!(
            peak_condensate > 1e-3,
            "the condensate pool never filled: {peak_condensate}"
        );
    }


    /// The re-sow is a CYCLE and not a RATCHET — the below-root store lands on a fixed
    /// point over five cycles instead of stepping down.
    ///
    /// ⚠ NO SINGLE GOLDEN CAN SHOW THIS, and neither can the single-`annual_reset` pin
    /// above it: `RootZoneCapture` is one-way within a season, so without a return leg
    /// every re-sow would move more of the profile permanently into the root zone, and a
    /// one-cycle test cannot tell a return from a smaller ratchet.
    ///
    /// ⚠ The claim is sharper than "not a ratchet" and the tolerance says so. The rule
    /// this replaced returned the abandoned column *at the drained upper limit*, which made
    /// every cycle start identical from year 2; the FRACTION rule instead CONVERGES — one
    /// transient cycle, then a fixed point held to round-off. So the assertion is a
    /// convergence at 1e-12, four orders below the transient it has to distinguish itself
    /// from, plus the transient's own existence and direction.
    /// ⚠ HONEST SCOPE, measured rather than assumed. This catches a return that STOPS (the
    /// true ratchet: the store steps down every cycle), not a return that is merely the
    /// wrong SIZE. A multiplicative drift on the fraction — `× 1.000001` — converges to a
    /// *different* fixed point rather than ratcheting, so it leaves this test green and is
    /// caught instead by the exact-value pins in `science.rs` and in
    /// `the_resow_returns_the_abandoned_fraction_and_preserves_ftsw`. The three tests are a
    /// set, and this is the one that covers the shape no single golden can see.
    /// Mirrors `test_the_resow_returns_the_abandoned_zones_water_so_there_is_no_ratchet`.
    #[test]
    fn the_resow_makes_a_cycle_and_not_a_ratchet_over_five_years() {
        let scenario = perennial_chamber_scenario();
        let years = PERENNIAL_CHAMBER_YEARS;
        let (state, integrator, resolver) = super::super::season_setup(&scenario, years).unwrap();
        let mut below: Vec<f64> = Vec::new();
        let mut both: Vec<f64> = Vec::new();
        let mut observe = |s: &State| {
            below.push(s.stocks[SUBSOIL_WATER].amount);
            both.push(s.stocks[SUBSOIL_WATER].amount + s.stocks[SOIL_WATER].amount);
        };
        let year = super::super::season_steps();
        let (_, rationed, _) = run_perennial(
            &integrator,
            state,
            &scenario,
            &resolver,
            super::super::BIO_DT,
            super::super::steps_for_years(years),
            year,
            &mut observe,
        )
        .expect("perennial");
        assert_eq!(rationed, 0);
        // The store at the same point in each cycle — one step after the re-sow refills it.
        // ⚠ In STEPS: `year` is a step count and the `+ 1` means "one integration step
        // after the reset", not "one day after".
        let at_cycle_start: Vec<f64> = (1..years).map(|i| below[i * year + 1]).collect();
        assert_eq!(at_cycle_start.len(), years - 1);
        let settled = &at_cycle_start[1..];
        for v in settled {
            assert!(
                (v - settled[0]).abs() <= 1e-12 * settled[0].abs(),
                "the below-root store drifts rather than settling: {at_cycle_start:?}"
            );
        }
        // The transient is ONE cycle wide, small, and in one direction — not the first two
        // steps of a slow ratchet that happen to look flat at this tolerance.
        assert_ne!(at_cycle_start[0], settled[0]);
        assert!((at_cycle_start[0] - settled[0]).abs() / settled[0] < 1e-3);
        assert!(at_cycle_start[0] > settled[0]);
        // The two soil stores TOGETHER are conserved across every cycle boundary, which is
        // what says the convergence is a REDISTRIBUTION and not a leak.
        let totals: Vec<f64> = (1..years).map(|i| both[i * year + 1]).collect();
        for t in &totals {
            assert!(
                (t - totals[0]).abs() <= 1e-14 * totals[0].abs(),
                "the two stores together are not conserved across cycles: {totals:?}"
            );
        }
        // NON-VACUITY: the store really is drawn down within a year, so "no ratchet" is not
        // trivially true of a mechanism that never ran.
        let lowest = below.iter().copied().fold(f64::INFINITY, f64::min);
        assert!(lowest < at_cycle_start[0] / 2.0, "the capture never ran: {lowest}");
    }

    /// `RootZoneCapture` is a BALANCED INTERNAL transfer — both legs are in-system soil
    /// stocks summing to zero, and neither is a boundary.
    ///
    /// It only re-labels which water the crop can reach. ⚠ If it ever gained a boundary
    /// leg, a sealed chamber's water would stop being conserved — and the every-step
    /// conservation gate would NOT name this flow, because it folds state deltas after all
    /// flows are applied. Batch A's review found the biosphere called
    /// `assert_flow_balanced_default` nowhere in the crate and added the gas-flow case;
    /// this is the water one.
    /// Mirrors `test_the_capture_is_a_balanced_internal_transfer`.
    #[test]
    fn the_root_zone_capture_is_a_balanced_internal_transfer() {
        let scenario = DEFAULT_SCENARIO;
        let (state, registry) = build_season(&scenario).expect("season");
        let resolver = super::super::weather_resolver(&scenario, 1).expect("resolver");
        let env = resolver.bind(&state, 1.0);
        let flow = registry
            .flows()
            .iter()
            .find(|f| f.id() == "biosphere.root_zone_capture")
            .expect("the capture is in the registry");
        let result = flow.evaluate(&state, &env, 1.0).expect("evaluate");
        simcore::flow::assert_flow_balanced_default(&result, &state.stocks)
            .expect("the capture must balance leg by leg");
        let sum: f64 = result.legs.iter().map(|l| l.amount).sum();
        assert_eq!(sum, 0.0, "a single-currency internal transfer must sum to zero");
        let touched: std::collections::BTreeSet<&str> =
            result.legs.iter().map(|l| l.stock.as_str()).collect();
        assert_eq!(
            touched,
            std::collections::BTreeSet::from([SOIL_WATER, SUBSOIL_WATER])
        );
        for leg in &result.legs {
            assert!(
                !leg.stock.starts_with("boundary."),
                "the capture crossed a boundary: {}",
                leg.stock
            );
        }
        // NON-VACUITY: it actually moved water on this step, or the sum is zero trivially.
        assert!(
            result.legs.iter().any(|l| l.amount != 0.0),
            "the capture moved nothing, so balance is vacuous here"
        );
    }

    // -----------------------------------------------------------------------------
    // S5 batch E — nitrogen: the two claims that live on a RUN rather than on a flow.
    // -----------------------------------------------------------------------------

    /// The re-sow splits the parent's nitrogen by CONCENTRATION; it does not hand it over.
    ///
    /// The seedling keeps the tissue concentration the dying crop had, and the remainder
    /// dies to litter as the balancing residual — the same idiom the carbon half uses, so
    /// nitrogen is conserved by construction rather than by formula. Before coupled
    /// shedding the reset was carbon-only, which left the seedling holding the whole
    /// parent's `plant_n`: an N windfall on a fraction of the biomass.
    ///
    /// ⚠⚠ THIS CLAIM WAS PREVIOUSLY GUARDED BY NOTHING THAT COULD SEE IT, IN EITHER TREE,
    /// and the two halves of that are different failures.
    ///
    /// In Rust: restoring the windfall (`seedling_n = old_plant_n`) reddened ZERO tests of
    /// `-p domains --lib` and only committed golden bytes of the whole workspace. It
    /// cannot be otherwise — the windfall is a REDISTRIBUTION between two stocks, so the
    /// conservation assertion that runs on every step and across every reset is blind to
    /// it by construction. Batch D's lesson, one mechanism over.
    ///
    /// In Python: `test_nitrogen_form.py::test_nitrogen_is_conserved_across_the_annual_reset`
    /// drives `PERENNIAL_CHAMBER_SCENARIO` through `run_season` — the driver with NO reset
    /// hook — so it never crosses a reset at all. Measured, not inferred: with the litter
    /// leg of the reset deleted outright, so that nitrogen is DESTROYED at every year
    /// boundary, that test still passes. (The mutation is caught, by
    /// `test_litter_pool_cn_is_TWO_regimes_...`, which drives the same scenario through
    /// `run_perennial` and trips the engine's own conservation gate — so the claim was
    /// held structurally, inside a test named for something else.) The file's own helper
    /// carries a warning that `resets` "is not a knob — it is a property of the scenario";
    /// the correction was applied to the helper and not to this test.
    ///
    /// So this successor asserts the SPLIT, not just the total: the total is what both
    /// readings agree on.
    /// Mirrors `test_nitrogen_is_conserved_across_the_annual_reset`, widened.
    #[test]
    fn the_resow_splits_the_parents_nitrogen_by_concentration_and_is_not_a_windfall() {
        let scenario = perennial_chamber_scenario();
        let (state, _integrator, _resolver) = super::super::season_setup(&scenario, 1).unwrap();
        let seedling_total = scenario.leaf_c0 + scenario.stem_c0 + scenario.root_c0;

        // A grown crop with a seed bank, and a nitrogen pool that is not a round number.
        let mut stocks = state.stocks.clone();
        for (id, amount) in [
            (LEAF_C, 4.0),
            (STEM_C, 2.0),
            (ROOT_C, 2.0),
            (STORAGE_C, seedling_total + 1.0),
        ] {
            stocks.insert(id.to_string(), stocks[id].with_amount(amount).unwrap());
        }
        let old_plant_n = 0.2;
        stocks.insert(
            PLANT_N.to_string(),
            stocks[PLANT_N].with_amount(old_plant_n).unwrap(),
        );
        let litter_n0 = stocks[LITTER_N].amount;
        let before = State::new(state.n, stocks, state.rng_seed, state.aux.clone()).unwrap();

        let after = annual_reset(&before, &scenario).expect("re-sow");

        // (a) THE SPLIT: the seedling inherits the parent's tissue concentration.
        // old_veg = 4 + 2 + 2 = 8 mol C, so conc = 0.2 / 8 = 0.025 kg N per mol C.
        let conc = old_plant_n / 8.0;
        assert_eq!(conc, 0.025);
        let want_seedling = conc * seedling_total;
        assert!(
            (after.stocks[PLANT_N].amount - want_seedling).abs() <= 1e-15 * want_seedling,
            "seedling holds {} against {want_seedling}",
            after.stocks[PLANT_N].amount
        );
        // (b) The remainder is the balancing residual into litter.
        let gained = after.stocks[LITTER_N].amount - litter_n0;
        assert!(
            (gained - (old_plant_n - want_seedling)).abs() <= 1e-15 * gained,
            "litter gained {gained}"
        );
        // (c) ...and only THEN the total, which is what a windfall also satisfies.
        assert!(
            (after.stocks[PLANT_N].amount + after.stocks[LITTER_N].amount
                - (old_plant_n + litter_n0))
                .abs()
                <= 1e-15 * old_plant_n,
            "nitrogen must be conserved across the reset"
        );
        // (d) The fixture can tell the two readings apart: the seedling is a small
        // fraction of the parent, so a windfall would be an order of magnitude more.
        assert!(
            old_plant_n > 10.0 * after.stocks[PLANT_N].amount,
            "the fixture cannot distinguish the split from the windfall"
        );
    }

    /// Only the open field leaves Greenwood's plateau — the roster fact the FORM rests on.
    ///
    /// Six of the seven frozen scenarios are carbon-limited chambers that peak an order of
    /// magnitude below the curve's 1 t/ha domain bound, so they run entirely on the flat
    /// branch and never see the power law at all. That is why the PLATEAU reading — the
    /// primary's own statement that %N is constant while growth is exponential — is what
    /// decided the form, and why extrapolating the declining branch downward would have
    /// manufactured a season-long N decline for every chamber in the tree.
    ///
    /// ⚠⚠ WHAT THIS TEST IS REACHED BY, measured and stated rather than assumed. The
    /// load-bearing half is `w < bound` — a claim about which SCENARIOS exist, not about a
    /// rate law, so no mutation of `target_n_concentration` can move it and none did:
    /// removing the plateau outright (E1), the mechanism this test is named for, left it
    /// GREEN. The one mutation that reddens it is E4, `f_N` reading an absolute amount
    /// instead of a concentration — which has nothing to do with the plateau and reddens it
    /// only by moving the trajectory until a chamber's peak crosses the margin below. That
    /// is "a number moved" wearing a reassuring name, inside this test rather than outside
    /// it, and it is recorded here because the batch's own method rejects that reading
    /// everywhere else. The margin is a characterization pin; the roster claim is the gate.
    ///
    /// ⚠ It is asserted rather than described because it is a claim about the ROSTER, and
    /// this repo has been bitten more than once by a scope claim outliving the roster that
    /// made it true. A new chamber scenario that grew past 1 t/ha would move onto a branch
    /// this reasoning assumes it never reaches, and nothing else would say so.
    /// Mirrors `test_only_open_season_enters_the_declining_branch`.
    #[test]
    fn only_the_open_field_crop_leaves_greenwoods_plateau() {
        let bound = params::nitrogen().n_target_w_plateau;
        let fold = params::nitrogen().dm_kg_per_mol_c;
        let peak_w = |scenario: &SeasonScenario, years: usize, perennial: bool| -> f64 {
            let (state, integrator, resolver) =
                super::super::season_setup(scenario, years).unwrap();
            let mut peak = f64::NEG_INFINITY;
            let mut observe = |s: &State| {
                let w =
                    s.stocks[LEAF_C].amount + s.stocks[STEM_C].amount + s.stocks[STORAGE_C].amount;
                peak = peak.max((w * fold / scenario.ground_area) * 10.0);
            };
            let steps = super::super::steps_for_years(years);
            if perennial {
                run_perennial(
                    &integrator,
                    state,
                    scenario,
                    &resolver,
                    BIO_DT,
                    steps,
                    super::super::season_steps(),
                    &mut observe,
                )
                .expect("perennial run");
            } else {
                run_season(
                    &integrator,
                    state,
                    &resolver,
                    BIO_DT,
                    steps,
                    None,
                    &mut observe,
                )
                .expect("season run");
            }
            peak
        };

        for (name, scenario, years, perennial) in [
            (
                "sealed",
                sealed_chamber_scenario(),
                SEALED_CHAMBER_YEARS,
                false,
            ),
            (
                "perennial",
                perennial_chamber_scenario(),
                PERENNIAL_CHAMBER_YEARS,
                true,
            ),
            (
                "consumer",
                consumer_chamber_scenario(),
                CONSUMER_CHAMBER_YEARS,
                true,
            ),
        ] {
            let w = peak_w(&scenario, years, perennial);
            assert!(
                w < bound,
                "{name} peaked at {w} t/ha, past the domain bound"
            );
            // ...and not by a whisker. Measured on this tree: sealed 0.37688,
            // perennial 0.32108, consumer 0.34375 t/ha — a ~2.6x margin under the bound.
            // ⚠ The Python docstring this is ported from quotes "0.09-0.63 t/ha", which is
            // a different tree's numbers; the RANGE is re-measured here rather than
            // carried over, because a ported band is a claim about the port's own run.
            assert!(w < 0.5 * bound, "{name} peaked at {w} t/ha");
        }
        // The control, and the half that makes the claim about the ROSTER rather than
        // about chambers in general: the open field DOES enter the declining branch.
        let open = peak_w(&DEFAULT_SCENARIO, 1, false);
        assert!(
            open > bound,
            "the open field must reach the power law: {open} t/ha"
        );
    }
}
