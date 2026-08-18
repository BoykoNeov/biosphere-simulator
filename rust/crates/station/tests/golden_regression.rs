//! **The reference compares its own runs against the committed goldens** — the station's
//! eight, plus the whole-census gates over all twenty-one files on disk. Stage-3 slice S2
//! of the reference flip; see `domains/tests/golden_regression.rs` for why this did not
//! exist (FINDING 3) and what a failure means.
//!
//! The census lives here rather than in `domains` for the reason `station::goldens` states:
//! `station` depends on `domains` and not the reverse, so this is the lowest crate that can
//! see all nineteen reference-authored goldens at once.

use domains::goldens::{committed, committed_goldens, compare, Cost, Golden, Numerics, Verdict};
use station::goldens::{all, STATION};

/// This file's own source, for the `#[ignore]` control below.
///
/// ⚠ `include_str!` of the file itself, the same crude-on-purpose device
/// `manifest_writer.rs` uses to check that the frozen literals are literals. A textual
/// check that is red on the real mistake beats an elegant one that is green on it.
const THIS_FILE: &str = include_str!("golden_regression.rs");

/// The `domains` half of the pair, so the `#[ignore]` census below covers **both** files.
///
/// ⚠ Added after review caught the gap: the first draft counted `#[ignore]` in this file
/// only, so one added to `domains/tests/golden_regression.rs` was invisible to the very
/// control written to make skipping visible. A census that covers half its subject is the
/// failure mode it exists to catch.
const DOMAINS_FILE: &str = include_str!("../../domains/tests/golden_regression.rs");

/// The workflow, so the CI step that runs the ignored test is itself guarded.
///
/// ⚠⚠ **This exists because the first draft claimed it could not.** The comment on
/// `the_ignored_set_is_exactly_the_expensive_roster` said *"nothing inside the suite can
/// guard this line"* — false by this repo's own idiom, three files away:
/// `manifest_writer.rs` greps the writer's source text and `science_gates` greps a file for
/// a recorded bound, both for exactly this reason (a check the type system cannot make).
/// A deleted CI step does not fail anything; it silently stops running the only gate on the
/// largest assembly in the repo. So the step is pinned textually here.
const CI_WORKFLOW: &str = include_str!("../../../../.github/workflows/ci.yml");

fn check(golden: &Golden) {
    let produced = (golden.run)();
    let expected = committed(golden.name);
    match compare(&produced, &expected, golden.numerics) {
        Verdict::ByteExact | Verdict::StructurallyEqual => {}
        Verdict::Differs(why) => panic!(
            "{}: the committed golden is no longer this run's output.\n\
             ⚠ Since the reference flip the golden IS the reference's bytes, so this is \
             the reference moving, not a port disagreement.\n\
             numerics: {:?} (platform: {})\n{why}",
            golden.name,
            golden.numerics,
            if cfg!(windows) {
                "windows — byte-exactness demanded"
            } else {
                "not the generation platform — transcendental goldens are compared \
                 structurally; see domains::goldens"
            },
        ),
    }
}

#[test]
fn every_cheap_station_golden_is_still_this_reference_s_output() {
    for golden in STATION.iter().filter(|g| g.cost == Cost::Cheap) {
        check(golden);
    }
}

/// The fully-coupled sealed station: ~1.3 M sub-steps, ~100 s at any optimization level.
///
/// ⚠⚠ **`#[ignore]`d, and the cost is the run rather than the build.** Measured
/// 2026-08-19: 378 s at the stock dev profile, 116 s at `opt-level = 2`, 93 s in release.
/// So no build knob buys this back and the choice was between paying it on every
/// `cargo test` and paying it in CI. CI runs it (`cargo test -- --ignored`); the control
/// below is what stops `#[ignore]` from spreading quietly to anything else.
#[test]
#[ignore = "~100s: 1.3M sub-steps over five domains. CI runs it via `cargo test -- --ignored`"]
fn the_sealed_station_golden_is_still_this_reference_s_output() {
    let golden = STATION
        .iter()
        .find(|g| g.name == "sealed_station_state.json")
        .expect("the sealed station is on the roster");
    check(golden);
}

