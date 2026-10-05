//! The plants' water loss on the minute step — the move's traps, pinned (LAB-ONLY; plan
//! `docs/plans/post-roadmap-room-temperature.md` §20; `station::air_split::water_on_fast_step`).
//!
//! The traps are those of the carbon budget's move (`tests/gas_exchange.rs`): a forcing read
//! for the wrong window, a daily rate multiplied by seconds, a flow left on both steps, a
//! plant-side change that never reaches the minute step. Plus one of this move's own: the aux
//! channel is additive, so a second recorder writing the temperature key would double it.
//! None of these breaks conservation. The season-level grade is the example
//! `cargo run --release -q -p station --example minute_transpiration`.

use std::collections::HashMap;

use domains::biosphere::science::humidity_target_kg;
use domains::biosphere::stocks::{RN_VAR, TEMP_VAR, WATER_VAPOR};
use domains::params;
use simcore::environment::{constant, Environment, SourceResolver};
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult};
use simcore::integrator::EulerIntegrator;
use simcore::registry::Registry;
use simcore::state::State;
use station::air_split::{
    build_split_station, split_scenario, water_on_fast_step, AirSplit, BVAD_CHAMBER_AIR_MOL,
    NET_RADIATION_RECORDER, WATER_LOSS_FLOWS,
};
use station::driver::run_master_day;
use station::gas_exchange::{
    window_key, GasExchangeStep, CARBON_BUDGET_FLOWS, PLANT_WINDOW_RECORDER,
};
use station::params as station_params;
use station::scenario::{sealed_station_scenario, SealedStationScenario};
use station::sealed::{sealed_bio_resolver, sealed_fast_resolver};

fn split(gas_exchange: GasExchangeStep, transpiration: GasExchangeStep) -> AirSplit {
    AirSplit {
        chamber_air_mol: BVAD_CHAMBER_AIR_MOL,
        fan_mol_per_s: 0.2,
        vapour_crosses: true,
        gas_exchange,
        watering: false,
        transpiration,
    }
}

fn build(s: &AirSplit) -> Result<(State, Registry, Registry), SimError> {
    build_split_station(
        &params::charge(),
        &params::thermal(),
        &params::crew(),
        &params::eclss(),
        &station_params::water_recovery(),
        &station_params::lamp(),
        &station_params::harvest(),
        &sealed_station_scenario(),
        s,
    )
}

/// The minute-water build and the scenario its resolvers are built from.
fn minute_build() -> (SealedStationScenario, State, Registry, Registry) {
    let s = split(GasExchangeStep::Minute, GasExchangeStep::Minute);
    let (state, bio, fast) = build(&s).expect("minute-water build");
    (
        split_scenario(&sealed_station_scenario(), &s),
        state,
        bio,
        fast,
    )
}

fn plant_resolver(scenario: &SealedStationScenario, held_22: bool) -> SourceResolver {
    let r = sealed_bio_resolver(&station_params::lamp(), scenario).expect("bio resolver");
    if !held_22 {
        return r;
    }
    let (mut forcings, shared) = r.into_parts();
    forcings.insert(TEMP_VAR.to_string(), constant(22.0).unwrap());
    SourceResolver::new(forcings, shared).unwrap()
}

/// A resolver serving exactly `temp` and `net radiation` — what the raw flows read on the
/// plant step, set to the recorded window.
fn window_resolver(temp: f64, rn: f64) -> SourceResolver {
    let forcings = HashMap::from([
        (TEMP_VAR.to_string(), constant(temp).unwrap()),
        (RN_VAR.to_string(), constant(rn).unwrap()),
    ]);
    SourceResolver::new(forcings, HashMap::new()).unwrap()
}

fn flow<'a>(reg: &'a Registry, id: &str) -> &'a dyn Flow {
    reg.flows()
        .iter()
        .find(|f| f.id() == id)
        .unwrap_or_else(|| panic!("{id} is not in this registry"))
        .as_ref()
}

fn assert_bits(got: &FlowResult, want: &FlowResult, what: &str) {
    assert_eq!(got.legs.len(), want.legs.len(), "{what}");
    for (g, w) in got.legs.iter().zip(&want.legs) {
        assert_eq!(g.stock, w.stock, "{what}");
        assert_eq!(
            g.amount.to_bits(),
            w.amount.to_bits(),
            "{what}: {}",
            g.stock
        );
    }
}

