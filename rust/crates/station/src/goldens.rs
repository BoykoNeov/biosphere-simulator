//! The station half of the golden roster — Stage-3 slice **S2** of the reference flip.
//!
//! The policy, the comparison and the `Golden` type live in [`domains::goldens`]; this
//! module holds the eight runs the assembled station produces and the *whole-census*
//! roster [`ALL`], which is here rather than in `domains` for the one structural reason
//! S1 kept running into: **`station` depends on `domains` and not the reverse**, so this
//! is the lowest crate that can see all nineteen.
//!
//! ⚠ That is the same rule S1 used to put the data in `rust/data/` — *put the thing where
//! the dependency is*. The alternative (a new workspace crate owning the census) was
//! considered and refused as unnecessary: unlike the three freeze manifests, which really
//! do span `domains` + `station` + `authoring`, the goldens stop at `station`.

use domains::goldens::{Cost, Golden, Numerics, Shape};
use domains::params;
use simcore::hexfloat;
use simcore::integrator::EulerIntegrator;
use simcore::state::State;

use crate::cabin::{build_cabin, cabin_resolver};
use crate::greenhouse::{
    build_greenhouse, greenhouse_bio_resolver, greenhouse_cabin_resolver, run_greenhouse,
};
use crate::harvest::{build_harvest, harvest_bio_resolver, harvest_cabin_resolver, run_harvest};
use crate::lighting::{build_lighting, lighting_bio_resolver, lighting_power_resolver, run_lighting};
use crate::params as station_params;
use crate::run_station;
use crate::scenario::{
    greenhouse_scenario, harvest_scenario, lighting_scenario, sealed_station_scenario,
    CABIN_GAS_SCENARIO, CABIN_GAS_STEPS, HEAT_CLOSURE_DAYS, HEAT_CLOSURE_SCENARIO,
    SEALED_ENERGY_DAYS, SEALED_ENERGY_YEARS, SEALED_STATION_SEASON_DAYS, WATER_RECOVERY_SCENARIO,
    WATER_RECOVERY_STEPS,
};
use crate::sealed::{build_sealed_station, run_sealed, sealed_bio_resolver, sealed_fast_resolver};
use crate::system::{build_station, station_resolver};
use crate::water::{build_water_recovery, water_recovery_resolver};

fn snapshot(state: &State) -> String {
    simcore::snapshot::from_engine(state).to_json()
}

/// The coupled crew ↔ ECLSS `CABIN_GAS_SCENARIO` — transcendental-free.
pub fn cabin_gas() -> String {
    let crew = params::crew();
    let eclss = params::eclss();
    let scenario = CABIN_GAS_SCENARIO;
    let (state, registry) = build_cabin(&crew, &eclss, &scenario).expect("build_cabin");
    let resolver = cabin_resolver(&scenario).expect("cabin_resolver");
    let integrator = EulerIntegrator::new(registry);
    let mut noop = |_: &State| {};
    let (final_state, rationed, events) = run_station(
        &integrator,
        state,
        &resolver,
        scenario.dt_seconds,
        CABIN_GAS_STEPS,
        &mut noop,
    )
    .expect("run cabin");

    assert_eq!(
        rationed, 0,
        "Tier-0: cabin rationed must be 0 (well-fed sizing)"
    );
    assert!(events.is_empty(), "Tier-0: cabin events must be empty");
    snapshot(&final_state)
}

/// The crew water-recovery loop — still only `*`/`+`/`-`/`/` atop the cabin.
pub fn water_recovery() -> String {
    let crew = params::crew();
    let eclss = params::eclss();
    let recovery = station_params::water_recovery();
    let scenario = WATER_RECOVERY_SCENARIO;
    let (state, registry) =
        build_water_recovery(&crew, &eclss, &recovery, &scenario).expect("build_water_recovery");
    let resolver = water_recovery_resolver(&scenario).expect("water_recovery_resolver");
    let integrator = EulerIntegrator::new(registry);
    let mut noop = |_: &State| {};
    let (final_state, rationed, events) = run_station(
        &integrator,
        state,
        &resolver,
        scenario.dt_seconds,
        WATER_RECOVERY_STEPS,
        &mut noop,
    )
    .expect("run water_recovery");

    assert_eq!(rationed, 0, "Tier-0: water-recovery rationed must be 0");
    assert!(
        events.is_empty(),
        "Tier-0: water-recovery events must be empty"
    );
    snapshot(&final_state)
}

