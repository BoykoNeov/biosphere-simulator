"""Phase-5 Step 6 tests: the standalone ECLSS flows (the first multi-quantity sibling).

Step 6 adds the ECLSS domain's four cabin-air flows — the first sibling touching
**three** conserved quantities (CARBON / OXYGEN / WATER) at once. Each flow balances
**per quantity** (``assert_flow_balanced`` groups legs by each stock's composition and
asserts every asserted quantity independently), which is what lets one flow
(``CrewMetabolism``) touch all three:

* **CrewMetabolism** — the forced multi-quantity crew/Phase-6 seam (six legs across
  three quantities, each balanced).
* **CO2Scrubber** / **Condenser** — first-order donor-controlled removals (the
  ``SelfDischarge`` idiom), each self-limiting to 0 at its stock's floor.
* **O2Makeup** — demand-controlled toward the setpoint (the restoring force that gives
  O₂ an attractor without a readout).

Three layers, mirroring ``test_thermal_flows`` / ``test_power_flows`` (the established
per-flow discipline):

* **Rate laws** — ``scrub_flux`` / ``condense_flux`` (first-order, 0 at floor) and
  ``makeup_flux`` (proportional to the setpoint deficit, 0 at the setpoint).
* **Flow level** — each flow's legs balance the quantities it touches (and touch no
  other), are dt-linear (``flux = rate·dt``), and self-limit on zero input / at the
  floor.
* **Loader** — the committed ECLSS params, with out-of-range and bad-unit rejection.

No scenario / golden / conservation-gate run here — those are the run test / golden (the
``assert_flow_balanced`` per-flow check is the Step-level gate). Pure-stdlib spine.
"""

import math
from pathlib import Path

import pytest

from domains.eclss.flows import (
    CO2Scrubber,
    Condenser,
    CrewMetabolism,
    EclssParams,
    O2Makeup,
    condense_flux,
    makeup_flux,
    scrub_flux,
)
from domains.eclss.loader import load_eclss_params
from domains.eclss.stocks import (
    CABIN_CO2,
    CABIN_H2O,
    CABIN_O2,
    CO2_PRODUCTION_VAR,
    CO2_REMOVED,
    H2O_PRODUCTION_VAR,
    HUMIDITY_CONDENSATE,
    METABOLIC_CO2_SOURCE,
    METABOLIC_H2O_SOURCE,
    METABOLIC_O2_SINK,
    O2_CONSUMPTION_VAR,
    O2_SUPPLY,
    cabin_co2_stock,
    cabin_h2o_stock,
    cabin_o2_stock,
)
from simcore import boundary
from simcore.environment import BoundEnvironment, SourceResolver, constant
from simcore.flow import assert_flow_balanced, per_quantity_residual
from simcore.ids import FlowId
from simcore.quantities import Quantity
from simcore.state import State

# A hand EclssParams for the pure rate-law checks (matches eclss.yaml so the hand values
# stay meaningful; the loader itself is exercised in the loader tests below).
_HAND = EclssParams(
    co2_scrub_rate=1.0e-3,
    condense_rate=5.0e-4,
    o2_makeup_gain=2.0e-3,
    o2_setpoint=10.0,
)


# --- shared fixtures ---------------------------------------------------------
def _state(*, o2: float = 10.0, co2: float = 3.0, h2o: float = 0.04) -> State:
    """A State with all nine ECLSS stocks (three cabin POOLs + six boundaries)."""
    stocks = {
        s.id: s
        for s in (
            cabin_o2_stock(o2),
            cabin_co2_stock(co2),
            cabin_h2o_stock(h2o),
            boundary.source(O2_SUPPLY, Quantity.OXYGEN, 0.0),
            boundary.sink(CO2_REMOVED, Quantity.CARBON, 0.0),
            boundary.sink(HUMIDITY_CONDENSATE, Quantity.WATER, 0.0),
            boundary.sink(METABOLIC_O2_SINK, Quantity.OXYGEN, 0.0),
            boundary.source(METABOLIC_CO2_SOURCE, Quantity.CARBON, 0.0),
            boundary.source(METABOLIC_H2O_SOURCE, Quantity.WATER, 0.0),
        )
    }
    return State(n=0, stocks=stocks, rng_seed=0)


