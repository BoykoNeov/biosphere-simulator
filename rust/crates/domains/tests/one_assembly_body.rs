//! The biosphere has **one** season-assembly body, and it is `build_season_with`.
//!
//! ## What this gates, and why a run cannot see it
//!
//! Slice 1 of the science-switch plan (`docs/plans/post-roadmap-science-switch.md`) lifted a
//! second assembly body out of `system.rs`: the test-private `trace_without_flow` collected
//! its own stocks, added its own carbon loss-sinks, extended its own flow/aux vectors and
//! called its own `State::new` / `Registry::new`. It was the **control** the root-zone-capture
//! diagnostics are differenced against, and it was assembled by a different body than the
//! subject. Nothing was wrong with it on the day it was written; the failure is dated, not
//! present: the day `build_season_with` gains a loss-sink quantity or a state variable, the
//! copy does not, the control stops controlling for the run it is compared against, and
//! **every gate stays green** — a diagnostic reporting the difference between two mechanisms
//! plus one assembly divergence, with no way to tell the parts apart.
//!
//! A run cannot see this. Both bodies build a valid season; they agree today, and they will
//! go on agreeing until the day one of them is edited. What can see it is the shape of the
//! tree, which is the same argument `tests/param_funnel.rs` and
//! `tests/biosphere_spine_purity.rs` make for being source scans: the violation is a second
//! call site, and a second call site is legal Rust that passes every other test in the repo.
//!
//! ## ⚠ Why this file and not the round-trip control
//!
//! `lab::mechanism`'s empty-drop test — no drops reproduces the ordinary build — is
//! `Registry::new(into_parts(Registry::new(…)))` and holds **by construction**. It is kept
//! and labelled there; it is not evidence the lift happened. *"If one side's copy came from
//! the other, the gate is a round trip."* This file is the gate with teeth: re-fork the
//! assembly anywhere under the spine and it goes red, naming the file and line.
//!
//! ## Why the spine is the whole subject, and not an arbitrary boundary
//!
//! `fn compartments(` is **module-private** — not `pub`, not `pub(crate)` — so no code outside
//! `system.rs` can walk the compartment builds at all. A second assembly body therefore cannot
//! exist anywhere else in the workspace; the station and the authoring platform consume
//! `build_season`. Scanning one directory is not a narrowing of the claim, it is the claim's
//! full extent, and [`the_compartments_definition_is_still_excluded_for_the_right_reason`]
//! pins the privacy the argument rests on.
//!
//! ## The three steps, and why not `State::new(`
//!
//! An assembly does three things a unit test never does: walk the compartments, add the
//! boundary loss-sinks, and close a registry over them. Each must happen exactly once in the
//! spine, inside `build_season_with`.
//!
//! Not `State::new(` — it appears seventeen times under the spine: **fifteen** in unit tests,
//! building a small state to exercise one rate law, which is the ordinary way a flow is tested
//! here, and twice in production (the assembly itself and the perennial reset, which rebuilds a
//! state rather than assembling a season). A gate over it would forbid unit testing to catch a
//! second assembly — the wrong trade and the wrong subject.
//!
//! ⚠ **The registry step is spelled more than one way, and a single literal would have been a
//! name making a claim its arithmetic does not check.** `Registry::flows_only` delegates to
//! `Registry::new`, so a fork calling it would have constructed a registry with this gate
//! green. Measured before it was closed: zero calls under the spine today. So the step's
//! spellings are **derived from `simcore`'s own `impl Registry`** — every `pub fn` returning
//! `Result<Registry, …>` — and a third constructor added tomorrow is scanned for without
//! editing this file. The counts are combined across the spellings: one registry construction
//! in the spine, however it is spelled.
//!
//! ⚠ **The strictness this buys is deliberate and has a cost worth naming:** a *unit test* in
//! the spine may not build itself a small registry either. That is the right trade here and not
//! merely a tolerated side effect — a flow is unit-tested by exercising its rate law against a
//! state (which is why `State::new` is left alone), while constructing a registry is the last
//! step of an assembly and nothing else. No spine test does it today. One that needs to should
//! live in `tests/`, outside the scanned directory, rather than widening this gate.

use std::path::PathBuf;

/// `rust/crates/domains/src/biosphere`.
fn spine_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/biosphere")
}

/// The one file allowed to assemble, and the one function inside it.
const ASSEMBLY_FILE: &str = "system.rs";
const ASSEMBLY_FN: &str = "pub fn build_season_with(";

/// `rust/crates/simcore/src/registry.rs`, whose `impl Registry` supplies the registry step's
/// spellings.
fn registry_source() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../simcore/src/registry.rs")
}

