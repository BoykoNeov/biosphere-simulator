//! The station's two-rate master-step driver — the port of `station.driver` (P7.5).
//!
//! Couples a **day-scale** slow domain (the biosphere: `dt` in days, a `thermal_time` aux
//! that must advance) to a **second-scale** fast domain (the cabin / Power: `dt = 60`/`3600`
//! s). `simcore::multirate` cannot bridge these — it splits ONE shared master `dt`, and
//! `substep` freezes the biosphere aux — so the driver does the operator split (Lie,
//! **interleaved**) **by hand**: per master day, in [`day_groups`]' equal groups, the slow
//! domain takes its group's `step_report`s (advancing aux **and** `n`), then the fast domain
//! takes the group's share of the day's `steps_per_day` `substep` calls (keeping `n`).
//!
//! ⚠ Until 2026-09-30 the day ran **slow-first** — every slow step, then every fast step —
//! so all four plant quarter-days drew on the cabin air as it stood at dawn and the crew's
//! exhalation for the day arrived only afterwards. Adopted on the user's call;
//! `docs/plans/post-roadmap-intraday-gas-exchange.md` §8. The retired order survives only in
//! the lab section at the bottom of this file.
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
/// re-sow state after a conservation check), then `slow_steps_per_day` times run one slow
/// `step_report` (advancing the phenology aux **and** `n`) followed by, group by group
/// ([`day_groups`]), its share of the fast `substep`s at `fast_dt` (keeping `n`),
/// asserting conservation over the full shared ledger after **each** fast sub-step. The fast
/// side refills the shared pools between the slow side's draws. Returns
/// `(next_state, day_rationed, day_events)`; refuses a split with no equal grouping.
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
    let groups = day_groups(steps_per_day, slow_steps_per_day)?;
    for _ in 0..groups.count {
        // Slow operator: this group's slow sub-steps (each advances the phenology aux AND n).
        for _ in 0..groups.slow {
            let slow_report = slow_integrator.step_report(&state, slow_resolver, slow_dt)?;
            state = slow_report.state;
            total_rationed += slow_report.rationed;
            events.extend(slow_report.events);
        }
        // Fast operator: this group's share of the day at fast_dt (n kept). substep skips
        // the conservation assert, so we own it here — after each sub-step, over the full
        // shared ledger — keeping the every-step teeth.
        for _ in 0..groups.fast {
            let before = state.clone();
            let fast_report = fast_integrator.substep(&state, fast_resolver, fast_dt)?;
            state = fast_report.state;
            assert_conserved_default(&before, &state)?;
            total_rationed += fast_report.rationed;
            events.extend(fast_report.events);
        }
    }
    Ok((state, total_rationed, events))
}

/// How a master day interleaves its two operators: `count` equal groups, each `slow` slow
/// steps then `fast` fast sub-steps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DayGroups {
    /// Groups per master day: the greatest common divisor of the two step counts.
    pub count: u64,
    /// Slow steps per group.
    pub slow: u64,
    /// Fast sub-steps per group.
    pub fast: u64,
}

/// The finest **equal** grouping of a master day: `gcd(steps_per_day, slow_steps_per_day)`
/// groups, each `slow_steps_per_day / gcd` slow steps followed by `steps_per_day / gcd` fast
/// ones.
///
/// When the fast count divides evenly by the slow one (every sealed/greenhouse scenario: 1440
/// cabin minutes over 16 plant steps) this is one slow step then its 90 fast ones, the
/// interleaved day adopted 2026-09-30, operation for operation. Until the 1/16-day step it
/// **refused** any other split; the lamp scenarios' 24 hourly power steps over 16 plant steps
/// made that rule unrunnable, and on the user's call (2026-09-30, "loosen the even-split
/// rule", `docs/plans/post-roadmap-step-sixteenth.md`) it now groups them: 8 groups of 2 plant
/// steps then 3 power hours.
///
/// ⚠ Still refused: counts sharing no common factor (with more than one slow step), because
/// the finest equal grouping is then the whole day, every slow step before every fast one,
/// which is the order the interleaving retired. An integer division that dropped a remainder
/// is impossible here by construction.
pub fn day_groups(steps_per_day: u64, slow_steps_per_day: u64) -> Result<DayGroups, SimError> {
    let gcd = {
        let (mut a, mut b) = (steps_per_day, slow_steps_per_day);
        while b != 0 {
            (a, b) = (b, a % b);
        }
        a
    };
    if steps_per_day == 0 || slow_steps_per_day == 0 || (gcd == 1 && slow_steps_per_day > 1) {
        return Err(SimError::Validation(format!(
            "the master day interleaves the two operators in equal groups, but steps_per_day \
             ({steps_per_day}) and slow_steps_per_day ({slow_steps_per_day}) share no common \
             factor: the only equal grouping is the whole day, every slow step before every \
             fast one, which is the retired slow-first order"
        )));
    }
    Ok(DayGroups {
        count: gcd,
        slow: slow_steps_per_day / gcd,
        fast: steps_per_day / gcd,
    })
}

