//! A hand-rolled reader for the **closed YAML subset** the scenario-file schema uses
//! (Phase 9, Step 4b / decision E).
//!
//! # Why hand-rolled (decision E, USER-CONFIRMED)
//!
//! Runtime scenario-file parsing forces a Rust YAML dependency — Phase 7 chose
//! Option-C hex-float bundles *precisely to avoid one*, but authored files are
//! arbitrary player/modder data that no pre-lowered bundle can carry. The choice is a
//! vetted crate vs a hand-rolled closed-subset parser; this project takes the
//! hand-rolled path. The load-bearing reason is **not** dep-avoidance (`serde_yaml` is
//! deprecated, but that is secondary) — it is the **parse-parity boundary**: a crate
//! would force reconciling *two independent YAML-1.1 implementations* (the crate's and
//! pyyaml's) with their edge-case divergences (the `1.0e7`-is-a-string hazard, decimal
//! rounding). A parser over a *documented, bounded* subset collapses that to **one
//! grammar we own on both sides** — exactly the S-expr / hex-float / Option-C ethos.
//!
//! # The accepted subset (Step-7 freeze material — keep this in sync with the schema)
//!
//! * **Block mappings** — `key: value` (a scalar value) or `key:` on its own line
//!   followed by a nested block **strictly more indented**.
//! * **Block sequences** — `- item` lines; each item is a scalar, or a mapping whose
//!   first entry rides the `- ` line and whose continuation entries align under it.
//! * **Scalars** — bare (`3600.0`, `crew.food_store`, `carbon`), single-quoted
//!   (`'…'`), or double-quoted (`"…"`). Numeric typing is deferred to the schema (a
//!   field knows whether it wants a number, a string, or the `number|expr` union).
//! * **Comments** — `#` to end-of-line when at line start or preceded by whitespace,
//!   never inside a quoted scalar.
//! * **Indentation** — spaces only (a tab in indentation is an error, as in YAML); a
//!   nested block must be *strictly* more indented than its parent key/dash.
//!
//! # Deliberately excluded (a file using these is an [`AuthoringError`], not silently
//! mis-parsed)
//!
//! Anchors/aliases (`&`/`*`), tags (`!!…`), flow style (`{a: 1}` / `[1, 2]`),
//! multi-line scalars (`|` / `>`), document markers (`---`/`...`), the YAML-1.1 bool
//! aliases (`yes`/`no`/`on`/`off` — only `true`/`false` are recognised, and only where
//! a bool field is expected), and merge keys (`<<`). The scenario files use none of
//! these; excluding them keeps the grammar finite and the parse-parity risk zero.

use crate::errors::AuthoringError;

/// A parsed YAML value in the closed subset: a scalar, a block mapping, or a block
/// sequence. Mappings preserve source order (the schema checks key uniqueness /
/// unknown keys); a scalar keeps whether it was quoted (the `number|expr` union in the
/// schema needs it — a quoted `"1.0"` is a string, a bare `1.0` is a number, mirroring
/// pyyaml).
#[derive(Debug, Clone, PartialEq)]
pub enum YamlValue {
    /// A scalar leaf. `text` is the content with any surrounding quotes stripped;
    /// `quoted` is true iff the source form was single- or double-quoted.
    Scalar { text: String, quoted: bool },
    /// A block mapping (ordered `key -> value`).
    Mapping(Vec<(String, YamlValue)>),
    /// A block sequence.
    Sequence(Vec<YamlValue>),
}

impl YamlValue {
    /// The mapping entries, or an error naming `context` if this is not a mapping.
    pub fn as_mapping(&self, context: &str) -> Result<&[(String, YamlValue)], AuthoringError> {
        match self {
            YamlValue::Mapping(entries) => Ok(entries),
            _ => Err(AuthoringError::new(format!("{context}: expected a mapping"))),
        }
    }

    /// The sequence items, or an error naming `context` if this is not a sequence.
    pub fn as_sequence(&self, context: &str) -> Result<&[YamlValue], AuthoringError> {
        match self {
            YamlValue::Sequence(items) => Ok(items),
            _ => Err(AuthoringError::new(format!("{context}: expected a sequence"))),
        }
    }

