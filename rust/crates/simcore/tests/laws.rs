//! The engine's **mathematical laws**, natively — slice C2 of the reference flip
//! (`docs/plans/post-roadmap-reference-flip.md` §5c).
//!
//! Eleven of the reference's twelve `@given` law sites live here; the twelfth (a
//! multi-step *domain* run) has no `simcore` subject and lives in
//! `crates/domains/tests/season_order_independence.rs`. The laws themselves are the
//! universal ones the project asserts everywhere: **conservation**, **non-negativity**,
//! **order-independence** — plus the canonical-reduction-order invariant (#15) they all
//! rest on.
//!
//! # ⚠⚠ Why there is no `proptest`, and why the case sets are STRONGER for it
//!
//! The plan (§5c, C2) said *"add `proptest`"*. Measured against the actual sites, that
//! was the wrong instrument for two thirds of them: **eight of the twelve reference laws
//! are permutations of three or four elements**. Hypothesis draws ~100 samples from a
//! space of **6** or **24**; this file **enumerates every one**. That is a measured
//! improvement over the reference law, not a workaround for a missing library —
//! `proptest 1.11.0` and its whole dependency tree are in the local registry cache, so
//! this is a choice. The choice is that a fifteen-crate dev-dependency, in a workspace
//! whose engine crates are zero-dep by charter, is the wrong price for the four laws that
//! genuinely need generated values.
//!
//! ⚠ **What is given up, stated rather than implied:** shrinking. A failing exhaustive
//! permutation is already minimal (it *is* one permutation of three elements); a failing
//! generated case is reported with its seed and index, and reproducing it is
//! deterministic, but nothing narrows it for you.
//!
//! # ⚠⚠ Three of the reference laws are UNFALSIFIABLE in Rust as written, and one such
//! test is already in this crate
//!
//! The reference's composition-fold, ledger-residual and `observe` laws shuffle the
//! **insertion order of a Python `dict`**. `State.stocks` is a `BTreeMap`, so "insertion
//! order" is not expressible: a shuffled build and a canonical build are *the same map*.
//! A Rust test that shuffles a `Vec` into a `BTreeMap` and asserts the results agree has
//! measured nothing — it is *an empty frozen set wearing the clothes of a passing one*
//! (`docs/log/reference-flip.md`, slice 3). `observation.rs`'s own
//! `insertion_order_independent` is exactly that shape today: its `forward` and
//! `backward` maps are identical before `observe` is ever called.
//!
//! So those three laws are re-expressed on the axis that **is** falsifiable here — the
//! *value* of the fold. The fixtures are chosen so the sorted-order accumulation and the
//! reverse-order accumulation differ **in bits**, and each law asserts both that the
//! implementation matches the sorted one and (the discriminator, so the assertion cannot
//! decay into a tautology) that the two hand-folds really do differ. Reversing the fold
//! in the reference source turns them red; the reference's own version would stay green.
//!
//! # The discipline every law here follows
//!
//! 1. **Exhaustive where the space is small**, generated only where it is not.
//! 2. **A discriminator assertion.** A permutation law over a fixture whose reduction is
//!    order-*insensitive* passes for any deterministic implementation, sorted or not. Every
//!    law here either proves its fixture is order-sensitive (by hand-folding it two ways
//!    and asserting the bits differ) or says in its own comment that it cannot.
//! 3. **A meta-assertion on the generator** ([`gen::Spread`]): a generator that silently
//!    collapsed to one-character ids and identity permutations would pass every law it
//!    feeds.

use std::collections::BTreeMap;

use simcore::auxiliary::AuxProcess;
use simcore::boundary;
use simcore::conservation::compute_ledger;
use simcore::environment::{constant, Environment, Schedule, SourceResolver};
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::ids::{AuxId, FlowId, StockId};
use simcore::integrator::{EulerIntegrator, Rk4Integrator};
use simcore::multirate::{multirate_step, Split};
use simcore::observation::observe;
use simcore::quantities::{Quantity, StockKind};
use simcore::registry::Registry;
use simcore::rng::CounterRng;
use simcore::state::{State, Stock};

// --------------------------------------------------------------------------- //
// Exhaustive permutations.                                                     //
// --------------------------------------------------------------------------- //

/// Every permutation of `0..n`, in a deterministic order.
///
/// Heap's algorithm, iterative form. `n` is 3 or 4 at every call site here (6 and 24
/// permutations); the reference's Hypothesis strategy samples this same space, so
/// enumerating it is the stronger statement rather than a different one.
fn permutations(n: usize) -> Vec<Vec<usize>> {
    let mut current: Vec<usize> = (0..n).collect();
    let mut out = vec![current.clone()];
    let mut counters = vec![0usize; n];
    let mut i = 0;
    while i < n {
        if counters[i] < i {
            let j = if i % 2 == 0 { 0 } else { counters[i] };
            current.swap(i, j);
            out.push(current.clone());
            counters[i] += 1;
            i = 0;
        } else {
            counters[i] = 0;
            i += 1;
        }
    }
    out
}

