"""Phase-5 Step 5 tests: the standalone Thermal flows (ENERGY only; energy closure).

Step 5 adds the Thermal domain's two energy-balanced flows — the second carrier of the
Phase-5 energy-closure decision (P5.1: ENERGY joined the asserted conserved set). Both
balance ENERGY (``Σ legs == 0``); the node → space rejection is the physically correct
vacuum mode (Stefan-Boltzmann ``T⁴``):

* **HeatInput** — ``heat_source → node``, the forced heat supply (heat → heat, no form
  change ⇒ **two legs**, no loss — unlike Power's 3-leg lossy ``SolarCharge``).
* **RadiatorReject** — ``node → boundary.space``, the donor-controlled nonlinear
  radiator ``ε·σ·A·(T⁴ − T_space⁴)·dt`` with ``T = T_space + node/C``.

Three layers, mirroring ``test_power_flows`` (the established per-flow discipline):

* **Rate laws** — ``temperature`` (``T_space + Q/C``, floor at ``Q = 0``) and
  ``radiated_power`` (the ``T⁴`` law: 0 at floor, monotone-increasing, a hand value).
* **Flow level** — each flow's legs balance ENERGY and touch no other quantity, are
  dt-linear (``flux = rate·dt``), self-limit on zero input / at the floor.
* **Loader** — the committed radiator params, with out-of-range and bad-unit rejection.

No scenario / golden / conservation-gate run here — those are the run test / golden (the
``assert_flow_balanced`` per-flow check is the Step-level gate). Pure-stdlib spine.
"""

import math
from pathlib import Path

import pytest

from domains.thermal.flows import (
    STEFAN_BOLTZMANN,
    HeatInput,
    RadiatorReject,
    ThermalParams,
    radiated_power,
    temperature,
)
from domains.thermal.loader import load_thermal_params
from domains.thermal.stocks import (
    HEAT_LOAD_VAR,
    HEAT_SOURCE,
    NODE,
    SPACE,
    node_stock,
)
from simcore import boundary
from simcore.environment import BoundEnvironment, SourceResolver, constant
from simcore.flow import assert_flow_balanced, per_quantity_residual
from simcore.ids import FlowId
from simcore.quantities import Quantity
from simcore.state import State

# A committed-params handle + a hand ThermalParams for the pure rate-law checks.
_PARAMS = load_thermal_params()
_HAND = ThermalParams(
    emissivity=0.85, radiator_area=10.0, heat_capacity=1.0e7, space_temperature=2.7
)


# --- shared fixtures ---------------------------------------------------------
def _state(*, node: float = 0.0) -> State:
    """A State with the three Thermal stocks (node POOL + the two boundaries)."""
    stocks = {
        s.id: s
        for s in (
            node_stock(node),
            boundary.source(HEAT_SOURCE, Quantity.ENERGY, 0.0),
            boundary.sink(SPACE, Quantity.ENERGY, 0.0),
        )
    }
    return State(n=0, stocks=stocks, rng_seed=0)


def _env(state: State, dt: float, *, heat_load: float = 0.0) -> BoundEnvironment:
    """Bind a constant heat-load forcing schedule to ``state`` + ``dt`` (#16)."""
    return SourceResolver(forcings={HEAT_LOAD_VAR: constant(heat_load)}).bind(state, dt)


def _heat_input() -> HeatInput:
    return HeatInput(
        FlowId("thermal.heat_input"), 0, heat_source=HEAT_SOURCE, node=NODE
    )


def _radiator(params: ThermalParams = _HAND) -> RadiatorReject:
    return RadiatorReject(
        FlowId("thermal.radiator_reject"), 0, node=NODE, space=SPACE, params=params
    )


# --- rate law: temperature ---------------------------------------------------
def test_temperature_is_reference_plus_heat_over_capacity() -> None:
    # T = T_space + Q/C — a hand value.
    t = temperature(1.0e7, heat_capacity=1.0e7, space_temperature=2.7)
    assert math.isclose(t, 3.7, rel_tol=1e-12)  # 2.7 + 1e7/1e7 = 3.7 K


def test_temperature_at_floor_is_space_temperature() -> None:
    # Q = 0 ⇒ T = T_space (the radiator floor, where rejection is exactly 0).
    assert temperature(0.0, heat_capacity=1.0e7, space_temperature=2.7) == 2.7


# --- rate law: radiated_power (the Stefan-Boltzmann T^4 law) ------------------
def test_radiated_power_zero_at_floor() -> None:
    # At Q = 0 (T = T_space) the driving term T^4 − T_space^4 is exactly 0 — structural
    # positivity at the floor (the radiator cannot pull the node negative).
    assert radiated_power(0.0, params=_HAND) == 0.0


