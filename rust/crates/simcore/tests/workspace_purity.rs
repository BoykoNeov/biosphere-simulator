//! The workspace **purity + layering gate** — the Rust successor to Python's
//! `tests/test_simcore_purity.py`, and the first thing in this repo that asserts either
//! half of FINDING 8 (reference-flip §5o):
//!
//! * *"`simcore` in **both** trees carries zero third-party deps"* — the Rust half had
//!   only ever been prose in a `Cargo.toml` comment. The Python scan reads **Python
//!   packages**, so it never saw a Rust manifest and loses no Rust coverage when it
//!   retires; there was none to lose.
//! * *"`gdext` appears in `rust/crates/godot_bridge` and nowhere else."*
//!
//! ⚠ **`test_biosphere_purity.py`'s successor is NOT here** — it is
//! `domains/tests/biosphere_spine_purity.rs`, and the split is the point rather than
//! tidiness. That gate's subject is intra-crate: the biosphere spine must not reach the
//! config boundary, while `domains -> config` is a legitimate declared edge. Nothing this
//! file asserts can see that violation, and a header claiming otherwise would have been the
//! most expensive kind of wrong — a gate that reads as coverage.
//!
//! ## What the subject is, and why it is the dependency graph rather than the text
//!
//! The Python originals scan **import statements**, because in Python an import is the
//! only thing that couples two packages. Cargo splits that in two: a crate cannot name a
//! type it has not declared a dependency on, so in Rust the **manifest edge is the
//! coupling** and the `use` line is its consequence. Measured before this file was
//! written: `gdext` appears in four engine-crate sources (`station/src/session.rs`,
//! `station/src/palette.rs`, `station/src/bin/sim.rs`, `station/tests/session_parity.rs`)
//! and **every one of those is a doc comment saying the crate is gdext-free**. A literal
//! text scan would have reddened on a clean tree and been widened until it passed.
//!
//! So the gate is over the manifests. `use godot::…` in an engine crate cannot compile
//! without the edge this file forbids, which makes the text half redundant *by
//! construction* rather than merely unnecessary. The one thing the edge check does not
//! imply is a re-export path, so [`nothing_depends_on_the_bridge`] closes that
//! separately.
//!
//! ## Placement
//!
//! It reads every member manifest, so no single crate is its natural owner; it lives
//! with `simcore` because the invariant's canonical statement in `CLAUDE.md` is about the
//! core, and because `simcore` is the crate every workspace test run compiles. Reading a
//! sibling's `Cargo.toml` is file I/O in a test, not a crate dependency — the layering it
//! asserts is not one it participates in.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// `rust/` — this crate's manifest dir is `rust/crates/simcore`.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// One declared dependency: the crate it names, the table it was declared in, and
/// whether it is a `path = …` sibling (in-workspace) or a registry crate (third-party).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Dep {
    name: String,
    section: String,
    is_path: bool,
}

/// Is `header` (the text inside `[...]`) a dependency table?
///
/// Covers the plain three **and** the target-conditional forms
/// (`target.'cfg(windows)'.dependencies`) — the section a scan keyed on `[dependencies]`
/// alone silently misses.
fn is_dependency_section(header: &str) -> bool {
    let tail = header.rsplit('.').next().unwrap_or(header);
    matches!(tail, "dependencies" | "dev-dependencies" | "build-dependencies")
}

/// Every dependency a manifest's text declares, across every dependency table.
///
/// A deliberately small hand-rolled reader — `simcore` may not take a TOML crate, and a
/// gate that had to violate the invariant it guards would be self-refuting. It handles
/// what a Cargo dependency table can hold: `name = "1"`, `name = { path = "…" }`, the
/// dotted `name.workspace = true` / `name.features = […]` forms (which carry **no**
/// version string, so a reader keyed on `version =` misses them entirely), and inline
/// tables spanning several lines.
fn declared_deps(manifest: &str) -> Vec<Dep> {
    let mut deps: Vec<Dep> = Vec::new();
    let mut section = String::new();
    let mut depth: i32 = 0; // brace depth, so a multi-line inline table's body is skipped
    for raw in manifest.lines() {
        let line = strip_comment(raw).trim().to_string();
        if line.is_empty() {
            continue;
        }
        if depth > 0 {
            depth += brace_delta(&line);
            continue;
        }
        if let Some(header) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = header.trim().to_string();
            continue;
        }
        if !is_dependency_section(&section) {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        // `serde.workspace = true` and `serde.features = [...]` both name `serde`.
        let name = key.trim().split('.').next().unwrap_or("").trim().trim_matches('"');
        if name.is_empty() {
            continue;
        }
        // `path` must be a *key* of the inline table, not any occurrence of the word: a
        // registry dep declaring `features = ["path"]` would otherwise be classified as an
        // in-workspace sibling and walk straight past the third-party clause.
        let is_path = has_key(value, "path");
        match deps.iter_mut().find(|d| d.name == name && d.section == section) {
            // Only record a name once per table, however many dotted keys it has.
            Some(existing) => existing.is_path |= is_path,
            None => deps.push(Dep {
                name: name.to_string(),
                section: section.clone(),
                is_path,
            }),
        }
        depth += brace_delta(&line);
    }
    deps
}

