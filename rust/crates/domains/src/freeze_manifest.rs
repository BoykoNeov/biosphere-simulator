//! Dump the **biosphere** port's half of the freeze manifest as JSON — slices 3 and 6 of
//! the reference flip (`docs/plans/post-roadmap-reference-flip.md`).
//!
//! ⚠⚠ **This program's standing changed in slice 6, and the change is the point.** In
//! slice 3 it was a *witness*: it dumped the flow/aux inventory and a Python gate checked
//! it against a manifest Python had generated, so the two sides had independent origins
//! and a disagreement was a finding. Since slice 6 the manifest is **generated from this
//! output** — `tests/test_freeze_manifest.py::_build_manifest()` shells this example and
//! splices the keys below in. So this is now the *producer* of the reference's half of
//! `docs/biosphere-reference.manifest.json`, and the Python-side helpers that used to
//! produce those keys (`_flow_set()`, `_aux_set()`, `_light_path_fingerprint()`, the
//! horizon constants) have become **conformance checks** on the checker.
//!
//! ⚠ **Derived, never hand-listed — and that is the whole point of the program.** No
//! roster of names appears here: the sets come from walking the built registries, so a
//! flow added to a compartment builder but wired into no golden still shows up. A version
//! of this file that printed a literal list would pass the gate while proving nothing.
//!
//! ## What is emitted, and what is deliberately NOT
//!
//! * `flow_set` / `aux_set` — the union of `Flow::type_name()` / `AuxProcess::type_name()`
//!   over the four canonical builds.
//! * `light_path_samples` — the within-day PAR **shape**, sampled on a fixed grid and
//!   written as hex-float text. The manifest stores a sha-256 *of these strings*; the
//!   hashing is left to Python because it is pure formatting and this crate has no digest
//!   dependency. ⚠ The sampling grid (three day lengths × the day's quarters) is
//!   necessarily written out on both sides — Python must sample the same grid to check
//!   itself against the reference. That is a duplicated literal whose *disagreement is
//!   red in both directions*, which is the only tolerable kind: change either grid and
//!   the fingerprints stop matching.
//! * `horizons` — the run lengths, as the port's own constants.
//! * `locked_dt_days` — ⚠ **emitted for CHECKING and never spliced into the manifest.**
//!   `integrator` and `dt_days` are the two deliberately hand-written literals of the
//!   biosphere contract (`docs/plans/post-roadmap-reference-flip.md` §2b): a manifest
//!   field that imports its own value from the code auto-follows the code, which is the
//!   opposite of a freeze, and moving the step on 2026-08-14 became a deliberate ceremony
//!   only because that literal went red. Re-anchoring must not quietly undo that. What
//!   slice 6 *adds* is the missing half — the frozen literal is now compared against the
//!   **reference tree's** constant (`test_the_locked_dt_matches_the_reference_tree`), so
//!   moving Rust's step without the ceremony is red instead of silent.
//! * ⚠ **`integrator` is deliberately NOT emitted, even though `dt_days` is.** The two
//!   literals are not symmetric: `BIO_DT` is a real constant in this tree, while the
//!   scheme is selected inline by each run helper and has no importable name on *either*
//!   side — the Python module has said so since P4.3. A `"EulerIntegrator"` string typed
//!   into this file would be a second hand-written literal checked against the first,
//!   which reads like a gate and is not one. It stays what it has always been: documented
//!   in the manifest, enforced by the goldens (an RK4 switch moves every one).
//! * `weather_sha256` — ⚠ **emitted for CHECKING and never spliced, like
//!   `locked_dt_days`. New in slice C9.** The manifest's `forcing/weather_sha256` stays
//!   Python-authored, and the reason is not inertia: since C9 the reference reads the
//!   weather fixture directly (`biosphere::weather`), but it reads it through a
//!   compile-time `include_str!`, so it knows the *bytes* and not the *name*. Authority
//!   over `forcing/weather_fixture` would therefore be a hand-typed duplicate of the
//!   include path — a literal dressed as a derivation — and splitting the pair (hash
//!   re-anchored, name not) would manufacture exactly the mixed authority slice 6 exists
//!   to record. What this key buys instead is the check that did not exist: the checker
//!   hashes the file it finds **on disk**, this hashes what the reference **compiled
//!   in**, and `test_the_weather_hash_matches_the_reference_tree` fails if they ever
//!   stop being the same bytes. That is the real hazard, and it is scheduled: the
//!   relocation slice moves this fixture out of `tests/`, and a move that updates one
//!   side's path and not the other's is now red instead of silent.
//! * `science_bands` / `liveness_floors` — the **science-gate census**, `scenario ->
//!   [claim]`, from [`domains::biosphere::science_gates::GATES`]. **New in slice C4, and
//!   it is the largest single block of the biosphere contract to change hands.**
//!
//!   ⚠⚠ Until C4 this file's own header and `test_freeze_manifest.py`'s authority note
//!   both said these fields could never arrive this way — *"a static AST census of
//!   science_gate markers on pytest functions. There is no Rust referent and there cannot
//!   be."* That was true of pytest markers and false of the claim: the census's job is to
//!   be **derived from the tree rather than hand-listed**, and Rust reaches it by making
//!   the declaration and the `#[test]` one thing instead of by parsing anything. An
//!   unexercised row is now a compile error rather than something a meta-test hunts.
//!
//!   ⚠ **Only 13 of the 15 gates are here.** `crew_mission` and `sealed_station` are
//!   *station*-manifest keys whose referents the reference does not carry yet; they are
//!   C4b, with their own ceremony. Emitting them from this program would file two station
//!   claims through the biosphere's producer.
//!
//!   ⚠ The **roster** is not emitted and does not move: which scenarios get a key (and
//!   which get an explicitly empty list, saying "measured, none") is the manifest's own
//!   hand-authored scenario set, and the checker still owns it. What moved is the CLAIMS.
//!
//! * `param_files` — the frozen param-file census, `filename -> newline-normalized
//!   sha-256`, from [`domains::biosphere::params::param_files`]. **New in slice C8, and it
//!   inverts what this file said for three slices.**
//!
//!   ⚠⚠ **What re-anchored is the RULE, not the number.** Both sides digest the same bytes,
//!   so all 15 recorded hashes are author-neutral by construction and the re-anchoring moved
//!   none of them (measured before it was done). What moved is (a) the **census** — this is
//!   the set the reference *loads*, a compile-time `include_str!` list, where Python's rule
//!   was a non-recursive glob of a package directory minus `demo.yaml`, so a file wired into
//!   no loader now drops out of the manifest instead of staying in it; and (b) the
//!   **normalization** — `config::provenance`, whose narrow carriage-return rule is kept
//!   from being able to disagree with Python's broader `splitlines` by a gate rather than
//!   by observation.
//!
//!   ⚠ The three slices before this one recorded, correctly for their day, that the port had
//!   **no referent** for this key: it read `biosphere_params.txt`, a table generated by the
//!   Python loaders whose `photo.` / `pheno.` / `vern.` prefixes were the generator's naming
//!   and not filenames (three come out of the single `phenology.yaml`). Slice C1 gave the
//!   reference the YAML loaders, which is what made a referent exist at all. That table is
//!   still in the tree — as C1's *control*, not as an input.

