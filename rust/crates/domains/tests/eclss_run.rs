//! The standalone ECLSS **run** — multi-quantity steady-state validation (Stage-3 S3).
//!
//! Subject: `tests/test_eclss_run.py`'s 16 collected cases from 14 definitions — the one
//! parametrized case there fans over the three species, and is written out here as three
//! named tests so a failure says *which* control loop broke rather than which parameter
//! id did.
//!
//! **Honest framing, and the contrast with Power and Thermal.** Power's flows are both
//! forced, so its boundedness is *constructed* by an exactly-balanced derived load. ECLSS
//! is like Thermal — its three control flows are donor-/demand-controlled restoring
//! forces, so each species has a **genuine emergent steady state** that any constant crew
//! load lands on with no tuning. But ECLSS is **linear**, unlike Thermal's `T⁴`, so its
//! contraction is *geometric*: the `SelfDischarge` law, once per species. That is why the
//! three contraction tests here can assert an exact law where Thermal's can only assert
//! monotone decrease.
//!
//! ⚠ The per-species steady states are a closed form with no Rust twin, so they are
//! re-derived below from each loop's own balance rather than transcribed (§5v).
//!
//! Helpers are duplicated from the sibling `*_run.rs` files by the house rule stated in
//! `power_run.rs`.

use std::collections::BTreeMap;

