//! The multi-rate master-step driver — the Rust port of `tests/test_multirate.py` (Stage-3
//! slice S4).
//!
//! `multirate.rs` had **no direct Rust test**. It was reachable only through `authoring`'s
//! partition tests and the station goldens, both of which exercise it as plumbing: they
//! would notice a driver that produced different numbers, and would not notice *why*.
//!
//! ## The claims that only a direct test can make
//!
//! * **The splitting order.** Strang + RK4 on both operators is **second**-order, not
//!   fourth — the operators' non-commutativity leaves an O(dt²) term no amount of
//!   sub-integration removes. Swap either operator for Euler and the composite collapses to
//!   first order, silently: it still runs, still conserves, still looks right. That silent
//!   collapse is the thing this file exists to catch, and the goldens cannot see it because
//!   they pin one `dt`.
//! * **The step-count contract.** `n` advances by exactly 1 per master step whatever `n_sub`
//!   is — sub-steps are internal. Time is `n·dt`, so an `n` that counted sub-steps would
//!   corrupt every forcing schedule in the tree while conserving mass perfectly.
//! * **The composite conservation gate.** Sub-steps deliberately skip the per-operation
//!   balance assert, so an unbalanced sub-delta must be caught at the master boundary or
//!   nowhere.
//! * **Extinction through the split** — per sub-operation, aggregated into one master
//!   report, with the events re-stamped to the committed `n`.
//!
//! ⚠ **The slow set does not step at `dt/n_sub`.** Under Strang it steps at `dt/2`,
//! independent of `n_sub` — `ops = [(slow, dt/2), …fast…, (slow, dt/2)]`. This repo has a
//! recorded case of that fact being got wrong twice in one phase (a performance prediction,
//! then a safety predicate that false-PASSED). Before this file, **nothing in the workspace
//! could see a wrong slow half-step size** — the acceptance battery measured that mutation at
//! 0 of 741 red. `the_two_strang_slow_halves_sum_to_dt_whatever_n_sub_is` is the gate on it,
//! and its doc comment records why the obvious candidates are all blind.
//!
//! ## Deliberately not here
//!
//! * Registration-order independence is `laws.rs::law_multirate_is_registration_order_independent`.
//! * "Both concrete integrators satisfy `Substepper`" is unrepresentable in Rust — a type
//!   that does not implement the trait does not compile.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use simcore::boundary;
use simcore::environment::{Environment, SourceResolver};
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::integrator::{EulerIntegrator, Rk4Integrator, Substepper};
use simcore::multirate::{multirate_step, Split};
use simcore::quantities::{Quantity, StockKind};
use simcore::registry::Registry;
use simcore::state::{State, Stock};

const CASC: &str = "casc";
const X: &str = "casc.x";
const Y: &str = "casc.y";
const Z: &str = "casc.z";
const W: &str = "casc.w";
const POP: &str = "casc.pop";
const POP_SINK: &str = "boundary.casc_popsink";

const X0: f64 = 1.0;
const Y0: f64 = 0.5;
const Z0: f64 = 0.0;
/// `k_f != k_s` so the closed form below is non-degenerate.
const KF: f64 = 1.0;
const KS: f64 = 0.3;
const T_END: f64 = 1.0;
/// A geometric (halving) ladder: `T_END/dt` is an integer at every rung, and the finest
/// Strang error (~1e-6) stays far above the f64 round-off floor — so the fitted slope
/// measures splitting error rather than numerical noise.
const DTS: [f64; 4] = [0.1, 0.05, 0.025, 0.0125];

/// The two schemes as a value — see `extinction.rs` for why a `simcore` test dispatches by
/// hand instead of over a trait.
#[derive(Debug, Clone, Copy)]
enum Scheme {
    Euler,
    Rk4,
}

impl Scheme {
    fn boxed(self, registry: Registry) -> Box<dyn Substepper> {
        match self {
            Scheme::Euler => Box::new(EulerIntegrator::new(registry)),
            Scheme::Rk4 => Box::new(Rk4Integrator::new(registry)),
        }
    }

