//! Dump the **station** port's half of the freeze manifest as JSON — slices 3 and 7 of the
//! reference flip (`docs/plans/post-roadmap-reference-flip.md`).
//!
//! ⚠⚠ **This program's standing changed in slice 7, exactly as the biosphere dump's did in
//! slice 6.** In slice 3 it was a *witness*: it dumped the inventory and a Python gate
//! checked it against a manifest Python had generated, so the two sides had independent
//! origins and a disagreement was a finding. Since slice 7 the manifest is **generated from
//! this output** — `tests/test_station_freeze_manifest.py::_build_manifest()` shells this
//! example and splices the keys below in. The Python-side helpers that used to produce them
//! (`_flow_set()`, `_aux_set()`, the two sealed-horizon constants) have become **conformance
//! checks on the checker**.
//!
//! ⚠ **This was the half of slice 3 nobody had measured.** Slice 2 measured the biosphere
//! names against the manifest and found them identical; the station set had only ever been
//! counted **by eye off a grep** (12 sibling `type_name` impls + 4 station ones = the
//! manifest's 16), and this repo has already priced that exact evidence as a guess — the
//! compiler found four impls the same grep missed, one slice ago.
//!
//! ## What is emitted, and what is deliberately NOT
//!
//! * `flow_set` / `aux_set` — the union of `Flow::type_name()` / `AuxProcess::type_name()`
//!   over the canonical registries selected by [`canonical`].
//! * `horizons` — the two sealed run lengths, as the port's own constants. ⚠ Note that
//!   `SEALED_ENERGY_YEARS = LONG_HORIZON_YEARS` in `scenario.rs`: after slice 7 the station
//!   and biosphere manifests are anchored to the **same** reference-side constant, so
//!   moving the decade horizon is one edit that unfreezes two contracts. A reader who
//!   assumes they are independent will predict the wrong diff.
//! * ⚠ **`integrator` is deliberately NOT emitted**, for the reason the biosphere dump gives
//!   at length: the scheme is selected inline by each run helper and has no importable name
//!   on *either* side, so a `"EulerIntegrator"` literal typed in here would be a second hand
//!   literal checked against the first — which reads like a gate and is none.
//! * ⚠ **No dt is emitted, and unlike the biosphere there is no `locked_dt_days` to check
//!   against.** The station's steps live in the manifest's `numerics_note` **prose**, which
//!   that module's own comment records as hand-maintained and ungated. The reference tree
//!   *does* have referents (`sealed_station_scenario()`'s `bio_dt` / `cabin_dt`, the energy
//!   scenario's `power_dt`), so the gate is buildable — but it needs a structured manifest
//!   key that does not exist, and adding one widens the frozen surface. That is its own
//!   unfreeze with its own ceremony, not a rider on this one. Recorded in the manifest's
//!   `_authority` entry for `numerics_note`.
//! * `param_files` — the frozen param-file census, `filename -> newline-normalized sha-256`,
//!   over the **eight** files this contract spans: five from `domains::params` (power × 2,
//!   thermal, eclss, crew) and three from `station::params`. **New in slice C8**, which
//!   reverses what this file said for two slices.
//!
//!   ⚠⚠ **What re-anchored is the RULE, not the number.** Both sides digest the same bytes,
//!   so all eight recorded hashes are author-neutral by construction and the re-anchoring
//!   moved none of them (measured before it was done). What moved is the **census** — this is
//!   the set the reference *loads* rather than a glob of Python package directories — and the
//!   **normalization**, in `config::provenance`.
//!
//!   ⚠ **No exclusion rule on this side, and the asymmetry is deliberate.** The biosphere
//!   census has two (non-recursion for four potato overrides, `demo.yaml` by name); these six
//!   directories hold nothing but frozen files. Saying so per side keeps a reader from
//!   generalising the harder rule to a place it does not apply.
//!
//! * `science_bands` / `liveness_floors` — the **science-gate census**, `scenario ->
//!   [claim]`, from [`station::science_gates::GATES`]. **New in slice C4b.**
//!
//!   ⚠⚠ The `_authority` note this key used to carry said *"There is no Rust referent
//!   and there cannot be one while the science gates are pytest-side"* — the third
//!   frozen `why` to argue against the slice that arrived (after the biosphere's
//!   identical sentence in C4 and `parity_vectors/*`'s in C7's authoring half). It was
//!   true of pytest markers and false of the claim: the census's requirement is that it
//!   be DERIVED from the tree rather than hand-listed, and the reference meets it by
//!   making the declaration and the `#[test]` one thing.
//!
//!   ⚠ Only **2** claims, and the smallness is the frozen result rather than a gap: 11
//!   of the 13 station scenarios carry no outside-sourced bound at all. The **roster** —
//!   which scenarios get a key, and which get an explicitly empty list meaning
//!   "measured, none" — is not emitted here; it is the manifest's own hand-authored
//!   scenario set and the checker still owns it. What moved is the CLAIMS.
//!
//!   ⚠ What this file said before C8 was true for its day: the port read
//!   `station_params.txt` / `sibling_params.txt`, Python-generated tables whose names carry no
//!   file prefix at all, so anything printed here would have compared Python against Python.
//!   Slice C1 gave the reference the YAML loaders, which is what created a referent.

