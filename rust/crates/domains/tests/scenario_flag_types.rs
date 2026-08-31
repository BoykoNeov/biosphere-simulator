//! A scenario **flag** can only subtract from the frozen type roster, never add to it.
//!
//! Slice 0 of the science-switch plan (`docs/plans/post-roadmap-science-switch.md`), and it
//! is owed whether or not the rest of that plan is ever built.
//!
//! ## The property, and the hole it closes
//!
//! `SeasonScenario` carries five boolean flags, and `compartments` branches on them: they
//! wire flows and aux processes in or out. That makes them **mechanism switches sitting in
//! production scenario config** — the very thing the science-switch seam is being built to
//! keep out of `build_season`.
//!
//! Today they are harmless, and the reason is worth stating because it is contingent rather
//! than structural: `DEFAULT_SCENARIO` sets `vernalization: true` and `stem_reserves: true`,
//! and the three chambers inherit it and add `sealed` (one of them `consumer`). So every
//! type any flag can wire is ON in at least one of the four canonical builds — which are
//! exactly the builds the freeze manifest's `flow_set` / `aux_set` are unioned from
//! (`biosphere::freeze_manifest::inventory`). Flip either literal in `DEFAULT_SCENARIO` to
//! `false` and `StemRemobilization` / `NitrogenSenescence` leave that union: the manifest is
//! **derived**, so it would follow the code silently, and a flag would have become a live
//! switch onto a mechanism outside the frozen contract with nothing in the tree noticing.
//!
//! ## Why the assertion is a subset and not a roster
//!
//! Listing which types each flag gates would put a second copy of the frozen manifest in the
//! tree, which `tests/type_identity.rs` refuses for the reason this repo has already paid
//! for: *a rule with two copies has one that is stale*. So the claim is comparative — every
//! reachable flag setting, on every canonical scenario, produces a type set **inside** the
//! canonical union — and it names nothing.
//!
//! ## Two anti-vacuity halves, because a subset check is passable by building nothing
//!
//! * [`each_flow_gating_flag_gains_at_least_one_type`] pins that four of the five flags
//!   really do add types when switched on, so the subset above has content — and pins
//!   `photoperiod` at **zero**, which makes the plan's "four flags, not five" a measurement
//!   rather than something inherited from a draft that also miscounted `perennial` as a
//!   fifth flag.
//! * [`the_toggle_roster_is_every_bool_field_of_the_scenario`] reads the `SeasonScenario`
//!   declaration and asserts this file toggles all of its `bool` fields. A sixth flag lands
//!   here by name instead of quietly escaping the scan — the derive-from-the-tree rule the
//!   science-gate census and `param_funnel.rs`'s loader roster both follow.
//!
//! ⚠ **What the sweep is, stated rather than implied: one flip from each of four canonical
//! bases** — 5 flags × 2 values × 4 scenarios, not the 32-point flag space. A type gated by a
//! conjunction lying two flips from every canonical base would be outside its reach. None is
//! today (`consumer` needs `sealed`, and the chambers supply it, so every real pair here is one
//! flip from some base), but that is a measurement of this roster and not a property of the
//! design — a claim about a space is dated to how it was measured.

use std::collections::BTreeSet;
use std::path::PathBuf;

use domains::biosphere::system::{
    build_season, consumer_chamber_scenario, perennial_chamber_scenario, sealed_chamber_scenario,
    SeasonScenario, DEFAULT_SCENARIO,
};

/// The four canonical builds — the same four `freeze_manifest::inventory` unions over.
fn canonical() -> Vec<(&'static str, SeasonScenario)> {
    vec![
        ("open_field", DEFAULT_SCENARIO),
        ("sealed_chamber", sealed_chamber_scenario()),
        ("perennial_chamber", perennial_chamber_scenario()),
        ("consumer_chamber", consumer_chamber_scenario()),
    ]
}

