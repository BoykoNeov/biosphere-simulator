//! **Every alternative mechanism is reachable only through the lab.** `build_season` can
//! produce the frozen flow set and nothing else.
//!
//! §6 of the science-switch plan (`docs/plans/post-roadmap-science-switch.md`). The failure it
//! prevents is a golden silently running a non-frozen mechanism set — the science equivalent
//! of a calibration wearing an experiment's name.
//!
//! ## Why a source scan, and why the run-inventory check is deliberately NOT here
//!
//! The obvious check is "build the four canonical scenarios and assert no lab type appears in
//! the inventory". It was measured against the gates that already exist and **left out**:
//! `freeze_manifest.rs`'s `inventory()` walks exactly those four builds and unions their
//! `type_name`s, and `tests/manifest_writer.rs` compares the written manifest to the committed
//! one byte for byte. So a lab type reaching a canonical build already reddens that gate. A
//! second check whose only mutation is a mutation something else catches is decoration, and
//! this repo's own record is that *a redundant guard has no mutation that reddens*.
//!
//! Two failures survive that reasoning, and they are what this file gates:
//!
//! 1. **a lab type constructed in spine code on a path no canonical scenario reaches** — a
//!    branch behind a flag no frozen scenario sets, a helper nothing calls yet. Invisible to
//!    every run and to every manifest; visible only to a scan of the tree. It is the same
//!    argument `one_assembly_body.rs` and `param_funnel.rs` make for being source scans;
//! 2. **a lab type wired in *and* the manifest regenerated.** The manifest is *derived* — it
//!    follows the code silently, which is the auto-follow hazard `locked_dt_days` is
//!    hand-written to avoid. Regenerate after wiring an alternative mechanism into a canonical
//!    build and `manifest_writer.rs` goes green again over a changed frozen roster. So the
//!    committed manifests are read here as **text** and asserted not to name a lab type.
//!
//! ## The roster is derived from the `type_name` LITERAL, not from the struct name
//!
//! Those are two different axes and the gate needs both. `Flow::type_name` is hand-written on
//! purpose (its own docstring: *"deliberately not defaulted from `std::any::type_name`"*),
//! because it is the string a freeze contract is anchored to. So an
//! `impl Flow for AltPhotosynthesis` returning `"CanopyAssimilation"` — a copy-paste, or a
//! deliberate disguise — would walk past a struct-name-derived roster wearing a frozen name.
//! Both spellings are collected, both are scanned for, and
//! [`every_lab_flow_type_reports_its_own_name`] asserts they agree, because a disagreement is
//! itself the finding.

use std::path::PathBuf;

/// `rust/crates/domains/src/lab`.
fn lab_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/lab")
}

/// `rust/crates/domains/src/biosphere` — the spine, the same subject `one_assembly_body.rs`
/// scans and for the same reason: `build_season_with` is the only assembly, and `compartments`
/// is module-private, so a flow can only enter a canonical build from this directory.
fn spine_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/biosphere")
}

/// The two committed manifests whose `flow_set` a biosphere flow can reach.
const BIOSPHERE_MANIFEST: &str = include_str!("../../../../docs/biosphere-reference.manifest.json");
const STATION_MANIFEST: &str = include_str!("../../../../docs/station-reference.manifest.json");

/// Every `.rs` file in `dir`, `(file name, source)`, sorted.
fn sources(dir: PathBuf) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{} is readable: {e}", dir.display()))
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

/// Everything from the first `//` onward. Doc comments start with `//` too, and this file's
/// subject is discussed in prose in the lab's own headers.
fn strip_line_comment(line: &str) -> &str {
    match line.find("//") {
        Some(at) => &line[..at],
        None => line,
    }
}

/// One lab flow type: the implementing struct's name, and the `type_name` it reports.
#[derive(Debug, PartialEq)]
struct LabFlow {
    file: String,
    struct_name: String,
    type_name: String,
}

/// Every `impl Flow for …` under `src/lab`, with the string literal its `type_name` returns.
///
/// ⚠ `#[cfg(test)]` is **not** skipped, the same choice `one_assembly_body.rs` made: a lab
/// type declared in test code is still a type whose name must not appear in the spine, and the
/// defect that file was written for lived in test code.
fn lab_flow_types() -> Vec<LabFlow> {
    let mut out = Vec::new();
    for (file, text) in sources(lab_dir()) {
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let Some(rest) = strip_line_comment(line)
                .trim()
                .strip_prefix("impl Flow for ")
            else {
                continue;
            };
            let struct_name = rest
                .split(['<', ' ', '{'])
                .next()
                .expect("split yields at least one part")
                .to_string();
            // The literal is read from the `type_name` body inside this `impl` block: forward
            // to `fn type_name`, then to the first quoted string after it. The block ends at
            // the first line that is a bare `}` at the impl's indentation — the tree is
            // rustfmt-formatted, which is what makes this reliable without a parser.
            let mut type_name = None;
            let mut seen_fn = false;
            for l in &lines[i + 1..] {
                if *l == "}" {
                    break;
                }
                if l.contains("fn type_name(") {
                    seen_fn = true;
                    continue;
                }
                if seen_fn {
                    if let Some(start) = l.find('"') {
                        let after = &l[start + 1..];
                        let end = after.find('"').expect("an unterminated string literal");
                        type_name = Some(after[..end].to_string());
                        break;
                    }
                }
            }
            out.push(LabFlow {
                file: file.clone(),
                struct_name,
                type_name: type_name.unwrap_or_else(|| {
                    panic!("an `impl Flow` in the lab has no `type_name` literal to read")
                }),
            });
        }
    }
    out
}

