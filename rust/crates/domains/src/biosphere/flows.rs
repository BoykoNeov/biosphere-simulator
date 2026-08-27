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

use super::light_path;
use super::params;
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
    // ⚠ No `daylength_var` since 2026-08-14: photosynthesis integrates over one day and
    // the day/night structure lives in the PAR forcing (`light_path`). `daylength_s`
    // survives as the phenology photoperiod signal only. Mirrors the Python field's
    // removal.
    pub soil_water_var: String,
    pub wssg: f64,
    pub rooted_depth_aux: String,
    pub soil_extractable_water: f64,
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
        // ⚠ The depth comes off the SNAPSHOT aux and must be the same step-entry depth
        // `Transpiration` and `extension_rate` read — this consumer reaches soil_water
        // through `env.get` while `Transpiration` reads the stock directly, so the two
        // could silently disagree about FTSW inside one step.
        let f_water = science::soil_water_stress(
            soil_water,
            snapshot
                .aux
                .get(&self.rooted_depth_aux)
                .copied()
                .unwrap_or(0.0),
            self.soil_extractable_water,
            self.ground_area,
            self.wssg,
        );
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
        let gass = science::canopy_assimilation(
            env.get(&self.par_var)?,
            lai,
            self.ci(env)?,
            env.get(&self.temp_var)?,
            light_path::SECONDS_PER_DAY,
            &self.photo,
            &self.canopy,
            self.ground_area,
            self.limitation(snapshot, env)?,
        );
        let mres =
            science::maintenance_respiration_flux(biomass, env.get(&self.temp_var)?, &self.resp);
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
    /// Stem-reserve FORMATION: `fstr` of this flow's OWN stem leg is deposited as
    /// shielded starch instead of structural stem ([E] SS3.2.4 p. 93, Listing 3 L17).
    ///
    /// It lives inside the partition split rather than in a flow of its own, and that
    /// is CORRECTNESS, not tidiness: a separate flow would have to WITHDRAW from
    /// `stem_c`, and arbitration scales withdrawals against the START-OF-STEP amount, so
    /// at emergence - where the day's stem growth can exceed the whole seedling stem -
    /// it would ration. Splitting the deposit cannot: `organ_total` is unchanged, so the
    /// CO2 and O2 legs never move.
    ///
    /// All three fields are the inert defaults (`None` / 0.0 / 0.0) for a crop with no
    /// reserve, and 0.0 is inert for the cessation too (DVS >= 0 always), so a wiring
    /// that supplies a stock and a fraction but forgets the bound gets NO split rather
    /// than an unbounded one - fail-closed, as in the Python.
    pub stem_reserve_c: Option<String>,
    pub fstr: f64,
    pub reserve_cessation_dvs: f64,
}

impl Flow for Allocation {
    fn type_name(&self) -> &'static str {
        "Allocation"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let (_, _, available) = self.ctx.budget(snapshot, env)?;
        let dmi = self.ctx.resp.growth_efficiency * available;
        let thermal_time = snapshot
            .aux
            .get(&self.thermal_time_aux)
            .copied()
            .unwrap_or(0.0);
        let dvs = science::development_stage(
            thermal_time,
            self.pheno.tsum_anthesis,
            self.pheno.tsum_maturity,
        );
        let (leaf, stem, root, storage) = science::partition(dmi, dvs, &self.table);
        let leaf_leg = leaf * dt;
        let stem_leg = stem * dt;
        let root_leg = root * dt;
        let storage_leg = storage * dt;
        let organ_total = leaf_leg + stem_leg + root_leg + storage_leg;
        // `organ_total` is formed from the WHOLE stem leg, before any reserve split: the
        // diverted starch is still carbon fixed out of the atmosphere into the plant, so
        // the CO2 source leg and the O2 release must not move.
        let mut legs = vec![
            leg(&self.co2_atmos, -organ_total)?,
            leg(&self.ctx.leaf_c, leaf_leg)?,
        ];
        match &self.stem_reserve_c {
            Some(reserve) if self.fstr != 0.0 && dvs < self.reserve_cessation_dvs => {
                let diverted = self.fstr * stem_leg;
                legs.push(leg(&self.ctx.stem_c, stem_leg - diverted)?);
                legs.push(leg(reserve, diverted)?);
            }
            _ => legs.push(leg(&self.ctx.stem_c, stem_leg)?),
        }
        legs.push(leg(&self.ctx.root_c, root_leg)?);
        legs.push(leg(&self.storage_c, storage_leg)?);
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
    fn type_name(&self) -> &'static str {
        "GrowthRespiration"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        if self.co2_atmos == self.co2_resp {
            return Ok(FlowResult::empty());
        }
        let (_, _, available) = self.ctx.budget(snapshot, env)?;
        let gres = (1.0 - self.ctx.resp.growth_efficiency) * available;
        let flux = gres * dt;
        FlowResult::new(vec![
            leg(&self.co2_atmos, -flux)?,
            leg(&self.co2_resp, flux)?,
        ])
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
    fn type_name(&self) -> &'static str {
        "MaintenanceRespiration"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
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
                let air_mol = self
                    .air_mol
                    .expect("sealed MaintenanceRespiration has air_mol");
                f_o2 = science::oxygen_limitation_factor(
                    amt(snapshot, o2),
                    air_mol,
                    self.ctx.resp.o2_half_saturation,
                );
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

// --- stem-reserve remobilization (the stem feeding the grain) ---------------

/// CARBON `stem_reserve_c -> storage_c` on `trigger <= DVS < cessation` (sum legs = 0).
///
/// Mirrors `domains.biosphere.stem_reserves.StemRemobilization`. First-order on the
/// standing reserve, so the draw is donor-controlled and therefore self-limiting: the
/// Euler arbitration backstop is structurally unreachable on it.
///
/// Outside the window the flow emits NO LEGS AT ALL rather than a zero one - the tree's
/// idiom for "this flow does not act today".
///
/// The window is half-open at BOTH ends. The upper end is [E]'s `FINISH DS = 2.`
/// (Listing 3 Line 114) and it is STRICT, which is load-bearing rather than stylistic:
/// our DVS *caps* at 2.0 instead of growing past it, so `<=` would leave the drain
/// running for the whole post-maturity tail (11 steps on `open_season`, two YEARS on
/// `sealed_chamber`, which never re-sows) - exactly what the cessation exists to stop.
///
/// The window reads DVS off the step-entry snapshot's thermal-time accumulator, the same
/// read `Allocation` makes, and shares its upper bound with `Allocation`'s
/// `reserve_cessation_dvs` so the fill and the drain stop on the same step.
pub struct StemRemobilization {
    pub id: String,
    pub stem_reserve_c: String,
    pub storage_c: String,
    pub thermal_time_aux: String,
    pub pheno: params::PhenologyParams,
    pub params: params::StemReserveParams,
}

impl Flow for StemRemobilization {
    fn type_name(&self) -> &'static str {
        "StemRemobilization"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let dvs = science::development_stage(
            snapshot
                .aux
                .get(&self.thermal_time_aux)
                .copied()
                .unwrap_or(0.0),
            self.pheno.tsum_anthesis,
            self.pheno.tsum_maturity,
        );
        if dvs < self.params.trigger_dvs || dvs >= self.params.cessation_dvs {
            return FlowResult::new(vec![]);
        }
        // Association is load-bearing: float multiplication is not associative, so
        // `(rate*reserve)*dt` and `rate*(reserve*dt)` can differ in the last bit. This is
        // the grouping the Python reference uses and the goldens were produced from.
        let reserve = amt(snapshot, &self.stem_reserve_c);
        let flux = self.params.remobilization_rate * reserve * dt;
        if flux == 0.0 {
            return FlowResult::new(vec![]);
        }
        FlowResult::new(vec![
            leg(&self.stem_reserve_c, -flux)?,
            leg(&self.storage_c, flux)?,
        ])
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
    /// Mutual shading (Van Keulen & Seligman via Penning de Vries p. 101).
    pub shade_rate: f64,
    pub lai_threshold: f64,
    pub sla_per_mol_c: f64,
    pub ground_area: f64,
}

impl Flow for Senescence {
    fn type_name(&self) -> &'static str {
        "Senescence"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let leaf_c = amt(snapshot, &self.leaf_c);
        let lai = leaf_c * self.sla_per_mol_c / self.ground_area;
        let rdr_leaf = crate::biosphere::science::mutual_shading_rate(
            lai,
            self.rdr_leaf,
            self.shade_rate,
            self.lai_threshold,
        );
        let leaf = rdr_leaf * leaf_c * dt;
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
    pub rooted_depth_aux: String,
    pub soil_extractable_water: f64,
    pub wssg: f64,
}

impl Flow for Transpiration {
    fn type_name(&self) -> &'static str {
        "Transpiration"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
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
        let f_water = science::soil_water_stress(
            soil_water,
            snapshot
                .aux
                .get(&self.rooted_depth_aux)
                .copied()
                .unwrap_or(0.0),
            self.soil_extractable_water,
            self.ground_area,
            self.wssg,
        );
        let daily_kg = potential * f_water * self.ground_area;
        let flux = daily_kg * dt;
        FlowResult::new(vec![
            leg(&self.soil_water, -flux)?,
            leg(&self.vapor_sink, flux)?,
        ])
    }
}

/// WATER `water_source -> soil_water` — **demand-driven, capacity-capped**.
///
/// `IRGW = min(capacity · ground_area · dt, max(0, TTSW − ATSW))` — [F] Eqn 14.8
/// composed with [F]'s own "a fixed amount ... defined by the capacity of the irrigation
/// system". ⚠ The forcing changed meaning on 2026-08-12: mm/day **applied** → mm/day
/// **available**. A zero is still a hard off, so an irrigation-cut window is unaffected.
pub struct Irrigation {
    pub id: String,
    pub water_source: String,
    pub soil_water: String,
    pub irrigation_var: String,
    pub ground_area: f64,
    pub rooted_depth_aux: String,
    pub soil_extractable_water: f64,
}

impl Flow for Irrigation {
    fn type_name(&self) -> &'static str {
        "Irrigation"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let capacity_kg = env.get(&self.irrigation_var)? * self.ground_area * dt;
        let deficit = science::transpirable_capacity(
            snapshot
                .aux
                .get(&self.rooted_depth_aux)
                .copied()
                .unwrap_or(0.0),
            self.soil_extractable_water,
            self.ground_area,
        ) - amt(snapshot, &self.soil_water);
        let flux = if deficit <= 0.0 {
            0.0
        } else if capacity_kg < deficit {
            capacity_kg
        } else {
            deficit
        };
        FlowResult::new(vec![
            leg(&self.water_source, -flux)?,
            leg(&self.soil_water, flux)?,
        ])
    }
}

/// WATER `soil_water -> subsoil_water` (`DRAIN`; [F] Eqns 14.11 + 14.12).
///
/// The inverse of `RootZoneCapture`, and the mechanism that gives the root zone a
/// bottom. `DRAIN = (ATSW − TTSW) · DRAINF` when `ATSW > TTSW`, else 0, and the
/// destination is `WSTORG` — **not** a boundary: [F] 14.12 is
/// `WSTORG = WSTORG + DRAIN − EWAT`, so no boundary is crossed and conservation is
/// structural. `drainage_factor = 0.0` shuts it off exactly (the valve).
///
/// ⚠ **Bit-identically inert on every frozen scenario** — with irrigation demand-driven
/// the zone is never over-filled, so no golden protects this flow. Its Rust pins have to
/// CONSTRUCT an over-filled zone; `cargo test` passing is not parity.
pub struct Drainage {
    pub id: String,
    pub soil_water: String,
    pub subsoil_water: String,
    pub drainage_factor: f64,
    pub rooted_depth_aux: String,
    pub soil_extractable_water: f64,
    pub ground_area: f64,
}

impl Flow for Drainage {
    fn type_name(&self) -> &'static str {
        "Drainage"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let available = amt(snapshot, &self.soil_water);
        let excess = available
            - science::transpirable_capacity(
                snapshot
                    .aux
                    .get(&self.rooted_depth_aux)
                    .copied()
                    .unwrap_or(0.0),
                self.soil_extractable_water,
                self.ground_area,
            );
        if excess <= 0.0 {
            return FlowResult::new(vec![
                leg(&self.soil_water, -0.0)?,
                leg(&self.subsoil_water, 0.0)?,
            ]);
        }
        let mut flux = excess * self.drainage_factor * dt;
        // Donor clamp, the sibling of Eqn 14.10's. It cannot bite while DRAINF <= 1, but
        // a scenario is free to declare more, and the arbitration backstop must not be
        // what catches it — every golden asserts that backstop fires zero times.
        if flux > available {
            flux = available;
        }
        FlowResult::new(vec![
            leg(&self.soil_water, -flux)?,
            leg(&self.subsoil_water, flux)?,
        ])
    }
}

/// NITROGEN `soil_n -> plant_n` (DEMAND-DEFICIT uptake, supply-gated).
///
/// `flux = min(target * biomass_c - plant_n, capacity * availability) * dt`. Greenwood's `W`
/// excludes fibrous roots (leaf+stem+storage); the deficit applies to f_N's own denominator
/// (leaf+stem+root). See `domains.biosphere.nitrogen.NitrogenUptake` for why those differ.
pub struct NitrogenUptake {
    pub id: String,
    pub soil_n: String,
    pub plant_n: String,
    pub leaf_c: String,
    pub stem_c: String,
    pub root_c: String,
    pub storage_c: String,
    pub max_uptake_capacity: f64,
    pub n_target_coefficient: f64,
    pub n_target_exponent: f64,
    pub n_target_w_plateau: f64,
    pub dm_kg_per_mol_c: f64,
    pub ground_area: f64,
    /// The root-functional-coupling gate: the accumulator naming rooted depth, and the
    /// reference layer it is measured against (scenario/soil data). Measured
    /// bit-identically inert on the frozen roster - see the Python root_depth.py.
    pub rooted_depth_aux: String,
    pub soil_layer_depth: f64,
    pub sn_residual: f64,
    pub sn_critical: f64,
}

impl Flow for NitrogenUptake {
    fn type_name(&self) -> &'static str {
        "NitrogenUptake"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let leaf = amt(snapshot, &self.leaf_c);
        let stem = amt(snapshot, &self.stem_c);
        let w_mol_c = leaf + stem + amt(snapshot, &self.storage_c);
        let biomass_c = leaf + stem + amt(snapshot, &self.root_c);
        // mol C -> kg DM -> t DM/ha (1 kg/m^2 == 10 t/ha)
        let w_t_ha = (w_mol_c * self.dm_kg_per_mol_c / self.ground_area) * 10.0;
        let target_per_mol_c = science::target_n_concentration(
            w_t_ha,
            self.n_target_coefficient,
            self.n_target_exponent,
            self.n_target_w_plateau,
        ) * self.dm_kg_per_mol_c;
        let deficit = (target_per_mol_c * biomass_c - amt(snapshot, &self.plant_n)).max(0.0);

        let availability = science::soil_n_availability(
            amt(snapshot, &self.soil_n),
            self.sn_residual,
            self.sn_critical,
        );
        let root_access = science::root_zone_fraction(
            snapshot
                .aux
                .get(&self.rooted_depth_aux)
                .copied()
                .unwrap_or(0.0),
            self.soil_layer_depth,
        );
        let capacity = self.max_uptake_capacity * self.ground_area * availability * root_access;
        let flux = deficit.min(capacity) * dt;
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
    fn type_name(&self) -> &'static str {
        "Fertilization"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        _snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let rate = env.get(&self.fertilization_var)?;
        let daily_kg = rate * self.ground_area;
        let flux = daily_kg * dt;
        FlowResult::new(vec![leg(&self.n_source, -flux)?, leg(&self.soil_n, flux)?])
    }
}

// --- decomposer / nitrogen return / water cycle / consumer ------------------

/// Split a decomposer carbon flux into `(respired, stabilized)` (mol C).
///
/// The single kernel all three humified flows share. The complement is computed by
/// SUBTRACTION, never as `moved * (1 - f)`, so the two destination legs sum back to the
/// withdrawal exactly in floating point and no partition round-off reaches the
/// conservation gate.
pub fn respired_and_stabilized(moved_c: f64, co2_fraction: f64) -> (f64, f64) {
    let respired = moved_c * co2_fraction;
    (respired, moved_c - respired)
}

/// CARBON+OXYGEN `litter_carbon + o2_pool -> carbon_pool + microbial_carbon`.
///
/// Single-currency CARBON until 2026-08-10 (the deliberate Phase-2 Step-4/5 split). The
/// humification split gives it a CO2 leg and the composition gate forces the O2 draw that
/// comes with it; the whole flux is `f_O2`-throttled, as `MicrobialRespiration` already
/// was, because aerobic decomposition IS the O2-consuming process.
pub struct Decomposition {
    pub id: String,
    pub litter_carbon: String,
    pub microbial_carbon: String,
    pub co2_pool: String,
    pub o2_pool: String,
    pub decomposition_rate: f64,
    pub litter_respired_fraction: f64,
    pub o2_half_saturation: f64,
    pub air_mol: f64,
}

impl Flow for Decomposition {
    fn type_name(&self) -> &'static str {
        "Decomposition"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let f_o2 = science::oxygen_limitation_factor(
            amt(snapshot, &self.o2_pool),
            self.air_mol,
            self.o2_half_saturation,
        );
        let decayed = self.decomposition_rate * amt(snapshot, &self.litter_carbon) * f_o2 * dt;
        let (respired, stabilized) =
            respired_and_stabilized(decayed, self.litter_respired_fraction);
        FlowResult::new(vec![
            leg(&self.litter_carbon, -decayed)?,
            leg(&self.microbial_carbon, stabilized)?,
            leg(&self.co2_pool, respired)?,
            leg(&self.o2_pool, -respired)?,
        ])
    }
}

/// CARBON+OXYGEN `humus_carbon + o2_pool -> carbon_pool + microbial_carbon`.
///
/// CENTURY's slow-SOM decomposition at `K6`, partitioned by `slow_respired_fraction`.
pub struct HumusDecomposition {
    pub id: String,
    pub humus_carbon: String,
    pub microbial_carbon: String,
    pub co2_pool: String,
    pub o2_pool: String,
    pub slow_decomposition_rate: f64,
    pub slow_respired_fraction: f64,
    pub o2_half_saturation: f64,
    pub air_mol: f64,
}

impl Flow for HumusDecomposition {
    fn type_name(&self) -> &'static str {
        "HumusDecomposition"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let f_o2 = science::oxygen_limitation_factor(
            amt(snapshot, &self.o2_pool),
            self.air_mol,
            self.o2_half_saturation,
        );
        let decayed = amt(snapshot, &self.humus_carbon) * self.slow_decomposition_rate * f_o2 * dt;
        let (respired, stabilized) = respired_and_stabilized(decayed, self.slow_respired_fraction);
        FlowResult::new(vec![
            leg(&self.humus_carbon, -decayed)?,
            leg(&self.microbial_carbon, stabilized)?,
            leg(&self.co2_pool, respired)?,
            leg(&self.o2_pool, -respired)?,
        ])
    }
}

/// CARBON+OXYGEN `microbial_carbon + o2_pool -> carbon_pool` (f_O2-throttled).
pub struct MicrobialRespiration {
    pub id: String,
    pub microbial_carbon: String,
    pub humus_carbon: String,
    pub co2_pool: String,
    pub o2_pool: String,
    pub microbial_respiration_rate: f64,
    pub active_stabilization_co2_fraction: f64,
    pub o2_half_saturation: f64,
    pub air_mol: f64,
}

impl Flow for MicrobialRespiration {
    fn type_name(&self) -> &'static str {
        "MicrobialRespiration"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let f_o2 = science::oxygen_limitation_factor(
            amt(snapshot, &self.o2_pool),
            self.air_mol,
            self.o2_half_saturation,
        );
        let turned =
            self.microbial_respiration_rate * amt(snapshot, &self.microbial_carbon) * f_o2 * dt;
        // `Es` of the turnover leaves as CO2; the rest is stabilised into slow SOM.
        let (respired, stabilized) =
            respired_and_stabilized(turned, self.active_stabilization_co2_fraction);
        FlowResult::new(vec![
            leg(&self.microbial_carbon, -turned)?,
            leg(&self.humus_carbon, stabilized)?,
            leg(&self.co2_pool, respired)?,
            leg(&self.o2_pool, -respired)?,
        ])
    }
}

/// NITROGEN `plant_n -> litter_n`, COUPLED to the senescing carbon.
///
/// `shed_N = min(plant_n/biomass_c, n_residual_per_mol_c) * shed_C`, where `shed_C` is the
/// identical per-organ flux `Senescence` sends to litter_carbon — recomputed here from the
/// same rates, since a flow may only read the step-entry snapshot. The `min` is
/// remobilization: a well-fed plant sheds only the residual concentration Van Hecke et al.
/// (2020) measure in mature straw, and retains the rest.
pub struct NitrogenSenescence {
    pub id: String,
    pub plant_n: String,
    pub litter_n: String,
    pub leaf_c: String,
    pub stem_c: String,
    pub root_c: String,
    pub rdr_leaf: f64,
    pub rdr_stem: f64,
    pub rdr_root: f64,
    pub n_residual_per_mol_c: f64,
    pub shade_rate: f64,
    pub lai_threshold: f64,
    pub sla_per_mol_c: f64,
    pub ground_area: f64,
}

impl Flow for NitrogenSenescence {
    fn type_name(&self) -> &'static str {
        "NitrogenSenescence"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let leaf = amt(snapshot, &self.leaf_c);
        let stem = amt(snapshot, &self.stem_c);
        let root = amt(snapshot, &self.root_c);
        // The identical arithmetic the Senescence flow above uses (Rust inlines the rate
        // law rather than routing it through `science`, so this mirrors the expression, not
        // a shared helper — the drift hazard that buys is pinned Python-side by comparing
        // this flow's shed carbon against Senescence's own litter leg).
        let lai = leaf * self.sla_per_mol_c / self.ground_area;
        let rdr_leaf = crate::biosphere::science::mutual_shading_rate(
            lai,
            self.rdr_leaf,
            self.shade_rate,
            self.lai_threshold,
        );
        let shed_carbon = rdr_leaf * leaf + self.rdr_stem * stem + self.rdr_root * root;
        let plant_n = amt(snapshot, &self.plant_n);
        let biomass_c = leaf + stem + root;
        let shed = if shed_carbon <= 0.0 || plant_n <= 0.0 || biomass_c <= 0.0 {
            0.0
        } else {
            (plant_n / biomass_c).min(self.n_residual_per_mol_c) * shed_carbon * dt
        };
        FlowResult::new(vec![leg(&self.plant_n, -shed)?, leg(&self.litter_n, shed)?])
    }
}

/// The nitrogen belonging to `moved_carbon` at the donor pool's own N:C.
///
/// The one kernel behind both microbe-mediated legs: a carbon flux leaving a pool takes
/// that pool's nitrogen with it. Returns 0.0 for an empty or non-positive pool, so
/// positivity is structural (never a divide-by-zero, never a negative leg).
fn carried_nitrogen(moved_carbon: f64, pool_n: f64, pool_c: f64) -> f64 {
    if moved_carbon <= 0.0 || pool_n <= 0.0 || pool_c <= 0.0 {
        return 0.0;
    }
    moved_carbon * (pool_n / pool_c)
}

/// NITROGEN `litter_n -> microbial_n`, carried by the decomposed carbon.
///
/// The N leg of `Decomposition`. It RECOMPUTES that flow's carbon flux from the same
/// rate rather than collapsing to `decomposition_rate * litter_n`: the two are equal
/// only while `Decomposition` stays first-order, and the collapsed form would read
/// identically today and silently outlive that premise.
pub struct LitterNitrogenTransfer {
    pub id: String,
    pub litter_n: String,
    pub microbial_n: String,
    pub soil_n: String,
    pub litter_carbon: String,
    pub o2_pool: String,
    pub decomposition_rate: f64,
    pub litter_respired_fraction: f64,
    pub o2_half_saturation: f64,
    pub air_mol: f64,
}

impl Flow for LitterNitrogenTransfer {
    fn type_name(&self) -> &'static str {
        "LitterNitrogenTransfer"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let litter_c = amt(snapshot, &self.litter_carbon);
        // The identical flux Decomposition decays out of litter_carbon -- f_O2 included,
        // which the carbon side gained with its CO2 leg (the humification split).
        let f_o2 = science::oxygen_limitation_factor(
            amt(snapshot, &self.o2_pool),
            self.air_mol,
            self.o2_half_saturation,
        );
        let decomposed = self.decomposition_rate * litter_c * f_o2 * dt;
        let moved = carried_nitrogen(decomposed, amt(snapshot, &self.litter_n), litter_c);
        // N follows the CARBON partition: the respired share mineralizes to soil_n, the
        // stabilised share carries its nitrogen into microbial biomass.
        let (mineralized, transferred) =
            respired_and_stabilized(moved, self.litter_respired_fraction);
        FlowResult::new(vec![
            leg(&self.litter_n, -moved)?,
            leg(&self.microbial_n, transferred)?,
            leg(&self.soil_n, mineralized)?,
        ])
    }
}

/// NITROGEN `microbial_n -> soil_n`, carried by the respired carbon.
///
/// The N leg of `MicrobialRespiration`, `f_O2` included -- which is the clearest reason
/// this recomputes rather than reusing a bare rate: the N release must throttle with the
/// carbon as O2 depletes.
pub struct MicrobialNitrogenRelease {
    pub id: String,
    pub microbial_n: String,
    pub soil_n: String,
    pub humus_n: String,
    pub microbial_carbon: String,
    pub o2_pool: String,
    pub microbial_respiration_rate: f64,
    pub active_stabilization_co2_fraction: f64,
    pub o2_half_saturation: f64,
    pub air_mol: f64,
}

