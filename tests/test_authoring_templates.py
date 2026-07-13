"""Phase-9 Step-3 gate: templates (parametrized scenario files → many habitats).

A **template** declares ``parameters`` and writes numeric fields (stock ``amount`` /
forcing ``const``) as **expressions** over them; an instantiation supplies ``overrides``
and the interpreter lowers the expressions to literals at build time (the deliberate
decision-A amendment — a new boundary-eval cross-port surface). Proven:

1. **Faithfulness → byte-identity.** The crew habitat template at ``crew_count = 1.0``
   (default, and via an explicit override) reproduces ``crew_state.json`` byte-for-byte:
   ``1.0 * base == base`` exactly, so the template lowers to the same floats as the
   literal ``crew_mission`` scenario. This validates the whole
   template→resolve→evaluate→interpret→run path against the frozen golden.
2. **The knob is load-bearing ("it bit").** At ``crew_count = 4.0`` every stock is ≈ 4×
   its single-crew value (reconstruct-to-tolerance — accumulate-then-scale ≠
   scale-then-accumulate in fp, the ``n_limited``/``water_biting`` discipline), the run
   still conserves every quantity every step, ``rationed == 0`` / ``events == ()``, and
   the final state is **not** byte-identical to the golden (a non-scaling knob could not
   move it).
3. **Referential integrity (build-time).** An override of an undeclared parameter, an
   expression referencing an undeclared parameter, and a stock/forcing/``n`` reference
   in a template expression (no ``State``/``env``/``n`` at build time) each raise
   ``AuthoringError`` — a template's parameter set is its explicit contract.

Zero core + zero domain change: the boundary evaluates ``+ − ×`` to literals the frozen
constructors receive; the engine is untouched.
"""

import math
from pathlib import Path

import pytest

import sim_io
from authoring.errors import AuthoringError
from authoring.interpreter import interpret, load_scenario
from authoring.run import run_scenario
from authoring.schema import ScenarioSpec
from authoring.template import eval_numeric_field, resolve_parameters
from simcore.conservation import compute_ledger
from simcore.quantities import Quantity

SCENARIO_DIR = Path(__file__).parent / "authoring" / "scenarios"
TEMPLATE_YAML = SCENARIO_DIR / "crew_habitat_template.yaml"
GOLDEN_PATH = Path(__file__).parent / "regression" / "golden" / "crew_state.json"


def _final_bytes(path: Path, overrides=None) -> bytes:
    states, _, _ = run_scenario(load_scenario(str(path), overrides=overrides))
    return sim_io.dumps(states[-1]).encode("utf-8")


def _final(path: Path, overrides=None):
    return run_scenario(load_scenario(str(path), overrides=overrides))


# --- 1. Faithfulness: crew_count = 1 reproduces the frozen Crew golden ---------------


def test_template_default_matches_crew_golden_bytes() -> None:
    # The default crew_count = 1.0 lowers `1.0 * base` to `base` exactly.
    assert _final_bytes(TEMPLATE_YAML) == GOLDEN_PATH.read_bytes()


def test_template_explicit_unit_override_matches_golden_bytes() -> None:
    # An explicit crew_count = 1.0 override is identical to the default.
    assert _final_bytes(TEMPLATE_YAML, {"crew_count": 1.0}) == GOLDEN_PATH.read_bytes()


# --- 2. The knob is load-bearing: crew_count = 4 scales everything 4× ----------------


def test_template_crew_count_four_scales_all_stocks() -> None:
    one_states, _, _ = _final(TEMPLATE_YAML, {"crew_count": 1.0})
    four_states, four_rationed, four_events = _final(TEMPLATE_YAML, {"crew_count": 4.0})
    assert four_rationed == 0
    assert four_events == ()
    one, four = one_states[-1], four_states[-1]
    assert set(one.stocks) == set(four.stocks)
    for sid in one.stocks:
        one_amt = one.stocks[sid].amount
        four_amt = four.stocks[sid].amount
        if one_amt == 0.0:
            # A sink that stayed empty at crew_count=1 stays empty at 4 (0 * 4 == 0).
            assert four_amt == 0.0
        else:
            # Reconstruct the 4× factor to floating tolerance.
            assert math.isclose(four_amt, 4.0 * one_amt, rel_tol=1e-12)


def test_template_crew_count_four_is_not_the_golden() -> None:
    # The knob genuinely moves the trajectory (a non-scaling knob could not).
    assert _final_bytes(TEMPLATE_YAML, {"crew_count": 4.0}) != GOLDEN_PATH.read_bytes()


def test_template_crew_count_four_conserves_every_step() -> None:
    states, _, _ = _final(TEMPLATE_YAML, {"crew_count": 4.0})
    for before, after in zip(states, states[1:], strict=False):
        ledger = {ql.quantity: ql for ql in compute_ledger(before, after)}
        for quantity in (Quantity.CARBON, Quantity.WATER, Quantity.OXYGEN):
            assert abs(ledger[quantity].residual) <= 1e-6


# --- 3. Referential integrity (build-time) -------------------------------------------


def test_override_of_undeclared_parameter_raises() -> None:
    with pytest.raises(AuthoringError, match="undeclared parameter 'crew_size'"):
        load_scenario(str(TEMPLATE_YAML), overrides={"crew_size": 4.0})


def test_resolve_parameters_merges_defaults_and_overrides() -> None:
    assert resolve_parameters({"a": 1.0, "b": 2.0}, {"b": 9.0}) == {"a": 1.0, "b": 9.0}
    # An integer override is coerced to float.
    assert resolve_parameters({"a": 1.0}, {"a": 4}) == {"a": 4.0}


def test_eval_numeric_field_passes_literals_through() -> None:
    assert eval_numeric_field(1000.0, {}, where="x") == 1000.0
    assert eval_numeric_field(4000, {}, where="x") == 4000.0


def test_eval_numeric_field_evaluates_param_expression() -> None:
    # left-before-right * mirrors the engine VM; 4.0 * 1000.0 is exact.
    assert (
        eval_numeric_field(
            "param('crew_count') * 1000.0", {"crew_count": 4.0}, where="x"
        )
        == 4000.0
    )


def test_eval_numeric_field_undeclared_param_raises() -> None:
    with pytest.raises(AuthoringError, match="undeclared parameter 'missing'"):
        eval_numeric_field("param('missing') * 2.0", {"crew_count": 1.0}, where="x")


@pytest.mark.parametrize("expr", ["stock('crew.food_store')", "forcing('x')", "n"])
def test_eval_numeric_field_rejects_runtime_refs(expr: str) -> None:
    # No State/env/n exists at build time — a runtime ref is an AuthoringError.
    with pytest.raises(AuthoringError, match="not available at build time"):
        eval_numeric_field(expr, {"crew_count": 1.0}, where="x")


def test_interpret_override_undeclared_is_rejected_via_spec() -> None:
    # The same guard at the interpret() entry point (no file), for a bare spec.
    spec = ScenarioSpec(
        name="empty",
        integrator="euler",
        dt=1.0,
        steps=0,
        stocks=[],
        flows=[],
    )
    with pytest.raises(AuthoringError, match="undeclared parameter 'q'"):
        interpret(spec, overrides={"q": 1.0})
