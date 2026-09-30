//! **Three ways to take the biosphere's step**, side by side — Step 2, slice 2 of the
//! 2026-09-29 review (`docs/plans/post-roadmap-step-options.md`).
//!
//! The draw census found one kind of store under pressure: a chamber CO₂ pool the crop draws
//! on, which the shipped quarter-day step can take up to 0.757 of in one step (1.15 under the
//! lab leaf form). This module builds the three proposed fixes so they can be scored against a
//! much finer step:
//!
//! * **A — a uniformly finer step:** [`run_uniform`] at any power-of-two steps per day. It is
//!   also the runner the fine-step answer comes from.
//! * **B — split a step only when it is tight:** [`run_split`], a driver that probes each
//!   step's withdrawals and takes a tight step as two halves, recursively, down to a depth
//!   limit.
//! * **C — the crop's uptake taken against the air it leaves behind:** [`ImplicitUptake`], a
//!   lab flow wrapping the frozen `biosphere.allocation` in a backward-Euler solve on the pool.
//!
//! # ⚠ `steps_for_years` and `season_steps()` are not used here
//!
//! Both carry `STEPS_PER_DAY = 4` inside them. Off a quarter-day step they return a count that
//! still runs cleanly — over the wrong number of days, which is the step-unfreeze's own mistake.
//! Every runner below takes its steps per day explicitly and derives both counts from
//! [`SEASON_DAYS`].
//!
//! # ⚠ This module takes no decision and endorses no scheme
//!
//! Nothing here is wired into a frozen run. C is not new science (the uptake law is the frozen
//! FvCB); it is a different way of taking the step, and is lab-only until the user decides.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use crate::biosphere::params::BiosphereParams;
use crate::biosphere::readouts::withdrawal_demand;
use crate::biosphere::stocks::{CARBON_POOL, CO2_POOL_VAR};
use crate::biosphere::system::{
    annual_reset_with, build_season_with, run_perennial_with, run_season, SeasonScenario,
};
use crate::biosphere::{season_setup_composed, SeasonBuild, SEASON_DAYS};
use crate::lab::mechanism::build_season_replacing;
use simcore::auxiliary::AuxProcess;
use simcore::conservation::assert_conserved_default;
use simcore::environment::{Environment, SourceResolver};
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult};
use simcore::integrator::EulerIntegrator;
use simcore::registry::Registry;
use simcore::state::State;

/// The frozen crop-uptake flow C wraps.
pub const ALLOCATION_ID: &str = "biosphere.allocation";

/// The two flows whose legs C's scope argument says cannot move between `C₀` and `C₁`: they
/// read the pool only through the carbon budget, and in a sealed chamber growth respiration is
/// empty and maintenance's only budget-dependent term is its organ-burn shortfall.
pub const BUDGET_SIBLINGS: [&str; 2] =
    ["biosphere.growth_respiration", "biosphere.maintenance_respiration"];

/// The step in days for `steps_per_day`, refusing anything that is not a power of two — the
/// only step sizes for which `n · dt` is exact and every coarser grid's boundaries are shared.
pub fn step_days(steps_per_day: usize) -> f64 {
    assert!(
        steps_per_day.is_power_of_two(),
        "steps_per_day must be a power of two, got {steps_per_day}"
    );
    1.0 / steps_per_day as f64
}

// --------------------------------------------------------------------------------------- //
// A — the uniform runner (and the fine-step answer)                                       //
// --------------------------------------------------------------------------------------- //