/// Drop a trailing `#` comment, respecting quotes (`path = "a#b"` is not a comment).
fn strip_comment(line: &str) -> &str {
    let mut in_quotes = false;
    for (i, b) in line.bytes().enumerate() {
        match b {
            b'"' => in_quotes = !in_quotes,
            b'#' if !in_quotes => return &line[..i],
            _ => {}
        }
    }
    line
}

/// Does the inline-table text `value` declare `key` as a key (`key = …`)?
fn has_key(value: &str, key: &str) -> bool {
    value.match_indices(key).any(|(at, _)| {
        let before = value[..at].chars().next_back();
        let after = value[at + key.len()..].trim_start();
        before.is_none_or(|c| !c.is_alphanumeric() && c != '_') && after.starts_with('=')
    })
}

fn brace_delta(line: &str) -> i32 {
    let open = line.chars().filter(|c| *c == '{').count() as i32;
    let close = line.chars().filter(|c| *c == '}').count() as i32;
    open - close
}

/// The `members = [...]` roster from the workspace manifest, in declaration order.
fn workspace_members(manifest: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut inside = false;
    for raw in manifest.lines() {
        let line = strip_comment(raw).trim().to_string();
        if !inside && !line.starts_with("members") {
            continue;
        }
        inside = true;
        for piece in line.split(',') {
            let trimmed = piece.trim().trim_matches(|c| c == '[' || c == ']' || c == '"');
            if let Some(name) = trimmed.strip_prefix("crates/") {
                out.push(name.trim_matches('"').to_string());
            }
        }
        if line.contains(']') {
            break;
        }
    }
    out
}

/// `crate name -> its declared deps`, read off disk for every workspace member.
fn workspace_deps() -> BTreeMap<String, Vec<Dep>> {
    let root = workspace_root();
    let members = workspace_members(
        &std::fs::read_to_string(root.join("Cargo.toml")).expect("workspace manifest is readable"),
    );
    members
        .into_iter()
        .map(|m| {
            let path: PathBuf = root.join("crates").join(&m).join("Cargo.toml");
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("member manifest {} is readable: {e}", path.display()));
            (m, declared_deps(&text))
        })
        .collect()
}

/// The crates that carry the simulation: the zero-third-party set. `godot_bridge` is
/// deliberately absent — it is the one crate whose whole job is the impure boundary.
const ENGINE_CRATES: [&str; 5] = ["authoring", "config", "domains", "simcore", "station"];

/// The layering, stated as the **complete** allowed out-edge set per engine crate.
/// `simcore` and `config` are leaves: `simcore` is the pure core, and `config` is the
/// file boundary that sits *below* `domains` and may not reach up into the engine.
fn allowed_edges(krate: &str) -> &'static [&'static str] {
    match krate {
        "simcore" => &[],
        "config" => &[],
        "domains" => &["config", "simcore"],
        "authoring" => &["config", "domains", "simcore"],
        "station" => &["config", "domains", "simcore"],
        other => panic!("no layering rule recorded for engine crate {other:?}"),
    }
}

// --------------------------------------------------------------------------- //
// The scan                                                                     //
// --------------------------------------------------------------------------- //

/// The anti-vacuity guard, and it is an **equality** rather than a subset: a new crate
/// joining the workspace must be given a layering rule here, not silently exempted by a
/// scan that never looks at it. The Rust analogue of the Python scans'
/// `test_scan_is_not_vacuous`, with the loophole those left (a subset assertion) closed —
/// a manifest roster is small enough to pin exactly.
#[test]
fn the_scan_sees_every_workspace_member_and_no_others() {
    let seen: BTreeSet<String> = workspace_deps().keys().cloned().collect();
    let expected: BTreeSet<String> =
        ["authoring", "config", "domains", "godot_bridge", "simcore", "station"]
            .iter()
            .map(|s| s.to_string())
            .collect();
    assert_eq!(
        seen, expected,
        "the workspace roster moved; give the new crate a layering rule in allowed_edges() \
         and a decision about whether it may carry third-party deps"
    );
}

