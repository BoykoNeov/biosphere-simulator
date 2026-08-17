//! The biosphere coefficients, **loaded from the frozen param YAML** (reference flip,
//! slice C1).
//!
//! # What changed
//!
//! Until C1 this module read `biosphere_params.txt` — a hex-float table that
//! `tests/crossport/gen_biosphere_params.py` produced by running the *Python* loaders,
//! **including their two core-ready folds**. So the schema, the unit guards, the bounds
//! and the folds all executed on the Python side and the port consumed the result. Now
//! this module does that work itself, through the [`config`] boundary crate.
//!
//! # ⚠ The two folds, and why their op order is copied rather than tidied
//!
//! The pure core never sees kg dry matter or the molar mass, so the boundary folds:
//!
//! * `sla_per_mol_c = sla[m²/kg DM] · M_C / carbon_fraction` (canopy), and
//! * `fold = M_C / carbon_fraction`, then `n_{residual,critical} · fold` (nitrogen).
//!
//! Note the two **associate differently** — canopy multiplies first, nitrogen divides
//! first. Measured before this was written (`§5d` of the plan): on today's values both
//! orders give identical bits, so the difference is currently inert. That is a fact
//! about these four numbers and **not** a licence to re-associate: a value change could
//! split them, and this port is the reference. The order below is the Python loaders'
//! order, deliberately.
//!
//! # Bit-neutrality is gated, not asserted
//!
//! `biosphere_params.txt` is **retained as the control**:
//! [`tests::every_value_matches_the_generated_table`] walks all 66 scalars and the
//! partition table and asserts bit equality against it. The generator is retired only
//! once that gate has been green, per the rule §5c sets for every generator.
//!
//! ⚠ `allocation.yaml` had to be **reformatted out of YAML flow style** for this slice —
//! the closed-subset reader rejects `- {dvs: 0.0, …}` by design. No value moved (the
//! generator reproduces its file byte-for-byte), but the file's sha-256 did, which is a
//! provenance unfreeze **no test can see**; see the ceremony record in
//! `docs/biosphere-reference.md`.

use config::{
    require_closed, require_half_open, require_non_negative, require_positive, ConfigError,
    ParamFile, YamlValue,
};

/// The kg dry-matter ↔ mol carbon bridge: M_C = 12.011 g/mol (IUPAC conventional
/// standard atomic weight of carbon, [12.0096, 12.0116]).
///
/// **Crop data, not generic dimensional analysis** — mol (substance) and kg (mass) are
/// dimensionally incompatible without a molar mass, which is exactly why Python keeps
/// this in the domain loader rather than in its units boundary, and why it lives here
/// rather than in [`config`].
pub const MOLAR_MASS_CARBON_KG_PER_MOL: f64 = 0.012011;

const CANOPY_YAML: &str = include_str!("../../../../../src/domains/biosphere/params/canopy.yaml");
const PHOTOSYNTHESIS_YAML: &str =
    include_str!("../../../../../src/domains/biosphere/params/photosynthesis.yaml");
const RESPIRATION_YAML: &str =
    include_str!("../../../../../src/domains/biosphere/params/respiration.yaml");
const TRANSPIRATION_YAML: &str =
    include_str!("../../../../../src/domains/biosphere/params/transpiration.yaml");
const PHENOLOGY_YAML: &str =
    include_str!("../../../../../src/domains/biosphere/params/phenology.yaml");
const SENESCENCE_YAML: &str =
    include_str!("../../../../../src/domains/biosphere/params/senescence.yaml");
const ROOT_DEPTH_YAML: &str =
    include_str!("../../../../../src/domains/biosphere/params/root_depth.yaml");
const STEM_RESERVES_YAML: &str =
    include_str!("../../../../../src/domains/biosphere/params/stem_reserves.yaml");
const NITROGEN_YAML: &str =
    include_str!("../../../../../src/domains/biosphere/params/nitrogen.yaml");
const DECOMPOSITION_YAML: &str =
    include_str!("../../../../../src/domains/biosphere/params/decomposition.yaml");
const MICROBIAL_RESPIRATION_YAML: &str =
    include_str!("../../../../../src/domains/biosphere/params/microbial_respiration.yaml");
const HUMIFICATION_YAML: &str =
    include_str!("../../../../../src/domains/biosphere/params/humification.yaml");
const WATER_CYCLE_YAML: &str =
    include_str!("../../../../../src/domains/biosphere/params/water_cycle.yaml");
const HERBIVORY_YAML: &str =
    include_str!("../../../../../src/domains/biosphere/params/herbivory.yaml");
const ALLOCATION_YAML: &str =
    include_str!("../../../../../src/domains/biosphere/params/allocation.yaml");

/// The `parameters` block of `phenology.yaml`, which **three** loaders read.
///
/// ⚠ One file, three structs: thermal-time phenology, vernalization and photoperiod all
/// live in `phenology.yaml` and each loader validates the *whole* 12-field block (the
/// `extra="forbid"` half). That is why the biosphere manifest's `param_files` has 15
/// entries against 17 loaders.
const PHENOLOGY_UNITS: [(&str, &str); 12] = [
    ("t_base", "degC"),
    ("t_cap", "degC"),
    ("tsum_anthesis", "degC*day"),
    ("tsum_maturity", "degC*day"),
    ("t_base_v", "degC"),
    ("t_opt_lower_v", "degC"),
    ("t_opt_upper_v", "degC"),
    ("t_ceiling_v", "degC"),
    ("vsen", "1/day"),
    ("vdsat", "day"),
    ("cpp", "h"),
    ("ppsen", "1/h"),
];

/// Absolute tolerance on a partition row summing to 1 (mirrors Python's
/// `_PARTITION_SUM_ATOL`).
const PARTITION_SUM_ATOL: f64 = 1e-9;

fn file(text: &str, name: &'static str) -> ParamFile {
    ParamFile::parse(text, name).unwrap_or_else(|e| panic!("{name} is malformed: {e}"))
}

fn checked<T>(result: Result<T, ConfigError>, name: &'static str) -> T {
    result.unwrap_or_else(|e| panic!("{name} failed its frozen bound/unit check: {e}"))
}

/// Read a whole `field → unit` table into a name-keyed lookup, guarding every unit and
/// rejecting any `parameters` key the table does not name.
fn guarded_map(
    f: &ParamFile,
    units: &[(&str, &str)],
    name: &'static str,
) -> std::collections::BTreeMap<String, f64> {
    let values = checked(f.guarded_set(units, name), name);
    units
        .iter()
        .map(|(field, _)| (*field).to_string())
        .zip(values)
        .collect()
}

/// The `carbon_fraction` bound, kg C per kg DM ∈ (0, 1] — shared by the two files that
/// fold with it, and by the same argument in both.
fn carbon_fraction(value: f64, name: &'static str) -> f64 {
    checked(
        require_half_open(value, 0.0, 1.0, "carbon_fraction", name),
        name,
    )
}

/// Beer–Lambert canopy params (core-ready — `sla_per_mol_c` is pre-folded).
#[derive(Debug, Clone, Copy)]
pub struct CanopyParams {
    pub sla_per_mol_c: f64,
    pub extinction_coef: f64,
}

