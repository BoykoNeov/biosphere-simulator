//! **Law 3 of 12** — a multi-step *domain* run is registration-order independent
//! (slice C2 of the reference flip, `docs/plans/post-roadmap-reference-flip.md` §5c).
//!
//! # ⚠⚠ Why this law is not in `crates/simcore/tests/laws.rs` with the other eleven
//!
//! Its reference subject is `tests/test_biosphere_demo.py::test_demo_run_is_registration_
//! order_independent`, which drives the **`demo` scenario** — and **that scenario does not
//! exist in this tree**. Checked, not assumed: `grep -rli demo rust/ --include=*.rs` finds
//! only a `config` unit-test fixture *named* `demo.yaml` and an unrelated string in
//! `station/src/bin/sim.rs`. The Python `demo` is one of the four orphan scenarios slice
//! **C6** retires (`demo_euler`, `demo_rk4`, `n_limited`, `water_biting`), so porting the
//! law onto it would mean porting a scenario that is scheduled for deletion.
//!
//! So the law is **re-homed onto the real season registry** rather than recorded as a gap.
//! That trade is worth stating in both directions:
//!
//! * **Gained.** The reference law shuffles a **three-flow skeleton** for 30 steps. This
//!   one shuffles the **whole default biosphere build** — every flow the open-field
//!   scenario wires — through a multi-step run against the real weather forcing. If
//!   canonical order were ever lost in a full assembly, the skeleton would not show it.
//! * **Lost.** `demo`'s own parameters and topology are no longer exercised by any law.
//!   Nothing else in this tree exercises them either, which is C6's whole point.
//! * **Also lost, and named because the reference law had it: the RK4 arm.** The reference
//!   runs its demo under *both* integrators. The frozen biosphere is Euler-only — the
//!   arbitration backstop is Euler-only by charter and a needed scale is a **hard error**
//!   under RK4 — so an RK4 arm here would be testing an unsupported configuration, not the
//!   law. The RK4 arm of the same law survives on the engine-level subject
//!   (`simcore/tests/laws.rs::law_step_is_registration_order_independent_for_both_integrators`),
//!   which is where both integrators are actually supported.
//!
//! # ⚠ The permutations here are STRUCTURAL, not exhaustive, and that is forced
//!
//! The other eight permutation laws enumerate `3!` or `4!` in full. The season registry
//! has more than twenty flows and `n!` is not a number one enumerates, so this law uses a
//! fixed, documented family instead of random shuffles: the identity, the reverse, every
//! rotation, and an adjacent-swap pass. Deterministic, reproducible, and — the reason it
//! needs no case generator at all — this file therefore carries **no second copy** of the
//! generator that lives in the `simcore` law suite.
//!
//! ⚠ Note what is *not* claimed: a structural family is not a random sample and does not
//! stand in for one. What it does cover is every "off by a rotation" and "adjacent pair
//! transposed" failure, which is the shape a lost or partial sort actually produces.

use simcore::flow::Flow;
use simcore::integrator::EulerIntegrator;
use simcore::registry::Registry;
use simcore::state::State;

use domains::biosphere::system::{build_season, weather_resolver, DEFAULT_SCENARIO};
use domains::biosphere::BIO_DT;

/// The structural permutation family, as index lists over `0..n`.
///
/// Identity, reverse, all rotations, and the adjacent-swap pass (`0<->1`, `2<->3`, …).
fn structural_permutations(n: usize) -> Vec<Vec<usize>> {
    let mut out: Vec<Vec<usize>> = vec![(0..n).collect(), (0..n).rev().collect()];
    for k in 1..n {
        out.push((0..n).map(|i| (i + k) % n).collect());
    }
    let mut swapped: Vec<usize> = (0..n).collect();
    for i in (0..n.saturating_sub(1)).step_by(2) {
        swapped.swap(i, i + 1);
    }
    out.push(swapped);
    out.sort();
    out.dedup();
    out
}

