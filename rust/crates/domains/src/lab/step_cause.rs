//! **Why the quarter-day step loses the jar's harvest** — Step 2, slice 3b of the 2026-09-29
//! review (`docs/plans/post-roadmap-step-cause.md`).
//!
//! Refining the step refines two things at once: how far the state moves per evaluation, and
//! how finely the day's light is resolved (PAR is the one forcing that varies within a day, and
//! it reaches a step as the mean over the step's window). The two instruments here separate
//! them:
//!
//! * **E1 — any step, light held in coarser blocks:** [`run_uniform_blocked_light`] runs at
//!   `steps_per_day` with PAR held at the mean of a `light_steps_per_day` window.
//! * **E2 — a coarse step, light integrated inside it:** [`LightQuadrature`] evaluates its inner
//!   flow at `k` sub-window PARs on the **same** start-of-step state and averages the legs.
//!   [`build_season_light_quadrature`] wraps every flow that reads the carbon budget.
//!
//! [`par_readers`] is E2's scope check (nothing outside the wrapped flows may read PAR), and
//! [`dose_partition_worst`] checks that the sub-windows partition the step's dose.
//!
//! # ⚠ E1 goes round the weather guard on purpose
//!
//! [`crate::biosphere::season_setup_composed`] keeps the weather out of the mechanism seam so a
//! mechanism A/B cannot move the forcing. Here the forcing's time resolution **is** the
//! variable, so E1 rebuilds the resolver from the public
//! [`weather_forcings`]/[`weather_shared`] and wraps PAR alone. Its control is that a block as
//! fine as the step reproduces the plain run bit for bit.
//!
//! # ⚠ This module takes no decision
//!
//! Neither instrument is a candidate scheme. E2 happens to be one shape a fix could take; that
//! is the user's call, on the numbers, and nothing here is wired into a frozen run.

use std::cell::Cell;
use std::collections::BTreeSet;

use crate::biosphere::params::BiosphereParams;
use crate::biosphere::stocks::PAR_VAR;
use crate::biosphere::system::{
    build_season_with, run_perennial_with, run_season, weather_forcings, weather_resolver,
    weather_shared, SeasonScenario,
};
use crate::biosphere::{SeasonBuild, SEASON_DAYS};
use crate::lab::mechanism::build_season_replacing;
use crate::lab::step_options::{step_days, ALLOCATION_ID};
use simcore::environment::{Environment, Schedule, SourceResolver};
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::integrator::EulerIntegrator;
use simcore::registry::Registry;
use simcore::state::State;

/// The three flows that read the crop's carbon budget (`CarbonContext::budget`), in canonical
/// order. E2 wraps all of them, so no two see different light within one step.
pub const BUDGET_FLOWS: [&str; 3] = [
    ALLOCATION_ID,
    "biosphere.growth_respiration",
    "biosphere.maintenance_respiration",
];

/// The season's own PAR schedule, built fresh (a `Schedule` is not `Clone`) — the one E2's
/// wrappers read.
pub fn par_schedule(scenario: &SeasonScenario, years: usize) -> Result<Schedule, SimError> {
    weather_forcings(scenario, years)?
        .remove(PAR_VAR)
        .ok_or_else(|| SimError::Validation(format!("the weather carries no {PAR_VAR:?}")))
}

// --------------------------------------------------------------------------------------- //
// E1 — light held in blocks                                                               //
// --------------------------------------------------------------------------------------- //

/// `base` read as the mean over the enclosing `block_steps_per_day` window, whatever step asks.
///
/// A step `dt` finer than the block sits inside block `⌊n·dt / Δ⌋` (exact: both are powers of
/// two), and gets that block's window mean `base(k, Δ)`. At `dt = Δ` this is `base(n, dt)`
/// exactly — the control.
fn blocked(base: Schedule, block_steps_per_day: usize) -> Schedule {
    let block = step_days(block_steps_per_day);
    Box::new(move |n, dt| {
        assert!(
            dt <= block,
            "a light block of {block} d cannot serve a coarser step of {dt} d"
        );
        base(((n as f64 * dt) / block).floor() as u64, block)
    })
}

