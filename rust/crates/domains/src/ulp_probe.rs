//! The **basis** under the Tier-2 bands: the propagated ±1-ULP transcendental sensitivity.
//!
//! Port of `tests/crossport/measure_tier2_bands.py`, the half D4 deferred — Stage 3, the
//! reference flip. D4 moved the tolerance *data* (`rust/data/tiers.json`) and the banded
//! *comparison* ([`crate::tiers`]) into the reference and left this behind, named, with a
//! deadline rather than an open end: the Python instrument substitutes a `math` reference
//! **inside the Python domain modules** and runs the **Python engine**, so it dies with the
//! tree S6 deletes. Until this module existed the committed bands were *asserted* in Rust
//! and *justified* only in Python.
//!
//! # What a band is, and why it is measured rather than derived
//!
//! Rust `f64::sin` / `powf` / `exp` and CPython `math.sin` / `**` / `math.exp` resolve to the
//! **same system libm** on one machine, so a direct port-vs-port comparison there reads 0.0 —
//! a *same-libm artifact*, not a measurement of the thing Tier 2 exists to bound. So the band
//! is sized against the **sensitivity**: perturb the scenario's dominant transcendental by
//! ±1 ULP, re-run to the final state, and take the worst relative deviation. That is how far
//! a one-ULP libm disagreement moves the whole trajectory, and a band must sit above it.
//!
//! # ⚠⚠ The trap this module is built around: a probe that measures nothing
//!
//! The Python instrument's own comments record it. Both biosphere probes once shimmed
//! `domains.biosphere.canopy`'s `math` after the layered canopy had moved the Beer-Lambert
//! `exp` into `photosynthesis`, so they perturbed a function the carbon path no longer
//! called and measured **exactly 0.0** — whereupon `sensitivity < band` kept passing,
//! vacuously, for weeks. **A re-measurement that reads zero is the failure mode, not the
//! result.** Three defences, and the third is the one the Python side never had:
//!
//! 1. [`Nudge::Off`] must reproduce the golden emitter's bytes **exactly** — so the probe
//!    harness cannot have drifted from the run it claims to perturb;
//! 2. every measured sensitivity must be `> 0.0`;
//! 3. every measured sensitivity must land within an order of magnitude of the figure the
//!    Python instrument measured for the same scenario (`tiers.json`'s
//!    `_reference_flip.measured_2026_08_16` block). A probe that reads `1e-30` is non-zero
//!    and still wrong; only the cross-check against the number this is a *re*-measurement of
//!    can see that.
//!
//! # The four seams, and why three of them need no engine change
//!
//! | run | Python shims | here |
//! |---|---|---|
//! | power (×2) | `domains.power.system.math.sin` | [`nudged_power_resolver`] — the schedule's `sin` **and** the load derived from it |
//! | thermal | `domains.thermal.flows.radiated_power` | [`NudgedRadiatorReject`] swapped into the registry |
//! | biosphere | `canopy.math.exp` + `photosynthesis.math.exp` | the `par` forcing |
//! | station / greenhouse | both of the above | the same seams, composed (in `station::ulp_probe`) |
//!
//! **Power.** Python shims the module's whole `math` reference, which reaches both
//! [`crate::power::solar_schedule`] *and* [`crate::power::balanced_load_w`] — the derived load
//! is a sum over that same `sin`. So [`nudged_power_resolver`] mirrors both, nudging the `sin`
//! **result** rather than the `peak · sin(…)` product.
//!
//! ⚠ Nudging the `sin` **result** rather than the `peak · sin(…)` product is not pedantry: a
//! one-ULP step is a *relative* perturbation of between `2⁻⁵³` and `2⁻⁵²` depending on where
//! the value sits inside its binade, so the two spellings are perturbations of different size.
//! The mirror is safe because of the control below — with [`Nudge::Off`] it must emit the
//! golden's exact bytes.
//!
//! # ⚠⚠ What the power re-measurement found, and it was not about the probe
//!
//! The first reading came back at **exactly half** the Python instrument's, on both power
//! runs. Chasing an exact factor of two rather than accepting it inside an order-of-magnitude
//! window is what turned it into a finding. The 24 schedule values are bit-identical across
//! the two ports, nudged and un-nudged alike; the **derived load** differs by one ULP.
//!
//! **CPython's builtin `sum()` has used Neumaier compensation for floats since 3.12.** It is
//! not a left-to-right accumulation, and [`crate::power::daily_solar_energy`] — whose own
//! comment claimed to mirror it — is. The two agree bit-for-bit on the frozen scenario, which
//! is why every golden matches and nothing caught it, and disagree the moment the summands
//! move. Compensating this one sum in a throwaway Rust probe reproduced **both** Python
//! figures exactly (`5.215406e-15`, `4.146325e-15`), which is what makes this measured rather
//! than inferred.
//!
//! The reference accumulates naively, so the naive number is the reference's own sensitivity
//! and the one asserted below. The band sits ~380× above it either way, so nothing moved.
//! ⚠ The durable half: **a compensated reduction is a second source of cross-port divergence,
//! independent of libm**, and the tolerance contract's rationale names only transcendentals.
//!
//! **Biosphere.** The `par` forcing is the exact seam, not an approximation, and it was
//! checked rather than argued. `incident_par` enters [`crate::biosphere::science::canopy_assimilation`]
//! in exactly one place — `absorbed_par = k · incident_par · exp(−k·depth·lai)` — so nudging
//! `incident_par` by a relative ULP and nudging the `exp` by a relative ULP are the *same*
//! perturbation of `absorbed_par`. And `env.get("par")` has exactly one consumer in the whole
//! workspace ([`crate::biosphere::flows`]'s `CarbonContext`); the station's lighting/sealed
//! seams *write* `PAR_VAR`, none reads it. So nothing else moves.
//!
//! ⚠ The asymmetry this probe inherited is now **half** resolved, and the half that remains
//! is the reason this note stays. Python shims `canopy.math` for `intercepted_fraction`'s
//! `exp`; on the reference side that function was dead (no production call site) and was
//! **deleted on 2026-08-27**, closing clause 4 of S5's exit gate. The Python twin is still
//! there and still shimmed, so `measure_tier2_bands.py` goes on perturbing a path its own
//! carbon budget no longer runs. That contribution was **measured at exactly zero** in
//! 2026-08-15 (both biosphere rows read `0.0` when `canopy` was the only shim, which is what
//! forced the second shim), so the Python instrument's numbers come from its
//! `photosynthesis` shim alone and the deletion cannot move them. It dies with the checker
//! at S6; nothing here depends on it.
//!
//! **Thermal.** The `t⁴` is inside a flow, and the subtraction `t⁴ − T_space⁴` can cancel, so
//! perturbing the flow's *output* would understate the sensitivity — the one place a cheaper
//! seam is measurably wrong. Hence the mirrored flow, which is eight lines and whose fidelity
//! defence 1 above proves.

