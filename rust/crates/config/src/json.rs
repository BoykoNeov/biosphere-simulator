//! A hand-rolled reader for the **closed JSON subset** the committed weather fixture
//! uses (reference flip, slice C9 — the weather path).
//!
//! # Why this exists
//!
//! The raw NASAPower weather that drives the biosphere season is a committed JSON
//! fixture. Until C9 the port could not read it: a Python generator
//! (`gen_biosphere_weather.py`) lowered it into a hex-float table which the Rust
//! `biosphere::weather` embedded with `include_str!`. That generator was the last one
//! in the flip with **no named successor** — while it stood, the reference's own
//! forcing data arrived through a Python script.
//!
//! # Why hand-rolled
//!
//! The same reason as [`crate::yaml`], and one more. The shared reason is the
//! **parse-parity boundary**: a third-party crate would mean reconciling two
//! independent implementations, where a subset we own is one grammar on both sides.
//! The extra reason is the zero-dependency charter — this crate sits below `domains`
//! and takes no third-party code at all.
//!
//! ⚠ **The float question was measured before this module was written, not assumed.**
//! All 916 values of the fixture (latitude + 3 × 305 observations) parse to the *exact
//! bits* the port read from the Python-generated hex-float table: `f64::from_str` and
//! CPython's `float()` are both correctly-rounded decimal → binary conversions. So the
//! reader replaces the generator with **no value moving anywhere**, which is what makes
//! C9 a re-anchoring slice rather than an unfreeze.
//!
//! # The accepted subset
//!
//! RFC 8259 JSON, minus nothing the fixture needs and with two deliberate bounds:
//!
//! * **Objects** preserve source order ([`JsonValue::Object`] is a `Vec`, not a map) —
//!   the fixture's `weather` array is a *time series*, and while order inside an object
//!   is not semantically load-bearing here, a reader that silently reorders anything is
//!   a reader whose output depends on its data structure. Duplicate keys are an error,
//!   not a last-one-wins.
//! * **Nesting depth** is capped ([`MAX_DEPTH`]) so a malformed file is a
//!   [`ConfigError`] rather than a blown stack.
//!
//! Numbers are validated against the JSON grammar *and then* handed to `f64::from_str`,
//! so the forms Rust accepts but JSON does not (`inf`, `NaN`, `+1`, `007`, `1.`) are
//! rejected here rather than silently admitted.

use crate::errors::ConfigError;

/// The deepest object/array nesting accepted. The fixture nests 3 deep; the cap exists
/// so a malformed document fails as an error instead of recursing to a stack overflow.
pub const MAX_DEPTH: usize = 32;

/// A parsed JSON value in the closed subset.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    /// An object, in **source order**, with duplicate keys rejected at parse time.
    Object(Vec<(String, JsonValue)>),
    /// An array, in source order.
    Array(Vec<JsonValue>),
    /// A string, with escapes (including `\uXXXX` surrogate pairs) resolved.
    String(String),
    /// A number. JSON has one numeric type; this reader lands it in `f64` exactly as
    /// CPython's `json` does.
    Number(f64),
    /// `true` / `false`.
    Bool(bool),
    /// `null`.
    Null,
}

impl JsonValue {
    /// The object entries, or an error naming `context` if this is not an object.
    pub fn as_object(&self, context: &str) -> Result<&[(String, JsonValue)], ConfigError> {
        match self {
            JsonValue::Object(entries) => Ok(entries),
            _ => Err(ConfigError::new(format!("{context}: expected an object"))),
        }
    }

    /// The array items, or an error naming `context` if this is not an array.
    pub fn as_array(&self, context: &str) -> Result<&[JsonValue], ConfigError> {
        match self {
            JsonValue::Array(items) => Ok(items),
            _ => Err(ConfigError::new(format!("{context}: expected an array"))),
        }
    }

    /// The string, or an error naming `context` if this is not a string.
    pub fn as_str(&self, context: &str) -> Result<&str, ConfigError> {
        match self {
            JsonValue::String(text) => Ok(text.as_str()),
            _ => Err(ConfigError::new(format!("{context}: expected a string"))),
        }
    }

    /// The number, or an error naming `context` if this is not a number.
    pub fn as_f64(&self, context: &str) -> Result<f64, ConfigError> {
        match self {
            JsonValue::Number(value) => Ok(*value),
            _ => Err(ConfigError::new(format!("{context}: expected a number"))),
        }
    }