/// THE WINDOW, and THE ADDITIVE CHANNEL. After each plant step from `n`, the recorded net
/// radiation is the resolver's at `n` (within an ULP), over two days — and the temperature and
/// PAR keys are still the window's values, not twice them (a second writer of either would
/// double it). Meaningful only because the net radiation changes between windows (asserted: it
/// is a per-day table, so it changes at the day boundary).
#[test]
fn the_plant_step_records_the_windows_net_radiation_and_nothing_twice() {
    let (scenario, state, bio, _) = minute_build();
    let plant = EulerIntegrator::new(bio);
    let reference = plant_resolver(&scenario, false);
    let mut s = state;
    let mut changes = 0;
    let mut previous = None;
    for n in 0..(2 * scenario.bio_steps_per_day) {
        let env = reference.bind(&s, scenario.bio_dt);
        let want: Vec<(String, f64)> = [RN_VAR, TEMP_VAR, domains::biosphere::stocks::PAR_VAR]
            .iter()
            .map(|v| (window_key(v), env.get(v).expect("forcing")))
            .collect();
        s = plant
            .step(&s, &reference, scenario.bio_dt)
            .expect("plant step");
        for (key, w) in &want {
            let got = s.aux[key];
            let ulps = (got.to_bits() as i64 - w.to_bits() as i64).abs();
            assert!(ulps <= 1, "n = {n}, {key}: recorded {got}, window {w}");
        }
        changes += u32::from(previous.is_some_and(|p| p != want[0].1));
        previous = Some(want[0].1);
    }
    assert!(
        changes > 0,
        "the net radiation never changed between windows"
    );
}

/// THE UNIT, and THE WINDOW AS READ. After a day on the minute build, each wrapped flow's legs
/// are the raw flow's, evaluated against the recorded window with `dt` in DAYS — bit for bit.
/// Both must move something, or the comparison is of zeros.
#[test]
fn the_wrapped_pair_is_the_raw_pair_on_a_minute_in_days() {
    let (scenario, state, bio, fast) = minute_build();
    let plant_r = plant_resolver(&scenario, false);
    let fast_r = sealed_fast_resolver(&params::charge(), &scenario).unwrap();
    let (states, rationed, _) = run_master_day(
        &EulerIntegrator::new(bio),
        &EulerIntegrator::new(fast),
        state,
        &plant_r,
        &fast_r,
        1,
        scenario.steps_per_day,
        scenario.bio_steps_per_day,
        scenario.bio_dt,
        scenario.cabin_dt,
        None,
    )
    .expect("one day");
    assert_eq!(rationed, 0);
    let s = states.last().unwrap();
    let (_, raw_bio, _) =
        build(&split(GasExchangeStep::Minute, GasExchangeStep::PlantStep)).expect("raw build");
    let (_, _, _, wrapped_fast) = minute_build();
    let window = window_resolver(s.aux[&window_key(TEMP_VAR)], s.aux[&window_key(RN_VAR)]);
    let fast_dt = scenario.cabin_dt;
    for id in WATER_LOSS_FLOWS {
        let got = flow(&wrapped_fast, id)
            .evaluate(s, &SourceResolver::empty().bind(s, fast_dt), fast_dt)
            .expect("wrapped");
        let want = flow(&raw_bio, id)
            .evaluate(s, &window.bind(s, fast_dt), fast_dt / 86_400.0)
            .expect("raw");
        assert_bits(&got, &want, id);
        assert!(
            got.legs.iter().any(|l| l.amount != 0.0),
            "{id} moved nothing, so the comparison was of zeros"
        );
    }
}

