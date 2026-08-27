//! Dump the **station** port's half of the freeze manifest as JSON — slices 3 and 7 of the
//! reference flip (`docs/plans/post-roadmap-reference-flip.md`).
//!
//!
//! ⚠ **The program is the thin part.** Everything this dumps and writes lives in
//! `station::freeze_manifest` since Stage-3 slice S2 — an `examples/` binary cannot be
//! called from a test, and the manifest byte gate has to call it. This file is now the
//! command-line surface and nothing else — and since S6 build item 1 the argument
//! parse is gone too (`config::manifest_cli`), leaving only the wiring.

use station::freeze_manifest::{committed_manifest_path, dump, write_manifest};
use config::manifest_cli::{parse_args, ManifestAction};

/// ⚠ **The decision is not here, and that is deliberate** (S6 build item 1). An
/// `examples/` program is a binary target, so nothing in `cargo test` can call into this
/// `main` — the same structural fact that kept the byte gate in Python until S2. So the
/// argument parse lives in `config::manifest_cli`, where a test can reach it, and the
/// default `--write-manifest` target lives in `station::freeze_manifest`, where this
/// crate's `tests/manifest_writer.rs` asserts it is *this* contract. What is left here is
/// the wiring, and `the_example_delegates_its_argument_parse` is what says so.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match parse_args("dump_station_inventory", &committed_manifest_path(), &args) {
        Ok(ManifestAction::Dump) => dump(),
        Ok(ManifestAction::Write(path)) => write_manifest(&path),
        Err(usage) => {
            eprintln!("{usage}");
            std::process::exit(2);
        }
    }
}
