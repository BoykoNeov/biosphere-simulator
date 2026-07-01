"""Phase-5 Step 7 tests: the standalone Crew flows (the net-consumer sibling).

Step 7 adds the Crew domain's three **forced** metabolic flows — the crew side of the
seam ECLSS's forced ``CrewMetabolism`` stood in for. Each flow balances **per quantity**
(``assert_flow_balanced`` groups legs by each stock's composition and asserts every
asserted quantity independently); two of the three (``FoodMetabolism`` /
``WaterBalance``) are **fractional splits** — the ``SolarCharge`` ``charge_split`` idiom
applied to a *mass* quantity, splitting one intake across two named output fates:

* **OxygenConsumption** — forced O₂ draw ``crew.o2_store → crew_o2_consumed`` (2 legs).
* **FoodMetabolism** — forced food-carbon draw split ``food_store → exhaled_co2 (f_resp)
  + fecal_waste (1−f_resp)`` (3 legs, CARBON).
* **WaterBalance** — forced water draw split ``water_store → crew_humidity (f_ins) +
  urine (1−f_ins)`` (3 legs, WATER).

Three layers, mirroring ``test_eclss_flows`` / ``test_power_flows`` (the established
per-flow discipline):

* **Rate laws** — ``carbon_split`` / ``water_split`` (the fraction split summing to the
  input; endpoints collapse one leg; zero input a no-op).
* **Flow level** — each flow's legs balance the one quantity it touches (and touch no
  other), are dt-linear (``flux = rate·dt``), and are a no-op on zero intake.
* **Loader** — the committed Crew params, with out-of-range and bad-unit rejection.

No scenario / golden / conservation-gate run here — those are the run test / golden (the
``assert_flow_balanced`` per-flow check is the Step-level gate). Pure-stdlib spine.
"""

import math
from pathlib import Path

import pytest

from domains.crew.flows import (
    CrewParams,
    FoodMetabolism,
    OxygenConsumption,
    WaterBalance,
    carbon_split,
    water_split,
)
from domains.crew.loader import load_crew_params
from domains.crew.stocks import (
    CREW_HUMIDITY,
    CREW_O2_CONSUMED,
    EXHALED_CO2,
    FECAL_WASTE,
    FOOD_INTAKE_VAR,
    FOOD_STORE,
    O2_INTAKE_VAR,
    O2_STORE,
    URINE,
    WATER_INTAKE_VAR,
    WATER_STORE,
    food_store_stock,
    o2_store_stock,
    water_store_stock,
)
from simcore import boundary
from simcore.environment import BoundEnvironment, SourceResolver, constant
from simcore.flow import assert_flow_balanced, per_quantity_residual
from simcore.ids import FlowId
from simcore.quantities import Quantity
from simcore.state import State

# A hand CrewParams for the pure rate-law checks (matches crew.yaml so the hand values
# stay meaningful; the loader itself is exercised in the loader tests below).
_HAND = CrewParams(respired_carbon_fraction=0.85, insensible_water_fraction=0.4)


# --- shared fixtures ---------------------------------------------------------
def _state(*, food: float = 1000.0, water: float = 60.0, o2: float = 2000.0) -> State:
    """A State with all eight Crew stocks (three stores + five output sinks)."""
    stocks = {
        s.id: s
        for s in (
            food_store_stock(food),
            water_store_stock(water),
            o2_store_stock(o2),
            boundary.sink(EXHALED_CO2, Quantity.CARBON, 0.0),
            boundary.sink(FECAL_WASTE, Quantity.CARBON, 0.0),
            boundary.sink(CREW_HUMIDITY, Quantity.WATER, 0.0),
            boundary.sink(URINE, Quantity.WATER, 0.0),
            boundary.sink(CREW_O2_CONSUMED, Quantity.OXYGEN, 0.0),
        )
    }
    return State(n=0, stocks=stocks, rng_seed=0)


def _env(
    state: State,
    dt: float,
    *,
    o2: float = 1.0e-3,
    food: float = 5.0e-4,
    water: float = 3.0e-5,
) -> BoundEnvironment:
    """Bind the three constant crew-intake forcing schedules to ``state`` + ``dt``."""
    return SourceResolver(
        forcings={
            O2_INTAKE_VAR: constant(o2),
            FOOD_INTAKE_VAR: constant(food),
            WATER_INTAKE_VAR: constant(water),
        }
    ).bind(state, dt)