/// FvCB photosynthesis params (reference temperature).
#[derive(Debug, Clone, Copy)]
pub struct PhotosynthesisParams {
    pub vcmax: f64,
    pub jmax: f64,
    pub quantum_yield: f64,
    pub theta: f64,
    pub gamma_star: f64,
    pub kc: f64,
    pub ko: f64,
    pub o2: f64,
    pub t_min: f64,
    pub t_opt_lo: f64,
    pub t_opt_hi: f64,
    pub t_max: f64,
}

/// Maintenance + growth respiration params.
#[derive(Debug, Clone, Copy)]
pub struct RespirationParams {
    pub maintenance_coef: f64,
    pub q10: f64,
    pub t_ref: f64,
    pub growth_efficiency: f64,
    pub o2_half_saturation: f64,
}

/// Penman–Monteith transpiration params.
#[derive(Debug, Clone, Copy)]
pub struct TranspirationParams {
    pub aerodynamic_resistance: f64,
    pub surface_resistance: f64,
}

/// Thermal-time phenology params.
#[derive(Debug, Clone, Copy)]
pub struct PhenologyParams {
    pub t_base: f64,
    pub t_cap: f64,
    pub tsum_anthesis: f64,
    pub tsum_maturity: f64,
}

/// Vernalization (cold-requirement) params — Soltani & Sinclair (2012) Ch. 8.
#[derive(Debug, Clone, Copy)]
pub struct VernalizationParams {
    pub t_base_v: f64,
    pub t_opt_lower_v: f64,
    pub t_opt_upper_v: f64,
    pub t_ceiling_v: f64,
    pub vsen: f64,
    pub vdsat: f64,
}

/// Photoperiod (daylength) params — Soltani & Sinclair (2012) Ch. 7, long-day form.
#[derive(Debug, Clone, Copy)]
pub struct PhotoperiodParams {
    pub cpp: f64,
    pub ppsen: f64,
}

/// Drought-acceleration params — Soltani & Sinclair (2012) Ch. 15, Eqn 15.8.
///
/// ⚠ `wssd` is a COEFFICIENT, not a threshold — Table 15.1's caption names it "a
/// coefficient of phenological development response to drought", and it scales an
/// already-computed `WSFG` rather than being compared against `FTSW`. The soil geometry
/// rides along because `WSFD` is defined THROUGH `WSFG`, which is defined on
/// `FTSW = ATSW/TTSW`; using anything but the same three values the other consumers use
/// would let phenology and growth disagree about the stress state inside one step.
#[derive(Debug, Clone, Copy)]
pub struct DroughtDevelopmentParams {
    pub wssd: f64,
    pub wssg: f64,
    pub soil_extractable_water: f64,
    pub ground_area: f64,
}

/// Per-organ relative senescence (death) rates.
#[derive(Debug, Clone, Copy)]
pub struct RootDepthParams {
    pub max_extension_rate: f64,
    pub max_rooted_depth: f64,
}

/// Stem-reserve remobilization (the stem feeding the grain).
///
/// Mirrors `domains.biosphere.stem_reserves.StemReserveParams`. `cessation_dvs` closes
/// BOTH halves of the mechanism at maturity - [E]'s Listing 3 Line 114 is
/// `FINISH DS = 2., CELVN = 3.`, i.e. its program has no state past maturity, so running
/// the form there would extrapolate it outside the program that defines it. See the
/// Python module for the quotes and for why that is a DOMAIN BOUNDARY rather than a
/// cited cessation rule.
#[derive(Debug, Clone, Copy)]
pub struct StemReserveParams {
    pub remobilizable_fraction: f64,
    pub remobilization_rate: f64,
    pub trigger_dvs: f64,
    pub cessation_dvs: f64,
}

/// Biomass senescence (relative organ death rates).
#[derive(Debug, Clone, Copy)]
pub struct SenescenceParams {
    pub rdr_leaf: f64,
    pub rdr_stem: f64,
    pub rdr_root: f64,
    /// Additional leaf relative death rate above `lai_threshold` (mutual shading).
    pub shade_rate: f64,
    pub lai_threshold: f64,
}

/// Nitrogen uptake + limitation params (core-ready — thresholds pre-folded to kg N/mol C).
#[derive(Debug, Clone, Copy)]
pub struct NitrogenParams {
    pub max_uptake_capacity: f64,
    pub n_residual_per_mol_c: f64,
    pub n_critical_per_mol_c: f64,
    pub n_target_coefficient: f64,
    pub n_target_exponent: f64,
    pub n_target_w_plateau: f64,
    pub dm_kg_per_mol_c: f64,
}

/// First-order litter-decay param.
#[derive(Debug, Clone, Copy)]
pub struct DecompositionParams {
    pub decomposition_rate: f64,
}

/// First-order microbial-respiration params.
#[derive(Debug, Clone, Copy)]
pub struct MicrobialRespirationParams {
    pub microbial_respiration_rate: f64,
    pub o2_half_saturation: f64,
}

/// The humification split (a carbon-use efficiency) -- CENTURY / Parton et al. 1987.
///
/// Three CO2 fractions that partition every decomposer carbon flux between CO2 and the
/// pool the remainder stabilises into, plus the slow-SOM pool's own first-order rate.
/// The frozen pre-2026-08-10 form implied a litter CO2 fraction of 0.0 and `Es = 1.0`,
/// both outside eq. [6]'s range -- see `params/humification.yaml`.
#[derive(Debug, Clone, Copy)]
pub struct HumificationParams {
    pub litter_respired_fraction: f64,
    pub active_stabilization_co2_fraction: f64,
    pub slow_respired_fraction: f64,
    pub slow_decomposition_rate: f64,
}

// The nitrogen return loop has NO params struct: both rates it ever held were retired
// by FORM changes. `n_senescence_rate` went when shedding became coupled to the senescing
// carbon at a cited residual concentration; `mineralization_rate` went when the return leg
// became microbe-mediated, because `decomposed_C / litter_C == decomposition_rate`
// identically, so the free N rate was redundant with the carbon one. The N legs therefore
// take `DecompositionParams` / `MicrobialRespirationParams`.

/// Water-cycle params (condensation + recycling).
#[derive(Debug, Clone, Copy)]
pub struct WaterCycleParams {
    pub condensation_rate: f64,
    pub recycling_rate: f64,
}

/// Minimal-consumer params (grazing + respiration + mortality + f_O2 Monod).
#[derive(Debug, Clone, Copy)]
pub struct HerbivoryParams {
    pub grazing_rate: f64,
    pub respiration_rate: f64,
    pub mortality_rate: f64,
    pub o2_half_saturation: f64,
}

/// One DVS knot of the leaf/stem/root/storage partition table.
#[derive(Debug, Clone, Copy)]
pub struct PartitionRow {
    pub dvs: f64,
    pub fl: f64,
    pub fs: f64,
    pub fr: f64,
    pub fo: f64,
}

/// DVS-keyed partition table.
#[derive(Debug, Clone)]
pub struct AllocationParams {
    pub table: Vec<PartitionRow>,
}