/// Run `scenario` for `years` seasons at `steps_per_day`, the way its golden drives it:
/// `perennial` re-sows every season through [`run_perennial_with`], otherwise [`run_season`]
/// with no reset. Returns `(final state, backstop firings)`.
///
/// At `steps_per_day = 4` this is the reference run: the same setup, the same runner, the same
/// `dt` and counts the frozen goldens use.
pub fn run_uniform(
    scenario: &SeasonScenario,
    years: usize,
    perennial: bool,
    p: &BiosphereParams,
    build: SeasonBuild<'_>,
    steps_per_day: usize,
    observer: &mut dyn FnMut(&State),
) -> Result<(State, u64), SimError> {
    let dt = step_days(steps_per_day);
    let (state, integrator, resolver) = season_setup_composed(scenario, years, p, build)?;
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
// B — split a step only when it is tight                                                  //
// --------------------------------------------------------------------------------------- //

/// Which stores B's rule judges a step by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Watched {
    /// The chamber CO₂ pool only — the one store the census found drawn by demand.
    ChamberCo2,
    /// Every store a flow withdraws from (the census's own set).
    Every,
}

/// B's rule: split while the largest watched `withdrawal ÷ held` exceeds `theta`, at most
/// `max_depth` halvings deep.
///
/// `theta = f64::INFINITY` never splits and never probes: the reference run. A negative
/// `theta` always splits, to exactly `max_depth` — the control that must equal a uniform run at
/// `4 · 2^max_depth` steps per day.
#[derive(Clone, Copy, Debug)]
pub struct SplitRule {
    pub theta: f64,
    pub watched: Watched,
    pub max_depth: u32,
}

/// What B did over a run.
#[derive(Clone, Debug, Default)]
pub struct SplitStats {
    /// Coarse steps taken.
    pub coarse_steps: u64,
    /// Coarse steps taken as more than one piece — the ones that would move a golden.
    pub split_steps: u64,
    /// Integrator steps actually taken (each a full flow evaluation).
    pub leaf_steps: u64,
    /// Probe evaluations (each a full flow evaluation).
    pub probes: u64,
    /// The deepest halving reached.
    pub deepest: u32,
    /// Coarse steps whose split reached the depth limit.
    pub bottomed: u64,
    /// Backstop firings.
    pub rationed: u64,
}

/// The largest watched `withdrawal ÷ held` if `state` were stepped by `h` — the census probe.
fn tightest_draw(
    integrator: &EulerIntegrator,
    resolver: &SourceResolver,
    state: &State,
    h: f64,
    watched: Watched,
) -> Result<f64, SimError> {
    let bound = resolver.bind(state, h);
    let results: Vec<FlowResult> = integrator
        .registry()
        .flows()
        .iter()
        .map(|f| f.evaluate(state, &bound, h))
        .collect::<Result<_, _>>()?;
    let mut worst = 0.0f64;
    for (stock, demand) in withdrawal_demand(&results, &state.stocks) {
        if watched == Watched::ChamberCo2 && stock != CARBON_POOL {
            continue;
        }
        let held = state.stocks[&stock].amount;
        let ratio = if held > 0.0 { demand / held } else { f64::INFINITY };
        worst = worst.max(ratio);
    }
    Ok(worst)
}

/// `state` with its step counter set to `n` — how a half-step sees the right time of day.
///
/// The forcing schedules read `t = n · dt`, so a half-step at `dt/2` taken from counter `2n`
/// sits at `t = n · dt`, and the second from `2n + 1` at `t = n·dt + dt/2`. Nothing else in the
/// biosphere reads the counter except the re-sow timing, which [`run_split`] applies on the
/// coarse counter.
fn renumbered(state: State, n: u64) -> Result<State, SimError> {
    State::new(n, state.stocks, state.rng_seed, state.aux)
}

