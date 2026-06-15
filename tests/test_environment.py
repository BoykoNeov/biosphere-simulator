"""Environment tests: the Protocol (step 3) and the source-resolver backends (step 5).

Step 5 builds the concrete backends behind the frozen ``get(var) -> float``
interface: a forcing branch (schedule at ``t = n*dt``, #14) and a shared-stock
branch (reads the bound immutable snapshot, #16). The headline property is
*indistinguishability* — a flow cannot tell which branch answered.
"""

import dataclasses

import pytest
from hypothesis import given
from hypothesis import strategies as st

from simcore.environment import (
    BoundEnvironment,
    Environment,
    SourceResolver,
    constant,
)
from simcore.flow import FlowResult, Leg
from simcore.ids import DomainId, FlowId, StockId
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import State, Stock

# --- fixtures --------------------------------------------------------------
LIGHT_STOCK = StockId("boundary.light")


def _light_stock(amount: float) -> Stock:
    """A boundary reservoir a shared env var can resolve to."""
    return Stock(
        id=LIGHT_STOCK,
        domain=DomainId("boundary"),
        quantity=Quantity.ENERGY,
        unit=canonical_unit(Quantity.ENERGY),
        amount=amount,
        kind=StockKind.BOUNDARY,
        unclamped=True,
    )


def _state(amount: float, *, n: int = 0) -> State:
    return State(n=n, stocks={LIGHT_STOCK: _light_stock(amount)}, rng_seed=0)


@dataclasses.dataclass(frozen=True)
class _ReadsEnvFlow:
    """A pure flow whose legs depend only on ``env.get(var)`` — so its result is a
    direct witness to what the resolver returned (used for the indistinguishability
    gate)."""

    id: FlowId
    priority: int
    var: str
    sink: StockId

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        value = env.get(self.var)
        return FlowResult(legs=(Leg(self.sink, value),))


# --- existing Protocol tests (step 3) --------------------------------------
def test_environment_protocol_accepts_a_get_implementation() -> None:
    class Forcing:
        def get(self, var: str) -> float:
            return 1.5

    assert isinstance(Forcing(), Environment)


def test_environment_protocol_rejects_missing_get() -> None:
    class NotAnEnv:
        def lookup(self, var: str) -> float:
            return 0.0

    assert not isinstance(NotAnEnv(), Environment)


def test_environment_get_returns_the_value() -> None:
    class Forcing:
        def get(self, var: str) -> float:
            return 2.0

    env: Environment = Forcing()
    assert env.get("light") == 2.0


# --- constant schedule -----------------------------------------------------
def test_constant_returns_value_for_any_n_and_dt() -> None:
    schedule = constant(7.5)
    assert schedule(0, 1.0) == 7.5
    assert schedule(123, 0.25) == 7.5


@pytest.mark.parametrize("bad", [float("nan"), float("inf"), float("-inf")])
def test_constant_rejects_non_finite(bad: float) -> None:
    with pytest.raises(ValueError, match="not finite"):
        constant(bad)


# --- forcing branch (evaluated at t = n*dt, integer n; #14) ----------------
def test_forcing_branch_receives_integer_n_and_dt() -> None:
    # A schedule that returns t = n*dt witnesses both that it sees the integer n
    # and that the value is evaluated at n*dt, never accumulated.
    resolver = SourceResolver(forcings={"t": lambda n, dt: n * dt})
    assert resolver.bind(_state(0.0, n=0), dt=0.5).get("t") == 0.0
    assert resolver.bind(_state(0.0, n=3), dt=0.5).get("t") == 1.5
    assert resolver.bind(_state(0.0, n=10), dt=0.1).get("t") == pytest.approx(1.0)


def test_forcing_branch_constant_is_fixed_across_n() -> None:
    resolver = SourceResolver(forcings={"solar": constant(42.0)})
    assert resolver.bind(_state(0.0, n=0), dt=1.0).get("solar") == 42.0
    assert resolver.bind(_state(0.0, n=999), dt=1.0).get("solar") == 42.0


def test_forcing_branch_rejects_non_finite_result() -> None:
    # Stock amounts are guaranteed finite; only a forcing schedule can leak NaN/Inf.
    resolver = SourceResolver(forcings={"bad": lambda n, dt: float("nan")})
    with pytest.raises(ValueError, match="non-finite"):
        resolver.bind(_state(0.0), dt=1.0).get("bad")


# --- shared-stock branch (reads the bound immutable snapshot; #16) ---------
def test_shared_branch_reads_bound_snapshot_amount() -> None:
    resolver = SourceResolver(shared={"light": LIGHT_STOCK})
    assert resolver.bind(_state(5.0), dt=1.0).get("light") == 5.0


