//! Phase-8 (P8.5) — the **cross-domain** perturbation composers, the Rust port of
//! `station.perturbations`.
//!
//! Each composer is a **scenario-layer intervention composed onto the already-assembled
//! station inputs**, and each is a **cascade with no cascade code** — a disturbance in one
//! domain propagates into another through a shared stock (or a shared *forcing*, #16)
//! alone. The generic seam-types ([`window_override`] / [`with_forcing`] / [`LeakFlow`])
//! are reused from [`domains::biosphere::perturbations`]; this module adds the
//! station-specific pieces: the multiplicative [`window_scale`], the windowed flow-scaler
//! [`ScaledFlow`], and the five `with_*` composers.
//!
//! **The move-pipeline shape (Rust, not Python).** Python rebuilds a `Registry` from
//! `list(registry.flows)` and shallow-copies `resolver.forcings`; Rust can do neither (a
//! `Box<dyn Flow>` / a `Schedule` is non-`Clone`). So a flow-modifying composer *consumes*
//! its [`Registry`] via [`Registry::into_parts`], transforms the owned flow vec, and
//! rebuilds; a forcing composer *consumes* its [`SourceResolver`] via
//! [`SourceResolver::into_parts`]. This is applied **before** the session takes ownership,
//! so the ordering matches Python's "compose, then run."
//!
//! **The two substrates (advisor, physics + compute).** Energy perturbations
//! ([`with_brownout`] / [`with_radiator_failure`]) run on the single-rate diurnal station
//! (Power → Thermal); matter perturbations ([`with_station_leak`] / [`with_crew_load_spike`]
//! / [`with_lighting_failure`]) run on a short two-rate sealed station. The station
//! regulators *erase* the naive pool-level signature (the Step-6/P6.8 finding), so a
//! matter cascade shows up as regulator **effort** + sinks, not pool level.
//!
//! **Zero parity concern / no golden.** A perturbed run is a diagnostic (the Phase-3/P6.8
//! "diagnostics, no golden" precedent); determinism (a re-run is bit-identical) is the
//! insurance. `ScaledFlow` with `health = 1` outside the window reproduces the wrapped flow
//! **bit-identically** (`x·1.0 == x`), so a window the run never reaches is inert.

use domains::biosphere::perturbations::{with_forcing, window_override, LeakFlow, LEAK_SINK, LEAK_VAR};
use domains::biosphere::stocks::PAR_VAR;
use domains::crew::FOOD_INTAKE_VAR;
use domains::power::SOLAR_POWER_VAR;
use domains::thermal::RADIATOR_REJECT;

use simcore::boundary::BOUNDARY_DOMAIN;
use simcore::environment::{constant, Environment, Schedule, SourceResolver};
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::quantities::StockKind;
use simcore::registry::Registry;
use simcore::state::{State, Stock};

use crate::flows::LAMP_POWER_VAR;

/// The radiator's windowed "health" forcing var (1.0 = nominal, 0.0 = total loss). Local
/// to this module — never added to any domain's catalog, so baseline assembly never carries
/// it (the `LEAK_VAR` discipline) and the frozen goldens are untouched.
pub const RADIATOR_HEALTH_VAR: &str = "radiator_health";

/// The station leak flow's id — station-owned, kept out of the biosphere-slow registry so
/// the sealed build's flow-id disjointness holds.
pub const STATION_LEAK: &str = "station.atmospheric_leak";

// --- the multiplicative window (the complement to window_override) --------------------

/// A [`Schedule`] returning `factor · base` on `[start, end)`, else `base` — the Rust port
/// of Python `window_scale`, the *multiplicative* sibling of [`window_override`].
///
/// A pure function of the integer step `n` (#14) that **scales** an existing, `n`-varying
/// schedule over the window (a brownout dims the diurnal solar half-sine; a crew load spike
/// lifts `food_intake` by `factor > 1`). `factor = 1` is a no-op (bit-identical);
/// `factor = 0` degenerates to a blackout. `base` is owned (moved into the closure); `dt`
/// threads to `base` unchanged.
pub fn window_scale(base: Schedule, start: u64, end: u64, factor: f64) -> Schedule {
    Box::new(move |n, dt| {
        if start <= n && n < end {
            factor * base(n, dt)
        } else {
            base(n, dt)
        }
    })
}

