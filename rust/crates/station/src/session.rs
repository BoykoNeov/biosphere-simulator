//! A caller-driven, owned-state simulation session — the steppable core the Godot
//! front-end (Phase 8) advances one step at a time, and the foundation of the Phase-8
//! parity guarantee.
//!
//! The Phase-7 runners ([`crate::run_station`], [`crate::driver::run_master_day`] /
//! [`crate::sealed::run_sealed`]) **own their loop** and hand each produced `State` to an
//! `observer` callback. A game loop must own the loop instead — so this module inverts
//! that control: a [`SimSession`] holds `State + registries + resolvers` and advances on
//! [`SimSession::step`]. It calls the **exact same** per-step primitives in the **exact
//! same** order as the runners ([`simcore::integrator::EulerIntegrator::step_report`] for
//! single-rate; [`crate::driver::advance_one_master_day`] for the two-rate master day), so
//! a session advanced `N` times is **bit-identical** to the corresponding
//! run-to-completion. That equivalence *is* the "the exact same simulation runs headless"
//! guarantee — and it is provable in `cargo test` with no Godot at all (see
//! `tests/session_parity.rs`). The genuine *cross-boundary* check — the same scenario
//! driven through the actual `gdext` cdylib Godot loads — is a separate first-class
//! deliverable in Steps 1 and 8 (an intra-process test cannot see FP-environment
//! divergence, e.g. FTZ/DAZ flags a game engine may set per-thread).
//!
//! **Determinism corollary (banked):** the core RNG is counter-based, keyed by
//! `(seed, key, n)` — there is **no sequential RNG state to serialize**. A session's
//! entire resumable identity is its current `State` (which carries `n` and the seed), so
//! pause / resume / save / load are trivially deterministic: reconstruct the registries
//! from the (fixed-palette) scenario id, load the `State`, and stepping continues
//! bit-identically. Phase-8 save/load (Step 7) rests on this.

use simcore::environment::SourceResolver;
use simcore::error::SimError;
use simcore::events::Event;
use simcore::integrator::EulerIntegrator;
use simcore::state::State;

use crate::driver::{advance_one_master_day, OwnedResetHook, SECONDS_PER_DAY};

/// The per-mode machinery a [`SimSession`] owns. A single-rate session ticks one
/// `step_report` per [`SimSession::step`]; a two-rate (sealed / greenhouse) session ticks
/// one **master day** — one slow biosphere step + `steps_per_day` fast sub-steps — per
/// step (`n` is the day count under the `substep`-keeps-`n` design).
enum Mode {
    SingleRate {
        integrator: EulerIntegrator,
        resolver: SourceResolver,
        dt: f64,
    },
    TwoRate {
        slow_integrator: EulerIntegrator,
        fast_integrator: EulerIntegrator,
        slow_resolver: SourceResolver,
        fast_resolver: SourceResolver,
        steps_per_day: u64,
        slow_dt: f64,
        fast_dt: f64,
        reset: Option<OwnedResetHook>,
    },
}

/// A caller-driven simulation: owns the current `State` and the machinery to advance it
/// one natural unit at a time (see the module docs). Godot's game loop holds one of these
/// and calls [`SimSession::step`] / [`SimSession::step_n`]; the accumulated `rationed`
/// count and `events` mirror what the run-to-completion runners return.
pub struct SimSession {
    mode: Mode,
    state: State,
    total_rationed: u64,
    events: Vec<Event>,
}

impl SimSession {
    /// A **single-rate** session (Power / Thermal / cabin: one `step_report` per
    /// [`step`](SimSession::step)). Mirrors [`crate::run_station`]'s setup; `step` mirrors
    /// its loop body exactly (minus the observer callback).
    pub fn single_rate(
        integrator: EulerIntegrator,
        initial: State,
        resolver: SourceResolver,
        dt: f64,
    ) -> Self {
        SimSession {
            mode: Mode::SingleRate {
                integrator,
                resolver,
                dt,
            },
            state: initial,
            total_rationed: 0,
            events: Vec::new(),
        }
    }

