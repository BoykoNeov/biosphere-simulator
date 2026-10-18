//! Step 7a of the 2026-09-29 review: run what is authored (`docs/plans/post-roadmap-authored-plants.md`).
//!
//! Every file in the repo's `scenarios/` — the authored CONTENT, not the fixtures under
//! `rust/data/scenarios/` that `scenario_files.rs` reads — is loaded and run to its own
//! horizon, and held to what the platform promises any authored file and nothing more:
//!
//! * it **builds** (the schema, the interpreter, the build-time `k·h < 1` precondition);
//! * it **runs** under [`run_scenario`]'s gates — conservation every step, no rationing, no
//!   demand-controlled flow found pointing backwards;
//! * it is **repeatable** — two runs end bit-identical.
//!
//! **Not a golden and not a manifest entry.** "Authored is not validated" stays true: these
//! files earn conservation and determinism, never scientific endorsement, so no value is
//! pinned here.
//!
//! ⚠ **Why this exists.** Until 2026-10-18 nothing in `rust/crates` ran these files, and a
//! frozen-side change (the O₂ setpoint, 10.0 → 1995.0 on 2026-09-06) falsified one file's
//! reasoning with nothing going red. The unvalidated half of the tree can be broken silently
//! by the validated half; the freeze ceremony does not look here, so this test does.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use authoring::{load_scenario, run_scenario};

/// The repo's authored-content directory, `<repo>/scenarios/`.
fn content_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../scenarios")
}

/// Every `.yaml` file on disk in [`content_dir`], sorted — enumerated, never listed by hand,
/// so a file added later is run without anyone remembering to add it here.
fn authored_files() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(content_dir())
        .expect("the repo's scenarios/ directory is readable")
        .map(|entry| entry.expect("a readable directory entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "yaml"))
        .collect();
    files.sort();
    files
}

/// The final amounts by stock id, as bits — the repeatability comparison.
fn final_bits(path: &Path) -> Vec<(String, u64)> {
    let built = load_scenario(path, &BTreeMap::new())
        .unwrap_or_else(|e| panic!("{} does not build: {e}", path.display()));
    let result =
        run_scenario(built).unwrap_or_else(|e| panic!("{} does not run: {e}", path.display()));
    result
        .final_state
        .stocks
        .iter()
        .map(|(id, stock)| (id.clone(), stock.amount.to_bits()))
        .collect()
}

#[test]
fn the_census_is_not_empty() {
    // An empty or mis-pointed directory would make the test below pass by vacuity.
    let files = authored_files();
    assert!(
        files.len() >= 2,
        "expected at least the two authored habitats in {}, found {files:?}",
        content_dir().display()
    );
}

#[test]
fn every_authored_file_builds_runs_clean_and_repeats() {
    for path in authored_files() {
        let first = final_bits(&path);
        assert!(!first.is_empty(), "{} has no stocks", path.display());
        assert_eq!(
            first,
            final_bits(&path),
            "{}: two runs of the same file ended differently",
            path.display()
        );
    }
}