    fn step(self, registry: Registry, state: &State, env: &SourceResolver, dt: f64) -> State {
        match self {
            Scheme::Euler => EulerIntegrator::new(registry).step(state, env, dt),
            Scheme::Rk4 => Rk4Integrator::new(registry).step(state, env, dt),
        }
        .expect("single-rate step")
    }
}

/// `src -> dst` at first-order `rate` (`leg == dt·rate·src`; balanced).
struct Cascade {
    id: String,
    src: String,
    dst: String,
    rate: f64,
    /// Tallies `evaluate` calls when present — the instrument for the eval-count case.
    calls: Option<Arc<AtomicUsize>>,
}

impl Cascade {
    fn new(id: &str, src: &str, dst: &str, rate: f64) -> Cascade {
        Cascade {
            id: id.to_string(),
            src: src.to_string(),
            dst: dst.to_string(),
            rate,
            calls: None,
        }
    }

    fn counting(id: &str, src: &str, dst: &str, rate: f64, calls: Arc<AtomicUsize>) -> Cascade {
        Cascade {
            calls: Some(calls),
            ..Cascade::new(id, src, dst, rate)
        }
    }
}

impl Flow for Cascade {
    fn type_name(&self) -> &'static str {
        "Cascade"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, snapshot: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        if let Some(calls) = &self.calls {
            calls.fetch_add(1, Ordering::SeqCst);
        }
        let moved = self.rate * snapshot.stocks[&self.src].amount * dt;
        FlowResult::new(vec![
            Leg::new(self.src.clone(), -moved)?,
            Leg::new(self.dst.clone(), moved)?,
        ])
    }
}

/// A deliberately **broken** flow: it deposits mass with no compensating withdrawal, so
/// `Σ legs != 0`. Injected into a sub-registry to prove the composite gate — and only it,
/// since sub-steps skip the per-operation assert — trips.
struct UnbalancedDeposit {
    id: String,
    dst: String,
    amount: f64,
}

impl Flow for UnbalancedDeposit {
    fn type_name(&self) -> &'static str {
        "UnbalancedDeposit"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, _snapshot: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        FlowResult::new(vec![Leg::new(self.dst.clone(), self.amount * dt)?])
    }
}

/// `pop -> sink` at `rate·pop·dt` — **proportional**, so it stops drawing once `pop` snaps to
/// zero and never over-draws the extinct stock.
struct PopDrain {
    id: String,
    pop: String,
    sink: String,
    rate: f64,
}

impl Flow for PopDrain {
    fn type_name(&self) -> &'static str {
        "PopDrain"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, snapshot: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let moved = self.rate * snapshot.stocks[&self.pop].amount * dt;
        FlowResult::new(vec![
            Leg::new(self.pop.clone(), -moved)?,
            Leg::new(self.sink.clone(), moved)?,
        ])
    }
}

fn pool(id: &str, amount: f64) -> Stock {
    Stock::new(
        id.to_string(),
        CASC.to_string(),
        Quantity::Carbon,
        Quantity::Carbon.canonical_unit(),
        amount,
        StockKind::Pool,
        0.0,
        false,
        BTreeMap::new(),
    )
    .expect("pool")
}

fn cascade_stocks() -> BTreeMap<String, Stock> {
    [pool(X, X0), pool(Y, Y0), pool(Z, Z0)]
        .into_iter()
        .map(|s| (s.id.clone(), s))
        .collect()
}

fn fast_flow() -> Box<dyn Flow> {
    Box::new(Cascade::new("casc.xy", X, Y, KF))
}

fn slow_flow() -> Box<dyn Flow> {
    Box::new(Cascade::new("casc.yz", Y, Z, KS))
}