def test_radiated_power_matches_stefan_boltzmann() -> None:
    # A hand value: at Q so that T = 300 K, R = εσA(300^4 − T_space^4).
    q = _HAND.heat_capacity * (300.0 - _HAND.space_temperature)  # C·(T − T_space)
    expected = (
        _HAND.emissivity
        * STEFAN_BOLTZMANN
        * _HAND.radiator_area
        * (300.0**4 - _HAND.space_temperature**4)
    )
    assert math.isclose(radiated_power(q, params=_HAND), expected, rel_tol=1e-12)


def test_radiated_power_is_monotone_increasing_in_heat() -> None:
    # More stored heat ⇒ hotter ⇒ more radiated (the restoring force: it rises with T).
    r_lo = radiated_power(1.0e9, params=_HAND)
    r_hi = radiated_power(2.0e9, params=_HAND)
    assert 0.0 < r_lo < r_hi


def test_stefan_boltzmann_constant_is_codata_value() -> None:
    # σ is the CODATA 2018 exact value (fixed since the 2019 SI redefinition), pinned so
    # a typo in the physics constant is caught (it feeds every radiated-power
    # computation).
    assert STEFAN_BOLTZMANN == 5.670374419e-8


# --- HeatInput ---------------------------------------------------------------
def test_heat_input_moves_supply_into_node() -> None:
    # Heat → heat, no loss: the supply leaves the source (−X) and lands wholly in the
    # node (+X). Two legs, single magnitude.
    state = _state()
    legs = {
        leg.stock: leg.amount
        for leg in _heat_input()
        .evaluate(state, _env(state, 1.0, heat_load=100.0), 1.0)
        .legs
    }
    assert set(legs) == {HEAT_SOURCE, NODE}
    assert math.isclose(legs[HEAT_SOURCE], -100.0, rel_tol=1e-12)
    assert math.isclose(legs[NODE], 100.0, rel_tol=1e-12)


def test_heat_input_balances_energy_only() -> None:
    # Single magnitude in both legs ⇒ ENERGY balances exactly; no other quantity moved.
    state = _state()
    result = _heat_input().evaluate(state, _env(state, 1.0, heat_load=100.0), 1.0)
    assert_flow_balanced(result, state.stocks)
    assert set(per_quantity_residual(result, state.stocks)) == {Quantity.ENERGY}


def test_heat_input_zero_load_is_noop() -> None:
    # No supply ⇒ zero-amount legs (the dt-independent self-limit).
    state = _state()
    legs = _heat_input().evaluate(state, _env(state, 1.0, heat_load=0.0), 1.0).legs
    assert all(leg.amount == 0.0 for leg in legs)


def test_heat_input_is_dt_linear() -> None:
    # flux = rate·dt — the increment-form contract (RK4 order; Phase-6 multi-rate-safe).
    state = _state()
    half = _heat_input().evaluate(state, _env(state, 0.5, heat_load=100.0), 0.5).legs
    full = _heat_input().evaluate(state, _env(state, 1.0, heat_load=100.0), 1.0).legs
    assert math.isclose(full[1].amount, 2.0 * half[1].amount, rel_tol=1e-12)


# --- RadiatorReject ----------------------------------------------------------
def test_radiator_rejects_node_heat_to_space() -> None:
    # Two legs: R = radiated_power(node)·dt leaves the node (−R), lands in space (+R).
    q = _HAND.heat_capacity * (300.0 - _HAND.space_temperature)
    state = _state(node=q)
    legs = {
        leg.stock: leg.amount
        for leg in _radiator().evaluate(state, _env(state, 1.0), 1.0).legs
    }
    assert set(legs) == {NODE, SPACE}
    expected = radiated_power(q, params=_HAND) * 1.0
    assert math.isclose(legs[NODE], -expected, rel_tol=1e-12)  # withdrawn
    assert math.isclose(legs[SPACE], expected, rel_tol=1e-12)  # radiated away


def test_radiator_balances_energy_only() -> None:
    # A single magnitude in both legs ⇒ ENERGY balances exactly; no other quantity hit.
    q = _HAND.heat_capacity * (300.0 - _HAND.space_temperature)
    state = _state(node=q)
    result = _radiator().evaluate(state, _env(state, 1.0), 1.0)
    assert_flow_balanced(result, state.stocks)
    assert set(per_quantity_residual(result, state.stocks)) == {Quantity.ENERGY}