// ⚠⚠ **RELOCATED from `examples/dump_biosphere_inventory.rs` by Stage-3 slice S2** (the reference flip,
// plan §5u). It moved for one structural reason: an `examples/` program is a **binary
// target**, so no integration test can call into it — which is why the byte-for-byte gate
// on the committed manifest was a *Python* program shelling out to `cargo run`, and why
// retiring the checker would have taken the gate with it (FINDING 2's first entry).
//
// The move is deliberately a **relocation, not a rewrite**: the code below is the example's
// verbatim, so the emitted manifest bytes cannot shift. The example keeps only its argument
// parsing and calls in here. `tests/manifest_writer.rs` now `include_str!`s THIS file for
// its anti-derived-literal greps, and compares `manifest_text()` against the committed
// contract.

/// The manifest exactly as it is serialized to disk — the byte gate's subject.
///
/// ⚠ One serialization, three callers: [`write_manifest`] writes it, the byte gate compares
/// it, and nothing re-derives it. C7's own lesson on the inventory walk applies again here —
/// *sharing makes the drift impossible instead of merely detectable*.
pub fn manifest_text() -> String {
    dumps(&manifest())
}
use config::canonical_json::{dumps, Json};
use config::provenance::{normalized_sha256, sha256_hex};
use crate::biosphere::light_path::half_sine_window_mean;
use crate::biosphere::science_gates::{ScienceGate, GATES, LIVENESS_FLOORS, SCIENCE_BANDS};
use crate::biosphere::system::{
    build_season, consumer_chamber_scenario, perennial_chamber_scenario, sealed_chamber_scenario,
    SeasonScenario, DEFAULT_SCENARIO,
};
use crate::biosphere::{
    BIO_DT, CONSUMER_CHAMBER_YEARS, LONG_HORIZON_YEARS, PERENNIAL_CHAMBER_YEARS,
    SEALED_CHAMBER_YEARS,
};
use simcore::hexfloat;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The four canonical builds — the same union the Python manifest is derived from
/// (`_canonical_registries()`): the open field carries the boundary-atmosphere producer
/// flows, the sealed chambers add the decomposer / water-cycle / consumer ones.
fn canonical() -> Vec<SeasonScenario> {
    vec![
        DEFAULT_SCENARIO,
        sealed_chamber_scenario(),
        perennial_chamber_scenario(),
        consumer_chamber_scenario(),
    ]
}

fn json_array(names: &BTreeSet<&str>) -> String {
    let items: Vec<String> = names.iter().map(|n| format!("\"{n}\"")).collect();
    format!("[{}]", items.join(", "))
}

