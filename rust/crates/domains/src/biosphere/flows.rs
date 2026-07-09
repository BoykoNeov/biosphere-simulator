//! The biosphere flows + the coupled carbon budget + the thermal-time aux — the Rust
//! port of the flow classes across `domains.biosphere.*` (Phase-7 P7.4).
//!
//! Every `evaluate` mirrors the Python arithmetic and **leg-emission order**
//! character-for-character (the reduction sums `co2_atmos` across Allocation/
//! GrowthRespiration/MaintenanceRespiration in flow-id × leg order, so leg order is
//! load-bearing). The `MaintenanceRespiration` shortfall loop walks the fixed
//! `(leaf, stem, root)` tuple with a running `respired`/`organ_burn` accumulation — that
//! literal order (not sorted/map order) is preserved.

use std::collections::BTreeMap;

use simcore::auxiliary::AuxProcess;
use simcore::environment::Environment;
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::state::State;

use super::params::{
    CanopyParams, NitrogenParams, PartitionRow, PhenologyParams, PhotosynthesisParams,
    RespirationParams,
};
use super::science;

/// Read a stock amount from the snapshot (a missing id is a build bug, like Python's
/// `snapshot.stocks[id]` KeyError).
fn amt(s: &State, id: &str) -> f64 {
    s.stocks[id].amount
}

fn leg(id: &str, amount: f64) -> Result<Leg, SimError> {
    Leg::new(id.to_string(), amount)
}

// --- the shared carbon budget (CarbonContext) -------------------------------

/// Shared inputs for the recomputed daily carbon budget `(GASS, MRES, available)`.
/// Held (cloned) by the three budget-coupled flows so they cannot drift.
#[derive(Debug, Clone)]
pub struct CarbonContext {
    pub leaf_c: String,
    pub stem_c: String,
    pub root_c: String,
    pub par_var: String,
    pub ci_var: String,
    pub temp_var: String,
    pub daylength_var: String,
    pub soil_water_var: String,
    pub sw_wilting: f64,
    pub sw_critical: f64,
    pub plant_n: String,
    pub photo: PhotosynthesisParams,
    pub canopy: CanopyParams,
    pub resp: RespirationParams,
    pub nitro: NitrogenParams,
    pub ground_area: f64,
    /// Sealed-chamber Ci source (all-or-nothing with `chamber_air_mol`/`ci_ratio`).
    pub co2_pool_var: Option<String>,
    pub chamber_air_mol: Option<f64>,
    pub ci_ratio: Option<f64>,
}

impl CarbonContext {
    fn ci(&self, env: &dyn Environment) -> Result<f64, SimError> {
        match &self.co2_pool_var {
            None => env.get(&self.ci_var),
            Some(var) => {
                let air_mol = self.chamber_air_mol.expect("sealed ctx has air_mol");
                let ci_ratio = self.ci_ratio.expect("sealed ctx has ci_ratio");
                Ok(science::ci_from_co2_pool(env.get(var)?, air_mol, ci_ratio))
            }
        }
    }

    /// `(leaf_carbon, Σ(leaf + stem + root))`.
    fn leaf_and_biomass(&self, snapshot: &State) -> (f64, f64) {
        let leaf = amt(snapshot, &self.leaf_c);
        let biomass = leaf + amt(snapshot, &self.stem_c) + amt(snapshot, &self.root_c);
        (leaf, biomass)
    }

    fn limitation(&self, snapshot: &State, env: &dyn Environment) -> Result<f64, SimError> {
        let soil_water = env.get(&self.soil_water_var)?;
        let f_water = science::water_stress_factor(soil_water, self.sw_wilting, self.sw_critical);
        let (_, biomass) = self.leaf_and_biomass(snapshot);
        let plant_n = amt(snapshot, &self.plant_n);
        let f_n = science::nitrogen_stress_factor(
            plant_n,
            biomass,
            self.nitro.n_residual_per_mol_c,
            self.nitro.n_critical_per_mol_c,
        );
        Ok(f_water * f_n)
    }

