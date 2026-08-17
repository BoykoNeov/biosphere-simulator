//! The sibling coefficients, **loaded from the frozen param YAML** (reference flip,
//! slice C1).
//!
//! # What changed, and what did not
//!
//! Until slice C1 this module read `sibling_params.txt` — a hex-float table that
//! `tests/crossport/gen_sibling_params.py` produced by running the *Python* loaders.
//! That made Python the loader for the canonical run, which is the hole in the middle of
//! "Rust is the reference": the schema, the unit guard and the bound check all executed
//! on the Python side and the port consumed the result.
//!
//! Now this module does that work itself, through the [`config`] boundary crate: the
//! same five files, the same exact-string unit guards, the same documented bounds.
//!
//! **Nothing about the numbers changed, and that was measured before it was built.**
//! Every one of these twelve values is a plain decimal in its YAML file, and `f64`
//! parsing is correctly rounded on both sides, so the loaded bits are the bits the
//! generated table pinned. `sibling_params.txt` is **retained as the control**, not as
//! the source: [`tests::every_value_matches_the_generated_table`] asserts bit equality
//! against it, which is what makes "bit-neutral" a checked claim rather than an
//! intention. The generator is retired only once that gate has been green — the rule
//! `docs/plans/post-roadmap-reference-flip.md` §5c sets for every generator.
//!
//! ⚠ The `include_str!` paths reach out of the Rust tree into `src/`, the Python
//! package. That is deliberate and temporary: under target state C the param files
//! cannot stay in a deleted package, and the relocation is named in §5d as this slice's
//! successor. The manifests key on **basenames**, so that move will shift neither a key
//! nor a hash.

use config::{require_closed, require_half_open, require_non_negative, require_positive};
use config::{ConfigError, ParamFile};

use crate::crew::CrewParams;
use crate::eclss::EclssParams;
use crate::power::{ChargeParams, SelfDischargeParams};
use crate::thermal::ThermalParams;

const CHARGE_YAML: &str = include_str!("../../../../src/domains/power/params/charge.yaml");
const SELF_DISCHARGE_YAML: &str =
    include_str!("../../../../src/domains/power/params/self_discharge.yaml");
const RADIATOR_YAML: &str = include_str!("../../../../src/domains/thermal/params/radiator.yaml");
const ECLSS_YAML: &str = include_str!("../../../../src/domains/eclss/params/eclss.yaml");
const CREW_YAML: &str = include_str!("../../../../src/domains/crew/params/crew.yaml");

/// Load a param file, panicking on a malformed one.
///
/// These five files are **frozen and embedded at compile time**, so a failure here is a
/// broken build artefact, not a runtime input error — the same standing the previous
/// `expect("sibling param hex-float parses")` had. Authored files, which *are* runtime
/// input, go through `authoring` and surface a `Result`.
fn file(text: &str, name: &'static str) -> ParamFile {
    ParamFile::parse(text, name).unwrap_or_else(|e| panic!("{name} is malformed: {e}"))
}

fn checked<T>(result: Result<T, ConfigError>, name: &'static str) -> T {
    result.unwrap_or_else(|e| panic!("{name} failed its frozen bound/unit check: {e}"))
}

/// The Power one-way charge efficiency η_c (`charge.yaml`).
///
/// η ∈ (0, 1]: 1 is lossless charging (the heat leg collapses to 0); 0 would be a
/// battery that stores nothing.
pub fn charge() -> ChargeParams {
    let f = file(CHARGE_YAML, "charge.yaml");
    let v = checked(
        f.guarded_set(&[("charge_efficiency", "dimensionless")], "charge.yaml"),
        "charge.yaml",
    );
    ChargeParams {
        charge_efficiency: checked(
            require_half_open(v[0], 0.0, 1.0, "charge_efficiency", "charge.yaml"),
            "charge.yaml",
        ),
    }
}

/// The Power first-order self-discharge rate k (`self_discharge.yaml`).
///
/// `k >= 0`: zero is valid (an ideal leak-free cell — inert, the herbivory "a zero rate
/// is valid" precedent), negative is not.
pub fn self_discharge() -> SelfDischargeParams {
    let f = file(SELF_DISCHARGE_YAML, "self_discharge.yaml");
    let v = checked(
        f.guarded_set(&[("self_discharge_rate", "1/s")], "self_discharge.yaml"),
        "self_discharge.yaml",
    );
    SelfDischargeParams {
        self_discharge_rate: checked(
            require_non_negative(v[0], "self_discharge_rate", "self_discharge.yaml"),
            "self_discharge.yaml",
        ),
    }
}

