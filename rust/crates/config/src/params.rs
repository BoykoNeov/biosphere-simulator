//! The param-file boundary: the `{value, unit, source}` schema, the unit guard, and the
//! bound helpers (reference flip, slice C1).
//!
//! # What this replaces, and what it deliberately is not
//!
//! Python's boundary is `config/loader.py` (a safe YAML read) + `config/units.py` (pint)
//! + a hand-written pydantic schema per process in each `domains.*.loader`. This module
//! is the first two, plus the parts of the third that every param file shares.
//!
//! **It is not a units library, and that is a measurement rather than a shortcut.** The
//! slice-C1 census (docs/plans/post-raodmap-reference-flip.md §5d) found that of the ~80
//! frozen coefficients, *every* declared unit is checked by **exact string comparison** —
//! `dimensionless`, `degC`, `1/day`, `J/K`, … — and the only two Python functions that
//! genuinely convert (`config.units.convert` / `to_canonical`) have six live callers
//! between them, **all of which are identities**: each is called with the unit the file
//! already declares. Reimplementing pint to reproduce six no-ops would be ceremony, not
//! validation. What the guard must do — and does — is reject a file whose declared unit
//! is not the one the consuming flow expects.
//!
//! # ⚠ The pyyaml numeric hazard is REAL in this tree, not hypothetical
//!
//! `thermal/params/radiator.yaml` declares `heat_capacity: 1.0e7`. YAML 1.1's resolver
//! requires a **signed** exponent, so pyyaml resolves that as the *string* `'1.0e7'`
//! (as it does `1e7` and `1.0E7`; only `1.0e+7` resolves as a float), and **pydantic
//! coerces it** to `10000000.0`. [`Entry::value`] therefore parses the scalar's *text*
//! as `f64` regardless of how a YAML resolver would have typed it — which is both what
//! pydantic does and what makes the port bit-neutral. See [`crate::yaml`], whose own
//! docstring names this hazard as the reason the reader is hand-rolled.

use crate::errors::ConfigError;
use crate::yaml::{parse_document, YamlValue};

/// One `{value, unit, source}` parameter entry — the structured param template every
/// process file has used since Phase-1 Step 3 (`docs/param-file-conventions.md`).
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// The magnitude, in the unit [`Entry::unit`] declares.
    pub value: f64,
    /// The declared unit, exact-string guarded by the consuming loader.
    pub unit: String,
    /// The clean-room provenance tag. **Recorded, never parsed** — every value cites its
    /// origin or carries a `TODO(cite)` provisional marker.
    pub source: String,
}

/// A parsed param file: `name`, `process`, and its `parameters` block.
///
/// The three top-level keys are exactly the frozen file schema; an extra or missing one
/// is an error, mirroring pydantic's `extra="forbid"`.
#[derive(Debug, Clone, PartialEq)]
pub struct ParamFile {
    /// The `name:` field (e.g. `winter_wheat`) — recorded, not resolved.
    pub name: String,
    /// The `process:` field (e.g. `canopy_light_interception`) — recorded, not resolved.
    pub process: String,
    /// The `parameters:` block, in source order.
    parameters: Vec<(String, YamlValue)>,
}

impl ParamFile {
    /// Parse a param file's text. `context` names the file in every error message.
    pub fn parse(text: &str, context: &str) -> Result<ParamFile, ConfigError> {
        let doc = parse_document(text)?;
        let top = doc.as_mapping(context)?;
        require_keys(top, &["name", "process", "parameters"], context)?;
        Ok(ParamFile {
            name: scalar_text(lookup(top, "name", context)?, context, "name")?.to_string(),
            process: scalar_text(lookup(top, "process", context)?, context, "process")?
                .to_string(),
            parameters: lookup(top, "parameters", context)?
                .as_mapping(context)?
                .to_vec(),
        })
    }

    /// The `parameters` block's keys, in source order.
    pub fn fields(&self) -> Vec<&str> {
        self.parameters.iter().map(|(k, _)| k.as_str()).collect()
    }

    /// The raw value of a `parameters` entry — the escape hatch for the two files whose
    /// block is a table rather than `{value, unit, source}` scalars (`allocation.yaml`).
    pub fn raw(&self, field: &str, context: &str) -> Result<&YamlValue, ConfigError> {
        lookup(&self.parameters, field, context)
    }

