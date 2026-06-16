"""Step-7 tests: the events module (the discrete-event record type).

Behavioural coverage of extinction events lives in ``test_extinction.py``; this
pins the data shape — ``ExtinctionEvent`` is a frozen, equality-by-value record.
"""

import dataclasses

import pytest

from simcore.events import Event, ExtinctionEvent
from simcore.ids import StockId
from simcore.quantities import Quantity


def test_extinction_event_is_frozen() -> None:
    ev = ExtinctionEvent(
        n=3, stock=StockId("bio.pop"), quantity=Quantity.CARBON, residual=0.5
    )
    with pytest.raises(dataclasses.FrozenInstanceError):
        ev.residual = 1.0  # type: ignore[misc]


def test_extinction_event_equality_is_by_value() -> None:
    a = ExtinctionEvent(3, StockId("bio.pop"), Quantity.CARBON, 0.5)
    b = ExtinctionEvent(3, StockId("bio.pop"), Quantity.CARBON, 0.5)
    assert a == b


def test_event_alias_is_the_extinction_event_type() -> None:
    # The Phase-0 event union is a single type; StepReport.events is annotated with
    # this alias so widening later is a one-line change.
    assert Event is ExtinctionEvent
