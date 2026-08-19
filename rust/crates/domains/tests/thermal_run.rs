//! The standalone Thermal **run** — equilibrium-temperature validation (Stage-3 slice S3).
//!
//! Subject: `tests/test_thermal_run.py`'s 15 cases.
//!
//! **Honest framing, and the contrast with Power.** Power's two flows are both *forced*,
//! so its SOC is a restoring-force-free accumulator whose boundedness had to be
//! *constructed*. Thermal is genuinely different: `RadiatorReject` is donor-controlled and
//! **nonlinear** (`T⁴`), so there is a real restoring force and a genuine emergent
//! equilibrium temperature — any constant load lands there, with no tuning. What that
//! buys is a convergence claim and a contraction claim that are not available on Power,
//! and it costs the forced-only RK4 ≡ Euler bit-identity, which becomes a tolerance
//! agreement here.
//!
//! ⚠ **Two closed forms are re-derived in this file rather than imported.**
//! `relaxation_time` has no Rust twin, and `radiated_power` is private to the crate. §5v
//! classifies both as *closed form* and forbids transcribing a Python-produced number, so
//! each is written out from its own algebra: `τ = C/(4εσA·T_eq³)` and
//! `R = εσA(T⁴ − T_space⁴)`. Their subject, `equilibrium_temperature`, is public and is
//! what the assertions actually pin.
//!
//! Helpers are duplicated from the sibling `*_run.rs` files by the house rule stated in
//! `power_run.rs`.

use std::collections::BTreeMap;

use domains::params;
use domains::thermal::{
    build_thermal, equilibrium_temperature, temperature, thermal_resolver, HeatInput,
    ThermalParams, ThermalScenario, EQUILIBRIUM_SCENARIO, EQUILIBRIUM_STEPS, HEAT_INPUT,
    HEAT_SOURCE, NODE, SPACE, STEFAN_BOLTZMANN,
};
use domains::{run_trajectory, StepIntegrator};
use simcore::conservation::compute_ledger;
use simcore::events::Event;
use simcore::flow::Flow;
use simcore::integrator::{EulerIntegrator, Rk4Integrator};
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::State;

const STEPS: u64 = EQUILIBRIUM_STEPS;
const DT: f64 = EQUILIBRIUM_SCENARIO.dt_seconds;

/// The emergent equilibrium temperature for the committed radiator under the scenario load.
fn t_eq() -> f64 {
    equilibrium_temperature(&params::thermal(), EQUILIBRIUM_SCENARIO.heat_load_w)
}

/// The stored heat that puts the node at `t` — `Q = C·(T − T_space)`.
fn heat_at(params: &ThermalParams, t: f64) -> f64 {
    params.heat_capacity * (t - params.space_temperature)
}

/// `R = ε·σ·A·(T⁴ − T_space⁴)` (W) — the Stefan-Boltzmann law re-derived here because
/// `thermal::radiated_power` is private to the crate (§5v, the closed-form rule).
fn radiated(params: &ThermalParams, node_joules: f64) -> f64 {
    let t = temperature(node_joules, params.heat_capacity, params.space_temperature);
    params.emissivity
        * STEFAN_BOLTZMANN
        * params.radiator_area
        * (t.powf(4.0) - params.space_temperature.powf(4.0))
}

/// `τ = C / (4·ε·σ·A·T_eq³)` (s) — the linearized relaxation time near equilibrium, the
/// e-folding time of a small perturbation about `T_eq`. It has no Rust twin, so it is
/// re-derived from the derivative of the `T⁴` rejection rather than imported.
fn relaxation_time(params: &ThermalParams, heat_load_w: f64) -> f64 {
    let t = equilibrium_temperature(params, heat_load_w);
    params.heat_capacity
        / (4.0 * params.emissivity * STEFAN_BOLTZMANN * params.radiator_area * t.powi(3))
}

