//! The command-line surface of the three manifest writers — Stage-3 slice **S6**,
//! build item 1 (`docs/plans/post-roadmap-reference-flip.md`).
//!
//! # ⚠⚠ Why an argument parser is boundary code and not a program's private business
//!
//! `tests/crossport/test_manifest_writer.py` carried two claims. The byte gate — *the
//! committed manifest is what the reference writes* — was ported by slice S2 into each
//! crate's `tests/manifest_writer.rs`, and moved from a `subprocess` pipe to a direct
//! `manifest_text()` call. Its **control** did not move, and its stated reason died with
//! the pipe: it existed because the Python gate passed `--write-manifest <tmp>` and *"a
//! writer that ignored the flag and wrote the file anyway would make the comparison pass
//! while proving the wrong thing."* The Rust gate passes no flag, so that sentence is
//! about a program shape that no longer exists.
//!
//! What survives the move is a different hazard, and it is contract-adjacent: **with no
//! path, `--write-manifest` defaults to the committed contract itself.** So an argument
//! the parse mishandles does not write a stray file — it rewrites a *freeze contract*,
//! unreviewed, and since S6 both freeze docs name this command as the interim
//! regeneration route. That claim has to keep an owner after the Python file goes.
//!
//! # ⚠ It is a library function because an `examples/` program is a binary target
//!
//! The same structural fact that forced the byte gate into Python forces this: nothing in
//! `cargo test` can call into a binary. The tree already answered it once — S2 moved the
//! writer out of `examples/` into `freeze_manifest`, leaving *"the command-line surface
//! and nothing else"* behind. This finishes that move: the surface is now the three-line
//! `match` in the example and the decision is here, reachable by an ordinary test rather
//! than by a `cargo`-inside-`cargo` subprocess.
//!
//! It sits in `config` rather than in each writer for the reason [`crate::yaml`] does: one
//! implementation, because *a policy with two implementations has one that is stale*. The
//! part that genuinely differs per crate — which contract is the default target — is a
//! parameter, and each crate's test asserts its own.

use std::path::{Path, PathBuf};

/// What the command line asked for. Every path the program can take is one of these
/// variants, so a fall-through is a compile error rather than a silent default write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestAction {
    /// No argument: dump the reference's half of the manifest as JSON on stdout.
    Dump,
    /// `--write-manifest [path]`: serialize the whole contract to `path`, which is the
    /// committed contract itself when the argument is omitted.
    Write(PathBuf),
}

/// Parse a manifest writer's arguments, or return the usage text to print on stderr.
///
/// `program` names the example in the usage text; `default_target` is the committed
/// contract this writer owns, used when `--write-manifest` is given no path.
///
/// ⚠ **An argument this does not understand is an error, never a fall-through.** The two
/// wrong outcomes it forbids are the ones that look like success: dumping to stdout (so a
/// mistyped flag "works" and writes nothing), and taking the no-path branch (so a mistyped
/// flag rewrites the committed contract).
///
/// ⚠ **A trailing argument is rejected too, and that is a strengthening over the Python
/// original.** `--write-manifest <path> <extra>` used to ignore `<extra>` silently, and
/// `--write-manifest --nonsense` used to write a file *named* `--nonsense`. Both are the
/// same species as the unknown flag — an argument the program received and did not act on
/// — so they are refused here rather than left as the residue of a `match` on the first
/// argument only.
pub fn parse_args(
    program: &str,
    default_target: &Path,
    args: &[String],
) -> Result<ManifestAction, String> {
    let usage = |problem: &str| -> String {
        format!(
            "usage: {program} [--write-manifest [path]]\n  \
             (no argument dumps the reference's half of the manifest as JSON)\n  \
             (--write-manifest with no path REWRITES the committed contract at {})\n\
             {problem}",
            default_target.display()
        )
    };
    match args.first().map(String::as_str) {
        None => Ok(ManifestAction::Dump),
        Some("--write-manifest") => match args.len() {
            1 => Ok(ManifestAction::Write(default_target.to_path_buf())),
            2 => Ok(ManifestAction::Write(PathBuf::from(&args[1]))),
            _ => Err(usage(&format!(
                "--write-manifest takes at most one path; unexpected extra argument: {}",
                args[2]
            ))),
        },
        Some(other) => Err(usage(&format!("unknown argument: {other}"))),
    }
}