impl Flow for MicrobialNitrogenRelease {
    fn type_name(&self) -> &'static str {
        "MicrobialNitrogenRelease"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let microbial_c = amt(snapshot, &self.microbial_carbon);
        // The identical flux MicrobialRespiration burns to CO2 -- f_O2 included.
        let f_o2 = science::oxygen_limitation_factor(
            amt(snapshot, &self.o2_pool),
            self.air_mol,
            self.o2_half_saturation,
        );
        let turned = self.microbial_respiration_rate * microbial_c * f_o2 * dt;
        let moved = carried_nitrogen(turned, amt(snapshot, &self.microbial_n), microbial_c);
        let (mineralized, stabilized) =
            respired_and_stabilized(moved, self.active_stabilization_co2_fraction);
        FlowResult::new(vec![
            leg(&self.microbial_n, -moved)?,
            leg(&self.soil_n, mineralized)?,
            leg(&self.humus_n, stabilized)?,
        ])
    }
}

/// NITROGEN `humus_n -> soil_n + microbial_n`, carried by the decayed slow-SOM carbon.
///
/// The N leg of `HumusDecomposition`, and the third member of the carried-nitrogen
/// family: no nitrogen rate exists anywhere in the decomposer chain.
pub struct HumusNitrogenRelease {
    pub id: String,
    pub humus_n: String,
    pub soil_n: String,
    pub microbial_n: String,
    pub humus_carbon: String,
    pub o2_pool: String,
    pub slow_decomposition_rate: f64,
    pub slow_respired_fraction: f64,
    pub o2_half_saturation: f64,
    pub air_mol: f64,
}

impl Flow for HumusNitrogenRelease {
    fn type_name(&self) -> &'static str {
        "HumusNitrogenRelease"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let humus_c = amt(snapshot, &self.humus_carbon);
        let f_o2 = science::oxygen_limitation_factor(
            amt(snapshot, &self.o2_pool),
            self.air_mol,
            self.o2_half_saturation,
        );
        let decayed = humus_c * self.slow_decomposition_rate * f_o2 * dt;
        let moved = carried_nitrogen(decayed, amt(snapshot, &self.humus_n), humus_c);
        let (mineralized, returned) = respired_and_stabilized(moved, self.slow_respired_fraction);
        FlowResult::new(vec![
            leg(&self.humus_n, -moved)?,
            leg(&self.soil_n, mineralized)?,
            leg(&self.microbial_n, returned)?,
        ])
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
    fn type_name(&self) -> &'static str {
        "Condensation"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let condensed = self.condensation_rate * amt(snapshot, &self.water_vapor) * dt;
        FlowResult::new(vec![
            leg(&self.water_vapor, -condensed)?,
            leg(&self.condensate, condensed)?,
        ])
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
    fn type_name(&self) -> &'static str {
        "Recycling"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let recycled = self.recycling_rate * amt(snapshot, &self.condensate) * dt;
        FlowResult::new(vec![
            leg(&self.condensate, -recycled)?,
            leg(&self.soil_water, recycled)?,
        ])
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
    fn type_name(&self) -> &'static str {
        "Grazing"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let grazed = self.grazing_rate * amt(snapshot, &self.leaf_c) * dt;
        FlowResult::new(vec![
            leg(&self.leaf_c, -grazed)?,
            leg(&self.consumer_carbon, grazed)?,
        ])
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
    fn type_name(&self) -> &'static str {
        "ConsumerRespiration"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let f_o2 = science::oxygen_limitation_factor(
            amt(snapshot, &self.o2_pool),
            self.air_mol,
            self.o2_half_saturation,
        );
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
    fn type_name(&self) -> &'static str {
        "ConsumerMortality"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let died = self.mortality_rate * amt(snapshot, &self.consumer_carbon) * dt;
        FlowResult::new(vec![
            leg(&self.consumer_carbon, -died)?,
            leg(&self.litter_carbon, died)?,
        ])
    }
}

// --- the thermal-time aux ---------------------------------------------------

/// `AuxProcess` advancing the `thermal_time` accumulator.
///
/// Optionally vernalization- and photoperiod-aware (post-roadmap scope (B) inc. 1): when
/// the modifier fields are `Some`, the degree-day rate is scaled by the Eqn-8.6 and
/// Eqn-7.6 factors, applied ONLY in the vegetative phase (`DVS < 1` — wheat is
/// insensitive to both cold and daylength at/after anthesis). With both `None` this is
/// byte-for-byte the pre-scope-(B) plain degree-day rate.
pub struct ThermalTimeAccumulation {
    pub id: String,
    pub accumulator: String,
    pub temp_var: String,
    pub t_base: f64,
    pub t_cap: f64,
    pub tsum_anthesis: f64,
    pub tsum_maturity: f64,
    pub vernalization: Option<params::VernalizationParams>,
    pub vernalization_accumulator: Option<String>,
    pub photoperiod: Option<params::PhotoperiodParams>,
    pub daylength_var: Option<String>,
    /// Drought acceleration ([F] Eqn 15.8) — the THIRD modifier. `None` leaves the rate
    /// byte-for-byte what it was, which is potato's case ([F] Table 15.1 has no potato
    /// row and populates `WSSD` for only two of its ten crops).
    pub drought: Option<params::DroughtDevelopmentParams>,
    pub drought_soil_water: Option<String>,
    pub drought_rooted_depth_aux: Option<String>,
}

impl ThermalTimeAccumulation {
    /// `DVS < 1` — the gate the two VEGETATIVE modifiers share. ⚠ `WSFD` is NOT gated by
    /// it: [F] Box 16.2 gates that one on `CTU > tuEMR` only, and this accumulator starts
    /// at emergence, so it runs through grain filling too.
    fn is_vegetative(&self, snapshot: &State) -> bool {
        let tt = snapshot.aux.get(&self.accumulator).copied().unwrap_or(0.0);
        science::development_stage(tt, self.tsum_anthesis, self.tsum_maturity) < 1.0
    }

    /// The Eqn-15.8 multiplier — 1 when drought acceleration is not configured.
    ///
    /// Routed through the same `science::soil_water_stress` the three other consumers
    /// use, on the same step-entry reads, so the fourth consumer cannot disagree with
    /// them about `FTSW` inside one step.
    fn drought_factor(&self, snapshot: &State) -> f64 {
        let (Some(d), Some(sw), Some(depth_aux)) = (
            self.drought,
            self.drought_soil_water.as_ref(),
            self.drought_rooted_depth_aux.as_ref(),
        ) else {
            return 1.0;
        };
        let wsfg = science::soil_water_stress(
            amt(snapshot, sw),
            snapshot.aux.get(depth_aux).copied().unwrap_or(0.0),
            d.soil_extractable_water,
            d.ground_area,
            d.wssg,
        );
        science::drought_development_factor(wsfg, d.wssd)
    }
}

impl AuxProcess for ThermalTimeAccumulation {
    fn type_name(&self) -> &'static str {
        "ThermalTimeAccumulation"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<BTreeMap<String, f64>, SimError> {
        let temp_c = env.get(&self.temp_var)?;
        let mut rate = science::daily_thermal_time(temp_c, self.t_base, self.t_cap);
        if self.is_vegetative(snapshot) {
            // The source's Eqn 7.4 "biological day" BD = tempfun * ppfun, extended by
            // Eqn 8.2's verfun: the modifiers MULTIPLY. Either may be absent.
            if let (Some(v), Some(acc)) =
                (self.vernalization, self.vernalization_accumulator.as_ref())
            {
                let cum = snapshot.aux.get(acc).copied().unwrap_or(0.0);
                rate *= science::vernalization_factor(cum, v.vsen, v.vdsat);
            }
            if let (Some(pp), Some(var)) = (self.photoperiod, self.daylength_var.as_ref()) {
                rate *= science::photoperiod_factor(env.get(var)? / 3600.0, pp.cpp, pp.ppsen);
            }
        }
        // OUTSIDE the vegetative branch and LAST — [F] Box 16.2 gates WSFD on emergence
        // only and applies it to the already-modified DTU.
        rate *= self.drought_factor(snapshot);
        Ok(BTreeMap::from([(self.accumulator.clone(), rate * dt)]))
    }
}

/// `AuxProcess` advancing the `rooted_depth` accumulator (`DEPORT`, [F] Box 14.1).
///
/// The rate, and every stop on it, come from `science::extension_rate` — see `evaluate`,
/// which states why that sharing is load-bearing rather than convenient.
///
/// ⚠ From the Phase-7 port until 2026-08-27 this carried `VernalizationAccumulation`'s doc
/// comment verbatim — a description of a different accumulator reading a different forcing.
/// Batch B recorded the misattribution and S6 moved it back; the text above is new, since
/// the displaced block belonged to the other struct and nothing described this one.
pub struct RootDepthExtension {
    pub id: String,
    pub accumulator: String,
    pub thermal_time_aux: String,
    pub temp_var: String,
    pub soil_water: String,
    pub subsoil_water: String,
    pub params: params::RootDepthParams,
    pub photo: params::PhotosynthesisParams,
    pub pheno: params::PhenologyParams,
    pub wssg: f64,
    pub soil_depth: f64,
    pub soil_extractable_water: f64,
    pub ground_area: f64,
}

impl AuxProcess for RootDepthExtension {
    fn type_name(&self) -> &'static str {
        "RootDepthExtension"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<BTreeMap<String, f64>, SimError> {
        // All four stops (crop cap, soil cap, flowering, dry subsoil) live in
        // `science::extension_rate` - the SAME function `RootZoneCapture` calls, so the
        // depth gained and the water released cannot be computed from different gates.
        // The caps are RATE cut-offs, not increment clamps: the aux contract wants a
        // dt-independent rate. Carried from Python deliberately - the port does not
        // re-decide it (port-mirror-carries-rule-not-rationale).
        let rate = science::extension_rate(
            snapshot.aux.get(&self.accumulator).copied().unwrap_or(0.0),
            snapshot
                .aux
                .get(&self.thermal_time_aux)
                .copied()
                .unwrap_or(0.0),
            env.get(&self.temp_var)?,
            amt(snapshot, &self.soil_water),
            amt(snapshot, &self.subsoil_water),
            &self.params,
            &self.photo,
            &self.pheno,
            self.wssg,
            self.soil_depth,
            self.soil_extractable_water,
            self.ground_area,
        );
        Ok(BTreeMap::from([(self.accumulator.clone(), rate * dt)]))
    }
}

/// WATER flow `subsoil_water -> soil_water` (`EWAT`, [F] Eqn 14.10) - the water side of
/// rooted depth. An internal transfer between two in-system soil stocks: it crosses no
/// boundary and only re-labels which water the crop can reach.
///
/// Mirrors `soil_layers.RootZoneCapture`, including the donor `min` clamp, which [F] Box
/// 14.1 writes as `If EWAT > WSTORG Then EWAT = WSTORG`.
pub struct RootZoneCapture {
    pub id: String,
    pub subsoil_water: String,
    pub soil_water: String,
    pub rooted_depth_aux: String,
    pub thermal_time_aux: String,
    pub temp_var: String,
    pub params: params::RootDepthParams,
    pub photo: params::PhotosynthesisParams,
    pub pheno: params::PhenologyParams,
    pub wssg: f64,
    pub soil_depth: f64,
    pub soil_extractable_water: f64,
    pub ground_area: f64,
}

impl Flow for RootZoneCapture {
    fn type_name(&self) -> &'static str {
        "RootZoneCapture"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let available = amt(snapshot, &self.subsoil_water);
        let rate = science::extension_rate(
            snapshot
                .aux
                .get(&self.rooted_depth_aux)
                .copied()
                .unwrap_or(0.0),
            snapshot
                .aux
                .get(&self.thermal_time_aux)
                .copied()
                .unwrap_or(0.0),
            env.get(&self.temp_var)?,
            amt(snapshot, &self.soil_water),
            available,
            &self.params,
            &self.photo,
            &self.pheno,
            self.wssg,
            self.soil_depth,
            self.soil_extractable_water,
            self.ground_area,
        );
        let demand =
            science::captured_water(rate * dt, self.soil_extractable_water, self.ground_area);
        let flux = if demand < available {
            demand
        } else {
            available
        };
        FlowResult::new(vec![
            leg(&self.subsoil_water, -flux)?,
            leg(&self.soil_water, flux)?,
        ])
    }
}

/// `AuxProcess` advancing the `vernalization_days` accumulator (the SECOND accumulator).
///
/// Structural mirror of `ThermalTimeAccumulation`. Reads AIR temperature: the source
/// prescribes crown temperature but notes the two differ only under snow cover, and no
/// snow forcing exists. De-vernalization (Eqn 8.5) needs daily MAXIMUM temperature, which
/// the forcing does not carry, so it is unimplementable rather than omitted.
pub struct VernalizationAccumulation {
    pub id: String,
    pub accumulator: String,
    pub temp_var: String,
    pub params: params::VernalizationParams,
}

