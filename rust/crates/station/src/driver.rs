//! The station's two-rate master-step driver — the port of `station.driver` (P7.5).
//!
//! Couples a **day-scale** slow domain (the biosphere: `dt` in days, a `thermal_time` aux
//! that must advance) to a **second-scale** fast domain (the cabin / Power: `dt = 60`/`3600`
//! s). `simcore::multirate` cannot bridge these — it splits ONE shared master `dt`, and
//! `substep` freezes the biosphere aux — so the driver does the operator split (Lie,
//! slow-first) **by hand**: per master day the slow domain takes `slow_steps_per_day`
//! `step_report`s (advancing aux **and** `n`), then the fast domain takes `steps_per_day`
//! `substep` calls (keeping `n`).
//!
//! **The load-bearing Tier-0 gate:** `substep` deliberately skips the conservation assert,
//! so the driver re-asserts it (`assert_conserved_default`) after **every** fast sub-step
//! over the whole shared ledger, and across each reset — this per-sub-step assertion **is**
//! the "conservation holds every step in Rust" primary cross-port gate for all the coupled
//! goldens (a completed run is itself the proof).

use simcore::conservation::assert_conserved_default;
use simcore::environment::SourceResolver;
use simcore::error::SimError;
use simcore::events::Event;
use simcore::integrator::{EulerIntegrator, Substepper};
use simcore::state::State;

/// Seconds in one master day — what `fast_dt · steps_per_day` must advance.
pub const SECONDS_PER_DAY: f64 = 86400.0;

/// Days in one master day — what `slow_dt · slow_steps_per_day` must advance.
pub const DAYS_PER_MASTER_DAY: f64 = 1.0;

/// A schedule-agnostic slow-domain reset hook `(n, state) -> Ok(Some(new_state))` on a
/// reset boundary (checked by the conservation gate then adopted) or `Ok(None)` otherwise —
/// the `run_season`/`annual_reset` re-sow hook, cross-domain (the P6.7 `slow_reset`).
pub type ResetHook<'a> = &'a dyn Fn(u64, &State) -> Result<Option<State>, SimError>;

/// An **owned** reset hook — the boxed form a caller-driven [`crate::session::SimSession`]
/// holds so it is self-contained across steps (a game loop can't keep a borrow alive
/// between frames). Coerces to [`ResetHook`] via `&*hook` at the call site.
pub type OwnedResetHook = Box<dyn Fn(u64, &State) -> Result<Option<State>, SimError>>;

