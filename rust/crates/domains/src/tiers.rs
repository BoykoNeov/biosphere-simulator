//! The cross-port **tolerance contract** — the reference's own reader for
//! `rust/data/tiers.json`, and the banded comparison that reads it.
//!
//! # What was missing, and why this is not a refactor
//!
//! `docs/native-port-reference.md` is one of the four freeze contracts, and its numbers —
//! the float tier per golden, the measured band, the floor — lived in `tests/crossport/`,
//! **read by no program in `rust/`** (the reference flip's plan, FINDING 2, fifth entry).
//! Meanwhile [`crate::goldens::compare`] carries **no numeric tolerance at all**: byte-exact
//! for [`Numerics::PureArithmetic`](crate::goldens::Numerics) or on Windows, and otherwise a
//! *structural* walk that asserts a hex-float leaf parses finite and says nothing about its
//! value. So on the one CI job that is a genuine cross-libm measurement — glibc Rust against
//! UCRT-generated goldens — the only assertion with teeth was Python's.
//!
//! Moving the data under `rust/` and giving the reference its own banded comparison is an
//! **unfreeze** of that contract, not a tidy-up. No band moved; what changed is which side
//! owns them.
//!
//! # The arithmetic, and the direction the floor pushes
//!
//! Tier 1 is bit-exact on parsed f64. Tier 2 is
//!
//! ```text
//! max over leaves of  |candidate - reference| / max(|reference|, floor)  <=  band
//! ```
//!
//! ⚠ The `floor` is **permissive, not restrictive**: it enlarges the denominator when a
//! reference leaf is smaller than it, so a near-zero leaf cannot blow the ratio up on an
//! absolutely-tiny difference. Dropping it would make the comparison *stricter*, not weaker —
//! worth stating because the natural assumption runs the other way, and a "simplification"
//! that removed it would fail loudly rather than pass quietly.
//!
//! Every band in the committed table is measured, never derived. This module will not invent
//! one: a Tier-2 entry whose band or floor is null is an error at comparison time, exactly as
//! the Python original refuses to compare without both.

use std::path::PathBuf;

use simcore::json::{self, JsonValue};

/// Bit-exact on parsed f64 (not on JSON bytes — a formatting change is not a science change).
pub const TIER_1_BIT_EXACT: u8 = 1;
/// Within the measured relative band.
pub const TIER_2_BAND: u8 = 2;

/// One golden's row of the tolerance contract.
#[derive(Debug, Clone)]
pub struct TierEntry {
    /// The golden's file name in `rust/data/golden/`.
    pub golden: String,
    /// The scenario key the band was measured against.
    pub key: String,
    /// `1` or `2`; see the constants above.
    pub float_tier: u8,
    /// Whether the per-step evaluation graph reaches a transcendental at all.
    pub transcendental_free: bool,
    /// The measured relative band — `None` only for Tier 1, where it must be absent.
    pub band: Option<f64>,
    /// The relative-error floor that goes with the band.
    pub floor: Option<f64>,
}

/// `rust/data/tiers.json` — beside the goldens it classifies.
pub fn tiers_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/tiers.json")
}

fn number(value: &JsonValue, what: &str) -> f64 {
    match value {
        JsonValue::Number(lexeme) => lexeme
            .parse::<f64>()
            .unwrap_or_else(|e| panic!("{what} = {lexeme:?} is not a number: {e}")),
        other => panic!("{what} is not a number: {other:?}"),
    }
}

fn optional_number(entry: &JsonValue, key: &str) -> Option<f64> {
    match entry.get(key) {
        None | Some(JsonValue::Null) => None,
        Some(value) => Some(number(value, key)),
    }
}

/// Every classified golden, in the order the committed table lists them.
///
/// ⚠ Read from the file rather than mirrored in Rust. A hand-copied table would be a second
/// source of truth for a frozen contract, which is the defect
/// `docs/log/coverage-roster-is-not-the-manifest.md` records this repo already making once.
pub fn entries() -> Vec<TierEntry> {
    let path = tiers_path();
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read the tolerance contract {}: {e}", path.display()));
    let root =
        json::parse(&text).unwrap_or_else(|e| panic!("{} is not JSON: {e:?}", path.display()));
    let goldens = match root.get("goldens") {
        Some(JsonValue::Array(items)) => items,
        other => panic!("tiers.json has no `goldens` array: {other:?}"),
    };
    goldens
        .iter()
        .map(|entry| {
            let text_field = |key: &str| match entry.get(key).and_then(JsonValue::as_str) {
                Some(s) => s.to_string(),
                None => panic!("a tiers.json entry has no string {key:?}: {entry:?}"),
            };
            let tier = number(entry.get("float_tier").expect("float_tier"), "float_tier");
            TierEntry {
                golden: text_field("golden"),
                key: text_field("key"),
                float_tier: tier as u8,
                transcendental_free: entry
                    .get("transcendental_free")
                    .and_then(JsonValue::as_bool)
                    .unwrap_or_else(|| panic!("no transcendental_free in {entry:?}")),
                band: optional_number(entry, "band"),
                floor: optional_number(entry, "floor"),
            }
        })
        .collect()
}

/// The row for one golden, by file name.
pub fn entry_for(golden: &str) -> Option<TierEntry> {
    entries().into_iter().find(|e| e.golden == golden)
}

/// One numeric leaf, paired across the two snapshots.
#[derive(Debug)]
pub struct Leaf {
    pub path: String,
    pub reference: f64,
    pub candidate: f64,
}

