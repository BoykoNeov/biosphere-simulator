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
const WRITER_SOURCE: &str = include_str!("../examples/dump_station_inventory.rs");

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