use config::canonical_json::{dumps, Json};
use config::provenance::normalized_sha256;
use domains::biosphere::science_gates::{ScienceGate, LIVENESS_FLOORS, SCIENCE_BANDS};
use domains::crew::{build_crew, MISSION_SCENARIO};
use domains::eclss::{build_eclss, STEADY_STATE_SCENARIO};
use domains::params;
use domains::power::{build_power, BOUNDED_SOC_SCENARIO};
use domains::thermal::{build_thermal, EQUILIBRIUM_SCENARIO};
use simcore::registry::Registry;
use station::params as station_params;
use station::scenario::{sealed_station_scenario, SEALED_ENERGY_YEARS, SEALED_STATION_YEARS};
use station::science_gates::GATES;
use station::sealed::build_sealed_station;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn json_array(names: &BTreeSet<&str>) -> String {
    let items: Vec<String> = names.iter().map(|n| format!("\"{n}\"")).collect();
    format!("[{}]", items.join(", "))
}

/// The canonical station registries — a **hand-mirrored** selection, and every line of it
/// is a judgement call taken from `_station_registries()` in the Python gate. Each one is
/// load-bearing on the *set*, so a mis-mirrored line reads as a port divergence that is
/// not one:
///
/// 1. the four **standalone** sibling registries, so the stand-ins the sealed assembly
///    drops (`HeatInput`, `CrewMetabolism`, `OxygenConsumption`, `FoodMetabolism`) appear
///    at all — they are pinned only by the standalone goldens;
/// 2. `Some(self_discharge)` to `build_power` — the opt-in third flow; `None` and
///    `SelfDischarge` leaves the set;
/// 3. `with_harvest = true` on the sealed build — `false` (the Tier-2 golden scope) and
///    `Harvest` leaves the set;
/// 4. the **fast** registry, `.2` of `(state, bio_reg, fast_reg)` — the tuple shape is
///    read off `build_sealed_station`, not assumed;
/// 5. the sealed build's biosphere **slow** registry (`.1`) deliberately **excluded** — the
///    biosphere is delegated to its own manifest, and including it would leak all 23
///    biosphere flows into this set.
///
/// `close_feces` is left at `false`, which is the Python builder's own default and so
/// matches the selection. ⚠ **It wires no flow either way — measured, not read**: flipping
/// it to `true` and re-running this program produces byte-identical output. That matters
/// because the claim is one the gate cannot check: if `close_feces` *did* wire a flow, the
/// resulting divergence would read as a mistake in one of the five calls above, and the
/// hunt would start in the wrong place.
fn canonical() -> Vec<Registry> {
    let charge = params::charge();
    let self_discharge = params::self_discharge();
    let thermal = params::thermal();
    let eclss = params::eclss();
    let crew = params::crew();
    let recovery = station_params::water_recovery();
    let lamp = station_params::lamp();
    let harvest = station_params::harvest();
    let sealed = sealed_station_scenario();

    let power_reg = build_power(&charge, &BOUNDED_SOC_SCENARIO, Some(self_discharge))
        .expect("build_power")
        .1;
    let thermal_reg = build_thermal(&thermal, &EQUILIBRIUM_SCENARIO)
        .expect("build_thermal")
        .1;
    let eclss_reg = build_eclss(&eclss, &STEADY_STATE_SCENARIO)
        .expect("build_eclss")
        .1;
    let crew_reg = build_crew(&crew, &MISSION_SCENARIO).expect("build_crew").1;
    let sealed_fast_reg = build_sealed_station(
        &charge, &thermal, &crew, &eclss, &recovery, &lamp, &harvest, &sealed, true, false,
    )
    .expect("build_sealed_station")
    .2;

    vec![power_reg, thermal_reg, eclss_reg, crew_reg, sealed_fast_reg]
}

