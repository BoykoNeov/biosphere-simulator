//! NASA BVAD Table 3-31 (Rev 2, Feb 2022, p. 58) — the nominal per-crewmember-per-day
//! metabolic interface values, and the arithmetic that turns them into this model's mol
//! accounting. **One crew member is defined here, once**: the station's scenarios and the
//! science gate's validation cabin both read it (`docs/plans/post-roadmap-crew-size.md` §4).
//!
//! Verbatim from `docs/bvad-reference.md`, which is the primary source; we cite the
//! document and copy no dataset (`docs/reuse-and-licenses.md`).
//!
//! ⚠ It lived in `science_gates.rs` as a `#[cfg(test)] mod bvad` until 2026-10-20, when the
//! station's crew became `N` of these crew members and the scenarios needed it too.

/// −m Carbon Dioxide Load (kg/CM-d).
pub const CO2_LOAD_KG: f64 = 1.085;
/// +m Oxygen Consumed (kg/CM-d).
pub const O2_CONSUMED_KG: f64 = 0.895;
/// −m Respiration and Perspiration Water (kg/CM-d).
pub const RESP_PERSP_WATER_KG: f64 = 2.946;
/// −m Urine Water (kg/CM-d).
pub const URINE_WATER_KG: f64 = 1.420;

/// Molar masses (g/mol) for the CARBON/OXYGEN mol accounting our stocks use.
pub const M_CO2: f64 = 44.009;
pub const M_O2: f64 = 31.998;

pub const SECONDS_PER_DAY: f64 = 86_400.0;

/// BVAD's CO₂ load in mol — and, because `CrewRespiration` fixes RQ = 1, also what
/// the model's O₂ consumption must equal.
pub const CO2_MOL: f64 = CO2_LOAD_KG * 1000.0 / M_CO2;
/// BVAD's O₂ consumption in mol.
pub const O2_MOL: f64 = O2_CONSUMED_KG * 1000.0 / M_O2;
/// The daily-effective molar RQ, blending BVAD's nominal + exercise periods.
pub const RQ_EFFECTIVE: f64 = CO2_MOL / O2_MOL;
/// The two modeled water fates (humidity + urine); metabolic + fecal water are not
/// modeled.
pub const MODELED_WATER_KG: f64 = RESP_PERSP_WATER_KG + URINE_WATER_KG;

/// The food-carbon intake (mol C/s) of `crew_count` crew members, given the LOADED
/// `respired_carbon_fraction` (`crew.yaml`): what they breathe out is BVAD's CO₂ load exactly.
///
/// **Carbon is calibrated to BVAD's directly-measured CO₂ load, not to the derived
/// food-carbon total** — the latter (24.654 + 1.332 mol C/CM-d) also rests on the 44–55 %
/// feces-carbon assumption that made `f_resp` what it is, so a constant total would be a hidden
/// second copy of `f_resp`. That also keeps the O₂ headline a clean number: model O₂ = model
/// CO₂ = BVAD CO₂ exactly, at RQ = 1.
pub fn food_intake_rate(crew_count: f64, respired_carbon_fraction: f64) -> f64 {
    crew_count * CO2_MOL / respired_carbon_fraction / SECONDS_PER_DAY
}

/// The modelled water intake (kg/s) of `crew_count` crew members (humidity + urine).
pub fn water_intake_rate(crew_count: f64) -> f64 {
    crew_count * MODELED_WATER_KG / SECONDS_PER_DAY
}