use std::collections::{BTreeMap, HashMap};

use simcore::environment::{constant, Environment, Schedule, SourceResolver};
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::integrator::EulerIntegrator;
use simcore::registry::Registry;
use simcore::state::{State, Stock};

use crate::biosphere::stocks::PAR_VAR;
use crate::biosphere::{
    consumer_chamber_scenario, perennial_chamber_scenario, run_perennial, season_setup,
    season_steps, steps_for_years, SeasonScenario, BIO_DT,
};
use crate::power::{
    build_power, ChargeParams, PowerScenario, BOUNDED_SOC_DAYS, BOUNDED_SOC_SCENARIO,
    LOAD_POWER_VAR, SELF_DISCHARGE_DAYS, SOLAR_POWER_VAR,
};
use crate::thermal::{
    build_thermal, temperature, thermal_resolver, ThermalParams, EQUILIBRIUM_SCENARIO,
    EQUILIBRIUM_STEPS, NODE, RADIATOR_REJECT, SPACE, STEFAN_BOLTZMANN,
};
use crate::{params, run, tiers};

// --------------------------------------------------------------------------- //
// The perturbation itself                                                      //
// --------------------------------------------------------------------------- //

/// Which way to move a transcendental's result by one unit in the last place.
///
/// [`Nudge::Off`] is not a convenience: it is the control. A probe harness with the nudge
/// off must emit the golden's exact bytes, which is the only thing that proves the harness
/// perturbs *the run the band is about* rather than a lookalike that has drifted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nudge {
    /// No perturbation — the baseline, and the fidelity control.
    Off,
    /// One ULP toward `+∞`.
    Up,
    /// One ULP toward `−∞`.
    Down,
}

impl Nudge {
    /// Both perturbing directions. A libm disagreement has no preferred sign and the
    /// propagated deviation is not symmetric, so every measurement takes the worse of the two.
    pub const BOTH: [Nudge; 2] = [Nudge::Up, Nudge::Down];