    /// Daily `(GASS, MRES, available)` at the step-entry snapshot.
    fn budget(&self, snapshot: &State, env: &dyn Environment) -> Result<(f64, f64, f64), SimError> {
        let (leaf, biomass) = self.leaf_and_biomass(snapshot);
        let lai = science::leaf_area_index(leaf, self.canopy.sla_per_mol_c, self.ground_area);
        let gass = science::daily_canopy_assimilation(
            env.get(&self.par_var)?,
            lai,
            self.ci(env)?,
            env.get(&self.temp_var)?,
            env.get(&self.daylength_var)?,
            &self.photo,
            &self.canopy,
            self.ground_area,
            self.limitation(snapshot, env)?,
        );
        let mres = science::maintenance_respiration_flux(biomass, env.get(&self.temp_var)?, &self.resp);
        Ok((gass, mres, science::available_for_growth(gass, mres)))
    }
}

// --- the carbon-budget flows ------------------------------------------------

/// CARBON growth `co2_atmos -> {leaf,stem,root,storage}` (+ O₂ leg when sealed).
pub struct Allocation {
    pub id: String,
    pub ctx: CarbonContext,
    pub co2_atmos: String,
    pub storage_c: String,
    pub thermal_time_aux: String,
    pub pheno: PhenologyParams,
    pub table: Vec<PartitionRow>,
    pub o2_pool: Option<String>,
}

impl Flow for Allocation {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, snapshot: &State, env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let (_, _, available) = self.ctx.budget(snapshot, env)?;
        let dmi = self.ctx.resp.growth_efficiency * available;
        let thermal_time = snapshot.aux.get(&self.thermal_time_aux).copied().unwrap_or(0.0);
        let dvs = science::development_stage(thermal_time, self.pheno.tsum_anthesis, self.pheno.tsum_maturity);
        let (leaf, stem, root, storage) = science::partition(dmi, dvs, &self.table);
        let leaf_leg = leaf * dt;
        let stem_leg = stem * dt;
        let root_leg = root * dt;
        let storage_leg = storage * dt;
        let organ_total = leaf_leg + stem_leg + root_leg + storage_leg;
        let mut legs = vec![
            leg(&self.co2_atmos, -organ_total)?,
            leg(&self.ctx.leaf_c, leaf_leg)?,
            leg(&self.ctx.stem_c, stem_leg)?,
            leg(&self.ctx.root_c, root_leg)?,
            leg(&self.storage_c, storage_leg)?,
        ];
        if let Some(o2) = &self.o2_pool {
            legs.push(leg(o2, organ_total)?);
        }
        FlowResult::new(legs)
    }
}

/// CARBON growth-conversion loss `co2_atmos -> co2_resp` (empty when source == sink).
pub struct GrowthRespiration {
    pub id: String,
    pub ctx: CarbonContext,
    pub co2_atmos: String,
    pub co2_resp: String,
}

impl Flow for GrowthRespiration {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, snapshot: &State, env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        if self.co2_atmos == self.co2_resp {
            return Ok(FlowResult::empty());
        }
        let (_, _, available) = self.ctx.budget(snapshot, env)?;
        let gres = (1.0 - self.ctx.resp.growth_efficiency) * available;
        let flux = gres * dt;
        FlowResult::new(vec![leg(&self.co2_atmos, -flux)?, leg(&self.co2_resp, flux)?])
    }
}

/// CARBON maintenance `{co2_atmos(covered), organs(shortfall)} -> co2_resp` (+ O₂ sealed).
pub struct MaintenanceRespiration {
    pub id: String,
    pub ctx: CarbonContext,
    pub co2_atmos: String,
    pub co2_resp: String,
    pub o2_pool: Option<String>,
    pub air_mol: Option<f64>,
}

