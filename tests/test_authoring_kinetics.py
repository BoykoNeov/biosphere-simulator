"""Step-2 tests for authored kinetics end-to-end: the parser, the safety spine, and
the **re-expression anchor** (the load-bearing "the VM is faithful" proof).

The anchor re-expresses the frozen Power ``SelfDischarge`` flow as a declarative
``kinetics`` flow and asserts the interpreted run is **bit-identical** to the frozen
constructor's trajectory, under Euler AND RK4 — the (B)-analogue of Step 0's
byte-identity anchor, with the frozen flow (not a new golden) as the oracle.
"""

from __future__ import annotations

from pathlib import Path

import pytest

import sim_io
from authoring.errors import AuthoringError
from authoring.expr_parser import parse_rate_expr
from authoring.interpreter import interpret, load_scenario
from authoring.schema import ScenarioSpec
from domains.power.flows import SelfDischarge
from domains.power.loader import load_self_discharge_params
from simcore.expr import BinOp, Const, ForcingRef, Neg, ParamRef, StepN, StockRef
from simcore.ids import FlowId, StockId
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.registry import Registry
from simcore.state import State

SCENARIOS = Path(__file__).parent / "authoring" / "scenarios"
DSL_SCENARIO = SCENARIOS / "self_discharge_dsl.yaml"


# --- the parser (text -> AST) ---------------------------------------------
def test_parse_leaf_forms() -> None:
    assert parse_rate_expr("1.5") == Const(1.5)
    assert parse_rate_expr('stock("power.battery")') == StockRef(
        StockId("power.battery")
    )
    assert parse_rate_expr('param("k")') == ParamRef("k")
    assert parse_rate_expr('forcing("load_power")') == ForcingRef("load_power")
    assert parse_rate_expr("n") == StepN()


def test_parse_self_discharge_rate() -> None:
    # The anchor's rate: param("self_discharge_rate") * stock("power.battery").
    ast = parse_rate_expr('param("self_discharge_rate") * stock("power.battery")')
    assert ast == BinOp(
        "*", ParamRef("self_discharge_rate"), StockRef(StockId("power.battery"))
    )


def test_parse_precedence_and_associativity() -> None:
    # a + b * c  ==  a + (b * c): * binds tighter than +.
    assert parse_rate_expr("1.0 + 2.0 * 3.0") == BinOp(
        "+", Const(1.0), BinOp("*", Const(2.0), Const(3.0))
    )
    # a - b - c  ==  (a - b) - c: left-associative.
    assert parse_rate_expr("10.0 - 3.0 - 2.0") == BinOp(
        "-", BinOp("-", Const(10.0), Const(3.0)), Const(2.0)
    )
    # Parens override precedence.
    assert parse_rate_expr("(1.0 + 2.0) * 3.0") == BinOp(
        "*", BinOp("+", Const(1.0), Const(2.0)), Const(3.0)
    )
    # Unary minus binds tighter than *: -a * b == (-a) * b.
    assert parse_rate_expr('-param("k") * n') == BinOp("*", Neg(ParamRef("k")), StepN())


@pytest.mark.parametrize(
    "text",
    [
        "",  # empty
        "1.0 +",  # trailing operator
        "1.0 2.0",  # trailing junk
        "(1.0",  # unbalanced paren
        "1.0 / 2.0",  # deferred operator
        "exp(1.0)",  # deferred function / unknown identifier
        "stock(power.battery)",  # unquoted ref argument
        'stock("")',  # empty ref argument
        "@",  # stray character
    ],
)
def test_parse_errors_are_authoring_errors(text: str) -> None:
    with pytest.raises(AuthoringError):
        parse_rate_expr(text)


@pytest.mark.parametrize(
    "text",
    [
        "1.5",
        "1000.0",
        "1.0e-8",
        'stock("power.battery")',
        'param("self_discharge_rate") * stock("power.battery")',
        "1.0 + 2.0 * 3.0",
        "10.0 - 3.0 - 2.0",
        "(1.0 + 2.0) * 3.0",
        '-param("k") * n',
        'forcing("load_power") + n',
    ],
)
def test_render_rate_expr_round_trips(text: str) -> None:
    # `render_rate_expr` is the inverse of `parse_rate_expr` used by Step-6c
    # id-namespacing to re-emit a rate whose refs were prefixed. The contract is
    # per-port round-trip stability: parse(render(parse(text))) == parse(text). Fully
    # parenthesized, so precedence/associativity survive the round-trip.
    from authoring.expr_parser import render_rate_expr

    ast = parse_rate_expr(text)
    assert parse_rate_expr(render_rate_expr(ast)) == ast


# --- the safety spine (build-time structural checks) ----------------------
def _minimal_kinetics_spec(**flow_overrides: object) -> dict[str, object]:
    """A minimal well-formed single-kinetics-flow scenario dict (battery→waste_heat)."""
    flow: dict[str, object] = {
        "id": "power.self_discharge",
        "kinetics": {
            "rate": 'param("self_discharge_rate") * stock("power.battery")',
            "stoichiometry": {"power.battery": -1, "boundary.waste_heat": 1},
        },
        "params": "self_discharge",
    }
    flow.update(flow_overrides)
    return {
        "name": "t",
        "integrator": "euler",
        "dt": 3600.0,
        "steps": 1,
        "stocks": [
            {
                "id": "power.battery",
                "domain": "power",
                "quantity": "energy",
                "kind": "pool",
                "amount": 1.0e7,
            },
            {
                "id": "boundary.waste_heat",
                "domain": "boundary",
                "quantity": "energy",
                "kind": "boundary",
                "amount": 0.0,
            },
        ],
        "flows": [flow],
    }


