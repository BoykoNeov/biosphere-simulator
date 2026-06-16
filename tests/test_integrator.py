"""Step-6 tests: Euler / RK4 integrator strategies.

Step 6 builds the stepping spine — evaluate → reduce (canonical order) → combine
→ apply once (``n -> n+1``) — for explicit Euler and classical RK4. The
arbitration backstop and extinction/conservation gates are steps 7–8, so these
tests exercise *non-arbitrating, finite-supply* scenarios only (no over-draw, no
extinction).

The headline correctness properties: RK4 is demonstrably 4th-order (vs Euler's
1st) on analytic exponential decay; the new combine/apply arithmetic conserves
mass (validated with the step-3 balance helper); the RK4 combine folds over the
*union* of stage keys; and a step is bit-identical under registration shuffle.
"""

import dataclasses
import math

import pytest
from hypothesis import given
from hypothesis import strategies as st

from simcore import boundary
from simcore.environment import Environment, SourceResolver
from simcore.flow import Flow, FlowResult, Leg, assert_flow_balanced
from simcore.ids import DomainId, FlowId, StockId
from simcore.integrator import (
    EulerIntegrator,
    Integrator,
    Rk4Integrator,
)
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock

# --- fixtures --------------------------------------------------------------
A = StockId("bio.a")
B = StockId("bio.b")
DST = StockId("bio.dst")
SINK_A = StockId("boundary.sink_a")
SINK_B = StockId("boundary.sink_b")
SRC = StockId("boundary.src")


def _pool(sid: StockId, amount: float, quantity: Quantity = Quantity.CARBON) -> Stock:
    return Stock(
        id=sid,
        domain=DomainId("bio"),
        quantity=quantity,
        unit=canonical_unit(quantity),
        amount=amount,
        kind=StockKind.POOL,
    )


class _NullEnv:
    """An Environment that resolves nothing (for flows that ignore env)."""

    def get(self, var: str) -> float:
        raise KeyError(var)


@dataclasses.dataclass(frozen=True)
class _DecayFlow:
    """``src -> sink`` (boundary) at first-order rate ``rate`` — dt-linear.

    legs: ``(src, -rate·src·dt)``, ``(sink, +rate·src·dt)``. Balanced, and clear of
    extinction (POOL src) and over-draw for ``rate·dt < 1``.
    """

    id: FlowId
    priority: int
    src: StockId
    sink: StockId
    rate: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        amount = self.rate * snapshot.stocks[self.src].amount * dt
        return FlowResult(legs=(Leg(self.src, -amount), Leg(self.sink, amount)))


@dataclasses.dataclass(frozen=True)
class _TransferFlow:
    """``src -> dst`` moving a fixed fraction of ``src`` per step (dt-linear)."""

    id: FlowId
    priority: int
    src: StockId
    dst: StockId
    frac: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        amount = self.frac * snapshot.stocks[self.src].amount * dt
        return FlowResult(legs=(Leg(self.src, -amount), Leg(self.dst, amount)))


@dataclasses.dataclass(frozen=True)
class _GatedFlow:
    """Active only when ``gate``'s amount < ``threshold``: moves ``rate·dt``.

    When inactive it returns **empty** legs — so the touched stocks are *absent*
    from that stage's ``k``. Used to exercise the union-of-keys combine.
    """

    id: FlowId
    priority: int
    gate: StockId
    threshold: float
    src: StockId
    sink: StockId
    rate: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        if snapshot.stocks[self.gate].amount < self.threshold:
            x = self.rate * dt
            return FlowResult(legs=(Leg(self.src, -x), Leg(self.sink, x)))
        return FlowResult(legs=())


@dataclasses.dataclass(frozen=True)
class _ForcingDepositFlow:
    """Deposits ``env.get(var)·dt`` into ``dst`` from a boundary ``src`` (balanced)."""

    id: FlowId
    priority: int
    var: str
    src: StockId
    dst: StockId

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        v = env.get(self.var) * dt
        return FlowResult(legs=(Leg(self.src, -v), Leg(self.dst, v)))


# --- Euler correctness -----------------------------------------------------
def test_euler_one_step_is_y_plus_dt_rate() -> None:
    a0, rate, dt = 10.0, 0.3, 0.5
    stocks = {A: _pool(A, a0), SINK_A: boundary.sink(SINK_A, Quantity.CARBON)}
    state = State(n=0, stocks=stocks, rng_seed=0)
    reg = Registry([_DecayFlow(FlowId("decay"), 0, A, SINK_A, rate)], stocks)

    nxt = EulerIntegrator(reg).step(state, SourceResolver(), dt)

    moved = rate * a0 * dt
    assert nxt.n == 1
    assert nxt.stocks[A].amount == a0 - moved
    assert nxt.stocks[SINK_A].amount == moved