/// Closed-form `y(t)` of the cascade `ẋ = −k_f x`, `ẏ = k_f x − k_s y` (for `k_f ≠ k_s`):
/// `y(t) = y0·e^{−k_s t} + k_f·x0·(e^{−k_f t} − e^{−k_s t}) / (k_s − k_f)`.
fn exact_y(t: f64) -> f64 {
    Y0 * (-KS * t).exp() + KF * X0 * ((-KF * t).exp() - (-KS * t).exp()) / (KS - KF)
}

/// Least-squares slope of `log(error)` against `log(dt)` — the observed order of accuracy.
fn fit_order(dts: &[f64], errors: &[f64]) -> f64 {
    let n = dts.len() as f64;
    let xs: Vec<f64> = dts.iter().map(|d| d.ln()).collect();
    let ys: Vec<f64> = errors.iter().map(|e| e.ln()).collect();
    let mean_x = xs.iter().sum::<f64>() / n;
    let mean_y = ys.iter().sum::<f64>() / n;
    let num: f64 = xs.iter().zip(&ys).map(|(x, y)| (x - mean_x) * (y - mean_y)).sum();
    let den: f64 = xs.iter().map(|x| (x - mean_x).powi(2)).sum();
    num / den
}

// --------------------------------------------------------------------------- //
// Equivalence and asymmetry against single-rate                                //
// --------------------------------------------------------------------------- //

/// All flows fast, `n_sub == 1`, an empty slow set: the two slow half-steps are no-ops and
/// the single full fast sub-step computes the same deltas as single-rate, so the produced
/// state matches **bit for bit** — amounts and `n` alike. The anchor that says the driver
/// adds nothing of its own on the degenerate partition.
fn all_fast_nsub1_matches_single_rate(scheme: Scheme) {
    let stocks = cascade_stocks();
    let state = State::new(0, stocks.clone(), 0, BTreeMap::new()).unwrap();
    let env = SourceResolver::empty();

    let single = scheme.step(
        Registry::flows_only(vec![fast_flow(), slow_flow()], &stocks).unwrap(),
        &state,
        &env,
        0.1,
    );
    let fast = scheme.boxed(Registry::flows_only(vec![fast_flow(), slow_flow()], &stocks).unwrap());
    let slow = scheme.boxed(Registry::flows_only(Vec::new(), &stocks).unwrap());
    let multi = multirate_step(slow.as_ref(), fast.as_ref(), &state, &env, 0.1, 1, Split::Strang)
        .expect("master step")
        .state;

    assert_eq!(multi.n, 1);
    assert_eq!(single.n, 1);
    for (id, stock) in &single.stocks {
        assert_eq!(
            multi.stocks[id].amount.to_bits(),
            stock.amount.to_bits(),
            "{scheme:?}: {id} differs bit for bit"
        );
    }
}

#[test]
fn all_fast_with_one_substep_reproduces_a_single_rate_euler_step_bitwise() {
    all_fast_nsub1_matches_single_rate(Scheme::Euler);
}

#[test]
fn all_fast_with_one_substep_reproduces_a_single_rate_rk4_step_bitwise() {
    all_fast_nsub1_matches_single_rate(Scheme::Rk4);
}

/// The asymmetry, and it is not a defect: with every flow in the *slow* set a Strang master
/// step is `slow(dt/2)` then `slow(dt/2)` — two half steps, which differ from one full `dt`
/// step at O(dt²). So, unlike the all-fast case, this does **not** reproduce single-rate.
/// Stated as a test because the natural expectation is that it would.
#[test]
fn all_slow_strang_does_not_reproduce_a_single_rate_step() {
    let stocks = cascade_stocks();
    let state = State::new(0, stocks.clone(), 0, BTreeMap::new()).unwrap();
    let env = SourceResolver::empty();

    let single = Scheme::Euler.step(
        Registry::flows_only(vec![fast_flow(), slow_flow()], &stocks).unwrap(),
        &state,
        &env,
        0.1,
    );
    let slow = EulerIntegrator::new(Registry::flows_only(vec![fast_flow(), slow_flow()], &stocks).unwrap());
    let fast = EulerIntegrator::new(Registry::flows_only(Vec::new(), &stocks).unwrap());
    let multi = multirate_step(&slow, &fast, &state, &env, 0.1, 1, Split::Strang)
        .expect("master step")
        .state;

    assert!(
        (multi.stocks[Y].amount - single.stocks[Y].amount).abs() > 1e-6,
        "the two-half-step composite must differ well above the float floor"
    );
}

