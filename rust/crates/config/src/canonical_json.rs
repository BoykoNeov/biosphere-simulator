//! The **write** half of the JSON boundary — a canonical serializer that reproduces
//! Python's `json.dumps(obj, indent=2, sort_keys=True) + "\n"` byte for byte. Slice C7
//! of the reference flip (`docs/plans/post-roadmap-reference-flip.md`).
//!
//! # Why byte-for-byte with a Python function is the requirement
//!
//! The three frozen manifests (`docs/*-reference.manifest.json`) are **committed files**
//! that Python wrote until C7. Moving the writer to the reference is a *re-anchoring*
//! only if the bytes do not move — the same discipline C1, C8 and C9 followed. So this
//! module's contract is not "emit valid JSON", it is "emit the file Python emitted", and
//! its gate is the regenerated manifest diffing empty.
//!
//! [`json`](crate::json) is the reader for the same shape; the two are deliberately
//! separate types. The reader keeps numbers as raw lexemes because a save file's integers
//! must not route through `f64`; the writer does the same, for a different reason — see
//! [`Json::Number`].
//!
//! # The four things Python does that a naive writer does not
//!
//! Measured against all three manifests before this module was written, rather than
//! recalled:
//!
//! 1. **`ensure_ascii` is on by default.** Every character above `0x7f` is written as a
//!    `\uXXXX` escape in **lowercase** hex. The manifests carry `⚠`, `—`, `§`, `Γ`, `τ`,
//!    `₂`, `×`, `→`, `↔` and `…` — 232 escapes across the three files. ⚠ The existing
//!    `dump_*_inventory` examples emit raw UTF-8 *on the stated grounds that* "the
//!    checker's own writer re-escapes to ASCII"; C7 is what makes that premise expire,
//!    which is why the escaping lives here and not there.
//! 2. **Empty containers stay on one line** — `"key": {}` and `"key": []`, never an
//!    indented pair of braces. There are 39 of them across the three manifests (25 in the
//!    station contract alone, where a scenario with no science claim carries an explicit
//!    empty list meaning *measured, none*).
//! 3. **No space before the colon, one after** — `"key": value` — and the separator
//!    between items is `,` followed by the newline, never `, `.
//! 4. **A trailing newline**, which `json.dumps` does not add and the project's golden
//!    discipline does. [`dumps`] adds it, so callers cannot forget it.
//!
//! ⚠ **Non-BMP characters are unreachable and therefore unimplemented.** Python escapes
//! them as a UTF-16 surrogate pair; measured across all three manifests, there is not one
//! character above `0xffff`. Rather than write surrogate-pair logic no test could
//! exercise, [`escape_into`] **panics** on such a character — an unwritable manifest
//! rather than a silently wrong one. Should a contract ever need an emoji, the panic is
//! the instruction to implement the pair.

use std::fmt::Write as _;

/// A JSON value for the writer.
///
/// Deliberately *not* [`crate::json::JsonValue`]: that type is a parser's output (owned
/// `String` keys, no ordering guarantee needed) and this one is a builder's input.
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    /// A number as its **literal text**, e.g. `"15"` or `"0.25"`.
    ///
    /// ⚠⚠ This is not laziness and it is not only about float formatting (though that
    /// too: Python's `repr` and Rust's `{}` disagree on values like `1e-05`). It is the
    /// structural guard the freeze contract needs. `dt_days` is one of the two
    /// deliberately **hand-written** literals of the biosphere manifest — a manifest that
    /// imported `BIO_DT` would auto-follow a step change, which is the opposite of a
    /// freeze, and the 2026-08-14 step move became a ceremony only because that literal
    /// went red. C7 moves the writer *into the tree that owns `BIO_DT`*, where splicing it
    /// in is a one-character mistake. A writer that takes text and not `f64` makes the
    /// mistake visible: `Json::num("0.25")` cannot be `BIO_DT` by accident.
    ///
    /// Constructed through [`Json::num`], which rejects a lexeme that is not a number.
    Number(String),
    Str(String),
    Array(Vec<Json>),
    /// Key-value pairs in **any** order — [`dumps`] sorts them, mirroring `sort_keys=True`.
    Object(Vec<(String, Json)>),
}

impl Json {
    /// A string value.
    pub fn s(text: impl Into<String>) -> Json {
        Json::Str(text.into())
    }

    /// An integer value.
    pub fn int(value: i64) -> Json {
        Json::Number(value.to_string())
    }

