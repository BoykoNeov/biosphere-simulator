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
use station::chamber::{plants_read_chamber, CHAMBER};
use station::driver::run_master_day;
use station::gas_exchange::{
    gas_exchange_on_fast_step, require_one_plant_step_per_group, split_carbon_budget, window_key,
    GasExchangeStep, OnFastStep, CARBON_BUDGET_FLOWS, PLANT_WINDOW_RECORDER, WINDOW_VARS,
};
use station::params as station_params;
use station::scenario::{sealed_station_scenario, SealedStationScenario};
use station::sealed::{
    build_sealed_station, build_sealed_station_at, build_sealed_station_unread,
    sealed_bio_resolver, sealed_fast_resolver,
};

/// The PLANT-STEP sealed build, **unread** — its plants not yet wrapped to read the chamber —
/// because this file moves the gas exchange itself, and the wrap must come after the move
/// (slice 3a, §24b; [`a_wrap_before_the_move_is_refused`]).
fn built(scenario: &SealedStationScenario) -> (State, Registry, Registry) {
    build_sealed_station_unread(
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
        GasExchangeStep::PlantStep,
        station::chamber::ChamberSurroundings::Outdoor,
    )
    .expect("build_sealed_station_unread")
}

/// The sealed build with the crop's gas exchange on the minute step, the plants then wrapped
/// to read the chamber — the reference's order.
fn built_minute(scenario: &SealedStationScenario) -> (State, Registry, Registry) {
    let (state, bio, fast) = built(scenario);
    let (bio, fast) =
        gas_exchange_on_fast_step(&state.stocks, bio, fast, &weather_shared(&scenario.bio))
            .expect("minute gas exchange");
    let bio = plants_read_chamber(bio, &state.stocks, &station_params::chamber()).expect("wrap");
    (state, bio, fast)
}

/// The plants' temperature (°C) on `state`: the chamber's, `Q / C_ch − 273.15` — the wrapper's
/// own expression.
fn chamber_c(state: &State) -> f64 {
    state.stocks[CHAMBER].amount / station_params::chamber().heat_capacity - 273.15
}

