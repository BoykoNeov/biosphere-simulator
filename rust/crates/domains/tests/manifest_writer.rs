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
/// ⚠ **Re-pointed by S2**: the writer moved from `examples/` into `src/freeze_manifest.rs`
/// so the byte gate below could call it (an `examples/` program is a binary target). The
/// anchors these tests grep travelled with the code, and
/// `each_frozen_literal_is_emitted_on_exactly_one_line` is what proves the re-point landed:
/// a stale path here would find ZERO lines and the `expect` would fire, not pass quietly.
const WRITER_SOURCE: &str = include_str!("../src/freeze_manifest.rs");

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
const COMMITTED: &str = include_str!("../../../../docs/biosphere-reference.manifest.json");

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
/// S2 moved the writer into `domains::freeze_manifest`, which is what makes this callable
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
    let regenerated = domains::freeze_manifest::manifest_text();
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
        "the committed docs/biosphere-reference.manifest.json is not what the reference \
         writes today.\n\
         Two readings, and the first question decides which:\n\
         * the reference tree changed and the manifest was not regenerated — that is an \
         UNFREEZE. Follow the ceremony in the contract's own doc, then re-run the writer \
         and review the diff.\n\
         * the manifest was edited by hand — it is a generated artifact; the edit belongs \
         in the writer (crates/domains/src/freeze_manifest.rs), which is what makes it \
         reproducible.\n\
         Regenerate: cd rust && cargo run -p domains --example dump_biosphere_inventory \
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
/// file is not truncated, and it is the biosphere contract rather than a sibling.
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
        "the committed biosphere manifest read as {} bytes — the include path is wrong \
         or the file is truncated, and the byte gate above is comparing against nothing",
        COMMITTED.len()
    );
    assert!(
        COMMITTED.contains("docs/biosphere-reference.md"),
        "the loaded manifest does not name docs/biosphere-reference.md as its contract \
         doc, so this gate is pointed at a DIFFERENT freeze contract. Every manifest \
         carries an _authority block, which is why that is not the thing to check here."
    );
}

/// ⚠ **The VALUE half of the anti-derived literal** — the residue S2's enumeration found.
///
/// The greps above assert `dt_days` is *typed*, not spliced. That is a claim about the
/// source text and says nothing about whether the typed number is still **true**. The
/// byte gate cannot help either: it regenerates from the same literal, so it agrees with
/// itself. The missing claim — *the frozen literal still equals the step the reference
/// actually runs* — was `test_inventory_parity.py::test_the_locked_dt_matches_the_reference_tree`,
/// which crossed the port boundary to make it. Inside one crate it is a direct comparison.
///
/// ⚠ Deliberately NOT extended to `integrator`, and the Python original's reasoning carries
/// over unchanged: there is no importable scheme name — each run helper selects it inline —
/// so the only symmetric version would compare two hand-written literals to each other,
/// which reads like a gate and is none. The integrator stays enforced by the goldens.
#[test]
fn the_frozen_step_still_equals_the_step_the_reference_runs() {
    let frozen: f64 = "0.25".parse().expect("the frozen dt_days literal parses");
    assert_eq!(
        frozen,
        domains::biosphere::BIO_DT,
        "the reference tree runs the biosphere at dt = {}, but the frozen manifest \
         declares {frozen}. Moving the step is an UNFREEZE (docs/biosphere-reference.md) — \
         the manifest literal is hand-written precisely so that this is a ceremony rather \
         than a silent follow, and this is the assertion that makes the ceremony happen.",
        domains::biosphere::BIO_DT
    );
    // The control: the literal compared here must be the one the writer emits, or this
    // test drifts away from the manifest it is about.
    assert!(
        WRITER_SOURCE.contains("Json::num(\"0.25\")"),
        "the writer no longer emits 0.25 as dt_days, so the literal checked above is not \
         the frozen one"
    );
}

/// ⚠⚠ **The weather hash — the THIRD residue, and the one that looks subsumed but is not.**
///
/// `forcing/weather_sha256` is deliberately **not spliced** into the manifest: the writer
/// emits it for *checking* (see this module's header on C9) while the frozen value stays
/// hand-authored. So the byte gate cannot see a divergence here — it regenerates a manifest
/// whose `forcing` block it copies rather than derives, and agrees with itself. The claim
/// that the two sides read the same weather bytes was
/// `test_inventory_parity.py::test_the_weather_hash_matches_the_reference_tree`.
///
/// ⚠ **This is what keeps a reach-out `include_str!` honest.** Since C9 the reference embeds
/// the fixture at compile time, so it carries a *copy* of bytes the contract names only by
/// filename. Without this, the embedded copy could drift from the frozen hash with every
/// other gate green — and "the reference reads its own forcing data" would stop being true
/// while still being written down.
#[test]
fn the_frozen_weather_hash_is_the_fixture_the_reference_actually_embeds() {
    let frozen = COMMITTED
        .lines()
        .find(|l| l.contains("\"weather_sha256\""))
        .and_then(|l| l.split('"').nth(3).map(str::to_string))
        .expect("the manifest declares forcing/weather_sha256");
    let embedded =
        config::provenance::normalized_sha256(domains::biosphere::weather::WEATHER_FIXTURE);
    assert_eq!(
        frozen, embedded,
        "the weather fixture the reference embeds hashes to {embedded}, but the frozen \
         manifest declares {frozen}. Since C9 the reference reads the fixture through a \
         compile-time include_str!, so this is the only gate that says the embedded copy \
         is still the frozen one — the byte gate cannot see it, because weather_sha256 is \
         emitted for checking and never spliced."
    );
    // The control: the hash extracted above must be a real 64-hex digest, not a fragment
    // of some other line that happened to match.
    assert_eq!(frozen.len(), 64, "extracted {frozen:?}, which is not a sha-256");
}
