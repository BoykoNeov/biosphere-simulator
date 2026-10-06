//! The sowing clock — Step 3c slice 4, stage 1 (`docs/plans/post-roadmap-room-temperature.md`
//! §25c). The cold program is read from the state's own sowing ([`SOWN_STEP`]) through a
//! wrapper on every flow and aux process, from two phase twins per variable in the resolvers.
//!
//! ⚠ **Why these tests exist beyond the golden.** In stage 1 the re-sow is still on the
//! calendar, so the state's sowing and the calendar agree on every step of every frozen run: a
//! reader left on the calendar would reproduce the golden too (advisor, §25a item 3). Only a
//! sowing OFF the calendar tells the two apart — P3 here. P4 is the other half: a reader that
//! escaped the wrapper fails loudly instead of reading one phase.

use std::cell::Cell;
use std::collections::BTreeMap;

use domains::biosphere::stocks::{DAYLENGTH_VAR, PAR_VAR, RN_VAR};
use domains::power::BATTERY;
use simcore::auxiliary::AuxProcess;
use simcore::environment::{constant, Environment, SourceResolver};
use simcore::error::SimError;
use simcore::integrator::EulerIntegrator;
use simcore::registry::Registry;
use simcore::state::State;
use station::chamber::{plants_read_chamber, ChamberSurroundings, CHAMBER_SETPOINT_VAR};
use station::flows::{LAMP, LAMP_POWER_VAR};
use station::gas_exchange::{window_key, GasExchangeStep, PLANT_WINDOW_RECORDER};
use station::lamp_shed::{rewire_for_shedding, LAMP_DELIVERY_AUX};
use station::scenario::{sealed_station_scenario, Phase, SealedStationScenario};
use station::sealed::{
    build_sealed_station, build_sealed_station_unread, sealed_bio_resolver, sealed_fast_resolver,
};
use station::sowing::{
    on_sowing_clock, twin, FAST_PROGRAM_VARS, PLANT_PROGRAM_VARS, SOWN_STEP,
};

const PER_DAY: u64 = domains::biosphere::STEPS_PER_DAY as u64;

fn built() -> (State, Registry, Registry) {
    build_sealed_station(
        &domains::params::charge(),
        &domains::params::thermal(),
        &domains::params::crew(),
        &domains::params::eclss(),
        &station::params::water_recovery(),
        &station::params::lamp(),
        &station::params::harvest(),
        &sealed_station_scenario(),
        false,
        false,
    )
    .expect("build")
}

fn resolvers(scenario: &SealedStationScenario) -> (SourceResolver, SourceResolver) {
    (
        sealed_bio_resolver(&station::params::lamp(), scenario).expect("bio resolver"),
        sealed_fast_resolver(&domains::params::charge(), scenario).expect("fast resolver"),
    )
}

/// `state` moved to step `n` with its crop sown at step `sown`.
fn sown_at(state: &State, n: u64, sown: u64) -> State {
    let mut aux = state.aux.clone();
    aux.insert(SOWN_STEP.to_string(), sown as f64);
    State::new(n, state.stocks.clone(), state.rng_seed, aux).expect("state")
}

/// P3 — **off the calendar**, every program variable on both sides follows the state's sowing.
/// A crop sown on day 10 is cold through day 65 and warm from day 66; the calendar would have
/// it warm from day 56. Each variable's two twins differ at midday, so a reader on the wrong
/// clock would read the other value.
#[test]
fn a_crop_sown_off_the_calendar_moves_its_cold_period_on_both_sides() {
    let scenario = sealed_station_scenario();
    let (state, _, _) = built();
    let midday = |day: u64| day * PER_DAY + PER_DAY / 2;
    let sown = 10 * PER_DAY;
    let (bio_r, fast_r) = resolvers(&scenario);
    for var in PLANT_PROGRAM_VARS.into_iter().chain(FAST_PROGRAM_VARS) {
        let (r, dt) = if PLANT_PROGRAM_VARS.contains(&var) {
            (&bio_r, scenario.bio_dt)
        } else {
            (&fast_r, scenario.cabin_dt)
        };
        let twin_at = |phase, n| {
            let s = sown_at(&state, n, sown);
            r.bind(&s, dt).get(&twin(var, phase)).unwrap()
        };
        for (day, phase) in [(60, Phase::Cold), (65, Phase::Cold), (66, Phase::Warm)] {
            let s = sown_at(&state, midday(day), sown);
            assert_eq!(scenario.phase(&s).unwrap(), phase, "{var} day {day}");
            let seen = read_through_wrapper(var, &s, &scenario);
            assert_eq!(
                seen.to_bits(),
                twin_at(phase, midday(day)).to_bits(),
                "{var} on day {day}: the wrapper did not read the {phase:?} twin"
            );
            let other = match phase {
                Phase::Cold => Phase::Warm,
                Phase::Warm => Phase::Cold,
            };
            assert_ne!(
                seen,
                twin_at(other, midday(day)),
                "{var} on day {day}: the two twins agree, so this pin proves nothing"
            );
        }
        // The same day on the calendar's clock (sown at 0) is warm: day 60 differs.
        let calendar = sown_at(&state, midday(60), 0);
        assert_eq!(
            read_through_wrapper(var, &calendar, &scenario).to_bits(),
            twin_at(Phase::Warm, midday(60)).to_bits(),
            "{var}: sown at 0, day 60 is warm"
        );
    }
}