/// Advance `state` (counter on the grid of step `h`) by `h`, splitting while the rule says so.
#[allow(clippy::too_many_arguments)]
fn advance(
    integrator: &EulerIntegrator,
    resolver: &SourceResolver,
    state: State,
    h: f64,
    depth: u32,
    rule: &SplitRule,
    stats: &mut SplitStats,
) -> Result<State, SimError> {
    let draw = if rule.theta.is_infinite() && rule.theta > 0.0 {
        0.0
    } else {
        stats.probes += 1;
        tightest_draw(integrator, resolver, &state, h, rule.watched)?
    };
    if draw <= rule.theta || depth == rule.max_depth {
        if depth == rule.max_depth && draw > 1.0 {
            return Err(SimError::Validation(format!(
                "option B refuses: at the depth limit ({depth} halvings, h = {h} d) step {} \
                 would still withdraw {draw} of a watched store — it refuses rather than ration",
                state.n
            )));
        }
        if depth == rule.max_depth && depth > 0 && draw > rule.theta {
            stats.bottomed += 1;
        }
        stats.leaf_steps += 1;
        let report = integrator.step_report(&state, resolver, h)?;
        stats.rationed += report.rationed;
        return Ok(report.state);
    }
    stats.deepest = stats.deepest.max(depth + 1);
    let n = state.n;
    let first = advance(integrator, resolver, renumbered(state, 2 * n)?, h / 2.0, depth + 1, rule, stats)?;
    let second = advance(integrator, resolver, first, h / 2.0, depth + 1, rule, stats)?;
    assert_eq!(second.n, 2 * n + 2, "a split step must end two fine steps on");
    renumbered(second, n + 1)
}

/// Option B over a whole run at `steps_per_day` coarse steps, driven the way the golden drives
/// it. The re-sow (when `perennial`) is `run_perennial_with`'s: consulted before each coarse
/// step, adopted through the conservation gate, and the observer sees the pre-re-sow state.
#[allow(clippy::too_many_arguments)]
pub fn run_split(
    scenario: &SeasonScenario,
    years: usize,
    perennial: bool,
    p: &BiosphereParams,
    build: SeasonBuild<'_>,
    steps_per_day: usize,
    rule: SplitRule,
    observer: &mut dyn FnMut(&State),
) -> Result<(State, SplitStats), SimError> {
    let dt = step_days(steps_per_day);
    let (mut state, integrator, resolver) = season_setup_composed(scenario, years, p, build)?;
    let season = (SEASON_DAYS * steps_per_day) as u64;
    let steps = season * years as u64;
    let mut stats = SplitStats::default();
    observer(&state);
    for _ in 0..steps {
        if perennial && state.n > 0 && state.n.is_multiple_of(season) {
            let reset = annual_reset_with(&state, scenario, p)?;
            assert_conserved_default(&state, &reset)?;
            state = reset;
        }
        let before = (stats.leaf_steps, stats.deepest);
        state = advance(&integrator, &resolver, state, dt, 0, &rule, &mut stats)?;
        stats.coarse_steps += 1;
        if stats.leaf_steps - before.0 > 1 {
            stats.split_steps += 1;
        }
        observer(&state);
    }
    Ok((state, stats))
}

// --------------------------------------------------------------------------------------- //
// C — the crop's uptake against the air it leaves behind                                  //
// --------------------------------------------------------------------------------------- //

/// `env` with the chamber CO₂ pool variable read as `value` — how the inner flow is asked
/// "what would you take if the air held this much?".
struct PoolAt<'a> {
    inner: &'a dyn Environment,
    value: f64,
}

impl Environment for PoolAt<'_> {
    fn get(&self, var: &str) -> Result<f64, SimError> {
        if var == CO2_POOL_VAR {
            Ok(self.value)
        } else {
            self.inner.get(var)
        }
    }
}

/// The net amount `flows` put into the chamber pool over `dt` if the air held `pool`.
pub fn pool_change(
    flows: &[Box<dyn Flow>],
    snapshot: &State,
    env: &dyn Environment,
    dt: f64,
    pool: f64,
) -> Result<f64, SimError> {
    let at = PoolAt { inner: env, value: pool };
    let mut net = 0.0;
    for f in flows {
        for leg in f.evaluate(snapshot, &at, dt)?.legs {
            if leg.stock == CARBON_POOL {
                net += leg.amount;
            }
        }
    }
    Ok(net)
}

