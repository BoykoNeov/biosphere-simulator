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

/// Intercepted fraction `1 − exp(−k · LAI)` (Monsi & Saeki).
pub fn intercepted_fraction(lai: f64, extinction_coef: f64) -> f64 {
    1.0 - (-extinction_coef * lai).exp()
}

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

/// Development stage `DVS ∈ [0, 2]` from thermal time (TSUM1/TSUM2).
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
}
