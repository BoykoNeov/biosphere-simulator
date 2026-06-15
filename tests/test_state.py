"""Step-2 tests for the immutable Stock / State primitives."""

import dataclasses
import math

import pytest

from simcore.ids import DomainId, StockId, UnitLabel
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import State, Stock


def _stock(sid: str = "bio.plant_c", amount: float = 1.0) -> Stock:
    return Stock(
        id=StockId(sid),
        domain=DomainId("bio"),
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=amount,
        kind=StockKind.POOL,
    )


# --- Stock -----------------------------------------------------------------
def test_stock_is_frozen() -> None:
    stock = _stock()
    with pytest.raises(dataclasses.FrozenInstanceError):
        stock.amount = 2.0  # type: ignore[misc]


def test_stock_replace_makes_a_new_stock() -> None:
    stock = _stock(amount=1.0)
    updated = dataclasses.replace(stock, amount=2.0)
    assert stock.amount == 1.0  # original untouched
    assert updated.amount == 2.0
    assert updated.id == stock.id


def test_stock_defaults() -> None:
    stock = _stock()
    assert stock.extinction_threshold == 0.0
    assert stock.unclamped is False


@pytest.mark.parametrize("bad", [float("nan"), float("inf"), float("-inf")])
def test_stock_rejects_non_finite_amount(bad: float) -> None:
    with pytest.raises(ValueError, match="not finite"):
        _stock(amount=bad)


def test_stock_rejects_non_finite_threshold() -> None:
    with pytest.raises(ValueError, match="not finite"):
        Stock(
            id=StockId("p"),
            domain=DomainId("bio"),
            quantity=Quantity.CARBON,
            unit=UnitLabel("mol"),
            amount=1.0,
            kind=StockKind.POPULATION,
            extinction_threshold=math.inf,
        )


# --- State -----------------------------------------------------------------
def test_state_is_frozen() -> None:
    state = State(n=0, stocks={}, rng_seed=0)
    with pytest.raises(dataclasses.FrozenInstanceError):
        state.n = 1  # type: ignore[misc]


def test_state_stocks_mapping_is_read_only() -> None:
    stock = _stock()
    state = State(n=0, stocks={stock.id: stock}, rng_seed=0)
    with pytest.raises(TypeError):
        state.stocks[StockId("x")] = stock  # type: ignore[index]


def test_state_detaches_from_caller_dict() -> None:
    stock = _stock()
    source = {stock.id: stock}
    state = State(n=0, stocks=source, rng_seed=0)
    source[StockId("late")] = _stock("late")  # mutate caller's dict afterwards
    assert StockId("late") not in state.stocks  # snapshot is unaffected


def test_state_rejects_negative_n() -> None:
    with pytest.raises(ValueError, match="n must be >= 0"):
        State(n=-1, stocks={}, rng_seed=0)


def test_state_rejects_key_id_mismatch() -> None:
    stock = _stock("real_id")
    with pytest.raises(ValueError, match="!= stock.id"):
        State(n=0, stocks={StockId("wrong_key"): stock}, rng_seed=0)


def test_state_round_trips_lookup() -> None:
    stock = _stock()
    state = State(n=5, stocks={stock.id: stock}, rng_seed=99)
    assert state.n == 5
    assert state.rng_seed == 99
    assert state.stocks[stock.id] is stock
