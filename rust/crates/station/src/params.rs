//! The station-owned coefficients, read from the generated hex-float file (P7.5).
//!
//! [`STATION_PARAMS`] is `tests/crossport/gen_station_params.py`'s output — each of the
//! three station-owned coefficients loaded through its frozen Python loader (pydantic
//! schema, exact-string unit guard, bound check) and emitted as a C99 hex-float. We
//! `include_str!` it and parse with the [`simcore::hexfloat`] codec, so the crate links no
//! YAML parser (the Step-3/4 Option-C precedent). `test_crossport.py::
//! test_station_params_in_sync` guards the file against generator drift.

use std::collections::BTreeMap;

use simcore::hexfloat;

use crate::flows::{HarvestParams, LampParams, WaterRecoveryParams};

/// The committed, generated station-param table (see `gen_station_params.py`).
const STATION_PARAMS: &str = include_str!("station_params.txt");

/// Parse the embedded file into a `name → value` table (comment/blank lines skipped).
fn table() -> BTreeMap<&'static str, f64> {
    let mut out: BTreeMap<&'static str, f64> = BTreeMap::new();
    for line in STATION_PARAMS.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split_whitespace();
        let name = fields.next().expect("station param line has a name");
        let hex = fields
            .next()
            .expect("station param line has a hex-float value");
        let value = hexfloat::parse(hex).expect("station param hex-float parses");
        out.insert(name, value);
    }
    out
}

/// Look up a required param, panicking with the missing key (a generation bug).
fn get(t: &BTreeMap<&'static str, f64>, key: &str) -> f64 {
    *t.get(key)
        .unwrap_or_else(|| panic!("missing station param {key:?} in station_params.txt"))
}

/// The crew water-recovery coefficients (`water_recovery.yaml`).
pub fn water_recovery() -> WaterRecoveryParams {
    let t = table();
    WaterRecoveryParams {
        recovery_rate: get(&t, "recovery_rate"),
        recovery_efficiency: get(&t, "recovery_efficiency"),
    }
}

/// The grow-lamp photon efficacy (`lamp.yaml`).
pub fn lamp() -> LampParams {
    let t = table();
    LampParams {
        photon_efficacy: get(&t, "photon_efficacy"),
    }
}

/// The grain-harvest rate (`harvest.yaml`).
pub fn harvest() -> HarvestParams {
    let t = table();
    HarvestParams {
        harvest_rate: get(&t, "harvest_rate"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_params_present_and_in_bounds() {
        let wr = water_recovery();
        assert!(wr.recovery_rate >= 0.0);
        assert!((0.0..=1.0).contains(&wr.recovery_efficiency));
        assert!(lamp().photon_efficacy > 0.0);
        assert!(harvest().harvest_rate >= 0.0);
    }
}
