//! **LAB-ONLY** — the grow lamp (and, since 2026-10-05, the plant chamber's heater) is shed at a
//! battery reserve, and the crop's light follows what the lamp actually got (the 2026-09-29
//! review's Step 3, slice 3a, option B).
//!
//! Plan and record: `docs/plans/post-roadmap-light-from-delivered-power.md`. Nothing in the
//! reference calls anything here: not a runner, not the session, not the bridge, not a golden.
//!
//! # Why it exists
//!
//! In the frozen station the crop's light is the lamp's **nameplate** (`par` is a pure
//! `Fn(n, dt)` schedule) and the `Lamp` flow draws from the battery separately, so a station
//! whose battery runs flat keeps its crop fully lit. Today the only thing that can give the
//! lamp less than nameplate is the arbitration backstop, which cuts life support and the lamp
//! by the same share. This module asks what the crop would see under an explicit lamp rule.
//!
//! # The rule — its FORM has a precedent, its NUMBER is WHAT-IF
//!
//! Form: Chung & Mazzocco, US patent 4,575,679 (General Electric, 1983) — the spacecraft load
//! is split into an *interruptible* load and an *uninterruptible* one, and *"the interruptible
//! load is shed when the battery system contains an amount of charge remaining that is
//! sufficient to enable the spacecraft system to survive until a positive battery charging
//! condition is achieved"*. Here the interruptible load is the lamp **and the plant chamber's
//! heater** ([`INTERRUPTIBLE_LOADS`]; the heater since 2026-10-05, the user's *"cut it with the
//! lamp"*, `docs/plans/post-roadmap-room-temperature.md` §23k–§23l) and life support
//! (`LoadDraw`) the uninterruptible one; both are **switched off**, not dimmed, on the same
//! battery reading. The heater follows the battery rule, not the lamp: a lamp that fails with a
//! healthy battery keeps its heater.
//!
//! ⚠ Untested: on a station whose battery recovers, switching back on brings up to the heater's
//! capacity of re-warming heat along with the lamp, and the rule has no latch, so the restore
//! can push the battery straight back under the reserve. The sealed station never restores.
//!
//! The rule is stateless: the lamp is on exactly when the battery holds at least the reserve
//! ([`lamp_on`]). Below the reserve it is off, and the battery can only climb back over the
//! reserve by charging, so "off until charging resumes" needs no stored latch. ⚠ That
//! restore reading is ours: the patent does not state a restore rule.
//!
//! Number: [`WHAT_IF_RESERVE_HOURS`]. No source gives a reserve for this station (the ISS's
//! 35 % depth of discharge was read and rejected: a battery-life limit, and a fraction of a
//! capacity this model does not have).
//!
//! # How the crop learns what the lamp got
//!
//! The lamp can switch inside the 90 cabin minutes that follow one plant step, so the crop
//! cannot sample the battery itself. [`run_shedding`] counts, over each power group, the lamp
//! power actually drawn against the nominal draw and writes the ratio into
//! `State.aux[`[`LAMP_DELIVERY_AUX`]`]` before the next plant step; a power step counts as lit
//! when the lamp draws from the battery on the step's starting state ([`lamp_draws`]; until
//! 2026-10-05, when its light arrived in `boundary.light_used`, a stock the sealed station no
//! longer has — its light now heats the plant chamber). Every slow flow and aux
//! process is wrapped ([`LampLitFlow`], [`LampLitAux`]) so that reading `par` (and, since
//! 2026-10-05, the lamp's `net_radiation`) returns the
//! schedule's value times that ratio. With nothing shed the two sums are the same numbers
//! added in the same order, so the ratio is exactly `1.0` and `x * 1.0 == x`: the rule-off
//! build reproduces the plain sealed run bit for bit (tested). The crop reads the light of
//! the **previous** group: a lag of one plant step (1/16 day).
//!
//! ⚠ A power step counts as lit when the lamp asks the battery for power, BEFORE arbitration,
//! so a lamp the backstop cut — partly or (since the 2026-10-05 detector) wholly — would count
//! as fully lit. The reserve exists so that the backstop never reaches the lamp; the tests
//! assert `rationed == 0` where they rely on it.

use std::collections::BTreeMap;

use domains::biosphere::stocks::{PAR_VAR, RN_VAR};
use domains::power::BATTERY;
use simcore::auxiliary::AuxProcess;
use simcore::conservation::assert_conserved_default;
use simcore::environment::{Environment, SourceResolver};
use simcore::error::SimError;
use simcore::events::Event;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::integrator::{EulerIntegrator, Substepper};
use simcore::registry::Registry;
use simcore::state::State;