/// The within-day PAR shape, sampled on the fingerprint's fixed grid.
///
/// Three day lengths × the four quarters of the day at the shipped step, each rendered
/// with the same hex-float writer the goldens use — so the record is exact rather than
/// tolerance-bound and moves on any change to the shape, including one that preserves the
/// day's dose. `daytime_mean_par = 400` is an arbitrary non-zero scale; the fingerprint is
/// about the *distribution*, and a zero scale would collapse every sample to `0x0.0p+0`.
fn raw_light_path_samples() -> Vec<String> {
    let mut samples = Vec::new();
    for daylength_h in [8.0_f64, 12.0, 16.0] {
        for k in 0..4 {
            let mean =
                half_sine_window_mean(f64::from(k) * 0.25, 0.25, 400.0, daylength_h * 3600.0)
                    .expect("the fingerprint grid lies inside one day");
            samples.push(hexfloat::format(mean));
        }
    }
    samples
}

/// The same grid, each sample quoted as a JSON string — the dump's array.
///
/// ⚠ Split from [`raw_light_path_samples`] in slice C7 so the *dump* (which hands the
/// samples to the checker to hash) and the *manifest writer* (which hashes them here)
/// read one grid. A second copy of these three day lengths would be the duplicated
/// literal the module header tolerates only across the port boundary, where a
/// disagreement is red in both directions.
fn light_path_samples() -> Vec<String> {
    raw_light_path_samples()
        .iter()
        .map(|s| format!("\"{s}\""))
        .collect()
}

/// Minimal JSON string escaping — the only characters JSON requires escaped, plus the
/// control range. The frozen claim strings are prose (they carry `⚠`, `Γ`, em dashes and
/// `τ`), which JSON takes as raw UTF-8; the checker's own writer re-escapes to ASCII when
/// it serializes the manifest, so nothing here needs to.
fn json_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// One field of the science-gate census as `scenario -> [claim]`.
///
/// ⚠ Sorted by [`ScienceGate`]'s `Ord` — `(scenario, field, quantity, bound, source,
/// locus)` — because that is the order the Python census produced (its dataclass is
/// `order=True` over the same fields in the same sequence) and the manifest is a byte
/// comparison. Rust orders `&str` by UTF-8 bytes and Python by code points, which agree.
///
/// ⚠ Scenarios with no gate are NOT emitted as empty lists here. The distinction between
/// an absent key and a deliberately-empty one is the manifest's, and the roster it is
/// taken over is hand-authored on the checker's side; inventing keys here would be this
/// program claiming authority over a set it cannot see.
fn census(field: &str) -> String {
    let mut gates: Vec<&ScienceGate> = GATES.iter().filter(|g| g.field == field).collect();
    gates.sort();
    // Tier-0 sanity on the program itself, the same standing as the empty-inventory
    // asserts below: since slice 6 the manifest is GENERATED from this output, so an
    // empty census here would be written INTO the frozen contract by a regeneration run
    // rather than merely compared against it.
    assert!(!gates.is_empty(), "no science gates for field {field}");
    let mut out = String::from("{");
    let mut current: Option<&str> = None;
    for gate in gates {
        if current != Some(gate.scenario) {
            if current.is_some() {
                out.push_str("], ");
            }
            out.push_str(&format!("{}: [", json_string(gate.scenario)));
            current = Some(gate.scenario);
        } else {
            out.push_str(", ");
        }
        out.push_str(&format!(
            "{{\"bound\": {}, \"locus\": {}, \"quantity\": {}, \"source\": {}}}",
            json_string(gate.bound),
            json_string(gate.locus),
            json_string(gate.quantity),
            json_string(gate.source),
        ));
    }
    if current.is_some() {
        out.push(']');
    }
    out.push('}');
    out
}

/// The fixture's filename, as the manifest records it.
///
/// ⚠ A hand literal, and slice C9 is why: the reference reads the fixture through a
/// compile-time `include_str!`, so it knows the **bytes** and not the **name**. A name
/// derived here would be a hand-typed duplicate of the include path — a literal dressed
/// as a derivation. The hash beside it *is* derived, from those bytes.
const WEATHER_FIXTURE_NAME: &str = "winter_wheat_weather.json";


/// The flow and aux inventories, walked from the four canonical builds.
///
/// ⚠⚠ **ONE walk, read by both halves of this program** — the dump and the manifest
/// writer. C7's first draft had the writer re-walk the registries, which put two
/// derivations of the same sets in one file with only `test_inventory_parity.py` (dump
/// against the *committed* manifest) able to notice them drifting. Sharing makes the
/// drift impossible instead of detectable, and costs that gate nothing: its subject is
/// staleness — the live tree against the frozen file — not one code path against another.
///
/// Tier-0 sanity: an inventory that came back empty would compare against a non-empty
/// manifest and fail loudly, but failing *here* names the cause (a build that wired
/// nothing) instead of reporting 23 missing flows. ⚠ Since slice 6 this matters more than
/// it did, and since C7 more again: an empty set here is *written into* the frozen
/// contract by a regeneration run, not merely compared against it.
fn inventory() -> (BTreeSet<&'static str>, BTreeSet<&'static str>) {
    let mut flows: BTreeSet<&str> = BTreeSet::new();
    let mut aux: BTreeSet<&str> = BTreeSet::new();
    for scenario in canonical() {
        let (_, registry) = build_season(&scenario).expect("canonical biosphere build");
        flows.extend(registry.flows().iter().map(|f| f.type_name()));
        aux.extend(registry.aux_processes().iter().map(|a| a.type_name()));
    }
    assert!(
        !flows.is_empty(),
        "canonical biosphere builds wired no flows"
    );
    assert!(
        !aux.is_empty(),
        "canonical biosphere builds wired no aux processes"
    );
    (flows, aux)
}