    /// `x` moved one ULP in this direction (`Off`, `0.0` and non-finite are identities).
    ///
    /// ⚠ **Zero is deliberately an identity**, and it is not tidiness. `nextafter(0, +∞)` is
    /// the smallest subnormal — a *relative* change of infinity, which no libm disagreement
    /// can produce. The solar schedule returns a literal `0.0` at night and `exp` is never
    /// zero, so a nudge at zero would model a divergence that cannot happen and would put
    /// light in the canopy at midnight.
    pub fn apply(self, x: f64) -> f64 {
        let up = match self {
            Nudge::Off => return x,
            Nudge::Up => true,
            Nudge::Down => false,
        };
        if x == 0.0 || !x.is_finite() {
            return x;
        }
        // IEEE-754 orders finite same-sign floats by their bit pattern: for `x > 0`
        // increasing bits move toward `+∞`, and for `x < 0` (sign bit set) they move away
        // from it. Hence the XOR-ish condition rather than a bare `+ 1`.
        let bits = x.to_bits();
        f64::from_bits(if (x > 0.0) == up { bits + 1 } else { bits - 1 })
    }
}

/// Rebuild `resolver` with one forcing's **output** nudged — the Rust analogue of Python
/// replacing a module's `math` attribute.
///
/// Consumes the resolver ([`SourceResolver::into_parts`]) because a [`simcore::environment::Schedule`]
/// is a non-`Clone` `Box<dyn Fn>`, exactly as [`crate::biosphere::perturbations::with_forcing`]
/// does. **Errors if `var` is not a forcing**: a probe that shims a variable the run does not
/// read is the 2026-08-15 vacuous-zero trap, and it must fail loudly rather than measure 0.0.
pub fn nudge_forcing(
    resolver: SourceResolver,
    var: &str,
    nudge: Nudge,
) -> Result<SourceResolver, SimError> {
    let (mut forcings, shared) = resolver.into_parts();
    let base = forcings.remove(var).ok_or_else(|| {
        SimError::Reference(format!(
            "ulp_probe: no forcing {var:?} to perturb — a probe that shims a var the run does \
             not read measures nothing"
        ))
    })?;
    forcings.insert(
        var.to_string(),
        Box::new(move |n, dt| nudge.apply(base(n, dt))),
    );
    SourceResolver::new(forcings, shared)
}

/// [`crate::power::solar_schedule`] with the half-sine `sin` moved one ULP.
///
/// An op-for-op mirror of the frozen schedule — same phase arithmetic, same window
/// comparison, same night branch returning a literal `0.0` — with the single `sin` result
/// nudged. The station reuses it: [`crate::power::power_resolver`] *is* the station's
/// resolver (`station::system::station_resolver` delegates to it).
fn nudged_solar_schedule(scenario: &PowerScenario, nudge: Nudge) -> Schedule {
    let spd = scenario.steps_per_day;
    let peak = scenario.solar_peak_w;
    let daylight_fraction = scenario.daylight_hours / 24.0;
    let sunrise = 0.5 - daylight_fraction / 2.0;
    let sunset = 0.5 + daylight_fraction / 2.0;
    Box::new(move |n: u64, _dt: f64| {
        let phase = (n % spd) as f64 / spd as f64;
        if sunrise <= phase && phase < sunset {
            peak * nudge.apply((std::f64::consts::PI * (phase - sunrise) / daylight_fraction).sin())
        } else {
            0.0
        }
    })
}

/// [`crate::power::balanced_load_w`] over the nudged schedule — Python's shim reaches the
/// derived load too, because the daily solar sum runs through the same `sin`.
fn nudged_balanced_load_w(charge: &ChargeParams, scenario: &PowerScenario, nudge: Nudge) -> f64 {
    let solar = nudged_solar_schedule(scenario, nudge);
    let dt = scenario.dt_seconds;
    let mut daily = 0.0;
    for n in 0..scenario.steps_per_day {
        daily += solar(n, dt) * dt;
    }
    let stored_per_day = charge.charge_efficiency * daily;
    let day_seconds = scenario.steps_per_day as f64 * scenario.dt_seconds;
    scenario.load_fraction * stored_per_day / day_seconds
}

/// [`crate::power::power_resolver`] with the half-sine `sin` nudged everywhere it is reached.
pub fn nudged_power_resolver(
    charge: &ChargeParams,
    scenario: &PowerScenario,
    nudge: Nudge,
) -> Result<SourceResolver, SimError> {
    let mut forcings: HashMap<String, Schedule> = HashMap::new();
    forcings.insert(
        SOLAR_POWER_VAR.to_string(),
        nudged_solar_schedule(scenario, nudge),
    );
    forcings.insert(
        LOAD_POWER_VAR.to_string(),
        constant(nudged_balanced_load_w(charge, scenario, nudge))?,
    );
    SourceResolver::new(forcings, HashMap::new())
}

