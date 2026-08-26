//! The cross-port tolerance contract — its **shape** gates, and the station's banded runs.
//!
//! Ported from `tests/crossport/test_crossport.py` (reference flip, Stage 3, §5ac decision 4).
//! The shape gates live here because `station` is the only crate that can see **both** frozen
//! rosters, and the contract's first claim is about the two of them together.
//!
//! # The contract this file defends
//!
//! `docs/native-port-reference.md` freezes a *tolerance*, not code: which goldens are
//! bit-exact candidates, which carry a measured band, and what those bands are. Until this
//! slice the numbers lived in `tests/crossport/tiers.json` — inside the tree S6 deletes, read
//! by no program in `rust/`. The data moved to `rust/data/tiers.json` and the reference now
//! reads it; that move is the **unfreeze**, and no band changed in it.

use domains::tiers::{self, TierEntry};
use station::goldens::all;

fn frozen_goldens() -> Vec<String> {
    let mut names: Vec<String> = domains::freeze_manifest::frozen_goldens()
        .into_iter()
        .chain(station::freeze_manifest::frozen_goldens())
        .map(|s| s.to_string())
        .collect();
    names.sort();
    names.dedup();
    names
}

#[test]
fn the_contract_classifies_exactly_the_frozen_goldens() {
    // Both directions, and that is the point: an unclassified frozen golden has no tolerance
    // at all, and a classified golden that nothing freezes is a row defending nothing.
    let frozen = frozen_goldens();
    let mut classified: Vec<String> = tiers::entries().into_iter().map(|e| e.golden).collect();
    classified.sort();
    classified.dedup();
    assert_eq!(
        classified,
        frozen,
        "the tolerance contract must classify exactly the frozen goldens.\n  frozen but \
         unclassified: {:?}\n  classified but not frozen: {:?}",
        frozen
            .iter()
            .filter(|n| !classified.contains(n))
            .collect::<Vec<_>>(),
        classified
            .iter()
            .filter(|n| !frozen.contains(n))
            .collect::<Vec<_>>(),
    );
    // ⚠ The roster is read from the two manifests' own source rather than from the committed
    // `.manifest.json` documents, so this cannot be satisfied by editing a doc. And it is
    // pinned at 20 because "both sides are empty" would otherwise satisfy the equality above.
    assert_eq!(
        frozen.len(),
        20,
        "the frozen roster is {} goldens, not the 20 the contract is written against \
         (7 biosphere + 13 station)",
        frozen.len()
    );
}

#[test]
fn every_entry_is_internally_consistent() {
    for e in tiers::entries() {
        match e.float_tier {
            tiers::TIER_1_BIT_EXACT => {
                assert!(
                    e.transcendental_free,
                    "{}: Tier 1 means the evaluation graph reaches no transcendental; \
                     `transcendental_free` says otherwise",
                    e.golden
                );
                assert!(
                    e.band.is_none() && e.floor.is_none(),
                    "{}: Tier 1 is bit-exact — a band or floor here is a contradiction, and \
                     a comparator reading it would silently permit a difference",
                    e.golden
                );
            }
            tiers::TIER_2_BAND => {
                assert!(
                    !e.transcendental_free,
                    "{}: Tier 2 exists because a transcendental is reachable; if the graph is \
                     transcendental-free the row should be Tier 1",
                    e.golden
                );
                // Both-null (not yet measured) or both-positive (measured). Never one of each
                // — that is a half-calibrated row, and `compare_at_tier` would refuse it at
                // the worst possible moment.
                let both_null = e.band.is_none() && e.floor.is_none();
                let both_measured =
                    matches!((e.band, e.floor), (Some(b), Some(f)) if b > 0.0 && f > 0.0);
                assert!(
                    both_null || both_measured,
                    "{}: band/floor must be both-null (unmeasured) or both-positive \
                     (measured), got band={:?} floor={:?}",
                    e.golden,
                    e.band,
                    e.floor
                );
            }
            other => panic!("{}: unknown float tier {other}", e.golden),
        }
    }
}

#[test]
fn the_tier1_set_is_the_four_transcendental_free_scenarios() {
    // Pins the Phase-7 Step-0 verdict: exactly crew / eclss / cabin_gas / water_recovery are
    // bit-exact candidates. A golden quietly moving to Tier 2 would buy itself a tolerance it
    // was never entitled to, and nothing else in the suite would notice.
    let mut tier1: Vec<String> = tiers::entries()
        .into_iter()
        .filter(|e| e.float_tier == tiers::TIER_1_BIT_EXACT)
        .map(|e| e.golden)
        .collect();
    tier1.sort();
    assert_eq!(
        tier1,
        vec![
            "cabin_gas_state.json",
            "crew_state.json",
            "eclss_state.json",
            "water_recovery_state.json",
        ]
    );
}

#[test]
fn power_is_tier2_not_tier1() {
    let power = tiers::entry_for("power_state.json").expect("power is classified");
    assert_eq!(
        power.float_tier,
        tiers::TIER_2_BAND,
        "power is NOT transcendental-free — the diurnal forcing is a half-sine"
    );
    assert!(!power.transcendental_free);
}

// --------------------------------------------------------------------------- //
// The station's own runs, inside their measured bands                          //
// --------------------------------------------------------------------------- //

fn check_at_tier(name: &str, produced: &str, entry: &TierEntry) {
    let expected = domains::goldens::committed(name);
    if let Err(why) = tiers::compare_at_tier(&expected, produced, entry) {
        panic!(
            "{name}: the reference's own run is outside the cross-port tolerance contract.\n\
             {why}\n\
             ⚠ Never widen the band to make this pass. The bands are MEASURED (a propagated \
             ±1-ULP transcendental sensitivity), so widening one is an unfreeze of \
             docs/native-port-reference.md, not a fix."
        );
    }
}

#[test]
fn every_cheap_station_golden_is_inside_its_measured_band() {
    let mut checked = 0;
    for golden in all() {
        if golden.cost != domains::goldens::Cost::Cheap {
            continue;
        }
        let Some(entry) = tiers::entry_for(golden.name) else {
            continue;
        };
        checked += 1;
        check_at_tier(golden.name, &(golden.run)(), &entry);
    }
    assert!(
        checked >= 7,
        "only {checked} cheap station goldens are under the tolerance contract — either the \
         contract lost rows or the reader is not finding them"
    );
}

/// The expensive ones, on the same terms `golden_regression.rs` gives them: `#[ignore]`d
/// locally, run by CI's `cargo test -- --ignored` step.
#[test]
#[ignore = "minutes — the sealed-station horizon; CI runs it via --ignored"]
fn every_expensive_station_golden_is_inside_its_measured_band() {
    let mut checked = 0;
    for golden in all() {
        if golden.cost == domains::goldens::Cost::Cheap {
            continue;
        }
        let Some(entry) = tiers::entry_for(golden.name) else {
            continue;
        };
        checked += 1;
        check_at_tier(golden.name, &(golden.run)(), &entry);
    }
    assert!(
        checked >= 1,
        "no expensive station golden is under the tolerance contract — if that is now true, \
         this test has no subject and should be deleted rather than left passing"
    );
}