def _crew_env(
    state: State,
    dt: float,
    *,
    o2: float = 0.004,
    co2: float = 0.003,
    h2o: float = 2.0e-5,
) -> BoundEnvironment:
    """Bind the three constant crew-rate forcing schedules to ``state`` + ``dt``."""
    return SourceResolver(
        forcings={
            O2_CONSUMPTION_VAR: constant(o2),
            CO2_PRODUCTION_VAR: constant(co2),
            H2O_PRODUCTION_VAR: constant(h2o),
        }
    ).bind(state, dt)


def _crew() -> CrewMetabolism:
    return CrewMetabolism(
        FlowId("eclss.crew_metabolism"),
        0,
        cabin_o2=CABIN_O2,
        cabin_co2=CABIN_CO2,
        cabin_h2o=CABIN_H2O,
        metabolic_o2_sink=METABOLIC_O2_SINK,
        metabolic_co2_source=METABOLIC_CO2_SOURCE,
        metabolic_h2o_source=METABOLIC_H2O_SOURCE,
    )


def _scrubber(params: EclssParams = _HAND) -> CO2Scrubber:
    return CO2Scrubber(
        FlowId("eclss.co2_scrubber"),
        0,
        cabin_co2=CABIN_CO2,
        co2_removed=CO2_REMOVED,
        params=params,
    )


def _condenser(params: EclssParams = _HAND) -> Condenser:
    return Condenser(
        FlowId("eclss.condenser"),
        0,
        cabin_h2o=CABIN_H2O,
        humidity_condensate=HUMIDITY_CONDENSATE,
        params=params,
    )


def _makeup(params: EclssParams = _HAND) -> O2Makeup:
    return O2Makeup(
        FlowId("eclss.o2_makeup"),
        0,
        o2_supply=O2_SUPPLY,
        cabin_o2=CABIN_O2,
        params=params,
    )


# --- rate law: scrub_flux / condense_flux (first-order donor-controlled) ------
def test_scrub_flux_is_first_order() -> None:
    # R = k_scrub · cabin_co2 — a hand value.
    assert math.isclose(scrub_flux(3.0, co2_scrub_rate=1.0e-3), 3.0e-3, rel_tol=1e-12)


def test_scrub_flux_zero_at_floor() -> None:
    # At cabin_co2 = 0 the scrubber removes exactly 0 (structural positivity at the
    # floor).
    assert scrub_flux(0.0, co2_scrub_rate=1.0e-3) == 0.0


def test_condense_flux_is_first_order() -> None:
    assert math.isclose(
        condense_flux(0.04, condense_rate=5.0e-4), 2.0e-5, rel_tol=1e-12
    )


def test_condense_flux_zero_at_floor() -> None:
    assert condense_flux(0.0, condense_rate=5.0e-4) == 0.0


# --- rate law: makeup_flux (demand-controlled toward the setpoint) ------------
def test_makeup_flux_is_proportional_to_the_deficit() -> None:
    # S = k_makeup · (o2_setpoint − cabin_o2): at 8 mol with setpoint 10, deficit 2.
    assert math.isclose(
        makeup_flux(8.0, o2_makeup_gain=2.0e-3, o2_setpoint=10.0), 4.0e-3, rel_tol=1e-12
    )


def test_makeup_flux_zero_at_setpoint() -> None:
    # At the setpoint the regulator is idle (no restoring push needed).
    assert makeup_flux(10.0, o2_makeup_gain=2.0e-3, o2_setpoint=10.0) == 0.0


