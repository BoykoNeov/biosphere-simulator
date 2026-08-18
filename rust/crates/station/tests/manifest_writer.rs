//! The guard for the one thing C7's own gate cannot see — the **anti-derived literals**
//! of the station freeze manifest (`docs/plans/post-roadmap-reference-flip.md`, C7's
//! station half).
//!
//! # ⚠⚠ Why this file exists, and why the trap here is worse than the biosphere's
//!
//! `numerics_note` is hand-maintained prose naming three integration steps, and
//! `integrator` is the anti-derived scheme literal. C7 moved the manifest writer **into
//! the crate that owns all three steps** (`sealed_station_scenario()`'s `bio_dt` and
//! `cabin_dt`, the energy scenario's `power_dt`), where splicing one is a one-line edit.
//!
//! The control was run before this file was written, and it came back **partial** — which
//! is the finding:
//!
//! | referent | the note as written | spliced with `{}` | regeneration gate |
//! |---|---|---|---|
//! | `bio_dt` | `dt=1/4 day` | `dt=0.25 day` | **red** — the bytes move |
//! | `cabin_dt` | `dt=60 s` | `dt=60 s` | **green** — `60.0_f64` Displays as `60` |
//! | `power_dt` | `dt=3600 s` | `dt=3600 s` | **green** — same reason |
//!
//! So two of the three would auto-follow the code with C7's whole gate — regenerate and
//! compare bytes — seeing nothing. And unlike the biosphere there is **no second guard to
//! fall back on**: that contract's `dt_days` is at least compared against `BIO_DT` across
//! the port boundary, while this manifest has no structured step key at all (adding one
//! widens the frozen surface, which is its own ceremony and has been declined twice).
//!
//! That is the step unfreeze's own lesson in a new place — *no test at `dt = 1` can tell
//! a correct conversion from a wrong one, because the two are the same integer* — except
//! that here the collision is with the **rendering**, not the value.
//!
//! # What this checks, and why it is crude on purpose
//!
//! It reads the writer's **source text** and asserts the literals are literals. There is
//! precedent in this tree for a textual check standing in for one the type system cannot
//! make. A crude check that is red on the real mistake beats an elegant one that is green
//! on it.
//!
//! The complementary half is structural and lives in the writer: `numerics_note` is
//! emitted through `Json::s(NUMERICS_NOTE)` where `NUMERICS_NOTE` is a `&str` const, so
//! splicing a step means writing a visible `format!` rather than changing a number.

/// The writer, as text. `include_str!` rather than a path walk, so a moved or renamed
/// example is a compile error rather than a test that quietly reads nothing.
/// ⚠ **Re-pointed by S2**: the writer moved from `examples/` into `src/freeze_manifest.rs`
/// so the byte gate below could call it (an `examples/` program is a binary target). The
/// anchors these tests grep travelled with the code, and
/// `each_frozen_literal_is_emitted_on_exactly_one_line` is what proves the re-point landed:
/// a stale path here would find ZERO lines and the `expect` would fire, not pass quietly.
const WRITER_SOURCE: &str = include_str!("../src/freeze_manifest.rs");

/// The three constants a `numerics_note` splice would reach for.
const STEP_CONSTANTS: [&str; 3] = ["bio_dt", "cabin_dt", "power_dt"];

/// The frozen numerics prose is a typed literal, never assembled from the steps it names.
#[test]
fn the_frozen_numerics_note_is_a_typed_literal_and_not_the_constants() {
    let line = WRITER_SOURCE
        .lines()
        .find(|l| l.contains("const NUMERICS_NOTE: &str ="))
        .expect("the writer declares a NUMERICS_NOTE const");
    assert!(
        line.contains("\"Euler everywhere;"),
        "numerics_note must be a hand-typed literal — a manifest that assembles the \
         steps auto-follows a step change, which is the opposite of a freeze. Found: \
         {line}"
    );
    for constant in STEP_CONSTANTS {
        assert!(
            !line.contains(constant),
            "the frozen numerics prose must not be spliced from {constant}, the step it \
             freezes. Two of the three splices are BYTE-IDENTICAL to the written note, \
             so the regeneration diff would not catch this: {line}"
        );
    }
}

/// ⚠ And the emission site, separately: the const could be honest while the key that
/// carries it is built by `format!`.
#[test]
fn the_numerics_note_is_emitted_by_name_and_not_formatted() {
    let line = WRITER_SOURCE
        .lines()
        .find(|l| l.contains("(\"numerics_note\", Json::"))
        .expect("the writer emits a numerics_note key");
    assert!(
        line.contains("Json::s(NUMERICS_NOTE)"),
        "numerics_note must be emitted from the const by name: {line}"
    );
    assert!(
        !line.contains("format!"),
        "a formatted numerics_note is how a step gets spliced invisibly: {line}"
    );
}