/// One step's backward-Euler solve.
#[derive(Clone, Debug)]
pub struct EndPool {
    /// The pool at the start of the step.
    pub c0: f64,
    /// What the explicit flows put into the pool this step, read at the start (0 for
    /// [`Scope::CropOnly`], which counts none).
    pub inflow: f64,
    /// The end-of-step pool the implicit flows are read against: the root of
    /// `X = C₀ + inflow + N(X)`, `N` the implicit flows' net pool change.
    pub c1: f64,
    /// `|X − C₀ − inflow − N(X)|` at the returned `X`.
    pub residual: f64,
    /// Implicit-set evaluations the solve cost.
    pub evaluations: u64,
}

/// Solve `X = C₀ + inflow + N(X)` for the end-of-step pool by bisection on `X`.
///
/// `N(X)` is the net pool change of the `implicit` flows read against `X`. For the crop's
/// budget it only falls as `X` rises (uptake rises with the air; the organ-burn return of
/// maintenance falls), so `h(X) = X − C₀ − inflow − N(X)` rises and has one root, on
/// `[0, C₀ + inflow + N(0)]`. **That is checked on every step, not assumed**: a bracket that
/// does not bracket is an error, and so is a residual above 1e-12 of the pool.
pub fn end_pool(
    implicit: &[Box<dyn Flow>],
    snapshot: &State,
    env: &dyn Environment,
    dt: f64,
    inflow: f64,
) -> Result<EndPool, SimError> {
    let c0 = env.get(CO2_POOL_VAR)?;
    let held = snapshot.stocks[CARBON_POOL].amount;
    if c0.to_bits() != held.to_bits() {
        return Err(SimError::Validation(format!(
            "the pool variable reads {c0} but the pool stock holds {held}: the solve would be \
             against a different air than the step's"
        )));
    }
    let mut evaluations = 0u64;
    let mut h = |x: f64| -> Result<f64, SimError> {
        evaluations += 1;
        Ok(x - c0 - inflow - pool_change(implicit, snapshot, env, dt, x)?)
    };
    let h0 = h(0.0)?;
    if h0 > 0.0 {
        return Err(SimError::Validation(format!(
            "step {}: h(0) = {h0} > 0 — the step takes more than the pool and its returns hold",
            snapshot.n
        )));
    }
    let (mut lo, mut hi) = (0.0f64, -h0);
    let h_hi = h(hi)?;
    // `hi = C₀ + inflow + N(0)` is itself the root whenever `N` is flat (a crop taking nothing),
    // and there `h(hi)` is zero only up to rounding — so the bracket is judged to the same
    // tolerance the residual is.
    let tolerance = 1e-12 * c0.max(hi);
    if h_hi < -tolerance {
        return Err(SimError::Validation(format!(
            "step {}: the bracket does not bracket — h({hi}) = {h_hi} < 0, so the crop's net draw \
             rose as the air thinned",
            snapshot.n
        )));
    }
    if h_hi > tolerance {
        for _ in 0..2000 {
            let mid = 0.5 * (lo + hi);
            if mid <= lo || mid >= hi {
                break;
            }
            if h(mid)? >= 0.0 {
                hi = mid;
            } else {
                lo = mid;
            }
        }
    }
    let residual = h(hi)?.abs();
    if residual > tolerance {
        return Err(SimError::Validation(format!(
            "step {}: the solve did not converge — |h(X)| = {residual} against a pool of {c0}",
            snapshot.n
        )));
    }
    Ok(EndPool { c0, inflow, c1: hi, residual, evaluations })
}

/// What C did over a run — shared between the flows inside the registry and the caller.
#[derive(Clone, Debug, Default)]
pub struct SolveLog {
    /// Steps solved (the chamber had a pool).
    pub solves: u64,
    /// Implicit-set evaluations spent solving.
    pub evaluations: u64,
    /// The largest `|X − C₀ − inflow − N(X)|` over the run.
    pub max_residual: f64,
    /// Steps on which the crop took nothing.
    pub idle: u64,
    /// Scope checks run (one per solved step, when checked).
    pub scope_checks: u64,
    /// Per explicit process: steps on which it gave a different answer at `X` than at `C₀` —
    /// a reader of the pool this scope leaves explicit, reported rather than hidden.
    pub differing: BTreeMap<String, u64>,
    /// Steps whose crop uptake exceeded the START-of-step pool (possible only when same-step
    /// returns are counted) — each one a backstop firing.
    pub beyond_start: u64,
}

