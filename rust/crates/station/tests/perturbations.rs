//! Phase-8 (P8.5) cross-domain perturbation cascades — the Rust port of the behavioural
//! asserts in `tests/test_station_perturbations.py`.
//!
//! Each perturbation is a **cascade with no cascade code**: a disturbance in one domain
//! propagates into another through a shared stock / shared forcing (#16), while conservation
//! holds (a completed run *is* the every-step proof — the gate lives in `step_report` /
//! the master-day driver) and `rationed` *behaves*. Direction-only asserts (never a
//! magnitude / `State == State` on a healthy run), each a perturbed-vs-baseline contrast.
//! No golden (the "diagnostics, no golden" precedent); the determinism re-runs are the
//! insurance.
//!
//! Two substrates: **energy** perturbations on the cheap single-rate diurnal station
//! (brownout, radiator failure); **matter** perturbations on a short two-rate sealed station
//! (leak, crew spike, lighting failure) with the window inside year 1 so the annual reset
//! never fires. The station regulators *erase* the pool level (P6.8), so matter cascades
//! assert regulator **effort** + sinks, not pool level.

use std::sync::OnceLock;

use domains::biosphere::perturbations::LEAK_SINK;
use domains::biosphere::stocks::{CARBON_POOL, LEAF_C, O2_POOL, ROOT_C, STEM_C, STORAGE_C};
use domains::crew::FOOD_STORE;
use domains::eclss::{CO2_REMOVED, O2_SUPPLY};
use domains::power::BATTERY;
use domains::thermal::{ThermalParams, NODE, SPACE};

use simcore::integrator::EulerIntegrator;
use simcore::state::State;

use station::driver::run_master_day;
use station::perturbations::{
    with_brownout, with_crew_load_spike, with_lighting_failure, with_radiator_failure,
    with_station_leak,
};
use station::scenario::{SealedStationScenario, HEAT_CLOSURE_SCENARIO};
use station::sealed::{build_sealed_station, sealed_bio_resolver, sealed_fast_resolver};
use station::system::{build_station, station_resolver};

// ============================ ENERGY: single-rate run_station ========================

const E_DAYS: u64 = 12;

fn e_spd() -> u64 {
    HEAT_CLOSURE_SCENARIO.power.steps_per_day
}
fn e_dt() -> f64 {
    HEAT_CLOSURE_SCENARIO.power.dt_seconds
}

fn node_temp(state: &State, thermal: &ThermalParams) -> f64 {
    thermal.space_temperature + state.stocks[NODE].amount / thermal.heat_capacity
}

/// Build the baseline Power → Thermal diurnal trajectory + the loaded params.
fn energy_baseline() -> (Vec<State>, ThermalParams) {
    let charge = domains::params::charge();
    let thermal = domains::params::thermal();
    let (state, reg) = build_station(&charge, &thermal, &HEAT_CLOSURE_SCENARIO, None).unwrap();
    let resolver = station_resolver(&charge, &HEAT_CLOSURE_SCENARIO).unwrap();
    let integ = EulerIntegrator::new(reg);
    let mut states = Vec::new();
    let (_f, rationed, events) = station::run_station(
        &integ,
        state,
        &resolver,
        e_dt(),
        E_DAYS * e_spd(),
        &mut |s| states.push(s.clone()),
    )
    .unwrap();
    assert_eq!(rationed, 0);
    assert!(events.is_empty());
    (states, thermal)
}

