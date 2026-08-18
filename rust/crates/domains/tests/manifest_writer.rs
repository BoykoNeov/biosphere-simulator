//! The guard for the one thing C7's own gate cannot see — the **anti-derived literals**
//! of the biosphere freeze manifest (`docs/plans/post-roadmap-reference-flip.md`, C7).
//!
//! # ⚠⚠ Why this file exists, measured rather than argued
//!
//! `dt_days` and `integrator` are frozen by *hand* on purpose: a manifest that read
//! `BIO_DT` would auto-follow a step change, which is the opposite of a freeze. The
//! 2026-08-14 step move became a deliberate ceremony only because that literal went red.
//!
//! C7 moved the manifest writer **into the crate that owns `BIO_DT`**, and the control
//! was run before this file was written: replacing `Json::num("0.25")` with
//! `Json::num(format!("{BIO_DT}"))` produces a **byte-identical manifest**. So the
//! regeneration diff — C7's whole gate — is blind to the violation, and so is the
//! cross-port check that compares the frozen literal against `BIO_DT` (it compares equal
//! either way; what it protects is the *ceremony*, which only exists while the literal is
//! typed). Today the two are the same number; the day someone splices the constant is the
//! day the freeze quietly stops being one, and **nothing would be red**.
//!
//! That is the same shape as the step unfreeze's own lesson — *no test at `dt = 1` can
//! tell a correct conversion from a wrong one, because the two are the same integer.*
//!
//! # What this checks, and why it is crude on purpose
//!
//! It reads the writer's **source text** and asserts the literals are literals. There is
//! precedent in this tree for a textual check standing in for one the type system cannot
//! make — `science_gates::the_bound_literals_appear_at_their_locus` greps a file for a
//! recorded bound. A crude check that is red on the real mistake beats an elegant one
//! that is green on it.
//!
//! The complementary half is structural and lives in the writer: `Json::Number` is
//! constructed only from **text** (`Json::num` takes no `f64`), so splicing the constant
//! is not a silent type coercion but a visible `format!`.

/// The writer, as text. `include_str!` rather than a path walk, so a moved or renamed
/// example is a compile error rather than a test that quietly reads nothing.
const WRITER_SOURCE: &str = include_str!("../examples/dump_biosphere_inventory.rs");

/// The frozen step is written as a quoted literal, never read from `BIO_DT`.
#[test]
fn the_frozen_step_is_a_typed_literal_and_not_the_constant() {
    let line = WRITER_SOURCE
        .lines()
        .find(|l| l.contains("(\"dt_days\", Json::"))
        .expect("the writer emits a dt_days key");
    assert!(
        line.contains("Json::num(\"0.25\")"),
        "dt_days must be a hand-typed literal — a manifest that derives the step \
         auto-follows a step change, which is the opposite of a freeze. Found: {line}"
    );
    assert!(
        !line.contains("BIO_DT"),
        "the frozen step must not be spliced from the constant it freezes: {line}"
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

/// ⚠ The control for the two tests above: they must find the line they check, and
/// exactly one of it. It earned its keep on the first run — the original anchor was the
/// bare key `"dt_days"`, which matches **two** lines (the emission site and the
/// `_authority` row that classifies it), and `find` would have read whichever came
/// first. The anchor is now the emission syntax, and this test is what says so.
#[test]
fn each_frozen_literal_is_emitted_on_exactly_one_line() {
    for key in ["(\"dt_days\", Json::", "(\"integrator\", Json::"] {
        let hits = WRITER_SOURCE.lines().filter(|l| l.contains(key)).count();
        assert_eq!(
            hits, 1,
            "expected exactly one emission site for {key}, found {hits} — a second site \
             means this file checks the wrong one"
        );
    }
}
