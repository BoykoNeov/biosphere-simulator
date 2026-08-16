//! `Flow::type_name` / `AuxProcess::type_name` — the **type-level** identity axis added
//! by slice 2 of the reference flip (`docs/plans/post-roadmap-reference-flip.md`).
//!
//! Exercised **through `Box<dyn Flow>` out of a built canonical registry**, which is the
//! only path that matters: it is how the freeze manifests' `flow_set` / `aux_set` are
//! derived on the Python side (`type(flow).__name__` over `registry.flows`), and a method
//! only ever called on a concrete type would prove nothing about that path.
//!
//! ⚠ **Deliberately property-based, with no roster of names here.** Asserting the 23
//! flow / 3 aux names against a hard-coded list would put a second copy of the frozen
//! manifest in the tree, and this repo has already paid for a rule whose two copies
//! disagreed. The authoritative comparison of Rust's inventory against the manifest is
//! **slice 3's**, in Python, against the manifest file itself. What lives here is what
//! slice 3 cannot check: that the values are well-formed, are a function of the *type*
//! rather than the instance, and do not collapse.

use domains::biosphere::system::{
    build_season, consumer_chamber_scenario, perennial_chamber_scenario, sealed_chamber_scenario,
    SeasonScenario, DEFAULT_SCENARIO,
};
use std::collections::BTreeSet;

/// The four canonical builds — the same union Python's `_flow_set()` takes
/// (`tests/test_freeze_manifest.py::_canonical_registries`): the open field carries the
/// producer flows, the chambers add the decomposer / water-cycle / consumer ones.
fn canonical() -> Vec<(&'static str, SeasonScenario)> {
    vec![
        ("open_field", DEFAULT_SCENARIO),
        ("sealed_chamber", sealed_chamber_scenario()),
        ("perennial_chamber", perennial_chamber_scenario()),
        ("consumer_chamber", consumer_chamber_scenario()),
    ]
}

/// A bare Python-style class name: `[A-Z][A-Za-z0-9]*`, ASCII throughout.
fn is_class_name(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_uppercase())
        && chars.all(|c| c.is_ascii_alphanumeric())
}

/// Collect `(type_name, id)` for every flow and aux process of one canonical build.
fn identities(scenario: &SeasonScenario) -> Vec<(&'static str, String)> {
    let (_, registry) = build_season(scenario).expect("canonical build");
    registry
        .flows()
        .iter()
        .map(|f| (f.type_name(), f.id().to_string()))
        .chain(
            registry
                .aux_processes()
                .iter()
                .map(|a| (a.type_name(), a.id().to_string())),
        )
        .collect()
}

/// Every reported name is a bare class name — never a path, never generic-parameterised.
///
/// This is the failure the *deliberate non-use* of [`std::any::type_name`] avoids, and
/// the only reason that decision is checkable at all: that function is documented as
/// having no guaranteed output format, so it can legally return
/// `domains::biosphere::flows::Senescence` or `Foo<Bar>` — strings that can never equal a
/// Python class name, on a value the freeze manifest will be anchored to from slice 6.
#[test]
fn every_canonical_type_name_is_a_bare_class_name() {
    for (label, scenario) in canonical() {
        for (type_name, id) in identities(&scenario) {
            assert!(
                is_class_name(type_name),
                "{label}: {id} reports type_name {type_name:?}, which is not a bare \
                 ASCII class name"
            );
        }
    }
}

/// `type_name` and `id` are **different axes**, not two spellings of one thing.
///
/// §2f of the plan: Python freezes `type(flow).__name__` — a **class** — while Rust's
/// `id()` is an **instance** identifier. If the two ever coincided, slice 3's comparison
/// would silently be measuring the id set against a class-name manifest and passing for
/// the wrong reason.
#[test]
fn type_name_is_not_the_instance_id() {
    for (label, scenario) in canonical() {
        for (type_name, id) in identities(&scenario) {
            assert_ne!(
                type_name, id,
                "{label}: type_name and id coincide — the class/instance distinction \
                 slice 3 relies on has been lost"
            );
        }
    }
}

/// No two distinct flows in one canonical build report the same `type_name`.
///
/// The collapse detector, and the reason no roster is hard-coded above: a copy-pasted
/// literal (`Senescence` returning `"Transpiration"`) shows up here as a shortfall in the
/// distinct count, derived entirely from the tree.
///
/// ⚠ **Assumption, true today and asserted rather than assumed:** no canonical biosphere
/// build instantiates one flow type twice — each is a single compartment, measured at
/// 11 / 19 / 19 / 22 flows with an equal number of distinct types. A build that
/// *legitimately* wires a second compartment turns this red; the fix is then to scope the
/// assertion to the new wiring, never to delete it. Note the union across all four is
/// deliberately **not** counted here — that cardinality is the manifest's, and slice 3's.
#[test]
fn distinct_flows_report_distinct_type_names() {
    for (label, scenario) in canonical() {
        let identities = identities(&scenario);
        let distinct: BTreeSet<&str> = identities.iter().map(|(t, _)| *t).collect();
        assert_eq!(
            distinct.len(),
            identities.len(),
            "{label}: {} wired flows/aux processes collapse onto {} distinct type names",
            identities.len(),
            distinct.len()
        );
    }
}
