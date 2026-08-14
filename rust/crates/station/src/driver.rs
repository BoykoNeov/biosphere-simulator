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