/// The enumerator itself, because every law below is only as good as its case list.
///
/// A `permutations` that returned just the identity would leave all eight permutation
/// laws green and vacuous — the failure this asserts away.
#[test]
fn the_permutation_enumerator_is_complete_and_not_degenerate() {
    for (n, expected) in [(1usize, 1usize), (2, 2), (3, 6), (4, 24)] {
        let perms = permutations(n);
        assert_eq!(perms.len(), expected, "n={n}: wrong permutation count");
        let mut seen = perms.clone();
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), expected, "n={n}: duplicate permutations");
        for p in &perms {
            let mut sorted = p.clone();
            sorted.sort_unstable();
            assert_eq!(
                sorted,
                (0..n).collect::<Vec<_>>(),
                "n={n}: not a permutation"
            );
        }
        let identity: Vec<usize> = (0..n).collect();
        assert!(perms.contains(&identity), "n={n}: identity missing");
        if n > 1 {
            assert!(
                perms.iter().any(|p| *p != identity),
                "n={n}: every permutation is the identity"
            );
        }
    }
}

/// Apply `perm` to `items`, consuming it (a `Box<dyn Flow>` cannot be cloned).
fn permute<T>(items: Vec<T>, perm: &[usize]) -> Vec<T> {
    let mut slots: Vec<Option<T>> = items.into_iter().map(Some).collect();
    perm.iter()
        .map(|&i| {
            slots[i]
                .take()
                .expect("a permutation visits each index once")
        })
        .collect()
}

// --------------------------------------------------------------------------- //
// The deterministic case generator (for the four laws a permutation cannot cover). //
// --------------------------------------------------------------------------- //

mod gen {
    /// A 64-bit linear congruential generator, for **test-case inputs only**.
    ///
    /// ⚠⚠ **Deliberately NOT `simcore::rng::mix64`, and that is the whole point of
    /// writing one.** `CounterRng` is the *subject* of two of the laws below, and
    /// `mix64` is splitmix64's finalizer — seeding a case set for the mixer from the
    /// mixer is the self-referential shape this project already had to dissolve once,
    /// for the cross-port RNG vectors (`docs/log/reference-flip.md`, slice 3: *the gate
    /// would compare the reference against itself*). An LCG is a structurally different
    /// family: multiply-add on the whole state, no xor-shift avalanche.
    ///
    /// Knuth's MMIX constants. The low bits of an LCG are weak, so every read takes the
    /// **high** 32 bits — ample for choosing lengths, characters and swap targets, and
    /// nothing here is a statistical claim.
    pub struct Lcg(u64);

    impl Lcg {
        pub fn new(seed: u64) -> Self {
            Lcg(seed)
        }

        fn next_u32(&mut self) -> u32 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (self.0 >> 32) as u32
        }

        /// A value in `0..n` (`n > 0`).
        pub fn below(&mut self, n: usize) -> usize {
            (self.next_u32() as usize) % n
        }

        /// A full 64-bit value (two reads, so the high-bits rule still holds).
        pub fn next_u64(&mut self) -> u64 {
            (u64::from(self.next_u32()) << 32) | u64::from(self.next_u32())
        }

        /// Shuffle in place (Fisher–Yates, descending).
        pub fn shuffle<T>(&mut self, items: &mut [T]) {
            for i in (1..items.len()).rev() {
                let j = self.below(i + 1);
                items.swap(i, j);
            }
        }
    }

    /// What a generated case set must actually *span*, accumulated while the law runs.
    ///
    /// ⚠ The failure this exists for: a generator that silently yielded only
    /// single-character ids and identity permutations **passes every law it feeds**. The
    /// law cannot see its own case set collapse, so the case set is asserted separately —
    /// *an empty frozen set is an inert comparison wearing the clothes of a passing one*,
    /// one level up.
    #[derive(Default)]
    pub struct Spread {
        pub cases: usize,
        pub lengths: std::collections::BTreeSet<usize>,
        pub max_item_len: usize,
        pub non_identity_orders: usize,
        pub distinct_scalars: std::collections::BTreeSet<u64>,
    }

    impl Spread {
        pub fn assert_spans(&self, what: &str, min_cases: usize, min_lengths: usize) {
            assert!(
                self.cases >= min_cases,
                "{what}: only {} cases generated (want >= {min_cases})",
                self.cases
            );
            assert!(
                self.lengths.len() >= min_lengths,
                "{what}: case sizes span only {:?} (want >= {min_lengths} distinct)",
                self.lengths
            );
        }
    }
}

use gen::{Lcg, Spread};

// --------------------------------------------------------------------------- //
// Test doubles (the reference's `_DrainFlow` / `_TransferFlow` / `_DecayFlow` /       //
// `_ConstRateAux`, ported).                                                    //
// --------------------------------------------------------------------------- //

/// Withdraws a fixed `amount` per `dt` from `src` into boundary `sink` — a *constant*
/// withdrawal, independent of the level, so it can over-draw (the reference's
/// `_DrainFlow`). Real saturating kinetics taper to 0 as a stock empties; this does not.
struct DrainFlow {
    id: FlowId,
    src: StockId,
    sink: StockId,
    amount: f64,
}

impl Flow for DrainFlow {
    fn type_name(&self) -> &'static str {
        "DrainFlow"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, _s: &State, _e: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let x = self.amount * dt;
        FlowResult::new(vec![
            Leg::new(self.src.clone(), -x)?,
            Leg::new(self.sink.clone(), x)?,
        ])
    }
}