/// Minimal JSON string escaping — the only characters JSON requires escaped, plus the
/// control range.
///
/// ⚠ **A second copy of `dump_biosphere_inventory.rs`'s, and it is deliberately temporary.**
/// C4b needed the census emitted from here one commit before C7's station half lands, and
/// that slice replaces both dumps' hand-rolled JSON with `config::canonical_json::Json` —
/// so sharing it now would build a shared helper with a one-commit lifetime. Named here
/// rather than left silent: this is the copy to delete, not the one to add a third to.
///
/// The frozen claim strings are prose (they carry em dashes), which JSON takes as raw
/// UTF-8; the checker's own writer re-escapes to ASCII when it serializes the manifest,
/// so nothing here needs to.
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
/// ⚠ Sorted by `ScienceGate`'s `Ord` — `(scenario, field, quantity, bound, source,
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
    // asserts below: since slice 7 the manifest is GENERATED from this output, so an
    // empty census here would be written INTO the frozen contract by a regeneration run
    // rather than merely compared against it. ⚠ It bites harder here than on the
    // biosphere: this contract's census is two claims deep, so the gap between correct
    // and empty is two rows.
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

/// The biosphere contract this one delegates to rather than re-hashing.
///
/// ⚠ A path, not a derived value — `_authority` marks it `hand`. Its target's
/// existence is checked on the Python side (`test_manifest_named_files_exist`), and
/// since C7 what that gate actually guards is a literal in THIS file.
const BIOSPHERE_MANIFEST: &str = "docs/biosphere-reference.manifest.json";

