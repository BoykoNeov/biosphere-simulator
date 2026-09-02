//! The regeneration path's gates — S6 build item 2.
//!
//! # ⚠⚠ What is testable here and what deliberately is not
//!
//! The tool's *act* — run nineteen goldens and compare — is minutes of CPU and is already
//! gated: `{domains,station}/tests/golden_regression.rs` compares every run against its
//! committed bytes on every `cargo test`. Duplicating that here would buy nothing and cost
//! a 1.3 M-sub-step station run per test binary.
//!
//! What was never gated on either port is everything *around* the act: the argument parse,
//! the selection, and — the one S6 recorded losing — the **validation that stands between a
//! produced artifact and a freeze contract's values on disk**. Those are what this file
//! owns, and all of them are cheap: the validator is exercised against the committed bytes,
//! which are read rather than produced.
//!
//! ⚠ **No test in this file writes into `rust/data/golden/`.** The write path is asserted
//! by its refusals and by the two-phase split, never by rewriting a contract to see if it
//! can be rewritten.

use domains::goldens::{committed, validate, Cost, Golden, Numerics, Shape};
use station::goldens::all;
use station::regen::{parse_args, regenerate_in, select, summary, Outcome, Request};

// --------------------------------------------------------------------------- //
// The validator — the check S6 lost, and the reason it needed a shape axis      //
// --------------------------------------------------------------------------- //

/// ⚠⚠ **Every committed golden is a well-formed artifact of its declared shape.**
///
/// This is the standing form of the check the Python tool ran only on the write path — and
/// running it against the *committed* bytes is strictly more than the original did, because
/// the original validated only what it had just produced and the report path validated
/// nothing at all. Cheap: nineteen file reads, no runs.
#[test]
fn every_committed_golden_validates_against_its_declared_shape() {
    for golden in all() {
        let text = committed(golden.name);
        validate(golden.name, &text, golden.shape).unwrap_or_else(|why| {
            panic!(
                "the committed {} does not validate as {:?}: {why}\n⚠ Either the file is \
                 malformed on disk, or its declared shape is wrong — and the shape is what \
                 the regeneration tool trusts before it writes.",
                golden.name, golden.shape
            )
        });
    }
}

/// ⚠⚠ **The one golden that is not a snapshot, named — because it is why the axis exists.**
///
/// `sealed_energy_drift_summary.json` is a folded summary with no `version` key. The Python
/// validator raised on a missing version, so from slice C5 (which moved this golden into the
/// emitted group) until S6, `--write` would have died part-way through — after rewriting
/// whichever earlier goldens had moved. Nobody saw it because nobody ran `--write`.
///
/// This pins the classification so a future edit cannot quietly re-declare it a snapshot and
/// re-create the abort. ⚠ It also pins the *count*: exactly one folded summary, so a second
/// one arriving is a decision somebody makes rather than a row that slips in.
#[test]
fn exactly_one_golden_is_a_folded_summary_and_it_is_the_drift_one() {
    let summaries: Vec<&str> = all()
        .iter()
        .filter(|g| g.shape == Shape::FoldedSummary)
        .map(|g| g.name)
        .collect();
    assert_eq!(
        summaries,
        vec!["sealed_energy_drift_summary.json"],
        "the folded-summary set moved. That set is exactly the goldens that CANNOT be \
         validated by reconstructing an engine state, which is why the shape is declared \
         rather than sniffed — see domains::goldens::Shape."
    );
}

/// ⚠ The control on the two tests above: the validator must be capable of failing.
///
/// A validator that returned `Ok` for everything would make
/// `every_committed_golden_validates_against_its_declared_shape` pass while standing
/// between nothing and the disk — the exact shape of a green gate with no subject. Three
/// corruptions, one per failure mode the write path must refuse.
#[test]
fn the_validator_refuses_what_a_write_must_never_put_on_disk() {
    let snapshot = committed("crew_state.json");

    // Truncation — the classic half-written file.
    let truncated = &snapshot[..snapshot.len() / 2];
    assert!(
        validate("truncated", truncated, Shape::StateSnapshot).is_err(),
        "a truncated snapshot must be refused"
    );

    // Well-formed JSON that is not a snapshot: the version guard, which is exactly the
    // check that fired on the folded summary.
    assert!(
        validate("versionless", "{\"n\": 1}", Shape::StateSnapshot).is_err(),
        "a JSON document with no schema version must be refused"
    );

    // A float leaf that turned into a label — caught by hexfloat::parse, not by a key name.
    let mangled = snapshot.replacen("\"0x", "\"NaNx", 1);
    assert_ne!(mangled, snapshot, "the corruption must actually change the text");
    assert!(
        validate("mangled", &mangled, Shape::StateSnapshot).is_err(),
        "a snapshot whose hex-float leaf stopped parsing must be refused"
    );
}