/// All frozen biosphere coefficients, parsed once from the generated file.
#[derive(Debug, Clone)]
pub struct BiosphereParams {
    pub canopy: CanopyParams,
    pub photo: PhotosynthesisParams,
    pub resp: RespirationParams,
    pub transp: TranspirationParams,
    pub pheno: PhenologyParams,
    pub vern: VernalizationParams,
    pub photoperiod: PhotoperiodParams,
    pub senesc: SenescenceParams,
    pub stem_reserve: StemReserveParams,
    pub rootd: RootDepthParams,
    pub nitro: NitrogenParams,
    pub decomp: DecompositionParams,
    pub micro: MicrobialRespirationParams,
    pub humi: HumificationParams,
    pub water: WaterCycleParams,
    pub herb: HerbivoryParams,
    pub alloc: AllocationParams,
}

/// Beer–Lambert canopy params, with `sla_per_mol_c` folded (`canopy.yaml`).
pub fn canopy() -> CanopyParams {
    const NAME: &str = "canopy.yaml";
    let f = file(CANOPY_YAML, NAME);
    let v = guarded_map(
        &f,
        &[
            ("extinction_coef", "dimensionless"),
            ("specific_leaf_area", "m^2/kg"),
            ("carbon_fraction", "dimensionless"),
        ],
        NAME,
    );
    let cf = carbon_fraction(v["carbon_fraction"], NAME);
    let k = checked(
        require_positive(v["extinction_coef"], "extinction_coef", NAME),
        NAME,
    );
    let sla = checked(
        require_positive(v["specific_leaf_area"], "specific_leaf_area", NAME),
        NAME,
    );
    CanopyParams {
        // ⚠ multiply first, then divide — the Python loader's order. See the header.
        sla_per_mol_c: sla * MOLAR_MASS_CARBON_KG_PER_MOL / cf,
        extinction_coef: k,
    }
}

/// FvCB photosynthesis params (`photosynthesis.yaml`).
pub fn photosynthesis() -> PhotosynthesisParams {
    const NAME: &str = "photosynthesis.yaml";
    let f = file(PHOTOSYNTHESIS_YAML, NAME);
    let v = guarded_map(
        &f,
        &[
            ("vcmax", "umol/m^2/s"),
            ("jmax", "umol/m^2/s"),
            ("quantum_yield", "mol/mol"),
            ("theta", "dimensionless"),
            ("gamma_star", "umol/mol"),
            ("kc", "umol/mol"),
            ("ko", "mmol/mol"),
            ("o2", "mmol/mol"),
            ("t_min", "degC"),
            ("t_opt_lo", "degC"),
            ("t_opt_hi", "degC"),
            ("t_max", "degC"),
        ],
        NAME,
    );
    for field in ["vcmax", "jmax", "gamma_star", "kc", "ko", "o2"] {
        checked(require_positive(v[field], field, NAME), NAME);
    }
    for field in ["quantum_yield", "theta"] {
        checked(require_half_open(v[field], 0.0, 1.0, field, NAME), NAME);
    }
    let (t_min, t_opt_lo, t_opt_hi, t_max) = (v["t_min"], v["t_opt_lo"], v["t_opt_hi"], v["t_max"]);
    // Well-ordered cardinals: the two strict pairs are divisors in the response curve.
    assert!(
        t_min < t_opt_lo && t_opt_lo <= t_opt_hi && t_opt_hi < t_max,
        "{NAME}: cardinal temperatures must satisfy t_min < t_opt_lo <= t_opt_hi < t_max, \
         got ({t_min}, {t_opt_lo}, {t_opt_hi}, {t_max})"
    );
    PhotosynthesisParams {
        vcmax: v["vcmax"],
        jmax: v["jmax"],
        quantum_yield: v["quantum_yield"],
        theta: v["theta"],
        gamma_star: v["gamma_star"],
        kc: v["kc"],
        ko: v["ko"],
        o2: v["o2"],
        t_min,
        t_opt_lo,
        t_opt_hi,
        t_max,
    }
}

/// Maintenance + growth respiration params (`respiration.yaml`).
pub fn respiration() -> RespirationParams {
    const NAME: &str = "respiration.yaml";
    let f = file(RESPIRATION_YAML, NAME);
    let v = guarded_map(
        &f,
        &[
            ("maintenance_coef", "1/day"),
            ("q10", "dimensionless"),
            ("t_ref", "degC"),
            ("growth_efficiency", "dimensionless"),
            ("o2_half_saturation", "mol/mol"),
        ],
        NAME,
    );
    for field in ["maintenance_coef", "q10"] {
        checked(require_positive(v[field], field, NAME), NAME);
    }
    checked(
        require_half_open(v["growth_efficiency"], 0.0, 1.0, "growth_efficiency", NAME),
        NAME,
    );
    checked(
        require_non_negative(v["o2_half_saturation"], "o2_half_saturation", NAME),
        NAME,
    );
    RespirationParams {
        maintenance_coef: v["maintenance_coef"],
        q10: v["q10"],
        t_ref: v["t_ref"],
        growth_efficiency: v["growth_efficiency"],
        o2_half_saturation: v["o2_half_saturation"],
    }
}

/// Penman–Monteith transpiration resistances (`transpiration.yaml`).
pub fn transpiration() -> TranspirationParams {
    const NAME: &str = "transpiration.yaml";
    let f = file(TRANSPIRATION_YAML, NAME);
    let v = guarded_map(
        &f,
        &[
            ("aerodynamic_resistance", "s/m"),
            ("surface_resistance", "s/m"),
        ],
        NAME,
    );
    for field in ["aerodynamic_resistance", "surface_resistance"] {
        checked(require_positive(v[field], field, NAME), NAME);
    }
    TranspirationParams {
        aerodynamic_resistance: v["aerodynamic_resistance"],
        surface_resistance: v["surface_resistance"],
    }
}

/// The whole 12-field `phenology.yaml` block, validated once for all three readers.
fn phenology_block() -> std::collections::BTreeMap<String, f64> {
    let f = file(PHENOLOGY_YAML, "phenology.yaml");
    guarded_map(&f, &PHENOLOGY_UNITS, "phenology.yaml")
}

/// Thermal-time phenology params (`phenology.yaml`).
pub fn phenology() -> PhenologyParams {
    const NAME: &str = "phenology.yaml";
    let v = phenology_block();
    assert!(
        v["t_base"] < v["t_cap"],
        "{NAME}: cardinal temperatures must satisfy t_base < t_cap, got ({}, {})",
        v["t_base"],
        v["t_cap"]
    );
    for field in ["tsum_anthesis", "tsum_maturity"] {
        checked(require_positive(v[field], field, NAME), NAME);
    }
    PhenologyParams {
        t_base: v["t_base"],
        t_cap: v["t_cap"],
        tsum_anthesis: v["tsum_anthesis"],
        tsum_maturity: v["tsum_maturity"],
    }
}