/// A flag: its field name, a reader and a writer. Hand-written accessors because Rust has no
/// reflection — which is exactly why the roster is gated against the struct declaration by
/// [`the_toggle_roster_is_every_bool_field_of_the_scenario`].
struct Flag {
    name: &'static str,
    get: fn(&SeasonScenario) -> bool,
    set: fn(&mut SeasonScenario, bool),
}

fn flags() -> Vec<Flag> {
    vec![
        Flag {
            name: "sealed",
            get: |s| s.sealed,
            set: |s, v| s.sealed = v,
        },
        Flag {
            name: "consumer",
            get: |s| s.consumer,
            set: |s, v| s.consumer = v,
        },
        Flag {
            name: "vernalization",
            get: |s| s.vernalization,
            set: |s, v| s.vernalization = v,
        },
        Flag {
            name: "photoperiod",
            get: |s| s.photoperiod,
            set: |s, v| s.photoperiod = v,
        },
        Flag {
            name: "stem_reserves",
            get: |s| s.stem_reserves,
            set: |s, v| s.stem_reserves = v,
        },
    ]
}

/// The flow + aux type names of one build, or `None` if the combination does not build.
///
/// ⚠ **An `Err` is read as "this combination is unreachable", and contributes nothing** —
/// which is a decision, not an accident: a scenario the assembly refuses cannot put a type
/// anywhere. The risk it carries is that *everything* errors and the subset assertions pass
/// over an empty set, so the caller counts the builds and
/// [`no_flag_setting_wires_a_type_outside_the_canonical_union`] asserts the count.
fn type_set(scenario: &SeasonScenario) -> Option<BTreeSet<&'static str>> {
    let (_, registry) = build_season(scenario).ok()?;
    Some(
        registry
            .flows()
            .iter()
            .map(|f| f.type_name())
            .chain(registry.aux_processes().iter().map(|a| a.type_name()))
            .collect(),
    )
}

/// The union the freeze manifest is derived from.
fn canonical_union() -> BTreeSet<&'static str> {
    let mut union = BTreeSet::new();
    for (label, scenario) in canonical() {
        union.extend(type_set(&scenario).unwrap_or_else(|| panic!("{label} does not build")));
    }
    assert!(!union.is_empty(), "the canonical builds wired nothing");
    union
}

/// ⚠⚠ **The gate.** No setting of any flag, on any canonical scenario, wires a flow or aux
/// type outside the frozen union.
#[test]
fn no_flag_setting_wires_a_type_outside_the_canonical_union() {
    let union = canonical_union();
    let mut built = 0usize;
    let mut unreachable: Vec<String> = Vec::new();

    for (label, scenario) in canonical() {
        for flag in flags() {
            for value in [false, true] {
                let mut variant = scenario;
                (flag.set)(&mut variant, value);
                let Some(types) = type_set(&variant) else {
                    unreachable.push(format!("{label}/{}={value}", flag.name));
                    continue;
                };
                built += 1;
                let outside: Vec<&&str> = types.difference(&union).collect();
                assert!(
                    outside.is_empty(),
                    "{label} with {}={value} wires {outside:?}, which no canonical build \
                     carries. A flag has become a switch onto a mechanism outside the frozen \
                     roster, and the manifest — being derived from the canonical builds — \
                     would follow the code silently.",
                    flag.name
                );
            }
        }
    }

    // The subset check above is satisfied by an empty type set, so the number of builds it
    // actually read is asserted rather than assumed: 4 scenarios x 5 flags x 2 values.
    assert_eq!(
        built + unreachable.len(),
        40,
        "the sweep visited {} combinations, expected 40",
        built + unreachable.len()
    );
    assert!(
        built > 0,
        "every flag combination failed to build: {unreachable:?}"
    );
}