/// The integrator is a literal too, and has no constant on either side to splice.
#[test]
fn the_frozen_integrator_is_a_typed_literal() {
    let line = WRITER_SOURCE
        .lines()
        .find(|l| l.contains("(\"integrator\", Json::"))
        .expect("the writer emits an integrator key");
    assert!(
        line.contains("Json::s(\"EulerIntegrator\")"),
        "integrator must be a hand-typed literal: {line}"
    );
}

/// ⚠ The control for the tests above: each must find the line it checks, and exactly one
/// of it. The biosphere's version of this test earned its keep on the first run — the
/// original anchor there was a bare key name, which matched **two** lines (the emission
/// site and the `_authority` row that classifies it), so `find` read whichever came
/// first. Every anchor here is emission syntax rather than a bare key, and this test is
/// what says so.
///
/// ⚠ `numerics_note` is the case that proves the point: the bare string `numerics_note`
/// appears **three** times in the writer (the const, the emission site, and the
/// `_authority` row), so an anchor on the key alone would silently check the wrong one.
#[test]
fn each_frozen_literal_is_emitted_on_exactly_one_line() {
    for anchor in [
        "const NUMERICS_NOTE: &str =",
        "(\"numerics_note\", Json::",
        "(\"integrator\", Json::",
    ] {
        let hits = WRITER_SOURCE.lines().filter(|l| l.contains(anchor)).count();
        assert_eq!(
            hits, 1,
            "expected exactly one site for {anchor}, found {hits} — a second site means \
             this file checks the wrong one"
        );
    }
    // The bare key really is ambiguous — asserted, so the reason above is a measurement
    // rather than a claim.
    let bare = WRITER_SOURCE
        .lines()
        .filter(|l| l.contains("numerics_note"))
        .count();
    assert!(
        bare > 1,
        "the bare key was expected to be ambiguous; if it is not, this control has lost \
         its subject and the anchors above no longer need to be emission syntax"
    );
}

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
const COMMITTED: &str = include_str!("../../../../docs/station-reference.manifest.json");

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
/// S2 moved the writer into `station::freeze_manifest`, which is what makes this callable
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
    let regenerated = station::freeze_manifest::manifest_text();
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
        "the committed docs/station-reference.manifest.json is not what the reference \
         writes today.\n\
         Two readings, and the first question decides which:\n\
         * the reference tree changed and the manifest was not regenerated — that is an \
         UNFREEZE. Follow the ceremony in the contract's own doc, then re-run the writer \
         and review the diff.\n\
         * the manifest was edited by hand — it is a generated artifact; the edit belongs \
         in the writer (crates/station/src/freeze_manifest.rs), which is what makes it \
         reproducible.\n\
         Regenerate: cd rust && cargo run -p station --example dump_station_inventory \
         -- --write-manifest\n{first}"
    );
}

/// ⚠ The control on the gate above: it must be comparing something.
///
/// A `COMMITTED` that resolved to an empty or truncated file would make the assertion
/// meaningful-looking and vacuous. The Python original could not make this mistake (it read
/// the file at runtime and would have failed loudly); `include_str!` of a wrong-but-present
/// path is the new failure mode this move introduces, so it gets its own assertion.
#[test]
fn the_committed_manifest_is_actually_loaded() {
    assert!(
        COMMITTED.len() > 1_000,
        "the committed station manifest read as {} bytes — the include path is wrong \
         or the file is truncated, and the byte gate above is comparing against nothing",
        COMMITTED.len()
    );
    assert!(
        COMMITTED.contains("\"_authority\""),
        "the committed station manifest has no _authority block — this is not the file \
         the gate is supposed to be comparing"
    );
}

/// ⚠⚠ **The station `aux_set` is empty BY DELEGATION, not by a dump that wired nothing.**
///
/// The residue S2's enumeration found in `test_inventory_parity.py`. `[] == []` is satisfied
/// by a dump that never walked anything — [[inventory-parity-built]]'s recorded lesson — and
/// since C7 the empty list is *written into* the frozen contract by a regeneration rather
/// than merely compared against one. So the byte gate agrees with itself here and proves
/// nothing about this axis.
///
/// What this owns is the *reason* the set is empty: the siblings and seams are all
/// conserved-quantity flows, and the biosphere's accumulators live in the slow registry the
/// station manifest **delegates** away. If a station-side aux process is ever added this
/// goes red, and the emptiness caveat stops applying.
#[test]
fn the_station_aux_axis_is_empty_by_delegation() {
    let manifest = station::freeze_manifest::manifest_text();
    assert!(
        manifest.contains("\"aux_set\": []"),
        "the station manifest now freezes aux processes — the empty-set caveat no longer \
         applies and the parity row gains teeth of its own. Re-read why this axis was \
         allowed to be empty before regenerating."
    );
    assert!(
        manifest.contains("\"delegates_to\": \"docs/biosphere-reference.manifest.json\""),
        "the delegation key is what makes an empty aux_set legitimate rather than a dump \
         that walked nothing. Without it, `[] == []` is inert."
    );
}