/// What a reader of `var` sees on `state` through the wrapper the sealed build applies: a
/// probe aux process, wrapped by [`on_sowing_clock`], reports what its environment answered.
fn read_through_wrapper(var: &'static str, state: &State, scenario: &SealedStationScenario) -> f64 {
    let (bio_r, fast_r) = resolvers(scenario);
    let plant_side = PLANT_PROGRAM_VARS.contains(&var);
    let seen = std::rc::Rc::new(Cell::new(f64::NAN));
    struct Shared {
        var: &'static str,
        seen: std::rc::Rc<Cell<f64>>,
    }
    impl AuxProcess for Shared {
        fn type_name(&self) -> &'static str {
            "Probe"
        }
        fn id(&self) -> &str {
            "probe"
        }
        fn evaluate(
            &self,
            _snapshot: &State,
            env: &dyn Environment,
            _dt: f64,
        ) -> Result<BTreeMap<String, f64>, SimError> {
            self.seen.set(env.get(self.var)?);
            Ok(BTreeMap::new())
        }
    }
    let reg = |aux: Vec<Box<dyn AuxProcess>>| Registry::new(vec![], &state.stocks, aux).unwrap();
    let probe: Vec<Box<dyn AuxProcess>> = vec![Box::new(Shared {
        var,
        seen: seen.clone(),
    })];
    let (bio, fast) = if plant_side {
        on_sowing_clock(reg(probe), reg(vec![]), &state.stocks, scenario).unwrap()
    } else {
        on_sowing_clock(reg(vec![]), reg(probe), &state.stocks, scenario).unwrap()
    };
    let (reg, resolver, dt) = if plant_side {
        (bio, bio_r, scenario.bio_dt)
    } else {
        (fast, fast_r, scenario.cabin_dt)
    };
    let p = &reg.aux_processes()[0];
    assert_eq!((p.type_name(), p.id()), ("Probe", "probe"), "the wrapper keeps identity");
    p.evaluate(state, &resolver.bind(state, dt), dt).unwrap();
    seen.get()
}

/// P3, on the REAL build: the built registries' own readers follow the state's sowing — the
/// plant window recorder records the cold `par` on day 60 of a crop sown on day 10, and the
/// lamp draws the dimmed cold power from the battery; sown at 0, both are the warm phase's.
#[test]
fn the_built_registries_read_the_states_sowing() {
    let scenario = sealed_station_scenario();
    let (state, bio_reg, fast_reg) = built();
    let (bio_r, fast_r) = resolvers(&scenario);
    let n = 60 * PER_DAY + PER_DAY / 2;
    let recorder = bio_reg
        .aux_processes()
        .iter()
        .find(|a| a.id() == PLANT_WINDOW_RECORDER)
        .expect("the window recorder");
    let lamp = fast_reg
        .flows()
        .iter()
        .find(|f| f.id() == LAMP)
        .expect("the lamp");
    let mut seen = BTreeMap::new();
    for (sown, phase) in [(10 * PER_DAY, Phase::Cold), (0, Phase::Warm)] {
        let mut s = sown_at(&state, n, sown);
        s.aux.remove(&window_key(PAR_VAR));
        let rec = recorder
            .evaluate(&s, &bio_r.bind(&s, scenario.bio_dt), scenario.bio_dt)
            .unwrap();
        let par = rec[&window_key(PAR_VAR)];
        assert_eq!(
            par.to_bits(),
            bio_r
                .bind(&s, scenario.bio_dt)
                .get(&twin(PAR_VAR, phase))
                .unwrap()
                .to_bits(),
            "the recorder did not record the {phase:?} par"
        );
        let legs = lamp
            .evaluate(&s, &fast_r.bind(&s, scenario.cabin_dt), scenario.cabin_dt)
            .unwrap();
        let drawn = -legs.legs.iter().find(|l| l.stock == BATTERY).unwrap().amount;
        let want = scenario.lamp_average_power(phase, station::params::lamp().photon_efficacy)
            * scenario.cabin_dt;
        assert_eq!(drawn.to_bits(), want.to_bits(), "the lamp drew the wrong phase");
        seen.insert(format!("{phase:?}"), (par, drawn));
    }
    assert!(seen["Cold"].0 < seen["Warm"].0 && seen["Cold"].1 < seen["Warm"].1);
}