#[test]
fn brownout_graceful_cools_node_without_rationing() {
    let (base_states, thermal) = energy_baseline();
    let charge = domains::params::charge();
    let (state, reg) = build_station(&charge, &thermal, &HEAT_CLOSURE_SCENARIO, None).unwrap();
    let resolver = station_resolver(&charge, &HEAT_CLOSURE_SCENARIO).unwrap();
    // A short/shallow afternoon dimming: SOC dips below baseline and the node cools, no
    // rationing (the Power→Thermal cascade, graceful arm).
    let perturbed = with_brownout(resolver, 85, 90, 0.5).unwrap();
    let integ = EulerIntegrator::new(reg);
    let mut states = Vec::new();
    let (_f, rationed, events) = station::run_station(
        &integ,
        state,
        &perturbed,
        e_dt(),
        E_DAYS * e_spd(),
        &mut |s| states.push(s.clone()),
    )
    .unwrap();
    assert_eq!(rationed, 0);
    assert!(events.is_empty());

    let base_soc_min = base_states.iter().map(|s| s.stocks[BATTERY].amount).fold(f64::INFINITY, f64::min);
    let soc_min = states.iter().map(|s| s.stocks[BATTERY].amount).fold(f64::INFINITY, f64::min);
    assert!(soc_min < base_soc_min && soc_min > 0.0, "SOC dips but never empties");

    let base_t_min = base_states.iter().map(|s| node_temp(s, &thermal)).fold(f64::INFINITY, f64::min);
    let t_min = states.iter().map(|s| node_temp(s, &thermal)).fold(f64::INFINITY, f64::min);
    assert!(t_min < base_t_min, "node cools — the cross-domain cascade");
}

#[test]
fn brownout_deep_emerges_rationing_still_conserving() {
    // The FAILURE cascade: a multi-day full blackout empties the battery so LoadDraw cannot
    // be met → rationed > 0 EMERGES (bounded), yet ENERGY still conserves (a completed run
    // is the every-step proof — the Euler backstop conserves as it rations).
    let charge = domains::params::charge();
    let thermal = domains::params::thermal();
    let (state, reg) = build_station(&charge, &thermal, &HEAT_CLOSURE_SCENARIO, None).unwrap();
    let resolver = station_resolver(&charge, &HEAT_CLOSURE_SCENARIO).unwrap();
    let perturbed = with_brownout(resolver, 2 * e_spd(), 8 * e_spd(), 0.0).unwrap();
    let integ = EulerIntegrator::new(reg);
    let (_f, rationed, events) = station::run_station(
        &integ,
        state,
        &perturbed,
        e_dt(),
        E_DAYS * e_spd(),
        &mut |_| {},
    )
    .unwrap();
    assert!(rationed > 0, "the emergent failure signature");
    assert!(rationed < E_DAYS * e_spd(), "bounded (not every step rations)");
    assert!(events.is_empty());
}

#[test]
fn radiator_failure_heats_node_conserving_no_rationing() {
    let (base_states, thermal) = energy_baseline();
    let charge = domains::params::charge();
    let (state, reg) = build_station(&charge, &thermal, &HEAT_CLOSURE_SCENARIO, None).unwrap();
    let resolver = station_resolver(&charge, &HEAT_CLOSURE_SCENARIO).unwrap();
    // Throttle RadiatorReject to 0 over a window → Power's real dissipation piles up in the
    // node → it HEATS (peak T above baseline). Energy stays in-system (conserved); the sink
    // stays monotonic; rationed == 0 (a POOL accumulation, not a withdrawal shortfall).
    let (preg, pres) =
        with_radiator_failure(&state, reg, resolver, 3 * e_spd(), 9 * e_spd(), 0.0).unwrap();
    let integ = EulerIntegrator::new(preg);
    let mut states = Vec::new();
    let (_f, rationed, events) = station::run_station(
        &integ,
        state,
        &pres,
        e_dt(),
        E_DAYS * e_spd(),
        &mut |s| states.push(s.clone()),
    )
    .unwrap();
    assert_eq!(rationed, 0);
    assert!(events.is_empty());

    let base_t_max = base_states.iter().map(|s| node_temp(s, &thermal)).fold(f64::NEG_INFINITY, f64::max);
    let t_max = states.iter().map(|s| node_temp(s, &thermal)).fold(f64::NEG_INFINITY, f64::max);
    assert!(t_max > base_t_max, "node overheats — the cross-domain cascade");
    // The space sink is monotonic (heat never un-radiates).
    for w in states.windows(2) {
        assert!(w[1].stocks[SPACE].amount >= w[0].stocks[SPACE].amount - 1e-6);
    }
}