    /// The scalar `(text, quoted)`, or an error naming `context` if this is not a
    /// scalar.
    pub fn as_scalar(&self, context: &str) -> Result<(&str, bool), AuthoringError> {
        match self {
            YamlValue::Scalar { text, quoted } => Ok((text.as_str(), *quoted)),
            _ => Err(AuthoringError::new(format!("{context}: expected a scalar"))),
        }
    }
}

/// One logical (comment-and-blank-stripped) source line, with its dash-list structure
/// resolved: a `- ` prefix is normalised away so `content` is the entry riding the
/// dash and `indent` is the column that entry (and its mapping-continuation siblings)
/// sits at.
#[derive(Debug)]
struct Line {
    /// The column `content` starts at (spaces before it, counting a `- ` prefix).
    indent: usize,
    /// True iff this line began a sequence item (`- …`).
    dash: bool,
    /// For a dash line, the column of the `-` itself (the sequence's own indent).
    dash_indent: usize,
    /// The line content (after indentation and any `- `), trailing comment removed.
    content: String,
    /// 1-based source line number (for error messages).
    lineno: usize,
}

/// Parse a whole scenario-file text into a [`YamlValue`] (a top-level mapping).
///
/// The reader is total over the closed subset and returns an [`AuthoringError`] on
/// anything outside it (a tab in indentation, a bad `- `/`key:` shape, an unterminated
/// quote, trailing junk after a dedent). An empty document is an error (a scenario
/// needs at least `name`/`stocks`/`flows`).
pub fn parse_document(text: &str) -> Result<YamlValue, AuthoringError> {
    let lines = tokenize(text)?;
    if lines.is_empty() {
        return Err(AuthoringError::new("empty scenario document"));
    }
    let mut cursor = 0usize;
    let root_indent = lines[0].indent;
    if lines[0].dash {
        return Err(AuthoringError::new(
            "scenario document must be a mapping (top-level '-' sequence not allowed)",
        ));
    }
    let value = parse_mapping(&lines, &mut cursor, root_indent)?;
    if cursor != lines.len() {
        let line = &lines[cursor];
        return Err(AuthoringError::new(format!(
            "line {}: unexpected indentation / trailing content {:?}",
            line.lineno, line.content
        )));
    }
    Ok(value)
}

/// Split the source into [`Line`]s, dropping blank and comment-only lines and
/// resolving each `- ` dash prefix into `(dash, dash_indent, content-indent)`.
fn tokenize(text: &str) -> Result<Vec<Line>, AuthoringError> {
    let mut lines = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let lineno = i + 1;
        let stripped = strip_comment(raw, lineno)?;
        if stripped.trim().is_empty() {
            continue;
        }
        if stripped.contains('\t') && stripped.len() != stripped.trim_start().len() {
            // A tab anywhere in the indentation is an error (YAML forbids it).
            let indent_part = &stripped[..stripped.len() - stripped.trim_start().len()];
            if indent_part.contains('\t') {
                return Err(AuthoringError::new(format!(
                    "line {lineno}: tab in indentation is not allowed (use spaces)"
                )));
            }
        }
        let indent = stripped.len() - stripped.trim_start().len();
        let rest = &stripped[indent..];
        if let Some(after_dash) = rest.strip_prefix('-') {
            // A sequence-item line: `- content` (or bare `-`).
            let content = after_dash.trim_start();
            let content_indent = indent + (rest.len() - content.len());
            lines.push(Line {
                indent: content_indent,
                dash: true,
                dash_indent: indent,
                content: content.to_string(),
                lineno,
            });
        } else {
            lines.push(Line {
                indent,
                dash: false,
                dash_indent: indent,
                content: rest.to_string(),
                lineno,
            });
        }
    }
    Ok(lines)
}

