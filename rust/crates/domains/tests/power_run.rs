//! The standalone Power **run** — bounded-SOC validation (Stage-3 slice S3).
//!
//! Subject: `tests/test_power_run.py`'s 14 cases. An integration test rather than a
//! `#[cfg(test)]` module because every case here drives the crate's public assembly
//! (`build_power` → `power_resolver` → [`run_trajectory`]), which is exactly how the
//! goldens and the station reach it — the `season_order_independence.rs` precedent.
//!
//! **Honest framing, carried over from the Python module and still true here.** Both
//! Power flows are *forced* (state-independent), so the system is a daily-balanced forced
//! linear accumulator, **not** an emergent attractor: boundedness is *constructed* by the
//! exact daily energy balance the derived load enforces, not restored by anything. What
//! that leaves as non-vacuous is ENERGY conserved every step, `rationed == 0`, the
//! day-over-day return, a genuine SOC swing, monotone waste heat, determinism, and
//! registration-order / integrator independence.
//!
//! Small helpers are **duplicated across the four `*_run.rs` files** rather than pulled
//! into a `tests/common/mod.rs`: `golden_regression.rs` states that choice for three
//! shared lines and this file follows the same house rule.

use std::collections::BTreeMap;

use domains::params;
use domains::power::{
    balanced_load_w, build_power, daily_solar_energy, power_resolver, solar_schedule, BATTERY,
    BOUNDED_SOC_DAYS, BOUNDED_SOC_SCENARIO, SOLAR_SOURCE, WASTE_HEAT,
};
use domains::{run_trajectory, StepIntegrator};
use simcore::conservation::compute_ledger;
use simcore::events::Event;
use simcore::flow::Flow;
use simcore::integrator::{EulerIntegrator, Rk4Integrator};
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::State;

const STEPS: u64 = BOUNDED_SOC_DAYS * BOUNDED_SOC_SCENARIO.steps_per_day;
const DT: f64 = BOUNDED_SOC_SCENARIO.dt_seconds;

/// Run the bounded-SOC scenario under `integrator`, keeping the whole trajectory.
fn run_with<F>(make: F) -> (Vec<State>, u64, Vec<Event>)
where
    F: FnOnce(Registry) -> Box<dyn StepIntegrator>,
{
    let charge = params::charge();
    let (state, registry) = build_power(&charge, &BOUNDED_SOC_SCENARIO, None).expect("build");
    let resolver = power_resolver(&charge, &BOUNDED_SOC_SCENARIO).expect("resolver");
    let integrator = make(registry);
    run_trajectory(integrator.as_ref(), state, &resolver, DT, STEPS).expect("run")
}

fn euler() -> (Vec<State>, u64, Vec<Event>) {
    run_with(|r| Box::new(EulerIntegrator::new(r)))
}

fn soc(states: &[State]) -> Vec<f64> {
    states.iter().map(|s| s.stocks[BATTERY].amount).collect()
}

/// The augmented-system ENERGY total: the unclamped source (cumulative supply, so it goes
/// very negative) + the battery POOL + the monotonic waste-heat sink.
fn energy_total(state: &State) -> f64 {
    state.stocks[SOLAR_SOURCE].amount
        + state.stocks[BATTERY].amount
        + state.stocks[WASTE_HEAT].amount
}

/// Every stock's amount as raw bits — the bit-identity comparison the Python `==` on
/// `State` performs.
fn bits(state: &State) -> BTreeMap<&str, u64> {
    state
        .stocks
        .iter()
        .map(|(id, s)| (id.as_str(), s.amount.to_bits()))
        .collect()
}