const AUTHORITY: [(&str, &str, &str); 16] = [
    (
        "_comment",
        "hand",
        "prose header",
    ),
    (
        "aux_set",
        "rust",
        "the same walk over AuxProcess::type_name(). ⚠ Legitimately EMPTY (the siblings and seams are all conserved-quantity flows; the biosphere's accumulators live in the delegated slow registry), so every assertion about it is [] == [] and a regeneration WRITES that empty list rather than splicing it (C7 moved the writer here; before that the checker spliced the dump's copy). The evidence that the walk happens at all is a measured control, not a green run — see the dump example and the module docstring.",
    ),
    (
        "delegates_to",
        "hand",
        "pointer to the biosphere manifest, which this contract delegates rather than re-hashes. A path, not a derived value — its target's existence is checked by test_manifest_named_files_exist",
    ),
    (
        "flow_set",
        "rust",
        "the union of Flow::type_name() over the canonical station registries in the reference tree — the four standalone siblings plus the maximal sealed fast registry, derived from built registries and never hand-listed",
    ),
    (
        "frozen_at_phase",
        "hand",
        "the phase this surface froze at",
    ),
    (
        "integrator",
        "hand",
        "the deliberate anti-derived literal, and unlike the biosphere's dt_days it has no importable constant on EITHER side — each run helper selects the scheme inline — so it is documented here and enforced by the goldens (an RK4 switch moves every one). A literal typed into the Rust dump to make the pair symmetric would read like a gate and be none.",
    ),
    (
        "liveness_floors/*",
        "rust",
        "the same census, for the bounds tuned to our own calibration rather than to an outside source — re-anchored with it in slice C4b. ⚠ This manifest's single floor is the thermal node's non-collapse bound, whose clearance is 1.6x (annual peaks sit at 160.12 K against a floor of 100.0); the companion stationarity and T_eq-proximity assertions are what give the gate its teeth, and they travelled with it.",
    ),
    (
        "numerics_note",
        "hand",
        "⚠ HAND-MAINTAINED PROSE THAT NOTHING CHECKS, and slice 7 deliberately left it that way. Unlike the biosphere the station has no structured dt key: the steps live inside this English sentence, so flipping one reddens nothing here. The reference tree DOES have referents — sealed_station_scenario()'s bio_dt / cabin_dt and the energy scenario's power_dt — so slice 6's dt_days treatment is buildable. It needs a structured key that does not exist, and adding one WIDENS the frozen surface, which is its own unfreeze with its own ceremony rather than a rider on a re-anchoring. Recorded here so the hole is a stated claim. ⚠⚠ C7's STATION HALF MADE IT WORSE AND MEASURED HOW: the writer now lives in the crate that owns all three steps, and splicing them is only PARTLY visible. bio_dt would render dt=0.25 day against the written dt=1/4 day, so the regeneration gate reddens; cabin_dt and power_dt render 60 and 3600, BYTE-IDENTICAL to what this sentence already says, because Rust prints 60.0_f64 as 60. So two of the three would auto-follow the code invisibly, with no structured key to compare them against on either side. The guard is rust/crates/station/tests/manifest_writer.rs, which reads the writer's own source and requires the emission site to be a quoted literal naming none of the three.",
    ),
    (
        "param_files/*",
        "rust",
        "⚠ RE-ANCHORED IN SLICE C8, the same finding as the biosphere's: what moved is the RULE, not the number. All 8 digests here (and all 15 there) are author-neutral — both sides hash the same file the same way — so the re-anchoring moved none of them. The CENSUS is now the set the reference LOADS: domains::params::param_files (power × 2, thermal, eclss, crew) plus station::params::param_files (water_recovery, lamp, harvest), eight compile-time include_str! entries instead of a glob over six Python package directories. ⚠ NO exclusion rule on this side, unlike the biosphere's 15-of-20 — these six directories hold nothing but frozen files, and the asymmetry is stated per side so nobody generalises the harder rule. The NORMALIZATION is config::provenance (hand-rolled sha-256 over LF-normalized text; every engine crate is zero-dep by charter). ⚠ Newly asserted with the re-anchor: every basename is unique across the six directories — this key is basename-KEYED, so a collision would silently collapse two files into one entry, and nothing had checked it. Prerequisite: slice C1.",
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
        "the golden is the reference tree's own output (golden_platform.RUST_AUTHORED, which this block is checked against, not restating). Unlike the param hashes here this one IS gated against the file on disk: a golden is machine-generated and its hash is newline-normalized, so 'the manifest pins bytes that exist' is a completeness claim, not the value re-assertion that param_files declines. ⚠ SLICE C5 removed this key's ONE exception and that is why the axis is now uniform. `scenarios/sealed_energy_drift/golden_sha256` used to be carved out as `python` — 'ONE RUN, TWO AUTHORS: drift.py's Python-side fold of the 15-yr sealed energy series; the fold IS the artifact, so its correct reference is Python's own output.' That stopped being true when `domains::biosphere::drift` gained the fold kit and `emit_sealed_energy_drift` began emitting the summary itself. ⚠ The HASH did not move (measured byte-identical before the change), so this is an authorship re-anchoring, not a value unfreeze — the same shape C8 found for param_files, where the digits were author-neutral and the RULE moved.",
    ),
    (
        "scenarios/*/scenario",
        "hand",
        "a human label for the scenario, not an identifier anything resolves",
    ),
    (
        "science_bands/*",
        "rust",
        "⚠⚠ RE-ANCHORED IN SLICE C4b, AND THIS ENTRY USED TO SAY IT COULD NOT BE. Its own text read 'a static AST census of science_gate markers on pytest functions (tests/science_gates.py). There is no Rust referent and there cannot be one while the science gates are pytest-side' — the SAME sentence the biosphere manifest carried until C4, and the third frozen 'why' in this flip to argue against the slice that arrived. It was true of pytest markers and false of the claim it appeared to make: the census's requirement is that it be DERIVED from the tree rather than hand-listed, and the reference meets it by making the declaration and the #[test] ONE THING (the science_gates! macro, exported from domains and invoked in rust/crates/station/src/science_gates.rs), so an unexercised entry is a compile error rather than something a meta-test hunts textually. ⚠ TWO tables, not one: a gate lives with the runs it reads, and these read the coupled cabin and the Power→Thermal station — station types, in a crate that depends on domains rather than the reverse. The mechanism and the transcribed bound-literal regex are shared; only the tables are split, the same way the two contracts are. ⚠ What moved is the LOCUS and nothing else — the quantity/bound/source strings are byte-identical to the pytest markers', and the Python test bodies stay as the checker's conformance half. ⚠ On this manifest the census is mostly EMPTY — 11 of 13 scenarios carry no outside-sourced bound — and the emptiness is itself the frozen claim; the ROSTER it is taken over is still this file's hand-authored scenario set, and a gate naming a scenario outside it RAISES during regeneration rather than being filtered away.",
    ),
    (
        "sealed_energy_years",
        "rust",
        "the reference tree's SEALED_ENERGY_YEARS. ⚠ In that tree it is DEFINED as LONG_HORIZON_YEARS, so since slice 7 this contract and the biosphere's are anchored to one reference-side constant: moving the decade horizon is a single edit that unfreezes two manifests. A reader who assumes they are independent will predict the wrong diff.",
    ),
    (
        "sealed_station_years",
        "rust",
        "the reference tree's SEALED_STATION_YEARS",
    ),
];

