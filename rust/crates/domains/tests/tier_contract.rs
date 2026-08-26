//! The cross-port tolerance contract, enforced by the reference — the `domains` half.
//!
//! Ported from `tests/crossport/test_crossport.py`'s banded comparisons (reference flip,
//! Stage 3, §5ac decision 4). The station's eight goldens and the contract's *shape* gates
//! are in `station/tests/tier_contract.rs`, which is the only crate that can see both rosters.
//!
//! # What this adds that `golden_regression.rs` cannot
//!
//! That file compares a fresh run against its committed golden under a **platform policy**:
//! byte-exact for pure-arithmetic goldens and on Windows, and otherwise a *structural* walk
//! that asserts each hex-float leaf parses finite and says **nothing about its value**. So on
//! Linux — the one CI job that is a genuine cross-libm measurement, glibc Rust against
//! UCRT-generated goldens — the reference had no numeric assertion at all. This file supplies
//! it: every leaf, within the golden's own **measured** band.
//!
//! The two are deliberately both kept. Structural equality catches a leaf that turned into a
//! label or a `NaN`; the band catches a value that moved. Neither implies the other.
//!
//! ⚠ On Windows this file is nearly free and reads as trivially green — the runs are
//! byte-identical to their goldens, so every deviation is exactly 0.0. That is not evidence
//! the gate works, which is why `the_band_arithmetic_rejects_what_it_should` below tests the
//! comparison itself on constructed input rather than on a run that cannot fail.

use domains::goldens::{committed, Cost, DOMAINS};
use domains::tiers::{self, TierEntry};

/// Compare one golden's fresh run against the committed bytes at its contracted tier.
fn check_at_tier(name: &str, produced: &str) {
    let Some(entry) = tiers::entry_for(name) else {
        // Not every golden on disk is under the tolerance contract — the contract pins 20 of
        // them. An unclassified golden is not a failure here; the *station* file asserts the
        // classified set is exactly the frozen set, which is where that claim belongs.
        return;
    };
    let expected = committed(name);
    match tiers::compare_at_tier(&expected, produced, &entry) {
        Ok(worst) => {
            // A run sitting near its band is a finding even though it passes: the band was
            // measured as a 1-ULP libm sensitivity, so most of it should be unused.
            if let Some(band) = entry.band {
                assert!(
                    worst <= band,
                    "unreachable: compare_at_tier returned Ok above its own band"
                );
            }
        }
        Err(why) => panic!(
            "{name}: the reference's own run is outside the cross-port tolerance contract.\n\
             {why}\n\
             ⚠ Since the reference flip the golden IS this code's output, so on the generation \
             platform this means the reference moved. Off it, this is the cross-libm gate: a \
             band that was measured as a 1-ULP transcendental sensitivity has been exceeded, \
             which is a real numeric divergence and not rounding. Never widen the band to make \
             this pass — the band is measured, and widening it is an unfreeze of \
             docs/native-port-reference.md."
        ),
    }
}

#[test]
fn every_classified_domains_golden_is_inside_its_measured_band() {
    let mut checked = 0;
    for golden in DOMAINS {
        assert_eq!(
            golden.cost,
            Cost::Cheap,
            "{}: a `domains` golden became Expensive and would silently slow every \
             `cargo test` here, exactly as in golden_regression.rs",
            golden.name
        );
        if tiers::entry_for(golden.name).is_some() {
            checked += 1;
            check_at_tier(golden.name, &(golden.run)());
        }
    }
    // ⚠ The anti-vacuity clause. Without it a `tiers.json` that stopped naming any `domains`
    // golden — or a reader that silently returned an empty table — would make this test pass
    // by checking nothing, which is the exact failure mode Stage 3 keeps finding.
    assert!(
        checked >= 7,
        "only {checked} `domains` goldens are under the tolerance contract; the biosphere \
         set alone is 7. Either the contract lost rows or the reader is not finding them."
    );
}

// --------------------------------------------------------------------------- //
// The comparison itself — tested on constructed input, not on runs that cannot fail //
// --------------------------------------------------------------------------- //

fn entry(tier: u8, band: Option<f64>, floor: Option<f64>) -> TierEntry {
    TierEntry {
        golden: "probe.json".to_string(),
        key: "probe".to_string(),
        float_tier: tier,
        transcendental_free: tier == 1,
        band,
        floor,
    }
}

/// Two one-leaf snapshots whose values differ by a chosen amount.
fn pair(reference: f64, candidate: f64) -> (String, String) {
    let hex = |v: f64| simcore::hexfloat::format(v);
    (
        format!("{{\"a\": \"{}\"}}", hex(reference)),
        format!("{{\"a\": \"{}\"}}", hex(candidate)),
    )
}

