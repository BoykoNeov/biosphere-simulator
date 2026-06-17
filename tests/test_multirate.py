"""Step-3 multi-rate sub-stepping gate (Phase 0.5) — the locked N2–N5 contract.

Exercises ``simcore.multirate.multirate_step`` (the operator-splitting driver) and
the ``substep`` primitive it composes. The properties under test, mapped to the
plan's Step-3 test plan:

* **Equivalence** — with ``n_sub == 1`` and an empty slow registry (all flows fast),
  a Strang master step reproduces the single-rate ``step`` **bit-for-bit** (the slow
  halves are no-ops; the one full fast sub-step *is* the single-rate step). The
  *all-slow* degenerate case deliberately does **not** reproduce single-rate (two
  ``dt/2`` halves ≠ one ``dt`` step at O(``dt²``) — the asymmetry worth noting).
* **Conservation (N4/N5)** — a coupled fast/slow scenario conserves total carbon at
  every master step (the composite gate runs inside ``multirate_step``); a
  deliberately *unbalanced* injected sub-delta trips that **boundary** gate (sub-steps
  skip the per-operation assert, so the tripwire fires at the composite).
* **Order (N4)** — on a non-commuting cascade with an analytic reference,
  Strang+RK4-on-**both** → 2nd-order (*not* 4th — the split caps it); Strang+**Euler**
  → 1st (the silent collapse to Lie); **Lie** → 1st. Fitted via the Step-1 harness
  (``lab.convergence.fit_order``).
* **``n`` accounting (N2)** — after ``k`` master steps ``state.n == k`` regardless of
  ``n_sub`` (sub-steps are internal; #14 preserved).
* **Determinism (#7/#15)** — bit-identical across runs and under flow-registration
  shuffle *within* each registry.
* **Speedup sanity** — a fast/slow split with ``n_sub > 1`` does **fewer** slow-flow
  evaluations than single-rate at the fast ``dt`` (the efficiency the feature exists
  for) — a count assertion, not a wall-clock one.

**The scenario — a non-commuting linear cascade ``x → y → z``.** Three CARBON
``POOL`` stocks; the **fast** operator is ``x → y`` (rate ``k_f·x``), the **slow**
operator is ``y → z`` (rate ``k_s·y``). They share stock ``y`` and therefore do
**not** commute (``[L_fast, L_slow] ≠ 0``) — which is what makes Strang exhibit its
true 2nd order rather than RK4's 4th (commuting operators would hide the split error).
The cascade is closed (``x + y + z`` constant), so it conserves carbon with no
boundary reservoir, and it has a closed-form solution (below) for the order fit. All
flows are first-order and ``dt``-linear (``leg == dt·rate``), so RK4 keeps its order
within each operator — leaving the split as the sole order-limiter. Test-local flow
dataclasses, mirroring the ``_DecayFlow`` / oscillator precedent.
"""

import dataclasses
import math

import pytest
from hypothesis import given
from hypothesis import strategies as st

from lab.convergence import fit_order
from simcore import boundary
from simcore.environment import Environment, SourceResolver
from simcore.flow import ConservationError, FlowResult, Leg
from simcore.ids import DomainId, FlowId, StockId
from simcore.integrator import EulerIntegrator, Rk4Integrator, Substepper
from simcore.multirate import Split, multirate_step
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock

# --- ids + cascade parameters ----------------------------------------------
CASC = DomainId("casc")
X = StockId("casc.x")
Y = StockId("casc.y")
Z = StockId("casc.z")
W = StockId("casc.w")  # only for the multi-flow determinism scenario
POP = StockId("casc.pop")  # only for the extinction-through-the-split test
POP_SINK = StockId("boundary.casc_popsink")

X0, Y0, Z0 = 1.0, 0.5, 0.0
KF, KS = 1.0, 0.3  # k_f ≠ k_s so the closed-form below is non-degenerate
T_END = 1.0
# Geometric (halving) dt ladder; T_END/dt is integer at every rung and the finest
# Strang error (~1e-6) stays far above the f64 round-off floor, so the fitted slope
# measures splitting error, not numerical noise (the Step-1 "stay above the floor"
# discipline).
DTS = (0.1, 0.05, 0.025, 0.0125)


# --- test-local flows ------------------------------------------------------
@dataclasses.dataclass(frozen=True)
class _Cascade:
    """``src -> dst`` at first-order ``rate`` (``leg == dt·rate·src``; balanced)."""

    id: FlowId
    priority: int
    src: StockId
    dst: StockId
    rate: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        moved = self.rate * snapshot.stocks[self.src].amount * dt
        return FlowResult(legs=(Leg(self.src, -moved), Leg(self.dst, moved)))


