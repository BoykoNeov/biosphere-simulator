//! The crop's gas exchange on the minute step — the adapter's traps, pinned
//! (`docs/plans/post-roadmap-room-temperature.md` §16–§17; module `station::gas_exchange`).
//!
//! Each trap is silent by nature: a light read for the wrong window, a daily rate multiplied
//! by seconds, a flow left on both steps, a plant-side change that never reaches
//! photosynthesis. None of them breaks conservation, so each needs its own pin. The
//! season-level effects are pinned in `tests/air_split.rs`.

use std::collections::BTreeMap;

use domains::biosphere::stocks::{LEAF_C, PAR_VAR, ROOT_C, STEM_C, STORAGE_C, TEMP_VAR};
use domains::biosphere::system::weather_shared;
use domains::params;
use simcore::environment::{constant, Environment, SourceResolver};
use simcore::flow::Flow;
use simcore::integrator::EulerIntegrator;
use simcore::registry::Registry;
use simcore::state::State;
use station::driver::run_master_day;
use station::gas_exchange::{
    gas_exchange_on_fast_step, require_one_plant_step_per_group, split_carbon_budget, window_key,
    OnFastStep, CARBON_BUDGET_FLOWS, PLANT_WINDOW_RECORDER, WINDOW_VARS,
};
use station::params as station_params;
use station::scenario::{sealed_station_scenario, SealedStationScenario};
use station::sealed::{build_sealed_station, sealed_bio_resolver, sealed_fast_resolver};

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

/// The sealed build with the crop's gas exchange on the minute step.
fn built_minute(scenario: &SealedStationScenario) -> (State, Registry, Registry) {
    let (state, bio, fast) = built(scenario);
    let (bio, fast) =
        gas_exchange_on_fast_step(&state.stocks, bio, fast, &weather_shared(&scenario.bio))
            .expect("minute gas exchange");
    (state, bio, fast)
}

fn resolver(scenario: &SealedStationScenario) -> SourceResolver {
    sealed_bio_resolver(&station_params::lamp(), scenario).expect("bio resolver")
}

fn at_n(state: &State, n: u64) -> State {
    State::new(n, state.stocks.clone(), state.rng_seed, state.aux.clone()).expect("state")
}

fn with_aux(state: &State, aux: BTreeMap<String, f64>) -> State {
    State::new(state.n, state.stocks.clone(), state.rng_seed, aux).expect("state")
}

fn plant_c(s: &State) -> f64 {
    [LEAF_C, STEM_C, ROOT_C, STORAGE_C]
        .iter()
        .map(|id| s.stocks[*id].amount)
        .sum()
}

/// THE WINDOW. The plant step records PAR and temperature for its OWN window: after the step
/// from `n`, the recorded values are the resolver's at `n`, exactly or within an ULP (the aux
/// channel is additive, so the record is `old + (X − old)`). Over three days of windows, and
/// the pin is meaningful only because consecutive windows' light differs somewhere (asserted).
#[test]
fn the_plant_step_records_its_own_window() {
    let scenario = sealed_station_scenario();
    let (state, bio, _) = built_minute(&scenario);
    let plant = EulerIntegrator::new(bio);
    let reference = resolver(&scenario);
    let (mut exact, mut one_ulp, mut light_changes) = (0, 0, 0);
    let mut previous_par = None;
    let mut s = state;
    for n in 0..(3 * scenario.bio_steps_per_day) {
        assert_eq!(s.n, n);
        let want: Vec<f64> = WINDOW_VARS
            .iter()
            .map(|v| reference.bind(&s, scenario.bio_dt).get(v).expect("forcing"))
            .collect();
        s = plant
            .step(&s, &reference, scenario.bio_dt)
            .expect("plant step");
        for (var, w) in WINDOW_VARS.iter().zip(&want) {
            let got = s.aux[&window_key(var)];
            let ulps = (got.to_bits() as i64 - w.to_bits() as i64).abs();
            assert!(ulps <= 1, "n = {n}, {var}: recorded {got}, window {w}");
            if ulps == 0 {
                exact += 1;
            } else {
                one_ulp += 1;
            }
        }
        light_changes += u32::from(previous_par.is_some_and(|p| p != want[0]));
        previous_par = Some(want[0]);
    }
    assert!(light_changes > 0, "the light never changed between windows");
    eprintln!("recorded exactly {exact}, within one ULP {one_ulp}");
}