/// Step `days` master days (slow ×`slow_steps_per_day` + fast ×`steps_per_day`), interleaved.
///
/// Per day: `slow_reset` (if given) is consulted first — a returned `Some(state)` is
/// conservation-checked then adopted; then, `slow_steps_per_day` times, the `slow_integrator`
/// runs one `step_report` at `slow_dt` (its own gate fires on each) and the `fast_integrator`
/// runs its group's share of the `substep` calls at `fast_dt` (`n`
/// kept), the driver asserting conservation after **each** over the full shared ledger. Returns `(states, total_rationed, events)`
/// with `states` one entry per **master day** — not per slow step — (length `days + 1`;
/// a golden pins the final one), so station trajectories stay day-indexed.
///
/// Requires `fast_dt · steps_per_day == 86400` s and `slow_dt · slow_steps_per_day == 1`
/// day, so both operators cover the same interval, and the two counts to share a common
/// factor ([`day_groups`]), so every group is followed by the same stretch of fast time.
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
    day_groups(steps_per_day, slow_steps_per_day)?;
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
// bridge. It exists so the lab can run a master day in either order and keep books per
// side. ⚠ The interleaved order was ADOPTED 2026-09-30: `advance_one_master_day` above took
// its body, and `DayOrder::Interleaved` is now the reference day, which a test holds bit for
// bit. `DayOrder::SlowFirst` is the RETIRED order, kept only so the lab can reproduce the
// record that retired it (`examples/intraday_exchange.rs`). §7 of the plan said to delete
// this section on adoption; §8 records why it was kept instead — its objection was to a
// switch in the reference, and nothing in the reference can reach this.

/// The order a master day runs its two operators in. **Lab-only.**
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DayOrder {
    /// The RETIRED order (the reference until 2026-09-30): every slow step, then every fast
    /// step (`P P P P c … c`).
    SlowFirst,
    /// The reference: one slow step, then that step's share of the fast steps, repeated
    /// (`P c…c P c…c P c…c P c…c`). The fast side refills a shared pool between the slow
    /// side's draws. With counts that do not divide evenly, the slow steps come in equal
    /// groups ([`day_groups`]); counts with no equal grouping are refused.
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
    /// the two step counts must have an equal grouping ([`day_groups`]).
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
        self.groups(order).map(|_| ())
    }

    /// The day's grouping in `order`: `SlowFirst` is one group of the whole day, and
    /// `Interleaved` is the reference's [`day_groups`] (refusing counts with no equal grouping).
    fn groups(&self, order: DayOrder) -> Result<DayGroups, SimError> {
        if order == DayOrder::SlowFirst {
            return Ok(DayGroups {
                count: 1,
                slow: self.slow_steps_per_day,
                fast: self.steps_per_day,
            });
        }
        day_groups(self.steps_per_day, self.slow_steps_per_day)
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
    /// reference. Under `Interleaved` the arithmetic is the reference day's, operation for
    /// operation — a test holds it to that, bit for bit.
    pub fn advance_day(
        &self,
        order: DayOrder,
        state: &State,
        totals: &mut SideTotals,
        observe: &mut dyn FnMut(Side, &State, &State),
    ) -> Result<State, SimError> {
        let groups = self.groups(order)?;
        let mut state = state.clone();
        if let Some(reset_fn) = self.slow_reset {
            if let Some(reset_state) = reset_fn(state.n, &state)? {
                assert_conserved_default(&state, &reset_state)?;
                observe(Side::Reset, &state, &reset_state);
                state = reset_state;
            }
        }
        for _ in 0..groups.count {
            for _ in 0..groups.slow {
                state = self.slow_step(state, totals, observe)?;
            }
            state = self.fast_steps(state, groups.fast, totals, observe)?;
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