@dataclasses.dataclass(frozen=True)
class _CountingCascade:
    """A ``_Cascade`` that tallies its ``evaluate`` calls into a shared counter.

    ``calls`` is a one-element list (a frozen dataclass may hold a mutable
    reference); ``calls[0]`` is incremented on every evaluation. Used only by the
    speedup-sanity test to count slow-flow evaluations.
    """

    id: FlowId
    priority: int
    src: StockId
    dst: StockId
    rate: float
    calls: list[int]

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        self.calls[0] += 1
        moved = self.rate * snapshot.stocks[self.src].amount * dt
        return FlowResult(legs=(Leg(self.src, -moved), Leg(self.dst, moved)))


@dataclasses.dataclass(frozen=True)
class _UnbalancedDeposit:
    """A deliberately **broken** flow: deposits mass with no compensating withdrawal.

    ``Σ legs != 0`` — it conserves nothing. Injected into a sub-registry to prove the
    composite conservation gate (and *only* it — sub-steps skip the assert) trips.
    """

    id: FlowId
    priority: int
    dst: StockId
    amount: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        return FlowResult(legs=(Leg(self.dst, self.amount * dt),))


@dataclasses.dataclass(frozen=True)
class _PopDrain:
    """``pop -> sink`` (boundary) at ``rate·pop·dt`` — self-limiting (proportional).

    Proportional so it stops drawing once ``pop`` snaps to 0 (no over-draw on the
    extinct stock); used to drive a POPULATION stock under its threshold inside a
    fast sub-step.
    """

    id: FlowId
    priority: int
    pop: StockId
    sink: StockId
    rate: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        moved = self.rate * snapshot.stocks[self.pop].amount * dt
        return FlowResult(legs=(Leg(self.pop, -moved), Leg(self.sink, moved)))


# --- builders --------------------------------------------------------------
def _pool(sid: StockId, amount: float) -> Stock:
    return Stock(
        id=sid,
        domain=CASC,
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=amount,
        kind=StockKind.POOL,
    )


def _cascade_stocks() -> dict[StockId, Stock]:
    return {X: _pool(X, X0), Y: _pool(Y, Y0), Z: _pool(Z, Z0)}


def _fast_flow() -> _Cascade:
    return _Cascade(FlowId("casc.xy"), 0, X, Y, KF)


def _slow_flow() -> _Cascade:
    return _Cascade(FlowId("casc.yz"), 0, Y, Z, KS)


def _exact_y(t: float) -> float:
    """Closed-form ``y(t)`` of the cascade ``ẋ=-k_f x``, ``ẏ=k_f x − k_s y`` (k_f≠k_s).

    ``y(t) = y0·e^{−k_s t} + k_f·x0·(e^{−k_f t} − e^{−k_s t})/(k_s − k_f)``.
    """
    return Y0 * math.exp(-KS * t) + KF * X0 * (
        math.exp(-KF * t) - math.exp(-KS * t)
    ) / (KS - KF)


# --- equivalence: all-fast n_sub=1 reproduces single-rate (bit-identical) ---
@pytest.mark.parametrize("cls", [EulerIntegrator, Rk4Integrator])
def test_all_fast_nsub1_reproduces_single_rate_bitwise(cls: type) -> None:
    """All flows fast + ``n_sub == 1`` + empty slow ⇒ bit-identical to ``step``.

    The slow half-steps are no-ops (empty registry) and the single full fast sub-step
    computes the *same* deltas as single-rate (both go through the integrator's shared
    ``_deltas``), so the produced state matches exactly — amounts and ``n``.
    """
    stocks = _cascade_stocks()
    state = State(n=0, stocks=stocks, rng_seed=0)
    env = SourceResolver()
    full = Registry([_fast_flow(), _slow_flow()], stocks)  # the whole model, as "fast"
    empty = Registry([], stocks)

    single = cls(full).step(state, env, 0.1)
    multi = multirate_step(cls(empty), cls(full), state, env, 0.1, 1).state

    assert multi.n == single.n == 1
    assert {s: v.amount for s, v in multi.stocks.items()} == {
        s: v.amount for s, v in single.stocks.items()
    }