/// Truncate a raw line at its comment, honouring quotes and the "whitespace before
/// `#`" YAML rule. A `#` inside a single- or double-quoted scalar, or one glued to a
/// non-space token (`a#b`), is *not* a comment.
fn strip_comment(raw: &str, lineno: usize) -> Result<String, AuthoringError> {
    let mut in_single = false;
    let mut in_double = false;
    let mut prev_ws = true; // start-of-line counts as "whitespace before"
    for (idx, ch) in raw.char_indices() {
        match ch {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            '#' if !in_single && !in_double && prev_ws => {
                return Ok(raw[..idx].to_string());
            }
            _ => {}
        }
        prev_ws = ch == ' ' || ch == '\t';
    }
    // A quote left open at end-of-line is malformed (multi-line scalars are excluded).
    if in_single || in_double {
        return Err(AuthoringError::new(format!(
            "line {lineno}: unterminated quoted scalar"
        )));
    }
    Ok(raw.to_string())
}

/// Parse a block mapping: consecutive non-dash lines at exactly `indent`, plus a dash
/// line at `indent` only when it opens the mapping's *first* entry (a sequence item's
/// riding entry). Each entry is `key: scalar` or `key:` + a strictly-deeper block.
fn parse_mapping(
    lines: &[Line],
    cursor: &mut usize,
    indent: usize,
) -> Result<YamlValue, AuthoringError> {
    let mut entries: Vec<(String, YamlValue)> = Vec::new();
    let first = *cursor;
    while *cursor < lines.len() {
        let line = &lines[*cursor];
        if line.indent != indent {
            break;
        }
        // A dash line at this indent belongs to an *enclosing* sequence, not this
        // mapping — unless it is the very first line (the sequence handed us the
        // item's riding entry, which is a dash line by construction).
        if line.dash && *cursor != first {
            break;
        }
        let (key, value) = parse_entry(lines, cursor, indent)?;
        if entries.iter().any(|(k, _)| *k == key) {
            return Err(AuthoringError::new(format!(
                "line {}: duplicate key {key:?}",
                line.lineno
            )));
        }
        entries.push((key, value));
    }
    Ok(YamlValue::Mapping(entries))
}

/// Parse one `key: …` mapping entry at `indent`, advancing `cursor` past it (and any
/// nested block it owns).
fn parse_entry(
    lines: &[Line],
    cursor: &mut usize,
    indent: usize,
) -> Result<(String, YamlValue), AuthoringError> {
    let line = &lines[*cursor];
    let lineno = line.lineno;
    let (key, rest) = split_key(&line.content, lineno)?;
    *cursor += 1;
    if !rest.is_empty() {
        // Inline scalar value.
        let value = parse_scalar(&rest, lineno)?;
        return Ok((key, value));
    }
    // `key:` with an empty value → a nested block strictly deeper, else it is an error
    // (the closed subset has no implicit-null value; every key carries a value).
    if *cursor >= lines.len() || lines[*cursor].indent <= indent {
        return Err(AuthoringError::new(format!(
            "line {lineno}: key {key:?} has no value (a nested block must be indented \
             deeper, an inline scalar must follow the ':')"
        )));
    }
    let child = parse_block(lines, cursor)?;
    Ok((key, child))
}

/// Parse a nested block (mapping or sequence) starting at `cursor`, dispatching on
/// whether the first line is a dash line. The block's indent is taken from that first
/// line.
fn parse_block(lines: &[Line], cursor: &mut usize) -> Result<YamlValue, AuthoringError> {
    let line = &lines[*cursor];
    if line.dash {
        parse_sequence(lines, cursor, line.dash_indent)
    } else {
        parse_mapping(lines, cursor, line.indent)
    }
}