// --------------------------------------------------------------------------- //
// Conservation, and the tripwire at the composite boundary                     //
// --------------------------------------------------------------------------- //

/// The coupled cascade conserves total carbon at every master step. The composite gate runs
/// inside `multirate_step`, so completing the run is already the assertion; the explicit
/// total pins the closure to the float floor rather than to whatever the gate's tolerance is.
#[test]
fn the_coupled_cascade_conserves_at_every_master_step() {
    let stocks = cascade_stocks();
    let mut state = State::new(0, stocks.clone(), 0, BTreeMap::new()).unwrap();
    let env = SourceResolver::empty();
    let slow = Rk4Integrator::new(Registry::flows_only(vec![slow_flow()], &stocks).unwrap());
    let fast = Rk4Integrator::new(Registry::flows_only(vec![fast_flow()], &stocks).unwrap());
    let total0 = X0 + Y0 + Z0;

    for step in 0..40 {
        state = multirate_step(&slow, &fast, &state, &env, 0.1, 3, Split::Strang)
            .expect("master step")
            .state;
        let total: f64 = state.stocks.values().map(|s| s.amount).sum();
        assert!((total - total0).abs() < 1e-12, "step {step}: total drifted to {total}");
    }
}

/// An unbalanced sub-delta raises at the **composite** boundary. Sub-steps skip the
/// per-operation assert by design, so if this gate were missing the violation would reach
/// the committed state with nothing complaining.
#[test]
fn an_unbalanced_sub_delta_trips_the_composite_gate() {
    let stocks = cascade_stocks();
    let state = State::new(0, stocks.clone(), 0, BTreeMap::new()).unwrap();
    let env = SourceResolver::empty();
    let fast = EulerIntegrator::new(
        Registry::flows_only(
            vec![
                fast_flow(),
                Box::new(UnbalancedDeposit {
                    id: "casc.bad".to_string(),
                    dst: X.to_string(),
                    amount: 5.0,
                }),
            ],
            &stocks,
        )
        .unwrap(),
    );
    let slow = EulerIntegrator::new(Registry::flows_only(vec![slow_flow()], &stocks).unwrap());

    match multirate_step(&slow, &fast, &state, &env, 0.1, 2, Split::Strang) {
        Err(SimError::Conservation(msg)) => assert!(msg.contains("conservation"), "{msg}"),
        other => panic!("the composite gate did not trip: {:?}", other.is_ok()),
    }
}

/// A **constant-rate** deposit — `amount·dt` regardless of state — so the arithmetic below is
/// exact rather than approximate. Balanced against an unclamped boundary source, so nothing
/// arbitrates.
struct ConstDeposit {
    id: String,
    src: String,
    dst: String,
    amount: f64,
}

impl Flow for ConstDeposit {
    fn type_name(&self) -> &'static str {
        "ConstDeposit"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(&self, _snapshot: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let moved = self.amount * dt;
        FlowResult::new(vec![
            Leg::new(self.src.clone(), -moved)?,
            Leg::new(self.dst.clone(), moved)?,
        ])
    }
}

