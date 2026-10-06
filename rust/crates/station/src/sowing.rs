//! The sealed station's sowing clock — Step 3c slice 4
//! (`docs/plans/post-roadmap-room-temperature.md` §25).
//!
//! The plant chamber's cold program runs from each **sowing**, not from a calendar. The sowing
//! is state: the aux entry [`SOWN_STEP`], the slow step count `n` at which the standing crop was
//! sown — seeded by the sealed build and rewritten by the sealed re-sow hook
//! ([`crate::sealed::sealed_reset_hook`]). The phase it gives is
//! [`SealedStationScenario::phase`].
//!
//! # Why twins, and a wrapper
//!
//! A forcing is a pure function of `n` (`simcore::environment::Schedule`) and cannot read the
//! state, and simcore is outside every unfreeze path. So each variable the program drives —
//! [`PLANT_PROGRAM_VARS`] on the plant step, [`FAST_PROGRAM_VARS`] on the fast one — is carried
//! in its resolver as two plain schedules, [`twin`]`(v, Cold)` and [`twin`]`(v, Warm)`, and a
//! wrapper on every flow and aux process ([`SowingClockFlow`], [`SowingClockAux`]) answers `v`
//! with the twin of the state's phase. The perturbations still compose schedules in the
//! resolver; they change both twins ([`crate::perturbations`]).
//!
//! ⚠ **The plain name is in no sealed resolver.** A reader that escaped the wrapper errors at
//! its first read rather than silently reading one phase; and the wrapper **refuses** an inner
//! environment that answers the plain name itself (a forcing it would otherwise override).
//!
//! ⚠ **Apply it last** ([`on_sowing_clock`]): the environment it builds must be the one every
//! inner wrapper reads through. A wrapper applied after it sees the twin names, not `v` — the
//! lab lamp shed's `LampLitEnv` therefore dims the twins too.

use std::collections::BTreeMap;

use domains::biosphere::stocks::{DAYLENGTH_VAR, PAR_VAR, RN_VAR};
use simcore::auxiliary::AuxProcess;
use simcore::environment::{Environment, Schedule, SourceResolver};
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult};
use simcore::registry::Registry;
use simcore::ids::StockId;
use simcore::state::{State, Stock};

use crate::chamber::CHAMBER_SETPOINT_VAR;
use crate::flows::LAMP_POWER_VAR;
use crate::scenario::{Phase, SealedStationScenario};

/// The aux entry holding the slow step `n` at which the standing crop was sown.
pub const SOWN_STEP: &str = "station.sown_step";

/// The plant-step variables the cold program drives (the lamp's light and day).
pub const PLANT_PROGRAM_VARS: [&str; 3] = [PAR_VAR, RN_VAR, DAYLENGTH_VAR];

/// The fast-step variables the cold program drives (the lamp's draw, the chamber's setpoint).
pub const FAST_PROGRAM_VARS: [&str; 2] = [LAMP_POWER_VAR, CHAMBER_SETPOINT_VAR];

/// The resolver name of `var`'s schedule in `phase`: `var@cold` / `var@warm`.
pub fn twin(var: &str, phase: Phase) -> String {
    match phase {
        Phase::Cold => format!("{var}@cold"),
        Phase::Warm => format!("{var}@warm"),
    }
}

/// The program variable a [`twin`] name stands for (`par@cold` → `par`); any other name as is.
pub fn untwin(var: &str) -> &str {
    var.strip_suffix("@cold")
        .or_else(|| var.strip_suffix("@warm"))
        .unwrap_or(var)
}

/// The step at which `state`'s crop was sown ([`SOWN_STEP`]). An error if the state carries
/// none — never a default, the omission `vernalization_days` once had at the sealed build — or
/// one that is not a whole, non-negative step count.
pub fn sown_step(state: &State) -> Result<u64, SimError> {
    let v = state.aux.get(SOWN_STEP).copied().ok_or_else(|| {
        SimError::Reference(format!(
            "the cold program's clock reads {SOWN_STEP:?}, which the state does not carry \
             (a sealed build seeds it; the sealed re-sow hook rewrites it)"
        ))
    })?;
    if !(v >= 0.0 && v.fract() == 0.0 && v < 2f64.powi(53)) {
        return Err(SimError::Validation(format!(
            "{SOWN_STEP:?} must be a whole, non-negative step count, got {v:?}"
        )));
    }
    Ok(v as u64)
}

/// `state` with its sowing set to its own step — what a sowing writes.
pub fn sow_now(state: State) -> Result<State, SimError> {
    let mut aux = state.aux;
    aux.insert(SOWN_STEP.to_string(), state.n as f64);
    State::new(state.n, state.stocks, state.rng_seed, aux)
}

/// Insert `var`'s two phase schedules into `forcings` under their [`twin`] names.
pub fn insert_twins(
    forcings: &mut std::collections::HashMap<String, Schedule>,
    var: &str,
    cold: Schedule,
    warm: Schedule,
) {
    forcings.insert(twin(var, Phase::Cold), cold);
    forcings.insert(twin(var, Phase::Warm), warm);
}