/// ⚠⚠ The anti-green-by-skip control, and the reason `Cost` is a roster field at all.
///
/// This repo has two recorded incidents of a gate that was green because it never ran
/// ([[pdf-pins-green-by-skip-on-ci]], and `test_manifest_writer.py`'s own note that its CI
/// collection was *checked, not inherited*). A bare `#[ignore]` is that shape: nothing
/// relates the attribute to a reason, so a second `#[ignore]` added for convenience is
/// indistinguishable from this one, which was added for a measured 100 seconds.
///
/// So the roster states the cost and this test asserts the two agree in **both**
/// directions: exactly one golden is `Expensive`, and exactly one `#[ignore]` appears in
/// this file. A new `#[ignore]` without a matching `Cost::Expensive` is red, and an
/// `Expensive` golden nobody wrote a test for is red. The census reads **both**
/// golden-regression files, so a skip added on the `domains` side is not invisible here.
///
/// ⚠ The companion [`ci_still_runs_the_ignored_tests`] guards the direction this comment
/// once claimed was unguardable — *"nothing inside the suite can guard this line"*. That
/// was false by this repo's own idiom (`manifest_writer.rs` greps the writer's source;
/// `science_gates` greps a file for a recorded bound), and it is corrected here rather
/// than quietly dropped: the claim was **wrong**, not merely incomplete.
#[test]
fn the_ignored_set_is_exactly_the_expensive_roster() {
    let expensive: Vec<&str> = all()
        .iter()
        .filter(|g| g.cost == Cost::Expensive)
        .map(|g| g.name)
        .collect();
    assert_eq!(
        expensive,
        vec!["sealed_station_state.json"],
        "the expensive roster moved. Every `Cost::Expensive` golden needs its own \
         `#[ignore]`d test here AND a CI step that runs it, or it is checked nowhere."
    );

    // ⚠ Anchored on the *attribute*, not on the bare string. The first draft counted
    // `#[ignore` anywhere and found **12** — this file's own prose discusses the attribute
    // eleven times. That is `manifest_writer.rs`'s recorded lesson landing again in a new
    // place: an anchor that matches prose as well as syntax checks whichever came first.
    // A doc line starts `///` or `//!`, so the attribute is the line that starts with it.
    let ignores = [THIS_FILE, DOMAINS_FILE]
        .iter()
        .flat_map(|src| src.lines())
        .filter(|l| l.trim_start().starts_with("#[ignore"))
        .count();
    assert_eq!(
        ignores, 1,
        "found {ignores} `#[ignore]` attributes in this file but {} expensive golden(s). \
         An `#[ignore]` with no measured cost beside it on the roster is a gate that \
         stopped running for a reason nobody wrote down.",
        expensive.len()
    );
    // The control on the anchor: the bare string really is ambiguous here, so the reason
    // above is a measurement rather than a claim.
    assert!(
        THIS_FILE.matches("#[ignore").count() > ignores,
        "the bare string was expected to be ambiguous in this file; if it is not, the \
         line-anchored count above no longer needs to be line-anchored"
    );
}

