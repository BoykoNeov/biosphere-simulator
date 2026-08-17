//! The hand-rolled sha-256 against an **independent** oracle — slice C8 of the reference
//! flip (`docs/plans/post-roadmap-reference-flip.md`).
//!
//! # ⚠⚠ Why this file replaced an in-crate padding test that looked stronger than it was
//!
//! The first version of this coverage lived in `provenance.rs` and asserted that the padded
//! length is a multiple of 64, at least `len + 9` bytes, and less than `len + 73`. Every one
//! of those is true — and the test **re-derived the padding with a copy of the same loop the
//! implementation uses**. A second copy of the code under test is not an oracle; if the loop
//! were wrong, both copies would be wrong together and the test would be green. *A policy
//! with two implementations has one that is stale* (`docs/log/reference-flip.md`, slice 5),
//! reached here in its sharper form: the two copies were not merely at risk of diverging,
//! they were **guaranteed to agree**.
//!
//! What replaces it is 201 digests minted by CPython's `hashlib` — OpenSSL's sha-256, an
//! implementation with no shared ancestry with ours — plus the four published FIPS 180-4
//! string vectors below.
//!
//! # ⚠ Why the frozen param files are NOT this coverage
//!
//! `param_files` gives 23 real digests to compare against the manifests, which is a strong
//! end-to-end check and a weak *padding* check: measured, only **two** of the 24 normalized
//! lengths (`phenology.yaml` at 58 and `radiator.yaml` at exactly 56) land in the
//! `len % 64 >= 56` window where the length field spills into a second block, **no** file is
//! short enough to be a single block, and none is empty. That coverage is incidental — a
//! one-character content edit to either file removes it, and nothing would say so.

use config::provenance::{normalized_sha256, sha256};

const VECTORS: &str = include_str!("data/sha256_vectors.txt");

fn hex(digest: [u8; 32]) -> String {
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

#[test]
fn sha256_matches_the_independent_oracle_at_every_length_through_200() {
    let mut checked = 0usize;
    let mut spill_cases = 0usize;
    let mut block_boundaries = 0usize;
    for line in VECTORS.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (len, expected) = line.split_once(' ').expect("`<len> <digest>` per line");
        let len: usize = len.parse().expect("length is an integer");
        assert_eq!(expected.len(), 64, "len={len}: not a sha-256 digest");
        assert_eq!(
            hex(sha256(&vec![b'a'; len])),
            expected,
            "sha-256 diverged from the oracle at length {len}"
        );
        checked += 1;
        if len % 64 >= 56 {
            spill_cases += 1;
        }
        if len > 0 && len.is_multiple_of(64) {
            block_boundaries += 1;
        }
    }
    // Guards against a silently-truncated or unresolved `include_str!` path — the same
    // check `rng_vectors.rs` and `hexfloat_roundtrip.rs` carry, for the same reason.
    assert_eq!(checked, 201, "expected 201 vectors, read {checked}");
    // ⚠ And against the file being *replaced* by one that no longer covers the case it
    // exists for. A vector file whose lengths all avoided the spill window would pass every
    // assertion above while testing nothing about padding.
    assert!(
        spill_cases >= 20,
        "only {spill_cases} vectors land in the len % 64 >= 56 spill window"
    );
    assert!(
        block_boundaries >= 3,
        "only {block_boundaries} vectors sit on an exact block boundary"
    );
}

#[test]
fn sha256_matches_the_published_fips_180_4_string_vectors() {
    // The standard's own examples, kept alongside the length sweep because they fix the
    // *compression function* against a document rather than against another program.
    let cases: [(&str, &str); 4] = [
        (
            "",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            "abc",
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
        (
            // 56 bytes — `56 % 64 == 56`, so this vector IS the spill boundary.
            "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
        ),
        (
            // 112 bytes — two exact blocks, no spill.
            "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmno\
             ijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
            "cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1",
        ),
    ];
    for (message, expected) in cases {
        assert_eq!(hex(sha256(message.as_bytes())), expected, "{message:?}");
    }
    // The lengths are asserted so a re-typed vector cannot silently stop covering the
    // boundary it was chosen for.
    assert_eq!(cases[2].0.len(), 56, "the spill-boundary vector moved");
    assert_eq!(cases[3].0.len(), 112, "the two-block vector moved");
}

#[test]
fn the_normalized_digest_composes_the_two_halves() {
    // `normalized_sha256` is normalization THEN sha-256, and the seam is worth pinning:
    // a version that hashed the raw text would still pass every vector above.
    assert_eq!(
        normalized_sha256("abc\r\n"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        "normalization did not run before the digest"
    );
    assert_eq!(normalized_sha256("abc"), normalized_sha256("abc\n"));
}