    /// Read one `{value, unit, source}` entry, rejecting extra or missing sub-keys.
    pub fn entry(&self, field: &str, context: &str) -> Result<Entry, ConfigError> {
        let where_ = format!("{context}: {field}");
        let map = self.raw(field, context)?.as_mapping(&where_)?;
        require_keys(map, &["value", "unit", "source"], &where_)?;
        let text = scalar_text(lookup(map, "value", &where_)?, &where_, "value")?;
        // ⚠ Parse the TEXT, not a resolver's verdict about it — the `1.0e7` hazard in
        // this module's header. This is exactly pydantic's `float` coercion.
        let value: f64 = text.trim().parse().map_err(|_| {
            ConfigError::new(format!("{where_}: value {text:?} is not a number"))
        })?;
        if !value.is_finite() {
            return Err(ConfigError::new(format!(
                "{where_}: value {text:?} is not finite"
            )));
        }
        Ok(Entry {
            value,
            unit: scalar_text(lookup(map, "unit", &where_)?, &where_, "unit")?.to_string(),
            source: scalar_text(lookup(map, "source", &where_)?, &where_, "source")?
                .to_string(),
        })
    }

    /// Read an entry's value, **exact-string guarding its declared unit** — the check
    /// every frozen param file is actually validated by (see the module header).
    pub fn guarded(
        &self,
        field: &str,
        expected_unit: &str,
        context: &str,
    ) -> Result<f64, ConfigError> {
        let entry = self.entry(field, context)?;
        if entry.unit != expected_unit {
            return Err(ConfigError::new(format!(
                "{context}: {field} must be declared in {expected_unit:?}, got {:?}",
                entry.unit
            )));
        }
        Ok(entry.value)
    }

    /// Read every field of a `field → expected unit` table, guarding each unit and
    /// rejecting any `parameters` key the table does not name.
    ///
    /// This is the `extra="forbid"` half of the pydantic schema: a param added to a file
    /// and wired to nothing must fail here rather than be silently ignored.
    pub fn guarded_set(
        &self,
        units: &[(&str, &str)],
        context: &str,
    ) -> Result<Vec<f64>, ConfigError> {
        let expected: Vec<&str> = units.iter().map(|(f, _)| *f).collect();
        require_keys(&self.parameters, &expected, context)?;
        units
            .iter()
            .map(|(field, unit)| self.guarded(field, unit, context))
            .collect()
    }
}

/// Every key in `entries` is named in `expected`, and every `expected` key is present.
///
/// Order-insensitive and duplicate-sensitive: the reader preserves source order and does
/// not dedupe, so a repeated key would otherwise shadow silently.
fn require_keys(
    entries: &[(String, YamlValue)],
    expected: &[&str],
    context: &str,
) -> Result<(), ConfigError> {
    for (key, _) in entries {
        if !expected.contains(&key.as_str()) {
            return Err(ConfigError::new(format!(
                "{context}: unexpected key {key:?} (expected one of {expected:?})"
            )));
        }
        if entries.iter().filter(|(k, _)| k == key).count() > 1 {
            return Err(ConfigError::new(format!(
                "{context}: duplicate key {key:?}"
            )));
        }
    }
    for key in expected {
        if !entries.iter().any(|(k, _)| k == key) {
            return Err(ConfigError::new(format!("{context}: missing key {key:?}")));
        }
    }
    Ok(())
}

fn lookup<'a>(
    entries: &'a [(String, YamlValue)],
    key: &str,
    context: &str,
) -> Result<&'a YamlValue, ConfigError> {
    entries
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
        .ok_or_else(|| ConfigError::new(format!("{context}: missing key {key:?}")))
}

fn scalar_text<'a>(
    value: &'a YamlValue,
    context: &str,
    field: &str,
) -> Result<&'a str, ConfigError> {
    match value {
        YamlValue::Scalar { text, .. } => Ok(text),
        _ => Err(ConfigError::new(format!(
            "{context}: {field} must be a scalar"
        ))),
    }
}

// --- the bound helpers ------------------------------------------------------
// The loaders' documented ranges, as named checks rather than open-coded `if`s, so a
// bound reads the same in every domain. Each mirrors a Python `raise ValueError`.

/// `value > 0`.
pub fn require_positive(value: f64, field: &str, context: &str) -> Result<f64, ConfigError> {
    if !(value > 0.0) {
        return Err(ConfigError::new(format!(
            "{context}: {field} must be > 0, got {value}"
        )));
    }
    Ok(value)
}

/// `value >= 0`. Distinct from [`require_positive`] on purpose — a zero rate is valid in
/// several places (an ideal leak-free cell, no decomposition) and must not be rejected.
pub fn require_non_negative(
    value: f64,
    field: &str,
    context: &str,
) -> Result<f64, ConfigError> {
    if !(value >= 0.0) {
        return Err(ConfigError::new(format!(
            "{context}: {field} must be >= 0, got {value}"
        )));
    }
    Ok(value)
}

