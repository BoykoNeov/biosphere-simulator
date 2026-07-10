//! Phase-8 (P8.3) — **time controls off the render thread.**
//!
//! # Why a worker thread at all (advisor #3)
//!
//! A two-rate [`CoreSession::step`] is one **master day** = one slow biosphere step +
//! `steps_per_day` (1440) fast cabin sub-steps, so fast-forwarding decades is *minutes* of
//! compute even in release. Stepping on Godot's render thread would freeze the UI. So a
//! [`TimeController`] spawns **one** long-lived worker thread that owns the stepping; the
//! render thread only sends commands ([`Cmd`]) down an `mpsc` channel and reads the latest
//! published snapshot out of an `Arc<Mutex<`[`Shared`]`>>` each frame — it never blocks on a
//! step, however expensive.
//!
//! # No engine change, no `Send` bound, no loader (the design that avoids all three)
//!
//! [`CoreSession`] is **not `Send`** — its resolvers are `Box<dyn Fn(u64,f64)->f64>`
//! (no `+ Send`), a frozen engine type we do not touch (the Phase-8 purity invariant). So
//! the session is **built on, lives on, and dies on the worker thread** — it never crosses a
//! thread boundary. Only plain-data `Send` types cross: [`Cmd`] down the channel, and the
//! [`Shared`] JSON/counters back up. That also means no state loader is needed here (Step 7's
//! concern): the worker is the single owner from birth.
//!
//! # The FP-environment verification is per-thread (advisor #3)
//!
//! Step 1 verified FTZ/DAZ are off on the render thread. Stepping now happens on a *different*
//! thread, so the check must move there. The worker reads MXCSR **on itself** at startup and
//! publishes [`Shared::fp_clean`]; because the sim only ever runs on this one worker, verifying
//! it here is exactly "the FP env of the thread the sim can run on." (A Rust `std::thread`
//! inherits the process-default IEEE env — clean by construction — so this is expected to pass;
//! it is a guard against a future design that hands stepping to a Godot-managed pool thread
//! that set FTZ/DAZ for SIMD throughput.)
//!
//! # Parity
//!
//! The worker calls the **same** [`CoreSession::step`] the synchronous [`crate::SimSession`]
//! and the Phase-7 runners call, so a worker fast-forwarded to `N` is **bit-identical** to a
//! synchronous `step_n(N)`. That is proved in `cargo test` with no Godot at all (the
//! `worker_fast_forward_is_bit_exact_vs_synchronous_*` tests compare hex-float snapshots).

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use godot::prelude::*;

use simcore::error::SimError;
use simcore::snapshot::from_engine;
use station::display::{project, DisplayContext};
use station::session::SimSession as CoreSession;

use crate::{build_session, fp_flags_clean, read_mxcsr};

/// How long a stepping call may run before it must publish progress and re-check for
/// commands. Bounds fast-forward responsiveness: a two-rate master day exceeds this, so the
/// worker publishes after each day; single-rate steps rip through thousands per budget. The
/// budget affects only *batching* (when we publish / poll commands) — never the trajectory,
/// so it is parity-neutral.
const STEP_BUDGET: Duration = Duration::from_millis(6);

/// The play-mode tick: after each batch of `speed` steps the worker sleeps this long, so
/// playback advances at a watchable, CPU-friendly rate (vs fast-forward, which does not sleep).
const PLAY_TICK: Duration = Duration::from_millis(16);

/// Default play speed: sim steps advanced per [`PLAY_TICK`].
const DEFAULT_SPEED: u64 = 1;

// ---------------------------------------------------------------------------
// The message types crossing the thread boundary (both plain-data `Send`).
// ---------------------------------------------------------------------------

/// A command from the render thread to the stepping worker.
pub(crate) enum Cmd {
    /// Advance continuously at the current speed (throttled by [`PLAY_TICK`]).
    Play,
    /// Stop advancing; the worker blocks until the next command.
    Pause,
    /// Advance exactly one natural unit, then pause.
    SingleStep,
    /// Advance as fast as possible up to this step count `n`, publishing progress, then pause.
    FastForwardTo(u64),
    /// Steps advanced per [`PLAY_TICK`] while playing (clamped to ≥ 1).
    SetSpeed(u64),
    /// Stop the worker and let it exit (sent on [`TimeController`] drop / rebuild).
    Shutdown,
}