def test_all_slow_strang_differs_from_single_rate() -> None:
    """The asymmetry (N4 note): all-*slow* Strang ≠ single-rate (two ``dt/2`` halves).

    With every flow in the *slow* set, a Strang master step is ``slow(dt/2)`` then
    ``slow(dt/2)`` — two half steps, which differ from one full ``dt`` step at
    O(``dt²``). So, unlike the all-fast case, this does **not** reproduce single-rate;
    both still conserve mass (the composite gate passed). Euler makes the gap obvious.
    """
    stocks = _cascade_stocks()
    state = State(n=0, stocks=stocks, rng_seed=0)
    env = SourceResolver()
    full = Registry([_fast_flow(), _slow_flow()], stocks)
    empty = Registry([], stocks)

    single = EulerIntegrator(full).step(state, env, 0.1)
    multi = multirate_step(
        EulerIntegrator(full), EulerIntegrator(empty), state, env, 0.1, 1
    ).state

    # The two-half-step composite is genuinely different (well above the float floor).
    assert abs(multi.stocks[Y].amount - single.stocks[Y].amount) > 1e-6


# --- conservation + the unbalanced-sub-delta tripwire (N4/N5) ---------------
def test_coupled_scenario_conserves_every_master_step() -> None:
    """The coupled cascade conserves total carbon at every master step.

    The composite conservation gate runs inside ``multirate_step`` (a violation would
    raise mid-run), so completing the run *is* the assertion; this pins the closure
    explicitly to the float floor over a multi-step run.
    """
    stocks = _cascade_stocks()
    state = State(n=0, stocks=stocks, rng_seed=0)
    env = SourceResolver()
    slow = Rk4Integrator(Registry([_slow_flow()], stocks))
    fast = Rk4Integrator(Registry([_fast_flow()], stocks))
    total0 = X0 + Y0 + Z0

    for _ in range(40):
        state = multirate_step(slow, fast, state, env, 0.1, 3).state
        total = sum(s.amount for s in state.stocks.values())
        assert abs(total - total0) < 1e-12


def test_unbalanced_sub_delta_trips_the_composite_gate() -> None:
    """A deliberately unbalanced sub-delta raises ``ConservationError`` — at the
    composite boundary (sub-steps skip the per-operation assert, so this gate is the
    one that catches it)."""
    stocks = _cascade_stocks()
    state = State(n=0, stocks=stocks, rng_seed=0)
    env = SourceResolver()
    # A broken flow in the fast set: it deposits carbon from nowhere.
    fast = EulerIntegrator(
        Registry(
            [_fast_flow(), _UnbalancedDeposit(FlowId("casc.bad"), 0, X, 5.0)], stocks
        )
    )
    slow = EulerIntegrator(Registry([_slow_flow()], stocks))

    with pytest.raises(ConservationError, match="conservation violated"):
        multirate_step(slow, fast, state, env, 0.1, 2)


# --- order of accuracy (N4) ------------------------------------------------
def _order_error(
    slow_cls: type, fast_cls: type, split: Split, n_sub: int, dt: float
) -> float:
    """Run the cascade to ``T_END`` under the split/scheme; return ``|y − y_exact|``.

    ``y`` carries the splitting error directly (it is the stock both operators touch);
    ``x`` would show only RK4's order (it evolves under the fast operator alone), so it
    is *not* the metric.
    """
    stocks = _cascade_stocks()
    state = State(n=0, stocks=stocks, rng_seed=0)
    slow = slow_cls(Registry([_slow_flow()], stocks))
    fast = fast_cls(Registry([_fast_flow()], stocks))
    env = SourceResolver()
    for _ in range(round(T_END / dt)):
        state = multirate_step(slow, fast, state, env, dt, n_sub, split=split).state
    return abs(state.stocks[Y].amount - _exact_y(T_END))


def test_strang_rk4_both_is_second_order() -> None:
    """Strang + RK4 on **both** operators → 2nd-order global (``min(2,4,4)=2``).

    Not 4th: the operators' non-commutativity is an O(``dt²``) splitting term no
    sub-integration removes (N4). Measured ≈ 2.00 across the ladder.
    """
    errors = [
        _order_error(Rk4Integrator, Rk4Integrator, Split.STRANG, 2, dt) for dt in DTS
    ]
    assert all(b < a for a, b in zip(errors[:-1], errors[1:], strict=True))
    p = fit_order(DTS, errors)
    assert 1.7 < p < 2.3


def test_strang_euler_collapses_to_first_order() -> None:
    """Strang + **Euler** on both → 1st-order (``min(2,1,1)=1``) — the silent collapse.

    A Euler operator caps the composite at order 1: Strang forfeits the 2nd order it
    was chosen for. This is the order-reduction the phase exists to catch. Measured ≈ 1.
    """
    errors = [
        _order_error(EulerIntegrator, EulerIntegrator, Split.STRANG, 2, dt)
        for dt in DTS
    ]
    p = fit_order(DTS, errors)
    assert 0.8 < p < 1.3