/// `lo <= value <= hi`.
pub fn require_closed(
    value: f64,
    lo: f64,
    hi: f64,
    field: &str,
    context: &str,
) -> Result<f64, ConfigError> {
    if !(lo <= value && value <= hi) {
        return Err(ConfigError::new(format!(
            "{context}: {field} must be in [{lo}, {hi}], got {value}"
        )));
    }
    Ok(value)
}

/// `lo < value <= hi` — the fraction/efficiency shape (zero is a degenerate model, one
/// is lossless and legitimate).
pub fn require_half_open(
    value: f64,
    lo: f64,
    hi: f64,
    field: &str,
    context: &str,
) -> Result<f64, ConfigError> {
    if !(lo < value && value <= hi) {
        return Err(ConfigError::new(format!(
            "{context}: {field} must be in ({lo}, {hi}], got {value}"
        )));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = "\
name: demo_crop
process: demo_process
parameters:
  a_rate:
    value: 0.25
    unit: \"1/day\"
    source: \"[A] table 1\"
";

    fn file() -> ParamFile {
        ParamFile::parse(GOOD, "demo.yaml").expect("parses")
    }

    #[test]
    fn reads_a_guarded_entry() {
        assert_eq!(file().guarded("a_rate", "1/day", "demo.yaml").unwrap(), 0.25);
    }

    #[test]
    fn a_wrong_declared_unit_is_rejected() {
        let err = file().guarded("a_rate", "1/s", "demo.yaml").unwrap_err();
        assert!(err.to_string().contains("must be declared in"), "{err}");
    }

    #[test]
    fn the_provenance_tag_is_required_but_not_parsed() {
        let without = GOOD.replace("    source: \"[A] table 1\"\n", "");
        let err = ParamFile::parse(&without, "demo.yaml")
            .unwrap()
            .entry("a_rate", "demo.yaml")
            .unwrap_err();
        assert!(err.to_string().contains("missing key \"source\""), "{err}");
        // ...and its content is never interpreted: a TODO marker loads fine.
        let todo = GOOD.replace("[A] table 1", "TODO(cite) — provisional");
        assert_eq!(
            ParamFile::parse(&todo, "demo.yaml")
                .unwrap()
                .entry("a_rate", "demo.yaml")
                .unwrap()
                .value,
            0.25
        );
    }

    #[test]
    fn an_unwired_extra_param_is_rejected_not_ignored() {
        // The `extra="forbid"` half: a param added to a file and consumed by nothing.
        let extra = format!("{GOOD}  b_rate:\n    value: 1.0\n    unit: \"1/day\"\n    source: \"x\"\n");
        let err = ParamFile::parse(&extra, "demo.yaml")
            .unwrap()
            .guarded_set(&[("a_rate", "1/day")], "demo.yaml")
            .unwrap_err();
        assert!(err.to_string().contains("unexpected key \"b_rate\""), "{err}");
    }

    #[test]
    fn an_unsigned_exponent_is_a_number_here_even_though_yaml_calls_it_a_string() {
        // ⚠ The live hazard: radiator.yaml's `1.0e7`. pyyaml resolves it as a str and
        // pydantic coerces it; we parse the text, which is the same answer and the same
        // bits. All four spellings must load identically.
        for spelling in ["1.0e7", "1.0e+7", "1e7", "1.0E7"] {
            let text = GOOD.replace("0.25", spelling);
            let got = ParamFile::parse(&text, "radiator.yaml")
                .unwrap()
                .guarded("a_rate", "1/day", "radiator.yaml")
                .unwrap();
            assert_eq!(got, 1.0e7, "spelling {spelling}");
        }
    }

    #[test]
    fn a_non_numeric_value_is_rejected() {
        let text = GOOD.replace("0.25", "\"not a number\"");
        let err = ParamFile::parse(&text, "demo.yaml")
            .unwrap()
            .entry("a_rate", "demo.yaml")
            .unwrap_err();
        assert!(err.to_string().contains("is not a number"), "{err}");
    }

    #[test]
    fn bounds_reject_what_the_loaders_reject() {
        assert!(require_positive(0.0, "k", "f").is_err());
        assert!(require_non_negative(0.0, "k", "f").is_ok());
        assert!(require_non_negative(-1e-18, "k", "f").is_err());
        assert!(require_half_open(0.0, 0.0, 1.0, "eta", "f").is_err());
        assert!(require_half_open(1.0, 0.0, 1.0, "eta", "f").is_ok());
        assert!(require_closed(0.0, 0.0, 1.0, "f", "f").is_ok());
        assert!(require_closed(1.5, 0.0, 1.0, "f", "f").is_err());
    }
}
