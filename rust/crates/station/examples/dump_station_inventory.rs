//! Dump the **station** port's own completeness inventory as JSON — slice 3 of the
//! reference flip (`docs/plans/post-roadmap-reference-flip.md`).
//!
//! The union of `Flow::type_name()` / `AuxProcess::type_name()` over the canonical station
//! registries, which is what `tests/test_station_freeze_manifest.py::_flow_set()` /
//! `_aux_set()` derive on the Python side.
//! `tests/crossport/test_inventory_parity.py` asserts this output equals
//! `docs/station-reference.manifest.json`.
//!
//! ⚠ **This is the half of slice 3 nobody had measured.** Slice 2 measured the biosphere
//! names against the manifest and found them identical; the station set had only ever been
//! counted **by eye off a grep** (12 sibling `type_name` impls + 4 station ones = the
//! manifest's 16), and this repo has already priced that exact evidence as a guess — the
//! compiler found four impls the same grep missed, one slice ago.
//!
//! ⚠⚠ **`param_files` is deliberately ABSENT** — same reason as the biosphere dump: this
//! port reads no YAML, only the Python-generated `station_params.txt` / `sibling_params.txt`,
//! whose names carry no file prefix at all. Printing a list here would make the gate compare
//! Python against Python. See `dump_biosphere_inventory.rs` for the full statement.

use domains::crew::{build_crew, MISSION_SCENARIO};
use domains::eclss::{build_eclss, STEADY_STATE_SCENARIO};
use domains::params;
use domains::power::{build_power, BOUNDED_SOC_SCENARIO};
use domains::thermal::{build_thermal, EQUILIBRIUM_SCENARIO};
use simcore::registry::Registry;
use station::params as station_params;
use station::scenario::sealed_station_scenario;
use station::sealed::build_sealed_station;
use std::collections::BTreeSet;

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

fn main() {
    let mut flows: BTreeSet<&str> = BTreeSet::new();
    let mut aux: BTreeSet<&str> = BTreeSet::new();
    let registries = canonical();
    for registry in &registries {
        flows.extend(registry.flows().iter().map(|f| f.type_name()));
        aux.extend(registry.aux_processes().iter().map(|a| a.type_name()));
    }

    // Tier-0 sanity on the program itself. ⚠ Only the FLOW axis can be checked this way:
    // the station `aux_set` is legitimately empty (the siblings and seams are all
    // conserved-quantity flows; the biosphere's accumulators live in the delegated slow
    // registry), so `!aux.is_empty()` would be a false assertion, and `[] == []` is why
    // the gate's aux row needs a measured negative control rather than a green run.
    assert!(!flows.is_empty(), "canonical station builds wired no flows");

    println!("{{");
    println!("  \"aux_set\": {},", json_array(&aux));
    println!("  \"flow_set\": {}", json_array(&flows));
    println!("}}");
}
