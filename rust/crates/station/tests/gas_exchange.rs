//! The crop's gas exchange on the minute step — the adapter's traps, pinned
//! (`docs/plans/post-roadmap-room-temperature.md` §16; module `station::gas_exchange`).
//!
//! Each trap is silent by nature: a light read 90 minutes early, a daily rate multiplied by
//! seconds, a flow left on both steps. None of them breaks conservation, so each needs its own
//! pin. The season-level effects are pinned in `tests/air_split.rs`.

use domains::biosphere::stocks::PAR_VAR;
use domains::params;
use simcore::environment::{Environment, SourceResolver};
use simcore::flow::Flow;
use simcore::registry::Registry;
use simcore::state::State;
use station::gas_exchange::{
    gas_exchange_on_fast_step, require_one_plant_step_per_group, split_carbon_budget, OnFastStep,
    PlantWindow, CARBON_BUDGET_FLOWS,
};
use station::params as station_params;
use station::scenario::{sealed_station_scenario, SealedStationScenario};
use station::sealed::{build_sealed_station, sealed_bio_resolver};

fn built(scenario: &SealedStationScenario) -> (State, Registry, Registry) {
    build_sealed_station(
        &params::charge(),
        &params::thermal(),
        &params::crew(),
        &params::eclss(),
        &station_params::water_recovery(),
        &station_params::lamp(),
        &station_params::harvest(),
        scenario,
        false,
        false,
    )
    .expect("build_sealed_station")
}

fn resolver(scenario: &SealedStationScenario) -> SourceResolver {
    sealed_bio_resolver(&station_params::lamp(), scenario).expect("bio resolver")
}

fn at_n(state: &State, n: u64) -> State {
    State::new(n, state.stocks.clone(), state.rng_seed, state.aux.clone()).expect("state")
}

/// THE WINDOW TRAP. The fast steps after plant step `n − 1 → n` lie in window `n − 1`, so the
/// light they read must be that window's — not window `n`'s, which is what `state.n` would
/// give. Checked over three days of windows, and the pin is only meaningful because the two
/// differ somewhere (asserted): the lamp's 16 h day does not fall on 90-minute boundaries.
#[test]
fn the_fast_step_reads_the_light_of_the_plant_window_it_lies_in() {
    let scenario = sealed_station_scenario();
    let (state, _, _) = built(&scenario);
    let window = PlantWindow::new(resolver(&scenario), scenario.bio_dt);
    let reference = resolver(&scenario);
    let mut windows_that_differ = 0;
    for n in 1..=(3 * scenario.bio_steps_per_day) {
        let snapshot = at_n(&state, n);
        let read = window
            .at(&snapshot)
            .expect("window")
            .get(PAR_VAR)
            .expect("PAR");
        let own = reference
            .bind(&at_n(&state, n - 1), scenario.bio_dt)
            .get(PAR_VAR)
            .expect("PAR");
        let next = reference
            .bind(&snapshot, scenario.bio_dt)
            .get(PAR_VAR)
            .expect("PAR");
        assert_eq!(
            read.to_bits(),
            own.to_bits(),
            "n = {n}: read {read}, window n − 1 {own}"
        );
        windows_that_differ += u32::from(own != next);
    }
    assert!(
        windows_that_differ > 0,
        "window n and n − 1 never differ here, so this pin cannot tell them apart"
    );
    assert!(
        window.at(&at_n(&state, 0)).is_err(),
        "a fast step before the first plant step has no window"
    );
}