/// What the plant step reads for `var` on `state`: the chamber for `temp`, the resolver else.
fn plant_reads(r: &SourceResolver, state: &State, var: &str, dt: f64) -> f64 {
    if var == TEMP_VAR {
        chamber_c(state)
    } else {
        r.bind(state, dt).get(var).expect("forcing")
    }
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
/// from `n`, the recorded values are what it read at `n` — the resolver's PAR, the chamber's
/// temperature (slice 3a) — exactly or within an ULP (the aux
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
            .map(|v| plant_reads(&reference, &s, v, scenario.bio_dt))
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
    // The raw flow reads `temp` from its environment: the plants' resolver, plus the chamber's
    // temperature on this state as a forcing (the unread build's resolver carries none).
    let (mut forcings, shared) = resolver(&scenario).into_parts();
    forcings.insert(TEMP_VAR.to_string(), constant(chamber_c(&state)).unwrap());
    let reference = SourceResolver::new(forcings, shared).unwrap();
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

/// The temperature half of the same trap: a change to the CHAMBER — the plants' temperature
/// since slice 3a — is what the plant step records for the minute step's respiration to read.
/// (Until 3a this held a plant-side 22 °C forcing, which the wrapper now refuses.)
#[test]
fn a_change_to_the_chamber_is_what_gets_recorded() {
    let scenario = sealed_station_scenario();
    let (state, bio, _) = built_minute(&scenario);
    let plant = EulerIntegrator::new(bio);
    let mut stocks = state.stocks.clone();
    stocks.get_mut(CHAMBER).unwrap().amount = station_params::chamber().heat_capacity * 295.15;
    let held = State::new(state.n, stocks, state.rng_seed, state.aux.clone()).unwrap();
    let r = resolver(&scenario);
    let after = plant.step(&held, &r, scenario.bio_dt).expect("plant step");
    let recorded = after.aux[&window_key(TEMP_VAR)];
    assert_eq!(recorded, chamber_c(&held));
    assert!((recorded - 22.0).abs() < 1e-9, "{recorded}");
    // Control: the unchanged chamber (day 0, the cold period) records its own value.
    let plain = plant.step(&state, &r, scenario.bio_dt).expect("plant step");
    assert_eq!(plain.aux[&window_key(TEMP_VAR)], chamber_c(&state));
    assert!(plain.aux[&window_key(TEMP_VAR)] < 5.0);
}

/// THE ORDER (slice 3a): wrapping the plants to read the chamber BEFORE moving the gas exchange
/// puts a wrapped flow on the minute step, whose recorded window answers `temp` — refused at
/// the first read, rather than silently reading the chamber live where the reference reads the
/// window.
#[test]
fn a_wrap_before_the_move_is_refused() {
    let scenario = sealed_station_scenario();
    let (state, bio, fast) = built(&scenario);
    let bio = plants_read_chamber(bio, &state.stocks, &station_params::chamber()).expect("wrap");
    let (bio, fast) =
        gas_exchange_on_fast_step(&state.stocks, bio, fast, &weather_shared(&scenario.bio))
            .expect("move");
    let after = EulerIntegrator::new(bio).step(&state, &resolver(&scenario), scenario.bio_dt);
    // The recorder joined the plant step after the wrap, so it is unwrapped and finds no `temp`
    // in the plants' resolver; had it found one, the moved flows would refuse it.
    let err = match after {
        Err(e) => e.to_string(),
        Ok(s) => {
            let budget = fast
                .flows()
                .iter()
                .find(|f| f.id() == CARBON_BUDGET_FLOWS[0])
                .unwrap();
            budget
                .evaluate(&s, &SourceResolver::empty().bind(&s, 60.0), 60.0)
                .expect_err("a wrapped flow on the minute step is refused")
                .to_string()
        }
    };
    assert!(err.contains("temp"), "{err}");
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

/// The reference IS the minute step since 2026-10-03: `build_sealed_station` hands back the
/// carbon budget on the fast registry and the recorder on the plant one.
#[test]
fn the_reference_sealed_station_takes_its_gas_exchange_on_the_minute() {
    let scenario = sealed_station_scenario();
    let (_, bio, fast) = build_sealed_station(
        &params::charge(),
        &params::thermal(),
        &params::crew(),
        &params::eclss(),
        &station_params::water_recovery(),
        &station_params::lamp(),
        &station_params::harvest(),
        &scenario,
        false,
        false,
    )
    .expect("build_sealed_station");
    for id in CARBON_BUDGET_FLOWS {
        assert!(
            fast.flows().iter().any(|f| f.id() == id),
            "{id} not on the minute step"
        );
        assert!(
            bio.flows().iter().all(|f| f.id() != id),
            "{id} still on the plant step"
        );
    }
    assert!(bio
        .aux_processes()
        .iter()
        .any(|a| a.id() == PLANT_WINDOW_RECORDER));
}

/// ⚠ **THE DISCARDED-REGISTRY TRAP, found by a missed prediction (2026-10-03).** A builder that
/// reuses another's plant registry and REBUILDS the fast one (`build_harvest`, the separate-air
/// build) would throw a minute-step carbon budget away with the registry it discards — the
/// crop then grows nothing at all, conserving perfectly. Measured once: `harvest`'s plant carbon
/// fell 17.6 %. So every crop build is held to stepping each carbon-budget flow EXACTLY once,
/// across its two registries, on the side its form names.
#[test]
fn every_crop_build_steps_the_carbon_budget_exactly_once() {
    use station::air_split::{build_split_station, AirSplit, BVAD_CHAMBER_AIR_MOL};
    use station::greenhouse::build_greenhouse_at;
    use station::harvest::build_harvest;
    use station::scenario::{greenhouse_scenario, harvest_scenario};

    let crew = params::crew();
    let eclss = params::eclss();
    let sealed = sealed_station_scenario();
    let check = |name: &str, bio: &Registry, fast: &Registry, minute: bool| {
        for id in CARBON_BUDGET_FLOWS {
            let on_bio = bio.flows().iter().filter(|f| f.id() == id).count();
            let on_fast = fast.flows().iter().filter(|f| f.id() == id).count();
            assert_eq!(
                on_bio + on_fast,
                1,
                "{name}: {id} stepped {} times",
                on_bio + on_fast
            );
            assert_eq!(on_fast == 1, minute, "{name}: {id} on the wrong side");
        }
    };
    for gas in [GasExchangeStep::PlantStep, GasExchangeStep::Minute] {
        let minute = gas == GasExchangeStep::Minute;
        let (_, bio, fast) = build_greenhouse_at(
            &crew,
            &eclss,
            &greenhouse_scenario(),
            true,
            domains::crew::FECAL_WASTE,
            gas,
        )
        .unwrap();
        check("greenhouse", &bio, &fast, minute);
        let (_, bio, fast) = build_sealed_station_at(
            &params::charge(),
            &params::thermal(),
            &crew,
            &eclss,
            &station_params::water_recovery(),
            &station_params::lamp(),
            &station_params::harvest(),
            &sealed,
            true,
            false,
            gas,
        )
        .unwrap();
        check("sealed", &bio, &fast, minute);
        let split = AirSplit {
            chamber_air_mol: BVAD_CHAMBER_AIR_MOL,
            fan_mol_per_s: 0.2,
            vapour_crosses: true,
            gas_exchange: gas,
            watering: false,
            transpiration: GasExchangeStep::PlantStep,
        };
        let (_, bio, fast) = build_split_station(
            &params::charge(),
            &params::thermal(),
            &crew,
            &eclss,
            &station_params::water_recovery(),
            &station_params::lamp(),
            &station_params::harvest(),
            &sealed,
            &split,
        )
        .unwrap();
        check("split", &bio, &fast, minute);
    }
    let (_, bio, fast) = build_harvest(
        &crew,
        &eclss,
        &station_params::harvest(),
        &harvest_scenario(),
        true,
        false,
    )
    .unwrap();
    check("harvest (the reference)", &bio, &fast, true);
}