impl Flow for MaintenanceRespiration {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, snapshot: &State, env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let (gass, mres, _) = self.ctx.budget(snapshot, env)?;
        let (leaf, biomass) = self.ctx.leaf_and_biomass(snapshot);
        let covered = gass.min(mres);
        let shortfall = mres - covered; // == max(0, MRES − GASS)
        let covered_flux = covered * dt;
        if self.co2_atmos == self.co2_resp {
            // Sealed chamber: covered is a CO₂→CO₂ round trip (dropped); only the
            // biomass-burned shortfall is a real respiration, O₂-throttled by f_O2.
            let mut f_o2 = 1.0;
            if let Some(o2) = &self.o2_pool {
                let air_mol = self.air_mol.expect("sealed MaintenanceRespiration has air_mol");
                f_o2 = science::oxygen_limitation_factor(amt(snapshot, o2), air_mol, self.ctx.resp.o2_half_saturation);
            }
            let mut legs: Vec<Leg> = Vec::new();
            let mut organ_burn = 0.0;
            if biomass > 0.0 && shortfall > 0.0 {
                let stem = amt(snapshot, &self.ctx.stem_c);
                let root = amt(snapshot, &self.ctx.root_c);
                for (organ_id, organ_c) in [
                    (&self.ctx.leaf_c, leaf),
                    (&self.ctx.stem_c, stem),
                    (&self.ctx.root_c, root),
                ] {
                    let share = f_o2 * shortfall * (organ_c / biomass) * dt;
                    legs.push(leg(organ_id, -share)?);
                    organ_burn += share;
                }
            }
            if organ_burn != 0.0 {
                legs.push(leg(&self.co2_resp, organ_burn)?);
                if let Some(o2) = &self.o2_pool {
                    legs.push(leg(o2, -organ_burn)?);
                }
            }
            return FlowResult::new(legs);
        }
        // Open field: covered from the atmosphere, shortfall from the organs.
        let mut legs = vec![leg(&self.co2_atmos, -covered_flux)?];
        let mut respired = covered_flux;
        if biomass > 0.0 && shortfall > 0.0 {
            let stem = amt(snapshot, &self.ctx.stem_c);
            let root = amt(snapshot, &self.ctx.root_c);
            for (organ_id, organ_c) in [
                (&self.ctx.leaf_c, leaf),
                (&self.ctx.stem_c, stem),
                (&self.ctx.root_c, root),
            ] {
                let share = shortfall * (organ_c / biomass) * dt;
                legs.push(leg(organ_id, -share)?);
                respired += share;
            }
        }
        legs.push(leg(&self.co2_resp, respired)?);
        FlowResult::new(legs)
    }
}

// --- senescence / transpiration / uptake (plants) ---------------------------

/// CARBON loss `{leaf,stem,root} -> litter_sink`.
pub struct Senescence {
    pub id: String,
    pub leaf_c: String,
    pub stem_c: String,
    pub root_c: String,
    pub litter_sink: String,
    pub rdr_leaf: f64,
    pub rdr_stem: f64,
    pub rdr_root: f64,
}

impl Flow for Senescence {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, snapshot: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let leaf = self.rdr_leaf * amt(snapshot, &self.leaf_c) * dt;
        let stem = self.rdr_stem * amt(snapshot, &self.stem_c) * dt;
        let root = self.rdr_root * amt(snapshot, &self.root_c) * dt;
        FlowResult::new(vec![
            leg(&self.leaf_c, -leaf)?,
            leg(&self.stem_c, -stem)?,
            leg(&self.root_c, -root)?,
            leg(&self.litter_sink, leaf + stem + root)?,
        ])
    }
}

/// WATER `soil_water -> vapor_sink` (Penman–Monteith · f_water).
pub struct Transpiration {
    pub id: String,
    pub soil_water: String,
    pub vapor_sink: String,
    pub rn_var: String,
    pub vpd_var: String,
    pub temp_var: String,
    pub aerodynamic_resistance: f64,
    pub surface_resistance: f64,
    pub ground_area: f64,
    pub sw_wilting: f64,
    pub sw_critical: f64,
}

impl Flow for Transpiration {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, snapshot: &State, env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let net_radiation = env.get(&self.rn_var)?;
        let vpd = env.get(&self.vpd_var)?;
        let temp_c = env.get(&self.temp_var)?;
        let soil_water = amt(snapshot, &self.soil_water);
        let potential = science::penman_monteith_transpiration(
            net_radiation,
            vpd,
            temp_c,
            self.aerodynamic_resistance,
            self.surface_resistance,
        );
        let f_water = science::water_stress_factor(soil_water, self.sw_wilting, self.sw_critical);
        let daily_kg = potential * f_water * self.ground_area;
        let flux = daily_kg * dt;
        FlowResult::new(vec![leg(&self.soil_water, -flux)?, leg(&self.vapor_sink, flux)?])
    }
}