/// The Thermal radiator properties (`radiator.yaml`).
///
/// ⚠ This is the file carrying `heat_capacity: 1.0e7` — the unsigned-exponent scalar
/// YAML 1.1 resolves as a *string*. See [`config::params`] for why parsing the text is
/// both faithful to pydantic and bit-neutral.
pub fn thermal() -> ThermalParams {
    let f = file(RADIATOR_YAML, "radiator.yaml");
    let v = checked(
        f.guarded_set(
            &[
                ("emissivity", "dimensionless"),
                ("radiator_area", "m^2"),
                ("heat_capacity", "J/K"),
                ("space_temperature", "K"),
            ],
            "radiator.yaml",
        ),
        "radiator.yaml",
    );
    ThermalParams {
        emissivity: checked(
            require_half_open(v[0], 0.0, 1.0, "emissivity", "radiator.yaml"),
            "radiator.yaml",
        ),
        radiator_area: checked(
            require_positive(v[1], "radiator_area", "radiator.yaml"),
            "radiator.yaml",
        ),
        heat_capacity: checked(
            require_positive(v[2], "heat_capacity", "radiator.yaml"),
            "radiator.yaml",
        ),
        space_temperature: checked(
            require_non_negative(v[3], "space_temperature", "radiator.yaml"),
            "radiator.yaml",
        ),
    }
}

/// The ECLSS control-loop coefficients (`eclss.yaml`). All four are strictly positive.
pub fn eclss() -> EclssParams {
    let f = file(ECLSS_YAML, "eclss.yaml");
    let v = checked(
        f.guarded_set(
            &[
                ("co2_scrub_rate", "1/s"),
                ("condense_rate", "1/s"),
                ("o2_makeup_gain", "1/s"),
                ("o2_setpoint", "mol"),
            ],
            "eclss.yaml",
        ),
        "eclss.yaml",
    );
    let names = [
        "co2_scrub_rate",
        "condense_rate",
        "o2_makeup_gain",
        "o2_setpoint",
    ];
    for (value, name) in v.iter().zip(names) {
        checked(require_positive(*value, name, "eclss.yaml"), "eclss.yaml");
    }
    EclssParams {
        co2_scrub_rate: v[0],
        condense_rate: v[1],
        o2_makeup_gain: v[2],
        o2_setpoint: v[3],
    }
}

/// The Crew metabolic-split fractions (`crew.yaml`). Both are fractions in [0, 1].
pub fn crew() -> CrewParams {
    let f = file(CREW_YAML, "crew.yaml");
    let v = checked(
        f.guarded_set(
            &[
                ("respired_carbon_fraction", "dimensionless"),
                ("insensible_water_fraction", "dimensionless"),
            ],
            "crew.yaml",
        ),
        "crew.yaml",
    );
    CrewParams {
        respired_carbon_fraction: checked(
            require_closed(v[0], 0.0, 1.0, "respired_carbon_fraction", "crew.yaml"),
            "crew.yaml",
        ),
        insensible_water_fraction: checked(
            require_closed(v[1], 0.0, 1.0, "insensible_water_fraction", "crew.yaml"),
            "crew.yaml",
        ),
    }
}

/// The **frozen sibling param-file census**: `(filename, embedded text)` for the five files
/// this module loads, in filename order (slice C8 of the reference flip).
///
/// The station manifest's `param_files` is these five plus the three
/// [`station::params::param_files`](../../station/params/fn.param_files.html) owns — eight
/// entries, every basename unique across the six directories, which is why the manifest can
/// key on basenames at all.
///
/// ⚠ **No exclusion rule here, and that asymmetry is the point.** The biosphere census has
/// two (non-recursion for the potato overrides, `demo.yaml` by name); these five directories
/// hold nothing but frozen files, so the rule is biosphere-only. Stating it per side keeps a
/// reader from generalising the harder rule to a place it does not apply.
pub fn param_files() -> Vec<(&'static str, &'static str)> {
    let mut files = vec![
        ("charge.yaml", CHARGE_YAML),
        ("crew.yaml", CREW_YAML),
        ("eclss.yaml", ECLSS_YAML),
        ("radiator.yaml", RADIATOR_YAML),
        ("self_discharge.yaml", SELF_DISCHARGE_YAML),
    ];
    files.sort_by_key(|(name, _)| *name);
    files
}