/// Wrap one existing forcing var's schedule with a transform, preserving the rest — the
/// Rust idiom for Python's `with_forcing(resolver, var, transform(resolver.forcings[var]))`
/// (which cannot be expressed directly here, as a `Schedule` is non-`Clone`). Consumes the
/// resolver, removes the owned base schedule for `var`, applies `f`, reinserts, rebuilds.
/// Errors if `var` is absent (the perturbation target must exist in the resolver).
fn map_forcing(
    resolver: SourceResolver,
    var: &str,
    f: impl FnOnce(Schedule) -> Schedule,
) -> Result<SourceResolver, SimError> {
    let (mut forcings, shared) = resolver.into_parts();
    let base = forcings.remove(var).ok_or_else(|| {
        SimError::Reference(format!(
            "perturbation target forcing var {var:?} is absent from the resolver"
        ))
    })?;
    forcings.insert(var.to_string(), f(base));
    SourceResolver::new(forcings, shared)
}

// --- forcing-override perturbations (no new flow / no structural change) ---------------

/// Scale the `solar_power` forcing by `factor` over `[start, end)` (the brownout).
///
/// The cross-domain energy perturbation: dimming the diurnal solar supply lowers the
/// battery SOC (Power) **and** the heat Power dissipates into `thermal.node`, so the node
/// **cools** (Thermal) — a cascade with no cascade code. A short/shallow window stays
/// graceful (`rationed == 0`, SOC dips but > 0); a deep/long one (`factor = 0`) empties the
/// battery so `LoadDraw` cannot be met and `rationed > 0` **emerges** (still conserving —
/// the Euler backstop conserves as it rations). Runs on the single-rate diurnal station.
pub fn with_brownout(
    resolver: SourceResolver,
    start: u64,
    end: u64,
    factor: f64,
) -> Result<SourceResolver, SimError> {
    map_forcing(resolver, SOLAR_POWER_VAR, |base| {
        window_scale(base, start, end, factor)
    })
}

/// Scale crew `food_intake` by `factor` over `[start, end)` (the crew load spike).
///
/// A raised metabolic food intake drives more respiration, so the cabin CO₂/O₂ loads jump —
/// but the regulators absorb the pools, so the emergent signature is **regulator effort**
/// (`CO2Scrubber` removes more, `O2Makeup` supplies more) plus a **faster `food_store`
/// drawdown**, not a pool-level shift. O₂ consumption is derived from food (RQ = 1, the
/// merged `CrewRespiration`), so scaling food scales the O₂ side too. Runs on the short
/// two-rate sealed station (the fast registry's `food_intake`).
pub fn with_crew_load_spike(
    resolver: SourceResolver,
    start: u64,
    end: u64,
    factor: f64,
) -> Result<SourceResolver, SimError> {
    map_forcing(resolver, FOOD_INTAKE_VAR, |base| {
        window_scale(base, start, end, factor)
    })
}

/// Zero `par` **and** `lamp_power` over `[start, end)` (the #16 lamp lighting failure).
///
/// The one **two-resolver** perturbation: the lamp is a single device whose failure has a
/// *photon* leg (the biosphere's `par` **forcing**, in `bio_resolver`) and an *energy* leg
/// (the `Lamp` flow's `lamp_power` draw, in `fast_resolver`) — a flow cannot tell forcing
/// from a shared stock (#16), so both must be cut **together**. The cascade: PAR → 0 stalls
/// growth (biomass below baseline); and the lamp draws no energy, so the battery is
/// **spared** (drains slower). Returns the `(bio, fast)` resolver pair. Runs on the short
/// two-rate sealed station.
pub fn with_lighting_failure(
    bio_resolver: SourceResolver,
    fast_resolver: SourceResolver,
    start: u64,
    end: u64,
) -> Result<(SourceResolver, SourceResolver), SimError> {
    let new_bio = map_forcing(bio_resolver, PAR_VAR, |base| {
        window_override(base, start, end, 0.0)
    })?;
    let new_fast = map_forcing(fast_resolver, LAMP_POWER_VAR, |base| {
        window_override(base, start, end, 0.0)
    })?;
    Ok((new_bio, new_fast))
}

// --- the windowed flow-scaler (the new third seam-type) -------------------------------