    /// A number from its literal text — see [`Json::Number`] for why this is the only way
    /// to write a non-integer.
    ///
    /// # Panics
    ///
    /// If `lexeme` is not a JSON number. A manifest is a frozen contract; a malformed
    /// number in it would be a file no reader accepts, so this fails at the write rather
    /// than shipping.
    pub fn num(lexeme: impl Into<String>) -> Json {
        let lexeme = lexeme.into();
        assert!(
            is_json_number(&lexeme),
            "not a JSON number literal: {lexeme:?}"
        );
        Json::Number(lexeme)
    }

    /// An object from any iterable of pairs. Order is irrelevant — [`dumps`] sorts.
    pub fn obj<K: Into<String>>(pairs: impl IntoIterator<Item = (K, Json)>) -> Json {
        Json::Object(pairs.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }

    /// An array of strings — the common shape (`flow_set`, `aux_set`, the op rosters).
    pub fn strs<S: Into<String>>(items: impl IntoIterator<Item = S>) -> Json {
        Json::Array(items.into_iter().map(Json::s).collect())
    }
}

/// Is `lexeme` a JSON number? The grammar from RFC 8259 §6, hand-checked (no crate may be
/// added to a zero-dependency tree).
fn is_json_number(lexeme: &str) -> bool {
    let mut chars = lexeme.chars().peekable();
    if chars.peek() == Some(&'-') {
        chars.next();
    }
    // int: `0` alone, or a nonzero digit followed by digits.
    match chars.next() {
        Some('0') => {}
        Some(c) if c.is_ascii_digit() => {
            while chars.peek().is_some_and(char::is_ascii_digit) {
                chars.next();
            }
        }
        _ => return false,
    }
    // frac
    if chars.peek() == Some(&'.') {
        chars.next();
        let mut digits = 0;
        while chars.peek().is_some_and(char::is_ascii_digit) {
            chars.next();
            digits += 1;
        }
        if digits == 0 {
            return false;
        }
    }
    // exp
    if matches!(chars.peek(), Some('e') | Some('E')) {
        chars.next();
        if matches!(chars.peek(), Some('+') | Some('-')) {
            chars.next();
        }
        let mut digits = 0;
        while chars.peek().is_some_and(char::is_ascii_digit) {
            chars.next();
            digits += 1;
        }
        if digits == 0 {
            return false;
        }
    }
    chars.next().is_none()
}

/// Append `text` as a quoted, ASCII-only JSON string — Python's `ensure_ascii=True`.
///
/// # Panics
///
/// On a character above the basic multilingual plane; see the module header for why that
/// is a panic and not surrogate-pair logic.
pub fn escape_into(out: &mut String, text: &str) {
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 || (c as u32) > 0x7e => {
                let code = c as u32;
                assert!(
                    code <= 0xffff,
                    "non-BMP character {c:?} needs surrogate-pair escaping, which this \
                     writer deliberately does not implement — see the module header"
                );
                let _ = write!(out, "\\u{code:04x}");
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Serialize `value` exactly as `json.dumps(value, indent=2, sort_keys=True) + "\n"`.
pub fn dumps(value: &Json) -> String {
    let mut out = String::new();
    write_value(&mut out, value, 0);
    out.push('\n');
    out
}

fn write_value(out: &mut String, value: &Json, depth: usize) {
    match value {
        Json::Null => out.push_str("null"),
        Json::Bool(true) => out.push_str("true"),
        Json::Bool(false) => out.push_str("false"),
        Json::Number(lexeme) => out.push_str(lexeme),
        Json::Str(text) => escape_into(out, text),
        Json::Array(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push_str("[\n");
            for (i, item) in items.iter().enumerate() {
                indent(out, depth + 1);
                write_value(out, item, depth + 1);
                out.push_str(if i + 1 == items.len() { "\n" } else { ",\n" });
            }
            indent(out, depth);
            out.push(']');
        }
        Json::Object(pairs) => {
            if pairs.is_empty() {
                out.push_str("{}");
                return;
            }
            let mut sorted: Vec<&(String, Json)> = pairs.iter().collect();
            // `sort_keys=True` orders by code point; Rust orders `&str` by UTF-8 bytes,
            // and the two agree (the argument `ScienceGate`'s `Ord` docstring already
            // makes for the census). `sort_by_key` on the borrowed key, so a duplicate
            // key keeps insertion order rather than being reordered arbitrarily — and
            // duplicates are a caller bug the assert below reports.
            sorted.sort_by(|a, b| a.0.cmp(&b.0));
            for pair in sorted.windows(2) {
                assert!(
                    pair[0].0 != pair[1].0,
                    "duplicate manifest key {:?}",
                    pair[0].0
                );
            }
            out.push_str("{\n");
            for (i, (key, item)) in sorted.iter().enumerate() {
                indent(out, depth + 1);
                escape_into(out, key);
                out.push_str(": ");
                write_value(out, item, depth + 1);
                out.push_str(if i + 1 == sorted.len() { "\n" } else { ",\n" });
            }
            indent(out, depth);
            out.push('}');
        }
    }
}

fn indent(out: &mut String, depth: usize) {
    for _ in 0..depth * 2 {
        out.push(' ');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The four Python behaviours the module header enumerates, in one expected string.
    ///
    /// ⚠ The expected text here was produced by **Python** (`json.dumps(..., indent=2,
    /// sort_keys=True)`) and transcribed, not written by reading this module's code. A
    /// self-consistent expectation would prove only that the writer is deterministic.
    #[test]
    fn matches_python_dumps_on_every_shape_the_manifests_use() {
        let value = Json::obj([
            ("zebra", Json::int(4)),
            ("alpha", Json::num("0.25")),
            ("empty_obj", Json::obj(Vec::<(String, Json)>::new())),
            ("empty_arr", Json::Array(vec![])),
            ("null", Json::Null),
            ("prose", Json::s("⚠ a — b §")),
            (
                "nested",
                Json::obj([(
                    "list",
                    Json::Array(vec![Json::s("x"), Json::obj([("k", Json::s("v"))])]),
                )]),
            ),
        ]);
        let expected = concat!(
            "{\n",
            "  \"alpha\": 0.25,\n",
            "  \"empty_arr\": [],\n",
            "  \"empty_obj\": {},\n",
            "  \"nested\": {\n",
            "    \"list\": [\n",
            "      \"x\",\n",
            "      {\n",
            "        \"k\": \"v\"\n",
            "      }\n",
            "    ]\n",
            "  },\n",
            "  \"null\": null,\n",
            "  \"prose\": \"\\u26a0 a \\u2014 b \\u00a7\",\n",
            "  \"zebra\": 4\n",
            "}\n",
        );
        assert_eq!(dumps(&value), expected);
    }

    #[test]
    fn escapes_the_ascii_specials_python_escapes() {
        let mut out = String::new();
        escape_into(&mut out, "a\"b\\c\nd\te\rf\u{08}g\u{0c}h\u{01}i");
        assert_eq!(out, r#""a\"b\\c\nd\te\rf\bg\fh\u0001i""#);
    }

    /// `/` is **not** escaped by `json.dumps`, and `~` (0x7e) is the last raw character.
    #[test]
    fn leaves_solidus_and_tilde_raw() {
        let mut out = String::new();
        escape_into(&mut out, "a/b~c");
        assert_eq!(out, "\"a/b~c\"");
    }

    #[test]
    fn number_lexemes_are_validated() {
        for good in ["0", "-1", "15", "0.25", "1e-05", "-2.5E+3"] {
            assert_eq!(Json::num(good), Json::Number(good.to_string()));
        }
        for bad in ["", "-", "01", ".5", "1.", "1e", "0x10", "nan", "1 "] {
            assert!(
                std::panic::catch_unwind(|| Json::num(bad)).is_err(),
                "accepted a non-number lexeme: {bad:?}"
            );
        }
    }

    #[test]
    fn keys_sort_by_code_point() {
        let value = Json::obj([
            ("b", Json::int(1)),
            ("A", Json::int(2)),
            ("_x", Json::int(3)),
            ("a", Json::int(4)),
        ]);
        // Python: sorted(["b", "A", "_x", "a"]) == ["A", "_x", "a", "b"]
        let text = dumps(&value);
        let keys: Vec<&str> = text
            .lines()
            .filter_map(|l| l.trim().strip_prefix('"'))
            .filter_map(|l| l.split('"').next())
            .collect();
        assert_eq!(keys, ["A", "_x", "a", "b"]);
    }

    #[test]
    fn a_duplicate_key_is_a_panic_not_a_silently_dropped_field() {
        let value = Json::obj([("k", Json::int(1)), ("k", Json::int(2))]);
        assert!(std::panic::catch_unwind(move || dumps(&value)).is_err());
    }

    /// The deliberate gap, asserted so it stays deliberate.
    #[test]
    fn a_non_bmp_character_panics_rather_than_writing_a_wrong_escape() {
        assert!(std::panic::catch_unwind(|| {
            let mut out = String::new();
            escape_into(&mut out, "🌱");
        })
        .is_err());
    }
}