fn run<F>(scenario: &ThermalScenario, make: F) -> (Vec<State>, u64, Vec<Event>)
where
    F: FnOnce(Registry) -> Box<dyn StepIntegrator>,
{
    let params = params::thermal();
    let (state, registry) = build_thermal(&params, scenario).expect("build");
    let resolver = thermal_resolver(scenario).expect("resolver");
    let integrator = make(registry);
    run_trajectory(
        integrator.as_ref(),
        state,
        &resolver,
        scenario.dt_seconds,
        STEPS,
    )
    .expect("run")
}

fn euler(scenario: &ThermalScenario) -> (Vec<State>, u64, Vec<Event>) {
    run(scenario, |r| Box::new(EulerIntegrator::new(r)))
}

fn node_temps(params: &ThermalParams, states: &[State]) -> Vec<f64> {
    states
        .iter()
        .map(|s| {
            temperature(
                s.stocks[NODE].amount,
                params.heat_capacity,
                params.space_temperature,
            )
        })
        .collect()
}

/// The augmented-system ENERGY total: the unclamped source + the node POOL + the
/// monotonic space sink.
fn energy_total(state: &State) -> f64 {
    state.stocks[HEAT_SOURCE].amount + state.stocks[NODE].amount + state.stocks[SPACE].amount
}

fn bits(state: &State) -> BTreeMap<&str, u64> {
    state
        .stocks
        .iter()
        .map(|(id, s)| (id.as_str(), s.amount.to_bits()))
        .collect()
}

/// Assert two states are identical: every stock's amount **bit for bit**, then the whole
/// `State`.
///
/// ⚠ Both halves, deliberately, and the second one is honest about being weak **today**.
/// `to_bits()` is the stricter float comparison — `PartialEq` treats `+0.0 == -0.0` — but it
/// reads only the amounts, while the Python case this ports compared the whole `State`.
/// `State` also carries `n`, `rng_seed` and `aux`; on the four siblings those are always the
/// step count, 0, and empty, so **given the bit comparison passes, the `State` comparison
/// cannot currently fail**. It is here because the ported claim is about the whole `State`
/// and because it starts biting the moment a sibling gains an aux process or an RNG draw —
/// not because it is measuring anything now. Said out loud rather than left to read as
/// coverage.
fn assert_same_state(a: &State, b: &State) {
    assert_eq!(bits(a), bits(b), "stock amounts differ bit for bit");
    assert_eq!(a, b, "the States differ outside their stock amounts");
}

// --- the payload: ENERGY conserved every step over the augmented system ---------------
#[test]
fn thermal_energy_conserved_every_step() {
    // Energy closure carried by a NONLINEAR radiator, not just Power's forced flows.
    let (states, _, _) = euler(&EQUILIBRIUM_SCENARIO);
    for pair in states.windows(2) {
        let ledger = compute_ledger(&pair[0], &pair[1]).expect("ledger");
        let energy = ledger
            .iter()
            .find(|q| q.quantity == Quantity::Energy)
            .expect("ENERGY is present");
        assert!(energy.residual.abs() <= 1e-6, "{:?}", energy.residual);
    }
}

#[test]
fn thermal_energy_total_is_invariant() {
    // Heat moves source → node → space; none vanishes.
    let (states, _, _) = euler(&EQUILIBRIUM_SCENARIO);
    let total0 = energy_total(&states[0]);
    assert!((total0 - EQUILIBRIUM_SCENARIO.node0).abs() < 1e-9);
    for s in &states {
        assert!((energy_total(s) - total0).abs() <= 1e-4);
    }
}

#[test]
fn thermal_only_energy_is_present() {
    let (states, _, _) = euler(&EQUILIBRIUM_SCENARIO);
    let ledger = compute_ledger(&states[0], &states[1]).expect("ledger");
    let quantities: Vec<Quantity> = ledger.iter().map(|q| q.quantity).collect();
    assert_eq!(quantities, vec![Quantity::Energy]);
}

// --- rationed == 0 / no events: well-fed sizing (τ >> dt) ------------------------------
#[test]
fn thermal_relaxation_time_is_many_steps() {
    // The load-bearing sizing constraint: τ >> dt is what keeps Euler from overshooting
    // the nonlinear radiator, so `rationed == 0` holds by SIZING and not structurally.
    let tau = relaxation_time(&params::thermal(), EQUILIBRIUM_SCENARIO.heat_load_w);
    assert!(tau / DT > 20.0, "τ/dt = {}", tau / DT);
}