# --- flow level: CrewMetabolism (multi-quantity, forced) ---------------------
def test_crew_metabolism_balances_all_three_quantities() -> None:
    # The multi-quantity payload: one flow, six legs, each of CARBON / OXYGEN /
    # WATER balances independently (assert raises on any imbalance).
    state = _state()
    dt = 60.0
    result = _crew().evaluate(state, _crew_env(state, dt), dt)
    assert_flow_balanced(result, state.stocks)
    residual = per_quantity_residual(result, state.stocks)
    for q in (Quantity.CARBON, Quantity.OXYGEN, Quantity.WATER):
        assert abs(residual[q]) < 1e-15


def test_crew_metabolism_touches_exactly_the_three_mass_quantities() -> None:
    state = _state()
    dt = 60.0
    result = _crew().evaluate(state, _crew_env(state, dt), dt)
    assert set(per_quantity_residual(result, state.stocks)) == {
        Quantity.CARBON,
        Quantity.OXYGEN,
        Quantity.WATER,
    }


def test_crew_metabolism_directions_and_dt_linearity() -> None:
    # O₂ leaves the cabin (consumed); CO₂ and H₂O enter it (produced). Each leg
    # magnitude is rate·dt (forced, dt-linear ⇒ RK4-order-safe).
    state = _state()
    dt = 60.0
    legs = {
        leg.stock: leg.amount
        for leg in _crew()
        .evaluate(state, _crew_env(state, dt, o2=0.004, co2=0.003, h2o=2.0e-5), dt)
        .legs
    }
    assert legs[CABIN_O2] == pytest.approx(-0.004 * dt)  # O₂ consumed
    assert legs[METABOLIC_O2_SINK] == pytest.approx(0.004 * dt)
    assert legs[CABIN_CO2] == pytest.approx(0.003 * dt)  # CO₂ produced
    assert legs[METABOLIC_CO2_SOURCE] == pytest.approx(-0.003 * dt)
    assert legs[CABIN_H2O] == pytest.approx(2.0e-5 * dt)  # H₂O produced
    assert legs[METABOLIC_H2O_SOURCE] == pytest.approx(-2.0e-5 * dt)


def test_crew_metabolism_zero_load_is_a_noop() -> None:
    state = _state()
    dt = 60.0
    result = _crew().evaluate(state, _crew_env(state, dt, o2=0.0, co2=0.0, h2o=0.0), dt)
    assert all(leg.amount == 0.0 for leg in result.legs)


# --- flow level: CO2Scrubber ------------------------------------------------
def test_co2_scrubber_balances_carbon_only() -> None:
    state = _state(co2=3.0)
    dt = 60.0
    result = _scrubber().evaluate(state, _crew_env(state, dt), dt)
    assert_flow_balanced(result, state.stocks)
    assert set(per_quantity_residual(result, state.stocks)) == {Quantity.CARBON}


def test_co2_scrubber_removes_first_order_and_dt_linear() -> None:
    state = _state(co2=3.0)
    dt = 60.0
    legs = {
        leg.stock: leg.amount
        for leg in _scrubber().evaluate(state, _crew_env(state, dt), dt).legs
    }
    expected = _HAND.co2_scrub_rate * 3.0 * dt
    assert legs[CABIN_CO2] == pytest.approx(-expected)
    assert legs[CO2_REMOVED] == pytest.approx(expected)


def test_co2_scrubber_self_limits_at_floor() -> None:
    state = _state(co2=0.0)
    dt = 60.0
    result = _scrubber().evaluate(state, _crew_env(state, dt), dt)
    assert all(leg.amount == 0.0 for leg in result.legs)


# --- flow level: Condenser --------------------------------------------------
def test_condenser_balances_water_only() -> None:
    state = _state(h2o=0.04)
    dt = 60.0
    result = _condenser().evaluate(state, _crew_env(state, dt), dt)
    assert_flow_balanced(result, state.stocks)
    assert set(per_quantity_residual(result, state.stocks)) == {Quantity.WATER}


def test_condenser_removes_first_order_and_dt_linear() -> None:
    state = _state(h2o=0.04)
    dt = 60.0
    legs = {
        leg.stock: leg.amount
        for leg in _condenser().evaluate(state, _crew_env(state, dt), dt).legs
    }
    expected = _HAND.condense_rate * 0.04 * dt
    assert legs[CABIN_H2O] == pytest.approx(-expected)
    assert legs[HUMIDITY_CONDENSATE] == pytest.approx(expected)