/// What "the air it leaves behind" counts, and which of the crop's flows read it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// Backward Euler on the crop's uptake alone: the allocation flow is read against
    /// `X = C₀ − U(X)`, and every other flow (maintenance and growth respiration included)
    /// against the start of the step.
    CropOnly,
    /// The crop's whole carbon budget (allocation, growth and maintenance respiration) is read
    /// against the end-of-step pool the step will produce: `X = C₀ + I + N(X)`, with `I` what
    /// every other flow returns to the pool over the same step, read at the start as the step
    /// itself reads them.
    ///
    /// ⚠ The backstop still judges the crop's withdrawal against the START-of-step pool
    /// (arbitration takes no same-step inflows), so an uptake that spends a same-step return
    /// beyond the start amount rations. Counted, not hidden.
    WithInflows,
}

impl Scope {
    /// The flow ids read against the end-of-step pool, in canonical order.
    pub fn implicit_ids(self) -> &'static [&'static str] {
        match self {
            Scope::CropOnly => &[ALLOCATION_ID],
            Scope::WithInflows => &[
                ALLOCATION_ID,
                "biosphere.growth_respiration",
                "biosphere.maintenance_respiration",
            ],
        }
    }
}

/// The rest of the season's processes, built fresh: read for the same-step returns under
/// [`Scope::WithInflows`], and for the scope check when a run is `checked`.
pub struct Witnesses {
    pub flows: Vec<Box<dyn Flow>>,
    pub aux: Vec<Box<dyn AuxProcess>>,
}

/// Option C: one of the crop's frozen flows, evaluated against the pool the step leaves.
///
/// Each implicit flow of the scope is replaced by one of these, each holding its own fresh copy
/// of the whole implicit set and solving the same deterministic equation, so all of them read
/// one `X` bit for bit. `id` and `priority` delegate to the target, so the reduction order is
/// the frozen one; `type_name` is its own. With no chamber pool (the open field) the target is
/// passed through untouched.
pub struct ImplicitUptake {
    implicit: Vec<Box<dyn Flow>>,
    target: usize,
    rest: Witnesses,
    scope: Scope,
    checked: bool,
    log: Option<Rc<RefCell<SolveLog>>>,
}

impl ImplicitUptake {
    /// `implicit[target]` read against the end-of-step pool; `rest` is every other process.
    /// Only the wrapper holding `log` records (one per season, so a step is counted once).
    pub fn new(
        implicit: Vec<Box<dyn Flow>>,
        target: usize,
        rest: Witnesses,
        scope: Scope,
        checked: bool,
        log: Option<Rc<RefCell<SolveLog>>>,
    ) -> ImplicitUptake {
        ImplicitUptake { implicit, target, rest, scope, checked, log }
    }

    fn check_scope(
        &self,
        log: &mut SolveLog,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
        x: f64,
    ) -> Result<(), SimError> {
        let at_x = PoolAt { inner: env, value: x };
        log.scope_checks += 1;
        for f in &self.rest.flows {
            let a = f.evaluate(snapshot, env, dt)?;
            let b = f.evaluate(snapshot, &at_x, dt)?;
            let same = a.legs.len() == b.legs.len()
                && a.legs.iter().zip(&b.legs).all(|(p, q)| {
                    p.stock == q.stock && p.amount.to_bits() == q.amount.to_bits()
                });
            if !same {
                if BUDGET_SIBLINGS.contains(&f.id()) {
                    return Err(SimError::Validation(format!(
                        "step {}: {} gives different legs at the end-of-step pool than at the \
                         start — the crop-only scope is not 'the crop's uptake only'",
                        snapshot.n,
                        f.id()
                    )));
                }
                *log.differing.entry(f.id().to_string()).or_insert(0) += 1;
            }
        }
        for a in &self.rest.aux {
            let p = a.evaluate(snapshot, env, dt)?;
            let q = a.evaluate(snapshot, &at_x, dt)?;
            let same = p.len() == q.len()
                && p.iter()
                    .zip(&q)
                    .all(|((k1, v1), (k2, v2))| k1 == k2 && v1.to_bits() == v2.to_bits());
            if !same {
                *log.differing.entry(format!("aux:{}", a.id())).or_insert(0) += 1;
            }
        }
        Ok(())
    }
}

