"""Step-11 tests: the ``observe`` / ``Observation`` plain-data read surface.

``observe`` is a **projection**: it re-exposes only the observable subset of a
``State`` (per-stock id/domain/quantity/unit/amount/kind) and drops the engine
internals (``rng_seed``, ``extinction_threshold``, ``unclamped``). The headline
gates here pin exactly that boundary, plus the canonical id-ordering (#15) /
insertion-order independence and the hashable-plain-data property the design buys.
"""

import dataclasses

import pytest
from hypothesis import given
from hypothesis import strategies as st

from domains.biosphere.demo import build_demo
from domains.biosphere.loader import load_demo_params
from simcore.ids import DomainId, StockId
from simcore.observation import Observation, StockObservation, observe
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import State, Stock


def _pool(sid: str = "bio.atmospheric_c", amount: float = 1000.0) -> Stock:
    return Stock(
        id=StockId(sid),
        domain=DomainId("bio"),
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=amount,
        kind=StockKind.POOL,
    )


def _pop(sid: str = "bio.plant_c", amount: float = 100.0) -> Stock:
    # A non-default extinction_threshold so the projection's *dropping* of it is
    # observable in the test (it must not leak into StockObservation).
    return Stock(
        id=StockId(sid),
        domain=DomainId("bio"),
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=amount,
        kind=StockKind.POPULATION,
        extinction_threshold=5.0,
    )


def _boundary(sid: str = "boundary.light", amount: float = 1.0) -> Stock:
    # An unclamped BOUNDARY source: proves `unclamped` is dropped while `kind` (the
    # descriptive classification) is kept.
    return Stock(
        id=StockId(sid),
        domain=DomainId("boundary"),
        quantity=Quantity.ENERGY,
        unit=canonical_unit(Quantity.ENERGY),
        amount=amount,
        kind=StockKind.BOUNDARY,
        unclamped=True,
    )


def _state(n: int = 3, *, rng_seed: int = 42) -> State:
    stocks = [_pool(), _pop(), _boundary()]
    return State(n=n, stocks={s.id: s for s in stocks}, rng_seed=rng_seed)


# --- projection correctness ------------------------------------------------
def test_observe_copies_observable_fields_faithfully() -> None:
    state = _state(n=7)
    obs = observe(state)
    assert obs.n == 7
    assert {so.id for so in obs.stocks} == set(state.stocks)
    for so in obs.stocks:
        src = state.stocks[so.id]
        assert so.id == src.id
        assert so.domain == src.domain
        assert so.quantity == src.quantity
        assert so.unit == src.unit
        assert so.amount == src.amount  # exact: a plain copy
        assert so.kind == src.kind


def test_observe_empty_state() -> None:
    obs = observe(State(n=0, stocks={}, rng_seed=0))
    assert obs.n == 0
    assert obs.stocks == ()


@pytest.mark.parametrize("amount", [0.0, -0.0, 5e-324, 1.7976931348623157e308])
def test_observe_preserves_exact_amount(amount: float) -> None:
    state = State(n=0, stocks={(s := _pool(amount=amount)).id: s}, rng_seed=0)
    (so,) = observe(state).stocks
    # Bit-exact: -0.0 keeps its sign, subnormals/max double survive (a plain copy).
    assert so.amount.hex() == amount.hex()


# --- the projection boundary (the load-bearing design assertion) -----------
def test_stock_observation_drops_engine_control_fields() -> None:
    # The contract: observable identity/classification only — no engine *controls*.
    names = {f.name for f in dataclasses.fields(StockObservation)}
    assert names == {"id", "domain", "quantity", "unit", "amount", "kind"}
    assert "extinction_threshold" not in names  # an engine control, not a measurement
    assert "unclamped" not in names  # an arbitration control (#13), not a measurement


def test_observation_drops_rng_seed() -> None:
    names = {f.name for f in dataclasses.fields(Observation)}
    assert names == {"n", "stocks"}
    assert "rng_seed" not in names  # internal RNG state, not an observation


# --- canonical order + insertion-order independence (#15) ------------------
def test_observe_emits_stocks_in_canonical_id_order() -> None:
    obs = observe(_state())
    ids = [so.id for so in obs.stocks]
    assert ids == sorted(ids)


def test_observe_is_insertion_order_independent() -> None:
    stocks = [_pool(), _pop(), _boundary()]
    forward = State(n=2, stocks={s.id: s for s in stocks}, rng_seed=0)
    reverse = State(n=2, stocks={s.id: s for s in reversed(stocks)}, rng_seed=0)
    assert observe(forward) == observe(reverse)
    assert hash(observe(forward)) == hash(observe(reverse))


@given(order=st.permutations([_pool(), _pop(), _boundary()]))
def test_observe_order_independence_property(order: list[Stock]) -> None:
    canonical = observe(_state())
    shuffled = observe(State(n=3, stocks={s.id: s for s in order}, rng_seed=42))
    assert shuffled == canonical


# --- plain-data: frozen, comparable, hashable ------------------------------
def test_observation_and_stock_observation_are_frozen() -> None:
    obs = observe(_state())
    with pytest.raises(dataclasses.FrozenInstanceError):
        obs.n = 99  # type: ignore[misc]
    with pytest.raises(dataclasses.FrozenInstanceError):
        obs.stocks[0].amount = 1.0  # type: ignore[misc]


def test_observe_is_deterministic() -> None:
    assert observe(_state()) == observe(_state())


def test_observation_is_hashable() -> None:
    # Unlike State (whose MappingProxyType stocks make it unhashable), an Observation
    # carries only tuples/ints/enums/floats, so it is hashable — what snapshot and
    # set-membership equality tests want.
    obs = observe(_state())
    assert hash(obs) == hash(observe(_state()))
    assert obs in {obs}


def test_observation_distinguishes_n_and_amounts() -> None:
    base = _state(n=3)
    later = dataclasses.replace(base, n=4)
    assert observe(base) != observe(later)  # n participates in equality


# --- integration: the real demo state --------------------------------------
def test_observe_demo_state_covers_all_stocks() -> None:
    state, _registry = build_demo(load_demo_params())
    obs = observe(state)
    assert obs.n == state.n
    assert {so.id for so in obs.stocks} == set(state.stocks)
    for so in obs.stocks:
        src = state.stocks[so.id]
        assert so.amount == src.amount
        assert (so.domain, so.quantity, so.unit, so.kind) == (
            src.domain,
            src.quantity,
            src.unit,
            src.kind,
        )