def _oxygen() -> OxygenConsumption:
    return OxygenConsumption(
        FlowId("crew.oxygen_consumption"),
        0,
        o2_store=O2_STORE,
        o2_consumed=CREW_O2_CONSUMED,
    )


def _food(params: CrewParams = _HAND) -> FoodMetabolism:
    return FoodMetabolism(
        FlowId("crew.food_metabolism"),
        0,
        food_store=FOOD_STORE,
        exhaled_co2=EXHALED_CO2,
        fecal_waste=FECAL_WASTE,
        params=params,
    )


def _water(params: CrewParams = _HAND) -> WaterBalance:
    return WaterBalance(
        FlowId("crew.water_balance"),
        0,
        water_store=WATER_STORE,
        crew_humidity=CREW_HUMIDITY,
        urine=URINE,
        params=params,
    )


# --- rate law: carbon_split / water_split (the fractional split) --------------
def test_carbon_split_partitions_the_intake() -> None:
    # respired = f·food, feces = (1−f)·food, summing to the input.
    respired, feces = carbon_split(10.0, respired_carbon_fraction=0.85)
    assert math.isclose(respired, 8.5, rel_tol=1e-12)
    assert math.isclose(feces, 1.5, rel_tol=1e-12)
    assert math.isclose(respired + feces, 10.0, rel_tol=1e-12)


def test_water_split_partitions_the_intake() -> None:
    humidity, urine = water_split(10.0, insensible_water_fraction=0.4)
    assert math.isclose(humidity, 4.0, rel_tol=1e-12)
    assert math.isclose(urine, 6.0, rel_tol=1e-12)
    assert math.isclose(humidity + urine, 10.0, rel_tol=1e-12)


def test_split_endpoint_collapses_one_leg() -> None:
    # f = 1 collapses the second leg to exactly 0 (valid degenerate split — the
    # SolarCharge η_c = 1 convention); f = 0 collapses the first.
    assert carbon_split(10.0, respired_carbon_fraction=1.0) == (10.0, 0.0)
    assert water_split(10.0, insensible_water_fraction=0.0) == (0.0, 10.0)


def test_split_zero_input_is_zero() -> None:
    assert carbon_split(0.0, respired_carbon_fraction=0.85) == (0.0, 0.0)
    assert water_split(0.0, insensible_water_fraction=0.4) == (0.0, 0.0)


# --- flow level: OxygenConsumption (forced, 2-leg) ---------------------------
def test_oxygen_consumption_balances_oxygen_only() -> None:
    state = _state()
    dt = 3600.0
    result = _oxygen().evaluate(state, _env(state, dt), dt)
    assert_flow_balanced(result, state.stocks)
    assert set(per_quantity_residual(result, state.stocks)) == {Quantity.OXYGEN}


def test_oxygen_consumption_direction_and_dt_linearity() -> None:
    # O₂ leaves the store into the consumed sink; magnitude rate·dt (forced, dt-linear ⇒
    # RK4-order-safe).
    state = _state()
    dt = 3600.0
    legs = {
        leg.stock: leg.amount
        for leg in _oxygen().evaluate(state, _env(state, dt, o2=1.0e-3), dt).legs
    }
    assert legs[O2_STORE] == pytest.approx(-1.0e-3 * dt)  # drawn from the store
    assert legs[CREW_O2_CONSUMED] == pytest.approx(1.0e-3 * dt)  # into the sink


def test_oxygen_consumption_zero_intake_is_a_noop() -> None:
    state = _state()
    dt = 3600.0
    result = _oxygen().evaluate(state, _env(state, dt, o2=0.0), dt)
    assert all(leg.amount == 0.0 for leg in result.legs)


# --- flow level: FoodMetabolism (forced, 3-leg split) ------------------------
def test_food_metabolism_balances_carbon_only() -> None:
    state = _state()
    dt = 3600.0
    result = _food().evaluate(state, _env(state, dt), dt)
    assert_flow_balanced(result, state.stocks)
    assert set(per_quantity_residual(result, state.stocks)) == {Quantity.CARBON}