def test_lie_split_is_first_order_even_with_rk4() -> None:
    """Lie + RK4 on both → 1st-order: the *split* (order 1) caps it, not the scheme."""
    errors = [
        _order_error(Rk4Integrator, Rk4Integrator, Split.LIE, 2, dt) for dt in DTS
    ]
    p = fit_order(DTS, errors)
    assert 0.8 < p < 1.3


# --- n accounting (N2): sub-steps are internal, #14 preserved ---------------
@pytest.mark.parametrize("n_sub", [1, 2, 5])
def test_n_advances_once_per_master_step_regardless_of_nsub(n_sub: int) -> None:
    """After k master steps ``state.n == k`` for any ``n_sub`` (sub-steps internal)."""
    stocks = _cascade_stocks()
    state = State(n=0, stocks=stocks, rng_seed=0)
    env = SourceResolver()
    slow = Rk4Integrator(Registry([_slow_flow()], stocks))
    fast = Rk4Integrator(Registry([_fast_flow()], stocks))
    for k in range(1, 13):
        state = multirate_step(slow, fast, state, env, 0.1, n_sub).state
        assert state.n == k


def test_nsub_must_be_positive() -> None:
    """``n_sub < 1`` is a scenario bug (no fast sub-steps) → ``ValueError``."""
    stocks = _cascade_stocks()
    state = State(n=0, stocks=stocks, rng_seed=0)
    env = SourceResolver()
    slow = Rk4Integrator(Registry([_slow_flow()], stocks))
    fast = Rk4Integrator(Registry([_fast_flow()], stocks))
    with pytest.raises(ValueError, match="n_sub must be >= 1"):
        multirate_step(slow, fast, state, env, 0.1, 0)


# --- determinism + registration-order independence (#7/#15) -----------------
# A multi-flow scenario so a within-registry shuffle is non-trivial: the fast set has
# three flows (out of x and y), the slow set two (into z). All balanced, all dt-linear.
def _multi_stocks() -> dict[StockId, Stock]:
    return {X: _pool(X, X0), Y: _pool(Y, Y0), Z: _pool(Z, Z0), W: _pool(W, 0.4)}


def _fast_flows() -> list:
    return [
        _Cascade(FlowId("casc.xy"), 0, X, Y, 0.5),
        _Cascade(FlowId("casc.xw"), 0, X, W, 0.3),
        _Cascade(FlowId("casc.yw"), 0, Y, W, 0.2),
    ]


def _slow_flows() -> list:
    return [
        _Cascade(FlowId("casc.wz"), 0, W, Z, 0.1),
        _Cascade(FlowId("casc.yz"), 0, Y, Z, 0.15),
    ]


def _run_multi(fast_flows: list, n: int = 6) -> dict[StockId, float]:
    stocks = _multi_stocks()
    state = State(n=0, stocks=stocks, rng_seed=0)
    env = SourceResolver()
    slow = Rk4Integrator(Registry(_slow_flows(), stocks))
    fast = Rk4Integrator(Registry(fast_flows, stocks))
    for _ in range(n):
        state = multirate_step(slow, fast, state, env, 0.1, 3).state
    return {s: v.amount for s, v in state.stocks.items()}


def test_multirate_is_deterministic_across_runs() -> None:
    """Two identical runs produce bit-identical state."""
    assert _run_multi(_fast_flows()) == _run_multi(_fast_flows())


@given(perm=st.permutations(range(3)))
def test_multirate_is_registration_order_independent(perm: list[int]) -> None:
    """Shuffling the fast registry's flows yields identical state (canonical #15)."""
    base = _fast_flows()
    shuffled = [base[i] for i in perm]
    assert _run_multi(base) == _run_multi(shuffled)