/// Vernalization cardinals (`phenology.yaml`) — Soltani & Sinclair Eqn 8.3 / 8.6.
pub fn vernalization() -> VernalizationParams {
    const NAME: &str = "phenology.yaml";
    let v = phenology_block();
    // A well-ordered response with a strictly positive ramp on each side; the two strict
    // pairs are divisors.
    assert!(
        v["t_base_v"] < v["t_opt_lower_v"]
            && v["t_opt_lower_v"] <= v["t_opt_upper_v"]
            && v["t_opt_upper_v"] < v["t_ceiling_v"],
        "{NAME}: vernalization cardinals must satisfy \
         t_base_v < t_opt_lower_v <= t_opt_upper_v < t_ceiling_v"
    );
    checked(require_positive(v["vdsat"], "vdsat", NAME), NAME);
    // A negative sensitivity would make cold *retard* development.
    checked(require_non_negative(v["vsen"], "vsen", NAME), NAME);
    VernalizationParams {
        t_base_v: v["t_base_v"],
        t_opt_lower_v: v["t_opt_lower_v"],
        t_opt_upper_v: v["t_opt_upper_v"],
        t_ceiling_v: v["t_ceiling_v"],
        vsen: v["vsen"],
        vdsat: v["vdsat"],
    }
}

/// Photoperiod (daylength) params (`phenology.yaml`) — long-day form.
pub fn photoperiod() -> PhotoperiodParams {
    const NAME: &str = "phenology.yaml";
    let v = phenology_block();
    checked(require_positive(v["cpp"], "cpp", NAME), NAME);
    checked(require_non_negative(v["ppsen"], "ppsen", NAME), NAME);
    PhotoperiodParams {
        cpp: v["cpp"],
        ppsen: v["ppsen"],
    }
}

/// Relative organ death rates + the mutual-shading term (`senescence.yaml`).
pub fn senescence() -> SenescenceParams {
    const NAME: &str = "senescence.yaml";
    let units: [(&str, &str); 5] = [
        ("rdr_leaf", "1/day"),
        ("rdr_stem", "1/day"),
        ("rdr_root", "1/day"),
        ("shade_rate", "1/day"),
        ("lai_threshold", "dimensionless"),
    ];
    let f = file(SENESCENCE_YAML, NAME);
    let v = guarded_map(&f, &units, NAME);
    for (field, _) in units {
        checked(require_non_negative(v[field], field, NAME), NAME);
    }
    SenescenceParams {
        rdr_leaf: v["rdr_leaf"],
        rdr_stem: v["rdr_stem"],
        rdr_root: v["rdr_root"],
        shade_rate: v["shade_rate"],
        lai_threshold: v["lai_threshold"],
    }
}

/// Root-extension params (`root_depth.yaml`).
pub fn root_depth() -> RootDepthParams {
    const NAME: &str = "root_depth.yaml";
    let units: [(&str, &str); 2] = [("max_extension_rate", "m/day"), ("max_rooted_depth", "m")];
    let f = file(ROOT_DEPTH_YAML, NAME);
    let v = guarded_map(&f, &units, NAME);
    for (field, _) in units {
        checked(require_positive(v[field], field, NAME), NAME);
    }
    RootDepthParams {
        max_extension_rate: v["max_extension_rate"],
        max_rooted_depth: v["max_rooted_depth"],
    }
}

/// Stem-reserve remobilization (`stem_reserves.yaml`).
pub fn stem_reserves() -> StemReserveParams {
    const NAME: &str = "stem_reserves.yaml";
    let f = file(STEM_RESERVES_YAML, NAME);
    let v = guarded_map(
        &f,
        &[
            ("remobilizable_fraction", "dimensionless"),
            ("remobilization_rate", "1/day"),
            ("trigger_dvs", "dimensionless"),
            ("cessation_dvs", "dimensionless"),
        ],
        NAME,
    );
    let (fstr, rate) = (v["remobilizable_fraction"], v["remobilization_rate"]);
    let (trigger, cessation) = (v["trigger_dvs"], v["cessation_dvs"]);
    assert!(
        0.0 < fstr && fstr < 1.0,
        "{NAME}: remobilizable_fraction must be in (0, 1), got {fstr}"
    );
    checked(
        require_half_open(rate, 0.0, 1.0, "remobilization_rate", NAME),
        NAME,
    );
    checked(require_closed(trigger, 0.0, 2.0, "trigger_dvs", NAME), NAME);
    // ⚠ `cessation_dvs` closes BOTH halves at maturity — a DOMAIN boundary, not a cited
    // cessation rule (see the struct's own doc comment).
    assert!(
        trigger < cessation && cessation <= 2.0,
        "{NAME}: must satisfy trigger_dvs < cessation_dvs <= 2, got ({trigger}, {cessation})"
    );
    StemReserveParams {
        remobilizable_fraction: fstr,
        remobilization_rate: rate,
        trigger_dvs: trigger,
        cessation_dvs: cessation,
    }
}

/// Nitrogen uptake + limitation, with the two thresholds folded (`nitrogen.yaml`).
pub fn nitrogen() -> NitrogenParams {
    const NAME: &str = "nitrogen.yaml";
    let f = file(NITROGEN_YAML, NAME);
    let v = guarded_map(
        &f,
        &[
            ("max_uptake_capacity", "kg/m^2/day"),
            ("n_residual", "kg/kg"),
            ("n_critical", "kg/kg"),
            ("n_target_coefficient", "kg/kg"),
            ("n_target_exponent", "dimensionless"),
            ("n_target_w_plateau", "t/ha"),
            ("carbon_fraction", "dimensionless"),
        ],
        NAME,
    );
    checked(
        require_positive(v["max_uptake_capacity"], "max_uptake_capacity", NAME),
        NAME,
    );
    // ⚠ MUST EQUAL canopy.yaml's carbon_fraction — a divergence models a silently
    // inconsistent plant. Both files fold with it; the dedup is a long-standing nicety.
    let cf = carbon_fraction(v["carbon_fraction"], NAME);
    let (n_residual, n_critical) = (v["n_residual"], v["n_critical"]);
    checked(require_non_negative(n_residual, "n_residual", NAME), NAME);
    assert!(
        n_residual < n_critical,
        "{NAME}: N-concentration thresholds must satisfy n_residual < n_critical, \
         got ({n_residual}, {n_critical})"
    );
    // ⚠ divide first, then multiply — the Python loader's order. See the header.
    let fold = MOLAR_MASS_CARBON_KG_PER_MOL / cf;
    checked(
        require_positive(v["n_target_w_plateau"], "n_target_w_plateau", NAME),
        NAME,
    );
    // The target must sit ABOVE the stress threshold, or the plant is stressed by
    // construction at every crop mass (Greenwood's curve declines, so the plateau is its
    // maximum; if even that is below critical, f_N < 1 always).
    assert!(
        v["n_target_coefficient"] > n_critical,
        "{NAME}: n_target_coefficient must exceed n_critical, got ({}, {n_critical})",
        v["n_target_coefficient"]
    );
    NitrogenParams {
        max_uptake_capacity: v["max_uptake_capacity"],
        n_residual_per_mol_c: n_residual * fold,
        n_critical_per_mol_c: n_critical * fold,
        n_target_coefficient: v["n_target_coefficient"],
        n_target_exponent: v["n_target_exponent"],
        n_target_w_plateau: v["n_target_w_plateau"],
        dm_kg_per_mol_c: fold,
    }
}