#[test]
fn thermal_never_rations() {
    let (_, rationed, _) = euler(&EQUILIBRIUM_SCENARIO);
    assert_eq!(rationed, 0);
}

#[test]
fn thermal_no_events() {
    let (_, _, events) = euler(&EQUILIBRIUM_SCENARIO);
    assert!(events.is_empty(), "{events:?}");
}

// --- the emergent equilibrium temperature (the genuine attractor) ----------------------
#[test]
fn thermal_equilibrium_balances_radiation_against_load() {
    // The defining identity of T_eq: at T_eq the radiated power equals the forced load,
    // i.e. `equilibrium_temperature` really solves εσA(T_eq⁴ − T_space⁴) = heat_load. Both
    // sides are written out here from the physics, so the test can disagree with the code.
    let params = params::thermal();
    let q_eq = heat_at(&params, t_eq());
    let r = radiated(&params, q_eq);
    assert!(
        (r - EQUILIBRIUM_SCENARIO.heat_load_w).abs() <= 1e-9 * EQUILIBRIUM_SCENARIO.heat_load_w,
        "{r} != {}",
        EQUILIBRIUM_SCENARIO.heat_load_w
    );
}

#[test]
fn thermal_warms_monotonically_from_cold() {
    // From node0 = 0 (T = T_space) the node warms monotonically — input exceeds rejection
    // all the way up, since rejection only rises toward the load. A monotone approach, not
    // a periodic swing: there is no diurnal forcing here.
    let params = params::thermal();
    let (states, _, _) = euler(&EQUILIBRIUM_SCENARIO);
    let temps = node_temps(&params, &states);
    assert!((temps[0] - params.space_temperature).abs() < 1e-9);
    for pair in temps.windows(2) {
        assert!(pair[0] <= pair[1] + 1e-9, "{:?} > {:?}", pair[0], pair[1]);
    }
}

#[test]
fn thermal_converges_to_equilibrium_temperature() {
    // After ~11 τ the node sits within a narrow band of the emergent T_eq — a genuine
    // attractor, unlike Power's constructed balance. The tiny residual gap is the T⁴
    // radiator being near-inert while cold.
    let params = params::thermal();
    let (states, _, _) = euler(&EQUILIBRIUM_SCENARIO);
    let temps = node_temps(&params, &states);
    let final_t = temps[temps.len() - 1];
    let target = t_eq();
    assert!((final_t - target).abs() < 0.5, "{final_t} vs {target}");
    // And it genuinely climbed most of the way — a real approach, not a flat line.
    assert!(final_t - params.space_temperature > 0.9 * (target - params.space_temperature));
}

// --- the restoring force: two runs contract (monotone, not geometric) ------------------
#[test]
fn thermal_two_runs_contract_to_the_attractor() {
    // Two runs differing ONLY in node0, one below equilibrium and one above. Identical
    // forcing ⇒ the HeatInput legs cancel in the difference, leaving only the radiator's.
    // The nonlinear restoring force pulls them together: |d_n| decreases MONOTONICALLY —
    // not by the exact geometric law SelfDischarge had, because T⁴ is nonlinear — and ends
    // far smaller than it began.
    let params = params::thermal();
    let q_eq = heat_at(&params, t_eq());
    let (cold, _, _) = euler(&EQUILIBRIUM_SCENARIO);
    let hot_scenario = ThermalScenario {
        node0: 2.0 * q_eq,
        ..EQUILIBRIUM_SCENARIO
    };
    let (hot, _, _) = euler(&hot_scenario);
    let diff: Vec<f64> = cold
        .iter()
        .zip(hot.iter())
        .map(|(c, h)| (h.stocks[NODE].amount - c.stocks[NODE].amount).abs())
        .collect();
    // Non-increasing (`<=`, not `<`): if the horizon were ever raised until both runs land
    // on the same floating-point fixed point, d_n → 0 and a strict `<` would fail though
    // the physics is fine. The "ended much smaller" check below carries the real claim.
    for pair in diff.windows(2) {
        assert!(pair[1] <= pair[0], "{:?} > {:?}", pair[1], pair[0]);
    }
    assert!(diff[diff.len() - 1] < 0.01 * diff[0]);
}