use crate::chamber::CHAMBER_HEATER;
use crate::driver::{day_groups, DAYS_PER_MASTER_DAY, SECONDS_PER_DAY};
use crate::flows::{LAMP, LAMP_POWER_VAR};
use crate::scenario::SealedStationScenario;

/// The loads switched off below the reserve: the lamp and the plant chamber's heater.
pub const INTERRUPTIBLE_LOADS: [&str; 2] = [LAMP, CHAMBER_HEATER];

/// The aux slot carrying the share of the nominal lamp power drawn over the last power group.
pub const LAMP_DELIVERY_AUX: &str = "station.lab.lamp_delivery";

/// WHAT-IF — would the crop feel a power shortage if the lamp were shed at a reserve: the
/// reserve is this many hours of the essential (life-support) load. 24 h is a round number,
/// chosen, not sourced.
pub const WHAT_IF_RESERVE_HOURS: f64 = 24.0;

/// The reserve (J): `hours` of the essential load `essential_load_w`.
pub fn reserve_joules(hours: f64, essential_load_w: f64) -> f64 {
    hours * 3600.0 * essential_load_w
}

/// The shedding rule: the lamp is on exactly when the battery holds at least the reserve.
/// A reserve of `f64::NEG_INFINITY` never sheds (the rule-off control).
pub fn lamp_on(battery_j: f64, reserve_j: f64) -> bool {
    battery_j >= reserve_j
}

/// An interruptible load, shed below the reserve: every leg of the wrapped flow (`Lamp` or the
/// chamber heater) times `1.0` or `0.0`, so the result stays balanced.
pub struct SheddingLoad {
    inner: Box<dyn Flow>,
    reserve_j: f64,
}

impl Flow for SheddingLoad {
    fn type_name(&self) -> &'static str {
        "SheddingLoad"
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
        let result = self.inner.evaluate(snapshot, env, dt)?;
        if lamp_on(battery(snapshot)?, self.reserve_j) {
            return Ok(result);
        }
        let legs: Vec<Leg> = result
            .legs
            .iter()
            .map(|leg| Leg::new(leg.stock.clone(), leg.amount * 0.0))
            .collect::<Result<Vec<_>, _>>()?;
        FlowResult::new(legs)
    }
}

fn battery(state: &State) -> Result<f64, SimError> {
    state
        .stocks
        .get(BATTERY)
        .map(|s| s.amount)
        .ok_or_else(|| SimError::Reference(format!("no {BATTERY:?} stock to shed a load on")))
}

/// Whether the fast registry's lamp draws from the battery on `state` — the lit detector.
///
/// ⚠ Until 2026-10-05 a step counted as lit when light arrived in `boundary.light_used`; the
/// sealed station's light now heats the plant chamber and that stock is gone (Step 3c slice
/// 2b-ii, `docs/plans/post-roadmap-room-temperature.md` §23g). The lamp is evaluated on the
/// step's starting state with the step's own environment, so a shed lamp ([`SheddingLoad`]
/// zeroes every leg) and a failed one (`lamp_power` 0) both read dark.
fn lamp_draws(
    fast_integrator: &EulerIntegrator,
    state: &State,
    fast_resolver: &SourceResolver,
    fast_dt: f64,
) -> Result<bool, SimError> {
    let lamp = fast_integrator
        .registry()
        .flows()
        .iter()
        .find(|f| f.id() == LAMP)
        .ok_or_else(|| SimError::Reference(format!("the fast registry carries no {LAMP:?}")))?;
    let env = fast_resolver.bind(state, fast_dt);
    let result = lamp.evaluate(state, &env, fast_dt)?;
    Ok(result
        .legs
        .iter()
        .any(|leg| leg.stock == BATTERY && leg.amount < 0.0))
}

fn delivery(state: &State) -> Result<f64, SimError> {
    state.aux.get(LAMP_DELIVERY_AUX).copied().ok_or_else(|| {
        SimError::Reference(format!(
            "the lamp-lit crop reads {LAMP_DELIVERY_AUX:?}, which the state does not carry \
             (run it through `run_shedding`)"
        ))
    })
}

/// An environment whose `par` and `net_radiation` are the inner values times the lamp's
/// delivered share: both are the lamp's (§21 of the room-temperature plan), so a shed lamp
/// darkens the crop's water loss as well as its photosynthesis.
struct LampLitEnv<'a> {
    inner: &'a dyn Environment,
    delivered: f64,
}

impl Environment for LampLitEnv<'_> {
    fn get(&self, var: &str) -> Result<f64, SimError> {
        let value = self.inner.get(var)?;
        if var == PAR_VAR || var == RN_VAR {
            Ok(value * self.delivered)
        } else {
            Ok(value)
        }
    }
}