/// Advance exactly **one** master day in place: consult `slow_reset` (adopting a returned
/// re-sow state after a conservation check), run `slow_steps_per_day` slow `step_report`s
/// (each advancing the phenology aux **and** `n`), then `steps_per_day` fast `substep`s at
/// `fast_dt` (keeping `n`), asserting conservation over the full shared ledger after
/// **each** sub-step. Returns `(next_state, day_rationed, day_events)`.
///
/// ⚠ `n` is the slow domain's **step** count, not the day count — it was the same number
/// only while the biosphere's step was one day. Nothing here needs it to be a day count
/// (`table_schedule` indexes `int(n · dt)`), but a `slow_reset` closure's period must be
/// in steps.
///
/// This is the per-day body of [`run_master_day`], extracted so a caller-driven session
/// ([`crate::session::SimSession::step`]) steps day-by-day through the **same** code — the
/// Phase-8 parity discipline: incremental stepping is bit-identical to run-to-completion
/// because it *is* the same function. Does not validate `fast_dt·steps_per_day == 86400`
/// (the runner / session constructor owns that once).
#[allow(clippy::too_many_arguments)]
pub fn advance_one_master_day(
    slow_integrator: &EulerIntegrator,
    fast_integrator: &EulerIntegrator,
    state: &State,
    slow_resolver: &SourceResolver,
    fast_resolver: &SourceResolver,
    steps_per_day: u64,
    slow_steps_per_day: u64,
    slow_dt: f64,
    fast_dt: f64,
    slow_reset: Option<ResetHook<'_>>,
) -> Result<(State, u64, Vec<Event>), SimError> {
    let mut state = state.clone();
    let mut total_rationed = 0u64;
    let mut events: Vec<Event> = Vec::new();
    // Scheduled slow-domain reset (re-sow), applied once per master day at a slow-step
    // boundary. ⚠ n is a STEP count, so the closure's period must be in steps.
    // Conservation re-asserted across it (annual_reset is CARBON-conserving).
    if let Some(reset_fn) = slow_reset {
        if let Some(reset_state) = reset_fn(state.n, &state)? {
            assert_conserved_default(&state, &reset_state)?;
            state = reset_state;
        }
    }
    // Slow operator: slow_steps_per_day sub-steps covering one day (each advances the
    // phenology aux AND n).
    for _ in 0..slow_steps_per_day {
        let slow_report = slow_integrator.step_report(&state, slow_resolver, slow_dt)?;
        state = slow_report.state;
        total_rationed += slow_report.rationed;
        events.extend(slow_report.events);
    }
    // Fast operator: steps_per_day sub-steps at fast_dt (n kept). substep skips the
    // conservation assert, so we own it here — after each sub-step, over the full shared
    // ledger — keeping the every-step teeth.
    for _ in 0..steps_per_day {
        let before = state.clone();
        let fast_report = fast_integrator.substep(&state, fast_resolver, fast_dt)?;
        state = fast_report.state;
        assert_conserved_default(&before, &state)?;
        total_rationed += fast_report.rationed;
        events.extend(fast_report.events);
    }
    Ok((state, total_rationed, events))
}

/// Step `days` master days (slow ×`slow_steps_per_day` + fast ×`steps_per_day`), slow-first.
///
/// Per day: `slow_reset` (if given) is consulted first — a returned `Some(state)` is
/// conservation-checked then adopted; then the `slow_integrator` runs `slow_steps_per_day`
/// `step_report`s at `slow_dt` (its own gate fires on each); then the `fast_integrator` runs
/// `steps_per_day` `substep` calls at `fast_dt` (`n` kept), the driver asserting conservation
/// after **each** over the full shared ledger. Returns `(states, total_rationed, events)`
/// with `states` one entry per **master day** — not per slow step — (length `days + 1`;
/// a golden pins the final one), so station trajectories stay day-indexed.
///
/// Requires `fast_dt · steps_per_day == 86400` s and `slow_dt · slow_steps_per_day == 1`
/// day, so both operators cover the same interval.
#[allow(clippy::too_many_arguments)]
pub fn run_master_day(
    slow_integrator: &EulerIntegrator,
    fast_integrator: &EulerIntegrator,
    initial: State,
    slow_resolver: &SourceResolver,
    fast_resolver: &SourceResolver,
    days: usize,
    steps_per_day: u64,
    slow_steps_per_day: u64,
    slow_dt: f64,
    fast_dt: f64,
    slow_reset: Option<ResetHook<'_>>,
) -> Result<(Vec<State>, u64, Vec<Event>), SimError> {
    if fast_dt * steps_per_day as f64 != SECONDS_PER_DAY {
        return Err(SimError::Validation(format!(
            "fast_dt*steps_per_day must equal one day ({SECONDS_PER_DAY} s) so the fast \
             operator covers one master day, got {fast_dt}*{steps_per_day} = {}",
            fast_dt * steps_per_day as f64
        )));
    }
    if slow_dt * slow_steps_per_day as f64 != DAYS_PER_MASTER_DAY {
        return Err(SimError::Validation(format!(
            "slow_dt*slow_steps_per_day must equal one day ({DAYS_PER_MASTER_DAY}) so the \
             slow operator covers one master day, got {slow_dt}*{slow_steps_per_day} = {}",
            slow_dt * slow_steps_per_day as f64
        )));
    }
    let mut state = initial;
    let mut states: Vec<State> = vec![state.clone()];
    let mut total_rationed = 0u64;
    let mut events: Vec<Event> = Vec::new();
    for _day in 0..days {
        let (next, day_rationed, day_events) = advance_one_master_day(
            slow_integrator,
            fast_integrator,
            &state,
            slow_resolver,
            fast_resolver,
            steps_per_day,
            slow_steps_per_day,
            slow_dt,
            fast_dt,
            slow_reset,
        )?;
        state = next;
        total_rationed += day_rationed;
        events.extend(day_events);
        states.push(state.clone());
    }
    Ok((states, total_rationed, events))
}