/// The season's resolver with PAR held in `light_steps_per_day` blocks; every other forcing
/// and the shared-stock map are the stock ones.
pub fn blocked_light_resolver(
    scenario: &SeasonScenario,
    years: usize,
    light_steps_per_day: usize,
) -> Result<SourceResolver, SimError> {
    let mut forcings = weather_forcings(scenario, years)?;
    let base = forcings
        .remove(PAR_VAR)
        .ok_or_else(|| SimError::Validation(format!("the weather carries no {PAR_VAR:?}")))?;
    forcings.insert(PAR_VAR.to_string(), blocked(base, light_steps_per_day));
    SourceResolver::new(forcings, weather_shared(scenario))
}

/// E1: [`crate::lab::step_options::run_uniform`] with the light held in `light_steps_per_day`
/// blocks. Returns `(final state, backstop firings)`.
#[allow(clippy::too_many_arguments)]
pub fn run_uniform_blocked_light(
    scenario: &SeasonScenario,
    years: usize,
    perennial: bool,
    p: &BiosphereParams,
    build: SeasonBuild<'_>,
    steps_per_day: usize,
    light_steps_per_day: usize,
    observer: &mut dyn FnMut(&State),
) -> Result<(State, u64), SimError> {
    assert!(
        light_steps_per_day <= steps_per_day,
        "the light block must be at least one step long"
    );
    let dt = step_days(steps_per_day);
    let (state, registry) = build(scenario, p)?;
    let integrator = EulerIntegrator::new(registry);
    let resolver = blocked_light_resolver(scenario, years, light_steps_per_day)?;
    let season = SEASON_DAYS * steps_per_day;
    let steps = season * years;
    let (end, rationed, _) = if perennial {
        run_perennial_with(
            &integrator, state, scenario, p, &resolver, dt, steps, season, observer,
        )?
    } else {
        run_season(&integrator, state, &resolver, dt, steps, None, observer)?
    };
    Ok((end, rationed))
}

// --------------------------------------------------------------------------------------- //
// E2 — light integrated inside the step                                                   //
// --------------------------------------------------------------------------------------- //

/// `env` with PAR read as `value`.
struct ParAt<'a> {
    inner: &'a dyn Environment,
    value: f64,
}

impl Environment for ParAt<'_> {
    fn get(&self, var: &str) -> Result<f64, SimError> {
        if var == PAR_VAR {
            Ok(self.value)
        } else {
            self.inner.get(var)
        }
    }
}

/// One frozen flow, evaluated at `k` sub-window lights on the step's own starting state and
/// averaged.
///
/// Sub-window `j` of step `n` reads the season's PAR schedule at the renumbered counter
/// `n·k + j` with step `dt/k` — the window a `k`-times finer step would see there. The legs
/// are summed stock by stock (in order of first appearance) and divided by `k`, so the flow
/// stays balanced (the mean of balanced legs is balanced). `id` and `priority` delegate, so the
/// reduction order is the frozen one.
pub struct LightQuadrature {
    inner: Box<dyn Flow>,
    par: Schedule,
    k: u64,
}

impl LightQuadrature {
    pub fn new(inner: Box<dyn Flow>, par: Schedule, k: u64) -> LightQuadrature {
        assert!(k >= 1, "at least one sub-window");
        LightQuadrature { inner, par, k }
    }
}

impl Flow for LightQuadrature {
    fn type_name(&self) -> &'static str {
        "LightQuadrature"
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
        let h = dt / self.k as f64;
        let mut legs: Vec<Leg> = Vec::new();
        for j in 0..self.k {
            let par = (self.par)(snapshot.n * self.k + j, h);
            let result = self.inner.evaluate(snapshot, &ParAt { inner: env, value: par }, dt)?;
            if j == 0 {
                // Taken as is, not added to a zero: `0.0 + (−0.0)` would change a sign bit and
                // break the k = 1 control for no physical reason.
                legs = result.legs;
                continue;
            }
            for leg in result.legs {
                match legs.iter_mut().find(|l| l.stock == leg.stock) {
                    Some(l) => l.amount += leg.amount,
                    None => legs.push(leg),
                }
            }
        }
        let k = self.k as f64;
        let legs = legs
            .into_iter()
            .map(|l| Leg::new(l.stock, l.amount / k))
            .collect::<Result<Vec<_>, _>>()?;
        FlowResult::new(legs)
    }
}