/// Wrap a flow; multiply **all** its legs by a windowed `health ∈ [0, 1]` forcing — the
/// Rust port of Python `ScaledFlow`.
///
/// A *degrade an existing process* perturbation (radiator failure). It scales the **whole
/// flow** — every leg by the same `env.get(health_var)` — so the result stays internally
/// balanced (`Σ (α·leg) = α·Σ leg = 0` per quantity), the "arbitration scales the whole
/// flow" invariant applied as a disturbance rather than a backstop. `health = 1` outside
/// the window reproduces the wrapped flow **bit-identically** (`x · 1.0 == x`). The wrapped
/// flow is unmodified (composition over inheritance); `id` / `priority` delegate, so the
/// [`Registry`] sorts it into the wrapped flow's slot (order-independence preserved).
pub struct ScaledFlow {
    inner: Box<dyn Flow>,
    health_var: String,
}

impl ScaledFlow {
    /// Wrap `inner`, scaling all its legs by the windowed `health_var` forcing.
    pub fn new(inner: Box<dyn Flow>, health_var: String) -> Self {
        ScaledFlow { inner, health_var }
    }
}

impl Flow for ScaledFlow {
    fn id(&self) -> &str {
        self.inner.id()
    }

    fn priority(&self) -> i64 {
        self.inner.priority()
    }

    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let alpha = env.get(&self.health_var)?;
        let result = self.inner.evaluate(snapshot, env, dt)?;
        let legs: Vec<Leg> = result
            .legs
            .iter()
            .map(|leg| Leg::new(leg.stock.clone(), leg.amount * alpha))
            .collect::<Result<Vec<_>, _>>()?;
        FlowResult::new(legs)
    }
}

/// Throttle `RadiatorReject` to `health` over `[start, end)` (the radiator failure).
///
/// Consumes `registry` ([`Registry::into_parts`]), wraps the `RADIATOR_REJECT` flow in a
/// [`ScaledFlow`] reading a windowed [`RADIATOR_HEALTH_VAR`] (`health` inside the window,
/// `1.0` outside), and rebuilds over `state.stocks` (the `Registry` re-sorts by id, so the
/// wrapped flow keeps the radiator's slot; aux processes carry). Over the window the
/// radiator sheds only `health ×` its nominal rejection, so the node accumulates Power's
/// real dissipation and **heats** — the overheating cascade. Energy is conserved throughout
/// (the scaled leg is still balanced — heat stays in the node instead of leaving to
/// `space`); `rationed == 0` (a POOL accumulation, not a withdrawal shortfall). Runs on the
/// single-rate station.
pub fn with_radiator_failure(
    state: &State,
    registry: Registry,
    resolver: SourceResolver,
    start: u64,
    end: u64,
    health: f64,
) -> Result<(Registry, SourceResolver), SimError> {
    let (flows, aux) = registry.into_parts();
    let wrapped: Vec<Box<dyn Flow>> = flows
        .into_iter()
        .map(|flow| {
            if flow.id() == RADIATOR_REJECT {
                Box::new(ScaledFlow::new(flow, RADIATOR_HEALTH_VAR.to_string())) as Box<dyn Flow>
            } else {
                flow
            }
        })
        .collect();
    let new_registry = Registry::new(wrapped, &state.stocks, aux)?;
    let new_resolver = with_forcing(
        resolver,
        RADIATOR_HEALTH_VAR,
        window_override(constant(1.0)?, start, end, health),
    )?;
    Ok((new_registry, new_resolver))
}

// --- the added leak flow (two-registry sealed build) ----------------------------------

