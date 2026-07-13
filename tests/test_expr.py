"""Step-2 tests for the bounded kinetics VM (``simcore.expr``): AST evaluator + flow.

The evaluator is exercised **op-for-op** against plain Python arithmetic (the same
op, same order the Rust port will mirror), and :class:`DeclarativeFlow` is checked for
balance, leg structure, and the increment-form (``rate → × dt``) contract. These are
the "the VM computes what it says" unit tests; the *faithfulness to a frozen flow*
proof is the re-expression anchor in ``test_authoring_kinetics``.
"""

import pytest

from simcore.expr import (
    BinOp,
    Const,
    DeclarativeFlow,
    ForcingRef,
    Neg,
    ParamRef,
    StepN,
    StockRef,
    eval_expr,
)
from simcore.flow import assert_flow_balanced
from simcore.ids import DomainId, FlowId, StockId
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import State, Stock

BATTERY = StockId("power.battery")
WASTE_HEAT = StockId("boundary.waste_heat")


class _DictEnv:
    """A minimal ``Environment`` resolving forcing vars from a dict."""

    def __init__(self, values: dict[str, float]) -> None:
        self._values = values

    def get(self, var: str) -> float:
        return self._values[var]


def _energy_stock(sid: StockId, amount: float, kind: StockKind) -> Stock:
    return Stock(
        id=sid,
        domain=DomainId(sid.split(".")[0]),
        quantity=Quantity.ENERGY,
        unit=canonical_unit(Quantity.ENERGY),
        amount=amount,
        kind=kind,
    )


def _state(battery: float = 5.0, n: int = 3) -> State:
    stocks = {
        BATTERY: _energy_stock(BATTERY, battery, StockKind.POOL),
        WASTE_HEAT: _energy_stock(WASTE_HEAT, 0.0, StockKind.BOUNDARY),
    }
    return State(n=n, stocks=stocks, rng_seed=0)


def _eval(node, *, battery: float = 5.0, n: int = 3, env=None, params=None) -> float:
    return eval_expr(
        node,
        _state(battery=battery, n=n),
        env if env is not None else _DictEnv({}),
        params or {},
    )


# --- leaf reads ------------------------------------------------------------
def test_const_evaluates_to_its_value() -> None:
    assert _eval(Const(2.5)) == 2.5


def test_stock_ref_reads_the_snapshot_amount() -> None:
    assert _eval(StockRef(BATTERY), battery=7.25) == 7.25


def test_param_ref_reads_the_param_map() -> None:
    assert _eval(ParamRef("k"), params={"k": 1.0e-8}) == 1.0e-8


def test_forcing_ref_reads_through_env() -> None:
    assert _eval(ForcingRef("load_power"), env=_DictEnv({"load_power": 42.0})) == 42.0


def test_step_n_reads_n_as_float() -> None:
    value = _eval(StepN(), n=9)
    assert value == 9.0
    assert isinstance(value, float)


def test_missing_param_raises_keyerror() -> None:
    # Belt-and-suspenders: the interpreter validates refs at build time; the raw VM
    # still surfaces a missing param as a KeyError rather than a wrong answer.
    with pytest.raises(KeyError):
        _eval(ParamRef("absent"), params={})


# --- operators, op-for-op --------------------------------------------------
def test_addition() -> None:
    assert _eval(BinOp("+", Const(0.1), Const(0.2))) == 0.1 + 0.2


def test_subtraction() -> None:
    assert _eval(BinOp("-", Const(1.0), Const(0.3))) == 1.0 - 0.3


def test_multiplication() -> None:
    assert _eval(BinOp("*", Const(1.0e-8), StockRef(BATTERY)), battery=1.0e7) == (
        1.0e-8 * 1.0e7
    )


def test_unary_negation() -> None:
    assert _eval(Neg(StockRef(BATTERY)), battery=3.0) == -3.0


def test_negation_is_exact_for_zero() -> None:
    # -1.0*x vs -x sign-of-zero equivalence is what makes DeclarativeFlow's leg
    # formula bit-identical to a frozen ``-leak``; assert the VM's Neg matches.
    import math

    assert math.copysign(1.0, _eval(Neg(Const(0.0)))) == -1.0


def test_left_associativity_of_subtraction() -> None:
    # (10 - 3) - 2, not 10 - (3 - 2): the evaluator combines left before right.
    expr = BinOp("-", BinOp("-", Const(10.0), Const(3.0)), Const(2.0))
    assert _eval(expr) == (10.0 - 3.0) - 2.0


def test_nested_rate_expression_op_for_op() -> None:
    # k * (battery - n): a demand-shaped rate, evaluated in the fixed op-order.
    expr = BinOp("*", ParamRef("k"), BinOp("-", StockRef(BATTERY), StepN()))
    value = _eval(expr, battery=8.0, n=2, params={"k": 0.5})
    assert value == 0.5 * (8.0 - 2.0)


def test_unsupported_binop_raises() -> None:
    with pytest.raises(ValueError, match="unsupported binary op"):
        _eval(BinOp("/", Const(1.0), Const(2.0)))


# --- DeclarativeFlow -------------------------------------------------------
def _self_discharge_flow(k: float) -> DeclarativeFlow:
    """The anchor's flow, built directly: rate ``k·battery``, −1/+1 ENERGY split."""
    return DeclarativeFlow(
        id=FlowId("power.self_discharge"),
        priority=0,
        rate=BinOp("*", ParamRef("k"), StockRef(BATTERY)),
        stoichiometry=((BATTERY, -1.0), (WASTE_HEAT, 1.0)),
        params=(("k", k),),
    )


def test_declarative_flow_legs_and_increment_form() -> None:
    flow = _self_discharge_flow(1.0e-8)
    result = flow.evaluate(_state(battery=1.0e7), _DictEnv({}), 3600.0)
    leak = (1.0e-8 * 1.0e7) * 3600.0
    # Legs are emitted in author (stoichiometry) order; each lands on a distinct stock.
    assert tuple(leg.stock for leg in result.legs) == (BATTERY, WASTE_HEAT)
    amounts = {leg.stock: leg.amount for leg in result.legs}
    assert amounts[BATTERY] == -leak
    assert amounts[WASTE_HEAT] == leak


def test_declarative_flow_is_energy_balanced() -> None:
    flow = _self_discharge_flow(1.0e-8)
    state = _state(battery=1.0e7)
    result = flow.evaluate(state, _DictEnv({}), 3600.0)
    # Raises ConservationError if unbalanced; the −1/+1 split balances exactly.
    assert_flow_balanced(result, state.stocks)


def test_declarative_flow_is_dt_linear() -> None:
    flow = _self_discharge_flow(1.0e-8)
    state = _state(battery=1.0e7)
    small = flow.evaluate(state, _DictEnv({}), 1.0)
    big = flow.evaluate(state, _DictEnv({}), 10.0)
    small_battery = {leg.stock: leg.amount for leg in small.legs}[BATTERY]
    big_battery = {leg.stock: leg.amount for leg in big.legs}[BATTERY]
    # The rate is dt-independent, so the increment scales exactly linearly in dt.
    assert big_battery == small_battery * 10.0


def test_declarative_flow_zero_battery_is_no_op() -> None:
    flow = _self_discharge_flow(1.0e-8)
    result = flow.evaluate(_state(battery=0.0), _DictEnv({}), 3600.0)
    assert all(leg.amount == 0.0 for leg in result.legs)
