//! Phase-8 (P8.1) — the GDExtension binding crate. **The only crate that depends on
//! `gdext`.** It wraps the frozen [`station::session::SimSession`] (Phase-8 Step 0) so
//! Godot's game loop can drive the sim one step at a time and read stock values / a
//! bit-exact snapshot back out.
//!
//! # The purity invariant (Phase-8's "`git diff src/` empty")
//!
//! `gdext` types (`GString`, `Gd`, `Base`, …) appear **only** inside this crate. The
//! engine crates (`simcore`, `domains`, `station`) stay dependency-free and carry no
//! gdext types in their signatures — this binding *wraps* the session, it never modifies
//! it. That is what keeps the WASM-future and the C#-someday options open and keeps
//! "core is pure" true across the FFI boundary (the analogue of Phase-7 making
//! `crew::carbon_split` `pub` rather than importing outward).
//!
//! # Two things beyond a naive `stock_amount` getter (advisor)
//!
//! 1. **[`SimSession::snapshot_json`]** returns the *Rust-side* `sim_io` hex-float JSON
//!    ([`simcore::snapshot::from_engine`] → `to_json`). All float→string formatting stays
//!    inside the cdylib, so the cross-boundary parity smoke stays on the exact golden
//!    codec — the "bit-exact" claim is never hostage to GDScript's float printing.
//! 2. **[`SimSession::fp_clean`] / [`SimSession::mxcsr`]** read the x86 MXCSR *on the
//!    thread that calls `step`*. A passing bit-exact `cabin_gas` smoke catches reordering
//!    but is blind to flush-to-zero if the graph never produces a denormal, so the direct
//!    FTZ/DAZ read is a complementary check: a game engine that sets those per-thread for
//!    SIMD throughput would silently diverge from the IEEE-default headless run.

use godot::prelude::*;

use simcore::error::SimError;
use simcore::integrator::EulerIntegrator;
use simcore::snapshot::from_engine;

/// The frozen owned-state session (Phase-8 Step 0). Aliased so the *Godot* class below
/// can also be called `SimSession` (its registered Godot name) without shadowing.
use station::session::SimSession as CoreSession;

// ---------------------------------------------------------------------------
// Free functions (no gdext) — the testable core. `cargo test` exercises these
// without a Godot runtime; the `#[func]` methods are thin wrappers.
// ---------------------------------------------------------------------------

/// Build the owned session for a fixed-palette scenario id (confirmed decision #1:
/// "build systems" = a fixed, code-defined palette; registry construction stays in
/// Rust). Step 1 ships the one Tier-1 tripwire scenario; later steps grow the palette.
fn build_session(scenario_id: &str) -> Result<CoreSession, SimError> {
    match scenario_id {
        "cabin_gas" => build_cabin_gas(),
        other => Err(SimError::Validation(format!(
            "godot_bridge: unknown scenario id {other:?} (Step-1 palette: \"cabin_gas\")"
        ))),
    }
}

/// The coupled crew ↔ ECLSS `CABIN_GAS_SCENARIO` as a single-rate session — mirrors the
/// [`station::run_station`] setup (and `tests/session_parity.rs`) exactly.
fn build_cabin_gas() -> Result<CoreSession, SimError> {
    let crew = domains::params::crew();
    let eclss = domains::params::eclss();
    let scenario = station::scenario::CABIN_GAS_SCENARIO;
    let (state, registry) = station::cabin::build_cabin(&crew, &eclss, &scenario)?;
    let resolver = station::cabin::cabin_resolver(&scenario)?;
    Ok(CoreSession::single_rate(
        EulerIntegrator::new(registry),
        state,
        resolver,
        scenario.dt_seconds,
    ))
}

const MXCSR_FTZ: u32 = 1 << 15; // flush-to-zero          (0x8000)
const MXCSR_DAZ: u32 = 1 << 6; //  denormals-are-zero     (0x0040)