/// Augment the sealed build with a windowed leak `pool → LEAK_SINK` (matter).
///
/// The two-registry analogue of the biosphere leak. The sealed station has **two**
/// registries over **one** shared stock dict, so this:
///
/// * adds a [`LEAK_SINK`] BOUNDARY stock whose composition **mirrors `pool`'s** (a
///   `{C:1,O:2}` cabin-CO₂ pool vents CARBON **and** OXYGEN in balance);
/// * rebuilds **both** registries over the augmented stock dict (biosphere-slow preserving
///   its aux), and appends the [`LeakFlow`] to the **fast** registry (`dt = 60 s`, the rate
///   the cabin flows act on the shared pool; `k_leak·dt < 1` at 60 s ⇒ `rationed == 0`).
///   `STATION_LEAK` is kept **out of the biosphere-slow registry** so the disjointness holds;
/// * wires the [`LEAK_VAR`] activation forcing into the **fast** resolver as a windowed
///   override (`1` on `[start, end)`, `0` else) — under the master-day driver `n` is the day
///   count, so the window activates on whole master days.
///
/// The interior's closure breaks over the window (mass leaves to `LEAK_SINK`) but **total**
/// mass (interior + sink) stays conserved. The cascade is pool-specific: a `CARBON_POOL`
/// leak lowers Ci (biomass↓, scrubber does less), an `O2_POOL` leak is absorbed by
/// `O2Makeup` (its `o2_supply` effort grows, `cabin_o2` flat). Consumes the two registries
/// + the fast resolver; returns the augmented `(state, bio_registry, fast_registry, fast_resolver)`.
#[allow(clippy::too_many_arguments)]
pub fn with_station_leak(
    state: &State,
    bio_registry: Registry,
    fast_registry: Registry,
    fast_resolver: SourceResolver,
    pool: &str,
    k_leak: f64,
    start: u64,
    end: u64,
) -> Result<(State, Registry, Registry, SourceResolver), SimError> {
    let pool_stock = state.stocks.get(pool).ok_or_else(|| {
        SimError::Reference(format!("station leak pool {pool:?} is absent from the state"))
    })?;
    let leak_sink = Stock::new(
        LEAK_SINK.to_string(),
        BOUNDARY_DOMAIN.to_string(),
        pool_stock.quantity,
        pool_stock.unit.clone(),
        0.0,
        StockKind::Boundary,
        0.0,
        false,
        pool_stock.composition.clone(),
    )?;
    let mut new_stocks = state.stocks.clone();
    new_stocks.insert(LEAK_SINK.to_string(), leak_sink);

    let leak = LeakFlow::new(
        STATION_LEAK.to_string(),
        0,
        pool.to_string(),
        LEAK_SINK.to_string(),
        LEAK_VAR.to_string(),
        k_leak,
    );

    let (bio_flows, bio_aux) = bio_registry.into_parts();
    let new_bio = Registry::new(bio_flows, &new_stocks, bio_aux)?;

    let (mut fast_flows, fast_aux) = fast_registry.into_parts();
    fast_flows.push(Box::new(leak));
    let new_fast = Registry::new(fast_flows, &new_stocks, fast_aux)?;

    let new_state = State::new(state.n, new_stocks, state.rng_seed, state.aux.clone())?;
    let new_fast_resolver = with_forcing(
        fast_resolver,
        LEAK_VAR,
        window_override(constant(0.0)?, start, end, 1.0),
    )?;
    Ok((new_state, new_bio, new_fast, new_fast_resolver))
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};

    use domains::thermal::{RadiatorReject, NODE, SPACE};
    use simcore::quantities::Quantity;

    use crate::system::{build_station, station_resolver};

    use super::*;

    fn cst(v: f64) -> Schedule {
        constant(v).unwrap()
    }

    fn at_step(n: u64) -> State {
        State::new(n, BTreeMap::new(), 0, BTreeMap::new()).unwrap()
    }

    #[test]
    fn window_scale_scales_inside_defers_outside() {
        let sched = window_scale(cst(4.0), 2, 5, 0.5);
        assert_eq!(sched(1, 1.0), 4.0); // before → base
        assert_eq!(sched(3, 1.0), 2.0); // inside → factor·base
        assert_eq!(sched(5, 1.0), 4.0); // end exclusive → base
        // factor = 1.0 is a bit-identical no-op.
        let noop = window_scale(cst(4.0), 2, 5, 1.0);
        assert_eq!(noop(3, 1.0), 4.0);
    }

    #[test]
    fn scaled_flow_scales_whole_flow_and_stays_balanced() {
        // The new seam-type's invariant (mirrors test_scaled_flow_scales_whole_flow_balanced):
        // every leg × the same alpha, so an internally-balanced flow stays balanced, and each
        // leg is exactly alpha× the wrapped leg. Checked on the real RadiatorReject.
        let thermal = domains::params::thermal();
        let inner = RadiatorReject::new(
            RADIATOR_REJECT.to_string(),
            NODE.to_string(),
            SPACE.to_string(),
            thermal,
        );
        let stocks = BTreeMap::from([
            (
                NODE.to_string(),
                domains::thermal::node_stock(5.0e6).unwrap(),
            ),
            (
                SPACE.to_string(),
                simcore::boundary::sink(SPACE.to_string(), Quantity::Energy, 0.0).unwrap(),
            ),
        ]);
        let state = State::new(0, stocks, 0, BTreeMap::new()).unwrap();

        let inner_legs = inner
            .evaluate(&state, &SourceResolver::empty().bind(&state, 60.0), 60.0)
            .unwrap();
        let scaled = ScaledFlow::new(Box::new(inner), RADIATOR_HEALTH_VAR.to_string());
        assert_eq!(scaled.id(), RADIATOR_REJECT);

        let mut forcings: HashMap<String, Schedule> = HashMap::new();
        forcings.insert(RADIATOR_HEALTH_VAR.to_string(), cst(0.25));
        let resolver = SourceResolver::new(forcings, HashMap::new()).unwrap();
        let scaled_legs = scaled
            .evaluate(&state, &resolver.bind(&state, 60.0), 60.0)
            .unwrap();

        for (s, i) in scaled_legs.legs.iter().zip(inner_legs.legs.iter()) {
            assert_eq!(s.stock, i.stock);
            assert_eq!(s.amount, 0.25 * i.amount); // exactly alpha× the wrapped leg
        }
        // The scaled flow is still per-quantity balanced (ENERGY sums to 0).
        assert!(
            simcore::flow::assert_flow_balanced_default(&scaled_legs, &state.stocks).is_ok()
        );

        // alpha = 1.0 reproduces the wrapped flow bit-identically (x·1.0 == x).
        let mut ones: HashMap<String, Schedule> = HashMap::new();
        ones.insert(RADIATOR_HEALTH_VAR.to_string(), cst(1.0));
        let one_res = SourceResolver::new(ones, HashMap::new()).unwrap();
        let one_legs = scaled
            .evaluate(&state, &one_res.bind(&state, 60.0), 60.0)
            .unwrap();
        for (s, i) in one_legs.legs.iter().zip(inner_legs.legs.iter()) {
            assert_eq!(s.amount, i.amount);
        }
    }

    #[test]
    fn with_brownout_dims_solar_only_in_window() {
        let charge = domains::params::charge();
        let scenario = crate::scenario::HEAT_CLOSURE_SCENARIO;
        let base = station_resolver(&charge, &scenario).unwrap();
        // Sample the baseline solar at a daytime step, then the same step under brownout.
        let midday = at_step(12); // midday-ish of day 0 (24 steps/day)
        let base_solar = base.bind(&midday, 3600.0).get(SOLAR_POWER_VAR).unwrap();
        assert!(base_solar > 0.0, "daytime solar should be positive");

        let perturbed = with_brownout(base, 10, 15, 0.5).unwrap();
        assert_eq!(
            perturbed.bind(&midday, 3600.0).get(SOLAR_POWER_VAR).unwrap(),
            0.5 * base_solar
        );
        // Outside the window: unchanged (bit-identical to the base half-sine).
        let out = at_step(20);
        let ref_res = station_resolver(&charge, &scenario).unwrap();
        assert_eq!(
            perturbed.bind(&out, 3600.0).get(SOLAR_POWER_VAR).unwrap(),
            ref_res.bind(&out, 3600.0).get(SOLAR_POWER_VAR).unwrap()
        );
    }

    #[test]
    fn with_radiator_failure_wraps_only_the_radiator_and_wires_health() {
        let charge = domains::params::charge();
        let thermal = domains::params::thermal();
        let scenario = crate::scenario::HEAT_CLOSURE_SCENARIO;
        let (state, registry) = build_station(&charge, &thermal, &scenario, None).unwrap();
        let resolver = station_resolver(&charge, &scenario).unwrap();
        let (reg, res) =
            with_radiator_failure(&state, registry, resolver, 3, 9, 0.0).unwrap();
        // The registry keeps every flow id (the ScaledFlow delegates RADIATOR_REJECT's id).
        let ids: Vec<&str> = reg.flows().iter().map(|f| f.id()).collect();
        assert!(ids.contains(&RADIATOR_REJECT));
        assert!(ids.contains(&"power.solar_charge"));
        // The health forcing is wired: 0.0 inside the window, 1.0 outside.
        let inside = State::new(5, state.stocks.clone(), 0, BTreeMap::new()).unwrap();
        assert_eq!(res.bind(&inside, 3600.0).get(RADIATOR_HEALTH_VAR).unwrap(), 0.0);
        let outside = State::new(0, state.stocks.clone(), 0, BTreeMap::new()).unwrap();
        assert_eq!(res.bind(&outside, 3600.0).get(RADIATOR_HEALTH_VAR).unwrap(), 1.0);
    }
}
