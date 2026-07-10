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
//! # The display projection (P8.2)
//!
//! **[`SimSession::observation_json`]** returns the multi-domain dashboard read — stocks
//! grouped per-domain plus the derived readouts (node temperature, battery SOC, per-quantity
//! totals, conservation residual, `events`, `rationed`), all computed Rust-side in
//! [`station::display`]. It is human-facing and **zero-parity** (plain decimal floats), kept
//! strictly separate from the bit-exact hex-float [`SimSession::snapshot_json`] parity path.
//! Step 2 also grows the fixed palette from `cabin_gas` to `{cabin_gas, station}` — the
//! Power → Thermal `station` is the entry with a real temperature and battery SOC to show.
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

use domains::crew::FECAL_WASTE;

use simcore::error::SimError;
use simcore::integrator::EulerIntegrator;
use simcore::snapshot::from_engine;

use station::display::{project, BatteryReadout, DisplayContext, ThermalReadout};

/// The frozen owned-state session (Phase-8 Step 0). Aliased so the *Godot* class below
/// can also be called `SimSession` (its registered Godot name) without shadowing.
use station::session::SimSession as CoreSession;

/// P8.3 — the off-render-thread time controller (play / pause / step / fast-forward).
mod time_control;

// ---------------------------------------------------------------------------
// Free functions (no gdext) — the testable core. `cargo test` exercises these
// without a Godot runtime; the `#[func]` methods are thin wrappers.
// ---------------------------------------------------------------------------

/// Build the owned session **and its display context** for a fixed-palette scenario id
/// (confirmed decision #1: "build systems" = a fixed, code-defined palette; registry
/// construction stays in Rust). Step 1 shipped the one Tier-1 tripwire (`cabin_gas`); Step 2
/// (P8.2) adds `station` (Power → Thermal) — the one palette entry with a real temperature
/// and battery SOC to show. The [`DisplayContext`] carries the per-scenario constants the
/// display readouts need but `State` lacks (thermal params, battery reference, the declared
/// shared-stock ids) — see [`station::display`].
pub(crate) fn build_session(
    scenario_id: &str,
) -> Result<(CoreSession, DisplayContext), SimError> {
    match scenario_id {
        "cabin_gas" => build_cabin_gas(),
        "station" => build_station(),
        "greenhouse" => build_greenhouse_session(),
        "sealed" => build_sealed_session(),
        other => Err(SimError::Validation(format!(
            "godot_bridge: unknown scenario id {other:?} (palette: \"cabin_gas\", \"station\", \
             \"greenhouse\", \"sealed\")"
        ))),
    }
}

/// The coupled crew ↔ ECLSS `CABIN_GAS_SCENARIO` as a single-rate session — mirrors the
/// [`station::run_station`] setup (and `tests/session_parity.rs`) exactly. No thermal node
/// or battery (both readouts project to `None`); the shared stocks are the three cabin-air
/// pools the crew breathes and ECLSS regulates (a construction-time fact of the assembly).
fn build_cabin_gas() -> Result<(CoreSession, DisplayContext), SimError> {
    let crew = domains::params::crew();
    let eclss = domains::params::eclss();
    let scenario = station::scenario::CABIN_GAS_SCENARIO;
    let (state, registry) = station::cabin::build_cabin(&crew, &eclss, &scenario)?;
    let resolver = station::cabin::cabin_resolver(&scenario)?;
    let session = CoreSession::single_rate(
        EulerIntegrator::new(registry),
        state,
        resolver,
        scenario.dt_seconds,
    );
    let ctx = DisplayContext {
        thermal: None,
        battery: None,
        shared_stock_ids: vec![
            domains::eclss::CABIN_O2.to_string(),
            domains::eclss::CABIN_CO2.to_string(),
            domains::eclss::CABIN_H2O.to_string(),
        ],
    };
    Ok((session, ctx))
}