/// The folded-summary arm refuses too, and its weakness is bounded rather than asserted
/// away: it cannot re-fire an engine invariant, but it must still catch a malformed write.
#[test]
fn the_folded_summary_arm_refuses_malformed_and_empty_documents() {
    let real = committed("sealed_energy_drift_summary.json");
    validate("real", &real, Shape::FoldedSummary).expect("the committed summary validates");

    assert!(validate("truncated", &real[..real.len() / 2], Shape::FoldedSummary).is_err());
    assert!(validate("not json", "{", Shape::FoldedSummary).is_err());
    assert!(validate("not an object", "[1, 2]", Shape::FoldedSummary).is_err());
    assert!(
        validate("empty", "{}", Shape::FoldedSummary).is_err(),
        "an empty object is a truncated write, not a summary"
    );
    assert!(
        validate("no floats", "{\"horizon_years\": 15}", Shape::FoldedSummary).is_err(),
        "a summary that folded nothing must be refused — the count is what makes this arm \
         more than a JSON parse"
    );
    assert!(
        validate("bad float", "{\"x\": \"0x1.zzp+0\"}", Shape::FoldedSummary).is_err(),
        "a hex-float-shaped leaf that does not parse must be refused"
    );
}

// --------------------------------------------------------------------------- //
// The argument parse                                                           //
// --------------------------------------------------------------------------- //

fn parse(args: &[&str]) -> Result<Request, String> {
    let owned: Vec<String> = args.iter().map(|s| (*s).to_string()).collect();
    parse_args(&owned)
}

/// ⚠ **Reporting is the default and `--write` is explicit** — the discipline the Python
/// tool carried, and the one thing a parse bug here would silently invert.
#[test]
fn no_argument_reports_and_writes_nothing() {
    assert_eq!(
        parse(&[]),
        Ok(Request {
            write: false,
            only: None
        })
    );
}

#[test]
fn the_write_flag_is_the_only_thing_that_turns_writing_on() {
    assert_eq!(
        parse(&["--write"]),
        Ok(Request {
            write: true,
            only: None
        })
    );
    assert_eq!(
        parse(&["--only", "season"]),
        Ok(Request {
            write: false,
            only: Some("season".to_string())
        }),
        "--only must not imply --write: a narrowed run is still a report by default"
    );
}

/// ⚠ An unknown argument is refused rather than ignored. The dangerous fall-through here is
/// not the same as the manifest writer's — there is no default target to clobber — but a
/// mistyped `--only` that silently became a full nineteen-golden `--write` is worse.
#[test]
fn an_unknown_argument_is_refused() {
    let err = parse(&["--onlyy", "season"]).expect_err("must be refused");
    assert!(err.contains("usage:") && err.contains("--onlyy"), "{err}");
    assert!(parse(&["--only"]).is_err(), "--only with no value must be refused");
}

// --------------------------------------------------------------------------- //
// The selection, and the narrowing it admits                                   //
// --------------------------------------------------------------------------- //

/// No filter selects the whole roster — the property the summary's arithmetic rests on.
#[test]
fn no_filter_selects_every_reference_authored_golden() {
    assert_eq!(select(None).len(), all().len());
    assert_eq!(select(None).len(), 19, "the roster moved; see the census gates");
}

/// A filter narrows, and a filter that matches nothing is not a quiet success.
#[test]
fn a_filter_narrows_and_a_filter_that_matches_nothing_selects_nothing() {
    let selected = select(Some("perennial"));
    assert_eq!(selected.len(), 2, "both perennial horizons");
    assert!(selected.iter().all(|g| g.name.contains("perennial")));
    assert!(
        select(Some("no_such_golden")).is_empty(),
        "an empty selection must be visible to the caller — regenerate() turns it into an \
         error rather than reporting '0 changed', which would read as coverage"
    );
}

