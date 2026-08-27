//! The biosphere has **one** production param load, and it is the value-switch seam.
//!
//! ## What this gates and why it is not the obvious test
//!
//! `system::build_season_with` takes a `&BiosphereParams`, so a harness can run a season
//! with one coefficient substituted (`docs/plans/post-roadmap-value-switch-harness.md`).
//! The failure that plan names in §7 is a substitution that **silently does nothing** — a
//! probe shimming a path the run never reaches, reporting "no effect" as a finding. This
//! tree has shipped that defect before (`cc44b41`).
//!
//! ⚠ **The obvious guard against it — "assert an override changes the output" — is inert by
//! construction here.** With a single funnel and `&BiosphereParams` threaded all the way
//! down by `compartments`, an override *cannot* fail to apply; a test asserting that it does
//! would pass for a reason that has nothing to do with what it claims to check. This repo has
//! shipped several of those and each one had to be found by mutation rather than by reading.
//!
//! So the gate is the **forward** one: the property that can actually rot as the tree grows
//! is the funnel being singular. A flow that reaches for `params::canopy()` at step time
//! instead of taking it from the threaded struct would compile, pass every golden (the frozen
//! values are the same either way) and quietly escape every override. Nothing else in the
//! tree can see that.
//!
//! ## Why a source scan
//!
//! Same reasoning as `biosphere_spine_purity.rs`, whose header states it for the config
//! boundary: the dependency graph cannot see this, because the call is *within* one crate and
//! `params` is a legitimate sibling module. A source scan is not redundant; it is the only
//! instrument that can see the violation.
//!
//! ## The roster is derived, never listed
//!
//! The loader names come from `params.rs` itself (`pub fn <name>() -> …`), so a loader added
//! tomorrow is scanned for without editing this file. *Derive from the tree, never hand-list*
//! is the rule the science-gate census exists to honour and it applies here too.

use std::collections::BTreeSet;
use std::path::PathBuf;

/// `rust/crates/domains/src/biosphere`.
fn spine_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/biosphere")
}

/// The one production site: `build_season` handing the frozen params to the seam.
const BLESSED_FILE: &str = "system.rs";
const BLESSED_TEXT: &str = "build_season_with(scenario, &params::biosphere())";

/// The census macro's invocation — its body is `#[cfg(test)]` only after expansion.
const GATE_MACRO_INVOCATION: &str = "science_gates! {";

// --------------------------------------------------------------------------- //
// Region model: what counts as production source                               //
// --------------------------------------------------------------------------- //

/// Drop every `#[cfg(test)]` item, returning `(line_number, code)` for production lines.
///
/// The tree is `rustfmt`-formatted, which is what makes this reliable without a parser: a
/// column-0 `#[cfg(test)]` attaches to a column-0 item, and a column-0 item's closing brace
/// is a bare `}` at column 0. Two shapes, both present in the spine:
///
/// * `#[cfg(test)]` + `mod x {` → skip to the next line that is exactly `}`;
/// * `#[cfg(test)]` + a single-line item (`use …;`, `const … ;`, `pub mod x;`) → skip to the
///   first line ending in `;`.
///
/// ⚠ **Indented `#[cfg(test)]` is deliberately not handled, and that is correct rather than a
/// simplification**: an indented one is inside an item this scan is already inside, and
/// `science_gates.rs` has one *inside a `macro_rules!` body*, where it is a template rather
/// than an item at all. [`the_cfg_test_skipping_is_not_over_eager`] pins that the spine's
/// eight column-0 markers in `science_gates.rs` do not swallow the production code between
/// them.
fn production_lines(source: &str) -> Vec<(usize, String)> {
    let lines: Vec<&str> = source.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        // ⚠ The census macro's invocation is test code that carries no `#[cfg(test)]` of its
        // own — the attribute is inside the macro's *definition*, on the `mod gate_tests` it
        // emits. Found by this gate on its first run, which is the whole argument for the
        // exclusion being an assertion rather than a line in a list:
        // `the_gate_body_exclusion_is_real` pins that the expansion really is `#[cfg(test)]`,
        // so a macro that started emitting production code turns this exclusion red instead
        // of turning it into a hole.
        if lines[i].starts_with(GATE_MACRO_INVOCATION) {
            let mut k = i + 1;
            while k < lines.len() && lines[k] != "}" {
                k += 1;
            }
            i = k + 1;
            continue;
        }
        if lines[i] == "#[cfg(test)]" {
            // Find the item this attribute introduces, skipping any further attributes and
            // the doc comments rustfmt allows between them.
            let mut j = i + 1;
            while j < lines.len()
                && (lines[j].starts_with("#[") || lines[j].starts_with("///"))
                && !lines[j].starts_with("#[cfg(test)]")
            {
                j += 1;
            }
            if j >= lines.len() {
                break;
            }
            if lines[j].ends_with('{') {
                // A block item: skip to its column-0 closing brace.
                let mut k = j + 1;
                while k < lines.len() && lines[k] != "}" {
                    k += 1;
                }
                i = k + 1;
            } else {
                // A single-line (or `;`-terminated) item.
                let mut k = j;
                while k < lines.len() && !lines[k].ends_with(';') {
                    k += 1;
                }
                i = k + 1;
            }
            continue;
        }
        out.push((i + 1, strip_line_comment(lines[i]).to_string()));
        i += 1;
    }
    out
}

