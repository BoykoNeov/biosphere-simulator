//! The standalone Crew **run** — the net-consumer mission validation (Stage-3 slice S3).
//!
//! Subject: `tests/test_crew_run.py`'s 12 cases.
//!
//! **Honest framing, and the contrast with the other three siblings.** Crew is the first
//! net-consumer / open-loop domain: unlike ECLSS and Thermal it has **no restoring force
//! and no attractor** — the stores simply run down, `store(n) = store0 − n·rate·dt`. It is
//! like Power's forced two-flow run, but a *monotone depletion* rather than a *constructed
//! balance*, since standalone Crew has no resupply. Because no flow reads a stock, the
//! forced-only **RK4 ≡ Euler bit-identity** that ECLSS and Thermal break *returns* here —
//! the symmetric bookend, and the reason that case is worth a test of its own.
//!
//! ⚠ The endurance closed form (`store0 / rate`) has no Rust twin and is re-derived below
//! rather than transcribed from a Python run (§5v).
//!
//! Helpers are duplicated from the sibling `*_run.rs` files by the house rule stated in
//! `power_run.rs`.

use std::collections::BTreeMap;

use domains::crew::{
    build_crew, crew_resolver, CrewScenario, CREW_HUMIDITY, CREW_O2_CONSUMED, EXHALED_CO2,
    FECAL_WASTE, FOOD_STORE, MISSION_DAYS, MISSION_SCENARIO, O2_STORE, URINE, WATER_STORE,
};
use domains::params;
use domains::{run_trajectory, StepIntegrator};
use simcore::conservation::compute_ledger;
use simcore::events::Event;
use simcore::flow::Flow;
use simcore::integrator::{EulerIntegrator, Rk4Integrator};
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::State;

const STEPS: u64 = MISSION_DAYS * MISSION_SCENARIO.steps_per_day;
const DT: f64 = MISSION_SCENARIO.dt_seconds;