/// THE OVERRIDE: a plant-side 22 °C is what the minute step's condenser reads. After one plant
/// step under a held 22 °C, the wrapped `Condensation` equals the raw one at 22 °C bit for bit,
/// and differs from the raw one at the weather's temperature for that window (asserted ≠ 22).
#[test]
fn a_plant_side_22c_reaches_the_minute_steps_condenser() {
    let (scenario, state, bio, fast) = minute_build();
    let held = plant_resolver(&scenario, true);
    let weather_t = plant_resolver(&scenario, false)
        .bind(&state, scenario.bio_dt)
        .get(TEMP_VAR)
        .unwrap();
    assert_ne!(
        weather_t, 22.0,
        "the weather already reads 22 °C: the pin would prove nothing"
    );
    let s = EulerIntegrator::new(bio)
        .step(&state, &held, scenario.bio_dt)
        .expect("plant step");
    // Below its target the condenser's draw does not read temperature at all (`rate·dt·v`), so
    // the chamber's vapour is set BETWEEN the two temperatures' targets, where it does.
    let target = |t: f64| {
        humidity_target_kg(
            t,
            scenario.bio.chamber_air_capacity_mol,
            domains::biosphere::params::water_cycle().humidity_setpoint,
        )
    };
    let mut stocks = s.stocks.clone();
    stocks.get_mut(WATER_VAPOR).unwrap().amount = 0.5 * (target(weather_t) + target(22.0));
    let s = State::new(s.n, stocks, s.rng_seed, s.aux.clone()).unwrap();
    let (_, raw_bio, _) =
        build(&split(GasExchangeStep::Minute, GasExchangeStep::PlantStep)).expect("raw build");
    let rn = s.aux[&window_key(RN_VAR)];
    let fast_dt = scenario.cabin_dt;
    let condensation = WATER_LOSS_FLOWS[1];
    let got = flow(&fast, condensation)
        .evaluate(&s, &SourceResolver::empty().bind(&s, fast_dt), fast_dt)
        .expect("wrapped");
    let at = |t: f64| {
        flow(&raw_bio, condensation)
            .evaluate(
                &s,
                &window_resolver(t, rn).bind(&s, fast_dt),
                fast_dt / 86_400.0,
            )
            .expect("raw")
    };
    assert_bits(&got, &at(22.0), "condensation at the held 22 °C");
    let weather = at(weather_t);
    assert!(
        got.legs
            .iter()
            .zip(&weather.legs)
            .any(|(g, w)| g.amount.to_bits() != w.amount.to_bits()),
        "the condenser reads the same at 22 °C and at {weather_t} °C: the pin cannot see the override"
    );
}

/// THE DOUBLE COUNT: the pair moves whole — out of the plant registry, into the fast one, ids
/// and types kept — the net-radiation recorder joins the plant step, the carbon budget stays
/// stepped exactly once, and on the plant-step form nothing moves.
#[test]
fn the_water_loss_pair_moves_whole_and_alone() {
    let count = |reg: &Registry, id: &str| reg.flows().iter().filter(|f| f.id() == id).count();
    let has_aux = |reg: &Registry, id: &str| reg.aux_processes().iter().any(|a| a.id() == id);

    let (_, bio_p, fast_p) =
        build(&split(GasExchangeStep::Minute, GasExchangeStep::PlantStep)).unwrap();
    let (_, _, bio_m, fast_m) = minute_build();
    assert_eq!(bio_m.flows().len(), bio_p.flows().len() - 2);
    assert_eq!(fast_m.flows().len(), fast_p.flows().len() + 2);
    let types = [
        ("biosphere.transpiration", "Transpiration"),
        ("biosphere.condensation", "Condensation"),
    ];
    for (id, type_name) in types {
        assert_eq!(
            (count(&bio_p, id), count(&fast_p, id)),
            (1, 0),
            "plant-step form: {id}"
        );
        assert_eq!(
            (count(&bio_m, id), count(&fast_m, id)),
            (0, 1),
            "minute form: {id}"
        );
        assert_eq!(flow(&fast_m, id).type_name(), type_name);
    }
    for id in CARBON_BUDGET_FLOWS {
        assert_eq!(
            (count(&bio_m, id), count(&fast_m, id)),
            (0, 1),
            "carbon budget: {id}"
        );
    }
    assert!(has_aux(&bio_m, NET_RADIATION_RECORDER));
    assert!(has_aux(&bio_m, PLANT_WINDOW_RECORDER));
    assert!(!has_aux(&bio_p, NET_RADIATION_RECORDER));
    assert!(fast_m.aux_processes().is_empty());
}

/// THE REFUSALS, at build time: water on the minute step with gas exchange on the plant step
/// (nothing would record the window's temperature), and a plant registry missing half the pair.
#[test]
fn the_move_refuses_what_it_cannot_do_whole() {
    let err = build(&split(GasExchangeStep::PlantStep, GasExchangeStep::Minute))
        .err()
        .expect("water on the minute with plant-step gas exchange must be refused");
    assert!(err.to_string().contains("gas_exchange"), "{err}");

    let (state, bio, fast) =
        build(&split(GasExchangeStep::Minute, GasExchangeStep::PlantStep)).unwrap();
    let (flows, aux) = bio.into_parts();
    let flows: Vec<_> = flows
        .into_iter()
        .filter(|f| f.id() != WATER_LOSS_FLOWS[1])
        .collect();
    let half = Registry::new(flows, &state.stocks, aux).unwrap();
    let scenario = split_scenario(
        &sealed_station_scenario(),
        &split(GasExchangeStep::Minute, GasExchangeStep::PlantStep),
    );
    let err = water_on_fast_step(
        &state.stocks,
        half,
        fast,
        &domains::biosphere::system::weather_shared(&scenario.bio),
    )
    .err()
    .expect("half the pair must be refused");
    assert!(err.to_string().contains("together"), "{err}");
}
