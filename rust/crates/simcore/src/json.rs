//! Minimal, zero-dependency JSON *value* parser — the **load** half of the
//! interchange (the emit half is hand-written in [`crate::snapshot`]).
//!
//! Phase 7 shipped only the emitter (the port produced JSON that Python `loads`
//! read); Phase-8 save/load (P8.7) needs the inverse — a Rust reader for the
//! `sim_io` snapshot shape (work item #3). Rather than a `serde` dependency, we
//! hand-roll a tiny recursive-descent parser, mirroring the core's stdlib-only
//! discipline (the Python side uses `json.loads`; we owe only a reader for the
//! small, fixed shape our own emitter and Python `json.dumps` produce).
//!
//! The parser is a faithful (BMP) JSON reader — objects, arrays, strings with the
//! standard escape set, numbers, `true`/`false`/`null` — deliberately more general
//! than the fixed snapshot shape so it is robust to either port's whitespace and to
//! a hand-edited save file. Numbers are kept as their raw lexeme and parsed as
//! integers on demand ([`JsonValue::as_u64`] / [`JsonValue::as_i64`]) — the snapshot
//! carries integers (`n`, `version`) only, and routing them through `f64` would risk
//! the >2^53 precision loss the hex-float / hex-seed discipline exists to avoid.

use std::fmt;

/// A parsed JSON value. `Number` retains the raw lexeme (parsed to an integer on
/// demand); the snapshot never needs a JSON float (every real number is a hex-float
/// *string*), so no `f64` accessor is offered — keeping the loader off the lossy path.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    /// The raw numeric lexeme (e.g. `"168"`), parsed via [`JsonValue::as_u64`].
    Number(String),
    Str(String),
    Array(Vec<JsonValue>),
    /// Member order is preserved as written; lookup is by [`JsonValue::get`].
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    /// The string payload, or `None` for any other kind.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::Str(s) => Some(s),
            _ => None,
        }
    }

    /// The boolean payload, or `None` for any other kind.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// The number as a `u64` (integers only — the snapshot's `n`), or `None` if this
    /// is not a `Number` or the lexeme is not a valid `u64`.
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            JsonValue::Number(s) => s.parse::<u64>().ok(),
            _ => None,
        }
    }

    /// The number as an `i64` (the snapshot's `version`), or `None` as [`as_u64`](Self::as_u64).
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            JsonValue::Number(s) => s.parse::<i64>().ok(),
            _ => None,
        }
    }

    /// The object members, or `None` for any other kind.
    pub fn as_object(&self) -> Option<&[(String, JsonValue)]> {
        match self {
            JsonValue::Object(members) => Some(members),
            _ => None,
        }
    }

    /// The array elements, or `None` for any other kind.
    pub fn as_array(&self) -> Option<&[JsonValue]> {
        match self {
            JsonValue::Array(items) => Some(items),
            _ => None,
        }
    }

    /// The value for `key` if this is an object containing it (first match; the
    /// snapshot never has duplicate keys). `None` otherwise.
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        self.as_object()?
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }
}

/// A JSON parse failure, carrying a human diagnostic (character offset + reason).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonError(pub String);

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid JSON: {}", self.0)
    }
}

impl std::error::Error for JsonError {}