def test_type_xor_kinetics_is_enforced() -> None:
    # Both type and kinetics → schema ValueError.
    with pytest.raises(ValueError, match="exactly one of"):
        ScenarioSpec.model_validate(_minimal_kinetics_spec(type="crew.food_metabolism"))
    # Neither → schema ValueError.
    bad = _minimal_kinetics_spec()
    del bad["flows"][0]["kinetics"]  # type: ignore[index]
    del bad["flows"][0]["params"]  # type: ignore[index]
    with pytest.raises(ValueError, match="exactly one of"):
        ScenarioSpec.model_validate(bad)


def test_kinetics_flow_rejects_wiring() -> None:
    with pytest.raises(ValueError, match="no 'wiring'"):
        ScenarioSpec.model_validate(
            _minimal_kinetics_spec(wiring={"foo": "power.battery"})
        )


def test_unbalanced_stoichiometry_is_rejected_at_build() -> None:
    # −1 / +2 does not conserve ENERGY: balance-by-construction gate fires at build.
    spec = ScenarioSpec.model_validate(
        _minimal_kinetics_spec(
            kinetics={
                "rate": 'param("self_discharge_rate") * stock("power.battery")',
                "stoichiometry": {"power.battery": -1, "boundary.waste_heat": 2},
            }
        )
    )
    with pytest.raises(AuthoringError, match="not balanced"):
        interpret(spec)


def test_rate_referencing_unknown_param_is_rejected() -> None:
    spec = ScenarioSpec.model_validate(
        _minimal_kinetics_spec(
            kinetics={
                "rate": 'param("nonexistent") * stock("power.battery")',
                "stoichiometry": {"power.battery": -1, "boundary.waste_heat": 1},
            }
        )
    )
    with pytest.raises(AuthoringError, match="param 'nonexistent'"):
        interpret(spec)


def test_stoichiometry_referencing_unknown_stock_is_rejected() -> None:
    spec = ScenarioSpec.model_validate(
        _minimal_kinetics_spec(
            kinetics={
                "rate": 'param("self_discharge_rate") * stock("power.battery")',
                "stoichiometry": {"power.battery": -1, "boundary.ghost": 1},
            }
        )
    )
    with pytest.raises(AuthoringError, match="unknown stock"):
        interpret(spec)


def test_param_pack_for_kinetics_is_deferred() -> None:
    spec = ScenarioSpec.model_validate(
        _minimal_kinetics_spec(params={"pack": "somewhere.yaml"})
    )
    with pytest.raises(AuthoringError, match="packs for authored"):
        interpret(spec)


# --- the re-expression anchor (bit-identity vs the frozen flow) -----------
def _run(
    registry: Registry, state: State, resolver, dt: float, steps: int, integrator_cls
) -> tuple[list[State], int]:
    integrator = integrator_cls(registry)
    states = [state]
    total_rationed = 0
    current = state
    for _ in range(steps):
        report = integrator.step_report(current, resolver, dt)
        current = report.state
        states.append(current)
        total_rationed += report.rationed
    return states, total_rationed


def _frozen_twin_registry(built) -> Registry:
    """A registry with the frozen ``SelfDischarge`` over the interpreted scenario's
    stocks — the oracle the DeclarativeFlow run must match bit-for-bit."""
    frozen = SelfDischarge(
        FlowId("power.self_discharge"),
        0,
        StockId("power.battery"),
        StockId("boundary.waste_heat"),
        load_self_discharge_params(),
    )
    return Registry([frozen], built.state.stocks)


@pytest.mark.parametrize("integrator_cls", [EulerIntegrator, Rk4Integrator])
def test_dsl_self_discharge_is_bit_identical_to_frozen(integrator_cls) -> None:
    built = load_scenario(str(DSL_SCENARIO))
    assert built.has_authored_kinetics  # the "authored ≠ validated" marker is set

    dsl_states, dsl_rationed = _run(
        built.registry,
        built.state,
        built.resolver,
        built.dt,
        built.steps,
        integrator_cls,
    )
    frozen_states, frozen_rationed = _run(
        _frozen_twin_registry(built),
        built.state,
        built.resolver,
        built.dt,
        built.steps,
        integrator_cls,
    )

    assert dsl_rationed == 0  # donor-controlled: k·dt ≪ 1, backstop never fires
    assert frozen_rationed == 0
    assert len(dsl_states) == built.steps + 1
    # Per-step State bit-identity via hex-float serialization (order- and ±0-robust).
    for i, (dsl, frozen) in enumerate(zip(dsl_states, frozen_states, strict=True)):
        assert sim_io.dumps(dsl) == sim_io.dumps(frozen), f"diverged at step {i}"


def test_dsl_self_discharge_actually_departs() -> None:
    # The anchor is not vacuous: the leak must move the battery over the run (otherwise
    # "bit-identical" is trivially true for two no-ops).
    built = load_scenario(str(DSL_SCENARIO))
    states, _ = _run(
        built.registry,
        built.state,
        built.resolver,
        built.dt,
        built.steps,
        EulerIntegrator,
    )
    battery0 = states[0].stocks[StockId("power.battery")].amount
    battery_final = states[-1].stocks[StockId("power.battery")].amount
    assert battery_final < battery0  # it bit
    waste_final = states[-1].stocks[StockId("boundary.waste_heat")].amount
    assert waste_final == pytest.approx(battery0 - battery_final)  # every joule named