/// First-order litter decay (`decomposition.yaml`). Zero is valid (no decomposition).
pub fn decomposition() -> DecompositionParams {
    const NAME: &str = "decomposition.yaml";
    let f = file(DECOMPOSITION_YAML, NAME);
    let v = guarded_map(&f, &[("decomposition_rate", "1/day")], NAME);
    DecompositionParams {
        decomposition_rate: checked(
            require_non_negative(v["decomposition_rate"], "decomposition_rate", NAME),
            NAME,
        ),
    }
}

/// First-order microbial respiration (`microbial_respiration.yaml`).
pub fn microbial_respiration() -> MicrobialRespirationParams {
    const NAME: &str = "microbial_respiration.yaml";
    let units: [(&str, &str); 2] = [
        ("microbial_respiration_rate", "1/day"),
        ("o2_half_saturation", "mol/mol"),
    ];
    let f = file(MICROBIAL_RESPIRATION_YAML, NAME);
    let v = guarded_map(&f, &units, NAME);
    for (field, _) in units {
        checked(require_non_negative(v[field], field, NAME), NAME);
    }
    MicrobialRespirationParams {
        microbial_respiration_rate: v["microbial_respiration_rate"],
        o2_half_saturation: v["o2_half_saturation"],
    }
}

/// The humification split, a carbon-use efficiency (`humification.yaml`).
pub fn humification() -> HumificationParams {
    const NAME: &str = "humification.yaml";
    let f = file(HUMIFICATION_YAML, NAME);
    let v = guarded_map(
        &f,
        &[
            ("litter_respired_fraction", "mol/mol"),
            ("active_stabilization_co2_fraction", "mol/mol"),
            ("slow_respired_fraction", "mol/mol"),
            ("slow_decomposition_rate", "1/day"),
        ],
        NAME,
    );
    for field in [
        "litter_respired_fraction",
        "active_stabilization_co2_fraction",
        "slow_respired_fraction",
    ] {
        checked(require_closed(v[field], 0.0, 1.0, field, NAME), NAME);
    }
    checked(
        require_non_negative(
            v["slow_decomposition_rate"],
            "slow_decomposition_rate",
            NAME,
        ),
        NAME,
    );
    HumificationParams {
        litter_respired_fraction: v["litter_respired_fraction"],
        active_stabilization_co2_fraction: v["active_stabilization_co2_fraction"],
        slow_respired_fraction: v["slow_respired_fraction"],
        slow_decomposition_rate: v["slow_decomposition_rate"],
    }
}

/// Condensation + recycling rates (`water_cycle.yaml`).
pub fn water_cycle() -> WaterCycleParams {
    const NAME: &str = "water_cycle.yaml";
    let units: [(&str, &str); 2] = [("condensation_rate", "1/day"), ("recycling_rate", "1/day")];
    let f = file(WATER_CYCLE_YAML, NAME);
    let v = guarded_map(&f, &units, NAME);
    for (field, _) in units {
        checked(require_non_negative(v[field], field, NAME), NAME);
    }
    WaterCycleParams {
        condensation_rate: v["condensation_rate"],
        recycling_rate: v["recycling_rate"],
    }
}

/// Minimal-consumer params (`herbivory.yaml`).
pub fn herbivory() -> HerbivoryParams {
    const NAME: &str = "herbivory.yaml";
    let units: [(&str, &str); 4] = [
        ("grazing_rate", "1/day"),
        ("respiration_rate", "1/day"),
        ("mortality_rate", "1/day"),
        ("o2_half_saturation", "mol/mol"),
    ];
    let f = file(HERBIVORY_YAML, NAME);
    let v = guarded_map(&f, &units, NAME);
    for (field, _) in units {
        checked(require_non_negative(v[field], field, NAME), NAME);
    }
    HerbivoryParams {
        grazing_rate: v["grazing_rate"],
        respiration_rate: v["respiration_rate"],
        mortality_rate: v["mortality_rate"],
        o2_half_saturation: v["o2_half_saturation"],
    }
}

/// The DVS-keyed partition table (`allocation.yaml`).
///
/// ⚠ Its `parameters` block is a **table**, not `{value, unit, source}` scalars — the one
/// biosphere file with a bespoke shape, and the reason [`config::ParamFile::raw`] exists.
/// The knots must strictly increase, every fraction must be in [0, 1], and each row must
/// sum to 1 within [`PARTITION_SUM_ATOL`]: ONE shared-breakpoint table, so sum-1 holds
/// everywhere by linearity rather than only at the knots.
pub fn allocation_from(text: &str, name: &'static str) -> AllocationParams {
    let f = file(text, name);
    let table = checked(f.raw("partition_table", name), name);
    let entries = checked(table.as_mapping(name), name);
    let rows_node = entries
        .iter()
        .find(|(k, _)| k == "rows")
        .map(|(_, v)| v)
        .unwrap_or_else(|| panic!("{name}: partition_table has no `rows`"));
    let items = checked(rows_node.as_sequence(name), name);
    assert!(
        items.len() >= 2,
        "{name}: partition table needs >= 2 rows, got {}",
        items.len()
    );

    let mut rows: Vec<PartitionRow> = Vec::with_capacity(items.len());
    for (i, item) in items.iter().enumerate() {
        let where_ = format!("{name}: row {i}");
        let map = checked(item.as_mapping(&where_), name);
        let read = |key: &str| -> f64 {
            let node = map
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v)
                .unwrap_or_else(|| panic!("{where_}: missing {key:?}"));
            match node {
                YamlValue::Scalar { text, .. } => text
                    .trim()
                    .parse()
                    .unwrap_or_else(|_| panic!("{where_}: {key} = {text:?} is not a number")),
                _ => panic!("{where_}: {key} must be a scalar"),
            }
        };
        rows.push(PartitionRow {
            dvs: read("dvs"),
            fl: read("fl"),
            fs: read("fs"),
            fr: read("fr"),
            fo: read("fo"),
        });
        assert_eq!(map.len(), 5, "{where_}: expected exactly dvs/fl/fs/fr/fo");
    }

    for (i, row) in rows.iter().enumerate() {
        if i > 0 {
            assert!(
                rows[i - 1].dvs < row.dvs,
                "{name}: partition dvs knots must strictly increase, \
                 got {} then {}",
                rows[i - 1].dvs,
                row.dvs
            );
        }
        for (label, frac) in [
            ("fl", row.fl),
            ("fs", row.fs),
            ("fr", row.fr),
            ("fo", row.fo),
        ] {
            assert!(
                (0.0..=1.0).contains(&frac),
                "{name}: row {i} {label} must be in [0, 1], got {frac}"
            );
        }
        let total = row.fl + row.fs + row.fr + row.fo;
        assert!(
            (total - 1.0).abs() <= PARTITION_SUM_ATOL,
            "{name}: row {i} fractions sum to {total}, not 1"
        );
    }
    AllocationParams { table: rows }
}

/// The frozen winter-wheat partition table.
pub fn allocation() -> AllocationParams {
    allocation_from(ALLOCATION_YAML, "allocation.yaml")
}

