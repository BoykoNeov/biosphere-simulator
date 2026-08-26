//! Provenance hashing for the frozen param files — the **census + normalization** half of
//! `param_files`, moved to the reference (`docs/plans/post-roadmap-reference-flip.md`, the
//! successor slice C1 named).
//!
//! # ⚠⚠ What actually moves to the reference here, and what does NOT
//!
//! **The twenty-three recorded digits are author-neutral by construction.** Both sides
//! compute the same digest from the same file, which is precisely why re-anchoring
//! `param_files` moves no value: measured before a line of this module was written, all 15
//! biosphere + 8 station hashes in the two manifests reproduce under Python's own rule and
//! under the rule implemented here. So *"`param_files` is now Rust's"* would be the wrong
//! headline. What moves is the pair of **rules**:
//!
//! 1. the **census** — which files count. Python's rule is a non-recursive glob of a
//!    package directory minus `demo.yaml`; the reference's is *the set it actually loads*,
//!    a compile-time `include_str!` list, cross-checked against the directory.
//! 2. the **normalization** — how bytes become a digest ([`normalize_newlines`]).
//!
//! # ⚠⚠ Why the newline rule is load-bearing TODAY, and not for the reason you would guess
//!
//! Measured with `git ls-files --eol` over all 24 param files: the index is **LF on every
//! one** and `.gitattributes` declares `eol=lf` — but the *working tree* copy of
//! `senescence.yaml` on the development box is **CRLF**. So the plausible story ("autocrlf
//! converts on checkout") is **false**; it would have hit all 24. What is true is narrower
//! and worse: `include_str!` embeds the **working tree**, so the reference's own
//! compiled-in bytes for one frozen param file differ between that box and Linux CI right
//! now. Without normalization the hash this module produces is platform-dependent and a
//! regenerated manifest is red on the other machine.
//!
//! # The rule, and why it is narrower than Python's
//!
//! Python's is `hashlib.sha256("\n".join(Path.read_text().splitlines()))`. `read_text`
//! applies universal-newline translation, and `str.splitlines` *additionally* splits on
//! eight further characters — vertical tab, form feed, the three ASCII separators, NEL, LS
//! and PS — enumerated by codepoint in [`EXOTIC_LINE_SEPARATORS`] (written as escapes there,
//! deliberately, so this source file contains none of them itself).
//! Reimplementing that set would be reimplementing a Python method. Instead the rule
//! here is the narrow one —
//! `\r\n` and lone `\r` become `\n`, then one trailing `\n` is dropped — and
//! [`contains_exotic_line_separator`] makes the difference **unreachable** rather than
//! merely absent: a param file carrying any of those characters is rejected by a gate, so
//! the two rules cannot silently disagree.

/// The Unicode/ASCII characters Python's `str.splitlines` treats as line breaks and this
/// module deliberately does not.
///
/// A file containing any of these would hash differently under the two rules. None of the
/// 24 frozen param files does (measured), and a gate keeps it that way.
pub const EXOTIC_LINE_SEPARATORS: [char; 8] = [
    '\u{000b}', // VT  — vertical tab
    '\u{000c}', // FF  — form feed
    '\u{001c}', // FS  — file separator
    '\u{001d}', // GS  — group separator
    '\u{001e}', // RS  — record separator
    '\u{0085}', // NEL — next line
    '\u{2028}', // LS  — line separator
    '\u{2029}', // PS  — paragraph separator
];

/// The first exotic line separator in `text`, if any — the guard that keeps this module's
/// narrow rule and Python's `splitlines` from being able to disagree.
pub fn contains_exotic_line_separator(text: &str) -> Option<char> {
    text.chars().find(|c| EXOTIC_LINE_SEPARATORS.contains(c))
}

/// Newline-normalize `text`: `\r\n` and lone `\r` become `\n`, then **one** trailing `\n`
/// is dropped.
///
/// The trailing-newline rule is not cosmetic — it is what Python's
/// `"\n".join(text.splitlines())` does: `"a\n"` folds to `"a"` and `"a\n\n"` to `"a\n"`.
pub fn normalize_newlines(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\r' {
            // `\r\n` collapses to one `\n`; a lone `\r` becomes one.
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            out.push('\n');
        } else {
            out.push(c);
        }
    }
    if out.ends_with('\n') {
        out.pop();
    }
    out
}

/// sha-256 of `text` after [`normalize_newlines`], lowercase hex — the value the frozen
/// manifests record under `param_files`.
pub fn normalized_sha256(text: &str) -> String {
    hex(&sha256(normalize_newlines(text).as_bytes()))
}

