"""Step-7 tests: the arbitration backstop (min-scaling, Euler-only) + firing count.

Pure-function level (``min_scaling`` / ``check_no_overdraw``): proportional sag on
over-draw, conservation-safety (whole-flow scaling), the ``unclamped``-source skip,
and the Euler-scale / RK4-hard-error asymmetry. Integrator level: ``EulerIntegrator``
throttles and reports firings while keeping clamped stocks non-negative and mass
conserved; ``Rk4Integrator`` hard-errors instead; both are registration-order
independent.
"""

import dataclasses

import pytest
from hypothesis import given
from hypothesis import strategies as st

from simcore import boundary
from simcore.arbitration import ArbitrationError, check_no_overdraw, min_scaling
from simcore.environment import Environment, SourceResolver
from simcore.flow import Flow, FlowResult, Leg, assert_flow_balanced
from simcore.ids import DomainId, FlowId, StockId
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock

# --- fixtures --------------------------------------------------------------
S = StockId("bio.s")
K1 = StockId("boundary.k1")
K2 = StockId("boundary.k2")
K3 = StockId("boundary.k3")


def _pool(sid: StockId, amount: float, quantity: Quantity = Quantity.CARBON) -> Stock:
    return Stock(
        id=sid,
        domain=DomainId("bio"),
        quantity=quantity,
        unit=canonical_unit(quantity),
        amount=amount,
        kind=StockKind.POOL,
    )


@dataclasses.dataclass(frozen=True)
class _DrainFlow:
    """Withdraws a fixed ``amount`` per ``dt`` from ``src`` into boundary ``sink``.

    A *constant* withdrawal (independent of the stock level) — the synthetic
    over-draw the backstop exists to catch (real saturating kinetics would taper to
    0 as the stock empties; this does not).
    """

    id: FlowId
    priority: int
    src: StockId
    sink: StockId
    amount: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        x = self.amount * dt
        return FlowResult(legs=(Leg(self.src, -x), Leg(self.sink, x)))


# --- pure min-scaling: proportional sag ------------------------------------
def test_min_scaling_throttles_overdraw_proportionally() -> None:
    # Two flows each want 8 from a stock holding 10: demand 16 > 10, scale 10/16 =
    # 0.625 (binary-exact), so each realized draw is 8*0.625 = 5.0 and the stock
    # lands at exactly 0 — non-negative and fully conserved.
    stocks = {
        S: _pool(S, 10.0),
        K1: boundary.sink(K1, Quantity.CARBON),
        K2: boundary.sink(K2, Quantity.CARBON),
    }
    results = [
        FlowResult(legs=(Leg(S, -8.0), Leg(K1, 8.0))),
        FlowResult(legs=(Leg(S, -8.0), Leg(K2, 8.0))),
    ]
    scaled, fired = min_scaling(results, stocks)

    assert fired == 2
    assert scaled[0].legs == (Leg(S, -5.0), Leg(K1, 5.0))
    assert scaled[1].legs == (Leg(S, -5.0), Leg(K2, 5.0))
    # realized draw on S == available; the proof's equality case.
    assert 5.0 + 5.0 == 10.0


def test_min_scaling_is_noop_when_supply_is_ample() -> None:
    # demand 16 < 100: scale_f == 1 for both, so the originals are returned
    # unchanged (object identity) and nothing fires.
    stocks = {
        S: _pool(S, 100.0),
        K1: boundary.sink(K1, Quantity.CARBON),
        K2: boundary.sink(K2, Quantity.CARBON),
    }
    results = [
        FlowResult(legs=(Leg(S, -8.0), Leg(K1, 8.0))),
        FlowResult(legs=(Leg(S, -8.0), Leg(K2, 8.0))),
    ]
    scaled, fired = min_scaling(results, stocks)

    assert fired == 0
    assert scaled[0] is results[0]
    assert scaled[1] is results[1]