// =====================================================================================
// LAB-ONLY — the day ORDER, as a choice (post-roadmap-intraday-gas-exchange.md)
// =====================================================================================
//
// Nothing in the reference calls anything below: not a runner, not the session, not the
// bridge. It exists so the lab can run a master day in a second order and keep books per
// side. The reference day is `advance_one_master_day` above, unedited. If the interleaved
// order is ever adopted, that function takes its body and this section is deleted rather
// than kept as a switch.

/// The order a master day runs its two operators in. **Lab-only.**
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DayOrder {
    /// The reference: every slow step, then every fast step (`P P P P c … c`).
    SlowFirst,
    /// One slow step, then that step's share of the fast steps, repeated
    /// (`P c…c P c…c P c…c P c…c`). The fast side refills a shared pool between the slow
    /// side's draws. Requires `steps_per_day` to divide evenly by `slow_steps_per_day`.
    Interleaved,
}

/// Which operator produced a step, as the lab's observer sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// The slow domain's re-sow hook adopted a new state.
    Reset,
    /// One slow `step_report` (advances `n` and the phenology aux).
    Slow,
    /// One fast `substep` (keeps `n`).
    Fast,
}

/// A day's (or a run's) rationing, **split by side**. The reference driver sums the two
/// into one integer, and the crew-loop record showed that sum cannot say which side rationed.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SideTotals {
    /// Backstop firings on the slow side.
    pub slow_rationed: u64,
    /// Backstop firings on the fast side.
    pub fast_rationed: u64,
    /// Every event, in the order the steps produced them.
    pub events: Vec<Event>,
}

/// The pieces of a two-rate run, bundled so the lab's day order is one argument rather than
/// eleven. **Lab-only.**
pub struct TwoRate<'a> {
    /// The slow (biosphere) integrator.
    pub slow: &'a EulerIntegrator,
    /// The fast (cabin / Power / Thermal) integrator.
    pub fast: &'a EulerIntegrator,
    /// The slow side's forcing.
    pub slow_resolver: &'a SourceResolver,
    /// The fast side's forcing.
    pub fast_resolver: &'a SourceResolver,
    /// Fast sub-steps per master day.
    pub steps_per_day: u64,
    /// Slow steps per master day.
    pub slow_steps_per_day: u64,
    /// The slow step (days).
    pub slow_dt: f64,
    /// The fast step (s).
    pub fast_dt: f64,
    /// The slow side's re-sow hook, consulted once per master day at its start.
    pub slow_reset: Option<ResetHook<'a>>,
}