/// The reference's half of the manifest, as JSON on stdout — the checker's input since
/// slice 3, and still the *checking* surface after C7 moved the *writing*.
///
/// ⚠ Deliberately unchanged by C7, raw UTF-8 and all. The manifest is now written
/// directly (no pipe), but `tests/crossport/test_inventory_parity.py` still reads this,
/// and the encoding pin it grew after slice C4's mojibake is a control that only has
/// teeth while there is non-ASCII to mangle.
pub fn dump() {
    let (flows, aux) = inventory();

    println!("{{");
    println!("  \"aux_set\": {},", json_array(&aux));
    println!("  \"flow_set\": {},", json_array(&flows));
    println!("  \"horizons\": {{");
    println!("    \"consumer_chamber_years\": {CONSUMER_CHAMBER_YEARS},");
    println!("    \"long_horizon_years\": {LONG_HORIZON_YEARS},");
    println!("    \"perennial_chamber_years\": {PERENNIAL_CHAMBER_YEARS},");
    println!("    \"sealed_chamber_years\": {SEALED_CHAMBER_YEARS}");
    println!("  }},");
    println!(
        "  \"light_path_samples\": [{}],",
        light_path_samples().join(", ")
    );
    println!("  \"param_files\": {{");
    let files = crate::biosphere::params::param_files();
    // Tier-0 sanity on the program itself, the same standing as the empty-inventory asserts
    // above and for the same slice-6 reason: since the manifest is GENERATED from this
    // output, an empty census here would be written into the frozen contract by a
    // regeneration run rather than merely compared against it.
    assert_eq!(
        files.len(),
        15,
        "the frozen biosphere param census is 15 files, got {}",
        files.len()
    );
    for (i, (name, text)) in files.iter().enumerate() {
        let comma = if i + 1 == files.len() { "" } else { "," };
        println!("    \"{name}\": \"{}\"{comma}", normalized_sha256(text));
    }
    println!("  }},");
    println!("  \"liveness_floors\": {},", census(LIVENESS_FLOORS));
    println!("  \"science_bands\": {},", census(SCIENCE_BANDS));
    println!("  \"locked_dt_days\": {BIO_DT:?},");
    println!(
        "  \"weather_sha256\": \"{}\"",
        normalized_sha256(crate::biosphere::weather::WEATHER_FIXTURE)
    );
    println!("}}");
}

// ==========================================================================
// The manifest writer — slice C7 of the reference flip
// ==========================================================================
//
// ⚠⚠ **What C7 moves is the WRITER, not the authority.** Until this slice
// `tests/test_freeze_manifest.py::_build_manifest()` assembled the file: it shelled the
// dump above, spliced the reference's keys into its own, and serialized the result. The
// manifest was therefore *authored* by the reference and *written* by the checker — a
// Python-shaped hole in the middle of a contract that says Rust is the reference.
//
// ⚠ **`_authority` records who produced the VALUE, not who ran the digest or the writer,
// and that is why this move is authority-neutral.** The precedent predates C7 and is in
// the table below: `scenarios/*/golden_sha256` has read `rust` since slice 4 while
// *Python* computed the digest, on the ground that the golden is the reference's own
// output. The same reading applies in reverse now — this program hashes
// `drift_summary.json`, whose fold Python still authors, without becoming its author.
//
// ⚠ **The step is the trap.** `dt_days` is one of the two deliberately anti-derived
// literals: a manifest that read `BIO_DT` would auto-follow a step change, which is the
// opposite of a freeze, and the 2026-08-14 step move became a ceremony only because this
// literal went red. C7 moves the writer *into the crate that owns `BIO_DT`*, where
// splicing it in is a one-character mistake. Two things stop it: the literal is written
// as **text** (`Json::num("0.25")` — `config::canonical_json::Json::Number` takes no
// `f64`, deliberately), and `test_the_locked_dt_matches_the_reference_tree` still checks
// the frozen literal against the constant across the port boundary.
//
// ⚠ **No pipe.** The file is written with `std::fs::write`, not printed for the checker
// to capture. Slice C4's regeneration froze cp1252-mangled prose into this contract with
// every gate green, because a `subprocess` pipe decoded UTF-8 with the Windows locale and
// *both* sides were mangled identically. Writing the file here deletes that class rather
// than inheriting it — and the bytes are pure ASCII besides, because the serializer
// escapes as Python's `ensure_ascii=True` does.