/// Load the frozen biosphere coefficients from the frozen param files.
pub fn biosphere() -> BiosphereParams {
    BiosphereParams {
        canopy: canopy(),
        photo: photosynthesis(),
        resp: respiration(),
        transp: transpiration(),
        pheno: phenology(),
        vern: vernalization(),
        photoperiod: photoperiod(),
        senesc: senescence(),
        stem_reserve: stem_reserves(),
        rootd: root_depth(),
        nitro: nitrogen(),
        decomp: decomposition(),
        micro: microbial_respiration(),
        humi: humification(),
        water: water_cycle(),
        herb: herbivory(),
        alloc: allocation(),
    }
}

/// The **frozen biosphere param-file census**: `(filename, embedded text)` for every YAML
/// file this module loads, in filename order (slice C8 of the reference flip).
///
/// # ⚠⚠ This is the set the reference LOADS, which is what re-anchors `param_files`
///
/// The recorded sha-256 values are author-neutral — both sides digest the same bytes — so
/// what actually moves to the reference is the **census rule**. Python's rule is a
/// non-recursive glob of a package directory minus `demo.yaml`; this is the list of files
/// the compiled reference actually reads. A file that stopped being loaded would drop out of
/// the manifest here, where under the glob rule it would stay in it.
///
/// # ⚠ The 15-of-20 rule, and why the five are excluded for TWO different reasons
///
/// `src/domains/biosphere/params/` holds **20** `*.yaml` files and the manifest names
/// **15**. The excluded five split:
///
/// * **four `crops/potato/*.yaml`** — excluded because the census is **non-recursive**. The
///   port has no potato build (its stage 2 is deferred), so it loads none of them.
/// * **`demo.yaml`** — excluded **by name**. It is a skeleton feeding two Python-only
///   scenarios that slice C6 retires.
///
/// *A directory is not a category*: a recursive walk picks the potato files up and the
/// census gate goes red looking like a port bug — **measured, not asserted**:
/// `tests::a_recursive_walk_reddens_the_census`, plus a control run that flipped this very
/// walk to descend and confirmed `the_census_matches_the_directory_on_disk` fires on its
/// roster assertion. ⚠ That control also showed the collision concretely: the recursive
/// listing contains `allocation.yaml`, `canopy.yaml`, `phenology.yaml` and `root_depth.yaml`
/// **twice**, so a basename-keyed manifest would not merely gain entries, it could overwrite
/// frozen ones.
///
/// # ⚠ Adding a param file is a THREE-place edit, deliberately
///
/// The list here, `dump_biosphere_inventory`'s `assert_eq!(files.len(), 15)`, and
/// `tests::the_census_matches_the_directory_on_disk`'s own count. This repo's standing
/// lesson is *a rule with two copies has one that is stale*, so the choice is stated rather
/// than left to be discovered: these are a **forcing function**, not a duplicated rule. The
/// list is the definition; the two literals exist so that adding a param file cannot happen
/// quietly — and the dump's fires **loudly during regeneration** rather than as a test
/// failure (measured: it panics with *"the frozen biosphere param census is 15 files, got
/// 14"*). The station side has the same shape with `8` / `5` / `3`.
pub fn param_files() -> Vec<(&'static str, &'static str)> {
    let mut files = vec![
        ("allocation.yaml", ALLOCATION_YAML),
        ("canopy.yaml", CANOPY_YAML),
        ("decomposition.yaml", DECOMPOSITION_YAML),
        ("herbivory.yaml", HERBIVORY_YAML),
        ("humification.yaml", HUMIFICATION_YAML),
        ("microbial_respiration.yaml", MICROBIAL_RESPIRATION_YAML),
        ("nitrogen.yaml", NITROGEN_YAML),
        ("phenology.yaml", PHENOLOGY_YAML),
        ("photosynthesis.yaml", PHOTOSYNTHESIS_YAML),
        ("respiration.yaml", RESPIRATION_YAML),
        ("root_depth.yaml", ROOT_DEPTH_YAML),
        ("senescence.yaml", SENESCENCE_YAML),
        ("stem_reserves.yaml", STEM_RESERVES_YAML),
        ("transpiration.yaml", TRANSPIRATION_YAML),
        ("water_cycle.yaml", WATER_CYCLE_YAML),
    ];
    files.sort_by_key(|(name, _)| *name);
    files
}

/// The directory the census is a census **of** — resolved at compile time, the same
/// reach-out `include_str!` above makes.
///
/// ⚠ Under target state C the param YAML cannot stay inside a deleted Python package, and
/// this constant makes the reach-out a **runtime** dependency as well as a compile-time one.
/// That sharpens the relocation trigger recorded in
/// `docs/plans/post-roadmap-reference-flip.md` §5d rather than resolving it.
pub const PARAMS_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../src/domains/biosphere/params"
);

