//! The sibling coefficients, read from the generated hex-float file (Phase-7 P7.3).
//!
//! [`SIBLING_PARAMS`] is `tests/crossport/gen_sibling_params.py`'s output — each of the
//! 12 Phase-5 coefficients loaded through its frozen Python loader (pydantic schema, the
//! exact-string unit guard, and the bound check) and emitted as a C99 hex-float. We
//! `include_str!` it and parse with the [`simcore::hexfloat`] codec, so the crate links
//! no YAML parser (no `serde_yaml`) and re-parses nothing at runtime beyond a 12-line
//! table.
//!
//! Decimal param values round-trip bit-identically across correctly-rounding parsers,
//! so this pins the exact loader-produced bits. `test_crossport.py::
//! test_sibling_params_in_sync` guards the file against generator drift.

use std::collections::BTreeMap;

use simcore::hexfloat;

use crate::crew::CrewParams;
use crate::eclss::EclssParams;
use crate::power::{ChargeParams, SelfDischargeParams};
use crate::thermal::ThermalParams;

/// The committed, generated param table (see `gen_sibling_params.py`).
const SIBLING_PARAMS: &str = include_str!("sibling_params.txt");

/// Parse the embedded file into a `name → value` table (comment/blank lines skipped).
fn table() -> BTreeMap<&'static str, f64> {
    let mut out: BTreeMap<&'static str, f64> = BTreeMap::new();
    for line in SIBLING_PARAMS.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split_whitespace();
        let name = fields.next().expect("sibling param line has a name");
        let hex = fields.next().expect("sibling param line has a hex-float value");
        let value = hexfloat::parse(hex).expect("sibling param hex-float parses");
        out.insert(name, value);
    }
    out
}

/// Look up a required param, panicking with the missing key (a generation bug).
fn get(t: &BTreeMap<&'static str, f64>, key: &str) -> f64 {
    *t.get(key)
        .unwrap_or_else(|| panic!("missing sibling param {key:?} in sibling_params.txt"))
}

/// The Power one-way charge efficiency η_c (`charge.yaml`).
pub fn charge() -> ChargeParams {
    let t = table();
    ChargeParams {
        charge_efficiency: get(&t, "charge_efficiency"),
    }
}

/// The Power first-order self-discharge rate k (`self_discharge.yaml`).
pub fn self_discharge() -> SelfDischargeParams {
    let t = table();
    SelfDischargeParams {
        self_discharge_rate: get(&t, "self_discharge_rate"),
    }
}

/// The Thermal radiator properties (`radiator.yaml`).
pub fn thermal() -> ThermalParams {
    let t = table();
    ThermalParams {
        emissivity: get(&t, "emissivity"),
        radiator_area: get(&t, "radiator_area"),
        heat_capacity: get(&t, "heat_capacity"),
        space_temperature: get(&t, "space_temperature"),
    }
}

/// The ECLSS control-loop coefficients (`eclss.yaml`).
pub fn eclss() -> EclssParams {
    let t = table();
    EclssParams {
        co2_scrub_rate: get(&t, "co2_scrub_rate"),
        condense_rate: get(&t, "condense_rate"),
        o2_makeup_gain: get(&t, "o2_makeup_gain"),
        o2_setpoint: get(&t, "o2_setpoint"),
    }
}

/// The Crew metabolic-split fractions (`crew.yaml`).
pub fn crew() -> CrewParams {
    let t = table();
    CrewParams {
        respired_carbon_fraction: get(&t, "respired_carbon_fraction"),
        insensible_water_fraction: get(&t, "insensible_water_fraction"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_twelve_params_present_and_finite() {
        let t = table();
        assert_eq!(t.len(), 12, "expected 12 sibling params");
        for v in t.values() {
            assert!(v.is_finite());
        }
    }

    #[test]
    fn bounds_match_the_loaders() {
        // Sanity: the loaded values sit in the loaders' documented ranges (a decimal
        // round-trip check, not a re-derivation).
        let c = charge();
        assert!(0.0 < c.charge_efficiency && c.charge_efficiency <= 1.0);
        let th = thermal();
        assert!(0.0 < th.emissivity && th.emissivity <= 1.0);
        assert!(th.radiator_area > 0.0 && th.heat_capacity > 0.0);
        let cr = crew();
        assert!((0.0..=1.0).contains(&cr.respired_carbon_fraction));
        assert!((0.0..=1.0).contains(&cr.insensible_water_fraction));
    }
}