/// **The two Strang slow halves sum to `dt`, whatever `n_sub` is.**
///
/// ⚠ This case exists because the acceptance battery found that nothing in the workspace
/// could see a wrong slow half-step *size*. Changing `ops` from `(slow, dt/2)` to
/// `(slow, dt/n_sub)` reddened **0 of 741** tests: every order-of-accuracy case above runs at
/// `n_sub == 2`, where `dt/n_sub` *is* `dt/2` and the mutation is literally a no-op; the
/// conservation, determinism and eval-count cases run at other `n_sub` but assert quantities
/// a wrong step size does not move. `authoring`'s
/// `a_non_empty_slow_set_is_driven_at_dt_over_2` has the behaviour in its **name** and
/// asserts only that the slow flow ran at all.
///
/// So the discriminator has to be `n_sub`-sensitive *and* exact. With a constant-rate flow,
/// an empty fast set and `n_sub = 3`, the slow operator must move exactly `amount·dt` —
/// `dt/2 + dt/2`. Under the wrong step size it moves `2·dt/3` of that, which no tolerance
/// hides and which is what a real scenario would silently under-integrate.
#[test]
fn the_two_strang_slow_halves_sum_to_dt_whatever_n_sub_is() {
    const SRC: &str = "boundary.casc_src";
    let (dt, rate) = (1.0, 0.25);
    let stocks: BTreeMap<String, Stock> = [
        boundary::source(SRC.to_string(), Quantity::Carbon, 1000.0, true).unwrap(),
        pool(Z, 0.0),
    ]
    .into_iter()
    .map(|s| (s.id.clone(), s))
    .collect();
    let state = State::new(0, stocks.clone(), 0, BTreeMap::new()).unwrap();
    let env = SourceResolver::empty();

    let slow = EulerIntegrator::new(
        Registry::flows_only(
            vec![Box::new(ConstDeposit {
                id: "casc.deposit".to_string(),
                src: SRC.to_string(),
                dst: Z.to_string(),
                amount: rate,
            })],
            &stocks,
        )
        .unwrap(),
    );
    let fast = EulerIntegrator::new(Registry::flows_only(Vec::new(), &stocks).unwrap());

    for n_sub in [1u32, 2, 3, 5] {
        let out = multirate_step(&slow, &fast, &state, &env, dt, n_sub, Split::Strang)
            .expect("master step")
            .state;
        assert_eq!(
            out.stocks[Z].amount,
            rate * dt,
            "n_sub={n_sub}: the two slow halves must sum to dt, not to 2*dt/{n_sub}"
        );
    }
}

// --------------------------------------------------------------------------- //
// Order of accuracy — the silent collapse                                      //
// --------------------------------------------------------------------------- //

/// Run the cascade to `T_END` under the given split/schemes and return `|y − y_exact|`.
///
/// `y` is the metric because it is the stock **both** operators touch, so it carries the
/// splitting error directly. `x` would show only the fast scheme's own order — it evolves
/// under the fast operator alone — which is exactly the measurement that would report a
/// collapsed composite as fine.
fn order_error(slow_scheme: Scheme, fast_scheme: Scheme, split: Split, n_sub: u32, dt: f64) -> f64 {
    let stocks = cascade_stocks();
    let mut state = State::new(0, stocks.clone(), 0, BTreeMap::new()).unwrap();
    let slow = slow_scheme.boxed(Registry::flows_only(vec![slow_flow()], &stocks).unwrap());
    let fast = fast_scheme.boxed(Registry::flows_only(vec![fast_flow()], &stocks).unwrap());
    let env = SourceResolver::empty();
    for _ in 0..(T_END / dt).round() as u64 {
        state = multirate_step(slow.as_ref(), fast.as_ref(), &state, &env, dt, n_sub, split)
            .expect("master step")
            .state;
    }
    (state.stocks[Y].amount - exact_y(T_END)).abs()
}

/// Strang + RK4 on **both** operators is second-order globally — `min(2, 4, 4)`. Not fourth:
/// the operators' non-commutativity is an O(dt²) term no sub-integration removes.
#[test]
fn strang_with_rk4_on_both_operators_is_second_order() {
    let errors: Vec<f64> = DTS
        .iter()
        .map(|dt| order_error(Scheme::Rk4, Scheme::Rk4, Split::Strang, 2, *dt))
        .collect();
    assert!(
        errors.windows(2).all(|w| w[1] < w[0]),
        "refining dt must reduce the error monotonically: {errors:?}"
    );
    let p = fit_order(&DTS, &errors);
    assert!((1.7..2.3).contains(&p), "fitted order {p} is not 2 ({errors:?})");
}