/// `src -> dst`, moving a fixed fraction of `src` per step (dt-linear) — the reference's
/// `_TransferFlow`.
struct TransferFlow {
    id: FlowId,
    src: StockId,
    dst: StockId,
    frac: f64,
}

impl Flow for TransferFlow {
    fn type_name(&self) -> &'static str {
        "TransferFlow"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, s: &State, _e: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let amount = self.frac * s.stocks[&self.src].amount * dt;
        FlowResult::new(vec![
            Leg::new(self.src.clone(), -amount)?,
            Leg::new(self.dst.clone(), amount)?,
        ])
    }
}

/// A no-op flow carrying only an id — the subject of the registry-ordering law.
struct NoopFlow(FlowId);

impl Flow for NoopFlow {
    fn type_name(&self) -> &'static str {
        "NoopFlow"
    }
    fn id(&self) -> &str {
        &self.0
    }
    fn evaluate(&self, _s: &State, _e: &dyn Environment, _dt: f64) -> Result<FlowResult, SimError> {
        Ok(FlowResult::empty())
    }
}

/// Constant-rate accumulator: increment == `rate·dt` (the reference's `_ConstRateAux`).
struct ConstRateAux {
    id: AuxId,
    name: String,
    rate: f64,
}

impl AuxProcess for ConstRateAux {
    fn type_name(&self) -> &'static str {
        "ConstRateAux"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        _s: &State,
        _e: &dyn Environment,
        dt: f64,
    ) -> Result<BTreeMap<String, f64>, SimError> {
        Ok(BTreeMap::from([(self.name.clone(), self.rate * dt)]))
    }
}

// --------------------------------------------------------------------------- //
// Stock / state helpers.                                                       //
// --------------------------------------------------------------------------- //

fn pool(id: &str, amount: f64) -> Stock {
    Stock::new(
        id.to_string(),
        "law".to_string(),
        Quantity::Carbon,
        Quantity::Carbon.canonical_unit(),
        amount,
        StockKind::Pool,
        0.0,
        false,
        BTreeMap::new(),
    )
    .expect("pool stock")
}

/// A POOL stock whose composition carries a second quantity — the subject of the
/// composition-fold law (a 1:1 stock folds through the default map instead).
fn composite_pool(id: &str, amount: f64, oxygen_per_carbon: f64) -> Stock {
    Stock::new(
        id.to_string(),
        "law".to_string(),
        Quantity::Carbon,
        Quantity::Carbon.canonical_unit(),
        amount,
        StockKind::Pool,
        0.0,
        false,
        BTreeMap::from([
            (Quantity::Carbon, 1.0),
            (Quantity::Oxygen, oxygen_per_carbon),
        ]),
    )
    .expect("composite pool stock")
}

fn state_of(n: u64, stocks: Vec<Stock>) -> State {
    let map: BTreeMap<StockId, Stock> = stocks.into_iter().map(|s| (s.id.clone(), s)).collect();
    State::new(n, map, 0, BTreeMap::new()).expect("state")
}

fn amounts(state: &State) -> Vec<(String, u64)> {
    state
        .stocks
        .iter()
        .map(|(id, s)| (id.clone(), s.amount.to_bits()))
        .collect()
}

/// Left-fold a slice of `f64` in the given order — the hand-computed reduction every
/// discriminator below compares against.
fn fold(values: &[f64]) -> f64 {
    let mut acc = 0.0;
    for v in values {
        acc += v;
    }
    acc
}

/// Assert `values` is order-**sensitive**: the left fold and its reverse differ in bits.
///
/// ⚠ This is the assertion that keeps an order-independence law from being satisfied by
/// any deterministic implementation, sorted or not. A fixture of three equal magnitudes
/// produces the same sum under every permutation, so the law would pass against a
/// registry that never sorted at all.
fn assert_order_sensitive(values: &[f64], what: &str) {
    let forward = fold(values);
    let mut reversed = values.to_vec();
    reversed.reverse();
    let backward = fold(&reversed);
    assert_ne!(
        forward.to_bits(),
        backward.to_bits(),
        "{what}: the fixture's reduction is order-INSENSITIVE ({forward:?} either way), so \
         the order-independence law below would pass without any canonical sort. Choose \
         magnitudes whose partial sums round differently."
    );
}

// --------------------------------------------------------------------------- //
// LAW 1 — arbitration: an Euler over-draw is registration-order independent.   //
// (reference: tests/test_arbitration.py::test_euler_overdraw_is_registration_  //
//  order_independent)                                                          //
// --------------------------------------------------------------------------- //