/// Every public `Registry` constructor, as a call spelling — derived, never listed.
///
/// A constructor is a `pub fn` of `impl Registry` returning `Result<Registry, …>`; the
/// accessors (`flows`, `into_parts`, `len`, …) return something else and are skipped. Deriving
/// this is the difference between a gate that keeps holding and one that held on the day it was
/// written: `flows_only` was a hole in the first draft of this file, and there is no reason to
/// think a third constructor would announce itself either.
fn registry_constructors() -> Vec<String> {
    let text = std::fs::read_to_string(registry_source()).expect("simcore/src/registry.rs");
    let body = text
        .split_once("impl Registry {")
        .expect("simcore no longer declares `impl Registry`")
        .1;
    let lines: Vec<&str> = body
        .lines()
        .map_while(|l| (l != "}").then_some(l))
        .collect();
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let Some(rest) = line.trim().strip_prefix("pub fn ") else {
            continue;
        };
        let Some((name, _)) = rest.split_once('(') else {
            continue;
        };
        // ⚠ The signature is read to its opening brace, not off this one line: `rustfmt`
        // wraps `Registry::new`'s parameter list over four lines, so the return type — the
        // thing that distinguishes a constructor from an accessor — is nowhere near the `pub
        // fn`. The first draft of this function matched one line and derived an EMPTY roster,
        // which its own anti-vacuity assertion caught immediately.
        let mut signature = String::new();
        for l in lines[i..].iter().take(12) {
            signature.push_str(l);
            if l.trim_end().ends_with('{') {
                break;
            }
        }
        if signature.contains("-> Result<Registry") {
            out.push(format!("Registry::{name}("));
        }
    }
    out
}

/// The three steps of an assembly, each with the spellings that count as taking it.
fn steps() -> Vec<(&'static str, Vec<String>)> {
    vec![
        (
            "walk the compartment builds",
            vec!["compartments(".to_string()],
        ),
        (
            "add the boundary loss-sinks",
            vec!["boundary::loss_sinks(".to_string()],
        ),
        ("close a registry over them", registry_constructors()),
    ]
}

/// Everything from the first `//` onward — doc comments (`///`, `//!`) start with `//` too,
/// and this file's subject is discussed in prose in several spine headers, including the
/// lifted helper's own docstring which names `Registry::new` to say why it no longer calls
/// it.
fn strip_line_comment(line: &str) -> &str {
    match line.find("//") {
        Some(at) => &line[..at],
        None => line,
    }
}

/// Every `.rs` file in the spine, `(file name, source)`, sorted.
///
/// ⚠ **`#[cfg(test)]` is deliberately NOT skipped here**, unlike `param_funnel.rs`. The body
/// this gate exists to keep singular *was* test code; a scan that skipped test code would
/// have been green throughout the defect it was written for.
fn spine_files() -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = std::fs::read_dir(spine_dir())
        .expect("the biosphere source directory is readable")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|p| p.extension().is_some_and(|e| e == "rs"))
        .map(|p| {
            let name = p
                .file_name()
                .expect("file name")
                .to_string_lossy()
                .into_owned();
            let text = std::fs::read_to_string(&p).expect("source is readable");
            (name, text)
        })
        .collect();
    out.sort();
    out
}

/// `(file, line number, code)` for every line of the spine whose code contains `needle`.
///
/// `fn compartments(` is the definition, not a call, and is excluded by name rather than by
/// a cleverer match: the exclusion is one string and it is asserted to be real by
/// [`the_compartments_definition_is_still_excluded_for_the_right_reason`].
fn hits(spellings: &[String]) -> Vec<(String, usize, String)> {
    let mut found = Vec::new();
    for (name, text) in spine_files() {
        for (i, line) in text.lines().enumerate() {
            let code = strip_line_comment(line);
            if spellings.iter().any(|n| code.contains(n.as_str()))
                && !code.contains("fn compartments(")
            {
                found.push((name.clone(), i + 1, code.trim().to_string()));
            }
        }
    }
    found
}

/// The inclusive line range of `build_season_with`'s body in `system.rs`.
///
/// The tree is `rustfmt`-formatted, which is what makes this reliable without a parser (the
/// same property `param_funnel.rs` relies on): a column-0 `pub fn …(` opens an item whose
/// closing brace is a bare `}` at column 0.
fn assembly_fn_range() -> (usize, usize) {
    let text = std::fs::read_to_string(spine_dir().join(ASSEMBLY_FILE)).expect("system.rs");
    let lines: Vec<&str> = text.lines().collect();
    let start = lines
        .iter()
        .position(|l| l.starts_with(ASSEMBLY_FN))
        .unwrap_or_else(|| {
            panic!("{ASSEMBLY_FILE} no longer declares `{ASSEMBLY_FN}` at column 0")
        });
    let end = lines[start..]
        .iter()
        .position(|l| *l == "}")
        .expect("build_season_with has no column-0 closing brace")
        + start;
    (start + 1, end + 1)
}

/// The scan read something. A moved directory or a changed extension would otherwise let
/// every assertion below pass over an empty set.
#[test]
fn the_scan_is_not_vacuous() {
    let names: Vec<String> = spine_files().into_iter().map(|(n, _)| n).collect();
    assert!(!names.is_empty(), "no biosphere sources discovered");
    for anchor in ["flows.rs", "system.rs", "params.rs", "mod.rs"] {
        assert!(
            names.contains(&anchor.to_string()),
            "{anchor} missing from {names:?}"
        );
    }
    let (start, end) = assembly_fn_range();
    assert!(
        end > start + 5,
        "build_season_with's body scanned as {} lines — the brace matching is wrong",
        end - start
    );
}