/// WATER `water_source -> soil_water` (scheduled irrigation).
pub struct Irrigation {
    pub id: String,
    pub water_source: String,
    pub soil_water: String,
    pub irrigation_var: String,
    pub ground_area: f64,
}

impl Flow for Irrigation {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, _snapshot: &State, env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let rate_mm_day = env.get(&self.irrigation_var)?;
        let daily_kg = rate_mm_day * self.ground_area;
        let flux = daily_kg * dt;
        FlowResult::new(vec![leg(&self.water_source, -flux)?, leg(&self.soil_water, flux)?])
    }
}

/// NITROGEN `soil_n -> plant_n` (capacity-gated uptake).
pub struct NitrogenUptake {
    pub id: String,
    pub soil_n: String,
    pub plant_n: String,
    pub max_uptake_capacity: f64,
    pub ground_area: f64,
    pub sn_residual: f64,
    pub sn_critical: f64,
}

impl Flow for NitrogenUptake {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, snapshot: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let soil_n = amt(snapshot, &self.soil_n);
        let availability = science::soil_n_availability(soil_n, self.sn_residual, self.sn_critical);
        let daily_kg = self.max_uptake_capacity * self.ground_area * availability;
        let flux = daily_kg * dt;
        FlowResult::new(vec![leg(&self.soil_n, -flux)?, leg(&self.plant_n, flux)?])
    }
}

/// NITROGEN `n_source -> soil_n` (scheduled fertilization).
pub struct Fertilization {
    pub id: String,
    pub n_source: String,
    pub soil_n: String,
    pub fertilization_var: String,
    pub ground_area: f64,
}

impl Flow for Fertilization {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, _snapshot: &State, env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let rate = env.get(&self.fertilization_var)?;
        let daily_kg = rate * self.ground_area;
        let flux = daily_kg * dt;
        FlowResult::new(vec![leg(&self.n_source, -flux)?, leg(&self.soil_n, flux)?])
    }
}

// --- decomposer / nitrogen return / water cycle / consumer ------------------

/// CARBON decay `litter_carbon -> microbial_carbon`.
pub struct Decomposition {
    pub id: String,
    pub litter_carbon: String,
    pub microbial_carbon: String,
    pub decomposition_rate: f64,
}

impl Flow for Decomposition {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, snapshot: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let decayed = self.decomposition_rate * amt(snapshot, &self.litter_carbon) * dt;
        FlowResult::new(vec![
            leg(&self.litter_carbon, -decayed)?,
            leg(&self.microbial_carbon, decayed)?,
        ])
    }
}

/// CARBON+OXYGEN `microbial_carbon + o2_pool -> carbon_pool` (f_O2-throttled).
pub struct MicrobialRespiration {
    pub id: String,
    pub microbial_carbon: String,
    pub co2_pool: String,
    pub o2_pool: String,
    pub microbial_respiration_rate: f64,
    pub o2_half_saturation: f64,
    pub air_mol: f64,
}

impl Flow for MicrobialRespiration {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, snapshot: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let f_o2 = science::oxygen_limitation_factor(amt(snapshot, &self.o2_pool), self.air_mol, self.o2_half_saturation);
        let respired = self.microbial_respiration_rate * amt(snapshot, &self.microbial_carbon) * f_o2 * dt;
        FlowResult::new(vec![
            leg(&self.microbial_carbon, -respired)?,
            leg(&self.co2_pool, respired)?,
            leg(&self.o2_pool, -respired)?,
        ])
    }
}

/// NITROGEN `plant_n -> litter_n`.
pub struct NitrogenSenescence {
    pub id: String,
    pub plant_n: String,
    pub litter_n: String,
    pub n_senescence_rate: f64,
}

impl Flow for NitrogenSenescence {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, snapshot: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let shed = self.n_senescence_rate * amt(snapshot, &self.plant_n) * dt;
        FlowResult::new(vec![leg(&self.plant_n, -shed)?, leg(&self.litter_n, shed)?])
    }
}