# --- RK4 correctness + order of accuracy -----------------------------------
def _decay_error(integrator_cls: type, dt: float, lam: float, t_end: float) -> float:
    a0 = 1.0
    steps = int(round(t_end / dt))
    stocks = {A: _pool(A, a0), SINK_A: boundary.sink(SINK_A, Quantity.CARBON)}
    state = State(n=0, stocks=stocks, rng_seed=0)
    reg = Registry([_DecayFlow(FlowId("decay"), 0, A, SINK_A, lam)], stocks)
    integrator = integrator_cls(reg)
    env = SourceResolver()
    for _ in range(steps):
        state = integrator.step(state, env, dt)
    exact = a0 * math.exp(-lam * t_end)
    return abs(state.stocks[A].amount - exact)


def test_rk4_far_more_accurate_than_euler_at_same_dt() -> None:
    lam, t_end, dt = 1.0, 1.0, 0.1
    rk4 = _decay_error(Rk4Integrator, dt, lam, t_end)
    euler = _decay_error(EulerIntegrator, dt, lam, t_end)
    assert rk4 < euler / 1000.0


def test_rk4_is_fourth_order_euler_is_first_order() -> None:
    lam, t_end = 1.0, 1.0
    # Halving dt: a 1st-order method's error ~halves; a 4th-order method's drops ~16x.
    euler_ratio = _decay_error(EulerIntegrator, 0.02, lam, t_end) / _decay_error(
        EulerIntegrator, 0.01, lam, t_end
    )
    rk4_ratio = _decay_error(Rk4Integrator, 0.02, lam, t_end) / _decay_error(
        Rk4Integrator, 0.01, lam, t_end
    )
    assert 1.8 < euler_ratio < 2.2
    assert 10.0 < rk4_ratio < 20.0


# --- dt-linearity contract (guards the increment-form RK4 identity) --------
def test_flow_legs_scale_linearly_with_dt() -> None:
    # The increment-form RK4 derivation assumes evaluate(y, dt) = dt·rate(y) with
    # rate dt-independent. A demo flow must therefore scale its legs ×2 from dt→2dt.
    stocks = {A: _pool(A, 10.0), SINK_A: boundary.sink(SINK_A, Quantity.CARBON)}
    state = State(n=0, stocks=stocks, rng_seed=0)
    flow = _DecayFlow(FlowId("decay"), 0, A, SINK_A, 0.3)
    env = _NullEnv()

    legs1 = {leg.stock: leg.amount for leg in flow.evaluate(state, env, 0.1).legs}
    legs2 = {leg.stock: leg.amount for leg in flow.evaluate(state, env, 0.2).legs}

    assert legs1.keys() == legs2.keys()
    for sid, amount in legs1.items():
        assert legs2[sid] == pytest.approx(2.0 * amount)


# --- conservation of the new combine/apply arithmetic (test-level) ---------
@pytest.mark.parametrize("integrator_cls", [EulerIntegrator, Rk4Integrator])
def test_step_applied_delta_conserves_mass(integrator_cls: type) -> None:
    # Each k_i is a sum of balanced flow legs; the linear ⅙-combine preserves that.
    # So the realized per-step delta, wrapped as a FlowResult, must balance under
    # the step-3 helper — several steps before the step-8 runtime gate exists.
    stocks = {
        A: _pool(A, 100.0),
        B: _pool(B, 10.0),
        SINK_A: boundary.sink(SINK_A, Quantity.CARBON),
    }
    state = State(n=0, stocks=stocks, rng_seed=0)
    reg = Registry(
        [
            _TransferFlow(FlowId("transfer"), 0, A, B, 0.1),
            _DecayFlow(FlowId("harvest"), 0, B, SINK_A, 0.2),
        ],
        stocks,
    )

    nxt = integrator_cls(reg).step(state, SourceResolver(), 0.5)

    delta = FlowResult(
        legs=tuple(
            Leg(sid, nxt.stocks[sid].amount - state.stocks[sid].amount)
            for sid in sorted(state.stocks)
        )
    )
    assert_flow_balanced(delta, state.stocks)