#[test]
fn law_euler_overdraw_is_registration_order_independent() {
    // Three flows competing for one scarce stock. ⚠ The reference fixture gives all three
    // the SAME withdrawal (8.0 from a stock of 10), which makes its demand sum
    // order-insensitive: every permutation yields the identical multiset of legs, so that
    // fixture cannot tell a sorting implementation from a non-sorting one. These amounts
    // can: 1.0 + 1.0 + 1e16 rounds to 1e16+2, while 1e16 + 1.0 + 1.0 rounds to 1e16 — a
    // different scale, and a different final stock amount.
    let drains = [1.0_f64, 1.0, 1e16];
    assert_order_sensitive(&drains, "law 1 demand sum");

    let stocks: BTreeMap<StockId, Stock> = [
        pool("law.scarce", 10.0),
        boundary::sink("boundary.k1".to_string(), Quantity::Carbon, 0.0).expect("sink"),
        boundary::sink("boundary.k2".to_string(), Quantity::Carbon, 0.0).expect("sink"),
        boundary::sink("boundary.k3".to_string(), Quantity::Carbon, 0.0).expect("sink"),
    ]
    .into_iter()
    .map(|s| (s.id.clone(), s))
    .collect();
    let state = State::new(0, stocks.clone(), 0, BTreeMap::new()).expect("state");
    let env = SourceResolver::empty();

    let build = || -> Vec<Box<dyn Flow>> {
        vec![
            Box::new(DrainFlow {
                id: "drain_1".to_string(),
                src: "law.scarce".to_string(),
                sink: "boundary.k1".to_string(),
                amount: drains[0],
            }),
            Box::new(DrainFlow {
                id: "drain_2".to_string(),
                src: "law.scarce".to_string(),
                sink: "boundary.k2".to_string(),
                amount: drains[1],
            }),
            Box::new(DrainFlow {
                id: "drain_3".to_string(),
                src: "law.scarce".to_string(),
                sink: "boundary.k3".to_string(),
                amount: drains[2],
            }),
        ]
    };

    let canonical = EulerIntegrator::new(Registry::new(build(), &stocks, Vec::new()).expect("reg"))
        .step_report(&state, &env, 1.0)
        .expect("canonical step");
    // The backstop must actually have fired, or this is a well-fed step wearing an
    // over-draw's clothes and the scaling reduction under test never ran. Three, not one:
    // `rationed` counts *firings*, and all three flows are scaled — pinned exactly, so a
    // change to what the counter counts is visible here rather than absorbed.
    assert_eq!(
        canonical.rationed, 3,
        "the fixture did not over-draw all three flows, so the scaling reduction under \
         test did not run"
    );

    for perm in permutations(3) {
        let shuffled = permute(build(), &perm);
        let other =
            EulerIntegrator::new(Registry::new(shuffled, &stocks, Vec::new()).expect("reg"))
                .step_report(&state, &env, 1.0)
                .expect("shuffled step");
        assert_eq!(
            amounts(&other.state),
            amounts(&canonical.state),
            "registration order {perm:?} changed the arbitrated amounts"
        );
    }
}

// --------------------------------------------------------------------------- //
// LAW 2 — the aux accumulator sum is registration-order independent.           //
// (reference: tests/test_aux.py::test_aux_sum_is_registration_order_independent) //
// --------------------------------------------------------------------------- //

#[test]
fn law_aux_accumulator_sum_is_registration_order_independent() {
    // Three processes writing ONE shared name with associativity-sensitive increments:
    // the canonical (AuxId-sorted) sum ((0 + 1) + 1e16) - 1e16 loses the 1 and is 0.0,
    // while the reverse order keeps it and is 1.0. The sort is what discriminates. Two
    // processes would be vacuous — float `+` is commutative.
    let rates = [1.0_f64, 1e16, -1e16];
    assert_order_sensitive(&rates, "law 2 aux sum");

    let build = || -> Vec<Box<dyn AuxProcess>> {
        vec![
            Box::new(ConstRateAux {
                id: "a".to_string(),
                name: "acc".to_string(),
                rate: rates[0],
            }),
            Box::new(ConstRateAux {
                id: "b".to_string(),
                name: "acc".to_string(),
                rate: rates[1],
            }),
            Box::new(ConstRateAux {
                id: "c".to_string(),
                name: "acc".to_string(),
                rate: rates[2],
            }),
        ]
    };
    let stocks: BTreeMap<StockId, Stock> = BTreeMap::new();
    let state = State::new(0, stocks.clone(), 0, BTreeMap::new()).expect("state");
    let env = SourceResolver::empty();

    let step_once = |procs: Vec<Box<dyn AuxProcess>>| -> f64 {
        let reg = Registry::new(Vec::new(), &stocks, procs).expect("reg");
        EulerIntegrator::new(reg)
            .step(&state, &env, 1.0)
            .expect("aux step")
            .aux["acc"]
    };

    let canonical = step_once(build());
    // Pins the canonical order as the AuxId-sorted one, so the law above is a real
    // discriminator rather than trivially true (the reference's sibling assertion).
    assert_eq!(canonical.to_bits(), 0.0_f64.to_bits());
    for perm in permutations(3) {
        assert_eq!(
            step_once(permute(build(), &perm)).to_bits(),
            canonical.to_bits(),
            "registration order {perm:?} changed the aux accumulator"
        );
    }
}

// --------------------------------------------------------------------------- //
// LAW 4 — the composition fold accumulates in canonical stock order.           //
// (reference: tests/test_composition.py::test_composition_fold_is_stock_order_ //
//  independent — re-expressed; see the module docstring)                        //
// --------------------------------------------------------------------------- //