/// [`crate::thermal::RadiatorReject`] with the Stefan-Boltzmann `t⁴` moved one ULP.
///
/// A byte-for-byte mirror of the frozen flow's `evaluate` except for the single nudged
/// `powf(4.0)`; every ingredient ([`temperature`], [`STEFAN_BOLTZMANN`], the params) is public,
/// so nothing about the rate law is re-derived here. It keeps the frozen flow's **id and
/// type name** on purpose: the perturbed registry must be structurally the frozen registry,
/// or [`Nudge::Off`] reproducing the golden would prove nothing.
pub struct NudgedRadiatorReject {
    node: String,
    space: String,
    params: ThermalParams,
    nudge: Nudge,
}

impl Flow for NudgedRadiatorReject {
    fn type_name(&self) -> &'static str {
        "RadiatorReject"
    }
    fn id(&self) -> &str {
        RADIATOR_REJECT
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let node_joules = snapshot
            .stocks
            .get(&self.node)
            .map(|s| s.amount)
            .ok_or_else(|| {
                SimError::Reference(format!("flow reads unknown stock {:?}", self.node))
            })?;
        let t = temperature(
            node_joules,
            self.params.heat_capacity,
            self.params.space_temperature,
        );
        let t4 = self.nudge.apply(t.powf(4.0)); // the transcendental, perturbed
        let rejected = self.params.emissivity
            * STEFAN_BOLTZMANN
            * self.params.radiator_area
            * (t4 - self.params.space_temperature.powf(4.0))
            * dt;
        FlowResult::new(vec![
            Leg::new(self.node.clone(), -rejected)?,
            Leg::new(self.space.clone(), rejected)?,
        ])
    }
}

/// Swap the registry's `RadiatorReject` for a [`NudgedRadiatorReject`].
///
/// Used by both the standalone thermal run and the coupled station one — they register the
/// *same* flow with the same ids ([`crate::thermal::RadiatorReject::new`] with [`NODE`] /
/// [`SPACE`]), so one seam serves both. Errors if the flow is absent, for the same reason
/// [`nudge_forcing`] does.
pub fn nudge_radiator(
    registry: Registry,
    stocks: &BTreeMap<String, Stock>,
    params: &ThermalParams,
    nudge: Nudge,
) -> Result<Registry, SimError> {
    let (flows, aux) = registry.into_parts();
    let mut swapped = false;
    let mut out: Vec<Box<dyn Flow>> = Vec::with_capacity(flows.len());
    for flow in flows {
        if flow.id() == RADIATOR_REJECT {
            out.push(Box::new(NudgedRadiatorReject {
                node: NODE.to_string(),
                space: SPACE.to_string(),
                params: *params,
                nudge,
            }));
            swapped = true;
        } else {
            out.push(flow);
        }
    }
    if !swapped {
        return Err(SimError::Reference(format!(
            "ulp_probe: no flow {RADIATOR_REJECT:?} in the registry to perturb"
        )));
    }
    Registry::new(out, stocks, aux)
}

// --------------------------------------------------------------------------- //
// The metric                                                                   //
// --------------------------------------------------------------------------- //

/// The worst relative deviation between two snapshots, and the leaf that produced it.
///
/// Deliberately [`crate::tiers`]'s own arithmetic rather than a second copy: the number a band
/// is compared *against* and the number a run is compared *with* must not be able to disagree
/// about what they measure. It therefore walks every hex-float leaf of the snapshot, a
/// superset of the stock amounts the Python instrument compared — which can only raise the
/// maximum, never lower it, so the basis stays conservative.
///
/// Panics on a shape mismatch: two snapshots of the same run that do not pair leaf-for-leaf
/// mean the harness perturbed the *structure*, which is not what a band bounds.
pub fn worst_relative_deviation(base: &str, perturbed: &str, floor: f64) -> (f64, String) {
    let leaves = tiers::paired_leaves(base, perturbed)
        .unwrap_or_else(|e| panic!("ulp_probe: base and perturbed snapshots do not pair: {e}"));
    let (worst, path) = tiers::max_abs_relative_deviation(&leaves, floor);
    (worst, path.to_string())
}

/// The relative-error floor. One value for the measurement and the contract alike — every
/// Tier-2 row in `rust/data/tiers.json` carries `1e-12`, and so did the Python instrument.
pub const FLOOR: f64 = 1e-12;

fn snapshot(state: &State) -> String {
    simcore::snapshot::from_engine(state).to_json()
}

