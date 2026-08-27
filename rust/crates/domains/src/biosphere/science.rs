//! Pure crop-science rate laws — the Rust port of the biosphere's leaf-level physics
//! (canopy / photosynthesis / respiration / transpiration / phenology / allocation /
//! nitrogen / chamber) (Phase-7 P7.4).
//!
//! Every function mirrors its Python twin character-for-character; the transcendentals
//! are op-for-op (`math.exp`→`.exp()`, `math.sqrt`→`.sqrt()`, `q10**e`→`.powf(e)`,
//! `(t+c)**2`→`.powf(2.0)`) so the cross-port deviation is bounded by last-ULP libm
//! differences (Tier 2). The `ValueError`-raising input guards (`ground_area > 0`, …) are
//! omitted — they never fire for the frozen scenarios and would force `Result` on hot
//! rate laws; the *behavioral* clamps (`lai == 0 → 0`, `max(0, …)`, piecewise cutoffs)
//! are kept exactly.

use super::params;
use super::params::{CanopyParams, PartitionRow, PhotosynthesisParams, RespirationParams};
use super::weather::{saturation_vapor_pressure, SVP_B, SVP_C};

/// µmol → mol (leaf-level FvCB is µmol CO₂; the CARBON currency is mol C).
const MICROMOL_TO_MOL: f64 = 1.0e-6;
/// mol/mol → µmol/mol (the FvCB Ci/Ca mole-fraction unit).
const MOLEFRAC_TO_MICRO: f64 = 1.0e6;

// Universal psychrometric constants (FAO-56 standard values, sea level).
const GAMMA_PSYCHROMETRIC: f64 = 67.0;
const AIR_DENSITY: f64 = 1.205;
const AIR_SPECIFIC_HEAT: f64 = 1013.0;
const LATENT_HEAT_VAPORIZATION: f64 = 2.45e6;
const SECONDS_PER_DAY: f64 = 86400.0;

// --- canopy (Beer–Lambert) --------------------------------------------------

/// `LAI = leaf_carbon · sla_per_mol_c / ground_area`.
pub fn leaf_area_index(leaf_carbon: f64, sla_per_mol_c: f64, ground_area: f64) -> f64 {
    leaf_carbon * sla_per_mol_c / ground_area
}

// ⚠ `intercepted_fraction` — the big-leaf `1 − exp(−k · LAI)` (Monsi & Saeki) — was DELETED
// here on 2026-08-27, resolving clause 4 of S5's exit gate. The layered canopy (2026-08-15)
// moved that `exp` into `canopy_assimilation`'s per-depth-point loop, leaving this function
// with no production call site in either tree; it survived only as an export and as unit
// tests of itself. Its claims did not go with it: `test_canopy.py`'s live physics landed on
// `the_depth_quadrature_conserves_photons_against_beer_lambert` in batch A, which checks the
// depth integral against the closed-form Beer–Lambert total rather than against a function
// nothing calls. The Python twin (`domains/biosphere/canopy.py`) outlives it by one slice: it
// is still a shim target for `tests/crossport/measure_tier2_bands.py`, and dies with the
// checker at S6.

// --- FvCB photosynthesis ----------------------------------------------------

/// Rubisco-limited `Ac = Vcmax·(Ci − Γ*) / (Ci + Kc·(1 + O/Ko))`.
pub fn rubisco_limited_rate(ci: f64, p: &PhotosynthesisParams) -> f64 {
    p.vcmax * (ci - p.gamma_star) / (ci + p.kc * (1.0 + p.o2 / p.ko))
}

/// Electron transport `J` — smaller root of `θJ² − (I₂+Jmax)J + I₂·Jmax = 0` (sqrt).
pub fn electron_transport_rate(absorbed_par: f64, p: &PhotosynthesisParams) -> f64 {
    let i2 = p.quantum_yield * absorbed_par;
    let b = i2 + p.jmax;
    let discriminant = b * b - 4.0 * p.theta * i2 * p.jmax;
    (b - discriminant.sqrt()) / (2.0 * p.theta)
}

/// Light/RuBP-limited `Aj = J·(Ci − Γ*) / (4·Ci + 8·Γ*)`.
pub fn light_limited_rate(ci: f64, j: f64, gamma_star: f64) -> f64 {
    j * (ci - gamma_star) / (4.0 * ci + 8.0 * gamma_star)
}

/// Gross leaf assimilation `Ag = max(0, min(Ac, Aj))`.
pub fn gross_leaf_assimilation(ci: f64, absorbed_par: f64, p: &PhotosynthesisParams) -> f64 {
    let ac = rubisco_limited_rate(ci, p);
    let j = electron_transport_rate(absorbed_par, p);
    let aj = light_limited_rate(ci, j, p.gamma_star);
    ac.min(aj).max(0.0)
}

/// Cardinal-temperature response `f_temp(T) ∈ [0, 1]` (piecewise-linear TMPFTB).
pub fn temperature_factor(temp_c: f64, p: &PhotosynthesisParams) -> f64 {
    if temp_c <= p.t_min || temp_c >= p.t_max {
        return 0.0;
    }
    if temp_c < p.t_opt_lo {
        return (temp_c - p.t_min) / (p.t_opt_lo - p.t_min);
    }
    if temp_c > p.t_opt_hi {
        return (p.t_max - temp_c) / (p.t_max - p.t_opt_hi);
    }
    1.0
}

/// Gross canopy assimilation over one window (mol C) — the big-leaf.
///
/// ⚠ `window_s` was `daylength_s` until 2026-08-14. The photoperiod is no longer a
/// multiplier here: the day/night structure lives in the PAR forcing (`light_path`), and
/// the daily carbon budget passes one **day** of seconds to get a per-day rate at this
/// step's PAR. Feeding it a photoperiod would multiply the day length in twice.
#[allow(clippy::too_many_arguments)]
pub fn canopy_assimilation(
    incident_par: f64,
    lai: f64,
    ci: f64,
    temp_c: f64,
    window_s: f64,
    photo: &PhotosynthesisParams,
    canopy: &CanopyParams,
    ground_area: f64,
    limitation: f64,
) -> f64 {
    if lai == 0.0 {
        return 0.0;
    }
    let k = canopy.extinction_coef;
    // Canonical (fixed-array) reduction order — the mirror of the Python tuple. The
    // abscissae are DERIVED from `sqrt(0.6)` in both ports rather than transcribed as
    // decimals, so the two agree on a correctly-rounded IEEE `sqrt` instead of on how
    // many digits someone copied out of the literature.
    let half_spread = 0.5 * 0.6_f64.sqrt();
    let depths = [0.5 - half_spread, 0.5, 0.5 + half_spread];
    let weights = [5.0 / 18.0, 8.0 / 18.0, 5.0 / 18.0];
    let mut weighted_leaf_rate = 0.0;
    for (depth, weight) in depths.iter().zip(weights.iter()) {
        let absorbed_par = k * incident_par * (-k * depth * lai).exp();
        weighted_leaf_rate += weight * gross_leaf_assimilation(ci, absorbed_par, photo);
    }
    let canopy_rate = weighted_leaf_rate * lai;
    let f_temp = temperature_factor(temp_c, photo);
    canopy_rate * window_s * ground_area * MICROMOL_TO_MOL * f_temp * limitation
}

/// Leaf relative death rate including mutual shading: `rdr + shade` above `LAI*`.
///
/// Van Keulen & Seligman (1987), via Penning de Vries et al. (1989) p. 101: in wheat,
/// leaf area is lost at 5 %/day once LAI exceeds 6, to account for mutual shading. A
/// **step**, not a ramp — the source's own form. The comparison is strict (`>`), so the
/// term is inert exactly AT the threshold.
pub fn mutual_shading_rate(lai: f64, rdr_leaf: f64, shade_rate: f64, lai_threshold: f64) -> f64 {
    if lai > lai_threshold {
        rdr_leaf + shade_rate
    } else {
        rdr_leaf
    }
}

// --- respiration ------------------------------------------------------------

/// Q10 temperature multiplier `q10^((T − T_ref)/10)`.
pub fn q10_factor(temp_c: f64, q10: f64, t_ref: f64) -> f64 {
    q10.powf((temp_c - t_ref) / 10.0)
}

/// Daily maintenance respiration `m_ref · biomass · Q10 · maturity` (maturity = 1).
pub fn maintenance_respiration_flux(biomass: f64, temp_c: f64, p: &RespirationParams) -> f64 {
    let maturity = 1.0;
    p.maintenance_coef * biomass * q10_factor(temp_c, p.q10, p.t_ref) * maturity
}

/// Assimilate available for growth `max(0, GASS − MRES)`.
pub fn available_for_growth(gross: f64, maintenance: f64) -> f64 {
    (gross - maintenance).max(0.0)
}

// --- transpiration (Penman–Monteith) ----------------------------------------

/// Slope of the saturation-vapour curve `Δ = B·C·e_s/(T+C)²`.
pub fn slope_svp(temp_c: f64) -> f64 {
    SVP_B * SVP_C * saturation_vapor_pressure(temp_c) / (temp_c + SVP_C).powf(2.0)
}

/// Potential transpiration (mm day⁻¹) from the PM combination equation (`soil_heat_flux`
/// defaults to 0 as in Python).
pub fn penman_monteith_transpiration(
    net_radiation: f64,
    vpd: f64,
    temp_c: f64,
    aerodynamic_resistance: f64,
    surface_resistance: f64,
) -> f64 {
    let soil_heat_flux = 0.0;
    let delta = slope_svp(temp_c);
    let available_energy = net_radiation - soil_heat_flux;
    let aerodynamic_term = AIR_DENSITY * AIR_SPECIFIC_HEAT * vpd / aerodynamic_resistance;
    let denominator =
        delta + GAMMA_PSYCHROMETRIC * (1.0 + surface_resistance / aerodynamic_resistance);
    let latent_flux = (delta * available_energy + aerodynamic_term) / denominator;
    (latent_flux / LATENT_HEAT_VAPORIZATION * SECONDS_PER_DAY).max(0.0)
}

/// The water a shrinking root zone leaves behind at a re-sow, in kg.
///
/// `returned = soil_water * (old_depth - rooted_depth0) / old_depth` — the abandoned
/// FRACTION of the water, from the declared-uniform distribution through the zone. It
/// preserves `FTSW` exactly across the re-sow, needs no clamp (the fraction is < 1), and
/// at the drained upper limit equals `captured_water(old_depth - rooted_depth0)`, the
/// cited-geometry form it generalises.
///
/// ⚠ **This replaced `min(captured_water(abandoned), soil_water)` on 2026-08-12.** That
/// form returned the abandoned column at the drained upper limit — a rounding error
/// against a 1150 kg store, and more than the whole store once the store is geometric,
/// at which point its clamp fired every re-sow and handed the entire root zone to the
/// subsoil. The port carries the rule, not the rationale.
pub fn resow_water_return(soil_water: f64, old_depth: f64, rooted_depth0: f64) -> f64 {
    if old_depth <= 0.0 {
        return 0.0;
    }
    let fraction = (old_depth - rooted_depth0) / old_depth;
    if fraction > 0.0 {
        soil_water * fraction
    } else {
        0.0
    }
}

/// `TTSW = DEPORT · EXTR` ([F] Eqn 14.6), in kg over `ground_area`.
///
/// Recomputed every step because the root zone grows. Identical arithmetic to
/// `captured_water`; the two must agree or a season stops being a closed cycle.
pub fn transpirable_capacity(
    rooted_depth: f64,
    soil_extractable_water: f64,
    ground_area: f64,
) -> f64 {
    rooted_depth * soil_extractable_water * WATER_DENSITY * ground_area
}

/// `FTSW = ATSW / TTSW` ([F] Eqn 14.7). Zero capacity ⇒ 0.0 (maximally stressed), and
/// **not** clamped above 1: an over-filled zone is a real state, the one `Drainage`
/// relieves.
pub fn fraction_transpirable(soil_water: f64, capacity: f64) -> f64 {
    if capacity <= 0.0 {
        return 0.0;
    }
    soil_water / capacity
}

/// `WSFG = min(1, FTSW/WSSG)` — the deficit factor ([F] Eqn 15.3, Box 14.1).
///
/// ⚠ **This replaced an absolute-kg ramp on 2026-08-12** (`sw_wilting`/`sw_critical`,
/// 20/60 kg). Those thresholds were only meaningful against a 1000 kg store — which was
/// 1000 mm of extractable water over 1 m², a 7.7 m soil column. See
/// `docs/plans/post-roadmap-soil-water-rebasing.md`. There is **no wilting floor**: the
/// response is linear to zero at `FTSW = 0`, so the shutoff is asymptotic rather than
/// hard. The port carries the rule, not the rationale — the Python reference measured
/// that the arbitration backstop still never fires.
pub fn water_stress_factor(ftsw: f64, threshold: f64) -> f64 {
    if ftsw >= threshold {
        return 1.0;
    }
    if ftsw > 0.0 {
        ftsw / threshold
    } else {
        0.0
    }
}

/// `WSFG` from the two raw state reads — the single path all three consumers take, so
/// they cannot disagree about `FTSW` within a step.
pub fn soil_water_stress(
    soil_water: f64,
    rooted_depth: f64,
    soil_extractable_water: f64,
    ground_area: f64,
    threshold: f64,
) -> f64 {
    let capacity = transpirable_capacity(rooted_depth, soil_extractable_water, ground_area);
    water_stress_factor(fraction_transpirable(soil_water, capacity), threshold)
}

// --- phenology --------------------------------------------------------------

/// Daily thermal-time increment (°C·day/day) — the cardinal-cap GDD rate.
pub fn daily_thermal_time(temp_c: f64, t_base: f64, t_cap: f64) -> f64 {
    if temp_c <= t_base {
        return 0.0;
    }
    if temp_c >= t_cap {
        return t_cap - t_base;
    }
    temp_c - t_base
}

/// Vernalization days per calendar day (day/day) — Soltani & Sinclair (2012) Eqn 8.3.
///
/// The 3-segment linear cold response with four cardinal temperatures (base `TBV`, lower
/// optimum `TP1V`, upper optimum `TP2V`, ceiling `TCV`): 0 at/below base, a linear ramp to
/// 1 at the lower optimum, the full-effect plateau across the optimum band, a linear ramp
/// back to 0 at the ceiling, and 0 at/above it. Hand-mirrored from
/// `domains/biosphere/phenology.py::vernalization_day` (post-roadmap scope (B) inc. 1).
///
/// The Python side raises on ill-ordered cardinals; here the ordering is a *loader*
/// invariant (the params arrive already validated through `biosphere_params.txt`), so
/// this stays a total function — the same split the rest of this module uses.
pub fn vernalization_day(
    temp_c: f64,
    t_base_v: f64,
    t_opt_lower_v: f64,
    t_opt_upper_v: f64,
    t_ceiling_v: f64,
) -> f64 {
    if temp_c <= t_base_v || temp_c >= t_ceiling_v {
        return 0.0;
    }
    if temp_c < t_opt_lower_v {
        return (temp_c - t_base_v) / (t_opt_lower_v - t_base_v);
    }
    if temp_c <= t_opt_upper_v {
        return 1.0;
    }
    (t_ceiling_v - temp_c) / (t_ceiling_v - t_opt_upper_v)
}

/// Development-rate multiplier `verfun ∈ [0, 1]` — Soltani & Sinclair (2012) Eqn 8.6.
///
/// `1 − vsen·(vdsat − CUMVER)` below saturation, 1 at/above it, clamped to `[0, 1]`. The
/// clamp is load-bearing: with the cited winter-wheat values (`vsen = 0.033`,
/// `vdsat = 50`) the unclamped value is −0.65 at zero cold, i.e. development is fully
/// ARRESTED rather than merely slowed until ~19.7 vernalization days accrue (a
/// *qualitative* cultivar in the source's terms).
pub fn vernalization_factor(vernalization_days: f64, vsen: f64, vdsat: f64) -> f64 {
    if vernalization_days >= vdsat {
        return 1.0;
    }
    (1.0 - vsen * (vdsat - vernalization_days)).clamp(0.0, 1.0)
}

/// Development-rate multiplier `ppfun ∈ [0, 1]` — Soltani & Sinclair (2012) Eqn 7.6.
///
/// The LONG-DAY form (wheat): `1 − ppsen·(CPP − PP)` below the critical photoperiod and 1
/// at/above it, clamped to `[0, 1]` (the source is explicit that a negative value becomes
/// zero, since development is a forward-only process). `daylength_h` is in HOURS — the
/// caller converts from the canonical `daylength_s` forcing.
pub fn photoperiod_factor(daylength_h: f64, cpp: f64, ppsen: f64) -> f64 {
    if daylength_h >= cpp {
        return 1.0;
    }
    (1.0 - ppsen * (cpp - daylength_h)).clamp(0.0, 1.0)
}