/// Parse a complete JSON document into a [`JsonValue`]. Trailing non-whitespace is
/// an error (the whole text must be one value), mirroring `json.loads`.
pub fn parse(text: &str) -> Result<JsonValue, JsonError> {
    let mut p = Parser {
        chars: text.chars().collect(),
        pos: 0,
    };
    p.skip_ws();
    let value = p.parse_value()?;
    p.skip_ws();
    if p.pos != p.chars.len() {
        return Err(p.err("trailing characters after top-level value"));
    }
    Ok(value)
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\n' | '\r')) {
            self.pos += 1;
        }
    }

    fn err(&self, msg: &str) -> JsonError {
        JsonError(format!("{msg} (at char {})", self.pos))
    }

    fn parse_value(&mut self) -> Result<JsonValue, JsonError> {
        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => Ok(JsonValue::Str(self.parse_string()?)),
            Some('t') => self.parse_literal("true", JsonValue::Bool(true)),
            Some('f') => self.parse_literal("false", JsonValue::Bool(false)),
            Some('n') => self.parse_literal("null", JsonValue::Null),
            Some(c) if c == '-' || c.is_ascii_digit() => self.parse_number(),
            _ => Err(self.err("expected a JSON value")),
        }
    }

    fn parse_literal(&mut self, word: &str, value: JsonValue) -> Result<JsonValue, JsonError> {
        for expected in word.chars() {
            if self.bump() != Some(expected) {
                return Err(self.err(&format!("expected literal {word:?}")));
            }
        }
        Ok(value)
    }

    fn parse_number(&mut self) -> Result<JsonValue, JsonError> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.pos += 1;
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.pos += 1;
        }
        // Optional fraction / exponent — accepted for generality, though the snapshot
        // carries only integer numbers (every real value is a hex-float *string*).
        if self.peek() == Some('.') {
            self.pos += 1;
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        if matches!(self.peek(), Some('e' | 'E')) {
            self.pos += 1;
            if matches!(self.peek(), Some('+' | '-')) {
                self.pos += 1;
            }
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        let lexeme: String = self.chars[start..self.pos].iter().collect();
        if lexeme.is_empty() || lexeme == "-" {
            return Err(self.err("malformed number"));
        }
        Ok(JsonValue::Number(lexeme))
    }

    /// Parse a string literal; assumes the current char is the opening quote.
    fn parse_string(&mut self) -> Result<String, JsonError> {
        if self.bump() != Some('"') {
            return Err(self.err("expected a string"));
        }
        let mut out = String::new();
        loop {
            match self.bump() {
                None => return Err(self.err("unterminated string")),
                Some('"') => return Ok(out),
                Some('\\') => match self.bump() {
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    Some('/') => out.push('/'),
                    Some('b') => out.push('\u{0008}'),
                    Some('f') => out.push('\u{000c}'),
                    Some('n') => out.push('\n'),
                    Some('r') => out.push('\r'),
                    Some('t') => out.push('\t'),
                    Some('u') => out.push(self.parse_unicode_escape()?),
                    _ => return Err(self.err("invalid escape sequence")),
                },
                Some(c) => out.push(c),
            }
        }
    }

    /// Parse the four hex digits after `\u` into a BMP char (surrogate pairs — never
    /// present in our ASCII data — are rejected rather than silently mishandled).
    fn parse_unicode_escape(&mut self) -> Result<char, JsonError> {
        let mut code: u32 = 0;
        for _ in 0..4 {
            let d = self
                .bump()
                .and_then(|c| c.to_digit(16))
                .ok_or_else(|| self.err("invalid \\u escape"))?;
            code = code * 16 + d;
        }
        char::from_u32(code).ok_or_else(|| self.err("invalid \\u code point (surrogate?)"))
    }

    fn parse_array(&mut self) -> Result<JsonValue, JsonError> {
        self.pos += 1; // consume '['
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(']') {
            self.pos += 1;
            return Ok(JsonValue::Array(items));
        }
        loop {
            self.skip_ws();
            items.push(self.parse_value()?);
            self.skip_ws();
            match self.bump() {
                Some(',') => continue,
                Some(']') => return Ok(JsonValue::Array(items)),
                _ => return Err(self.err("expected ',' or ']' in array")),
            }
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, JsonError> {
        self.pos += 1; // consume '{'
        let mut members = Vec::new();
        self.skip_ws();
        if self.peek() == Some('}') {
            self.pos += 1;
            return Ok(JsonValue::Object(members));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some('"') {
                return Err(self.err("expected a string key in object"));
            }
            let key = self.parse_string()?;
            self.skip_ws();
            if self.bump() != Some(':') {
                return Err(self.err("expected ':' after object key"));
            }
            self.skip_ws();
            let value = self.parse_value()?;
            members.push((key, value));
            self.skip_ws();
            match self.bump() {
                Some(',') => continue,
                Some('}') => return Ok(JsonValue::Object(members)),
                _ => return Err(self.err("expected ',' or '}' in object")),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_snapshot_shape() {
        let text = r#"{
          "aux": {},
          "n": 168,
          "rng_seed": "0x0",
          "stocks": [
            {"id": "a", "unclamped": false, "amount": "0x1.0p+0"}
          ],
          "version": 3
        }"#;
        let v = parse(text).unwrap();
        assert_eq!(v.get("n").unwrap().as_u64(), Some(168));
        assert_eq!(v.get("version").unwrap().as_i64(), Some(3));
        assert_eq!(v.get("rng_seed").unwrap().as_str(), Some("0x0"));
        assert!(v.get("aux").unwrap().as_object().unwrap().is_empty());
        let stocks = v.get("stocks").unwrap().as_array().unwrap();
        assert_eq!(stocks.len(), 1);
        assert_eq!(stocks[0].get("unclamped").unwrap().as_bool(), Some(false));
        assert_eq!(stocks[0].get("amount").unwrap().as_str(), Some("0x1.0p+0"));
    }

    #[test]
    fn handles_escapes_and_whitespace() {
        let v = parse(r#" { "k" : "a\"b\\c\n\tdA" , "e" : [ ] } "#).unwrap();
        assert_eq!(v.get("k").unwrap().as_str(), Some("a\"b\\c\n\tdA"));
        assert!(v.get("e").unwrap().as_array().unwrap().is_empty());
    }

    #[test]
    fn integers_do_not_route_through_f64() {
        // A seed-sized integer would lose precision as an f64; kept as a lexeme it is
        // exact (the snapshot stores the seed as a hex string, but this guards the path).
        let v = parse(r#"{"big": 9007199254740993}"#).unwrap();
        assert_eq!(v.get("big").unwrap().as_u64(), Some(9_007_199_254_740_993));
    }

    #[test]
    fn negative_and_fractional_numbers_parse() {
        assert_eq!(parse("-5").unwrap().as_i64(), Some(-5));
        // A fraction/exponent is accepted (generality) but has no u64 reading.
        assert!(matches!(parse("1.5e3"), Ok(JsonValue::Number(_))));
    }

    #[test]
    fn rejects_malformed() {
        assert!(parse("").is_err());
        assert!(parse("{").is_err());
        assert!(parse(r#"{"k": }"#).is_err());
        assert!(parse("[1 2]").is_err());
        assert!(parse(r#"{"k": 1} trailing"#).is_err());
        assert!(parse(r#""unterminated"#).is_err());
        assert!(parse("nul").is_err());
    }
}