impl Flow for ImplicitUptake {
    fn type_name(&self) -> &'static str {
        "ImplicitUptake"
    }

    fn id(&self) -> &str {
        self.implicit[self.target].id()
    }

    fn priority(&self) -> i64 {
        self.implicit[self.target].priority()
    }

    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let target = self.implicit[self.target].as_ref();
        if !snapshot.stocks.contains_key(CARBON_POOL) {
            return target.evaluate(snapshot, env, dt);
        }
        let inflow = match self.scope {
            Scope::CropOnly => 0.0,
            Scope::WithInflows => pool_change(&self.rest.flows, snapshot, env, dt, env.get(CO2_POOL_VAR)?)?,
        };
        let solved = end_pool(&self.implicit, snapshot, env, dt, inflow)?;
        let result = target.evaluate(snapshot, &PoolAt { inner: env, value: solved.c1 }, dt)?;
        if let Some(log) = &self.log {
            let mut log = log.borrow_mut();
            log.solves += 1;
            log.evaluations += solved.evaluations;
            log.max_residual = log.max_residual.max(solved.residual);
            let uptake = -result
                .legs
                .iter()
                .filter(|l| l.stock == CARBON_POOL)
                .map(|l| l.amount)
                .sum::<f64>();
            if uptake == 0.0 {
                log.idle += 1;
            }
            if uptake > solved.c0 {
                log.beyond_start += 1;
            }
            if self.checked {
                self.check_scope(&mut log, snapshot, env, dt, solved.c1)?;
            }
        }
        Ok(result)
    }
}

/// The season with each of `scope`'s implicit flows replaced by an [`ImplicitUptake`] around
/// fresh copies of the frozen flows, through the mechanism lab's replacing composer (one
/// assembly body). The allocation wrapper keeps the log.
///
/// `checked` runs the scope check every step: every explicit flow and aux process evaluated
/// at `C₀` and at `X`. It costs evaluations and changes no number.
pub fn build_season_implicit_uptake(
    scenario: &SeasonScenario,
    p: &BiosphereParams,
    scope: Scope,
    checked: bool,
    log: Rc<RefCell<SolveLog>>,
) -> Result<(State, Registry), SimError> {
    let ids = scope.implicit_ids();
    let take = || -> Result<(Vec<Box<dyn Flow>>, Witnesses), SimError> {
        let (_, fresh) = build_season_with(scenario, p)?;
        let (flows, aux) = fresh.into_parts();
        let mut implicit: Vec<Option<Box<dyn Flow>>> = ids.iter().map(|_| None).collect();
        let mut rest = Vec::new();
        for f in flows {
            match ids.iter().position(|id| *id == f.id()) {
                Some(i) => implicit[i] = Some(f),
                None => rest.push(f),
            }
        }
        let implicit = implicit
            .into_iter()
            .zip(ids)
            .map(|(f, id)| {
                f.ok_or_else(|| {
                    SimError::Validation(format!("{id:?} is not in this season's registry"))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok((implicit, Witnesses { flows: rest, aux }))
    };
    let mut replacements: Vec<(&str, Box<dyn Flow>)> = Vec::new();
    for (i, id) in ids.iter().enumerate() {
        let (implicit, rest) = take()?;
        let keeps_log = (i == 0).then(|| Rc::clone(&log));
        replacements.push((
            id,
            Box::new(ImplicitUptake::new(implicit, i, rest, scope, checked, keeps_log)),
        ));
    }
    build_season_replacing(scenario, p, replacements)
}