/// A scenario's horizon: a named constant of this tree, or a literal.
enum Years {
    /// Names one of the reference's own horizon constants — the manifest's `years` is a
    /// property of the reference, not of the checker (slice 6).
    Named(&'static str),
    /// ⚠ `open_season` only. A single season has no named constant on *either* side — it
    /// is what "one season" means — and inventing one to make the table uniform would be
    /// a manifest field with a made-up referent.
    Literal(i64),
}

/// The frozen scenario roster: `(name, human label, horizon, golden filename)`.
///
/// ⚠ **Hand-authored, and it stays that way.** The label is documentation and the roster
/// is a *choice* about which runs the contract freezes, not something derivable from the
/// tree — a scenario the reference can build but that no golden pins would otherwise walk
/// into the manifest on its own. The `_authority` table marks `scenarios/*/scenario` and
/// `scenarios/*/golden` `hand` for exactly this reason.
/// The frozen golden filenames this contract pins, in roster order.
///
/// Exposed for the cross-port tolerance gate (`tiers.rs`), which must check that the
/// tolerance table classifies **exactly** the frozen set — no orphan row, no unclassified
/// golden. Reading the roster from the manifest's own source is the point: the alternative,
/// parsing the committed `.manifest.json` out of `docs/`, would make an engine crate depend
/// on a document and give the check a second source of truth.
pub fn frozen_goldens() -> Vec<&'static str> {
    SCENARIOS.iter().map(|(_, _, _, golden)| *golden).collect()
}

const SCENARIOS: [(&str, &str, Years, &str); 7] = [
    (
        "open_season",
        "DEFAULT_SCENARIO (open field)",
        Years::Literal(1),
        "season_euler_state.json",
    ),
    (
        "sealed_chamber",
        "SEALED_CHAMBER_SCENARIO",
        Years::Named("sealed_chamber_years"),
        "sealed_chamber_state.json",
    ),
    (
        "perennial_chamber",
        "PERENNIAL_CHAMBER_SCENARIO",
        Years::Named("perennial_chamber_years"),
        "perennial_chamber_state.json",
    ),
    (
        "consumer_chamber",
        "CONSUMER_CHAMBER_SCENARIO",
        Years::Named("consumer_chamber_years"),
        "consumer_chamber_state.json",
    ),
    (
        "perennial_long_horizon",
        "PERENNIAL_CHAMBER_SCENARIO",
        Years::Named("long_horizon_years"),
        "perennial_long_horizon_state.json",
    ),
    (
        "consumer_long_horizon",
        "CONSUMER_CHAMBER_SCENARIO",
        Years::Named("long_horizon_years"),
        "consumer_long_horizon_state.json",
    ),
    (
        "drift_summary",
        "PERENNIAL_CHAMBER_SCENARIO + CONSUMER_CHAMBER_SCENARIO (stability signature)",
        Years::Named("long_horizon_years"),
        "drift_summary.json",
    ),
];

/// The reference's horizon constants under the names the manifest uses.
fn horizon(name: &str) -> i64 {
    let years = match name {
        "sealed_chamber_years" => SEALED_CHAMBER_YEARS,
        "perennial_chamber_years" => PERENNIAL_CHAMBER_YEARS,
        "consumer_chamber_years" => CONSUMER_CHAMBER_YEARS,
        "long_horizon_years" => LONG_HORIZON_YEARS,
        other => panic!("no horizon constant named {other:?}"),
    };
    i64::try_from(years).expect("a horizon fits in i64")
}

/// The repository root, from this crate's own location.
///
/// ⚠ It used to be the same reach-out as the `include_str!`s this crate carried — the
/// goldens lived under `tests/` in the Python tree. Stage-3 slice S1 moved them to
/// `rust/data/golden/`, so this now climbs to the repo root only to come back down inside
/// `rust/`; the docs directory is the one thing still genuinely outside. What it must not
/// become is a *hidden* reach-out — hence one function, named, with the hop count in one
/// place.
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // rust/crates/
        .and_then(Path::parent) // rust/
        .and_then(Path::parent) // the repo root
        .expect("the crate sits three levels below the repo root")
        .to_path_buf()
}

/// sha-256 over newline-normalized file content — the provenance rule, applied to a file
/// this program did not compile in.
fn file_sha256(path: &Path) -> String {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read {} for hashing: {e}", path.display()));
    assert!(
        config::provenance::contains_exotic_line_separator(&text).is_none(),
        "{} carries a line separator the narrow normalization rule does not handle — \
         see config::provenance",
        path.display()
    );
    normalized_sha256(&text)
}