    /// A **two-rate** session (sealed / greenhouse: one **master day** per
    /// [`step`](SimSession::step) — one slow biosphere `step_report` + `steps_per_day`
    /// fast sub-steps, the conservation gate re-asserted after each). Mirrors
    /// [`crate::sealed::run_sealed`]'s setup; `step` calls the same
    /// [`crate::driver::advance_one_master_day`] its loop does. Pass the re-sow hook from
    /// [`crate::sealed::sealed_reset_hook`] (or `None` for the greenhouse's no-reset seam).
    ///
    /// Requires `fast_dt · steps_per_day == 86400` (one day), exactly like
    /// [`crate::driver::run_master_day`] — validated once here at construction.
    #[allow(clippy::too_many_arguments)]
    pub fn two_rate(
        slow_integrator: EulerIntegrator,
        fast_integrator: EulerIntegrator,
        initial: State,
        slow_resolver: SourceResolver,
        fast_resolver: SourceResolver,
        steps_per_day: u64,
        slow_dt: f64,
        fast_dt: f64,
        reset: Option<OwnedResetHook>,
    ) -> Result<Self, SimError> {
        if fast_dt * steps_per_day as f64 != SECONDS_PER_DAY {
            return Err(SimError::Validation(format!(
                "fast_dt*steps_per_day must equal one day ({SECONDS_PER_DAY} s) so n stays \
                 the day count, got {fast_dt}*{steps_per_day} = {}",
                fast_dt * steps_per_day as f64
            )));
        }
        Ok(SimSession {
            mode: Mode::TwoRate {
                slow_integrator,
                fast_integrator,
                slow_resolver,
                fast_resolver,
                steps_per_day,
                slow_dt,
                fast_dt,
                reset,
            },
            state: initial,
            total_rationed: 0,
            events: Vec::new(),
        })
    }

    /// The current simulation state (advances with each [`step`](SimSession::step)).
    pub fn state(&self) -> &State {
        &self.state
    }

    /// The current integer step count `n` — steps taken (single-rate) or master days
    /// taken (two-rate; `n` is the day count).
    pub fn n(&self) -> u64 {
        self.state.n
    }

    /// Total flows scaled by the Euler backstop so far (a golden run asserts this is 0).
    pub fn total_rationed(&self) -> u64 {
        self.total_rationed
    }

    /// Extinction events emitted so far, in order.
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Advance **one natural unit**: one `step_report` (single-rate) or one master day
    /// (two-rate). Bit-identical to the corresponding single iteration of the runner's
    /// loop, because it calls the same primitive.
    pub fn step(&mut self) -> Result<(), SimError> {
        match &self.mode {
            Mode::SingleRate {
                integrator,
                resolver,
                dt,
            } => {
                let report = integrator.step_report(&self.state, resolver, *dt)?;
                self.state = report.state;
                self.total_rationed += report.rationed;
                self.events.extend(report.events);
            }
            Mode::TwoRate {
                slow_integrator,
                fast_integrator,
                slow_resolver,
                fast_resolver,
                steps_per_day,
                slow_dt,
                fast_dt,
                reset,
            } => {
                let (next, day_rationed, day_events) = advance_one_master_day(
                    slow_integrator,
                    fast_integrator,
                    &self.state,
                    slow_resolver,
                    fast_resolver,
                    *steps_per_day,
                    *slow_dt,
                    *fast_dt,
                    reset.as_deref(),
                )?;
                self.state = next;
                self.total_rationed += day_rationed;
                self.events.extend(day_events);
            }
        }
        Ok(())
    }

    /// Advance `k` natural units (fast-forward). Because observation is separate from
    /// stepping, "advance `k` without observing intermediates" is just this loop — the
    /// free, parity-safe fast-forward primitive Phase-8 Step 3 drives off the render
    /// thread.
    pub fn step_n(&mut self, k: u64) -> Result<(), SimError> {
        for _ in 0..k {
            self.step()?;
        }
        Ok(())
    }
}
