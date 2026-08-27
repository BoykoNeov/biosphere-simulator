//! The param-file boundary: the `{value, unit, source}` schema, the unit guard, and the
//! bound helpers (reference flip, slice C1).
//!
//! # What this replaces, and what it deliberately is not
//!
//! Python's boundary is three things: `config/loader.py` (a safe YAML read),
//! `config/units.py` (pint), and a hand-written pydantic schema per process inside each
//! `domains.*.loader`. This module is the first two, plus the parts of the third that
//! every param file shares.
//!
//! **It is not a units library, and that is a measurement rather than a shortcut.** The
//! slice-C1 census (docs/plans/post-roadmap-reference-flip.md §5d) found that of the ~80
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

/// `value > 0` — **and not NaN**.
///
/// ⚠ Written as "is it Greater?" rather than `value <= 0.0` on purpose: the two differ on
/// NaN, which the second form would silently accept. That mirrors Python's
/// `if not value > 0.0`, which rejects NaN for the same reason. [`ParamFile::entry`]
/// already refuses a non-finite value, so this is belt-and-braces at the boundary — but
/// these helpers are public, and a bound that lets NaN through is not a bound.
pub fn require_positive(value: f64, field: &str, context: &str) -> Result<f64, ConfigError> {
    if value.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
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
    if !matches!(
        value.partial_cmp(&0.0),
        Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)
    ) {
        return Err(ConfigError::new(format!(
            "{context}: {field} must be >= 0, got {value}"
        )));
    }
    Ok(value)
}

/// `lo <= value <= hi` — **and not NaN**.
///
/// ⚠ Like [`require_positive`], the negation wraps the comparison rather than inverting
/// it: `value < lo || value > hi` would **accept NaN**. Clippy does not flag this one only
/// because the negation covers a compound expression — do not "simplify" it into the
/// inverted form.
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
/// is lossless and legitimate) — **and not NaN**.
///
/// ⚠ Same as [`require_closed`]: the negation wraps the comparison deliberately, and the
/// inverted form would admit NaN. Clippy is silent here for the same structural reason.
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