#[test]
fn law_composition_fold_accumulates_in_canonical_stock_order() {
    // Multi-quantity stocks with mixed magnitudes. The OXYGEN residual is the sum of
    // `delta · coeff` over the stocks in id order; the coefficients differ, so the
    // per-stock oxygen contributions do too, and the fold order decides the last bits:
    // `(1e16 + 1) + 1` is `1e16`, while `(1 + 1) + 1e16` is `1e16 + 2`.
    //
    // ⚠ The CARBON residual over the same three stocks is order-*insensitive*
    // (`1e16 + 2 + 0.5` rounds the same either way), which is why only the oxygen fold
    // carries the discriminator. Carbon is still asserted — it just is not the axis that
    // proves the sort ran, and saying so is the difference between a check and a claim.
    let deltas = [1e16_f64, 2.0, 0.5]; // a, b, c in id order
    let coeffs = [1.0_f64, 0.5, 2.0];
    let oxygen: Vec<f64> = deltas.iter().zip(coeffs).map(|(d, c)| d * c).collect();
    assert_order_sensitive(&oxygen, "law 4 oxygen residual");

    let base = [1e8_f64, 3.0, 7.0];
    let before = state_of(
        0,
        vec![
            composite_pool("law.a", base[0], coeffs[0]),
            composite_pool("law.b", base[1], coeffs[1]),
            composite_pool("law.c", base[2], coeffs[2]),
        ],
    );
    let after = state_of(
        1,
        vec![
            composite_pool("law.a", base[0] + deltas[0], coeffs[0]),
            composite_pool("law.b", base[1] + deltas[1], coeffs[1]),
            composite_pool("law.c", base[2] + deltas[2], coeffs[2]),
        ],
    );

    let ledger = compute_ledger(&before, &after).expect("ledger");
    let by_quantity: BTreeMap<Quantity, f64> =
        ledger.iter().map(|q| (q.quantity, q.residual)).collect();

    // The implementation must reproduce the id-sorted fold, bit for bit — and NOT the
    // reverse one, which `assert_order_sensitive` has just proved is a different number.
    assert_eq!(
        by_quantity[&Quantity::Oxygen].to_bits(),
        fold(&oxygen).to_bits(),
        "the composition fold did not accumulate in canonical (id-sorted) stock order"
    );
    assert_eq!(
        by_quantity[&Quantity::Carbon].to_bits(),
        fold(&deltas).to_bits()
    );
}

// --------------------------------------------------------------------------- //
// LAW 5 — the conservation ledger's residual accumulates in canonical order.   //
// (reference: tests/test_conservation.py::test_compute_ledger_residual_is_     //
//  stock_insertion_order_independent — re-expressed)                            //
// --------------------------------------------------------------------------- //

#[test]
fn law_ledger_residual_accumulates_in_canonical_stock_order() {
    // The reference's own fixture, whose deltas cancel to exactly 0.0 in id order and to
    // 2.78e-17 reversed — so the sorted fold is observably the one that ran.
    let deltas = [0.1_f64, -1.0, -0.1, 1.0];
    assert_order_sensitive(&deltas, "law 5 carbon residual");
    assert_eq!(fold(&deltas).to_bits(), 0.0_f64.to_bits());

    let ids = ["law.a", "law.b", "law.c", "law.d"];
    let before_amounts = [1e8_f64, 3.0, 1e8, 7.0];
    let before = state_of(
        0,
        ids.iter()
            .zip(before_amounts)
            .map(|(id, a)| pool(id, a))
            .collect(),
    );
    let after = state_of(
        1,
        ids.iter()
            .zip(before_amounts.iter().zip(deltas))
            .map(|(id, (a, d))| pool(id, a + d))
            .collect(),
    );

    let ledger = compute_ledger(&before, &after).expect("ledger");
    let carbon = ledger
        .iter()
        .find(|q| q.quantity == Quantity::Carbon)
        .expect("carbon ledger row");
    assert_eq!(
        carbon.residual.to_bits(),
        fold(&deltas).to_bits(),
        "the ledger residual did not accumulate in canonical (id-sorted) stock order"
    );
}

// --------------------------------------------------------------------------- //
// LAW 6 — a forcing value depends only on (n, dt).                             //
// (reference: tests/test_environment.py::test_forcing_value_depends_only_on_n_ //
//  and_dt)                                                                     //
// --------------------------------------------------------------------------- //

#[test]
fn law_forcing_value_depends_only_on_n_and_dt() {
    // A forcing var's value must depend only on `(n, dt)` — never on the snapshot's stock
    // contents — so the forcing and shared-stock branches cannot bleed into each other.
    //
    // ⚠ Generated rather than exhaustive, but the *interesting* inputs here are edge
    // cases, not random draws: the fixed grid below carries ±0, subnormals, the extrema
    // and the powers of two that a naive `f32`-width Hypothesis draw reaches only by
    // luck. The generated part is the (n, dt) pairing.
    let interesting: [f64; 12] = [
        0.0,
        -0.0,
        1.0,
        -1.0,
        f64::MIN_POSITIVE,
        -f64::MIN_POSITIVE,
        f64::MAX,
        f64::MIN,
        5e-324, // the smallest subnormal
        1e16,
        -1e16,
        0.1,
    ];
    let mut rng = Lcg::new(0x0000_0006_1AB0_0006);
    let mut spread = Spread::default();

    for &forcing_value in &interesting {
        for &stock_amount in &interesting {
            let n = rng.next_u64() % 1_000_001;
            let dt = 1e-6 + (rng.below(1_000_000) as f64) * 1e-5;
            let resolver = SourceResolver::new(
                std::collections::HashMap::from([(
                    "solar".to_string(),
                    constant(forcing_value).expect("finite constant") as Schedule,
                )]),
                std::collections::HashMap::new(),
            )
            .expect("resolver");
            let state = state_of(n, vec![pool("law.a", stock_amount)]);
            let got = resolver.bind(&state, dt).get("solar").expect("solar");
            assert_eq!(
                got.to_bits(),
                forcing_value.to_bits(),
                "a forcing read moved with the stock contents (n={n}, dt={dt}, \
                 stock={stock_amount:?})"
            );
            spread.cases += 1;
            spread.distinct_scalars.insert(stock_amount.to_bits());
            spread.lengths.insert(n as usize % 7);
        }
    }
    spread.assert_spans("law 6", 144, 2);
    assert!(
        spread.distinct_scalars.len() >= 12,
        "law 6 exercised only {} distinct stock amounts",
        spread.distinct_scalars.len()
    );
}