/// ⚠⚠ **A narrowed run says it was narrowed.** This is the hazard `--only` introduces and
/// the whole of the answer to it: "0 would change" over one golden must not read like "0
/// would change" over the directory.
#[test]
fn the_summary_of_a_narrowed_run_names_the_narrowing() {
    let outcomes = vec![Outcome {
        name: "season_euler_state.json",
        changed: false,
        ulp_only: false,
        written: false,
    }];
    let narrowed = summary(
        &Request {
            write: false,
            only: Some("season".to_string()),
        },
        &outcomes,
    );
    assert!(narrowed.contains("NARROWED"), "{narrowed}");
    assert!(
        narrowed.contains("1 of 19"),
        "the summary must show the selection against the whole roster: {narrowed}"
    );

    let full = summary(
        &Request {
            write: false,
            only: None,
        },
        &outcomes,
    );
    // The control: the marker must be absent on an unnarrowed run, or it says nothing.
    assert!(!full.contains("NARROWED"), "{full}");
}

/// The report tells the operator what to do next, and only when there is something to do.
#[test]
fn a_reported_change_names_the_next_step_and_a_clean_report_does_not() {
    let changed = vec![Outcome {
        name: "season_euler_state.json",
        changed: true,
        ulp_only: false,
        written: false,
    }];
    let clean = vec![Outcome {
        name: "season_euler_state.json",
        changed: false,
        ulp_only: false,
        written: false,
    }];
    let request = Request {
        write: false,
        only: None,
    };
    assert!(summary(&request, &changed).contains("--write"));
    assert!(summary(&request, &changed).contains("manifest writer"));
    assert!(!summary(&request, &clean).contains("--write"));
}

// --------------------------------------------------------------------------- //
// The frozen-golden census — the last claim ported out of the Python provenance //
// gate (S6 build item 3)                                                        //
// --------------------------------------------------------------------------- //

/// ⚠⚠ **Every golden a freeze manifest names is one the reference authors.**
///
/// The successor to `test_golden_provenance.test_every_frozen_golden_has_a_rust_emitter_or_
/// a_stated_reason`. It is the one claim of that file that the census gates in
/// `golden_regression.rs` do **not** cover: those partition the *directory*, this ties the
/// **contracts** to the authorship roster. A frozen golden the reference cannot produce is a
/// contract whose values nothing on the canonical side can regenerate — which is precisely
/// the state the flip exists to end.
///
/// ⚠ The manifests are read as **data**, never hand-listed, so a scenario added to a
/// contract joins this gate's subject automatically.
#[test]
fn every_frozen_golden_is_one_the_reference_authors() {
    let mut frozen: Vec<String> = Vec::new();
    for name in [
        "biosphere-reference.manifest.json",
        "station-reference.manifest.json",
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../docs")
            .join(name);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let manifest = simcore::json::parse(&text)
            .unwrap_or_else(|e| panic!("{name} is not JSON: {e:?}"));
        let scenarios = manifest
            .get("scenarios")
            .and_then(simcore::json::JsonValue::as_object)
            .unwrap_or_else(|| panic!("{name} has no scenarios object"));
        for (_, entry) in scenarios {
            let golden = entry
                .get("golden")
                .and_then(simcore::json::JsonValue::as_str)
                .unwrap_or_else(|| panic!("{name}: a scenario entry names no golden"));
            frozen.push(golden.to_string());
        }
    }
    frozen.sort();
    frozen.dedup();

    // The control first: the manifests must actually have been read. A path typo, or a
    // renamed key, would leave this empty and every assertion below vacuous.
    assert_eq!(
        frozen.len(),
        20,
        "expected the 20 frozen goldens (7 biosphere + 13 station), found {}: {frozen:?}",
        frozen.len()
    );

    let authored: Vec<&str> = all().iter().map(|g| g.name).collect();
    let unauthored: Vec<&String> = frozen
        .iter()
        .filter(|f| !authored.contains(&f.as_str()))
        .collect();
    assert_eq!(
        unauthored,
        vec![&"drift_summary.json".to_string()],
        "the set of FROZEN goldens the reference cannot produce moved.\n⚠ Exactly one is \
         expected and it is a recorded, dated gap: `drift_summary.json` is Python's fold, \
         left behind by C5 for a reason that has since dissolved (the comparator that made \
         it a blocker no longer exists) — it is now an unfreeze with its own ceremony, not \
         a classification. Anything else here is a frozen contract whose values nothing on \
         the canonical side can regenerate."
    );
}