/// sha-256 of `message` as lowercase hex, with **no** newline normalization.
///
/// The normalized form above is for *files*, whose line endings are a checkout artifact.
/// This is for text the reference itself assembled — the light-path fingerprint, which is
/// hex-float samples joined by `|` and therefore has no line endings to normalize. Added
/// in slice C7, so the manifest writer does not have to route a newline-free string
/// through a rule about newlines to say what it means.
pub fn sha256_hex(message: &[u8]) -> String {
    hex(&sha256(message))
}

fn hex(digest: &[u8; 32]) -> String {
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(HEX_DIGITS[usize::from(byte >> 4)]);
        out.push_str(HEX_DIGITS[usize::from(byte & 0x0f)]);
    }
    out
}

const HEX_DIGITS: [&str; 16] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f",
];

// --------------------------------------------------------------------------- //
// sha-256 (FIPS 180-4), by hand.                                               //
// --------------------------------------------------------------------------- //
//
// ⚠ Hand-rolled because **every engine crate is zero-dep by charter** and this one sits at
// the very bottom of the layering. That is the same trade `yaml.rs` documents, with the
// same discipline behind it: a hand-rolled primitive earns published test vectors. Those
// live in `tests/sha256_vectors.rs`, against an INDEPENDENT oracle, and that file's
// docstring records both why (an earlier in-crate test re-derived the padding with a copy of
// the loop below — two copies guaranteed to agree) and what the frozen param files do not
// cover on their own.

/// The first 32 bits of the fractional parts of the cube roots of the first 64 primes.
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// The first 32 bits of the fractional parts of the square roots of the first 8 primes.
const H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// sha-256 of `message`, as the raw 32-byte digest.
pub fn sha256(message: &[u8]) -> [u8; 32] {
    let mut h = H0;

    // Padding: a `0x80` byte, then zeros, then the bit length as a big-endian u64, so the
    // total is a multiple of 64. ⚠ When `len % 64 >= 56` the length field does not fit in
    // the current block and the padding spills into an extra one — the case a hand-rolled
    // implementation gets wrong and the frozen files only incidentally cover.
    let bit_len = (message.len() as u64).wrapping_mul(8);
    let mut padded = Vec::with_capacity(message.len() + 72);
    padded.extend_from_slice(message);
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());
    debug_assert_eq!(padded.len() % 64, 0);

    for block in padded.as_chunks::<64>().0 {
        let mut w = [0u32; 64];
        for (i, word) in block.as_chunks::<4>().0.iter().enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        for (slot, value) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *slot = slot.wrapping_add(value);
        }
    }

    let mut digest = [0u8; 32];
    for (chunk, word) in digest.as_chunks_mut::<4>().0.iter_mut().zip(h) {
        chunk.copy_from_slice(&word.to_be_bytes());
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::*;

    // ⚠ The sha-256 vectors deliberately live in `tests/sha256_vectors.rs`, against an
    // INDEPENDENT oracle (CPython's hashlib / OpenSSL) plus the published FIPS 180-4
    // strings. An earlier in-crate test asserted the padding invariants by re-deriving the
    // padding with a copy of the loop below — two copies of the code under test, guaranteed
    // to agree, which is a weak test wearing a strong one's clothes. That file's docstring
    // records why.

    #[test]
    fn normalization_folds_crlf_lone_cr_and_one_trailing_newline() {
        assert_eq!(normalize_newlines("a\r\nb\r\nc\r\n"), "a\nb\nc");
        assert_eq!(normalize_newlines("a\nb\nc\n"), "a\nb\nc");
        assert_eq!(normalize_newlines("a\rb\rc\r"), "a\nb\nc");
        // Exactly ONE trailing newline goes, matching `"\n".join(text.splitlines())`.
        assert_eq!(normalize_newlines("a\n\n"), "a\n");
        assert_eq!(normalize_newlines("a"), "a");
        assert_eq!(normalize_newlines(""), "");
        assert_eq!(normalize_newlines("\n"), "");
        // The whole point: the three spellings of one file agree.
        let lf = normalized_sha256("name: x\nprocess: y\n");
        assert_eq!(lf, normalized_sha256("name: x\r\nprocess: y\r\n"));
        assert_eq!(lf, normalized_sha256("name: x\rprocess: y\r"));
        assert_eq!(lf, normalized_sha256("name: x\nprocess: y"));
    }

    #[test]
    fn the_exotic_separator_guard_finds_what_normalization_does_not_fold() {
        assert_eq!(contains_exotic_line_separator("plain\ntext\r\n"), None);
        for c in EXOTIC_LINE_SEPARATORS {
            let text = format!("a{c}b");
            assert_eq!(
                contains_exotic_line_separator(&text),
                Some(c),
                "{c:?} not detected"
            );
            // ⚠ And the reason the guard exists: normalization leaves it in place, so the
            // digest here and Python's `splitlines`-based one WOULD differ on this input.
            assert!(normalize_newlines(&text).contains(c));
        }
    }
}