use domains::eclss::{
    build_eclss, eclss_resolver, CrewMetabolism, EclssScenario, CABIN_CO2, CABIN_H2O, CABIN_O2,
    CO2_REMOVED, CREW_METABOLISM, HUMIDITY_CONDENSATE, METABOLIC_CO2_SOURCE, METABOLIC_H2O_SOURCE,
    METABOLIC_O2_SINK, O2_SUPPLY, STEADY_STATE_SCENARIO, STEADY_STATE_STEPS,
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

const STEPS: u64 = STEADY_STATE_STEPS;
const DT: f64 = STEADY_STATE_SCENARIO.dt_seconds;

/// The emergent per-species cabin steady states `(o2, co2, h2o)`, a closed form.
///
/// At steady state each control loop's removal or supply balances the crew load:
/// `co2_eq = P_co2 / k_scrub`, `h2o_eq = P_h2o / k_cond`, and
/// `o2_eq = o2_setpoint − Con_o2 / k_makeup` — so cabin O₂ sits just *below* the setpoint,
/// which is the whole reason the regulator is not idle at rest. Each is algebraic, so
/// there is nothing to iterate and nothing to read off a run.
fn steady_state(scenario: &EclssScenario) -> (f64, f64, f64) {
    let p = params::eclss();
    (
        p.o2_setpoint - scenario.o2_consumption_rate / p.o2_makeup_gain,
        scenario.co2_production_rate / p.co2_scrub_rate,
        scenario.h2o_production_rate / p.condense_rate,
    )
}

fn run<F>(scenario: &EclssScenario, make: F) -> (Vec<State>, u64, Vec<Event>)
where
    F: FnOnce(Registry) -> Box<dyn StepIntegrator>,
{
    let p = params::eclss();
    let (state, registry) = build_eclss(&p, scenario).expect("build");
    let resolver = eclss_resolver(scenario).expect("resolver");
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

fn euler(scenario: &EclssScenario) -> (Vec<State>, u64, Vec<Event>) {
    run(scenario, |r| Box::new(EulerIntegrator::new(r)))
}

fn series(states: &[State], stock: &str) -> Vec<f64> {
    states.iter().map(|s| s.stocks[stock].amount).collect()
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

/// The shared body of the three per-species contraction tests.
///
/// Two runs differing ONLY in one species' initial amount, under identical crew forcing —
/// so the `CrewMetabolism` legs cancel in the difference and only that species' control
/// loop is left. Because each loop is LINEAR the difference decays by the exact law
/// `d_n = d_0·(1 − k·dt)^n`, which is what makes this stronger than Thermal's
/// monotone-contraction claim.
fn assert_contracts_geometrically(stock: &str, offset: EclssScenario, rate: f64) {
    let (a, _, _) = euler(&STEADY_STATE_SCENARIO);
    let (b, _, _) = euler(&offset);
    let (sa, sb) = (series(&a, stock), series(&b, stock));
    let d0 = (sb[0] - sa[0]).abs();
    assert!(d0 > 0.0, "{stock}: the two runs did not actually differ");
    let decay = 1.0 - rate * DT;
    for n in 0..sa.len() {
        let d = (sb[n] - sa[n]).abs();
        let predicted = d0 * decay.powi(n as i32);
        assert!(
            (d - predicted).abs() <= 1e-12,
            "{stock} at n = {n}: {d} != {predicted}"
        );
    }
    // And it genuinely contracted — a real approach, not a flat line.
    let last = sa.len() - 1;
    assert!((sb[last] - sa[last]).abs() < 1e-6 * d0);
}

// --- the payload: all three quantities conserved every step -----------------------------
#[test]
fn eclss_three_quantities_conserved_every_step() {
    // The first multi-quantity sibling: three quantities gated simultaneously, not one.
    let (states, _, _) = euler(&STEADY_STATE_SCENARIO);
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
fn eclss_only_the_three_mass_quantities_present() {
    // ⚠ `compute_ledger` documents NAME-sorted order, which differs from the enum's own
    // order on OXYGEN vs WATER — see `crew_run.rs`.
    let (states, _, _) = euler(&STEADY_STATE_SCENARIO);
    let ledger = compute_ledger(&states[0], &states[1]).expect("ledger");
    let quantities: Vec<&str> = ledger.iter().map(|q| q.quantity.name()).collect();
    assert_eq!(quantities, vec!["CARBON", "OXYGEN", "WATER"]);
}

#[test]
fn eclss_augmented_totals_are_invariant() {
    // Integral form. Carbon and water start at 0 in the cabin, so each of those totals is
    // 0 (the negative-going boundary source cancels the cabin and the sink); oxygen totals
    // to the initial cabin inventory.
    let (states, _, _) = euler(&STEADY_STATE_SCENARIO);
    for s in &states {
        let oxygen = s.stocks[CABIN_O2].amount
            + s.stocks[O2_SUPPLY].amount
            + s.stocks[METABOLIC_O2_SINK].amount;
        let carbon = s.stocks[CABIN_CO2].amount
            + s.stocks[CO2_REMOVED].amount
            + s.stocks[METABOLIC_CO2_SOURCE].amount;
        let water = s.stocks[CABIN_H2O].amount
            + s.stocks[HUMIDITY_CONDENSATE].amount
            + s.stocks[METABOLIC_H2O_SOURCE].amount;
        assert!(
            (oxygen - STEADY_STATE_SCENARIO.cabin_o2_0).abs() <= 1e-9,
            "{oxygen}"
        );
        assert!(carbon.abs() <= 1e-9, "{carbon}");
        assert!(water.abs() <= 1e-9, "{water}");
    }
}

// --- rationed == 0 / no events -----------------------------------------------------------
#[test]
fn eclss_never_rations() {
    // CO₂/H₂O positivity is STRUCTURAL (k·dt < 1); O₂ positivity is by SIZING — cabin O₂
    // settles at 8 mol, far from empty. The two halves are different arguments and the
    // next test pins only the structural one.
    let (_, rationed, _) = euler(&STEADY_STATE_SCENARIO);
    assert_eq!(rationed, 0);
}

#[test]
fn eclss_structural_positivity_fractions_below_one() {
    let p = params::eclss();
    assert!(p.co2_scrub_rate * DT < 1.0);
    assert!(p.condense_rate * DT < 1.0);
}

#[test]
fn eclss_no_events() {
    let (_, _, events) = euler(&STEADY_STATE_SCENARIO);
    assert!(events.is_empty(), "{events:?}");
}

// --- the emergent steady states (genuine attractors) --------------------------------------
#[test]
fn eclss_converges_to_the_steady_states() {
    // After many time constants each species sits within a narrow band of its emergent
    // steady state — the restoring forces pulled them there, unlike Power's constructed
    // balance.
    let (states, _, _) = euler(&STEADY_STATE_SCENARIO);
    let (o2_eq, co2_eq, h2o_eq) = steady_state(&STEADY_STATE_SCENARIO);
    let final_state = &states[states.len() - 1];
    for (stock, want) in [(CABIN_CO2, co2_eq), (CABIN_H2O, h2o_eq), (CABIN_O2, o2_eq)] {
        let got = final_state.stocks[stock].amount;
        assert!((got - want).abs() <= 1e-6, "{stock}: {got} != {want}");
    }
}

#[test]
fn eclss_species_move_monotonically_to_steady_state() {
    // From the clean cabin: CO₂ and H₂O rise monotonically from 0 to their equilibria; O₂
    // falls monotonically from the setpoint. A constant crew load gives monotone
    // relaxation, with no periodic structure at all.
    let (states, _, _) = euler(&STEADY_STATE_SCENARIO);
    let co2 = series(&states, CABIN_CO2);
    let h2o = series(&states, CABIN_H2O);
    let o2 = series(&states, CABIN_O2);
    assert!(co2[0].abs() < 1e-12 && h2o[0].abs() < 1e-12);
    assert!((o2[0] - STEADY_STATE_SCENARIO.cabin_o2_0).abs() < 1e-12);
    for pair in co2.windows(2) {
        assert!(pair[0] <= pair[1] + 1e-15);
    }
    for pair in h2o.windows(2) {
        assert!(pair[0] <= pair[1] + 1e-15);
    }
    for pair in o2.windows(2) {
        assert!(pair[0] >= pair[1] - 1e-15);
    }
}

// --- the restoring forces: two runs contract GEOMETRICALLY, per species -------------------
#[test]
fn eclss_co2_contracts_geometrically() {
    let offset = EclssScenario {
        cabin_co2_0: STEADY_STATE_SCENARIO.cabin_co2_0 + 1.0,
        ..STEADY_STATE_SCENARIO
    };
    assert_contracts_geometrically(CABIN_CO2, offset, params::eclss().co2_scrub_rate);
}

#[test]
fn eclss_h2o_contracts_geometrically() {
    let offset = EclssScenario {
        cabin_h2o_0: STEADY_STATE_SCENARIO.cabin_h2o_0 + 1.0,
        ..STEADY_STATE_SCENARIO
    };
    assert_contracts_geometrically(CABIN_H2O, offset, params::eclss().condense_rate);
}

#[test]
fn eclss_o2_contracts_geometrically() {
    // ⚠ O₂ offsets DOWNWARD (setpoint − 2) so both runs stay at or below the setpoint. The
    // regulator is unclamped and would reverse above it, which is a different regime and
    // not what this test is about.
    let offset = EclssScenario {
        cabin_o2_0: STEADY_STATE_SCENARIO.cabin_o2_0 - 2.0,
        ..STEADY_STATE_SCENARIO
    };
    assert_contracts_geometrically(CABIN_O2, offset, params::eclss().o2_makeup_gain);
}

#[test]
fn eclss_without_control_difference_is_constant() {
    // The contrast that makes the three contractions meaningful: with the control flows
    // removed — only CrewMetabolism left — there is NO restoring force, so a cabin_co2
    // offset propagates undecayed.
    //
    // ⚠ O₂ consumption is zeroed here so that dropping O2Makeup does not deplete cabin_o2
    // and trip the backstop. That is an artifact of removing the makeup, unrelated to the
    // CO₂ restoring-force point this contrast isolates: CrewMetabolism only *produces*
    // CO₂, so cabin_co2 grows without bound and an offset simply persists.
    let p = params::eclss();
    let base = EclssScenario {
        o2_consumption_rate: 0.0,
        ..STEADY_STATE_SCENARIO
    };
    let other = EclssScenario {
        cabin_co2_0: base.cabin_co2_0 + 1.0,
        ..base
    };
    let (state_a, _) = build_eclss(&p, &base).expect("build a");
    let (state_b, _) = build_eclss(&p, &other).expect("build b");
    let resolver = eclss_resolver(&base).expect("resolver");
    let only_crew = || -> Vec<Box<dyn Flow>> {
        vec![Box::new(CrewMetabolism::new(
            CREW_METABOLISM.to_string(),
            CABIN_O2.to_string(),
            CABIN_CO2.to_string(),
            CABIN_H2O.to_string(),
            METABOLIC_O2_SINK.to_string(),
            METABOLIC_CO2_SOURCE.to_string(),
            METABOLIC_H2O_SOURCE.to_string(),
        ))]
    };
    let reg_a = Registry::flows_only(only_crew(), &state_a.stocks).expect("registry a");
    let reg_b = Registry::flows_only(only_crew(), &state_b.stocks).expect("registry b");
    let (a, ra, _) =
        run_trajectory(&EulerIntegrator::new(reg_a), state_a, &resolver, DT, STEPS).expect("run a");
    let (b, rb, _) =
        run_trajectory(&EulerIntegrator::new(reg_b), state_b, &resolver, DT, STEPS).expect("run b");
    assert_eq!((ra, rb), (0, 0));
    for (sa, sb) in a.iter().zip(b.iter()) {
        let d = sb.stocks[CABIN_CO2].amount - sa.stocks[CABIN_CO2].amount;
        assert!((d - 1.0).abs() <= 1e-12, "{d}");
    }
}

// --- the monotonic diagnostics -------------------------------------------------------------
#[test]
fn eclss_removal_sinks_are_monotonic() {
    // co2_removed and humidity_condensate only ever receive — the free scrubbed/recovered
    // diagnostics.
    let (states, _, _) = euler(&STEADY_STATE_SCENARIO);
    for stock in [CO2_REMOVED, HUMIDITY_CONDENSATE] {
        let amounts = series(&states, stock);
        for pair in amounts.windows(2) {
            assert!(pair[0] <= pair[1] + 1e-15, "{stock}: {pair:?}");
        }
        assert!(
            amounts[amounts.len() - 1] > amounts[0],
            "{stock} never grew"
        );
    }
}

// --- determinism / integrator / registration-order independence ------------------------------
#[test]
fn eclss_is_deterministic() {
    let (a, ra, ea) = euler(&STEADY_STATE_SCENARIO);
    let (b, rb, eb) = euler(&STEADY_STATE_SCENARIO);
    assert_same_state(&b[b.len() - 1], &a[a.len() - 1]);
    assert_eq!((rb, eb.len()), (ra, ea.len()));
}

#[test]
fn eclss_rk4_agrees_with_euler_to_tolerance() {
    // The control flows are state-dependent, so RK4 ≢ Euler bit-for-bit — the forced-only
    // identity does not hold. They agree to O(dt²).
    let (e, _, _) = euler(&STEADY_STATE_SCENARIO);
    let (r, _, _) = run(&STEADY_STATE_SCENARIO, |reg| {
        Box::new(Rk4Integrator::new(reg))
    });
    let e_final = e[e.len() - 1].stocks[CABIN_CO2].amount;
    let r_final = r[r.len() - 1].stocks[CABIN_CO2].amount;
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
fn eclss_registration_order_independent() {
    // The Registry sorts flows by id, so ANY registration order must yield a bit-identical
    // run. Both assertions are load-bearing and for different reasons:
    //
    // * the RUN comparison is the property itself;
    // * the ORDER comparison is what stays falsifiable when the run comparison does not.
    //   `season_order_independence.rs` paid for that lesson at season scale — deleting the
    //   sort left its run comparison green, because re-associating comparably-sized sums
    //   moves no bits — and see `permutations` above for the second, smaller trap this
    //   file hit on its own.
    let p = params::eclss();
    let (_, probe) = build_eclss(&p, &STEADY_STATE_SCENARIO).expect("build");
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
    let (baseline, base_rationed, base_events) = euler(&STEADY_STATE_SCENARIO);
    for perm in &family {
        let (state, registry) = build_eclss(&p, &STEADY_STATE_SCENARIO).expect("build");
        let resolver = eclss_resolver(&STEADY_STATE_SCENARIO).expect("resolver");
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