def test_condenser_self_limits_at_floor() -> None:
    state = _state(h2o=0.0)
    dt = 60.0
    result = _condenser().evaluate(state, _crew_env(state, dt), dt)
    assert all(leg.amount == 0.0 for leg in result.legs)


# --- flow level: O2Makeup ---------------------------------------------------
def test_o2_makeup_balances_oxygen_only() -> None:
    state = _state(o2=8.0)
    dt = 60.0
    result = _makeup().evaluate(state, _crew_env(state, dt), dt)
    assert_flow_balanced(result, state.stocks)
    assert set(per_quantity_residual(result, state.stocks)) == {Quantity.OXYGEN}


def test_o2_makeup_adds_toward_setpoint_and_dt_linear() -> None:
    state = _state(o2=8.0)  # 2 mol below the 10 mol setpoint
    dt = 60.0
    legs = {
        leg.stock: leg.amount
        for leg in _makeup().evaluate(state, _crew_env(state, dt), dt).legs
    }
    expected = _HAND.o2_makeup_gain * (10.0 - 8.0) * dt
    assert legs[O2_SUPPLY] == pytest.approx(-expected)  # drawn from the tank
    assert legs[CABIN_O2] == pytest.approx(expected)  # added to the cabin
    assert expected > 0.0


def test_o2_makeup_idle_at_setpoint() -> None:
    state = _state(o2=10.0)  # at the setpoint
    dt = 60.0
    result = _makeup().evaluate(state, _crew_env(state, dt), dt)
    assert all(leg.amount == 0.0 for leg in result.legs)


# --- loader: the committed eclss.yaml ---------------------------------------
def test_loader_reads_committed_params() -> None:
    p = load_eclss_params()
    assert p.co2_scrub_rate == pytest.approx(1.0e-3)
    assert p.condense_rate == pytest.approx(5.0e-4)
    assert p.o2_makeup_gain == pytest.approx(2.0e-3)
    assert p.o2_setpoint == pytest.approx(10.0)


def _write(tmp_path: Path, body: str) -> Path:
    p = tmp_path / "eclss.yaml"
    p.write_text(body, encoding="utf-8")
    return p


_GOOD = """
name: eclss
process: cabin_air_control
parameters:
  co2_scrub_rate: {{value: {k1}, unit: "1/s", source: "test"}}
  condense_rate: {{value: {k2}, unit: "{u2}", source: "test"}}
  o2_makeup_gain: {{value: {k3}, unit: "1/s", source: "test"}}
  o2_setpoint: {{value: {sp}, unit: "mol", source: "test"}}
"""


def test_loader_rejects_nonpositive_rate(tmp_path: Path) -> None:
    path = _write(
        tmp_path, _GOOD.format(k1=0.0, k2=5.0e-4, u2="1/s", k3=2.0e-3, sp=10.0)
    )
    with pytest.raises(ValueError, match="co2_scrub_rate must be > 0"):
        load_eclss_params(path)


def test_loader_rejects_nonpositive_setpoint(tmp_path: Path) -> None:
    path = _write(
        tmp_path, _GOOD.format(k1=1.0e-3, k2=5.0e-4, u2="1/s", k3=2.0e-3, sp=-1.0)
    )
    with pytest.raises(ValueError, match="o2_setpoint must be > 0"):
        load_eclss_params(path)


def test_loader_rejects_bad_unit(tmp_path: Path) -> None:
    # condense_rate declared in the wrong unit — exact-string guard fires.
    path = _write(
        tmp_path, _GOOD.format(k1=1.0e-3, k2=5.0e-4, u2="1/min", k3=2.0e-3, sp=10.0)
    )
    with pytest.raises(ValueError, match="condense_rate must be declared in '1/s'"):
        load_eclss_params(path)