/// The coupled Power → Thermal `HEAT_CLOSURE_SCENARIO`, 7 days.
pub fn station() -> String {
    let charge = params::charge();
    let thermal = params::thermal();
    let scenario = HEAT_CLOSURE_SCENARIO;
    let (state, registry) =
        build_station(&charge, &thermal, &scenario, None).expect("build_station");
    let resolver = station_resolver(&charge, &scenario).expect("station_resolver");
    let integrator = EulerIntegrator::new(registry);
    let steps = HEAT_CLOSURE_DAYS * scenario.power.steps_per_day;
    let mut noop = |_: &State| {};
    let (final_state, rationed, events) = run_station(
        &integrator,
        state,
        &resolver,
        scenario.power.dt_seconds,
        steps,
        &mut noop,
    )
    .expect("run station");

    assert_eq!(
        rationed, 0,
        "Tier-0: station rationed must be 0 (well-fed sizing)"
    );
    assert!(
        events.is_empty(),
        "Tier-0: station events must be empty (no POPULATION stock)"
    );
    snapshot(&final_state)
}

/// The biosphere ↔ cabin `GREENHOUSE_SCENARIO` (two-rate), day 7.
pub fn greenhouse() -> String {
    let crew = params::crew();
    let eclss = params::eclss();
    let scenario = greenhouse_scenario();
    let (state, bio_reg, cabin_reg) = build_greenhouse(
        &crew,
        &eclss,
        &scenario,
        true,
        domains::crew::FECAL_WASTE,
    )
    .expect("build_greenhouse");
    let bio_resolver = greenhouse_bio_resolver(&scenario).expect("bio_resolver");
    let cabin_resolver = greenhouse_cabin_resolver(&scenario).expect("cabin_resolver");
    let (states, rationed, events) = run_greenhouse(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(cabin_reg),
        state,
        &bio_resolver,
        &cabin_resolver,
        &scenario,
    )
    .expect("run greenhouse");

    assert_eq!(rationed, 0, "Tier-0: greenhouse rationed must be 0");
    assert!(events.is_empty(), "Tier-0: greenhouse events must be empty");
    snapshot(states.last().expect("at least one day boundary"))
}

/// The Power → biosphere `LIGHTING_SCENARIO` (two-rate), day 7.
pub fn lighting() -> String {
    let lamp = station_params::lamp();
    let scenario = lighting_scenario();
    let (state, bio_reg, power_reg) =
        build_lighting(&lamp, &scenario, true).expect("build_lighting");
    let bio_resolver = lighting_bio_resolver(&lamp, &scenario, true).expect("bio_resolver");
    let power_resolver = lighting_power_resolver(&scenario).expect("power_resolver");
    let (states, rationed, events) = run_lighting(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(power_reg),
        state,
        &bio_resolver,
        &power_resolver,
        &scenario,
    )
    .expect("run lighting");

    assert_eq!(rationed, 0, "Tier-0: lighting rationed must be 0");
    assert!(events.is_empty(), "Tier-0: lighting events must be empty");
    snapshot(states.last().expect("at least one day boundary"))
}

/// The biomass/food `HARVEST_SCENARIO` — the closed trophic ring, day 7.
pub fn harvest() -> String {
    let crew = params::crew();
    let eclss = params::eclss();
    let harvest_params = station_params::harvest();
    let scenario = harvest_scenario();
    let (state, bio_reg, cabin_reg) =
        build_harvest(&crew, &eclss, &harvest_params, &scenario, true, true)
            .expect("build_harvest");
    let bio_resolver = harvest_bio_resolver(&scenario).expect("bio_resolver");
    let cabin_resolver = harvest_cabin_resolver(&scenario).expect("cabin_resolver");
    let (states, rationed, events) = run_harvest(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(cabin_reg),
        state,
        &bio_resolver,
        &cabin_resolver,
        &scenario,
    )
    .expect("run harvest");

    assert_eq!(rationed, 0, "Tier-0: harvest rationed must be 0");
    assert!(events.is_empty(), "Tier-0: harvest events must be empty");
    snapshot(states.last().expect("at least one day boundary"))
}