/// The coupled Power → Thermal `HEAT_CLOSURE_SCENARIO` as a single-rate session — mirrors
/// the [`station::system::build_station`] setup (and `examples/emit_station.rs`) exactly.
/// This is the palette entry with a real node temperature and battery SOC: the display
/// context carries the thermal params (`T = T_space + Q/C`) and the initial battery charge
/// (the SOC reference), and highlights `thermal.node` — the stock Power dissipates into and
/// Thermal radiates from (cross-domain by construction, the Step-1 seam).
fn build_station() -> Result<(CoreSession, DisplayContext), SimError> {
    let charge = domains::params::charge();
    let thermal = domains::params::thermal();
    let scenario = station::scenario::HEAT_CLOSURE_SCENARIO;
    let (state, registry) = station::system::build_station(&charge, &thermal, &scenario, None)?;
    let resolver = station::system::station_resolver(&charge, &scenario)?;
    let session = CoreSession::single_rate(
        EulerIntegrator::new(registry),
        state,
        resolver,
        scenario.power.dt_seconds,
    );
    let ctx = DisplayContext {
        thermal: Some(ThermalReadout {
            node_id: domains::thermal::NODE.to_string(),
            heat_capacity: thermal.heat_capacity,
            space_temperature: thermal.space_temperature,
        }),
        battery: Some(BatteryReadout {
            battery_id: domains::power::BATTERY.to_string(),
            initial_charge: scenario.power.battery0,
        }),
        shared_stock_ids: vec![domains::thermal::NODE.to_string()],
    };
    Ok((session, ctx))
}

/// The biosphere ↔ cabin `greenhouse` as a **two-rate** session (P8.3) — the palette's
/// first two-rate entry, and the reason time controls run off the render thread: each
/// [`CoreSession::step`] is one **master day** = one slow biosphere step + `steps_per_day`
/// (1440) fast cabin sub-steps, so fast-forwarding many days is real compute the UI must
/// not block on. Mirrors [`station::greenhouse::run_greenhouse`]'s setup (and
/// `tests/session_parity.rs`) with `reset = None`. The shared stocks are the biosphere
/// carbon/O₂ pools the cabin flows are re-pointed at (one cabin air stock; a construction-
/// time fact of the reversed greenhouse seam). No thermal node or battery in this assembly,
/// so both scalar readouts project to `None`.
fn build_greenhouse_session() -> Result<(CoreSession, DisplayContext), SimError> {
    let crew = domains::params::crew();
    let eclss = domains::params::eclss();
    let scenario = station::scenario::greenhouse_scenario();
    let (state, bio_registry, cabin_registry) =
        station::greenhouse::build_greenhouse(&crew, &eclss, &scenario, true, FECAL_WASTE)?;
    let session = CoreSession::two_rate(
        EulerIntegrator::new(bio_registry),
        EulerIntegrator::new(cabin_registry),
        state,
        station::greenhouse::greenhouse_bio_resolver(&scenario)?,
        station::greenhouse::greenhouse_cabin_resolver(&scenario)?,
        scenario.steps_per_day,
        scenario.bio_dt,
        scenario.cabin_dt,
        None,
    )?;
    let ctx = DisplayContext {
        thermal: None,
        battery: None,
        shared_stock_ids: vec![
            domains::biosphere::stocks::CARBON_POOL.to_string(),
            domains::biosphere::stocks::O2_POOL.to_string(),
        ],
    };
    Ok((session, ctx))
}

/// The full sealed station as a **two-rate** session (P8.3) — the multi-year, re-sown
/// scenario that *is* "fast-forward decades." Mirrors [`station::sealed::run_sealed`]'s
/// construction (and the sealed branch of `tests/session_parity.rs`): every Phase-6 seam over
/// one shared stock dict + two registries, with the real `sealed_reset_hook` re-sowing the
/// biosphere each season. It carries a real node temperature and battery SOC (Power → Thermal
/// is inside), and highlights the cross-domain shared stocks (`thermal.node` + the biosphere
/// carbon/O₂ pools the cabin breathes). This is the palette entry the off-render-thread
/// fast-forward exists for: each master day is 1440 fast sub-steps, and a decade is thousands
/// of master days.
fn build_sealed_session() -> Result<(CoreSession, DisplayContext), SimError> {
    let charge = domains::params::charge();
    let thermal = domains::params::thermal();
    let crew = domains::params::crew();
    let eclss = domains::params::eclss();
    let recovery = station::params::water_recovery();
    let lamp = station::params::lamp();
    let harvest = station::params::harvest();
    let scenario = station::scenario::sealed_station_scenario();
    let (state, bio_registry, fast_registry) = station::sealed::build_sealed_station(
        &charge, &thermal, &crew, &eclss, &recovery, &lamp, &harvest, &scenario, false, false,
    )?;
    let session = CoreSession::two_rate(
        EulerIntegrator::new(bio_registry),
        EulerIntegrator::new(fast_registry),
        state,
        station::sealed::sealed_bio_resolver(&lamp, &scenario)?,
        station::sealed::sealed_fast_resolver(&charge, &scenario)?,
        scenario.steps_per_day,
        scenario.bio_dt,
        scenario.cabin_dt,
        Some(station::sealed::sealed_reset_hook(&scenario)),
    )?;
    let ctx = DisplayContext {
        thermal: Some(ThermalReadout {
            node_id: domains::thermal::NODE.to_string(),
            heat_capacity: thermal.heat_capacity,
            space_temperature: thermal.space_temperature,
        }),
        battery: Some(BatteryReadout {
            battery_id: domains::power::BATTERY.to_string(),
            initial_charge: scenario.battery0,
        }),
        shared_stock_ids: vec![
            domains::thermal::NODE.to_string(),
            domains::biosphere::stocks::CARBON_POOL.to_string(),
            domains::biosphere::stocks::O2_POOL.to_string(),
        ],
    };
    Ok((session, ctx))
}