const SCENARIOS: [(&str, &str, &str); 13] = [
    (
        "cabin_gas",
        "CABIN_GAS_SCENARIO (P6.2 crew↔ECLSS)",
        "cabin_gas_state.json",
    ),
    (
        "crew_mission",
        "MISSION_SCENARIO (standalone Crew)",
        "crew_state.json",
    ),
    (
        "eclss_steady_state",
        "STEADY_STATE_SCENARIO (standalone ECLSS)",
        "eclss_state.json",
    ),
    (
        "greenhouse",
        "GREENHOUSE_SCENARIO (P6.3 biosphere↔cabin)",
        "greenhouse_state.json",
    ),
    (
        "harvest",
        "HARVEST_SCENARIO (P6.6 biomass→food)",
        "harvest_state.json",
    ),
    (
        "lighting",
        "LIGHTING_SCENARIO (P6.5 Power→biosphere lamp)",
        "lighting_state.json",
    ),
    (
        "power_bounded_soc",
        "BOUNDED_SOC_SCENARIO (standalone Power)",
        "power_state.json",
    ),
    (
        "power_self_discharge",
        "SELF_DISCHARGE (standalone Power + SelfDischarge)",
        "power_self_discharge_state.json",
    ),
    (
        "sealed_energy_drift",
        "HEAT_CLOSURE_SCENARIO 15-yr (P6.7 Tier-1 energy stability signature)",
        "sealed_energy_drift_summary.json",
    ),
    (
        "sealed_station",
        "SEALED_STATION_SCENARIO (P6.7 Tier-2 combined-ledger multi-year)",
        "sealed_station_state.json",
    ),
    (
        "station_heat_closure",
        "HEAT_CLOSURE_SCENARIO (P6.1 Power→Thermal heat closure)",
        "station_state.json",
    ),
    (
        "thermal_equilibrium",
        "EQUILIBRIUM_SCENARIO (standalone Thermal)",
        "thermal_state.json",
    ),
    (
        "water_recovery",
        "WATER_RECOVERY_SCENARIO (P6.4 crew water loop)",
        "water_recovery_state.json",
    ),
];