// --------------------------------------------------------------------------- //
// LAW 7 — `step` is registration-order independent, for both integrators.      //
// (reference: tests/test_integrator.py::test_step_is_registration_order_       //
//  independent)                                                                //
// --------------------------------------------------------------------------- //

#[test]
fn law_step_is_registration_order_independent_for_both_integrators() {
    // ⚠ The reference fixture drains its source with **two** flows, and a two-element
    // float sum is commutative — so that shape cannot be order-sensitive at all, whatever
    // the magnitudes. It takes three legs on one stock. These three drain `law.a` at rates
    // spanning sixteen orders: the delta is `(0.5 + 0.5) + 5e15` = `5e15 + 1` in flow-id
    // order and `5e15` reversed.
    const SRC: f64 = 2e16;
    const DT: f64 = 0.5;
    let fracs = [5e-17_f64, 5e-17, 0.5];
    let contributions: Vec<f64> = fracs.iter().map(|f| f * SRC * DT).collect();
    assert_order_sensitive(&contributions, "law 7 source delta");
    assert!(
        fold(&contributions) < SRC,
        "the fixture over-draws, which routes it through arbitration instead of the plain \
         step this law is about"
    );

    let stocks: BTreeMap<StockId, Stock> = [
        pool("law.a", SRC),
        pool("law.d1", 0.0),
        pool("law.d2", 0.0),
        boundary::sink("boundary.k".to_string(), Quantity::Carbon, 0.0).expect("sink"),
    ]
    .into_iter()
    .map(|s| (s.id.clone(), s))
    .collect();
    let state = State::new(0, stocks.clone(), 0, BTreeMap::new()).expect("state");
    let env = SourceResolver::empty();

    let build = || -> Vec<Box<dyn Flow>> {
        vec![
            Box::new(TransferFlow {
                id: "t1_tiny".to_string(),
                src: "law.a".to_string(),
                dst: "law.d1".to_string(),
                frac: fracs[0],
            }),
            Box::new(TransferFlow {
                id: "t2_tiny".to_string(),
                src: "law.a".to_string(),
                dst: "law.d2".to_string(),
                frac: fracs[1],
            }),
            Box::new(TransferFlow {
                id: "t3_bulk".to_string(),
                src: "law.a".to_string(),
                dst: "boundary.k".to_string(),
                frac: fracs[2],
            }),
        ]
    };

    for integrator in ["euler", "rk4"] {
        let step = |flows: Vec<Box<dyn Flow>>| -> State {
            let reg = Registry::new(flows, &stocks, Vec::new()).expect("reg");
            match integrator {
                "euler" => EulerIntegrator::new(reg).step(&state, &env, DT),
                _ => Rk4Integrator::new(reg).step(&state, &env, DT),
            }
            .expect("step")
        };
        let canonical = step(build());
        for perm in permutations(3) {
            assert_eq!(
                amounts(&step(permute(build(), &perm))),
                amounts(&canonical),
                "{integrator}: registration order {perm:?} changed the step"
            );
        }
    }
}

// --------------------------------------------------------------------------- //
// LAW 8 — a multi-rate master step is registration-order independent.          //
// (reference: tests/test_multirate.py::test_multirate_is_registration_order_   //
//  independent)                                                                //
// --------------------------------------------------------------------------- //