/// Invariant #11, the Rust half: no engine crate declares a registry dependency, in
/// **any** dependency table. `[dev-dependencies]` counts — a test-only third-party crate
/// is still a third-party crate in a tree whose readers are deliberately hand-rolled, and
/// it is the table a narrower gate would have missed.
#[test]
fn no_engine_crate_carries_a_third_party_dependency() {
    for (krate, deps) in workspace_deps() {
        if !ENGINE_CRATES.contains(&krate.as_str()) {
            continue;
        }
        let third_party: Vec<&Dep> = deps.iter().filter(|d| !d.is_path).collect();
        assert!(
            third_party.is_empty(),
            "{krate} declares third-party dependencies {third_party:?} — the engine crates \
             carry zero of them (CLAUDE.md, \"Core is pure\"); the readers are hand-rolled"
        );
    }
}

/// `simcore` is stricter still: **zero** dependencies of any kind, path ones included.
/// This is the clause that catches the core reaching down into the boundary layer — the
/// Rust shape of the Python scan's second half (`simcore` importing `sim_io` / `config` /
/// `domains`), which is a *path* edge here and so invisible to the third-party clause.
#[test]
fn simcore_declares_no_dependencies_at_all() {
    let deps = workspace_deps().remove("simcore").expect("simcore is a member");
    assert!(
        deps.is_empty(),
        "simcore declares {deps:?} — the pure core depends on nothing, not even a sibling \
         crate; boundary code lives in config/ and sim_io/"
    );
}

/// The layering, every edge at once: each engine out-edge is on its crate's allowed list.
/// `config`'s empty list is the "sits below `domains`, may not reach up into the engine"
/// rule; `domains`' list is the crate-scale analogue of "no domain imports another".
#[test]
fn every_engine_edge_is_one_the_layering_allows() {
    for (krate, deps) in workspace_deps() {
        if !ENGINE_CRATES.contains(&krate.as_str()) {
            continue;
        }
        let allowed = allowed_edges(&krate);
        for dep in &deps {
            assert!(
                allowed.contains(&dep.name.as_str()),
                "{krate} -> {} is not an edge the layering allows (allowed: {allowed:?}); an \
                 inversion here is an unfreeze-shaped decision, not a refactor",
                dep.name
            );
        }
    }
}

/// FINDING 8's second half: the `gdext` edge exists in exactly one manifest. Stated over
/// the dependency graph rather than the source text — see this file's header for the
/// measurement that decided that.
#[test]
fn only_the_bridge_depends_on_gdext() {
    for (krate, deps) in workspace_deps() {
        if krate == "godot_bridge" {
            continue;
        }
        for dep in &deps {
            assert!(
                !(dep.name.starts_with("godot") || dep.name.starts_with("gdext")),
                "{krate} declares {} in [{}] — gdext appears in godot_bridge and nowhere else, \
                 so no gdext type can reach an engine crate",
                dep.name,
                dep.section
            );
        }
    }
}

/// The anti-vacuity guard on the clause above: if the bridge ever stopped naming the
/// crate (renamed, vendored, feature-gated away), `only_the_bridge_depends_on_gdext`
/// would pass over a workspace with no gdext in it at all and assert nothing.
#[test]
fn the_bridge_really_does_depend_on_gdext() {
    let deps = workspace_deps()
        .remove("godot_bridge")
        .expect("godot_bridge is a member");
    assert!(
        deps.iter().any(|d| d.name.starts_with("godot") && !d.is_path),
        "godot_bridge declares no gdext dependency ({deps:?}) — the containment gate above \
         would then be vacuously green"
    );
}

/// The one thing the edge check does *not* imply: a gdext type could still reach an engine
/// crate by re-export if something depended on the bridge. Nothing may.
#[test]
fn nothing_depends_on_the_bridge() {
    for (krate, deps) in workspace_deps() {
        for dep in &deps {
            assert_ne!(
                dep.name, "godot_bridge",
                "{krate} depends on godot_bridge — the bridge is the top of the graph, and a \
                 crate below it could re-export a gdext type through this edge"
            );
        }
    }
}

// --------------------------------------------------------------------------- //
// Discrimination: the reader catches what it claims to                         //
//                                                                              //
// All green on a clean tree only proves the scan does not false-positive. These //
// are the Python originals' `test_detector_flags_*` controls, ported subject by  //
// subject, over the manifest shapes a hand-rolled reader can plausibly miss.     //
// --------------------------------------------------------------------------- //