/// The latest simulation snapshot the worker publishes and the render thread reads. Plain
/// data so it crosses the boundary freely; the two JSON strings are computed **on the worker**
/// (the display projection for the UI, the hex-float snapshot for the parity/FP smoke).
#[derive(Default)]
pub(crate) struct Shared {
    /// The P8.2 display projection (plain decimal floats) — what the dashboard renders.
    observation_json: String,
    /// The `sim_io` hex-float snapshot (bit-exact codec) — the cross-boundary parity anchor.
    snapshot_json: String,
    n: u64,
    total_rationed: u64,
    playing: bool,
    fast_forwarding: bool,
    target_n: u64,
    /// Raw MXCSR read on the worker thread (diagnostic; `fp_clean` is the assertion).
    mxcsr: u32,
    /// FTZ and DAZ both OFF on the worker thread — the per-thread FP verification (advisor #3).
    fp_clean: bool,
    /// A conservation/arbitration error string if stepping faulted (the worker then pauses).
    error: Option<String>,
}

/// The internal run state of the worker's loop.
enum RunState {
    Paused,
    SteppingOnce,
    Playing,
    FastForwarding(u64),
}

// ---------------------------------------------------------------------------
// The worker: builds and owns the session, runs the command loop.
// ---------------------------------------------------------------------------

/// The channel + shared state + join handle the [`TimeController`] holds for one worker.
pub(crate) struct WorkerHandles {
    tx: Sender<Cmd>,
    shared: Arc<Mutex<Shared>>,
    handle: JoinHandle<()>,
}

/// Spawn the stepping worker for a fixed-palette scenario id. Blocks briefly until the worker
/// reports whether the session built (so the caller gets a synchronous success/failure like
/// the sync [`crate::SimSession::build`]), then returns the live handles. The session itself
/// is built **on the worker thread** — never on the caller's — so the FP verification and all
/// stepping stay on that one thread.
pub(crate) fn spawn_worker(scenario_id: &str) -> Result<WorkerHandles, String> {
    let (tx, rx) = mpsc::channel::<Cmd>();
    let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();
    let shared = Arc::new(Mutex::new(Shared::default()));
    let shared_worker = Arc::clone(&shared);
    let id = scenario_id.to_string();

    let handle = std::thread::Builder::new()
        .name(format!("sim-worker-{id}"))
        .spawn(move || match build_session(&id) {
            Ok((session, ctx)) => worker_loop(session, ctx, rx, shared_worker, ready_tx),
            Err(err) => {
                // Report the build failure; nothing to run.
                ready_tx.send(Err(format!("{err:?}"))).ok();
            }
        })
        .map_err(|e| format!("thread spawn failed: {e}"))?;

    match ready_rx.recv() {
        Ok(Ok(())) => Ok(WorkerHandles { tx, shared, handle }),
        Ok(Err(build_err)) => {
            handle.join().ok();
            Err(build_err)
        }
        Err(_) => {
            handle.join().ok();
            Err("worker exited before reporting readiness".to_string())
        }
    }
}

