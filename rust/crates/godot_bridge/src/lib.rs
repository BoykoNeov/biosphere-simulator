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

use std::collections::BTreeMap;
use std::path::Path;

use godot::prelude::*;

use simcore::error::SimError;
use simcore::integrator::EulerIntegrator;
use simcore::snapshot::from_engine;

use station::builder::{assemble, BuildContext, Component};
use station::display::{project, BatteryReadout, DisplayContext, ThermalReadout};
use station::perturbations::{
    with_brownout, with_crew_load_spike, with_lighting_failure, with_radiator_failure,
    with_station_leak,
};

/// The frozen owned-state session (Phase-8 Step 0). Aliased so the *Godot* class below
/// can also be called `SimSession` (its registered Godot name) without shadowing.
use station::session::SimSession as CoreSession;

/// P8.3 — the off-render-thread time controller (play / pause / step / fast-forward).
mod time_control;

/// P8.7 — the versioned save-record wrapper (recipe + embedded v3 state snapshot).
mod save;

use save::{parse_save_record, save_record_json, Recipe};

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
    // The named-scenario dispatch lives gdext-free in `station::palette` so the headless CLI
    // (`station`'s `sim` bin) and this Godot cdylib build from the **same** shared builder —
    // the by-construction "the exact same simulation runs headless" guarantee (Phase-8 Step 8).
    // This wrapper adds nothing but the Godot plumbing.
    station::palette::build_scenario(scenario_id)
}

/// Build a session from a **fixed-palette composition** (Phase-8 P8.6) — the player picks a
/// set of parts (`"power_plant"`, `"radiator"`, `"self_discharge"`) rather than a whole
/// pre-built scenario. Delegates to [`station::builder::assemble`] (each part a thin delegate
/// to the frozen domain constructors, wired by shared stock id), so a `{power_plant, radiator}`
/// composition is bit-identical to the frozen `build_station` (`tests/builder_parity.rs`) — the
/// builder is a pure refactor, no new science. The [`DisplayContext`] is derived from which
/// parts are present: a `radiator` contributes the node temperature readout (and highlights
/// `thermal.node`), a `power_plant` the battery SOC. Single-rate only (the whole ENERGY palette
/// runs at the Power dt); two-rate composition is deferred (P8.6 scope).
pub(crate) fn build_composed_session(
    component_ids: &[String],
) -> Result<(CoreSession, DisplayContext), SimError> {
    let components: Vec<Component> = component_ids
        .iter()
        .map(|id| {
            Component::from_id(id).ok_or_else(|| {
                SimError::Validation(format!(
                    "godot_bridge: unknown component id {id:?} (palette: \"power_plant\", \
                     \"radiator\", \"self_discharge\")"
                ))
            })
        })
        .collect::<Result<_, _>>()?;
    let scenario = station::scenario::HEAT_CLOSURE_SCENARIO;
    let ctx = BuildContext {
        charge: domains::params::charge(),
        thermal: domains::params::thermal(),
        self_discharge: domains::params::self_discharge(),
        scenario,
    };
    let (state, registry, resolver) = assemble(&components, &ctx)?;
    let session = CoreSession::single_rate(
        EulerIntegrator::new(registry),
        state,
        resolver,
        scenario.power.dt_seconds,
    );

    // The display readouts reflect what was actually composed: a node temperature only with a
    // radiator (the stock that carries heat), a battery SOC only with a power plant.
    let has_power = components.contains(&Component::PowerPlant);
    let has_radiator = components.contains(&Component::Radiator);
    let display = DisplayContext {
        thermal: has_radiator.then(|| ThermalReadout {
            node_id: domains::thermal::NODE.to_string(),
            heat_capacity: ctx.thermal.heat_capacity,
            space_temperature: ctx.thermal.space_temperature,
        }),
        battery: has_power.then(|| BatteryReadout {
            battery_id: domains::power::BATTERY.to_string(),
            initial_charge: scenario.power.battery0,
        }),
        // `thermal.node` is the cross-domain shared stock (Power dissipates in, radiator sheds
        // out) — highlighted only when a radiator makes it exist.
        shared_stock_ids: if has_radiator {
            vec![domains::thermal::NODE.to_string()]
        } else {
            vec![]
        },
    };
    Ok((session, display))
}

