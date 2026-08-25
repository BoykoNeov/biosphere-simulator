//! The biosphere **spine** is free of the config boundary — the Rust port of
//! `tests/test_biosphere_purity.py` (Stage-3 slice S4).
//!
//! ## Why this is a separate file from `simcore/tests/workspace_purity.rs`
//!
//! That file gates the **dependency graph**, and for `gdext` the graph is the whole story:
//! a crate cannot name a type it has not declared a dependency on, so forbidding the
//! manifest edge forbids the type. **The same reasoning does not carry here, and assuming it
//! did would have shipped a gate that asserts nothing.** The biosphere lives *inside*
//! `domains`, and `domains -> config` is a legitimate, necessary edge — the param loader
//! needs it. So every module in `domains/src/biosphere/` may `use config::…` and the
//! manifest gate stays green. A source scan is not redundant here; it is the only thing
//! that can see the violation.
//!
//! ## The subject, ported
//!
//! The biosphere is split: the flows / rates / scenario assembly stay stdlib-pure so the
//! simulation runs headless, while the **loader** is the sole config boundary. The Python
//! gate excluded exactly one file, `loader.py`, and pinned that the exclusion was real.
//!
//! ⚠ **In the Rust tree the boundary is two modules, not one**, and that is a genuine
//! difference rather than a looser port: [`params.rs`] is `loader.py`'s counterpart, and
//! [`weather.rs`] became a second boundary when slice C9 moved the raw-weather path
//! (`config::json` + the ISO-date calendar) into the reference. Both exclusions carry the
//! Python original's paired assertion — that the excluded file genuinely *does* reach the
//! boundary — so an exclusion cannot quietly become a typo that hides a leak.
//!
//! ## What this file does not claim
//!
//! The four sibling domains (`power.rs`, `thermal.rs`, `eclss.rs`, `crew.rs`) and
//! `domains/src/params.rs` are **not** scanned. The Python original's subject was the
//! biosphere spine alone, and inventing a wider claim here would be a new gate wearing a
//! port's name.

use std::collections::BTreeSet;
use std::path::PathBuf;

/// `rust/crates/domains/src/biosphere`.
fn spine_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/biosphere")
}

/// The modules allowed to reach the config boundary. See the header: two, not one, and
/// each is pinned below.
const BOUNDARY: [&str; 2] = ["params.rs", "weather.rs"];

/// Does `source` reference the `config` crate in **code** (not in prose)?
///
/// Line comments are stripped first, then `config` is matched as a whole token — the char
/// before and after must not continue an identifier. Both halves earn their place:
///
/// * Without comment stripping the scan would flag `weather.rs`'s own header, and several
///   modules discuss the boundary in prose.
/// * Without token matching it would flag `flows.rs`, the largest file in the spine, for
///   the phrase *"when drought acceleration is not **configur**ed"* — a substring hit in a
///   doc comment, in a file that touches the boundary nowhere.
///
/// A bare `config` token in Rust source can only be the crate: `use config::…`,
/// `config::provenance::…`, or `extern crate config`.
fn references_config(source: &str) -> bool {
    source.lines().any(|line| {
        let code = strip_line_comment(line);
        let bytes = code.as_bytes();
        code.match_indices("config").any(|(at, _)| {
            let before_ok = at == 0 || !is_ident_byte(bytes[at - 1]);
            let after = at + "config".len();
            let after_ok = after >= bytes.len() || !is_ident_byte(bytes[after]);
            before_ok && after_ok
        })
    })
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Everything from the first `//` onward. Rust's doc comments (`///`, `//!`) start with
/// `//` too, so one rule covers all three.
fn strip_line_comment(line: &str) -> &str {
    match line.find("//") {
        Some(at) => &line[..at],
        None => line,
    }
}

/// Every `.rs` file in the spine directory, by file name.
fn spine_files() -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = std::fs::read_dir(spine_dir())
        .expect("the biosphere source directory is readable")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|p| p.extension().is_some_and(|e| e == "rs"))
        .map(|p| {
            let name = p.file_name().expect("file name").to_string_lossy().into_owned();
            let text = std::fs::read_to_string(&p).expect("source is readable");
            (name, text)
        })
        .collect();
    out.sort();
    out
}