/// THE UNIT TRAP, and the window again at the level of a whole flow: the wrapped allocation's
/// legs are the inner allocation's, evaluated against window `n − 1` with `dt` in DAYS.
#[test]
fn the_wrapped_flow_is_the_inner_flow_on_a_minute_in_days() {
    let scenario = sealed_station_scenario();
    let (state, bio_a, _) = built(&scenario);
    let (_, bio_b, _) = built(&scenario);
    let (_, mut raw) = split_carbon_budget(bio_a, &state.stocks).expect("split");
    let (_, inner) = split_carbon_budget(bio_b, &state.stocks).expect("split");
    let raw_allocation = raw.remove(0);
    let wrapped = OnFastStep::new(
        inner.into_iter().next().expect("allocation"),
        PlantWindow::new(resolver(&scenario), scenario.bio_dt),
    );
    assert_eq!(raw_allocation.id(), "biosphere.allocation");
    assert_eq!(wrapped.id(), "biosphere.allocation");
    let reference = resolver(&scenario);
    let fast_dt = scenario.cabin_dt;
    let mut lit = 0;
    for n in 1..=scenario.bio_steps_per_day {
        let snapshot = at_n(&state, n);
        let got = wrapped
            .evaluate(
                &snapshot,
                &SourceResolver::empty().bind(&snapshot, fast_dt),
                fast_dt,
            )
            .expect("wrapped");
        let window_state = at_n(&state, n - 1);
        let want = raw_allocation
            .evaluate(
                &window_state,
                &reference.bind(&window_state, scenario.bio_dt),
                fast_dt / 86_400.0,
            )
            .expect("raw");
        assert_eq!(got.legs.len(), want.legs.len(), "n = {n}");
        for (g, w) in got.legs.iter().zip(&want.legs) {
            assert_eq!(g.stock, w.stock);
            assert_eq!(
                g.amount.to_bits(),
                w.amount.to_bits(),
                "n = {n}, {}",
                g.stock
            );
        }
        lit += u32::from(got.legs.iter().any(|l| l.amount != 0.0));
    }
    assert!(
        lit > 0,
        "no window grew anything, so the legs compared were all zero"
    );
}

/// THE DOUBLE-COUNT TRAP: the three move whole — out of the plant registry, into the fast one,
/// ids and types kept — and nothing else moves.
#[test]
fn the_carbon_budget_moves_whole_and_alone() {
    let scenario = sealed_station_scenario();
    let (state, bio, fast) = built(&scenario);
    let (bio_n, fast_n) = (bio.flows().len(), fast.flows().len());
    let aux_n = bio.aux_processes().len();
    let (bio, fast) = gas_exchange_on_fast_step(&state.stocks, bio, fast, scenario.bio_dt, || {
        Ok(resolver(&scenario))
    })
    .expect("move");
    assert_eq!(bio.flows().len(), bio_n - 3);
    assert_eq!(fast.flows().len(), fast_n + 3);
    for id in CARBON_BUDGET_FLOWS {
        assert!(
            bio.flows().iter().all(|f| f.id() != id),
            "{id} stayed on the plant step"
        );
    }
    let moved: Vec<(&str, &str)> = fast
        .flows()
        .iter()
        .filter(|f| CARBON_BUDGET_FLOWS.contains(&f.id()))
        .map(|f| (f.id(), f.type_name()))
        .collect();
    assert_eq!(
        moved,
        vec![
            ("biosphere.allocation", "Allocation"),
            ("biosphere.growth_respiration", "GrowthRespiration"),
            (
                "biosphere.maintenance_respiration",
                "MaintenanceRespiration"
            ),
        ]
    );
    assert_eq!(
        bio.aux_processes().len(),
        aux_n,
        "the plant step keeps every aux process"
    );
    assert!(fast.aux_processes().is_empty());
}

/// A day of 24 power hours over 16 plant steps groups 2 plant steps per 3 hours, and then a
/// fast step's window has two answers: refused. 1440 cabin minutes over 16 plant steps is fine.
#[test]
fn a_day_without_one_plant_step_per_group_is_refused() {
    assert!(require_one_plant_step_per_group(1440, 16).is_ok());
    assert!(require_one_plant_step_per_group(24, 16).is_err());
    assert!(require_one_plant_step_per_group(1440, 0).is_err());
}