/// Development-rate multiplier `WSFD` — Soltani & Sinclair (2012) Eqn 15.8.
///
/// `WSFD = (1 − WSFG)·WSSD + 1`, where `WSFG` is the growth/transpiration deficit factor
/// (`water_stress_factor`, Eqn 15.3). Drought HASTENS development in most species
/// (Table 15.2), so unlike `verfun`/`ppfun` this is **not** a `[0, 1]` limitation factor —
/// it is a ratio on `[0, 1 + WSSD]`. Unstressed (`WSFG = 1`) it is EXACTLY 1.0, which is
/// what keeps every non-water-limited scenario bit-identical on both ports.
///
/// Mirrors `domains.biosphere.phenology.drought_development_factor`. ⚠ The Python side
/// REJECTS `wssd < -1` (below it development would run backwards, which the source rules
/// out); the port carries the rule, not the rationale — but Rust has no constructor to
/// raise from, so the bound is enforced where scenarios are declared rather than here.
/// Negative `WSSD` down to −1 is [F]'s own provision for species drought delays.
pub fn drought_development_factor(wsfg: f64, wssd: f64) -> f64 {
    (1.0 - wsfg) * wssd + 1.0
}

/// `FROOT1 = min(depth / layer, 1)` - the fraction of the reference soil layer the
/// roots have reached ([F] Soltani & Sinclair). A multiplicative gate on a supply term,
/// so it can only reduce a flow, never reverse it.
///
/// Mirrors `domains.biosphere.root_depth.root_zone_fraction`. NOT a function of root
/// carbon: [E] p. 136 states rooted depth is simulated independently of root mass.
pub fn root_zone_fraction(rooted_depth: f64, soil_layer_depth: f64) -> f64 {
    if rooted_depth <= 0.0 {
        return 0.0;
    }
    let fraction = rooted_depth / soil_layer_depth;
    if fraction < 1.0 {
        fraction
    } else {
        1.0
    }
}

/// Water density, kg m^-3 — the constant `soil_layers.py::WATER_DENSITY` mirrors.
pub const WATER_DENSITY: f64 = 1000.0;

/// The water a newly explored soil column of thickness `depth_increment` (m) holds, kg.
///
/// `m * (m^3/m^3) * kg/m^3 * m^2 = kg`. Mirrors `soil_layers.captured_water`; shared by
/// the capture flow and the re-sow return so a season is an exactly closed cycle.
pub fn captured_water(depth_increment: f64, soil_extractable_water: f64, ground_area: f64) -> f64 {
    depth_increment * soil_extractable_water * WATER_DENSITY * ground_area
}

/// `GRTD` — the gated rooted-depth extension rate (m/day). **The single source**, called
/// by both `RootDepthExtension` (which integrates it) and `RootZoneCapture` (which turns
/// it into water). They must not be able to disagree: a capture computed from an ungated
/// rate would move water for depth the roots did not gain.
///
/// Mirrors `root_depth.extension_rate`. Four cited stops, each cutting the RATE to zero
/// (not clamping an increment — the aux channel's dt-independence contract):
/// crop cap ([E] Table 25), soil cap ([F] Box 14.1 `DEPORT >= SOLDEP`; [E] Listing 7
/// L33), flowering ([E] p. 136), and a dry subsoil ([F] Box 14.1 `If WSTORG = 0 Then
/// GRTD = 0` — roots do not extend into dry soil).
#[allow(clippy::too_many_arguments)]
pub fn extension_rate(
    depth: f64,
    thermal_time: f64,
    temp_c: f64,
    soil_water: f64,
    subsoil_water: f64,
    params: &params::RootDepthParams,
    photo: &params::PhotosynthesisParams,
    pheno: &params::PhenologyParams,
    wssg: f64,
    soil_depth: f64,
    soil_extractable_water: f64,
    ground_area: f64,
) -> f64 {
    if depth >= params.max_rooted_depth || depth >= soil_depth {
        return 0.0;
    }
    if subsoil_water <= 0.0 {
        return 0.0;
    }
    let dvs = development_stage(thermal_time, pheno.tsum_anthesis, pheno.tsum_maturity);
    if dvs >= 1.0 {
        return 0.0;
    }
    let f_temp = temperature_factor(temp_c, photo);
    let f_water = soil_water_stress(soil_water, depth, soil_extractable_water, ground_area, wssg);
    params.max_extension_rate * f_water * f_temp
}

/// Development stage `DVS ∈ [0, 2]` from thermal time (TSUM1/TSUM2).
///
/// ⚠ This line spent from the Phase-7 port until 2026-08-27 attached to `root_zone_fraction`
/// three items below, where it read as that function's first sentence; batch B recorded the
/// misattribution and S6 moved it back. Doc-only, no behaviour.
pub fn development_stage(thermal_time: f64, tsum_anthesis: f64, tsum_maturity: f64) -> f64 {
    if thermal_time <= 0.0 {
        return 0.0;
    }
    if thermal_time < tsum_anthesis {
        return thermal_time / tsum_anthesis;
    }
    let reproductive = 1.0 + (thermal_time - tsum_anthesis) / tsum_maturity;
    reproductive.min(2.0)
}

// --- allocation -------------------------------------------------------------

/// Interpolate `(FL, FS, FR, FO)` at `dvs` from the partition table (flat-extrapolated).
pub fn partition_fractions(dvs: f64, table: &[PartitionRow]) -> (f64, f64, f64, f64) {
    let first = &table[0];
    let last = &table[table.len() - 1];
    if dvs <= first.dvs {
        return (first.fl, first.fs, first.fr, first.fo);
    }
    if dvs >= last.dvs {
        return (last.fl, last.fs, last.fr, last.fo);
    }
    for pair in table.windows(2) {
        let (lo, hi) = (&pair[0], &pair[1]);
        if lo.dvs <= dvs && dvs <= hi.dvs {
            let w = (dvs - lo.dvs) / (hi.dvs - lo.dvs);
            return (
                lo.fl + w * (hi.fl - lo.fl),
                lo.fs + w * (hi.fs - lo.fs),
                lo.fr + w * (hi.fr - lo.fr),
                lo.fo + w * (hi.fo - lo.fo),
            );
        }
    }
    unreachable!("dvs strictly inside the increasing knots always brackets")
}

/// Split a daily increment `dmi` into `(leaf, stem, root, storage)`.
pub fn partition(dmi: f64, dvs: f64, table: &[PartitionRow]) -> (f64, f64, f64, f64) {
    let (fl, fs, fr, fo) = partition_fractions(dvs, table);
    (fl * dmi, fs * dmi, fr * dmi, fo * dmi)
}

// --- nitrogen ---------------------------------------------------------------

/// Soil-N availability factor `∈ [0, 1]` (uptake supply side).
/// Greenwood's target whole-crop N concentration (kg N / kg DM) at crop mass `W` (t/ha).
///
/// Greenwood et al. (1990) eqn (6): `%N = a * W^-b` for `W > 1.0 t/ha`, with a = 5.697 for
/// C3 crops and b = 0.5; CONSTANT at `a` below the bound. The plateau is the primary's own
/// statement, not an interpolation — below 1 t/ha growth is exponential and %N stays
/// constant (Agren 1985), and the paper omits all data there. Mirrors
/// `domains.biosphere.nitrogen.target_n_concentration`.
///
/// A non-positive `w_plateau` is a param-file error, caught at load in Python; here the
/// guard degenerates to the plateau branch rather than panicking mid-step.
pub fn target_n_concentration(w_t_ha: f64, coefficient: f64, exponent: f64, w_plateau: f64) -> f64 {
    if w_plateau <= 0.0 || w_t_ha <= w_plateau {
        return coefficient;
    }
    coefficient * w_t_ha.powf(-exponent)
}

pub fn soil_n_availability(soil_n: f64, sn_residual: f64, sn_critical: f64) -> f64 {
    if soil_n <= sn_residual {
        return 0.0;
    }
    if soil_n >= sn_critical {
        return 1.0;
    }
    (soil_n - sn_residual) / (sn_critical - sn_residual)
}

/// Plant-N stress factor `f_N ∈ [0, 1]` (the photosynthesis limiter).
pub fn nitrogen_stress_factor(
    plant_n: f64,
    biomass_c: f64,
    n_residual_per_mol_c: f64,
    n_critical_per_mol_c: f64,
) -> f64 {
    if biomass_c <= 0.0 {
        return 1.0;
    }
    let conc = plant_n / biomass_c;
    if conc <= n_residual_per_mol_c {
        return 0.0;
    }
    if conc >= n_critical_per_mol_c {
        return 1.0;
    }
    (conc - n_residual_per_mol_c) / (n_critical_per_mol_c - n_residual_per_mol_c)
}

// --- chamber seam -----------------------------------------------------------

/// Intercellular `Ci` (µmol mol⁻¹) from a finite chamber carbon pool.
pub fn ci_from_co2_pool(co2_mol: f64, air_mol: f64, ci_ratio: f64) -> f64 {
    let ca = co2_mol / air_mol * MOLEFRAC_TO_MICRO;
    ci_ratio * ca
}