/// ⚠ The `Numerics` axis is orthogonal to `Shape`, and this says so rather than leaving it
/// to be inferred: the folded summary is `Transcendental`, so a reader cannot conclude
/// "summary ⇒ ungated" or the reverse from one example.
#[test]
fn the_shape_axis_is_not_a_restatement_of_the_numerics_axis() {
    let summary_golden = all()
        .into_iter()
        .find(|g| g.shape == Shape::FoldedSummary)
        .expect("there is one");
    assert_eq!(summary_golden.numerics, Numerics::Transcendental);
    let pure_snapshots = all()
        .iter()
        .filter(|g| g.shape == Shape::StateSnapshot && g.numerics == Numerics::PureArithmetic)
        .count();
    assert!(
        pure_snapshots > 0,
        "the two axes must cut the roster differently, or one of them is decoration"
    );
}

// --------------------------------------------------------------------------- //
// The two-phase write — the fix, and the control that makes it a claim          //
// --------------------------------------------------------------------------- //

/// A golden whose "run" produces something no validator can accept.
fn produces_garbage() -> String {
    "this is not a snapshot".to_string()
}

/// ⚠⚠ **Nothing is written unless everything validated** — the defect the Python original
/// shipped, stated as a test rather than as a paragraph.
///
/// `regen_goldens_from_rust.py` validated inline, inside the write loop, so a bad candidate
/// aborted with earlier goldens **already rewritten**. That abort was reachable and not
/// hypothetical: `sealed_energy_drift_summary.json` has had no `version` key since C5, and
/// the validator raised on a missing one. Nobody saw it because nobody ran `--write`.
///
/// The discriminating setup is a selection where a **changed** golden comes *before* a
/// failing one. Under an inline implementation the first is on disk by the time the second
/// raises; under the two-phase one, neither is touched. It runs against a temp directory
/// because the alternative is rewriting a freeze contract's values to find out whether they
/// get rewritten.
#[test]
fn a_validation_failure_leaves_every_earlier_golden_untouched() {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("regen_two_phase");
    std::fs::create_dir_all(&dir).expect("temp dir");

    // A real golden, seeded with content that does NOT match its run — so phase 2 would
    // rewrite it the moment it got there.
    let real = all()
        .into_iter()
        .find(|g| g.name == "season_euler_state.json")
        .expect("the season golden is on the roster");
    let stale = "{\"deliberately\": \"stale\"}\n";
    std::fs::write(dir.join(real.name), stale).expect("seed");

    // ⚠ `'static` because the roster is: the tool holds `&'static Golden`, so a fabricated
    // one has to be a const to be passed at all. That is the type system carrying the
    // "the roster is compiled in" rule into the test.
    const BROKEN: Golden = Golden {
        name: "not_a_real_golden.json",
        run: produces_garbage,
        numerics: Numerics::PureArithmetic,
        cost: Cost::Cheap,
        shape: Shape::StateSnapshot,
    };

    let err = regenerate_in(&dir, &[real, &BROKEN], true)
        .expect_err("a candidate that does not validate must abort the whole run");
    assert!(err.contains("REFUSED"), "{err}");

    assert_eq!(
        std::fs::read_to_string(dir.join(real.name)).expect("still there"),
        stale,
        "⚠ the golden ahead of the failing one was REWRITTEN before the abort — that is \
         the Python original's defect, back. Validation must complete for every candidate \
         before any byte is written."
    );

    // The control: this test only means something if the good golden really would have
    // been written. Run it alone and it changes.
    let outcomes = regenerate_in(&dir, &[real], true).expect("the real golden validates");
    assert!(
        outcomes[0].changed && outcomes[0].written,
        "the seeded content must differ from the run, or the assertion above passes \
         because nothing would have been written either way"
    );
    assert_ne!(
        std::fs::read_to_string(dir.join(real.name)).expect("still there"),
        stale
    );
}