# --- RK4 combine folds over the UNION of stage keys ------------------------
def test_rk4_combine_includes_stocks_touched_only_at_a_perturbed_stage() -> None:
    # A decays fast enough that it crosses the gate threshold *within* the half-step
    # RK4 perturbations: the gated flow is inactive at y_n (so B is absent from k1)
    # but active at stages 2–4. A combine that iterated only k1's keys would drop B.
    a0 = 10.0
    stocks = {
        A: _pool(A, a0),
        B: _pool(B, 10.0),
        SINK_A: boundary.sink(SINK_A, Quantity.CARBON),
        SINK_B: boundary.sink(SINK_B, Quantity.CARBON),
    }
    state = State(n=0, stocks=stocks, rng_seed=0)
    reg = Registry(
        [
            _DecayFlow(FlowId("decay_a"), 0, A, SINK_A, 0.2),  # k1 for A = -2.0
            _GatedFlow(FlowId("gated"), 0, A, 9.5, B, SINK_B, 1.0),
        ],
        stocks,
    )

    nxt = Rk4Integrator(reg).step(state, SourceResolver(), 1.0)

    # B was touched only at perturbed stages — it must still have changed.
    assert nxt.stocks[B].amount < 10.0 - 1e-9
    # And that change conserves carbon against its boundary sink.
    assert nxt.stocks[SINK_B].amount == pytest.approx(10.0 - nxt.stocks[B].amount)


# --- RK4 reads forcing at the step's n for all four stages ------------------
def test_rk4_forcing_is_piecewise_constant_within_a_step() -> None:
    # schedule returns float(n); RK4 stage states keep the step's n, so all four
    # stages read the same value. A buggy impl that advanced n per stage (or read
    # n+1) would give a different deposit.
    dt, start_n = 0.5, 5
    stocks = {
        SRC: boundary.source(SRC, Quantity.ENERGY, 1000.0),
        DST: _pool(DST, 0.0, quantity=Quantity.ENERGY),
    }
    state = State(n=start_n, stocks=stocks, rng_seed=0)
    resolver = SourceResolver(forcings={"f": lambda n, _dt: float(n)})
    reg = Registry([_ForcingDepositFlow(FlowId("deposit"), 0, "f", SRC, DST)], stocks)

    nxt = Rk4Integrator(reg).step(state, resolver, dt)

    assert nxt.n == start_n + 1
    assert nxt.stocks[DST].amount == pytest.approx(start_n * dt)


# --- referential integrity (apply path) ------------------------------------
def test_step_raises_on_leg_to_unknown_stock() -> None:
    @dataclasses.dataclass(frozen=True)
    class _GhostFlow:
        id: FlowId
        priority: int

        def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
            return FlowResult(legs=(Leg(StockId("bio.ghost"), 1.0),))

    stocks = {A: _pool(A, 1.0)}
    state = State(n=0, stocks=stocks, rng_seed=0)
    reg = Registry([_GhostFlow(FlowId("ghost"), 0)], stocks)

    with pytest.raises(KeyError, match="unknown stock"):
        EulerIntegrator(reg).step(state, SourceResolver(), 1.0)


# --- determinism + registration-order independence -------------------------
def _shuffle_scenario() -> tuple[list[Flow], dict[StockId, Stock], State]:
    stocks = {
        A: _pool(A, 100.0),
        B: _pool(B, 10.0),
        DST: _pool(DST, 1.0),
        SINK_A: boundary.sink(SINK_A, Quantity.CARBON),
    }
    flows: list[Flow] = [
        _TransferFlow(FlowId("f_transfer"), 0, A, B, 0.1),
        _DecayFlow(FlowId("a_decay"), 0, B, SINK_A, 0.2),
        _TransferFlow(FlowId("m_transfer"), 0, A, DST, 0.05),
    ]
    return flows, stocks, State(n=0, stocks=stocks, rng_seed=0)


@pytest.mark.parametrize("integrator_cls", [EulerIntegrator, Rk4Integrator])
@given(perm=st.permutations(range(3)))
def test_step_is_registration_order_independent(
    integrator_cls: type, perm: list[int]
) -> None:
    flows, stocks, state = _shuffle_scenario()
    shuffled = [flows[i] for i in perm]

    base = integrator_cls(Registry(flows, stocks)).step(state, SourceResolver(), 0.5)
    other = integrator_cls(Registry(shuffled, stocks)).step(
        state, SourceResolver(), 0.5
    )

    # Bit-identical amounts (exact ==), validating canonical reduction.
    assert {s: st_.amount for s, st_ in base.stocks.items()} == {
        s: st_.amount for s, st_ in other.stocks.items()
    }


def test_step_is_deterministic_across_runs() -> None:
    flows, stocks, state = _shuffle_scenario()
    reg = Registry(flows, stocks)
    first = Rk4Integrator(reg).step(state, SourceResolver(), 0.5)
    second = Rk4Integrator(reg).step(state, SourceResolver(), 0.5)
    assert {s: st_.amount for s, st_ in first.stocks.items()} == {
        s: st_.amount for s, st_ in second.stocks.items()
    }


# --- Protocol satisfaction -------------------------------------------------
def test_integrators_satisfy_the_protocol() -> None:
    reg = Registry([], {A: _pool(A, 1.0)})
    assert isinstance(EulerIntegrator(reg), Integrator)
    assert isinstance(Rk4Integrator(reg), Integrator)