def test_min_scaling_skips_unclamped_sources() -> None:
    # A flow draws 100 from a source holding only 1 — but the source is unclamped
    # (e.g. solar), so min-scaling never throttles it (decision #13).
    src = boundary.source(StockId("boundary.solar"), Quantity.CARBON, amount=1.0)
    dst = _pool(StockId("bio.dst"), 0.0)
    stocks = {src.id: src, dst.id: dst}
    results = [FlowResult(legs=(Leg(src.id, -100.0), Leg(dst.id, 100.0)))]

    scaled, fired = min_scaling(results, stocks)

    assert fired == 0
    assert scaled[0] is results[0]


def test_whole_flow_scaling_preserves_multi_quantity_balance() -> None:
    # A 2-quantity stoichiometric flow (carbon + water) over-draws its scarce carbon
    # source; scaling the WHOLE flow by one factor keeps each quantity balanced.
    s_c = _pool(StockId("bio.s_c"), 1.0, Quantity.CARBON)  # scarce: demand 2 > 1
    p_c = _pool(StockId("bio.p_c"), 0.0, Quantity.CARBON)
    s_w = _pool(StockId("bio.s_w"), 10.0, Quantity.WATER)  # ample
    p_w = _pool(StockId("bio.p_w"), 0.0, Quantity.WATER)
    stocks = {s.id: s for s in (s_c, p_c, s_w, p_w)}
    flow = FlowResult(
        legs=(
            Leg(s_c.id, -2.0),
            Leg(p_c.id, 2.0),
            Leg(s_w.id, -1.0),
            Leg(p_w.id, 1.0),
        )
    )

    scaled, fired = min_scaling([flow], stocks)

    assert fired == 1
    # scale_f = min(carbon 1/2, water 1) = 0.5
    assert scaled[0].legs == (
        Leg(s_c.id, -1.0),
        Leg(p_c.id, 1.0),
        Leg(s_w.id, -0.5),
        Leg(p_w.id, 0.5),
    )
    assert_flow_balanced(scaled[0], stocks)  # still balanced per quantity


# --- RK4 hard-error guard --------------------------------------------------
def test_check_no_overdraw_raises_on_overdraw() -> None:
    stocks = {S: _pool(S, 10.0), K1: boundary.sink(K1, Quantity.CARBON)}
    results = [FlowResult(legs=(Leg(S, -16.0), Leg(K1, 16.0)))]  # 16 > 10
    with pytest.raises(ArbitrationError):
        check_no_overdraw(results, stocks)


def test_check_no_overdraw_passes_when_ample_and_skips_unclamped() -> None:
    src = boundary.source(StockId("boundary.solar"), Quantity.CARBON, amount=1.0)
    dst = _pool(StockId("bio.dst"), 0.0)
    stocks = {S: _pool(S, 100.0), src.id: src, dst.id: dst}
    results = [
        FlowResult(legs=(Leg(S, -8.0), Leg(K1, 8.0))),  # ample
        FlowResult(legs=(Leg(src.id, -100.0), Leg(dst.id, 100.0))),  # unclamped src
    ]
    stocks[K1] = boundary.sink(K1, Quantity.CARBON)
    check_no_overdraw(results, stocks)  # must not raise


# --- integrator level: Euler throttles + reports, RK4 hard-errors ----------
def _overdraw_scenario() -> tuple[dict[StockId, Stock], list[Flow], State]:
    stocks = {
        S: _pool(S, 10.0),
        K1: boundary.sink(K1, Quantity.CARBON),
        K2: boundary.sink(K2, Quantity.CARBON),
    }
    flows: list[Flow] = [
        _DrainFlow(FlowId("drain_1"), 0, S, K1, 8.0),
        _DrainFlow(FlowId("drain_2"), 0, S, K2, 8.0),
    ]
    return stocks, flows, State(n=0, stocks=stocks, rng_seed=0)