/// The worker's stepping loop. Reads the FP env **on this thread**, publishes an initial
/// snapshot, signals readiness, then services commands until [`Cmd::Shutdown`] or the last
/// sender drops.
fn worker_loop(
    mut session: CoreSession,
    ctx: DisplayContext,
    rx: Receiver<Cmd>,
    shared: Arc<Mutex<Shared>>,
    ready: Sender<Result<(), String>>,
) {
    // The P8.3 per-thread FP verification: FTZ/DAZ on the very thread that runs `step`.
    let mxcsr = read_mxcsr();
    let fp_clean = fp_flags_clean(mxcsr);

    let mut speed = DEFAULT_SPEED;
    let mut run = RunState::Paused;
    let mut error: Option<String> = None;

    publish(&shared, &session, &ctx, &run, mxcsr, fp_clean, &error);
    ready.send(Ok(())).ok();

    loop {
        match run {
            RunState::Paused => match rx.recv() {
                // Block until a command arrives; a Shutdown / dropped sender ends the loop.
                Ok(Cmd::Shutdown) | Err(_) => break,
                Ok(cmd) => apply(cmd, &mut run, &mut speed),
            },
            RunState::SteppingOnce => {
                step_batch(&mut session, 1, None, &mut run, &mut error);
                run = RunState::Paused;
                publish(&shared, &session, &ctx, &run, mxcsr, fp_clean, &error);
            }
            RunState::Playing => {
                if drain_commands(&rx, &mut run, &mut speed) {
                    break;
                }
                if !matches!(run, RunState::Playing) {
                    continue;
                }
                step_batch(&mut session, speed, None, &mut run, &mut error);
                publish(&shared, &session, &ctx, &run, mxcsr, fp_clean, &error);
                std::thread::sleep(PLAY_TICK);
            }
            RunState::FastForwarding(target) => {
                if drain_commands(&rx, &mut run, &mut speed) {
                    break;
                }
                if !matches!(run, RunState::FastForwarding(_)) {
                    continue;
                }
                if session.n() >= target {
                    run = RunState::Paused;
                    publish(&shared, &session, &ctx, &run, mxcsr, fp_clean, &error);
                    continue;
                }
                // A budgeted batch toward the target: at least one step, then publish
                // progress and re-check commands (so Pause / Shutdown interrupt promptly).
                step_batch(&mut session, u64::MAX, Some(target), &mut run, &mut error);
                publish(&shared, &session, &ctx, &run, mxcsr, fp_clean, &error);
            }
        }
    }
}

/// Map a command onto the run state / speed. Does not step (SingleStep just arms the
/// [`RunState::SteppingOnce`] transient). [`Cmd::Shutdown`] is handled at the call sites.
fn apply(cmd: Cmd, run: &mut RunState, speed: &mut u64) {
    match cmd {
        Cmd::Play => *run = RunState::Playing,
        Cmd::Pause => *run = RunState::Paused,
        Cmd::SingleStep => *run = RunState::SteppingOnce,
        Cmd::FastForwardTo(target) => *run = RunState::FastForwarding(target),
        Cmd::SetSpeed(s) => *speed = s.max(1),
        Cmd::Shutdown => {}
    }
}

/// Non-blocking drain of all pending commands (used from Playing / FastForwarding, which must
/// not block). Returns `true` if a [`Cmd::Shutdown`] was seen (the loop should break).
fn drain_commands(rx: &Receiver<Cmd>, run: &mut RunState, speed: &mut u64) -> bool {
    loop {
        match rx.try_recv() {
            Ok(Cmd::Shutdown) => return true,
            Ok(cmd) => apply(cmd, run, speed),
            Err(mpsc::TryRecvError::Empty) => return false,
            Err(mpsc::TryRecvError::Disconnected) => return true,
        }
    }
}

/// Step until `max_steps` taken **or** `until` reached **or** the time budget elapsed —
/// whichever first, but always at least one step so a single slow master day still makes
/// progress. On a conservation/arbitration fault, records the error and pauses.
fn step_batch(
    session: &mut CoreSession,
    max_steps: u64,
    until: Option<u64>,
    run: &mut RunState,
    error: &mut Option<String>,
) {
    let start = Instant::now();
    let mut did = 0u64;
    loop {
        if let Some(target) = until {
            if session.n() >= target {
                break;
            }
        }
        if did >= max_steps {
            break;
        }
        if did > 0 && start.elapsed() >= STEP_BUDGET {
            break;
        }
        if let Err(err) = session.step() {
            record_fault(&err, run, error);
            break;
        }
        did += 1;
    }
}