#[test]
fn thermal_without_radiator_difference_is_constant() {
    // The contrast that makes the contraction meaningful: with the radiator removed (only
    // the forced HeatInput), there is NO restoring force, so a node0 offset propagates
    // undecayed — d_n == d_0 for every n. The radiator is exactly what turns this constant
    // into a contraction.
    let params = params::thermal();
    let other = ThermalScenario {
        node0: EQUILIBRIUM_SCENARIO.node0 + 1.0e9,
        ..EQUILIBRIUM_SCENARIO
    };
    let (state_a, _) = build_thermal(&params, &EQUILIBRIUM_SCENARIO).expect("build a");
    let (state_b, _) = build_thermal(&params, &other).expect("build b");
    let resolver = thermal_resolver(&EQUILIBRIUM_SCENARIO).expect("resolver");
    let only_input = || -> Vec<Box<dyn Flow>> {
        vec![Box::new(HeatInput::new(
            HEAT_INPUT.to_string(),
            HEAT_SOURCE.to_string(),
            NODE.to_string(),
        ))]
    };
    let reg_a = Registry::flows_only(only_input(), &state_a.stocks).expect("registry a");
    let reg_b = Registry::flows_only(only_input(), &state_b.stocks).expect("registry b");
    let (a, ra, _) =
        run_trajectory(&EulerIntegrator::new(reg_a), state_a, &resolver, DT, STEPS).expect("run a");
    let (b, rb, _) =
        run_trajectory(&EulerIntegrator::new(reg_b), state_b, &resolver, DT, STEPS).expect("run b");
    assert_eq!((ra, rb), (0, 0));
    for (sa, sb) in a.iter().zip(b.iter()) {
        let d = sb.stocks[NODE].amount - sa.stocks[NODE].amount;
        assert!((d - 1.0e9).abs() <= 1e-3, "{d}");
    }
}

// --- the monotonic heat-rejected diagnostic --------------------------------------------
#[test]
fn thermal_space_sink_is_monotonic() {
    // `space` only ever receives — radiation is one-way to deep space — so it is
    // non-decreasing every step and strictly grows: the free heat-rejected accumulator,
    // the permanent boundary Thermal cannot move inward.
    let (states, _, _) = euler(&EQUILIBRIUM_SCENARIO);
    let rejected: Vec<f64> = states.iter().map(|s| s.stocks[SPACE].amount).collect();
    for pair in rejected.windows(2) {
        assert!(pair[0] <= pair[1]);
    }
    assert!(rejected[rejected.len() - 1] > rejected[0]);
    assert!(rejected[0] > -1.0);
}

// --- determinism / integrator / registration-order independence -------------------------
#[test]
fn thermal_is_deterministic() {
    let (a, ra, ea) = euler(&EQUILIBRIUM_SCENARIO);
    let (b, rb, eb) = euler(&EQUILIBRIUM_SCENARIO);
    assert_same_state(&b[b.len() - 1], &a[a.len() - 1]);
    assert_eq!((rb, eb.len()), (ra, ea.len()));
}

#[test]
fn thermal_rk4_agrees_with_euler_to_tolerance() {
    // The radiator is state-dependent AND nonlinear, so — unlike Power's forced-only run —
    // RK4 ≢ Euler bit-for-bit. They agree to O(dt²): a real tolerance agreement.
    let (e, _, _) = euler(&EQUILIBRIUM_SCENARIO);
    let (r, _, _) = run(&EQUILIBRIUM_SCENARIO, |reg| {
        Box::new(Rk4Integrator::new(reg))
    });
    let e_final = e[e.len() - 1].stocks[NODE].amount;
    let r_final = r[r.len() - 1].stocks[NODE].amount;
    assert_ne!(
        r_final.to_bits(),
        e_final.to_bits(),
        "the identity did not break"
    );
    assert!((r_final - e_final).abs() <= 1e-4 * e_final.abs());
}