def test_radiator_is_donor_controlled_zero_at_floor() -> None:
    # Reads the node stock: at Q = 0 (T = T_space) the rejection is exactly 0 — the
    # donor-controlled self-limit at the floor (structural positivity there). Two legs
    # still emitted (both zero-amount).
    state = _state(node=0.0)
    legs = _radiator().evaluate(state, _env(state, 1.0), 1.0).legs
    assert len(legs) == 2
    assert all(leg.amount == 0.0 for leg in legs)


def test_radiator_reads_the_snapshot_not_the_env() -> None:
    # The rejection depends on the node's amount (donor-controlled), independent of the
    # heat-load forcing — hotter node radiates more even at zero heat load.
    q = _HAND.heat_capacity * (300.0 - _HAND.space_temperature)
    cold = _radiator().evaluate(_state(node=q / 2.0), _env(_state(), 1.0), 1.0).legs
    hot = _radiator().evaluate(_state(node=q), _env(_state(), 1.0), 1.0).legs
    assert abs(hot[1].amount) > abs(cold[1].amount) > 0.0


def test_radiator_is_dt_linear() -> None:
    # flux = rate·dt on the SAME snapshot (the increment-form contract).
    q = _HAND.heat_capacity * (300.0 - _HAND.space_temperature)
    state = _state(node=q)
    half = _radiator().evaluate(state, _env(state, 0.5), 0.5).legs
    full = _radiator().evaluate(state, _env(state, 1.0), 1.0).legs
    assert math.isclose(full[1].amount, 2.0 * half[1].amount, rel_tol=1e-12)


# --- loader ------------------------------------------------------------------
def test_loader_reads_committed_params() -> None:
    p = load_thermal_params()
    assert p.emissivity == 0.85
    assert p.radiator_area == 10.0
    assert p.heat_capacity == 1.0e7
    assert p.space_temperature == 2.7


def _write_radiator(
    tmp_path: Path,
    *,
    emissivity: tuple[float, str] = (0.85, "dimensionless"),
    radiator_area: tuple[float, str] = (10.0, "m^2"),
    heat_capacity: tuple[float, str] = (1.0e7, "J/K"),
    space_temperature: tuple[float, str] = (2.7, "K"),
) -> Path:
    def block(name: str, vu: tuple[float, str]) -> str:
        return (
            f'  {name}:\n    value: {vu[0]}\n    unit: {vu[1]!r}\n    source: "test"\n'
        )

    bad = tmp_path / "radiator.yaml"
    bad.write_text(
        "name: thermal\nprocess: radiation\nparameters:\n"
        + block("emissivity", emissivity)
        + block("radiator_area", radiator_area)
        + block("heat_capacity", heat_capacity)
        + block("space_temperature", space_temperature),
        encoding="utf-8",
    )
    return bad


def test_loader_rejects_zero_emissivity(tmp_path: Path) -> None:
    # 0 = a surface that radiates nothing (no rejection path); (0, 1] required.
    with pytest.raises(ValueError, match="emissivity must be in"):
        load_thermal_params(
            _write_radiator(tmp_path, emissivity=(0.0, "dimensionless"))
        )


def test_loader_rejects_above_one_emissivity(tmp_path: Path) -> None:
    # > 1 would radiate more than a black body.
    with pytest.raises(ValueError, match="emissivity must be in"):
        load_thermal_params(
            _write_radiator(tmp_path, emissivity=(1.5, "dimensionless"))
        )


def test_loader_rejects_nonpositive_area(tmp_path: Path) -> None:
    with pytest.raises(ValueError, match="radiator_area must be > 0"):
        load_thermal_params(_write_radiator(tmp_path, radiator_area=(0.0, "m^2")))


def test_loader_rejects_nonpositive_heat_capacity(tmp_path: Path) -> None:
    with pytest.raises(ValueError, match="heat_capacity must be > 0"):
        load_thermal_params(_write_radiator(tmp_path, heat_capacity=(0.0, "J/K")))


def test_loader_rejects_negative_space_temperature(tmp_path: Path) -> None:
    with pytest.raises(ValueError, match="space_temperature must be >= 0"):
        load_thermal_params(_write_radiator(tmp_path, space_temperature=(-1.0, "K")))


def test_loader_rejects_bad_unit(tmp_path: Path) -> None:
    # Absolute K is the exact-guarded temperature unit (the T^4 law needs absolute);
    # degC is rejected (unlike the biosphere's degC kinetics).
    with pytest.raises(ValueError, match="must be declared in"):
        load_thermal_params(_write_radiator(tmp_path, space_temperature=(2.7, "degC")))