def test_euler_step_throttles_overdraw_keeps_nonnegative_and_reports() -> None:
    stocks, flows, state = _overdraw_scenario()
    report = EulerIntegrator(Registry(flows, stocks)).step_report(
        state, SourceResolver(), 1.0
    )

    assert report.rationed == 2
    assert report.state.stocks[S].amount >= 0.0  # the non-negativity invariant
    assert report.state.stocks[S].amount == 0.0  # exact landing for these magnitudes
    assert report.state.stocks[K1].amount == 5.0
    assert report.state.stocks[K2].amount == 5.0
    assert report.events == ()
    # the realized whole-step delta conserves carbon (boundary sinks counted).
    delta = FlowResult(
        legs=tuple(
            Leg(sid, report.state.stocks[sid].amount - state.stocks[sid].amount)
            for sid in sorted(state.stocks)
        )
    )
    assert_flow_balanced(delta, state.stocks)


def test_euler_step_reports_zero_firings_when_well_fed() -> None:
    stocks = {
        S: _pool(S, 1000.0),
        K1: boundary.sink(K1, Quantity.CARBON),
        K2: boundary.sink(K2, Quantity.CARBON),
    }
    flows: list[Flow] = [
        _DrainFlow(FlowId("drain_1"), 0, S, K1, 8.0),
        _DrainFlow(FlowId("drain_2"), 0, S, K2, 8.0),
    ]
    state = State(n=0, stocks=stocks, rng_seed=0)
    report = EulerIntegrator(Registry(flows, stocks)).step_report(
        state, SourceResolver(), 1.0
    )
    assert report.rationed == 0


def test_rk4_step_hard_errors_on_overdraw() -> None:
    stocks, flows, state = _overdraw_scenario()
    with pytest.raises(ArbitrationError):
        Rk4Integrator(Registry(flows, stocks)).step(state, SourceResolver(), 1.0)


def test_rk4_step_runs_when_well_fed() -> None:
    stocks = {S: _pool(S, 1000.0), K1: boundary.sink(K1, Quantity.CARBON)}
    flows: list[Flow] = [_DrainFlow(FlowId("drain"), 0, S, K1, 8.0)]
    state = State(n=0, stocks=stocks, rng_seed=0)
    report = Rk4Integrator(Registry(flows, stocks)).step_report(
        state, SourceResolver(), 1.0
    )
    assert report.rationed == 0
    assert report.state.stocks[S].amount == pytest.approx(1000.0 - 8.0)


# --- registration-order independence under arbitration ---------------------
@given(perm=st.permutations(range(3)))
def test_euler_overdraw_is_registration_order_independent(perm: list[int]) -> None:
    # Three flows competing for one scarce stock: the per-stock demand sum and the
    # scaled per-stock reduction must be bit-identical under registration shuffle
    # (canonical-order reductions, #15) — even though the scale (10/24) is inexact.
    stocks = {
        S: _pool(S, 10.0),
        K1: boundary.sink(K1, Quantity.CARBON),
        K2: boundary.sink(K2, Quantity.CARBON),
        K3: boundary.sink(K3, Quantity.CARBON),
    }
    flows: list[Flow] = [
        _DrainFlow(FlowId("drain_1"), 0, S, K1, 8.0),
        _DrainFlow(FlowId("drain_2"), 0, S, K2, 8.0),
        _DrainFlow(FlowId("drain_3"), 0, S, K3, 8.0),
    ]
    state = State(n=0, stocks=stocks, rng_seed=0)
    shuffled = [flows[i] for i in perm]

    base = EulerIntegrator(Registry(flows, stocks)).step(state, SourceResolver(), 1.0)
    other = EulerIntegrator(Registry(shuffled, stocks)).step(
        state, SourceResolver(), 1.0
    )

    assert {s: st_.amount for s, st_ in base.stocks.items()} == {
        s: st_.amount for s, st_ in other.stocks.items()
    }