/// Every permutation of `0..n`, in lexicographic order.
///
/// ⚠⚠ **Full enumeration, and a single reverse is NOT good enough — measured, not assumed.**
/// The first version of this test permuted by `into_parts()` + `reverse()`. Under the
/// control that deletes the flow sort in `Registry::new`, three of the four sibling
/// registries reddened and **Power stayed green**: its build order is
/// `[solar_charge, load_draw]`, whose reverse *is* canonical order, so with no sort the
/// "permuted" registry iterated canonically anyway and both assertions passed. One
/// hand-picked permutation is a coin flip on whether it discriminates. The siblings carry
/// 2, 3, 3 and 4 flows, so `n!` is 2, 6, 6 and 24 — enumerating all of them is cheap and
/// cannot miss.
fn permutations(n: usize) -> Vec<Vec<usize>> {
    let mut out: Vec<Vec<usize>> = vec![Vec::new()];
    for _ in 0..n {
        let mut next: Vec<Vec<usize>> = Vec::new();
        for p in &out {
            for v in 0..n {
                if !p.contains(&v) {
                    let mut q = p.clone();
                    q.push(v);
                    next.push(q);
                }
            }
        }
        out = next;
    }
    out
}

/// Apply `perm` to `items`, consuming it (`Box<dyn Flow>` is not `Clone`).
fn permute(items: Vec<Box<dyn Flow>>, perm: &[usize]) -> Vec<Box<dyn Flow>> {
    let mut slots: Vec<Option<Box<dyn Flow>>> = items.into_iter().map(Some).collect();
    perm.iter()
        .map(|&i| slots[i].take().expect("a permutation visits each index once"))
        .collect()
}

#[test]
fn thermal_registration_order_independent() {
    // The Registry sorts flows by id, so ANY registration order must yield a bit-identical
    // run. Both assertions are load-bearing and for different reasons:
    //
    // * the RUN comparison is the property itself;
    // * the ORDER comparison is what stays falsifiable when the run comparison does not.
    //   `season_order_independence.rs` paid for that lesson at season scale — deleting the
    //   sort left its run comparison green, because re-associating comparably-sized sums
    //   moves no bits — and see `permutations` above for the second, smaller trap this
    //   file hit on its own.
    let params = params::thermal();
    let (_, probe) = build_thermal(&params, &EQUILIBRIUM_SCENARIO).expect("build");
    let n = probe.len();
    let family = permutations(n);
    assert_eq!(
        family.len(),
        (1..=n).product::<usize>(),
        "{n} flows should give {} permutations, got {}",
        (1..=n).product::<usize>(),
        family.len()
    );
    let identity: Vec<usize> = (0..n).collect();
    assert!(
        family.iter().any(|p| *p != identity),
        "the permutation family collapsed to the identity"
    );
    let (baseline, base_rationed, base_events) = euler(&EQUILIBRIUM_SCENARIO);
    for perm in &family {
        let (state, registry) = build_thermal(&params, &EQUILIBRIUM_SCENARIO).expect("build");
        let resolver = thermal_resolver(&EQUILIBRIUM_SCENARIO).expect("resolver");
        let (flows, aux) = registry.into_parts();
        let reg = Registry::new(permute(flows, perm), &state.stocks, aux).expect("registry");
        let order: Vec<String> = reg.flows().iter().map(|f| f.id().to_string()).collect();
        let mut sorted = order.clone();
        sorted.sort();
        assert_eq!(
            order, sorted,
            "registration order {perm:?} survived into the registry's iteration order"
        );
        let (states, rationed, events) =
            run_trajectory(&EulerIntegrator::new(reg), state, &resolver, DT, STEPS).expect("run");
        assert_same_state(&states[states.len() - 1], &baseline[baseline.len() - 1]);
        assert_eq!((rationed, events.len()), (base_rationed, base_events.len()));
    }
}