/// The census's own exclusion: the skeleton file the frozen set leaves out **by name**.
pub const EXCLUDED_PARAM_FILE: &str = "demo.yaml";

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    /// The retained control (see the module header): the hex-float table the Python
    /// loaders produced, kept **only** so this file's load can be checked against it.
    const GENERATED_TABLE: &str = include_str!("biosphere_params.txt");

    fn generated() -> (BTreeMap<&'static str, f64>, Vec<PartitionRow>) {
        let mut scalars: BTreeMap<&'static str, f64> = BTreeMap::new();
        let mut rows: Vec<PartitionRow> = Vec::new();
        for line in GENERATED_TABLE.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut fields = line.split('\t');
            let head = fields.next().expect("a leading field");
            if head == "partition_row" {
                let mut next =
                    || simcore::hexfloat::parse(fields.next().expect("a field")).expect("parses");
                rows.push(PartitionRow {
                    dvs: next(),
                    fl: next(),
                    fs: next(),
                    fr: next(),
                    fo: next(),
                });
            } else {
                let hex = fields.next().expect("a hex-float");
                scalars.insert(head, simcore::hexfloat::parse(hex).expect("parses"));
            }
        }
        (scalars, rows)
    }

    /// The 66 scalars, paired with the generator's own names for them.
    fn loaded_pairs() -> Vec<(&'static str, f64)> {
        let p = biosphere();
        vec![
            ("canopy.sla_per_mol_c", p.canopy.sla_per_mol_c),
            ("canopy.extinction_coef", p.canopy.extinction_coef),
            ("photo.vcmax", p.photo.vcmax),
            ("photo.jmax", p.photo.jmax),
            ("photo.quantum_yield", p.photo.quantum_yield),
            ("photo.theta", p.photo.theta),
            ("photo.gamma_star", p.photo.gamma_star),
            ("photo.kc", p.photo.kc),
            ("photo.ko", p.photo.ko),
            ("photo.o2", p.photo.o2),
            ("photo.t_min", p.photo.t_min),
            ("photo.t_opt_lo", p.photo.t_opt_lo),
            ("photo.t_opt_hi", p.photo.t_opt_hi),
            ("photo.t_max", p.photo.t_max),
            ("resp.maintenance_coef", p.resp.maintenance_coef),
            ("resp.q10", p.resp.q10),
            ("resp.t_ref", p.resp.t_ref),
            ("resp.growth_efficiency", p.resp.growth_efficiency),
            ("resp.o2_half_saturation", p.resp.o2_half_saturation),
            (
                "transp.aerodynamic_resistance",
                p.transp.aerodynamic_resistance,
            ),
            ("transp.surface_resistance", p.transp.surface_resistance),
            ("pheno.t_base", p.pheno.t_base),
            ("pheno.t_cap", p.pheno.t_cap),
            ("pheno.tsum_anthesis", p.pheno.tsum_anthesis),
            ("pheno.tsum_maturity", p.pheno.tsum_maturity),
            ("vern.t_base_v", p.vern.t_base_v),
            ("vern.t_opt_lower_v", p.vern.t_opt_lower_v),
            ("vern.t_opt_upper_v", p.vern.t_opt_upper_v),
            ("vern.t_ceiling_v", p.vern.t_ceiling_v),
            ("vern.vsen", p.vern.vsen),
            ("vern.vdsat", p.vern.vdsat),
            ("photo.cpp", p.photoperiod.cpp),
            ("photo.ppsen", p.photoperiod.ppsen),
            ("rootd.max_extension_rate", p.rootd.max_extension_rate),
            ("rootd.max_rooted_depth", p.rootd.max_rooted_depth),
            (
                "stemres.remobilizable_fraction",
                p.stem_reserve.remobilizable_fraction,
            ),
            (
                "stemres.remobilization_rate",
                p.stem_reserve.remobilization_rate,
            ),
            ("stemres.trigger_dvs", p.stem_reserve.trigger_dvs),
            ("stemres.cessation_dvs", p.stem_reserve.cessation_dvs),
            ("senesc.rdr_leaf", p.senesc.rdr_leaf),
            ("senesc.rdr_stem", p.senesc.rdr_stem),
            ("senesc.rdr_root", p.senesc.rdr_root),
            ("senesc.shade_rate", p.senesc.shade_rate),
            ("senesc.lai_threshold", p.senesc.lai_threshold),
            ("nitro.max_uptake_capacity", p.nitro.max_uptake_capacity),
            ("nitro.n_residual_per_mol_c", p.nitro.n_residual_per_mol_c),
            ("nitro.n_critical_per_mol_c", p.nitro.n_critical_per_mol_c),
            ("nitro.n_target_coefficient", p.nitro.n_target_coefficient),
            ("nitro.n_target_exponent", p.nitro.n_target_exponent),
            ("nitro.n_target_w_plateau", p.nitro.n_target_w_plateau),
            ("nitro.dm_kg_per_mol_c", p.nitro.dm_kg_per_mol_c),
            ("decomp.decomposition_rate", p.decomp.decomposition_rate),
            (
                "micro.microbial_respiration_rate",
                p.micro.microbial_respiration_rate,
            ),
            ("micro.o2_half_saturation", p.micro.o2_half_saturation),
            (
                "humi.litter_respired_fraction",
                p.humi.litter_respired_fraction,
            ),
            (
                "humi.active_stabilization_co2_fraction",
                p.humi.active_stabilization_co2_fraction,
            ),
            ("humi.slow_respired_fraction", p.humi.slow_respired_fraction),
            (
                "humi.slow_decomposition_rate",
                p.humi.slow_decomposition_rate,
            ),
            ("water.condensation_rate", p.water.condensation_rate),
            ("water.recycling_rate", p.water.recycling_rate),
            ("herb.grazing_rate", p.herb.grazing_rate),
            ("herb.respiration_rate", p.herb.respiration_rate),
            ("herb.mortality_rate", p.herb.mortality_rate),
            ("herb.o2_half_saturation", p.herb.o2_half_saturation),
        ]
    }

    /// ⚠⚠ **The slice's gate on the biosphere side, folds included.** Every value this
    /// module now derives from the YAML is bit-identical to what the Python loaders
    /// produced — not "within a band", the same `f64`. A single moved bit means C1
    /// stopped being a re-anchoring and became an unfreeze with 18 goldens behind it.
    #[test]
    fn every_value_matches_the_generated_table() {
        let (want, want_rows) = generated();
        let got = loaded_pairs();
        for (name, loaded) in &got {
            let expected = want
                .get(name)
                .unwrap_or_else(|| panic!("{name} is not in the control table"));
            assert_eq!(
                loaded.to_bits(),
                expected.to_bits(),
                "{name}: loaded {loaded:?} != generated {expected:?}"
            );
        }
        // ⚠ Both directions: a scalar the control names and this module never loads
        // would otherwise pass unnoticed — the completeness half of the gate.
        let loaded_names: std::collections::BTreeSet<&str> = got.iter().map(|(n, _)| *n).collect();
        let control_names: std::collections::BTreeSet<&str> = want.keys().copied().collect();
        assert_eq!(loaded_names, control_names, "the two name sets must match");

        let rows = allocation().table;
        assert_eq!(rows.len(), want_rows.len(), "same number of partition rows");
        for (i, (got, want)) in rows.iter().zip(&want_rows).enumerate() {
            for (label, a, b) in [
                ("dvs", got.dvs, want.dvs),
                ("fl", got.fl, want.fl),
                ("fs", got.fs, want.fs),
                ("fr", got.fr, want.fr),
                ("fo", got.fo, want.fo),
            ] {
                assert_eq!(a.to_bits(), b.to_bits(), "partition row {i} {label}");
            }
        }
    }

    /// ⚠ The MUST-EQUAL constraint between the two files that fold with the carbon
    /// fraction. Python documents it and enforces it nowhere; a divergence models a
    /// silently inconsistent plant (leaf area per mol C and N per mol C disagreeing
    /// about what a mol of carbon weighs), so it is asserted here rather than trusted.
    #[test]
    fn the_two_carbon_fractions_agree() {
        let canopy_cf = MOLAR_MASS_CARBON_KG_PER_MOL
            / (canopy().sla_per_mol_c
                / file(CANOPY_YAML, "canopy.yaml")
                    .guarded("specific_leaf_area", "m^2/kg", "canopy.yaml")
                    .unwrap());
        let nitro_cf = MOLAR_MASS_CARBON_KG_PER_MOL / nitrogen().dm_kg_per_mol_c;
        assert!(
            (canopy_cf - nitro_cf).abs() < 1e-12,
            "canopy.yaml and nitrogen.yaml disagree about carbon_fraction: \
             {canopy_cf} vs {nitro_cf}"
        );
    }

    /// ⚠ The reformat this slice forced: `allocation.yaml` is now block style, and the
    /// reader **rejects** the flow style it used to be written in. Pinned so a future
    /// edit back to `- {dvs: …}` fails loudly here rather than at a build a week later.
    #[test]
    fn the_partition_table_is_block_style_because_flow_style_is_rejected() {
        assert!(
            !ALLOCATION_YAML.contains("- {"),
            "allocation.yaml must stay in the reader's closed subset (no flow style)"
        );
        let flow = "\
name: winter_wheat
process: allocation
parameters:
  partition_table:
    source: \"x\"
    rows:
      - {dvs: 0.0, fl: 1.0, fs: 0.0, fr: 0.0, fo: 0.0}
";
        assert!(
            ParamFile::parse(flow, "allocation.yaml").is_err(),
            "flow style must be rejected, not silently mis-parsed"
        );
    }

    /// ⚠ A row that does not sum to 1 must be rejected. Without this the sum-1 invariant
    /// is a comment: nothing else in the load would notice.
    #[test]
    fn a_partition_row_that_does_not_sum_to_one_is_rejected() {
        let broken = ALLOCATION_YAML.replacen("fl: 0.55", "fl: 0.56", 1);
        assert_ne!(broken, ALLOCATION_YAML, "the substitution must apply");
        let caught = std::panic::catch_unwind(|| {
            allocation_from(Box::leak(broken.into_boxed_str()), "allocation.yaml")
        });
        assert!(caught.is_err(), "a row summing to 1.01 must be rejected");
    }

    #[test]
    fn loads_all_groups_and_the_partition_table() {
        let p = biosphere();
        assert!(p.canopy.sla_per_mol_c > 0.0);
        assert!(p.photo.t_min < p.photo.t_opt_lo);
        assert_eq!(p.alloc.table.len(), 3);
        // Each partition row sums to 1 (the loader-enforced invariant, round-trip check).
        for row in &p.alloc.table {
            let total = row.fl + row.fs + row.fr + row.fo;
            assert!((total - 1.0).abs() < 1e-9, "row sums to {total}");
        }
    }

    /// The census equals the directory, under the **non-recursive minus `demo.yaml`** rule.
    ///
    /// ⚠⚠ **The completeness half of `param_files`, and the direction that matters is the
    /// one a value comparison cannot see.** The digests are author-neutral (both sides hash
    /// the same file), so the only thing that can go wrong with this key is the *roster*: a
    /// param file added to the tree and wired into no loader, or a loader dropped while its
    /// file stays. The first is exactly what `param_files` under the glob rule would have
    /// hidden — the manifest would have gained a hash for a file the reference never reads.
    #[test]
    fn the_census_matches_the_directory_on_disk() {
        let mut on_disk: Vec<String> = std::fs::read_dir(PARAMS_DIR)
            .expect("the frozen params directory is readable")
            .map(|entry| entry.expect("a readable dir entry"))
            // NON-RECURSIVE: `read_dir` does not descend, which is what leaves the four
            // `crops/potato/*.yaml` out. The `crops/` entry itself is dropped by
            // `is_file()` (and again by the `.yaml` suffix filter) — stated because the
            // exclusion is a property of these two filters, not of the directory's name.
            .filter(|entry| entry.path().is_file())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.ends_with(".yaml") && name != EXCLUDED_PARAM_FILE)
            .collect();
        on_disk.sort();

        let census: Vec<String> = param_files()
            .iter()
            .map(|(name, _)| (*name).to_string())
            .collect();
        assert_eq!(
            census, on_disk,
            "the loaded param-file census and the directory disagree. This is a ROSTER \
             finding, not a value one: either a param file was added and wired into no \
             loader, or a loader was dropped and its file left behind. Do NOT 'fix' it by \
             editing whichever list is shorter."
        );
        assert_eq!(
            census.len(),
            15,
            "the frozen biosphere param set is 15 files"
        );
    }

    /// A recursive walk DOES pick the potato overrides up, and the census would redden.
    ///
    /// ⚠⚠ This exists because the claim was **prose in three places** — the manifest's
    /// `_authority`, `docs/biosphere-reference.md` and the plan — before it was measured
    /// anywhere. It reproduces the *mistake*, not the fix: a recursive walk is what a
    /// reasonable person writes, and it silently adds four names to a **frozen contract**.
    #[test]
    fn a_recursive_walk_reddens_the_census() {
        // The walk the census must NOT be: descend one level into `crops/<crop>/`.
        let mut recursive: Vec<String> = Vec::new();
        for entry in std::fs::read_dir(PARAMS_DIR).expect("params dir") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                for crop in std::fs::read_dir(&path).expect("crops dir") {
                    for f in std::fs::read_dir(crop.expect("dir entry").path()).expect("crop") {
                        let name = f
                            .expect("dir entry")
                            .file_name()
                            .to_string_lossy()
                            .into_owned();
                        if name.ends_with(".yaml") {
                            recursive.push(name);
                        }
                    }
                }
            } else {
                let name = path
                    .file_name()
                    .expect("a file name")
                    .to_string_lossy()
                    .into_owned();
                if name.ends_with(".yaml") && name != EXCLUDED_PARAM_FILE {
                    recursive.push(name);
                }
            }
        }
        recursive.sort();
        let census: Vec<String> = param_files()
            .iter()
            .map(|(name, _)| (*name).to_string())
            .collect();
        assert_ne!(
            recursive, census,
            "a recursive walk no longer differs from the census — the hazard this test \
             documents has gone away, and three prose claims about it are now stale"
        );
        assert_eq!(
            recursive.len(),
            census.len() + 4,
            "expected exactly four extra names from crops/potato, got {recursive:?}"
        );
        // ⚠ And the sharp half: the extras are not new names, they are DUPLICATES of frozen
        // ones, so a basename-keyed manifest could overwrite a frozen hash rather than grow.
        let mut deduped = recursive.clone();
        deduped.dedup();
        assert!(
            deduped.len() < recursive.len(),
            "the potato overrides no longer collide with frozen basenames"
        );
    }

    /// The four potato overrides exist, and the census does **not** see them.
    ///
    /// ⚠ A free negative control, because the subject is already on disk: swap `read_dir`
    /// above for a recursive walk and the census gains four names, the manifest gains four
    /// hashes, and the failure reads like a port bug. *A directory is not a category.*
    #[test]
    fn the_recursive_walk_would_see_four_more_and_the_census_does_not() {
        let nested = std::path::Path::new(PARAMS_DIR).join("crops/potato");
        let mut overrides: Vec<String> = std::fs::read_dir(&nested)
            .expect("the potato override directory is readable")
            .map(|entry| entry.expect("a readable dir entry"))
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.ends_with(".yaml"))
            .collect();
        overrides.sort();
        assert_eq!(
            overrides.len(),
            4,
            "expected four potato overrides, found {overrides:?} — if this changed, the \
             15-of-20 rule in `param_files`'s doc comment is stale"
        );
        // Two of the four share a BASENAME with a frozen file (`allocation.yaml`,
        // `canopy.yaml`), so a recursive walk would not merely add names — it would collide
        // with the frozen ones and could overwrite a hash in place.
        let census: Vec<&str> = param_files().iter().map(|(n, _)| *n).collect();
        let colliding: Vec<&String> = overrides
            .iter()
            .filter(|n| census.contains(&n.as_str()))
            .collect();
        assert!(
            !colliding.is_empty(),
            "the basename collision this test documents has gone away; the recursive-walk \
             hazard is now milder than the comment claims"
        );
    }

    /// No frozen param file contains a separator Python's `splitlines` would break on.
    ///
    /// ⚠ This is what makes the reference's narrow newline rule and Python's broader one
    /// unable to disagree, rather than merely observed not to. Without it the two hash rules
    /// differ on a file nobody has written yet.
    #[test]
    fn no_frozen_param_file_carries_an_exotic_line_separator() {
        for (name, text) in param_files() {
            assert_eq!(
                config::provenance::contains_exotic_line_separator(text),
                None,
                "{name} contains a character Python's splitlines treats as a line break but \
                 the reference's normalization does not — the two hash rules would diverge"
            );
        }
    }
}
