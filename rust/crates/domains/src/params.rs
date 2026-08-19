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
//! ⚠ **The reach-out is gone as of Stage-3 slice S1** (2026-08-18). These files used to
//! be `include_str!`-ed out of `src/domains/<domain>/params/` — the Python package — which
//! meant the reference did not compile without a tree scheduled for deletion. They now live
//! in `crates/domains/params/<domain>/`, next to the crate that reads them. The move was a
//! pure rename: the manifests key on **basenames**, so it shifted neither a key nor a hash,
//! and the byte gate proved it rather than the plan asserting it.

use config::{require_closed, require_half_open, require_non_negative, require_positive};
use config::{ConfigError, ParamFile};

use crate::crew::CrewParams;
use crate::eclss::EclssParams;
use crate::power::{ChargeParams, SelfDischargeParams};
use crate::thermal::ThermalParams;

const CHARGE_YAML: &str = include_str!("../params/power/charge.yaml");
const SELF_DISCHARGE_YAML: &str = include_str!("../params/power/self_discharge.yaml");
const RADIATOR_YAML: &str = include_str!("../params/thermal/radiator.yaml");
const ECLSS_YAML: &str = include_str!("../params/eclss/eclss.yaml");
const CREW_YAML: &str = include_str!("../params/crew/crew.yaml");

/// Panic on a frozen file that does not load.
///
/// These five files are **frozen and embedded at compile time**, so a failure here is a
/// broken build artefact, not a runtime input error — the same standing the previous
/// `expect("sibling param hex-float parses")` had. Authored files, which *are* runtime
/// input, go through `authoring` and surface a `Result`.
fn frozen<T>(result: Result<T, ConfigError>, name: &'static str) -> T {
    result.unwrap_or_else(|e| panic!("{name} failed its frozen schema/unit/bound check: {e}"))
}

// --------------------------------------------------------------------------- //
// The per-file loaders, each over `&str` and each returning a `Result`.        //
// --------------------------------------------------------------------------- //
//
// ⚠⚠ **The bound guards live in THESE functions, not in their public wrappers, and that
// placement is the whole point.** Stage-3 slice S3 measured the previous arrangement
// (`§5v` measurement 3): `tests::bounds_match_the_loaders` reads as the gate on the five
// files' bound wiring, and deleting `charge()`'s `require_half_open` wrapper left all 488
// workspace tests green. It could not see the deletion because it asserts that the
// *committed values* lie inside their ranges, and deleting a check moves no committed
// value.
//
// A rejection test needs a bad file to hand the loader, and the public entry points read
// `include_str!`-ed constants — there was no runtime path to give them one. So each file
// gets a `*_from(&str)` loader carrying the parse, the unit guard AND the bounds, and the
// `#[cfg(test)]` block below feeds each one a deliberately-bad text. Every rejection is
// paired with the same fixture in range, which is what makes it a control rather than an
// assertion about a typo.

fn charge_from(text: &str, name: &str) -> Result<ChargeParams, ConfigError> {
    let f = ParamFile::parse(text, name)?;
    let v = f.guarded_set(&[("charge_efficiency", "dimensionless")], name)?;
    Ok(ChargeParams {
        charge_efficiency: require_half_open(v[0], 0.0, 1.0, "charge_efficiency", name)?,
    })
}

fn self_discharge_from(text: &str, name: &str) -> Result<SelfDischargeParams, ConfigError> {
    let f = ParamFile::parse(text, name)?;
    let v = f.guarded_set(&[("self_discharge_rate", "1/s")], name)?;
    Ok(SelfDischargeParams {
        self_discharge_rate: require_non_negative(v[0], "self_discharge_rate", name)?,
    })
}

fn thermal_from(text: &str, name: &str) -> Result<ThermalParams, ConfigError> {
    let f = ParamFile::parse(text, name)?;
    let v = f.guarded_set(
        &[
            ("emissivity", "dimensionless"),
            ("radiator_area", "m^2"),
            ("heat_capacity", "J/K"),
            ("space_temperature", "K"),
        ],
        name,
    )?;
    Ok(ThermalParams {
        emissivity: require_half_open(v[0], 0.0, 1.0, "emissivity", name)?,
        radiator_area: require_positive(v[1], "radiator_area", name)?,
        heat_capacity: require_positive(v[2], "heat_capacity", name)?,
        space_temperature: require_non_negative(v[3], "space_temperature", name)?,
    })
}