/// Strang + **Euler** on both collapses to first order — `min(2, 1, 1)`. Strang forfeits the
/// second order it was chosen for, and nothing about the run announces it: it converges, it
/// conserves, it just converges an order slower. This is the case the whole file is for.
#[test]
fn strang_with_euler_on_both_operators_collapses_to_first_order() {
    let errors: Vec<f64> = DTS
        .iter()
        .map(|dt| order_error(Scheme::Euler, Scheme::Euler, Split::Strang, 2, *dt))
        .collect();
    let p = fit_order(&DTS, &errors);
    assert!((0.8..1.3).contains(&p), "fitted order {p} is not 1 ({errors:?})");
}

/// Lie + RK4 on both is first-order too: here the **split** caps the composite, not the
/// scheme. The pair with the case above separates the two causes of an order-1 composite,
/// which a single measurement cannot.
#[test]
fn the_lie_split_is_first_order_even_with_rk4_on_both_operators() {
    let errors: Vec<f64> = DTS
        .iter()
        .map(|dt| order_error(Scheme::Rk4, Scheme::Rk4, Split::Lie, 2, *dt))
        .collect();
    let p = fit_order(&DTS, &errors);
    assert!((0.8..1.3).contains(&p), "fitted order {p} is not 1 ({errors:?})");
}

// --------------------------------------------------------------------------- //
// The step-count contract                                                      //
// --------------------------------------------------------------------------- //

/// After `k` master steps `state.n == k`, for any `n_sub` — sub-steps are internal. Time is
/// `n·dt`, so an `n` that counted sub-steps would silently rescale every forcing schedule in
/// the tree while conserving mass perfectly.
fn n_advances_once_per_master_step(n_sub: u32) {
    let stocks = cascade_stocks();
    let mut state = State::new(0, stocks.clone(), 0, BTreeMap::new()).unwrap();
    let env = SourceResolver::empty();
    let slow = Rk4Integrator::new(Registry::flows_only(vec![slow_flow()], &stocks).unwrap());
    let fast = Rk4Integrator::new(Registry::flows_only(vec![fast_flow()], &stocks).unwrap());
    for k in 1..=12u64 {
        state = multirate_step(&slow, &fast, &state, &env, 0.1, n_sub, Split::Strang)
            .expect("master step")
            .state;
        assert_eq!(state.n, k, "n_sub={n_sub}");
    }
}

#[test]
fn n_advances_once_per_master_step_with_one_substep() {
    n_advances_once_per_master_step(1);
}

#[test]
fn n_advances_once_per_master_step_with_two_substeps() {
    n_advances_once_per_master_step(2);
}

#[test]
fn n_advances_once_per_master_step_with_five_substeps() {
    n_advances_once_per_master_step(5);
}

/// `n_sub < 1` is a scenario bug — no fast sub-steps at all — and is refused at the call
/// rather than treated as "run the slow half-steps only".
#[test]
fn a_non_positive_n_sub_is_refused() {
    let stocks = cascade_stocks();
    let state = State::new(0, stocks.clone(), 0, BTreeMap::new()).unwrap();
    let env = SourceResolver::empty();
    let slow = Rk4Integrator::new(Registry::flows_only(vec![slow_flow()], &stocks).unwrap());
    let fast = Rk4Integrator::new(Registry::flows_only(vec![fast_flow()], &stocks).unwrap());

    match multirate_step(&slow, &fast, &state, &env, 0.1, 0, Split::Strang) {
        Err(SimError::Validation(msg)) => assert!(msg.contains("n_sub"), "{msg}"),
        other => panic!("n_sub=0 was accepted: {:?}", other.is_ok()),
    }
}