/// O₂ self-limitation `f_O2 = x_O2 / (K_O2 + x_O2) ∈ [0, 1]`.
pub fn oxygen_limitation_factor(o2_mol: f64, air_mol: f64, k_o2: f64) -> f64 {
    let x_o2 = o2_mol.max(0.0) / air_mol;
    let denom = k_o2 + x_o2;
    if denom <= 0.0 {
        return 0.0;
    }
    x_o2 / denom
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The column arithmetic, as a hand value: `m * (m^3/m^3) * kg/m^3 * m^2 = kg`.
    ///
    /// ⚠ It does NOT pin argument order, and the reason is worth recording so nobody
    /// adds a test for it: `soil_extractable_water` and `ground_area` are symmetric
    /// FACTORS OF A PRODUCT, so transposing them is arithmetically identical for every
    /// input, in both ports. It is not a bug that testing could catch — it is not a bug.
    /// (This was checked by mutation, after being flagged as a plausible hazard.) What
    /// callers CAN get wrong is dropping a factor; that is pinned on a non-unit plot in
    /// `system.rs::capture_scales_with_ground_area_at_its_call_sites`.
    /// Mirrors `tests/test_soil_layers.py::test_captured_water_is_the_column_arithmetic`.
    #[test]
    fn captured_water_is_the_column_arithmetic() {
        // 1 m of soil at EXTR 0.13 over 2 m2 holds 260 kg of extractable water.
        assert_eq!(captured_water(1.0, 0.13, 2.0), 260.0);
        assert_eq!(captured_water(0.0, 0.13, 1.0), 0.0);
        assert_eq!(WATER_DENSITY, 1000.0);
    }

    /// `WSFD` ([F] Eqn 15.8) against the source's own two worked examples.
    ///
    /// ⚠ NOTHING ELSE IN THE RUST SUITE CAN CATCH THIS. `WSFD` is bit-identically
    /// inert on every Rust scenario — all of them hold `WSFG == 1` — so no golden, no
    /// parity run and no session test changes if the factor is wrong, dropped, or
    /// inverted. Measured, not assumed: with the whole function replaced by `1.0` the
    /// entire suite stayed green. That is the same position `Drainage` and `root_depth`
    /// are in, and the reason these pins have to CONSTRUCT the stressed state.
    /// Mirrors `tests/test_phenology.py` (the `drought_development_factor` block).
    #[test]
    fn drought_development_factor_reproduces_the_sources_worked_examples() {
        // "if WSSD is 0.4, the maximum value of WSFD at WSFG = 0 is equal to 1.4".
        assert_eq!(drought_development_factor(0.0, 0.4), 1.4);
        // "if WSSD is -0.4, then WSFD will be 0.6 when FTSW and hence WSFG reach 0".
        assert_eq!(drought_development_factor(0.0, -0.4), 0.6);
        // Linear in between (Table 15.3's shape).
        assert_eq!(drought_development_factor(0.5, 0.4), 1.2);
        // -1 arrests development entirely; it is the floor of the form.
        assert_eq!(drought_development_factor(0.0, -1.0), 0.0);
    }

    /// The EXACT identity the whole freeze rests on, on both ports: unstressed must be
    /// 1.0 to the bit, not approximately 1.0. A form that returned `1.0 + 1e-16` here
    /// would move every golden's last ULP on the Python side while the Rust suite,
    /// having no stressed scenario, stayed green.
    #[test]
    fn wsfd_is_exactly_one_when_unstressed() {
        for wssd in [0.4, -0.4, 1.5, -1.0, 0.0] {
            assert_eq!(drought_development_factor(1.0, wssd), 1.0);
        }
    }

    /// The dry-subsoil stop ([F] Box 14.1 `If WSTORG = 0 Then GRTD = 0`), which no
    /// golden and no parity run can catch: it is a branch, not a value.
    #[test]
    fn a_dry_subsoil_stops_extension() {
        let params = params::RootDepthParams {
            max_extension_rate: 0.018,
            max_rooted_depth: 1.3,
        };
        // Only the four cardinals matter to `extension_rate` (via `temperature_factor`);
        // the FvCB fields are inert here, so they carry literature-typical placeholders.
        let photo = params::PhotosynthesisParams {
            vcmax: 100.0,
            jmax: 180.0,
            quantum_yield: 0.385,
            theta: 0.7,
            gamma_star: 42.75,
            kc: 404.9,
            ko: 278400.0,
            o2: 210000.0,
            t_min: 0.0,
            t_opt_lo: 15.0,
            t_opt_hi: 25.0,
            t_max: 40.0,
        };
        let pheno = params::PhenologyParams {
            t_base: 0.0,
            t_cap: 30.0,
            tsum_anthesis: 1100.0,
            tsum_maturity: 900.0,
        };
        let call = |subsoil: f64| {
            extension_rate(
                0.15, 0.0, 20.0, 1000.0, subsoil, &params, &photo, &pheno, 0.30, 1.5, 0.13, 1.0,
            )
        };
        assert_eq!(call(0.0), 0.0);
        assert_eq!(call(-1.0), 0.0); // `<= 0`, so round-off past zero still stops
        assert!(call(195.0) > 0.0); // non-vacuous: it does run with water below
    }

    /// The SOIL's rooting cap ([F] Box 14.1 `If DEPORT >= SOLDEP Then GRTD = 0`; [E]
    /// Listing 7 L33 takes "the shallowest of the rooted depths set by the soil and by
    /// the crop"). No Rust scenario declares a soil shallower than the crop's own cap,
    /// so this branch is unreachable from any run here — measured: dropping it left the
    /// rest of the suite green. The Python side pins it as a whole-season behaviour.
    #[test]
    fn a_shallow_soil_caps_rooting_before_the_crop_does() {
        let params = params::RootDepthParams {
            max_extension_rate: 0.018,
            max_rooted_depth: 1.3,
        };
        let photo = params::PhotosynthesisParams {
            vcmax: 100.0,
            jmax: 180.0,
            quantum_yield: 0.385,
            theta: 0.7,
            gamma_star: 42.75,
            kc: 404.9,
            ko: 278400.0,
            o2: 210000.0,
            t_min: 0.0,
            t_opt_lo: 15.0,
            t_opt_hi: 25.0,
            t_max: 40.0,
        };
        let pheno = params::PhenologyParams {
            t_base: 0.0,
            t_cap: 30.0,
            tsum_anthesis: 1100.0,
            tsum_maturity: 900.0,
        };
        let at = |depth: f64, soil_depth: f64| {
            extension_rate(
                depth, 0.0, 20.0, 1000.0, 195.0, &params, &photo, &pheno, 0.30, soil_depth, 0.13,
                1.0,
            )
        };
        // Below both caps it runs; at the SOIL cap it stops even though the crop's own
        // 1.3 m cap is far away.
        assert!(at(0.4, 1.5) > 0.0);
        assert_eq!(at(0.5, 0.5), 0.0);
        assert_eq!(at(0.6, 0.5), 0.0);
        assert!(at(0.6, 1.5) > 0.0); // ...and it is the SOIL cap doing it, not the depth
    }

    /// The `f_N` RAMP and the uptake shutoff, called directly — the successor to the
    /// Python `n_limited` scenario, retired by C6 of the reference flip.
    ///
    /// ⚠ **Every scenario in the Rust roster holds `f_N == 1`** (they all take the
    /// default `plant_n0`, which sits above `n_critical_per_mol_c` for the whole season),
    /// so the interior of this ramp is unreachable from any run here — the same position
    /// `WSFD`, `Drainage` and `root_depth` are in, and the reason this pin has to
    /// CONSTRUCT the concentration rather than drive a scenario. `n_limited` was the one
    /// run in either tree that reached it; when it was deleted this test and the wiring
    /// pin in `system.rs` took over its claims.
    #[test]
    fn the_nitrogen_stress_ramp_is_linear_between_its_two_knots() {
        // The frozen band (nitrogen.yaml): residual 1/90, critical 1/45 kg N per kg C.
        let (res, crit) = (1.0 / 90.0, 1.0 / 45.0);
        let at = |conc: f64| nitrogen_stress_factor(conc, 1.0, res, crit);
        // The two knots are EXACT — an `f_N` of 0.999... at saturation would move every
        // golden's last ULP, and no scenario here would notice.
        assert_eq!(at(crit), 1.0);
        assert_eq!(at(2.0 * crit), 1.0);
        assert_eq!(at(res), 0.0);
        assert_eq!(at(0.0), 0.0);
        // Strictly increasing through the interior, on the FROZEN band.
        let interior: Vec<f64> = (1..10)
            .map(|i| at(res + f64::from(i) / 10.0 * (crit - res)))
            .collect();
        assert!(
            interior.iter().all(|f| *f > 0.0 && *f < 1.0),
            "{interior:?}"
        );
        assert!(
            interior.windows(2).all(|w| w[1] > w[0]),
            "the ramp is not monotone: {interior:?}"
        );
        // ...and the LINEARITY is pinned on a band whose knots are exactly representable.
        // ⚠ It cannot be pinned on the frozen band: 1/90 and 1/45 are not binary
        // fractions, so `at(res + 0.5*(crit - res))` reads 0.49999999999999994 — the
        // reconstruction's round-off, not the function's. Asserting `== 0.5` there would
        // have been a pin on an accident of the arithmetic used to build the input.
        let clean = |conc: f64| nitrogen_stress_factor(conc, 1.0, 1.0, 3.0);
        assert_eq!(clean(2.0), 0.5);
        assert_eq!(clean(1.5), 0.25);
        assert_eq!(clean(2.5), 0.75);
        // Zero biomass is the structural guard, not a division: an unsown plot is
        // UNSTRESSED, not maximally stressed.
        assert_eq!(nitrogen_stress_factor(0.0, 0.0, res, crit), 1.0);
        assert_eq!(nitrogen_stress_factor(0.0, -1.0, res, crit), 1.0);
    }

    /// `soil_n_availability`'s HARD OFF — `n_limited`'s "pure dilution" regime.
    ///
    /// The scenario declared `soil_n0 = 0.5` against `sn_residual = 1.0`, so availability
    /// is identically zero, `NitrogenUptake` yields a zero leg every step and `plant_n`
    /// stays at its sowing value. The falling `f_N` was then wholly the growing biomass
    /// diluting a FIXED reserve. Mirrors the retired
    /// `tests/test_n_limited.py::test_n_limited_is_pure_dilution`.
    #[test]
    fn soil_n_below_the_residual_shuts_uptake_off_entirely() {
        let (res, crit) = (1.0, 5.0);
        assert_eq!(soil_n_availability(0.5, res, crit), 0.0);
        assert_eq!(soil_n_availability(res, res, crit), 0.0); // `<=`, so AT it is off
        assert_eq!(soil_n_availability(crit, res, crit), 1.0);
        assert_eq!(soil_n_availability(1e9, res, crit), 1.0);
        // The middle ramp is an integrated never-run-hot path in both trees; it is
        // linear, and that is asserted here because nothing else can reach it.
        assert_eq!(soil_n_availability(3.0, res, crit), 0.5);
    }

    // -----------------------------------------------------------------------------
    // S5 batch A — carbon capture: FvCB + the depth-resolved canopy.
    //
    // Ported from `tests/test_photosynthesis.py` and `tests/test_canopy.py`. Every
    // literal below is hand-computed from the cited equation and the params named in
    // `_photo()`, with the arithmetic written out in the comment — never read back out
    // of a run of this tree (§5ad exit gate, clause 2).
    //
    // ⚠ The Python fixture holds the params as literals ON PURPOSE, so that the physics
    // pins are independent of the loader. Kept: `_photo()` and `_canopy()` do not call
    // `params::photosynthesis()`, so a loader regression cannot silently move a pin.
    // -----------------------------------------------------------------------------

    /// The committed winter-wheat placeholders, mirroring `photosynthesis.yaml`.
    fn _photo() -> PhotosynthesisParams {
        PhotosynthesisParams {
            vcmax: 100.0,
            jmax: 180.0,
            quantum_yield: 0.3,
            theta: 0.7,
            gamma_star: 42.75,
            kc: 404.9,
            ko: 278.4,
            o2: 210.0,
            t_min: 0.0,
            t_opt_lo: 15.0,
            t_opt_hi: 25.0,
            t_max: 35.0,
        }
    }

    /// ⚠ `sla_per_mol_c` is deliberately the OLD 0.5872… fold, not today's value. The
    /// Python fixture froze it so the canopy pin below isolates the aggregator's form
    /// from the parameter change that shipped the same day. Copying the live value here
    /// would silently re-point the pin at a different LAI.
    fn _canopy() -> CanopyParams {
        CanopyParams {
            sla_per_mol_c: 0.5872044444444445,
            extinction_coef: 0.6,
        }
    }

    /// `Ac = Vcmax·(Ci − Γ*) / (Ci + Kc·(1 + O/Ko))` — Farquhar, von Caemmerer & Berry
    /// (1980). Hand value: `100·(400 − 42.75) / (400 + 404.9·(1 + 210/278.4))`.
    /// Mirrors `test_photosynthesis.py::test_rubisco_limited_rate_known_value`.
    #[test]
    fn rubisco_limited_rate_is_the_hand_computed_fvcb_value() {
        let ac = rubisco_limited_rate(400.0, &_photo());
        assert!(
            (ac - 32.175_401_396_692_39).abs() <= 1e-12 * 32.175_401_396_692_39,
            "Ac {ac}"
        );
    }

    /// `J` is the SMALLER root of `θJ² − (I₂+Jmax)J + I₂·Jmax = 0` with `I₂ = α·PAR`.
    ///
    /// ⚠ The root choice is the whole content of this function: the larger root exceeds
    /// `Jmax`, which is unphysical, so saturation is asserted alongside the point value.
    /// Mirrors the three `electron_transport` cases in `test_photosynthesis.py`.
    #[test]
    fn electron_transport_is_the_smaller_root_and_saturates_below_jmax() {
        let p = _photo();
        let j = electron_transport_rate(500.0, &p);
        assert!(
            (j - 105.369_374_350_752_44).abs() <= 1e-12 * 105.369_374_350_752_44,
            "J {j}"
        );
        // No light, no electron transport: I₂ = 0 ⇒ the smaller root is 0 exactly.
        assert_eq!(electron_transport_rate(0.0, &p), 0.0);
        // Saturating light drives J toward Jmax FROM BELOW and never through it.
        let saturated = electron_transport_rate(1.0e6, &p);
        assert!(saturated < p.jmax, "J {saturated} reached or passed Jmax");
        assert!((saturated - p.jmax).abs() <= 1e-3 * p.jmax, "J {saturated}");
    }

    /// `Aj = J·(Ci − Γ*) / (4·Ci + 8·Γ*)` at the `J` pinned above.
    /// Mirrors `test_photosynthesis.py::test_light_limited_rate_known_value`.
    #[test]
    fn light_limited_rate_is_the_hand_computed_value() {
        let p = _photo();
        let j = electron_transport_rate(500.0, &p);
        let aj = light_limited_rate(400.0, j, p.gamma_star);
        assert!(
            (aj - 19.383_732_742_948_663).abs() <= 1e-12 * 19.383_732_742_948_663,
            "Aj {aj}"
        );
    }

    /// `Ag = max(0, min(Ac, Aj))` — the CO-LIMITATION, and the `min` is load-bearing.
    ///
    /// ⚠ At `Ci = 400`, `PAR = 500` the two limits are 32.175 and 19.384, so the leaf is
    /// light-limited and `Ag` must read the SMALLER. Inverting the co-limitation to
    /// `max` is the M2 mutation of §5ad's control battery, which reddened eleven tests
    /// and not one of them was about photosynthesis. This is that test.
    #[test]
    fn gross_leaf_assimilation_is_the_co_limited_minimum() {
        let p = _photo();
        let ag = gross_leaf_assimilation(400.0, 500.0, &p);
        let ac = rubisco_limited_rate(400.0, &p);
        let aj = light_limited_rate(400.0, electron_transport_rate(500.0, &p), p.gamma_star);
        assert!(
            aj < ac,
            "the fixture must be light-limited: Ac {ac}, Aj {aj}"
        );
        assert!(
            (ag - 19.383_732_742_948_663).abs() <= 1e-12 * 19.383_732_742_948_663,
            "Ag {ag}"
        );
    }

    /// The load-bearing clamp: at `Ci ≤ Γ*` the `(Ci − Γ*)` factor is ≤ 0, and gross
    /// uptake floors at zero so the carbon SOURCE flow can never become a withdrawal
    /// from plant carbon. Mirrors the parametrized clamp case and the zero-PAR case.
    #[test]
    fn gross_leaf_assimilation_clamps_at_or_below_the_compensation_point() {
        let p = _photo();
        for ci in [42.75, 30.0, 0.0] {
            assert_eq!(gross_leaf_assimilation(ci, 500.0, &p), 0.0, "Ci {ci}");
        }
        // No light ⇒ Aj = 0 ⇒ min(Ac, Aj) = 0. Assimilation → 0 as light → 0.
        assert_eq!(gross_leaf_assimilation(400.0, 0.0, &p), 0.0);
    }

    /// Strictly more light, strictly more carbon — the response is monotone in PAR.
    #[test]
    fn gross_leaf_assimilation_is_monotone_increasing_in_par() {
        let p = _photo();
        let rates: Vec<f64> = [50.0, 100.0, 200.0, 400.0]
            .iter()
            .map(|par| gross_leaf_assimilation(400.0, *par, &p))
            .collect();
        assert!(
            rates.windows(2).all(|w| w[1] > w[0]),
            "not monotone in PAR: {rates:?}"
        );
    }

    /// The piecewise-linear cardinal-temperature response at all nine of its corners.
    ///
    /// ⚠ The two ramp midpoints are the cases that distinguish a ramp from a step, and
    /// the two plateau ends are the cases that distinguish `<`/`<=`. Both pairs are
    /// here for that reason rather than for symmetry.
    #[test]
    fn the_temperature_factor_hits_all_nine_cardinal_points() {
        let p = _photo();
        let cases = [
            (-5.0, 0.0), // below t_min
            (0.0, 0.0),  // at t_min
            (7.5, 0.5),  // midpoint of the up-ramp [0, 15]
            (15.0, 1.0), // start of the plateau
            (20.0, 1.0), // inside the plateau
            (25.0, 1.0), // end of the plateau
            (30.0, 0.5), // midpoint of the down-ramp [25, 35]
            (35.0, 0.0), // at t_max
            (40.0, 0.0), // above t_max
        ];
        for (temp, expected) in cases {
            let f = temperature_factor(temp, &p);
            assert!((f - expected).abs() <= 1e-12, "f_temp({temp}) = {f}");
        }
    }

    /// The composed daily flux, as a hand value.
    ///
    /// ⚠ RE-DERIVED 2026-08-15 in the Python original and carried across verbatim: the
    /// aggregator stopped being a big leaf at the mean PAR and became the cited 3-point
    /// Gaussian depth integral, so this value moved 1.3778614691309006 →
    /// 1.3219831112621092, i.e. **4.05 % LOWER**. The SIGN is the point: `Ag` is concave
    /// in PAR, so resolving the canopy into depths can only lower the sum. A port that
    /// silently picked up the old number would be re-introducing the Jensen high-bias.
    /// Mirrors `test_photosynthesis.py::test_canopy_assimilation_known_value`.
    #[test]
    fn canopy_assimilation_is_the_hand_composed_daily_flux() {
        let lai = 5.0 * _canopy().sla_per_mol_c / 1.0; // 5 mol leaf C over 1 m² ⇒ 2.936
        let daily = canopy_assimilation(
            800.0,
            lai,
            400.0,
            20.0,
            43200.0,
            &_photo(),
            &_canopy(),
            1.0,
            1.0,
        );
        assert!(
            (daily - 1.321_983_111_262_109_2).abs() <= 1e-12 * 1.321_983_111_262_109_2,
            "daily flux {daily}"
        );
    }

    /// `f_temp` multiplies the WHOLE canopy flux, so the up-ramp midpoint halves it
    /// exactly. A temperature response applied per-layer instead would not.
    #[test]
    fn canopy_assimilation_scales_exactly_with_the_temperature_factor() {
        let (p, c) = (_photo(), _canopy());
        let lai = 5.0 * c.sla_per_mol_c;
        let warm = canopy_assimilation(800.0, lai, 400.0, 20.0, 43200.0, &p, &c, 1.0, 1.0);
        let cool = canopy_assimilation(800.0, lai, 400.0, 7.5, 43200.0, &p, &c, 1.0, 1.0);
        assert!(
            (cool - warm * 0.5).abs() <= 1e-12 * warm,
            "cool {cool} vs warm/2 {}",
            warm * 0.5
        );
    }

    /// No leaf area, no carbon — and no `0/0` on the way there.
    ///
    /// ⚠ The small-LAI limit is the case a per-leaf-area formulation gets wrong: the
    /// mean absorbed PAR per unit leaf tends to `k·incident_par` rather than diverging,
    /// so the flux must vanish smoothly instead of blowing up or reading `NaN`.
    #[test]
    fn canopy_assimilation_vanishes_smoothly_as_lai_goes_to_zero() {
        let (p, c) = (_photo(), _canopy());
        assert_eq!(
            canopy_assimilation(800.0, 0.0, 400.0, 20.0, 43200.0, &p, &c, 1.0, 1.0),
            0.0
        );
        let flux: Vec<f64> = [1e-3, 1e-6, 1e-9]
            .iter()
            .map(|lai| canopy_assimilation(800.0, *lai, 400.0, 20.0, 43200.0, &p, &c, 1.0, 1.0))
            .collect();
        assert!(flux.iter().all(|f| f.is_finite() && *f > 0.0), "{flux:?}");
        assert!(flux[0] > flux[1] && flux[1] > flux[2], "{flux:?}");
    }

    /// ⚠⚠ **THE ONE THE GOLDENS WERE DOING ALONE.** The depth quadrature must conserve
    /// photons: it redistributes light through the canopy and creates none.
    ///
    /// The absorption profile is `k·PAR·exp(−k·L)`, whose exact integral over
    /// `L ∈ [0, LAI]` is `PAR·(1 − exp(−k·LAI))` — Beer–Lambert, Monsi & Saeki (1953).
    /// The reference evaluates that integral by 3-point Gauss–Legendre. So in a regime
    /// where the leaf response is LINEAR in absorbed PAR, the canopy flux must equal the
    /// Beer–Lambert total times that linear coefficient.
    ///
    /// The linear regime is real, not a fiction: at `I₂ ≪ Jmax` the smaller root of the
    /// non-rectangular hyperbola tends to `I₂ = α·PAR`, so `Ag → α·PAR·(Ci − Γ*)/(4Ci +
    /// 8Γ*)`. PAR is therefore driven to 1e-4 µmol here — far into that limit — and the
    /// coefficient is taken from the function itself at a single leaf, so the test
    /// compares the CANOPY INTEGRAL against the closed form rather than restating the
    /// loop.
    ///
    /// ⚠ Why this test exists: §5ad's M1c control replaced the Gaussian weights with a
    /// flat average and reddened **four tests, all four of them committed-byte golden
    /// comparisons**. No behavioural gate moved. The quadrature is the one part of the
    /// canopy whose correctness was asserted by nothing that survives a regeneration.
    ///
    /// ⚠ **The tolerance is DERIVED per canopy, not picked**, and the first attempt was
    /// wrong in the instructive direction: a flat `1e-4` held at `LAI 2.936` and failed at
    /// `LAI 6` with a residual of 7.3e-4, because the 3-point Gauss error grows as the
    /// SIXTH power of `k·LAI`. The classical bound for `n = 3` on `[0, 1]` is
    /// `(3!)⁴ / (7·(6!)³) · max|f⁽⁶⁾|`, and for `f(x) = exp(−a·x)` that maximum is `a⁶`;
    /// dividing by the integral `(1 − e⁻ᵃ)/a` makes it relative. The gate therefore
    /// tightens itself on open canopies — 4.0e-3 at `LAI 6`, 3.1e-8 at `LAI 1` — instead
    /// of being set everywhere by its loosest case, which is what a flat tolerance does.
    ///
    /// ⚠ The `1e-6` floor is a second error term, not slack: the probe regime is very
    /// nearly linear rather than exactly linear, and the leaf coefficient is read at a
    /// different PAR from the one the shaded layers see. That residual is `~I₂/Jmax ≈ 2e-7`,
    /// so a floor an order above it stops the sparse-canopy cases asserting past their own
    /// arithmetic. Both terms sit far below the 26 % error M1c introduces.
    #[test]
    fn the_depth_quadrature_conserves_photons_against_beer_lambert() {
        let (p, c) = (_photo(), _canopy());
        let (ci, temp_c, window_s, area) = (400.0, 20.0, 43200.0, 1.0);
        // Deep in the linear-response regime: I₂ = 0.3 * 1e-4 is ~6e-8 of Jmax.
        let par = 1.0e-4;
        // The per-leaf response coefficient, read from the reference at one leaf rather
        // than re-derived, so this compares the INTEGRAL and not the leaf law.
        let per_unit_absorbed = gross_leaf_assimilation(ci, par, &p) / par;
        assert!(
            per_unit_absorbed > 0.0,
            "the linear regime must be positive"
        );

        for lai in [0.5, 1.0, 2.936, 6.0] {
            let produced = canopy_assimilation(par, lai, ci, temp_c, window_s, &p, &c, area, 1.0);
            // Beer–Lambert total absorbed, times the linear leaf response, times the
            // same window / area / unit factors the aggregator applies.
            let absorbed = par * (1.0 - (-c.extinction_coef * lai).exp());
            let expected = absorbed
                * per_unit_absorbed
                * window_s
                * area
                * MICROMOL_TO_MOL
                * temperature_factor(temp_c, &p);
            // Gauss–Legendre n=3 truncation bound on exp(-a·x) over [0, 1], relative to
            // the integral itself, floored by the probe's residual non-linearity.
            let a = c.extinction_coef * lai;
            const GAUSS3_COEF: f64 = 1296.0 / (7.0 * 720.0 * 720.0 * 720.0);
            let integral = (1.0 - (-a).exp()) / a;
            let tol = (GAUSS3_COEF * a.powi(6) / integral).max(1e-6);
            let relative = (produced - expected).abs() / expected;
            assert!(
                relative <= tol,
                "LAI {lai}: the depth quadrature lost or invented photons — \
                 produced {produced}, Beer-Lambert {expected}, relative {relative} \
                 against a derived bound of {tol}"
            );
        }
    }

    /// `LAI = leaf_carbon · sla_per_mol_c / ground_area`, exact in binary.
    ///
    /// ⚠ `ground_area` is a DIVISOR here and a factor in `canopy_assimilation`; the
    /// inverse-scaling case is what distinguishes the two roles, so it is asserted
    /// rather than left to the point value.
    /// Mirrors the `leaf_area_index` block of `test_canopy.py`.
    #[test]
    fn leaf_area_index_is_carbon_times_sla_over_ground() {
        assert_eq!(leaf_area_index(100.0, 0.5, 2.0), 25.0);
        assert_eq!(leaf_area_index(0.0, 0.5, 2.0), 0.0);
        let small = leaf_area_index(100.0, 0.5, 1.0);
        let large = leaf_area_index(100.0, 0.5, 2.0);
        assert_eq!(large, small / 2.0);
    }

    // --- the chamber seam (batch A's third file) ---------------------------------
    //
    // ⚠ These two have NO Python ancestor in `test_gas_exchange.py`: that file's subject
    // is flow-level stoichiometry, and it reaches `ci_from_co2_pool` only through the
    // sealed `CarbonContext`. They are written here as ADDITIONAL coverage of two of the
    // 28 untested `science.rs` functions, and the by-name census must not count them as
    // successors to claims they do not cover.

    /// `Ci = ci_ratio · (co2_mol / air_mol) · 1e6` — the finite chamber's Ci seam.
    ///
    /// The mole-fraction → µmol mol⁻¹ conversion and the Ci/Ca ratio are separate
    /// factors, and both are pinned: 0.4 mol CO₂ in 1000 mol of air is a mole fraction
    /// of 4e-4, i.e. `Ca = 400` µmol mol⁻¹, and at `ci_ratio = 0.7` that is `Ci = 280`.
    /// The `ci_ratio = 1` case is asserted separately because it is the only input for
    /// which Ci and Ca coincide — the case that distinguishes the two factors' roles.
    #[test]
    fn ci_from_a_finite_pool_is_the_mole_fraction_times_the_ci_ratio() {
        assert_eq!(ci_from_co2_pool(0.4, 1000.0, 0.7), 280.0);
        assert_eq!(ci_from_co2_pool(0.4, 1000.0, 1.0), 400.0);
        // Linear in the pool, inverse in the air basis: halving the air doubles Ci.
        assert_eq!(ci_from_co2_pool(0.8, 1000.0, 0.7), 560.0);
        assert_eq!(ci_from_co2_pool(0.4, 500.0, 0.7), 560.0);
        // An emptied chamber is Ci = 0, not a division artefact.
        assert_eq!(ci_from_co2_pool(0.0, 1000.0, 0.7), 0.0);
    }

    /// `f_O2 = x / (K + x)` — the Michaelis self-limit on sealed respiration.
    ///
    /// The half-saturation point is the function's defining property and needs no
    /// literature value to check: at `x = K` the factor is exactly ½, at `x = 9K`
    /// exactly 0.9, whatever `K` is. Written that way on purpose — a pin on today's
    /// `o2_half_saturation` would be a snapshot of a param file, not of the form.
    ///
    /// ⚠ The `max(0, …)` clamp is behaviour, not an input guard: a chamber driven to a
    /// slightly negative O₂ by float noise must yield `f_O2 = 0`, never a negative
    /// factor that would REVERSE the burn's sign. It is asserted here for that reason.
    #[test]
    fn oxygen_limitation_is_michaelis_and_half_saturates_at_k() {
        let air = 1000.0;
        for k in [1.0e-4, 1.0e-3, 0.05] {
            // x == K  ⇒  K/(K+K) = 1/2, exactly, for any K.
            assert_eq!(oxygen_limitation_factor(k * air, air, k), 0.5, "K = {k}");
            // x == 9K ⇒  9K/(K+9K) = 9/10.
            let f9 = oxygen_limitation_factor(9.0 * k * air, air, k);
            assert!((f9 - 0.9).abs() <= 1e-15, "K = {k}: f(9K) = {f9}");
            // An empty chamber shuts respiration off entirely.
            assert_eq!(oxygen_limitation_factor(0.0, air, k), 0.0, "K = {k}");
            // Saturating: below 1 always, and monotone increasing.
            let lo = oxygen_limitation_factor(0.10 * air, air, k);
            let hi = oxygen_limitation_factor(0.21 * air, air, k);
            assert!(lo < hi && hi < 1.0, "K = {k}: {lo} !< {hi} !< 1");
        }
    }

    /// The clamp's DIRECTION, on its own, because getting it wrong is silent.
    #[test]
    fn a_negative_oxygen_amount_clamps_to_zero_rather_than_reversing_the_sign() {
        let f = oxygen_limitation_factor(-1.0e-9, 1000.0, 1.0e-4);
        assert_eq!(f, 0.0);
    }

    // --- S5 batch B: phenology, the equation half -------------------------------
    //
    // Ported from `tests/test_phenology.py` (the `daily_thermal_time`,
    // `development_stage`, `vernalization_day`, `vernalization_factor` and
    // `photoperiod_factor` blocks). Every expected value is hand-computed from the cited
    // equation with the arithmetic in the comment, never read out of this tree.
    //
    // ⚠⚠ **The before-battery is the reason these exist, and it is worse than batch
    // A's.** Eight live mutations were run against `cargo test -p domains --lib`
    // (197 tests, logs in `M:\claud_projects\temp\s5-batch-b`):
    //
    //   | mutation                                            | red | about the mechanism |
    //   |-----------------------------------------------------|----:|--------------------:|
    //   | uncap `daily_thermal_time` at `t_cap`                |   0 |                   0 |
    //   | `development_stage` reproductive divisor -> TSUM1    |   0 |                   0 |
    //   | drop `development_stage`'s 2.0 cap                   |   0 |                   0 |
    //   | flip `vernalization_day`'s upper ramp                |   0 |                   0 |
    //   | drop `vernalization_factor`'s clamp                  |   0 |                   0 |
    //   | `photoperiod_factor` long-day -> short-day           |   3 |                   0 |
    //   | drop the photoperiod multiply in the accumulator     |   3 |                   0 |
    //   | drop the vernalization multiply in the accumulator   |   3 |                   0 |
    //
    // The three reds are the SAME three tests every time - a peak-LAI band, a
    // mutual-shading regime check and a trajectory fixed-point. None of them is about
    // phenology; they redden because a broken development rate moves a trajectory and a
    // band somewhere else notices, which is 5ad's finding restated on a second batch.
    //
    // ⚠⚠ **A separate probe separated "untested" from "unreachable", and they are
    // not the same defect.** Replacing each branch body with a `panic!` and re-running
    // measures whether the suite ever ENTERS it: the reproductive branch fires in 23
    // tests, the `DVS = 2` cap in 20, the vernalization upper ramp in 20 and the
    // `verfun` clamp in 20 - all live, all asserted by nothing. The `t_cap` plateau
    // fires in **zero tests of the entire workspace, goldens included**: no scenario is
    // ever that warm. That one is recorded as a finding rather than fixed here - a test
    // can pin the branch, but only a scenario can exercise it.

    /// `daily_thermal_time` - the cardinal-capped degree-day rate, at its cardinals.
    ///
    /// Hand values with `t_base = 5`, `t_cap = 25` (band = 20 degC), deliberately NOT the
    /// committed `(0, 30)`: with a base of zero the subtraction is invisible, so a rate
    /// that returned the raw temperature would pass on the frozen params.
    /// Mirrors `test_daily_thermal_time_cardinal_values` and its two neighbours.
    #[test]
    fn thermal_time_is_the_degree_day_rate_capped_at_both_cardinals() {
        // Every value below is exact in binary, so these are equalities, not bands.
        for (temp, expected) in [
            (-3.0, 0.0),  // below base
            (5.0, 0.0),   // AT base - the boundary is closed at zero
            (12.0, 7.0),  // mid-band: 12 - 5
            (15.0, 10.0), // mid-band: 15 - 5
            (25.0, 20.0), // AT cap -> the band width, t_cap - t_base
            (33.0, 20.0), // above cap -> the plateau, not 28
        ] {
            assert_eq!(
                daily_thermal_time(temp, 5.0, 25.0),
                expected,
                "T = {temp} degC"
            );
        }
        // The plateau is exactly the band width however hot it gets - the claim that
        // separates a capped rate from an uncapped one, and the one no scenario in the
        // tree can make (nothing ever reaches 30 degC).
        for temp in [25.0, 100.0, 1.0e6] {
            assert_eq!(daily_thermal_time(temp, 5.0, 25.0), 20.0);
        }
        // Monotone non-decreasing across both breakpoints.
        let ladder = [-10.0, 0.0, 5.0, 5.0001, 15.0, 24.999, 25.0, 50.0];
        let rates: Vec<f64> = ladder
            .iter()
            .map(|t| daily_thermal_time(*t, 5.0, 25.0))
            .collect();
        assert!(
            rates.windows(2).all(|w| w[0] <= w[1]),
            "the rate must never fall as it warms: {rates:?}"
        );
    }

    /// `development_stage` - the two-phase TSUM ramp, at its cardinals.
    ///
    /// Round sums (TSUM1 = 1000, TSUM2 = 500) so every literal is exact: DVS 0 at
    /// emergence, 0.5 at half of TSUM1, 1 at anthesis, 1.5 halfway through TSUM2, 2 at
    /// maturity, and capped at 2 beyond it.
    ///
    /// ⚠ The two phases divide by DIFFERENT sums, and the mutation that swaps them
    /// (reproductive / TSUM1 instead of / TSUM2) reddens nothing else in the binary -
    /// 1250 degC*day would read 1.25 instead of 1.5. It is pinned here because the
    /// reproductive branch IS live (23 tests enter it) and no other test looks at it.
    /// Mirrors `test_development_stage_cardinal_values` and its monotonicity neighbour.
    #[test]
    fn development_stage_is_the_two_phase_tsum_ramp() {
        for (thermal_time, expected) in [
            (-50.0, 0.0),  // before emergence - clamped, not negative
            (0.0, 0.0),    // emergence
            (500.0, 0.5),  // half the vegetative sum
            (1000.0, 1.0), // ANTHESIS: the boundary is CLOSED, which is what gates the
            //                two vegetative modifiers off at exactly this point
            (1250.0, 1.5), // 1 + 250/500 - the reproductive divisor is TSUM2
            (1500.0, 2.0), // maturity
            (3000.0, 2.0), // past maturity -> the cap, not 5.0
        ] {
            assert_eq!(
                development_stage(thermal_time, 1000.0, 500.0),
                expected,
                "TT = {thermal_time} degC*day"
            );
        }
        let ladder = [0.0, 250.0, 1000.0, 1100.0, 1500.0, 5000.0];
        let dvs: Vec<f64> = ladder
            .iter()
            .map(|tt| development_stage(*tt, 1000.0, 500.0))
            .collect();
        assert!(
            dvs.windows(2).all(|w| w[0] <= w[1]),
            "development is forward-only: {dvs:?}"
        );
    }

    /// `vernalization_day` (VERDAY) - Soltani & Sinclair (2012) Eqn 8.3, at its four
    /// cardinals and both ramp midpoints.
    ///
    /// Committed winter-Europe wheat cardinals (Fig. 8.1 / Table 8.1): base -1 degC,
    /// lower optimum 0 degC, upper optimum 8 degC, ceiling 12 degC. Both ramps are exact
    /// dyadics: the lower midpoint -0.5 gives (-0.5 - -1)/(0 - -1) = 0.5, the upper
    /// midpoint 10 gives (12 - 10)/(12 - 8) = 0.5.
    ///
    /// ⚠ Both BOUNDARIES are closed at zero - at the base AND at the ceiling - so
    /// a `<`/`<=` slip at either end is a silent full vernalization day.
    /// Mirrors `test_vernalization_day_matches_hand_computed_literals`.
    #[test]
    fn vernalization_day_is_the_three_segment_cold_response() {
        let c = (-1.0, 0.0, 8.0, 12.0);
        for (temp, expected) in [
            (-5.0, 0.0), // below base
            (-1.0, 0.0), // AT base - closed
            (-0.5, 0.5), // lower ramp midpoint
            (0.0, 1.0),  // lower optimum - full effect begins
            (4.0, 1.0),  // inside the optimum band
            (8.0, 1.0),  // upper optimum - still full effect
            (10.0, 0.5), // upper ramp midpoint
            (12.0, 0.0), // AT ceiling - closed
            (20.0, 0.0), // above ceiling
        ] {
            assert_eq!(
                vernalization_day(temp, c.0, c.1, c.2, c.3),
                expected,
                "T = {temp} degC"
            );
        }
        // Bounded on [0, 1] and unimodal - no interior dip, which is what a swapped
        // ramp numerator would produce.
        let xs: Vec<f64> = (0..160).map(|i| i as f64 * 0.25 - 8.0).collect();
        let vs: Vec<f64> = xs
            .iter()
            .map(|x| vernalization_day(*x, c.0, c.1, c.2, c.3))
            .collect();
        assert!(vs.iter().all(|v| (0.0..=1.0).contains(v)), "{vs:?}");
        let peak = vs
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).expect("no NaN in the response"))
            .expect("the sweep is non-empty")
            .0;
        assert!(
            vs[..peak].windows(2).all(|w| w[0] <= w[1]),
            "the cold response must rise to its plateau without a dip"
        );
        assert!(
            vs[peak..].windows(2).all(|w| w[0] >= w[1]),
            "...and fall from it without a rise"
        );
    }

    /// ⚠⚠ **THE TRANSCRIPTION CHECK.** Soltani & Sinclair (2012) p. 91 states an
    /// arithmetic CONSEQUENCE of Eqn 8.3: five days at 7 degC give 5 vernalization days,
    /// while five days at 10 degC - or at -0.5 degC - give only 2.5.
    ///
    /// That is the one claim in this block whose expected values were written down by
    /// the source rather than derived here, so it verifies the equation was READ
    /// correctly rather than merely copied plausibly. The two half-value cases sit on
    /// opposite ramps, so a single mis-transcribed cardinal breaks one of them.
    /// Mirrors `test_vernalization_day_reproduces_the_sources_own_worked_example`.
    #[test]
    fn vernalization_day_reproduces_the_sources_own_worked_example() {
        let c = (-1.0, 0.0, 8.0, 12.0);
        assert_eq!(5.0 * vernalization_day(7.0, c.0, c.1, c.2, c.3), 5.0);
        assert_eq!(5.0 * vernalization_day(10.0, c.0, c.1, c.2, c.3), 2.5);
        assert_eq!(5.0 * vernalization_day(-0.5, c.0, c.1, c.2, c.3), 2.5);
    }

    /// `vernalization_factor` (VERFUN) - Eqn 8.6, and why its clamp is LOAD-BEARING.
    ///
    /// With the cited winter-Europe values `vsen*vdsat = 0.033 x 50 = 1.65 > 1`, so the
    /// unclamped expression is `1 - 1.65 = -0.65` at zero accumulated cold. Clamping
    /// makes this cultivar QUALITATIVE in the source's terms (Fig. 8.2): development is
    /// fully ARRESTED, not merely slowed, until the break-even
    /// `CUMVER = vdsat - 1/vsen = 50 - 30.303... = 19.697` days accrue. Without the clamp
    /// the rate goes NEGATIVE and thermal time runs backwards.
    ///
    /// ⚠ Measured: removing the clamp reddens nothing in the binary, and the clamp
    /// is live (20 tests enter it with a raw value outside [0, 1]).
    /// Mirrors `test_vernalization_factor_is_qualitative_for_winter_europe_wheat` and
    /// `test_a_quantitative_cultivar_never_reaches_the_clamp`.
    #[test]
    fn vernalization_factor_arrests_a_qualitative_cultivar_and_saturates_at_one() {
        let (vsen, vdsat) = (0.033, 50.0);
        assert!(vsen * vdsat > 1.0, "the cited cultivar must be qualitative");
        assert_eq!(vernalization_factor(0.0, vsen, vdsat), 0.0);
        // Break-even: 1 - vsen*(vdsat - c) = 0  =>  c = vdsat - 1/vsen = 19.696969...
        let breakeven = vdsat - 1.0 / vsen;
        assert!(
            (breakeven - 19.697).abs() <= 1.0e-3,
            "break-even {breakeven}"
        );
        assert_eq!(vernalization_factor(breakeven - 0.1, vsen, vdsat), 0.0);
        assert!(vernalization_factor(breakeven + 0.1, vsen, vdsat) > 0.0);
        // Saturation: at and beyond vdsat it is exactly 1 and STAYS there.
        assert_eq!(vernalization_factor(vdsat, vsen, vdsat), 1.0);
        assert_eq!(vernalization_factor(vdsat + 500.0, vsen, vdsat), 1.0);
        // Mid-curve, hand-computed: 1 - 0.033*(50 - 30) = 1 - 0.66 = 0.34.
        let mid = vernalization_factor(30.0, vsen, vdsat);
        assert!((mid - 0.34).abs() <= 1.0e-12, "verfun(30) = {mid}");
        // The OTHER branch of Fig. 8.2: a quantitative cultivar (vsen*vdsat < 1) never
        // reaches the clamp - 1 - 0.003*50 = 0.85 with no cold at all.
        let quantitative = vernalization_factor(0.0, 0.003, 50.0);
        assert!(
            (quantitative - 0.85).abs() <= 1.0e-12,
            "quantitative verfun(0) = {quantitative}"
        );
        // Monotone non-decreasing in accumulated cold, and bounded.
        let vs: Vec<f64> = (0..140)
            .map(|c| vernalization_factor(c as f64 * 0.5, vsen, vdsat))
            .collect();
        assert!(vs.windows(2).all(|w| w[0] <= w[1]), "{vs:?}");
        assert!(vs.iter().all(|v| (0.0..=1.0).contains(v)));
    }

    /// `photoperiod_factor` (PPFUN) - Eqn 7.6 in its LONG-DAY form, the one wheat uses.
    ///
    /// Committed winter-Europe values (Table 7.2): critical photoperiod `cpp = 16` h,
    /// sensitivity `ppsen = 0.09` per hour. Hand-computed pins:
    ///   * 14.53 h (midsummer at 52 degN): 1 - 0.09*(16 - 14.53) = 1 - 0.1323 = 0.8677
    ///   * 8 h: 1 - 0.09*8 = 1 - 0.72 = 0.28
    ///   * 5 h: 1 - 0.09*11 = 1 - 0.99 = 0.01
    ///
    /// ⚠ The DIRECTION is the whole content: a long-day crop is slowed by SHORT
    /// days, so the factor rises with daylength and is 1 at/above `cpp`. Swapping the
    /// comparison turns this into a short-day crop and reddens only three unrelated
    /// bands. Mirrors `test_photoperiod_factor_matches_hand_computed_literals`.
    #[test]
    fn photoperiod_factor_is_the_long_day_response_and_never_goes_negative() {
        let (cpp, ppsen) = (16.0, 0.09);
        assert_eq!(photoperiod_factor(16.0, cpp, ppsen), 1.0); // AT cpp - no slowdown
        assert_eq!(photoperiod_factor(20.0, cpp, ppsen), 1.0); // above - never > 1
        for (hours, expected) in [(14.53, 0.8677), (8.0, 0.28), (5.0, 0.01)] {
            let got = photoperiod_factor(hours, cpp, ppsen);
            assert!(
                (got - expected).abs() <= 1.0e-12,
                "ppfun({hours} h) = {got}, hand-computed {expected}"
            );
        }
        // The source is explicit that a negative ppfun becomes zero, because
        // development is a forward-only process. At cpp = 16 and ppsen = 0.09 the
        // unclamped value at zero daylength is 1 - 1.44 = -0.44.
        assert!(
            1.0 - ppsen * cpp < 0.0,
            "the clamp must have something to clamp"
        );
        assert_eq!(photoperiod_factor(0.0, cpp, ppsen), 0.0);
        // ...and with a steeper sensitivity, far more negative: 1 - 0.2*16 = -2.2.
        assert_eq!(photoperiod_factor(0.0, 16.0, 0.2), 0.0);
        // Monotone non-decreasing in daylength, bounded on [0, 1].
        let vs: Vec<f64> = (0..97)
            .map(|h| photoperiod_factor(h as f64 * 0.25, cpp, ppsen))
            .collect();
        assert!(vs.windows(2).all(|w| w[0] <= w[1]), "{vs:?}");
        assert!(vs.iter().all(|v| (0.0..=1.0).contains(v)));
    }

    /// ⚠ **The structural difference between the two vegetative modifiers**, and
    /// the property that identified which mechanism the PCSE oracle was using.
    ///
    /// Vernalization reads an ACCUMULATOR: once saturated it is pinned at 1 and cannot
    /// fall again whatever the weather does afterwards - it has memory. Photoperiod
    /// reads an INSTANTANEOUS driver: the same daylength always gives the same factor,
    /// and a later shorter day drops it back down - no ratchet. A trajectory whose
    /// development multiplier keeps climbing after the cold requirement is met therefore
    /// cannot be vernalization-driven.
    ///
    /// This is a claim about the two forms TOGETHER, which is why it is one test rather
    /// than a line in each. Mirrors
    /// `test_vernalization_has_memory_and_photoperiod_does_not`.
    #[test]
    fn vernalization_has_memory_and_photoperiod_does_not() {
        let (vsen, vdsat) = (0.033, 50.0);
        assert_eq!(vernalization_factor(vdsat, vsen, vdsat), 1.0);
        assert_eq!(vernalization_factor(vdsat * 3.0, vsen, vdsat), 1.0);
        let (cpp, ppsen) = (16.0, 0.09);
        let long_day = photoperiod_factor(15.0, cpp, ppsen);
        let short_day = photoperiod_factor(8.0, cpp, ppsen);
        assert!(short_day < long_day, "{short_day} !< {long_day}");
        assert_eq!(photoperiod_factor(15.0, cpp, ppsen), long_day);
    }

    /// `WSFD` breaks BOTH patterns its two neighbours set, and that is the reason it is
    /// easy to "fix" into consistency with them.
    ///
    /// `verfun` and `ppfun` are limitation factors bounded above by 1; `WSFD` is a ratio
    /// on `[0, 1 + WSSD]` and drought HASTENS development ([F] Table 15.2). Written
    /// against the other two functions rather than as a bare inequality, because the
    /// claim IS the comparison. Mirrors `test_wsfd_may_exceed_one_unlike_its_two_
    /// neighbours` and `test_wsfd_is_monotone_increasing_as_water_runs_out`.
    #[test]
    fn wsfd_may_exceed_one_unlike_the_two_vegetative_modifiers() {
        assert!(drought_development_factor(0.0, 0.4) > 1.0);
        assert!(vernalization_factor(0.0, 0.01, 50.0) <= 1.0);
        assert!(photoperiod_factor(0.0, 16.0, 0.09) <= 1.0);
        // Monotone INCREASING as the water runs out (WSFG falls from 1 to 0).
        let factors: Vec<f64> = (0..=10)
            .rev()
            .map(|w| drought_development_factor(w as f64 / 10.0, 0.4))
            .collect();
        assert!(
            factors.windows(2).all(|w| w[0] <= w[1]),
            "drought must accelerate monotonically: {factors:?}"
        );
    }

    // --- S5 batch C: the water equations ------------------------------------------
    //
    // ⚠ THE BEFORE-BATTERY THIS BLOCK ANSWERS. Sixteen mutations of the water science
    // were run against `cargo test -p domains --lib` (221 passed) BEFORE any of these
    // tests existed. Exactly ONE reddened a test whose subject was the mutated mechanism
    // (`a_dry_subsoil_stops_extension`, below). Eight reddened NOTHING AT ALL: the
    // analytic SVP slope, the Penman–Monteith canopy-resistance term, its negative-energy
    // clamp, the zero-capacity limb of `FTSW`, both guards on `root_zone_fraction`, the
    // re-sow return's zero-depth limb, and a doubled condensation rate. The rest reddened
    // strangers — dropping the water-stress factor from transpiration outright moved one
    // test, and that one is about drought-accelerated phenology.
    // Record: `docs/plans/post-roadmap-reference-flip.md` §5ag.

    /// `e_s(T)` against FAO-56's own table, and `Δ` alongside it.
    ///
    /// ⚠ SCOPE, STATED HONESTLY (the Python file states it too and it survives the port):
    /// the `e_s` magnitudes are an external cross-check — FAO-56 tabulates
    /// `e_s(20 °C) = 2.339 kPa` and this tree computes it from Tetens' constants. The
    /// SLOPE literals are only formula-consistent (FAO-56's 4098-form and this module's
    /// analytic `B·C` derivative agree to these digits), so they are not independent
    /// evidence; the independent slope check is the finite difference below.
    /// Mirrors `tests/test_transpiration.py::test_svp_and_slope_match_fao56_table`.
    #[test]
    fn saturation_vapour_pressure_and_its_slope_match_the_fao56_table() {
        for (temp_c, es_kpa, slope_kpa) in [
            (0.0, 0.6108, 0.04445),
            (10.0, 1.2280, 0.08228),
            (20.0, 2.3383, 0.14474),
            (30.0, 4.2431, 0.24336),
        ] {
            let es = saturation_vapor_pressure(temp_c) / 1000.0;
            let slope = slope_svp(temp_c) / 1000.0;
            assert!(
                (es - es_kpa).abs() <= 2e-3 * es_kpa,
                "e_s({temp_c}) = {es} kPa, FAO-56 says {es_kpa}"
            );
            assert!(
                (slope - slope_kpa).abs() <= 2e-3 * slope_kpa,
                "slope({temp_c}) = {slope} kPa/C, want {slope_kpa}"
            );
        }
    }

    /// `Δ` IS the derivative of `e_s`, checked against a central difference rather than
    /// against a second transcription of the same closed form.
    ///
    /// This is the one genuinely independent check on the slope: it would redden for a
    /// dropped or transposed factor even if the table literals above were themselves
    /// copied out of this tree. Measured: the before-battery's `slope_svp` mutation
    /// (dropping the `SVP_C` factor) reddened NOTHING in the whole lib binary.
    /// Mirrors `test_slope_is_the_analytic_derivative_of_svp`.
    #[test]
    fn the_svp_slope_is_the_analytic_derivative_of_the_curve() {
        let (t, h) = (18.0, 1e-4);
        let numeric =
            (saturation_vapor_pressure(t + h) - saturation_vapor_pressure(t - h)) / (2.0 * h);
        let analytic = slope_svp(t);
        assert!(
            (analytic - numeric).abs() <= 1e-6 * numeric.abs(),
            "analytic {analytic} vs finite difference {numeric}"
        );
    }

    /// The combination equation at one operating point, **hand-composed in the comment**.
    ///
    /// ⚠ The Python test calls its `6.158958394549651` a "pinned regression literal" — a
    /// value read out of the tree, which S5's exit gate (clause 2) rejects. The same
    /// number is legitimate as a HAND computation, so the derivation is written out here
    /// and each intermediate is asserted, which is what makes it re-checkable without
    /// running anything:
    ///
    /// ```text
    ///   T = 20 °C, Rn = 200 W/m2, VPD = 1000 Pa, r_a = 50 s/m, r_s = 70 s/m, G = 0
    ///   e_s(20)  = 610.8 · exp(17.27·20/257.3)        = 2338.2813 Pa
    ///   Δ        = 17.27 · 237.3 · e_s / (257.3)^2    =  144.7462 Pa/°C
    ///   aero     = ρ·c_p·VPD/r_a = 1.205·1013·1000/50 = 24413.3 W/m2
    ///   denom    = Δ + γ(1 + r_s/r_a) = 144.7462 + 67·2.4 = 305.5462
    ///   λE       = (Δ·Rn + aero)/denom = (28949.25 + 24413.3)/305.5462 = 174.6464 W/m2
    ///   T_pot    = λE/λ · 86400 = 174.6464/2.45e6 · 86400 = 6.1590 mm/day
    /// ```
    ///
    /// Mirrors `test_penman_monteith_pinned_value`.
    #[test]
    fn penman_monteith_is_the_hand_composed_combination_equation() {
        let (ra, rs) = (50.0, 70.0);
        // Each intermediate on its own, so a wrong answer names its own factor.
        let es = saturation_vapor_pressure(20.0);
        assert!((es - 2338.2813).abs() < 1e-3, "e_s(20) = {es}");
        let delta = slope_svp(20.0);
        assert!((delta - 144.7462).abs() < 1e-3, "delta = {delta}");
        let aero: f64 = 1.205 * 1013.0 * 1000.0 / ra;
        assert!((aero - 24413.3).abs() < 1e-9, "aero = {aero}");
        let denom = delta + 67.0 * (1.0 + rs / ra);
        assert!((denom - 305.5462).abs() < 1e-3, "denom = {denom}");
        let latent = (delta * 200.0 + aero) / denom;
        assert!((latent - 174.6464).abs() < 1e-3, "latent = {latent}");
        let want = latent / 2.45e6 * 86400.0;

        let got = penman_monteith_transpiration(200.0, 1000.0, 20.0, ra, rs);
        assert!(
            (got - want).abs() <= 1e-12 * want,
            "PM gave {got}, the hand composition gives {want}"
        );
        // ...and the composition really is ~6.16 mm/day, a realistic summer potential.
        assert!((got - 6.1590).abs() < 1e-3, "{got} mm/day");
        // THE CANOPY RESISTANCE IS LOAD-BEARING. Dropping `(1 + r_s/r_a)` from the
        // denominator reddened nothing in the lib binary before this line; it is a
        // different mechanism (an open water surface), not a rounding difference. The
        // bound is two-sided on the measured 1.4432 rather than a one-sided "is bigger",
        // so a later change that shrinks the term is caught as well as one that drops it.
        let no_canopy = (delta * 200.0 + aero) / (delta + 67.0) / 2.45e6 * 86400.0;
        let ratio = no_canopy / got;
        assert!(
            (1.44..1.45).contains(&ratio),
            "the r_s term must matter: {no_canopy} vs {got} (ratio {ratio})"
        );
    }

    /// No available energy and no vapour deficit ⇒ no evaporative demand, exactly.
    /// Mirrors `test_penman_monteith_zero_energy_zero_vpd_is_zero`.
    #[test]
    fn penman_monteith_is_zero_with_no_energy_and_no_vapour_deficit() {
        assert_eq!(
            penman_monteith_transpiration(0.0, 0.0, 20.0, 50.0, 70.0),
            0.0
        );
    }

    /// The negative-energy clamp — **and the finding that nothing in this tree can reach
    /// it**.
    ///
    /// The Python test justifies the clamp by saying daily-average net radiation "goes
    /// negative on short midwinter days (the winter-wheat season overwinters)". Measured
    /// on the Rust side and that is FALSE HERE, for a structural reason rather than a
    /// mild-winter one: `weather::net_radiation` is net SHORTWAVE only,
    /// `(1 − α)·IRRAD/86400`, with no longwave-loss term, so it is non-negative for every
    /// non-negative irradiance — and `vapor_pressure_deficit` is itself `max(0, …)`. Both
    /// drivers of `λE` are therefore non-negative at every call site in the tree.
    ///
    /// ⚠ Measured, not reasoned: replacing the clamp with a `panic!` on a negative
    /// `mm/day` left `cargo test --workspace --no-fail-fast` fully green, goldens
    /// included. The clamp is not merely untested, it is UNREACHABLE from every scenario.
    /// It is pinned here at the function's own contract (it is `pub`, and a longwave term
    /// is the obvious next weather science), and the unreachability is asserted rather
    /// than left as a comment that could rot.
    /// Mirrors `test_penman_monteith_clamps_negative_radiation_to_zero`.
    #[test]
    fn penman_monteith_clamps_negative_available_energy_and_no_weather_row_can_reach_it() {
        // (a) The clamp itself, on the function's own contract. Unclamped this operating
        // point gives −0.4705 mm/day — a sink flipped into a deposit.
        let (ra, rs) = (50.0, 70.0);
        assert_eq!(penman_monteith_transpiration(-80.0, 50.0, 2.0, ra, rs), 0.0);
        let delta = slope_svp(2.0);
        let unclamped = (delta * -80.0 + 1.205 * 1013.0 * 50.0 / ra)
            / (delta + 67.0 * (1.0 + rs / ra))
            / 2.45e6
            * 86400.0;
        assert!(
            unclamped < 0.0,
            "the fixture must actually be a clamped case: {unclamped}"
        );
        assert!((unclamped + 0.4705).abs() < 1e-3, "{unclamped} mm/day");

        // (b) THE UNREACHABILITY, asserted over the committed weather rather than argued.
        let (_latitude, rows) =
            super::super::weather::read_weather_facts(super::super::weather::WEATHER_FIXTURE)
                .expect("the committed weather fixture");
        assert!(rows.len() > 300, "the fixture must be a whole season");
        for row in &rows {
            let rn = super::super::weather::net_radiation(row.irrad_j_m2_day);
            assert!(rn >= 0.0, "net radiation went negative: {rn}");
            let vpd = super::super::weather::vapor_pressure_deficit(row.temp_c, row.vap_hpa);
            assert!(vpd >= 0.0, "VPD went negative: {vpd}");
            assert!(
                penman_monteith_transpiration(rn, vpd, row.temp_c, ra, rs) > 0.0,
                "day {} demanded nothing at all",
                row.day_of_year
            );
        }
    }

    /// `TTSW = DEPORT · EXTR · ρ · A` ([F] Eqn 14.6), and the identity with
    /// `captured_water` that makes a season a closed cycle.
    ///
    /// The two names price different things — a newly explored SLAB versus the whole
    /// COLUMN — and the re-sow return uses one where the stress denominator uses the
    /// other. They must be the same arithmetic or water appears and disappears at a
    /// re-sow. ⚠ The before-battery's `ground_area` mutation reddened three tests, only
    /// one of which is about the area factor.
    /// Mirrors `test_transpirable_capacity_is_the_column_arithmetic` and
    /// `test_transpirable_capacity_agrees_with_captured_water`.
    #[test]
    fn transpirable_capacity_is_the_column_arithmetic_and_agrees_with_captured_water() {
        // 1 m at EXTR 0.13 over 1 m2 = 130 kg; a fraction of the depth over twice the
        // plot is 39 kg, which is linear in BOTH factors at once.
        assert_eq!(transpirable_capacity(1.0, 0.13, 1.0), 130.0);
        let thirty_nine = transpirable_capacity(0.15, 0.13, 2.0);
        assert!((thirty_nine - 39.0).abs() < 1e-12, "{thirty_nine}");
        assert_eq!(transpirable_capacity(0.0, 0.13, 1.0), 0.0);
        // The anti-drift identity, to the BIT (not `approx`): the same product.
        for depth in [0.15, 0.4, 1.3] {
            for area in [1.0, 2.5] {
                assert_eq!(
                    transpirable_capacity(depth, 0.13, area),
                    captured_water(depth, 0.13, area),
                    "the two names for one product drifted at depth {depth}, area {area}"
                );
            }
        }
    }

    /// `FTSW = ATSW / TTSW` ([F] Eqn 14.7) — the cardinals, and the two things it
    /// deliberately does NOT do.
    ///
    /// It is not clamped above 1 (an over-filled zone is a real state, the one `Drainage`
    /// relieves), and a zero capacity returns 0.0 — maximally stressed — rather than
    /// raising or returning 1. ⚠ The zero-capacity limb is UNREACHABLE from every
    /// scenario and every golden (measured with a `panic!` probe under
    /// `cargo test --workspace`): `rooted_depth0` is a cited positive everywhere, so no
    /// run ever asks. Flipping its return to 1.0 — "a crop with no root zone is
    /// unstressed" — reddened nothing at all before this test.
    /// Mirrors `test_fraction_transpirable_is_atsw_over_ttsw` and
    /// `test_fraction_transpirable_returns_zero_for_zero_capacity`.
    #[test]
    fn fraction_transpirable_is_atsw_over_ttsw_and_is_not_clamped_above_one() {
        for (soil_water, expected) in [
            (0.0, 0.0),   // empty root zone
            (39.0, 0.30), // exactly at the WSSG threshold
            (65.0, 0.50), // half the capacity
            (130.0, 1.0), // the drained upper limit
            (260.0, 2.0), // ⚠ NOT clamped: over-filled is a real, reportable state
        ] {
            let got = fraction_transpirable(soil_water, 130.0);
            assert!(
                (got - expected).abs() <= 1e-12 * expected.max(1.0),
                "FTSW({soil_water}) = {got}, want {expected}"
            );
        }
        // The zero-capacity limb: maximally stressed, not unstressed, and not a panic.
        assert_eq!(fraction_transpirable(5.0, 0.0), 0.0);
        assert_eq!(fraction_transpirable(0.0, 0.0), 0.0);
        // A negative capacity cannot arise, but if it did it must not invert the sign.
        assert_eq!(fraction_transpirable(5.0, -1.0), 0.0);
    }

    /// `WSFG = min(1, FTSW/WSSG)` ([F] Eqn 15.3, Box 14.1) — the ramp, the cap, and the
    /// absence of a wilting floor.
    ///
    /// Both ends are load-bearing and neither was covered. **The cap**: without it an
    /// over-filled zone manufactures growth, and the before-battery shows removing it
    /// reddens fifteen tests, EVERY ONE of them a compensation-point or leaf-cycle gate —
    /// "a number moved" wearing a behavioural name. **The floor**: [F]'s form reaches zero
    /// only AT `FTSW = 0`, so the shutoff is asymptotic; reinstating a hard floor at 0.05
    /// reddened exactly one stranger. The absence is a deliberate rule, not an oversight —
    /// it is what the absolute-kg ramp this replaced used to provide.
    /// Mirrors `test_water_stress_factor_cardinal_values` and
    /// `test_water_stress_factor_has_no_wilting_floor`.
    #[test]
    fn water_stress_factor_is_the_wssg_ramp_with_a_cap_and_no_wilting_floor() {
        let wssg = 0.30;
        for (ftsw, expected) in [
            (0.0, 0.0),    // no transpirable water left
            (0.075, 0.25), // a quarter of the way up the ramp
            (0.15, 0.5),   // half
            (0.30, 1.0),   // at the threshold
            (0.85, 1.0),   // above it: unstressed
            (1.30, 1.0),   // over-filled is still just unstressed, never > 1
            (7.00, 1.0),   // ...however over-filled
        ] {
            let got = water_stress_factor(ftsw, wssg);
            assert!(
                (got - expected).abs() <= 1e-12 * expected.max(1.0),
                "WSFG({ftsw}) = {got}, want {expected}"
            );
        }
        // NO WILTING FLOOR: positive everywhere above zero, exactly zero only at zero.
        assert!(water_stress_factor(1e-12, wssg) > 0.0);
        assert_eq!(water_stress_factor(0.0, wssg), 0.0);
        // ⚠ The empty-zone limb is UNREACHABLE from every scenario and golden (measured
        // with a `panic!` probe under `cargo test --workspace`), so this is the only
        // thing in the tree that enters it.
        assert_eq!(water_stress_factor(-1.0, wssg), 0.0);
    }

    /// `soil_water_stress` composes the three, and the composition's own property: a root
    /// zone at the drained upper limit is unstressed AT EVERY DEPTH.
    ///
    /// That property is why re-basing the stores on geometry was safe — a shallow zone is
    /// not a dry one — and the absolute-kg band it replaced could not express it (it read
    /// a full 19.5 kg zone as below wilting, which killed every sealed chamber; measured
    /// before the form was written). Also pins the `ground_area` factor in the
    /// DENOMINATOR: hardcoding it to 1.0 there reddened two tests, neither of which is
    /// about the stress path.
    /// Mirrors `test_soil_water_stress_composes_the_three` and
    /// `test_a_full_root_zone_is_unstressed_at_every_depth`.
    #[test]
    fn soil_water_stress_composes_the_three_and_a_full_zone_is_unstressed_at_any_depth() {
        let (extr, wssg) = (0.13, 0.30);
        // 39 kg in a 1.0 m zone over 1 m2 is FTSW 0.30 == WSSG, i.e. exactly unstressed.
        assert_eq!(soil_water_stress(39.0, 1.0, extr, 1.0, wssg), 1.0);
        // 19.5 kg is FTSW 0.15, half the ramp.
        let half = soil_water_stress(19.5, 1.0, extr, 1.0, wssg);
        assert!((half - 0.5).abs() <= 1e-12, "{half}");
        // The area is in the denominator: the SAME water over twice the plot is half as
        // full in FTSW terms, so it is half as unstressed.
        let wide = soil_water_stress(39.0, 1.0, extr, 2.0, wssg);
        assert!((wide - 0.5).abs() <= 1e-12, "{wide}");
        // FTSW0 = MAI independent of depth, for a zone at the drained upper limit.
        for depth in [0.15, 0.3, 0.75, 1.3] {
            for area in [1.0, 3.0] {
                let full = transpirable_capacity(depth, extr, area);
                assert_eq!(
                    soil_water_stress(full, depth, extr, area, wssg),
                    1.0,
                    "a full {depth} m zone over {area} m2 read as stressed"
                );
            }
        }
    }

    /// `FROOT1` — the root-zone access gate ([E] p. 136; it is NOT a function of root
    /// mass), a clamped ratio that can only reduce a supply, never reverse or amplify it.
    ///
    /// ⚠ BOTH of its guards were unreachable before this test. Removing the clamp at 1
    /// (so a deep crop manufactures nitrogen) and removing the non-positive-depth guard
    /// (so a negative depth reverses the flow) each reddened NOTHING in the lib binary.
    /// The saturation branch itself is live — a `panic!` in it stops 20 tests — but
    /// nothing was checking what it returns.
    /// Mirrors `test_root_zone_fraction_is_a_clamped_ratio` and
    /// `test_root_zone_fraction_can_only_reduce_never_reverse`.
    #[test]
    fn root_zone_fraction_is_a_clamped_ratio_that_can_only_reduce() {
        for (depth, layer, expected) in [
            (0.0, 0.30, 0.0),  // a sown seed reaches nothing
            (-1.0, 0.30, 0.0), // defensive: never negative
            (0.15, 0.30, 0.5), // half the layer
            (0.30, 0.30, 1.0), // exactly the layer
            (1.30, 0.30, 1.0), // deeper than the layer still saturates at 1
        ] {
            assert_eq!(
                root_zone_fraction(depth, layer),
                expected,
                "FROOT1({depth}, {layer})"
            );
        }
        // It multiplies a SUPPLY term, so it must live in [0, 1] for every input a run
        // could hand it: > 1 manufactures nitrogen, < 0 reverses the flow.
        for depth in [0.0, 1e-9, 0.05, 0.3, 1.3, 1e6] {
            let f = root_zone_fraction(depth, 0.30);
            assert!((0.0..=1.0).contains(&f), "FROOT1({depth}) = {f}");
        }
    }

    /// The re-sow return's own arithmetic, and its two degenerate limbs.
    ///
    /// The season-level properties (a redistribution, `FTSW` preserved, the fraction rule)
    /// are pinned in `system.rs`'s `the_resow_returns_the_abandoned_fraction_and_preserves
    /// _ftsw`. What is left here is the function's contract on inputs no season produces:
    /// a zero old depth (divide by zero) and a zone that abandoned nothing. ⚠ Both limbs
    /// are UNREACHABLE from every scenario and golden — measured with `panic!` probes
    /// under `cargo test --workspace`, which stayed fully green — and deleting either
    /// guard reddened nothing in the lib binary.
    #[test]
    fn resow_water_return_is_the_abandoned_fraction_and_its_degenerate_limbs_are_safe() {
        // The rule: the abandoned FRACTION of the water, not the abandoned column at the
        // drained upper limit. A 1.3 m zone holding 100 kg, re-sown to 0.15 m, gives back
        // (1.3 - 0.15)/1.3 = 0.8846... of it.
        let returned = resow_water_return(100.0, 1.3, 0.15);
        let want = 100.0 * (1.3 - 0.15) / 1.3;
        assert!(
            (returned - want).abs() <= 1e-12 * want,
            "{returned} vs {want}"
        );
        // It can never exceed the store — the fraction is < 1 by construction, which is
        // why the rule needs no clamp (the form it replaced did, and the clamp fired
        // every re-sow once the stores became geometric).
        for held in [0.0, 1.0, 1e6] {
            for old in [0.16, 0.5, 1.3, 40.0] {
                let out = resow_water_return(held, old, 0.15);
                assert!(out <= held, "returned {out} from a store of {held}");
                assert!(out >= 0.0, "returned a negative {out}");
            }
        }
        // Limb 1: a zero (or negative) old depth returns nothing rather than NaN.
        assert_eq!(resow_water_return(100.0, 0.0, 0.15), 0.0);
        assert_eq!(resow_water_return(100.0, -1.0, 0.15), 0.0);
        // Limb 2: a zone that abandoned nothing (or grew) gives back nothing rather than
        // a negative — which would run the transfer backwards into the root zone.
        assert_eq!(resow_water_return(100.0, 0.15, 0.15), 0.0);
        assert_eq!(resow_water_return(100.0, 0.10, 0.15), 0.0);
    }

    // --- batch D: the carbon-spending equations -----------------------------
    //
    // ⚠ THE PROVENANCE OF EVERY NUMBER IN THIS BLOCK, stated once. The respiration
    // scalars (`m_ref = 0.02`, `Q10 = 2`, `t_ref = 25`, `Yg = 0.75`) and the whole
    // partition table are `TODO(cite)` PROVISIONAL PLACEHOLDERS in their own param
    // files, and the partition table is additionally a FITTED one — the winter-wheat
    // backfill from the cited Table 18 was taken and REFUSED because it drives peak LAI
    // to 2.201 against a 5.0–8.0 contract band
    // (docs/plans/post-roadmap-wheat-partition-backfill.md). So S5's exit-gate clause 2
    // is satisfied here by its MIDDLE limb only: every expected value below is
    // hand-computed from the stated equation and the committed inputs, with the
    // arithmetic written out, and NONE is presented as a number a source states. Pinning
    // a fitted value as though it were cited is the overclaim species that batch A's and
    // batch C's reviews each caught once.

    fn resp_params() -> RespirationParams {
        RespirationParams {
            maintenance_coef: 0.02,
            q10: 2.0,
            t_ref: 25.0,
            growth_efficiency: 0.75,
            o2_half_saturation: 0.001, // inert here: the rate laws never touch O₂
        }
    }

    /// The committed `allocation.yaml` table, as a literal.
    ///
    /// ⚠ Deliberately NOT `params::allocation().table` — these tests are about the
    /// interpolation ARITHMETIC, so reading the table through the loader would make
    /// every expected value below depend on the file, and a table edit would then redden
    /// the equation tests rather than the value pin that owns it
    /// (`params::tests::every_value_matches_the_generated_table`). Two gates, two
    /// subjects.
    fn table() -> Vec<PartitionRow> {
        vec![
            PartitionRow {
                dvs: 0.0,
                fl: 0.55,
                fs: 0.10,
                fr: 0.35,
                fo: 0.00,
            },
            PartitionRow {
                dvs: 1.0,
                fl: 0.30,
                fs: 0.50,
                fr: 0.20,
                fo: 0.00,
            },
            PartitionRow {
                dvs: 2.0,
                fl: 0.00,
                fs: 0.10,
                fr: 0.10,
                fo: 0.80,
            },
        ]
    }

    fn close(a: f64, b: f64, what: &str) {
        assert!(
            (a - b).abs() <= 1e-12 * b.abs().max(1.0),
            "{what}: got {a}, want {b}"
        );
    }

    /// `Q10^((T − t_ref)/10)` on the doubling ladder either side of the reference.
    ///
    /// Hand-computed from the exponent alone: at `t_ref` the exponent is 0 so the factor
    /// is exactly 1; each +10 °C multiplies by `q10` and each −10 divides by it. With the
    /// committed `q10 = 2`, `t_ref = 25` that is 1, 2, 4, ½, ¼ at 25, 35, 45, 15, 5 °C.
    ///
    /// ⚠ §5ad's control M3 measured this function BARE: changing the per-10 °C exponent
    /// to per-5 reddened 6 tests and NOT ONE of them was about temperature response.
    /// This is the test that was missing.
    /// Mirrors `tests/test_respiration.py::test_q10_factor_known_values`.
    #[test]
    fn q10_is_the_doubling_ladder_either_side_of_the_reference_temperature() {
        for (temp, want) in [
            (25.0, 1.0),
            (35.0, 2.0),
            (45.0, 4.0),
            (15.0, 0.5),
            (5.0, 0.25),
        ] {
            close(
                q10_factor(temp, 2.0, 25.0),
                want,
                &format!("q10 at {temp} °C"),
            );
        }
        // The reference temperature is a REFERENCE, not a floor: below it the factor
        // is < 1 and never clamps. A `max(0, …)` here would be invisible above t_ref.
        assert!(q10_factor(-5.0, 2.0, 25.0) < 1.0);
        // It is a pure exponential in the DIFFERENCE, so shifting both ends by the same
        // amount is exactly identical — which is what makes `t_ref` a real parameter
        // rather than a constant that happens to be passed in.
        assert_eq!(q10_factor(35.0, 2.0, 25.0), q10_factor(45.0, 2.0, 35.0));
    }

    /// `MRES = m_ref · biomass · Q10 · maturity`, hand-composed.
    ///
    /// At the reference temperature Q10 is exactly 1, so `0.02 · 5 = 0.1` mol C/day; at
    /// 35 °C the Q10 doubles it to 0.2 and at 15 °C halves it to 0.05. Linear in biomass
    /// by inspection of the product, and exactly zero for no tissue — positivity here is
    /// structural rather than clamped, which is why the zero is an EXACT equality.
    ///
    /// ⚠ `maturity` is hard-coded to 1.0 in this port and Python's optional argument gets
    /// no successor. It is not drift: nothing in EITHER tree ever passes anything but the
    /// default, so that seam was exercised by its own test and by nothing else. Recorded
    /// in batch D's disposition list rather than ported.
    /// Mirrors the `maintenance_respiration_*` block of `tests/test_respiration.py`.
    #[test]
    fn maintenance_respiration_is_the_reference_rate_scaled_by_biomass_and_q10() {
        let p = resp_params();
        close(
            maintenance_respiration_flux(5.0, 25.0, &p),
            0.1,
            "MRES at t_ref",
        );
        close(
            maintenance_respiration_flux(5.0, 35.0, &p),
            0.2,
            "MRES at +10 °C",
        );
        close(
            maintenance_respiration_flux(5.0, 15.0, &p),
            0.05,
            "MRES at −10 °C",
        );
        // Linear in biomass at a fixed temperature: twice the tissue, twice the cost.
        let a = maintenance_respiration_flux(3.0, 20.0, &p);
        let b = maintenance_respiration_flux(6.0, 20.0, &p);
        close(b, 2.0 * a, "MRES doubles with biomass");
        // No tissue, no cost — exactly, at any temperature.
        assert_eq!(maintenance_respiration_flux(0.0, 30.0, &p), 0.0);
        assert_eq!(maintenance_respiration_flux(0.0, -10.0, &p), 0.0);
    }

    /// `available = max(0, GASS − MRES)`, and the clamp is the whole point.
    ///
    /// Hand values: `1.0 − 0.2 = 0.8`. The clamp limb is asserted as an EXACT zero at
    /// both the strict-deficit and the exactly-equal case, because a `>` written where
    /// `>=` belongs is invisible everywhere except at equality.
    ///
    /// ⚠ Why the clamp is load-bearing rather than defensive: `Allocation` multiplies
    /// this by `Yg` and `GrowthRespiration` by `(1 − Yg)`, so a NEGATIVE `available`
    /// runs both flows BACKWARDS — the plant would deposit carbon into the atmosphere
    /// and un-respire. §5ad's battery measured the unclamped form reddening 21 tests,
    /// none of them about the carbon budget.
    /// Mirrors `tests/test_respiration.py::test_growth_respiration_clamps_*`.
    #[test]
    fn available_for_growth_is_the_difference_and_the_clamp_is_exact_at_equality() {
        close(available_for_growth(1.0, 0.2), 0.8, "surplus day");
        assert_eq!(available_for_growth(0.2, 1.0), 0.0);
        assert_eq!(available_for_growth(1.0, 1.0), 0.0);
        // The composed growth-respiration loss Python returns from its own function:
        // (1 − Yg) · available = 0.25 · 0.8 = 0.2 mol C/day. Composed here rather than
        // wrapped, because Rust computes it inline in `GrowthRespiration::evaluate`.
        close(
            (1.0 - resp_params().growth_efficiency) * available_for_growth(1.0, 0.2),
            0.2,
            "GRES on the surplus day",
        );
    }

    /// The three knots are the table's own rows, returned unchanged.
    ///
    /// Mirrors `tests/test_allocation.py::test_partition_fractions_known_values`.
    #[test]
    fn partition_fractions_at_a_knot_are_that_row_verbatim() {
        let t = table();
        for row in &t {
            let got = partition_fractions(row.dvs, &t);
            assert_eq!(got, (row.fl, row.fs, row.fr, row.fo), "at DVS {}", row.dvs);
        }
    }

    /// Between knots it is a linear interpolation, pinned OFF THE MIDPOINT.
    ///
    /// ⚠⚠ THE OFF-MIDPOINT PART IS THE TEST, not a flourish. At DVS 0.5 the weight is
    /// 0.5, so a weight computed as `1 − w` instead of `w` — the interpolation running
    /// BACKWARDS, the leaf taking the stem's share — returns the identical answer. A
    /// midpoint pin is symmetric under exactly the mistake it exists to catch. §5ad's
    /// battery measured the reversed weight reddening 2 tests, neither about allocation.
    ///
    /// Hand-derived from `lo + w·(hi − lo)` with `w = (dvs − lo.dvs)/(hi.dvs − lo.dvs)`:
    ///   DVS 0.25, w = 0.25 between rows 0 and 1:
    ///     fl 0.55 + 0.25·(0.30 − 0.55) = 0.55 − 0.0625 = 0.4875
    ///     fs 0.10 + 0.25·(0.50 − 0.10) = 0.10 + 0.10   = 0.20
    ///     fr 0.35 + 0.25·(0.20 − 0.35) = 0.35 − 0.0375 = 0.3125
    ///     fo 0
    ///   DVS 1.75, w = 0.75 between rows 1 and 2:
    ///     fl 0.30 + 0.75·(0.00 − 0.30) = 0.075
    ///     fs 0.50 + 0.75·(0.10 − 0.50) = 0.20
    ///     fr 0.20 + 0.75·(0.10 − 0.20) = 0.125
    ///     fo 0.00 + 0.75·(0.80 − 0.00) = 0.60
    /// Mirrors `tests/test_allocation.py::test_partition_fractions_known_values[0.5|1.5]`.
    #[test]
    fn partition_fractions_between_knots_interpolate_in_the_right_direction() {
        let t = table();
        let (fl, fs, fr, fo) = partition_fractions(0.25, &t);
        close(fl, 0.4875, "fl at DVS 0.25");
        close(fs, 0.20, "fs at DVS 0.25");
        close(fr, 0.3125, "fr at DVS 0.25");
        assert_eq!(fo, 0.0);
        let (fl, fs, fr, fo) = partition_fractions(1.75, &t);
        close(fl, 0.075, "fl at DVS 1.75");
        close(fs, 0.20, "fs at DVS 1.75");
        close(fr, 0.125, "fr at DVS 1.75");
        close(fo, 0.60, "fo at DVS 1.75");
        // The midpoint, for completeness — and it is precisely the case that CANNOT see
        // a reversed weight, which is why it is not left to carry this claim alone.
        let (fl, ..) = partition_fractions(0.5, &t);
        close(fl, 0.425, "fl at the midpoint");
    }

    /// Outside the table the fractions FLAT-EXTRAPOLATE from the nearer END row.
    ///
    /// ⚠ The two limbs must be pinned separately: a top limb returning the FIRST row
    /// instead of the last still sums to 1, still returns a legal set of fractions, and
    /// reddened 2 tests in §5ad's battery — neither about allocation. The below-table
    /// limb is reached by any pre-emergence step, so neither branch is hypothetical.
    /// Mirrors `tests/test_allocation.py::test_partition_fractions_known_values[-1.0|3.0]`.
    #[test]
    fn partition_fractions_flat_extrapolate_from_the_nearer_end_row() {
        let t = table();
        let below = partition_fractions(-1.0, &t);
        assert_eq!(below, (0.55, 0.10, 0.35, 0.00));
        let above = partition_fractions(3.0, &t);
        assert_eq!(above, (0.00, 0.10, 0.10, 0.80));
        assert_ne!(below, above, "the two ends must not extrapolate to one row");
    }

    /// The four fractions sum to 1 at, between and outside every knot.
    ///
    /// A property of the FILE SHAPE rather than of the arithmetic: one shared-breakpoint
    /// table interpolates to sum 1 everywhere by linearity (`lerp(1,1) = 1`), which is
    /// exactly why `allocation.yaml`'s header refuses independent FL/FS/FR/FO tables.
    /// Mirrors `tests/test_allocation.py::test_partition_fractions_sum_to_one_everywhere`.
    #[test]
    fn the_partition_fractions_sum_to_one_everywhere_including_outside_the_table() {
        let t = table();
        for dvs in [-2.0, 0.0, 0.13, 0.5, 0.99, 1.0, 1.37, 2.0, 5.0] {
            let (fl, fs, fr, fo) = partition_fractions(dvs, &t);
            close(fl + fs + fr + fo, 1.0, &format!("sum at DVS {dvs}"));
            for (label, f) in [("fl", fl), ("fs", fs), ("fr", fr), ("fo", fo)] {
                assert!((0.0..=1.0).contains(&f), "{label} at DVS {dvs} is {f}");
            }
        }
    }

    /// `partition` splits a given DMI into four legs that sum back to it exactly.
    ///
    /// Hand values at DVS 0 with DMI 10: `(5.5, 1.0, 3.5, 0.0)`; at DVS 1.5, the midpoint
    /// of rows 1 and 2, `(1.5, 3.0, 1.5, 4.0)`.
    ///
    /// ⚠ The GRAIN leg is asserted by identity, not only through the sum. §5ad's battery
    /// routed the storage share into the ROOT leg — DMI conserved to the last bit, every
    /// flow still balanced — and reddened 2 tests, neither about where the carbon went.
    /// A sum-preserving reshuffle is invisible to conservation BY CONSTRUCTION, which is
    /// the shape of most of this batch.
    /// Mirrors `tests/test_allocation.py::test_partition_splits_dmi_exactly` and
    /// `test_partition_fills_storage_in_the_reproductive_phase`.
    #[test]
    fn partition_splits_the_increment_into_four_named_legs_that_sum_back_to_it() {
        let t = table();
        let (leaf, stem, root, storage) = partition(10.0, 0.0, &t);
        close(leaf, 5.5, "leaf at DVS 0");
        close(stem, 1.0, "stem at DVS 0");
        close(root, 3.5, "root at DVS 0");
        assert_eq!(storage, 0.0, "no grain before anthesis");
        close(
            leaf + stem + root + storage,
            10.0,
            "the four legs sum to DMI",
        );

        let (leaf, stem, root, storage) = partition(10.0, 1.5, &t);
        close(leaf, 1.5, "leaf at DVS 1.5");
        close(stem, 3.0, "stem at DVS 1.5");
        close(root, 1.5, "root at DVS 1.5");
        close(storage, 4.0, "grain at DVS 1.5");
        close(
            leaf + stem + root + storage,
            10.0,
            "the four legs sum to DMI",
        );

        // The grain fills ONLY in the reproductive phase, and the boundary is anthesis.
        for dvs in [0.0, 0.5, 0.99, 1.0] {
            assert_eq!(partition(10.0, dvs, &t).3, 0.0, "grain at DVS {dvs}");
        }
        assert!(
            partition(10.0, 1.01, &t).3 > 0.0,
            "grain starts after anthesis"
        );
        // Zero in, zero out on every leg — the split creates nothing.
        assert_eq!(partition(0.0, 1.5, &t), (0.0, 0.0, 0.0, 0.0));
    }
    // -----------------------------------------------------------------------------
    // S5 batch E — nitrogen: Greenwood's target curve, and one asymmetry the existing
    // availability pin cannot see.
    //
    // Ported from `tests/test_nitrogen_form.py` (the curve block) and
    // `tests/test_nitrogen.py` (the availability block). `f_N` itself needs no successor
    // here — `the_nitrogen_stress_ramp_is_linear_between_its_two_knots` above already
    // pins both knots, the zero-biomass guard and the interior linearity, and a second
    // copy of that claim would inflate the count and assert nothing new.
    //
    // ⚠ `target_n_concentration` had NO direct test in either surface before this batch.
    // Measured: removing the plateau entirely, and flipping the exponent's sign, each
    // reddened ZERO tests of `-p domains --lib` and only committed golden bytes of the
    // full workspace. Greenwood's domain bound is the one form the primary contradicts,
    // and it was guarded by numbers in a file.
    // -----------------------------------------------------------------------------

    /// Greenwood eqn (6) either side of its stated domain bound.
    ///
    /// `%N = a·W^-b` for `W > 1 t/ha` with `a = 5.697 %` (C3) and `b = 0.5`; CONSTANT at
    /// `a` below the bound. The plateau is the paper's own statement rather than our
    /// interpolation — below 1 t/ha growth is near-exponential, so plant %N does not
    /// change with mass (Ågren 1985) — and [A] omits all data there. Extrapolating the
    /// declining branch down into that region manufactures a season-long decline for
    /// crops far too small to have one, which is what makes it the one candidate form the
    /// primary rules out rather than merely leaves unsupported.
    ///
    /// Every literal is hand-computed from the equation: `5.697 / sqrt(4) = 2.8485 %` and
    /// `5.697 / sqrt(16) = 1.424250 %`, i.e. the concentration halves per four-fold mass,
    /// which IS `b = 0.5`.
    /// Mirrors `test_target_is_constant_below_greenwoods_domain_bound`,
    /// `test_target_declines_as_a_power_law_above_the_bound` and
    /// `test_target_is_continuous_at_the_bound`.
    #[test]
    fn the_greenwood_target_is_flat_below_its_domain_bound_and_declines_above_it() {
        let (a, b, bound) = (0.05697, 0.5, 1.0);
        let at = |w: f64| target_n_concentration(w, a, b, bound);
        // The plateau, INCLUDING its right-hand endpoint: the bound itself is flat.
        for w in [0.0, 1e-6, 0.09, 0.35, 0.63, 0.999, 1.0] {
            assert_eq!(at(w), a, "the plateau is flat at W = {w}");
        }
        // The declining branch, at two masses whose square roots are exact.
        assert_eq!(at(4.0), 0.028485);
        assert_eq!(at(16.0), 0.01424250);
        // ...and it really does halve per four-fold mass, which is the exponent.
        assert!((at(16.0) * 2.0 - at(4.0)).abs() <= 1e-18, "b is not 0.5");
        // Strictly DECREASING above the bound — the sign of the exponent, stated as the
        // property rather than as another value.
        let ws = [1.0, 2.0, 5.0, 10.0, 20.0, 50.0];
        let vals: Vec<f64> = ws.iter().map(|w| at(*w)).collect();
        assert!(
            vals.windows(2).all(|w| w[1] < w[0]),
            "the target must fall with crop mass: {vals:?}"
        );
        // No STEP at the bound: the plateau meets the curve at `a`, by construction.
        assert_eq!(at(1.0), a);
        // ⚠ The tolerance is RELATIVE and is not decoration: `(1 + 1e-12)^-0.5` differs
        // from 1 in the 13th place, so the gap here is ~3e-14 in absolute terms. An
        // absolute `1e-15` reads as a tighter claim and is simply a wrong one.
        assert!(
            (at(1.0 + 1e-12) - a).abs() <= 1e-9 * a,
            "the two branches must meet: {} vs {a}",
            at(1.0 + 1e-12)
        );
    }

    /// The crop mass at which Greenwood's target meets the flat stress threshold.
    ///
    /// `W* = (a / n_critical)^(1/b) = (5.697 / 1.5)^2 = 14.4248 t/ha`. Below it the
    /// target sits above critical and `f_N == 1`; above it the plant is stressed at its
    /// own target concentration. The arithmetic is derived here from the COMMITTED params
    /// rather than written as a literal, so a change to either number moves it.
    ///
    /// This is the number `science_gates::open_season_peaks_below_the_greenwood_crossing`
    /// is named for, and that gate asserts the frozen crop stays under it. What it does
    /// NOT assert is that the crossing is where the curve actually crosses — a gate on
    /// `peak_w < 14.4248` stays green if the curve moves out from under the constant. So
    /// this pin is the gate's other half rather than a second copy of it.
    /// Mirrors `test_the_target_meets_n_critical_at_14_42_t_per_ha`.
    #[test]
    fn the_greenwood_target_meets_the_stress_threshold_at_the_crossing() {
        let p = crate::biosphere::params::nitrogen();
        // Back out the flat threshold on Greenwood's own basis (kg N / kg DM).
        let n_critical_kg_kg = p.n_critical_per_mol_c / p.dm_kg_per_mol_c;
        let crossing = (p.n_target_coefficient / n_critical_kg_kg).powf(1.0 / p.n_target_exponent);
        assert!(
            (crossing - 14.4248).abs() < 1.5e-3,
            "the crossing moved: {crossing}"
        );
        // ...and the curve really does cross THERE, in both directions — the assertion a
        // bare literal cannot make.
        let at = |w: f64| {
            target_n_concentration(
                w,
                p.n_target_coefficient,
                p.n_target_exponent,
                p.n_target_w_plateau,
            )
        };
        assert!(at(crossing * 0.99) > n_critical_kg_kg);
        assert!(at(crossing * 1.01) < n_critical_kg_kg);
    }

    /// The non-positive domain bound DEGENERATES here rather than raising, and that is an
    /// inherited port decision, not a lost guard.
    ///
    /// Python's `target_n_concentration` raises on `w_plateau <= 0`; this one returns the
    /// plateau value, for the reason this module's header gives for the three
    /// `canopy_assimilation` guards — a `Result` on a hot rate law buys nothing the file
    /// boundary cannot buy more cheaply. The rejection lives at the loader instead, and
    /// `params::tests::the_nitrogen_bounds_are_each_rejected_at_their_own_shape` is the
    /// test that owns it. Pinned as BEHAVIOUR here so the pair is visible from both ends:
    /// if this function ever grows a panic, this test says so rather than a scenario
    /// dying mid-step.
    /// Mirrors the omitted half of `test_target_rejects_a_non_positive_plateau_bound`.
    #[test]
    fn a_non_positive_domain_bound_degenerates_to_the_plateau_and_does_not_panic() {
        for bad in [0.0, -1.0] {
            assert_eq!(target_n_concentration(2.0, 0.05, 0.5, bad), 0.05);
            assert_eq!(target_n_concentration(1e6, 0.05, 0.5, bad), 0.05);
        }
    }

    /// The soil-N availability ramp at a point that is NOT its own symmetry point.
    ///
    /// ⚠ THIS TEST EXISTS BECAUSE OF A MEASUREMENT, and the measurement is the finding.
    /// `soil_n_below_the_residual_shuts_uptake_off_entirely` above asserts the two knots
    /// and exactly one interior value — the MIDPOINT, `0.5`. The midpoint is a fixed
    /// point of the map `x -> 1 - x`, so **replacing the whole interior ramp with its own
    /// inversion left every test in the workspace green**: the `-p domains --lib` battery
    /// scored zero, and so did the goldens and both tier-contract bands. A branch probe
    /// says why the goldens cannot help — the interior limb is reached by ONE test in the
    /// whole binary (that one), because every frozen scenario sits either below the
    /// residual or above the critical point and never on the ramp.
    ///
    /// So the quarter points are the assertion, not a decoration: `0.25` and `0.75` are
    /// each other's images under the inversion, which is exactly what makes an inverted
    /// ramp visible. The Python side had them all along
    /// (`test_soil_n_availability_cardinal_values` parametrizes six points); the port
    /// carried the midpoint over and lost the discriminating ones.
    /// Mirrors `test_soil_n_availability_cardinal_values`.
    #[test]
    fn the_availability_ramp_is_pinned_off_its_own_symmetry_point() {
        // The band `[0.01, 0.05]` of the Python fixture, whose quarter points are exact
        // in binary: (0.02 - 0.01) / 0.04 = 0.25.
        let (res, crit) = (0.01, 0.05);
        let at = |soil_n: f64| soil_n_availability(soil_n, res, crit);
        assert_eq!(at(0.02), 0.25);
        assert_eq!(at(0.04), 0.75);
        // ...and the ramp is INCREASING through them, which is the property an inversion
        // breaks and a midpoint cannot see.
        assert!(at(0.02) < at(0.03) && at(0.03) < at(0.04));
        // The two clamps, at a band the frozen scenarios do not use, so the pin is about
        // the function rather than about a scenario's numbers.
        assert_eq!(at(0.005), 0.0);
        assert_eq!(at(0.07), 1.0);

        // ⚠ AN INVERTED BAND, asserted as it BEHAVES rather than left as a prose gap.
        // Python's `soil_n_availability` raises on `sn_residual >= sn_critical`; this one
        // does not, per this module's header decision, and until 2026-08-27 nothing
        // downstream rejected it either — `sn_residual`/`sn_critical` are SCENARIO fields,
        // not param-file entries, so unlike `n_residual`/`n_critical` there was no loader to
        // own the rule. S6 gave it one: `system::validate_scenario`, called by
        // `build_season`.
        //
        // ⚠ THE FUNCTION IS UNCHANGED AND THIS PIN STILL BITES — that is the point of
        // keeping it. The guard closes the door scenarios come through; the rate law is
        // still permissive, and what it does with an inverted band is asserted here rather
        // than left to be inferred from the guard's existence. It degenerates to a STEP at
        // `sn_residual`, the interior unreachable because the two conditions overlap.
        // Pinned the way batch D pinned `allocation.yaml`'s two mutations that LOAD, so the
        // validator arriving reads as a decision instead of a guard quietly materializing.
        // ⚠ Independent of the ramp above by construction: the interior limb never runs
        // here, so inverting the ramp cannot move it.
        let inverted = |soil_n: f64| soil_n_availability(soil_n, 0.05, 0.01);
        assert_eq!(inverted(0.03), 0.0); // below the (higher) residual — off
        assert_eq!(inverted(0.05), 0.0); // AT it — still off, the `<=`
        assert_eq!(inverted(0.0500001), 1.0); // and immediately saturated: a step, not a ramp
        assert_eq!(inverted(1e9), 1.0);
    }
    /// `f_N` reads a CONCENTRATION, and the existing pin evaluates at a denominator of 1.
    ///
    /// ⚠ THE SECOND MEASURED BLIND SPOT OF THIS BATCH, and the same species as the
    /// availability midpoint above. `the_nitrogen_stress_ramp_is_linear_between_its_two_knots`
    /// passes `biomass_c = 1.0` on every call it makes, so `plant_n / biomass_c` and
    /// `plant_n` are the same number for all of them: **replacing the concentration with
    /// the bare amount left that test green.** It reddened ten tests elsewhere, and exactly
    /// one of them was about the nitrogen factor — `flows::tests::the_limitation_is_the_
    /// product_and_both_factors_actually_bite`, a flow-level pin one layer out.
    ///
    /// A denominator of one is the arithmetic identity of having no denominator. What this
    /// pin adds is the only thing that separates them: the SAME plant nitrogen against two
    /// different biomasses must give two different answers, and each must be the ramp read
    /// at that state's own concentration — which is what makes `f_N` a dilution factor
    /// rather than a stock threshold.
    #[test]
    fn the_stress_factor_reads_a_concentration_and_not_an_amount() {
        let (res, crit) = (1.0, 3.0);
        // One amount of nitrogen, two crops. The lean crop is at conc 2.0 (mid-ramp);
        // the same nitrogen in a crop twice as large is at 1.0, which is the residual.
        let plant_n = 4.0;
        assert_eq!(nitrogen_stress_factor(plant_n, 2.0, res, crit), 0.5);
        assert_eq!(nitrogen_stress_factor(plant_n, 4.0, res, crit), 0.0);
        // ...and growth alone stresses a plant that has taken up nothing new — the
        // "pure dilution" regime, stated as the function's own property rather than as a
        // scenario's outcome.
        let diluting: Vec<f64> = [1.5, 2.0, 3.0, 4.0]
            .iter()
            .map(|biomass| nitrogen_stress_factor(plant_n, *biomass, res, crit))
            .collect();
        assert!(
            diluting.windows(2).all(|w| w[1] < w[0]),
            "f_N must fall as the crop grows into a fixed reserve: {diluting:?}"
        );
    }
    // -----------------------------------------------------------------------------
    // S5 batch G, the senescence batch: the EQUATION half.
    //
    // ⚠ `mutual_shading_rate` already had a caller-side claim before this batch —
    // `science_gates.rs::the_vks_mutual_shading_regime_is_modelled_not_merely_avoided`,
    // the one genuine direct catch §5ad's control battery found. It is not a duplicate of
    // what is below and neither subsumes the other: the gate evaluates the function at
    // exactly two points, `LAI*` and `LAI* + 1e-9`, so it pins the knot and the strictness
    // of `>`. Measured on the five-mutation battery:
    //
    //   * dropping the shade term entirely:            1 red, the gate — its own subject
    //   * `>` relaxed to `>=`:                         1 red, the gate — its own subject
    //   * flat step -> proportional to the excess:     1 red, the gate — its own subject
    //   * a step that STOPS again above LAI 10:        **0 reds anywhere**
    //   * a special case returning 0 at zero LAI:      **0 reds anywhere**
    //
    // The last two are the FAR FIELD, and they are exactly the two points the Python
    // original evaluates that the gate does not. That is batch E's "a pin evaluated at its
    // subject's symmetry point is not a pin" arriving from the other side: here the pin is
    // at the knot, which is the right place for the knot's own claim and the blind spot
    // for the shape's.
    // -----------------------------------------------------------------------------

    /// The mutual-shading term is a STEP that is FLAT above the threshold, and it is the
    /// bare `rdr_leaf` all the way down to a bare canopy.
    ///
    /// [A] p. 101, quoting Van Keulen & Seligman (1987): leaf area is lost at 5 %/day
    /// **once** LAI exceeds 6. "Once ... exceeds" is the source's own form — the elevated
    /// rate does not decay, and it is not proportional to the excess. The
    /// SUCROS/WOFOST `(LAI − LAI*)/LAI*` shape is a different lineage and is deliberately
    /// not imported, which is a claim about the FAR FIELD and is unfalsifiable at the knot.
    ///
    /// The constants are held as literals rather than read through `params::senescence()`
    /// — batch A's convention, so a loader regression cannot silently move a physics pin.
    /// Mirrors `test_mutual_shading_is_a_STEP_at_the_cited_threshold`.
    #[test]
    fn the_mutual_shading_step_is_flat_above_the_threshold_and_absent_below_it() {
        const RDR: f64 = 0.02;
        const SHADE: f64 = 0.05;
        const THRESHOLD: f64 = 6.0;
        let at = |lai: f64| mutual_shading_rate(lai, RDR, SHADE, THRESHOLD);

        // Below, including the degenerate bare canopy: the term is simply not there.
        assert_eq!(at(0.0), RDR, "a bare canopy shades nothing");
        assert_eq!(at(5.999), RDR);
        assert_eq!(at(THRESHOLD), RDR, "inert AT the threshold (strict `>`)");

        // Above: `rdr + shade`, and it STAYS there. 50 is ~8x the threshold, far outside
        // anything a wheat canopy reaches, which is the point — the claim is about the
        // FORM and a form is only pinned where it could have bent.
        let want = RDR + SHADE;
        assert_eq!(at(6.001), want);
        assert_eq!(at(50.0), want, "a step, not a ramp and not a pulse");
        assert_eq!(at(1.0e6), want, "and it never comes back down");
    }

    /// ⚠ THE LICENSING STEP for a rule stated on leaf AREA in a tree with no area state.
    ///
    /// Van Keulen & Seligman give a rate of leaf AREA loss, "independently of leaf weight
    /// loss", and `Senescence` applies it to leaf CARBON. The transfer is legitimate only
    /// because `specific_leaf_area` is a single constant with no DVS keying, so LAI is
    /// LINEAR in leaf carbon and a relative area rate IS a relative carbon rate, exactly.
    /// Linearity is what the identity needs, and it is checkable; constancy is not the
    /// same claim. It is also [A]'s own stated default, one sentence before the quote.
    ///
    /// ⚠ THE LIMITATION, pinned so it cannot be dropped: V-K&S separated area from weight
    /// BECAUSE specific leaf weight varies by leaf cohort — [A]'s Figure 40, on the very
    /// same page, plots it from ~230 to ~530 kg/ha over a season. A single constant cannot
    /// express that, so the tree inherits their rule under an assumption they explicitly
    /// declined to make. That is recorded here and in `senescence.yaml`, not asserted.
    ///
    /// The `CanopyParams` destructure is the field census: it is the COMPILER that fails
    /// if the struct grows a DVS-keyed field, not a grep. The Python original asserted the
    /// field list by name, which a rename defeats.
    ///
    /// ⚠ **This is NOT a duplicate of `leaf_area_index_is_carbon_times_sla_over_ground`
    /// two tests up, and that was measured rather than argued.** That test pins the
    /// formula at one point, at zero, and under an area halving — which a QUADRATIC
    /// satisfies exactly: `leaf_c² · sla / (100 · A)` returns 25.0 at its point, 0 at
    /// zero, and still doubles when the area halves. Run as a mutation, it left that test
    /// GREEN and reddened only this one. A point value is not a shape, and the area rule's
    /// licence is a claim about the shape.
    /// Mirrors `test_the_area_rule_transfers_because_lai_is_LINEAR_in_leaf_carbon`.
    #[test]
    fn leaf_area_index_is_linear_in_leaf_carbon_which_is_what_licenses_the_area_rule() {
        let crate::biosphere::params::CanopyParams {
            sla_per_mol_c,
            extinction_coef: _,
        } = crate::biosphere::params::canopy();

        // A non-unit ground area on purpose: the identity must be the LEAF carbon's, not
        // an artefact of dividing by one.
        let one = leaf_area_index(1.0, sla_per_mol_c, 3.0);
        for x in [0.5, 2.0, 7.5, 1.0e3] {
            let got = leaf_area_index(x, sla_per_mol_c, 3.0);
            let want = x * one;
            // ⚠ Not bit-exact, and the reason is arithmetic rather than modelling:
            // `(x*sla)/A` and `x*((1*sla)/A)` associate differently and can land 1 ULP
            // apart. The IDENTITY is exact; its float evaluation is not, so the tolerance
            // is a few ULP rather than zero. Stated instead of quietly loosened.
            assert!(
                (got - want).abs() <= 4.0 * f64::EPSILON * want.abs(),
                "LAI({x}) = {got}, linearity wants {want}"
            );
        }
        // ...and zero leaf carbon is zero LAI, which is the limb the step function's
        // `at(0.0)` case above is reached through in a real run.
        assert_eq!(leaf_area_index(0.0, sla_per_mol_c, 3.0), 0.0);
    }
}