/// Parse a block sequence: consecutive dash lines whose `-` sits at `dash_indent`.
/// Each item is the value riding its `- ` line — a scalar, or (the common case) a
/// mapping whose riding entry is the dash line and whose continuation entries align at
/// the item's content indent.
fn parse_sequence(
    lines: &[Line],
    cursor: &mut usize,
    dash_indent: usize,
) -> Result<YamlValue, AuthoringError> {
    let mut items: Vec<YamlValue> = Vec::new();
    while *cursor < lines.len() {
        let line = &lines[*cursor];
        if !line.dash || line.dash_indent != dash_indent {
            break;
        }
        let content_indent = line.indent;
        if line.content.is_empty() {
            // `-` alone: the item is a nested block on the following deeper lines.
            *cursor += 1;
            if *cursor >= lines.len() || lines[*cursor].indent <= dash_indent {
                return Err(AuthoringError::new(format!(
                    "line {}: empty sequence item (a nested block must follow, indented \
                     deeper than the '-')",
                    line.lineno
                )));
            }
            items.push(parse_block(lines, cursor)?);
        } else if is_entry(&line.content) {
            // The item is a mapping; its riding entry is this dash line, continuation
            // entries are non-dash lines at content_indent. parse_mapping consumes the
            // dash line (it is the first line) then the aligned continuation entries.
            items.push(parse_mapping(lines, cursor, content_indent)?);
        } else {
            // A scalar sequence item.
            let value = parse_scalar(&line.content, line.lineno)?;
            *cursor += 1;
            items.push(value);
        }
    }
    Ok(YamlValue::Sequence(items))
}

/// Does `content` look like the start of a `key: …` mapping entry (vs a bare scalar
/// sequence item)? True iff it has a `key:` head outside quotes.
fn is_entry(content: &str) -> bool {
    split_key(content, 0).is_ok()
}

/// Split `content` into `(key, rest)` at the first `:` that is followed by a space or
/// end-of-line and is not inside a quoted scalar. `rest` is trimmed; empty means the
/// value is a nested block.
fn split_key(content: &str, lineno: usize) -> Result<(String, String), AuthoringError> {
    let mut in_single = false;
    let mut in_double = false;
    let bytes = content.as_bytes();
    for (idx, ch) in content.char_indices() {
        match ch {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            ':' if !in_single && !in_double => {
                let next = bytes.get(idx + 1).copied();
                if next.is_none() || next == Some(b' ') {
                    let key = content[..idx].trim();
                    let rest = content[idx + 1..].trim();
                    if key.is_empty() {
                        return Err(AuthoringError::new(format!(
                            "line {lineno}: empty mapping key in {content:?}"
                        )));
                    }
                    return Ok((unquote_key(key), rest.to_string()));
                }
            }
            _ => {}
        }
    }
    Err(AuthoringError::new(format!(
        "line {lineno}: expected 'key: value' or 'key:' in {content:?}"
    )))
}

/// A mapping key may itself be quoted (rare, but pyyaml allows it); strip a matching
/// pair. Keys in the scenario files are all bare dotted identifiers.
fn unquote_key(key: &str) -> String {
    if (key.starts_with('"') && key.ends_with('"') && key.len() >= 2)
        || (key.starts_with('\'') && key.ends_with('\'') && key.len() >= 2)
    {
        key[1..key.len() - 1].to_string()
    } else {
        key.to_string()
    }
}

/// Parse a scalar token into a [`YamlValue::Scalar`], stripping a surrounding quote
/// pair and recording whether it was quoted (the schema's `number|expr` union needs
/// it). A `#`-comment has already been removed by [`strip_comment`].
fn parse_scalar(token: &str, lineno: usize) -> Result<YamlValue, AuthoringError> {
    let token = token.trim();
    if token.len() >= 2 && token.starts_with('"') && token.ends_with('"') {
        return Ok(YamlValue::Scalar {
            text: token[1..token.len() - 1].to_string(),
            quoted: true,
        });
    }
    if token.len() >= 2 && token.starts_with('\'') && token.ends_with('\'') {
        return Ok(YamlValue::Scalar {
            text: token[1..token.len() - 1].to_string(),
            quoted: true,
        });
    }
    if token.starts_with('"') || token.starts_with('\'') {
        return Err(AuthoringError::new(format!(
            "line {lineno}: unterminated / malformed quoted scalar {token:?}"
        )));
    }
    // Reject the excluded flow-style / anchor / tag forms explicitly rather than
    // treating them as bare strings (they would silently mis-parse otherwise).
    if let Some(first) = token.chars().next() {
        if "{}[]&*!|>".contains(first) {
            return Err(AuthoringError::new(format!(
                "line {lineno}: scalar {token:?} uses an excluded YAML feature \
                 (flow-style / anchor / tag / block-scalar are not in the subset)"
            )));
        }
    }
    Ok(YamlValue::Scalar {
        text: token.to_string(),
        quoted: false,
    })
}