/// Pair every hex-float leaf of two snapshots, failing if the two trees are not the same
/// shape.
///
/// ⚠ A leaf is identified by **parsing**, not by key name — the snapshot codec spells floats
/// as strings (`"0x1.87e…p+3"`), so JSON's own types cannot tell a float from a unit label.
/// That is [`crate::goldens::compare_structural`]'s rule, kept identical here on purpose: two
/// different answers to "which leaves are numbers" would make the two gates disagree about
/// what they are even comparing.
pub fn paired_leaves(reference: &str, candidate: &str) -> Result<Vec<Leaf>, String> {
    let r = json::parse(reference).map_err(|e| format!("reference is not JSON: {e:?}"))?;
    let c = json::parse(candidate).map_err(|e| format!("candidate is not JSON: {e:?}"))?;
    let mut leaves = Vec::new();
    collect(&r, &c, "$", &mut leaves)?;
    if leaves.is_empty() {
        return Err(
            "the two snapshots share no numeric leaves at all — a comparison over an \
                    empty set is vacuously true, which is not a pass"
                .to_string(),
        );
    }
    Ok(leaves)
}

fn collect(
    reference: &JsonValue,
    candidate: &JsonValue,
    path: &str,
    out: &mut Vec<Leaf>,
) -> Result<(), String> {
    match (reference, candidate) {
        (JsonValue::Object(r), JsonValue::Object(c)) => {
            if r.len() != c.len() {
                return Err(format!("{path}: key count {} vs {}", r.len(), c.len()));
            }
            for ((rk, rv), (ck, cv)) in r.iter().zip(c.iter()) {
                if rk != ck {
                    return Err(format!("{path}: key {rk:?} where the candidate has {ck:?}"));
                }
                collect(rv, cv, &format!("{path}.{rk}"), out)?;
            }
            Ok(())
        }
        (JsonValue::Array(r), JsonValue::Array(c)) => {
            if r.len() != c.len() {
                return Err(format!("{path}: {} elements vs {}", r.len(), c.len()));
            }
            for (i, (rv, cv)) in r.iter().zip(c.iter()).enumerate() {
                collect(rv, cv, &format!("{path}[{i}]"), out)?;
            }
            Ok(())
        }
        (JsonValue::Str(r), JsonValue::Str(c)) => {
            match (
                simcore::hexfloat::parse(r).ok(),
                simcore::hexfloat::parse(c).ok(),
            ) {
                (Some(rf), Some(cf)) => {
                    if !rf.is_finite() || !cf.is_finite() {
                        return Err(format!("{path}: non-finite hex-float {r:?} vs {c:?}"));
                    }
                    out.push(Leaf {
                        path: path.to_string(),
                        reference: rf,
                        candidate: cf,
                    });
                    Ok(())
                }
                (None, None) => {
                    if r == c {
                        Ok(())
                    } else {
                        Err(format!("{path}: {r:?} vs {c:?}"))
                    }
                }
                _ => Err(format!(
                    "{path}: one side is a hex-float and the other is not — {r:?} vs {c:?}"
                )),
            }
        }
        _ => {
            if reference == candidate {
                Ok(())
            } else {
                Err(format!("{path}: {reference:?} vs {candidate:?}"))
            }
        }
    }
}

/// The worst pointwise relative deviation, and the leaf that produced it.
pub fn max_abs_relative_deviation(leaves: &[Leaf], floor: f64) -> (f64, &str) {
    assert!(
        floor > 0.0,
        "floor must be strictly positive, got {floor} — it is the scale below which a relative \
         error stops meaning anything, not an optional refinement"
    );
    let mut worst = 0.0_f64;
    let mut where_ = "";
    for leaf in leaves {
        let deviation = (leaf.candidate - leaf.reference).abs() / leaf.reference.abs().max(floor);
        if deviation >= worst {
            worst = deviation;
            where_ = leaf.path.as_str();
        }
    }
    (worst, where_)
}

/// Compare a fresh run against a committed golden under that golden's row of the contract.
///
/// Returns the measured worst deviation on success, so a caller can report how much of the
/// band a passing run actually used — a run sitting at 90 % of its band is a finding even
/// though it passes.
pub fn compare_at_tier(reference: &str, candidate: &str, entry: &TierEntry) -> Result<f64, String> {
    let leaves = paired_leaves(reference, candidate)?;
    match entry.float_tier {
        TIER_1_BIT_EXACT => {
            for leaf in &leaves {
                if leaf.reference.to_bits() != leaf.candidate.to_bits() {
                    return Err(format!(
                        "{}: Tier 1 is BIT-EXACT and {} differs — {:?} vs {:?}. This scenario's \
                         evaluation graph reaches no transcendental, so the two sides must agree \
                         to the last bit on any platform; a difference is a defect, never libm.",
                        entry.golden, leaf.path, leaf.reference, leaf.candidate
                    ));
                }
            }
            Ok(0.0)
        }
        TIER_2_BAND => {
            let (band, floor) = match (entry.band, entry.floor) {
                (Some(b), Some(f)) => (b, f),
                _ => {
                    return Err(format!(
                        "{}: a Tier-2 comparison needs an explicit MEASURED band and floor; they \
                         are null in the contract until calibrated. This module will not invent \
                         a tolerance.",
                        entry.golden
                    ))
                }
            };
            let (worst, at) = max_abs_relative_deviation(&leaves, floor);
            if worst > band {
                return Err(format!(
                    "{}: worst relative deviation {worst:.3e} at {at} exceeds the measured band \
                     {band:.3e} (floor {floor:.3e}) over {} leaves.",
                    entry.golden,
                    leaves.len()
                ));
            }
            Ok(worst)
        }
        other => Err(format!("{}: unknown float tier {other}", entry.golden)),
    }
}