/// THE UNIT. The wrapped allocation's legs are the inner allocation's, evaluated against the
/// recorded window with `dt` in DAYS — bit for bit, on every window of a day where it grows.
#[test]
fn the_wrapped_flow_is_the_inner_flow_on_a_minute_in_days() {
    let scenario = sealed_station_scenario();
    let (state, bio_a, _) = built(&scenario);
    let (_, bio_b, _) = built(&scenario);
    let (_, mut raw, _) = split_carbon_budget(bio_a).expect("split");
    let (_, inner, _) = split_carbon_budget(bio_b).expect("split");
    let raw_allocation = raw.remove(0);
    let wrapped = OnFastStep::new(
        inner.into_iter().next().expect("allocation"),
        weather_shared(&scenario.bio),
    );
    assert_eq!(raw_allocation.id(), "biosphere.allocation");
    assert_eq!(wrapped.id(), "biosphere.allocation");
    let reference = resolver(&scenario);
    let fast_dt = scenario.cabin_dt;
    let mut lit = 0;
    for n in 0..scenario.bio_steps_per_day {
        let window = at_n(&state, n);
        let mut aux = window.aux.clone();
        for var in WINDOW_VARS {
            let value = reference.bind(&window, scenario.bio_dt).get(var).unwrap();
            aux.insert(window_key(var), value);
        }
        let snapshot = with_aux(&window, aux);
        let got = wrapped
            .evaluate(
                &snapshot,
                &SourceResolver::empty().bind(&snapshot, fast_dt),
                fast_dt,
            )
            .expect("wrapped");
        let want = raw_allocation
            .evaluate(
                &snapshot,
                &reference.bind(&snapshot, scenario.bio_dt),
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
    // Before any plant step has recorded a window, the minute step refuses rather than read 0.
    assert!(wrapped
        .evaluate(
            &state,
            &SourceResolver::empty().bind(&state, fast_dt),
            fast_dt
        )
        .is_err());
}

/// THE TRAP THAT CHANGED THE DESIGN: a change made to the PLANT side's inputs must reach the
/// crop's carbon budget on the minute step, with nothing else wired. Here the plant-side light
/// is switched off: over two days the crop must fix nothing (it can only respire), where the
/// same build with its light grows. A build-time copy of the plant resolver would keep fixing
/// carbon in the dark — and conserve perfectly while doing it.
#[test]
fn a_change_to_the_plant_side_light_reaches_the_minute_step() {
    let scenario = sealed_station_scenario();
    let run = |dark: bool| {
        let (state, bio, fast) = built_minute(&scenario);
        let mut plant_r = resolver(&scenario);
        if dark {
            let (mut forcings, shared) = plant_r.into_parts();
            forcings.insert(PAR_VAR.to_string(), constant(0.0).unwrap());
            plant_r = SourceResolver::new(forcings, shared).unwrap();
        }
        let fast_r = sealed_fast_resolver(&params::charge(), &scenario).unwrap();
        let c0 = plant_c(&state);
        let (states, rationed, _) = run_master_day(
            &EulerIntegrator::new(bio),
            &EulerIntegrator::new(fast),
            state,
            &plant_r,
            &fast_r,
            2,
            scenario.steps_per_day,
            scenario.bio_steps_per_day,
            scenario.bio_dt,
            scenario.cabin_dt,
            None,
        )
        .expect("run");
        assert_eq!(rationed, 0);
        let end = states.last().unwrap();
        if dark {
            assert_eq!(
                end.aux[&window_key(PAR_VAR)],
                0.0,
                "the dark was not recorded"
            );
        }
        plant_c(end) - c0
    };
    let lit = run(false);
    let dark = run(true);
    assert!(lit > 0.0, "the lit crop did not grow: {lit}");
    assert!(dark <= 0.0, "the crop fixed {dark} mol C in the dark");
}

/// The temperature half of the same trap: a plant-side temperature override is what the
/// plant step records for the minute step's respiration to read.
#[test]
fn a_change_to_the_plant_side_temperature_is_what_gets_recorded() {
    let scenario = sealed_station_scenario();
    let (state, bio, _) = built_minute(&scenario);
    let (mut forcings, shared) = resolver(&scenario).into_parts();
    forcings.insert(TEMP_VAR.to_string(), constant(22.0).unwrap());
    let held = SourceResolver::new(forcings, shared).unwrap();
    let after = EulerIntegrator::new(bio)
        .step(&state, &held, scenario.bio_dt)
        .expect("plant step");
    assert_eq!(after.aux[&window_key(TEMP_VAR)], 22.0);
}

/// THE DOUBLE COUNT: the three move whole — out of the plant registry, into the fast one, ids
/// and types kept — the recorder joins the plant step, and nothing else moves.
#[test]
fn the_carbon_budget_moves_whole_and_alone() {
    let scenario = sealed_station_scenario();
    let (state, bio, fast) = built(&scenario);
    let (bio_n, fast_n, aux_n) = (
        bio.flows().len(),
        fast.flows().len(),
        bio.aux_processes().len(),
    );
    let (bio, fast) =
        gas_exchange_on_fast_step(&state.stocks, bio, fast, &weather_shared(&scenario.bio))
            .expect("move");
    assert_eq!(bio.flows().len(), bio_n - 3);
    assert_eq!(fast.flows().len(), fast_n + 3);
    assert_eq!(bio.aux_processes().len(), aux_n + 1);
    assert!(bio
        .aux_processes()
        .iter()
        .any(|a| a.id() == PLANT_WINDOW_RECORDER));
    assert!(fast.aux_processes().is_empty());
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
}

/// A day of 24 power hours over 16 plant steps groups 2 plant steps per 3 hours, and then the
/// window recorded last would stand for both: refused. 1440 cabin minutes over 16 is fine.
#[test]
fn a_day_without_one_plant_step_per_group_is_refused() {
    assert!(require_one_plant_step_per_group(1440, 16).is_ok());
    assert!(require_one_plant_step_per_group(24, 16).is_err());
    assert!(require_one_plant_step_per_group(1440, 0).is_err());
}