// --------------------------------------------------------------------------- //
// Determinism, and the efficiency the driver exists for                        //
// --------------------------------------------------------------------------- //

fn multi_stocks() -> BTreeMap<String, Stock> {
    [pool(X, X0), pool(Y, Y0), pool(Z, Z0), pool(W, 0.4)]
        .into_iter()
        .map(|s| (s.id.clone(), s))
        .collect()
}

fn run_multi() -> BTreeMap<String, u64> {
    let stocks = multi_stocks();
    let mut state = State::new(0, stocks.clone(), 0, BTreeMap::new()).unwrap();
    let env = SourceResolver::empty();
    let slow = Rk4Integrator::new(
        Registry::flows_only(
            vec![
                Box::new(Cascade::new("casc.wz", W, Z, 0.1)),
                Box::new(Cascade::new("casc.yz", Y, Z, 0.15)),
            ],
            &stocks,
        )
        .unwrap(),
    );
    let fast = Rk4Integrator::new(
        Registry::flows_only(
            vec![
                Box::new(Cascade::new("casc.xy", X, Y, 0.5)),
                Box::new(Cascade::new("casc.xw", X, W, 0.3)),
                Box::new(Cascade::new("casc.yw", Y, W, 0.2)),
            ],
            &stocks,
        )
        .unwrap(),
    );
    for _ in 0..6 {
        state = multirate_step(&slow, &fast, &state, &env, 0.1, 3, Split::Strang)
            .expect("master step")
            .state;
    }
    state
        .stocks
        .iter()
        .map(|(id, s)| (id.clone(), s.amount.to_bits()))
        .collect()
}

#[test]
fn two_identical_multirate_runs_are_bit_identical() {
    assert_eq!(run_multi(), run_multi());
}

/// The efficiency multi-rate exists for, as a **count** rather than a wall clock: with
/// `n_sub` fast sub-steps the slow flow is evaluated only in the two Strang halves
/// (2 × 4 RK4 stages = 8), *independent of `n_sub`*, where single-rate at the fast `dt`
/// evaluates it every sub-step (`n_sub` × 4 = 16).
///
/// ⚠ **What the `8` does and does not pin.** It pins that the number of slow *evaluations*
/// is `n_sub`-independent — two slow ops, four RK4 stages each — which is the efficiency
/// claim. It does **not** detect a slow half-step of the wrong *size*: `ops` holds exactly
/// two slow entries whatever `n_sub` is, so changing `dt/2` to `dt/n_sub` leaves this count
/// at 8. The order-of-accuracy cases above are what see that. Spelled out because the first
/// draft of this comment claimed the opposite, in the file whose header warns about exactly
/// this confusion — the third recorded instance of reasoning about `n_sub` as though it
/// governed the slow rate class.
#[test]
fn the_slow_flow_is_evaluated_fewer_times_than_single_rate_at_the_fast_dt() {
    let (n_sub, dt) = (4u32, 0.1);

    let mr_calls = Arc::new(AtomicUsize::new(0));
    let stocks = cascade_stocks();
    let state = State::new(0, stocks.clone(), 0, BTreeMap::new()).unwrap();
    let env = SourceResolver::empty();
    let slow = Rk4Integrator::new(
        Registry::flows_only(
            vec![Box::new(Cascade::counting("casc.yz", Y, Z, KS, mr_calls.clone()))],
            &stocks,
        )
        .unwrap(),
    );
    let fast = Rk4Integrator::new(Registry::flows_only(vec![fast_flow()], &stocks).unwrap());
    multirate_step(&slow, &fast, &state, &env, dt, n_sub, Split::Strang).expect("master step");

    let sr_calls = Arc::new(AtomicUsize::new(0));
    let stocks2 = cascade_stocks();
    let mut state2 = State::new(0, stocks2.clone(), 0, BTreeMap::new()).unwrap();
    let combined = Rk4Integrator::new(
        Registry::flows_only(
            vec![
                Box::new(Cascade::counting("casc.yz", Y, Z, KS, sr_calls.clone())),
                fast_flow(),
            ],
            &stocks2,
        )
        .unwrap(),
    );
    for _ in 0..n_sub {
        state2 = combined
            .step(&state2, &env, dt / n_sub as f64)
            .expect("single-rate step");
    }

    assert_eq!(
        mr_calls.load(Ordering::SeqCst),
        8,
        "two Strang slow halves x four RK4 stages, independent of n_sub"
    );
    assert_eq!(sr_calls.load(Ordering::SeqCst), (n_sub as usize) * 4);
    assert!(mr_calls.load(Ordering::SeqCst) < sr_calls.load(Ordering::SeqCst));
    assert_eq!(state2.n, n_sub as u64, "the single-rate comparison really ran");
}