impl AuxProcess for VernalizationAccumulation {
    fn type_name(&self) -> &'static str {
        "VernalizationAccumulation"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        _snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<BTreeMap<String, f64>, SimError> {
        let p = self.params;
        let rate = science::vernalization_day(
            env.get(&self.temp_var)?,
            p.t_base_v,
            p.t_opt_lower_v,
            p.t_opt_upper_v,
            p.t_ceiling_v,
        );
        Ok(BTreeMap::from([(self.accumulator.clone(), rate * dt)]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simcore::environment::{constant, Schedule, SourceResolver};
    use simcore::flow::assert_flow_balanced_default;
    use simcore::quantities::{Quantity, StockKind};
    use simcore::state::Stock;
    use std::collections::HashMap;

    // -----------------------------------------------------------------------------
    // S5 batch A, the gas-exchange third: the multi-quantity (CARBON+OXYGEN) flows.
    //
    // ⚠ These are FLOW-level, not equation-level, and that is why they are here rather
    // than in `science.rs`. `test_gas_exchange.py`'s subject is leg structure — which
    // stocks a transfer touches and in what proportion — and §5ad's roster wrongly filed
    // it under batch A's `science.rs` surface. The correction is in §5ae.
    //
    // ⚠ What is deliberately NOT ported, and why the absence is a decision:
    //
    //   * `test_allocation_balances_carbon_and_oxygen`,
    //     `test_maintenance_closed_balances_carbon_and_oxygen`,
    //     `test_sealed_conserves_oxygen_exactly`,
    //     `test_sealed_co2_o2_anti_correlate_at_pq1` — all four are the SAME claim as
    //     the engine's own machinery. The CO₂ pool's composition is `{C:1, O:2}` and the
    //     O₂ pool's is `{O:2}`, so "one O₂ released per carbon fixed" is precisely what
    //     OXYGEN balance says; and with no boundary O₂ stock, `2·(CO₂+O₂) = const`
    //     forces `ΔO₂ = −ΔCO₂` step for step. `assert_conserved` runs every step of
    //     every run, so a completed sealed run already asserts them.
    //   * `test_maintenance_closed_emits_single_pool_leg` — `FlowResult::new` REJECTS a
    //     duplicate leg, so the withdraw+deposit pair the Python test rules out is not a
    //     wrong flow in Rust, it is an `Err`. Guarded by the constructor, harder than by
    //     a test.
    //   * `test_sealed_o2_stays_far_from_rationing` — its premise is false in the
    //     reference. It is the "`f_O2` is deferred" guard, and `f_O2` is LIVE here
    //     (`MaintenanceRespiration` and the six soil flows all call
    //     `oxygen_limitation_factor`). The reference's sealed chamber DEPLETES O₂ on
    //     purpose; `system.rs::sealed_chamber_runs_well_fed` asserts that depletion and
    //     `rationed == 0` together, which is the successor claim.
    //
    // What IS ported are the claims that survive removing the balance assertion: the
    // magnitude and distribution of the burn, the sealed/open branch difference, and
    // the empty-flow no-ops. Each one below was mutation-checked against
    // `cargo test -p domains --lib`.
    // -----------------------------------------------------------------------------

    const LEAF: &str = "biosphere.leaf_c";
    const STEM: &str = "biosphere.stem_c";
    const ROOT: &str = "biosphere.root_c";
    const STORAGE: &str = "biosphere.storage_c";
    const RESERVE: &str = "biosphere.stem_reserve_c";
    const PLANT_N: &str = "biosphere.plant_n";
    const SOIL_WATER: &str = "biosphere.soil_water";
    const CO2: &str = "biosphere.carbon_pool";
    const O2: &str = "biosphere.o2_pool";
    const THERMAL_TIME: &str = "thermal_time";
    /// ⚠ The AUX keys are the ENGINE's, imported, not literals like the stock ids above.
    ///
    /// The stock ids may be literals here: a flow reads the id this module hands it, so a
    /// literal that drifted would break these tests loudly. An AUX key cannot — the reads go
    /// through `unwrap_or(0.0)`, so a key nothing writes returns a plausible zero and the
    /// test passes on it. This constant was the bare literal `"biosphere.rooted_depth"` from
    /// batch A until 2026-08-27, while the engine's is `"rooted_depth"`; harmless where it
    /// sat, because these tests both wrote and read it, and a trap for any aux test that
    /// inherited it. Batch B declined to inherit it and said so; S6 removed the shadow.
    use super::super::stocks::ROOTED_DEPTH;

    /// The chamber's air basis, in mol — the `f_O2` and Ci denominators.
    const AIR_MOL: f64 = 1000.0;
    /// 1.0 m of root zone at EXTR 0.13 over 1 m² holds 130 kg, so the 100 kg fill below
    /// is FTSW 0.77 — well above `wssg`, i.e. unstressed. (Mirrors the Python fixture's
    /// 2026-08-12 geometry re-basing.)
    const TEST_DEPTH: f64 = 1.0;
    const EXTR: f64 = 0.13;
    const WSSG: f64 = 0.30;

    /// Open-field context: Ci comes off the forcing, as in Phase 1.
    fn ctx_open() -> CarbonContext {
        CarbonContext {
            leaf_c: LEAF.to_string(),
            stem_c: STEM.to_string(),
            root_c: ROOT.to_string(),
            par_var: "par".to_string(),
            ci_var: "ci".to_string(),
            temp_var: "temp".to_string(),
            soil_water_var: "soil_water".to_string(),
            wssg: WSSG,
            rooted_depth_aux: ROOTED_DEPTH.to_string(),
            soil_extractable_water: EXTR,
            plant_n: PLANT_N.to_string(),
            photo: params::photosynthesis(),
            canopy: params::canopy(),
            resp: params::respiration(),
            nitro: params::nitrogen(),
            ground_area: 1.0,
            co2_pool_var: None,
            chamber_air_mol: None,
            ci_ratio: None,
        }
    }

    /// Sealed context: Ci is computed from the finite CO₂ pool through the chamber seam.
    fn ctx_sealed(ci_ratio: f64) -> CarbonContext {
        CarbonContext {
            co2_pool_var: Some("co2_pool".to_string()),
            chamber_air_mol: Some(AIR_MOL),
            ci_ratio: Some(ci_ratio),
            ..ctx_open()
        }
    }

    fn state(leaf: f64, stem: f64, root: f64, co2: f64, o2: f64, thermal_time: f64) -> State {
        let carbon = Quantity::Carbon.canonical_unit();
        let mut stocks: BTreeMap<String, Stock> = BTreeMap::new();
        for (id, amount) in [
            (LEAF, leaf),
            (STEM, stem),
            (ROOT, root),
            (STORAGE, 0.0),
            (RESERVE, 0.0),
        ] {
            stocks.insert(
                id.to_string(),
                Stock::new(
                    id.to_string(),
                    "biosphere".to_string(),
                    Quantity::Carbon,
                    carbon.clone(),
                    amount,
                    StockKind::Population,
                    0.0,
                    false,
                    BTreeMap::new(),
                )
                .expect("organ stock"),
            );
        }
        let mut pool = |id: &str, q: Quantity, amount: f64, comp: BTreeMap<Quantity, f64>| {
            stocks.insert(
                id.to_string(),
                Stock::new(
                    id.to_string(),
                    "biosphere".to_string(),
                    q,
                    q.canonical_unit(),
                    amount,
                    StockKind::Pool,
                    0.0,
                    false,
                    comp,
                )
                .expect("pool stock"),
            );
        };
        pool(PLANT_N, Quantity::Nitrogen, 1.0, BTreeMap::new());
        pool(SOIL_WATER, Quantity::Water, 100.0, BTreeMap::new());
        // The CO₂ pool is a true molecular stock: 1 mol C + 2 mol O per mol CO₂.
        pool(
            CO2,
            Quantity::Carbon,
            co2,
            BTreeMap::from([(Quantity::Carbon, 1.0), (Quantity::Oxygen, 2.0)]),
        );
        // Its O₂ counterpart: 2 mol OXYGEN per mol O₂.
        pool(
            O2,
            Quantity::Oxygen,
            o2,
            BTreeMap::from([(Quantity::Oxygen, 2.0)]),
        );
        State::new(
            0,
            stocks,
            0,
            BTreeMap::from([
                (THERMAL_TIME.to_string(), thermal_time),
                (ROOTED_DEPTH.to_string(), TEST_DEPTH),
            ]),
        )
        .expect("fixture state")
    }

    /// A well-fed vegetative state: DVS ≈ 0.5, organs 3 : 1 : 1, chamber gases filled.
    fn growing_state() -> State {
        state(3.0, 1.0, 1.0, 0.4, 0.21 * AIR_MOL, 550.0)
    }

    /// Put carbon in the STORAGE organ, which `leaf_and_biomass` excludes.
    ///
    /// It exists to make the maintenance denominator falsifiable. With storage at 0 — the
    /// state the first draft of these tests used — `leaf + stem + root` and
    /// `leaf + stem + root + storage` are the same number, so every share and every burn
    /// total agrees under both readings and the tests cannot tell them apart. Filling it
    /// separates the two.
    fn with_storage(mut s: State, amount: f64) -> State {
        s.stocks.get_mut(STORAGE).expect("storage stock").amount = amount;
        s
    }

    fn resolver(par: f64, ci: f64) -> SourceResolver {
        let forcings: HashMap<String, Schedule> = HashMap::from([
            ("par".to_string(), constant(par).expect("par")),
            ("ci".to_string(), constant(ci).expect("ci")),
            ("temp".to_string(), constant(20.0).expect("temp")),
        ]);
        let shared: HashMap<String, String> = HashMap::from([
            ("soil_water".to_string(), SOIL_WATER.to_string()),
            ("co2_pool".to_string(), CO2.to_string()),
        ]);
        SourceResolver::new(forcings, shared).expect("resolver")
    }

    fn legs_of(flow: &dyn Flow, s: &State, par: f64) -> BTreeMap<String, f64> {
        let r = resolver(par, 400.0);
        let env = r.bind(s, 1.0);
        flow.evaluate(s, &env, 1.0)
            .expect("evaluate")
            .legs
            .iter()
            .map(|l| (l.stock.clone(), l.amount))
            .collect()
    }

    fn allocation(ctx: CarbonContext, o2_pool: Option<String>) -> Allocation {
        Allocation {
            id: "biosphere.allocation".to_string(),
            ctx,
            co2_atmos: CO2.to_string(),
            storage_c: STORAGE.to_string(),
            thermal_time_aux: THERMAL_TIME.to_string(),
            pheno: params::phenology(),
            table: params::allocation().table,
            o2_pool,
            stem_reserve_c: None,
            fstr: 0.0,
            reserve_cessation_dvs: 0.0,
        }
    }

    /// The sealed maintenance flow: source pool == sink pool, so `covered` is a round
    /// trip and only the organ-burned shortfall is a real respiration.
    fn maintenance_sealed() -> MaintenanceRespiration {
        MaintenanceRespiration {
            id: "biosphere.maintenance_respiration".to_string(),
            ctx: ctx_open(),
            co2_atmos: CO2.to_string(),
            co2_resp: CO2.to_string(),
            o2_pool: Some(O2.to_string()),
            air_mol: Some(AIR_MOL),
        }
    }

    // --- Allocation: CO₂ → biomass + O₂ ------------------------------------------

    /// PQ = 1: every mol C fixed into an organ releases one mol O₂ and draws one mol C
    /// from the pool, and the sum runs over ALL FOUR organs including storage.
    ///
    /// ⚠ Honest scope: given the pool compositions this is OXYGEN balance restated, so
    /// its mutation-red is on loan from `assert_flow_balanced`. It is ported anyway
    /// because it is the claim that would have to be re-checked if a composition ever
    /// changed, and because the FOUR-organ sum (storage included) is the part a reader
    /// gets wrong. Its independent content is the organ ROSTER, not the ratio.
    /// Mirrors `test_gas_exchange.py::test_allocation_releases_o2_equal_to_carbon_fixed`
    /// and `::test_allocation_storage_carbon_releases_o2_too`.
    #[test]
    fn allocation_releases_one_oxygen_per_carbon_fixed_across_all_four_organs() {
        // DVS 1.5 (post-anthesis) so the storage leg is nonzero and joins the sum.
        let s = state(3.0, 1.0, 1.0, 0.4, 0.21 * AIR_MOL, 1100.0 + 375.0);
        let legs = legs_of(&allocation(ctx_open(), Some(O2.to_string())), &s, 800.0);
        let fixed = legs[LEAF] + legs[STEM] + legs[ROOT] + legs[STORAGE];
        assert!(fixed > 0.0, "no carbon fixed: the fixture is not growing");
        assert!(legs[STORAGE] > 0.0, "grain is not filling at DVS 1.5");
        assert_eq!(legs[O2], fixed);
        assert_eq!(legs[CO2], -fixed);
    }

    /// `o2_pool: None` (open field) keeps the single-currency Phase-1 legs.
    ///
    /// Balance-immune in BOTH directions, which is what makes it worth writing: an open
    /// field has no O₂ stock at all, so an unconditional leg would be a missing-stock
    /// panic rather than an imbalance, and a conditional one that fires on the wrong
    /// branch changes nothing any conserved quantity can see.
    /// Mirrors `test_gas_exchange.py::test_allocation_open_field_has_no_o2_leg`.
    #[test]
    fn allocation_in_the_open_field_emits_no_oxygen_leg() {
        let legs = legs_of(&allocation(ctx_open(), None), &growing_state(), 800.0);
        assert!(!legs.contains_key(O2));
        assert!(
            legs[LEAF] > 0.0,
            "the open-field flow still grows the plant"
        );
    }

    /// The sealed Ci seam is WIRED: `CarbonContext::ci` reads the finite pool, not the
    /// `ci` forcing. Magnitude-only and therefore balance-immune — a context that fell
    /// back to the forcing would fix a different amount of carbon with every leg still
    /// summing to zero.
    ///
    /// The fixture makes the two sources disagree on purpose: the forcing is 400
    /// µmol mol⁻¹ while 0.4 mol of CO₂ in 1000 mol of air at `ci_ratio = 0.5` is 200.
    /// No Python ancestor — `test_chamber.py` owns the seam there; this is the wiring
    /// claim that makes `science.rs::ci_from_a_finite_pool_…` reachable from a flow.
    #[test]
    fn the_sealed_context_reads_ci_from_the_pool_and_not_the_forcing() {
        let s = growing_state();
        let sealed = legs_of(&allocation(ctx_sealed(0.5), None), &s, 800.0);
        let open = legs_of(&allocation(ctx_open(), None), &s, 800.0);
        assert!(
            sealed[LEAF] < open[LEAF],
            "Ci 200 must fix less than Ci 400: sealed {} vs open {}",
            sealed[LEAF],
            open[LEAF]
        );
        // And it is the SAME arithmetic: a sealed ratio of 1.0 puts Ci back at Ca = 400.
        let matched = legs_of(&allocation(ctx_sealed(1.0), None), &s, 800.0);
        assert_eq!(matched[LEAF], open[LEAF]);
    }

    /// ⚠⚠ PER-FLOW balance on the three gas flows — the successor to
    /// `test_allocation_balances_carbon_and_oxygen` and
    /// `test_maintenance_closed_balances_carbon_and_oxygen`, which §5ae first recorded as
    /// needing none.
    ///
    /// **That reasoning was wrong, and the error is worth keeping.** It said the claim was
    /// already made by "the engine's own machinery". What runs every step is
    /// `assert_conserved`, which folds **state deltas** across every stock after every flow
    /// has been applied — a step-level claim. `assert_flow_balanced` is the **local** one:
    /// this flow, on its own, moves no net CARBON or OXYGEN. The step-level fold cannot see
    /// an imbalance that another flow in the same step cancels, and it diagnoses one it does
    /// see as "the step drifted", naming no flow.
    ///
    /// The gap was found by grepping for the assertion rather than by reasoning about it:
    /// `crew`, `eclss`, `power` and `thermal` each call `assert_flow_balanced_default` in
    /// their own in-src tests, and the **biosphere called it nowhere in the entire crate**.
    /// So this was not a claim the engine already made — it was the one domain that had
    /// dropped it. *Grep for the assertion before recording a claim as covered.*
    #[test]
    fn every_gas_flow_balances_carbon_and_oxygen_leg_by_leg() {
        let r = resolver(800.0, 400.0);
        let dark = resolver(0.0, 400.0);
        // Storage filled and DVS past anthesis, so all four organ legs are live.
        let s = with_storage(
            state(3.0, 1.0, 1.0, 0.4, 0.21 * AIR_MOL, 1100.0 + 375.0),
            10.0,
        );

        let alloc = allocation(ctx_open(), Some(O2.to_string()));
        let result = alloc
            .evaluate(&s, &r.bind(&s, 1.0), 1.0)
            .expect("allocation");
        assert!(
            result.legs.len() >= 6,
            "the fixture is not exercising every leg"
        );
        assert_flow_balanced_default(&result, &s.stocks).expect("Allocation balances");

        // The sealed maintenance burn, on the day it actually burns.
        let m = maintenance_sealed();
        let burning = m
            .evaluate(&s, &dark.bind(&s, 1.0), 1.0)
            .expect("maintenance");
        assert!(!burning.legs.is_empty(), "the fixture is not burning");
        assert_flow_balanced_default(&burning, &s.stocks).expect("sealed burn balances");

        // The sealed chamber's Ci seam changes the amounts, not the balance.
        let sealed_alloc = allocation(ctx_sealed(0.7), Some(O2.to_string()));
        let sealed = sealed_alloc
            .evaluate(&s, &r.bind(&s, 1.0), 1.0)
            .expect("sealed allocation");
        assert_flow_balanced_default(&sealed, &s.stocks).expect("sealed Allocation balances");

        // And the open-field growth respiration, the one flow with a boundary sink.
        let growth = GrowthRespiration {
            id: "biosphere.growth_respiration".to_string(),
            ctx: ctx_open(),
            co2_atmos: CO2.to_string(),
            co2_resp: "boundary.co2".to_string(),
        };
        let gres = growth.evaluate(&s, &r.bind(&s, 1.0), 1.0).expect("growth");
        assert!(!gres.legs.is_empty());
        // ⚠ `boundary.co2` is not in the fixture's stock map, and `assert_flow_balanced`
        // reads compositions from it — so this one is asserted on CARBON directly, which
        // is the only quantity a single-currency boundary sink can be checked on here.
        let net: f64 = gres.legs.iter().map(|l| l.amount).sum();
        assert!(net.abs() <= 1e-15, "GrowthRespiration nets {net}, not 0");
    }

    // --- MaintenanceRespiration: biomass + O₂ → CO₂ ------------------------------

    /// A deficit day (dark ⇒ GASS = 0): the shortfall is burned out of the organs and
    /// returned to the pool as CO₂, consuming one mol O₂ per mol C burned.
    /// Mirrors `::test_maintenance_closed_burns_biomass_to_co2_consuming_o2`.
    #[test]
    fn sealed_maintenance_burns_organs_to_co2_and_consumes_oxygen_one_for_one() {
        let legs = legs_of(&maintenance_sealed(), &growing_state(), 0.0);
        let burned = -(legs[LEAF] + legs[STEM] + legs[ROOT]);
        assert!(
            burned > 0.0,
            "nothing burned: the fixture is not in deficit"
        );
        assert_eq!(legs[CO2], burned);
        assert_eq!(legs[O2], -burned);
    }

    /// ⚠⚠ The balance-immune magnitude claim, and the reason this batch is not just a
    /// restatement of conservation: the sealed burn is THROTTLED by `f_O2`.
    ///
    /// Scaling the whole burn leaves CARBON and OXYGEN balanced and leaves PQ = 1 intact
    /// — organs, pool and O₂ all move by the same amount — so nothing in the balance or
    /// conservation machinery can see the factor at all. What pins it is the Michaelis
    /// ratio: at a chamber mole fraction of exactly `K` the factor is ½ and at `9K` it
    /// is 9/10, so the two burns must stand in the ratio 5/9 whatever `K` is.
    ///
    /// No Python ancestor: `f_O2` was still deferred when `test_gas_exchange.py` was
    /// written (its header says so, and that prose is stale against this tree).
    #[test]
    fn the_sealed_burn_is_throttled_by_f_o2_in_the_michaelis_ratio() {
        let k = params::respiration().o2_half_saturation;
        let burn = |o2: f64| {
            let s = state(3.0, 1.0, 1.0, 0.4, o2, 550.0);
            legs_of(&maintenance_sealed(), &s, 0.0)[CO2]
        };
        // The reference point: x = K ⇒ f_O2 = 1/2.
        let at_k = burn(k * AIR_MOL);
        assert!(at_k > 0.0, "the fixture is not in deficit");
        // ⚠ SWEPT, not pinned at one pair. A single ratio can be right by coincidence if
        // the burn depends on O₂ through some path other than `f_O2`; the whole curve
        // cannot. For every multiple m of K the factor is m/(1+m), so the burn must be
        // `2·m/(1+m)` times the burn at K — exactly, for every m.
        for m in [0.5, 2.0, 4.0, 9.0, 100.0, 2100.0] {
            let want = at_k * 2.0 * m / (1.0 + m);
            let got = burn(m * k * AIR_MOL);
            assert!(
                (got - want).abs() <= 1e-12 * want,
                "f_O2 is not throttling the burn at x = {m}·K: {got}, want {want}"
            );
        }
    }

    /// The shortfall is split across the organs IN PROPORTION to their carbon, not
    /// evenly. Balance-immune: any split summing to the same total balances identically,
    /// so only a distribution test can see this.
    #[test]
    fn the_sealed_burn_is_split_in_proportion_to_organ_carbon() {
        // ⚠ Storage is filled to 10 mol C on purpose. The denominator is `leaf + stem +
        // root` — grain does NOT pay maintenance and does NOT dilute the organ shares —
        // and with storage at 0 that reading is indistinguishable from "the whole plant".
        let s = with_storage(growing_state(), 10.0);
        let legs = legs_of(&maintenance_sealed(), &s, 0.0);
        // 3 : 1 : 1 leaf : stem : root, so the leaf pays three fifths of the burn.
        let total = legs[LEAF] + legs[STEM] + legs[ROOT];
        assert!(
            (legs[LEAF] / total - 0.6).abs() <= 1e-12,
            "leaf share {} of {total}",
            legs[LEAF]
        );
        assert_eq!(legs[STEM], legs[ROOT]);
        assert!(
            (legs[STEM] / total - 0.2).abs() <= 1e-12,
            "stem share {} of {total}",
            legs[STEM]
        );
        // ...and the burn TOTAL is untouched by the grain, which is the same claim about
        // the same denominator, read off MRES instead of off the shares.
        assert_eq!(
            legs[CO2],
            legs_of(&maintenance_sealed(), &growing_state(), 0.0)[CO2]
        );
    }

    /// `covered = min(GASS, MRES)`: on a surplus day the shortfall is zero, the covered
    /// maintenance is a CO₂→CO₂ round trip on the single pool, and the flow is EMPTY —
    /// not a set of zero legs.
    ///
    /// Balance-immune twice over: a zero leg balances exactly as an absent one does, and
    /// a `min` flipped to `max` scales the burn without unbalancing anything.
    /// Mirrors `::test_maintenance_closed_surplus_day_is_noop`.
    #[test]
    fn sealed_maintenance_on_a_surplus_day_is_an_empty_flow() {
        let legs = legs_of(&maintenance_sealed(), &growing_state(), 800.0);
        assert!(legs.is_empty(), "surplus day emitted legs: {legs:?}");
    }

    /// A partial deficit (large biomass, tiny canopy ⇒ 0 < GASS < MRES) still burns only
    /// the shortfall: the covered half stays the dropped round trip, and what daylight
    /// covered is exactly `f_O2·GASS·dt`.
    /// Mirrors `::test_maintenance_closed_partial_deficit_balances`.
    #[test]
    fn sealed_maintenance_burns_only_the_shortfall_on_a_partial_deficit_day() {
        let s = state(0.1, 50.0, 50.0, 0.4, 0.21 * AIR_MOL, 550.0);
        let lit = legs_of(&maintenance_sealed(), &s, 800.0)[CO2];
        let dark = legs_of(&maintenance_sealed(), &s, 0.0)[CO2];
        assert!(lit > 0.0, "the partial-deficit fixture is not in deficit");
        assert!(
            lit < dark,
            "daylight did not cover any maintenance: {lit} vs {dark}"
        );
        let r = resolver(800.0, 400.0);
        let env = r.bind(&s, 1.0);
        let (gass, _, _) = ctx_open().budget(&s, &env).expect("budget");
        let f_o2 = science::oxygen_limitation_factor(
            0.21 * AIR_MOL,
            AIR_MOL,
            params::respiration().o2_half_saturation,
        );
        assert!(
            (dark - lit - f_o2 * gass).abs() <= 1e-12 * f_o2 * gass,
            "the covered part is not GASS: {dark} − {lit} vs {}",
            f_o2 * gass
        );
    }

    /// The OPEN-field branch is different in kind, not merely in wiring: the covered
    /// maintenance is a real draw on the atmosphere, so it appears as a leg even on a
    /// surplus day — the day the sealed branch emits nothing at all.
    #[test]
    fn open_field_maintenance_draws_the_covered_part_from_the_atmosphere() {
        let open = MaintenanceRespiration {
            id: "biosphere.maintenance_respiration".to_string(),
            ctx: ctx_open(),
            co2_atmos: CO2.to_string(),
            co2_resp: "boundary.co2".to_string(),
            o2_pool: None,
            air_mol: None,
        };
        let legs = legs_of(&open, &growing_state(), 800.0);
        assert!(
            legs[CO2] < 0.0,
            "the open field must WITHDRAW the covered maintenance"
        );
        assert!(legs["boundary.co2"] > 0.0);
    }

    // --- GrowthRespiration -------------------------------------------------------

    /// Growth-conversion carbon is gross-assimilated and immediately respired, so in a
    /// sealed chamber it is a CO₂→CO₂ round trip whose O₂ release is reconsumed: an
    /// empty flow. Balance-immune — the round trip it replaces balances perfectly.
    /// Mirrors `::test_growth_resp_closed_is_noop`.
    #[test]
    fn sealed_growth_respiration_is_an_empty_round_trip() {
        let sealed = GrowthRespiration {
            id: "biosphere.growth_respiration".to_string(),
            ctx: ctx_open(),
            co2_atmos: CO2.to_string(),
            co2_resp: CO2.to_string(),
        };
        assert!(legs_of(&sealed, &growing_state(), 800.0).is_empty());
        // The open-field counterpart is NOT empty — so the emptiness is the branch, not
        // a fixture that happens to produce no growth respiration.
        let open = GrowthRespiration {
            id: "biosphere.growth_respiration".to_string(),
            ctx: ctx_open(),
            co2_atmos: CO2.to_string(),
            co2_resp: "boundary.co2".to_string(),
        };
        assert!(legs_of(&open, &growing_state(), 800.0)["boundary.co2"] > 0.0);
    }

    // -----------------------------------------------------------------------------
    // S5 batch B, the aux-process half: `ThermalTimeAccumulation` and
    // `VernalizationAccumulation`.
    //
    // These live here rather than in `science.rs` for the same reason batch A's
    // gas-exchange tests do: the subject is the PROCESS - what it reads, what it
    // multiplies, and when a modifier is gated off - not the equations it composes,
    // which are pinned one file over. Two aux-level claims already had successors in
    // `system.rs` before this batch (`wsfd_uses_wssg_and_is_not_gated_off_at_anthesis`
    // and `drought_acceleration_is_wired_into_the_accumulator_and_no_scenario_shows_it`);
    // they are NOT duplicated here, and the census records them under `system.rs`.
    //
    // ⚠ The measured hole these close: dropping either vegetative modifier's
    // multiply reddens three tests, and not one of the three is about phenology (a
    // peak-LAI band, a mutual-shading regime check, a trajectory fixed-point). The
    // multiplicative COMPOSITION and the anthesis GATE had no test of their own.
    //
    // ⚠ The aux keys are imported, never re-declared: an aux read goes through
    // `unwrap_or(0.0)`, so a key nothing writes returns a plausible zero instead of failing.
    // Batch A's module above had exactly that shadow (`"biosphere.rooted_depth"` against the
    // engine's `"rooted_depth"`) and this block was written not to inherit it; S6 removed
    // the shadow at its source on 2026-08-27, and the rule stated here is why both blocks
    // now import rather than declare.
    use super::super::stocks::{
        pool_stock, ROOTED_DEPTH as AUX_ROOTED_DEPTH, SOIL, THERMAL_TIME as AUX_THERMAL_TIME,
        VERNALIZATION_DAYS as AUX_VERNALIZATION_DAYS,
    };

    /// The committed winter-Europe wheat cardinals, held as LITERALS rather than read
    /// through `params::vernalization()` - batch A's convention, so a loader regression
    /// cannot silently move a physics pin.
    const VERN: params::VernalizationParams = params::VernalizationParams {
        t_base_v: -1.0,
        t_opt_lower_v: 0.0,
        t_opt_upper_v: 8.0,
        t_ceiling_v: 12.0,
        vsen: 0.033,
        vdsat: 50.0,
    };
    const PHOTOPERIOD: params::PhotoperiodParams = params::PhotoperiodParams {
        cpp: 16.0,
        ppsen: 0.09,
    };
    /// `(t_base, t_cap, tsum_anthesis, tsum_maturity)` - the committed phenology block.
    const PHENO: (f64, f64, f64, f64) = (0.0, 30.0, 1100.0, 750.0);

    /// A bare accumulator with every optional modifier switched off.
    fn plain_thermal_time() -> ThermalTimeAccumulation {
        ThermalTimeAccumulation {
            id: "test.thermal_time".to_string(),
            accumulator: AUX_THERMAL_TIME.to_string(),
            temp_var: "temp".to_string(),
            t_base: PHENO.0,
            t_cap: PHENO.1,
            tsum_anthesis: PHENO.2,
            tsum_maturity: PHENO.3,
            vernalization: None,
            vernalization_accumulator: None,
            photoperiod: None,
            daylength_var: None,
            drought: None,
            drought_soil_water: None,
            drought_rooted_depth_aux: None,
        }
    }

    /// Constant `temp` (degC) and, optionally, constant `daylength_s` (SECONDS - the
    /// canonical forcing unit; the accumulator divides by 3600 itself).
    fn weather(temp_c: f64, daylength_h: Option<f64>) -> SourceResolver {
        let mut forcings: HashMap<String, Schedule> =
            HashMap::from([("temp".to_string(), constant(temp_c).expect("temp"))]);
        if let Some(h) = daylength_h {
            forcings.insert(
                "daylength_s".to_string(),
                constant(h * 3600.0).expect("daylength"),
            );
        }
        SourceResolver::new(forcings, HashMap::new()).expect("resolver")
    }

    /// A snapshot carrying only aux values - no stocks, which is all the two vegetative
    /// modifiers read.
    fn aux_only(entries: &[(&str, f64)]) -> State {
        State::new(
            0,
            BTreeMap::new(),
            0,
            entries
                .iter()
                .map(|(k, v)| ((*k).to_string(), *v))
                .collect(),
        )
        .expect("aux-only snapshot")
    }

    fn increment(proc_: &dyn AuxProcess, snapshot: &State, r: &SourceResolver, dt: f64) -> f64 {
        let env = r.bind(snapshot, dt);
        let out = proc_.evaluate(snapshot, &env, dt).expect("aux evaluate");
        assert_eq!(out.len(), 1, "an accumulator advances exactly one aux key");
        *out.values().next().expect("the single increment")
    }

    /// The INCREMENT FORM, and the byte-for-byte guarantee for a crop with no cold or
    /// daylength requirement: with both modifiers absent the accumulator is exactly
    /// `daily_thermal_time(T) * dt`, the pre-scope-(B) behaviour.
    ///
    /// At 18 degC with the committed `t_base = 0` that is 18.0 per day, so a half-day
    /// step is exactly 9.0. Mirrors `test_aux_process_returns_increment_form` and
    /// `test_thermal_time_aux_without_modifiers_is_the_plain_rate`.
    #[test]
    fn thermal_time_aux_without_modifiers_is_the_plain_rate_times_dt() {
        let proc_ = plain_thermal_time();
        let snap = aux_only(&[(AUX_THERMAL_TIME, 0.0)]);
        assert_eq!(increment(&proc_, &snap, &weather(18.0, None), 1.0), 18.0);
        assert_eq!(increment(&proc_, &snap, &weather(18.0, None), 0.5), 9.0);
        // Cold below base: no thermal time at all, read through `env` (decision #16).
        assert_eq!(increment(&proc_, &snap, &weather(-4.0, None), 1.0), 0.0);
    }

    /// The SECOND accumulator, in its own increment form: 4 degC is inside the cited
    /// optimum band `[0, 8]`, so VERDAY is exactly 1 day/day and a half-day step accrues
    /// exactly 0.5. Mirrors `test_vernalization_aux_returns_increment_form`.
    ///
    /// ⚠ It reads AIR temperature and ignores the snapshot entirely - the source
    /// prescribes crown temperature but notes the two differ only under snow cover, and
    /// no snow forcing exists.
    #[test]
    fn vernalization_aux_accrues_verday_times_dt() {
        let proc_ = VernalizationAccumulation {
            id: "test.vernalization_days".to_string(),
            accumulator: AUX_VERNALIZATION_DAYS.to_string(),
            temp_var: "temp".to_string(),
            params: VERN,
        };
        let snap = aux_only(&[(AUX_VERNALIZATION_DAYS, 0.0)]);
        assert_eq!(increment(&proc_, &snap, &weather(4.0, None), 0.5), 0.5);
        assert_eq!(increment(&proc_, &snap, &weather(4.0, None), 1.0), 1.0);
        // Above the ceiling there is no cold to accrue, however long the step.
        assert_eq!(increment(&proc_, &snap, &weather(20.0, None), 1.0), 0.0);
    }

    /// ⚠ **The two vegetative modifiers MULTIPLY** - Eqn 7.4's biological day
    /// `BD = tempfun * ppfun`, extended by Eqn 8.2's `verfun`. Adding them instead, or
    /// applying only the last one written, is caught by nothing else in the binary.
    ///
    /// Hand-computed: fully vernalized (60 >= vdsat = 50) gives `verfun = 1`; a 10-hour
    /// day at `cpp = 16`, `ppsen = 0.09` gives `ppfun = 1 - 0.09*6 = 0.46`; the plain
    /// rate at 18 degC is 18. So the increment is `18 * 1 * 0.46 = 8.28`.
    /// Mirrors `test_thermal_time_aux_applies_both_modifiers_multiplicatively`.
    #[test]
    fn thermal_time_aux_multiplies_the_two_vegetative_modifiers() {
        let proc_ = ThermalTimeAccumulation {
            vernalization: Some(VERN),
            vernalization_accumulator: Some(AUX_VERNALIZATION_DAYS.to_string()),
            photoperiod: Some(PHOTOPERIOD),
            daylength_var: Some("daylength_s".to_string()),
            ..plain_thermal_time()
        };
        // Vegetative (thermal_time 0 => DVS 0), fully vernalized, 10 h day.
        let snap = aux_only(&[(AUX_THERMAL_TIME, 0.0), (AUX_VERNALIZATION_DAYS, 60.0)]);
        let got = increment(&proc_, &snap, &weather(18.0, Some(10.0)), 1.0);
        assert!(
            (got - 8.28).abs() <= 1.0e-12,
            "18 * verfun(60) * ppfun(10 h) must be 8.28, got {got}"
        );
        // The daylength arrives in SECONDS and the process converts: the same increment
        // must NOT be reproduced by feeding hours, which would read 10/3600 h - a
        // near-zero daylength, hence ppfun clamped to 0.
        let hours_by_mistake = SourceResolver::new(
            HashMap::from([
                ("temp".to_string(), constant(18.0).expect("temp")),
                (
                    "daylength_s".to_string(),
                    constant(10.0).expect("daylength"),
                ),
            ]),
            HashMap::new(),
        )
        .expect("resolver");
        assert_eq!(increment(&proc_, &snap, &hours_by_mistake, 1.0), 0.0);
    }

    /// The qualitative cultivar's ARREST, at the process level: with no accumulated cold
    /// `verfun` is 0, so thermal time does not advance AT ALL despite warm weather. A
    /// clamp-free `verfun` would make this NEGATIVE and run development backwards.
    /// Mirrors `test_thermal_time_aux_arrests_completely_when_unvernalized`.
    #[test]
    fn thermal_time_aux_arrests_completely_when_unvernalized() {
        let proc_ = ThermalTimeAccumulation {
            vernalization: Some(VERN),
            vernalization_accumulator: Some(AUX_VERNALIZATION_DAYS.to_string()),
            ..plain_thermal_time()
        };
        let snap = aux_only(&[(AUX_THERMAL_TIME, 0.0), (AUX_VERNALIZATION_DAYS, 0.0)]);
        assert_eq!(increment(&proc_, &snap, &weather(18.0, None), 1.0), 0.0);
        // ...and it is an ARREST, not a slowdown: still zero over a longer step.
        assert_eq!(increment(&proc_, &snap, &weather(18.0, None), 4.0), 0.0);
    }

    /// ⚠ **The anthesis gate, ON its boundary.** Wheat is insensitive to both cold
    /// and daylength at and after anthesis, so past `DVS = 1` the PLAIN degree-day rate
    /// must be recovered EXACTLY - even with zero cold and an 8-hour day, which before
    /// anthesis would arrest development entirely.
    ///
    /// The boundary case is the point: `thermal_time == tsum_anthesis` gives `DVS == 1.0`
    /// exactly, and `is_vegetative` tests `< 1.0`, so the gate closes AT anthesis rather
    /// than after it. One step earlier the same weather gives zero.
    /// Mirrors `test_thermal_time_modifiers_are_gated_off_at_and_after_anthesis`.
    #[test]
    fn the_vegetative_modifiers_are_gated_off_at_and_after_anthesis() {
        let proc_ = ThermalTimeAccumulation {
            vernalization: Some(VERN),
            vernalization_accumulator: Some(AUX_VERNALIZATION_DAYS.to_string()),
            photoperiod: Some(PHOTOPERIOD),
            daylength_var: Some("daylength_s".to_string()),
            ..plain_thermal_time()
        };
        let cold_and_dark = weather(18.0, Some(8.0));
        let at = |thermal_time: f64| {
            let snap = aux_only(&[
                (AUX_THERMAL_TIME, thermal_time),
                (AUX_VERNALIZATION_DAYS, 0.0),
            ]);
            increment(&proc_, &snap, &cold_and_dark, 1.0)
        };
        // AT anthesis: the gate is closed, so the modifiers are off and the rate is plain.
        assert_eq!(at(PHENO.2), 18.0);
        assert_eq!(at(PHENO.2 + 1.0), 18.0);
        // ...and JUST short of it the same inputs arrest development completely, which is
        // what makes the assertion above a claim about the GATE and not about the weather.
        assert_eq!(at(PHENO.2 - 1.0), 0.0);
    }

    /// The byte-for-byte guarantee for the THIRD modifier: a crop with no cited `WSSD`
    /// gets exactly the pre-existing rate even on a bone-dry root zone.
    ///
    /// ⚠ **What this guards is `drought_factor`'s early return, NOT the wiring**, and the
    /// distinction is worth stating because the obvious claim is the wrong one: this test
    /// hand-builds the struct with `drought: None`, so `build_plants` never runs and a
    /// modifier wired unconditionally there is invisible here. Measured - that mutation
    /// reddens `system.rs`'s two wiring tests and leaves this one green. The claim it does
    /// make is that the `let-else` returns 1.0 rather than reading a partially-configured
    /// drought seam: with `drought: None` but a real dry root zone in the snapshot, an
    /// early return that fell through would read `WSFG = 0` and multiply by 1.4.
    ///
    /// ⚠ The dry state is CONSTRUCTED. No scenario in the Rust roster is water
    /// limited - every one of them holds `WSFG == 1` - so this condition cannot be
    /// reached by running anything the tree ships.
    /// Mirrors `test_thermal_time_aux_without_drought_is_the_plain_rate`.
    #[test]
    fn thermal_time_aux_without_a_cited_wssd_ignores_a_bone_dry_root_zone() {
        let mut stocks = BTreeMap::new();
        stocks.insert(
            SOIL_WATER.to_string(),
            pool_stock(SOIL_WATER, SOIL, Quantity::Water, 0.0).expect("dry soil water"),
        );
        let dry = State::new(
            0,
            stocks,
            0,
            BTreeMap::from([
                (AUX_THERMAL_TIME.to_string(), 0.0),
                (AUX_ROOTED_DEPTH.to_string(), 0.15),
            ]),
        )
        .expect("bone-dry snapshot");
        assert_eq!(
            increment(&plain_thermal_time(), &dry, &weather(18.0, None), 1.0),
            18.0
        );
    }

    // -----------------------------------------------------------------------------
    // S5 batch C, the water flows: `Transpiration`, `Irrigation`, and the two closed-loop
    // cycle flows. FLOW-level, so they live here rather than in `science.rs` — the claims
    // are about legs and limbs, not about the equations the legs are computed from.
    //
    // ⚠ These fixtures build their OWN state and use the canonical aux key
    // `stocks::ROOTED_DEPTH` (`"rooted_depth"`) rather than this module's `ROOTED_DEPTH`
    // const, which is the different string `"biosphere.rooted_depth"`. The two are
    // interchangeable only while a test writes the aux key AND the flow's
    // `rooted_depth_aux` field from the same constant; batch B recorded the shadow, and
    // the safe habit is to use the engine's own name where the engine's own name is what
    // a season would write.
    // -----------------------------------------------------------------------------

    const VAPOR_SINK: &str = "boundary.vapor";
    const IRRIGATION_SUPPLY: &str = "boundary.irrigation_supply";
    const WATER_VAPOR: &str = "biosphere.water_vapor";
    const CONDENSATE: &str = "biosphere.condensate";

    /// A WATER-only state: the root zone at `soil_water`, a boundary sink and supply, and
    /// the two cycle pools. The round geometry mirrors the Python fixture — a 1.0 m zone
    /// at EXTR 0.13 over 1 m² holds 130 kg, so `wssg = 0.30` sits at exactly 39 kg.
    fn water_only_state(soil_water: f64, depth: f64, vapor: f64, condensate: f64) -> State {
        let unit = Quantity::Water.canonical_unit();
        let mut stocks: BTreeMap<String, Stock> = BTreeMap::new();
        let mut put = |id: &str, domain: &str, amount: f64, kind: StockKind, unclamped: bool| {
            stocks.insert(
                id.to_string(),
                Stock::new(
                    id.to_string(),
                    domain.to_string(),
                    Quantity::Water,
                    unit.clone(),
                    amount,
                    kind,
                    0.0,
                    unclamped,
                    BTreeMap::new(),
                )
                .expect("water stock"),
            );
        };
        put(SOIL_WATER, "biosphere", soil_water, StockKind::Pool, false);
        put(WATER_VAPOR, "biosphere", vapor, StockKind::Pool, false);
        put(CONDENSATE, "biosphere", condensate, StockKind::Pool, false);
        put(VAPOR_SINK, "boundary", 0.0, StockKind::Boundary, false);
        put(
            IRRIGATION_SUPPLY,
            "boundary",
            1.0e9,
            StockKind::Boundary,
            true,
        );
        State::new(
            0,
            stocks,
            0,
            BTreeMap::from([(super::super::stocks::ROOTED_DEPTH.to_string(), depth)]),
        )
        .expect("water fixture state")
    }

    /// The transpiration forcings: `Rn = 200 W/m²`, `VPD = 1000 Pa`, `T = 20 °C` — the
    /// operating point `science.rs`'s hand-composed Penman–Monteith test derives.
    fn water_resolver(rn: f64, irrigation: f64) -> SourceResolver {
        let forcings: HashMap<String, Schedule> = HashMap::from([
            ("rn".to_string(), constant(rn).expect("rn")),
            ("vpd".to_string(), constant(1000.0).expect("vpd")),
            ("temp".to_string(), constant(20.0).expect("temp")),
            (
                "irrigation".to_string(),
                constant(irrigation).expect("irrigation"),
            ),
        ]);
        SourceResolver::new(forcings, HashMap::new()).expect("water resolver")
    }

    fn transpiration_flow(ground_area: f64) -> Transpiration {
        let p = params::transpiration();
        Transpiration {
            id: "biosphere.transpiration".to_string(),
            soil_water: SOIL_WATER.to_string(),
            vapor_sink: VAPOR_SINK.to_string(),
            rn_var: "rn".to_string(),
            vpd_var: "vpd".to_string(),
            temp_var: "temp".to_string(),
            aerodynamic_resistance: p.aerodynamic_resistance,
            surface_resistance: p.surface_resistance,
            ground_area,
            rooted_depth_aux: super::super::stocks::ROOTED_DEPTH.to_string(),
            soil_extractable_water: EXTR,
            wssg: WSSG,
        }
    }

    fn irrigation_flow(ground_area: f64) -> Irrigation {
        Irrigation {
            id: "biosphere.irrigation".to_string(),
            water_source: IRRIGATION_SUPPLY.to_string(),
            soil_water: SOIL_WATER.to_string(),
            irrigation_var: "irrigation".to_string(),
            ground_area,
            rooted_depth_aux: super::super::stocks::ROOTED_DEPTH.to_string(),
            soil_extractable_water: EXTR,
        }
    }

    fn water_legs(
        flow: &dyn Flow,
        s: &State,
        rn: f64,
        irrigation: f64,
        dt: f64,
    ) -> BTreeMap<String, f64> {
        let r = water_resolver(rn, irrigation);
        let env = r.bind(s, dt);
        flow.evaluate(s, &env, dt)
            .expect("evaluate")
            .legs
            .iter()
            .map(|l| (l.stock.clone(), l.amount))
            .collect()
    }

    /// The transpiration leg IS `PM · WSFG · area`, composed from the three factors
    /// separately rather than compared with a number this tree produced.
    ///
    /// ⚠ THE BEFORE-BATTERY'S SHARPEST WATER READING. Deleting `f_water` from this
    /// product entirely — a plant that transpires as if it were never water-limited —
    /// reddened exactly ONE test in the 221-test lib binary, and that test is about
    /// drought-ACCELERATED phenology. A whole feedback removed, noticed by one stranger.
    /// Mirrors `test_transpiration_leg_is_pm_times_fwater_times_area`.
    #[test]
    fn the_transpiration_leg_is_potential_times_water_stress_times_area() {
        // 25 kg in a 130 kg zone is FTSW 0.1923, so WSFG = 0.1923/0.30 = 0.6410...
        let s = water_only_state(25.0, TEST_DEPTH, 0.0, 0.0);
        let legs = water_legs(&transpiration_flow(1.0), &s, 200.0, 0.0, 1.0);
        let p = params::transpiration();
        let potential = science::penman_monteith_transpiration(
            200.0,
            1000.0,
            20.0,
            p.aerodynamic_resistance,
            p.surface_resistance,
        );
        let f_water = (25.0 / 130.0) / WSSG;
        assert!(
            (f_water - 0.641_025_641_025_641).abs() < 1e-12,
            "the fixture is not the FTSW the comment claims: {f_water}"
        );
        let want = potential * f_water * 1.0;
        assert!(
            (legs[SOIL_WATER] + want).abs() <= 1e-12 * want,
            "soil leg {} vs -{want}",
            legs[SOIL_WATER]
        );
        assert!(
            (legs[VAPOR_SINK] - want).abs() <= 1e-12 * want,
            "vapor leg {} vs {want}",
            legs[VAPOR_SINK]
        );
        // ...and the stress factor really is doing work here: an unstressed zone of the
        // same depth transpires strictly more, by exactly 1/f_water.
        let full = water_only_state(130.0, TEST_DEPTH, 0.0, 0.0);
        let unstressed = water_legs(&transpiration_flow(1.0), &full, 200.0, 0.0, 1.0)[VAPOR_SINK];
        assert!(
            (unstressed * f_water - want).abs() <= 1e-12 * want,
            "the stressed leg is not the unstressed one scaled by WSFG"
        );
    }

    /// An empty root zone transpires exactly zero, so the flow can never drive the pool
    /// negative from 0.
    ///
    /// ⚠ The shutoff MOVED when the stress form was re-based on geometry (2026-08-12):
    /// `WSFG` reaches 0 only at `FTSW = 0`, where the absolute-kg ramp it replaced shut
    /// off at a nonzero wilting mass. Exactly zero at exactly empty is what replaces the
    /// old structural-positivity guarantee.
    /// Mirrors `test_transpiration_shuts_off_at_an_empty_root_zone`.
    #[test]
    fn transpiration_shuts_off_exactly_at_an_empty_root_zone() {
        let s = water_only_state(0.0, TEST_DEPTH, 0.0, 0.0);
        let legs = water_legs(&transpiration_flow(1.0), &s, 200.0, 0.0, 1.0);
        assert_eq!(legs[SOIL_WATER], 0.0);
        assert_eq!(legs[VAPOR_SINK], 0.0);
        // A hair of water is a hair of transpiration — asymptotic, not a hard floor.
        let hair = water_only_state(1e-9, TEST_DEPTH, 0.0, 0.0);
        assert!(water_legs(&transpiration_flow(1.0), &hair, 200.0, 0.0, 1.0)[VAPOR_SINK] > 0.0);
    }

    /// The extensive transform, BOTH halves — and the half that is a surprise.
    ///
    /// `ground_area` appears twice since the re-basing: once in the demand and once in
    /// `TTSW`. So tripling the PLOT triples the flux only if the WATER triples with it;
    /// a three-times-larger plot on the SAME water is three times drier, and the two
    /// factors cancel exactly. The second half is the physically right consequence of
    /// sizing the soil honestly, and a test that pinned only the first would pass on a
    /// build that had dropped the denominator's area.
    /// Mirrors `test_transpiration_scales_with_ground_area_under_the_extensive_transform`
    /// and `test_a_bigger_plot_on_the_same_water_is_drier`.
    #[test]
    fn transpiration_scales_with_the_plot_only_when_the_water_scales_with_it() {
        let one = water_legs(
            &transpiration_flow(1.0),
            &water_only_state(25.0, TEST_DEPTH, 0.0, 0.0),
            200.0,
            0.0,
            1.0,
        )[VAPOR_SINK];
        // (a) The similarity transform: three times the plot AND three times the water.
        let triple = water_legs(
            &transpiration_flow(3.0),
            &water_only_state(75.0, TEST_DEPTH, 0.0, 0.0),
            200.0,
            0.0,
            1.0,
        )[VAPOR_SINK];
        assert!(
            (triple - 3.0 * one).abs() <= 1e-12 * one,
            "{triple} != 3 x {one}"
        );
        // (b) Three times the plot on the SAME water: exactly unchanged.
        let same_water = water_legs(
            &transpiration_flow(3.0),
            &water_only_state(25.0, TEST_DEPTH, 0.0, 0.0),
            200.0,
            0.0,
            1.0,
        )[VAPOR_SINK];
        assert!(
            (same_water - one).abs() <= 1e-12 * one,
            "a bigger plot on the same water must transpire the same: {same_water} vs {one}"
        );
    }

    /// Both water flows are dt-linear and balanced leg-for-leg.
    ///
    /// dt-linearity is what makes the step size a numerical choice rather than a
    /// scientific one; the balance is what keeps a sealed chamber's water conserved.
    /// Mirrors `test_transpiration_scales_linearly_with_dt`,
    /// `test_transpiration_is_water_balanced` and `test_irrigation_is_water_balanced`.
    #[test]
    fn the_water_flows_are_dt_linear_and_balanced() {
        let s = water_only_state(25.0, TEST_DEPTH, 0.0, 0.0);
        let transp = transpiration_flow(1.0);
        let irrig = irrigation_flow(1.0);
        for (name, flow) in [
            ("transpiration", &transp as &dyn Flow),
            ("irrigation", &irrig as &dyn Flow),
        ] {
            let full = water_legs(flow, &s, 200.0, 5.0, 1.0);
            let half = water_legs(flow, &s, 200.0, 5.0, 0.5);
            for (stock, amount) in &full {
                assert!(
                    (half[stock] - 0.5 * amount).abs() <= 1e-12 * amount.abs().max(1.0),
                    "{name}: {stock} is not dt-linear ({} vs {amount})",
                    half[stock]
                );
                assert_ne!(
                    *amount, 0.0,
                    "{name}: {stock} moved nothing, so this is vacuous"
                );
            }
            let r = water_resolver(200.0, 5.0);
            let env = r.bind(&s, 1.0);
            assert_flow_balanced_default(
                &flow.evaluate(&s, &env, 1.0).expect("evaluate"),
                &s.stocks,
            )
            .unwrap_or_else(|e| panic!("{name} is unbalanced: {e:?}"));
        }
    }

    /// `IRGW = min(capacity · A · dt, max(0, TTSW − ATSW))` — BOTH limbs, at the
    /// crossover, so neither can be dropped without a red test.
    ///
    /// ⚠ Both limbs are LIVE on the frozen roster (a `panic!` probe in either stops seven
    /// tests), and neither was asserted: replacing the whole `min` with the bare capacity
    /// reddened one test, about the root-zone capture. The forcing changed MEANING on
    /// 2026-08-12 — mm/day applied became mm/day AVAILABLE — so what the capacity limb
    /// means is "the irrigation system's throughput", not "the amount applied".
    /// Mirrors `test_irrigation_takes_the_smaller_of_capacity_and_deficit` and
    /// `test_irrigation_leg_is_rate_times_area`.
    #[test]
    fn irrigation_takes_the_smaller_of_the_system_capacity_and_the_deficit() {
        let full = science::transpirable_capacity(TEST_DEPTH, EXTR, 1.0); // 130 kg
                                                                          // (a) deficit 4 kg against a 5 kg/day capacity ⇒ the DEFICIT binds.
        let s = water_only_state(full - 4.0, TEST_DEPTH, 0.0, 0.0);
        let legs = water_legs(&irrigation_flow(1.0), &s, 200.0, 5.0, 1.0);
        assert!((legs[SOIL_WATER] - 4.0).abs() <= 1e-12 * 4.0, "{:?}", legs);
        // (b) deficit 9 kg against the same capacity ⇒ the CAPACITY binds.
        let s = water_only_state(full - 9.0, TEST_DEPTH, 0.0, 0.0);
        let legs = water_legs(&irrigation_flow(1.0), &s, 200.0, 5.0, 1.0);
        assert!((legs[SOIL_WATER] - 5.0).abs() <= 1e-12 * 5.0, "{:?}", legs);
        // (c) The capacity limb carries the AREA: 5 mm/day over 2 m² is 10 kg/day, and
        // the 2 m² zone holds 260 kg so the deficit is nowhere near binding.
        let wide = water_only_state(15.0, TEST_DEPTH, 0.0, 0.0);
        let legs = water_legs(&irrigation_flow(2.0), &wide, 200.0, 5.0, 1.0);
        assert!(
            (legs[SOIL_WATER] - 10.0).abs() <= 1e-12 * 10.0,
            "{:?}",
            legs
        );
        assert!(
            (legs[IRRIGATION_SUPPLY] + 10.0).abs() <= 1e-12 * 10.0,
            "the supply leg must mirror the soil leg"
        );
    }

    /// A full zone takes nothing, an over-full zone does not run the supply BACKWARDS,
    /// and a zero capacity is a hard off whatever the deficit.
    ///
    /// The first is [F] Eqn 14.8's own limb and the reason `Drainage` is inert on the
    /// frozen roster — the supply tracks the deficit instead of pushing a flat rate into
    /// a bucket with a bottom, so "water non-limiting" became a checkable claim rather
    /// than a label. The last is how an irrigation-cut window works, and it had to survive
    /// the rate → capacity re-interpretation unchanged.
    /// Mirrors `test_irrigation_stops_at_the_drained_upper_limit` and
    /// `test_a_zero_capacity_is_still_a_hard_off`.
    #[test]
    fn irrigation_stops_at_the_drained_upper_limit_and_a_zero_capacity_is_a_hard_off() {
        let full = science::transpirable_capacity(TEST_DEPTH, EXTR, 2.0);
        let at_dul = water_only_state(full, TEST_DEPTH, 0.0, 0.0);
        let legs = water_legs(&irrigation_flow(2.0), &at_dul, 200.0, 5.0, 1.0);
        assert_eq!(legs[SOIL_WATER], 0.0);
        assert_eq!(legs[IRRIGATION_SUPPLY], 0.0);
        // An OVER-full zone: still zero, and emphatically not a negative (which would
        // pump water out of the soil and into the boundary supply).
        let over = water_only_state(full * 2.0, TEST_DEPTH, 0.0, 0.0);
        let legs = water_legs(&irrigation_flow(2.0), &over, 200.0, 5.0, 1.0);
        assert_eq!(legs[SOIL_WATER], 0.0);
        assert_eq!(legs[IRRIGATION_SUPPLY], 0.0);
        // A zero capacity against a DEEP deficit: only the zero can stop it.
        let parched = water_only_state(1.0, TEST_DEPTH, 0.0, 0.0);
        let legs = water_legs(&irrigation_flow(1.0), &parched, 200.0, 0.0, 1.0);
        assert_eq!(legs[SOIL_WATER], 0.0);
    }

    /// `Condensation` and `Recycling` are first-order in their OWN donor pool — the
    /// engineered-condenser framing, and the structural positivity that keeps the
    /// arbitration backstop out of the closed water ring.
    ///
    /// ⚠ Pinned against a hand rate rather than the committed one, so the LAW is the
    /// subject and not the parameter. The before-battery is the reason both halves are
    /// here: doubling the condensation rate reddened NOTHING in the lib binary, and
    /// making `Recycling` read `soil_water` instead of `condensate` — a change of donor
    /// that stays perfectly balanced — reddened nine tests, every one of them a
    /// compensation-point or chamber gate. A balanced mutation is invisible to
    /// conservation by construction; only the rate law itself can catch it.
    /// Mirrors `test_condensation_flux_is_first_order_in_vapor`,
    /// `test_recycling_flux_is_first_order_in_condensate`,
    /// `test_fluxes_are_zero_at_empty_pools` and `test_cycle_flows_are_dt_linear`.
    #[test]
    fn the_two_cycle_flows_are_first_order_in_their_own_donor_pool() {
        let cond = Condensation {
            id: "biosphere.condensation".to_string(),
            water_vapor: WATER_VAPOR.to_string(),
            condensate: CONDENSATE.to_string(),
            condensation_rate: 0.5,
        };
        let rec = Recycling {
            id: "biosphere.recycling".to_string(),
            condensate: CONDENSATE.to_string(),
            soil_water: SOIL_WATER.to_string(),
            recycling_rate: 0.5,
        };
        // k = 0.5 per day: half the standing pool per day, linear in the pool.
        for (vapor, condensate) in [(2.0, 2.0), (4.0, 4.0)] {
            let s = water_only_state(100.0, TEST_DEPTH, vapor, condensate);
            let c = water_legs(&cond, &s, 200.0, 0.0, 1.0);
            assert!(
                (c[CONDENSATE] - 0.5 * vapor).abs() <= 1e-12 * vapor,
                "condensation {} != 0.5 x {vapor}",
                c[CONDENSATE]
            );
            assert_eq!(c[WATER_VAPOR], -c[CONDENSATE], "condensation is unbalanced");
            let r = water_legs(&rec, &s, 200.0, 0.0, 1.0);
            assert!(
                (r[SOIL_WATER] - 0.5 * condensate).abs() <= 1e-12 * condensate,
                "recycling {} != 0.5 x {condensate}",
                r[SOIL_WATER]
            );
            assert_eq!(r[CONDENSATE], -r[SOIL_WATER], "recycling is unbalanced");
        }
        // ⚠ EACH READS ITS OWN DONOR. With a full 100 kg root zone and an empty
        // condensate pool, `Recycling` must move NOTHING — the mutation that reads
        // `soil_water` instead would move 50 kg here and still balance perfectly.
        let s = water_only_state(100.0, TEST_DEPTH, 6.0, 0.0);
        assert_eq!(water_legs(&rec, &s, 200.0, 0.0, 1.0)[SOIL_WATER], 0.0);
        // ...and symmetrically for condensation against an empty vapour pool.
        let s = water_only_state(100.0, TEST_DEPTH, 0.0, 6.0);
        assert_eq!(water_legs(&cond, &s, 200.0, 0.0, 1.0)[CONDENSATE], 0.0);
        // Self-limiting: no standing pool, no flux, so positivity is structural.
        let empty = water_only_state(0.0, TEST_DEPTH, 0.0, 0.0);
        assert_eq!(water_legs(&cond, &empty, 200.0, 0.0, 1.0)[CONDENSATE], 0.0);
        assert_eq!(water_legs(&rec, &empty, 200.0, 0.0, 1.0)[SOIL_WATER], 0.0);
        // dt-linear, like every other rate law in the tree.
        let s = water_only_state(100.0, TEST_DEPTH, 4.0, 4.0);
        assert_eq!(
            water_legs(&cond, &s, 200.0, 0.0, 0.5)[CONDENSATE],
            0.5 * water_legs(&cond, &s, 200.0, 0.0, 1.0)[CONDENSATE]
        );
        assert_eq!(
            water_legs(&rec, &s, 200.0, 0.0, 0.5)[SOIL_WATER],
            0.5 * water_legs(&rec, &s, 200.0, 0.0, 1.0)[SOIL_WATER]
        );
    }

    // --- batch D: the carbon BUDGET, and the two halves of the stem reserve ------
    //
    // ⚠ WHY THIS HALF OF THE BATCH IS THE DANGEROUS ONE, stated once. Almost every claim
    // below is about a REDISTRIBUTION — the partition table splitting one increment four
    // ways, `fstr` moving part of the stem leg into shielded starch, the maintenance
    // shortfall burning organs in proportion, the reserve draining into the grain. Every
    // one of those keeps each flow's legs summing exactly as they did, so the
    // biosphere's strongest machinery — `assert_flow_balanced`, the conservation
    // assertion on every step, the boundary ledger — is blind to ALL of them by
    // construction. §5ad's battery for this batch was built around that: of eight
    // sum-preserving mutations, exactly two reddened anything whose subject was the
    // mutated mechanism, and both of those were value or leg-SHAPE pins rather than
    // rate laws. These tests have to state the rate law itself.

    /// Put carbon in the stem RESERVE, which starts empty in `state()`.
    fn with_reserve(mut s: State, amount: f64) -> State {
        s.stocks.get_mut(RESERVE).expect("reserve stock").amount = amount;
        s
    }

    /// Thermal time that puts `development_stage` exactly at `dvs`.
    ///
    /// Vegetative `DVS = tt / tsum_anthesis`; reproductive
    /// `DVS = 1 + (tt − tsum_anthesis)/tsum_maturity`, capped at 2. With the committed
    /// (1100, 750) that is 550 for DVS 0.5, 1100 for 1.0, 1475 for 1.5, 1850 for 2.0.
    fn thermal_time_for(dvs: f64) -> f64 {
        let p = params::phenology();
        if dvs <= 1.0 {
            dvs * p.tsum_anthesis
        } else {
            p.tsum_anthesis + (dvs - 1.0) * p.tsum_maturity
        }
    }

    fn open_growth_respiration() -> GrowthRespiration {
        GrowthRespiration {
            id: "biosphere.growth_respiration".to_string(),
            ctx: ctx_open(),
            co2_atmos: CO2.to_string(),
            co2_resp: "boundary.co2".to_string(),
        }
    }

    fn open_maintenance() -> MaintenanceRespiration {
        MaintenanceRespiration {
            id: "biosphere.maintenance_respiration".to_string(),
            ctx: ctx_open(),
            co2_atmos: CO2.to_string(),
            co2_resp: "boundary.co2".to_string(),
            o2_pool: None,
            air_mol: None,
        }
    }

    fn remobilization(p: params::StemReserveParams) -> StemRemobilization {
        StemRemobilization {
            id: "biosphere.stem_remobilization".to_string(),
            stem_reserve_c: RESERVE.to_string(),
            storage_c: STORAGE.to_string(),
            thermal_time_aux: THERMAL_TIME.to_string(),
            pheno: params::phenology(),
            params: p,
        }
    }

    /// `Allocation` wired with a live stem reserve.
    fn allocation_with_reserve(fstr: f64, cessation: f64) -> Allocation {
        Allocation {
            stem_reserve_c: Some(RESERVE.to_string()),
            fstr,
            reserve_cessation_dvs: cessation,
            ..allocation(ctx_open(), None)
        }
    }

    fn budget_of(ctx: &CarbonContext, s: &State, par: f64) -> (f64, f64, f64) {
        let r = resolver(par, 400.0);
        let env = r.bind(s, 1.0);
        ctx.budget(s, &env).expect("budget")
    }

    /// The shared budget is the three standalone rate laws, composed.
    ///
    /// `GASS` is the canopy aggregator at the state's own LAI and limitation, `MRES` is
    /// the maintenance flux at the maintained biomass, and `available` is their clamped
    /// difference. Recomposed here from `science.rs` rather than compared against a
    /// number this tree produced — the same shape as the standalone equation tests, one
    /// layer up. This is what stops the three budget-coupled flows from silently
    /// disagreeing about the day they are all spending.
    ///
    /// ⚠⚠ READ THE SCOPE BEFORE TRUSTING THIS ONE. It is a **composition** check and it
    /// is structurally BLIND to anything inside the functions it composes: it calls the
    /// same `science.rs` entry points the flow does, so a wrong rate law moves both sides
    /// identically and this test stays green. Measured, not argued — it reddened under
    /// **none** of the fifteen mutations in §5ah's battery, including the two that broke
    /// `maintenance_respiration_flux` and `available_for_growth` outright. Those are owned
    /// by the equation tests in `science.rs`.
    ///
    /// What it DOES own is everything between the functions, and that was measured too:
    /// maintaining the LEAF instead of the whole biomass (a wrong argument) and
    /// transposing `available_for_growth`'s two arguments both redden it. *A composition
    /// check is real coverage of the wiring and no coverage at all of the arithmetic; the
    /// after-battery could not see the difference, because it only ever asked whether each
    /// MUTATION was caught and never whether each new TEST was reachable.*
    /// Mirrors `tests/test_carbon_budget.py::test_context_budget_matches_standalone_rate_laws`.
    #[test]
    fn the_shared_carbon_budget_is_the_standalone_rate_laws_recomposed() {
        let ctx = ctx_open();
        let s = with_storage(growing_state(), 2.0);
        let (gass, mres, available) = budget_of(&ctx, &s, 800.0);

        let leaf = s.stocks[LEAF].amount;
        let biomass = leaf + s.stocks[STEM].amount + s.stocks[ROOT].amount;
        let lai = science::leaf_area_index(leaf, ctx.canopy.sla_per_mol_c, ctx.ground_area);
        let f_water = science::soil_water_stress(
            s.stocks[SOIL_WATER].amount,
            TEST_DEPTH,
            EXTR,
            ctx.ground_area,
            WSSG,
        );
        let f_n = science::nitrogen_stress_factor(
            s.stocks[PLANT_N].amount,
            biomass,
            ctx.nitro.n_residual_per_mol_c,
            ctx.nitro.n_critical_per_mol_c,
        );
        let want_gass = science::canopy_assimilation(
            800.0,
            lai,
            400.0,
            20.0,
            light_path::SECONDS_PER_DAY,
            &ctx.photo,
            &ctx.canopy,
            ctx.ground_area,
            f_water * f_n,
        );
        let want_mres = science::maintenance_respiration_flux(biomass, 20.0, &ctx.resp);
        assert_eq!(gass.to_bits(), want_gass.to_bits(), "GASS");
        assert_eq!(mres.to_bits(), want_mres.to_bits(), "MRES");
        assert_eq!(
            available.to_bits(),
            science::available_for_growth(want_gass, want_mres).to_bits(),
            "available"
        );
        assert!(
            available > 0.0,
            "the fixture must be a surplus day, not a no-op"
        );
    }

    /// The limitation is the PRODUCT of the two stress factors, and BOTH of them bite at
    /// the operating point this test constructs.
    ///
    /// ⚠⚠ THE FIXTURE IS THE TEST, and the first version of it made this gate INERT. It
    /// asserted `lim == f_water · f_n` on a state whose `plant_n` was 1.0 against 5 mol C
    /// of biomass — a concentration a hundred times the critical one, so `f_n` saturated
    /// at exactly 1.0 and the claim degenerated to `lim == f_water`. Measured: deleting
    /// the nitrogen factor from the product outright reddened **one** test in 282, and it
    /// was not this one. A test named for a product cannot see one of its factors when
    /// that factor is pinned at the multiplicative identity.
    ///
    /// So the stressed state is DERIVED from the ramp instead of guessed. `f_N` is linear
    /// between `n_residual` and `n_critical` per mol C, so a plant nitrogen of
    /// `midpoint · (leaf + stem + root)` puts the concentration exactly halfway and
    /// `f_N = 0.5`. ⚠ Derived from the loaded params rather than written as a literal,
    /// because the first draft of this fixture hardcoded 0.001/0.002 — the PYTHON TEST
    /// FIXTURE's thresholds, not the committed file's (0.005/0.015, folded) — and read
    /// `f_N = 1.0` for a state it had just declared stressed. A test that constructs a
    /// stressed state must construct it from the numbers the code will actually use.
    ///
    /// The same value does a second job, and it holds for any thresholds: against the
    /// LEAF alone (3 of the 5 mol C) the concentration is 5/3 of the midpoint, which is
    /// above `n_critical` whenever the midpoint is, so `f_N` would read 1.0. One fixture
    /// therefore separates "the nitrogen factor is missing" from "its denominator is the
    /// wrong organ set", and both were measured to redden it.
    ///
    /// Mirrors `test_context_limitation_is_f_water_times_f_n` and
    /// `test_context_limitation_is_one_at_the_non_limiting_point`.
    #[test]
    fn the_limitation_is_the_product_and_both_factors_actually_bite() {
        let ctx = ctx_open();
        let r = resolver(800.0, 400.0);
        let f_n_of = |s: &State| {
            let biomass = s.stocks[LEAF].amount + s.stocks[STEM].amount + s.stocks[ROOT].amount;
            science::nitrogen_stress_factor(
                s.stocks[PLANT_N].amount,
                biomass,
                ctx.nitro.n_residual_per_mol_c,
                ctx.nitro.n_critical_per_mol_c,
            )
        };
        let f_water_of = |s: &State| {
            science::soil_water_stress(
                s.stocks[SOIL_WATER].amount,
                TEST_DEPTH,
                EXTR,
                ctx.ground_area,
                WSSG,
            )
        };
        let lim_of = |s: &State| ctx.limitation(s, &r.bind(s, 1.0)).expect("limitation");
        // The exact midpoint of f_N's ramp, over the fixture's 5 mol C of maintained
        // biomass. Derived, never a literal — see the note above.
        let midpoint = (ctx.nitro.n_residual_per_mol_c + ctx.nitro.n_critical_per_mol_c) / 2.0;
        let half_stress_n = midpoint * 5.0;
        // ⚠ A DERIVED half is not a BIT half: `midpoint · 5 / 5` round-trips to
        // 0.5000000000000001, so the ramp assertions carry a tolerance while the
        // saturated ones (1.0) and the water factor stay EXACT — those really are exact,
        // and weakening them would hide a real drift.
        let half = |got: f64, what: &str| {
            assert!(
                (got - 0.5).abs() <= 1e-12,
                "{what}: got {got}, want 0.5 (the ramp midpoint)"
            );
        };

        // (a) Neither factor bites: FTSW 0.77 against wssg 0.30, N far above critical.
        let easy = growing_state();
        assert_eq!(f_n_of(&easy), 1.0);
        assert_eq!(f_water_of(&easy), 1.0);
        assert_eq!(lim_of(&easy), 1.0, "the non-limiting point is exactly one");

        // (b) NITROGEN alone bites, at the exact midpoint of its ramp.
        let mut lean = growing_state();
        lean.stocks.get_mut(PLANT_N).expect("plant N").amount = half_stress_n;
        half(
            f_n_of(&lean),
            "the fixture must sit ON the ramp, not past it",
        );
        assert_eq!(f_water_of(&lean), 1.0, "only nitrogen may bite here");
        half(lim_of(&lean), "lim == f_water · f_n == 1.0 · 0.5");
        // ⚠ The denominator is leaf + stem + root, NOT the leaf. Against the leaf alone
        // (3 of the 5 mol C) this same state reads 5/3 of the midpoint, which is above
        // `n_critical` for any thresholds — so f_N would read 1.0 and the assertion above
        // is exactly the one that separates the two organ sets.
        let leaf_only = science::nitrogen_stress_factor(
            half_stress_n,
            easy.stocks[LEAF].amount,
            ctx.nitro.n_residual_per_mol_c,
            ctx.nitro.n_critical_per_mol_c,
        );
        assert_eq!(
            leaf_only, 1.0,
            "a leaf-only denominator would NOT be stressed"
        );

        // (c) WATER alone bites: FTSW = 0.5 · wssg = 0.15 of a 130 kg zone is 19.5 kg,
        // which is WSFG = 0.5 exactly.
        let mut dry = growing_state();
        dry.stocks.get_mut(SOIL_WATER).expect("soil water").amount = 19.5;
        assert_eq!(f_water_of(&dry), 0.5);
        assert_eq!(f_n_of(&dry), 1.0, "only water may bite here");
        assert_eq!(lim_of(&dry), 0.5);

        // (d) BOTH bite, and the composition is a product rather than a min or a mean —
        // which is the distinction (b) and (c) alone cannot make, since 0.5 is 0.5 under
        // all three rules. 0.5 · 0.5 = 0.25; a `min` would give 0.5 and a mean 0.5.
        let mut both = growing_state();
        both.stocks.get_mut(PLANT_N).expect("plant N").amount = half_stress_n;
        both.stocks.get_mut(SOIL_WATER).expect("soil water").amount = 19.5;
        half(f_n_of(&both), "nitrogen at the ramp midpoint");
        assert_eq!(f_water_of(&both), 0.5);
        assert!(
            (lim_of(&both) - 0.25).abs() <= 1e-12,
            "a product, not a min (0.5) or a mean (0.5): got {}",
            lim_of(&both)
        );
    }

    /// The growth-respiration leg is `(1 − Yg) · available`, on an OPEN-field wiring.
    ///
    /// ⚠⚠ THIS TEST EXISTS BECAUSE NOTHING ELSE IN THE TREE DOES. §5ad's battery replaced
    /// the complement `(1 − Yg)` with `Yg` — respiring three times as much carbon as the
    /// law says, with the leg still balanced — and reddened **zero** tests of
    /// `cargo test -p domains --lib`. Not "no test about growth respiration": no test at
    /// all. It moves carbon between two BOUNDARY stocks, so no organ, no chamber gas and
    /// no conserved quantity moves with it, and only the goldens' committed bytes see it.
    /// Mirrors `test_carbon_budget.py::test_growth_flow_leg_is_the_composed_loss_and_pins_the_step6_literal`.
    #[test]
    fn the_growth_respiration_leg_is_the_complement_of_the_growth_efficiency() {
        let ctx = ctx_open();
        let s = growing_state();
        let (_, _, available) = budget_of(&ctx, &s, 800.0);
        let legs = legs_of(&open_growth_respiration(), &s, 800.0);
        let want = (1.0 - ctx.resp.growth_efficiency) * available;
        assert_eq!(legs["boundary.co2"].to_bits(), want.to_bits(), "GRES leg");
        assert_eq!(legs[CO2].to_bits(), (-want).to_bits(), "its source leg");
        // The complement is the SMALLER share at the committed Yg = 0.75, which is what
        // makes the wrong-way-round form (Yg instead of 1 − Yg) a 3x error rather than a
        // rounding one. Asserted so the direction is pinned, not just the magnitude.
        assert!(
            want < ctx.resp.growth_efficiency * available,
            "at Yg > 0.5 the respired share must be the smaller one"
        );
    }

    /// `Allocation`'s four organ legs ARE `partition(Yg · available, DVS)`, and the CO₂
    /// leg is their exact sum.
    ///
    /// The budget is recomposed rather than quoted, so the test states the rule
    /// (`DMI = Yg · available`, split by the table at this state's DVS) rather than a
    /// number this tree produced.
    /// Mirrors `test_allocation_legs_are_the_partitioned_increment` and
    /// `test_allocation_dmi_agrees_with_growth_resp_budget`.
    #[test]
    fn the_allocation_legs_are_the_partitioned_increment_and_the_co2_leg_is_their_sum() {
        let ctx = ctx_open();
        // DVS 1.5, so all four legs including the grain are nonzero.
        let s = state(3.0, 1.0, 1.0, 0.4, 0.21 * AIR_MOL, thermal_time_for(1.5));
        let (_, _, available) = budget_of(&ctx, &s, 800.0);
        let dmi = ctx.resp.growth_efficiency * available;
        let (leaf, stem, root, storage) = science::partition(dmi, 1.5, &params::allocation().table);

        let legs = legs_of(&allocation(ctx_open(), None), &s, 800.0);
        assert_eq!(legs[LEAF].to_bits(), leaf.to_bits(), "leaf leg");
        assert_eq!(legs[STEM].to_bits(), stem.to_bits(), "stem leg");
        assert_eq!(legs[ROOT].to_bits(), root.to_bits(), "root leg");
        assert_eq!(legs[STORAGE].to_bits(), storage.to_bits(), "grain leg");
        assert!(storage > 0.0, "the fixture must be past anthesis");
        assert_eq!(
            legs[CO2].to_bits(),
            (-(leaf + stem + root + storage)).to_bits(),
            "the CO2 leg is the exact sum of the four organ legs"
        );
        // DMI and the growth-respiration loss together are the whole day's assimilate:
        // Yg · available + (1 − Yg) · available == available.
        let gres = legs_of(&open_growth_respiration(), &s, 800.0)["boundary.co2"];
        let sum = dmi + gres;
        assert!(
            (sum - available).abs() <= 1e-12 * available,
            "DMI + GRES = {sum} != available {available}"
        );
    }

    /// Open-field maintenance covers what it can from the atmosphere and burns the rest
    /// out of the organs IN PROPORTION to each organ's carbon.
    ///
    /// ⚠ The proportional split is a pure redistribution — replacing `organ_c / biomass`
    /// with a flat one-third burns the identical TOTAL and leaves every leg balanced.
    /// §5ad measured that mutation reddening one test in 259, and that one is about a
    /// deep-water crop. So the shares are asserted individually, against the organ
    /// ratios, and not merely through their sum.
    /// Mirrors `test_maintenance_deficit_day_draws_the_shortfall_from_organs` and
    /// `test_maintenance_partial_deficit_splits_covered_and_shortfall`.
    #[test]
    fn open_field_maintenance_burns_the_shortfall_in_proportion_to_each_organ() {
        let ctx = ctx_open();
        // A DARK deficit day: no PAR, so GASS is 0 and the whole of MRES is a shortfall.
        // Organs 3 : 1 : 1 make the three shares 0.6, 0.2, 0.2 of the burn.
        let s = state(3.0, 1.0, 1.0, 0.4, 0.21 * AIR_MOL, 550.0);
        let (gass, mres, _) = budget_of(&ctx, &s, 0.0);
        assert_eq!(gass, 0.0, "the fixture must be a genuine dark day");
        let legs = legs_of(&open_maintenance(), &s, 0.0);
        let biomass = 3.0 + 1.0 + 1.0;
        for (id, organ_c) in [(LEAF, 3.0), (STEM, 1.0), (ROOT, 1.0)] {
            let want = -(mres * (organ_c / biomass));
            assert_eq!(legs[id].to_bits(), want.to_bits(), "{id} share of the burn");
        }
        // Nothing is drawn from the atmosphere on a day with no assimilate, and the
        // respired leg is the whole burn.
        assert_eq!(
            legs.get(CO2).copied().unwrap_or(0.0),
            0.0,
            "no covered part"
        );
        assert!((legs["boundary.co2"] - mres).abs() <= 1e-12 * mres);

        // A SURPLUS day is the other limb: covered entirely from the atmosphere, no
        // organ touched at all. Asserted here so the covered/shortfall cap is pinned
        // from both sides — `covered = min(GASS, MRES)` uncapped reddened 7 tests.
        let legs = legs_of(&open_maintenance(), &s, 800.0);
        let (gass, mres, _) = budget_of(&ctx, &s, 800.0);
        assert!(gass > mres, "the fixture must be a genuine surplus day");
        assert_eq!(legs[CO2].to_bits(), (-mres).to_bits(), "covered == MRES");
        for id in [LEAF, STEM, ROOT] {
            assert_eq!(legs.get(id).copied().unwrap_or(0.0), 0.0, "{id} untouched");
        }
    }

    /// The stem-reserve FORMATION diverts `fstr` of the stem leg and moves nothing else.
    ///
    /// ⚠⚠ §5ad's battery deleted this split entirely — the reserve receiving nothing, the
    /// stem keeping the whole leg — and reddened 2 tests of 259, neither about stem
    /// reserves. The mechanism has its own 1,643-line Python file and shipped only on an
    /// explicit call from the user, and in Rust it was guarded by nothing that names it.
    ///
    /// The invariance of `organ_total` is the second half of the claim and it is
    /// CORRECTNESS rather than tidiness: the diverted starch is still carbon fixed out of
    /// the atmosphere, so the CO₂ source leg and the O₂ release must not move when the
    /// split turns on. Asserted by comparing against the same flow with `fstr = 0`.
    /// Mirrors the formation half of `tests/test_stem_reserves.py`.
    #[test]
    fn the_reserve_formation_diverts_the_stem_share_and_moves_no_other_leg() {
        let fstr = params::stem_reserves().remobilizable_fraction;
        let s = state(3.0, 1.0, 1.0, 0.4, 0.21 * AIR_MOL, thermal_time_for(0.5));
        let plain = legs_of(&allocation(ctx_open(), Some(O2.to_string())), &s, 800.0);
        let split = legs_of(
            &Allocation {
                o2_pool: Some(O2.to_string()),
                ..allocation_with_reserve(fstr, 2.0)
            },
            &s,
            800.0,
        );
        let stem_leg = plain[STEM];
        assert!(stem_leg > 0.0, "the fixture must actually grow stem");
        assert_eq!(
            split[RESERVE].to_bits(),
            (fstr * stem_leg).to_bits(),
            "the reserve gets exactly fstr of the stem leg"
        );
        assert_eq!(
            split[STEM].to_bits(),
            (stem_leg - fstr * stem_leg).to_bits(),
            "and the stem keeps the complement, by subtraction"
        );
        for id in [LEAF, ROOT, STORAGE, CO2, O2] {
            assert_eq!(
                split[id].to_bits(),
                plain[id].to_bits(),
                "{id} must be bit-identical with and without the split"
            );
        }
    }

    /// Both halves of the reserve STOP at `cessation_dvs`, and the bound is strict.
    ///
    /// ⚠⚠ The strictness is the load-bearing part and NOTHING measured it. Our DVS *caps*
    /// at 2.0 rather than growing past it, so a `<=` leaves both halves running for the
    /// whole post-maturity tail — 11 steps on `open_season`, two YEARS on
    /// `sealed_chamber`, which never re-sows. §5ad's battery loosened each half in turn:
    /// **zero** tests of `-p domains --lib` reddened for either, and applied together
    /// they moved only committed golden bytes.
    ///
    /// Pinned at three stages: inside the window (both halves act), exactly AT the
    /// cessation (both stop), and above it (both stop and stay stopped).
    /// Mirrors `test_the_transfer_actually_STOPS_and_the_ungated_control_reproduces`,
    /// `test_the_two_halves_share_ONE_cessation_number_in_the_shipped_wiring` and
    /// `test_a_forgotten_cessation_fails_CLOSED_rather_than_running_unbounded`.
    #[test]
    fn both_halves_of_the_reserve_stop_at_the_cessation_and_the_bound_is_strict() {
        let p = params::stem_reserves();
        let flow = remobilization(p);
        let filled = |dvs: f64| {
            with_reserve(
                state(3.0, 1.0, 1.0, 0.4, 0.21 * AIR_MOL, thermal_time_for(dvs)),
                10.0,
            )
        };
        // Inside the window the drain acts...
        let inside = legs_of(&flow, &filled(1.5), 800.0);
        assert!(inside[RESERVE] < 0.0 && inside[STORAGE] > 0.0);
        // ...and exactly AT the cessation it emits no legs at all — not a zero leg.
        assert!(
            legs_of(&flow, &filled(p.cessation_dvs), 800.0).is_empty(),
            "the drain must stop AT the cessation, not after it"
        );
        // DVS caps at 2.0, so "above" is reached by any later thermal time.
        assert!(legs_of(&flow, &filled(p.cessation_dvs + 5.0), 800.0).is_empty());
        // Below the trigger it has not started yet — the window is half-open at BOTH
        // ends, and a test that only checked the top would pass on an ungated flow.
        assert!(legs_of(&flow, &filled(p.trigger_dvs - 0.01), 800.0).is_empty());

        // The FILL shares the same number, so the two stop on the same step. A drain
        // that stopped alone would leave the dead stem stashing starch forever.
        let fill = Allocation {
            ..allocation_with_reserve(p.remobilizable_fraction, p.cessation_dvs)
        };
        assert!(
            legs_of(&fill, &filled(1.5), 800.0)[RESERVE] > 0.0,
            "the fill must be live inside the window"
        );
        let at_cessation = legs_of(&fill, &filled(p.cessation_dvs), 800.0);
        assert_eq!(
            at_cessation.get(RESERVE).copied().unwrap_or(0.0),
            0.0,
            "the fill must stop at the same stage the drain does"
        );
        // Fail-CLOSED: a wiring that supplies a stock and a fraction but forgets the
        // bound gets NO split rather than an unbounded one (DVS >= 0 always).
        let forgotten = allocation_with_reserve(p.remobilizable_fraction, 0.0);
        assert_eq!(
            legs_of(&forgotten, &filled(0.5), 800.0)
                .get(RESERVE)
                .copied()
                .unwrap_or(0.0),
            0.0,
            "a forgotten cessation must fail closed"
        );
    }

    /// The drain runs stem → grain, first-order on the STANDING reserve.
    ///
    /// ⚠⚠ §5ad's battery reversed this flow — the grain feeding the stem — and reddened
    /// **28** tests of 259, not one of which is about stem reserves. Twenty-eight reds
    /// with zero information about what broke is the same reading as a golden red: a
    /// number moved. The direction is therefore asserted by NAME here, not inferred from
    /// a trajectory.
    ///
    /// First-order means the draw is donor-controlled and therefore self-limiting, which
    /// is why the Euler arbitration backstop is structurally unreachable on it: doubling
    /// the standing reserve exactly doubles the flux.
    /// Mirrors `test_the_drain_rate_is_BIT_INERT_on_carbon_and_only_relabels` (the form
    /// half) and the remobilization block of `tests/test_stem_reserves.py`.
    #[test]
    fn the_remobilization_drains_the_reserve_into_the_grain_and_is_first_order() {
        let p = params::stem_reserves();
        let flow = remobilization(p);
        let at = |reserve: f64| {
            with_reserve(
                state(3.0, 1.0, 1.0, 0.4, 0.21 * AIR_MOL, thermal_time_for(1.5)),
                reserve,
            )
        };
        let legs = legs_of(&flow, &at(10.0), 800.0);
        let want = p.remobilization_rate * 10.0;
        assert_eq!(
            legs[RESERVE].to_bits(),
            (-want).to_bits(),
            "the reserve pays"
        );
        assert_eq!(
            legs[STORAGE].to_bits(),
            want.to_bits(),
            "the grain receives"
        );
        assert_eq!(legs[RESERVE] + legs[STORAGE], 0.0, "an internal transfer");

        // First-order: twice the standing reserve, exactly twice the flux.
        let double = legs_of(&flow, &at(20.0), 800.0);
        assert_eq!(double[STORAGE].to_bits(), (2.0 * want).to_bits());
        // An empty reserve emits no legs at all rather than a zero pair.
        assert!(legs_of(&flow, &at(0.0), 800.0).is_empty());
        // Donor-controlled, so it can never overdraw its own donor in one step at the
        // committed rate — the reason arbitration is unreachable here.
        assert!(
            want < 10.0,
            "a first-order draw must be a fraction of the stock"
        );
    }

    /// All three budget-coupled flows read ONE budget, clamp together in the dark, and
    /// scale linearly with `dt`.
    ///
    /// The structural claim is that they cannot drift: each holds a clone of the same
    /// `CarbonContext`, so a state that produces no assimilate produces no growth, no
    /// growth respiration and no allocation — from the same arithmetic, not from three
    /// agreeing accidents. `dt`-linearity is asserted on the same three flows in one
    /// test because it is one property of the whole budget, and a per-flow version would
    /// pass while two of them disagreed about the day.
    /// Mirrors `test_three_flows_share_one_budget`, the three `*_clamps_to_zero_in_the_dark`
    /// tests and the three `*_scales_linearly_with_dt` tests.
    #[test]
    fn the_three_budget_flows_share_one_budget_clamp_together_and_scale_with_dt() {
        let s = growing_state();
        // The dark: GASS is 0, so `available` clamps and BOTH growth-side flows vanish.
        // Maintenance does NOT — it becomes a pure organ burn, which is the asymmetry.
        let (gass, _, available) = budget_of(&ctx_open(), &s, 0.0);
        assert_eq!((gass, available), (0.0, 0.0));
        for id in [LEAF, STEM, ROOT, STORAGE] {
            assert_eq!(
                legs_of(&allocation(ctx_open(), None), &s, 0.0)
                    .get(id)
                    .copied()
                    .unwrap_or(0.0),
                0.0,
                "{id} must not grow in the dark"
            );
        }
        assert_eq!(
            legs_of(&open_growth_respiration(), &s, 0.0)["boundary.co2"],
            0.0
        );
        assert!(
            legs_of(&open_maintenance(), &s, 0.0)["boundary.co2"] > 0.0,
            "maintenance is NOT clamped in the dark — the plant still pays rent"
        );

        // dt-linearity, on all three at once.
        let r = resolver(800.0, 400.0);
        let flows: Vec<Box<dyn Flow>> = vec![
            Box::new(allocation(ctx_open(), None)),
            Box::new(open_growth_respiration()),
            Box::new(open_maintenance()),
        ];
        for flow in &flows {
            let env = r.bind(&s, 1.0);
            let full: BTreeMap<String, f64> = flow
                .evaluate(&s, &env, 1.0)
                .expect("dt 1")
                .legs
                .iter()
                .map(|l| (l.stock.clone(), l.amount))
                .collect();
            let half: BTreeMap<String, f64> = flow
                .evaluate(&s, &env, 0.5)
                .expect("dt 0.5")
                .legs
                .iter()
                .map(|l| (l.stock.clone(), l.amount))
                .collect();
            assert_eq!(full.len(), half.len(), "{} leg count", flow.type_name());
            for (id, amount) in &full {
                assert_eq!(
                    half[id].to_bits(),
                    (0.5 * amount).to_bits(),
                    "{} leg {id} must halve with dt",
                    flow.type_name()
                );
            }
        }
    }

    /// The limitation scales growth and allocation IDENTICALLY.
    ///
    /// Both flows multiply the same `available`, so a stress that halves assimilation
    /// must move both by the same ratio — the property that says the budget is shared
    /// rather than recomputed twice with a drifting argument.
    ///
    /// ⚠ Its scope, measured rather than assumed (same pass that found the limitation
    /// fixture inert). It reddens when the limitation reaches only ONE of the two flows,
    /// and it is blind to a CONSTANT that reaches only one: scaling `Allocation`'s DMI by
    /// a factor `GrowthRespiration` does not share leaves the two ratios identical, and
    /// that mutation left this test green. A ratio test sees a factor that VARIES with the
    /// state and cannot see one that does not.
    /// Mirrors `test_limitation_scales_growth_and_allocation_identically`.
    #[test]
    fn the_limitation_scales_growth_and_allocation_by_the_same_ratio() {
        let s = with_storage(growing_state(), 2.0);
        let mut dry = with_storage(growing_state(), 2.0);
        dry.stocks.get_mut(SOIL_WATER).expect("soil water").amount = 19.5;

        let alloc = allocation(ctx_open(), None);
        let gres = open_growth_respiration();
        let (a_wet, a_dry) = (
            legs_of(&alloc, &s, 800.0)[LEAF],
            legs_of(&alloc, &dry, 800.0)[LEAF],
        );
        let (g_wet, g_dry) = (
            legs_of(&gres, &s, 800.0)["boundary.co2"],
            legs_of(&gres, &dry, 800.0)["boundary.co2"],
        );
        assert!(
            a_dry < a_wet && g_dry < g_wet,
            "the stress must actually bite"
        );
        // ⚠ NOT bit-equality: the two ratios are `Yg·A/Yg·B` and `(1−Yg)·A/(1−Yg)·B`,
        // algebraically identical but computed through different products, so the last
        // bits may differ. The claim is the ratio, and it is asserted as such.
        let (ra, rg) = (a_dry / a_wet, g_dry / g_wet);
        assert!(
            (ra - rg).abs() <= 1e-12,
            "allocation scaled by {ra}, growth respiration by {rg}"
        );
    }
    // -----------------------------------------------------------------------------
    // S5 batch E — nitrogen: the three flows the frozen wiring builds.
    //
    // Ported from `tests/test_nitrogen.py` (the assembled-flow block) and
    // `tests/test_nitrogen_form.py` (claims 3 and 4). Before this batch the three flows
    // between them were guarded like this, measured on `-p domains --lib` and then on the
    // golden + tier binaries:
    //
    //   * dropping the demand limb, so a full plant keeps drawing:  1 red, about the ROOT-ZONE gate
    //   * dropping the deficit's non-negative clamp:                1 red, about the WIRING
    //   * feeding Greenwood's curve the root-inclusive mass:        0 reds, goldens only
    //   * dropping the availability gate from the capacity:         1 red, about the WIRING
    //   * dropping the ground-area factor (uptake, and fertilization): 0 reds ANYWHERE
    //   * dropping the shed remobilization `min`:                   0 reds, goldens only
    //
    // The ground-area pair is the batch C finding on two more call sites: every frozen
    // scenario is 1 m2, so an area factor is invisible to the whole suite, the goldens and
    // the cross-port comparison alike. The tests below are therefore constructed states,
    // not scenario runs.
    // -----------------------------------------------------------------------------

    const SOIL_N: &str = "biosphere.soil_n";
    const LITTER_N: &str = "biosphere.litter_n";
    const LITTER_SINK: &str = "biosphere.litter_carbon";
    const N_SOURCE: &str = "boundary.fertilizer_supply";

    /// The soil-N band of the Python fixture. Scenario/soil data, not crop params.
    const SN_RESIDUAL: f64 = 0.01;
    const SN_CRITICAL: f64 = 0.05;

    /// A state carrying the nitrogen stocks the base fixture does not build.
    ///
    /// `storage` is a separate argument on purpose: Greenwood's `W` is leaf+stem+storage
    /// and `f_N`'s denominator is leaf+stem+root, so a fixture with `storage == root`
    /// cannot tell the two denominators apart — the same trap `with_storage` was added to
    /// escape one mechanism over.
    fn n_state(leaf: f64, stem: f64, root: f64, storage: f64, plant_n: f64, soil_n: f64) -> State {
        let mut s = with_storage(state(leaf, stem, root, 0.4, 0.21 * AIR_MOL, 550.0), storage);
        s.stocks.get_mut(PLANT_N).expect("plant n").amount = plant_n;
        for (id, amount, kind) in [
            (SOIL_N, soil_n, StockKind::Pool),
            (LITTER_N, 0.0, StockKind::Pool),
            (N_SOURCE, 1.0e9, StockKind::Boundary),
        ] {
            s.stocks.insert(
                id.to_string(),
                Stock::new(
                    id.to_string(),
                    "biosphere".to_string(),
                    Quantity::Nitrogen,
                    Quantity::Nitrogen.canonical_unit(),
                    amount,
                    kind,
                    0.0,
                    false,
                    BTreeMap::new(),
                )
                .expect("nitrogen stock"),
            );
        }
        s.stocks.insert(
            LITTER_SINK.to_string(),
            Stock::new(
                LITTER_SINK.to_string(),
                "biosphere".to_string(),
                Quantity::Carbon,
                Quantity::Carbon.canonical_unit(),
                0.0,
                StockKind::Pool,
                0.0,
                false,
                BTreeMap::new(),
            )
            .expect("litter carbon"),
        );
        s
    }

    fn uptake(ground_area: f64) -> NitrogenUptake {
        let p = params::nitrogen();
        NitrogenUptake {
            id: "biosphere.n_uptake".to_string(),
            soil_n: SOIL_N.to_string(),
            plant_n: PLANT_N.to_string(),
            leaf_c: LEAF.to_string(),
            stem_c: STEM.to_string(),
            root_c: ROOT.to_string(),
            storage_c: STORAGE.to_string(),
            max_uptake_capacity: p.max_uptake_capacity,
            n_target_coefficient: p.n_target_coefficient,
            n_target_exponent: p.n_target_exponent,
            n_target_w_plateau: p.n_target_w_plateau,
            dm_kg_per_mol_c: p.dm_kg_per_mol_c,
            ground_area,
            rooted_depth_aux: ROOTED_DEPTH.to_string(),
            // A reference layer this thin is reached by any nonzero rooted depth, so the
            // root-access gate is fully OPEN and every assertion below is about the uptake
            // law. The gate's own behaviour is pinned in `system.rs`.
            soil_layer_depth: 1e-9,
            sn_residual: SN_RESIDUAL,
            sn_critical: SN_CRITICAL,
        }
    }

    fn fertilization(ground_area: f64) -> Fertilization {
        Fertilization {
            id: "biosphere.fertilization".to_string(),
            n_source: N_SOURCE.to_string(),
            soil_n: SOIL_N.to_string(),
            fertilization_var: "fertilization".to_string(),
            ground_area,
        }
    }

    /// Evaluate with a resolver that also carries the N-application forcing.
    fn n_legs_of(flow: &dyn Flow, s: &State, dt: f64, fert: f64) -> BTreeMap<String, f64> {
        let forcings: HashMap<String, Schedule> = HashMap::from([
            ("par".to_string(), constant(800.0).expect("par")),
            ("ci".to_string(), constant(400.0).expect("ci")),
            ("temp".to_string(), constant(20.0).expect("temp")),
            ("fertilization".to_string(), constant(fert).expect("fert")),
        ]);
        let shared: HashMap<String, String> = HashMap::from([
            ("soil_water".to_string(), SOIL_WATER.to_string()),
            ("co2_pool".to_string(), CO2.to_string()),
        ]);
        let r = SourceResolver::new(forcings, shared).expect("resolver");
        let env = r.bind(s, dt);
        flow.evaluate(s, &env, dt)
            .expect("evaluate")
            .legs
            .iter()
            .map(|l| (l.stock.clone(), l.amount))
            .collect()
    }

    /// `min(deficit, capacity·availability)` — and BOTH arms are exercised, because a
    /// fixture that only ever reaches one of them cannot tell the law from either half.
    ///
    /// The starved plant is supply-bound: `W = 0.5 mol C = 0.1335 t/ha` sits on
    /// Greenwood's plateau, so the target is `a = 5.697 %` and the deficit is
    /// `0.05697 · 0.0266911 · 2.0 = 3.041e-3 kg N` — four times the day's supply of
    /// `0.0015 · 1 m² · 0.5 = 7.5e-4 kg N`. The nearly-full plant is demand-bound: it is
    /// `1e-5 kg N` short with the soil saturated, so the draw is the SHORTFALL.
    /// Mirrors `test_uptake_leg_is_capacity_times_availability_times_area`,
    /// `test_uptake_is_demand_limited_when_the_shortfall_is_small` and
    /// `test_uptake_shuts_off_at_residual_soil_n`.
    #[test]
    fn the_uptake_draws_the_smaller_of_the_deficit_and_the_supply_and_both_arms_bite() {
        // --- the SUPPLY arm: soil_n 0.03 in the band [0.01, 0.05] => availability 0.5.
        let starved = n_state(0.5, 0.0, 1.5, 0.0, 0.0, 0.03);
        let legs = n_legs_of(&uptake(1.0), &starved, 1.0, 0.0);
        // ⚠ NOT bit-equality, and the reason is the fixture rather than the flow: the
        // availability `(0.03 - 0.01) / 0.04` reconstructs 0.5 as 0.4999999999999999,
        // because none of 0.03/0.01/0.05 is a binary fraction. Asserting `== 0.00075`
        // would be a pin on the round-off of the arithmetic used to build the INPUT — the
        // same trap `the_nitrogen_stress_ramp_is_linear_between_its_two_knots` records for
        // the 1/90 and 1/45 knots.
        assert!(
            (legs[PLANT_N] - 0.00075).abs() <= 1e-15 * 0.00075,
            "{:?}",
            legs[PLANT_N]
        );
        assert_eq!(legs[SOIL_N], -legs[PLANT_N]);

        // --- the DEMAND arm: the same plant, a saturated soil, and 1e-5 kg N short.
        let p = params::nitrogen();
        let target_per_mol_c = p.n_target_coefficient * p.dm_kg_per_mol_c;
        let full = target_per_mol_c * 2.0;
        let shortfall = 1.0e-5;
        let nearly = n_state(0.5, 0.0, 1.5, 0.0, full - shortfall, 1.0);
        let legs = n_legs_of(&uptake(1.0), &nearly, 1.0, 0.0);
        assert!(
            (legs[PLANT_N] - shortfall).abs() <= 1e-9 * shortfall,
            "the demand arm drew {}",
            legs[PLANT_N]
        );
        assert!(
            (legs[SOIL_N] + legs[PLANT_N]).abs() <= 1e-18,
            "the flow must be nitrogen-balanced leg for leg"
        );
        // The two arms really are different selections: the same plant on a saturated soil
        // could have drawn the full day's capacity, and did not.
        assert!(
            legs[PLANT_N] < 0.0015 * 0.01,
            "the demand arm must dominate"
        );

        // --- the HARD OFF: soil N AT the residual point supplies nothing, however large
        // the plant's deficit.
        let shut = n_state(0.5, 0.0, 1.5, 0.0, 0.0, SN_RESIDUAL);
        let legs = n_legs_of(&uptake(1.0), &shut, 1.0, 0.0);
        assert_eq!(legs[PLANT_N], 0.0);
        assert_eq!(legs[SOIL_N], 0.0);
    }

    /// The shortfall closes in ONE step at `dt = 1` and the step after draws nothing.
    ///
    /// The deficit is a STOCK read as a per-day rate, so at the frozen step it is a
    /// deadbeat controller — and there is no restoring force to overshoot against, because
    /// the deficit clamps at zero. That is what separates it from a demand-controlled
    /// makeup whose error is SIGNED and can oscillate (bucket 2's export-fidelity
    /// finding), and it is why the clamp is a modelling statement rather than a guard.
    /// Mirrors `test_demand_limited_uptake_is_dt_linear_and_deadbeat_at_dt_1`.
    #[test]
    fn the_uptake_closes_the_shortfall_in_one_step_and_does_not_overshoot() {
        let p = params::nitrogen();
        let full = p.n_target_coefficient * p.dm_kg_per_mol_c * 2.0;
        let shortfall = 1.0e-5;
        let s = n_state(0.5, 0.0, 1.5, 0.0, full - shortfall, 1.0);
        let flow = uptake(1.0);
        // dt-linear: the RATE comes from the snapshot alone, never from dt.
        let half = n_legs_of(&flow, &s, 0.5, 0.0)[PLANT_N];
        let one = n_legs_of(&flow, &s, 1.0, 0.0)[PLANT_N];
        assert_eq!(one.to_bits(), (2.0 * half).to_bits());
        // Deadbeat at the frozen step...
        assert!((one - shortfall).abs() <= 1e-9 * shortfall);
        // ...and the settled plant draws exactly zero on the next step.
        let settled = n_state(0.5, 0.0, 1.5, 0.0, full - shortfall + one, 1.0);
        for (_, amount) in n_legs_of(&flow, &settled, 1.0, 0.0) {
            assert_eq!(amount, 0.0, "a settled plant must draw nothing");
        }
    }

    /// A plant AT or ABOVE its target takes up nothing, however rich the soil.
    ///
    /// This is what makes `plant_n` a tracked quantity rather than a monotone
    /// accumulator. ⚠ It is also the assertion that catches the clamp being dropped:
    /// without `.max(0.0)` an over-full plant computes a NEGATIVE deficit and the flow
    /// runs backwards, pumping nitrogen out of the plant and into the soil at a rate set
    /// by how far above target it is. Measured before this test existed: that mutation
    /// reddened one test in 282, and that test is about the wiring, not the law.
    /// Mirrors `test_uptake_stops_entirely_at_or_above_the_target_concentration`.
    #[test]
    fn the_uptake_stops_at_the_target_and_never_runs_backwards() {
        let p = params::nitrogen();
        let full = p.n_target_coefficient * p.dm_kg_per_mol_c * 2.0;
        for factor in [1.0, 1.5, 100.0] {
            let s = n_state(0.5, 0.0, 1.5, 0.0, full * factor, 1.0);
            for (id, amount) in n_legs_of(&uptake(1.0), &s, 1.0, 0.0) {
                assert_eq!(amount, 0.0, "leg {id} at {factor}x the target");
            }
        }
    }

    /// ⚠ THE TWO DENOMINATORS, which differ on purpose and were guarded by nothing.
    ///
    /// Greenwood's `%N` and `W` "refer to the dry matter in the whole plant (EXCLUDING
    /// fibrous roots)", so the curve is evaluated on leaf+stem+storage. The deficit it
    /// produces is then applied to `f_N`'s own denominator, leaf+stem+root — the pool the
    /// stress factor will read back. `nitrogen.yaml`'s own source tag records the delta
    /// and records that feeding the curve the root-INCLUSIVE mass was measured and is
    /// worse.
    ///
    /// Nothing in the tree said so. Measured: swapping `storage_c` for `root_c` in the
    /// curve's argument reddened ZERO tests of `-p domains --lib` and only committed
    /// golden bytes of the whole workspace — because six of the seven frozen scenarios
    /// peak an order of magnitude below the 1 t/ha domain bound, where the target is a
    /// CONSTANT and the argument cannot matter. This state is built above the bound on
    /// both readings so that it can.
    ///
    /// The arithmetic, hand-computed: `W = 20 + 10 + 15 = 45 mol C`, i.e.
    /// `45 · 0.0266911 kg = 1.2011 kg/m² = 12.011 t/ha`; the target there is
    /// `5.697 / sqrt(12.011) = 1.6438 %`, or `4.38756e-4 kg N per mol C`. Against a
    /// biomass of `20 + 10 + 5 = 35 mol C` that is `1.53565e-2 kg N` at target, so a plant
    /// holding `1.53e-2` is `5.6468e-5 kg N` short — the demand arm, well under the day's
    /// `1.5e-3` supply. Under the swapped reading the curve sees `9.3419 t/ha`, asks for
    /// `1.7413e-2`, and the draw saturates at capacity instead: a 27-fold difference, not
    /// a last-bit one.
    /// Mirrors the delta recorded in `nitrogen.yaml`'s `n_target_coefficient` source tag.
    #[test]
    fn greenwoods_mass_excludes_the_fibrous_roots_that_the_deficit_is_applied_to() {
        let s = n_state(20.0, 10.0, 5.0, 15.0, 0.0153, 1.0);
        let drawn = n_legs_of(&uptake(1.0), &s, 1.0, 0.0)[PLANT_N];
        assert!(
            (drawn - 5.646780255798463e-5).abs() <= 1e-15,
            "the shoot-basis deficit is 5.64678e-5 kg N, drew {drawn}"
        );
        // ...and the root-inclusive reading is a different answer, not a rounding of the
        // same one. Asserted rather than described, so this test is discriminating by
        // construction rather than by hope.
        let p = params::nitrogen();
        let swapped_w_t_ha = (20.0 + 10.0 + 5.0) * p.dm_kg_per_mol_c * 10.0;
        let swapped_target = crate::biosphere::science::target_n_concentration(
            swapped_w_t_ha,
            p.n_target_coefficient,
            p.n_target_exponent,
            p.n_target_w_plateau,
        ) * p.dm_kg_per_mol_c;
        let swapped_draw = (swapped_target * 35.0 - 0.0153).min(0.0015);
        assert!(
            swapped_draw > 20.0 * drawn,
            "the fixture cannot tell the denominators apart: {drawn} vs {swapped_draw}"
        );
    }

    /// Both nitrogen flows that carry a plot area actually scale with it.
    ///
    /// ⚠ Every frozen scenario is 1 m², so a dropped area factor computes the identical
    /// number and is invisible to the goldens, to the tier bands and to the cross-port
    /// comparison. Measured before this test existed: dropping it from the uptake capacity
    /// and dropping it from the fertilization rate each left the ENTIRE workspace green.
    /// That is the same hole `system.rs::capture_scales_with_ground_area_at_its_call_sites`
    /// was written for one currency over, on two more call sites.
    /// Mirrors `test_uptake_scales_with_ground_area` and
    /// `test_fertilization_leg_is_rate_times_area`.
    #[test]
    fn the_uptake_and_the_fertilization_both_scale_with_the_plot() {
        // Uptake: the same starved plant on a 1 m² and a 3 m² plot. The plant's state is
        // held fixed on purpose — this asserts the CAPACITY's area factor, which is the
        // one the mutation removes.
        let s = n_state(0.5, 0.0, 1.5, 0.0, 0.0, 0.03);
        let one = n_legs_of(&uptake(1.0), &s, 1.0, 0.0)[PLANT_N];
        let three = n_legs_of(&uptake(3.0), &s, 1.0, 0.0)[PLANT_N];
        // Relative, not bit-exact: the area enters the capacity product in a different
        // position than the `3.0 *` here, so the last bit may differ. The claim is the
        // proportionality, and it is asserted as such.
        assert!(
            (three - 3.0 * one).abs() <= 1e-15 * three,
            "{one} m-2 against {three} on 3 m2"
        );

        // Fertilization: 0.001 kg N m⁻² day⁻¹ over 2 m² is 0.002 kg N/day into the soil.
        let legs = n_legs_of(&fertilization(2.0), &s, 1.0, 0.001);
        assert_eq!(legs[SOIL_N], 0.002);
        assert_eq!(legs[N_SOURCE], -0.002);
        // ...and it is dt-linear and balanced, which is the whole of the rest of the flow.
        let half = n_legs_of(&fertilization(2.0), &s, 0.5, 0.001);
        assert_eq!(half[SOIL_N].to_bits(), (0.5 * legs[SOIL_N]).to_bits());
        assert_eq!(half[SOIL_N] + half[N_SOURCE], 0.0);
    }

    /// The shed nitrogen is the senescing CARBON times the remobilized concentration.
    ///
    /// One physical event, two currency legs, and no channel between them: a flow may only
    /// read the step-entry snapshot, so `NitrogenSenescence` RECOMPUTES the carbon flux
    /// `Senescence` is sending to litter. That recomputation is the hazard the Python side
    /// pinned by comparing the two flows' legs, and it is pinned the same way here.
    ///
    /// The `min` is remobilization: a well-fed plant retains its nitrogen and sheds only
    /// the residual concentration [C] measures in mature straw. ⚠ Dropping it entirely
    /// reddened no test of `-p domains --lib` and only golden bytes; and a branch probe
    /// says the LEAN arm — a plant already below the residual — is reached by NOTHING in
    /// the binary, because every frozen scenario runs above `n_critical` all season. Both
    /// arms are therefore constructed here rather than driven.
    /// Mirrors `test_shed_nitrogen_uses_the_same_carbon_flux_as_the_senescence_flow`.
    #[test]
    fn the_shed_nitrogen_is_the_senescing_carbon_at_the_remobilized_concentration() {
        let sen = params::senescence();
        let canopy = params::canopy();
        let nitro = params::nitrogen();
        let carbon = Senescence {
            id: "probe.senescence".to_string(),
            leaf_c: LEAF.to_string(),
            stem_c: STEM.to_string(),
            root_c: ROOT.to_string(),
            litter_sink: LITTER_SINK.to_string(),
            rdr_leaf: sen.rdr_leaf,
            rdr_stem: sen.rdr_stem,
            rdr_root: sen.rdr_root,
            shade_rate: sen.shade_rate,
            lai_threshold: sen.lai_threshold,
            sla_per_mol_c: canopy.sla_per_mol_c,
            ground_area: 1.0,
        };
        let shed_n = |plant_n: f64| {
            NitrogenSenescence {
                id: "biosphere.nitrogen_senescence".to_string(),
                plant_n: PLANT_N.to_string(),
                litter_n: LITTER_N.to_string(),
                leaf_c: LEAF.to_string(),
                stem_c: STEM.to_string(),
                root_c: ROOT.to_string(),
                rdr_leaf: sen.rdr_leaf,
                rdr_stem: sen.rdr_stem,
                rdr_root: sen.rdr_root,
                n_residual_per_mol_c: nitro.n_residual_per_mol_c,
                shade_rate: sen.shade_rate,
                lai_threshold: sen.lai_threshold,
                sla_per_mol_c: canopy.sla_per_mol_c,
                ground_area: 1.0,
            }
            .evaluate(
                &n_state(3.0, 1.0, 1.0, 0.0, plant_n, 1.0),
                &SourceResolver::new(HashMap::new(), HashMap::new())
                    .expect("resolver")
                    .bind(&n_state(3.0, 1.0, 1.0, 0.0, plant_n, 1.0), 1.0),
                1.0,
            )
            .expect("shed")
            .legs
            .iter()
            .map(|l| (l.stock.clone(), l.amount))
            .collect::<BTreeMap<String, f64>>()
        };

        let biomass = 5.0; // leaf 3 + stem 1 + root 1
        let shed_c = legs_of(&carbon, &n_state(3.0, 1.0, 1.0, 0.0, 1.0, 1.0), 800.0)[LITTER_SINK];
        assert!(shed_c > 0.0, "the carbon leg must actually shed");

        // --- the WELL-FED arm: concentration above the residual, so the plant retains the
        // difference and litter leaves at the straw concentration.
        let rich = 1.0; // 0.2 kg N/mol C, far above the 1.3346e-4 residual
        let legs = shed_n(rich);
        let want = nitro.n_residual_per_mol_c * shed_c;
        assert!(
            (legs[LITTER_N] - want).abs() <= 1e-15 * want,
            "well-fed shed {} against {want}",
            legs[LITTER_N]
        );
        assert_eq!(legs[PLANT_N], -legs[LITTER_N]);

        // --- the LEAN arm, which no scenario in the tree reaches: a plant already below
        // the residual sheds at its OWN concentration, because there is nothing left to
        // remobilize. Without the `min` this arm and the one above are the same line.
        let lean = 0.5 * nitro.n_residual_per_mol_c * biomass;
        let legs = shed_n(lean);
        let want = (lean / biomass) * shed_c;
        assert!(
            (legs[LITTER_N] - want).abs() <= 1e-15 * want,
            "lean shed {} against {want}",
            legs[LITTER_N]
        );
        assert!(
            legs[LITTER_N] < 0.6 * nitro.n_residual_per_mol_c * shed_c,
            "the lean arm must be strictly leaner than the residual one"
        );
    }

    /// The shed material's C:N is `carbon_fraction / n_residual = 0.45 / 0.005 = 90`.
    ///
    /// The deliverable of the N-cycle form change, in one number: litter composition is
    /// now a consequence of two CITED concentrations ([B] Raimanova 2024 for the carbon
    /// fraction, [C] Van Hecke 2020 for the residual N) rather than the ratio of two
    /// unrelated first-order rate constants — which is what it was before, and which
    /// measured 0.004. Real wheat straw is ~80, so this is the right quantity in the right
    /// place rather than a fitted one.
    /// Mirrors `test_shed_material_has_a_straw_like_carbon_to_nitrogen_ratio`.
    #[test]
    fn the_shed_material_has_a_straw_like_carbon_to_nitrogen_ratio() {
        let p = params::nitrogen();
        // Back to Greenwood's basis: kg N per kg DM, then C:N on a mass basis.
        let n_residual_kg_kg = p.n_residual_per_mol_c / p.dm_kg_per_mol_c;
        let carbon_fraction = params::MOLAR_MASS_CARBON_KG_PER_MOL / p.dm_kg_per_mol_c;
        let shed_cn = carbon_fraction / n_residual_kg_kg;
        assert!((shed_cn - 90.0).abs() <= 1e-9, "shed C:N is {shed_cn}");
        assert!(
            (60.0..120.0).contains(&shed_cn),
            "the shed material left the real-residue band"
        );
    }
    // -----------------------------------------------------------------------------
    // S5 batch G, the senescence batch: the FLOW half.
    //
    // `test_allocation.py`'s seven senescence flow/equation tests, minus the two that are
    // owned elsewhere. `senescence_flux` has no Rust counterpart to point an equation test
    // at — Rust inlines `rate · organ · dt` in `Senescence::evaluate` — so its two Python
    // tests (proportionality, and zero organ → zero) are covered here at flow level, which
    // is the same disposition §5ad prescribed for the soil-carbon family.
    //
    // ⚠ `test_senescence_flow_is_carbon_balanced` gets NO successor, and that is batches A
    // and D's recorded disposition rather than a gap: `assert_conserved` runs every step of
    // every run, so a flow that failed it could not survive a single scenario. The legs
    // test below asserts all four legs exactly, which implies the balance anyway.
    //
    // Measured before writing, with `cargo test -p domains --lib --no-fail-fast`
    // (baseline 298 passed):
    //
    //   * the stem simply does not senesce (`rdr_stem = 0`):   2 red, NEITHER about it —
    //         one is batch E's shed-nitrogen pin, one is the mutual-shading GATE, which
    //         reddens because a bigger standing stem moves the trajectory and the peak-LAI
    //         crossing with it. "A number moved, wearing a reassuring name."
    //   * the stem shed at the ROOT's rate (0.005 -> 0.01):    1 red, batch E's N pin only
    //   * the flux not proportional to organ carbon:          28 red, all trajectory movers
    //   * `dt` dropped from all three legs:                   18 red, all trajectory movers
    //   * the LAI dropped its ground-area divisor:            **0 reds anywhere**
    //
    // The last one is batch C's ground-area finding on a THIRD call site (`Senescence`,
    // after the capture and the uptake/fertilization pair): every frozen scenario is 1 m²,
    // so an area factor is invisible to this binary, to the goldens, and to the cross-port
    // comparison alike. The test below is therefore a constructed state on a 2 m² plot.
    // -----------------------------------------------------------------------------

    /// The committed rates, as literals — batch A's convention (a loader regression must
    /// not be able to move a physics pin silently). They mirror `senescence.yaml`, and
    /// `params.rs::the_committed_senescence_rates_are_the_five_values_the_file_states` is
    /// what ties these literals to that file.
    const G_RDR_LEAF: f64 = 0.02;
    const G_RDR_STEM: f64 = 0.005;
    const G_RDR_ROOT: f64 = 0.01;
    /// The Python fixture's SLA: 0.6 m² per mol C, so on 1 m² LAI is simply `0.6 · leaf_c`
    /// and the threshold crossing lands on a round leaf carbon.
    const G_SLA: f64 = 0.6;

    fn senescence_flow(ground_area: f64) -> Senescence {
        Senescence {
            id: "biosphere.senescence".to_string(),
            leaf_c: LEAF.to_string(),
            stem_c: STEM.to_string(),
            root_c: ROOT.to_string(),
            litter_sink: LITTER_SINK.to_string(),
            rdr_leaf: G_RDR_LEAF,
            rdr_stem: G_RDR_STEM,
            rdr_root: G_RDR_ROOT,
            // The CITED constants, but every state below except the closure pair sits well
            // under LAI 6, so the shading term is inert there by construction.
            shade_rate: 0.05,
            lai_threshold: 6.0,
            sla_per_mol_c: G_SLA,
            ground_area,
        }
    }

    /// A state carrying the litter BOUNDARY sink the organ fixture does not build.
    fn sen_state(leaf: f64, stem: f64, root: f64) -> State {
        let mut s = state(leaf, stem, root, 0.4, 0.21 * AIR_MOL, 550.0);
        s.stocks.insert(
            LITTER_SINK.to_string(),
            Stock::new(
                LITTER_SINK.to_string(),
                "biosphere".to_string(),
                Quantity::Carbon,
                Quantity::Carbon.canonical_unit(),
                0.0,
                StockKind::Boundary,
                0.0,
                false,
                BTreeMap::new(),
            )
            .expect("litter sink"),
        );
        s
    }

    /// `legs_of` fixes `dt = 1`; senescence needs it as a knob.
    fn sen_legs(flow: &Senescence, s: &State, dt: f64) -> BTreeMap<String, f64> {
        let r = resolver(0.0, 400.0);
        let env = r.bind(s, dt);
        flow.evaluate(s, &env, dt)
            .expect("evaluate")
            .legs
            .iter()
            .map(|l| (l.stock.clone(), l.amount))
            .collect()
    }

    /// All four legs, hand-computed from the rates and the organ carbon.
    ///
    /// Each organ loses `rdr · organ · dt` and the litter sink receives the sum — one
    /// atomic stoichiometric transfer, not three. Three distinct rates on three distinct
    /// organ sizes, so no pair of legs can be swapped without the arithmetic noticing:
    /// that is what makes this the test the "stem shed at the root's rate" mutation dies
    /// on, which nothing about senescence caught before.
    /// Mirrors `test_senescence_legs_are_the_hand_computed_losses`,
    /// `test_senescence_flux_proportional_to_organ_carbon` and
    /// `test_senescence_flux_zero_organ_is_zero`.
    #[test]
    fn the_senescence_legs_are_the_per_organ_relative_losses() {
        let s = sen_state(3.0, 1.0, 1.0);
        let legs = sen_legs(&senescence_flow(1.0), &s, 1.0);
        let (leaf, stem, root) = (3.0 * G_RDR_LEAF, 1.0 * G_RDR_STEM, 1.0 * G_RDR_ROOT);
        assert_eq!(legs[LEAF], -leaf);
        assert_eq!(legs[STEM], -stem);
        assert_eq!(legs[ROOT], -root);
        assert_eq!(legs[LITTER_SINK], leaf + stem + root);

        // Proportional to the organ, and self-limiting: an organ at zero sheds nothing, so
        // positivity is structural rather than clamped. A flux that read a fixed amount,
        // or `sqrt`, satisfies neither.
        let double = sen_legs(&senescence_flow(1.0), &sen_state(6.0, 2.0, 2.0), 1.0);
        for id in [LEAF, STEM, ROOT] {
            assert_eq!(double[id].to_bits(), (2.0 * legs[id]).to_bits(), "{id}");
        }
        let empty = sen_legs(&senescence_flow(1.0), &sen_state(0.0, 0.0, 0.0), 1.0);
        for id in [LEAF, STEM, ROOT, LITTER_SINK] {
            assert_eq!(empty[id], 0.0, "{id} on a dead plant");
        }
    }

    /// Every leg is linear in `dt`, bit-exactly.
    ///
    /// Asserted on `to_bits` rather than a tolerance: a first-order rate law times a step
    /// has no rounding to hide behind, and the biosphere is frozen at `dt = ¼`, so a leg
    /// that was only approximately dt-linear would be a different mechanism at the frozen
    /// step than at the fixture's.
    /// Mirrors `test_senescence_scales_linearly_with_dt`.
    #[test]
    fn the_senescence_legs_are_bit_exactly_linear_in_dt() {
        let s = sen_state(3.0, 1.0, 1.0);
        let flow = senescence_flow(1.0);
        let one = sen_legs(&flow, &s, 1.0);
        for dt in [0.5, 0.25, 0.125] {
            let scaled = sen_legs(&flow, &s, dt);
            for id in [LEAF, STEM, ROOT, LITTER_SINK] {
                assert_eq!(
                    scaled[id].to_bits(),
                    (dt * one[id]).to_bits(),
                    "{id} at dt={dt}"
                );
            }
        }
    }

    /// The mutual-shading term reaches the flow, and it moves the LEAF leg only.
    ///
    /// Two states either side of the cited threshold on one flow: at SLA 0.6 over 1 m²,
    /// LAI is `0.6 · leaf_c`, so 9 mol C is LAI 5.4 and 11 mol C is LAI 6.6. ⚠ The leaf
    /// legs are compared against their own hand-computed values rather than against each
    /// other, because the leg is also proportional to leaf carbon and the two states must
    /// differ in leaf carbon to differ in LAI — a bare ratio would be confounded.
    ///
    /// The stem leg is asserted UNCHANGED across the crossing: mutual shading is a
    /// canopy-closure mechanism, and a term that leaked into the other organs would still
    /// balance, still be dt-linear, and still shed faster once the canopy closed.
    /// Mirrors `test_senescence_sheds_FASTER_once_the_canopy_closes`.
    #[test]
    fn senescence_sheds_faster_from_the_leaf_only_once_the_canopy_closes() {
        let flow = senescence_flow(1.0);
        let below = sen_legs(&flow, &sen_state(9.0, 1.0, 1.0), 1.0); // LAI 5.4
        let above = sen_legs(&flow, &sen_state(11.0, 1.0, 1.0), 1.0); // LAI 6.6
        assert_eq!(below[LEAF], -9.0 * G_RDR_LEAF);
        assert_eq!(above[LEAF], -11.0 * (G_RDR_LEAF + 0.05));
        // ...and the other two organs never hear about it.
        assert_eq!(below[STEM], above[STEM]);
        assert_eq!(below[ROOT], above[ROOT]);
        assert_eq!(above[STEM], -G_RDR_STEM); // stem carbon is 1.0 in both states
    }

    /// ⚠ THE CANOPY IS MEASURED PER GROUND AREA, at `Senescence`'s own call site.
    ///
    /// Every frozen scenario is 1 m², so a `Senescence` that computed LAI as bare
    /// `leaf_c · SLA` — dropping the divisor entirely — returns the identical number for
    /// every run in the tree. Measured: that mutant left `cargo test -p domains --lib` at
    /// 298 passed / 0 failed, and it is invisible to the goldens and to the cross-port
    /// comparison for the same reason. Batch C's finding on a third call site, so this is
    /// a constructed state on a 2 m² plot rather than a scenario run.
    ///
    /// The claim is stated as a CROSSING rather than as a value: the same standing leaf
    /// carbon is inside the mutual-shading regime on 1 m² and outside it on 2 m², which is
    /// the only way an area error changes behaviour rather than merely changing a number.
    #[test]
    fn the_senescence_canopy_is_measured_per_ground_area() {
        // 11 mol C: LAI 6.6 on 1 m² (in the regime), LAI 3.3 on 2 m² (out of it).
        let s = sen_state(11.0, 1.0, 1.0);
        let dense = sen_legs(&senescence_flow(1.0), &s, 1.0);
        let spread = sen_legs(&senescence_flow(2.0), &s, 1.0);
        assert_eq!(dense[LEAF], -11.0 * (G_RDR_LEAF + 0.05), "6.6 > 6, shading");
        assert_eq!(spread[LEAF], -11.0 * G_RDR_LEAF, "3.3 < 6, no shading");
        // ...and the two organs with no area term in their rate law are untouched, so the
        // difference above cannot be read as the plot scaling everything.
        assert_eq!(dense[STEM], spread[STEM]);
        assert_eq!(dense[ROOT], spread[ROOT]);
    }

    // =============================================================================
    // S5 batch F, the soil-carbon batch: the six decomposer flows.
    //
    // ⚠ These are FLOW-level and there is no equation-level alternative — that is the
    // property §5ad named when it filed F as "the batch that is not like the others" and
    // held it back. What §5ad ALSO said is that F "cannot be written as pure-function
    // tests without changing production code", and **that premise is false**.
    // `Decomposition` and its five siblings are plain structs with public fields,
    // `respired_and_stabilized` is already `pub`, and `carried_nitrogen` is reachable from
    // here through `use super::*`. Constructing a flow and calling `evaluate` gets
    // leg-level assertions with no production change at all — which is exactly the
    // technique batch A's gas-exchange third already used one screen above. Recorded
    // rather than silently skipped, because an unstated no-op reads as the question never
    // having been asked.
    //
    // ⚠ What is deliberately NOT ported, and why each absence is a decision:
    //
    //   * `test_loader_reads_committed_rate` (x2), `test_loader_reads_the_cited_partition`
    //     and `test_loader_reads_the_cited_stabilization_efficiency` — all four assert a
    //     committed scalar, and all seven decomposer scalars (`decomp.decomposition_rate`,
    //     `micro.*`, `humi.*`) are ALREADY pinned bit-exactly, as hex-float literals, by
    //     C1's `params::tests::every_value_matches_the_generated_table`. A second copy is
    //     the shape this project has been bitten by — a rule with two copies has one that
    //     goes stale — so the claim is left where it already lives. Same disposition batch
    //     G gave the five senescence values.
    //   * `test_decomposition_balances_carbon_AND_oxygen`,
    //     `test_respiration_balances_carbon_and_oxygen`, `test_flows_balance_nitrogen_only`
    //     and the four `test_sealed_conserves_{carbon,oxygen,nitrogen}_exactly` — the
    //     engine's own machinery. `assert_conserved` runs every step of every run, so a
    //     completed scenario run is the proof. Batch A recorded the identical disposition
    //     for the identical reason.
    //   * `test_sealed_never_rations` (x3) and `test_sealed_no_extinction` (x3) —
    //     `system.rs::sealed_chamber_runs_well_fed` already asserts `rationed == 0` and
    //     `events.is_empty()` on that very run. Six copies of one claim.
    //   * `test_sealed_o2_stays_far_from_rationing` — its premise is false in the
    //     reference, and batch A already recorded why: `f_O2` is LIVE here and the sealed
    //     chamber depletes O2 on purpose.
    //   * `test_there_is_no_mineralization_param_file_or_loader` and the `hasattr` half of
    //     `test_the_free_mineralization_rate_no_longer_EXISTS_to_be_calibrated` — guarded
    //     by the COMPILER, harder than by a test. There is no `mineralization` module and
    //     no `MineralizationParams`; `params.rs` reaches its files through `include_str!`,
    //     so a re-added file is not silently unread but unread AND caught by
    //     `params::tests::the_census_matches_the_directory_on_disk`. Same disposition
    //     batch D gave `test_context_storage_excluded_from_biomass`.
    //   * `test_the_retired_provenance_record_is_preserved` — its subject is a Python-tree
    //     archive path (`docs/retired/mineralization.yaml`). That is the ungated-prose gap
    //     recorded elsewhere, not a science claim this batch can port.
    //   * `test_the_return_legs_take_the_DECOMPOSER_params_not_their_own` — a Python
    //     `isinstance` check on a params object. The Rust flows carry bare `f64` rate
    //     fields, so there is no type to assert; what the claim is really about is the
    //     WIRING, and that is asserted against the built registry in `system.rs` instead.
    //
    // Each test below was mutation-checked against `cargo test -p domains --lib`.
    // -----------------------------------------------------------------------------

    const LITTER_C: &str = "biosphere.litter_carbon";
    const MICROBIAL_C: &str = "biosphere.microbial_carbon";
    const HUMUS_C: &str = "biosphere.humus_carbon";
    const MICROBIAL_N: &str = "biosphere.microbial_n";
    const HUMUS_N: &str = "biosphere.humus_n";

    /// The three committed decomposer rates and the three committed CO2 shares.
    ///
    /// Named here as the tests' own arithmetic inputs. They are NOT a second copy of the
    /// value gate: every assertion below drives the flows with these numbers explicitly
    /// and checks the ARITHMETIC, while `every_value_matches_the_generated_table` owns
    /// whether the committed files still hold them.
    const F_K_LITTER: f64 = 0.011; // decomposition.yaml
    const F_K_MICROBIAL: f64 = 0.016; // microbial_respiration.yaml
    /// humification.yaml — [A] Parton 1987 p. 1176's `K6 = 0.0038 week-1`, per day.
    const F_K_HUMUS: f64 = 0.0038 / 7.0;
    const F_CO2_LITTER: f64 = 0.45; // Parton 1987 p. 1174, surface structural litter
    const F_CO2_MICROBIAL: f64 = 0.85; // Parton 1987 eq. [6] at T = 0 (Es)
    const F_CO2_HUMUS: f64 = 0.55; // CENTURY's slow-SOM respiration share
    /// `microbial_respiration.yaml`'s committed `o2_half_saturation` (mol/mol).
    const F_K_O2: f64 = 1e-4;
    /// A full chamber: 21 % O2 of `AIR_MOL`.
    const F_FULL_O2: f64 = 0.21 * AIR_MOL;

    /// `a` within a relative `1e-12` of `b` — the soil legs span 1e-5 to 1e-2, so one
    /// absolute tolerance cannot serve them all.
    fn f_close(a: f64, b: f64) -> bool {
        (a - b).abs() <= 1e-12 * b.abs()
    }

    /// A soil state: the three carbon pools, their three N counterparts, soil N and the
    /// two gas pools. `o2` is in mol against `AIR_MOL`, so the mole fraction is `o2/1000`.
    fn soil_state(
        litter_c: f64,
        microbial_c: f64,
        humus_c: f64,
        litter_n: f64,
        microbial_n: f64,
        humus_n: f64,
        o2: f64,
    ) -> State {
        let mut stocks: BTreeMap<String, Stock> = BTreeMap::new();
        let mut pool = |id: &str, q: Quantity, amount: f64, comp: BTreeMap<Quantity, f64>| {
            stocks.insert(
                id.to_string(),
                Stock::new(
                    id.to_string(),
                    "biosphere".to_string(),
                    q,
                    q.canonical_unit(),
                    amount,
                    StockKind::Pool,
                    0.0,
                    false,
                    comp,
                )
                .expect("soil pool"),
            );
        };
        for (id, amount) in [
            (LITTER_C, litter_c),
            (MICROBIAL_C, microbial_c),
            (HUMUS_C, humus_c),
        ] {
            pool(id, Quantity::Carbon, amount, BTreeMap::new());
        }
        for (id, amount) in [
            (LITTER_N, litter_n),
            (MICROBIAL_N, microbial_n),
            (HUMUS_N, humus_n),
            (SOIL_N, 100.0),
        ] {
            pool(id, Quantity::Nitrogen, amount, BTreeMap::new());
        }
        pool(
            CO2,
            Quantity::Carbon,
            0.4,
            BTreeMap::from([(Quantity::Carbon, 1.0), (Quantity::Oxygen, 2.0)]),
        );
        pool(
            O2,
            Quantity::Oxygen,
            o2,
            BTreeMap::from([(Quantity::Oxygen, 2.0)]),
        );
        State::new(0, stocks, 0, BTreeMap::new()).expect("soil fixture state")
    }

    /// The default soil fixture: 4 mol litter C, 2 mol microbial C, 100 mol humus C, and
    /// the three N pools at their own distinct N:C.
    ///
    /// ⚠ The three N:C ratios are deliberately DISTINCT (0.5, 0.025, 0.001). With one
    /// shared ratio, a leg reading the wrong pool's nitrogen returns the right number and
    /// every assertion below stays green.
    fn soil_fixture(o2: f64) -> State {
        soil_state(4.0, 2.0, 100.0, 2.0, 0.05, 0.1, o2)
    }

    fn decomposition_flow(k_o2: f64) -> Decomposition {
        Decomposition {
            id: "biosphere.decomposition".to_string(),
            litter_carbon: LITTER_C.to_string(),
            microbial_carbon: MICROBIAL_C.to_string(),
            co2_pool: CO2.to_string(),
            o2_pool: O2.to_string(),
            decomposition_rate: F_K_LITTER,
            litter_respired_fraction: F_CO2_LITTER,
            o2_half_saturation: k_o2,
            air_mol: AIR_MOL,
        }
    }

    fn microbial_respiration_flow(k_o2: f64) -> MicrobialRespiration {
        MicrobialRespiration {
            id: "biosphere.microbial_respiration".to_string(),
            microbial_carbon: MICROBIAL_C.to_string(),
            humus_carbon: HUMUS_C.to_string(),
            co2_pool: CO2.to_string(),
            o2_pool: O2.to_string(),
            microbial_respiration_rate: F_K_MICROBIAL,
            active_stabilization_co2_fraction: F_CO2_MICROBIAL,
            o2_half_saturation: k_o2,
            air_mol: AIR_MOL,
        }
    }

    fn humus_decomposition_flow(k_o2: f64) -> HumusDecomposition {
        HumusDecomposition {
            id: "biosphere.humus_decomposition".to_string(),
            humus_carbon: HUMUS_C.to_string(),
            microbial_carbon: MICROBIAL_C.to_string(),
            co2_pool: CO2.to_string(),
            o2_pool: O2.to_string(),
            slow_decomposition_rate: F_K_HUMUS,
            slow_respired_fraction: F_CO2_HUMUS,
            o2_half_saturation: k_o2,
            air_mol: AIR_MOL,
        }
    }

    fn litter_n_transfer_flow(k_o2: f64) -> LitterNitrogenTransfer {
        LitterNitrogenTransfer {
            id: "biosphere.litter_n_transfer".to_string(),
            litter_n: LITTER_N.to_string(),
            microbial_n: MICROBIAL_N.to_string(),
            soil_n: SOIL_N.to_string(),
            litter_carbon: LITTER_C.to_string(),
            o2_pool: O2.to_string(),
            decomposition_rate: F_K_LITTER,
            litter_respired_fraction: F_CO2_LITTER,
            o2_half_saturation: k_o2,
            air_mol: AIR_MOL,
        }
    }

    fn microbial_n_release_flow(k_o2: f64) -> MicrobialNitrogenRelease {
        MicrobialNitrogenRelease {
            id: "biosphere.microbial_n_release".to_string(),
            microbial_n: MICROBIAL_N.to_string(),
            soil_n: SOIL_N.to_string(),
            humus_n: HUMUS_N.to_string(),
            microbial_carbon: MICROBIAL_C.to_string(),
            o2_pool: O2.to_string(),
            microbial_respiration_rate: F_K_MICROBIAL,
            active_stabilization_co2_fraction: F_CO2_MICROBIAL,
            o2_half_saturation: k_o2,
            air_mol: AIR_MOL,
        }
    }

    fn humus_n_release_flow(k_o2: f64) -> HumusNitrogenRelease {
        HumusNitrogenRelease {
            id: "biosphere.humus_n_release".to_string(),
            humus_n: HUMUS_N.to_string(),
            soil_n: SOIL_N.to_string(),
            microbial_n: MICROBIAL_N.to_string(),
            humus_carbon: HUMUS_C.to_string(),
            o2_pool: O2.to_string(),
            slow_decomposition_rate: F_K_HUMUS,
            slow_respired_fraction: F_CO2_HUMUS,
            o2_half_saturation: k_o2,
            air_mol: AIR_MOL,
        }
    }

    /// Evaluate a soil flow at `dt`; none of the six read a forcing.
    fn soil_legs(flow: &dyn Flow, s: &State, dt: f64) -> BTreeMap<String, f64> {
        let r = SourceResolver::new(HashMap::new(), HashMap::new()).expect("resolver");
        let env = r.bind(s, dt);
        flow.evaluate(s, &env, dt)
            .expect("evaluate")
            .legs
            .iter()
            .map(|l| (l.stock.clone(), l.amount))
            .collect()
    }

    /// The three carbon flows, boxed, at a given `o2_half_saturation`.
    fn carbon_soil_flows(k_o2: f64) -> Vec<(&'static str, Box<dyn Flow>, &'static str)> {
        vec![
            (
                "decomposition",
                Box::new(decomposition_flow(k_o2)) as Box<dyn Flow>,
                LITTER_C,
            ),
            (
                "microbial_respiration",
                Box::new(microbial_respiration_flow(k_o2)) as Box<dyn Flow>,
                MICROBIAL_C,
            ),
            (
                "humus_decomposition",
                Box::new(humus_decomposition_flow(k_o2)) as Box<dyn Flow>,
                HUMUS_C,
            ),
        ]
    }

    /// The three nitrogen flows, boxed, at a given `o2_half_saturation`.
    fn nitrogen_soil_flows(k_o2: f64) -> Vec<(&'static str, Box<dyn Flow>, &'static str)> {
        vec![
            (
                "litter_n_transfer",
                Box::new(litter_n_transfer_flow(k_o2)) as Box<dyn Flow>,
                LITTER_N,
            ),
            (
                "microbial_n_release",
                Box::new(microbial_n_release_flow(k_o2)) as Box<dyn Flow>,
                MICROBIAL_N,
            ),
            (
                "humus_n_release",
                Box::new(humus_n_release_flow(k_o2)) as Box<dyn Flow>,
                HUMUS_N,
            ),
        ]
    }

    /// F1 — each humified flow sends **its own** cited share to CO2, and the three shares
    /// are DIFFERENT numbers.
    ///
    /// The withdrawals are hand-computed from `k · pool · dt` at `f_O2 ≡ 1` (`k_o2 = 0`,
    /// which the loader permits and which the Python fixtures use for the same reason):
    ///
    /// * litter    `0.011 · 4 = 0.044` mol C/day, 45 % of it (`0.0198`) to CO2
    /// * microbial `0.016 · 2 = 0.032` mol C/day, 85 % of it (`0.0272`) to CO2
    /// * humus     `0.000542857142857142857 · 100 = 0.05428571428571428…`, 55 % to CO2
    ///
    /// ⚠ The three shares are 0.45 / 0.85 / 0.55 and the test drives all three flows in
    /// one table, so a wiring that read one flow's fraction into another — the failure a
    /// single-flow test structurally cannot see — reddens here. Each flow's O2 leg is
    /// asserted to be the NEGATIVE of its own CO2 leg, which states the PQ = 1
    /// stoichiometry where it can be wrong rather than leaving it to the composition gate.
    /// Mirrors `test_decomposition_PARTITIONS_the_decayed_litter_into_co2_and_microbes`
    /// and `test_respiration_burns_microbial_to_co2_consuming_o2`.
    #[test]
    fn the_three_humified_flows_each_send_their_own_cited_share_to_co2() {
        let s = soil_fixture(F_FULL_O2);
        for (label, legs, donor, receiver, moved, share) in [
            (
                "litter",
                soil_legs(&decomposition_flow(0.0), &s, 1.0),
                LITTER_C,
                MICROBIAL_C,
                0.044,
                F_CO2_LITTER,
            ),
            (
                "microbial",
                soil_legs(&microbial_respiration_flow(0.0), &s, 1.0),
                MICROBIAL_C,
                HUMUS_C,
                0.032,
                F_CO2_MICROBIAL,
            ),
            (
                "humus",
                soil_legs(&humus_decomposition_flow(0.0), &s, 1.0),
                HUMUS_C,
                MICROBIAL_C,
                F_K_HUMUS * 100.0,
                F_CO2_HUMUS,
            ),
        ] {
            assert!(
                f_close(-legs[donor], moved),
                "{label}: withdrew {} not {moved}",
                -legs[donor]
            );
            let respired = moved * share;
            assert!(
                f_close(legs[CO2], respired),
                "{label}: {} to CO2, not {respired}",
                legs[CO2]
            );
            assert!(
                f_close(legs[receiver], moved - respired),
                "{label}: {} stabilised, not {}",
                legs[receiver],
                moved - respired
            );
            // PQ = 1 on the RESPIRED leg: one O2 consumed per carbon burned, and not per
            // carbon moved. A flow drawing O2 against the whole withdrawal would still
            // balance OXYGEN — but only by sending the whole withdrawal to CO2, which the
            // two assertions above rule out.
            assert_eq!(legs[O2], -legs[CO2], "{label}: the O2 draw is the burned C");
        }
    }

    /// F2 — every decomposer flux is first-order in ITS OWN donor and self-limits at an
    /// empty one.
    ///
    /// Doubling the donor doubles the withdrawal (`k · pool`), and an empty donor gives
    /// every leg exactly zero — which is what makes positivity structural rather than
    /// clamped. Asserted per flow with the other two pools held fixed, so a flow reading
    /// the wrong pool's carbon reddens instead of merely returning a different number.
    /// Mirrors `test_flux_is_first_order_in_litter`, `test_flux_is_zero_at_zero_litter`,
    /// `test_flux_is_first_order_in_microbial`, `test_flux_is_zero_at_zero_microbial`,
    /// `test_decomposition_self_limits_at_zero_litter` and
    /// `test_respiration_self_limits_at_zero_microbial`.
    #[test]
    fn every_decomposer_flux_is_first_order_in_its_own_donor_and_zero_at_an_empty_one() {
        let base = soil_fixture(F_FULL_O2);
        for (label, flow, donor) in carbon_soil_flows(0.0) {
            let one = soil_legs(flow.as_ref(), &base, 1.0);
            let mut doubled = base.clone();
            doubled.stocks.get_mut(donor).expect("donor").amount *= 2.0;
            let two = soil_legs(flow.as_ref(), &doubled, 1.0);
            assert_eq!(
                two[donor].to_bits(),
                (2.0 * one[donor]).to_bits(),
                "{label} is not first-order in its own donor"
            );
            let mut empty = base.clone();
            empty.stocks.get_mut(donor).expect("donor").amount = 0.0;
            for (id, amount) in soil_legs(flow.as_ref(), &empty, 1.0) {
                assert_eq!(amount, 0.0, "{label}: leg {id} is live at an empty donor");
            }
        }
    }

    /// F3 — every soil leg is bit-exactly linear in `dt` (the increment-form contract).
    ///
    /// All six flows, every leg: `leg(dt) == dt · leg(1)` to the bit. Stated over the
    /// whole leg vector rather than one representative leg, because a partition applied
    /// before the `dt` scaling rather than after would leave the withdrawal linear and one
    /// destination not.
    /// Mirrors `test_decomposition_is_dt_linear`, `test_respiration_is_dt_linear` and
    /// `test_flows_are_dt_linear`.
    #[test]
    fn every_soil_leg_is_bit_exactly_linear_in_dt() {
        let s = soil_fixture(F_FULL_O2);
        let mut flows = carbon_soil_flows(F_K_O2);
        flows.extend(nitrogen_soil_flows(F_K_O2));
        for (label, flow, _) in &flows {
            let one = soil_legs(flow.as_ref(), &s, 1.0);
            for dt in [0.25, 0.5] {
                for (id, amount) in soil_legs(flow.as_ref(), &s, dt) {
                    assert_eq!(
                        amount.to_bits(),
                        (dt * one[&id]).to_bits(),
                        "{label}: {id} at dt={dt}"
                    );
                }
            }
        }
    }

    /// F4 — the partition complement is computed by SUBTRACTION, and that is measurable.
    ///
    /// `respired_and_stabilized`'s own doc comment says the complement is `moved -
    /// respired` rather than `moved · (1 - f)` "so the two destination legs sum back to
    /// the withdrawal exactly in floating point and no partition round-off reaches the
    /// conservation gate".
    ///
    /// ⚠ Asserting `respired + stabilized == moved` ALONE would be a tautology: it cannot
    /// fail for the subtraction form at any input, so it is an assertion its subject
    /// cannot move. The claim is only worth something with a control showing the rejected
    /// form genuinely differs somewhere, and at most inputs it does not — `0.044` at
    /// `f = 0.45` gives bit-identical complements both ways.
    ///
    /// `moved = 5e-5` at `f = 0.45` is an input where they part, found by search: the
    /// multiplication complement is `2.7500000000000004e-5` against the subtraction's
    /// `2.75e-5`, and its sum with the respired share overshoots `moved` by one ulp. Both
    /// halves are stated, which is what makes the exactness claim falsifiable at all.
    #[test]
    fn the_partition_complement_is_computed_by_subtraction_and_that_is_measurable() {
        const MOVED: f64 = 5e-5;
        let (respired, stabilized) = respired_and_stabilized(MOVED, F_CO2_LITTER);
        assert_eq!(respired + stabilized, MOVED, "the legs must sum exactly");
        assert_eq!(stabilized.to_bits(), (MOVED - respired).to_bits());
        // ...and the control. Without it the two assertions above hold for EVERY
        // implementation of the partition, including the one the doc comment rejects.
        let multiplied = MOVED * (1.0 - F_CO2_LITTER);
        assert_ne!(
            stabilized.to_bits(),
            multiplied.to_bits(),
            "the fixture does not distinguish the two forms"
        );
        assert_ne!(respired + multiplied, MOVED);
    }

    /// F5 — the O2 throttle is live on all SIX soil flows, and it is measured where it
    /// BITES rather than where its subject cannot move it.
    ///
    /// ⚠⚠ This is batch G's headline defect stated as a precondition rather than found
    /// afterwards. At the chamber's own fill `x_O2 = 0.21` against `K_O2 = 1e-4`, so
    /// `f_O2 = 0.99952`: deleting the factor outright moves every number by 5e-4, which no
    /// tolerance loose enough to be written would catch. Every assertion here is therefore
    /// evaluated at a DEPLETED pool, where the factor is the difference between a half and
    /// a whole.
    ///
    /// The knot and its shape, hand-computed from `f = x/(K + x)` with `x = o2/1000`:
    ///
    /// * `o2 = 0.1` mol → `x = 1e-4 = K` → `f = 1/2` exactly
    /// * `o2 = 0.3` mol → `x = 3e-4`     → `f = 3/4` exactly
    /// * `o2 = 0.9` mol → `x = 9e-4`     → `f = 9/10`
    ///
    /// ⚠ Three points, not one. A pin at the half-saturation knot alone is satisfied by
    /// any curve through it — `x/(2K + x)` would give 1/3 and 3/5 at the other two and
    /// 1/2 nowhere near — so a wrong `K` or a wrong FORM is invisible to it. That is batch
    /// G's mutual-shading finding (a pin AT the knot is blind to the shape either side of
    /// it) applied before writing rather than found after.
    /// Mirrors `test_release_leg_throttles_with_f_o2_rather_than_ignoring_it` and the
    /// `o2_half_saturation = 0` isolation every Python fixture in this batch relies on.
    #[test]
    fn the_oxygen_throttle_is_live_on_all_six_soil_flows_and_shaped_either_side_of_its_knot() {
        let mut flows = carbon_soil_flows(F_K_O2);
        flows.extend(nitrogen_soil_flows(F_K_O2));
        for (label, flow, donor) in &flows {
            // The unthrottled withdrawal, recovered from the full-pool one: at x = 0.21
            // the factor is 0.21/(0.21 + K), which is the number this test refuses to
            // assert anything at.
            let at_fill = -soil_legs(flow.as_ref(), &soil_fixture(F_FULL_O2), 1.0)[*donor];
            let unthrottled = at_fill / (0.21 / (0.21 + F_K_O2));
            for (o2, want) in [(0.1, 0.5), (0.3, 0.75), (0.9, 0.9)] {
                let moved = -soil_legs(flow.as_ref(), &soil_fixture(o2), 1.0)[*donor];
                let ratio = moved / unthrottled;
                assert!(
                    (ratio - want).abs() < 1e-12,
                    "{label} at o2={o2}: f_O2 reads {ratio}, not {want}"
                );
            }
        }
    }

    /// F6 — `carried_nitrogen` moves the donor pool's OWN ratio, and is zero at every
    /// degenerate input.
    ///
    /// `moved_C · (pool_N / pool_C)`: a hand value (`0.5 · 2/10 = 0.1`) and linearity in
    /// the carbon moved. The degenerate half is what makes positivity structural — an
    /// empty or absent pool moves nothing, so there is never a divide-by-zero and never a
    /// negative leg — and it includes a NEGATIVE `moved_carbon`, which the guard rejects
    /// rather than propagating into a leg that would deposit into its own donor.
    /// Mirrors `test_carried_nitrogen_moves_the_donor_pools_own_ratio` and
    /// `test_carried_nitrogen_is_zero_at_every_degenerate_input`.
    #[test]
    fn carried_nitrogen_moves_the_donor_pools_own_ratio_and_is_zero_at_every_degenerate_input() {
        assert!(f_close(carried_nitrogen(0.5, 2.0, 10.0), 0.1));
        assert!(f_close(carried_nitrogen(1.0, 2.0, 10.0), 0.2));
        assert_eq!(carried_nitrogen(0.0, 2.0, 10.0), 0.0);
        assert_eq!(carried_nitrogen(0.5, 0.0, 10.0), 0.0);
        assert_eq!(carried_nitrogen(0.5, 2.0, 0.0), 0.0);
        assert_eq!(carried_nitrogen(-1.0, 2.0, 10.0), 0.0);
    }

    /// F7 — THE IDENTITY THAT RETIRED `mineralization_rate`, pinned as an equivalence.
    ///
    /// `Decomposition` withdraws `k · litter_C`, so the nitrogen riding it is
    /// `k · litter_C · (litter_N / litter_C) == k · litter_N`. The free N rate was
    /// therefore never independent: stoichiometry forces it to equal the carbon decay
    /// rate, which is *why* retiring it was a citation upgrade rather than a
    /// recalibration.
    ///
    /// ⚠ Pinned as an EQUIVALENCE, never as an implementation. The flow deliberately does
    /// NOT collapse to `decomposition_rate · litter_n`, because the identity holds only
    /// while `Decomposition` stays first-order and the collapsed form would read
    /// identically today and silently outlive that premise.
    ///
    /// ⚠⚠ **And the first draft's attempt to state that second half was WRONG, in a way
    /// worth recording rather than quietly fixing.** It halved the litter CARBON while
    /// holding the nitrogen, expecting the carried N to halve — but the pool's own ratio
    /// is the multiplier, so `k · C · (N/C)` cancels `C` exactly and the number does not
    /// move. It measured `0.022` against an expected `0.011` and failed. *The carbon
    /// amount is the one input this identity is structurally blind to*, which is the whole
    /// reason it is an identity. What DOES separate the two forms is any input the carbon
    /// flux carries and a bare rate does not: the `f_O2` throttle (F5, F9) and an empty
    /// carbon pool under standing nitrogen (F10). The second half below uses the first of
    /// those, so this test rules out the collapsed form rather than merely describing it.
    /// Mirrors `test_carried_n_on_a_first_order_carbon_flux_is_that_same_rate`.
    #[test]
    fn the_nitrogen_riding_a_first_order_carbon_flux_is_that_same_carbon_rate() {
        let s = soil_fixture(F_FULL_O2);
        let withdrawn = -soil_legs(&litter_n_transfer_flow(0.0), &s, 1.0)[LITTER_N];
        assert!(
            f_close(withdrawn, F_K_LITTER * 2.0),
            "the carried N reads {withdrawn}, not k · litter_n"
        );
        // ...and it is the CARBON flux that carries it, not the rate: at f_O2 = 1/2 the
        // collapsed `k · litter_n` returns the unchanged 0.022 and this flow returns half.
        let throttled =
            -soil_legs(&litter_n_transfer_flow(F_K_O2), &soil_fixture(0.1), 1.0)[LITTER_N];
        assert!(
            f_close(throttled, withdrawn / 2.0),
            "the carried N does not follow the carbon flux: {throttled}"
        );
    }

    /// F8 — each nitrogen leg splits its pool EXACTLY as its carbon sibling splits the
    /// carbon, and the three splits are different numbers.
    ///
    /// The withdrawal is `carried_nitrogen` at the donor's own N:C; the partition is the
    /// sibling's own CO2 share, because *the nitrogen of the carbon that left as CO2* is
    /// what mineralizes. That is not an approximation chosen for tidiness: the textbook
    /// mineralization/immobilization balance reduces to it exactly when donor and receiver
    /// carry the same C:N, which is this tree's own stoichiometry (no homeostatic
    /// microbial C:N — measured and refused). Hand-computed on the fixture at `f_O2 ≡ 1`:
    ///
    /// * litter:    `0.011 · 4 · (2/4) = 0.022` N, 45 % (`0.0099`) to soil
    /// * microbial: `0.016 · 2 · (0.05/2) = 8e-4` N, 85 % (`6.8e-4`) to soil
    /// * humus:     `k_h · 100 · (0.1/100)` N, 55 % to soil
    ///
    /// ⚠ The three N:C ratios in the fixture are distinct, so a leg reading the wrong
    /// pool's nitrogen reddens rather than returning a plausible number.
    /// Mirrors `test_litter_transfer_splits_its_nitrogen_the_way_the_carbon_split`,
    /// `test_microbial_release_splits_its_nitrogen_the_way_the_carbon_split` and
    /// `test_mineralization_is_the_nitrogen_of_the_carbon_that_LEFT_AS_CO2`.
    #[test]
    fn each_nitrogen_leg_splits_its_pool_exactly_as_its_carbon_sibling_splits_the_carbon() {
        let s = soil_fixture(F_FULL_O2);
        for (label, legs, donor, mineral_share, other, moved) in [
            (
                "litter",
                soil_legs(&litter_n_transfer_flow(0.0), &s, 1.0),
                LITTER_N,
                F_CO2_LITTER,
                MICROBIAL_N,
                0.022,
            ),
            (
                "microbial",
                soil_legs(&microbial_n_release_flow(0.0), &s, 1.0),
                MICROBIAL_N,
                F_CO2_MICROBIAL,
                HUMUS_N,
                8e-4,
            ),
            (
                "humus",
                soil_legs(&humus_n_release_flow(0.0), &s, 1.0),
                HUMUS_N,
                F_CO2_HUMUS,
                MICROBIAL_N,
                F_K_HUMUS * 100.0 * 0.001,
            ),
        ] {
            assert!(
                f_close(-legs[donor], moved),
                "{label}: withdrew {} not {moved}",
                -legs[donor]
            );
            let mineralized = moved * mineral_share;
            assert!(
                f_close(legs[SOIL_N], mineralized),
                "{label}: {} mineralized, not {mineralized}",
                legs[SOIL_N]
            );
            assert!(
                f_close(legs[other], moved - mineralized),
                "{label}: {} to {other}, not {}",
                legs[other],
                moved - mineralized
            );
        }
    }

    /// F9 — the nitrogen legs recompute EXACTLY what their carbon siblings move, `f_O2`
    /// included, and the check is run where `f_O2` is not 1.
    ///
    /// A flow may read only the step-entry snapshot, so there is no channel by which
    /// `Decomposition` could hand `LitterNitrogenTransfer` its computed flux —
    /// recomputation from the same rate on the same snapshot is the only pure form. The
    /// hazard that creates is silent drift if someone changes one and not the other, and
    /// the symptom would be a wrong pool C:N rather than a crash. So the agreement is
    /// pinned against the ACTUAL sibling flow, bit for bit, not against a re-derivation.
    ///
    /// ⚠ Run at `o2 = 0.1` mol, i.e. `f_O2 = 0.5`. At the chamber fill the factor is
    /// 0.99952 and an N leg that dropped it would agree with its sibling to 5e-4 — inside
    /// any tolerance loose enough to be written. Same trap as F5, one flow further on, and
    /// the third assertion is the control that says the throttle really is biting here
    /// rather than the two flows agreeing because both ignore it.
    /// Mirrors `test_transfer_leg_recomputes_EXACTLY_the_carbon_Decomposition_moves` and
    /// `test_release_leg_recomputes_EXACTLY_the_carbon_MicrobialRespiration_burns`.
    #[test]
    fn each_nitrogen_leg_recomputes_its_siblings_carbon_flux_with_the_o2_throttle_biting() {
        let s = soil_fixture(0.1); // x_O2 = 1e-4 = K, so f_O2 = 1/2 exactly
        for (label, carbon, nitrogen, free, c_donor, n_donor, n_per_c) in [
            (
                "litter",
                soil_legs(&decomposition_flow(F_K_O2), &s, 1.0),
                soil_legs(&litter_n_transfer_flow(F_K_O2), &s, 1.0),
                soil_legs(&decomposition_flow(0.0), &s, 1.0),
                LITTER_C,
                LITTER_N,
                2.0 / 4.0,
            ),
            (
                "microbial",
                soil_legs(&microbial_respiration_flow(F_K_O2), &s, 1.0),
                soil_legs(&microbial_n_release_flow(F_K_O2), &s, 1.0),
                soil_legs(&microbial_respiration_flow(0.0), &s, 1.0),
                MICROBIAL_C,
                MICROBIAL_N,
                0.05 / 2.0,
            ),
            (
                "humus",
                soil_legs(&humus_decomposition_flow(F_K_O2), &s, 1.0),
                soil_legs(&humus_n_release_flow(F_K_O2), &s, 1.0),
                soil_legs(&humus_decomposition_flow(0.0), &s, 1.0),
                HUMUS_C,
                HUMUS_N,
                0.1 / 100.0,
            ),
        ] {
            let moved_c = -carbon[c_donor];
            let moved_n = -nitrogen[n_donor];
            assert_eq!(
                moved_n.to_bits(),
                (moved_c * n_per_c).to_bits(),
                "{label}: the N leg does not ride its sibling's carbon"
            );
            assert!(
                moved_c < 0.75 * -free[c_donor],
                "{label}: f_O2 is not biting in this fixture, so the pin says nothing"
            );
        }
    }

    /// F10 — no nitrogen leaves a soil pool whose CARBON is not moving.
    ///
    /// The half the retired free-rate form could not express: under a direct
    /// `litter_n -> soil_n` jump at a `mineralization_rate`, a standing N pool mineralized
    /// every step regardless of whether any carbon was decomposing. That decoupling is
    /// exactly what the microbe-mediated form removes, so it is pinned from the zero end —
    /// a full nitrogen pool over an EMPTY carbon pool must move nothing at all.
    ///
    /// ⚠ And the inverse, which is the mediation claim that SURVIVED the humification
    /// split: with carbon moving, the soil leg is strictly between zero and the whole
    /// withdrawal. Before the split the guard was "no litter N ever reaches soil N in one
    /// step", and that stopped being true when part of the litter carbon began leaving as
    /// CO2 at the litter step. The two-sided bound is the form that is still true, and a
    /// re-collapsed direct mineralization — which would send the WHOLE withdrawal to
    /// soil — still fails it. *A pin guarding a mechanism you removed is decoration.*
    /// Mirrors `test_return_legs_self_limit_when_no_carbon_is_moving`,
    /// `test_return_legs_self_limit_at_an_empty_donor` and
    /// `test_only_the_RESPIRED_share_reaches_soil_n_the_rest_still_transits`.
    #[test]
    fn no_nitrogen_leaves_a_soil_pool_whose_carbon_is_not_moving() {
        let full = soil_fixture(F_FULL_O2);
        for ((label, flow, n_donor), c_donor) in
            nitrogen_soil_flows(F_K_O2)
                .into_iter()
                .zip([LITTER_C, MICROBIAL_C, HUMUS_C])
        {
            let mut no_carbon = full.clone();
            no_carbon.stocks.get_mut(c_donor).expect("carbon").amount = 0.0;
            no_carbon.stocks.get_mut(n_donor).expect("nitrogen").amount = 5.0;
            for (id, amount) in soil_legs(flow.as_ref(), &no_carbon, 1.0) {
                assert_eq!(amount, 0.0, "{label}: leg {id} moved with no carbon");
            }
            let mut no_nitrogen = full.clone();
            no_nitrogen
                .stocks
                .get_mut(n_donor)
                .expect("nitrogen")
                .amount = 0.0;
            for (id, amount) in soil_legs(flow.as_ref(), &no_nitrogen, 1.0) {
                assert_eq!(amount, 0.0, "{label}: leg {id} moved with no nitrogen");
            }
            let legs = soil_legs(flow.as_ref(), &full, 1.0);
            let withdrawn = -legs[n_donor];
            assert!(
                withdrawn > 0.0,
                "{label}: nothing moved on the live fixture"
            );
            assert!(
                legs[SOIL_N] > 0.0 && legs[SOIL_N] < withdrawn,
                "{label}: {} of {withdrawn} reached soil_n",
                legs[SOIL_N]
            );
        }
    }

    /// F18 — no nitrogen is shed by a plant that is not losing tissue, in any of the three
    /// ways it can fail to be.
    ///
    /// ⚠ Added by the batch F review, which found the claim by DIFFING the disposition
    /// table against the 100 input tests rather than by reading it. Batch E ported this
    /// flow's rate law and both arms of its `min`; the degenerate-input half had neither a
    /// successor nor a row, and *a coverage table assembled by reading cannot see its own
    /// omissions.*
    ///
    /// ⚠⚠ **The first draft of this test called itself a GUARD test and it is not one —
    /// measured, not argued.** `NitrogenSenescence` opens with
    /// `if shed_carbon <= 0 || plant_n <= 0 || biomass_c <= 0 { 0.0 }`, and **deleting that
    /// whole condition leaves `cargo test -p domains --lib` entirely green**, because every
    /// disjunct is arithmetically redundant: the body multiplies by `shed_carbon`, so a
    /// zero there is zero either way; `plant_n = 0` makes the concentration zero; and
    /// `biomass_c = 0` gives `inf` or `NaN`, both of which `f64::min` and the multiplication
    /// by a zero `shed_carbon` collapse back to zero. The condition is defensive code, and
    /// no mutation of it can redden anything. The first draft's claim that the divisor arm
    /// "returns NaN without its guard" was simply false — the third time in this batch that
    /// an assertion was written where its subject cannot move it, and the third time in our
    /// own column.
    ///
    /// **What this test actually pins is the COUPLING**, which is the claim the Python
    /// originals are for: under the retired `n_senescence_rate` form, a standing `plant_n`
    /// shed nitrogen every step regardless of whether any tissue was dying. Restoring that
    /// form reddens exactly three tests and this is one of them. The three states below are
    /// three different ways for nothing to be dying — no nitrogen, no biomass, no death
    /// rate — and an uncoupled form sheds in the last two.
    /// Mirrors `test_n_shedding_flux_is_zero_at_every_degenerate_input`,
    /// `test_n_senescence_self_limits_at_zero_plant_n` and
    /// `test_n_senescence_self_limits_when_no_carbon_is_senescing`.
    #[test]
    fn the_shed_nitrogen_is_zero_on_each_of_its_three_degenerate_arms() {
        let sen = params::senescence();
        let canopy = params::canopy();
        let nitro = params::nitrogen();
        let flow = |rdr: f64| NitrogenSenescence {
            id: "biosphere.nitrogen_senescence".to_string(),
            plant_n: PLANT_N.to_string(),
            litter_n: LITTER_N.to_string(),
            leaf_c: LEAF.to_string(),
            stem_c: STEM.to_string(),
            root_c: ROOT.to_string(),
            rdr_leaf: rdr * sen.rdr_leaf,
            rdr_stem: rdr * sen.rdr_stem,
            rdr_root: rdr * sen.rdr_root,
            n_residual_per_mol_c: nitro.n_residual_per_mol_c,
            shade_rate: rdr * sen.shade_rate,
            lai_threshold: sen.lai_threshold,
            sla_per_mol_c: canopy.sla_per_mol_c,
            ground_area: 1.0,
        };
        for (label, rdr, s) in [
            // no nitrogen in a live, senescing plant
            ("empty plant_n", 1.0, n_state(3.0, 1.0, 1.0, 0.0, 0.0, 1.0)),
            // no biomass at all, but nitrogen still standing: the state an uncoupled
            // rate form would shed from most obviously
            ("empty biomass", 1.0, n_state(0.0, 0.0, 0.0, 0.0, 1.0, 1.0)),
            // organs and nitrogen both present, but nothing is dying
            (
                "nothing senescing",
                0.0,
                n_state(3.0, 1.0, 1.0, 0.0, 1.0, 1.0),
            ),
        ] {
            let r = SourceResolver::new(HashMap::new(), HashMap::new()).expect("resolver");
            let env = r.bind(&s, 1.0);
            for l in flow(rdr).evaluate(&s, &env, 1.0).expect("shed").legs {
                assert_eq!(l.amount, 0.0, "{label}: leg {} is live", l.stock);
            }
        }
        // ...and the control: the same flow on a live plant sheds a strictly positive
        // amount, so the three zeros above are the coupling rather than an inert fixture.
        let live = n_state(3.0, 1.0, 1.0, 0.0, 1.0, 1.0);
        let r = SourceResolver::new(HashMap::new(), HashMap::new()).expect("resolver");
        let env = r.bind(&live, 1.0);
        let shed = flow(1.0)
            .evaluate(&live, &env, 1.0)
            .expect("shed")
            .legs
            .iter()
            .find(|l| l.stock == LITTER_N)
            .map(|l| l.amount)
            .expect("a litter_n leg");
        assert!(shed > 0.0, "the fixture does not shed at all");
    }
}
