//! The mechanism-switch seam, end to end: a composed registry reaches the **run**.
//!
//! The science-side twin of `value_switch_run.rs`, and it is run **both directions** for the
//! same recorded reason: *"a substituted run differs"* passes if the lab silently returned
//! garbage, *"a baseline run matches"* passes if the seam ignored its argument entirely, and
//! a no-op diff needs a two-direction control.
//!
//! ## ⚠ The two answers here are known BY CONSTRUCTION, and that is the point
//!
//! §8 of the science-switch plan refuses to validate this seam against an archived
//! measurement. The tempting one — the retired big-leaf canopy against the shipped layered
//! scheme — is not available as a control at all: that build moved three things at once, so
//! the repo holds no isolated number for the pair. So the seam is checked against arithmetic
//! instead:
//!
//! * a flow replaced by a [`ScaledMechanism`] at **1.0** reproduces the baseline bit for bit
//!   (`x · 1.0 == x`), and at **0.5** halves its target's legs **exactly** (both factors are
//!   exact in binary floating point);
//! * a flow replaced by a freshly built identical instance reproduces the baseline bit for
//!   bit.
//!
//! ⚠ **Those two are not equally strong, and the docstrings say which is which.** A composer
//! that found the target, dropped it and kept the **original** box passes every no-op test
//! green — the argument never inserted, the run unchanged, nothing to see. Only the scaled
//! replacement can fail against that composer. The no-op control is the "and nothing else
//! moved" half, not the evidence.
//!
//! ## What is NOT here
//!
//! No science pair. The tree holds no second form of any biosphere process (§2C of the plan,
//! measured), and choosing which one to author first is a decision this harness does not
//! take. Every number below is arithmetic on the frozen tree.

use domains::biosphere::params::{self, BiosphereParams};
use domains::biosphere::stocks::{LEAF_C, STEM_C, STORAGE_C};
use domains::biosphere::{
    build_season_with, run_season, season_setup, steps_for_years, weather_resolver, SeasonScenario,
    BIO_DT, DEFAULT_SCENARIO,
};
use domains::lab::mechanism::{
    build_season_adding, build_season_replacing, build_season_without, ScaledMechanism,
};
use simcore::environment::Environment;
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::integrator::EulerIntegrator;
use simcore::registry::Registry;
use simcore::state::State;

/// The subject of the replacement controls: a flow with non-zero legs from the very first
/// step (the sown crop's standing biomass burns immediately), so the halving is measurable at
/// `n = 0` without running a season first.
const MAINTENANCE: &str = "biosphere.maintenance_respiration";
const ROOT_ZONE_CAPTURE: &str = "biosphere.root_zone_capture";
/// One season is enough: every claim here is bit-identity or a first-step leg.
const YEARS: usize = 1;

fn frozen() -> BiosphereParams {
    params::biosphere()
}

/// The whole stock dict, every step — a stronger comparison than one series, and the one a
/// mechanism swap needs: a replaced flow can move a stock the leaf-carbon series never sees.
///
/// Returns `(stock ids, amounts step-major)`; the ids are compared before the amounts, so a
/// run that gained or lost a stock reports that rather than a length mismatch.
fn run_all_stocks(
    state: State,
    registry: Registry,
    scenario: &SeasonScenario,
) -> (Vec<String>, Vec<f64>) {
    let resolver = weather_resolver(scenario, YEARS).expect("resolver");
    let integrator = EulerIntegrator::new(registry);
    let steps = steps_for_years(YEARS);
    let ids: Vec<String> = state.stocks.keys().cloned().collect();
    let mut series: Vec<f64> = Vec::with_capacity((steps + 1) * ids.len());
    {
        let mut observe = |s: &State| {
            for stock in s.stocks.values() {
                series.push(stock.amount);
            }
        };
        let (_final, _rationed, events) = run_season(
            &integrator,
            state,
            &resolver,
            BIO_DT,
            steps,
            None,
            &mut observe,
        )
        .expect("season");
        assert!(events.is_empty(), "unexpected extinction: {events:?}");
    }
    (ids, series)
}

/// The baseline, through the ordinary frozen entry point — no `_with`, no lab.
fn frozen_run(scenario: &SeasonScenario) -> (Vec<String>, Vec<f64>) {
    let (state, integrator, resolver) = season_setup(scenario, YEARS).expect("setup");
    let steps = steps_for_years(YEARS);
    let ids: Vec<String> = state.stocks.keys().cloned().collect();
    let mut series: Vec<f64> = Vec::with_capacity((steps + 1) * ids.len());
    {
        let mut observe = |s: &State| {
            for stock in s.stocks.values() {
                series.push(stock.amount);
            }
        };
        let (_final, _rationed, events) = run_season(
            &integrator,
            state,
            &resolver,
            BIO_DT,
            steps,
            None,
            &mut observe,
        )
        .expect("season");
        assert!(events.is_empty(), "unexpected extinction: {events:?}");
    }
    (ids, series)
}

