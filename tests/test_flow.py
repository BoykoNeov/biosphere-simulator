"""Step-3 tests: Leg / FlowResult shapes, balance helpers, purity, domains_touched.

Balance and referential integrity are *evaluation-time* (decisions #1/#2), so all
the helpers operate over an evaluated ``FlowResult`` plus the stock table that
resolves each leg's quantity/domain.
"""

import dataclasses

import pytest

from simcore.environment import Environment
from simcore.flow import (
    ConservationError,
    Flow,
    FlowResult,
    Leg,
    assert_flow_balanced,
    domains_touched,
    per_quantity_residual,
)
from simcore.ids import DomainId, FlowId, StockId
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import State, Stock

# --- fixtures --------------------------------------------------------------
# A small two-domain stock table: biosphere carbon/energy + a boundary reservoir.
ATM_C = StockId("bio.atm_c")
PLANT_C = StockId("bio.plant_c")
ENERGY = StockId("bio.energy")
OUTSIDE_C = StockId("boundary.outside_c")


def _stock(
    sid: StockId,
    *,
    domain: str = "bio",
    quantity: Quantity = Quantity.CARBON,
    amount: float = 1.0,
    kind: StockKind = StockKind.POOL,
) -> Stock:
    return Stock(
        id=sid,
        domain=DomainId(domain),
        quantity=quantity,
        unit=canonical_unit(quantity),
        amount=amount,
        kind=kind,
    )


def _stocks() -> dict[StockId, Stock]:
    members = [
        _stock(ATM_C, quantity=Quantity.CARBON, amount=100.0),
        _stock(PLANT_C, quantity=Quantity.CARBON, amount=10.0),
        _stock(ENERGY, quantity=Quantity.ENERGY, amount=0.0),
        _stock(
            OUTSIDE_C,
            domain="boundary",
            quantity=Quantity.CARBON,
            kind=StockKind.BOUNDARY,
        ),
    ]
    return {s.id: s for s in members}


class _NullEnv:
    """Minimal Environment that resolves nothing (step-5 backends not needed here)."""

    def get(self, var: str) -> float:
        raise KeyError(var)


@dataclasses.dataclass(frozen=True)
class _ScaledTransferFlow:
    """A pure flow: withdraw a fraction of ``src`` and deposit it into ``dst``."""

    id: FlowId
    priority: int
    src: StockId
    dst: StockId
    frac: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        amount = snapshot.stocks[self.src].amount * self.frac * dt
        return FlowResult(legs=(Leg(self.src, -amount), Leg(self.dst, amount)))


# --- Leg -------------------------------------------------------------------
def test_leg_is_frozen() -> None:
    leg = Leg(ATM_C, 1.0)
    with pytest.raises(dataclasses.FrozenInstanceError):
        leg.amount = 2.0  # type: ignore[misc]


@pytest.mark.parametrize("bad", [float("nan"), float("inf"), float("-inf")])
def test_leg_rejects_non_finite_amount(bad: float) -> None:
    with pytest.raises(ValueError, match="not finite"):
        Leg(ATM_C, bad)


# --- FlowResult ------------------------------------------------------------
def test_flowresult_is_frozen() -> None:
    result = FlowResult(legs=())
    with pytest.raises(dataclasses.FrozenInstanceError):
        result.legs = ()  # type: ignore[misc]


def test_flowresult_empty_legs_is_a_valid_no_op() -> None:
    assert FlowResult(legs=()).legs == ()


def test_flowresult_rejects_duplicate_stock_leg() -> None:
    with pytest.raises(ValueError, match="duplicate"):
        FlowResult(legs=(Leg(ATM_C, 1.0), Leg(ATM_C, -1.0)))


def test_flowresult_coerces_legs_to_tuple() -> None:
    # Passing a list must still yield a hashable/comparable tuple-backed result.
    result = FlowResult(legs=[Leg(ATM_C, 1.0)])  # type: ignore[arg-type]
    assert isinstance(result.legs, tuple)
    assert result == FlowResult(legs=(Leg(ATM_C, 1.0),))