// --------------------------------------------------------------------------- //
// The scan                                                                     //
// --------------------------------------------------------------------------- //

/// The anti-vacuity guard. If discovery globbed nothing — a moved directory, a changed
/// extension — the scan below would pass over an empty set and report a pure spine that it
/// never read.
///
/// Anchored on names that must exist rather than pinned to the exact roster, unlike the
/// workspace-member check in `simcore/tests/workspace_purity.rs`: crates are added once a
/// phase and modules are added by ordinary science work, so an equality here would be
/// churn rather than a decision point.
#[test]
fn the_spine_scan_is_not_vacuous() {
    let names: BTreeSet<String> = spine_files().into_iter().map(|(n, _)| n).collect();
    assert!(!names.is_empty(), "no biosphere sources discovered — the scan is vacuous");
    for anchor in ["flows.rs", "system.rs", "stocks.rs", "mod.rs"] {
        assert!(names.contains(anchor), "{anchor} is missing from {names:?}");
    }
    assert!(
        names.len() >= 8,
        "only {} spine files found; the glob is probably wrong: {names:?}",
        names.len()
    );
}

/// The gate itself: every spine module outside the two boundary files is free of `config`.
#[test]
fn every_spine_module_outside_the_boundary_is_free_of_config() {
    for (name, source) in spine_files() {
        if BOUNDARY.contains(&name.as_str()) {
            continue;
        }
        assert!(
            !references_config(&source),
            "{name} reaches the config boundary; the biosphere spine stays stdlib-pure so it \
             runs headless, and config belongs to {BOUNDARY:?} alone"
        );
    }
}

/// The exclusions are real, not typos: each boundary file exists **and** genuinely reaches
/// `config`, so the scan would have flagged it. The Python original's
/// `test_loader_is_the_excluded_boundary`, one per boundary module.
#[test]
fn each_excluded_module_really_is_a_config_boundary() {
    let files = spine_files();
    for boundary in BOUNDARY {
        let (_, source) = files
            .iter()
            .find(|(n, _)| n == boundary)
            .unwrap_or_else(|| panic!("the excluded module {boundary} does not exist"));
        assert!(
            references_config(source),
            "{boundary} is excluded from the scan but does not reference config — the \
             exclusion is hiding nothing and should be removed"
        );
    }
}

// --------------------------------------------------------------------------- //
// Discrimination: the detector catches what it claims to                       //
// --------------------------------------------------------------------------- //

#[test]
fn the_detector_flags_a_use_statement() {
    assert!(references_config("use config::ParamFile;\n"));
}

#[test]
fn the_detector_flags_a_qualified_call() {
    assert!(references_config(
        "        let ok = config::provenance::sha256_hex(text);\n"
    ));
}

#[test]
fn the_detector_flags_an_extern_crate() {
    assert!(references_config("extern crate config;\n"));
}

#[test]
fn the_detector_ignores_config_in_a_line_comment() {
    assert!(!references_config("let x = 1; // reads config::json here\n"));
}

#[test]
fn the_detector_ignores_config_in_a_doc_comment() {
    assert!(!references_config(
        "//! the boundary is `config` — see config::date\n/// uses config::ParamFile\n"
    ));
}

/// The substring trap, and it is not hypothetical: `flows.rs` carries the phrase *"not
/// configured"*, so a `contains(\"config\")` scan would fail on the largest module in the
/// spine and the natural fix — excluding `flows.rs` — would gut the gate.
#[test]
fn the_detector_ignores_words_that_merely_start_with_config() {
    assert!(!references_config(
        "    /// The multiplier is 1 when drought acceleration is not configured.\n"
    ));
    assert!(!references_config("let configured = true;\nlet reconfigure = 1;\n"));
}

#[test]
fn the_detector_passes_a_pure_module() {
    assert!(!references_config(
        "use simcore::flow::Flow;\nuse crate::biosphere::stocks::Stocks;\nuse std::collections::BTreeMap;\n"
    ));
}