fn eclss_from(text: &str, name: &str) -> Result<EclssParams, ConfigError> {
    let f = ParamFile::parse(text, name)?;
    let v = f.guarded_set(
        &[
            ("co2_scrub_rate", "1/s"),
            ("condense_rate", "1/s"),
            ("o2_makeup_gain", "1/s"),
            ("o2_setpoint", "mol"),
        ],
        name,
    )?;
    Ok(EclssParams {
        co2_scrub_rate: require_positive(v[0], "co2_scrub_rate", name)?,
        condense_rate: require_positive(v[1], "condense_rate", name)?,
        o2_makeup_gain: require_positive(v[2], "o2_makeup_gain", name)?,
        o2_setpoint: require_positive(v[3], "o2_setpoint", name)?,
    })
}

fn crew_from(text: &str, name: &str) -> Result<CrewParams, ConfigError> {
    let f = ParamFile::parse(text, name)?;
    let v = f.guarded_set(
        &[
            ("respired_carbon_fraction", "dimensionless"),
            ("insensible_water_fraction", "dimensionless"),
        ],
        name,
    )?;
    Ok(CrewParams {
        respired_carbon_fraction: require_closed(v[0], 0.0, 1.0, "respired_carbon_fraction", name)?,
        insensible_water_fraction: require_closed(
            v[1],
            0.0,
            1.0,
            "insensible_water_fraction",
            name,
        )?,
    })
}

/// The Power one-way charge efficiency η_c (`charge.yaml`).
///
/// η ∈ (0, 1]: 1 is lossless charging (the heat leg collapses to 0); 0 would be a
/// battery that stores nothing.
pub fn charge() -> ChargeParams {
    frozen(charge_from(CHARGE_YAML, "charge.yaml"), "charge.yaml")
}

/// The Power first-order self-discharge rate k (`self_discharge.yaml`).
///
/// `k >= 0`: zero is valid (an ideal leak-free cell — inert, the herbivory "a zero rate
/// is valid" precedent), negative is not.
pub fn self_discharge() -> SelfDischargeParams {
    frozen(
        self_discharge_from(SELF_DISCHARGE_YAML, "self_discharge.yaml"),
        "self_discharge.yaml",
    )
}

/// The Thermal radiator properties (`radiator.yaml`).
///
/// ⚠ This is the file carrying `heat_capacity: 1.0e7` — the unsigned-exponent scalar
/// YAML 1.1 resolves as a *string*. See [`config::params`] for why parsing the text is
/// both faithful to pydantic and bit-neutral.
pub fn thermal() -> ThermalParams {
    frozen(
        thermal_from(RADIATOR_YAML, "radiator.yaml"),
        "radiator.yaml",
    )
}

/// The ECLSS control-loop coefficients (`eclss.yaml`). All four are strictly positive.
pub fn eclss() -> EclssParams {
    frozen(eclss_from(ECLSS_YAML, "eclss.yaml"), "eclss.yaml")
}

