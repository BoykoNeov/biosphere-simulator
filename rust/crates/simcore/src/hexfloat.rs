//! C99 hex-float codec: `f64` ⇄ the exact string form Python's `float.hex()`
//! emits (and `float.fromhex()` accepts). This is the interchange the frozen
//! goldens are written in (`sim_io` stores every amount as a hex-float string),
//! so bit-exact fidelity here is the foundation the whole port stands on.
//!
//! Rust std has **no** hex-float parser or `{:a}`-style formatter for `f64`, so
//! both directions are hand-rolled from the IEEE-754 bit decomposition. The design
//! goal is *exact bit round-trip*, not merely matching Python's string spelling —
//! but because `format` mirrors `float.hex()`'s canonical layout digit-for-digit,
//! it does match it too (the round-trip test pins both facts against Python-emitted
//! vectors).
//!
//! Canonical layout (what `float.hex()` produces, and all this parser accepts):
//!
//! * normal:     `[-]0x1.{13 hex}p{±}{dec}`   e.g. `0x1.87e90ff972484p+3`
//! * subnormal:  `[-]0x0.{13 hex}p-1022`      e.g. `0x0.0000000000001p-1022`
//! * zero:       `[-]0x0.0p+0`
//! * non-finite: `inf` / `-inf` / `nan` — never stored in a `State`; handled for
//!   codec completeness so a stray value fails loudly downstream rather than
//!   silently, and so the codec is total
//!
//! The fraction carries exactly the 52 mantissa bits (≤13 hex digits, zero-padded
//! to 13 by `format`). The leading digit is `1` for normals, `0` for subnormals and
//! zero — so `parse` reconstructs the IEEE bits directly by inverting `format`,
//! which is exact by construction (no rounding, no power-of-two scaling pitfalls).

/// The 52-bit mantissa field mask.
const MANTISSA_MASK: u64 = 0x000f_ffff_ffff_ffff;
/// Exponent bias for IEEE-754 binary64.
const BIAS: i64 = 1023;
/// The exponent an IEEE-754 double subnormal is written with (`p-1022`).
const SUBNORMAL_EXP: i64 = -1022;

/// A hex-float parse failure. Carries the offending text for a loud diagnostic —
/// the port must never silently coerce a malformed value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(pub String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid hex-float {:?}", self.0)
    }
}

impl std::error::Error for ParseError {}

/// Format an `f64` as the exact C99 hex-float string `float.hex()` emits.
///
/// Mirrors CPython's canonical layout: leading `1.` for normals / `0.` for
/// subnormals and zero, a 13-hex-digit fraction (the full 52-bit mantissa,
/// zero-padded), and a signed decimal `p` exponent.
pub fn format(x: f64) -> String {
    let bits = x.to_bits();
    let sign = (bits >> 63) & 1;
    let exp_field = ((bits >> 52) & 0x7ff) as i64;
    let mantissa = bits & MANTISSA_MASK;
    let sign_str = if sign == 1 { "-" } else { "" };

    if exp_field == 0x7ff {
        // inf / nan — never present in a State; kept so the codec is total.
        return if mantissa == 0 {
            format!("{sign_str}inf")
        } else {
            "nan".to_string()
        };
    }

    if exp_field == 0 && mantissa == 0 {
        // ±0.0 is spelled with a single fraction digit, matching float.hex().
        return format!("{sign_str}0x0.0p+0");
    }

    let (lead, exp) = if exp_field == 0 {
        (0u64, SUBNORMAL_EXP) // subnormal
    } else {
        (1u64, exp_field - BIAS) // normal
    };
    let exp_sign = if exp >= 0 { "+" } else { "-" };
    format!("{sign_str}0x{lead}.{mantissa:013x}p{exp_sign}{}", exp.abs())
}