/// One science-gate field as `scenario -> [claim]`, filed under the roster above.
///
/// ⚠ Every roster scenario gets a key, including the ones with no gate: an empty list
/// says *measured, none* and an absent key says nothing, and a reader reaching for
/// `.get(name, [])` cannot tell them apart. `drift_summary` is the case that forces it.
///
/// ⚠ A gate naming a scenario outside the roster **panics** rather than being filtered
/// away. Both this manifest and the station's filter by scenario, so a typo or a gate on
/// authored content would otherwise be dropped by both in silence — the filter looking
/// exactly like a clean result.
fn census_json(field: &str) -> Json {
    let mut by_scenario: Vec<(String, Vec<Json>)> = SCENARIOS
        .iter()
        .map(|(name, ..)| ((*name).to_string(), Vec::new()))
        .collect();
    let mut gates: Vec<&ScienceGate> = GATES.iter().filter(|g| g.field == field).collect();
    gates.sort();
    assert!(!gates.is_empty(), "no science gates for field {field}");
    for gate in gates {
        let slot = by_scenario
            .iter_mut()
            .find(|(name, _)| name == gate.scenario)
            .unwrap_or_else(|| {
                panic!(
                    "a {field} gate names scenario {:?}, which is not in this manifest's \
                     roster. Either the roster moved or the gate names the wrong \
                     scenario — both are decisions, and neither may be resolved by \
                     dropping the claim.",
                    gate.scenario
                )
            });
        slot.1.push(Json::obj([
            ("bound", Json::s(gate.bound)),
            ("locus", Json::s(gate.locus)),
            ("quantity", Json::s(gate.quantity)),
            ("source", Json::s(gate.source)),
        ]));
    }
    Json::obj(
        by_scenario
            .into_iter()
            .map(|(name, claims)| (name, Json::Array(claims))),
    )
}