/// The raw MXCSR control/status word on the **calling thread** (SSE FP control). Bit 15
/// is FTZ, bit 6 is DAZ; the IEEE default (and the headless reference environment) has
/// both OFF. Must be read on the very thread that runs `step` — the flags are per-thread
/// — which is why the wrapper is an instance method (Step 3 moves stepping to a worker
/// thread → re-check there).
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
fn read_mxcsr() -> u32 {
    let mut csr: u32 = 0;
    // SAFETY: `stmxcsr` stores the 32-bit MXCSR into the 4-byte slot `csr` points at;
    // the pointer is valid, aligned, and exclusively borrowed for the store.
    unsafe {
        core::arch::asm!(
            "stmxcsr [{ptr}]",
            ptr = in(reg) core::ptr::addr_of_mut!(csr),
            options(nostack, preserves_flags),
        );
    }
    csr
}

/// Non-x86 fallback: MXCSR does not exist (FTZ/DAZ live in aarch64's FPCR). The Phase-8
/// smoke target is Windows/x86_64, so report a clean control word rather than block the
/// build; revisit if a non-x86 port is ever gated on FP-env parity.
#[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
fn read_mxcsr() -> u32 {
    0
}

/// True iff FTZ **and** DAZ are both OFF — the IEEE / headless default the cross-boundary
/// parity guarantee relies on.
fn fp_flags_clean(mxcsr: u32) -> bool {
    (mxcsr & (MXCSR_FTZ | MXCSR_DAZ)) == 0
}

// ---------------------------------------------------------------------------
// The GDExtension surface.
// ---------------------------------------------------------------------------

struct GodotBridgeExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GodotBridgeExtension {}

/// The Godot-facing simulation session (registered Godot class name `SimSession`).
/// Instantiated from GDScript with `SimSession.new()`, then `build(scenario_id)`;
/// thereafter `step()` / `step_n(k)` advance it and `stock_amount` / `snapshot_json`
/// read it back. It owns nothing gdext-specific beyond the wrapper — all simulation
/// state lives in the frozen [`CoreSession`].
#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct SimSession {
    inner: Option<CoreSession>,
    base: Base<RefCounted>,
}

#[godot_api]
impl SimSession {
    /// Construct the owned session for a fixed-palette scenario id. Returns `false` (and
    /// logs) on an unknown id or a build error; `true` on success. Idempotently replaces
    /// any prior session.
    #[func]
    fn build(&mut self, scenario_id: GString) -> bool {
        match build_session(&scenario_id.to_string()) {
            Ok(session) => {
                self.inner = Some(session);
                true
            }
            Err(err) => {
                godot_error!("SimSession.build failed: {err:?}");
                false
            }
        }
    }

    /// Advance one natural unit (one `step_report` for the single-rate `cabin_gas`).
    /// Returns `false` (and logs) if called before [`build`](Self::build) or on a
    /// conservation/arbitration error.
    #[func]
    fn step(&mut self) -> bool {
        match self.inner.as_mut() {
            Some(session) => match session.step() {
                Ok(()) => true,
                Err(err) => {
                    godot_error!("SimSession.step failed: {err:?}");
                    false
                }
            },
            None => {
                godot_error!("SimSession.step called before build()");
                false
            }
        }
    }

    /// Advance `k` natural units (fast-forward). Rejects negative `k`.
    #[func]
    fn step_n(&mut self, k: i64) -> bool {
        if k < 0 {
            godot_error!("SimSession.step_n negative k={k}");
            return false;
        }
        match self.inner.as_mut() {
            Some(session) => match session.step_n(k as u64) {
                Ok(()) => true,
                Err(err) => {
                    godot_error!("SimSession.step_n failed: {err:?}");
                    false
                }
            },
            None => {
                godot_error!("SimSession.step_n called before build()");
                false
            }
        }
    }

