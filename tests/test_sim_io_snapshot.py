"""Step-9 tests: outer ``sim_io`` state snapshot round-trip (JSON, hex-float).

Realizes the "Serialization round-trip" exit gate: ``state -> plain data -> state``
reproduces a **bit-identical continuation**, with **hex-float** in the golden file
for exact cross-run / cross-port comparison.

Coverage:
  * round-trip equality (``loads(dumps(s)) == s``) and bit-exact continuation (a
    step on the round-tripped state matches a step on the original);
  * hex-float exactness for *nasty* doubles — ``0.1``, ``pi``, signed ``-0.0``,
    smallest subnormal, large magnitude — survive bit-for-bit (asserted via
    ``.hex()``, which distinguishes ``-0.0`` from ``0.0`` where ``==`` would not);
  * ``rng_seed`` above 2**53 survives (the cross-port precision fix: stored as a
    hex string, not a JSON number);
  * a committed golden's exact **bytes** match ``dumps`` (format/cross-port pin)
    and the golden loads back to the same state;
  * insertion-order independence of the serialized bytes (canonical #15);
  * reconstruction through the core constructors rejects a tampered (NaN/Inf,
    unclamped-non-boundary) golden, and an unknown schema version is rejected.
"""

import dataclasses
import math
from pathlib import Path

import pytest

import sim_io
from sim_io import snapshot
from simcore import boundary
from simcore.environment import Environment, SourceResolver
from simcore.flow import FlowResult, Leg
from simcore.ids import DomainId, FlowId, StockId
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock

GOLDEN = Path(__file__).parent / "regression" / "golden" / "state_snapshot.json"

# A 64-bit seed deliberately above 2**53 (9_007_199_254_740_992): a JSON *number*
# would lose precision in an f64 reader (Rust/JS), which is why the seed is stored
# as a hex string. A Python-only round-trip would otherwise hide that bug.
_BIG_SEED = 0x0123456789ABCDEF


def _golden_state() -> State:
    """A representative state spanning every stock kind and several nasty floats.

    The single source of truth for both the committed golden file and the tests
    that pin it. Stocks intentionally cover POOL / POPULATION / BOUNDARY, an
    ``unclamped`` source, a loss-sink, and the awkward doubles ``pi``, ``0.1``,
    ``-0.0`` (signed zero), and the smallest positive subnormal. ``aux`` (the
    non-conserved channel, P2) carries its own nasty floats — a representative
    ``thermal_time`` plus ``-0.0`` and the smallest subnormal — to pin aux
    hex-float exactness (incl. the sign bit) in the committed golden.
    """
    stocks = [
        Stock(
            id=StockId("bio.atmo_c"),
            domain=DomainId("bio"),
            quantity=Quantity.CARBON,
            unit=canonical_unit(Quantity.CARBON),
            amount=math.pi,
            kind=StockKind.POOL,
        ),
        Stock(
            id=StockId("bio.plant_c"),
            domain=DomainId("bio"),
            quantity=Quantity.CARBON,
            unit=canonical_unit(Quantity.CARBON),
            amount=0.1,
            kind=StockKind.POPULATION,
            extinction_threshold=1e-6,
        ),
        Stock(
            id=StockId("bio.water"),
            domain=DomainId("bio"),
            quantity=Quantity.WATER,
            unit=canonical_unit(Quantity.WATER),
            amount=5e-324,  # smallest positive subnormal double
            kind=StockKind.POOL,
        ),
        # Signed zero: a loss-sink seeded at -0.0 to prove hex-float keeps the sign
        # bit (which value equality would silently lose).
        dataclasses.replace(boundary.loss_sink(Quantity.CARBON), amount=-0.0),
        boundary.source(StockId("boundary.solar"), Quantity.ENERGY, 1e300),
    ]
    aux = {"thermal_time": math.pi, "neg_zero": -0.0, "subnormal": 5e-324}
    return State(n=42, stocks={s.id: s for s in stocks}, rng_seed=_BIG_SEED, aux=aux)


# --- round-trip ------------------------------------------------------------
def test_round_trip_equal() -> None:
    state = _golden_state()
    assert sim_io.loads(sim_io.dumps(state)) == state


def test_round_trip_is_bit_exact_per_amount() -> None:
    # Value equality (==) treats -0.0 == 0.0 and ignores bit identity; assert the
    # exact hex of every float field survives instead.
    state = _golden_state()
    back = sim_io.loads(sim_io.dumps(state))
    for sid, stock in state.stocks.items():
        assert back.stocks[sid].amount.hex() == stock.amount.hex()
        assert (
            back.stocks[sid].extinction_threshold.hex()
            == stock.extinction_threshold.hex()
        )