/// ⚠⚠ **The gate.** Each assembly step happens exactly once in the spine, inside
/// `build_season_with`.
///
/// A second body — a test helper that re-collects the stocks, a diagnostic that rebuilds the
/// registry, a second entry point — lands here by file and line. The fix is never to widen
/// this list: it is to compose onto the one build, which is what
/// `lab::mechanism::build_season_without` exists to make easy.
#[test]
fn the_spine_assembles_a_season_in_exactly_one_place() {
    let (start, end) = assembly_fn_range();
    for (step, spellings) in steps() {
        assert!(
            !spellings.is_empty(),
            "the `{step}` step has no spellings to scan for — the derivation returned nothing"
        );
        let found = hits(&spellings);
        assert_eq!(
            found.len(),
            1,
            "the spine takes the `{step}` step {} times, expected 1 (spelled {spellings:?}):\n  \
             {}\nA second assembly body is the defect slice 1 of the science-switch plan \
             removed — compose onto `build_season_with` (see \
             `lab::mechanism::build_season_without`) rather than re-assembling.",
            found.len(),
            found
                .iter()
                .map(|(f, n, c)| format!("{f}:{n}  {c}"))
                .collect::<Vec<_>>()
                .join("\n  ")
        );
        let (file, line, code) = &found[0];
        assert_eq!(
            file, ASSEMBLY_FILE,
            "the `{step}` step moved to {file}:{line}"
        );
        assert!(
            (start..=end).contains(line),
            "the `{step}` step is taken at {file}:{line} ({code}), outside build_season_with's \
             body ({start}..={end})"
        );
    }
}

/// The registry step's spellings really are derived, and really do include the delegating
/// constructor that was the first draft's hole.
///
/// ⚠ This is the anti-vacuity half of the derivation: a `registry_constructors` that silently
/// returned only `new` — a changed `impl` header, a reformatted signature — would restore the
/// hole and leave every assertion above green.
#[test]
fn the_registry_spellings_are_derived_and_include_the_delegating_constructor() {
    let spellings = registry_constructors();
    assert!(
        spellings.contains(&"Registry::new(".to_string()),
        "the derivation lost the primary constructor: {spellings:?}"
    );
    assert!(
        spellings.contains(&"Registry::flows_only(".to_string()),
        "`flows_only` is no longer derived as a constructor: {spellings:?}. It delegates to \
         `Registry::new`, so dropping it from the scan reopens the hole this test exists for."
    );
    // The accessors must NOT be in it, or the scan would flag every `registry.flows()` read.
    for accessor in [
        "Registry::flows(",
        "Registry::into_parts(",
        "Registry::len(",
    ] {
        assert!(
            !spellings.contains(&accessor.to_string()),
            "{accessor} was derived as a constructor: {spellings:?}"
        );
    }
}

/// The `fn compartments(` exclusion is an assertion, not a line in a list — and it carries the
/// **privacy** the header's scope argument rests on.
///
/// It exists because the definition contains the call needle. If the function were renamed
/// or removed, the exclusion would silently become a hole that hides a real second call, so
/// the premise is checked rather than trusted — the shape `param_funnel.rs`'s
/// `the_gate_body_exclusion_is_real` established.
///
/// ⚠ The column-0 `starts_with("fn compartments(")` assertion below is doing double duty: it
/// is also what makes *"a second assembly body cannot exist outside `system.rs`"* true rather
/// than assumed. A `pub fn` or `pub(crate) fn` there would fail it, and would mean this file's
/// one-directory scan had quietly become a partial claim about the workspace.
#[test]
fn the_compartments_definition_is_still_excluded_for_the_right_reason() {
    let text = std::fs::read_to_string(spine_dir().join(ASSEMBLY_FILE)).expect("system.rs");
    let definitions: Vec<&str> = text
        .lines()
        .filter(|l| strip_line_comment(l).contains("fn compartments("))
        .collect();
    assert_eq!(
        definitions.len(),
        1,
        "expected exactly one `fn compartments(` declaration, found {definitions:?} — the \
         exclusion in `hits` now covers more than the definition it was written for"
    );
    assert!(
        definitions[0].starts_with("fn compartments("),
        "the compartments declaration is no longer a column-0 item: {:?}",
        definitions[0]
    );
}

/// The lifted helper really did move onto the seam, named rather than inferred.
///
/// The gate above would also pass if `trace_without_flow` had simply been **deleted**, which
/// would take the two root-zone-capture diagnostics with it. This pins the other half: the
/// helper is still there and it goes through the lab.
#[test]
fn the_knockout_helper_goes_through_the_lab_seam() {
    let text = std::fs::read_to_string(spine_dir().join(ASSEMBLY_FILE)).expect("system.rs");
    assert!(
        text.contains("fn trace_without_flow("),
        "the knockout helper is gone — the root-zone-capture diagnostics lost their control"
    );
    assert!(
        text.contains("lab::mechanism::build_season_without("),
        "trace_without_flow no longer calls the lab seam — it has been re-forked"
    );
}