    /// The current integer step count `n` (steps taken single-rate; master days
    /// two-rate). `-1` before [`build`](Self::build).
    #[func]
    fn step_count(&self) -> i64 {
        self.inner.as_ref().map(|s| s.n() as i64).unwrap_or(-1)
    }

    /// The current amount of one stock by id, for a live Label readout. `NaN` before
    /// [`build`](Self::build) or for an unknown id (a distinctly non-numeric sentinel).
    #[func]
    fn stock_amount(&self, id: GString) -> f64 {
        match self.inner.as_ref() {
            Some(session) => session
                .state()
                .stocks
                .get(&id.to_string())
                .map(|stock| stock.amount)
                .unwrap_or(f64::NAN),
            None => f64::NAN,
        }
    }

    /// The current `State` as `sim_io` hex-float JSON — the **Rust-side** codec
    /// ([`from_engine`] → `to_json`), so the cross-boundary parity smoke never leaves the
    /// golden discipline. Empty string before [`build`](Self::build).
    #[func]
    fn snapshot_json(&self) -> GString {
        match self.inner.as_ref() {
            Some(session) => GString::from(from_engine(session.state()).to_json().as_str()),
            None => GString::from(""),
        }
    }

    /// Total flows scaled by the Euler backstop so far (a golden run asserts `0`).
    #[func]
    fn total_rationed(&self) -> i64 {
        self.inner.as_ref().map(|s| s.total_rationed() as i64).unwrap_or(0)
    }

    /// The raw MXCSR control word on **this** (the stepping) thread. Diagnostic;
    /// [`fp_clean`](Self::fp_clean) is the assertion the smoke uses.
    #[func]
    fn mxcsr(&self) -> i64 {
        read_mxcsr() as i64
    }

    /// True iff FTZ and DAZ are both OFF on the calling thread — the IEEE / headless
    /// default the cross-boundary parity relies on. Read this on the same thread that
    /// calls [`step`](Self::step).
    #[func]
    fn fp_clean(&self) -> bool {
        fp_flags_clean(read_mxcsr())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_session_knows_cabin_gas_and_rejects_unknown() {
        assert!(build_session("cabin_gas").is_ok());
        // `CoreSession` isn't `Debug`, so match the `Err` without `unwrap_err`.
        match build_session("no_such_scenario") {
            Err(SimError::Validation(_)) => {}
            Err(other) => panic!("expected Validation error, got {other:?}"),
            Ok(_) => panic!("unknown scenario id must not build"),
        }
    }

    /// The session built through the bridge steps the frozen `cabin_gas` horizon with
    /// `rationed == 0` / no events — the same Tier-0 payload the emit example asserts.
    #[test]
    fn cabin_gas_session_steps_well_fed() {
        let mut session = build_session("cabin_gas").unwrap();
        session
            .step_n(station::scenario::CABIN_GAS_STEPS)
            .unwrap();
        assert_eq!(session.n(), station::scenario::CABIN_GAS_STEPS);
        assert_eq!(session.total_rationed(), 0);
        assert!(session.events().is_empty());
        // The snapshot codec is reachable and non-empty (byte-parity vs the golden is
        // the Python smoke's job — this only proves the wrapper path is wired).
        assert!(from_engine(session.state()).to_json().contains("cabin_o2"));
    }

    #[test]
    fn fp_flags_clean_decodes_ftz_and_daz() {
        assert!(fp_flags_clean(0));
        assert!(fp_flags_clean(0x1F80)); // default masks set, FTZ/DAZ clear
        assert!(!fp_flags_clean(MXCSR_FTZ));
        assert!(!fp_flags_clean(MXCSR_DAZ));
        assert!(!fp_flags_clean(MXCSR_FTZ | MXCSR_DAZ));
    }

    /// The headless Rust test thread is itself IEEE-default (FTZ/DAZ off) — the baseline
    /// the Godot-thread `fp_clean()` smoke is compared against.
    #[test]
    fn headless_thread_fp_env_is_clean() {
        assert!(fp_flags_clean(read_mxcsr()));
    }
}