/// Record a stepping fault and pause. Errors should not happen on the well-fed frozen
/// scenarios (a golden run asserts `rationed == 0`), but a caller-driven session must fail
/// safe rather than panic on the worker thread.
fn record_fault(err: &SimError, run: &mut RunState, error: &mut Option<String>) {
    *error = Some(format!("{err:?}"));
    *run = RunState::Paused;
}

/// Publish the current session snapshot to the shared cell (a short critical section — the
/// two JSON strings are built *before* the lock is taken).
fn publish(
    shared: &Arc<Mutex<Shared>>,
    session: &CoreSession,
    ctx: &DisplayContext,
    run: &RunState,
    mxcsr: u32,
    fp_clean: bool,
    error: &Option<String>,
) {
    let observation_json = project(
        session.state(),
        ctx,
        session.total_rationed(),
        session.events().len(),
        session.max_residual(),
    )
    .to_json();
    let snapshot_json = from_engine(session.state()).to_json();
    let (playing, fast_forwarding, target_n) = match run {
        RunState::Playing => (true, false, 0),
        RunState::FastForwarding(t) => (false, true, *t),
        _ => (false, false, 0),
    };

    let mut g = shared.lock().expect("sim worker shared state poisoned");
    g.observation_json = observation_json;
    g.snapshot_json = snapshot_json;
    g.n = session.n();
    g.total_rationed = session.total_rationed();
    g.playing = playing;
    g.fast_forwarding = fast_forwarding;
    g.target_n = target_n;
    g.mxcsr = mxcsr;
    g.fp_clean = fp_clean;
    g.error = error.clone();
}

// ---------------------------------------------------------------------------
// The GDExtension surface.
// ---------------------------------------------------------------------------

/// The Godot-facing time controller (registered Godot class name `TimeController`). Owns one
/// stepping worker thread; the render thread calls the command methods (`play` / `pause` /
/// `single_step` / `fast_forward_to` / `set_speed`) and reads the latest snapshot
/// (`observation_json` / `step_count` / …) each frame without ever blocking on a step.
#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct TimeController {
    worker: Option<WorkerHandles>,
    base: Base<RefCounted>,
}

impl TimeController {
    /// Lock and read the shared cell, if a worker is running.
    fn shared(&self) -> Option<MutexGuard<'_, Shared>> {
        self.worker
            .as_ref()
            .map(|w| w.shared.lock().expect("sim worker shared state poisoned"))
    }

    /// Send a command to the worker (ignored if none is running or the worker has exited).
    fn send(&self, cmd: Cmd) {
        if let Some(w) = self.worker.as_ref() {
            w.tx.send(cmd).ok();
        }
    }

    /// Stop and join the current worker, if any.
    fn teardown(&mut self) {
        if let Some(w) = self.worker.take() {
            w.tx.send(Cmd::Shutdown).ok();
            w.handle.join().ok();
        }
    }
}

#[godot_api]
impl TimeController {
    /// Build (and start, paused) the stepping worker for a fixed-palette scenario id. Returns
    /// `false` (and logs) on an unknown id or build error. Idempotently replaces any prior
    /// worker. Palette: `"cabin_gas"`, `"station"`, `"greenhouse"` (the two-rate one).
    #[func]
    fn build(&mut self, scenario_id: GString) -> bool {
        self.teardown();
        match spawn_worker(&scenario_id.to_string()) {
            Ok(worker) => {
                self.worker = Some(worker);
                true
            }
            Err(err) => {
                godot_error!("TimeController.build failed: {err}");
                false
            }
        }
    }

    /// Advance continuously at the current speed (throttled — watchable playback).
    #[func]
    fn play(&self) {
        self.send(Cmd::Play);
    }

    /// Stop advancing.
    #[func]
    fn pause(&self) {
        self.send(Cmd::Pause);
    }

    /// Advance exactly one natural unit, then pause.
    #[func]
    fn single_step(&self) {
        self.send(Cmd::SingleStep);
    }

