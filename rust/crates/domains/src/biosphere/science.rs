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
    let f_int = intercepted_fraction(lai, canopy.extinction_coef);
    let mean_absorbed_par = incident_par * f_int / lai;
    let leaf_rate = gross_leaf_assimilation(ci, mean_absorbed_par, photo);
    let canopy_rate = leaf_rate * lai;
    let f_temp = temperature_factor(temp_c, photo);
    canopy_rate * window_s * ground_area * MICROMOL_TO_MOL * f_temp * limitation
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
}
