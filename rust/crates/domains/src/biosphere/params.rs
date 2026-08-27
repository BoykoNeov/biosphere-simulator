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

const CANOPY_YAML: &str = include_str!("../../params/biosphere/canopy.yaml");
const PHOTOSYNTHESIS_YAML: &str = include_str!("../../params/biosphere/photosynthesis.yaml");
const RESPIRATION_YAML: &str = include_str!("../../params/biosphere/respiration.yaml");
const TRANSPIRATION_YAML: &str = include_str!("../../params/biosphere/transpiration.yaml");
const PHENOLOGY_YAML: &str = include_str!("../../params/biosphere/phenology.yaml");
const SENESCENCE_YAML: &str = include_str!("../../params/biosphere/senescence.yaml");
const ROOT_DEPTH_YAML: &str = include_str!("../../params/biosphere/root_depth.yaml");
const STEM_RESERVES_YAML: &str = include_str!("../../params/biosphere/stem_reserves.yaml");
const NITROGEN_YAML: &str = include_str!("../../params/biosphere/nitrogen.yaml");
const DECOMPOSITION_YAML: &str = include_str!("../../params/biosphere/decomposition.yaml");
const MICROBIAL_RESPIRATION_YAML: &str =
    include_str!("../../params/biosphere/microbial_respiration.yaml");
const HUMIFICATION_YAML: &str = include_str!("../../params/biosphere/humification.yaml");
const WATER_CYCLE_YAML: &str = include_str!("../../params/biosphere/water_cycle.yaml");
const HERBIVORY_YAML: &str = include_str!("../../params/biosphere/herbivory.yaml");
const ALLOCATION_YAML: &str = include_str!("../../params/biosphere/allocation.yaml");

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
    respiration_from(RESPIRATION_YAML, "respiration.yaml")
}

/// The same reader over an arbitrary text, so the guards above can be exercised.
///
/// ⚠ A PRODUCTION CHANGE INSIDE A TESTING BATCH, and it is the same one batches B and C
/// each made rather than a fresh decision: every other guarded loader in this module was
/// already split into a text-taking core plus a thin wrapper, because a loader that can
/// only ever read its own `include_str!` has guards no test can reach. This splits the
/// last two (`respiration`, `stem_reserves`). It is NOT the extraction §5ad rules out —
/// that one is about lifting equations out of `Flow::demand` to make them unit-testable,
/// which changes what the science is made of. This changes nothing but who may call it,
/// and the committed values are asserted unchanged by the params census.
pub fn respiration_from(text: &str, name: &'static str) -> RespirationParams {
    let f = file(text, name);
    let v = guarded_map(
        &f,
        &[
            ("maintenance_coef", "1/day"),
            ("q10", "dimensionless"),
            ("t_ref", "degC"),
            ("growth_efficiency", "dimensionless"),
            ("o2_half_saturation", "mol/mol"),
        ],
        name,
    );
    for field in ["maintenance_coef", "q10"] {
        checked(require_positive(v[field], field, name), name);
    }
    checked(
        require_half_open(v["growth_efficiency"], 0.0, 1.0, "growth_efficiency", name),
        name,
    );
    checked(
        require_non_negative(v["o2_half_saturation"], "o2_half_saturation", name),
        name,
    );
    RespirationParams {
        maintenance_coef: v["maintenance_coef"],
        q10: v["q10"],
        t_ref: v["t_ref"],
        growth_efficiency: v["growth_efficiency"],
        o2_half_saturation: v["o2_half_saturation"],
    }
}

/// Penman–Monteith transpiration resistances, from arbitrary file TEXT.
///
/// ⚠ The `_from` split is the `allocation_from` / `phenology_from` shape and exists for
/// one measured reason: the positivity guards below read `TRANSPIRATION_YAML` through
/// `include_str!`, so the only file they could ever see was the committed one — which is
/// valid. Measured, not assumed: before this split, deleting the whole guard loop left
/// `cargo test -p domains --lib` at 221 passed / 0 failed, against a live control
/// (declaring `aerodynamic_resistance` in `min/m`) that reddened 25. A guard that cannot
/// be handed a bad file is a comment. Nothing about the committed load changed.
///
/// Both resistances are DIVISORS in the combination equation — a zero `r_a` is an
/// infinity in the aerodynamic term, not a slow crop.
pub fn transpiration_from(text: &str, name: &'static str) -> TranspirationParams {
    let f = file(text, name);
    let v = guarded_map(
        &f,
        &[
            ("aerodynamic_resistance", "s/m"),
            ("surface_resistance", "s/m"),
        ],
        name,
    );
    for field in ["aerodynamic_resistance", "surface_resistance"] {
        checked(require_positive(v[field], field, name), name);
    }
    TranspirationParams {
        aerodynamic_resistance: v["aerodynamic_resistance"],
        surface_resistance: v["surface_resistance"],
    }
}

/// Penman–Monteith transpiration resistances (`transpiration.yaml`).
pub fn transpiration() -> TranspirationParams {
    transpiration_from(TRANSPIRATION_YAML, "transpiration.yaml")
}

/// The whole 12-field `phenology.yaml` block, validated once for all three readers.
///
/// ⚠ The `_from` split below is the `allocation_from` shape, and it exists for one
/// reason: the three SEMANTIC guards (the cardinal band, the vernalization ordering, the
/// positive sums) read `PHENOLOGY_YAML` through `include_str!`, so the only file they could
/// ever see was the committed one — which is valid. Measured, not assumed: before this
/// split, deleting any of the three left `cargo test -p domains --lib` at 216 passed / 0
/// failed, while the live control (declaring `t_base` in kelvin) reddened 29. A guard that
/// cannot be handed a bad file is a comment. Nothing about the committed load changed —
/// each public reader is its own `_from` at `PHENOLOGY_YAML`.
fn phenology_block_from(text: &str, name: &'static str) -> std::collections::BTreeMap<String, f64> {
    let f = file(text, name);
    guarded_map(&f, &PHENOLOGY_UNITS, name)
}

/// Thermal-time phenology params from arbitrary file text — the testable half of
/// [`phenology`].
pub fn phenology_from(text: &str, name: &'static str) -> PhenologyParams {
    let v = phenology_block_from(text, name);
    assert!(
        v["t_base"] < v["t_cap"],
        "{name}: cardinal temperatures must satisfy t_base < t_cap, got ({}, {})",
        v["t_base"],
        v["t_cap"]
    );
    for field in ["tsum_anthesis", "tsum_maturity"] {
        checked(require_positive(v[field], field, name), name);
    }
    PhenologyParams {
        t_base: v["t_base"],
        t_cap: v["t_cap"],
        tsum_anthesis: v["tsum_anthesis"],
        tsum_maturity: v["tsum_maturity"],
    }
}

/// Thermal-time phenology params (`phenology.yaml`).
pub fn phenology() -> PhenologyParams {
    phenology_from(PHENOLOGY_YAML, "phenology.yaml")
}

/// Vernalization cardinals from arbitrary file text — the testable half of
/// [`vernalization`].
pub fn vernalization_from(text: &str, name: &'static str) -> VernalizationParams {
    let v = phenology_block_from(text, name);
    // A well-ordered response with a strictly positive ramp on each side; the two strict
    // pairs are divisors.
    assert!(
        v["t_base_v"] < v["t_opt_lower_v"]
            && v["t_opt_lower_v"] <= v["t_opt_upper_v"]
            && v["t_opt_upper_v"] < v["t_ceiling_v"],
        "{name}: vernalization cardinals must satisfy \
         t_base_v < t_opt_lower_v <= t_opt_upper_v < t_ceiling_v"
    );
    checked(require_positive(v["vdsat"], "vdsat", name), name);
    // A negative sensitivity would make cold *retard* development.
    checked(require_non_negative(v["vsen"], "vsen", name), name);
    VernalizationParams {
        t_base_v: v["t_base_v"],
        t_opt_lower_v: v["t_opt_lower_v"],
        t_opt_upper_v: v["t_opt_upper_v"],
        t_ceiling_v: v["t_ceiling_v"],
        vsen: v["vsen"],
        vdsat: v["vdsat"],
    }
}

/// Vernalization cardinals (`phenology.yaml`) — Soltani & Sinclair Eqn 8.3 / 8.6.
pub fn vernalization() -> VernalizationParams {
    vernalization_from(PHENOLOGY_YAML, "phenology.yaml")
}

/// Photoperiod params from arbitrary file text — the testable half of [`photoperiod`].
pub fn photoperiod_from(text: &str, name: &'static str) -> PhotoperiodParams {
    let v = phenology_block_from(text, name);
    checked(require_positive(v["cpp"], "cpp", name), name);
    checked(require_non_negative(v["ppsen"], "ppsen", name), name);
    PhotoperiodParams {
        cpp: v["cpp"],
        ppsen: v["ppsen"],
    }
}

/// Photoperiod (daylength) params (`phenology.yaml`) — long-day form.
pub fn photoperiod() -> PhotoperiodParams {
    photoperiod_from(PHENOLOGY_YAML, "phenology.yaml")
}

/// Relative organ death rates + the mutual-shading term (`senescence.yaml`).
pub fn senescence() -> SenescenceParams {
    senescence_from(SENESCENCE_YAML, "senescence.yaml")
}

/// [`senescence`] from arbitrary file TEXT — see [`transpiration_from`] for why the split
/// exists.
///
/// ⚠ **This split is S5 batch G's one production change, and the reason is a
/// measurement rather than a preference.** Before it, the `require_non_negative` loop
/// below could only ever be handed the committed file, which is valid — so the guard was
/// INERT: deleting it outright left `cargo test -p domains --lib` at **298 passed / 0
/// failed**, exactly as batch B measured for the three phenology guards. A negative
/// relative death rate is not a slow organ, it is an organ that GROWS out of the litter
/// sink at a fixed relative rate, and `Senescence`'s legs would carry the wrong sign with
/// conservation still satisfied. Taken as an explicit decision, not slipped in: the
/// alternative was four Python loader tests dying at S6 with no successor.
pub fn senescence_from(text: &str, name: &'static str) -> SenescenceParams {
    let units: [(&str, &str); 5] = [
        ("rdr_leaf", "1/day"),
        ("rdr_stem", "1/day"),
        ("rdr_root", "1/day"),
        ("shade_rate", "1/day"),
        ("lai_threshold", "dimensionless"),
    ];
    let f = file(text, name);
    let v = guarded_map(&f, &units, name);
    for (field, _) in units {
        checked(require_non_negative(v[field], field, name), name);
    }
    SenescenceParams {
        rdr_leaf: v["rdr_leaf"],
        rdr_stem: v["rdr_stem"],
        rdr_root: v["rdr_root"],
        shade_rate: v["shade_rate"],
        lai_threshold: v["lai_threshold"],
    }
}

/// Root-extension params, from arbitrary file TEXT — see [`transpiration_from`] for why
/// the split exists.
///
/// Both bounds disable the mechanism SILENTLY and no golden would notice: a zero rate
/// freezes rooted depth at sowing so the root-zone access gate is shut forever, and a
/// zero maximum depth divides by a crop that cannot root. Measured inert before the
/// split, exactly as the transpiration pair was.
pub fn root_depth_from(text: &str, name: &'static str) -> RootDepthParams {
    let units: [(&str, &str); 2] = [("max_extension_rate", "m/day"), ("max_rooted_depth", "m")];
    let f = file(text, name);
    let v = guarded_map(&f, &units, name);
    for (field, _) in units {
        checked(require_positive(v[field], field, name), name);
    }
    RootDepthParams {
        max_extension_rate: v["max_extension_rate"],
        max_rooted_depth: v["max_rooted_depth"],
    }
}

/// Root-extension params (`root_depth.yaml`).
pub fn root_depth() -> RootDepthParams {
    root_depth_from(ROOT_DEPTH_YAML, "root_depth.yaml")
}

/// Stem-reserve remobilization (`stem_reserves.yaml`).
pub fn stem_reserves() -> StemReserveParams {
    stem_reserves_from(STEM_RESERVES_YAML, "stem_reserves.yaml")
}