/// Rewrite one `{value, unit, source}` entry's **value** in a param file's TEXT.
///
/// # Why a text rewrite rather than a struct edit
///
/// This is the substitution half of the value-switch seam
/// (`docs/plans/post-roadmap-value-switch-harness.md`). Handing the result back through the
/// ordinary loader means an experimental value passes the *same* schema, exact-string unit
/// guard, frozen bounds and boundary folds as a committed one. The Python harness this
/// replaces edited the already-constructed dataclass and so bypassed all four: an
/// out-of-range experimental value ran silently. Here it cannot.
///
/// # ⚠ It writes nothing to disk, and that is the plan's whole safety property
///
/// The input is the `include_str!`-ed text; the output is a `String` that lives for one run.
/// No file is touched, so no per-file digest in any manifest can move.
///
/// # ⚠ A silent no-op is impossible by construction
///
/// The harness's structural failure mode is a substitution that quietly misses (§7 of the
/// plan; `cc44b41` is this tree's own instance). Three things make that unrepresentable
/// here: the field must parse as a `{value, unit, source}` entry **before** the rewrite,
/// **exactly one** line may change, and the result is re-parsed and the new value compared
/// **bit for bit** with the one asked for. Any of the three failing is an `Err`, never a
/// quietly-unchanged file.
pub fn with_override(
    text: &str,
    field: &str,
    value: f64,
    context: &str,
) -> Result<String, ConfigError> {
    if !value.is_finite() {
        return Err(ConfigError::new(format!(
            "{context}: override {field} = {value} is not finite"
        )));
    }
    // Rejects a misspelled field, a field that is a table rather than a scalar entry
    // (`allocation.yaml`'s partition rows), and a malformed file — before anything is
    // rewritten.
    ParamFile::parse(text, context)?.entry(field, context)?;

    let key_line = format!("  {field}:");
    let mut out: Vec<String> = Vec::with_capacity(text.lines().count());
    let mut in_field = false;
    let mut rewritten = 0usize;
    for line in text.lines() {
        if line == key_line {
            in_field = true;
            out.push(line.to_string());
            continue;
        }
        if in_field {
            if let Some(rest) = line.strip_prefix("    value:") {
                let _ = rest;
                out.push(format!("    value: {value:?}"));
                rewritten += 1;
                in_field = false;
                continue;
            }
            // The entry's own sub-keys are indented further; anything else ends it.
            if !line.starts_with("    ") {
                in_field = false;
            }
        }
        out.push(line.to_string());
    }
    if rewritten != 1 {
        return Err(ConfigError::new(format!(
            "{context}: rewriting {field} touched {rewritten} lines, expected exactly 1"
        )));
    }
    let mut result = out.join("\n");
    if text.ends_with('\n') {
        result.push('\n');
    }

    // The proof, not a convention: re-read the rewritten text through the real parser and
    // require the value to be the one asked for, bit for bit.
    let got = ParamFile::parse(&result, context)?.entry(field, context)?.value;
    if got.to_bits() != value.to_bits() {
        return Err(ConfigError::new(format!(
            "{context}: {field} re-read as {got:?} after an override to {value:?}"
        )));
    }
    Ok(result)
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

    /// ⚠ **All four bounds reject NaN, and this is the test that makes that a fact rather
    /// than a doc comment.** Each is written with the negation *wrapping* the comparison
    /// (`!(value > 0.0)`, not `value <= 0.0`) precisely because the inverted form admits
    /// NaN — and clippy actively suggests the inverted form for two of them. Without this
    /// test, taking that suggestion would widen every bound in the tree and turn nothing
    /// red.
    #[test]
    fn every_bound_rejects_nan() {
        let nan = f64::NAN;
        assert!(require_positive(nan, "k", "f").is_err());
        assert!(require_non_negative(nan, "k", "f").is_err());
        assert!(require_closed(nan, 0.0, 1.0, "f", "f").is_err());
        assert!(require_half_open(nan, 0.0, 1.0, "eta", "f").is_err());
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

    // ------------------------------------------------------------------ //
    // The value-switch substitution                                       //
    // ------------------------------------------------------------------ //

    const TWO: &str = "\
name: demo_crop
process: demo_process
parameters:
  a_rate:
    value: 0.25
    unit: \"1/day\"
    source: \"[A] table 1\"
  b_rate:
    value: 4.0
    unit: \"1/day\"
    source: \"[A] table 2\"
";

    #[test]
    fn an_override_moves_the_value_and_nothing_else() {
        let out = with_override(TWO, "a_rate", 0.65, "demo.yaml").expect("override");
        let f = ParamFile::parse(&out, "demo.yaml").expect("re-parses");
        assert_eq!(f.entry("a_rate", "demo.yaml").unwrap().value, 0.65);
        // The other entry, the units, the sources and the file's shape are untouched.
        assert_eq!(f.entry("b_rate", "demo.yaml").unwrap().value, 4.0);
        assert_eq!(f.entry("a_rate", "demo.yaml").unwrap().unit, "1/day");
        assert_eq!(f.entry("a_rate", "demo.yaml").unwrap().source, "[A] table 1");
        assert_eq!(out.lines().count(), TWO.lines().count());
        let changed: Vec<_> = TWO
            .lines()
            .zip(out.lines())
            .filter(|(a, b)| a != b)
            .collect();
        assert_eq!(changed.len(), 1, "{changed:?}");
    }

    /// ⚠ The `1.0e7` hazard from the module header, applied to the OUTPUT side: an
    /// override must be written in a form the reader's text parse reads back identically.
    /// `{:?}` on `f64` is the shortest round-tripping form, and the check is bit equality.
    #[test]
    fn an_override_round_trips_awkward_magnitudes() {
        for v in [1.0e7, 1e-7, 0.1 + 0.2, 6.02214076e23, -3.5, 0.0] {
            let out = with_override(TWO, "a_rate", v, "demo.yaml").expect("override");
            let got = ParamFile::parse(&out, "demo.yaml")
                .unwrap()
                .entry("a_rate", "demo.yaml")
                .unwrap()
                .value;
            assert_eq!(got.to_bits(), v.to_bits(), "{v:?} round-tripped as {got:?}");
        }
    }

    /// The §7 guard: a substitution that misses its target is an error, never a quiet
    /// no-change that a harness would report as "this parameter does not matter".
    #[test]
    fn a_missed_substitution_is_loud() {
        for bad in ["a_rat", "A_RATE", "value", "parameters", ""] {
            assert!(
                with_override(TWO, bad, 1.0, "demo.yaml").is_err(),
                "{bad:?} was accepted as a field"
            );
        }
    }

    #[test]
    fn a_non_finite_override_is_rejected() {
        assert!(with_override(TWO, "a_rate", f64::NAN, "demo.yaml").is_err());
        assert!(with_override(TWO, "a_rate", f64::INFINITY, "demo.yaml").is_err());
    }

    /// A table-shaped entry (`allocation.yaml`'s partition rows) has no single `value:`
    /// line, so it must be refused rather than half-rewritten.
    #[test]
    fn a_table_entry_is_refused() {
        let table = "\
name: demo_crop
process: demo_process
parameters:
  rows:
    - dvs: 0.0
      leaf: 0.5
";
        assert!(with_override(table, "rows", 1.0, "demo.yaml").is_err());
    }
}