impl TwoRate<'_> {
    /// The reference runner's two day-length guards, plus the interleaved order's own:
    /// the fast steps must split evenly across the slow ones.
    pub fn validate(&self, order: DayOrder) -> Result<(), SimError> {
        if self.fast_dt * self.steps_per_day as f64 != SECONDS_PER_DAY {
            return Err(SimError::Validation(format!(
                "fast_dt*steps_per_day must equal one day ({SECONDS_PER_DAY} s), got {}*{}",
                self.fast_dt, self.steps_per_day
            )));
        }
        if self.slow_dt * self.slow_steps_per_day as f64 != DAYS_PER_MASTER_DAY {
            return Err(SimError::Validation(format!(
                "slow_dt*slow_steps_per_day must equal one day ({DAYS_PER_MASTER_DAY}), got \
                 {}*{}",
                self.slow_dt, self.slow_steps_per_day
            )));
        }
        self.fast_per_slow(order).map(|_| ())
    }

    /// Fast sub-steps after each slow step under `Interleaved`; refuses an uneven split.
    fn fast_per_slow(&self, order: DayOrder) -> Result<u64, SimError> {
        if order == DayOrder::SlowFirst {
            return Ok(self.steps_per_day);
        }
        if self.slow_steps_per_day == 0
            || !self.steps_per_day.is_multiple_of(self.slow_steps_per_day)
        {
            return Err(SimError::Validation(format!(
                "the interleaved day order needs steps_per_day ({}) to divide evenly by \
                 slow_steps_per_day ({}): each slow step must be followed by the same stretch \
                 of fast time",
                self.steps_per_day, self.slow_steps_per_day
            )));
        }
        Ok(self.steps_per_day / self.slow_steps_per_day)
    }

    /// One slow step, observed; returns the new state.
    fn slow_step(
        &self,
        state: State,
        totals: &mut SideTotals,
        observe: &mut dyn FnMut(Side, &State, &State),
    ) -> Result<State, SimError> {
        let report = self
            .slow
            .step_report(&state, self.slow_resolver, self.slow_dt)?;
        observe(Side::Slow, &state, &report.state);
        totals.slow_rationed += report.rationed;
        totals.events.extend(report.events);
        Ok(report.state)
    }

    /// `count` fast sub-steps, each conservation-asserted over the whole shared ledger
    /// exactly as the reference day does it, and observed.
    fn fast_steps(
        &self,
        mut state: State,
        count: u64,
        totals: &mut SideTotals,
        observe: &mut dyn FnMut(Side, &State, &State),
    ) -> Result<State, SimError> {
        for _ in 0..count {
            let before = state.clone();
            let report = self
                .fast
                .substep(&state, self.fast_resolver, self.fast_dt)?;
            state = report.state;
            assert_conserved_default(&before, &state)?;
            observe(Side::Fast, &before, &state);
            totals.fast_rationed += report.rationed;
            totals.events.extend(report.events);
        }
        Ok(state)
    }

    /// Advance one master day in `order`. The re-sow hook is consulted first, as in the
    /// reference. Under `SlowFirst` the arithmetic is the reference day's, operation for
    /// operation — a test holds it to that, bit for bit.
    pub fn advance_day(
        &self,
        order: DayOrder,
        state: &State,
        totals: &mut SideTotals,
        observe: &mut dyn FnMut(Side, &State, &State),
    ) -> Result<State, SimError> {
        let fast_per_slow = self.fast_per_slow(order)?;
        let mut state = state.clone();
        if let Some(reset_fn) = self.slow_reset {
            if let Some(reset_state) = reset_fn(state.n, &state)? {
                assert_conserved_default(&state, &reset_state)?;
                observe(Side::Reset, &state, &reset_state);
                state = reset_state;
            }
        }
        match order {
            DayOrder::SlowFirst => {
                for _ in 0..self.slow_steps_per_day {
                    state = self.slow_step(state, totals, observe)?;
                }
                state = self.fast_steps(state, self.steps_per_day, totals, observe)?;
            }
            DayOrder::Interleaved => {
                for _ in 0..self.slow_steps_per_day {
                    state = self.slow_step(state, totals, observe)?;
                    state = self.fast_steps(state, fast_per_slow, totals, observe)?;
                }
            }
        }
        Ok(state)
    }

    /// Run `days` master days in `order`. Returns one state per master day (length
    /// `days + 1`, as the reference runner) and the run's split totals.
    pub fn run(
        &self,
        order: DayOrder,
        initial: State,
        days: usize,
        observe: &mut dyn FnMut(Side, &State, &State),
    ) -> Result<(Vec<State>, SideTotals), SimError> {
        self.validate(order)?;
        let mut totals = SideTotals::default();
        let mut state = initial;
        let mut states = vec![state.clone()];
        for _ in 0..days {
            state = self.advance_day(order, &state, &mut totals, observe)?;
            states.push(state.clone());
        }
        Ok((states, totals))
    }
}
