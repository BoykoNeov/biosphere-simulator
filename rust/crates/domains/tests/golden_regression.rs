//! **The reference compares its own runs against the committed goldens** — the eleven
//! `domains` ones. Stage-3 slice S2 of the reference flip; the station's eight and the
//! whole-census gates are in `station/tests/golden_regression.rs`.
//!
//! # ⚠⚠ What did not exist before this file
//!
//! `docs/plans/post-roadmap-reference-flip.md` §5q, FINDING 3, searched exhaustively and
//! reported: **"No Rust test compares a run against a committed golden."** The reference
//! emitted 19 of the 21 goldens and every comparison lived in Python — in the tree S6
//! deletes. The regression contract, i.e. *this run still produces these bytes*, had no
//! owner on the side that is now canonical.
//!
//! The Python original is 17 `tests/test_regression_*.py` modules routed through
//! `tests/golden_platform.assert_matches_golden`, plus
//! `test_golden_provenance.test_rust_reproduces_the_committed_golden_bytes` (which shelled
//! out to `cargo run` for exactly the reason this file exists: an `examples/` program is a
//! binary and Python was the only caller that could reach it).
//!
//! ⚠ **This does not retire the Python side.** S2 builds the successor; S6 retires the
//! original, and only after S3–S5 have theirs. The two overlap deliberately — a slice that
//! deletes its predecessor before the successor has run in CI is how a gate goes missing.
//!
//! # What a failure here means
//!
//! Exactly one thing, and it is not "the ports disagree": since slice 5 the golden **is**
//! this code's output. So a mismatch is the reference moving. Decide whether the move was
//! intended; if it was, regenerate (`uv run python tests/crossport/regen_goldens_from_rust.py
//! --write`) and re-run the freeze-manifest ceremony for whichever contract names the file.
//! Never widen this test.

use domains::goldens::{committed, compare, Cost, Golden, Verdict, DOMAINS};

/// Compare one golden and produce a failure message that says which side to look at.
///
/// Shared with the station's file by duplication of three lines rather than by a helper
/// crate — deliberately: a `tests/` module cannot be shared across crates without a
/// published helper, and the alternative (a new workspace member for two functions) is the
/// speculative crate the workspace comment already refuses.
pub fn check(golden: &Golden) {
    let produced = (golden.run)();
    let expected = committed(golden.name);
    match compare(&produced, &expected, golden.numerics) {
        Verdict::ByteExact | Verdict::StructurallyEqual => {}
        Verdict::Differs(why) => panic!(
            "{}: the committed golden is no longer this run's output.\n\
             ⚠ Since the reference flip the golden IS the reference's bytes, so this is \
             the reference moving, not a port disagreement.\n\
             numerics: {:?} (platform: {})\n{why}",
            golden.name,
            golden.numerics,
            if cfg!(windows) {
                "windows — byte-exactness demanded"
            } else {
                "not the generation platform — transcendental goldens are compared \
                 structurally; see domains::goldens"
            },
        ),
    }
}

#[test]
fn every_domains_golden_is_still_this_reference_s_output() {
    for golden in DOMAINS {
        assert_eq!(
            golden.cost,
            Cost::Cheap,
            "{}: a `domains` golden became Expensive. It is not `#[ignore]`d here, so it \
             would silently slow every `cargo test`. Give it the treatment \
             `sealed_station_state.json` has in the station's file, and extend that \
             crate's ignored-set control to name it.",
            golden.name
        );
        check(golden);
    }
}

/// ⚠ The control on the file above, and it is the one that makes the platform policy
/// honest rather than decorative.
///
/// The comparison is *classified*, not skipped: [`Numerics::PureArithmetic`] goldens are
/// byte-compared everywhere and transcendental ones structurally off Windows. That leaves
/// one silent failure mode — a golden reclassified as transcendental to make a real
/// regression go away on CI. This asserts the pure-arithmetic set is exactly the two the
/// Python side leaves ungated in this crate (`crew`, `eclss` — no `@windows_golden_only`
/// marker in `tests/test_regression_{crew,eclss}.py`), so a reclassification is a visible
/// edit to this literal rather than a one-word change in a roster.
#[test]
fn the_pure_arithmetic_set_is_what_the_python_policy_says_it_is() {
    use domains::goldens::Numerics;
    let mut pure: Vec<&str> = DOMAINS
        .iter()
        .filter(|g| g.numerics == Numerics::PureArithmetic)
        .map(|g| g.name)
        .collect();
    pure.sort();
    assert_eq!(
        pure,
        vec!["crew_state.json", "eclss_state.json"],
        "the transcendental classification moved. It is inherited from \
         `@windows_golden_only`'s placement in tests/test_regression_*.py, NOT chosen \
         here — a golden becoming `Transcendental` weakens its gate off Windows, so the \
         change belongs in the same commit as the reason for it."
    );
}