def test_aux_round_trip_is_bit_exact() -> None:
    # Aux values round-trip bit-for-bit via hex-float. Asserted via .hex() (not ==)
    # so the ``-0.0`` sign bit and the subnormal survive — the same reason the
    # per-amount bit-exact test exists.
    state = _golden_state()
    back = sim_io.loads(sim_io.dumps(state))
    assert back.aux.keys() == state.aux.keys()
    for name, value in state.aux.items():
        assert back.aux[name].hex() == value.hex()


def test_dumps_is_idempotent() -> None:
    # Serializing the round-tripped state yields identical text (no drift on a
    # second pass — the format is a fixed point).
    state = _golden_state()
    text = sim_io.dumps(state)
    assert sim_io.dumps(sim_io.loads(text)) == text


@pytest.mark.parametrize(
    "value",
    [0.0, -0.0, 0.1, math.pi, 1e300, 5e-324, -2.5, 1.0 / 3.0],
)
def test_hex_float_amount_exact(value: float) -> None:
    stock = Stock(
        id=StockId("bio.x"),
        domain=DomainId("bio"),
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=value,
        kind=StockKind.POOL,
    )
    state = State(n=0, stocks={stock.id: stock}, rng_seed=0)
    back = sim_io.loads(sim_io.dumps(state))
    assert back.stocks[stock.id].amount.hex() == value.hex()


def test_rng_seed_above_2_53_survives() -> None:
    # The cross-port precision fix: a >2**53 seed must come back exactly.
    state = State(n=0, stocks={}, rng_seed=_BIG_SEED)
    assert sim_io.loads(sim_io.dumps(state)).rng_seed == _BIG_SEED
    # And it is stored as a string, not a JSON number, in the serialized form.
    assert isinstance(sim_io.state_to_dict(state)["rng_seed"], str)


# --- bit-identical continuation -------------------------------------------
@dataclasses.dataclass(frozen=True)
class _DecayFlow:
    """``src -> boundary sink`` first-order decay (dt-linear, balanced)."""

    id: FlowId
    priority: int
    src: StockId
    sink: StockId
    rate: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        amount = self.rate * snapshot.stocks[self.src].amount * dt
        return FlowResult(legs=(Leg(self.src, -amount), Leg(self.sink, amount)))


def test_continuation_is_bit_identical() -> None:
    # The gate's headline: a step taken on the round-tripped state is bit-identical
    # to a step on the original (round-trip preserves every bit a pure step reads).
    src = StockId("bio.c")
    sink = StockId("boundary.sink")
    stocks = [
        Stock(
            src,
            DomainId("bio"),
            Quantity.CARBON,
            canonical_unit(Quantity.CARBON),
            3.0,
            StockKind.POOL,
        ),
        boundary.sink(sink, Quantity.CARBON),
    ]
    state = State(n=7, stocks={s.id: s for s in stocks}, rng_seed=_BIG_SEED)

    registry = Registry(
        flows=[_DecayFlow(FlowId("decay"), 0, src, sink, 0.25)],
        stocks=state.stocks,
    )
    integrator = EulerIntegrator(registry)
    resolver = SourceResolver()

    roundtrip = sim_io.loads(sim_io.dumps(state))
    original_next = integrator.step(state, resolver, dt=1.0)
    roundtrip_next = integrator.step(roundtrip, resolver, dt=1.0)
    assert sim_io.dumps(original_next) == sim_io.dumps(roundtrip_next)


# --- golden file -----------------------------------------------------------
def test_golden_bytes_match() -> None:
    # Byte-exact (not text-mode) comparison: pins the on-disk format and is the
    # cross-port conformance target. A drift in field order, hex formatting, or the
    # seed encoding fails here.
    assert sim_io.dumps(_golden_state()).encode("utf-8") == GOLDEN.read_bytes()


def test_golden_loads_to_state() -> None:
    assert sim_io.loads(GOLDEN.read_text(encoding="utf-8")) == _golden_state()


# --- canonical order / insertion-order independence ------------------------
def test_serialization_is_insertion_order_independent() -> None:
    # Shuffling BOTH the stock and the aux insertion order must not change the bytes
    # (canonical id/key sort, #15) — stocks are pre-sorted as a list, aux keys by
    # json.dumps(sort_keys=True) + the explicit pre-sort.
    state = _golden_state()
    shuffled_stocks = dict(list(state.stocks.items())[::-1])
    shuffled_aux = dict(list(state.aux.items())[::-1])
    shuffled = State(
        n=state.n, stocks=shuffled_stocks, rng_seed=state.rng_seed, aux=shuffled_aux
    )
    assert sim_io.dumps(shuffled) == sim_io.dumps(state)