#[test]
fn inspection_is_truthful_under_deep_brownout_rationing() {
    // The P8.4 seam discharged on a REAL palette scenario (advisor): a deep brownout empties
    // the battery so LoadDraw rations (`scale < 1`) on the single-rate `station` entry — the
    // reachable rationing case. At every step, inspect the pre-step state and confirm
    // `before + applied_delta == after` for every stock (the scale-aware identity holds under
    // rationing, unlike the raw-leg sum), and that rationing actually occurred.
    let charge = domains::params::charge();
    let thermal = domains::params::thermal();
    let (mut state, reg) = build_station(&charge, &thermal, &HEAT_CLOSURE_SCENARIO, None).unwrap();
    let resolver = station_resolver(&charge, &HEAT_CLOSURE_SCENARIO).unwrap();
    let perturbed = with_brownout(resolver, 2 * e_spd(), 8 * e_spd(), 0.0).unwrap();
    let integ = EulerIntegrator::new(reg);

    let mut saw_rationing = false;
    for _ in 0..(E_DAYS * e_spd()) {
        let insp =
            station::inspection::inspect_flows(integ.registry(), &state, &perturbed, e_dt())
                .unwrap();
        let report = integ.step_report(&state, &perturbed, e_dt()).unwrap();
        if report.rationed > 0 {
            saw_rationing = true;
            // A rationed step must show at least one throttled flow in the inspection.
            assert!(insp.flows.iter().any(|f| f.scale < 1.0));
        }
        for (sid, before) in &state.stocks {
            let after = report.state.stocks[sid].amount;
            let applied = insp.applied_delta(sid);
            assert!(
                (before.amount + applied - after).abs() <= 1e-9 * after.abs() + 1e-9,
                "inspection lied about {sid} under rationing: {} + {applied} != {after}",
                before.amount
            );
        }
        state = report.state;
    }
    assert!(saw_rationing, "deep brownout should drive LoadDraw into rationing");
}

#[test]
fn radiator_failure_outside_window_is_baseline() {
    // The ScaledFlow bit-identity: health == 1 outside the window reproduces the wrapped flow
    // EXACTLY (x·1.0 == x), so a failure window the run never reaches gives the byte-identical
    // baseline trajectory.
    let (base_states, thermal) = energy_baseline();
    let charge = domains::params::charge();
    let (state, reg) = build_station(&charge, &thermal, &HEAT_CLOSURE_SCENARIO, None).unwrap();
    let resolver = station_resolver(&charge, &HEAT_CLOSURE_SCENARIO).unwrap();
    let (preg, pres) =
        with_radiator_failure(&state, reg, resolver, 100 * e_spd(), 101 * e_spd(), 0.0).unwrap();
    let integ = EulerIntegrator::new(preg);
    let (final_state, _r, _e) = station::run_station(
        &integ,
        state,
        &pres,
        e_dt(),
        E_DAYS * e_spd(),
        &mut |_| {},
    )
    .unwrap();
    assert_eq!(&final_state, base_states.last().unwrap());
}

// ============================ MATTER: short two-rate run_master_day ===================

const M_DAYS: usize = 8;
const M_START: u64 = 2;
const M_END: u64 = 7; // window (master days) — inside year 1
const K_LEAK: f64 = 1.0e-3;

// season_days >> the run horizon ⇒ the annual reset never fires (window stays in year 1).
fn matter_scenario() -> SealedStationScenario {
    SealedStationScenario {
        years: 1,
        season_days: 305,
        ..station::scenario::sealed_station_scenario()
    }
}

fn biomass(state: &State) -> f64 {
    [LEAF_C, STEM_C, ROOT_C, STORAGE_C]
        .iter()
        .map(|s| state.stocks[*s].amount)
        .sum()
}