# --- speedup sanity: fewer slow-flow evals than single-rate at the fast dt ---
def test_multirate_does_fewer_slow_evals_than_single_rate_at_fast_dt() -> None:
    """A count assertion (not wall-clock): with ``n_sub`` fast sub-steps, the slow flow
    is evaluated only in the two Strang halves (2 × 4 RK4 stages = 8), independent of
    ``n_sub``, whereas single-rate at the fast ``dt`` evaluates it every sub-step
    (``n_sub`` × 4). For ``n_sub = 4`` that is 8 vs 16 — the efficiency multi-rate
    exists for."""
    n_sub, dt = 4, 0.1

    # Multi-rate: slow and fast in disjoint registries.
    mr_calls = [0]
    stocks = _cascade_stocks()
    state = State(n=0, stocks=stocks, rng_seed=0)
    env = SourceResolver()
    slow = Rk4Integrator(
        Registry([_CountingCascade(FlowId("casc.yz"), 0, Y, Z, KS, mr_calls)], stocks)
    )
    fast = Rk4Integrator(Registry([_fast_flow()], stocks))
    multirate_step(slow, fast, state, env, dt, n_sub)

    # Single-rate over the combined registry, n_sub steps at the fast dt.
    sr_calls = [0]
    stocks2 = _cascade_stocks()
    state2 = State(n=0, stocks=stocks2, rng_seed=0)
    combined = Rk4Integrator(
        Registry(
            [_CountingCascade(FlowId("casc.yz"), 0, Y, Z, KS, sr_calls), _fast_flow()],
            stocks2,
        )
    )
    for _ in range(n_sub):
        state2 = combined.step(state2, env, dt / n_sub)

    assert mr_calls[0] == 8  # 2 Strang slow halves × 4 RK4 stages
    assert sr_calls[0] == n_sub * 4  # 16
    assert mr_calls[0] < sr_calls[0]


# --- extinction runs per sub-operation (N5 / the CLAUDE.md mass invariant) ---
def test_extinction_fires_inside_a_substep_aggregated_and_conserving() -> None:
    """A POPULATION stock driven below threshold inside a fast sub-step snaps to 0,
    its residual routes to the loss-sink, the event is aggregated into the master
    ``StepReport``, and the composite gate still conserves — extinction "runs per
    sub-operation as in single-rate" (Step-3 contract), through the split.

    Pins the multi-rate-specific extinction wiring that the POOL scenarios never reach:
    cross-sub-step event aggregation, the event ``n`` **re-stamp**, the
    once-per-master-step firing (the ``amount != 0`` guard must stop a re-fire in later
    sub-steps), and that snap-to-zero stays mass-conserving across the split. The
    initial state **must** carry the loss-sink (referential integrity, decision #6) —
    omitting it would raise ``KeyError`` in the extinction pass.
    """
    carbon = canonical_unit(Quantity.CARBON)
    pop = Stock(
        id=POP,
        domain=CASC,
        quantity=Quantity.CARBON,
        unit=carbon,
        amount=0.6,
        kind=StockKind.POPULATION,
        extinction_threshold=0.5,
    )
    stocks = {
        POP: pop,
        POP_SINK: boundary.sink(POP_SINK, Quantity.CARBON),
        boundary.loss_sink_id(Quantity.CARBON): boundary.loss_sink(Quantity.CARBON),
    }
    state = State(n=3, stocks=stocks, rng_seed=0)  # start at n=3 to pin the re-stamp
    env = SourceResolver()
    # Drain lives in the FAST set, so it (and its extinction) sub-steps; slow is empty.
    fast = EulerIntegrator(
        Registry([_PopDrain(FlowId("casc.drain"), 0, POP, POP_SINK, 1.0)], stocks)
    )
    slow = EulerIntegrator(Registry([], stocks))
    total0 = sum(s.amount for s in stocks.values())

    report = multirate_step(slow, fast, state, env, 0.5, 3)

    # Fires exactly once: after snapping to 0 the amount-guard blocks a re-fire in the
    # remaining sub-steps (single-rate fires extinction once per step; so does this).
    assert len(report.events) == 1
    event = report.events[0]
    assert event.stock == POP
    # Re-stamped to the produced n (before.n + 1 == 4), matching the single-rate
    # ExtinctionEvent.n convention — NOT the sub-step's observed n (which is 3).
    assert event.n == 4
    assert report.state.n == 4
    # Snapped to 0, the residual is in the loss-sink, and total carbon is conserved
    # across the whole master step (extinction is mass-conserving through the split).
    assert report.state.stocks[POP].amount == 0.0
    loss = report.state.stocks[boundary.loss_sink_id(Quantity.CARBON)].amount
    assert loss == pytest.approx(event.residual) and loss > 0.0
    total1 = sum(s.amount for s in report.state.stocks.values())
    assert abs(total1 - total0) < 1e-12


# --- protocol satisfaction --------------------------------------------------
def test_concrete_integrators_satisfy_substepper() -> None:
    """Both concrete strategies satisfy the ``Substepper`` protocol the driver targets
    (the additive ``substep`` capability, kept off the frozen ``Integrator`` surface).
    """
    reg = Registry([], _cascade_stocks())
    assert isinstance(EulerIntegrator(reg), Substepper)
    assert isinstance(Rk4Integrator(reg), Substepper)
