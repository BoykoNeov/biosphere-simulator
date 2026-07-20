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

/// Daily gross canopy assimilation (mol C day⁻¹) — the provisional big-leaf.
#[allow(clippy::too_many_arguments)]
pub fn daily_canopy_assimilation(
    incident_par: f64,
    lai: f64,
    ci: f64,
    temp_c: f64,
    daylength_s: f64,
    photo: &PhotosynthesisParams,
    canopy: &CanopyParams,
    ground_area: f64,
    limitation: f64,
) -> f64 {
    if lai == 0.0 {
        return 0.0;
    }
    let f_int = intercepted_fraction(lai, canopy.extinction_coef);
    let mean_absorbed_par = incident_par * f_int / lai;
    let leaf_rate = gross_leaf_assimilation(ci, mean_absorbed_par, photo);
    let canopy_rate = leaf_rate * lai;
    let f_temp = temperature_factor(temp_c, photo);
    canopy_rate * daylength_s * ground_area * MICROMOL_TO_MOL * f_temp * limitation
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

/// Soil-water stress factor `f_water ∈ [0, 1]`.
pub fn water_stress_factor(soil_water: f64, sw_wilting: f64, sw_critical: f64) -> f64 {
    if soil_water <= sw_wilting {
        return 0.0;
    }
    if soil_water >= sw_critical {
        return 1.0;
    }
    (soil_water - sw_wilting) / (sw_critical - sw_wilting)
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

/// Development stage `DVS ∈ [0, 2]` from thermal time (TSUM1/TSUM2).
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