/// ⚠⚠ The other half of the `#[ignore]` discipline: **CI must still run it.**
///
/// The roster control above guards one direction (a skip appearing without a measured
/// cost). This guards the other, and it is the one that actually loses coverage: an
/// `#[ignore]`d test that nothing runs anywhere is not a slow gate, it is **no gate**, and
/// deleting the workflow step fails nothing. A malformed workflow is worse still — GitHub
/// silently does not run it, which is this repo's two recorded green-by-skip incidents in
/// their purest form.
///
/// Crude on purpose, and with the same standing as `manifest_writer.rs`'s source greps: it
/// cannot check that CI is *green*, only that the step is still spelled in the file.
#[test]
fn ci_still_runs_the_ignored_tests() {
    assert!(
        CI_WORKFLOW.contains("cargo test -- --ignored"),
        "the `cargo test -- --ignored` step is gone from .github/workflows/ci.yml. \
         `the_sealed_station_golden_is_still_this_reference_s_output` is `#[ignore]`d, so \
         that step is the ONLY thing that runs it — without it the largest assembly in the \
         repo is checked nowhere. Restore the step, or un-ignore the test and accept the \
         ~100 s on every `cargo test`."
    );
    // ⚠ The control on the anchor: the string must be a `run:` command, not a mention in
    // the explanatory comment block that sits directly above it. Same lesson as the
    // `#[ignore]` count in this file, which the first draft got wrong in exactly this way.
    let as_command = CI_WORKFLOW
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            !t.starts_with('#') && t.contains("cargo test -- --ignored")
        })
        .count();
    assert_eq!(
        as_command, 1,
        "expected exactly one uncommented `cargo test -- --ignored` line in ci.yml, found \
         {as_command} — a match that is only inside the comment means this test passes \
         while nothing runs the ignored gate"
    );
}

// --------------------------------------------------------------------------- //
// The census — the successor to test_golden_provenance.py                      //
// --------------------------------------------------------------------------- //

/// The two goldens on disk the reference does **not** author, each with the reason.
///
/// ⚠ The successor to `regen_goldens_from_rust.{PYTHON_FOLDED, NO_RUST_REFERENT}`. Both
/// entries carry a *measured* reason rather than a category judgement, and the reasons are
/// carried across verbatim in substance because the classification is what a test cannot
/// re-derive: a test can check that no emitter produces them, not *why*.
const NOT_REFERENCE_AUTHORED: &[(&str, &str)] = &[
    (
        "drift_summary.json",
        "folded Python-side. C5 ported the fold to Rust (`domains::biosphere::drift`) and \
         this artifact still did not move: folding the Rust series moves 4 of its 45 \
         values (<=7 ULP, consumer years 3-4), which would need Python tolerance-gating \
         and turn `test_every_diverging_scenario_keeps_a_byte_gated_sibling` red. \
         Deferred to its own ceremony; plan §5h.",
    ),
    (
        "state_snapshot.json",
        "not a simulation run at all — a hand-authored `sim_io` serialization fixture that \
         the reference *reads* (`simcore/src/snapshot.rs` reconstructs its bits). It is an \
         INPUT to the port, so 'regenerate it from Rust' would be a round trip.",
    ),
];

/// The two rosters partition the directory — no leftovers, no phantoms.
///
/// ⚠ Enumerated from `rust/data/golden/`, never hand-listed:
/// `docs/log/coverage-roster-is-not-the-manifest.md` records this repo believing a
/// hand-maintained list was the census.
#[test]
fn every_committed_golden_is_classified() {
    let mut classified: Vec<&str> = all().iter().map(|g| g.name).collect();
    classified.extend(NOT_REFERENCE_AUTHORED.iter().map(|(name, _)| *name));
    classified.sort();
    let on_disk = committed_goldens();

    let unclassified: Vec<&String> = on_disk
        .iter()
        .filter(|n| !classified.contains(&n.as_str()))
        .collect();
    let phantom: Vec<&&str> = classified
        .iter()
        .filter(|n| !on_disk.iter().any(|d| d == *n))
        .collect();
    assert!(
        unclassified.is_empty() && phantom.is_empty(),
        "the golden census is out of step with rust/data/golden/:\n  \
         unclassified on disk: {unclassified:?}\n  classified but gone:  {phantom:?}\n\
         ⚠ A new golden must either join a crate's roster with the run that produces it, \
         or go on NOT_REFERENCE_AUTHORED *with the reason*. Do not widen this assertion."
    );
}

/// A golden cannot be both reference-authored and not.
#[test]
fn the_two_rosters_are_disjoint() {
    for (name, _) in NOT_REFERENCE_AUTHORED {
        assert!(
            !all().iter().any(|g| g.name == *name),
            "{name} is on both rosters — it cannot be authored by the reference and \
             lack a reference referent at once"
        );
    }
}