def test_shared_branch_rebinding_reflects_new_snapshot() -> None:
    # The crux of #16: the bound view reads the *current* snapshot, never a cached
    # value — rebinding to a snapshot with a different amount changes the result.
    resolver = SourceResolver(shared={"light": LIGHT_STOCK})
    assert resolver.bind(_state(5.0), dt=1.0).get("light") == 5.0
    assert resolver.bind(_state(8.0), dt=1.0).get("light") == 8.0


def test_shared_branch_missing_stock_raises_keyerror() -> None:
    # Referential integrity is resolve-time by design (consistent with flow legs).
    resolver = SourceResolver(shared={"ghost": StockId("boundary.does_not_exist")})
    with pytest.raises(KeyError):
        resolver.bind(_state(1.0), dt=1.0).get("ghost")


# --- indistinguishability (the headline: reader can't tell which branch) ---
def test_forcing_and_shared_are_indistinguishable_across_n() -> None:
    # A flow reading env.get("x") must produce an *equal* FlowResult whether "x"
    # is a constant forcing V or a stock holding V — across several n, so the test
    # also pins that constant-forcing and static-stock stay equivalent as steps
    # advance (a one-binding comparison could pass even if forcing ignored n).
    value = 3.25
    sink = StockId("boundary.sink")
    flow = _ReadsEnvFlow(id=FlowId("reads_x"), priority=0, var="x", sink=sink)

    forcing_env_resolver = SourceResolver(forcings={"x": constant(value)})
    shared_env_resolver = SourceResolver(shared={"x": LIGHT_STOCK})

    for n in (0, 1, 7, 1000):
        forced = flow.evaluate(
            _state(0.0, n=n), forcing_env_resolver.bind(_state(0.0, n=n), dt=1.0), 1.0
        )
        snap = _state(value, n=n)
        coupled = flow.evaluate(snap, shared_env_resolver.bind(snap, dt=1.0), 1.0)
        assert forced == coupled


# --- mixed wiring: one resolver dispatches both branches -------------------
def test_mixed_resolver_dispatches_forcing_and_shared_from_one_bind() -> None:
    # The step-10 demo shape: a single resolver wiring some vars as forcing and
    # others as a shared Boundary stock, dispatched correctly from one bind.
    resolver = SourceResolver(
        forcings={"solar": constant(42.0)},
        shared={"light": LIGHT_STOCK},
    )
    bound = resolver.bind(_state(5.0, n=7), dt=1.0)
    assert bound.get("solar") == 42.0
    assert bound.get("light") == 5.0


# --- branch isolation (Hypothesis): forcing ignores stock contents ---------
@given(
    n=st.integers(min_value=0, max_value=10**6),
    dt=st.floats(min_value=1e-6, max_value=10.0, allow_nan=False, allow_infinity=False),
    stock_amount=st.floats(allow_nan=False, allow_infinity=False, width=32),
    forcing_value=st.floats(allow_nan=False, allow_infinity=False, width=32),
)
def test_forcing_value_depends_only_on_n_and_dt(
    n: int, dt: float, stock_amount: float, forcing_value: float
) -> None:
    # A forcing var's value must depend only on (n, dt) — never on the snapshot's
    # stock contents — so the two branches cannot bleed into each other.
    resolver = SourceResolver(forcings={"solar": constant(forcing_value)})
    assert resolver.bind(_state(stock_amount, n=n), dt=dt).get("solar") == forcing_value


# --- construction & wiring -------------------------------------------------
def test_resolver_rejects_var_wired_as_both() -> None:
    with pytest.raises(ValueError, match="both forcing and shared"):
        SourceResolver(forcings={"x": constant(1.0)}, shared={"x": LIGHT_STOCK})


def test_empty_resolver_is_valid_but_resolves_nothing() -> None:
    resolver = SourceResolver()
    with pytest.raises(KeyError, match="unknown env var"):
        resolver.bind(_state(0.0), dt=1.0).get("anything")


def test_unknown_var_raises_keyerror() -> None:
    resolver = SourceResolver(forcings={"solar": constant(1.0)})
    with pytest.raises(KeyError, match="unknown env var"):
        resolver.bind(_state(0.0), dt=1.0).get("missing")


# --- backend satisfies the interface ---------------------------------------
def test_bound_environment_satisfies_the_protocol() -> None:
    bound = SourceResolver().bind(_state(0.0), dt=1.0)
    assert isinstance(bound, BoundEnvironment)
    assert isinstance(bound, Environment)