#[test]
fn the_reader_sees_a_plain_registry_dependency() {
    let deps = declared_deps("[package]\nname = \"x\"\n\n[dependencies]\nnumpy = \"1.2\"\n");
    assert_eq!(deps.len(), 1, "{deps:?}");
    assert_eq!(deps[0].name, "numpy");
    assert!(!deps[0].is_path);
}

#[test]
fn the_reader_sees_a_dev_dependency() {
    // The table a scan keyed on `[dependencies]` alone misses. No engine crate has one
    // today, so on the real tree nothing else asserts this clause.
    let deps = declared_deps("[dev-dependencies]\nproptest = \"1\"\n");
    assert_eq!(deps.len(), 1, "{deps:?}");
    assert_eq!(deps[0].section, "dev-dependencies");
    assert!(!deps[0].is_path);
}

#[test]
fn the_reader_sees_a_build_dependency() {
    let deps = declared_deps("[build-dependencies]\ncc = \"1\"\n");
    assert_eq!(deps.len(), 1, "{deps:?}");
    assert_eq!(deps[0].section, "build-dependencies");
}

#[test]
fn the_reader_sees_a_target_conditional_dependency() {
    let deps = declared_deps("[target.'cfg(windows)'.dependencies]\nwinapi = \"0.3\"\n");
    assert_eq!(deps.len(), 1, "target-conditional deps are deps: {deps:?}");
    assert_eq!(deps[0].name, "winapi");
    assert!(!deps[0].is_path);
}

#[test]
fn the_reader_sees_a_workspace_inherited_dependency() {
    // `serde.workspace = true` carries no version string at all — a reader keyed on
    // `version =` (or on a quoted value) reports zero deps for a manifest full of them.
    let deps =
        declared_deps("[dependencies]\nserde.workspace = true\nserde.features = [\"derive\"]\n");
    assert_eq!(deps.len(), 1, "one dep named twice is one dep: {deps:?}");
    assert_eq!(deps[0].name, "serde");
    assert!(!deps[0].is_path);
}

#[test]
fn the_reader_sees_a_dependency_inside_a_multi_line_inline_table() {
    let deps = declared_deps(
        "[dependencies]\nfoo = {\n    version = \"1\",\n    features = [\"a\"],\n}\nbar = \"2\"\n",
    );
    let names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
    assert_eq!(names, ["foo", "bar"], "the table's body is not a dep: {deps:?}");
}

#[test]
fn the_reader_distinguishes_a_path_sibling_from_a_registry_crate() {
    let deps =
        declared_deps("[dependencies]\nsimcore = { path = \"../simcore\" }\nquote = \"1\"\n");
    let by_name: BTreeMap<&str, bool> = deps.iter().map(|d| (d.name.as_str(), d.is_path)).collect();
    assert!(by_name["simcore"]);
    assert!(!by_name["quote"]);
}

/// The word `path` appearing in a *value* does not make a dependency an in-workspace
/// sibling. Without this, `foo = { version = "1", features = ["path"] }` would be classified
/// as a path dep and walk straight past `no_engine_crate_carries_a_third_party_dependency` —
/// a hole in the direction that matters, since the clause only ever *excuses* path deps.
#[test]
fn the_reader_does_not_mistake_the_word_path_in_a_value_for_a_path_dependency() {
    let deps = declared_deps("[dependencies]
foo = { version = \"1\", features = [\"path\"] }
");
    assert_eq!(deps.len(), 1, "{deps:?}");
    assert!(!deps[0].is_path, "a feature named path is not a path dependency: {deps:?}");
}

#[test]
fn the_reader_ignores_a_commented_out_dependency() {
    // Every manifest in this workspace carries a prose block above its `[dependencies]`
    // explaining why it has none; a reader that read comments would flag all of them.
    let deps = declared_deps("[dependencies]\n# numpy = \"1.2\"  <- never\n");
    assert!(deps.is_empty(), "a commented-out dep is not a dep: {deps:?}");
}

#[test]
fn the_reader_ignores_non_dependency_tables() {
    let deps = declared_deps(
        "[package]\nname = \"x\"\nversion = \"0.1.0\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    assert!(deps.is_empty(), "package/lib keys are not deps: {deps:?}");
}

#[test]
fn the_member_reader_finds_the_roster() {
    let members = workspace_members(
        "[workspace]\nresolver = \"2\"\nmembers = [\n    \"crates/config\",\n    \"crates/simcore\",\n]\n\n# a comment naming crates/nothing\n",
    );
    assert_eq!(members, ["config", "simcore"]);
}