/// The season with each of [`BUDGET_FLOWS`] wrapped in a [`LightQuadrature`] of `k`
/// sub-windows, through the mechanism lab's replacing composer. `years` sizes the PAR table the
/// wrappers read, exactly as the resolver's is sized.
pub fn build_season_light_quadrature(
    scenario: &SeasonScenario,
    p: &BiosphereParams,
    years: usize,
    k: u64,
) -> Result<(State, Registry), SimError> {
    let (_, fresh) = build_season_with(scenario, p)?;
    let (flows, _) = fresh.into_parts();
    let mut replacements: Vec<(&str, Box<dyn Flow>)> = Vec::new();
    for f in flows {
        if let Some(id) = BUDGET_FLOWS.iter().find(|id| **id == f.id()) {
            replacements.push((id, Box::new(LightQuadrature::new(f, par_schedule(scenario, years)?, k))));
        }
    }
    if replacements.len() != BUDGET_FLOWS.len() {
        return Err(SimError::Validation(format!(
            "this season carries {} of the {} budget flows — a partial wrap would let two of \
             them read different light",
            replacements.len(),
            BUDGET_FLOWS.len()
        )));
    }
    build_season_replacing(scenario, p, replacements)
}

// --------------------------------------------------------------------------------------- //
// E2's controls                                                                           //
// --------------------------------------------------------------------------------------- //

/// `env`, remembering whether PAR was read.
struct Recording<'a> {
    inner: &'a dyn Environment,
    read_par: Cell<bool>,
}

impl Environment for Recording<'_> {
    fn get(&self, var: &str) -> Result<f64, SimError> {
        if var == PAR_VAR {
            self.read_par.set(true);
        }
        self.inner.get(var)
    }
}

/// Every flow and aux process (`aux:`-prefixed) that reads PAR on any step of one season at a
/// quarter day, plus the count of lit steps (so an empty answer cannot be a dark season).
///
/// Probed on a fresh copy of the registry against the stock resolver, at every state the plain
/// run passes through. An evaluation that errors is an error here too.
pub fn par_readers(
    scenario: &SeasonScenario,
    p: &BiosphereParams,
) -> Result<(BTreeSet<String>, u64), SimError> {
    let dt = step_days(4);
    let (_, probe) = build_season_with(scenario, p)?;
    let (flows, aux) = probe.into_parts();
    let resolver = weather_resolver(scenario, 1)?;
    let (state, registry) = build_season_with(scenario, p)?;
    let integrator = EulerIntegrator::new(registry);
    let mut readers: BTreeSet<String> = BTreeSet::new();
    let mut lit = 0u64;
    let mut failed: Option<SimError> = None;
    let mut observe = |s: &State| {
        if failed.is_some() {
            return;
        }
        let bound = resolver.bind(s, dt);
        match bound.get(PAR_VAR) {
            Ok(par) if par > 0.0 => lit += 1,
            Ok(_) => {}
            Err(e) => failed = Some(e),
        }
        for f in &flows {
            let rec = Recording { inner: &bound, read_par: Cell::new(false) };
            if let Err(e) = f.evaluate(s, &rec, dt) {
                failed = Some(e);
                return;
            }
            if rec.read_par.get() {
                readers.insert(f.id().to_string());
            }
        }
        for a in &aux {
            let rec = Recording { inner: &bound, read_par: Cell::new(false) };
            if let Err(e) = a.evaluate(s, &rec, dt) {
                failed = Some(e);
                return;
            }
            if rec.read_par.get() {
                readers.insert(format!("aux:{}", a.id()));
            }
        }
    };
    run_season(&integrator, state, &resolver, dt, SEASON_DAYS * 4, None, &mut observe)?;
    if let Some(e) = failed {
        return Err(e);
    }
    Ok((readers, lit))
}

/// The largest relative gap, over every lit quarter-day step of `years` seasons, between the
/// step's dose `par(n, ¼)·¼` and the sum of its `k` sub-window doses — `0` if the sub-windows
/// partition the window, as the analytic window mean says they must.
pub fn dose_partition_worst(
    scenario: &SeasonScenario,
    years: usize,
    k: u64,
) -> Result<f64, SimError> {
    let par = par_schedule(scenario, years)?;
    let dt = step_days(4);
    let h = dt / k as f64;
    let mut worst = 0.0f64;
    for n in 0..(SEASON_DAYS * 4 * years) as u64 {
        let whole = par(n, dt) * dt;
        let parts: f64 = (0..k).map(|j| par(n * k + j, h) * h).sum();
        if whole > 0.0 {
            worst = worst.max((parts - whole).abs() / whole);
        } else if parts != 0.0 {
            worst = f64::INFINITY;
        }
    }
    Ok(worst)
}