/// Everything from the first `//` onward. Doc comments (`///`, `//!`) start with `//` too.
fn strip_line_comment(line: &str) -> &str {
    match line.find("//") {
        Some(at) => &line[..at],
        None => line,
    }
}

/// Every `.rs` file in the spine, `(file name, source)`, sorted.
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

// --------------------------------------------------------------------------- //
// The roster, derived from params.rs                                           //
// --------------------------------------------------------------------------- //

/// The zero-argument loaders — `pub fn <name>() -> …` in `params.rs`'s production half.
///
/// The `_from(text, name)` variants are deliberately **not** in the roster: those are the
/// seam's own entry point, and calling one is how a substitution is built.
fn loader_names() -> BTreeSet<String> {
    let text = std::fs::read_to_string(spine_dir().join("params.rs")).expect("params.rs");
    production_lines(&text)
        .into_iter()
        .filter_map(|(_, code)| {
            let rest = code.trim().strip_prefix("pub fn ")?;
            let (name, tail) = rest.split_once('(')?;
            tail.starts_with(") ->").then(|| name.to_string())
        })
        .collect()
}

/// Every `params::<loader>(` occurrence in a file's production source.
fn loader_calls(source: &str, roster: &BTreeSet<String>) -> Vec<(usize, String)> {
    production_lines(source)
        .into_iter()
        .filter(|(_, code)| {
            roster
                .iter()
                .any(|name| code.contains(&format!("params::{name}(")))
        })
        .collect()
}

// --------------------------------------------------------------------------- //
// Anti-vacuity                                                                 //
// --------------------------------------------------------------------------- //

#[test]
fn the_scan_is_not_vacuous() {
    let names: BTreeSet<String> = spine_files().into_iter().map(|(n, _)| n).collect();
    assert!(!names.is_empty(), "no biosphere sources discovered");
    for anchor in ["flows.rs", "system.rs", "params.rs", "mod.rs"] {
        assert!(names.contains(anchor), "{anchor} missing from {names:?}");
    }
}

#[test]
fn the_loader_roster_is_derived_and_populated() {
    let roster = loader_names();
    // Anchored on names that must exist rather than pinned to a count: loaders arrive with
    // ordinary science work, and an equality here would be churn rather than a decision.
    for anchor in ["biosphere", "canopy", "photosynthesis", "allocation"] {
        assert!(roster.contains(anchor), "{anchor} missing from {roster:?}");
    }
    assert!(
        roster.len() >= 17,
        "the roster collapsed to {} entries: {roster:?}",
        roster.len()
    );
    // The seam's own entry points must NOT be in it — they take arguments.
    assert!(!roster.contains("canopy_from"), "{roster:?}");
}