// --------------------------------------------------------------------------- //
// Extinction through the split                                                 //
// --------------------------------------------------------------------------- //

/// A POPULATION stock driven below threshold **inside a fast sub-step** snaps to 0, routes
/// its residual, has its event aggregated into the master report and **re-stamped** to the
/// committed `n`, and the composite gate still closes.
///
/// This is the multi-rate-specific extinction wiring that no POOL scenario reaches:
/// cross-sub-step event aggregation, the `n` re-stamp (the run starts at `n = 3` precisely
/// so a report that forgot to re-stamp would show 3 rather than 4), the once-per-master-step
/// firing — the `amount != 0` guard has to stop a re-fire in the *later* sub-steps of the
/// same master step — and that the snap stays mass-conserving across the split.
#[test]
fn extinction_inside_a_substep_is_aggregated_restamped_and_conserving() {
    let pop = Stock::new(
        POP.to_string(),
        CASC.to_string(),
        Quantity::Carbon,
        Quantity::Carbon.canonical_unit(),
        0.6,
        StockKind::Population,
        0.5,
        false,
        BTreeMap::new(),
    )
    .expect("population");
    let loss_id = boundary::loss_sink_id(Quantity::Carbon);
    let stocks: BTreeMap<String, Stock> = [
        pop,
        boundary::sink(POP_SINK.to_string(), Quantity::Carbon, 0.0).unwrap(),
        boundary::loss_sink(Quantity::Carbon, 0.0).unwrap(),
    ]
    .into_iter()
    .map(|s| (s.id.clone(), s))
    .collect();
    // Start at n = 3 to pin the re-stamp.
    let state = State::new(3, stocks.clone(), 0, BTreeMap::new()).unwrap();
    let env = SourceResolver::empty();
    let total0: f64 = state.stocks.values().map(|s| s.amount).sum();

    let fast = EulerIntegrator::new(
        Registry::flows_only(
            vec![Box::new(PopDrain {
                id: "casc.drain".to_string(),
                pop: POP.to_string(),
                sink: POP_SINK.to_string(),
                rate: 0.5,
            })],
            &stocks,
        )
        .unwrap(),
    );
    let slow = EulerIntegrator::new(Registry::flows_only(Vec::new(), &stocks).unwrap());

    let report = multirate_step(&slow, &fast, &state, &env, 1.0, 4, Split::Strang).expect("master step");

    assert_eq!(report.state.n, 4);
    assert_eq!(report.state.stocks[POP].amount, 0.0, "the population snapped");
    assert_eq!(
        report.events.len(),
        1,
        "one event for the master step, not one per sub-step: {:?}",
        report.events
    );
    assert_eq!(report.events[0].n, 4, "the event was re-stamped to the committed n");
    assert_eq!(report.events[0].stock, POP);
    assert!(
        report.state.stocks[&loss_id].amount > 0.0,
        "the residual reached the loss-sink"
    );
    let total: f64 = report.state.stocks.values().map(|s| s.amount).sum();
    assert!(
        (total - total0).abs() < 1e-12,
        "the snap stays mass-conserving across the split ({total} vs {total0})"
    );
}