/// The other half: the flags that gate flows really do gate flows — and `photoperiod` really
/// does not.
///
/// Gains are unioned **across** the canonical scenarios because two of the flags are
/// conditional on a third: `consumer` wires nothing unless `sealed` is also set
/// (`build_consumers` returns an empty build otherwise), so its gain is only visible on a
/// chamber. A per-scenario assertion would be a claim about wiring that this file does not
/// need to make.
#[test]
fn each_flow_gating_flag_gains_at_least_one_type() {
    for flag in flags() {
        let mut gained: BTreeSet<&'static str> = BTreeSet::new();
        for (_, scenario) in canonical() {
            let mut on = scenario;
            let mut off = scenario;
            (flag.set)(&mut on, true);
            (flag.set)(&mut off, false);
            if let (Some(on), Some(off)) = (type_set(&on), type_set(&off)) {
                gained.extend(on.difference(&off));
            }
        }
        if flag.name == "photoperiod" {
            // Measured, not assumed. `photoperiod` selects a *rate law's* behaviour inside
            // an already-wired process; it adds no flow and no aux. If this ever gains a
            // type it is a fifth flow-gating flag and §2B of the plan needs rewriting.
            assert!(
                gained.is_empty(),
                "photoperiod now gates {gained:?} — it has become a flow-gating flag"
            );
        } else {
            assert!(
                !gained.is_empty(),
                "{} gates no type on any canonical scenario — either the flag stopped \
                 wiring anything (in which case the subset gate above is inert for it) or \
                 the scenario it needs as company is gone",
                flag.name
            );
        }
    }
}

/// The roster is every `bool` field of `SeasonScenario`, read off the declaration.
///
/// Rust cannot enumerate a struct's fields at run time, so the hand-written accessors above
/// are unavoidable — but a hand list that nothing checks is the *"a census ported as a LIST
/// is the failure it prevents"* shape. A sixth flag therefore reddens this test, by name,
/// with the file to edit.
#[test]
fn the_toggle_roster_is_every_bool_field_of_the_scenario() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/biosphere/system.rs");
    let text = std::fs::read_to_string(&path).expect("system.rs is readable");
    let body = text
        .split_once("pub struct SeasonScenario {")
        .expect("system.rs no longer declares SeasonScenario")
        .1
        .split_once("\n}")
        .expect("the SeasonScenario declaration has no column-0 closing brace")
        .0;
    let declared: BTreeSet<&str> = body
        .lines()
        .filter_map(|l| l.trim().strip_prefix("pub "))
        .filter_map(|l| l.strip_suffix(": bool,"))
        .collect();
    let toggled: BTreeSet<&str> = flags().iter().map(|f| f.name).collect();
    assert_eq!(
        declared,
        toggled,
        "SeasonScenario's bool fields and this file's toggle roster disagree:\n  declared \
         but not toggled: {:?}\n  toggled but not declared: {:?}\nA new flag is a new \
         mechanism switch in production scenario config — add it to `flags()` here.",
        declared.difference(&toggled).collect::<Vec<_>>(),
        toggled.difference(&declared).collect::<Vec<_>>(),
    );
    assert_eq!(declared.len(), 5, "expected five flags, found {declared:?}");
}

/// The reader half of the accessors is used — otherwise `Flag::get` is dead weight and the
/// roster gate above is checking half a struct.
///
/// It also states the contingency the header describes as a *measurement*: the two literals
/// in `DEFAULT_SCENARIO` that make the union complete.
#[test]
fn the_defaults_that_make_the_union_complete_are_still_set() {
    let by_name = |name: &str| flags().into_iter().find(|f| f.name == name).expect(name);
    assert!(
        (by_name("vernalization").get)(&DEFAULT_SCENARIO),
        "DEFAULT_SCENARIO.vernalization is false — VernalizationAccumulation has left the \
         canonical union and the frozen manifest with it"
    );
    assert!(
        (by_name("stem_reserves").get)(&DEFAULT_SCENARIO),
        "DEFAULT_SCENARIO.stem_reserves is false — StemRemobilization has left the canonical \
         union and the frozen manifest with it"
    );
    assert!(
        (by_name("sealed").get)(&sealed_chamber_scenario()),
        "the sealed chamber is not sealed"
    );
    assert!(
        (by_name("consumer").get)(&consumer_chamber_scenario()),
        "the consumer chamber has no consumer"
    );
}