/// The `_authority` block, as `(manifest path, side, why)`.
///
/// ⚠ `side` is who produced the **value** — not who hashed it and not who wrote the file;
/// see the header above. A `python` row is a key still authored by the retiring checker,
/// which under C is a queue and not a classification.
///
/// ⚠ The prose was moved here from `tests/test_freeze_manifest.py::_AUTHORITY`
/// mechanically (generated from the committed manifest and diffed), not retyped: it is
/// frozen text, and a re-anchoring that quietly reworded the contract would be a value
/// change wearing a refactor's clothes.
const AUTHORITY: [(&str, &str, &str); 19] = [
    (
        "_comment",
        "hand",
        "prose header",
    ),
    (
        "aux_set",
        "rust",
        "the same walk over AuxProcess::type_name()",
    ),
    (
        "dt_days",
        "hand",
        "the second anti-derived literal: a manifest that imported BIO_DT would auto-follow a step change, which is the opposite of a freeze — the 2026-08-14 step move became a ceremony only because this literal went red.Slice 6 added the missing half instead: the crossport gate checks it against the REFERENCE tree's BIO_DT, so moving Rust's step without the ceremony is red rather than silent.",
    ),
    (
        "flow_set",
        "rust",
        "the union of Flow::type_name() over the four canonical builds in the reference tree — derived from built registries, never hand-listed",
    ),
    (
        "forcing/light_path",
        "rust",
        "sha-256 of the reference tree's own light-path samples. Measured on 2026-08-16 before re-anchoring: Rust reproduces all twelve hex-float samples byte for byte, so the hash did not move — this key is gated exactly, not tolerance-bound, and could not have been re-anchored on a prediction.",
    ),
    (
        "forcing/weather_fixture",
        "hand",
        "⚠ RECLASSIFIED IN SLICE C7, AND THE SIDE MOVED python → hand WITHOUT THE VALUE MOVING. C9 gave the reference the fixture itself, but through a compile-time include_str!, so it knows the BYTES and not the NAME: a filename derived here would be a hand-typed duplicate of the include path, a literal dressed as a derivation. While the checker still wrote this file, 'python' was a fair description of who typed the name; C7 moved the writer, and leaving it 'python' would name a producer that no longer touches the file. It belongs in the same category as integrator — a name with no importable referent on EITHER side. ⚠ STAGE-3 SLICE S1 MET THE CONDITION THIS ENTRY NAMED (2026-08-18): the fixture left tests/oracle/ for crates/domains/data/, a directory the reference owns and can read at runtime, exactly as C8's param census already does. The derivation is therefore now BUILDABLE and deliberately NOT built here — deriving the name flips this side hand → rust, which is a re-anchoring, and S1's whole claim is that it moves data and not authority. Taking both in one batch would make the byte-neutrality claim unfalsifiable. The successor is named rather than left implicit: derive weather_fixture from the data directory, with a control that breaks the derivation and reddens a Rust test.",
    ),
    (
        "forcing/weather_sha256",
        "rust",
        "⚠ RE-ANCHORED IN SLICE C7, AND THE VALUE COULD NOT MOVE. The reference has emitted this hash since C9 — of the fixture text it COMPILED IN — for checking only, while the manifest recorded the checker's hash of the same file on disk, and test_the_weather_hash_matches_the_reference_tree has asserted the two are the same bytes ever since. So this re-anchors between two sides a gate already holds equal, which is both why it is free and why C9 declined to do it: C9 would have had to split the pair, moving the hash while the NAME beside it had no Rust referent. C7 resolves the split the other way — the name is reclassified hand, because it is nobody's derivation, and the hash goes to the side that derives it.",
    ),
    (
        "frozen_at_phase",
        "hand",
        "the phase this surface froze at",
    ),
    (
        "integrator",
        "hand",
        "one of the two deliberate anti-derived literals. Unlike dt_days it has no importable constant on EITHER side — each run helper selects the scheme inline — so it is documented here and enforced by the goldens (an RK4 switch moves every one). A literal typed into the Rust dump to make the pair symmetric would read like a gate and be none.",
    ),
    (
        "liveness_floors/*",
        "rust",
        "the same census, for the bounds tuned to our own calibration rather than to an outside source — re-anchored with it in slice C4. ⚠ This is the family that has already been retuned twice, so it is the one the bound-literal check was written for: every numeric literal in a recorded bound must appear textually in the file its locus names, and that check now runs on BOTH sides (Rust's the_bound_literals_appear_at_their_locus and the checker's test_science_gate_bounds_name_a_literal_present_at_their_locus). ⚠ One Python test carried TWO markers through a parametrization; in the reference the row IS the test, so it became two tests with two loci. Same claims, same numbers, one more locus string.",
    ),
    (
        "long_horizon_years",
        "rust",
        "the reference tree's LONG_HORIZON_YEARS",
    ),
    (
        "param_files/*",
        "rust",
        "⚠ RE-ANCHORED IN SLICE C8, AND WHAT MOVED IS THE RULE, NOT THE NUMBER. The 23 digits across the two manifests are author-neutral by construction — both sides hash the same file with the same rule, which is why re-anchoring moved not one of them (measured first, then done). Two rules did move to the reference. (1) The CENSUS: this is now the set the reference actually LOADS (domains::biosphere::params::param_files, a compile-time include_str! list) rather than a non-recursive glob of a Python package directory minus demo.yaml — so a file wired into no loader drops OUT of the manifest instead of staying in it. The 15-of-20 rule survives with its two different exclusion reasons (four crops/potato overrides by non-recursion, demo.yaml by name), asserted Rust-side against the directory. (2) The NORMALIZATION: config::provenance, a hand-rolled sha-256 (every engine crate is zero-dep by charter) over LF-normalized text. That rule is load-bearing TODAY, not in principle: include_str! embeds the WORKING TREE, and one frozen file (senescence.yaml) is CRLF on the dev box while the git index is LF everywhere — so the un-normalized hash would differ between that box and Linux CI. Python's helpers are retained as conformance checks on the checker. Prerequisite: slice C1, which gave the reference the loaders. ⚠ STAGE-3 SLICE S1 MOVED THE GROUND UNDER THIS KEY WITHOUT MOVING THE KEY: the fifteen files (and demo.yaml, and the four crops/potato overrides) left src/domains/biosphere/params/ for crates/domains/params/biosphere/, so the census now reads a directory the reference owns instead of a Python package scheduled for deletion. Not one hash moved — the manifest is basename-keyed and the normalization is path-independent — and the 15-of-20 rule with its two different exclusions carried over verbatim BECAUSE the whole directory moved, subdirectory included. Had the potato overrides been left behind, a_recursive_walk_reddens_the_census would have gone green-by-vacuity: a control with no test to redden.",
    ),
    (
        "reference_doc",
        "hand",
        "pointer to the prose half of the contract",
    ),
    (
        "scenarios/*/golden",
        "hand",
        "the artifact's filename",
    ),
    (
        "scenarios/*/golden_sha256",
        "rust",
        "the golden is the reference tree's own output (golden_platform.RUST_AUTHORED, which this block is checked against, not restating). Unlike the other hashes here this one IS gated against the file on disk: a golden is machine-generated and its hash is newline-normalized, so 'the manifest pins bytes that exist' is a completeness claim, not the value re-assertion that param_files declines.",
    ),
    (
        "scenarios/*/scenario",
        "hand",
        "a human label for the scenario, not an identifier anything resolves",
    ),
    (
        "scenarios/*/years",
        "rust",
        "the reference tree's horizon constant",
    ),
    (
        "scenarios/drift_summary/golden_sha256",
        "python",
        "⚠ ONE RUN, TWO AUTHORS. This is drift.py's Python-side fold of the same 15-yr perennial trajectory whose final state Rust authors next door, and the two engines differ by 1 ULP on it. The fold is the artifact, and its correct reference is Python's own output — so the golden axis is not '6 Rust, 1 folded' scenario by scenario.",
    ),
    (
        "science_bands/*",
        "rust",
        "⚠⚠ RE-ANCHORED IN SLICE C4, AND THIS ENTRY USED TO SAY IT COULD NOT BE. Its own text read 'a static AST census of science_gate markers on pytest functions. There is no Rust referent and there cannot be one while the science gates are pytest-side' — true of pytest markers, false of the claim it appeared to make. The census's requirement is that it be DERIVED from the tree rather than hand-listed; Python met it by parsing decorators with ast, and the reference meets it by making the declaration and the #[test] ONE THING (the science_gates! macro in rust/crates/domains/src/biosphere/science_gates.rs emits both the roster row and the test that executes it), so an unexercised entry is a compile error rather than something a meta-test hunts textually. Together with liveness_floors this is about half the manifest by content — the single largest block of any contract to change hands. ⚠ What moved is the CLAIMS and their LOCI, not the values: the 13 quantity/bound/source strings are byte-identical to the Python census's and every gate's verdict was measured identical on both ports BEFORE the port was written (§5j). ⚠ The KEY SET is still the checker's: which scenarios get an entry, and which get an explicitly empty list meaning 'measured, none', is this manifest's own hand roster, and a Rust gate naming a scenario outside it RAISES during regeneration rather than being filtered away. ⚠ Two markers did NOT move: crew_mission and sealed_station are station-manifest keys whose referents the reference does not carry yet, and they are slice C4b.",
    ),
];