/// A freshly built instance of `id`, taken out of a second ordinary build — the only way to
/// get one without re-forking the assembly (`compartments` is private to `system.rs`).
fn fresh_flow(scenario: &SeasonScenario, id: &str) -> Box<dyn Flow> {
    let (_, registry) = build_season_with(scenario, &frozen()).expect("build");
    let (flows, _) = registry.into_parts();
    flows
        .into_iter()
        .find(|f| f.id() == id)
        .unwrap_or_else(|| panic!("{id} is not in the build"))
}

fn assert_bit_identical(a: &(Vec<String>, Vec<f64>), b: &(Vec<String>, Vec<f64>), what: &str) {
    assert_eq!(a.0, b.0, "{what}: the stock set moved");
    assert_eq!(a.1.len(), b.1.len(), "{what}: sample count");
    for (i, (x, y)) in a.1.iter().zip(&b.1).enumerate() {
        assert_eq!(
            x.to_bits(),
            y.to_bits(),
            "{what}: sample {i} ({} vs {})",
            x,
            y
        );
    }
}

fn peak(series: &(Vec<String>, Vec<f64>), stock: &str) -> f64 {
    let width = series.0.len();
    let at = series
        .0
        .iter()
        .position(|id| id == stock)
        .unwrap_or_else(|| panic!("{stock} is not in this run"));
    series
        .1
        .iter()
        .skip(at)
        .step_by(width)
        .copied()
        .fold(f64::NEG_INFINITY, f64::max)
}

// --- direction 1: the seam changes nothing when nothing is meant to change -------------

/// ⚠ The **weaker** of the two identity controls, kept and labelled. A composer that ignored
/// its argument and kept the original flow would pass this exactly as well.
#[test]
fn a_no_op_replacement_is_bit_identical_to_the_frozen_path() {
    let base = frozen_run(&DEFAULT_SCENARIO);
    let (state, registry) = build_season_replacing(
        &DEFAULT_SCENARIO,
        &frozen(),
        vec![(MAINTENANCE, fresh_flow(&DEFAULT_SCENARIO, MAINTENANCE))],
    )
    .expect("no-op replacement");
    let composed = run_all_stocks(state, registry, &DEFAULT_SCENARIO);
    assert_bit_identical(&base, &composed, "no-op replacement");
}

/// ⚠⚠ The **evidence**: the argument reached the registry (its `type_name` is the wrapper's,
/// asserted in the module's unit tests) *and* the run is unchanged, because scaling by one is
/// the identity on every leg. A composer that dropped the target without inserting fails this
/// one — the run would be missing a whole process.
#[test]
fn a_scaled_replacement_at_one_is_bit_identical_to_the_frozen_path() {
    let base = frozen_run(&DEFAULT_SCENARIO);
    let wrapped = ScaledMechanism::new(fresh_flow(&DEFAULT_SCENARIO, MAINTENANCE), 1.0);
    let (state, registry) = build_season_replacing(
        &DEFAULT_SCENARIO,
        &frozen(),
        vec![(MAINTENANCE, Box::new(wrapped))],
    )
    .expect("scaled replacement");
    let composed = run_all_stocks(state, registry, &DEFAULT_SCENARIO);
    assert_bit_identical(&base, &composed, "scaled replacement at 1.0");
}

// --- direction 2: a swap reaches the run ------------------------------------------------

/// The halving, at the leg level and on the first step: **exact**, not approximate.
///
/// ⚠ The non-zero assertion is not decoration. A probe measuring a flow that happens to
/// return nothing at the state it is evaluated at reads `0.0 == 0.5 · 0.0` and passes
/// vacuously — this repo has logged exactly that failure in the ULP probe, where a shimmed
/// function the carbon path no longer called measured 0.0 for weeks.
#[test]
fn a_scaled_replacement_halves_its_targets_legs_exactly() {
    let (state, registry) = build_season_with(&DEFAULT_SCENARIO, &frozen()).expect("build");
    let resolver = weather_resolver(&DEFAULT_SCENARIO, YEARS).expect("resolver");
    let env = resolver.bind(&state, BIO_DT);

    let base = registry
        .flows()
        .iter()
        .find(|f| f.id() == MAINTENANCE)
        .expect("the target is in the baseline registry");
    let base_legs = base.evaluate(&state, &env, BIO_DT).expect("evaluate");
    assert!(
        base_legs.legs.iter().any(|l| l.amount != 0.0),
        "{MAINTENANCE} returns nothing at the initial state — the control would pass vacuously"
    );

    let scaled = ScaledMechanism::new(fresh_flow(&DEFAULT_SCENARIO, MAINTENANCE), 0.5);
    let scaled_legs = scaled.evaluate(&state, &env, BIO_DT).expect("evaluate");

    assert_eq!(scaled_legs.legs.len(), base_legs.legs.len());
    for (b, s) in base_legs.legs.iter().zip(&scaled_legs.legs) {
        assert_eq!(b.stock, s.stock, "the wrapper reordered the legs");
        assert_eq!(
            (b.amount * 0.5).to_bits(),
            s.amount.to_bits(),
            "leg on {}: {} is not half of {}",
            b.stock,
            s.amount,
            b.amount
        );
    }
    assert_eq!(scaled.id(), MAINTENANCE, "the wrapper must keep the slot");
    assert_eq!(scaled.type_name(), "ScaledMechanism");
}

