//! **The hull breach** — a composer that vents an assembled chamber's whole atmosphere.
//!
//! Plan: `docs/plans/post-roadmap-atmosphere.md` §1.3.
//!
//! # ⚠ Why this is not in `src/biosphere/perturbations.rs`, where its primitives live
//!
//! It **recomposes a registry** (`into_parts` → append → `Registry::new`), and
//! `tests/one_assembly_body.rs` holds the biosphere spine to assembling a season in exactly
//! one place — `system::build_season_with`. That gate's own remedy line is *"the fix is never
//! to widen this list: it is to compose onto the one build"*, and that is precisely what this
//! module does: it takes an **already-built** `(State, Registry)` and appends to it, so it
//! cannot cause the drift the gate exists against.
//!
//! Living beside the spine rather than inside it is the established shape here, not a dodge —
//! `lab::mechanism::build_season_without` recomposes the same way for knockouts, `ulp_probe`
//! for its probe registry, and `station::perturbations::with_station_leak` for the station's
//! single-pool leak, which is this composer's direct ancestor. `biosphere::perturbations`
//! keeps the *primitives* ([`LeakFlow`](crate::biosphere::perturbations::LeakFlow),
//! `window_override`, `with_forcing`); the composing happens in the consumer.

use simcore::boundary::BOUNDARY_DOMAIN;
use simcore::environment::{constant, SourceResolver};
use simcore::error::SimError;
use simcore::flow::Flow;
use simcore::quantities::StockKind;
use simcore::registry::Registry;
use simcore::state::{State, Stock};

use crate::biosphere::perturbations::{window_override, with_forcing, LeakFlow};
use crate::biosphere::stocks::{CARBON_POOL, CHAMBER_INERT, O2_POOL, WATER_VAPOR};

// --- the hull breach: the whole atmosphere leaves, at one rate ------------------------

/// The hull breach's windowed-activation forcing var. Harness-local, like [`LEAK_VAR`].
pub const BREACH_VAR: &str = "hull_breach";

/// Id prefix for the breach's **per-gas** boundary sinks (`boundary.breach_sink.o2_pool`).
///
/// ⚠ The reason this is a prefix and [`LEAK_SINK`] is a single constant: a breach vents four
/// species at once, and four legs into one sink would need that sink to carry four
/// compositions. One sink per gas keeps every leg per-quantity balanced, which is what makes
/// the breach **conservative** — `interior + sinks` is constant across the window. The
/// predecessor plan deferred a two-gas leak for exactly this collision.
pub const BREACH_SINK_PREFIX: &str = "boundary.breach_sink.";

/// Id prefix for the breach's per-gas leak flows.
pub const BREACH_FLOW_PREFIX: &str = "biosphere.hull_breach.";

/// The chamber's gas stocks, in the fixed order the breach builds its flows in.
///
/// ⚠ All four, and water vapour is not an oversight: vapour occupies volume like any other
/// gas, so a breach that left it behind would raise the humidity of a chamber that is losing
/// its air. Stocks absent from the state (an open field has none of them) are skipped.
const BREACH_GASES: [&str; 4] = [CARBON_POOL, O2_POOL, CHAMBER_INERT, WATER_VAPOR];

/// Vent **every** gas in the chamber over `[start, end)`, first-order at a single `k_leak`.
///
/// # Why one rate constant is the physics, not a simplification
///
/// A well-mixed volume venting to vacuum loses each species in proportion to **its own partial
/// pressure**, so every species leaves first-order in its own amount at the *same* `k`. Then
/// `n_i(t) = n_i(0)·e^{-kt}` for all `i`, and therefore:
///
/// * every **ratio** `n_i/n_j` is invariant — the breach is composition-preserving, with no
///   coefficient tuned to make it so;
/// * the **total** falls, so pressure falls;
/// * each species' partial pressure `n_i/n_ref` falls by the same factor.
///
/// That last line is the point of the whole atmosphere item: this is the state E2 could only
/// *impose* by hand, now **arrived at** by a mechanism that conserves mass into a boundary.
///
/// ⚠ `k_leak` is per unit of the `dt` the composed domain runs at (per **day** in the
/// biosphere, per second in the station's fast loop), because [`LeakFlow`] multiplies by the
/// step it is handed. `k_leak·dt < 1` keeps the arbitration backstop unfired.
///
/// Diagnostic only — no golden pins a breached run; determinism is the insurance.
pub fn with_hull_breach(
    state: &State,
    registry: Registry,
    resolver: SourceResolver,
    k_leak: f64,
    start: u64,
    end: u64,
) -> Result<(State, Registry, SourceResolver), SimError> {
    let mut stocks = state.stocks.clone();
    let mut breaches: Vec<Box<dyn Flow>> = Vec::new();

    for gas in BREACH_GASES {
        let Some(pool) = state.stocks.get(gas) else {
            continue;
        };
        // The short name is the id past its domain dot — `biosphere.o2_pool` → `o2_pool` —
        // so the sink id is readable and stays unique across the four.
        let short = gas.rsplit('.').next().unwrap_or(gas);
        let sink_id = format!("{BREACH_SINK_PREFIX}{short}");
        stocks.insert(
            sink_id.clone(),
            Stock::new(
                sink_id.clone(),
                BOUNDARY_DOMAIN.to_string(),
                pool.quantity,
                pool.unit.clone(),
                0.0,
                StockKind::Boundary,
                0.0,
                false,
                // Mirrors the vented pool exactly, so the two legs balance in EVERY
                // quantity the pool carries — a CO2 pool vents both CARBON and OXYGEN.
                pool.composition.clone(),
            )?,
        );
        breaches.push(Box::new(LeakFlow::new(
            format!("{BREACH_FLOW_PREFIX}{short}"),
            0,
            gas.to_string(),
            sink_id,
            BREACH_VAR.to_string(),
            k_leak,
        )));
    }
    if breaches.is_empty() {
        return Err(SimError::Reference(
            "hull breach composed onto a state with no chamber gas stocks — an open field              has no atmosphere to lose"
                .to_string(),
        ));
    }

    let (mut flows, aux) = registry.into_parts();
    flows.append(&mut breaches);
    let registry = Registry::new(flows, &stocks, aux)?;
    let state = State::new(state.n, stocks, state.rng_seed, state.aux.clone())?;
    let resolver = with_forcing(
        resolver,
        BREACH_VAR,
        window_override(constant(0.0)?, start, end, 1.0),
    )?;
    Ok((state, registry, resolver))
}