const COMMENT: &str = "Phase-6 Step-10 station freeze manifest (P6.10). Names the frozen WHOLE-ASSEMBLY station reference surface (Phase-5 siblings + the station seams); the biosphere is delegated to docs/biosphere-reference.manifest.json (see delegates_to). See docs/station-reference.md for the freeze contract + the unfreeze discipline. Hashes are newline-normalized sha-256 PROVENANCE (value enforcement is the scenario goldens). Each key's producer and why is in _authority: this file has MIXED authority since slice 7 of the reference flip. Regenerate on a deliberate unfreeze, from rust/: cargo run --example dump_station_inventory -- --write-manifest. C7 moved the WRITER to the reference; tests/test_station_freeze_manifest.py has none and is now only a checker.";

const NUMERICS_NOTE: &str = "Euler everywhere; dt per scenario (enforced by goldens, no importable constant). Sealed reference: biosphere-slow dt=1/4 day, 4 slow sub-steps per master day + everything-fast dt=60 s; Tier-1 energy single-rate dt=3600 s.";

/// The flow and aux inventories, and the eight-file param census — walked ONCE and read
/// by **both** halves of this program, the dump and the manifest writer.
///
/// ⚠ The biosphere half's first draft had the writer re-walk the registries, which put
/// two derivations of the same sets in one file with only the parity gate able to notice
/// them drifting. Sharing makes the drift impossible rather than merely detectable, and
/// costs that gate nothing: its subject is staleness — the live tree against the frozen
/// file — not one code path against another.
///
/// Tier-0 sanity here rather than at each use site, so a build that wired nothing names
/// its own cause instead of reporting sixteen missing flows. ⚠ Only the FLOW axis can be
/// checked this way: the station `aux_set` is legitimately empty (the siblings and seams
/// are all conserved-quantity flows; the biosphere's accumulators live in the delegated
/// slow registry), so `!aux.is_empty()` would be a false assertion, and `[] == []` is why
/// that axis needs a measured negative control rather than a green run.
///
/// ⚠⚠ **Since C7's station half that empty list is WRITTEN by this program, not spliced
/// by the checker.** The two-direction rename control slice 6 used is unrunnable here —
/// there is no station aux process to rename — so the substitute, re-run when this half
/// landed, is to wire one in temporarily and confirm the regenerated manifest **gains the
/// name**. That is the only evidence this walk reaches `aux_processes()` at all.
fn inventory() -> (
    BTreeSet<&'static str>,
    BTreeSet<&'static str>,
    Vec<(&'static str, &'static str)>,
) {
    let mut flows: BTreeSet<&str> = BTreeSet::new();
    let mut aux: BTreeSet<&str> = BTreeSet::new();
    for registry in &canonical() {
        flows.extend(registry.flows().iter().map(|f| f.type_name()));
        aux.extend(registry.aux_processes().iter().map(|a| a.type_name()));
    }
    assert!(!flows.is_empty(), "canonical station builds wired no flows");

    // The eight the station contract spans, sorted by basename. Every basename is unique
    // across the six directories (asserted in `station::params`'s tests, because the
    // manifest keys on basenames and a collision would silently collapse two entries).
    let mut files: Vec<(&str, &str)> = params::param_files();
    files.extend(station_params::param_files());
    files.sort_by_key(|(name, _)| *name);
    assert_eq!(
        files.len(),
        8,
        "the frozen station param census is 8 files, got {}",
        files.len()
    );
    (flows, aux, files)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None => dump(),
        Some("--write-manifest") => {
            let path = args.get(1).map(PathBuf::from).unwrap_or_else(|| {
                repo_root()
                    .join("docs")
                    .join("station-reference.manifest.json")
            });
            write_manifest(&path);
        }
        Some(other) => {
            eprintln!(
                "usage: dump_station_inventory [--write-manifest [path]]
                   (no argument dumps the reference's half of the manifest as JSON)
                 unknown argument: {other}"
            );
            std::process::exit(2);
        }
    }
}