    /// The value under `key`, or an error naming `context` if this is not an object or
    /// the key is absent.
    pub fn get(&self, key: &str, context: &str) -> Result<&JsonValue, ConfigError> {
        for (name, value) in self.as_object(context)? {
            if name == key {
                return Ok(value);
            }
        }
        Err(ConfigError::new(format!("{context}: missing key '{key}'")))
    }
}

/// Parse a whole JSON document into a [`JsonValue`].
///
/// Total over the closed subset: anything outside it — a trailing comma, a bare word,
/// an unterminated string, a number in a form JSON forbids, nesting past [`MAX_DEPTH`],
/// or trailing content after the top-level value — is a [`ConfigError`].
pub fn parse_json(text: &str) -> Result<JsonValue, ConfigError> {
    let bytes = text.as_bytes();
    let mut parser = Parser { bytes, pos: 0 };
    parser.skip_whitespace();
    let value = parser.parse_value(0)?;
    parser.skip_whitespace();
    if parser.pos != bytes.len() {
        return Err(parser.error("trailing content after the top-level value"));
    }
    Ok(value)
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Parser<'_> {
    fn error(&self, what: impl std::fmt::Display) -> ConfigError {
        ConfigError::new(format!("JSON at byte {}: {what}", self.pos))
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    /// Consume `expected`, or error naming what was wanted.
    fn expect(&mut self, expected: u8) -> Result<(), ConfigError> {
        if self.peek() == Some(expected) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.error(format!("expected '{}'", expected as char)))
        }
    }

    fn parse_value(&mut self, depth: usize) -> Result<JsonValue, ConfigError> {
        if depth > MAX_DEPTH {
            return Err(self.error(format!("nesting deeper than {MAX_DEPTH}")));
        }
        match self.peek() {
            Some(b'{') => self.parse_object(depth),
            Some(b'[') => self.parse_array(depth),
            Some(b'"') => Ok(JsonValue::String(self.parse_string()?)),
            Some(b't') => self.parse_literal("true", JsonValue::Bool(true)),
            Some(b'f') => self.parse_literal("false", JsonValue::Bool(false)),
            Some(b'n') => self.parse_literal("null", JsonValue::Null),
            Some(b'-' | b'0'..=b'9') => self.parse_number(),
            Some(other) => Err(self.error(format!("unexpected byte '{}'", other as char))),
            None => Err(self.error("unexpected end of document")),
        }
    }

    fn parse_literal(&mut self, word: &str, value: JsonValue) -> Result<JsonValue, ConfigError> {
        if self.bytes[self.pos..].starts_with(word.as_bytes()) {
            self.pos += word.len();
            Ok(value)
        } else {
            Err(self.error(format!("expected '{word}'")))
        }
    }

    fn parse_object(&mut self, depth: usize) -> Result<JsonValue, ConfigError> {
        self.expect(b'{')?;
        let mut entries: Vec<(String, JsonValue)> = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(JsonValue::Object(entries));
        }
        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            if entries.iter().any(|(name, _)| *name == key) {
                return Err(self.error(format!("duplicate key '{key}'")));
            }
            self.skip_whitespace();
            self.expect(b':')?;
            self.skip_whitespace();
            let value = self.parse_value(depth + 1)?;
            entries.push((key, value));
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(JsonValue::Object(entries));
                }
                _ => return Err(self.error("expected ',' or '}' in object")),
            }
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<JsonValue, ConfigError> {
        self.expect(b'[')?;
        let mut items: Vec<JsonValue> = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(JsonValue::Array(items));
        }
        loop {
            self.skip_whitespace();
            items.push(self.parse_value(depth + 1)?);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(JsonValue::Array(items));
                }
                _ => return Err(self.error("expected ',' or ']' in array")),
            }
        }
    }

    fn parse_string(&mut self) -> Result<String, ConfigError> {
        self.expect(b'"')?;
        let mut out = String::new();
        loop {
            let byte = self
                .peek()
                .ok_or_else(|| self.error("unterminated string"))?;
            match byte {
                b'"' => {
                    self.pos += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.pos += 1;
                    let escape = self
                        .peek()
                        .ok_or_else(|| self.error("unterminated escape"))?;
                    self.pos += 1;
                    match escape {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000C}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => out.push(self.parse_unicode_escape()?),
                        other => {
                            return Err(self.error(format!("unknown escape '\\{}'", other as char)))
                        }
                    }
                }
                0x00..=0x1F => return Err(self.error("unescaped control character in string")),
                _ => {
                    // Copy the whole UTF-8 sequence; `text` was a `&str`, so the bytes
                    // are valid by construction and the boundary is where the next
                    // non-continuation byte starts.
                    let start = self.pos;
                    self.pos += 1;
                    while matches!(self.peek(), Some(b) if b & 0b1100_0000 == 0b1000_0000) {
                        self.pos += 1;
                    }
                    out.push_str(
                        std::str::from_utf8(&self.bytes[start..self.pos])
                            .map_err(|_| self.error("invalid UTF-8 in string"))?,
                    );
                }
            }
        }
    }

    /// The four hex digits after `\u`, resolving a surrogate **pair** into one char.
    ///
    /// A lone surrogate is an error rather than a replacement character: the fixture's
    /// only escape is a BMP em-dash, and silently substituting U+FFFD would turn a
    /// corrupt file into a plausible one.
    fn parse_unicode_escape(&mut self) -> Result<char, ConfigError> {
        let first = self.parse_hex4()?;
        if (0xDC00..=0xDFFF).contains(&first) {
            return Err(self.error("lone low surrogate in \\u escape"));
        }
        if !(0xD800..=0xDBFF).contains(&first) {
            return char::from_u32(first).ok_or_else(|| self.error("invalid \\u escape"));
        }
        if self.peek() != Some(b'\\') {
            return Err(self.error("high surrogate not followed by a \\u escape"));
        }
        self.pos += 1;
        if self.peek() != Some(b'u') {
            return Err(self.error("high surrogate not followed by a \\u escape"));
        }
        self.pos += 1;
        let second = self.parse_hex4()?;
        if !(0xDC00..=0xDFFF).contains(&second) {
            return Err(self.error("high surrogate not followed by a low surrogate"));
        }
        let combined = 0x10000 + ((first - 0xD800) << 10) + (second - 0xDC00);
        char::from_u32(combined).ok_or_else(|| self.error("invalid surrogate pair"))
    }

    fn parse_hex4(&mut self) -> Result<u32, ConfigError> {
        let end = self.pos + 4;
        if end > self.bytes.len() {
            return Err(self.error("truncated \\u escape"));
        }
        let digits = std::str::from_utf8(&self.bytes[self.pos..end])
            .map_err(|_| self.error("invalid \\u escape"))?;
        let value =
            u32::from_str_radix(digits, 16).map_err(|_| self.error("invalid \\u escape"))?;
        self.pos = end;
        Ok(value)
    }

    /// Scan a number against the **JSON** grammar, then hand the exact source slice to
    /// `f64::from_str`.
    ///
    /// Validating first is what keeps the two readers honest: Rust accepts `inf`,
    /// `NaN`, `+1`, `1.` and `007`, and CPython's `json` accepts none of them. The scan
    /// is the whole difference between "reads JSON" and "reads whatever Rust will take".
    fn parse_number(&mut self) -> Result<JsonValue, ConfigError> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        // int: `0` alone, or a nonzero digit and more digits. No leading zeros.
        match self.peek() {
            Some(b'0') => self.pos += 1,
            Some(b'1'..=b'9') => {
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.pos += 1;
                }
            }
            _ => return Err(self.error("expected a digit")),
        }
        // frac: a `.` must be followed by at least one digit.
        if self.peek() == Some(b'.') {
            self.pos += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.error("expected a digit after '.'"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        // exp: `e`/`E`, an optional sign, at least one digit.
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.error("expected a digit in the exponent"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        let literal = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| self.error("invalid number"))?;
        let value: f64 = literal
            .parse()
            .map_err(|_| self.error(format!("'{literal}' is not a number")))?;
        Ok(JsonValue::Number(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> JsonValue {
        parse_json(text).expect("parses")
    }

    #[test]
    fn reads_the_shapes_the_fixture_uses() {
        let value =
            parse(r#"{"provenance": {"latitude": 52.0, "n": 305}, "weather": [{"TEMP": -1.5}]}"#);
        assert_eq!(
            value
                .get("provenance", "doc")
                .unwrap()
                .get("latitude", "provenance")
                .unwrap()
                .as_f64("latitude")
                .unwrap(),
            52.0
        );
        let rows = value
            .get("weather", "doc")
            .unwrap()
            .as_array("weather")
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].get("TEMP", "row").unwrap().as_f64("TEMP").unwrap(),
            -1.5
        );
    }

    #[test]
    fn object_and_array_order_is_source_order() {
        // Load-bearing: the fixture's `weather` array is a time series, and a reader
        // that reorders it would drive the season with shuffled days while every value
        // in it stayed correct.
        let value = parse(r#"{"b": 1, "a": 2, "c": [3, 1, 2]}"#);
        let keys: Vec<&str> = value
            .as_object("doc")
            .unwrap()
            .iter()
            .map(|(k, _)| k.as_str())
            .collect();
        assert_eq!(keys, vec!["b", "a", "c"]);
        let items: Vec<f64> = value
            .get("c", "doc")
            .unwrap()
            .as_array("c")
            .unwrap()
            .iter()
            .map(|v| v.as_f64("item").unwrap())
            .collect();
        assert_eq!(items, vec![3.0, 1.0, 2.0]);
    }

    #[test]
    fn nested_containers_scalars_and_whitespace() {
        let value =
            parse("{\n  \"a\" : [ { \"b\" : null } , true , false ] ,\r\n\t\"c\" : \"x\"\n}");
        assert_eq!(value.get("c", "doc").unwrap().as_str("c").unwrap(), "x");
        let items = value.get("a", "doc").unwrap().as_array("a").unwrap();
        assert_eq!(items[0].get("b", "row").unwrap(), &JsonValue::Null);
        assert_eq!(items[1], JsonValue::Bool(true));
        assert_eq!(items[2], JsonValue::Bool(false));
        assert_eq!(parse("[]"), JsonValue::Array(vec![]));
        assert_eq!(parse("{}"), JsonValue::Object(vec![]));
    }

    #[test]
    fn string_escapes_including_a_surrogate_pair() {
        // `—` is the em-dash the committed fixture's description actually carries.
        let value = parse(r#"{"s": "a—b \"q\" \\ \/ \b\f\n\r\t 🚀"}"#);
        assert_eq!(
            value.get("s", "doc").unwrap().as_str("s").unwrap(),
            "a\u{2014}b \"q\" \\ / \u{8}\u{c}\n\r\t \u{1F680}"
        );
    }

    #[test]
    fn numbers_accepted_by_rust_but_not_by_json_are_rejected() {
        // The reason the scan exists at all: every one of these parses fine with
        // `f64::from_str` and none of them is JSON.
        for bad in ["inf", "-inf", "NaN", "+1", "007", "1.", ".5", "1e", "1e+"] {
            assert!(
                parse_json(bad).is_err(),
                "'{bad}' is not JSON but was accepted"
            );
        }
        assert_eq!(parse("-0.0"), JsonValue::Number(-0.0));
        assert_eq!(parse("1e3"), JsonValue::Number(1000.0));
        assert_eq!(parse("-1.5E-2"), JsonValue::Number(-0.015));
        assert_eq!(parse("0"), JsonValue::Number(0.0));
    }

    #[test]
    fn malformed_documents_are_errors_not_partial_reads() {
        for bad in [
            r#"{"a": 1,}"#,
            r#"{"a" 1}"#,
            r#"{a: 1}"#,
            r#"[1 2]"#,
            r#"{"a": 1} trailing"#,
            r#""unterminated"#,
            r#"{"a": 1, "a": 2}"#,
            "",
            r#"{"s": "raw
newline"}"#,
            r#"{"s": "\q"}"#,
            r#"{"s": "\ud83d"}"#,
            r#"{"s": "\udc00"}"#,
        ] {
            assert!(parse_json(bad).is_err(), "'{bad}' should not parse");
        }
    }

    #[test]
    fn nesting_past_the_cap_is_an_error_not_a_stack_overflow() {
        let deep = format!(
            "{}1{}",
            "[".repeat(MAX_DEPTH + 2),
            "]".repeat(MAX_DEPTH + 2)
        );
        assert!(parse_json(&deep).is_err());
        let ok = format!(
            "{}1{}",
            "[".repeat(MAX_DEPTH - 1),
            "]".repeat(MAX_DEPTH - 1)
        );
        assert!(parse_json(&ok).is_ok());
    }

    #[test]
    fn accessors_name_their_context_on_a_type_mismatch() {
        let value = parse(r#"{"a": 1}"#);
        let err = value.as_array("the document").unwrap_err();
        assert!(err.message.contains("the document"), "{}", err.message);
        let err = value.get("missing", "the document").unwrap_err();
        assert!(err.message.contains("missing"), "{}", err.message);
    }
}