/// Build the sealed pieces (`false, false` — the palette `sealed` config, no harvest / open
/// feces), matching the session-parity conventions.
fn sealed_build() -> (State, simcore::registry::Registry, simcore::registry::Registry) {
    let scn = matter_scenario();
    build_sealed_station(
        &domains::params::charge(),
        &domains::params::thermal(),
        &domains::params::crew(),
        &domains::params::eclss(),
        &station::params::water_recovery(),
        &station::params::lamp(),
        &station::params::harvest(),
        &scn,
        false,
        false,
    )
    .unwrap()
}

fn sealed_run(
    state: State,
    bio_reg: simcore::registry::Registry,
    fast_reg: simcore::registry::Registry,
    bio_res: &simcore::environment::SourceResolver,
    fast_res: &simcore::environment::SourceResolver,
) -> (Vec<State>, u64, Vec<simcore::events::Event>) {
    let scn = matter_scenario();
    run_master_day(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(fast_reg),
        state,
        bio_res,
        fast_res,
        M_DAYS,
        scn.steps_per_day,
        scn.bio_dt,
        scn.cabin_dt,
        None,
    )
    .unwrap()
}

/// The unperturbed short sealed final state — cached so each matter cascade pays one run.
fn sealed_baseline() -> &'static State {
    static BASE: OnceLock<State> = OnceLock::new();
    BASE.get_or_init(|| {
        let scn = matter_scenario();
        let (state, bio_reg, fast_reg) = sealed_build();
        let bio_res = sealed_bio_resolver(&station::params::lamp(), &scn).unwrap();
        let fast_res = sealed_fast_resolver(&domains::params::charge(), &scn).unwrap();
        let (states, rationed, events) =
            sealed_run(state, bio_reg, fast_reg, &bio_res, &fast_res);
        assert_eq!(rationed, 0);
        assert!(events.is_empty());
        states.last().unwrap().clone()
    })
}

fn run_carbon_leak(pool: &str) -> State {
    let scn = matter_scenario();
    let (state, bio_reg, fast_reg) = sealed_build();
    let fast_res = sealed_fast_resolver(&domains::params::charge(), &scn).unwrap();
    let bio_res = sealed_bio_resolver(&station::params::lamp(), &scn).unwrap();
    let (state, bio_reg, fast_reg, fast_res) = with_station_leak(
        &state, bio_reg, fast_reg, fast_res, pool, K_LEAK, M_START, M_END,
    )
    .unwrap();
    let (states, rationed, events) = sealed_run(state, bio_reg, fast_reg, &bio_res, &fast_res);
    assert_eq!(rationed, 0);
    assert!(events.is_empty());
    states.last().unwrap().clone()
}

#[test]
fn carbon_leak_lowers_biomass_and_scrubber_effort() {
    // CARBON_POOL is only-removed (the scrubber cannot push it up): the leak lowers Ci within
    // the window, so the plant assimilates LESS (biomass below baseline) AND the scrubber does
    // LESS work (leaked CO₂ was not there to remove). The leak-sink strictly accumulates.
    let baseline = sealed_baseline();
    let leaked = run_carbon_leak(CARBON_POOL);
    assert!(biomass(&leaked) < biomass(baseline));
    assert!(leaked.stocks[CO2_REMOVED].amount < baseline.stocks[CO2_REMOVED].amount);
    assert!(leaked.stocks[LEAK_SINK].amount > 0.0);
}

#[test]
fn o2_leak_is_absorbed_by_makeup_effort() {
    // O2_POOL is DEFENDED (O2Makeup is demand-controlled), so — unlike CARBON — the leak
    // surfaces as makeup EFFORT, not a pool/biology change: o2_supply supplies strictly MORE
    // (its cumulative bookkeeping runs further negative), the plant is UNTOUCHED, and the
    // leak-sink accumulates. The two pools fail differently.
    let baseline = sealed_baseline();
    let leaked = run_carbon_leak(O2_POOL);
    assert!(leaked.stocks[O2_SUPPLY].amount < baseline.stocks[O2_SUPPLY].amount);
    let rel = (biomass(&leaked) - biomass(baseline)).abs() / biomass(baseline).abs();
    assert!(rel < 1e-6, "biomass ≈ baseline (rel {rel})");
    assert!(leaked.stocks[LEAK_SINK].amount > 0.0);
}