/// The reference's half of the manifest, as JSON on stdout — the checking surface.
///
/// ⚠ Deliberately still a `println!` stream of raw UTF-8, unchanged by C7's station half.
/// `tests/crossport/test_inventory_parity.py` reads it, and the `encoding="utf-8"` pin
/// that reader grew after C4's mojibake is a control that only has teeth while there is
/// non-ASCII to mangle — which, since C4b put the claim text here, there is.
fn dump() {
    let (flows, aux, files) = inventory();

    println!("{{");
    println!("  \"aux_set\": {},", json_array(&aux));
    println!("  \"flow_set\": {},", json_array(&flows));
    println!("  \"horizons\": {{");
    println!("    \"sealed_energy_years\": {SEALED_ENERGY_YEARS},");
    println!("    \"sealed_station_years\": {SEALED_STATION_YEARS}");
    println!("  }},");
    println!("  \"liveness_floors\": {},", census(LIVENESS_FLOORS));
    println!("  \"param_files\": {{");
    for (i, (name, text)) in files.iter().enumerate() {
        let comma = if i + 1 == files.len() { "" } else { "," };
        println!("    \"{name}\": \"{}\"{comma}", normalized_sha256(text));
    }
    println!("  }},");
    println!("  \"science_bands\": {}", census(SCIENCE_BANDS));
    println!("}}");
}

// ==========================================================================
// The manifest writer — C7's station half of the reference flip
// ==========================================================================
//
// ⚠⚠ **What C7 moves is the WRITER, not the authority.** Until this slice
// `tests/test_station_freeze_manifest.py::_build_manifest()` assembled the file: it
// shelled the dump above, spliced the reference's keys into its own, and serialized the
// result. So the contract was *authored* by the reference and *written* by the checker —
// a Python-shaped hole in the middle of a file whose first line says Rust is the
// reference. This is the last of the three.
//
// ⚠ **`_authority` records who produced the VALUE, not who ran the digest and not who
// wrote the file.** The precedent predates C7 and sits in the table below:
// `scenarios/*/golden_sha256` has read `rust` since slice 4 while *Python* computed the
// digest, because the golden is the reference's own output. So the move is
// authority-neutral by construction, and the rows that change side below change for
// their own stated reasons.
//
// ⚠⚠ **THE TRAP, AND HERE IT IS PARTIAL — WHICH IS WORSE THAN THE BIOSPHERE'S.**
// `numerics_note` is hand-maintained prose naming three steps, and this writer now lives
// in the crate that owns all three (`sealed_station_scenario()`'s `bio_dt` / `cabin_dt`,
// the energy scenario's `power_dt`). Measured, not argued:
//
//   * splicing `bio_dt` gives `dt=0.25 day` against the written `dt=1/4 day` — the bytes
//     move and the regeneration gate is red;
//   * splicing `cabin_dt` gives `dt=60 s`, and `power_dt` gives `dt=3600 s` — **both
//     byte-identical**, because Rust's `Display` prints `60.0_f64` as `60`.
//
// So two of the three referents would auto-follow the code invisibly, and unlike the
// biosphere there is **no second guard at all**: that contract's `dt_days` is at least
// compared against `BIO_DT` across the port boundary, while the station has no structured
// step key to compare (adding one widens the frozen surface — its own ceremony, declined
// again here, see the `numerics_note` row in the table).
//
// `crates/station/tests/manifest_writer.rs` is the guard: it reads this file's own source
// and asserts the emission site is a quoted literal naming none of the three constants.
//
// ⚠ **No pipe.** The file is written with `std::fs::write`. C4's first regeneration froze
// cp1252-mangled prose into a contract with every gate green, because a `subprocess` pipe
// decoded UTF-8 with the Windows locale and both sides were mangled identically. Writing
// here deletes that class rather than inheriting it.