    /// Fast-forward (off the render thread) up to step count `n`, then pause. A no-op if the
    /// worker is already at or past `n`. Negative `n` is ignored.
    #[func]
    fn fast_forward_to(&self, n: i64) {
        if n < 0 {
            godot_error!("TimeController.fast_forward_to negative n={n}");
            return;
        }
        self.send(Cmd::FastForwardTo(n as u64));
    }

    /// Set the playback speed: sim steps advanced per ~16 ms tick while playing (clamped ≥ 1).
    #[func]
    fn set_speed(&self, steps_per_tick: i64) {
        self.send(Cmd::SetSpeed(steps_per_tick.max(1) as u64));
    }

    /// The current integer step count `n` (`-1` before [`build`](Self::build)).
    #[func]
    fn step_count(&self) -> i64 {
        self.shared().map(|s| s.n as i64).unwrap_or(-1)
    }

    /// The **display projection** JSON (P8.2) — the multi-domain dashboard read. Empty string
    /// before [`build`](Self::build). Read every frame off the shared snapshot; never blocks
    /// on a step.
    #[func]
    fn observation_json(&self) -> GString {
        self.shared()
            .map(|s| GString::from(s.observation_json.as_str()))
            .unwrap_or_default()
    }

    /// The `sim_io` hex-float snapshot JSON — the cross-boundary parity anchor (same codec as
    /// the sync [`crate::SimSession::snapshot_json`]). Empty string before [`build`](Self::build).
    #[func]
    fn snapshot_json(&self) -> GString {
        self.shared()
            .map(|s| GString::from(s.snapshot_json.as_str()))
            .unwrap_or_default()
    }

    /// True while playback is advancing.
    #[func]
    fn is_playing(&self) -> bool {
        self.shared().map(|s| s.playing).unwrap_or(false)
    }

    /// True while a fast-forward is in progress.
    #[func]
    fn is_fast_forwarding(&self) -> bool {
        self.shared().map(|s| s.fast_forwarding).unwrap_or(false)
    }

    /// The fast-forward target `n` (0 when not fast-forwarding).
    #[func]
    fn target_n(&self) -> i64 {
        self.shared().map(|s| s.target_n as i64).unwrap_or(0)
    }

    /// Total flows scaled by the Euler backstop so far (a golden run asserts `0`).
    #[func]
    fn total_rationed(&self) -> i64 {
        self.shared().map(|s| s.total_rationed as i64).unwrap_or(0)
    }

    /// FTZ and DAZ both OFF **on the worker (stepping) thread** — the P8.3 per-thread FP
    /// verification (advisor #3). `false` before [`build`](Self::build).
    #[func]
    fn fp_clean(&self) -> bool {
        self.shared().map(|s| s.fp_clean).unwrap_or(false)
    }

    /// Raw MXCSR read on the worker thread (diagnostic; [`fp_clean`](Self::fp_clean) is the
    /// assertion). `0` before [`build`](Self::build).
    #[func]
    fn worker_mxcsr(&self) -> i64 {
        self.shared().map(|s| s.mxcsr as i64).unwrap_or(0)
    }

    /// The last stepping fault message, or empty string if healthy / not built. A golden run
    /// stays empty (`rationed == 0`).
    #[func]
    fn error_message(&self) -> GString {
        self.shared()
            .and_then(|s| s.error.clone())
            .map(|e| GString::from(e.as_str()))
            .unwrap_or_default()
    }
}

