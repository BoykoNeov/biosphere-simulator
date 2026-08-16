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
//! * ⚠⚠ **`param_files` is absent, and it is not an oversight.** The manifest's param axis
//!   names the frozen `params/*.yaml` files, and **this port has no referent for it**:
//!   Rust reads no YAML. It reads `src/biosphere/biosphere_params.txt`, a file *generated
//!   by Python* (`tests/crossport/gen_biosphere_params.py`) whose `photo.` / `pheno.` /
//!   `vern.` prefixes are the generator's naming, not filenames — three of them come out
//!   of the single `phenology.yaml`, and the sibling/station equivalents carry no prefix
//!   at all. Anything printed under that key would be a copy of the Python list
//!   travelling through Rust and back. The key stays Python-retained until slice 9
//!   decides who loads the params; the manifest's `_authority` block says so in writing.

use domains::biosphere::light_path::half_sine_window_mean;
use domains::biosphere::system::{
    build_season, consumer_chamber_scenario, perennial_chamber_scenario, sealed_chamber_scenario,
    SeasonScenario, DEFAULT_SCENARIO,
};
use domains::biosphere::{
    BIO_DT, CONSUMER_CHAMBER_YEARS, LONG_HORIZON_YEARS, PERENNIAL_CHAMBER_YEARS,
    SEALED_CHAMBER_YEARS,
};
use simcore::hexfloat;
use std::collections::BTreeSet;

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
fn light_path_samples() -> Vec<String> {
    let mut samples = Vec::new();
    for daylength_h in [8.0_f64, 12.0, 16.0] {
        for k in 0..4 {
            let mean =
                half_sine_window_mean(f64::from(k) * 0.25, 0.25, 400.0, daylength_h * 3600.0)
                    .expect("the fingerprint grid lies inside one day");
            samples.push(format!("\"{}\"", hexfloat::format(mean)));
        }
    }
    samples
}

fn main() {
    let mut flows: BTreeSet<&str> = BTreeSet::new();
    let mut aux: BTreeSet<&str> = BTreeSet::new();
    for scenario in canonical() {
        let (_, registry) = build_season(&scenario).expect("canonical biosphere build");
        flows.extend(registry.flows().iter().map(|f| f.type_name()));
        aux.extend(registry.aux_processes().iter().map(|a| a.type_name()));
    }

    // Tier-0 sanity on the program itself: an inventory that came back empty would
    // compare against a non-empty manifest and fail loudly, but failing *here* names the
    // cause (a build that wired nothing) instead of reporting 23 missing flows. ⚠ Since
    // slice 6 this matters more than it did: an empty set here would be *written into*
    // the manifest by a regeneration run, not merely compared against it.
    assert!(
        !flows.is_empty(),
        "canonical biosphere builds wired no flows"
    );
    assert!(
        !aux.is_empty(),
        "canonical biosphere builds wired no aux processes"
    );

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
    println!("  \"locked_dt_days\": {BIO_DT:?}");
    println!("}}");
}