/// The repository root, from this crate's own location.
///
/// ⚠ A third copy of the same three-hop walk (the biosphere dump and the census modules
/// have the others), and it is unavoidable rather than sloppy: `CARGO_MANIFEST_DIR` is
/// per-crate, so a shared helper would resolve to whichever crate compiled it. What must
/// not happen is a *hidden* reach-out — hence one named function per crate with the hop
/// count in one place.
fn repo_root() -> PathBuf {
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

/// One science-gate field as `scenario -> [claim]`, filed under the roster below.
///
/// ⚠ Every roster scenario gets a key, including the **eleven** with no gate: an empty
/// list says *measured, none* and an absent key says nothing, and a reader reaching for
/// `.get(name, [])` cannot tell them apart. On this contract that is the frozen claim
/// itself — 11 of 13 station scenarios carry no outside-sourced bound.
///
/// ⚠⚠ Slice C4b hit exactly this: its first regeneration handed the *dump's* shape
/// through (the dump emits only scenarios that have a claim, deliberately) and silently
/// deleted those eleven keys. Predicting the diff is what caught it.
///
/// ⚠ A gate naming a scenario outside the roster **panics** rather than being filtered
/// away. Both manifests filter the census by scenario, so a typo or a gate filed against
/// the wrong contract would otherwise be dropped by both in silence — the filter looking
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

/// The `_authority` block as JSON.
fn authority_json() -> Json {
    Json::obj(AUTHORITY.iter().map(|(path, side, why)| {
        (
            *path,
            Json::obj([("side", Json::s(*side)), ("why", Json::s(*why))]),
        )
    }))
}

/// The whole station freeze manifest.
fn manifest() -> Json {
    let root = repo_root();
    let golden_dir = root.join("rust").join("data").join("golden");
    let (flows, aux, files) = inventory();

    let scenarios = Json::obj(SCENARIOS.iter().map(|(name, label, golden)| {
        (
            *name,
            Json::obj([
                ("golden", Json::s(*golden)),
                (
                    "golden_sha256",
                    Json::s(file_sha256(&golden_dir.join(golden))),
                ),
                ("scenario", Json::s(*label)),
            ]),
        )
    }));

    Json::obj([
        ("_authority", authority_json()),
        ("_comment", Json::s(COMMENT)),
        ("aux_set", Json::strs(aux.iter().copied())),
        ("delegates_to", Json::s(BIOSPHERE_MANIFEST)),
        ("flow_set", Json::strs(flows.iter().copied())),
        ("frozen_at_phase", Json::int(6)),
        // ⚠ The anti-derived literal: the scheme is selected inline by each run helper
        // and has no importable name on EITHER side, so a constant spliced here would be
        // a second hand literal checked against the first. Enforced by the goldens.
        ("integrator", Json::s("EulerIntegrator")),
        ("liveness_floors", census_json(LIVENESS_FLOORS)),
        // ⚠⚠ TEXT, and read the writer header before touching it. This crate owns all
        // three steps this sentence names, and splicing two of them is byte-invisible.
        ("numerics_note", Json::s(NUMERICS_NOTE)),
        (
            "param_files",
            Json::obj(
                files
                    .iter()
                    .map(|(name, text)| (*name, Json::s(normalized_sha256(text)))),
            ),
        ),
        ("reference_doc", Json::s("docs/station-reference.md")),
        ("scenarios", scenarios),
        ("science_bands", census_json(SCIENCE_BANDS)),
        (
            "sealed_energy_years",
            Json::int(i64::try_from(SEALED_ENERGY_YEARS).expect("a horizon fits in i64")),
        ),
        (
            "sealed_station_years",
            Json::int(i64::try_from(SEALED_STATION_YEARS).expect("a horizon fits in i64")),
        ),
    ])
}

/// Write the manifest to `path`, and report what changed.
///
/// ⚠ Reports rather than asserts: this is the *regeneration* entry point, run on a
/// deliberate unfreeze, so a moved byte is the thing being reviewed. The assertion that
/// the committed file matches lives on the checking side
/// (`tests/crossport/test_manifest_writer.py`), where a stale manifest is red in CI.
fn write_manifest(path: &Path) {
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