/// The fully-coupled sealed station over the multi-year horizon.
///
/// ⚠⚠ **~1.3 M sub-steps; this is the [`Cost::Expensive`] entry.** Its real payload is the
/// per-sub-step conservation assert inside the driver (the Tier-0 gate): a completed run
/// is itself proof the combined ledger balanced every sub-step over the full five-domain
/// assembly. ⚠ That assert is a `Result` in this port, not a `debug_assert!` — measured
/// while sizing this slice — so it is live at every optimization level and a release-built
/// run is not a weaker one.
pub fn sealed_station() -> String {
    let charge = params::charge();
    let thermal = params::thermal();
    let crew = params::crew();
    let eclss = params::eclss();
    let recovery = station_params::water_recovery();
    let lamp = station_params::lamp();
    let harvest_params = station_params::harvest();
    let scenario = sealed_station_scenario();

    let (state, bio_reg, fast_reg) = build_sealed_station(
        &charge,
        &thermal,
        &crew,
        &eclss,
        &recovery,
        &lamp,
        &harvest_params,
        &scenario,
        false,
        false,
    )
    .expect("build_sealed_station");
    let bio_resolver = sealed_bio_resolver(&lamp, &scenario).expect("sealed_bio_resolver");
    let fast_resolver = sealed_fast_resolver(&charge, &scenario).expect("sealed_fast_resolver");

    let (states, rationed, events) = run_sealed(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(fast_reg),
        state,
        &bio_resolver,
        &fast_resolver,
        &scenario,
    )
    .expect("run sealed station");

    assert_eq!(rationed, 0, "Tier-0: sealed station rationed must be 0");
    assert!(
        events.is_empty(),
        "Tier-0: sealed station events must be empty"
    );
    snapshot(states.last().expect("at least one day boundary"))
}

/// The sealed station's 15-yr energy **stability signature**, folded in Rust since C5.
///
/// ⚠ The canonical JSON below is a `write!`, not a serializer, so the key order
/// (`indent=2, sort_keys=True` + trailing newline, matching `sim_io.dumps`) is an
/// invariant of *this function*. The byte compare against the committed golden is what
/// holds it — which as of S2 is a **Rust** test rather than only a Python one.
pub fn sealed_energy_drift() -> String {
    use domains::biosphere::drift::{is_stationary, same_phase_diffs, year_summaries};
    use domains::thermal::NODE;
    use std::fmt::Write as _;

    let charge = params::charge();
    let thermal = params::thermal();
    let scenario = HEAT_CLOSURE_SCENARIO;
    let (state, registry) =
        build_station(&charge, &thermal, &scenario, None).expect("build_station");
    let resolver = station_resolver(&charge, &scenario).expect("station_resolver");
    let integrator = EulerIntegrator::new(registry);
    let steps = SEALED_ENERGY_DAYS * scenario.power.steps_per_day;

    // ⚠ The node TEMPERATURE is reduced in the observer, not after the run. Python's
    // `peak_temp` computes `space_temp + node/C` per state and then takes the `max` over
    // the segment, so reducing per step and folding `max` afterwards is the same sequence
    // of IEEE operations — and it avoids materializing 109,801 `State`s to fold over.
    let mut node_temp: Vec<f64> = Vec::new();
    let (_final, rationed, events) = run_station(
        &integrator,
        state,
        &resolver,
        scenario.power.dt_seconds,
        steps,
        &mut |s: &State| {
            node_temp
                .push(thermal.space_temperature + s.stocks[NODE].amount / thermal.heat_capacity)
        },
    )
    .expect("run energy drift");

    assert_eq!(rationed, 0, "Tier-0: energy drift rationed must be 0");
    assert!(
        events.is_empty(),
        "Tier-0: energy drift events must be empty"
    );

    // ⚠ In DAYS times steps-per-day, i.e. the trajectory's own index unit — `run_station`
    // appends one state per POWER step. The biosphere's `run_season` trajectory is indexed
    // differently (see `year_summaries`); passing one unit where the other is meant is the
    // trap `docs/plans/post-roadmap-step-unfreeze.md` §1 records.
    let steps_per_year = scenario.power.steps_per_day as usize * SEALED_STATION_SEASON_DAYS;
    let peaks = year_summaries(&node_temp, steps_per_year, |segment: &[f64]| {
        segment.iter().fold(f64::NEG_INFINITY, |acc, &t| acc.max(t))
    });
    // Period 1, not 2: Power carries no seasonal forcing, so the node's attractor is a
    // fixed point and consecutive years ARE the same phase.
    let stationary = is_stationary(&same_phase_diffs(&peaks, 1), 0.1, 1e-3, 0);

    let mut out = String::new();
    writeln!(out, "{{").unwrap();
    writeln!(out, "  \"horizon_years\": {SEALED_ENERGY_YEARS},").unwrap();
    writeln!(out, "  \"is_stationary\": {stationary},").unwrap();
    writeln!(out, "  \"node_peak_temp_k\": [").unwrap();
    for (i, v) in peaks.iter().enumerate() {
        let comma = if i + 1 < peaks.len() { "," } else { "" };
        writeln!(out, "    \"{}\"{}", hexfloat::format(*v), comma).unwrap();
    }
    writeln!(out, "  ]").unwrap();
    writeln!(out, "}}").unwrap();
    out
}