#[test]
fn law_multirate_is_registration_order_independent() {
    let stocks: BTreeMap<StockId, Stock> = [
        pool("law.x", 1e8),
        pool("law.y", 1.0),
        pool("law.z", 0.0),
        boundary::sink("boundary.k".to_string(), Quantity::Carbon, 0.0).expect("sink"),
    ]
    .into_iter()
    .map(|s| (s.id.clone(), s))
    .collect();
    let env = SourceResolver::empty();

    let slow_flows = || -> Vec<Box<dyn Flow>> {
        vec![Box::new(TransferFlow {
            id: "casc.yz".to_string(),
            src: "law.y".to_string(),
            dst: "law.z".to_string(),
            frac: 0.05,
        })]
    };
    let fast_flows = || -> Vec<Box<dyn Flow>> {
        vec![
            Box::new(TransferFlow {
                id: "fast.a".to_string(),
                src: "law.x".to_string(),
                dst: "law.y".to_string(),
                frac: 0.1,
            }),
            Box::new(TransferFlow {
                id: "fast.b".to_string(),
                src: "law.x".to_string(),
                dst: "law.z".to_string(),
                frac: 5e-9,
            }),
            Box::new(TransferFlow {
                id: "fast.c".to_string(),
                src: "law.y".to_string(),
                dst: "boundary.k".to_string(),
                frac: 0.02,
            }),
        ]
    };

    let run = |fast: Vec<Box<dyn Flow>>| -> Vec<(String, u64)> {
        let slow =
            Rk4Integrator::new(Registry::new(slow_flows(), &stocks, Vec::new()).expect("reg"));
        let fast = Rk4Integrator::new(Registry::new(fast, &stocks, Vec::new()).expect("reg"));
        let mut state = State::new(0, stocks.clone(), 0, BTreeMap::new()).expect("state");
        for _ in 0..5 {
            state = multirate_step(&slow, &fast, &state, &env, 0.1, 3, Split::Strang)
                .expect("multirate step")
                .state;
        }
        amounts(&state)
    };

    let canonical = run(fast_flows());
    for perm in permutations(3) {
        assert_eq!(
            run(permute(fast_flows(), &perm)),
            canonical,
            "fast-registry order {perm:?} changed the multi-rate run"
        );
    }
}

// --------------------------------------------------------------------------- //
// LAW 9 — `observe` emits stocks in canonical id order.                        //
// (reference: tests/test_observation.py::test_observe_order_independence_      //
//  property — re-expressed)                                                    //
// --------------------------------------------------------------------------- //

#[test]
fn law_observe_emits_stocks_in_canonical_id_order() {
    // ⚠ The reference law shuffles a dict's insertion order, which a `BTreeMap` makes
    // unexpressible; `observation.rs::insertion_order_independent` is that inert shape
    // already. What IS falsifiable is the emitted *sequence*: reverse the iteration in
    // `observe` and this goes red.
    let build_order = ["law.z", "law.a", "law.m"];
    let mut expected: Vec<&str> = build_order.to_vec();
    expected.sort_unstable();
    // The discriminator: the build order must NOT already be the canonical one, or the
    // assertion below holds for an implementation that simply preserved insertion order.
    assert_ne!(
        build_order.to_vec(),
        expected,
        "the fixture's build order is already sorted, so this law checks nothing"
    );

    let state = state_of(
        3,
        build_order
            .iter()
            .enumerate()
            .map(|(i, id)| pool(id, i as f64 + 1.0))
            .collect(),
    );
    let obs = observe(&state);
    let ids: Vec<&str> = obs.stocks.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, expected, "observe did not emit canonical id order");
    // And each observation still carries its own stock's amount — a sorted-but-scrambled
    // projection would pass the id check alone.
    for so in &obs.stocks {
        let want = state.stocks[&so.id].amount;
        assert_eq!(so.amount().to_bits(), want.to_bits());
    }
}

// --------------------------------------------------------------------------- //
// LAW 10 — the registry's iteration order is canonical for ARBITRARY id sets.  //
// (reference: tests/test_registry.py::test_registry_registration_order_        //
//  independence)                                                               //
// --------------------------------------------------------------------------- //

#[test]
fn law_registry_iteration_is_canonical_for_arbitrary_id_sets() {
    // The reference draws unique ids over `[a-z0-9._]`, length 1..8, list size 1..12, and
    // a permutation of them. ⚠ Its own comment says ASCII-only "so Python's str sort
    // matches the future Rust UTF-8 byte sort" — under C that future is now, and this is
    // the test on the other side of that sentence.
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789._";
    let stocks: BTreeMap<StockId, Stock> = BTreeMap::new();
    let mut rng = Lcg::new(0x513C_E10A_7A55_0001);
    let mut spread = Spread::default();

    for _case in 0..200 {
        let count = 1 + rng.below(12);
        let mut ids: Vec<String> = Vec::new();
        while ids.len() < count {
            let len = 1 + rng.below(8);
            let id: String = (0..len)
                .map(|_| ALPHABET[rng.below(ALPHABET.len())] as char)
                .collect();
            if !ids.contains(&id) {
                spread.max_item_len = spread.max_item_len.max(id.len());
                ids.push(id);
            }
        }
        let mut sorted = ids.clone();
        sorted.sort();

        let mut registration: Vec<String> = ids.clone();
        rng.shuffle(&mut registration);
        if registration != ids {
            spread.non_identity_orders += 1;
        }

        let flows: Vec<Box<dyn Flow>> = registration
            .iter()
            .map(|id| Box::new(NoopFlow(id.clone())) as Box<dyn Flow>)
            .collect();
        let reg = Registry::new(flows, &stocks, Vec::new()).expect("registry");
        let got: Vec<&str> = reg.flows().iter().map(|f| f.id()).collect();
        assert_eq!(
            got, sorted,
            "registration order {registration:?} did not iterate canonically"
        );

        spread.cases += 1;
        spread.lengths.insert(count);
    }

    spread.assert_spans("law 10", 200, 8);
    assert!(
        spread.max_item_len > 1,
        "law 10 generated only single-character ids"
    );
    assert!(
        spread.non_identity_orders > 100,
        "law 10 shuffled the registration order only {} times of 200 — the generator \
         collapsed to (near-)identity and the law is vacuous",
        spread.non_identity_orders
    );
}

