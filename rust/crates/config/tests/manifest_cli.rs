//! The manifest writers' argument parse — S6 build item 1, the successor to
//! `tests/crossport/test_manifest_writer.py::test_the_writer_refuses_an_unknown_argument`.
//!
//! # ⚠⚠ The claim being ported is NOT the one the Python docstring gave
//!
//! The Python control said it existed because the byte gate passed `--write-manifest
//! <tmp>` and *"a writer that ignored the flag and wrote the file anyway would make the
//! comparison pass while proving the wrong thing."* Slice S2 moved the byte gate to a
//! direct `manifest_text()` call, so there is no flag on the gate's path any more and that
//! sentence describes a program shape this repo no longer has. Porting it verbatim would
//! have been a test written to a dead rationale.
//!
//! What survives — and is *worse* than the original, which is why it earns a successor
//! rather than a retirement — is the **default**. `--write-manifest` with no path resolves
//! to the committed contract itself, so an argument this parse mishandles does not produce
//! a stray temp file: it rewrites a freeze contract with no review. Since S6 both freeze
//! docs name this command as the interim regeneration route, so it is a live operator path
//! and not a curiosity.
//!
//! # ⚠ The mutations, and which test is the one that reddens
//!
//! | mutation | reddens |
//! |---|---|
//! | an unknown argument falls through to `Dump` | `an_unknown_argument_is_refused…` |
//! | an unknown argument falls through to the **default write** | the same, and it is the dangerous half |
//! | the path argument is ignored and the default is written | `the_given_path_is_the_one_written…` |
//! | no argument writes instead of dumping | `no_argument_dumps…` |
//! | a trailing argument is silently ignored | `a_trailing_argument_is_refused` |
//!
//! ⚠ There is deliberately **no** test that the usage text has a particular wording. It is
//! stderr help, gated by nobody, and a test on its prose would be a pin on formatting
//! wearing the name of a gate. What *is* asserted is that the message names the argument
//! that was refused and the file that would have been overwritten — the two facts an
//! operator needs to act, and the two a silent refusal would lose.

use std::path::{Path, PathBuf};

use config::manifest_cli::{parse_args, ManifestAction};

const PROGRAM: &str = "dump_biosphere_inventory";

fn default_target() -> PathBuf {
    PathBuf::from("/repo/docs/biosphere-reference.manifest.json")
}

fn parse(args: &[&str]) -> Result<ManifestAction, String> {
    let owned: Vec<String> = args.iter().map(|s| (*s).to_string()).collect();
    parse_args(PROGRAM, &default_target(), &owned)
}

/// No argument is the dump, and it must not become a write.
#[test]
fn no_argument_dumps_and_writes_nothing() {
    assert_eq!(parse(&[]), Ok(ManifestAction::Dump));
}

/// `--write-manifest` with no path targets the committed contract — stated as a test
/// rather than left implicit, because it is the whole reason the parse needs a gate.
#[test]
fn the_bare_flag_targets_the_committed_contract() {
    assert_eq!(
        parse(&["--write-manifest"]),
        Ok(ManifestAction::Write(default_target())),
        "--write-manifest with no path must resolve to the caller's committed contract; \
         a parse that resolved it to anything else would send a regeneration somewhere \
         nobody reviews"
    );
}

/// The path is honoured. ⚠ This is the Python control's assertion, kept for the mutation
/// it catches rather than for its original reason: a parse that ignored the path and took
/// the default branch would rewrite the **committed contract** while the caller believed
/// it was writing a temp file.
#[test]
fn the_given_path_is_the_one_written_and_not_the_default() {
    let target = Path::new("/tmp/scratch.manifest.json");
    assert_eq!(
        parse(&["--write-manifest", "/tmp/scratch.manifest.json"]),
        Ok(ManifestAction::Write(target.to_path_buf())),
    );
    // The control: the two paths this test distinguishes must actually be different, or
    // the assertion above passes under the mutation it is written to catch.
    assert_ne!(target, default_target());
}

/// ⚠ An argument the program does not understand is an error, never a fall-through — and
/// the two fall-throughs it forbids are the ones that look like success.
#[test]
fn an_unknown_argument_is_refused_rather_than_ignored() {
    let err = parse(&["--nonsense"]).expect_err("an unknown argument must not be accepted");
    assert!(
        err.contains("usage:") && err.contains("--nonsense"),
        "the refusal must name the argument it refused: {err}"
    );
    assert!(
        err.contains("biosphere-reference.manifest.json"),
        "the usage text must name the file --write-manifest would overwrite, which is \
         the fact that makes the flag dangerous: {err}"
    );
}

/// The `=` spelling is a real operator mistake, and it is refused rather than silently
/// treated as a filename to create.
#[test]
fn the_equals_spelling_is_refused() {
    assert!(parse(&["--write-manifest=/tmp/x.json"]).is_err());
}

/// ⚠ **A strengthening over the Python original, not a port of it.** `--write-manifest
/// <path> <extra>` used to ignore `<extra>`, and `--write-manifest --nonsense` used to
/// write a file *named* `--nonsense`. Both are the same species as the unknown flag — an
/// argument the program received and did not act on.
#[test]
fn a_trailing_argument_is_refused() {
    let err = parse(&["--write-manifest", "/tmp/x.json", "--extra"])
        .expect_err("a trailing argument must not be ignored");
    assert!(err.contains("--extra"), "{err}");
}

/// ⚠ The control on every test above: `ManifestAction` must be able to tell its two
/// variants apart. `assert_eq!` on an enum whose `PartialEq` was derived wrong — or on a
/// single-variant enum — would pass every assertion in this file while proving nothing.
#[test]
fn the_two_actions_are_distinguishable() {
    assert_ne!(
        ManifestAction::Dump,
        ManifestAction::Write(default_target())
    );
    assert_ne!(
        ManifestAction::Write(default_target()),
        ManifestAction::Write(PathBuf::from("/tmp/other.json"))
    );
}
