"""Step-7 tests: extinction-with-loss-sink (decision #6) + events.

A POPULATION stock below its ``extinction_threshold`` snaps to exactly 0 and its
residual is routed into the quantity's numerical-loss boundary sink so the ledger
still balances; an ``ExtinctionEvent`` records it. The absorbing property: an
already-extinct stock does not re-fire (no event-spam), and a sub-threshold inflow
("noise") is re-snapped rather than reviving the stock — while a supra-threshold
(scenario-scale) inflow survives. POOL stocks are never zeroed-with-loss.
"""

import dataclasses

import pytest

from simcore import boundary
from simcore.environment import Environment, SourceResolver
from simcore.events import ExtinctionEvent
from simcore.flow import FlowResult, Leg, assert_flow_balanced
from simcore.ids import DomainId, FlowId, StockId
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock

P = StockId("bio.pop")
SRC = StockId("boundary.src_c")


def _population(amount: float, threshold: float) -> Stock:
    return Stock(
        id=P,
        domain=DomainId("bio"),
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=amount,
        kind=StockKind.POPULATION,
        extinction_threshold=threshold,
    )


@dataclasses.dataclass(frozen=True)
class _DepositFlow:
    """Deposits a fixed ``amount`` per ``dt`` into ``dst`` from an unclamped source.

    The source is unclamped (decision #13), so it is never throttled and never
    triggers the RK4 over-draw guard — letting these tests isolate extinction.
    """

    id: FlowId
    priority: int
    src: StockId
    dst: StockId
    amount: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        x = self.amount * dt
        return FlowResult(legs=(Leg(self.src, -x), Leg(self.dst, x)))


# --- the basic snap (scheme-independent) -----------------------------------
@pytest.mark.parametrize("integrator_cls", [EulerIntegrator, Rk4Integrator])
def test_population_below_threshold_snaps_to_zero_and_routes_residual(
    integrator_cls: type,
) -> None:
    ls = boundary.loss_sink(Quantity.CARBON)
    stocks = {P: _population(0.3, threshold=0.5), ls.id: ls}
    state = State(n=0, stocks=stocks, rng_seed=0)

    report = integrator_cls(Registry([], stocks)).step_report(
        state, SourceResolver(), 1.0
    )

    assert report.state.stocks[P].amount == 0.0
    assert report.state.stocks[ls.id].amount == 0.3  # residual routed, conserved
    assert len(report.events) == 1
    ev = report.events[0]
    assert isinstance(ev, ExtinctionEvent)
    assert ev.stock == P
    assert ev.quantity is Quantity.CARBON
    assert ev.residual == 0.3
    assert ev.n == 1  # the post-apply step count
    # conservation: the population's loss exactly equals the loss-sink's gain.
    delta = FlowResult(
        legs=(
            Leg(P, report.state.stocks[P].amount - 0.3),
            Leg(ls.id, report.state.stocks[ls.id].amount - 0.0),
        )
    )
    assert_flow_balanced(delta, state.stocks)


def test_pool_below_threshold_is_never_zeroed() -> None:
    # A POOL carries an extinction_threshold field but is exempt: extinction is
    # POPULATION-only (decision #6). It must pass through untouched.
    pool = Stock(
        id=P,
        domain=DomainId("bio"),
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=0.3,
        kind=StockKind.POOL,
        extinction_threshold=0.5,
    )
    stocks = {P: pool}
    state = State(n=0, stocks=stocks, rng_seed=0)

    report = EulerIntegrator(Registry([], stocks)).step_report(
        state, SourceResolver(), 1.0
    )
    assert report.state.stocks[P].amount == 0.3
    assert report.events == ()


# --- absorbing: no event-spam, no revival from sub-threshold noise ----------
def test_already_extinct_stock_does_not_refire() -> None:
    # Sitting at exactly 0 below a positive threshold must NOT re-emit an event or
    # re-route every step (the `amount != 0` guard). No loss-sink is even needed.
    stocks = {P: _population(0.0, threshold=0.5)}
    state = State(n=0, stocks=stocks, rng_seed=0)

    report = EulerIntegrator(Registry([], stocks)).step_report(
        state, SourceResolver(), 1.0
    )
    assert report.state.stocks[P].amount == 0.0
    assert report.events == ()


def test_subthreshold_inflow_is_resnapped_not_revived() -> None:
    # An extinct stock receives a sub-threshold "noise" inflow of 0.1 (< 0.5). After
    # apply it is 0.1; the extinction pass re-snaps it to 0 and routes 0.1 — it does
    # NOT revive. Whole-step mass conserves (source -0.1 == loss-sink +0.1).
    ls = boundary.loss_sink(Quantity.CARBON)
    src = boundary.source(SRC, Quantity.CARBON, amount=1000.0)
    stocks = {P: _population(0.0, threshold=0.5), src.id: src, ls.id: ls}
    state = State(n=0, stocks=stocks, rng_seed=0)
    flow = _DepositFlow(FlowId("noise"), 0, SRC, P, 0.1)

    report = EulerIntegrator(Registry([flow], stocks)).step_report(
        state, SourceResolver(), 1.0
    )

    assert report.state.stocks[P].amount == 0.0  # re-snapped, not revived
    assert report.state.stocks[ls.id].amount == pytest.approx(0.1)
    assert len(report.events) == 1
    # source supplied 0.1; population net 0; loss-sink absorbed 0.1 → conserved.
    src_delta = report.state.stocks[src.id].amount - 1000.0
    assert src_delta == pytest.approx(-0.1)


def test_suprathreshold_inflow_survives() -> None:
    # A scenario-scale inflow of 1.0 (>= 0.5) revives the stock: it stays alive and
    # emits no extinction event.
    ls = boundary.loss_sink(Quantity.CARBON)
    src = boundary.source(SRC, Quantity.CARBON, amount=1000.0)
    stocks = {P: _population(0.0, threshold=0.5), src.id: src, ls.id: ls}
    state = State(n=0, stocks=stocks, rng_seed=0)
    flow = _DepositFlow(FlowId("reintroduce"), 0, SRC, P, 1.0)

    report = EulerIntegrator(Registry([flow], stocks)).step_report(
        state, SourceResolver(), 1.0
    )

    assert report.state.stocks[P].amount == pytest.approx(1.0)
    assert report.events == ()


# --- referential integrity: missing loss-sink -------------------------------
def test_extinction_without_loss_sink_raises() -> None:
    # The initial state must include the boundary loss-sinks; routing into a missing
    # one is a referential-integrity KeyError (the apply path's contract).
    stocks = {P: _population(0.3, threshold=0.5)}  # no loss-sink
    state = State(n=0, stocks=stocks, rng_seed=0)
    with pytest.raises(KeyError, match="loss-sink"):
        EulerIntegrator(Registry([], stocks)).step(state, SourceResolver(), 1.0)