/// The family itself, because the law below is only as good as its case list.
///
/// A `structural_permutations` that returned only the identity would leave the law green
/// and vacuous — the failure this asserts away.
#[test]
fn the_structural_family_is_well_formed_and_not_degenerate() {
    for n in [1usize, 2, 3, 8, 23] {
        let family = structural_permutations(n);
        for p in &family {
            let mut sorted = p.clone();
            sorted.sort_unstable();
            assert_eq!(
                sorted,
                (0..n).collect::<Vec<_>>(),
                "n={n}: not a permutation"
            );
        }
        let identity: Vec<usize> = (0..n).collect();
        assert!(family.contains(&identity), "n={n}: identity missing");
        if n > 1 {
            assert!(
                family.iter().any(|p| *p != identity),
                "n={n}: the family collapsed to the identity"
            );
            // `n == 2` has only one non-identity permutation at all — reverse, rotation
            // and adjacent-swap are the same list — so the family is exactly 2 there.
            let want = if n == 2 { 2 } else { 3 };
            assert!(family.len() >= want, "n={n}: only {} members", family.len());
        }
    }
}

/// Apply `perm` to `items`, consuming it (`Box<dyn Flow>` is not `Clone`).
fn permute(items: Vec<Box<dyn Flow>>, perm: &[usize]) -> Vec<Box<dyn Flow>> {
    let mut slots: Vec<Option<Box<dyn Flow>>> = items.into_iter().map(Some).collect();
    perm.iter()
        .map(|&i| {
            slots[i]
                .take()
                .expect("a permutation visits each index once")
        })
        .collect()
}

fn amounts(state: &State) -> Vec<(String, u64)> {
    state
        .stocks
        .iter()
        .map(|(id, s)| (id.clone(), s.amount.to_bits()))
        .collect()
}

/// Build the season and run `steps` Euler steps with the flows re-registered in `perm`.
///
/// `Registry::into_parts` is the documented rebuild-through primitive (Rust cannot clone a
/// `Box<dyn Flow>` out of a `&Registry`), so the flows come back **owned** and can be
/// re-registered in any order. What comes back is already canonical, which is precisely
/// what makes any permutation of it an arbitrary registration order.
fn run_with_order(perm: &[usize], steps: u32) -> Vec<(String, u64)> {
    let (state, registry) = build_season(&DEFAULT_SCENARIO).expect("season build");
    let resolver = weather_resolver(&DEFAULT_SCENARIO, 1).expect("weather resolver");
    let (flows, aux) = registry.into_parts();
    let reg = Registry::new(permute(flows, perm), &state.stocks, aux).expect("registry");
    let integrator = EulerIntegrator::new(reg);
    let mut current = state;
    for _ in 0..steps {
        current = integrator
            .step(&current, &resolver, BIO_DT)
            .expect("season step");
    }
    amounts(&current)
}

/// `(flows, aux processes, stocks)` the default build wires — the subject's size.
fn season_shape() -> (usize, usize, usize) {
    let (state, registry) = build_season(&DEFAULT_SCENARIO).expect("season build");
    let n_stocks = state.stocks.len();
    let (flows, aux) = registry.into_parts();
    (flows.len(), aux.len(), n_stocks)
}

#[test]
fn law_a_multi_step_season_run_is_registration_order_independent() {
    const STEPS: u32 = 30;

    let (n_flows, n_aux, n_stocks) = season_shape();
    let identity: Vec<usize> = (0..n_flows).collect();

    // The subject must not be a skeleton. If the default build ever collapsed to a couple
    // of flows this law would be back at the reference's scale without anyone noticing —
    // and being the REAL assembly is the entire reason it was re-homed here.
    assert!(
        n_flows >= 8,
        "the default season build wired only {n_flows} flows"
    );
    assert!(
        n_aux >= 1,
        "the default season build wired no aux process; the aux reduction is half of what \
         this law covers"
    );
    assert!(
        n_stocks >= 8,
        "the default season build wired {n_stocks} stocks"
    );

    let canonical = run_with_order(&identity, STEPS);
    let family = structural_permutations(n_flows);
    assert!(
        family.len() >= n_flows,
        "only {} permutations for {n_flows} flows",
        family.len()
    );
    for perm in &family {
        let other = run_with_order(perm, STEPS);
        assert_eq!(
            other, canonical,
            "registration order {perm:?} changed a {STEPS}-step season run"
        );
    }
}