// --------------------------------------------------------------------------- #
// Scalar typing helpers (the schema uses these; the YAML-1.1 numeric rule lives here) #
// --------------------------------------------------------------------------- #

/// Does a **bare** scalar text denote a YAML-1.1 number (int or float)? This is the
/// one place the `1.0e7`-is-a-string hazard is pinned: a float must contain a `.`, and
/// an exponent (if present) must carry an explicit sign — exactly pyyaml's YAML-1.1
/// float resolver. A dotless `1e-3` is therefore **not** a number (it is a string), so
/// scenario files write every float with a decimal point (the documented discipline).
pub fn is_yaml_number(text: &str) -> bool {
    is_yaml_int(text) || is_yaml_float(text)
}

/// A YAML-1.1 integer: optional sign, then one or more digits.
fn is_yaml_int(text: &str) -> bool {
    let body = text.strip_prefix(['+', '-']).unwrap_or(text);
    !body.is_empty() && body.bytes().all(|b| b.is_ascii_digit())
}

/// A YAML-1.1 float (the strict resolver): optional sign, a mantissa **containing a
/// `.`** with at least one digit, then an optional `[eE][+-]DIGITS` exponent (the sign
/// is mandatory). `1.0`, `.5`, `5.`, `1.0e+7`, `5.0e-4` are floats; `1e7`, `1.0e7`,
/// `1` (that is an int), `1.` with no digits at all are not.
fn is_yaml_float(text: &str) -> bool {
    let body = text.strip_prefix(['+', '-']).unwrap_or(text);
    let (mantissa, exponent) = match body.split_once(['e', 'E']) {
        Some((m, e)) => (m, Some(e)),
        None => (body, None),
    };
    // Mantissa must contain a dot and at least one digit overall.
    let Some((int_part, frac_part)) = mantissa.split_once('.') else {
        return false;
    };
    if !int_part.bytes().all(|b| b.is_ascii_digit()) || !frac_part.bytes().all(|b| b.is_ascii_digit())
    {
        return false;
    }
    if int_part.is_empty() && frac_part.is_empty() {
        return false; // a lone "."
    }
    match exponent {
        None => true,
        Some(exp) => {
            // Exponent sign is mandatory in the strict YAML-1.1 resolver.
            let signed = exp.strip_prefix(['+', '-']);
            match signed {
                Some(digits) => !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()),
                None => false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scalar(text: &str, quoted: bool) -> YamlValue {
        YamlValue::Scalar {
            text: text.to_string(),
            quoted,
        }
    }

    #[test]
    fn flat_mapping() {
        let doc = parse_document("name: crew\ndt: 3600.0\n").unwrap();
        assert_eq!(
            doc,
            YamlValue::Mapping(vec![
                ("name".to_string(), scalar("crew", false)),
                ("dt".to_string(), scalar("3600.0", false)),
            ])
        );
    }

    #[test]
    fn nested_mapping_needs_deeper_indent() {
        let doc = parse_document("a:\n  b: 1\n  c: 2\n").unwrap();
        let entries = doc.as_mapping("root").unwrap();
        let inner = entries[0].1.as_mapping("a").unwrap();
        assert_eq!(inner.len(), 2);
        assert_eq!(inner[0].0, "b");
    }

    #[test]
    fn sequence_of_mappings_riding_the_dash() {
        // The stocks/flows shape: a `- id: …` riding entry + aligned continuation.
        let text = "stocks:\n  - id: a\n    amount: 1.0\n  - id: b\n    amount: 2.0\n";
        let doc = parse_document(text).unwrap();
        let seq = doc.as_mapping("root").unwrap()[0].1.as_sequence("stocks").unwrap();
        assert_eq!(seq.len(), 2);
        let first = seq[0].as_mapping("item").unwrap();
        assert_eq!(first[0], ("id".to_string(), scalar("a", false)));
        assert_eq!(first[1], ("amount".to_string(), scalar("1.0", false)));
        let second = seq[1].as_mapping("item").unwrap();
        assert_eq!(second[0], ("id".to_string(), scalar("b", false)));
    }

    #[test]
    fn trailing_comment_is_stripped_but_not_inside_quotes() {
        let doc = parse_document("a: 1.0   # a comment\nb: \"x # y\"\n").unwrap();
        let entries = doc.as_mapping("root").unwrap();
        assert_eq!(entries[0].1, scalar("1.0", false));
        // The '#' inside the double-quoted scalar is preserved.
        assert_eq!(entries[1].1, scalar("x # y", true));
    }

    #[test]
    fn quoted_scalars_record_quotedness() {
        let doc = parse_document("a: 'x'\nb: \"y\"\nc: z\n").unwrap();
        let e = doc.as_mapping("root").unwrap();
        assert_eq!(e[0].1, scalar("x", true));
        assert_eq!(e[1].1, scalar("y", true));
        assert_eq!(e[2].1, scalar("z", false));
    }

    #[test]
    fn template_expression_value_with_star_and_inner_quotes() {
        // The template amount form: a double-quoted expression containing single quotes.
        let doc = parse_document("amount: \"param('crew_count') * 1000.0\"\n").unwrap();
        assert_eq!(
            doc.as_mapping("root").unwrap()[0].1,
            scalar("param('crew_count') * 1000.0", true)
        );
    }

    #[test]
    fn negative_and_int_stoichiometry_values() {
        let doc = parse_document("stoich:\n  a: -1\n  b: 1\n").unwrap();
        let inner = doc.as_mapping("root").unwrap()[0].1.as_mapping("stoich").unwrap();
        assert_eq!(inner[0].1, scalar("-1", false));
        assert_eq!(inner[1].1, scalar("1", false));
    }

    #[test]
    fn tab_in_indentation_is_rejected() {
        assert!(parse_document("a:\n\tb: 1\n").is_err());
    }

    #[test]
    fn unterminated_quote_is_rejected() {
        assert!(parse_document("a: 'oops\n").is_err());
    }

    #[test]
    fn flow_style_is_rejected() {
        assert!(parse_document("a: {b: 1}\n").is_err());
        assert!(parse_document("a: [1, 2]\n").is_err());
    }

    #[test]
    fn empty_document_is_rejected() {
        assert!(parse_document("\n# only a comment\n").is_err());
    }

    #[test]
    fn top_level_sequence_is_rejected() {
        assert!(parse_document("- a\n- b\n").is_err());
    }

    #[test]
    fn duplicate_key_is_rejected() {
        assert!(parse_document("a: 1\na: 2\n").is_err());
    }

    // --- the YAML-1.1 numeric rule (the `1.0e7`-is-a-string hazard) ---------
    #[test]
    fn yaml_number_classification() {
        // Integers.
        assert!(is_yaml_number("0"));
        assert!(is_yaml_number("168"));
        assert!(is_yaml_number("-1"));
        assert!(is_yaml_number("+1"));
        // Floats WITH a dot (and signed exponents).
        assert!(is_yaml_number("3600.0"));
        assert!(is_yaml_number("0.0"));
        assert!(is_yaml_number("1.0e-8"));
        assert!(is_yaml_number("1.0e+7"));
        assert!(is_yaml_number("5.0e-4"));
        assert!(is_yaml_number(".5"));
        assert!(is_yaml_number("5."));
        // The hazard: dotless exponent → NOT a number (a string), matching pyyaml.
        assert!(!is_yaml_number("1e-3"));
        assert!(!is_yaml_number("1e7"));
        // Unsigned exponent → NOT a number in the strict YAML-1.1 resolver.
        assert!(!is_yaml_number("1.0e7"));
        // Non-numbers.
        assert!(!is_yaml_number("carbon"));
        assert!(!is_yaml_number("crew.food_store"));
        assert!(!is_yaml_number("."));
        assert!(!is_yaml_number(""));
    }
}