/// NITROGEN `litter_n -> soil_n`.
pub struct Mineralization {
    pub id: String,
    pub litter_n: String,
    pub soil_n: String,
    pub mineralization_rate: f64,
}

impl Flow for Mineralization {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, snapshot: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let mineralized = self.mineralization_rate * amt(snapshot, &self.litter_n) * dt;
        FlowResult::new(vec![leg(&self.litter_n, -mineralized)?, leg(&self.soil_n, mineralized)?])
    }
}

/// WATER `water_vapor -> condensate`.
pub struct Condensation {
    pub id: String,
    pub water_vapor: String,
    pub condensate: String,
    pub condensation_rate: f64,
}

impl Flow for Condensation {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, snapshot: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let condensed = self.condensation_rate * amt(snapshot, &self.water_vapor) * dt;
        FlowResult::new(vec![leg(&self.water_vapor, -condensed)?, leg(&self.condensate, condensed)?])
    }
}

/// WATER `condensate -> soil_water`.
pub struct Recycling {
    pub id: String,
    pub condensate: String,
    pub soil_water: String,
    pub recycling_rate: f64,
}

impl Flow for Recycling {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, snapshot: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let recycled = self.recycling_rate * amt(snapshot, &self.condensate) * dt;
        FlowResult::new(vec![leg(&self.condensate, -recycled)?, leg(&self.soil_water, recycled)?])
    }
}

/// CARBON `leaf_c -> consumer_carbon`.
pub struct Grazing {
    pub id: String,
    pub leaf_c: String,
    pub consumer_carbon: String,
    pub grazing_rate: f64,
}

impl Flow for Grazing {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, snapshot: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let grazed = self.grazing_rate * amt(snapshot, &self.leaf_c) * dt;
        FlowResult::new(vec![leg(&self.leaf_c, -grazed)?, leg(&self.consumer_carbon, grazed)?])
    }
}

/// CARBON+OXYGEN `consumer_carbon + o2_pool -> carbon_pool` (f_O2-throttled).
pub struct ConsumerRespiration {
    pub id: String,
    pub consumer_carbon: String,
    pub co2_pool: String,
    pub o2_pool: String,
    pub respiration_rate: f64,
    pub o2_half_saturation: f64,
    pub air_mol: f64,
}

impl Flow for ConsumerRespiration {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, snapshot: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let f_o2 = science::oxygen_limitation_factor(amt(snapshot, &self.o2_pool), self.air_mol, self.o2_half_saturation);
        let respired = self.respiration_rate * amt(snapshot, &self.consumer_carbon) * f_o2 * dt;
        FlowResult::new(vec![
            leg(&self.consumer_carbon, -respired)?,
            leg(&self.co2_pool, respired)?,
            leg(&self.o2_pool, -respired)?,
        ])
    }
}

/// CARBON `consumer_carbon -> litter_carbon`.
pub struct ConsumerMortality {
    pub id: String,
    pub consumer_carbon: String,
    pub litter_carbon: String,
    pub mortality_rate: f64,
}

impl Flow for ConsumerMortality {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, snapshot: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let died = self.mortality_rate * amt(snapshot, &self.consumer_carbon) * dt;
        FlowResult::new(vec![leg(&self.consumer_carbon, -died)?, leg(&self.litter_carbon, died)?])
    }
}

// --- the thermal-time aux ---------------------------------------------------

/// `AuxProcess` advancing the `thermal_time` accumulator.
pub struct ThermalTimeAccumulation {
    pub id: String,
    pub accumulator: String,
    pub temp_var: String,
    pub t_base: f64,
    pub t_cap: f64,
}

impl AuxProcess for ThermalTimeAccumulation {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, _snapshot: &State, env: &dyn Environment, dt: f64) -> Result<BTreeMap<String, f64>, SimError> {
        let temp_c = env.get(&self.temp_var)?;
        let rate = science::daily_thermal_time(temp_c, self.t_base, self.t_cap);
        Ok(BTreeMap::from([(self.accumulator.clone(), rate * dt)]))
    }
}