/// A slow flow that reads `par` scaled by the lamp's delivered share.
pub struct LampLitFlow {
    inner: Box<dyn Flow>,
}

impl Flow for LampLitFlow {
    fn type_name(&self) -> &'static str {
        "LampLitFlow"
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
        let lit = LampLitEnv {
            inner: env,
            delivered: delivery(snapshot)?,
        };
        self.inner.evaluate(snapshot, &lit, dt)
    }
}

/// A slow aux process that reads `par` scaled by the lamp's delivered share.
pub struct LampLitAux {
    inner: Box<dyn AuxProcess>,
}

impl AuxProcess for LampLitAux {
    fn type_name(&self) -> &'static str {
        "LampLitAux"
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
        let lit = LampLitEnv {
            inner: env,
            delivered: delivery(snapshot)?,
        };
        self.inner.evaluate(snapshot, &lit, dt)
    }
}

/// Re-wire a built sealed station for the lab: the fast registry's `Lamp` and chamber heater
/// each become a [`SheddingLoad`] at `reserve_j`, every slow flow and aux process becomes
/// lamp-lit, and the state carries [`LAMP_DELIVERY_AUX`] — 1.0 unless it already carries one,
/// as a state saved by [`run_shedding`] does. Takes the pieces `build_sealed_station` returns, so a
/// perturbation can be composed before or after.
pub fn rewire_for_shedding(
    state: State,
    bio_reg: Registry,
    fast_reg: Registry,
    reserve_j: f64,
) -> Result<(State, Registry, Registry), SimError> {
    let (bio_flows, bio_aux) = bio_reg.into_parts();
    let lit_flows: Vec<Box<dyn Flow>> = bio_flows
        .into_iter()
        .map(|inner| Box::new(LampLitFlow { inner }) as Box<dyn Flow>)
        .collect();
    let lit_aux: Vec<Box<dyn AuxProcess>> = bio_aux
        .into_iter()
        .map(|inner| Box::new(LampLitAux { inner }) as Box<dyn AuxProcess>)
        .collect();
    let (fast_flows, fast_aux) = fast_reg.into_parts();
    let mut found = [false; INTERRUPTIBLE_LOADS.len()];
    let shed_flows: Vec<Box<dyn Flow>> = fast_flows
        .into_iter()
        .map(
            |flow| match INTERRUPTIBLE_LOADS.iter().position(|id| *id == flow.id()) {
                Some(i) => {
                    found[i] = true;
                    Box::new(SheddingLoad {
                        inner: flow,
                        reserve_j,
                    }) as Box<dyn Flow>
                }
                None => flow,
            },
        )
        .collect();
    if let Some(i) = found.iter().position(|f| !f) {
        return Err(SimError::Reference(format!(
            "the fast registry carries no {:?} flow to shed",
            INTERRUPTIBLE_LOADS[i]
        )));
    }
    let mut state = state;
    // A fresh build starts lit; a resumed one keeps the share its saved day carried.
    state
        .aux
        .entry(LAMP_DELIVERY_AUX.to_string())
        .or_insert(1.0);
    let bio = Registry::new(lit_flows, &state.stocks, lit_aux)?;
    let fast = Registry::new(shed_flows, &state.stocks, fast_aux)?;
    Ok((state, bio, fast))
}

/// What the lab driver saw of the lamp: the delivered share written after each power group,
/// in order (`days × groups` entries).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShedLog {
    /// The delivered share after each group (1.0 = the lamp drew its full nominal power).
    pub delivery: Vec<f64>,
}

