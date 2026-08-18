//! The **authoring** freeze contract's byte gate — Stage-3 slice S2 of the reference flip.
//!
//! ⚠ This crate had no `manifest_writer.rs` before S2, and the reason is recorded rather
//! than filled in: C7 measured the authoring contract for the anti-derived-literal trap its
//! two siblings carry and **found none** — its hand-authored keys are a phase number, two
//! repo paths and two blocks of prose, and this crate owns no constant any of them could be
//! spliced from. So there are no source-text greps here, only the byte comparison. *A
//! control with no test to redden is the finding, not a gap to fill* — inventing a guard to
//! match the siblings' shape would be the co-adaptation this repo refuses.

// --------------------------------------------------------------------------- //
// The byte gate — S2's successor to tests/crossport/test_manifest_writer.py    //
// --------------------------------------------------------------------------- //

/// The committed contract, as bytes.
///
/// ⚠ **Reading out of `docs/` is not an S1 regression, and the distinction matters.** S1's
/// rule is that the reference must not compile or read out of *the tree being deleted*
/// (`src/`, `tests/`). `docs/` is neither: it is where the freeze **contracts** live, it
/// outlives the checker, and the writer's own `repo_root()` already makes this exact climb
/// to decide where `--write-manifest` defaults to. A manifest gate that could not see the
/// committed manifest would have no subject.
const COMMITTED: &str = include_str!("../../../../docs/authoring-reference.manifest.json");

/// ⚠⚠ **The gate FINDING 2 named first, and the reason it could not be written until now.**
///
/// C7's headline is that no Python program *writes* a frozen contract. That stayed true and
/// was never the whole picture: **the program that CHECKED the contract was still Python** —
/// `tests/crossport/test_manifest_writer.py`, which shells out to `cargo run
/// --write-manifest` and compares the bytes. It had to shell out, because an `examples/`
/// program is a binary target and nothing in `cargo test` can call into one. So retiring
/// the checker at S6 would have disarmed the trap C7 installed (*a provenance-only edit now
/// FORCES a regeneration*) with nothing red.
///
/// S2 moved the writer into `authoring::freeze_manifest`, which is what makes this callable
/// at all. What it catches, unchanged from the Python original:
///
/// * a frozen surface that moved and was not regenerated — that is an **unfreeze**;
/// * a **hand edit** to the committed manifest, which is a generated artifact;
/// * a change to the writer's own serialization.
///
/// ⚠ What it deliberately does NOT catch is the anti-derived literals — splicing the
/// constant produces a byte-identical manifest (measured; see this file's header). That is
/// why the source-text greps above exist and are not redundant with this.
///
/// ⚠ **No pipe, and that is load-bearing.** Slice C4 froze cp1252-mangled prose into a
/// contract with every gate green, because a `subprocess` pipe decoded UTF-8 with the
/// Windows locale and *both* sides were mangled identically. Here there is no process
/// boundary at all: the writer is a function call and both sides are `&str`.
#[test]
fn the_committed_manifest_is_what_the_reference_writes() {
    let regenerated = authoring::freeze_manifest::manifest_text();
    if regenerated == COMMITTED {
        return;
    }
    let first = regenerated
        .lines()
        .zip(COMMITTED.lines())
        .enumerate()
        .find(|(_, (a, b))| a != b)
        .map(|(i, (a, b))| format!("line {}:\n  writes:    {a}\n  committed: {b}", i + 1))
        .unwrap_or_else(|| {
            format!(
                "identical for {} lines, then the lengths differ ({} vs {})",
                regenerated.lines().count().min(COMMITTED.lines().count()),
                regenerated.lines().count(),
                COMMITTED.lines().count()
            )
        });
    panic!(
        "the committed docs/authoring-reference.manifest.json is not what the reference \
         writes today.\n\
         Two readings, and the first question decides which:\n\
         * the reference tree changed and the manifest was not regenerated — that is an \
         UNFREEZE. Follow the ceremony in the contract's own doc, then re-run the writer \
         and review the diff.\n\
         * the manifest was edited by hand — it is a generated artifact; the edit belongs \
         in the writer (crates/authoring/src/freeze_manifest.rs), which is what makes it \
         reproducible.\n\
         Regenerate: cd rust && cargo run -p authoring --example dump_authoring_inventory \
         -- --write-manifest\n{first}"
    );
}

/// ⚠ The control on the gate above: it must be comparing **this** contract, not
/// nothing and not a sibling.
///
/// ⚠⚠ **Narrowed after review, because the first version's prose overstated it.** It
/// claimed to guard "an `include_str!` of a wrong-but-present path", which the byte gate
/// already catches loudly, and it discriminated on `"_authority"` — a string ALL THREE
/// manifests carry, so it could not tell them apart at all. Same species as the "nothing
/// inside the suite can guard this line" claim S2's first half had to retract: **a doc
/// comment asserting a property nobody tested.** It now claims only what it owns — the
/// file is not truncated, and it is the authoring contract rather than a sibling.
///
/// It still earns its keep because `include_str!` embeds at **compile time**: unlike the
/// Python original, which read the file at runtime and would have thrown, a
/// stale-but-present path here is silent.
///
/// ⚠ And this gate depends on `.gitattributes` (`* text=auto eol=lf`). `include_str!`
/// does **not** normalize line endings while `dumps` always emits LF, so a checkout
/// producing CRLF manifests would redden the byte gate with the WRONG diagnosis ("edited
/// by hand"). Checked rather than assumed: all three committed manifests carry zero CR
/// bytes and the attribute pins `eol=lf`.
#[test]
fn the_committed_manifest_is_this_contract_and_is_not_truncated() {
    assert!(
        COMMITTED.len() > 1_000,
        "the committed authoring manifest read as {} bytes — the include path is wrong \
         or the file is truncated, and the byte gate above is comparing against nothing",
        COMMITTED.len()
    );
    assert!(
        COMMITTED.contains("docs/authoring-reference.md"),
        "the loaded manifest does not name docs/authoring-reference.md as its contract \
         doc, so this gate is pointed at a DIFFERENT freeze contract. Every manifest \
         carries an _authority block, which is why that is not the thing to check here."
    );
}