/// Replace `var`'s schedule in `forcings` by `f` of it — or, where `var` is carried as its two
/// phase [`twin`]s, both of them, so a perturbation reaches the cold period as well as the warm
/// one. Refuses a map carrying `var` beside a twin (ambiguous), or neither (the perturbation's
/// target must exist).
pub fn map_program_or_plain(
    forcings: &mut std::collections::HashMap<String, Schedule>,
    var: &str,
    f: impl Fn(Schedule) -> Schedule,
) -> Result<(), SimError> {
    let twins = Phase::BOTH.map(|phase| twin(var, phase));
    let carried = twins.iter().filter(|t| forcings.contains_key(*t)).count();
    match (forcings.contains_key(var), carried) {
        (true, 0) => {
            let base = forcings.remove(var).expect("checked present");
            forcings.insert(var.to_string(), f(base));
            Ok(())
        }
        (false, 2) => {
            for t in twins {
                let base = forcings.remove(&t).expect("checked present");
                forcings.insert(t, f(base));
            }
            Ok(())
        }
        (false, 0) => Err(SimError::Reference(format!(
            "perturbation target forcing var {var:?} is absent from the resolver (neither it \
             nor its phase twins)"
        ))),
        _ => Err(SimError::Validation(format!(
            "forcing var {var:?} is carried inconsistently (plain: {}, twins: {carried} of 2): \
             a perturbation would reach only part of it",
            forcings.contains_key(var)
        ))),
    }
}

/// What a wrapped flow reads: each of `vars` is the twin of the state's phase; every other
/// variable the inner environment's.
struct SowingClockEnv<'a> {
    inner: &'a dyn Environment,
    snapshot: &'a State,
    scenario: &'a SealedStationScenario,
    vars: &'static [&'static str],
}

impl Environment for SowingClockEnv<'_> {
    fn get(&self, var: &str) -> Result<f64, SimError> {
        if !self.vars.contains(&var) {
            return self.inner.get(var);
        }
        if let Ok(wired) = self.inner.get(var) {
            return Err(SimError::Validation(format!(
                "the cold program answers {var:?} from the sowing clock, but the environment \
                 also answers it ({wired}): a plain forcing would be silently overridden. Carry \
                 it as its two phase twins ({:?}, {:?})",
                twin(var, Phase::Cold),
                twin(var, Phase::Warm)
            )));
        }
        let phase = self.scenario.phase(self.snapshot)?;
        self.inner.get(&twin(var, phase))
    }
}

/// A flow that reads the cold program through the sowing clock. Keeps the inner flow's type
/// name, id and priority: it is the same flow under the same program, read from the state's
/// sowing rather than the calendar — and the station manifest's flow set is keyed on type
/// names (the [`crate::chamber::PlantsReadChamberFlow`] precedent).
pub struct SowingClockFlow {
    inner: Box<dyn Flow>,
    scenario: SealedStationScenario,
    vars: &'static [&'static str],
}

impl Flow for SowingClockFlow {
    fn type_name(&self) -> &'static str {
        self.inner.type_name()
    }

    fn id(&self) -> &str {
        self.inner.id()
    }

    fn priority(&self) -> i64 {
        self.inner.priority()
    }

    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let env = SowingClockEnv {
            inner: env,
            snapshot,
            scenario: &self.scenario,
            vars: self.vars,
        };
        self.inner.evaluate(snapshot, &env, dt)
    }
}

/// An aux process that reads the cold program through the sowing clock. Keeps the inner
/// process's type name and id.
pub struct SowingClockAux {
    inner: Box<dyn AuxProcess>,
    scenario: SealedStationScenario,
    vars: &'static [&'static str],
}

impl AuxProcess for SowingClockAux {
    fn type_name(&self) -> &'static str {
        self.inner.type_name()
    }

    fn id(&self) -> &str {
        self.inner.id()
    }

    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<BTreeMap<String, f64>, SimError> {
        let env = SowingClockEnv {
            inner: env,
            snapshot,
            scenario: &self.scenario,
            vars: self.vars,
        };
        self.inner.evaluate(snapshot, &env, dt)
    }
}

/// Wrap every flow and aux process of `reg` so it reads `vars` through the sowing clock.
fn wrap(
    reg: Registry,
    stocks: &BTreeMap<StockId, Stock>,
    scenario: &SealedStationScenario,
    vars: &'static [&'static str],
) -> Result<Registry, SimError> {
    let scenario = *scenario;
    let (flows, aux) = reg.into_parts();
    let flows: Vec<Box<dyn Flow>> = flows
        .into_iter()
        .map(|inner| {
            Box::new(SowingClockFlow {
                inner,
                scenario,
                vars,
            }) as Box<dyn Flow>
        })
        .collect();
    let aux: Vec<Box<dyn AuxProcess>> = aux
        .into_iter()
        .map(|inner| {
            Box::new(SowingClockAux {
                inner,
                scenario,
                vars,
            }) as Box<dyn AuxProcess>
        })
        .collect();
    Registry::new(flows, stocks, aux)
}