// --- the payload: ENERGY conserved every step over the augmented system -------------
#[test]
fn power_energy_conserved_every_step() {
    // The per-step ENERGY ledger residual (Δsolar + Δbattery + Δwaste_heat) is ≈ 0. This
    // echoes the gate the integrator itself runs; pinning it here is the receipt.
    let (states, _, _) = euler();
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
fn power_energy_total_is_invariant() {
    // The integral form: the total ENERGY across all three stocks never moves from the
    // initial SOC, because every flow has Σ legs == 0. "Every joule named", integrated.
    let (states, _, _) = euler();
    let total0 = energy_total(&states[0]);
    assert!((total0 - BOUNDED_SOC_SCENARIO.battery0).abs() < 1e-9);
    for s in &states {
        assert!((energy_total(s) - total0).abs() <= 1e-4);
    }
}

#[test]
fn power_only_energy_is_present() {
    // Power is a pure-ENERGY domain: the ledger names ENERGY and nothing else, so the
    // gate's other-quantity branches are vacuously skipped.
    let (states, _, _) = euler();
    let ledger = compute_ledger(&states[0], &states[1]).expect("ledger");
    let quantities: Vec<Quantity> = ledger.iter().map(|q| q.quantity).collect();
    assert_eq!(quantities, vec![Quantity::Energy]);
}

// --- rationed == 0 / events empty: well-fed sizing -----------------------------------
#[test]
fn power_never_rations() {
    // battery0 is sized a few times the within-day drawdown, so the Euler backstop never
    // fires — positivity from sizing, the Phase-1 discipline.
    let (_, rationed, _) = euler();
    assert_eq!(rationed, 0);
}

#[test]
fn power_no_events() {
    // No POPULATION stock ⇒ extinction can never fire ⇒ no events, and no loss-sink.
    let (_, _, events) = euler();
    assert!(events.is_empty(), "{events:?}");
}

// --- the bounded, day-periodic SOC swing ---------------------------------------------
#[test]
fn power_soc_swings_and_stays_positive() {
    // A genuine charge/discharge swing (min materially below max — not a flat line) that
    // never approaches empty, with a full step's load draw of margin. That margin is what
    // makes `rationed == 0` structural rather than lucky.
    let (states, _, _) = euler();
    let s = soc(&states);
    let lo = s.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = s.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    assert!(
        hi - lo > 0.1 * BOUNDED_SOC_SCENARIO.battery0,
        "swing {hi} - {lo}"
    );
    let one_step_draw = balanced_load_w(&params::charge(), &BOUNDED_SOC_SCENARIO) * DT;
    assert!(one_step_draw > 0.0);
    assert!(
        lo > one_step_draw,
        "min SOC {lo} within one step's draw of empty"
    );
}

#[test]
fn power_soc_returns_each_day() {
    // The real "diurnal cycle" claim: at every day boundary the SOC equals the initial SOC
    // to round-off. True ONLY because the day is balanced — a drifting run fails this — so
    // it is the non-vacuous test that the derived load really balances charge against
    // discharge.
    let (states, _, _) = euler();
    let spd = BOUNDED_SOC_SCENARIO.steps_per_day as usize;
    for day in 0..=(BOUNDED_SOC_DAYS as usize) {
        let amount = states[day * spd].stocks[BATTERY].amount;
        let want = BOUNDED_SOC_SCENARIO.battery0;
        assert!(
            (amount - want).abs() <= 1e-9 * want.abs() + 1e-6,
            "day {day}: {amount} != {want}"
        );
    }
}

#[test]
fn power_charges_by_day_discharges_by_night() {
    // A direction check with teeth: over the first day the SOC dips to its minimum at the
    // morning crossover (load exceeds stored solar at dawn) and peaks in the afternoon, so
    // the in-day minimum is NOT at a day boundary. That is a real diurnal shape rather
    // than a monotone trend that happens to return.
    let (states, _, _) = euler();
    let spd = BOUNDED_SOC_SCENARIO.steps_per_day as usize;
    let first_day = soc(&states[..=spd]);
    let (argmin, _) = first_day
        .iter()
        .enumerate()
        .fold((0usize, f64::INFINITY), |acc, (i, &v)| {
            if v < acc.1 {
                (i, v)
            } else {
                acc
            }
        });
    assert!(
        argmin != 0 && argmin != spd,
        "minimum at boundary index {argmin}"
    );
}

// --- the monotonic heat-generated diagnostic -----------------------------------------
#[test]
fn power_waste_heat_is_monotonic() {
    // waste_heat only ever receives (charge loss + the 100 %-dissipative load), so it is
    // non-decreasing every step and strictly grows over the run — the free
    // heat-generated / "usefulness is not conserved" accumulator.
    let (states, _, _) = euler();
    let heat: Vec<f64> = states.iter().map(|s| s.stocks[WASTE_HEAT].amount).collect();
    for pair in heat.windows(2) {
        assert!(pair[0] <= pair[1], "{:?} > {:?}", pair[0], pair[1]);
    }
    assert!(heat[heat.len() - 1] > heat[0]);
    assert!(heat[0] > -1.0);
}

// --- the balance identity -------------------------------------------------------------
#[test]
fn balanced_load_matches_daily_stored_solar() {
    // The derived load's daily energy equals the daily STORED solar (η_c · supplied) — the
    // exact-balance condition that makes the SOC bounded, and the one place a Power
    // resolver reads η_c at all. A closed form, re-derived here from its own definition
    // rather than pinned to a number a run printed.
    let charge = params::charge();
    let load_w = balanced_load_w(&charge, &BOUNDED_SOC_SCENARIO);
    let e_load_day =
        load_w * BOUNDED_SOC_SCENARIO.steps_per_day as f64 * BOUNDED_SOC_SCENARIO.dt_seconds;
    let e_stored_day = charge.charge_efficiency * daily_solar_energy(&BOUNDED_SOC_SCENARIO);
    assert!(
        (e_load_day - e_stored_day).abs() <= 1e-12 * e_stored_day.abs(),
        "{e_load_day} != {e_stored_day}"
    );
}

// --- the diurnal solar shape -----------------------------------------------------------
#[test]
fn solar_schedule_night_zero_noon_peak() {
    // A half-sine over the 12 h daylight window: 0 at night, peak at solar noon, periodic
    // across days. The sunrise edge (phase 0.25) is in-window but evaluates to exactly 0.
    let sched = solar_schedule(&BOUNDED_SOC_SCENARIO);
    let spd = BOUNDED_SOC_SCENARIO.steps_per_day;
    assert_eq!(sched(0, DT), 0.0);
    let noon = sched(spd / 2, DT);
    assert!(
        (noon - BOUNDED_SOC_SCENARIO.solar_peak_w).abs()
            <= 1e-12 * BOUNDED_SOC_SCENARIO.solar_peak_w,
        "noon {noon}"
    );
    assert_eq!(sched(spd / 2, DT), sched(spd / 2 + spd, DT));
    assert_eq!(sched(spd / 4, DT), 0.0);
    assert_eq!(sched(2, DT), 0.0);
}

// --- determinism / integrator / registration-order independence -----------------------
#[test]
fn power_is_deterministic() {
    // Bit-identical on a re-run — the golden's premise.
    let (a, ra, ea) = euler();
    let (b, rb, eb) = euler();
    assert_eq!(bits(&b[b.len() - 1]), bits(&a[a.len() - 1]));
    assert_eq!((rb, eb.len()), (ra, ea.len()));
}

#[test]
fn power_rk4_equals_euler() {
    // Because the flows are state-independent, every RK4 stage derivative is identical
    // (k1 = k2 = k3 = k4) and the ⅙-combine reproduces k1 exactly — so RK4 ≡ Euler
    // bit-for-bit here. That is an algebraic identity, NOT numerical-robustness evidence;
    // it doubles as a guard that the flows stay forced, since a state-dependent flow would
    // break it.
    let (e, _, _) = euler();
    let (r, rationed, events) = run_with(|reg| Box::new(Rk4Integrator::new(reg)));
    assert_eq!(bits(&r[r.len() - 1]), bits(&e[e.len() - 1]));
    // ⚠ The `rationed == 0` half is VACUOUS in this port and is asserted only to say so:
    // `Rk4Integrator`'s own contract is that a needed scale is a hard error and
    // `StepReport.rationed` is always 0. The bit-identity above is this test's content.
    assert_eq!((rationed, events.len()), (0, 0));
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
fn power_registration_order_independent() {
    // The Registry sorts flows by id, so ANY registration order must yield a bit-identical
    // run. Both assertions are load-bearing and for different reasons:
    //
    // * the RUN comparison is the property itself;
    // * the ORDER comparison is what stays falsifiable when the run comparison does not.
    //   `season_order_independence.rs` paid for that lesson at season scale — deleting the
    //   sort left its run comparison green, because re-associating comparably-sized sums
    //   moves no bits — and see `permutations` above for the second, smaller trap this
    //   file hit on its own.
    let charge = params::charge();
    let (_, probe) = build_power(&charge, &BOUNDED_SOC_SCENARIO, None).expect("build");
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
    let (baseline, base_rationed, base_events) = euler();
    for perm in &family {
        let (state, registry) =
            build_power(&charge, &BOUNDED_SOC_SCENARIO, None).expect("build");
        let resolver = power_resolver(&charge, &BOUNDED_SOC_SCENARIO).expect("resolver");
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
        assert_eq!(
            bits(&states[states.len() - 1]),
            bits(&baseline[baseline.len() - 1]),
            "registration order {perm:?} changed the run"
        );
        assert_eq!((rationed, events.len()), (base_rationed, base_events.len()));
    }
}
