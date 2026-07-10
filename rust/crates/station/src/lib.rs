//! Native Rust port of the frozen Python `station.*` cross-domain assembly (P7.5).
//!
//! The assembly layer that imports both the four Phase-5 siblings ([`domains::power`],
//! [`domains::thermal`], [`domains::eclss`], [`domains::crew`]) and the [`domains::biosphere`]
//! and owns **all** cross-domain wiring by *choosing which stock id each flow points at*
//! (finding #1) — no domain imports another. The eight coupled goldens split by tier:
//! `cabin_gas` / `water_recovery` are **Tier-1 bit-exact** (transcendental-free, cabin-only);
//! `station_state`, `greenhouse`, `lighting`, `harvest`, `sealed_station`, and the
//! `sealed_energy_drift` summary are **Tier-2** (a `sin`/`T⁴`/FvCB transcendental in the
//! graph). Tier-0 (`rationed==0`, `events==()`, conservation every (sub-)step) is asserted
//! in Rust by the runners + the emit examples.
//!
//! Every `evaluate` / builder mirrors the Python arithmetic and leg/stock construction order
//! character-for-character; the two-rate [`driver::run_master_day`] ports the per-substep
//! `assert_conserved` teeth (the primary Tier-0 gate — `substep` deliberately skips it).

pub mod cabin;
pub mod display;
pub mod driver;
pub mod flows;
pub mod greenhouse;
pub mod harvest;
pub mod inspection;
pub mod lighting;
pub mod params;
pub mod perturbations;
pub mod scenario;
pub mod sealed;
pub mod session;
pub mod stocks;
pub mod system;
pub mod water;

use simcore::environment::SourceResolver;
use simcore::error::SimError;
use simcore::events::Event;
use simcore::integrator::EulerIntegrator;
use simcore::state::State;

/// Step `steps` times under Euler (single-rate), calling `observer` on the initial state
/// and each produced state; returns `(final_state, total_rationed, events)`.
///
/// The `station.system.run_station` analogue (no reset hook — the single-rate seams have no
/// phenology/intervention). The every-step conservation gate runs inside
/// [`EulerIntegrator::step_report`], so a completed run proves the combined ledger balanced
/// every step (the Tier-0 leg). `observer` lets the caller collect the trajectory (the
/// energy-drift run reads per-step node amounts) without every runner materializing a `Vec`.
pub fn run_station(
    integrator: &EulerIntegrator,
    initial: State,
    resolver: &SourceResolver,
    dt: f64,
    steps: u64,
    observer: &mut dyn FnMut(&State),
) -> Result<(State, u64, Vec<Event>), SimError> {
    let mut state = initial;
    let mut total_rationed = 0u64;
    let mut events: Vec<Event> = Vec::new();
    observer(&state);
    for _ in 0..steps {
        let report = integrator.step_report(&state, resolver, dt)?;
        state = report.state;
        observer(&state);
        total_rationed += report.rationed;
        events.extend(report.events);
    }
    Ok((state, total_rationed, events))
}
