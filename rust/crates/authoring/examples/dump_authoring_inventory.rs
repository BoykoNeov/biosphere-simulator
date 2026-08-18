//! Dump the **authoring platform's** half of its freeze manifest as JSON, and — since
//! slice C7 — **write the whole contract**. Reference flip
//! (`docs/plans/post-roadmap-reference-flip.md`).
//!
//! ⚠ **The program is the thin part.** Everything this dumps and writes lives in
//! `authoring::freeze_manifest` since Stage-3 slice S2 — an `examples/` binary cannot be
//! called from a test, and the manifest byte gate has to call it. This file is now the
//! command-line surface and nothing else.

use authoring::freeze_manifest::{dump, repo_root, write_manifest};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None => dump(),
        Some("--write-manifest") => {
            let path = args.get(1).map(PathBuf::from).unwrap_or_else(|| {
                repo_root()
                    .join("docs")
                    .join("authoring-reference.manifest.json")
            });
            write_manifest(&path);
        }
        Some(other) => {
            eprintln!(
                "usage: dump_authoring_inventory [--write-manifest [path]]
                   (no argument dumps the reference's half of the manifest as JSON)
                 unknown argument: {other}"
            );
            std::process::exit(2);
        }
    }
}