/// Parse a C99 hex-float string into the exact `f64` it denotes.
///
/// Accepts the canonical subset `float.hex()` emits (see the module docs). The
/// reconstruction inverts [`format`]: it reads the leading digit, the ≤13-digit
/// fraction, and the binary exponent, then assembles the IEEE-754 bit pattern
/// directly — exact, with no rounding step.
pub fn parse(s: &str) -> Result<f64, ParseError> {
    let err = || ParseError(s.to_string());

    // Sign.
    let (sign_bit, body) = match s.strip_prefix('-') {
        Some(rest) => (1u64, rest),
        None => (0u64, s.strip_prefix('+').unwrap_or(s)),
    };

    // Non-finite tokens (case-sensitive, matching float.hex()).
    match body {
        "inf" | "Infinity" => return Ok(f64::from_bits((sign_bit << 63) | (0x7ff << 52))),
        "nan" => return Ok(f64::NAN),
        _ => {}
    }

    // Strip the required 0x / 0X prefix.
    let body = body
        .strip_prefix("0x")
        .or_else(|| body.strip_prefix("0X"))
        .ok_or_else(err)?;

    // Split mantissa and binary (p) exponent.
    let (mant, exp_str) = split_once_ci(body, 'p').ok_or_else(err)?;
    let exp: i64 = exp_str.parse().map_err(|_| err())?;

    // Split the integer part and the fraction.
    let (int_str, frac_str) = match mant.split_once('.') {
        Some((i, f)) => (i, f),
        None => (mant, ""),
    };

    // Canonical float.hex() always has a single leading digit, 0 or 1.
    let lead: u64 = match int_str {
        "0" => 0,
        "1" => 1,
        _ => return Err(err()),
    };

    let frac_len = frac_str.len();
    if frac_len > 13 {
        // More than 52 bits of fraction cannot be a canonical double form.
        return Err(err());
    }
    let frac_int: u64 = if frac_str.is_empty() {
        0
    } else {
        u64::from_str_radix(frac_str, 16).map_err(|_| err())?
    };
    // Left-align the fraction hex digits into the 52-bit mantissa field.
    let mantissa_bits = frac_int << (52 - 4 * frac_len);

    let bits = if lead == 1 {
        // Normal: exp is the unbiased exponent.
        let exp_field = exp + BIAS;
        if !(1..=2046).contains(&exp_field) {
            return Err(err());
        }
        (sign_bit << 63) | ((exp_field as u64) << 52) | mantissa_bits
    } else if frac_int == 0 {
        // ±0.0 (leading 0, empty/zero fraction).
        sign_bit << 63
    } else {
        // Subnormal: canonically written with exponent -1022.
        if exp != SUBNORMAL_EXP {
            return Err(err());
        }
        (sign_bit << 63) | mantissa_bits
    };

    Ok(f64::from_bits(bits))
}

/// Split on the first ASCII `p`/`P`, returning the halves without the separator.
fn split_once_ci(s: &str, sep: char) -> Option<(&str, &str)> {
    let up = sep.to_ascii_uppercase();
    let lo = sep.to_ascii_lowercase();
    let idx = s.find([up, lo])?;
    Some((&s[..idx], &s[idx + 1..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bits → format → parse → bits, for a spread of hand-picked doubles. The
    /// Python-vector round-trip lives in `tests/hexfloat_roundtrip.rs`; this is a
    /// fast in-crate smoke check.
    #[test]
    fn round_trip_bits() {
        let samples: &[u64] = &[
            0x0000_0000_0000_0000, // +0.0
            0x8000_0000_0000_0000, // -0.0
            0x3ff0_0000_0000_0000, // 1.0
            0xbff0_0000_0000_0000, // -1.0
            0x4000_0000_0000_0000, // 2.0
            0x3fe0_0000_0000_0000, // 0.5
            0x0000_0000_0000_0001, // min subnormal
            0x000f_ffff_ffff_ffff, // max subnormal
            0x0010_0000_0000_0000, // min normal
            0x7fef_ffff_ffff_ffff, // max normal
            0x3fb9_9999_9999_999a, // 0.1
            0x4009_21fb_5444_2d18, // pi
        ];
        for &b in samples {
            let x = f64::from_bits(b);
            let s = format(x);
            let back = parse(&s).expect("parse must accept our own format output");
            assert_eq!(back.to_bits(), b, "round-trip failed for {s}");
        }
    }

    #[test]
    fn zero_spellings() {
        assert_eq!(format(0.0), "0x0.0p+0");
        assert_eq!(format(-0.0), "-0x0.0p+0");
        assert_eq!(parse("0x0.0p+0").unwrap().to_bits(), 0);
        assert_eq!(parse("-0x0.0p+0").unwrap().to_bits(), 1u64 << 63);
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse("").is_err());
        assert!(parse("1.0").is_err()); // no 0x
        assert!(parse("0x1.0").is_err()); // no exponent
        assert!(parse("0x2.0p+0").is_err()); // non-canonical lead digit
    }
}