/// ⚠⚠ A last-bit difference on a transcendental golden is `ulp-only`, never `CHANGED`,
/// and is not rewritten even under `--write`.
///
/// Measured before this test existed (2026-09-02, Linux): the byte-comparing tool reported
/// eleven of nineteen goldens `CHANGED` on the untouched tree — the UCRT-minted
/// transcendental set, last-ULP different under glibc — while `golden_regression.rs` was
/// green. The tool now reaches the gate's verdict. This seeds a transcendental golden with
/// its own fresh bytes minus one bit in one float leaf, which is exactly the off-platform
/// state, and asserts the classification and the no-write in one run.
///
/// ⚠ `cfg(not(windows))`: on the generation platform the gate accepts nothing less than
/// byte-exact, so there the same seed is correctly `CHANGED` — the classification is a
/// property of the platform policy, not of the bytes.
#[cfg(not(windows))]
#[test]
fn a_last_bit_difference_off_platform_is_ulp_only_and_is_never_rewritten() {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("regen_ulp_only");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let real = all()
        .into_iter()
        .find(|g| g.name == "season_euler_state.json")
        .expect("on the roster");
    assert_eq!(
        real.numerics,
        domains::goldens::Numerics::Transcendental,
        "the seed must be a transcendental golden or the structural path is unreachable"
    );
    let fresh = (real.run)();
    // Flip the last hex digit of the first float leaf: `"0x1.…7p+3"` → `…6p+3` or `…8p+3`.
    let at = fresh
        .find("p+")
        .or_else(|| fresh.find("p-"))
        .expect("a hex-float leaf");
    let mut bytes = fresh.clone().into_bytes();
    bytes[at - 1] = if bytes[at - 1] == b'0' { b'1' } else { b'0' };
    let perturbed = String::from_utf8(bytes).expect("still utf-8");
    assert_ne!(perturbed, fresh, "the perturbation must change a byte");
    std::fs::write(dir.join(real.name), &perturbed).expect("seed");

    let outcomes = regenerate_in(&dir, &[real], true).expect("validates");
    assert!(
        outcomes[0].ulp_only,
        "a last-bit difference off-platform is ulp-only"
    );
    assert!(!outcomes[0].changed, "…and is not a change");
    assert!(
        !outcomes[0].written,
        "…and is not rewritten, even under --write"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join(real.name)).expect("still there"),
        perturbed,
        "the generation platform's bytes must survive a --write from another platform"
    );

    let text = summary(
        &Request {
            write: true,
            only: None,
        },
        &outcomes,
    );
    assert!(text.contains("0 rewritten"), "{text}");
    assert!(text.contains("1 differ only in the last bits"), "{text}");

    // The control: the same seed with a REAL difference (a non-float leaf) is CHANGED.
    let real_change = fresh.replacen("\"version\"", "\"versoin\"", 1);
    assert_ne!(
        real_change, fresh,
        "the golden must carry a version key to perturb"
    );
    std::fs::write(dir.join(real.name), &real_change).expect("seed");
    let outcomes = regenerate_in(&dir, &[real], false).expect("validates");
    assert!(outcomes[0].changed && !outcomes[0].ulp_only);
}

/// ⚠ The report path writes nothing — the default, and the one a mistake here inverts.
#[test]
fn a_report_run_writes_nothing_even_when_the_golden_moved() {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("regen_report_only");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let real = all()
        .into_iter()
        .find(|g| g.name == "season_euler_state.json")
        .expect("on the roster");
    let stale = "{\"deliberately\": \"stale\"}\n";
    std::fs::write(dir.join(real.name), stale).expect("seed");

    let outcomes = regenerate_in(&dir, &[real], false).expect("validates");
    assert!(outcomes[0].changed, "the seed must differ from the run");
    assert!(!outcomes[0].written);
    assert_eq!(
        std::fs::read_to_string(dir.join(real.name)).expect("still there"),
        stale,
        "reporting is the default and it must never write"
    );
}