const MXCSR_FTZ: u32 = 1 << 15; // flush-to-zero          (0x8000)
const MXCSR_DAZ: u32 = 1 << 6; //  denormals-are-zero     (0x0040)

/// The raw MXCSR control/status word on the **calling thread** (SSE FP control). Bit 15
/// is FTZ, bit 6 is DAZ; the IEEE default (and the headless reference environment) has
/// both OFF. Must be read on the very thread that runs `step` — the flags are per-thread
/// — which is why the wrapper is an instance method (Step 3 moves stepping to a worker
/// thread → re-check there).
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) fn read_mxcsr() -> u32 {
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
pub(crate) fn read_mxcsr() -> u32 {
    0
}

/// True iff FTZ **and** DAZ are both OFF — the IEEE / headless default the cross-boundary
/// parity guarantee relies on.
pub(crate) fn fp_flags_clean(mxcsr: u32) -> bool {
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
    /// Per-scenario display constants (thermal params, battery reference, shared-stock
    /// ids) built alongside `inner` — the display readouts need what `State` lacks.
    display: Option<DisplayContext>,
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
            Ok((session, display)) => {
                self.inner = Some(session);
                self.display = Some(display);
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

    /// The **display projection** as plain-float JSON (P8.2) — the multi-domain dashboard
    /// read: stocks grouped per-domain, per-quantity totals, the declared shared-stock ids,
    /// and the derived scalars (node temperature, battery SOC % of initial, last-step
    /// conservation residual, `events`, `rationed`). Every number is computed Rust-side
    /// ([`project`] → [`station::display::DisplayProjection::to_json`]); the game only
    /// renders it. Unlike [`snapshot_json`](Self::snapshot_json) this is a human-facing,
    /// zero-parity readout, so it uses normal decimal formatting — the bit-exact snapshot
    /// path stays on the hex-float codec. Empty string before [`build`](Self::build).
    #[func]
    fn observation_json(&self) -> GString {
        match (self.inner.as_ref(), self.display.as_ref()) {
            (Some(session), Some(ctx)) => {
                let proj = project(
                    session.state(),
                    ctx,
                    session.total_rationed(),
                    session.events().len(),
                    session.max_residual(),
                );
                GString::from(proj.to_json().as_str())
            }
            _ => GString::from(""),
        }
    }

    /// The **flow-level inspection** as plain-float JSON (P8.4) — every flow with its
    /// evaluated legs at the current state, for the "select a stock, see the contributing
    /// flows and their legs" panel: `{"n":..,"flows":[{"id":..,"legs":[{"stock":..,
    /// "amount":..}]}]}`. Computed Rust-side ([`station::inspection`]); the game only renders
    /// it. **Single-rate palette entries only** (`cabin_gas`, `station`): a two-rate session
    /// (`greenhouse` / `sealed`) returns `""` — inspecting its fast registry alone would hide
    /// the biosphere flows (see [`station::inspection`]), so it is deferred exactly as P8.2
    /// deferred the two-rate scalar readouts. Empty string before [`build`](Self::build) or on
    /// an evaluation error (logged).
    #[func]
    fn flow_inspection_json(&self) -> GString {
        match self.inner.as_ref() {
            Some(session) => match session.inspect_flows() {
                Ok(Some(insp)) => GString::from(insp.to_json().as_str()),
                Ok(None) => GString::from(""), // two-rate: deferred, not an error
                Err(err) => {
                    godot_error!("SimSession.flow_inspection_json failed: {err:?}");
                    GString::from("")
                }
            },
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
    fn build_session_knows_the_palette_and_rejects_unknown() {
        assert!(build_session("cabin_gas").is_ok());
        assert!(build_session("station").is_ok());
        assert!(build_session("greenhouse").is_ok());
        assert!(build_session("sealed").is_ok());
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
        let (mut session, _ctx) = build_session("cabin_gas").unwrap();
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

    /// The P8.2 display projection is reachable through the bridge and carries the derived
    /// readouts. `cabin_gas` (no node, no battery) reports both scalars `null` and
    /// highlights the three cabin-air pools; `station` (Power → Thermal) reports a real
    /// temperature and SOC and highlights `thermal.node`.
    #[test]
    fn observation_projection_carries_the_derived_readouts() {
        let (mut cabin, cabin_ctx) = build_session("cabin_gas").unwrap();
        cabin.step_n(50).unwrap();
        let json = project(
            cabin.state(),
            &cabin_ctx,
            cabin.total_rationed(),
            cabin.events().len(),
            cabin.max_residual(),
        )
        .to_json();
        assert!(json.contains("\"temperature_k\":null"));
        assert!(json.contains("\"soc_percent_of_initial\":null"));
        assert!(json.contains("eclss.cabin_o2")); // grouped + highlighted
        assert!(json.contains("\"eclss\":["));

        let (mut station, station_ctx) = build_session("station").unwrap();
        station.step_n(station::scenario::HEAT_CLOSURE_DAYS * 24).unwrap();
        let proj = project(
            station.state(),
            &station_ctx,
            station.total_rationed(),
            station.events().len(),
            station.max_residual(),
        );
        // Node warmed to a physical equilibrium (hundreds of K); SOC returns near 100%
        // of initial after whole balanced days.
        let t = proj.temperature_k.expect("station has a node temperature");
        assert!((100.0..400.0).contains(&t), "T_eq out of range: {t}");
        let soc = proj.soc_percent_of_initial.expect("station has a battery SOC");
        assert!((80.0..120.0).contains(&soc), "SOC out of range: {soc}");
        assert!(proj.to_json().contains("\"shared_stock_ids\":[\"thermal.node\"]"));
    }

    /// The two-rate `sealed` arm (the "decades" scenario) builds and steps a few master days
    /// well-fed with the real re-sow hook wired in — cheap coverage of the palette arm without
    /// the multi-year cost (the full-horizon parity lives in `tests/session_parity.rs`).
    #[test]
    fn sealed_session_steps_well_fed() {
        let (mut session, _ctx) = build_session("sealed").unwrap();
        session.step_n(3).unwrap();
        assert_eq!(session.n(), 3, "three master days");
        assert_eq!(session.total_rationed(), 0);
        assert!(session.events().is_empty());
    }

    /// The P8.4 flow inspection is reachable through the bridge for single-rate palette
    /// entries and carries the real station flows + their legs; a two-rate entry returns
    /// `None` (deferred).
    #[test]
    fn flow_inspection_single_rate_carries_flows_two_rate_is_none() {
        let (mut station, _ctx) = build_session("station").unwrap();
        station.step_n(5).unwrap();
        let insp = station
            .inspect_flows()
            .unwrap()
            .expect("station is single-rate");
        let ids: Vec<&str> = insp.flows.iter().map(|f| f.id.as_str()).collect();
        // The Power → Thermal station registry: charge + load draw + radiator (HeatInput
        // dropped — Power's dissipation is the input, the Step-1 seam).
        assert!(ids.contains(&"power.solar_charge"), "flows: {ids:?}");
        assert!(ids.contains(&"power.load_draw"), "flows: {ids:?}");
        assert!(ids.contains(&"thermal.radiator_reject"), "flows: {ids:?}");
        // The radiator moves heat off `thermal.node` (a contributing flow for the node).
        let touching = insp.flows_touching(domains::thermal::NODE);
        assert!(
            touching.iter().any(|(id, _)| *id == "thermal.radiator_reject"),
            "radiator should touch the node: {touching:?}"
        );
        let json = insp.to_json();
        assert!(json.starts_with("{\"n\":") && json.contains("\"flows\":["));

        // Two-rate greenhouse → None (deferred, not an error).
        let (greenhouse, _ctx) = build_session("greenhouse").unwrap();
        assert!(greenhouse.inspect_flows().unwrap().is_none());
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