/// Put both registries of a sealed build on the sowing clock: the plant registry reads
/// [`PLANT_PROGRAM_VARS`] through it, the fast registry [`FAST_PROGRAM_VARS`].
///
/// ⚠ Apply it **last** — after any move of plant flows onto the fast step and after
/// [`crate::chamber::plants_read_chamber`] (whose `temp` is disjoint, so their mutual order is
/// free). A flow moved after it would read the twins through the fast step's window.
pub fn on_sowing_clock(
    bio_reg: Registry,
    fast_reg: Registry,
    stocks: &BTreeMap<StockId, Stock>,
    scenario: &SealedStationScenario,
) -> Result<(Registry, Registry), SimError> {
    Ok((
        wrap(bio_reg, stocks, scenario, &PLANT_PROGRAM_VARS)?,
        wrap(fast_reg, stocks, scenario, &FAST_PROGRAM_VARS)?,
    ))
}

/// The value of program variable `var` on `state`, read straight from `resolver`'s twin for the
/// state's phase — for a driver that reads a forcing itself rather than through a flow (the lab
/// shedding driver's lamp draw).
pub fn program_value(
    resolver: &SourceResolver,
    var: &str,
    state: &State,
    scenario: &SealedStationScenario,
    dt: f64,
) -> Result<f64, SimError> {
    let phase = scenario.phase(state)?;
    resolver.bind(state, dt).get(&twin(var, phase))
}

#[cfg(test)]
mod tests {
    use super::*;
    use simcore::environment::constant;
    use std::collections::HashMap;

    fn state(n: u64, sown: Option<f64>) -> State {
        let aux = sown
            .map(|v| BTreeMap::from([(SOWN_STEP.to_string(), v)]))
            .unwrap_or_default();
        State::new(n, BTreeMap::new(), 0, aux).unwrap()
    }

    /// The clock reads whole, non-negative steps only, and a sowing writes the state's own `n`.
    #[test]
    fn the_sowing_is_a_whole_step_and_is_never_defaulted() {
        assert_eq!(sown_step(&state(7, Some(3.0))).unwrap(), 3);
        assert!(matches!(
            sown_step(&state(7, None)),
            Err(SimError::Reference(_))
        ));
        for bad in [-1.0, 2.5] {
            assert!(sown_step(&state(7, Some(bad))).is_err());
        }
        assert_eq!(sown_step(&sow_now(state(42, Some(3.0))).unwrap()).unwrap(), 42);
    }

    /// The environment selects the twin of the state's phase, passes other variables through,
    /// and refuses an inner environment that answers the plain name.
    #[test]
    fn the_clock_selects_the_twin_and_refuses_a_plain_forcing() {
        let scenario = crate::scenario::sealed_station_scenario();
        let per_day = scenario.bio_steps_per_day;
        let mut forcings: HashMap<String, Schedule> = HashMap::new();
        insert_twins(&mut forcings, PAR_VAR, constant(1.0).unwrap(), constant(2.0).unwrap());
        forcings.insert("other".to_string(), constant(9.0).unwrap());
        let resolver = SourceResolver::new(forcings, HashMap::new()).unwrap();
        let read = |n: u64, var: &str| {
            let s = state(n, Some(0.0));
            let inner = resolver.bind(&s, scenario.bio_dt);
            let env = SowingClockEnv {
                inner: &inner,
                snapshot: &s,
                scenario: &scenario,
                vars: &PLANT_PROGRAM_VARS,
            };
            env.get(var)
        };
        assert_eq!(read(0, PAR_VAR).unwrap(), 1.0);
        assert_eq!(read(56 * per_day, PAR_VAR).unwrap(), 2.0);
        assert_eq!(read(0, "other").unwrap(), 9.0);
        // A program variable with no twins is unknown, not zero.
        assert!(read(0, DAYLENGTH_VAR).is_err());

        let mut plain: HashMap<String, Schedule> = HashMap::new();
        insert_twins(&mut plain, PAR_VAR, constant(1.0).unwrap(), constant(2.0).unwrap());
        plain.insert(PAR_VAR.to_string(), constant(5.0).unwrap());
        let plain = SourceResolver::new(plain, HashMap::new()).unwrap();
        let s = state(0, Some(0.0));
        let inner = plain.bind(&s, scenario.bio_dt);
        let env = SowingClockEnv {
            inner: &inner,
            snapshot: &s,
            scenario: &scenario,
            vars: &PLANT_PROGRAM_VARS,
        };
        assert!(matches!(env.get(PAR_VAR), Err(SimError::Validation(_))));
    }
}