/// Both spellings of every lab flow type — what the spine and the manifests are scanned for.
fn lab_names() -> Vec<String> {
    let mut names: Vec<String> = lab_flow_types()
        .into_iter()
        .flat_map(|f| [f.struct_name, f.type_name])
        .collect();
    names.sort();
    names.dedup();
    names
}

/// `needle` as a whole word in `code` — so `ScaledMechanism` does not match inside
/// `ScaledMechanismBuilder`, and a shorter name cannot be found inside an unrelated identifier.
fn mentions(code: &str, needle: &str) -> bool {
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    code.match_indices(needle).any(|(at, _)| {
        let before = code[..at].chars().next_back();
        let after = code[at + needle.len()..].chars().next();
        !before.is_some_and(is_word) && !after.is_some_and(is_word)
    })
}

/// The derivation read something, and it read the type this batch added.
///
/// ⚠ The anti-vacuity half. A moved directory, a renamed trait, a reformatted `impl` header —
/// each would derive an **empty** roster and leave every assertion below passing over nothing.
#[test]
fn the_roster_is_derived_and_not_vacuous() {
    let files: Vec<String> = sources(lab_dir()).into_iter().map(|(n, _)| n).collect();
    assert!(
        files.contains(&"mechanism.rs".to_string()),
        "the lab's mechanism module is missing from {files:?}"
    );
    let roster = lab_flow_types();
    assert!(
        !roster.is_empty(),
        "no `impl Flow for` found under src/lab — the derivation reads nothing and every \
         assertion in this file is vacuous"
    );
    assert!(
        roster
            .iter()
            .any(|f| f.struct_name == "ScaledMechanism" && f.type_name == "ScaledMechanism"),
        "the scaled replacement is not in the roster: {roster:?}"
    );
}

/// A lab flow type reports its **own** name.
///
/// ⚠ This is the disguise check, and it is the reason the roster is read off the literal. A
/// lab type returning a frozen type's name would land in a canonical build's inventory as that
/// frozen type — the manifest would not move, `type_identity.rs` would not move, and an
/// alternative mechanism would be running inside the freeze under a name that is not its own.
#[test]
fn every_lab_flow_type_reports_its_own_name() {
    for f in lab_flow_types() {
        assert_eq!(
            f.struct_name, f.type_name,
            "{}: `impl Flow for {}` reports type_name {:?} — a lab mechanism must not wear \
             another type's name",
            f.file, f.struct_name, f.type_name
        );
    }
}

/// ⚠⚠ **The gate.** No lab mechanism is named anywhere in the biosphere spine.
///
/// Not "not wired into a canonical scenario" — *not present*. A construction on a branch no
/// frozen scenario reaches is invisible to every run and every manifest, and is exactly how an
/// alternative mechanism ends up one flag away from the frozen path. The fix is never to widen
/// this: an alternative form is composed on afterwards, through `lab::mechanism`.
#[test]
fn no_lab_mechanism_is_named_under_the_spine() {
    let names = lab_names();
    assert!(!names.is_empty(), "nothing to scan for");
    let mut found: Vec<String> = Vec::new();
    for (file, text) in sources(spine_dir()) {
        for (i, line) in text.lines().enumerate() {
            let code = strip_line_comment(line);
            for name in &names {
                if mentions(code, name) {
                    found.push(format!("{file}:{}  {}", i + 1, code.trim()));
                }
            }
        }
    }
    assert!(
        found.is_empty(),
        "lab-only mechanisms {names:?} are named in the spine:\n  {}\nAn alternative mechanism \
         must be composed onto the built registry (`lab::mechanism`), never constructed where \
         `build_season` can reach it.",
        found.join("\n  ")
    );
}

/// And no **committed manifest** names one — the half `manifest_writer.rs` cannot see.
///
/// That gate regenerates the manifest from the four canonical builds and compares bytes, so it
/// is green whenever the file matches the code. Wire a lab mechanism into a canonical build,
/// regenerate, and the frozen roster has silently grown a type that is not frozen science.
/// Reading the committed text directly is the check that does not follow the code.
#[test]
fn no_committed_manifest_names_a_lab_mechanism() {
    for (name, manifest, anchor) in [
        (
            "docs/biosphere-reference.manifest.json",
            BIOSPHERE_MANIFEST,
            "MaintenanceRespiration",
        ),
        // ⚠ A different anchor, not an oversight: the station manifest **delegates** the
        // biosphere, so its own `flow_set` holds the sibling domains' flows and names no
        // biosphere type at all. A shared anchor here would have failed for the right reason
        // and been "fixed" by weakening the check.
        (
            "docs/station-reference.manifest.json",
            STATION_MANIFEST,
            "CO2Scrubber",
        ),
    ] {
        // Anti-vacuity: the file really is the manifest, with a frozen flow type in it.
        assert!(
            manifest.contains(&format!("\"{anchor}\"")),
            "{name} does not name {anchor:?} — this scan is reading the wrong file, or the \
             manifest's shape changed"
        );
        for lab in lab_names() {
            assert!(
                !manifest.contains(&format!("\"{lab}\"")),
                "{name} names the lab-only mechanism {lab:?} — an alternative mechanism has \
                 entered the frozen roster, and regenerating the manifest is what hid it"
            );
        }
    }
}