impl Drop for TimeController {
    fn drop(&mut self) {
        self.teardown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Poll the shared cell until `pred` holds or `timeout` elapses. Returns whether it held.
    fn wait_until(
        shared: &Arc<Mutex<Shared>>,
        pred: impl Fn(&Shared) -> bool,
        timeout: Duration,
    ) -> bool {
        let start = Instant::now();
        loop {
            if pred(&shared.lock().unwrap()) {
                return true;
            }
            if start.elapsed() > timeout {
                return false;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    fn shutdown(w: WorkerHandles) {
        w.tx.send(Cmd::Shutdown).ok();
        w.handle.join().ok();
    }

    /// The load-bearing P8.3 check: the worker reads a clean FP env **on its own thread**.
    #[test]
    fn worker_reports_fp_clean_on_its_stepping_thread() {
        let w = spawn_worker("cabin_gas").unwrap();
        assert!(
            w.shared.lock().unwrap().fp_clean,
            "the stepping thread must have FTZ/DAZ off"
        );
        shutdown(w);
    }

    #[test]
    fn build_rejects_unknown_scenario() {
        assert!(spawn_worker("no_such_scenario").is_err());
    }

    #[test]
    fn single_step_advances_exactly_one_then_pauses() {
        let w = spawn_worker("cabin_gas").unwrap();
        assert_eq!(w.shared.lock().unwrap().n, 0, "starts at n=0");
        w.tx.send(Cmd::SingleStep).ok();
        assert!(wait_until(&w.shared, |s| s.n == 1, Duration::from_secs(2)));
        // Stays put (no auto-advance after a single step).
        std::thread::sleep(Duration::from_millis(80));
        assert_eq!(w.shared.lock().unwrap().n, 1);
        shutdown(w);
    }

    #[test]
    fn play_advances_then_pause_stops() {
        let w = spawn_worker("cabin_gas").unwrap();
        w.tx.send(Cmd::SetSpeed(50)).ok();
        w.tx.send(Cmd::Play).ok();
        assert!(wait_until(&w.shared, |s| s.n > 0, Duration::from_secs(2)));
        w.tx.send(Cmd::Pause).ok();
        // Let the pause settle, then confirm it is truly stopped.
        std::thread::sleep(Duration::from_millis(80));
        let a = w.shared.lock().unwrap().n;
        std::thread::sleep(Duration::from_millis(80));
        let b = w.shared.lock().unwrap().n;
        assert_eq!(a, b, "a paused worker must not advance");
        shutdown(w);
    }

    #[test]
    fn fast_forward_reaches_target_and_pauses() {
        let w = spawn_worker("cabin_gas").unwrap();
        let target = 300u64;
        w.tx.send(Cmd::FastForwardTo(target)).ok();
        assert!(wait_until(
            &w.shared,
            |s| s.n == target && !s.fast_forwarding,
            Duration::from_secs(10),
        ));
        shutdown(w);
    }

    /// The threaded-parity teeth: a session fast-forwarded to `N` **on the worker thread** is
    /// bit-identical (hex-float snapshot) to a synchronous `step_n(N)` — "the sim on the
    /// worker thread == the sim on the main thread." Single-rate (`cabin_gas`, Tier-1).
    #[test]
    fn worker_fast_forward_is_bit_exact_vs_synchronous_cabin_gas() {
        threaded_parity("cabin_gas", station::scenario::CABIN_GAS_STEPS);
    }

    /// Same, for the **two-rate** greenhouse (the expensive 1440-substep/day path the worker
    /// thread exists to carry).
    #[test]
    fn worker_fast_forward_is_bit_exact_vs_synchronous_greenhouse() {
        let days = station::scenario::greenhouse_scenario().days as u64;
        threaded_parity("greenhouse", days);
    }

    fn threaded_parity(scenario_id: &str, target: u64) {
        // Reference: synchronous session stepped `target`, hex-float snapshot.
        let (mut sync, _ctx) = build_session(scenario_id).unwrap();
        sync.step_n(target).unwrap();
        let want = from_engine(sync.state()).to_json();

        // Worker: fast-forward to `target` on its own thread, read its published snapshot.
        let w = spawn_worker(scenario_id).unwrap();
        w.tx.send(Cmd::FastForwardTo(target)).ok();
        assert!(wait_until(
            &w.shared,
            |s| s.n == target && !s.fast_forwarding,
            Duration::from_secs(60),
        ));
        let got = w.shared.lock().unwrap().snapshot_json.clone();
        let err = w.shared.lock().unwrap().error.clone();
        shutdown(w);

        assert_eq!(err, None, "worker stepping faulted: {err:?}");
        assert_eq!(
            got, want,
            "worker-thread fast-forward must be bit-identical to synchronous step_n"
        );
    }
}