/// P4 — a sealed registry run WITHOUT the clock errors at its first read of a program variable
/// (the plain name is in no sealed resolver), rather than reading one phase.
#[test]
fn a_build_off_the_clock_fails_loudly() {
    let scenario = sealed_station_scenario();
    let (state, bio_reg, _fast) = build_sealed_station_unread(
        &domains::params::charge(),
        &domains::params::thermal(),
        &domains::params::crew(),
        &domains::params::eclss(),
        &station::params::water_recovery(),
        &station::params::lamp(),
        &station::params::harvest(),
        &scenario,
        false,
        false,
        GasExchangeStep::Minute,
        ChamberSurroundings::Outdoor,
    )
    .expect("build");
    let bio_reg = plants_read_chamber(bio_reg, &state.stocks, &station::params::chamber())
        .expect("chamber wrap");
    let (bio_r, _) = resolvers(&scenario);
    let err = EulerIntegrator::new(bio_reg)
        .step(&state, &bio_r, scenario.bio_dt)
        .expect_err("an unwrapped plant registry must not step");
    let msg = err.to_string();
    assert!(
        PLANT_PROGRAM_VARS.iter().any(|v| msg.contains(&format!("{v:?}"))),
        "{msg}"
    );
}

/// P4 — a resolver that carries a program variable under its plain name beside the clock is
/// refused at the first read: the wrapper would otherwise silently override it.
#[test]
fn a_plain_program_forcing_beside_the_clock_is_refused() {
    let scenario = sealed_station_scenario();
    let (state, bio_reg, _) = built();
    let (bio_r, _) = resolvers(&scenario);
    let (mut forcings, shared) = bio_r.into_parts();
    forcings.insert(DAYLENGTH_VAR.to_string(), constant(1.0).unwrap());
    let bio_r = SourceResolver::new(forcings, shared).unwrap();
    let err = EulerIntegrator::new(bio_reg)
        .step(&state, &bio_r, scenario.bio_dt)
        .expect_err("a plain daylength beside its twins must be refused");
    assert!(matches!(err, SimError::Validation(_)), "{err}");
    assert!(err.to_string().contains("silently overridden"), "{err}");
    let _ = (RN_VAR, LAMP_POWER_VAR, CHAMBER_SETPOINT_VAR);
}

/// THE ORDER (§25c): the lab lamp shed rewires a build that is already on the clock, so its
/// `LampLitEnv` sits OUTSIDE the clock and is asked for the twins. It must dim them: at half
/// delivery the recorder records half the cold phase's `par`.
#[test]
fn the_shed_lamp_dims_the_twin_the_clock_selects() {
    let scenario = sealed_station_scenario();
    let (state, bio_reg, fast_reg) = built();
    let (state, bio_reg, _) = rewire_for_shedding(state, bio_reg, fast_reg, 0.0).unwrap();
    let (bio_r, _) = resolvers(&scenario);
    let n = 20 * PER_DAY + PER_DAY / 2;
    let mut s = sown_at(&state, n, 0);
    s.aux.insert(LAMP_DELIVERY_AUX.to_string(), 0.5);
    s.aux.remove(&window_key(PAR_VAR));
    let recorder = bio_reg
        .aux_processes()
        .iter()
        .find(|a| a.id() == PLANT_WINDOW_RECORDER)
        .expect("the window recorder");
    let rec = recorder
        .evaluate(&s, &bio_r.bind(&s, scenario.bio_dt), scenario.bio_dt)
        .unwrap();
    let full = bio_r
        .bind(&s, scenario.bio_dt)
        .get(&twin(PAR_VAR, Phase::Cold))
        .unwrap();
    assert!(full > 0.0, "midday of a cold day is lit");
    assert_eq!(rec[&window_key(PAR_VAR)], 0.5 * full);
}