/// Every roster entry names a real file, and no two entries name the same one.
///
/// ⚠ The control on the census above: `every_committed_golden_is_classified` compares two
/// sets, and a duplicated name is invisible to a set comparison. Two runs claiming the same
/// golden is exactly the mistake the horizon-argument emitters (`perennial`, `consumer`,
/// each serving two goldens) make easy to write.
#[test]
fn the_roster_names_are_unique_and_real() {
    let names: Vec<&str> = all().iter().map(|g| g.name).collect();
    let mut sorted = names.clone();
    sorted.sort();
    let before = sorted.len();
    sorted.dedup();
    assert_eq!(
        before,
        sorted.len(),
        "two roster entries name the same golden: {names:?}"
    );
    for name in &names {
        assert!(
            domains::goldens::golden_dir().join(name).is_file(),
            "{name} is on a roster but is not a file in rust/data/golden/"
        );
    }
}

/// ⚠⚠ **The counted forcing literal — S1's gate, re-homed where it survives S6.**
///
/// S1 (plan §5s) found the golden census prose stale in *two directions at once*:
/// `golden_platform.py` said "eighteen of the twenty-five", when C5 had made it 19 authored
/// and C6 had taken the directory to 21. Nothing was broken — the rosters are derived and
/// were right — but every sentence a reader uses to orient was wrong, and no gate has ever
/// owned that layer. S1's fix was a counted literal in
/// `tests/crossport/test_golden_provenance.py`, and it recorded its own defect in the act:
/// **that file is in the tree S6 deletes**, so the fix closed a prose-rot hole with a gate
/// that dies. The plan's FINDING 2 names carrying it as S2's job.
///
/// This is that successor. Deliberately a **tripwire, not a second census**: it cannot
/// check that a sentence is true, only that somebody looked when the count moved. Same
/// shape and same standing as `param_files`'s `assert_eq!(files.len(), 15)`, and stated as
/// such so nobody later "simplifies" it into a derived check that would rot silently.
#[test]
fn the_golden_census_counts_are_what_the_prose_says() {
    let on_disk = committed_goldens().len();
    let authored = all().len();
    assert_eq!(
        (on_disk, authored),
        (21, 19),
        "the golden census moved: {authored} of {on_disk} are reference-authored, not \
         19 of 21. That is fine — but the counts are quoted as PROSE in CLAUDE.md \
         ('21 golden files (19 the reference's own bytes)'), in \
         rust/crates/station/src/goldens.rs and rust/crates/domains/src/goldens.rs \
         ('the eleven goldens'), and (while the checker lives) in \
         tests/golden_platform.py and tests/crossport/regen_goldens_from_rust.py. \
         Nothing else checks them. Update those, then this literal, in the same commit."
    );
}

/// ⚠ The station's half of the platform classification — see the `domains` file's twin.
///
/// The pure-arithmetic set is inherited from `@windows_golden_only`'s placement in
/// `tests/test_regression_*.py` and not chosen here: `cabin` and `water_recovery` carry no
/// marker (forced/linear crew respiration + first-order ECLSS controls; no biosphere, no
/// `sin`/`powf`), everything else does. Across both crates that is four ungated goldens,
/// matching `golden_platform.py`'s own list.
#[test]
fn the_pure_arithmetic_set_is_what_the_python_policy_says_it_is() {
    let mut pure: Vec<&str> = all()
        .iter()
        .filter(|g| g.numerics == Numerics::PureArithmetic)
        .map(|g| g.name)
        .collect();
    pure.sort();
    assert_eq!(
        pure,
        vec![
            "cabin_gas_state.json",
            "crew_state.json",
            "eclss_state.json",
            "water_recovery_state.json",
        ],
        "the transcendental classification moved. A golden becoming `Transcendental` \
         weakens its gate off the generation platform, so the change belongs in the same \
         commit as the reason for it."
    );
}