def test_food_metabolism_splits_intake_and_dt_linear() -> None:
    # The forced food intake q = rate·dt splits into respired CO₂ (f_resp) and fecal
    # waste (1−f_resp), routing to different Phase-6 destinations.
    state = _state()
    dt = 3600.0
    q = 5.0e-4 * dt
    legs = {
        leg.stock: leg.amount
        for leg in _food().evaluate(state, _env(state, dt, food=5.0e-4), dt).legs
    }
    assert legs[FOOD_STORE] == pytest.approx(-q)  # drawn from the store
    assert legs[EXHALED_CO2] == pytest.approx(0.85 * q)  # respired CO₂
    assert legs[FECAL_WASTE] == pytest.approx(0.15 * q)  # egested feces


def test_food_metabolism_always_three_legs() -> None:
    # Three legs even when f_resp = 1 collapses the feces leg to 0 (the SolarCharge
    # "structural legs even at zero amount" convention).
    state = _state()
    dt = 3600.0
    params = CrewParams(respired_carbon_fraction=1.0, insensible_water_fraction=0.4)
    result = _food(params).evaluate(state, _env(state, dt, food=5.0e-4), dt)
    assert len(result.legs) == 3
    legs = {leg.stock: leg.amount for leg in result.legs}
    assert legs[FECAL_WASTE] == 0.0


def test_food_metabolism_zero_intake_is_a_noop() -> None:
    state = _state()
    dt = 3600.0
    result = _food().evaluate(state, _env(state, dt, food=0.0), dt)
    assert all(leg.amount == 0.0 for leg in result.legs)


# --- flow level: WaterBalance (forced, 3-leg split) --------------------------
def test_water_balance_balances_water_only() -> None:
    state = _state()
    dt = 3600.0
    result = _water().evaluate(state, _env(state, dt), dt)
    assert_flow_balanced(result, state.stocks)
    assert set(per_quantity_residual(result, state.stocks)) == {Quantity.WATER}


def test_water_balance_splits_intake_and_dt_linear() -> None:
    state = _state()
    dt = 3600.0
    q = 3.0e-5 * dt
    legs = {
        leg.stock: leg.amount
        for leg in _water().evaluate(state, _env(state, dt, water=3.0e-5), dt).legs
    }
    assert legs[WATER_STORE] == pytest.approx(-q)  # drawn from the store
    assert legs[CREW_HUMIDITY] == pytest.approx(0.4 * q)  # insensible humidity
    assert legs[URINE] == pytest.approx(0.6 * q)  # urine


def test_water_balance_zero_intake_is_a_noop() -> None:
    state = _state()
    dt = 3600.0
    result = _water().evaluate(state, _env(state, dt, water=0.0), dt)
    assert all(leg.amount == 0.0 for leg in result.legs)


# --- loader: the committed crew.yaml -----------------------------------------
def test_loader_reads_committed_params() -> None:
    p = load_crew_params()
    assert p.respired_carbon_fraction == pytest.approx(0.85)
    assert p.insensible_water_fraction == pytest.approx(0.4)


def _write(tmp_path: Path, body: str) -> Path:
    p = tmp_path / "crew.yaml"
    p.write_text(body, encoding="utf-8")
    return p


_GOOD = """
name: crew
process: metabolic_split
parameters:
  respired_carbon_fraction: {{value: {f1}, unit: "{u1}", source: "test"}}
  insensible_water_fraction: {{value: {f2}, unit: "dimensionless", source: "test"}}
"""


def test_loader_rejects_out_of_range_fraction(tmp_path: Path) -> None:
    path = _write(tmp_path, _GOOD.format(f1=1.5, u1="dimensionless", f2=0.4))
    with pytest.raises(ValueError, match="respired_carbon_fraction must be in"):
        load_crew_params(path)


def test_loader_rejects_negative_fraction(tmp_path: Path) -> None:
    # insensible_water_fraction below 0 — the [0, 1] bound rejects it.
    path = _write(tmp_path, _GOOD.format(f1=0.85, u1="dimensionless", f2=-0.1))
    with pytest.raises(ValueError, match="insensible_water_fraction must be in"):
        load_crew_params(path)


def test_loader_rejects_bad_unit(tmp_path: Path) -> None:
    # respired_carbon_fraction declared in the wrong unit — exact-string guard fires.
    path = _write(tmp_path, _GOOD.format(f1=0.85, u1="1", f2=0.4))
    with pytest.raises(
        ValueError, match="respired_carbon_fraction must be declared in 'dimensionless'"
    ):
        load_crew_params(path)