/// ⚠ The control that makes the scan's silence meaningful, and it uses the **real tree**
/// rather than a synthetic string: `flows.rs` genuinely contains `params::canopy()` — in its
/// test module — so a scan that reported nothing for it would be reporting on nothing.
#[test]
fn the_cfg_test_skipping_is_not_over_eager() {
    let roster = loader_names();
    let files: Vec<(String, String)> = spine_files();
    let flows = &files
        .iter()
        .find(|(n, _)| n == "flows.rs")
        .expect("flows.rs")
        .1;
    assert!(
        flows.contains("params::canopy()"),
        "the control's premise is gone: flows.rs no longer calls a loader anywhere"
    );
    assert!(
        loader_calls(flows, &roster).is_empty(),
        "flows.rs's loader calls are all test-only; the scan saw one as production"
    );

    // The same file's production half must still be READ, or the emptiness above is vacuous.
    let production = production_lines(flows);
    assert!(
        production.iter().any(|(_, c)| c.contains("impl Flow for")),
        "flows.rs's production half was swallowed by the cfg(test) skip"
    );

    // `science_gates.rs` is the hard case: eight column-0 `#[cfg(test)]` markers with
    // production items between them.
    let gates = &files
        .iter()
        .find(|(n, _)| n == "science_gates.rs")
        .expect("science_gates.rs")
        .1;
    let gate_production = production_lines(gates);
    assert!(
        gate_production
            .iter()
            .any(|(_, c)| c.contains("pub const GATE_SOURCE_FILE")),
        "science_gates.rs's production consts were swallowed by the cfg(test) skip"
    );
    assert!(
        gate_production
            .iter()
            .any(|(_, c)| c.contains("macro_rules! science_gates")),
        "the census macro was swallowed by the cfg(test) skip"
    );
}

/// The `science_gates! { … }` exclusion is an assertion, not a line in a list.
///
/// The invocation's body is skipped because the macro emits it into `#[cfg(test)] mod
/// gate_tests`. If that ever stops being true — the macro grows a production arm, the
/// `#[cfg(test)]` is dropped — the exclusion silently becomes a hole big enough to hide every
/// gate body. So the premise is checked here rather than trusted.
#[test]
fn the_gate_body_exclusion_is_real() {
    let text = std::fs::read_to_string(spine_dir().join("science_gates.rs")).expect("readable");
    let (definition, _) = text
        .split_once(GATE_MACRO_INVOCATION)
        .expect("the census macro is still invoked in this file");
    let emitted = definition
        .split_once("macro_rules! science_gates")
        .expect("the census macro is still defined in this file")
        .1;
    let at = emitted
        .find("mod gate_tests")
        .expect("the macro still emits a gate_tests module");
    assert!(
        emitted[..at].trim_end().ends_with("#[cfg(test)]"),
        "the census macro no longer emits its gate bodies under #[cfg(test)] — the exclusion \
         in `production_lines` is now a hole"
    );
    // And the exclusion must actually be doing work, or it is decoration.
    let (_, invocation) = text
        .split_once(GATE_MACRO_INVOCATION)
        .expect("invocation present");
    assert!(
        invocation.contains("params::senescence()"),
        "no gate body calls a loader any more — this exclusion's subject is gone"
    );
}

#[test]
fn a_comment_is_not_a_call() {
    let roster = loader_names();
    let source = "fn f() {\n    // params::canopy() is what this used to do\n}\n";
    assert!(loader_calls(source, &roster).is_empty());
    let real = "fn f() {\n    let c = params::canopy();\n}\n";
    assert_eq!(loader_calls(real, &roster).len(), 1);
}

// --------------------------------------------------------------------------- //
// The gate                                                                     //
// --------------------------------------------------------------------------- //

/// **The funnel is one.** Every production param load in the biosphere spine is
/// `build_season`'s hand-off to `build_season_with`; everything else takes its params from
/// the threaded `&BiosphereParams`.
///
/// If this goes red, the new call site is not necessarily wrong — but it is outside the
/// value-switch seam, so a substituted run would not reach it. Either thread the params to
/// it, or bless it here **and** say in the harness's output that it is unreachable by an
/// override. Silently widening the roster is the one response that defeats the gate.
#[test]
fn the_biosphere_has_exactly_one_production_param_load() {
    let roster = loader_names();
    let mut found: Vec<(String, usize, String)> = Vec::new();
    for (name, source) in spine_files() {
        if name == "params.rs" {
            continue; // the loaders' own file: `biosphere()` is built from the other 17.
        }
        for (line, code) in loader_calls(&source, &roster) {
            found.push((name.clone(), line, code.trim().to_string()));
        }
    }
    assert_eq!(
        found.len(),
        1,
        "expected exactly one production param load, found {}: {found:#?}",
        found.len()
    );
    let (file, _, code) = &found[0];
    assert_eq!(file, BLESSED_FILE, "the load moved file: {found:#?}");
    assert!(
        code.contains(BLESSED_TEXT),
        "the one load is not the seam's hand-off: {code}"
    );
}