/// And the halved flow moves the **run** — the other half of the two-direction control.
///
/// ⚠ The assertion is on the **direction**, never a value: maintenance respiration burns
/// standing biomass, so burning half as much leaves more carbon in the crop and the season's
/// peak leaf carbon rises. Pinning the number would freeze an experimental result into a
/// test, which is the thing this harness exists to avoid.
#[test]
fn a_halved_mechanism_moves_the_run_in_the_direction_the_science_says() {
    let base = frozen_run(&DEFAULT_SCENARIO);
    let wrapped = ScaledMechanism::new(fresh_flow(&DEFAULT_SCENARIO, MAINTENANCE), 0.5);
    let (state, registry) = build_season_replacing(
        &DEFAULT_SCENARIO,
        &frozen(),
        vec![(MAINTENANCE, Box::new(wrapped))],
    )
    .expect("scaled replacement");
    let halved = run_all_stocks(state, registry, &DEFAULT_SCENARIO);

    assert_eq!(base.0, halved.0, "the stock set moved");
    assert!(
        base.1
            .iter()
            .zip(&halved.1)
            .any(|(a, b)| a.to_bits() != b.to_bits()),
        "the halved run is bit-identical to the baseline — the replacement never reached the \
         run"
    );
    assert!(
        peak(&halved, LEAF_C) > peak(&base, LEAF_C),
        "peak leaf carbon fell with half the maintenance respiration: {} vs {}",
        peak(&halved, LEAF_C),
        peak(&base, LEAF_C)
    );
}

/// The **drop** side reaches the run too — slice 1 shipped with registry-shape tests only.
#[test]
fn a_knockout_moves_the_run() {
    let base = frozen_run(&DEFAULT_SCENARIO);
    let (state, registry) =
        build_season_without(&DEFAULT_SCENARIO, &frozen(), &[ROOT_ZONE_CAPTURE]).expect("drop");
    let without = run_all_stocks(state, registry, &DEFAULT_SCENARIO);
    assert_eq!(base.0, without.0, "a drop must not move the stock set");
    assert!(
        base.1
            .iter()
            .zip(&without.1)
            .any(|(a, b)| a.to_bits() != b.to_bits()),
        "dropping {ROOT_ZONE_CAPTURE} changed nothing — the knockout never reached the run"
    );
}

// --- the addition, over the frozen stock set -------------------------------------------

/// A test-local added process: a small carbon transfer between two organs the season already
/// carries.
///
/// Balanced by construction (one carbon stock down, another up by the same amount), which is
/// what lets it run at all — every flow is internally balanced and conservation is asserted
/// every step. ⚠ It is a **redistribution**, and this tree has recorded that a redistribution
/// is invisible to conservation by construction: the run differing is the evidence it
/// arrived, not the conservation check passing.
struct LabTransfer;

impl Flow for LabTransfer {
    fn type_name(&self) -> &'static str {
        "LabTransfer"
    }
    fn id(&self) -> &str {
        "biosphere.lab_transfer"
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let moved = 1e-4 * snapshot.stocks[STORAGE_C].amount * dt;
        FlowResult::new(vec![
            Leg::new(STORAGE_C.to_string(), -moved)?,
            Leg::new(STEM_C.to_string(), moved)?,
        ])
    }
}

/// An added process reaches the run. The composer's other half — a flow the frozen build does
/// not carry at all — with the same two-direction discipline: the baseline above is the
/// unchanged side.
#[test]
fn an_addition_reaches_the_run() {
    let base = frozen_run(&DEFAULT_SCENARIO);
    let (state, registry) =
        build_season_adding(&DEFAULT_SCENARIO, &frozen(), vec![Box::new(LabTransfer)])
            .expect("addition");
    let with_extra = run_all_stocks(state, registry, &DEFAULT_SCENARIO);
    assert_eq!(
        base.0, with_extra.0,
        "a flow-only addition must not move the stock set"
    );
    assert!(
        base.1
            .iter()
            .zip(&with_extra.1)
            .any(|(a, b)| a.to_bits() != b.to_bits()),
        "the added flow changed nothing — it never reached the run"
    );
    assert!(
        peak(&with_extra, STEM_C) > peak(&base, STEM_C),
        "the transfer moved carbon out of the stem it deposits into"
    );
}