#[test]
fn crew_spike_raises_regulator_effort_and_drains_food() {
    // A doubled food intake drives more respiration → BOTH regulators work harder (co2_removed
    // up AND o2_supply further negative) and food_store depletes faster. The gas pools are held
    // at setpoint by the regulators, so the signature is EFFORT, not level.
    let baseline = sealed_baseline();
    let scn = matter_scenario();
    let (state, bio_reg, fast_reg) = sealed_build();
    let bio_res = sealed_bio_resolver(&station::params::lamp(), &scn).unwrap();
    let fast_res = sealed_fast_resolver(&domains::params::charge(), &scn).unwrap();
    let fast_res = with_crew_load_spike(fast_res, M_START, M_END, 2.0).unwrap();
    let (states, rationed, events) = sealed_run(state, bio_reg, fast_reg, &bio_res, &fast_res);
    assert_eq!(rationed, 0);
    assert!(events.is_empty());
    let spiked = states.last().unwrap();

    assert!(spiked.stocks[CO2_REMOVED].amount > baseline.stocks[CO2_REMOVED].amount);
    assert!(spiked.stocks[O2_SUPPLY].amount < baseline.stocks[O2_SUPPLY].amount);
    assert!(spiked.stocks[FOOD_STORE].amount < baseline.stocks[FOOD_STORE].amount);
    // Regulator-erasure: the day-boundary pools return to the SAME setpoint as baseline.
    for pool in [CARBON_POOL, O2_POOL] {
        let rel = (spiked.stocks[pool].amount - baseline.stocks[pool].amount).abs()
            / baseline.stocks[pool].amount.abs();
        assert!(rel < 1e-6, "{pool} returns to setpoint (rel {rel})");
    }
}

#[test]
fn lighting_failure_stalls_growth_and_spares_battery() {
    // The #16 lamp is ONE device with two legs: cutting PAR (a forcing) stalls growth (biomass
    // below baseline), and cutting the lamp draw (the Lamp flow's energy) SPARES the battery
    // (it drains slower ⇒ higher SOC). One intervention, a cascade in both directions.
    let baseline = sealed_baseline();
    let scn = matter_scenario();
    let (state, bio_reg, fast_reg) = sealed_build();
    let bio_res = sealed_bio_resolver(&station::params::lamp(), &scn).unwrap();
    let fast_res = sealed_fast_resolver(&domains::params::charge(), &scn).unwrap();
    let (bio_res, fast_res) = with_lighting_failure(bio_res, fast_res, M_START, M_END).unwrap();
    let (states, rationed, events) = sealed_run(state, bio_reg, fast_reg, &bio_res, &fast_res);
    assert_eq!(rationed, 0);
    assert!(events.is_empty());
    let failed = states.last().unwrap();

    assert!(biomass(failed) < biomass(baseline), "growth stalls");
    assert!(
        failed.stocks[BATTERY].amount > baseline.stocks[BATTERY].amount,
        "battery spared"
    );
    // Regulator-erasure: the scrubber holds CARBON_POOL at setpoint despite less assimilation.
    let rel = (failed.stocks[CARBON_POOL].amount - baseline.stocks[CARBON_POOL].amount).abs()
        / baseline.stocks[CARBON_POOL].amount.abs();
    assert!(rel < 1e-6, "carbon pool returns to setpoint (rel {rel})");
}

#[test]
fn matter_perturbation_is_deterministic() {
    // The no-golden insurance (matter side): a perturbed two-rate sealed run is bit-identical
    // on re-run (stands in for the absent golden).
    let a = run_carbon_leak(CARBON_POOL);
    let b = run_carbon_leak(CARBON_POOL);
    assert_eq!(a, b);
}