// --------------------------------------------------------------------------- //
// LAW 11 — `draw` is a deterministic pure function into [0, 1).                //
// (reference: tests/test_rng.py::test_draw_is_deterministic_and_in_unit_       //
//  interval)                                                                   //
// --------------------------------------------------------------------------- //

/// 2**53 — the divisor `CounterRng::draw` maps its 53-bit integer through.
///
/// ⚠ A deliberate second copy of `rng::FLOAT_DIVISOR` (which is private). It is the
/// tolerable kind: a divergence is red, in both directions, and it is what makes the
/// bound below a statement about `draw`'s *whole domain* rather than about samples.
const FLOAT_DIVISOR: f64 = 9_007_199_254_740_992.0;

#[test]
fn law_draw_is_deterministic_and_in_the_unit_interval() {
    let mut rng = Lcg::new(0x0D8A_0000_0000_0011);
    let mut spread = Spread::default();

    for _case in 0..500 {
        let seed = rng.next_u64();
        let key_len = 1 + rng.below(3);
        let key: Vec<u64> = (0..key_len).map(|_| rng.next_u64() % 1001).collect();
        let step = rng.next_u64() % 1_000_000_001;

        let counter = CounterRng::new(seed);
        let a = counter.draw(&key, step);
        let b = counter.draw(&key, step);
        assert_eq!(a.to_bits(), b.to_bits(), "draw is not a pure function");
        assert!(
            (0.0..1.0).contains(&a),
            "draw({key:?}, {step}) = {a:?} left [0, 1)"
        );
        // The formula identity: `draw` IS `(draw_u64 >> 11) / 2**53`. Together with the
        // supremum test below this bounds `draw` over its ENTIRE domain, which sampling
        // cannot do — the extremum is exactly where a sampled law is weakest, on either
        // side of the port.
        assert_eq!(
            a.to_bits(),
            ((counter.draw_u64(&key, step) >> 11) as f64 / FLOAT_DIVISOR).to_bits(),
            "draw diverged from (draw_u64 >> 11) / 2**53"
        );

        spread.cases += 1;
        spread.lengths.insert(key_len);
        spread.distinct_scalars.insert(a.to_bits());
    }

    spread.assert_spans("law 11", 500, 3);
    assert!(
        spread.distinct_scalars.len() > 400,
        "law 11 saw only {} distinct draws in 500 cases",
        spread.distinct_scalars.len()
    );
}

#[test]
fn law_draw_cannot_reach_one_for_any_mixer_output() {
    // The supremum over the whole u64 domain, not a sample: `x >> 11` is at most 2**53-1,
    // which is exact in f64, and the divisor is a power of two, so the quotient is exact
    // and strictly below 1.0. ⚠ This is the half of "never exactly 1.0" that sampling
    // 100 (or 500) draws does not establish on either port.
    let largest = ((u64::MAX >> 11) as f64) / FLOAT_DIVISOR;
    assert!(largest < 1.0, "the maximal draw is {largest:?}, not < 1.0");
    assert_eq!((u64::MAX >> 11), (1u64 << 53) - 1);
    // Exactness of the two steps the bound leans on.
    assert_eq!(((1u64 << 53) - 1) as f64 as u64, (1u64 << 53) - 1);
    assert_eq!(FLOAT_DIVISOR.to_bits(), ((1u64 << 53) as f64).to_bits());
}

// --------------------------------------------------------------------------- //
// LAW 12 — draws are independent of the order they are requested in.           //
// (reference: tests/test_rng.py::test_draws_are_order_independent)              //
// --------------------------------------------------------------------------- //

#[test]
fn law_draws_are_order_independent() {
    // The value for a given `(key, step)` must not depend on the order draws are
    // requested in — the core of decision #12, and the reason the generator is
    // counter-based and keyed rather than sequential-state.
    let mut rng = Lcg::new(0x0D8A_0000_0000_0012);
    let mut spread = Spread::default();

    for _case in 0..200 {
        let seed = rng.next_u64();
        let count = 1 + rng.below(25);
        let mut pairs: Vec<(u64, u64)> = Vec::new();
        while pairs.len() < count {
            let pair = (rng.next_u64() % 1001, rng.next_u64() % 1001);
            if !pairs.contains(&pair) {
                pairs.push(pair);
            }
        }
        let counter = CounterRng::new(seed);
        let forward: BTreeMap<(u64, u64), u64> = pairs
            .iter()
            .map(|&(k, s)| ((k, s), counter.draw(&[k], s).to_bits()))
            .collect();

        let mut shuffled = pairs.clone();
        rng.shuffle(&mut shuffled);
        if shuffled != pairs {
            spread.non_identity_orders += 1;
        }
        let backward: BTreeMap<(u64, u64), u64> = shuffled
            .iter()
            .map(|&(k, s)| ((k, s), counter.draw(&[k], s).to_bits()))
            .collect();

        assert_eq!(
            forward, backward,
            "request order changed a draw (seed={seed}, {count} pairs)"
        );
        spread.cases += 1;
        spread.lengths.insert(count);
    }

    spread.assert_spans("law 12", 200, 10);
    assert!(
        spread.non_identity_orders > 100,
        "law 12 re-ordered the request list only {} times of 200 — a generator that \
         never shuffles makes this law compare a map with itself",
        spread.non_identity_orders
    );
}