/// The Crew metabolic-split fractions (`crew.yaml`). Both are fractions in [0, 1].
pub fn crew() -> CrewParams {
    frozen(crew_from(CREW_YAML, "crew.yaml"), "crew.yaml")
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

/// The four directories the sibling census is a census **of** (five files: Power carries
/// two), `(directory, expected count)`.
///
/// Resolved at compile time against this crate's own root, the same as the `include_str!`s
/// above. ⚠ Before Stage-3 slice S1 these pointed into a Python package scheduled for
/// deletion, which made the census a *runtime* dependency on the dying tree as well as a
/// compile-time one. S1 moved the files here; the census now reads the reference's own
/// ground.
pub const PARAM_DIRS: [(&str, usize); 4] = [
    (concat!(env!("CARGO_MANIFEST_DIR"), "/params/power"), 2),
    (concat!(env!("CARGO_MANIFEST_DIR"), "/params/thermal"), 1),
    (concat!(env!("CARGO_MANIFEST_DIR"), "/params/eclss"), 1),
    (concat!(env!("CARGO_MANIFEST_DIR"), "/params/crew"), 1),
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

    // ----------------------------------------------------------------------- //
    // The 23 loader-rejection gates (Stage-3 slice S3).                        //
    // ----------------------------------------------------------------------- //
    //
    // Subject: the loader halves of `tests/test_power_flows.py` (8),
    // `test_thermal_flows.py` (7), `test_eclss_flows.py` (4) and `test_crew_flows.py` (4).
    // They live here rather than beside the flow tests because what they exercise is the
    // *file boundary*, and this is the module that reads the files.
    //
    // ⚠ **Two of these discharge a finding, not just coverage.** §5v measurement 3 found
    // `bounds_match_the_loaders` (below) inert — it cannot see a bound guard vanish — and
    // also found it silent about `eclss.yaml` altogether: the four ECLSS bounds were
    // guarded by nothing on the reference side. `eclss_loader_rejects_*` closes that hole.
    //
    // Each rejection carries its own control: the SAME fixture with the offending value
    // put back in range must load. Without that half, a fixture with a typo in it would
    // "reject" for the wrong reason and read as a passing gate — which is the failure mode
    // this whole section exists to stop reproducing.

    /// One `{value, unit, source}` entry, block style (the frozen files' own shape).
    fn block(name: &str, value: &str, unit: &str) -> String {
        format!("  {name}:\n    value: {value}\n    unit: \"{unit}\"\n    source: \"test\"\n")
    }

    fn charge_yaml(value: &str, unit: &str) -> String {
        format!(
            "name: power\nprocess: charge\nparameters:\n{}",
            block("charge_efficiency", value, unit)
        )
    }

    fn self_discharge_yaml(value: &str, unit: &str) -> String {
        format!(
            "name: power\nprocess: self_discharge\nparameters:\n{}",
            block("self_discharge_rate", value, unit)
        )
    }

    fn radiator_yaml(
        emissivity: (&str, &str),
        area: (&str, &str),
        capacity: (&str, &str),
        space: (&str, &str),
    ) -> String {
        format!(
            "name: thermal\nprocess: radiation\nparameters:\n{}{}{}{}",
            block("emissivity", emissivity.0, emissivity.1),
            block("radiator_area", area.0, area.1),
            block("heat_capacity", capacity.0, capacity.1),
            block("space_temperature", space.0, space.1),
        )
    }

    fn eclss_yaml(
        scrub: (&str, &str),
        condense: (&str, &str),
        makeup: (&str, &str),
        setpoint: (&str, &str),
    ) -> String {
        format!(
            "name: eclss\nprocess: cabin_air_control\nparameters:\n{}{}{}{}",
            block("co2_scrub_rate", scrub.0, scrub.1),
            block("condense_rate", condense.0, condense.1),
            block("o2_makeup_gain", makeup.0, makeup.1),
            block("o2_setpoint", setpoint.0, setpoint.1),
        )
    }

    fn crew_yaml(respired: (&str, &str), insensible: (&str, &str)) -> String {
        format!(
            "name: crew\nprocess: metabolic_split\nparameters:\n{}{}",
            block("respired_carbon_fraction", respired.0, respired.1),
            block("insensible_water_fraction", insensible.0, insensible.1),
        )
    }

    /// Assert `result` is the rejection whose message names `needle`.
    fn rejected<T: std::fmt::Debug>(result: Result<T, ConfigError>, needle: &str) {
        match result {
            Ok(v) => panic!("expected a rejection naming {needle:?}, got {v:?}"),
            Err(e) => assert!(
                e.to_string().contains(needle),
                "rejected, but for the wrong reason: {e} (expected a message naming {needle:?})"
            ),
        }
    }

    // --- charge.yaml (4) ----------------------------------------------------------
    #[test]
    fn charge_loader_reads_the_committed_efficiency() {
        assert_eq!(charge().charge_efficiency, 0.95);
    }

    /// ⚠ **An M-bound site.** Deleting `charge_from`'s `require_half_open` must redden
    /// this test and `charge_loader_rejects_above_one_efficiency` — it is the mutation
    /// with no golden backstop at all, because it moves no committed value.
    #[test]
    fn charge_loader_rejects_zero_efficiency() {
        // 0 = a battery that stores nothing; (0, 1] is required.
        rejected(
            charge_from(&charge_yaml("0.0", "dimensionless"), "charge.yaml"),
            "charge_efficiency must be in",
        );
        // The control: the same fixture in range loads, so the rejection is about the
        // bound and not about the fixture.
        assert_eq!(
            charge_from(&charge_yaml("0.95", "dimensionless"), "charge.yaml")
                .expect("the in-range control loads")
                .charge_efficiency,
            0.95
        );
    }

    #[test]
    fn charge_loader_rejects_above_one_efficiency() {
        // > 1 would create energy on charge.
        rejected(
            charge_from(&charge_yaml("1.5", "dimensionless"), "charge.yaml"),
            "charge_efficiency must be in",
        );
        // ⚠ 1.0 is the legitimate endpoint (lossless charging), so the control here is the
        // boundary itself: a `require_closed` mistaken for `require_half_open` would pass
        // the test above and fail this line.
        assert_eq!(
            charge_from(&charge_yaml("1.0", "dimensionless"), "charge.yaml")
                .expect("η = 1 is lossless, not out of range")
                .charge_efficiency,
            1.0
        );
    }

    #[test]
    fn charge_loader_rejects_a_bad_unit() {
        rejected(
            charge_from(&charge_yaml("0.95", "J"), "charge.yaml"),
            "must be declared in",
        );
        assert!(charge_from(&charge_yaml("0.95", "dimensionless"), "charge.yaml").is_ok());
    }

    // --- self_discharge.yaml (4) --------------------------------------------------
    #[test]
    fn self_discharge_loader_reads_the_committed_rate() {
        assert_eq!(self_discharge().self_discharge_rate, 1.0e-8);
    }

    #[test]
    fn self_discharge_loader_accepts_a_zero_rate() {
        // 0 = an ideal leak-free cell (inert machinery) — valid, the herbivory precedent.
        // `>= 0` is the bound here, unlike the strictly-positive efficiency, so this case
        // is the one that would catch `require_non_negative` being "tightened" to
        // `require_positive`.
        assert_eq!(
            self_discharge_from(&self_discharge_yaml("0.0", "1/s"), "self_discharge.yaml")
                .expect("a zero rate is valid")
                .self_discharge_rate,
            0.0
        );
    }

    /// ⚠ **An M-bound site** (`require_non_negative` in `self_discharge_from`).
    #[test]
    fn self_discharge_loader_rejects_a_negative_rate() {
        // < 0 would CREATE energy on the leak.
        rejected(
            self_discharge_from(
                &self_discharge_yaml("-1.0e-8", "1/s"),
                "self_discharge.yaml",
            ),
            "self_discharge_rate must be >= 0",
        );
        assert!(
            self_discharge_from(&self_discharge_yaml("1.0e-8", "1/s"), "self_discharge.yaml")
                .is_ok()
        );
    }

    #[test]
    fn self_discharge_loader_rejects_a_bad_unit() {
        // Per-second is the exact-guarded unit (Power's natural time unit); /day rejected.
        rejected(
            self_discharge_from(
                &self_discharge_yaml("1.0e-8", "1/day"),
                "self_discharge.yaml",
            ),
            "must be declared in",
        );
        assert!(
            self_discharge_from(&self_discharge_yaml("1.0e-8", "1/s"), "self_discharge.yaml")
                .is_ok()
        );
    }

    // --- radiator.yaml (7) --------------------------------------------------------
    /// The committed fixture, as the control every radiator rejection below re-uses.
    fn good_radiator() -> String {
        radiator_yaml(
            ("0.85", "dimensionless"),
            ("10.0", "m^2"),
            ("1.0e7", "J/K"),
            ("2.7", "K"),
        )
    }

    #[test]
    fn radiator_loader_reads_the_committed_params() {
        let p = thermal();
        assert_eq!(p.emissivity, 0.85);
        assert_eq!(p.radiator_area, 10.0);
        assert_eq!(p.heat_capacity, 1.0e7);
        assert_eq!(p.space_temperature, 2.7);
    }

    /// ⚠ **An M-bound site** (`require_half_open` on `emissivity`).
    #[test]
    fn radiator_loader_rejects_zero_emissivity() {
        // 0 = a surface that radiates nothing, i.e. no rejection path at all.
        let bad = radiator_yaml(
            ("0.0", "dimensionless"),
            ("10.0", "m^2"),
            ("1.0e7", "J/K"),
            ("2.7", "K"),
        );
        rejected(thermal_from(&bad, "radiator.yaml"), "emissivity must be in");
        assert!(thermal_from(&good_radiator(), "radiator.yaml").is_ok());
    }

    #[test]
    fn radiator_loader_rejects_above_one_emissivity() {
        // > 1 would radiate more than a black body.
        let bad = radiator_yaml(
            ("1.5", "dimensionless"),
            ("10.0", "m^2"),
            ("1.0e7", "J/K"),
            ("2.7", "K"),
        );
        rejected(thermal_from(&bad, "radiator.yaml"), "emissivity must be in");
        // ε = 1 is a black body — the legitimate endpoint, as for η_c.
        let black = radiator_yaml(
            ("1.0", "dimensionless"),
            ("10.0", "m^2"),
            ("1.0e7", "J/K"),
            ("2.7", "K"),
        );
        assert_eq!(
            thermal_from(&black, "radiator.yaml")
                .expect("a black body is not out of range")
                .emissivity,
            1.0
        );
    }

    /// ⚠ **An M-bound site** (`require_positive` on `radiator_area`).
    #[test]
    fn radiator_loader_rejects_a_nonpositive_area() {
        let bad = radiator_yaml(
            ("0.85", "dimensionless"),
            ("0.0", "m^2"),
            ("1.0e7", "J/K"),
            ("2.7", "K"),
        );
        rejected(
            thermal_from(&bad, "radiator.yaml"),
            "radiator_area must be > 0",
        );
        assert!(thermal_from(&good_radiator(), "radiator.yaml").is_ok());
    }

    #[test]
    fn radiator_loader_rejects_a_nonpositive_heat_capacity() {
        let bad = radiator_yaml(
            ("0.85", "dimensionless"),
            ("10.0", "m^2"),
            ("0.0", "J/K"),
            ("2.7", "K"),
        );
        rejected(
            thermal_from(&bad, "radiator.yaml"),
            "heat_capacity must be > 0",
        );
        assert!(thermal_from(&good_radiator(), "radiator.yaml").is_ok());
    }

    #[test]
    fn radiator_loader_rejects_a_negative_space_temperature() {
        // Absolute temperature: below 0 K is not a sink, it is a typo.
        let bad = radiator_yaml(
            ("0.85", "dimensionless"),
            ("10.0", "m^2"),
            ("1.0e7", "J/K"),
            ("-1.0", "K"),
        );
        rejected(
            thermal_from(&bad, "radiator.yaml"),
            "space_temperature must be >= 0",
        );
        // 0 K is valid (a perfectly cold sink), which is why the bound is `>= 0`.
        let absolute_zero = radiator_yaml(
            ("0.85", "dimensionless"),
            ("10.0", "m^2"),
            ("1.0e7", "J/K"),
            ("0.0", "K"),
        );
        assert_eq!(
            thermal_from(&absolute_zero, "radiator.yaml")
                .expect("0 K is a valid sink temperature")
                .space_temperature,
            0.0
        );
    }

    #[test]
    fn radiator_loader_rejects_a_bad_unit() {
        // Absolute K is the exact-guarded temperature unit — the T⁴ law needs an absolute
        // scale — so degC is rejected here, unlike the biosphere's degC kinetics.
        let bad = radiator_yaml(
            ("0.85", "dimensionless"),
            ("10.0", "m^2"),
            ("1.0e7", "J/K"),
            ("2.7", "degC"),
        );
        rejected(thermal_from(&bad, "radiator.yaml"), "must be declared in");
        assert!(thermal_from(&good_radiator(), "radiator.yaml").is_ok());
    }

    // --- eclss.yaml (4) — the roster hole §5v measurement 3 found -------------------
    fn good_eclss() -> String {
        eclss_yaml(
            ("1.0e-3", "1/s"),
            ("5.0e-4", "1/s"),
            ("2.0e-3", "1/s"),
            ("10.0", "mol"),
        )
    }

    #[test]
    fn eclss_loader_reads_the_committed_params() {
        let p = eclss();
        assert_eq!(p.co2_scrub_rate, 1.0e-3);
        assert_eq!(p.condense_rate, 5.0e-4);
        assert_eq!(p.o2_makeup_gain, 2.0e-3);
        assert_eq!(p.o2_setpoint, 10.0);
    }

    /// ⚠ **An M-bound site, and the one `bounds_match_the_loaders` never covered at all.**
    #[test]
    fn eclss_loader_rejects_a_nonpositive_rate() {
        // k_scrub = 0 is a scrubber that does not scrub — the loop stops being a loop.
        let bad = eclss_yaml(
            ("0.0", "1/s"),
            ("5.0e-4", "1/s"),
            ("2.0e-3", "1/s"),
            ("10.0", "mol"),
        );
        rejected(eclss_from(&bad, "eclss.yaml"), "co2_scrub_rate must be > 0");
        assert!(eclss_from(&good_eclss(), "eclss.yaml").is_ok());
    }

    #[test]
    fn eclss_loader_rejects_a_nonpositive_setpoint() {
        let bad = eclss_yaml(
            ("1.0e-3", "1/s"),
            ("5.0e-4", "1/s"),
            ("2.0e-3", "1/s"),
            ("-1.0", "mol"),
        );
        rejected(eclss_from(&bad, "eclss.yaml"), "o2_setpoint must be > 0");
        assert!(eclss_from(&good_eclss(), "eclss.yaml").is_ok());
    }

    #[test]
    fn eclss_loader_rejects_a_bad_unit() {
        let bad = eclss_yaml(
            ("1.0e-3", "1/s"),
            ("5.0e-4", "1/min"),
            ("2.0e-3", "1/s"),
            ("10.0", "mol"),
        );
        rejected(
            eclss_from(&bad, "eclss.yaml"),
            "condense_rate must be declared in",
        );
        assert!(eclss_from(&good_eclss(), "eclss.yaml").is_ok());
    }

    // --- crew.yaml (4) ------------------------------------------------------------
    fn good_crew() -> String {
        crew_yaml(("0.949", "dimensionless"), ("0.675", "dimensionless"))
    }

    #[test]
    fn crew_loader_reads_the_committed_params() {
        // BVAD-calibrated (was 0.85 / 0.4 illustrative) — see `docs/bvad-reference.md`.
        let p = crew();
        assert_eq!(p.respired_carbon_fraction, 0.949);
        assert_eq!(p.insensible_water_fraction, 0.675);
    }

    /// ⚠ **An M-bound site** (`require_closed` on `respired_carbon_fraction`).
    #[test]
    fn crew_loader_rejects_an_out_of_range_fraction() {
        let bad = crew_yaml(("1.5", "dimensionless"), ("0.4", "dimensionless"));
        rejected(
            crew_from(&bad, "crew.yaml"),
            "respired_carbon_fraction must be in",
        );
        assert!(crew_from(&good_crew(), "crew.yaml").is_ok());
    }

    #[test]
    fn crew_loader_rejects_a_negative_fraction() {
        let bad = crew_yaml(("0.85", "dimensionless"), ("-0.1", "dimensionless"));
        rejected(
            crew_from(&bad, "crew.yaml"),
            "insensible_water_fraction must be in",
        );
        // ⚠ Both endpoints are legitimate here — the bound is CLOSED, not half-open like
        // η_c and ε: a crew that respires all of its food carbon, or none of it, is a
        // degenerate but well-defined split. This is the control that would catch the
        // bound being copied from `charge_from`.
        let endpoints = crew_yaml(("0.0", "dimensionless"), ("1.0", "dimensionless"));
        let p = crew_from(&endpoints, "crew.yaml").expect("0 and 1 are valid fractions");
        assert_eq!(p.respired_carbon_fraction, 0.0);
        assert_eq!(p.insensible_water_fraction, 1.0);
    }

    #[test]
    fn crew_loader_rejects_a_bad_unit() {
        let bad = crew_yaml(("0.85", "1"), ("0.4", "dimensionless"));
        rejected(
            crew_from(&bad, "crew.yaml"),
            "respired_carbon_fraction must be declared in",
        );
        assert!(crew_from(&good_crew(), "crew.yaml").is_ok());
    }

    /// ⚠⚠ **Measured INERT by Stage-3 slice S3, and kept as the record of that.**
    ///
    /// This reads as the gate on the five files' bound wiring. It is not one: it asserts
    /// that the *committed values* lie inside their ranges, and deleting a loader's bound
    /// check moves no committed value. §5v measurement 3 deleted `charge_from`'s
    /// `require_half_open` and all 488 workspace tests stayed green — the exact
    /// "inert by construction" shape this log keeps recording.
    ///
    /// It is not deleted, because what it *does* assert is still true and cheap: the
    /// committed numbers are in range. It is simply not the gate its name implies. The
    /// gates are the `*_loader_rejects_*` tests above, each of which hands a loader a
    /// deliberately-bad file and carries its own in-range control.
    ///
    /// ⚠ Note what is still missing here and no longer missing above: this test says
    /// nothing about `eclss.yaml` at all.
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