#[test]
fn the_band_arithmetic_rejects_what_it_should() {
    let e = entry(2, Some(1e-12), Some(1e-12));

    // Inside the band passes...
    let (r, c) = pair(1.0, 1.0 + 5e-13);
    assert!(tiers::compare_at_tier(&r, &c, &e).is_ok());

    // ...and outside it fails, by name and with the number.
    let (r, c) = pair(1.0, 1.0 + 5e-11);
    let err = tiers::compare_at_tier(&r, &c, &e).expect_err("50x the band must fail");
    assert!(err.contains("exceeds the measured band"), "{err}");
}

#[test]
fn tier_one_is_bit_exact_and_a_band_cannot_rescue_it() {
    // ⚠ The clause that keeps Tier 1 meaning what it says. A Tier-1 scenario reaches no
    // transcendental, so the two sides must agree to the last bit on ANY platform; a
    // difference is a defect, never libm. Handing the comparison a generous band must not
    // change that.
    let e = entry(1, Some(1e-3), Some(1e-12));
    let (r, c) = pair(1.0, 1.0 + f64::EPSILON);
    let err = tiers::compare_at_tier(&r, &c, &e).expect_err("one ULP must fail Tier 1");
    assert!(err.contains("BIT-EXACT"), "{err}");
}

#[test]
fn a_tier_two_entry_without_a_measured_band_refuses_to_compare() {
    // The contract's own rule: bands are measured, never derived. An uncalibrated row must
    // be an error, not a silently permissive comparison.
    let (r, c) = pair(1.0, 2.0);
    for e in [entry(2, None, None), entry(2, Some(1e-12), None)] {
        let err = tiers::compare_at_tier(&r, &c, &e).expect_err("must refuse");
        assert!(err.contains("will not invent"), "{err}");
    }
}

#[test]
fn the_floor_is_permissive_and_only_near_zero() {
    // ⚠ Stated as a test because the natural assumption runs the other way. `floor` enlarges
    // the denominator when |reference| < floor, so it makes the comparison MORE forgiving on
    // near-zero leaves and does nothing at all elsewhere. A "simplification" that dropped it
    // would make this stricter — it would fail loudly, not pass quietly — and this pins the
    // direction so nobody has to re-derive it.
    let leaves =
        tiers::paired_leaves(&pair(1e-18, 1e-15).0, &pair(1e-18, 1e-15).1).expect("one leaf each");
    let (with_floor, _) = tiers::max_abs_relative_deviation(&leaves, 1e-12);
    let (tight_floor, _) = tiers::max_abs_relative_deviation(&leaves, 1e-30);
    assert!(
        with_floor < tight_floor,
        "a larger floor must REDUCE the reported deviation on a near-zero leaf: \
         {with_floor:e} vs {tight_floor:e}"
    );

    // ...and away from zero the floor is inert.
    let big = tiers::paired_leaves(&pair(1.0, 1.0 + 1e-13).0, &pair(1.0, 1.0 + 1e-13).1)
        .expect("one leaf each");
    let (a, _) = tiers::max_abs_relative_deviation(&big, 1e-12);
    let (b, _) = tiers::max_abs_relative_deviation(&big, 1e-30);
    assert_eq!(a, b, "the floor must not touch a leaf of magnitude 1");
}

#[test]
fn a_shape_mismatch_is_an_error_and_an_empty_comparison_is_not_a_pass() {
    // Two snapshots of different shape must fail rather than compare the leaves they happen
    // to share...
    let err = tiers::paired_leaves(r#"{"a": "0x1p+0"}"#, r#"{"a": "0x1p+0", "b": "0x1p+0"}"#)
        .expect_err("differing key counts must fail");
    assert!(err.contains("key count"), "{err}");

    // ...and a comparison that finds NO numeric leaves is a failure, not a vacuous pass.
    let err = tiers::paired_leaves(r#"{"unit": "kg"}"#, r#"{"unit": "kg"}"#)
        .expect_err("no numeric leaves must fail");
    assert!(err.contains("vacuously true"), "{err}");
}

#[test]
fn the_structural_walk_is_blind_to_the_value_this_file_checks() {
    // ⚠⚠ The reason this file exists, demonstrated rather than asserted — and demonstrable on
    // ANY platform, which the golden-nudging control is not.
    //
    // `goldens::compare_structural` is what `goldens::compare` falls back to for a
    // transcendental golden off its generation platform, i.e. on the Linux CI job that is the
    // repo's only genuine cross-libm measurement. Hand it two snapshots whose values differ by
    // ten times a measured band and it reports **equal**: it checks that a hex-float leaf
    // parses finite, never what it parses to.
    let (r, c) = pair(1.0, 1.0 + 1e-11);
    assert!(
        domains::goldens::compare_structural(&c, &r).is_ok(),
        "the structural walk was expected to be value-blind; if it now compares values, this          file's justification has changed and should be rewritten rather than deleted"
    );

    // The same pair, under the tolerance contract at power's measured band, is a failure.
    let e = entry(2, Some(1e-12), Some(1e-12));
    assert!(
        tiers::compare_at_tier(&r, &c, &e).is_err(),
        "the banded comparison must catch what the structural walk cannot — otherwise this          whole file adds nothing off Windows"
    );
}
