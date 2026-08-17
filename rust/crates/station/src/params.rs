//! The station-owned coefficients, **loaded from the frozen param YAML** (reference
//! flip, slice C1).
//!
//! Until C1 this module read `station_params.txt`, a hex-float table produced by running
//! the *Python* station loaders — so the schema, the unit guard and the bound check all
//! executed on the Python side and the port consumed the answer. Now this module does
//! that work itself, through the [`config`] boundary crate.
//!
//! **No number moved, and that is gated rather than asserted:**
//! [`tests::every_value_matches_the_generated_table`] compares every loaded value with
//! the retained table bit-for-bit. See `crates/domains/src/params.rs` for the full
//! rationale, and `docs/plans/post-roadmap-reference-flip.md` §5d for the measurement
//! that priced the slice before it was built.

use config::{require_closed, require_half_open, require_non_negative, ConfigError, ParamFile};
use domains::biosphere::weather::PAR_UMOL_PER_J;

use crate::flows::{HarvestParams, LampParams, WaterRecoveryParams};

const WATER_RECOVERY_YAML: &str =
    include_str!("../../../../src/station/params/water_recovery.yaml");
const LAMP_YAML: &str = include_str!("../../../../src/station/params/lamp.yaml");
const HARVEST_YAML: &str = include_str!("../../../../src/station/params/harvest.yaml");

fn file(text: &str, name: &'static str) -> ParamFile {
    ParamFile::parse(text, name).unwrap_or_else(|e| panic!("{name} is malformed: {e}"))
}

fn checked<T>(result: Result<T, ConfigError>, name: &'static str) -> T {
    result.unwrap_or_else(|e| panic!("{name} failed its frozen bound/unit check: {e}"))
}

/// The crew water-recovery coefficients (`water_recovery.yaml`).
///
/// The rate is `>= 0` (zero is a valid, inert recycler); the efficiency is a fraction in
/// [0, 1].
pub fn water_recovery() -> WaterRecoveryParams {
    let f = file(WATER_RECOVERY_YAML, "water_recovery.yaml");
    let v = checked(
        f.guarded_set(
            &[
                ("recovery_rate", "1/s"),
                ("recovery_efficiency", "dimensionless"),
            ],
            "water_recovery.yaml",
        ),
        "water_recovery.yaml",
    );
    WaterRecoveryParams {
        recovery_rate: checked(
            require_non_negative(v[0], "recovery_rate", "water_recovery.yaml"),
            "water_recovery.yaml",
        ),
        recovery_efficiency: checked(
            require_closed(v[1], 0.0, 1.0, "recovery_efficiency", "water_recovery.yaml"),
            "water_recovery.yaml",
        ),
    }
}

/// The grow-lamp photon efficacy (`lamp.yaml`).
///
/// ⚠ Its ceiling is **physical, not conventional**: `PAR_UMOL_PER_J` is the efficacy at
/// which every input joule becomes a PAR photon (η_lamp = 1, the waste-heat leg exactly
/// 0). A value above it would be an over-unity lamp. The bound is therefore expressed
/// against the same constant the flow divides by, never a literal.
pub fn lamp() -> LampParams {
    let f = file(LAMP_YAML, "lamp.yaml");
    let v = checked(
        f.guarded_set(&[("photon_efficacy", "umol/J")], "lamp.yaml"),
        "lamp.yaml",
    );
    LampParams {
        photon_efficacy: checked(
            require_half_open(v[0], 0.0, PAR_UMOL_PER_J, "photon_efficacy", "lamp.yaml"),
            "lamp.yaml",
        ),
    }
}

/// The grain-harvest rate (`harvest.yaml`). `>= 0`; zero is a valid, inert harvester.
pub fn harvest() -> HarvestParams {
    let f = file(HARVEST_YAML, "harvest.yaml");
    let v = checked(
        f.guarded_set(&[("harvest_rate", "1/s")], "harvest.yaml"),
        "harvest.yaml",
    );
    HarvestParams {
        harvest_rate: checked(
            require_non_negative(v[0], "harvest_rate", "harvest.yaml"),
            "harvest.yaml",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The retained control: the hex-float table the Python loaders produced.
    const GENERATED_TABLE: &str = include_str!("station_params.txt");

    fn generated() -> std::collections::BTreeMap<&'static str, f64> {
        let mut out = std::collections::BTreeMap::new();
        for line in GENERATED_TABLE.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut fields = line.split_whitespace();
            let name = fields.next().expect("a name");
            let hex = fields.next().expect("a hex-float");
            out.insert(name, simcore::hexfloat::parse(hex).expect("parses"));
        }
        out
    }

    /// ⚠⚠ **The slice's gate on the station side.** Bit equality, not a band.
    #[test]
    fn every_value_matches_the_generated_table() {
        let t = generated();
        let wr = water_recovery();
        let pairs: [(&str, f64); 4] = [
            ("recovery_rate", wr.recovery_rate),
            ("recovery_efficiency", wr.recovery_efficiency),
            ("photon_efficacy", lamp().photon_efficacy),
            ("harvest_rate", harvest().harvest_rate),
        ];
        for (name, loaded) in pairs {
            assert_eq!(
                loaded.to_bits(),
                t[name].to_bits(),
                "{name}: loaded {loaded:?} != generated {:?}",
                t[name]
            );
        }
        assert_eq!(t.len(), 4, "the control table still names exactly 4 params");
    }

    /// Load `lamp.yaml` with its efficacy replaced, to drive the real bound.
    fn lamp_with_efficacy(value: f64) -> Result<f64, ConfigError> {
        let doctored = LAMP_YAML.replacen("value: 2.5", &format!("value: {value}"), 1);
        assert_ne!(doctored, LAMP_YAML, "the substitution must actually apply");
        let v = ParamFile::parse(&doctored, "lamp.yaml")
            .expect("still parses")
            .guarded_set(&[("photon_efficacy", "umol/J")], "lamp.yaml")?;
        require_half_open(v[0], 0.0, PAR_UMOL_PER_J, "photon_efficacy", "lamp.yaml")
    }

    /// ⚠ The lamp ceiling is **physical** and is expressed against the same constant the
    /// flow divides by, so a change to `PAR_UMOL_PER_J` cannot leave a stale literal
    /// here. Driven through the real file, not asserted on the helper: an over-unity
    /// lamp is rejected, and the ceiling itself (η_lamp = 1 exactly) is accepted.
    #[test]
    fn an_over_unity_lamp_is_rejected() {
        assert!(lamp_with_efficacy(PAR_UMOL_PER_J * 1.001).is_err());
        assert_eq!(lamp_with_efficacy(PAR_UMOL_PER_J).unwrap(), PAR_UMOL_PER_J);
        assert!(lamp_with_efficacy(0.0).is_err());
    }

    #[test]
    fn all_params_present_and_in_bounds() {
        let wr = water_recovery();
        assert!(wr.recovery_rate >= 0.0);
        assert!((0.0..=1.0).contains(&wr.recovery_efficiency));
        assert!(lamp().photon_efficacy > 0.0);
        assert!(harvest().harvest_rate >= 0.0);
    }
}