/// The five directories the sibling census is a census **of**, `(directory, expected count)`.
///
/// Resolved at compile time, the same reach-out `include_str!` above makes. ⚠ Under target
/// state C these paths point into a Python package scheduled for deletion; the census makes
/// that a runtime dependency too, which sharpens the relocation trigger recorded in
/// `docs/plans/post-roadmap-reference-flip.md` §5d rather than resolving it.
pub const PARAM_DIRS: [(&str, usize); 4] = [
    (
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../src/domains/power/params"
        ),
        2,
    ),
    (
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../src/domains/thermal/params"
        ),
        1,
    ),
    (
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../src/domains/eclss/params"
        ),
        1,
    ),
    (
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../src/domains/crew/params"
        ),
        1,
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// The retained control (see the module header): the hex-float table the Python
    /// loaders produced, kept **only** so this file's load can be checked against it.
    const GENERATED_TABLE: &str = include_str!("sibling_params.txt");

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

    /// ⚠⚠ **The slice's gate.** Every value this module now loads out of the YAML is
    /// bit-identical to what the Python loaders produced. Not "within a band" — the
    /// same `f64`. A single moved bit here means C1 stopped being a re-anchoring and
    /// became an unfreeze with the goldens behind it.
    #[test]
    fn every_value_matches_the_generated_table() {
        let t = generated();
        let ch = charge();
        let sd = self_discharge();
        let th = thermal();
        let ec = eclss();
        let cr = crew();
        let pairs: [(&str, f64); 12] = [
            ("charge_efficiency", ch.charge_efficiency),
            ("self_discharge_rate", sd.self_discharge_rate),
            ("emissivity", th.emissivity),
            ("radiator_area", th.radiator_area),
            ("heat_capacity", th.heat_capacity),
            ("space_temperature", th.space_temperature),
            ("co2_scrub_rate", ec.co2_scrub_rate),
            ("condense_rate", ec.condense_rate),
            ("o2_makeup_gain", ec.o2_makeup_gain),
            ("o2_setpoint", ec.o2_setpoint),
            ("respired_carbon_fraction", cr.respired_carbon_fraction),
            ("insensible_water_fraction", cr.insensible_water_fraction),
        ];
        for (name, loaded) in pairs {
            let want = t[name];
            assert_eq!(
                loaded.to_bits(),
                want.to_bits(),
                "{name}: loaded {loaded:?} ({:x}) != generated {want:?} ({:x})",
                loaded.to_bits(),
                want.to_bits()
            );
        }
        assert_eq!(
            t.len(),
            12,
            "the control table still names exactly 12 params"
        );
    }

    /// ⚠ The control's own completeness: a param added to a YAML file and wired to
    /// nothing must fail the load, not be silently dropped. `guarded_set` rejects an
    /// unexpected key, so this asserts the mechanism rather than trusting it.
    #[test]
    fn an_extra_param_in_a_file_is_rejected() {
        let extra = CREW_YAML.replace(
            "parameters:\n",
            "parameters:\n  bogus:\n    value: 1.0\n    unit: \"dimensionless\"\n    source: \"x\"\n",
        );
        let f = ParamFile::parse(&extra, "crew.yaml").expect("still parses");
        assert!(f
            .guarded_set(
                &[
                    ("respired_carbon_fraction", "dimensionless"),
                    ("insensible_water_fraction", "dimensionless"),
                ],
                "crew.yaml",
            )
            .is_err());
    }

    /// ⚠ The unit guard is load-bearing, not decoration: re-declaring a rate in the
    /// wrong unit must fail even though the number is untouched.
    #[test]
    fn a_re_declared_unit_is_rejected() {
        let wrong = ECLSS_YAML.replacen("unit: \"1/s\"", "unit: \"1/day\"", 1);
        let f = ParamFile::parse(&wrong, "eclss.yaml").expect("still parses");
        assert!(f.guarded("co2_scrub_rate", "1/s", "eclss.yaml").is_err());
    }

    #[test]
    fn bounds_match_the_loaders() {
        let c = charge();
        assert!(0.0 < c.charge_efficiency && c.charge_efficiency <= 1.0);
        let th = thermal();
        assert!(0.0 < th.emissivity && th.emissivity <= 1.0);
        assert!(th.radiator_area > 0.0 && th.heat_capacity > 0.0);
        let cr = crew();
        assert!((0.0..=1.0).contains(&cr.respired_carbon_fraction));
        assert!((0.0..=1.0).contains(&cr.insensible_water_fraction));
    }

    /// The census equals what the five sibling directories hold.
    ///
    /// ⚠⚠ The completeness half of `param_files`, on the side where the digests cannot show
    /// a problem: both ports hash the same file, so the only thing that can go wrong with
    /// this key is the **roster** — a param file added and wired into no loader, or a loader
    /// dropped with its file left behind.
    #[test]
    fn the_census_matches_the_directories_on_disk() {
        let mut on_disk: Vec<String> = Vec::new();
        for (dir, expected) in PARAM_DIRS {
            let names: Vec<String> = std::fs::read_dir(dir)
                .unwrap_or_else(|e| panic!("{dir} is readable: {e}"))
                .map(|entry| entry.expect("a readable dir entry"))
                .filter(|entry| entry.path().is_file())
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .filter(|name| name.ends_with(".yaml"))
                .collect();
            assert_eq!(
                names.len(),
                expected,
                "{dir} holds {names:?}, not {expected} file(s) — the census's per-directory                  count is stale"
            );
            on_disk.extend(names);
        }
        on_disk.sort();

        let census: Vec<String> = param_files()
            .iter()
            .map(|(name, _)| (*name).to_string())
            .collect();
        assert_eq!(
            census, on_disk,
            "the loaded sibling param-file census and the directories disagree. A ROSTER              finding, not a value one: do NOT 'fix' it by editing whichever list is shorter."
        );
        assert_eq!(census.len(), 5, "the frozen sibling param set is 5 files");
    }

    /// No frozen sibling param file carries a separator Python's `splitlines` breaks on.
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
