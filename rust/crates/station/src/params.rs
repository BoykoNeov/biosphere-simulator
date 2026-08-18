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

const WATER_RECOVERY_YAML: &str = include_str!("../params/water_recovery.yaml");
const LAMP_YAML: &str = include_str!("../params/lamp.yaml");
const HARVEST_YAML: &str = include_str!("../params/harvest.yaml");

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

/// The **frozen station param-file census**: `(filename, embedded text)` for the three files
/// this module loads, in filename order (slice C8 of the reference flip).
///
/// The station manifest's `param_files` is these three plus the five
/// [`domains::params::param_files`] owns. No exclusion rule — `crates/station/params/` holds
/// nothing but frozen files.
pub fn param_files() -> Vec<(&'static str, &'static str)> {
    let mut files = vec![
        ("harvest.yaml", HARVEST_YAML),
        ("lamp.yaml", LAMP_YAML),
        ("water_recovery.yaml", WATER_RECOVERY_YAML),
    ];
    files.sort_by_key(|(name, _)| *name);
    files
}

/// The directory the station census is a census **of**, and its expected file count.
pub const PARAMS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/params");

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

    /// The census equals what `crates/station/params/` holds.
    ///
    /// ⚠⚠ The completeness half of `param_files` — see the sibling twin in
    /// `crates/domains/src/params.rs`. The digests are author-neutral; the roster is not.
    #[test]
    fn the_census_matches_the_directory_on_disk() {
        let mut on_disk: Vec<String> = std::fs::read_dir(PARAMS_DIR)
            .expect("the station params directory is readable")
            .map(|entry| entry.expect("a readable dir entry"))
            .filter(|entry| entry.path().is_file())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.ends_with(".yaml"))
            .collect();
        on_disk.sort();

        let census: Vec<String> = param_files()
            .iter()
            .map(|(name, _)| (*name).to_string())
            .collect();
        assert_eq!(
            census, on_disk,
            "the loaded station param-file census and the directory disagree. A ROSTER              finding, not a value one: do NOT 'fix' it by editing whichever list is shorter."
        );
        assert_eq!(census.len(), 3, "the frozen station param set is 3 files");
    }

    /// Every basename is unique across the SIX directories the station manifest spans.
    ///
    /// ⚠ The manifest keys `param_files` on basenames, so a name appearing in two of the six
    /// directories would silently collapse two files into one entry. Nothing asserted this
    /// before; Python's `_param_paths()` doc *claims* uniqueness and its dict would quietly
    /// keep whichever directory came last.
    ///
    /// ⚠⚠ **The directory-level claim is COMPOSED from two gates, not asserted by one**,
    /// and saying so is the difference between a check and a claim. This test covers the two
    /// **compile-time include lists**; that those lists match what the six directories hold
    /// is the separate per-directory census (`the_census_matches_the_directory_on_disk` here
    /// and in `domains::params`). A duplicate *file on disk* cannot reach this assertion at
    /// all — it reddens the census instead — which is why the control below adds a duplicate
    /// to a **list**.
    #[test]
    fn every_basename_is_unique_across_the_station_and_sibling_directories() {
        let mut all: Vec<&str> = param_files().iter().map(|(n, _)| *n).collect();
        all.extend(domains::params::param_files().iter().map(|(n, _)| *n));
        let mut sorted = all.clone();
        sorted.sort_unstable();
        let mut deduped = sorted.clone();
        deduped.dedup();
        assert_eq!(
            sorted, deduped,
            "a param-file basename appears twice across the six directories the station              manifest spans; a basename-keyed `param_files` would collapse them"
        );
        assert_eq!(
            all.len(),
            8,
            "the frozen station contract names 8 param files"
        );
    }

    /// The uniqueness assertion above has teeth: a duplicated list entry reddens it.
    ///
    /// ⚠ Built because the assertion was the newest in slice C8 and the one both manifests'
    /// `_authority` text advertises, and it had **no control** — control G planted a
    /// duplicate `lamp.yaml` on disk and reddened the *census* instead, proving a different
    /// thing. The subject here is the two compile-time lists, so the control has to be a
    /// list, not a file.
    #[test]
    fn a_duplicated_basename_across_the_two_lists_is_detected() {
        let mut all: Vec<&str> = param_files().iter().map(|(n, _)| *n).collect();
        all.extend(domains::params::param_files().iter().map(|(n, _)| *n));
        // The control: a name that already exists on the sibling side, added again here.
        all.push("crew.yaml");
        let mut sorted = all.clone();
        sorted.sort_unstable();
        let mut deduped = sorted.clone();
        deduped.dedup();
        assert_ne!(
            sorted, deduped,
            "a duplicated basename went undetected — the uniqueness check above is inert"
        );
        assert_eq!(sorted.len(), deduped.len() + 1, "exactly one duplicate");
    }

    /// No frozen station param file carries a separator Python's `splitlines` breaks on.
    #[test]
    fn no_frozen_param_file_carries_an_exotic_line_separator() {
        for (name, text) in param_files() {
            assert_eq!(
                config::provenance::contains_exotic_line_separator(text),
                None,
                "{name} contains a character Python's splitlines treats as a line break but                  the reference's normalization does not — the two hash rules would diverge"
            );
        }
    }
}