/// The same reader over an arbitrary text (see `respiration_from` for why).
pub fn stem_reserves_from(text: &str, name: &'static str) -> StemReserveParams {
    let f = file(text, name);
    let v = guarded_map(
        &f,
        &[
            ("remobilizable_fraction", "dimensionless"),
            ("remobilization_rate", "1/day"),
            ("trigger_dvs", "dimensionless"),
            ("cessation_dvs", "dimensionless"),
        ],
        name,
    );
    let (fstr, rate) = (v["remobilizable_fraction"], v["remobilization_rate"]);
    let (trigger, cessation) = (v["trigger_dvs"], v["cessation_dvs"]);
    assert!(
        0.0 < fstr && fstr < 1.0,
        "{name}: remobilizable_fraction must be in (0, 1), got {fstr}"
    );
    checked(
        require_half_open(rate, 0.0, 1.0, "remobilization_rate", name),
        name,
    );
    checked(require_closed(trigger, 0.0, 2.0, "trigger_dvs", name), name);
    // ⚠ `cessation_dvs` closes BOTH halves at maturity — a DOMAIN boundary, not a cited
    // cessation rule (see the struct's own doc comment).
    assert!(
        trigger < cessation && cessation <= 2.0,
        "{name}: must satisfy trigger_dvs < cessation_dvs <= 2, got ({trigger}, {cessation})"
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
    nitrogen_from(NITROGEN_YAML, "nitrogen.yaml")
}

/// The same reader over an arbitrary text, so the guards below can be exercised.
///
/// ⚠ A PRODUCTION CHANGE INSIDE A TESTING BATCH, and the FOURTH instance of the one
/// batches B, C and D each made rather than a fresh decision: a loader wired to its own
/// `include_str!` has guards no test can reach. `nitrogen.yaml` carries more of them than
/// any other biosphere file — a unit, a positivity bound, a `[0, 1]` fraction, an ordered
/// concentration band, a positive domain bound, and the ordering rule between the target
/// coefficient and `n_critical` — and before this split every one of them was unreachable.
/// It is NOT the extraction §5ad rules out: nothing about what the science is made of
/// moves, and the committed values stay pinned by C8's params census.
pub fn nitrogen_from(text: &str, name: &'static str) -> NitrogenParams {
    let f = file(text, name);
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
        name,
    );
    checked(
        require_positive(v["max_uptake_capacity"], "max_uptake_capacity", name),
        name,
    );
    // ⚠ MUST EQUAL canopy.yaml's carbon_fraction — a divergence models a silently
    // inconsistent plant. Both files fold with it; the dedup is a long-standing nicety.
    let cf = carbon_fraction(v["carbon_fraction"], name);
    let (n_residual, n_critical) = (v["n_residual"], v["n_critical"]);
    checked(require_non_negative(n_residual, "n_residual", name), name);
    assert!(
        n_residual < n_critical,
        "{name}: N-concentration thresholds must satisfy n_residual < n_critical, \
         got ({n_residual}, {n_critical})"
    );
    // ⚠ divide first, then multiply — the Python loader's order. See the header.
    let fold = MOLAR_MASS_CARBON_KG_PER_MOL / cf;
    checked(
        require_positive(v["n_target_w_plateau"], "n_target_w_plateau", name),
        name,
    );
    // The target must sit ABOVE the stress threshold, or the plant is stressed by
    // construction at every crop mass (Greenwood's curve declines, so the plateau is its
    // maximum; if even that is below critical, f_N < 1 always).
    assert!(
        v["n_target_coefficient"] > n_critical,
        "{name}: n_target_coefficient must exceed n_critical, got ({}, {n_critical})",
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

/// First-order litter decay, from arbitrary file TEXT — see [`transpiration_from`] for
/// why the split exists.
///
/// ⚠ NON-NEGATIVE, not positive: the file's own header states *"A zero rate is valid (no
/// decomposition)"*, so a `require_positive` here would reject a legal file. A NEGATIVE
/// rate would run the decomposer chain backwards — microbial biomass and CO2 flowing back
/// into standing litter through a flow whose legs say the opposite — and the flow-level
/// direction pins could not catch it, because the legs would still balance.
pub fn decomposition_from(text: &str, name: &'static str) -> DecompositionParams {
    let f = file(text, name);
    let v = guarded_map(&f, &[("decomposition_rate", "1/day")], name);
    DecompositionParams {
        decomposition_rate: checked(
            require_non_negative(v["decomposition_rate"], "decomposition_rate", name),
            name,
        ),
    }
}

/// First-order litter decay (`decomposition.yaml`). Zero is valid (no decomposition).
pub fn decomposition() -> DecompositionParams {
    decomposition_from(DECOMPOSITION_YAML, "decomposition.yaml")
}

/// First-order microbial respiration, from arbitrary file TEXT — see
/// [`transpiration_from`] for why the split exists.
pub fn microbial_respiration_from(text: &str, name: &'static str) -> MicrobialRespirationParams {
    let units: [(&str, &str); 2] = [
        ("microbial_respiration_rate", "1/day"),
        ("o2_half_saturation", "mol/mol"),
    ];
    let f = file(text, name);
    let v = guarded_map(&f, &units, name);
    for (field, _) in units {
        checked(require_non_negative(v[field], field, name), name);
    }
    MicrobialRespirationParams {
        microbial_respiration_rate: v["microbial_respiration_rate"],
        o2_half_saturation: v["o2_half_saturation"],
    }
}

/// First-order microbial respiration (`microbial_respiration.yaml`).
pub fn microbial_respiration() -> MicrobialRespirationParams {
    microbial_respiration_from(MICROBIAL_RESPIRATION_YAML, "microbial_respiration.yaml")
}

/// The humification split, from arbitrary file TEXT — see [`transpiration_from`] for why
/// the split exists.
///
/// ⚠ The three CO2 shares are guarded on the CLOSED unit interval and the slow rate is
/// guarded non-negative, which is two different rules in one file. A share outside [0, 1]
/// is not a hot partition: it would send MORE carbon to CO2 than the flow withdrew, and the
/// complement — computed by subtraction — would come out negative, i.e. a destination leg
/// that withdraws from its own receiver while the flow still balances.
pub fn humification_from(text: &str, name: &'static str) -> HumificationParams {
    let f = file(text, name);
    let v = guarded_map(
        &f,
        &[
            ("litter_respired_fraction", "mol/mol"),
            ("active_stabilization_co2_fraction", "mol/mol"),
            ("slow_respired_fraction", "mol/mol"),
            ("slow_decomposition_rate", "1/day"),
        ],
        name,
    );
    for field in [
        "litter_respired_fraction",
        "active_stabilization_co2_fraction",
        "slow_respired_fraction",
    ] {
        checked(require_closed(v[field], 0.0, 1.0, field, name), name);
    }
    checked(
        require_non_negative(
            v["slow_decomposition_rate"],
            "slow_decomposition_rate",
            name,
        ),
        name,
    );
    HumificationParams {
        litter_respired_fraction: v["litter_respired_fraction"],
        active_stabilization_co2_fraction: v["active_stabilization_co2_fraction"],
        slow_respired_fraction: v["slow_respired_fraction"],
        slow_decomposition_rate: v["slow_decomposition_rate"],
    }
}

/// The humification split, a carbon-use efficiency (`humification.yaml`).
pub fn humification() -> HumificationParams {
    humification_from(HUMIFICATION_YAML, "humification.yaml")
}

/// Condensation + recycling rates, from arbitrary file TEXT — see [`transpiration_from`]
/// for why the split exists.
///
/// ⚠ NON-NEGATIVE, not positive, and the difference is an inherited design decision rather
/// than a looser guard. The file's own header states it: *"A zero rate is valid (no
/// condensation / no recycling); negative is rejected (it would create water)."* A negative
/// rate would run the ring backwards — condensate evaporating into vapour through a flow
/// whose legs say the opposite — which the flow-level direction pins could not catch,
/// because the legs would still balance.
///
/// ⚠ **A first draft of this comment justified the zero by saying it is how every
/// open-field scenario declares no condenser. That is FALSE and it was measured, not
/// argued**: the ring is built only inside the `sealed` branch, so an open-field scenario
/// omits the two flows entirely rather than declaring zero rates, and nothing in the tree
/// declares a zero (the shipped file is 0.5/0.5). The guard's shape is the FILE's rule, not
/// a claim about the roster — batch A's overclaim shape, caught in review.
pub fn water_cycle_from(text: &str, name: &'static str) -> WaterCycleParams {
    let units: [(&str, &str); 2] = [("condensation_rate", "1/day"), ("recycling_rate", "1/day")];
    let f = file(text, name);
    let v = guarded_map(&f, &units, name);
    for (field, _) in units {
        checked(require_non_negative(v[field], field, name), name);
    }
    WaterCycleParams {
        condensation_rate: v["condensation_rate"],
        recycling_rate: v["recycling_rate"],
    }
}

/// Condensation + recycling rates (`water_cycle.yaml`).
pub fn water_cycle() -> WaterCycleParams {
    water_cycle_from(WATER_CYCLE_YAML, "water_cycle.yaml")
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
/// `crates/domains/params/biosphere/` holds **20** `*.yaml` files and the manifest names
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

/// The directory the census is a census **of** — resolved at compile time against this
/// crate's own root, the same as the `include_str!`s above.
///
/// ⚠ Until Stage-3 slice S1 this pointed into `src/domains/biosphere/params/`, a Python
/// package scheduled for deletion, which made the census a **runtime** dependency on the
/// dying tree as well as a compile-time one — the sharper half of the same problem. S1 moved
/// the whole directory here, `crops/potato/` and `demo.yaml` included, so the census rule and
/// its two different exclusions carry over verbatim.
pub const PARAMS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/params/biosphere");

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

    // --- S5 batch B: the phenology loader ---------------------------------------
    //
    // Ported from `tests/test_phenology.py`'s config-boundary block. The three schema
    // guards below are reachable because they live in the READER (`ParamFile::parse` and
    // `guarded_set`), which takes text.
    //
    // ⚠ **The three SEMANTIC guards are not reachable, and that was measured, not
    // assumed.** `phenology()`'s `t_base < t_cap` assertion, `vernalization()`'s
    // cardinal-ordering assertion and the `tsum` positivity check all read
    // `PHENOLOGY_YAML` through `include_str!`, so the only file they can ever see is the
    // committed one - which is valid. Removing any of the three leaves
    // `cargo test -p domains --lib` at 212 passed / 0 failed; the control (declaring
    // `t_base` in K, which the committed file is not) reddens 29. Python has five tests
    // for exactly those three rules and they have NO successor here. Closing that needs
    // text-injectable variants of the three readers - the shape `allocation_from` already
    // uses in this file - which is a production change and therefore a decision of its
    // own, not something to slip inside a testing batch.

    /// The committed block IS the two cited "Winter Europe" rows - the same cultivar
    /// class in both chapters, not a mix of two.
    ///
    /// ⚠ This is NOT a second copy of `every_value_matches_the_generated_table`.
    /// That test is a ROUND TRIP: it compares the load against a table the Python loaders
    /// produced from the same YAML, so it pins agreement between two readers of one file
    /// and would happily agree on a wrong number. This one names the SOURCE ROW each
    /// value comes from, so replacing a cardinal with a plausible neighbour from [D]'s
    /// range (-1.3 / 3.8 / 6.0 / 15.7) fails here and nowhere else.
    /// Mirrors `test_load_phenology_params_matches_committed_values` and
    /// `test_committed_file_loads_the_cited_winter_europe_values`.
    #[test]
    fn the_committed_phenology_block_is_the_cited_winter_europe_parameterization() {
        let v = phenology_block_from(PHENOLOGY_YAML, "phenology.yaml");
        // Cardinal-cap GDD, [A] McMaster & Wilhelm - both still TODO(cite) placeholders.
        assert_eq!((v["t_base"], v["t_cap"]), (0.0, 30.0));
        // [E] Penning de Vries Table 12 / Table 15, winter-wheat rows.
        assert_eq!((v["tsum_anthesis"], v["tsum_maturity"]), (1100.0, 750.0));
        // [C] Soltani & Sinclair Fig. 8.1 - the wheat vernalization cardinals.
        assert_eq!(
            (
                v["t_base_v"],
                v["t_opt_lower_v"],
                v["t_opt_upper_v"],
                v["t_ceiling_v"]
            ),
            (-1.0, 0.0, 8.0, 12.0)
        );
        // [C] Table 8.1, row "Wheat / Winter Europe".
        assert_eq!((v["vsen"], v["vdsat"]), (0.033, 50.0));
        // [C] Table 7.2, the SAME row of the SAME cultivar class, one chapter over.
        assert_eq!((v["cpp"], v["ppsen"]), (16.0, 0.09));
        // The property those two rows jointly imply, asserted where the numbers live:
        // vsen*vdsat = 1.65 > 1 makes this cultivar QUALITATIVE, so `verfun`'s clamp is
        // load-bearing rather than defensive. A quieter cultivar would silently turn the
        // clamp into dead code and the equation-level test into a tautology.
        assert!(
            v["vsen"] * v["vdsat"] > 1.0,
            "the committed cultivar must stay qualitative"
        );
        // ...and the ordering rule, RESTATED against the committed values.
        //
        // ⚠ This is not coverage of the loader's guard and must not be counted as such:
        // deleting `vernalization()`'s ordering assertion leaves these three green,
        // because they read the same valid file the guard reads. What they do assert is
        // that the committed values still SATISFY the rule - which is what makes a future
        // re-parameterization that violates it fail here rather than at a caller.
        assert!(v["t_base"] < v["t_cap"]);
        assert!(
            v["t_base_v"] < v["t_opt_lower_v"]
                && v["t_opt_lower_v"] <= v["t_opt_upper_v"]
                && v["t_opt_upper_v"] < v["t_ceiling_v"]
        );
        assert!(v["tsum_anthesis"] > 0.0 && v["tsum_maturity"] > 0.0);
    }

    /// A wrong declared unit is rejected, on the REAL file text rather than a fixture.
    ///
    /// Both cases matter and they fail for different reasons: a temperature declared in
    /// kelvin is a unit the tree never uses, while a thermal SUM declared as a bare
    /// `degC` is the plausible slip - a sum is not a temperature, and the two differ by
    /// a factor of time. Mirrors `test_phen_loader_rejects_a_wrong_unit`.
    #[test]
    fn a_wrong_phenology_unit_is_rejected() {
        for (from, to, what) in [
            (
                "  t_base:\n    value: 0.0\n    unit: \"degC\"",
                "  t_base:\n    value: 0.0\n    unit: \"K\"",
                "a cardinal temperature in kelvin",
            ),
            (
                "  tsum_anthesis:\n    value: 1100.0\n    unit: \"degC*day\"",
                "  tsum_anthesis:\n    value: 1100.0\n    unit: \"degC\"",
                "a thermal sum declared as a bare temperature",
            ),
        ] {
            assert_eq!(
                PHENOLOGY_YAML.matches(from).count(),
                1,
                "the substitution must apply exactly once: {what}"
            );
            let broken = PHENOLOGY_YAML.replace(from, to);
            let f = ParamFile::parse(&broken, "phenology.yaml").expect("still parses");
            assert!(
                f.guarded_set(&PHENOLOGY_UNITS, "phenology.yaml").is_err(),
                "{what} must be rejected"
            );
        }
    }

    /// A param added to the file and wired to nothing must FAIL, not be ignored - the
    /// `extra="forbid"` half of the schema. Mirrors
    /// `test_phen_loader_rejects_an_unknown_field`.
    #[test]
    fn an_unknown_phenology_field_is_rejected() {
        let extended = format!(
            "{PHENOLOGY_YAML}  bogus:\n    value: 1.0\n    unit: \"degC\"\n    source: \"x\"\n"
        );
        let f = ParamFile::parse(&extended, "phenology.yaml").expect("still parses");
        assert_eq!(
            f.fields().len(),
            PHENOLOGY_UNITS.len() + 1,
            "the extra key must actually reach the reader"
        );
        assert!(
            f.guarded_set(&PHENOLOGY_UNITS, "phenology.yaml").is_err(),
            "a param wired to nothing must be rejected, not silently ignored"
        );
    }

    /// An entry missing its `source` is rejected - the provenance half of the schema.
    ///
    /// Renaming the key rather than deleting the line breaks it BOTH ways at once (the
    /// required key is absent and an unexpected one is present), which is what the
    /// entry-level `require_keys` is for. Mirrors
    /// `test_phen_loader_rejects_a_missing_source`.
    #[test]
    fn a_phenology_entry_without_a_source_is_rejected() {
        let from = "  vsen:\n    value: 0.033\n    unit: \"1/day\"\n    source:";
        let to = "  vsen:\n    value: 0.033\n    unit: \"1/day\"\n    provenance:";
        assert_eq!(PHENOLOGY_YAML.matches(from).count(), 1);
        let broken = PHENOLOGY_YAML.replace(from, to);
        let f = ParamFile::parse(&broken, "phenology.yaml").expect("still parses");
        assert!(
            f.guarded_set(&PHENOLOGY_UNITS, "phenology.yaml").is_err(),
            "an unsourced param must be rejected"
        );
    }

    // --- S5 batch B: the three SEMANTIC loader guards, now reachable ----------------
    //
    // ⚠ These five tests are the reason `phenology_from`/`vernalization_from`/
    // `photoperiod_from` exist. Before the split the three guards could only ever see the
    // committed file, which is valid, so they were inert - deleting any of the three left
    // `cargo test -p domains --lib` at 216 passed / 0 failed against a live control
    // (declaring `t_base` in kelvin reddened 29). Adding the injectable readers was a
    // production change inside a testing batch and was taken as an explicit decision, not
    // slipped in: the alternative was five Python tests dying at S6 with no successor.
    //
    // Each case mutates the REAL file text rather than a synthetic fixture, so a schema
    // change cannot leave a fixture behind still passing. The guards panic rather than
    // returning `Err` (the frozen loaders' idiom), so the assertion is on `catch_unwind`,
    // exactly as `a_partition_row_that_does_not_sum_to_one_is_rejected` does.

    /// Mutate one `value:` line of the committed file, asserting the substitution applies.
    fn phenology_with(field: &str, value: &str) -> &'static str {
        let from = format!("  {field}:\n    value: ");
        let at = PHENOLOGY_YAML
            .find(&from)
            .unwrap_or_else(|| panic!("{field} is not a top-level phenology param"));
        let start = at + from.len();
        let end = start
            + PHENOLOGY_YAML[start..]
                .find('\n')
                .expect("a value line ends");
        let mut out = String::with_capacity(PHENOLOGY_YAML.len());
        out.push_str(&PHENOLOGY_YAML[..start]);
        out.push_str(value);
        out.push_str(&PHENOLOGY_YAML[end..]);
        assert_ne!(out, PHENOLOGY_YAML, "the substitution must apply");
        // `_from` takes a plain `&str`, but `catch_unwind` wants the captured reference
        // unwind-safe; leaking is what the allocation test already does here.
        Box::leak(out.into_boxed_str())
    }

    /// ⚠ **The panic hook is deliberately NOT suppressed, and that is a correction.**
    /// The first draft wrapped each call in `take_hook`/`set_hook(no-op)`/restore to keep
    /// the expected panics quiet. `set_hook` is PROCESS-GLOBAL and cargo runs these tests
    /// on parallel threads, so two concurrent calls interleave: A installs the no-op, B
    /// takes the *no-op* as its "previous", A restores the real hook, B restores the
    /// no-op — and every panic for the rest of the run prints nothing. It cannot cause a
    /// false pass; it silently destroys the FAILURE MESSAGE of some other test in some
    /// later run, which is this slice's own failure mode one level removed. The
    /// backtraces these produce are noise, and noise is the correct price.
    /// `a_partition_row_that_does_not_sum_to_one_is_rejected` never suppressed it either.
    fn rejects(f: impl FnOnce() + std::panic::UnwindSafe, what: &str) {
        assert!(
            std::panic::catch_unwind(f).is_err(),
            "{what} must be rejected, not loaded"
        );
    }

    /// An inverted cardinal band is rejected: `t_base` above `t_cap` would make the
    /// degree-day rate NEGATIVE on the plateau branch, which is the one branch no scenario
    /// in the tree reaches. Mirrors `test_phen_loader_rejects_inverted_cardinal_band`.
    #[test]
    fn an_inverted_phenology_cardinal_band_is_rejected() {
        let broken = phenology_with("t_base", "40.0");
        rejects(
            || {
                phenology_from(broken, "phenology.yaml");
            },
            "t_base above t_cap",
        );
        // ...and the committed ordering still loads, so the guard is not simply always-on.
        assert_eq!(phenology_from(PHENOLOGY_YAML, "phenology.yaml").t_base, 0.0);
    }

    /// A non-positive thermal sum is rejected - both of them are DIVISORS in
    /// `development_stage`, so a zero is an infinity in DVS rather than a slow crop.
    /// Mirrors `test_phen_loader_rejects_non_positive_sum`.
    #[test]
    fn a_non_positive_thermal_sum_is_rejected() {
        for field in ["tsum_anthesis", "tsum_maturity"] {
            let zero = phenology_with(field, "0.0");
            rejects(
                || {
                    phenology_from(zero, "phenology.yaml");
                },
                field,
            );
            let negative = phenology_with(field, "-1.0");
            rejects(
                || {
                    phenology_from(negative, "phenology.yaml");
                },
                field,
            );
        }
    }

    /// Ill-ordered vernalization cardinals are rejected, in all three ways the ordering can
    /// break. Two of the pairs are DIVISORS (the ramp widths), so an equal pair is a
    /// division by zero and a swapped pair is a negative-width ramp - the response would
    /// come out inverted rather than merely wrong.
    /// Mirrors `test_vernalization_day_rejects_ill_ordered_cardinals`.
    #[test]
    fn ill_ordered_vernalization_cardinals_are_rejected() {
        for (field, value, what) in [
            (
                "t_base_v",
                "0.0",
                "base equal to the lower optimum (a zero-width ramp)",
            ),
            ("t_opt_lower_v", "9.0", "lower optimum above the upper one"),
            ("t_ceiling_v", "8.0", "ceiling equal to the upper optimum"),
        ] {
            let broken = phenology_with(field, value);
            rejects(
                || {
                    vernalization_from(broken, "phenology.yaml");
                },
                what,
            );
        }
        // The committed set loads, so none of the three above passes vacuously.
        assert_eq!(
            vernalization_from(PHENOLOGY_YAML, "phenology.yaml").t_ceiling_v,
            12.0
        );
    }

    /// The two saturation params are bound-checked: `vdsat` is a DIVISOR-shaped saturation
    /// point that must be positive, and a negative `vsen` would make cold RETARD
    /// development - the opposite of the mechanism.
    /// Mirrors `test_vernalization_factor_rejects_bad_params`.
    #[test]
    fn a_bad_vernalization_sensitivity_or_saturation_is_rejected() {
        rejects(
            || {
                vernalization_from(phenology_with("vdsat", "0.0"), "phenology.yaml");
            },
            "vdsat of zero",
        );
        rejects(
            || {
                vernalization_from(phenology_with("vsen", "-0.1"), "phenology.yaml");
            },
            "a negative vsen",
        );
        // `vsen == 0` is LEGAL and must stay so: it is the day-neutral cultivar, whose
        // verfun is 1 everywhere. A guard that rejected it would forbid a real crop.
        assert_eq!(
            vernalization_from(phenology_with("vsen", "0.0"), "phenology.yaml").vsen,
            0.0
        );
    }

    /// The photoperiod pair, same shape: `cpp` is the critical daylength and must be
    /// positive, and a negative `ppsen` would make SHORT days accelerate a long-day crop.
    /// `ppsen == 0` is legal - that is the day-neutral crop, and the tree ships one.
    /// Mirrors `test_photoperiod_factor_rejects_bad_params`.
    #[test]
    fn a_bad_photoperiod_pair_is_rejected() {
        rejects(
            || {
                photoperiod_from(phenology_with("cpp", "0.0"), "phenology.yaml");
            },
            "a critical photoperiod of zero",
        );
        rejects(
            || {
                photoperiod_from(phenology_with("ppsen", "-0.1"), "phenology.yaml");
            },
            "a negative ppsen",
        );
        assert_eq!(
            photoperiod_from(phenology_with("ppsen", "0.0"), "phenology.yaml").ppsen,
            0.0
        );
    }

    // -----------------------------------------------------------------------------
    // S5 batch C: the three water param files.
    //
    // ⚠ THESE GUARDS WERE COMMENTS UNTIL THIS BATCH, and it was measured rather than
    // assumed: deleting the whole positivity loop from `transpiration()`, from
    // `root_depth()` or from `water_cycle()` each left `cargo test -p domains --lib` at
    // 221 passed / 0 failed, against a live control (declaring `aerodynamic_resistance`
    // in `min/m`) that reddened 25. All three read their file through `include_str!`, so
    // the only text they could ever see was the committed one. The `_from` split is the
    // same production change batch B took for `phenology.yaml`, applied to the shape it
    // was answered for — see §5ag.
    // -----------------------------------------------------------------------------

    /// Substitute one `value:` line of any committed param text, asserting it applied.
    ///
    /// The generic sibling of `phenology_with`. It mutates the REAL file rather than a
    /// synthetic fixture, so a schema change cannot leave a fixture behind still passing.
    fn value_of(text: &'static str, field: &str, value: &str) -> &'static str {
        let from = format!("  {field}:\n    value: ");
        let at = text
            .find(&from)
            .unwrap_or_else(|| panic!("{field} is not a top-level param of this file"));
        let start = at + from.len();
        let end = start + text[start..].find('\n').expect("a value line ends");
        let mut out = String::with_capacity(text.len());
        out.push_str(&text[..start]);
        out.push_str(value);
        out.push_str(&text[end..]);
        assert_ne!(out, text, "the substitution must apply");
        Box::leak(out.into_boxed_str())
    }

    /// The committed Penman–Monteith resistances, and what they honestly are.
    ///
    /// ⚠ Pinned as VALUES with their provenance status stated: both are `TODO(cite)`
    /// PROVISIONAL literature-typical placeholders (r_a ~30–100 s/m, r_s ~50–100 s/m for
    /// a well-watered crop), not cited points. Recorded here so the pin is not misread as
    /// an endorsement — it asserts that the numbers the frozen goldens were produced with
    /// are these, not that a source states them.
    /// Mirrors `test_load_transpiration_params_matches_committed_values`.
    #[test]
    fn the_committed_transpiration_resistances_are_the_provisional_literature_typical_pair() {
        let p = transpiration();
        assert_eq!(p.aerodynamic_resistance, 50.0);
        assert_eq!(p.surface_resistance, 70.0);
        // The status, asserted rather than left in a comment that could rot.
        assert!(
            TRANSPIRATION_YAML.contains("TODO(cite)"),
            "the provisional flag has gone; the pin above now over-claims"
        );
        // Both sit inside the ranges the file's own header states.
        assert!((30.0..=100.0).contains(&p.aerodynamic_resistance));
        assert!((50.0..=100.0).contains(&p.surface_resistance));
    }

    /// A non-positive resistance is rejected, and the LEGAL boundary still loads.
    ///
    /// Both are DIVISORS in the combination equation: `r_a = 0` is an infinity in the
    /// aerodynamic term and in `r_s/r_a`, not a slow crop. ⚠ The second half is the part
    /// a rejection-only test cannot have — a guard tuned one notch too tight forbids a
    /// real crop rather than a bad file, and here the committed 50/70 is what proves the
    /// guard is not simply always-on.
    /// Mirrors `test_transp_loader_rejects_non_positive`.
    #[test]
    fn a_non_positive_transpiration_resistance_is_rejected() {
        for field in ["aerodynamic_resistance", "surface_resistance"] {
            for bad in ["0.0", "-1.0"] {
                let broken = value_of(TRANSPIRATION_YAML, field, bad);
                rejects(
                    || {
                        transpiration_from(broken, "transpiration.yaml");
                    },
                    &format!("{field} = {bad}"),
                );
            }
        }
        // The committed file still loads through the same reader.
        assert_eq!(
            transpiration_from(TRANSPIRATION_YAML, "transpiration.yaml").surface_resistance,
            70.0
        );
    }

    /// A wrong declared unit, an unknown field and a missing source are each rejected on
    /// `transpiration.yaml`.
    ///
    /// `min/m` is the plausible slip rather than a nonsense unit — it is the same
    /// quantity at 60× the scale, so it would load a canopy sixty times more resistant
    /// and produce a perfectly plausible-looking season.
    /// Mirrors `test_transp_loader_rejects_a_wrong_unit`,
    /// `test_transp_loader_rejects_an_unknown_field` and
    /// `test_transp_loader_rejects_a_missing_source`.
    #[test]
    fn a_malformed_transpiration_entry_is_rejected() {
        let from = "  aerodynamic_resistance:\n    value: 50.0\n    unit: \"s/m\"";
        assert_eq!(TRANSPIRATION_YAML.matches(from).count(), 1);
        let wrong_unit = Box::leak(
            TRANSPIRATION_YAML
                .replace(
                    from,
                    "  aerodynamic_resistance:\n    value: 50.0\n    unit: \"min/m\"",
                )
                .into_boxed_str(),
        );
        rejects(
            || {
                transpiration_from(wrong_unit, "transpiration.yaml");
            },
            "a resistance declared in min/m",
        );
        let extended = Box::leak(
            format!(
                "{TRANSPIRATION_YAML}  bogus:\n    value: 1.0\n    unit: \"s/m\"\n    source: \"x\"\n"
            )
            .into_boxed_str(),
        );
        rejects(
            || {
                transpiration_from(extended, "transpiration.yaml");
            },
            "a param wired to nothing",
        );
        let no_source = Box::leak(
            TRANSPIRATION_YAML
                .replacen(
                    "    source: \"TODO(cite)",
                    "    provenance: \"TODO(cite)",
                    1,
                )
                .into_boxed_str(),
        );
        assert_ne!(no_source, TRANSPIRATION_YAML, "the substitution must apply");
        rejects(
            || {
                transpiration_from(no_source, "transpiration.yaml");
            },
            "an entry without a source",
        );
    }

    /// The winter-wheat rooting habit is [E] Table 25's own row, not the body text's
    /// cross-species range and not spring wheat's.
    ///
    /// Pinned as VALUES because the file's provenance is the whole point: 0.018 m/day and
    /// 1.3 m are the "Wheat winter" row (Gregory et al., 1978). Spring wheat is 0.012 /
    /// 1.8 — slower but deeper — so the two wheats are not interchangeable, and the
    /// body-text range is 3–5 cm/day, which our 1.8 cm/day sits below.
    /// Mirrors `test_winter_wheat_carries_table_25s_own_row`.
    #[test]
    fn winter_wheat_carries_table_25s_own_root_depth_row() {
        let p = root_depth();
        assert_eq!(p.max_extension_rate, 0.018);
        assert_eq!(p.max_rooted_depth, 1.3);
        // Not spring wheat's row, which is the one adjacent misreading.
        assert_ne!(p.max_extension_rate, 0.012);
        assert_ne!(p.max_rooted_depth, 1.8);
        // Below the body text's general 3-5 cm/day range, the cautious direction.
        assert!(p.max_extension_rate < 0.03);
        // Inside [E] p. 137's stated 0.5-1.5 m species range.
        assert!((0.5..=1.5).contains(&p.max_rooted_depth));
    }

    /// Both root-depth bounds are rejected at zero, and the LEGAL pair still loads.
    ///
    /// Both disable the mechanism SILENTLY and no golden would notice: a zero rate freezes
    /// depth at sowing so the root-zone access gate is shut for the whole season, and a
    /// zero maximum depth is a crop that cannot root at all. That is exactly the shape
    /// this file's own header warns about — the mechanism is bit-identically inert on
    /// every frozen scenario — so the guard has to be where the value is read.
    /// Mirrors `test_a_non_positive_parameter_is_rejected_at_the_boundary`.
    #[test]
    fn a_non_positive_root_depth_bound_is_rejected() {
        for field in ["max_extension_rate", "max_rooted_depth"] {
            for bad in ["0.0", "-0.5"] {
                let broken = value_of(ROOT_DEPTH_YAML, field, bad);
                rejects(
                    || {
                        root_depth_from(broken, "root_depth.yaml");
                    },
                    &format!("{field} = {bad}"),
                );
            }
        }
        assert_eq!(
            root_depth_from(ROOT_DEPTH_YAML, "root_depth.yaml").max_rooted_depth,
            1.3
        );
    }

    /// Potato OVERRIDES the rooting habit rather than sharing wheat's, and its numbers
    /// come from its own reference.
    ///
    /// [E] Table 25 gives potato a row of its own (Vos & Groenwold, 1986) differing in
    /// BOTH values, so sharing wheat's file would assert a rooting habit the source
    /// contradicts. ⚠ The port has NO potato build — its stage 2 is deferred and the four
    /// `crops/potato/*.yaml` overrides are deliberately outside the census (see
    /// `the_recursive_walk_would_see_four_more_and_the_census_does_not`) — so this reads
    /// the override off disk rather than through `include_str!`, which would quietly add
    /// a file to `param_files()` and therefore to the freeze manifest.
    /// Mirrors `test_potato_overrides_root_depth_rather_than_sharing_wheats`.
    #[test]
    fn potato_overrides_the_rooting_habit_rather_than_sharing_wheats() {
        let path = std::path::Path::new(PARAMS_DIR)
            .join("crops/potato")
            .join("root_depth.yaml");
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("the potato override at {path:?} is readable: {e}"));
        let text: &'static str = Box::leak(text.into_boxed_str());
        let potato = root_depth_from(text, "root_depth.yaml");
        assert_eq!(potato.max_extension_rate, 0.014);
        // ⚠ 0.9 is the MIDPOINT of the source's "0.8-1.0" range, recorded as such in the
        // file. The choice of midpoint is ours; either endpoint is equally cited.
        assert_eq!(potato.max_rooted_depth, 0.9);
        // The qualitative fact the two numbers really assert: potato roots shallower and
        // slower than winter wheat. That is the claim a shared file would destroy.
        let wheat = root_depth();
        assert!(potato.max_rooted_depth < wheat.max_rooted_depth);
        assert!(potato.max_extension_rate < wheat.max_extension_rate);
        // ...and it goes through the SAME guards, so the override cannot smuggle a zero.
        rejects(
            || {
                root_depth_from(value_of(text, "max_rooted_depth", "0.0"), "root_depth.yaml");
            },
            "a potato override with a zero depth",
        );
    }

    /// The two water-cycle rates, and the positivity argument that rests on them.
    ///
    /// Both are DESIGN values (a ~2-day engineered-condenser turnover), deliberately equal
    /// so neither half of the ring is the bottleneck. What makes them load-bearing rather
    /// than decorative is `k·dt < 1`: each first-order draw self-limits against the
    /// start-of-step pool, which is why the closed water ring never needs the arbitration
    /// backstop. Asserted at the engine's ACTUAL step, not at 1.0 day.
    /// Mirrors `test_loader_reads_committed_rates`.
    #[test]
    fn the_committed_water_cycle_rates_are_the_matched_design_pair() {
        let p = water_cycle();
        assert_eq!(p.condensation_rate, 0.5);
        assert_eq!(p.recycling_rate, 0.5);
        assert_eq!(
            p.condensation_rate, p.recycling_rate,
            "the two halves of the ring must stay matched"
        );
        for k in [p.condensation_rate, p.recycling_rate] {
            assert!(
                k * super::super::BIO_DT < 1.0,
                "k*dt = {} would let a first-order draw exceed its own pool",
                k * super::super::BIO_DT
            );
        }
    }

    /// A NEGATIVE cycle rate is rejected; a ZERO one is legal, and that asymmetry is a
    /// design decision rather than a looser guard.
    ///
    /// A zero is how a chamber with no condenser is declared — the file header's own rule.
    /// ⚠ NOT a claim about the roster: no scenario in the tree declares one, because the
    /// ring is built only inside the `sealed` branch and an open-field scenario omits the
    /// flows entirely. A negative would run the ring backwards: condensate evaporating into
    /// vapour through a flow whose legs say the opposite, which the flow-level direction
    /// pins would not catch because the legs would still balance.
    /// Mirrors `test_loader_rejects_negative_rate`.
    #[test]
    fn a_negative_water_cycle_rate_is_rejected_but_a_zero_one_is_legal() {
        for field in ["condensation_rate", "recycling_rate"] {
            let broken = value_of(WATER_CYCLE_YAML, field, "-0.1");
            rejects(
                || {
                    water_cycle_from(broken, "water_cycle.yaml");
                },
                &format!("{field} = -0.1"),
            );
            // THE LEGAL BOUNDARY: zero is a chamber with no condenser, not a bad file —
            // the file header's own rule. ⚠ No scenario in the tree declares one (the ring
            // is built only in the sealed branch, which omits the flows rather than zeroing
            // them), so this half asserts the GUARD's shape and nothing about the roster.
            let off = value_of(WATER_CYCLE_YAML, field, "0.0");
            let loaded = water_cycle_from(off, "water_cycle.yaml");
            let got = if field == "condensation_rate" {
                loaded.condensation_rate
            } else {
                loaded.recycling_rate
            };
            assert_eq!(got, 0.0, "a zero {field} must load, not be rejected");
        }
    }

    /// A wrong declared unit on `water_cycle.yaml` is rejected.
    ///
    /// `1/year` is the plausible slip: the same dimension at 365× the scale, so it would
    /// load a condenser that recovers half the standing vapour per YEAR and still produce
    /// a run that conserves water perfectly.
    /// Mirrors `test_loader_rejects_bad_unit`.
    #[test]
    fn a_wrong_water_cycle_unit_is_rejected() {
        let from = "  condensation_rate:\n    value: 0.5\n    unit: \"1/day\"";
        assert_eq!(WATER_CYCLE_YAML.matches(from).count(), 1);
        let broken = Box::leak(
            WATER_CYCLE_YAML
                .replace(
                    from,
                    "  condensation_rate:\n    value: 0.5\n    unit: \"1/year\"",
                )
                .into_boxed_str(),
        );
        rejects(
            || {
                water_cycle_from(broken, "water_cycle.yaml");
            },
            "a condensation rate declared per year",
        );
    }

    // --- batch D: the three carbon-spending param files and their guards --------
    //
    // ⚠ WHAT THESE DO AND DO NOT OWN. The committed VALUES of all three files are already
    // pinned bit-for-bit by `every_value_matches_the_generated_table` (C8's params
    // census), which also carries the partition table row by row. What had no test at
    // all is the other half — the REJECTIONS: the unit strings, the bound checks and the
    // structural rules that decide which files are legal in the first place. A guard
    // nothing exercises is a guard that can be deleted with the suite green, which is
    // this slice's whole subject.

    /// Every unit string in `respiration.yaml` is exact-matched at the loader.
    ///
    /// The exact-string guard is the one that counts (`config/units.py`'s live pint
    /// conversions were all measured to be identities), so a file that renames `1/day`
    /// to `per_day` must be refused rather than coerced.
    /// Mirrors `tests/test_respiration.py::test_resp_loader_rejects_a_wrong_unit`.
    #[test]
    fn a_wrong_respiration_unit_is_rejected() {
        for (field, wrong) in [
            ("maintenance_coef", "per_day"),
            ("q10", "1"),
            ("t_ref", "K"),
            ("growth_efficiency", "fraction"),
            ("o2_half_saturation", "ppm"),
        ] {
            let broken = unit_of(RESPIRATION_YAML, field, wrong);
            rejects(
                || {
                    respiration_from(broken, "respiration.yaml");
                },
                &format!("{field} declared in {wrong:?}"),
            );
        }
        // The committed file still loads through the same reader.
        assert_eq!(
            respiration_from(RESPIRATION_YAML, "respiration.yaml").q10,
            2.0
        );
    }

    /// The respiration bound checks: two strictly positive rates, an efficiency in
    /// `(0, 1]` and a non-negative half-saturation.
    ///
    /// ⚠ THE THREE BOUNDS HAVE DELIBERATELY DIFFERENT SHAPES and the test asserts the
    /// difference, because "they must all be positive" is the plausible wrong reading and
    /// it is wrong at BOTH ends. `growth_efficiency` is half-open `(0, 1]`, so 1.0 is
    /// LEGAL — a lossless conversion is degenerate but not malformed — while 0.0 is
    /// refused. `o2_half_saturation` is the opposite: zero is legal (the `f_O2` throttle
    /// turned off) and only a negative is refused.
    ///
    /// ⚠ The first draft of this test asserted the mirror image of both — a `[0, 1)`
    /// efficiency and a legal zero rate — and went red on its first run. `require_half_open`
    /// says which way round it is in as many words ("zero is a degenerate model, one is
    /// lossless and legitimate"); it was written from the range NAME rather than from the
    /// helper, and the helper was four files away. Recorded because a bound test written
    /// from the wrong end passes on every input except the two that define it.
    /// Mirrors `test_resp_loader_rejects_non_positive_rates` and
    /// `test_resp_loader_rejects_out_of_unit_interval_efficiency`.
    #[test]
    fn the_respiration_bounds_are_rejected_each_at_its_own_shape() {
        for field in ["maintenance_coef", "q10"] {
            for bad in ["0.0", "-1.0"] {
                let broken = value_of(RESPIRATION_YAML, field, bad);
                rejects(
                    || {
                        respiration_from(broken, "respiration.yaml");
                    },
                    &format!("{field} = {bad}"),
                );
            }
        }
        for bad in ["-0.1", "0.0", "1.5"] {
            let broken = value_of(RESPIRATION_YAML, "growth_efficiency", bad);
            rejects(
                || {
                    respiration_from(broken, "respiration.yaml");
                },
                &format!("growth_efficiency = {bad}"),
            );
        }
        // ...and exactly 1.0 is LEGAL: a lossless conversion, degenerate but not
        // malformed. This is the assertion that stops the bound being "simplified" to a
        // closed interval or an exclusive one.
        let lossless = value_of(RESPIRATION_YAML, "growth_efficiency", "1.0");
        assert_eq!(
            respiration_from(lossless, "respiration.yaml").growth_efficiency,
            1.0
        );
        let broken = value_of(RESPIRATION_YAML, "o2_half_saturation", "-1e-9");
        rejects(
            || {
                respiration_from(broken, "respiration.yaml");
            },
            "a negative O2 half-saturation",
        );
        // ...but a ZERO half-saturation is legal — it turns the f_O2 throttle off, which
        // is a real wiring and not a malformed file. This is the assertion that stops
        // the guard from being tightened to `require_positive` by mistake.
        let zeroed = value_of(RESPIRATION_YAML, "o2_half_saturation", "0.0");
        assert_eq!(
            respiration_from(zeroed, "respiration.yaml").o2_half_saturation,
            0.0
        );
    }

    /// The partition table's three structural rules, each rejected on its own.
    ///
    /// A table is legal only if its DVS knots strictly increase, every fraction is in
    /// `[0, 1]`, each row sums to 1 and there are at least two rows to interpolate
    /// between. ⚠ The strictly-increasing rule is asserted with an EQUAL pair as well as
    /// a decreasing one: equal knots make the interpolation weight a division by zero,
    /// and a `<=` written for `<` is invisible on any decreasing case.
    /// Mirrors `test_alloc_loader_rejects_non_increasing_dvs`,
    /// `test_alloc_loader_rejects_out_of_range_fraction` and
    /// `test_alloc_loader_rejects_too_few_rows`.
    #[test]
    fn the_partition_tables_structural_rules_are_each_rejected_separately() {
        // Knots that do not strictly increase — decreasing, and equal.
        for (from, to) in [("- dvs: 1.0", "- dvs: -1.0"), ("- dvs: 1.0", "- dvs: 0.0")] {
            let broken = ALLOCATION_YAML.replacen(from, to, 1);
            assert_ne!(broken, ALLOCATION_YAML, "the substitution must apply");
            rejects(
                || {
                    allocation_from(Box::leak(broken.into_boxed_str()), "allocation.yaml");
                },
                &format!("dvs knots {to}"),
            );
        }
        // A fraction outside [0, 1]. Both are paired with a compensating change so the
        // ROW STILL SUMS TO 1 — otherwise the sum rule fires first and this test would
        // be checking the guard next door.
        for (bad_fl, bad_fr) in [("fl: -0.10", "fr: 1.00"), ("fl: 1.50", "fr: -0.60")] {
            let broken = ALLOCATION_YAML
                .replacen("fl: 0.55", bad_fl, 1)
                .replacen("fr: 0.35", bad_fr, 1);
            assert_ne!(broken, ALLOCATION_YAML, "the substitution must apply");
            rejects(
                || {
                    allocation_from(Box::leak(broken.into_boxed_str()), "allocation.yaml");
                },
                &format!("{bad_fl} with {bad_fr}"),
            );
        }
        // Fewer than two rows: nothing to interpolate between.
        let one_row = ALLOCATION_YAML
            .split("      - dvs: 1.0")
            .next()
            .expect("the table has a second row")
            .to_string();
        rejects(
            || {
                allocation_from(Box::leak(one_row.into_boxed_str()), "allocation.yaml");
            },
            "a one-row partition table",
        );
    }

    /// The stem-reserve guards, including the ORDERING rule that spans two fields.
    ///
    /// ⚠⚠ `trigger_dvs < cessation_dvs <= 2` is the one that matters and it is not a
    /// per-field bound: a window whose ends are each individually legal can still be
    /// EMPTY (trigger == cessation) or UNREACHABLE (cessation above the DVS cap, which
    /// caps at 2.0, so the drain would never stop). Both are asserted, because a
    /// per-field check would pass either.
    /// Mirrors `tests/test_stem_reserves.py::test_the_loader_refuses_a_cessation_that_is_unreachable_or_empty`.
    #[test]
    fn the_stem_reserve_window_is_rejected_when_empty_or_unreachable() {
        // An EMPTY window: the two ends coincide, so the mechanism can never act.
        let broken = value_of(STEM_RESERVES_YAML, "trigger_dvs", "2.0");
        rejects(
            || {
                stem_reserves_from(broken, "stem_reserves.yaml");
            },
            "trigger == cessation (an empty window)",
        );
        // An INVERTED window.
        let broken = value_of(STEM_RESERVES_YAML, "cessation_dvs", "0.5");
        rejects(
            || {
                stem_reserves_from(broken, "stem_reserves.yaml");
            },
            "cessation below the trigger",
        );
        // An UNREACHABLE cessation: DVS caps at 2.0, so a bound above it never stops
        // the drain and the post-maturity tail runs forever.
        let broken = value_of(STEM_RESERVES_YAML, "cessation_dvs", "2.5");
        rejects(
            || {
                stem_reserves_from(broken, "stem_reserves.yaml");
            },
            "a cessation above the DVS cap",
        );
        // The per-field bounds, each on its own.
        for bad in ["0.0", "1.0", "-0.1"] {
            let broken = value_of(STEM_RESERVES_YAML, "remobilizable_fraction", bad);
            rejects(
                || {
                    stem_reserves_from(broken, "stem_reserves.yaml");
                },
                &format!("remobilizable_fraction = {bad}"),
            );
        }
        for bad in ["-0.1", "0.0"] {
            let broken = value_of(STEM_RESERVES_YAML, "remobilization_rate", bad);
            rejects(
                || {
                    stem_reserves_from(broken, "stem_reserves.yaml");
                },
                &format!("remobilization_rate = {bad}"),
            );
        }
        // ⚠ THE TWO FRACTIONS ARE BOUNDED DIFFERENTLY AND BOTH ENDS MATTER. The RATE is
        // `(0, 1]`, so 1.0 — the whole standing reserve drained in one day — is legal.
        // The FRACTION is open at BOTH ends (`0 < fstr < 1`), so 1.0 is refused, because
        // a stem that diverts its entire growth into starch never builds structure at
        // all. Asserted side by side, because reading either bound off the other is the
        // mistake, and a zero rate is where the file's own note says nitrogen moves.
        let whole = value_of(STEM_RESERVES_YAML, "remobilization_rate", "1.0");
        assert_eq!(
            stem_reserves_from(whole, "stem_reserves.yaml").remobilization_rate,
            1.0
        );
        // The committed window is the one the flows actually use.
        let p = stem_reserves_from(STEM_RESERVES_YAML, "stem_reserves.yaml");
        assert!(p.trigger_dvs < p.cessation_dvs && p.cessation_dvs <= 2.0);
    }

    /// Provenance and field-set: enforced at the LOADER for two of the three files, and
    /// at the MANIFEST for the third.
    ///
    /// ⚠⚠ THE ASYMMETRY IS THE FINDING, and it was measured rather than assumed.
    /// `respiration.yaml` and `stem_reserves.yaml` go through `guarded_map`, which is
    /// where the `source:` requirement and the exact-field-set rule live, so both are
    /// refused at load. `allocation.yaml` does NOT: its schema is a LIST of rows rather
    /// than flat value/unit/source scalars, so `allocation_from` reads the table through
    /// the raw node API and never meets those guards. Probed here before this test was
    /// written: stripping its `source:` and adding an unknown top-level key were BOTH
    /// accepted.
    ///
    /// It is not unguarded, and this is the part worth writing down rather than filing as
    /// a gap: the file's newline-normalized sha-256 is pinned in
    /// `docs/biosphere-reference.manifest.json` under `param_files`, and since slice C7
    /// the reference WRITES that manifest and `tests/crossport/test_manifest_writer.py`
    /// compares the committed bytes. So a provenance-only edit to `allocation.yaml` is
    /// caught — as a STALE MANIFEST, not as a load error. Two different failures, two
    /// different fixes, and only one of them names the file.
    ///
    /// What has no guard either way is a FUTURE list-shaped param file: it would inherit
    /// allocation's loader shape and nothing would require it to carry a source until it
    /// reached the manifest census. Recorded as an S6 item, not fixed inside a testing
    /// batch.
    /// Mirrors the `rejects_a_missing_source` / `rejects_an_unknown_field` tests of
    /// `test_respiration.py` and `test_allocation.py`.
    #[test]
    fn provenance_is_enforced_at_the_loader_for_two_files_and_at_the_manifest_for_the_third() {
        // The two guarded files: an entry that loses its `source:` is refused.
        let stripped = strip_first_source(RESPIRATION_YAML);
        rejects(
            || {
                respiration_from(stripped, "respiration.yaml");
            },
            "a respiration entry with no source",
        );
        let stripped = strip_first_source(STEM_RESERVES_YAML);
        rejects(
            || {
                stem_reserves_from(stripped, "stem_reserves.yaml");
            },
            "a stem-reserve entry with no source",
        );
        // ...and so is a key wired to nothing, which is how a parameter gets "set" in a
        // file and never reaches the model.
        let extra = RESPIRATION_YAML.replacen(
            "parameters:
",
            "parameters:
  mystery_coefficient:
    value: 1.0
    unit: \"1/day\"
    source: \"x\"
",
            1,
        );
        assert_ne!(extra, RESPIRATION_YAML, "the substitution must apply");
        rejects(
            || {
                respiration_from(Box::leak(extra.into_boxed_str()), "respiration.yaml");
            },
            "an unknown respiration field",
        );

        // ⚠ THE THIRD FILE, asserted as it actually behaves. Both mutations LOAD, and
        // that is pinned rather than left implicit — if `allocation_from` is ever routed
        // through `guarded_map`, this assertion is what says so out loud instead of a
        // guard quietly appearing.
        let no_source = ALLOCATION_YAML.replacen("    source:", "    provenance:", 1);
        assert_ne!(no_source, ALLOCATION_YAML, "the substitution must apply");
        allocation_from(Box::leak(no_source.into_boxed_str()), "allocation.yaml");
        let extra = format!(
            "{ALLOCATION_YAML}
  mystery:
    value: 1.0
"
        );
        allocation_from(Box::leak(extra.into_boxed_str()), "allocation.yaml");

        // What allocation's loader DOES enforce is the row shape: exactly dvs/fl/fs/fr/fo.
        let sixth = ALLOCATION_YAML.replacen(
            "        fo: 0.00",
            "        fo: 0.00
        fx: 0.00",
            1,
        );
        assert_ne!(sixth, ALLOCATION_YAML, "the substitution must apply");
        rejects(
            || {
                allocation_from(Box::leak(sixth.into_boxed_str()), "allocation.yaml");
            },
            "a partition row with a sixth key",
        );
    }

    /// Replace the declared unit of one top-level param (`value_of`, one field over).
    fn unit_of(text: &'static str, field: &str, unit: &str) -> &'static str {
        let anchor = format!(
            "  {field}:
"
        );
        let at = text
            .find(&anchor)
            .unwrap_or_else(|| panic!("{field} is not a top-level param of this file"));
        let key = "    unit: \"";
        let start = at + text[at..].find(key).expect("the entry declares a unit") + key.len();
        let end = start + text[start..].find('"').expect("the unit is quoted");
        let mut out = String::with_capacity(text.len());
        out.push_str(&text[..start]);
        out.push_str(unit);
        out.push_str(&text[end..]);
        assert_ne!(out, text, "the substitution must apply");
        Box::leak(out.into_boxed_str())
    }

    /// Rename the FIRST `source:` key, so exactly ONE entry loses its provenance.
    ///
    /// One entry rather than the whole file: a file-wide strip would be refused by
    /// whichever guard happens to run first and would not say which rule fired.
    fn strip_first_source(text: &'static str) -> &'static str {
        let out = text.replacen("    source:", "    provenance:", 1);
        assert_ne!(out, text, "the file must carry at least one source");
        Box::leak(out.into_boxed_str())
    }
    // -----------------------------------------------------------------------------
    // S5 batch E — nitrogen: the loader's guards, reachable for the first time.
    //
    // Ported from `tests/test_nitrogen.py`'s config-boundary block and
    // `tests/test_nitrogen_form.py::test_committed_params_are_the_values_read_off_the_primary`.
    // `nitrogen.yaml` carries more guards than any other biosphere param file and, before
    // this batch's `nitrogen_from` split, every one of them was unreachable from any test.
    // -----------------------------------------------------------------------------

    /// Each guard on `nitrogen.yaml`, rejected on its own shape — and each LEGAL edge
    /// asserted beside it, so a bound cannot be quietly widened or narrowed.
    ///
    /// ⚠ Two of these have no Python counterpart and are Rust-side ADDITIONS rather than
    /// ports, written down here so they read as such: the `n_target_coefficient >
    /// n_critical` ordering (without it Greenwood's curve sits below the stress threshold
    /// at *every* crop mass, so `f_N < 1` by construction from the first step) and the
    /// EQUAL case of the concentration band, which is what separates the `<` the loader
    /// writes from the `<=` it could have been written with.
    ///
    /// Mirrors `test_nitrogen_loader_rejects_a_wrong_unit`,
    /// `_rejects_non_positive_capacity`, `_rejects_out_of_range_carbon_fraction`,
    /// `_rejects_inverted_concentration_band`, `_rejects_negative_residual`,
    /// `_rejects_a_missing_source`, `_rejects_an_unknown_field` and
    /// `test_target_rejects_a_non_positive_plateau_bound`.
    #[test]
    fn the_nitrogen_bounds_are_each_rejected_at_their_own_shape() {
        // The capacity is a RATE: zero is a plant that can never take up nitrogen at all.
        for bad in ["0.0", "-1.0"] {
            let broken = value_of(NITROGEN_YAML, "max_uptake_capacity", bad);
            rejects(
                || {
                    nitrogen_from(broken, "nitrogen.yaml");
                },
                &format!("max_uptake_capacity = {bad}"),
            );
        }
        // The carbon fraction is `(0, 1]` — the same shape, and the same argument, as
        // canopy.yaml's: zero is a plant made of no carbon, and one is legitimate (dry
        // matter that is all carbon is degenerate, not malformed).
        for bad in ["0.0", "-0.1", "1.5"] {
            let broken = value_of(NITROGEN_YAML, "carbon_fraction", bad);
            rejects(
                || {
                    nitrogen_from(broken, "nitrogen.yaml");
                },
                &format!("carbon_fraction = {bad}"),
            );
        }
        let lossless = value_of(NITROGEN_YAML, "carbon_fraction", "1.0");
        assert_eq!(
            nitrogen_from(lossless, "nitrogen.yaml").dm_kg_per_mol_c,
            MOLAR_MASS_CARBON_KG_PER_MOL
        );
        // A NEGATIVE residual is refused...
        let broken = value_of(NITROGEN_YAML, "n_residual", "-0.001");
        rejects(
            || {
                nitrogen_from(broken, "nitrogen.yaml");
            },
            "a negative n_residual",
        );
        // ...but ZERO is legal: a plant that can be stripped to bare carbon is a
        // degenerate model, not a malformed file, and `f_N`'s ramp is still well defined.
        // This is the assertion that stops the guard being tightened to `require_positive`.
        let zeroed = value_of(NITROGEN_YAML, "n_residual", "0.0");
        assert_eq!(
            nitrogen_from(zeroed, "nitrogen.yaml").n_residual_per_mol_c,
            0.0
        );
        // The concentration band must be ORDERED — inverted, and (the discriminating
        // case) EQUAL, which makes the ramp a division by zero rather than a ramp.
        for bad in ["0.02", "0.015"] {
            let broken = value_of(NITROGEN_YAML, "n_residual", bad);
            rejects(
                || {
                    nitrogen_from(broken, "nitrogen.yaml");
                },
                &format!("n_residual = {bad} against n_critical = 0.015"),
            );
        }
        // Greenwood's domain bound is a positive crop mass. ⚠ The FUNCTION does not raise
        // on a non-positive bound — `science::target_n_concentration` degenerates to the
        // plateau branch instead, a recorded port decision — so this loader guard is the
        // only thing standing between the file and that degeneracy, and it is the
        // successor to Python's `test_target_rejects_a_non_positive_plateau_bound`.
        for bad in ["0.0", "-1.0"] {
            let broken = value_of(NITROGEN_YAML, "n_target_w_plateau", bad);
            rejects(
                || {
                    nitrogen_from(broken, "nitrogen.yaml");
                },
                &format!("n_target_w_plateau = {bad}"),
            );
        }
        // ⚠ RUST-SIDE ADDITION, no Python counterpart. The plateau is the curve's
        // MAXIMUM, so a target coefficient at or below `n_critical` means the plant is
        // stressed at its own target at every crop mass. Equal is refused too: a plant
        // sitting exactly at critical reads as unstressed only by luck of the ramp's `>=`.
        for bad in ["0.015", "0.014"] {
            let broken = value_of(NITROGEN_YAML, "n_target_coefficient", bad);
            rejects(
                || {
                    nitrogen_from(broken, "nitrogen.yaml");
                },
                &format!("n_target_coefficient = {bad} against n_critical = 0.015"),
            );
        }
        // The unit is an EXACT string: kg/ha/day is the same physical dimension a
        // ten-thousand-fold out, which is exactly the mistake a dimension check passes.
        let rescaled = unit_of(NITROGEN_YAML, "max_uptake_capacity", "kg/ha/day");
        rejects(
            || {
                nitrogen_from(rescaled, "nitrogen.yaml");
            },
            "max_uptake_capacity in kg/ha/day",
        );
        // Provenance, and a key wired to nothing.
        let stripped = strip_first_source(NITROGEN_YAML);
        rejects(
            || {
                nitrogen_from(stripped, "nitrogen.yaml");
            },
            "a nitrogen entry with no source",
        );
        let extra = NITROGEN_YAML.replacen(
            "parameters:\n",
            "parameters:\n  mystery_threshold:\n    value: 1.0\n    unit: \"kg/kg\"\n    source: \"x\"\n",
            1,
        );
        assert_ne!(extra, NITROGEN_YAML, "the substitution must apply");
        rejects(
            || {
                nitrogen_from(Box::leak(extra.into_boxed_str()), "nitrogen.yaml");
            },
            "an unknown nitrogen field",
        );
        // ...and the committed file still loads, so none of the guards above is simply
        // always-on.
        assert_eq!(nitrogen().max_uptake_capacity, 0.0015);
    }

    /// The kg N/kg DM → kg N/mol C fold, against literals computed OUTSIDE the loader.
    ///
    /// `M_C / carbon_fraction = 0.012011 / 0.45 = 0.026691111… kg DM per mol C`, so
    /// `0.005 kg N/kg DM → 1.3345556e-4` and `0.015 → 4.0036667e-4 kg N/mol C`. Written
    /// this way rather than as `n_residual * fold` on purpose: restating the loader's own
    /// formula would assert that the loader matches itself, and would pass just as
    /// happily under a fold applied in the wrong DIRECTION (`cf / M_C`).
    ///
    /// ⚠ A pin on the fold's ORDER was written and then MEASURED INERT rather than
    /// shipped: `0.005 * (M_C / cf)` and `0.005 * M_C / cf` are bit-identical at these
    /// values, so an assertion that the loader divides before it multiplies would have
    /// been green under both orders. The order comment in `nitrogen_from` stays a comment.
    /// Mirrors `test_load_nitrogen_params_applies_carbon_fraction_fold` and
    /// `test_committed_params_are_the_values_read_off_the_primary`.
    #[test]
    fn the_committed_nitrogen_thresholds_fold_to_kg_n_per_mol_c() {
        let p = nitrogen();
        assert!((p.n_residual_per_mol_c - 1.3345556e-4).abs() < 1e-10);
        assert!((p.n_critical_per_mol_c - 4.0036667e-4).abs() < 1e-10);
        // Greenwood eqn (6) as this file carries it: a = 5.697 % (the equation's digits,
        // NOT the abstract's rounded 5.7), b = 0.5, domain bound 1.0 t/ha.
        assert_eq!(p.n_target_coefficient, 0.05697);
        assert_eq!(p.n_target_exponent, 0.5);
        assert_eq!(p.n_target_w_plateau, 1.0);
    }
    // -----------------------------------------------------------------------------
    // S5 batch G, the senescence batch: the LOADER half.
    //
    // ⚠ Three of `test_allocation.py`'s seven senescence loader tests get NO successor
    // here, and each absence is an ownership fact rather than a narrowing:
    //
    //   * `test_senescence_params_file_exists` — the file reaches this module through
    //     `include_str!`, so its absence is a COMPILE error. Guarded harder than by a
    //     test, the disposition batch D gave `test_context_storage_excluded_from_biomass`.
    //   * `test_senescence_loader_rejects_a_wrong_unit` — owned one layer down by
    //     `config`'s `a_wrong_declared_unit_is_rejected`, which is the exact-string guard
    //     `guarded_map` calls. A per-loader copy would assert that `config` still works.
    //   * `test_senescence_loader_rejects_a_missing_source` — likewise: `config`'s
    //     `ParamEntry` requires all three of `{value, unit, source}` and has its own pin.
    //
    // What is NOT owned elsewhere is the domain bound — a relative death rate may not be
    // negative — and before `senescence_from` existed nothing could reach it. Measured:
    // deleting the whole `require_non_negative` loop left this binary at 298 passed / 0
    // failed.
    // -----------------------------------------------------------------------------

    /// Mutate one `value:` line of the committed senescence file, asserting the
    /// substitution applies. The `phenology_with` idiom, against the REAL file text so a
    /// schema change cannot leave a synthetic fixture behind still passing.
    fn senescence_with(field: &str, value: &str) -> &'static str {
        let from = format!("  {field}:\n    value: ");
        let at = SENESCENCE_YAML
            .find(&from)
            .unwrap_or_else(|| panic!("{field} is not a top-level senescence param"));
        let start = at + from.len();
        let end = start
            + SENESCENCE_YAML[start..]
                .find('\n')
                .expect("a value line ends");
        let mut out = String::with_capacity(SENESCENCE_YAML.len());
        out.push_str(&SENESCENCE_YAML[..start]);
        out.push_str(value);
        out.push_str(&SENESCENCE_YAML[end..]);
        assert_ne!(out, SENESCENCE_YAML, "the substitution must apply");
        Box::leak(out.into_boxed_str())
    }

    /// A negative rate is rejected on EVERY one of the five fields; a zero rate LOADS.
    ///
    /// The two halves are one claim. A negative relative death rate is not a slow organ:
    /// `Senescence`'s legs would come out with the organ GAINING carbon out of the litter
    /// sink at a fixed relative rate, internally balanced the whole way, so conservation
    /// cannot see it and neither can the arbitration backstop. A negative `lai_threshold`
    /// puts every canopy permanently in the mutual-shading regime.
    ///
    /// ⚠ And the accept half is what stops the guard being always-on: `senescence.yaml`'s
    /// own header states "a zero rate is valid (no turnover of that organ)", which is
    /// exactly the case a `require_positive` would have broken.
    /// Mirrors `test_senescence_loader_rejects_a_negative_rate` and
    /// `test_senescence_loader_accepts_zero_rate`.
    ///
    /// ⚠ **The accept half READS BACK THE FIELD UNDER TEST, and the first draft did not.**
    /// It asserted `…rdr_leaf` on every iteration, so for four of the five fields it checked
    /// that the field it had NOT touched was unchanged and never looked at the one it had.
    /// A loader that dropped `shade_rate` to a default, or failed to parse `lai_threshold`
    /// and fell back, would have passed it. Measured: hard-coding `shade_rate` to its
    /// committed value inside the loader left the first draft GREEN and reddens this one.
    /// That is this batch's own mutual-shading finding — a pin evaluated where its subject
    /// is invisible — for the third time in one batch, and the third time it was in OUR
    /// column rather than the tree's.
    #[test]
    fn a_negative_senescence_rate_is_rejected_and_a_zero_one_loads() {
        fn field_of(p: &SenescenceParams, field: &str) -> f64 {
            match field {
                "rdr_leaf" => p.rdr_leaf,
                "rdr_stem" => p.rdr_stem,
                "rdr_root" => p.rdr_root,
                "shade_rate" => p.shade_rate,
                "lai_threshold" => p.lai_threshold,
                other => panic!("{other} is not a senescence field"),
            }
        }
        for field in [
            "rdr_leaf",
            "rdr_stem",
            "rdr_root",
            "shade_rate",
            "lai_threshold",
        ] {
            let negative = senescence_with(field, "-0.01");
            rejects(
                || {
                    senescence_from(negative, "senescence.yaml");
                },
                field,
            );
            let zero = senescence_with(field, "0.0");
            let loaded = senescence_from(zero, "senescence.yaml");
            assert_eq!(
                field_of(&loaded, field),
                0.0,
                "a zero {field} must LOAD, and must load AS zero"
            );
        }
        // ...and the committed file still loads, so the rejection is not simply always-on —
        // with all five fields DISTINCT, which is what a loader returning one constant (and
        // therefore passing every `== 0.0` above) fails. Stated as distinctness rather than
        // as five literals on purpose: the VALUES are already pinned bit-exactly by C1's
        // `every_value_matches_the_generated_table`, which is also what would catch a
        // permutation of the five keys. This half is the control for the loop, not a second
        // copy of that gate.
        let p = senescence();
        let mut seen: Vec<u64> = [
            "rdr_leaf",
            "rdr_stem",
            "rdr_root",
            "shade_rate",
            "lai_threshold",
        ]
        .iter()
        .map(|f| field_of(&p, f).to_bits())
        .collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(
            seen.len(),
            5,
            "the five fields must not collapse to one value"
        );
    }

    // ⚠ `test_load_senescence_params_matches_committed_values` gets NO successor, and this
    // batch WROTE one before measuring and then deleted it. All five senescence values are
    // already pinned bit-exactly, as literals, by C1's own gate
    // `every_value_matches_the_generated_table` — `senesc.rdr_leaf` … `senesc.lai_threshold`
    // are five of its rows, and its control table is committed literals rather than
    // anything regenerated from the loader. Measured: doubling `rdr_root` in the YAML
    // reddens that gate. A second copy asserting the same five numbers would be the shape
    // this project has been bitten by before — a rule with two copies has one that goes
    // stale — so the claim is left where it already lives.

    /// ⚠ `rdr_root` is the CLOSEST of the three death rates to its source, and that is a
    /// separate finding from `rdr_leaf`'s — do not quote the leaf's "runs fast" reading as
    /// covering all three.
    ///
    /// [A] Listing 5 ("Crop data for rice, variety IR36", p. 212) is the table §3.2.6
    /// cites. Its root function `LRTT` plateaus at **0.010/day** from DS 1.8, and ours is
    /// 0.01/day flat — inside 10 %. Its leaf function `LLVT` plateaus at **0.012/day**,
    /// and ours is 0.02/day, i.e. **1.667×** the source where the source is fastest. So
    /// the root gap is the FORM only (zero before anthesis) with the magnitude already
    /// literature-centred; the leaf gap is form AND value.
    ///
    /// The two plateau values are the SOURCE's, read off the page image, and the ratio is
    /// arithmetic on them — not a number this tree produced. The Python original's third
    /// assertion (`LISTING5_STEM == ((0.0, 0.0),)`) has no successor and is not an
    /// absence: it asserts that a constant defined four lines above it still holds the
    /// value it was given, which is true of any constant.
    /// Mirrors `test_rdr_root_is_the_closest_of_the_three_to_its_source`.
    #[test]
    fn rdr_root_is_the_closest_of_the_three_frozen_rates_to_its_cited_source() {
        const LISTING5_LEAF_PLATEAU: f64 = 0.012; // LLVT at DS 1.8-2.5
        const LISTING5_ROOT_PLATEAU: f64 = 0.010; // LRTT at DS 1.8-2.5
        let p = senescence();
        let root_gap = (p.rdr_root - LISTING5_ROOT_PLATEAU).abs() / LISTING5_ROOT_PLATEAU;
        assert!(root_gap < 0.10, "rdr_root is {root_gap} from its source");
        let leaf_ratio = p.rdr_leaf / LISTING5_LEAF_PLATEAU;
        assert!(
            (leaf_ratio - 1.667).abs() < 1e-3,
            "rdr_leaf is {leaf_ratio}x its source's plateau"
        );
        // ...and the ORDERING is the claim, stated so a calibration that moved both could
        // not leave both bounds satisfied while inverting which one is the outlier.
        assert!(
            root_gap < (leaf_ratio - 1.0).abs(),
            "the root must stay the nearer of the two"
        );

        // ⚠ THE FORM GAP, which is the load-bearing half and survives BOTH readings of the
        // source. `LLVT` and `LRTT` are exactly **0.0/day at DS 0 through 1.0**, and so is
        // the exercise table [A] p. 113 that our own record quoted for five citation rounds
        // — the two disagree by 12.5x on the terminal magnitude and not at all on this. Our
        // three rates are bare constants applied from DS 0, i.e. non-zero over the entire
        // vegetative phase where every reading of the source is zero. That is the
        // degenerate case of the form we cite, it is DIAGNOSED AND NOT TAKEN, and the
        // reason is in `senescence.yaml`'s header: the flat rate has been standing in for
        // canopy regulation, and the DS-keyed form takes `open_season`'s peak LAI to 16.4
        // against real wheat's ~5-8.
        // Mirrors the surviving half of
        // `test_the_two_source_tables_disagree_by_an_order_but_agree_on_the_form`. Its
        // other half — the 12.5x ratio between two tables that exist only in the test file
        // — has no successor: it is arithmetic on the test's own constants.
        assert!(
            p.rdr_leaf > 0.0 && p.rdr_stem > 0.0 && p.rdr_root > 0.0,
            "the frozen rates shed from DS 0, where the source sheds nothing"
        );
    }

    // -----------------------------------------------------------------------------
    // S5 batch F — soil carbon: the three decomposer files' loader guards, reachable for
    // the first time.
    //
    // The three `_from(text, name)` splits above are this batch's ONLY production change,
    // and they are the mechanical precedent-following kind: ten such splits already exist
    // in this file (`respiration_from`, `transpiration_from`, `phenology_from`,
    // `senescence_from`, `water_cycle_from`, …), each for exactly this reason — the
    // committed text reaches the loader through `include_str!`, so without a text-taking
    // entry point a rejection rule has no caller that can hand it a broken file and the
    // guard is unreachable from any test. ⚠ This is NOT the "production extraction" §5ad
    // held batch F back for. That was about the soil SCIENCE having no extracted
    // functions, and it turned out not to need one (see the batch F block in `flows.rs`).
    //
    // ⚠ The four "loader reads the committed value" tests get NO successor: all seven
    // decomposer scalars are pinned bit-exactly, as hex-float literals, by
    // `every_value_matches_the_generated_table` above. Batch G's rule — a rule with two
    // copies has one that goes stale — and its own measurement that the second copy adds
    // nothing.
    // -----------------------------------------------------------------------------

    /// A NEGATIVE decomposer rate is rejected on every field of both files; a ZERO one
    /// LOADS, and loads AS zero.
    ///
    /// The two halves are one claim. `decomposition.yaml`'s own header states *"A zero rate
    /// is valid (no decomposition)"*, so a `require_positive` would reject a legal file —
    /// which is what makes the accept half the control that stops the guard being
    /// always-on. A NEGATIVE rate is the dangerous one and is invisible downstream: the
    /// decomposer chain would run backwards, microbial biomass and CO2 flowing back into
    /// standing litter, with every leg internally balanced the whole way. Conservation
    /// cannot see it and neither can the arbitration backstop.
    ///
    /// ⚠ **The accept half READS BACK THE FIELD UNDER TEST.** That is batch G's review
    /// finding written into this batch before the fact rather than after it: an accept half
    /// that reads one fixed field asserts, for every OTHER field, that the field it did not
    /// touch is unchanged — and never looks at the one it did.
    /// Mirrors `test_loader_rejects_negative_rate` in `test_decomposition.py` and
    /// `test_microbial_respiration.py`.
    #[test]
    fn a_negative_decomposer_rate_is_rejected_and_a_zero_one_loads() {
        let broken = value_of(DECOMPOSITION_YAML, "decomposition_rate", "-0.01");
        rejects(
            || {
                decomposition_from(broken, "decomposition.yaml");
            },
            "decomposition_rate",
        );
        let zero = value_of(DECOMPOSITION_YAML, "decomposition_rate", "0.0");
        assert_eq!(
            decomposition_from(zero, "decomposition.yaml").decomposition_rate,
            0.0,
            "a zero decomposition_rate must LOAD, and load AS zero"
        );

        fn micro_field(p: &MicrobialRespirationParams, field: &str) -> f64 {
            match field {
                "microbial_respiration_rate" => p.microbial_respiration_rate,
                "o2_half_saturation" => p.o2_half_saturation,
                other => panic!("{other} is not a microbial_respiration field"),
            }
        }
        for field in ["microbial_respiration_rate", "o2_half_saturation"] {
            let broken = value_of(MICROBIAL_RESPIRATION_YAML, field, "-0.01");
            rejects(
                || {
                    microbial_respiration_from(broken, "microbial_respiration.yaml");
                },
                field,
            );
            let zero = value_of(MICROBIAL_RESPIRATION_YAML, field, "0.0");
            let loaded = microbial_respiration_from(zero, "microbial_respiration.yaml");
            assert_eq!(
                micro_field(&loaded, field),
                0.0,
                "a zero {field} must LOAD, and load AS zero"
            );
        }
        // ...and the committed file still loads, with its two fields DISTINCT — the
        // control for the loop, which a loader returning one constant (and so passing
        // every `== 0.0` above) fails. Stated as distinctness rather than as two literals
        // because the VALUES are already pinned by `every_value_matches_the_generated_table`.
        let p = microbial_respiration();
        assert_ne!(
            p.microbial_respiration_rate.to_bits(),
            p.o2_half_saturation.to_bits(),
            "the two fields must not collapse to one value"
        );
    }

    /// Each humification share is rejected OUTSIDE the closed unit interval and accepted at
    /// both ends of it; the slow rate is rejected only below zero.
    ///
    /// Two different rules in one file, so they are exercised separately. A share above 1
    /// is not a merely hot partition: `respired_and_stabilized` computes the complement by
    /// SUBTRACTION, so `f > 1` makes the stabilised leg NEGATIVE — a destination leg that
    /// withdraws from its own receiver — and the flow still balances, which is exactly the
    /// class of error the conservation gate cannot see. `f = 0` and `f = 1` are both legal
    /// and meaningful (`f = 0` is the pre-2026-08-10 frozen form, in which the whole
    /// decayed flux reached the receiving pool), which is why the bound is CLOSED and why
    /// the accept half matters.
    ///
    /// ⚠ The accept half reads back the field under test, per the note above. ⚠ And the
    /// three shares are checked with the OTHER two left at their committed values, so a
    /// guard applied to the wrong field of the four reddens rather than passing.
    #[test]
    fn each_humification_share_is_rejected_outside_the_closed_unit_interval() {
        fn humi_field(p: &HumificationParams, field: &str) -> f64 {
            match field {
                "litter_respired_fraction" => p.litter_respired_fraction,
                "active_stabilization_co2_fraction" => p.active_stabilization_co2_fraction,
                "slow_respired_fraction" => p.slow_respired_fraction,
                "slow_decomposition_rate" => p.slow_decomposition_rate,
                other => panic!("{other} is not a humification field"),
            }
        }
        for field in [
            "litter_respired_fraction",
            "active_stabilization_co2_fraction",
            "slow_respired_fraction",
        ] {
            for bad in ["-0.01", "1.01"] {
                let broken = value_of(HUMIFICATION_YAML, field, bad);
                rejects(
                    || {
                        humification_from(broken, "humification.yaml");
                    },
                    field,
                );
            }
            for (edge, want) in [("0.0", 0.0), ("1.0", 1.0)] {
                let ok = value_of(HUMIFICATION_YAML, field, edge);
                assert_eq!(
                    humi_field(&humification_from(ok, "humification.yaml"), field),
                    want,
                    "{field} at {edge} is inside the CLOSED interval and must load"
                );
            }
        }
        // The slow rate is a rate, not a share: below zero is rejected, and ABOVE one is
        // not — a `require_closed` copied onto it would reject a legal (if fast) file.
        let negative = value_of(HUMIFICATION_YAML, "slow_decomposition_rate", "-0.01");
        rejects(
            || {
                humification_from(negative, "humification.yaml");
            },
            "slow_decomposition_rate",
        );
        let fast = value_of(HUMIFICATION_YAML, "slow_decomposition_rate", "2.0");
        assert_eq!(
            humification_from(fast, "humification.yaml").slow_decomposition_rate,
            2.0,
            "the slow rate carries no upper bound"
        );
        // ...and the committed file's four values are DISTINCT, which is what a loader
        // returning one constant for all four fails.
        let p = humification();
        let mut seen: Vec<u64> = [
            "litter_respired_fraction",
            "active_stabilization_co2_fraction",
            "slow_respired_fraction",
            "slow_decomposition_rate",
        ]
        .iter()
        .map(|f| humi_field(&p, f).to_bits())
        .collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), 4, "the four fields must not collapse");
    }

    /// A wrong unit is rejected on every field of all three decomposer files.
    ///
    /// The unit guard is an exact string match, and the failure it exists to catch is a
    /// silent factor: `1/year` where `1/day` is meant would run the decomposer chain 365x
    /// slow, produce a perfectly conserved, perfectly plausible trajectory, and move no
    /// bound anything else asserts.
    /// Mirrors `test_loader_rejects_bad_unit` in `test_decomposition.py` and
    /// `test_microbial_respiration.py`.
    #[test]
    fn a_wrong_decomposer_unit_is_rejected() {
        let broken = unit_of(DECOMPOSITION_YAML, "decomposition_rate", "1/year");
        rejects(
            || {
                decomposition_from(broken, "decomposition.yaml");
            },
            "decomposition_rate",
        );
        for field in ["microbial_respiration_rate", "o2_half_saturation"] {
            let broken = unit_of(MICROBIAL_RESPIRATION_YAML, field, "1/year");
            rejects(
                || {
                    microbial_respiration_from(broken, "microbial_respiration.yaml");
                },
                field,
            );
        }
        for field in [
            "litter_respired_fraction",
            "active_stabilization_co2_fraction",
            "slow_respired_fraction",
            "slow_decomposition_rate",
        ] {
            let broken = unit_of(HUMIFICATION_YAML, field, "kg/m^2");
            rejects(
                || {
                    humification_from(broken, "humification.yaml");
                },
                field,
            );
        }
    }

    /// The frozen decomposer rates are safe at the frozen STEP, and the litter rate is a
    /// DPM-like one on RothC's own scale.
    ///
    /// Two live claims recovered from `test_soil_fractionation.py`, which is otherwise a
    /// record of a REFUSED design (see the batch F note in the plan doc). Both are about
    /// the tree that exists rather than about the fractionated form that does not:
    ///
    /// * `k · dt < 1` for all three first-order decomposer rates at `BIO_DT`. This is what
    ///   makes every decomposer draw self-limit against its own start-of-step pool, and it
    ///   is why the sealed chambers never need the arbitration backstop. ⚠ Asserted at the
    ///   engine's ACTUAL step: the Python original had the `dt` in its docstring and not in
    ///   its expression, dividing by 365 and stopping, which blessed a rate against a step
    ///   nothing checked.
    /// * [RothC] Coleman & Jenkinson, RothC-26.3 guide §1.5 p. 9 states the plant-material
    ///   decay constants as `DPM 10.0/yr` and `RPM 0.3/yr`. Ours is `0.011/day = 4.015/yr`
    ///   — below the decomposable pool, an order above the resistant one. That places the
    ///   single bulk litter pool on the DECOMPOSABLE side of RothC's split, which is the
    ///   provenance claim `decomposition.yaml`'s own header makes ("fast edge, top of the
    ///   cited range") stated as an ordering rather than as prose.
    ///
    /// ⚠ Stated as a BAND and an ORDERING, and deliberately NOT as the value. A first
    /// draft opened with `(ours_yr - 4.015).abs() < 1e-12`, which made the two assertions
    /// after it decoration: a moved rate fails the value pin first and the band never runs.
    /// It was also a duplicate — `decomp.decomposition_rate` is pinned bit-exactly by
    /// `every_value_matches_the_generated_table`, and 4.015 is that number times 365. The
    /// claim this test is FOR is where our rate sits on RothC's scale, so that is all it
    /// asserts. The two RothC constants are the SOURCE's, not numbers this tree produced.
    /// Mirrors `test_every_rothc_rate_is_safe_at_the_frozen_timestep` and
    /// `test_our_rate_sits_between_the_two_plant_material_rates`.
    #[test]
    fn the_frozen_decomposer_rates_are_step_safe_and_the_litter_rate_is_dpm_like() {
        let rates = [
            decomposition().decomposition_rate,
            microbial_respiration().microbial_respiration_rate,
            humification().slow_decomposition_rate,
        ];
        for k in rates {
            assert!(
                k > 0.0 && k * super::super::BIO_DT < 1.0,
                "k*dt = {} would let a first-order draw exceed its own pool",
                k * super::super::BIO_DT
            );
        }
        // [RothC] §1.5 p. 9 — the two plant-material compartments, in 1/year.
        const K_DPM_YR: f64 = 10.0;
        const K_RPM_YR: f64 = 0.3;
        let ours_yr = decomposition().decomposition_rate * 365.0;
        assert!(
            K_RPM_YR < ours_yr && ours_yr < K_DPM_YR,
            "{ours_yr}/yr is outside RothC's plant-material span"
        );
        // ...and it is nearer the decomposable end than the resistant one, which is the
        // half a two-sided band alone does not say. (Currently 4.015/yr, i.e. 13.4x RPM
        // against 2.5x below DPM.)
        assert!(
            ours_yr / K_RPM_YR > K_DPM_YR / ours_yr,
            "{ours_yr}/yr is nearer the RESISTANT end"
        );
        // The slow pool's rate is [A] Parton 1987 p. 1176's K6 = 0.0038/week, and the
        // committed daily value is that number divided by 7 — arithmetic on the source,
        // not a value read out of this tree.
        assert!(
            (humification().slow_decomposition_rate - 0.0038 / 7.0).abs() < 1e-18,
            "the slow rate is not K6/7"
        );
    }
}