/// Step a lab sealed station `days` master days in the reference's interleaved order, with
/// the lamp-delivery bookkeeping between the two operators. No re-sow hook: a lab run stays
/// inside one season (the caller's horizon must be shorter than `season_days`).
///
/// Per group: run the group's slow steps (reading the share the state carries), then its
/// fast sub-steps — asserting conservation after each, as the reference driver does
/// — while summing the lamp power drawn against the nominal draw. A sub-step counts as lit
/// when the lamp, evaluated on the sub-step's starting state, draws from the battery
/// ([`lamp_draws`]; until 2026-10-05, when light arrived in `boundary.light_used`), and then
/// adds the `lamp_power` forcing it was asked for; the new share is the ratio, written into
/// the state at once. So the crop follows what the lamp **does**, not the rule's formula — a
/// failed lamp darkens it too, and a lamp the rule should have shed but did not leaves it lit.
#[allow(clippy::too_many_arguments)]
pub fn run_shedding(
    bio_integrator: &EulerIntegrator,
    fast_integrator: &EulerIntegrator,
    initial: State,
    bio_resolver: &SourceResolver,
    fast_resolver: &SourceResolver,
    scenario: &SealedStationScenario,
    days: usize,
) -> Result<(Vec<State>, u64, Vec<Event>, ShedLog), SimError> {
    if days >= scenario.season_days {
        return Err(SimError::Validation(format!(
            "the lab shedding driver runs no re-sow hook, so its horizon ({days} days) must \
             stay inside one season ({} days)",
            scenario.season_days
        )));
    }
    let (steps_per_day, slow_dt, fast_dt) =
        (scenario.steps_per_day, scenario.bio_dt, scenario.cabin_dt);
    if fast_dt * steps_per_day as f64 != SECONDS_PER_DAY
        || slow_dt * scenario.bio_steps_per_day as f64 != DAYS_PER_MASTER_DAY
    {
        return Err(SimError::Validation(
            "both operators must cover one master day".to_string(),
        ));
    }
    let groups = day_groups(steps_per_day, scenario.bio_steps_per_day)?;
    let lamp_power = fast_resolver
        .forcings()
        .get(LAMP_POWER_VAR)
        .ok_or_else(|| {
            SimError::Reference(format!("the fast resolver carries no {LAMP_POWER_VAR:?}"))
        })?;
    let nominal_lamp_w = crate::sealed::lighting_average_power(scenario);

    let mut state = initial;
    let mut states = vec![state.clone()];
    let mut rationed = 0u64;
    let mut events: Vec<Event> = Vec::new();
    let mut log = ShedLog::default();
    delivery(&state)?;
    for _day in 0..days {
        for _ in 0..groups.count {
            for _ in 0..groups.slow {
                let report = bio_integrator.step_report(&state, bio_resolver, slow_dt)?;
                state = report.state;
                rationed += report.rationed;
                events.extend(report.events);
            }
            let (mut drawn, mut nominal) = (0.0_f64, 0.0_f64);
            for _ in 0..groups.fast {
                let before = state.clone();
                let lit = lamp_draws(fast_integrator, &before, fast_resolver, fast_dt)?;
                let report = fast_integrator.substep(&state, fast_resolver, fast_dt)?;
                state = report.state;
                assert_conserved_default(&before, &state)?;
                if lit {
                    drawn += lamp_power(before.n, fast_dt);
                }
                nominal += nominal_lamp_w;
                rationed += report.rationed;
                events.extend(report.events);
            }
            // Written the moment it is known, so a day-end state carries its own last
            // group's share and a run resumed from it continues bit for bit (tested).
            let delivered = drawn / nominal;
            state.aux.insert(LAMP_DELIVERY_AUX.to_string(), delivered);
            log.delivery.push(delivered);
        }
        states.push(state.clone());
    }
    Ok((states, rationed, events, log))
}

/// The lab's reserve for a sealed scenario: [`WHAT_IF_RESERVE_HOURS`] of its life-support load.
pub fn what_if_reserve(
    charge: &domains::power::ChargeParams,
    scenario: &SealedStationScenario,
) -> f64 {
    reserve_joules(
        WHAT_IF_RESERVE_HOURS,
        domains::power::balanced_load_w(charge, &scenario.power),
    )
}

/// Cut **only** the lamp's power draw (the fast resolver's `lamp_power`) to zero over
/// `[start, end)` — the half of `with_lighting_failure` the lab crop should feel unaided.
pub fn with_lamp_power_cut(
    fast_resolver: SourceResolver,
    start: u64,
    end: u64,
) -> Result<SourceResolver, SimError> {
    let (mut forcings, shared) = fast_resolver.into_parts();
    let base = forcings.remove(LAMP_POWER_VAR).ok_or_else(|| {
        SimError::Reference(format!("the fast resolver carries no {LAMP_POWER_VAR:?}"))
    })?;
    forcings.insert(
        LAMP_POWER_VAR.to_string(),
        domains::biosphere::perturbations::window_override(base, start, end, 0.0),
    );
    SourceResolver::new(forcings, shared)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rule_is_on_at_the_reserve_and_off_just_below_it() {
        assert!(lamp_on(10.0, 10.0));
        assert!(!lamp_on(f64::from_bits(10.0_f64.to_bits() - 1), 10.0));
        assert!(lamp_on(0.0, f64::NEG_INFINITY));
    }

    #[test]
    fn the_reserve_is_hours_of_the_essential_load() {
        assert_eq!(reserve_joules(24.0, 100.0), 8.64e6);
    }
}