/// Worst deviation over both nudge directions, given a run parameterised by [`Nudge`].
pub fn worst_over_both_directions(run_at: impl Fn(Nudge) -> String) -> (f64, String) {
    let base = run_at(Nudge::Off);
    let mut worst = 0.0_f64;
    let mut where_ = String::new();
    for nudge in Nudge::BOTH {
        let (deviation, path) = worst_relative_deviation(&base, &run_at(nudge), FLOOR);
        if deviation >= worst {
            worst = deviation;
            where_ = path;
        }
    }
    (worst, where_)
}

// --------------------------------------------------------------------------- //
// The perturbed runs — mirrors of the `crate::goldens` emitters                //
// --------------------------------------------------------------------------- //

/// [`crate::goldens::power`] / [`crate::goldens::power_self_discharge`] with the half-sine
/// `sin` nudged (both the `solar_power` forcing and the `load_power` constant derived from it).
pub fn power_snapshot(with_self_discharge: bool, nudge: Nudge) -> String {
    let charge = params::charge();
    let scenario = BOUNDED_SOC_SCENARIO;
    let self_discharge = with_self_discharge.then(params::self_discharge);
    let (state, registry) = build_power(&charge, &scenario, self_discharge).expect("build_power");
    let resolver = nudged_power_resolver(&charge, &scenario, nudge).expect("power_resolver");
    let integrator = EulerIntegrator::new(registry);
    let days = if with_self_discharge {
        SELF_DISCHARGE_DAYS
    } else {
        BOUNDED_SOC_DAYS
    };
    let (final_state, _, _) = run(
        &integrator,
        state,
        &resolver,
        scenario.dt_seconds,
        days * scenario.steps_per_day,
    )
    .expect("run power");
    snapshot(&final_state)
}

/// [`crate::goldens::thermal`] with the Stefan-Boltzmann `t⁴` nudged.
pub fn thermal_snapshot(nudge: Nudge) -> String {
    let p = params::thermal();
    let scenario = EQUILIBRIUM_SCENARIO;
    let (state, registry) = build_thermal(&p, &scenario).expect("build_thermal");
    let registry = nudge_radiator(registry, &state.stocks, &p, nudge).expect("nudge radiator");
    let resolver = thermal_resolver(&scenario).expect("thermal_resolver");
    let integrator = EulerIntegrator::new(registry);
    let (final_state, _, _) = run(
        &integrator,
        state,
        &resolver,
        scenario.dt_seconds,
        EQUILIBRIUM_STEPS,
    )
    .expect("run thermal");
    snapshot(&final_state)
}

/// [`crate::goldens::perennial_chamber`] / [`crate::goldens::consumer_chamber`] with the
/// Beer-Lambert `exp` nudged through the `par` forcing (see the module header).
pub fn perennial_snapshot(scenario: &SeasonScenario, years: usize, nudge: Nudge) -> String {
    let (state, integrator, resolver) = season_setup(scenario, years).expect("season_setup");
    let resolver = nudge_forcing(resolver, PAR_VAR, nudge).expect("nudge par");
    let mut noop = |_: &State| {};
    let (final_state, _, _) = run_perennial(
        &integrator,
        state,
        scenario,
        &resolver,
        BIO_DT,
        steps_for_years(years),
        season_steps(),
        &mut noop,
    )
    .expect("run_perennial");
    snapshot(&final_state)
}

// --------------------------------------------------------------------------- //
// The measurements                                                             //
// --------------------------------------------------------------------------- //

/// The propagated ±1-ULP `sin` sensitivity of a standalone Power run.
///
/// `with_self_discharge` selects the two Tier-2 power goldens: 7 days without, 14 with.
pub fn power_sensitivity(with_self_discharge: bool) -> (f64, String) {
    worst_over_both_directions(|nudge| power_snapshot(with_self_discharge, nudge))
}

/// The propagated ±1-ULP `t⁴` sensitivity of the standalone Thermal equilibrium run.
pub fn thermal_sensitivity() -> (f64, String) {
    worst_over_both_directions(thermal_snapshot)
}

/// The propagated ±1-ULP Beer-Lambert sensitivity shared by the seven biosphere goldens.
///
/// The worse of the two 15-year sealed runs — the perennial chamber and the consumer chamber
/// — matching the Python instrument's representative pair. The contracting limit cycle barely
/// amplifies one ULP, which is why a band ~3000× above this is still tight enough to catch a
/// port defect.
pub fn biosphere_sensitivity(years: usize) -> (f64, String) {
    let mut worst = 0.0_f64;
    let mut where_ = String::new();
    for scenario in [perennial_chamber_scenario(), consumer_chamber_scenario()] {
        let (deviation, path) =
            worst_over_both_directions(|nudge| perennial_snapshot(&scenario, years, nudge));
        if deviation >= worst {
            worst = deviation;
            where_ = path;
        }
    }
    (worst, where_)
}