def test_stocks_emitted_in_canonical_id_order() -> None:
    stocks = sim_io.state_to_dict(_golden_state())["stocks"]
    assert isinstance(stocks, list)
    ids = [s["id"] for s in stocks]
    assert ids == sorted(ids)


def test_aux_emitted_in_canonical_key_order() -> None:
    aux = sim_io.state_to_dict(_golden_state())["aux"]
    assert isinstance(aux, dict)
    keys = list(aux)
    assert keys == sorted(keys)


# --- fail-loud reconstruction ----------------------------------------------
@pytest.mark.parametrize("bad", ["nan", "inf", "-inf"])
def test_tampered_non_finite_amount_rejected(bad: str) -> None:
    # A hand-edited golden with a non-finite hex amount is rejected because
    # reconstruction goes through Stock.__post_init__ (no malformed state escapes).
    data = sim_io.state_to_dict(_golden_state())
    stocks = data["stocks"]
    assert isinstance(stocks, list)
    stocks[0]["amount"] = bad
    with pytest.raises(ValueError, match="not finite"):
        sim_io.state_from_dict(data)


def test_tampered_unclamped_non_boundary_rejected() -> None:
    # The unclamped ⇒ BOUNDARY guard also re-fires on load.
    data = sim_io.state_to_dict(_golden_state())
    stocks = data["stocks"]
    assert isinstance(stocks, list)
    pool = next(s for s in stocks if s["kind"] == StockKind.POOL.value)
    pool["unclamped"] = True
    with pytest.raises(ValueError, match="unclamped"):
        sim_io.state_from_dict(data)


@pytest.mark.parametrize("version", [0, 1, 2, 4, "3", None])
def test_unknown_schema_version_rejected(version: object) -> None:
    # Includes the now-superseded v1/v2: a stale golden is rejected outright (no
    # migration machinery; the v1->v2 aux and v2->v3 composition bumps each
    # regenerated all goldens).
    data = sim_io.state_to_dict(_golden_state())
    data["version"] = version
    with pytest.raises(ValueError, match="schema version"):
        sim_io.state_from_dict(data)


def test_missing_version_rejected() -> None:
    data = sim_io.state_to_dict(_golden_state())
    del data["version"]
    with pytest.raises(ValueError, match="schema version"):
        sim_io.state_from_dict(data)


def test_schema_version_constant_exposed() -> None:
    assert sim_io.SCHEMA_VERSION == snapshot.SCHEMA_VERSION == 3


def test_multikey_composition_round_trips_and_key_sorts() -> None:
    # The committed golden is all 1:1 (it only ever exercises ``{q: "0x1p+0"}``), so
    # a non-unit, multi-key composition gets its own round-trip with teeth: a CO2-like
    # POOL ``{CARBON:1, OXYGEN:2}`` must survive bit-for-bit and serialize key-sorted.
    co2 = Stock(
        id=StockId("bio.co2"),
        domain=DomainId("bio"),
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=math.pi,
        kind=StockKind.POOL,
        composition={Quantity.CARBON: 1.0, Quantity.OXYGEN: 2.0},
    )
    state = State(n=0, stocks={co2.id: co2}, rng_seed=0)

    assert sim_io.loads(sim_io.dumps(state)) == state  # bit-identical round-trip

    data = sim_io.state_to_dict(state)
    stocks = data["stocks"]
    assert isinstance(stocks, list)
    comp = stocks[0]["composition"]
    # Key-sorted by quantity value (canonical order, #15); coeffs as exact hex-floats.
    assert list(comp.keys()) == ["carbon", "oxygen"]
    assert comp["carbon"] == (1.0).hex()
    assert comp["oxygen"] == (2.0).hex()


def _regenerate() -> None:
    """Rewrite the committed ``state_snapshot.json`` golden from the current serializer.

    A deliberately separate, explicit action — NOT reachable from a test run — so a
    verify run can never overwrite the golden it is meant to check (mirrors
    ``test_regression_demo._regenerate``). Run via::

        uv run python tests/test_sim_io_snapshot.py

    Review the diff before committing: a change here means the on-disk format moved
    (e.g. a schema bump like the v1→v2 aux addition).
    """
    GOLDEN.write_bytes(sim_io.dumps(_golden_state()).encode("utf-8"))
    print(f"wrote {GOLDEN}")


if __name__ == "__main__":
    _regenerate()