# --- per_quantity_residual -------------------------------------------------
def test_per_quantity_residual_reports_energy_diagnostic() -> None:
    stocks = _stocks()
    # Carbon balances; energy is a lone +3 (unclosed) — reported, not asserted.
    result = FlowResult(legs=(Leg(ATM_C, -5.0), Leg(PLANT_C, 5.0), Leg(ENERGY, 3.0)))
    residual = per_quantity_residual(result, stocks)
    assert residual[Quantity.CARBON] == pytest.approx(0.0)
    assert residual[Quantity.ENERGY] == pytest.approx(3.0)
    # Only quantities actually touched are keyed (absent quantity ⇒ trivially 0).
    assert Quantity.WATER not in residual


def test_per_quantity_residual_unknown_stock_raises_keyerror() -> None:
    # Referential integrity is the apply path's job (step 5/6), not this helper's.
    with pytest.raises(KeyError):
        per_quantity_residual(FlowResult(legs=(Leg(StockId("ghost"), 1.0),)), _stocks())


# --- assert_flow_balanced --------------------------------------------------
def test_assert_flow_balanced_passes_a_balanced_flow() -> None:
    result = FlowResult(legs=(Leg(ATM_C, -5.0), Leg(PLANT_C, 5.0)))
    assert_flow_balanced(result, _stocks())  # no raise


def test_assert_flow_balanced_rejects_carbon_imbalance() -> None:
    result = FlowResult(legs=(Leg(ATM_C, -5.0), Leg(PLANT_C, 4.0)))
    with pytest.raises(ConservationError, match="CARBON"):
        assert_flow_balanced(result, _stocks())


def test_assert_flow_balanced_tolerates_energy_imbalance() -> None:
    # ENERGY is exempt (decision #8: energy closure is Phase 5/6). Carbon balanced.
    result = FlowResult(legs=(Leg(ATM_C, -5.0), Leg(PLANT_C, 5.0), Leg(ENERGY, 3.0)))
    assert_flow_balanced(result, _stocks())  # no raise despite the energy residual


def test_assert_flow_balanced_applies_absolute_tolerance() -> None:
    # A residual within atol passes — proving the tolerance is actually applied.
    result = FlowResult(legs=(Leg(ATM_C, -5.0), Leg(PLANT_C, 5.0 + 1e-12)))
    assert_flow_balanced(result, _stocks())


def test_assert_flow_balanced_relative_tolerance_scales_with_magnitude() -> None:
    stocks = _stocks()
    # rtol * scale dominates atol at large magnitude: residual 1e-4 vs ~1e-3 slack.
    ok = FlowResult(legs=(Leg(ATM_C, -1e6), Leg(PLANT_C, 1e6 + 1e-4)))
    assert_flow_balanced(ok, stocks)
    # A residual an order of magnitude above the slack must still fail.
    bad = FlowResult(legs=(Leg(ATM_C, -1e6), Leg(PLANT_C, 1e6 + 1e-2)))
    with pytest.raises(ConservationError):
        assert_flow_balanced(bad, stocks)


# --- Flow purity -----------------------------------------------------------
def test_flow_evaluate_is_pure_and_deterministic() -> None:
    flow = _ScaledTransferFlow(
        id=FlowId("photosynthesis"),
        priority=0,
        src=ATM_C,
        dst=PLANT_C,
        frac=0.1,
    )
    state = State(n=3, stocks=_stocks(), rng_seed=0)
    env = _NullEnv()
    first = flow.evaluate(state, env, dt=1.0)
    second = flow.evaluate(state, env, dt=1.0)
    assert first == second  # same snapshot ⇒ identical result
    assert isinstance(flow, Flow)
    assert_flow_balanced(first, state.stocks)  # the transfer conserves carbon


# --- domains_touched / cross-domain ----------------------------------------
def test_domains_touched_single_domain() -> None:
    result = FlowResult(legs=(Leg(ATM_C, -5.0), Leg(PLANT_C, 5.0)))
    assert domains_touched(result, _stocks()) == frozenset({DomainId("bio")})


def test_domains_touched_cross_domain_harvest_is_balanced() -> None:
    # Synthetic Harvest: plant carbon -> outside (boundary) reservoir. Step-10
    # builds the real demo; here it only exercises the cross-domain path.
    harvest = FlowResult(legs=(Leg(PLANT_C, -2.0), Leg(OUTSIDE_C, 2.0)))
    stocks = _stocks()
    assert domains_touched(harvest, stocks) == frozenset(
        {DomainId("bio"), DomainId("boundary")}
    )
    assert_flow_balanced(harvest, stocks)  # balances once the boundary is counted