fn run<F>(scenario: &CrewScenario, make: F) -> (Vec<State>, u64, Vec<Event>)
where
    F: FnOnce(Registry) -> Box<dyn StepIntegrator>,
{
    let params = params::crew();
    let (state, registry) = build_crew(&params, scenario).expect("build");
    let resolver = crew_resolver(scenario).expect("resolver");
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

fn euler() -> (Vec<State>, u64, Vec<Event>) {
    run(&MISSION_SCENARIO, |r| Box::new(EulerIntegrator::new(r)))
}

/// `(stock id, initial inventory, endurance in seconds)` for the three provisioned stores.
///
/// Endurance is `store0 / rate` — the closed form for a constant forced draw with no
/// resupply, written out here because there is no `depletion_times` in the Rust tree.
fn stores() -> [(&'static str, f64, f64); 3] {
    [
        (
            FOOD_STORE,
            MISSION_SCENARIO.food_store0,
            MISSION_SCENARIO.food_store0 / MISSION_SCENARIO.food_intake_rate,
        ),
        (
            WATER_STORE,
            MISSION_SCENARIO.water_store0,
            MISSION_SCENARIO.water_store0 / MISSION_SCENARIO.water_intake_rate,
        ),
        (
            O2_STORE,
            MISSION_SCENARIO.o2_store0,
            MISSION_SCENARIO.o2_store0 / MISSION_SCENARIO.o2_intake_rate,
        ),
    ]
}

fn bits(state: &State) -> BTreeMap<&str, u64> {
    state
        .stocks
        .iter()
        .map(|(id, s)| (id.as_str(), s.amount.to_bits()))
        .collect()
}

// --- the payload: all three quantities conserved every step ----------------------------
#[test]
fn crew_three_quantities_conserved_every_step() {
    // The augmented store+sink ledger balances CARBON, OXYGEN and WATER simultaneously.
    let (states, _, _) = euler();
    for pair in states.windows(2) {
        let ledger = compute_ledger(&pair[0], &pair[1]).expect("ledger");
        for q in [Quantity::Carbon, Quantity::Oxygen, Quantity::Water] {
            let entry = ledger
                .iter()
                .find(|l| l.quantity == q)
                .unwrap_or_else(|| panic!("{q:?} is present"));
            assert!(entry.residual.abs() <= 1e-6, "{q:?}: {:?}", entry.residual);
        }
    }
}

#[test]
fn crew_only_the_three_mass_quantities_present() {
    // No ENERGY, no NITROGEN stock ⇒ the gate skips them entirely.
    //
    // ⚠ `compute_ledger` documents NAME-sorted quantity order, which is not the enum's own
    // order (`per_quantity_residual` returns that one, via a `BTreeMap<Quantity, _>`). The
    // two differ on OXYGEN vs WATER, so the expected list here is written by name and not
    // copied from a flow-level test.
    let (states, _, _) = euler();
    let ledger = compute_ledger(&states[0], &states[1]).expect("ledger");
    let quantities: Vec<&str> = ledger.iter().map(|q| q.quantity.name()).collect();
    assert_eq!(quantities, vec!["CARBON", "OXYGEN", "WATER"]);
}

#[test]
fn crew_augmented_totals_are_invariant() {
    // Integral form: each quantity's total never moves from the initial provisioned
    // inventory. The stores hold the inventories, so unlike ECLSS no negative-going
    // boundary source is needed — carbon totals to food0, water to water0, oxygen to o2_0.
    let (states, _, _) = euler();
    for s in &states {
        let carbon = s.stocks[FOOD_STORE].amount
            + s.stocks[EXHALED_CO2].amount
            + s.stocks[FECAL_WASTE].amount;
        let water =
            s.stocks[WATER_STORE].amount + s.stocks[CREW_HUMIDITY].amount + s.stocks[URINE].amount;
        let oxygen = s.stocks[O2_STORE].amount + s.stocks[CREW_O2_CONSUMED].amount;
        assert!(
            (carbon - MISSION_SCENARIO.food_store0).abs() <= 1e-9,
            "{carbon}"
        );
        assert!(
            (water - MISSION_SCENARIO.water_store0).abs() <= 1e-9,
            "{water}"
        );
        assert!(
            (oxygen - MISSION_SCENARIO.o2_store0).abs() <= 1e-9,
            "{oxygen}"
        );
    }
}

// --- rationed == 0 / no events ----------------------------------------------------------
#[test]
fn crew_never_rations() {
    // Well-fed sizing: every store's endurance exceeds the mission, so no forced draw
    // over-draws a store and the backstop never fires.
    let (_, rationed, _) = euler();
    assert_eq!(rationed, 0);
}

#[test]
fn crew_stores_stay_positive_and_well_fed() {
    // The well-fed claim with teeth. The 7-day mission is ~30 % of each store's ~23-day
    // endurance, so each store ends near 70 % of its initial inventory: a MATERIAL
    // drawdown that is nonetheless nowhere near empty.
    let (states, _, _) = euler();
    let final_state = &states[states.len() - 1];
    for (stock, initial, _) in stores() {
        let end = final_state.stocks[stock].amount;
        assert!(end > 0.0, "{stock}: {end}");
        assert!(
            0.6 * initial < end && end < 0.8 * initial,
            "{stock}: {end} is not a material-but-safe drawdown of {initial}"
        );
    }
}

#[test]
fn crew_no_events() {
    // No POPULATION stock — crew count is fixed scenario data, not a stock — so extinction
    // can never fire.
    let (_, _, events) = euler();
    assert!(events.is_empty(), "{events:?}");
}

// --- monotone depletion + closed-form endurance -----------------------------------------
#[test]
fn crew_stores_deplete_monotonically() {
    // Constant forced draw, no resupply ⇒ monotone depletion: no restoring force, no
    // attractor.
    let (states, _, _) = euler();
    for (stock, _, _) in stores() {
        let amounts: Vec<f64> = states.iter().map(|s| s.stocks[stock].amount).collect();
        for pair in amounts.windows(2) {
            assert!(pair[0] >= pair[1] - 1e-15, "{stock}: {:?}", pair);
        }
        assert!(
            amounts[amounts.len() - 1] < amounts[0],
            "{stock} never ran down"
        );
    }
}

#[test]
fn crew_depletion_matches_closed_form() {
    // The linear draw matches the endurance closed form: after the horizon a store holds
    // `store0 · (1 − horizon/endurance)`. Re-derived above from `store0 / rate`, not read
    // off a run.
    let (states, _, _) = euler();
    let final_state = &states[states.len() - 1];
    let horizon = STEPS as f64 * DT;
    for (stock, initial, endurance) in stores() {
        let expected = initial * (1.0 - horizon / endurance);
        let actual = final_state.stocks[stock].amount;
        assert!(
            (actual - expected).abs() <= 1e-9 * expected.abs(),
            "{stock}: {actual} != {expected}"
        );
    }
}

// --- the monotonic output diagnostics ----------------------------------------------------
#[test]
fn crew_output_sinks_are_monotonic() {
    // Every output sink only ever receives, so each is non-decreasing every step and
    // strictly grows over the mission — the free cumulative-output diagnostics.
    let (states, _, _) = euler();
    for stock in [
        EXHALED_CO2,
        FECAL_WASTE,
        CREW_HUMIDITY,
        URINE,
        CREW_O2_CONSUMED,
    ] {
        let amounts: Vec<f64> = states.iter().map(|s| s.stocks[stock].amount).collect();
        for pair in amounts.windows(2) {
            assert!(pair[0] <= pair[1] + 1e-15, "{stock}: {:?}", pair);
        }
        assert!(
            amounts[amounts.len() - 1] > amounts[0],
            "{stock} never grew"
        );
    }
}

// --- determinism / integrator / registration-order independence ---------------------------
#[test]
fn crew_is_deterministic() {
    let (a, ra, ea) = euler();
    let (b, rb, eb) = euler();
    assert_eq!(bits(&b[b.len() - 1]), bits(&a[a.len() - 1]));
    assert_eq!((rb, eb.len()), (ra, ea.len()));
}

#[test]
fn crew_rk4_equals_euler_bit_for_bit() {
    // Every flow is FORCED, so every RK4 stage derivative is identical (k1 = k2 = k3 = k4)
    // and the ⅙-combine reproduces k1 exactly: RK4 ≡ Euler BIT-FOR-BIT. This is the
    // forced-only identity ECLSS and Thermal break, revived here — NOT a tolerance
    // agreement, and it also guards against a future state-dependent Crew flow slipping in
    // unnoticed.
    let (e, _, _) = euler();
    let (r, rationed, events) = run(&MISSION_SCENARIO, |reg| Box::new(Rk4Integrator::new(reg)));
    assert_eq!(bits(&r[r.len() - 1]), bits(&e[e.len() - 1]));
    // ⚠ As in `power_run.rs`, the `rationed == 0` half is VACUOUS in this port and is kept
    // only so that is said out loud: `Rk4Integrator` reports 0 by contract.
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
fn crew_registration_order_independent() {
    // The Registry sorts flows by id, so ANY registration order must yield a bit-identical
    // run. Both assertions are load-bearing and for different reasons:
    //
    // * the RUN comparison is the property itself;
    // * the ORDER comparison is what stays falsifiable when the run comparison does not.
    //   `season_order_independence.rs` paid for that lesson at season scale — deleting the
    //   sort left its run comparison green, because re-associating comparably-sized sums
    //   moves no bits — and see `permutations` above for the second, smaller trap this
    //   file hit on its own.
    let params = params::crew();
    let (_, probe) = build_crew(&params, &MISSION_SCENARIO).expect("build");
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
        let (state, registry) = build_crew(&params, &MISSION_SCENARIO).expect("build");
        let resolver = crew_resolver(&MISSION_SCENARIO).expect("resolver");
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