/// The eight goldens the assembled station produces.
pub const STATION: &[Golden] = &[
    Golden {
        name: "cabin_gas_state.json",
        run: cabin_gas,
        numerics: Numerics::PureArithmetic,
        cost: Cost::Cheap,
        shape: Shape::StateSnapshot,
    },
    Golden {
        name: "water_recovery_state.json",
        run: water_recovery,
        numerics: Numerics::PureArithmetic,
        cost: Cost::Cheap,
        shape: Shape::StateSnapshot,
    },
    Golden {
        name: "station_state.json",
        run: station,
        numerics: Numerics::Transcendental,
        cost: Cost::Cheap,
        shape: Shape::StateSnapshot,
    },
    Golden {
        name: "greenhouse_state.json",
        run: greenhouse,
        numerics: Numerics::Transcendental,
        cost: Cost::Cheap,
        shape: Shape::StateSnapshot,
    },
    Golden {
        name: "lighting_state.json",
        run: lighting,
        numerics: Numerics::Transcendental,
        cost: Cost::Cheap,
        shape: Shape::StateSnapshot,
    },
    Golden {
        name: "harvest_state.json",
        run: harvest,
        numerics: Numerics::Transcendental,
        cost: Cost::Cheap,
        shape: Shape::StateSnapshot,
    },
    Golden {
        name: "sealed_station_state.json",
        run: sealed_station,
        numerics: Numerics::Transcendental,
        cost: Cost::Expensive,
        shape: Shape::StateSnapshot,
    },
    Golden {
        name: "sealed_energy_drift_summary.json",
        run: sealed_energy_drift,
        numerics: Numerics::Transcendental,
        cost: Cost::Cheap,
        // ⚠ The one that is not a snapshot, and the reason the axis exists — see
        // `Shape`'s header for the check that had been unrunnable since C5 because of it.
        shape: Shape::FoldedSummary,
    },
];

/// **Every golden the reference authors** — the nineteen, across both crates.
///
/// ⚠ This is the Rust successor to `tests/golden_platform.RUST_AUTHORED` and
/// `tests/crossport/regen_goldens_from_rust.RUST_EMITTERS`, which the Python side keeps as
/// *two* rosters with a gate asserting they name the same files. Here they are one thing:
/// a name that cannot be spelled without the function beside it, so the duplication the
/// Python gate exists to police does not arise. The two goldens on disk that are **not**
/// here are the ones Python authors — `drift_summary.json` (folded Python-side; the fold
/// moved to Rust in C5 but the artifact did not, for a measured reason) and
/// `state_snapshot.json` (a hand-authored `sim_io` fixture the reference *reads*).
pub fn all() -> Vec<&'static Golden> {
    domains::goldens::DOMAINS.iter().chain(STATION).collect()
}