/// Build a session from a **declarative scenario file** (Phase-9 P8/Step-5 — "Godot loads a
/// file at runtime"): the "author, not program" payoff. The file is parsed + interpreted by
/// the frozen [`authoring`] boundary ([`authoring::load_scenario`]) — the *same* code the
/// headless `emit_authored` example runs — and this wraps the resulting graph in the
/// single-rate [`CoreSession`]. The only bridge-specific part is that trivial
/// `BuiltScenario → SimSession` wrap; the graph build (schema, wiring, balance-by-construction,
/// template boundary-eval) all stays in `authoring`, so a file-loaded session is bit-identical
/// to the headless run (proven intra-process by the faithfulness cargo test, and across the FFI
/// boundary by `from_file_smoke.gd`).
///
/// Returns the session, an (empty) [`DisplayContext`] — authored files declare no display hints
/// yet, so both derived scalars project `null` and no stock is highlighted (zero parity concern,
/// display-only) — and the authored `steps` horizon the file itself declares (so the caller can
/// step exactly what the headless run does). `overrides` instantiate a template's `parameters`
/// (Step-3 `param('crew_count')` knobs); pass an empty map for the common no-override case.
///
/// **Single-rate / Euler only** (the [`SimSession::single_rate`] shape the [`authoring::run`]
/// harness itself is scoped to): a file requesting `rk4` (no single-rate rk4 session exists) is a
/// loud [`SimError::Validation`], deferred exactly as two-rate authoring is. Any authoring parse/
/// interpret failure (unknown flow type, bad wiring, unbalanced authored stoichiometry, a missing
/// file) is mapped to a [`SimError::Validation`] the UI renders as `false`.
pub(crate) fn build_session_from_file(
    path: &Path,
    overrides: &BTreeMap<String, f64>,
) -> Result<(CoreSession, DisplayContext, u64), SimError> {
    let built = authoring::load_scenario(path, overrides)
        .map_err(|e| SimError::Validation(format!("authoring: {}", e.message)))?;
    if built.integrator != "euler" {
        return Err(SimError::Validation(format!(
            "godot_bridge: file-loaded scenario {:?} requested integrator {:?}, but a \
             file-loaded session is single-rate Euler only (rk4 file scenarios are deferred, \
             exactly as two-rate authoring is)",
            built.name, built.integrator
        )));
    }
    let steps = built.steps;
    let session = CoreSession::single_rate(
        EulerIntegrator::new(built.registry),
        built.state,
        built.resolver,
        built.dt,
    );
    // Authored files carry no display hints in the schema (no thermal node / battery / declared
    // shared stocks), so the display context is empty. The observation projection still renders
    // (per-domain stock groups + per-quantity totals); a Step-6 refinement could let a file
    // declare hints. This empty-context path is otherwise unexercised by the palette (all four
    // entries declare ≥1 shared stock), so a cargo test asserts `observation_json` is well-formed.
    let display = DisplayContext {
        thermal: None,
        battery: None,
        shared_stock_ids: vec![],
    };
    Ok((session, display, steps))
}

/// Rebuild a session from a save's [`Recipe`] (Phase-8 P8.7) — the dispatch save/load
/// shares with the build FFI: a [`Recipe::Named`] scenario goes through
/// [`build_session`], a [`Recipe::Composed`] palette set through
/// [`build_composed_session`]. The registry is reconstructed deterministically from the
/// recipe; the caller then restores the saved `State` via
/// [`station::session::SimSession::load_state`].
pub(crate) fn build_from_recipe(
    recipe: &Recipe,
) -> Result<(CoreSession, DisplayContext), SimError> {
    match recipe {
        Recipe::Named(id) => build_session(id),
        Recipe::Composed(ids) => build_composed_session(ids),
    }
}

/// Build a **perturbed** session for a fixed-palette scenario (Phase-8 P8.5) — the
/// interactive "perturb systems" primitive (work item #4). **Build-time windowed:** the
/// perturbation is a pure function of the integer step `n` over `[start, end)`, so it is
/// deterministic / parity-clean and needs no live-mutation surface on the session (a live
/// "trigger now" is expressible as a window opening at the current `n`, but is deferred —
/// build-time config satisfies the fixed-palette posture). Energy perturbations (`brownout`,
/// `radiator_failure`) apply to the single-rate `station`; matter perturbations
/// (`carbon_leak`, `o2_leak`, `crew_spike`, `lighting_failure`) to the two-rate `sealed`.
/// `magnitude` is the one scalar knob (brownout/crew factor, radiator health, leak rate;
/// ignored by `lighting_failure`). The cross-domain cascade emerges for free — the composers
/// are `station::perturbations`, computing no game logic here.
pub(crate) fn build_perturbed_session(
    scenario_id: &str,
    kind: &str,
    start: u64,
    end: u64,
    magnitude: f64,
) -> Result<(CoreSession, DisplayContext), SimError> {
    match scenario_id {
        "station" => build_station_perturbed(kind, start, end, magnitude),
        "sealed" => build_sealed_perturbed(kind, start, end, magnitude),
        other => Err(SimError::Validation(format!(
            "godot_bridge: perturbations apply to 'station' (energy) or 'sealed' (matter), \
             not {other:?}"
        ))),
    }
}