const COMMENT: &str = "Phase-4 freeze manifest (P4.3). Names the frozen biosphere reference surface. See docs/biosphere-reference.md for the freeze contract + the unfreeze discipline. Hashes are newline-normalized sha-256 PROVENANCE (value enforcement is the scenario goldens). Each key's producer and why is in _authority: this file has MIXED authority since slice 6 of the reference flip. Regenerate on a deliberate unfreeze, from rust/: cargo run --example dump_biosphere_inventory -- --write-manifest. Slice C7 moved the WRITER to the reference; tests/test_freeze_manifest.py has none and is now only a checker.";

/// The `_authority` block as JSON.
fn authority_json() -> Json {
    Json::obj(AUTHORITY.iter().map(|(path, side, why)| {
        (
            *path,
            Json::obj([("side", Json::s(*side)), ("why", Json::s(*why))]),
        )
    }))
}

/// The whole biosphere freeze manifest.
pub fn manifest() -> Json {
    let root = repo_root();
    let golden_dir = root.join("rust").join("data").join("golden");
    let (flows, aux) = inventory();

    let scenarios = Json::obj(SCENARIOS.iter().map(|(name, label, years, golden)| {
        (
            *name,
            Json::obj([
                ("golden", Json::s(*golden)),
                (
                    "golden_sha256",
                    Json::s(file_sha256(&golden_dir.join(golden))),
                ),
                ("scenario", Json::s(*label)),
                (
                    "years",
                    Json::int(match years {
                        Years::Named(key) => horizon(key),
                        Years::Literal(n) => *n,
                    }),
                ),
            ]),
        )
    }));

    let params = crate::biosphere::params::param_files();
    assert_eq!(
        params.len(),
        15,
        "the frozen biosphere param census is 15 files, got {}",
        params.len()
    );

    Json::obj([
        ("_authority", authority_json()),
        ("_comment", Json::s(COMMENT)),
        ("aux_set", Json::strs(aux.iter().copied())),
        // ⚠ TEXT, not `BIO_DT`. See the header: this is the anti-derived literal, and
        // `Json::num` takes a lexeme precisely so the constant cannot be spliced in by a
        // one-character edit.
        ("dt_days", Json::num("0.25")),
        ("flow_set", Json::strs(flows.iter().copied())),
        (
            "forcing",
            Json::obj([
                (
                    "light_path",
                    Json::s(sha256_hex(raw_light_path_samples().join("|").as_bytes())),
                ),
                ("weather_fixture", Json::s(WEATHER_FIXTURE_NAME)),
                (
                    "weather_sha256",
                    Json::s(normalized_sha256(
                        crate::biosphere::weather::WEATHER_FIXTURE,
                    )),
                ),
            ]),
        ),
        ("frozen_at_phase", Json::int(4)),
        // ⚠ The second anti-derived literal, and unlike `dt_days` it has no importable
        // constant on either side — each run helper selects the scheme inline. It is
        // documented here and enforced by the goldens (an RK4 switch moves every one).
        ("integrator", Json::s("EulerIntegrator")),
        ("liveness_floors", census_json(LIVENESS_FLOORS)),
        (
            "long_horizon_years",
            Json::int(horizon("long_horizon_years")),
        ),
        (
            "param_files",
            Json::obj(
                params
                    .iter()
                    .map(|(name, text)| (*name, Json::s(normalized_sha256(text)))),
            ),
        ),
        ("reference_doc", Json::s("docs/biosphere-reference.md")),
        ("scenarios", scenarios),
        ("science_bands", census_json(SCIENCE_BANDS)),
    ])
}

/// Write the manifest to `path`, and report what changed.
///
/// ⚠ Reports rather than asserts: this is the *regeneration* entry point, run on a
/// deliberate unfreeze, so a moved byte is the thing being reviewed. The assertion that
/// the committed file matches lives on the checking side
/// (`tests/crossport/test_manifest_writer.py`), where a stale manifest is red in CI.
pub fn write_manifest(path: &Path) {
    let text = dumps(&manifest());
    let previous = std::fs::read_to_string(path).ok();
    std::fs::write(path, text.as_bytes())
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
    match previous {
        Some(old) if old == text => eprintln!("unchanged: {}", path.display()),
        Some(_) => eprintln!("REWRITTEN (review the diff): {}", path.display()),
        None => eprintln!("created: {}", path.display()),
    }
}