/// The single-rate `station` (Power → Thermal) under an energy perturbation. `brownout` dims
/// the solar forcing (deep ⇒ the battery empties and `LoadDraw` rations — the failure
/// cascade); `radiator_failure` throttles `RadiatorReject` (the node overheats).
fn build_station_perturbed(
    kind: &str,
    start: u64,
    end: u64,
    magnitude: f64,
) -> Result<(CoreSession, DisplayContext), SimError> {
    let charge = domains::params::charge();
    let thermal = domains::params::thermal();
    let scenario = station::scenario::HEAT_CLOSURE_SCENARIO;
    let (state, registry) = station::system::build_station(&charge, &thermal, &scenario, None)?;
    let resolver = station::system::station_resolver(&charge, &scenario)?;
    let (registry, resolver) = match kind {
        "brownout" => (registry, with_brownout(resolver, start, end, magnitude)?),
        "radiator_failure" => {
            with_radiator_failure(&state, registry, resolver, start, end, magnitude)?
        }
        other => {
            return Err(SimError::Validation(format!(
                "station perturbations are 'brownout' | 'radiator_failure', not {other:?}"
            )))
        }
    };
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

/// The two-rate `sealed` station under a matter perturbation — the leak (per-pool), crew
/// spike, or lighting failure, over one shared stock dict + two registries, with the real
/// re-sow hook. The station regulators erase the naive pool level, so the cascade surfaces as
/// regulator effort + sinks (P6.8). `carbon_leak` / `o2_leak` pass `magnitude` as the leak
/// rate `k_leak`; `crew_spike` as the intake factor; `lighting_failure` ignores it.
fn build_sealed_perturbed(
    kind: &str,
    start: u64,
    end: u64,
    magnitude: f64,
) -> Result<(CoreSession, DisplayContext), SimError> {
    let charge = domains::params::charge();
    let thermal = domains::params::thermal();
    let crew = domains::params::crew();
    let eclss = domains::params::eclss();
    let recovery = station::params::water_recovery();
    let lamp = station::params::lamp();
    let harvest = station::params::harvest();
    let scenario = station::scenario::sealed_station_scenario();
    let (state, bio_reg, fast_reg) = station::sealed::build_sealed_station(
        &charge, &thermal, &crew, &eclss, &recovery, &lamp, &harvest, &scenario, false, false,
    )?;
    let bio_res = station::sealed::sealed_bio_resolver(&lamp, &scenario)?;
    let fast_res = station::sealed::sealed_fast_resolver(&charge, &scenario)?;

    let (state, bio_reg, fast_reg, bio_res, fast_res) = match kind {
        "carbon_leak" => {
            let (s, b, f, fr) = with_station_leak(
                &state,
                bio_reg,
                fast_reg,
                fast_res,
                domains::biosphere::stocks::CARBON_POOL,
                magnitude,
                start,
                end,
            )?;
            (s, b, f, bio_res, fr)
        }
        "o2_leak" => {
            let (s, b, f, fr) = with_station_leak(
                &state,
                bio_reg,
                fast_reg,
                fast_res,
                domains::biosphere::stocks::O2_POOL,
                magnitude,
                start,
                end,
            )?;
            (s, b, f, bio_res, fr)
        }
        "crew_spike" => {
            let fr = with_crew_load_spike(fast_res, start, end, magnitude)?;
            (state, bio_reg, fast_reg, bio_res, fr)
        }
        "lighting_failure" => {
            let (br, fr) = with_lighting_failure(bio_res, fast_res, start, end)?;
            (state, bio_reg, fast_reg, br, fr)
        }
        other => {
            return Err(SimError::Validation(format!(
                "sealed perturbations are 'carbon_leak' | 'o2_leak' | 'crew_spike' | \
                 'lighting_failure', not {other:?}"
            )))
        }
    };
    let session = CoreSession::two_rate(
        EulerIntegrator::new(bio_reg),
        EulerIntegrator::new(fast_reg),
        state,
        bio_res,
        fast_res,
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
    /// How this session was built (P8.7) — set by `build` / `build_composed`, so `save`
    /// can record the recipe alongside the state. `None` for a perturbed session (not
    /// saveable) or before any build.
    recipe: Option<Recipe>,
    /// The horizon a **file-loaded** scenario declares (Phase-9 Step 5) — `steps` from the
    /// scenario file, so the caller can step exactly what the headless run does
    /// (`total_steps()`). `None` for palette / composed / perturbed sessions (their horizon
    /// is a known constant, not file-declared).
    authored_steps: Option<u64>,
    base: Base<RefCounted>,
}

/// Non-`#[func]` helpers, kept out of the `#[godot_api]` block. Shared by
/// `build_from_file` / `build_from_file_with`.
impl SimSession {
    fn build_from_file_impl(&mut self, path: &str, overrides: &BTreeMap<String, f64>) -> bool {
        match build_session_from_file(Path::new(path), overrides) {
            Ok((session, display, steps)) => {
                self.inner = Some(session);
                self.display = Some(display);
                self.authored_steps = Some(steps);
                // No fixed-palette recipe (its identity is the on-disk file) — not saveable, the
                // perturbed-session precedent. A `File` recipe is a deferred follow-on.
                self.recipe = None;
                true
            }
            Err(err) => {
                godot_error!("SimSession.build_from_file failed: {err:?}");
                false
            }
        }
    }
}

#[godot_api]
impl SimSession {
    /// Construct the owned session for a fixed-palette scenario id. Returns `false` (and
    /// logs) on an unknown id or a build error; `true` on success. Idempotently replaces
    /// any prior session.
    #[func]
    fn build(&mut self, scenario_id: GString) -> bool {
        let id = scenario_id.to_string();
        match build_session(&id) {
            Ok((session, display)) => {
                self.inner = Some(session);
                self.display = Some(display);
                self.recipe = Some(Recipe::Named(id));
                self.authored_steps = None;
                true
            }
            Err(err) => {
                godot_error!("SimSession.build failed: {err:?}");
                false
            }
        }
    }

    /// Construct a session from a **fixed-palette composition** (Phase-8 P8.6) — the player
    /// passes the chosen part ids (`"power_plant"`, `"radiator"`, `"self_discharge"`) and the
    /// bounded Rust builder assembles them (registry construction stays Rust-side). Returns
    /// `false` (and logs) on an unknown id, an empty/duplicate set, or a dependency violation
    /// (`self_discharge` without `power_plant`); `true` on success. Idempotently replaces any
    /// prior session. A `{power_plant, radiator}` composition reproduces the frozen `station`
    /// bit-for-bit — the builder is a pure refactor, not new science.
    #[func]
    fn build_composed(&mut self, component_ids: PackedStringArray) -> bool {
        let ids: Vec<String> = component_ids.to_vec().iter().map(|g| g.to_string()).collect();
        match build_composed_session(&ids) {
            Ok((session, display)) => {
                self.inner = Some(session);
                self.display = Some(display);
                self.recipe = Some(Recipe::Composed(ids));
                self.authored_steps = None;
                true
            }
            Err(err) => {
                godot_error!("SimSession.build_composed failed: {err:?}");
                false
            }
        }
    }

    /// Construct a **perturbed** session (Phase-8 P8.5) — the interactive "perturb systems"
    /// primitive. `scenario_id` is `"station"` (energy) or `"sealed"` (matter); `kind` is the
    /// perturbation (`brownout` / `radiator_failure` for station; `carbon_leak` / `o2_leak` /
    /// `crew_spike` / `lighting_failure` for sealed); `[start, end)` is the window in steps
    /// (single-rate) or master days (two-rate); `magnitude` the one scalar knob. Returns
    /// `false` (and logs) on a negative window, an unknown scenario/kind pairing, or a build
    /// error. Idempotently replaces any prior session. The cross-domain cascade then emerges
    /// as the session steps — no game-side domain logic.
    #[func]
    fn build_perturbed(
        &mut self,
        scenario_id: GString,
        kind: GString,
        start: i64,
        end: i64,
        magnitude: f64,
    ) -> bool {
        if start < 0 || end < 0 {
            godot_error!("SimSession.build_perturbed negative window start={start} end={end}");
            return false;
        }
        match build_perturbed_session(
            &scenario_id.to_string(),
            &kind.to_string(),
            start as u64,
            end as u64,
            magnitude,
        ) {
            Ok((session, display)) => {
                self.inner = Some(session);
                self.display = Some(display);
                // A perturbed session is not saveable (P8.7 scope): the perturbation is a
                // build-time window that has no place in the fixed-palette recipe. `save`
                // reports it unavailable rather than dropping the perturbation silently.
                self.recipe = None;
                self.authored_steps = None;
                true
            }
            Err(err) => {
                godot_error!("SimSession.build_perturbed failed: {err:?}");
                false
            }
        }
    }

    /// Construct a session from a **declarative scenario file** (Phase-9 Step 5 — "Godot
    /// loads a file at runtime"): the "author, not program" payoff. `path` is an **OS
    /// filesystem path** (from GDScript, `ProjectSettings.globalize_path(...)` or a
    /// `--`-passed absolute path — not a `res://` URI). The file is parsed + interpreted by
    /// the frozen [`authoring`] boundary and wrapped in the single-rate session; a modder /
    /// player can load new authored scenarios with no recompile. Returns `false` (and logs)
    /// on any authoring error (unknown flow type, bad wiring, unbalanced authored
    /// stoichiometry, a missing file, or a two-rate/`rk4` scenario — deferred). Idempotently
    /// replaces any prior session. Use [`total_steps`](Self::total_steps) to read the
    /// file-declared horizon, then [`step_n`](Self::step_n) it.
    ///
    /// A file-loaded session is **not saveable** via the P8.7 recipe wrapper (its identity is
    /// the file + overrides on disk, not a fixed-palette id) — [`save`](Self::save) reports it
    /// unavailable, exactly like a perturbed session. A `File` recipe is a deferred follow-on.
    #[func]
    fn build_from_file(&mut self, path: GString) -> bool {
        self.build_from_file_impl(&path.to_string(), &BTreeMap::new())
    }

    /// [`build_from_file`](Self::build_from_file) for a **template** — supply parallel arrays
    /// of parameter names + values (the Step-3 `parameters` overrides, e.g. `crew_count = 4.0`)
    /// as **typed scalars, no JSON parser** (the P8.5 typed-FFI ethos). The arrays must be the
    /// same length; a mismatch (or any authoring/build error) returns `false` and logs.
    #[func]
    fn build_from_file_with(
        &mut self,
        path: GString,
        override_names: PackedStringArray,
        override_values: PackedFloat64Array,
    ) -> bool {
        let names = override_names.to_vec();
        let values = override_values.to_vec();
        if names.len() != values.len() {
            godot_error!(
                "SimSession.build_from_file_with: override_names ({}) and override_values ({}) \
                 length mismatch",
                names.len(),
                values.len()
            );
            return false;
        }
        let overrides: BTreeMap<String, f64> = names
            .iter()
            .zip(values.iter())
            .map(|(n, v)| (n.to_string(), *v))
            .collect();
        self.build_from_file_impl(&path.to_string(), &overrides)
    }

    /// The horizon a **file-loaded** scenario declared (`steps` from the file), so the caller
    /// steps exactly what the headless `emit_authored` run does. `-1` for a palette / composed /
    /// perturbed session (their horizon is a known constant, not file-declared) or before build.
    #[func]
    fn total_steps(&self) -> i64 {
        self.authored_steps.map(|s| s as i64).unwrap_or(-1)
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

    /// **Save** the session (P8.7) — the save-record JSON (`save_version` + the build
    /// `recipe` + the current `State` as an embedded v3 snapshot). The game writes this to
    /// disk (`FileAccess`); [`load`](Self::load) restores it. Returns `""` (and logs) if no
    /// saveable session is built — before `build`, or for a **perturbed** session (not
    /// saveable, P8.7 scope: the perturbation window has no recipe). The state is carried
    /// through the bit-exact hex-float codec, so a save round-trips a resumable trajectory.
    #[func]
    fn save(&self) -> GString {
        match (self.recipe.as_ref(), self.inner.as_ref()) {
            (Some(recipe), Some(session)) => {
                GString::from(save_record_json(recipe, session.state()).as_str())
            }
            (None, Some(_)) => {
                godot_error!(
                    "SimSession.save: this session is not saveable (perturbed sessions have \
                     no fixed-palette recipe — P8.7 scope)"
                );
                GString::from("")
            }
            _ => {
                godot_error!("SimSession.save called before build()");
                GString::from("")
            }
        }
    }

    /// **Load** a save record (P8.7) — rebuild the session from the record's `recipe` (the
    /// fixed-palette scenario / composition) and restore the saved `State`, so stepping
    /// resumes bit-identically (the `(seed, key, n)` determinism corollary). Returns
    /// `false` (and logs) on a malformed record, an unknown `save_version`, an unknown
    /// recipe, or a stock-set mismatch. Idempotently replaces any prior session.
    #[func]
    fn load(&mut self, save_text: GString) -> bool {
        let (recipe, state) = match parse_save_record(&save_text.to_string()) {
            Ok(pair) => pair,
            Err(err) => {
                godot_error!("SimSession.load: bad save record: {err:?}");
                return false;
            }
        };
        let (mut session, display) = match build_from_recipe(&recipe) {
            Ok(pair) => pair,
            Err(err) => {
                godot_error!("SimSession.load: recipe rebuild failed: {err:?}");
                return false;
            }
        };
        if let Err(err) = session.load_state(state) {
            godot_error!("SimSession.load: state restore failed: {err:?}");
            return false;
        }
        self.inner = Some(session);
        self.display = Some(display);
        self.recipe = Some(recipe);
        self.authored_steps = None;
        true
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

    /// The **objectives** evaluation (P8.8) as plain JSON — whether a "survive to
    /// `target_step`" goal is met, and each health clause behind it (`reached_target`,
    /// `conserved`, `no_rationing`, `no_extinction`, `survived`). Pure predicates over the
    /// session diagnostics (`n`, `total_rationed`, `events`, `max_residual`) — no game-side
    /// domain logic ([`station::objectives`]). A perturbation that drives rationing or an
    /// extinction flips `survived`, so the same objective distinguishes a stable run from a
    /// failing one. Empty string before [`build`](Self::build); rejects a negative target.
    #[func]
    fn objectives_json(&self, target_step: i64) -> GString {
        let Some(session) = self.inner.as_ref() else {
            godot_error!("SimSession.objectives_json called before build()");
            return GString::from("");
        };
        if target_step < 0 {
            godot_error!("SimSession.objectives_json negative target_step={target_step}");
            return GString::from("");
        }
        let objective = station::objectives::Objective::survive(target_step as u64);
        let report = station::objectives::evaluate(
            &objective,
            session.n(),
            session.total_rationed(),
            session.events().len(),
            session.max_residual(),
        );
        GString::from(report.to_json().as_str())
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

    /// The P8.6 fixed-palette composition path: `{power_plant, radiator}` builds a single-rate
    /// session that steps well-fed and reaches a physical node temperature (the byte-identity
    /// with `build_station` is proven Rust-side in `station/tests/builder_parity.rs`; here we
    /// only prove the *bridge* wires the composition + its derived display context).
    #[test]
    fn composed_energy_station_builds_steps_and_carries_readouts() {
        let (mut session, ctx) =
            build_composed_session(&["power_plant".into(), "radiator".into()]).unwrap();
        session.step_n(station::scenario::HEAT_CLOSURE_DAYS * 24).unwrap();
        assert_eq!(session.total_rationed(), 0);
        assert!(session.events().is_empty());
        // The display context reflects the parts: a radiator ⇒ node temperature + highlight,
        // a power plant ⇒ battery SOC.
        let proj = project(
            session.state(),
            &ctx,
            session.total_rationed(),
            session.events().len(),
            session.max_residual(),
        );
        let t = proj.temperature_k.expect("radiator ⇒ node temperature");
        assert!((100.0..400.0).contains(&t), "T_eq out of range: {t}");
        assert!(proj.soc_percent_of_initial.is_some(), "power plant ⇒ battery SOC");
        assert!(proj.to_json().contains("thermal.node"));
    }

    /// Composition subsets shape the display context: a lone `power_plant` (dissipating to the
    /// boundary sink) has a battery SOC but no node temperature; a `power_plant + self_discharge`
    /// still single-rate, no radiator.
    #[test]
    fn composed_power_plant_alone_has_soc_but_no_temperature() {
        let (session, ctx) = build_composed_session(&["power_plant".into()]).unwrap();
        let proj = project(session.state(), &ctx, 0, 0, session.max_residual());
        assert!(proj.temperature_k.is_none(), "no radiator ⇒ no node temperature");
        assert!(proj.soc_percent_of_initial.is_some(), "power plant ⇒ SOC");

        // Adding the leak keeps it single-rate and buildable.
        assert!(build_composed_session(&["power_plant".into(), "self_discharge".into()]).is_ok());
    }

    /// The bridge surfaces the builder's validation as an `Err` (the UI renders it as `false`):
    /// an unknown component id, an empty set, and the `self_discharge`-without-`power_plant`
    /// dependency violation.
    #[test]
    fn composed_rejects_bad_compositions() {
        assert!(matches!(
            build_composed_session(&["no_such_part".into()]),
            Err(SimError::Validation(_))
        ));
        assert!(matches!(build_composed_session(&[]), Err(SimError::Validation(_))));
        assert!(matches!(
            build_composed_session(&["self_discharge".into()]),
            Err(SimError::Validation(_))
        ));
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

    /// The P8.5 perturbation primitive: a perturbed `station` (deep brownout) builds and, as
    /// it steps, drives `LoadDraw` into rationing (the failure cascade emerges) while still
    /// stepping without error (conservation holds — the Euler backstop rations as it conserves).
    #[test]
    fn build_perturbed_station_brownout_emerges_rationing() {
        let spd = station::scenario::HEAT_CLOSURE_SCENARIO.power.steps_per_day;
        // Deep blackout on days [2, 8): factor 0.0.
        let (mut session, _ctx) =
            build_perturbed_session("station", "brownout", 2 * spd, 8 * spd, 0.0).unwrap();
        session.step_n(12 * spd).unwrap();
        assert!(session.total_rationed() > 0, "the failure cascade should ration");
    }

    /// A perturbed `sealed` (carbon leak) builds with the augmented `LEAK_SINK` stock and steps
    /// well-fed (`rationed == 0` — `k_leak·dt < 1`), the leak-sink accumulating.
    #[test]
    fn build_perturbed_sealed_carbon_leak_builds_and_steps() {
        let (mut session, _ctx) =
            build_perturbed_session("sealed", "carbon_leak", 1, 5, 1.0e-3).unwrap();
        // The leak augments the state with a boundary sink mirroring CARBON_POOL's composition.
        assert!(session.state().stocks.contains_key("boundary.leak_sink"));
        session.step_n(3).unwrap();
        assert_eq!(session.total_rationed(), 0);
    }

    /// Perturbation validation: an unknown scenario/kind pairing is a Validation error (the
    /// UI surfaces it as `false`, never a silent wrong build).
    #[test]
    fn build_perturbed_rejects_bad_pairings() {
        // Wrong substrate: brownout is a station (energy) perturbation, not sealed.
        assert!(matches!(
            build_perturbed_session("sealed", "brownout", 1, 5, 0.0),
            Err(SimError::Validation(_))
        ));
        // Unknown kind.
        assert!(matches!(
            build_perturbed_session("station", "no_such_kind", 1, 5, 0.0),
            Err(SimError::Validation(_))
        ));
        // Unknown scenario (cabin_gas/greenhouse are not perturbable in the palette).
        assert!(matches!(
            build_perturbed_session("cabin_gas", "brownout", 1, 5, 0.0),
            Err(SimError::Validation(_))
        ));
    }

    /// The P8.7 save/load path through the bridge free functions: a session saved at
    /// step A (via `save_record_json`), reloaded (`parse_save_record` → `build_from_recipe`
    /// → `load_state`), and stepped B more, is bit-identical to a straight run of A+B. The
    /// station-level `tests/session_save_load.rs` proves the relation directly; this proves
    /// the *bridge composition* of recipe + record + rebuild is wired correctly.
    #[test]
    fn save_load_round_trip_through_the_bridge_resumes_bit_identical() {
        let (mut straight, _) = build_session("cabin_gas").unwrap();
        straight.step_n(300).unwrap();
        let straight_final = from_engine(straight.state()).to_json();

        let (mut saver, _) = build_session("cabin_gas").unwrap();
        saver.step_n(120).unwrap();
        let record = save_record_json(&Recipe::Named("cabin_gas".to_string()), saver.state());

        let (recipe, state) = parse_save_record(&record).unwrap();
        assert_eq!(recipe, Recipe::Named("cabin_gas".to_string()));
        let (mut resumed, _) = build_from_recipe(&recipe).unwrap();
        resumed.load_state(state).unwrap();
        resumed.step_n(180).unwrap();

        assert_eq!(
            from_engine(resumed.state()).to_json(),
            straight_final,
            "bridge save/load resume is bit-identical to a straight run"
        );
    }

    /// The **two-rate** bridge save/load path end-to-end: a `greenhouse` session saved at
    /// day 2 (via the bridge record) and resumed through `build_from_recipe` + `load_state`
    /// is bit-identical to a straight run — the pure-Rust `session_save_load.rs` proves the
    /// relation, this proves the bridge composition (`Recipe::Named` two-rate rebuild + the
    /// two-rate `load_state`) is wired. Cheap (a few master days).
    #[test]
    fn save_load_round_trip_two_rate_greenhouse_through_the_bridge() {
        let (mut straight, _) = build_session("greenhouse").unwrap();
        straight.step_n(3).unwrap();
        let straight_final = from_engine(straight.state()).to_json();

        let (mut saver, _) = build_session("greenhouse").unwrap();
        saver.step_n(2).unwrap();
        let record = save_record_json(&Recipe::Named("greenhouse".to_string()), saver.state());

        let (recipe, state) = parse_save_record(&record).unwrap();
        let (mut resumed, _) = build_from_recipe(&recipe).unwrap();
        resumed.load_state(state).unwrap();
        assert_eq!(resumed.n(), 2, "loaded two-rate state carries the master-day count");
        resumed.step_n(1).unwrap();

        assert_eq!(
            from_engine(resumed.state()).to_json(),
            straight_final,
            "two-rate bridge save/load resume is bit-identical (aux/phenology survives)"
        );
    }

    #[test]
    fn build_from_recipe_dispatches_named_and_composed() {
        assert!(build_from_recipe(&Recipe::Named("station".to_string())).is_ok());
        assert!(build_from_recipe(&Recipe::Composed(vec![
            "power_plant".to_string(),
            "radiator".to_string()
        ]))
        .is_ok());
        assert!(matches!(
            build_from_recipe(&Recipe::Named("no_such".to_string())),
            Err(SimError::Validation(_))
        ));
    }

    /// A composed recipe survives the full round-trip too (save records both palette paths).
    #[test]
    fn save_load_round_trip_composed_recipe() {
        let (mut saver, _) =
            build_composed_session(&["power_plant".into(), "radiator".into()]).unwrap();
        saver.step_n(24).unwrap();
        let record = save_record_json(
            &Recipe::Composed(vec!["power_plant".to_string(), "radiator".to_string()]),
            saver.state(),
        );
        let (recipe, state) = parse_save_record(&record).unwrap();
        let (mut resumed, _) = build_from_recipe(&recipe).unwrap();
        resumed.load_state(state).unwrap();
        assert_eq!(resumed.n(), 24, "resumed composed session carries n");
    }

    /// The scenario-file directory (repo `tests/authoring/scenarios`), from the crate root.
    fn scenarios_dir() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../tests/authoring/scenarios")
    }

    /// Phase-9 Step 5: a declarative scenario **file** builds a single-rate session that steps
    /// the file-declared horizon well-fed (the same Tier-0 payload the emit example asserts).
    #[test]
    fn build_session_from_file_loads_crew_and_steps_well_fed() {
        let path = scenarios_dir().join("crew_mission.yaml");
        let (mut session, _ctx, steps) =
            build_session_from_file(&path, &BTreeMap::new()).unwrap();
        assert_eq!(steps, 168, "crew_mission declares 168 steps");
        session.step_n(steps).unwrap();
        assert_eq!(session.n(), 168);
        assert_eq!(session.total_rationed(), 0);
        assert!(session.events().is_empty());
        assert!(from_engine(session.state()).to_json().contains("crew.food_store"));
    }

    /// **The drift guard.** `SimSession::single_rate.step()` and `authoring::run_scenario`'s
    /// Euler loop are *parallel* implementations (not a shared extract like Step-0's
    /// `advance_one_master_day`), so this exact hex-float equality is the only thing keeping
    /// the file-loaded session bit-identical to the headless run.
    #[test]
    fn build_session_from_file_is_bit_identical_to_run_scenario() {
        let path = scenarios_dir().join("crew_mission.yaml");
        let (mut session, _ctx, steps) =
            build_session_from_file(&path, &BTreeMap::new()).unwrap();
        session.step_n(steps).unwrap();
        let session_final = from_engine(session.state()).to_json();

        let built = authoring::load_scenario(&path, &BTreeMap::new()).unwrap();
        let result = authoring::run_scenario(built).unwrap();
        let headless_final = from_engine(&result.final_state).to_json();

        assert_eq!(
            session_final, headless_final,
            "the file-loaded SimSession must be bit-identical to authoring::run_scenario"
        );
    }

    /// A **template** file with a `crew_count` override: the knob bites (≈4× the food store —
    /// the Step-3 gate), and the empty-`shared_stock_ids` display path (unexercised by the
    /// palette, all of whose entries declare ≥1 shared stock) still projects a well-formed
    /// observation with both derived scalars `null`.
    #[test]
    fn build_session_from_file_template_override_bites_and_projects() {
        let path = scenarios_dir().join("crew_habitat_template.yaml");
        let (base, _c, _s) = build_session_from_file(&path, &BTreeMap::new()).unwrap();
        let base_food = base.state().stocks.get("crew.food_store").unwrap().amount;

        let overrides: BTreeMap<String, f64> =
            [("crew_count".to_string(), 4.0)].into_iter().collect();
        let (session, ctx, _s) = build_session_from_file(&path, &overrides).unwrap();
        let big_food = session.state().stocks.get("crew.food_store").unwrap().amount;
        assert!(
            (big_food - 4.0 * base_food).abs() < 1e-9 * base_food.max(1.0),
            "crew_count=4.0 should ~4× the food store: base={base_food} big={big_food}"
        );

        assert!(ctx.shared_stock_ids.is_empty());
        let proj = project(session.state(), &ctx, 0, 0, session.max_residual());
        let json = proj.to_json();
        assert!(json.contains("\"temperature_k\":null"));
        assert!(json.contains("\"soc_percent_of_initial\":null"));
        assert!(json.contains("\"shared_stock_ids\":[]"));
        assert!(json.contains("crew.food_store"));
    }

    /// A missing file and an `rk4` file-loaded scenario are both loud `Validation` errors (the
    /// UI renders `false`). The rk4 file is derived from the committed `crew_mission` by
    /// swapping the integrator, written to a unique temp path.
    #[test]
    fn build_session_from_file_rejects_rk4_and_missing_file() {
        let missing = scenarios_dir().join("no_such_scenario.yaml");
        assert!(matches!(
            build_session_from_file(&missing, &BTreeMap::new()),
            Err(SimError::Validation(_))
        ));

        let crew =
            std::fs::read_to_string(scenarios_dir().join("crew_mission.yaml")).unwrap();
        let rk4 = crew.replace("integrator: euler", "integrator: rk4");
        assert!(rk4.contains("integrator: rk4"), "the integrator swap must apply");
        let tmp = std::env::temp_dir().join("godot_bridge_from_file_rk4.yaml");
        std::fs::write(&tmp, rk4).unwrap();
        let result = build_session_from_file(&tmp, &BTreeMap::new());
        let _ = std::fs::remove_file(&tmp);
        assert!(
            matches!(result, Err(SimError::Validation(_))),
            "an rk4 file-loaded scenario must be rejected (single-rate Euler only; deferred)"
        );
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
