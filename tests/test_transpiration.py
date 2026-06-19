"""Phase-1 Step-7 tests: Penman–Monteith transpiration + root uptake (WATER flows).

The first WATER-currency process. Layers (mirroring Steps 5/6):

* **Rate laws** (``domains.biosphere.transpiration``, pure stdlib): the saturation
  vapour pressure and its slope checked against **FAO-56 published table values**
  (independent literals); the Penman–Monteith combination equation at a pinned
  operating point and its dark/dry floor; the soil-water stress factor ``f_water``
  cardinal values + the wilting/critical clamp.
* **The assembled flows**: ``Transpiration`` (water-balanced, dt-linear, self-limiting
  via ``f_water`` on the step-entry soil water) and ``Irrigation`` (water-balanced,
  tracking the irrigation forcing).
* **Config boundary** (``load_transpiration_params``): the committed file loads to the
  expected params; bad units / out-of-range values / a missing source are rejected.
"""

import math
from pathlib import Path
from typing import Any

import pytest
import yaml
from pydantic import ValidationError

from domains.biosphere.loader import (
    TRANSPIRATION_PARAMS_PATH,
    load_transpiration_params,
)
from domains.biosphere.transpiration import (
    Irrigation,
    Transpiration,
    TranspirationParams,
    penman_monteith_transpiration,
    saturation_vapor_pressure,
    slope_svp,
    water_stress_factor,
)
from simcore.environment import SourceResolver, constant
from simcore.flow import assert_flow_balanced
from simcore.ids import DomainId, FlowId, StockId
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import State, Stock

# The committed winter-wheat provisional placeholders (mirror transpiration.yaml).
_RA, _RS = 50.0, 70.0


def _params() -> TranspirationParams:
    return TranspirationParams(aerodynamic_resistance=_RA, surface_resistance=_RS)


# --- saturation vapour pressure + slope: FAO-56 table values ----------------
# Genuine independent literals — the published FAO-56 (Allen et al. 1998) Table values
# of e_s and Δ, NOT a restatement of the module formula. kPa, rel_tol loose enough to
# absorb the table's own rounding.
@pytest.mark.parametrize(
    ("temp", "es_kpa", "slope_kpa"),
    [
        (0.0, 0.6108, 0.04445),
        (10.0, 1.2280, 0.08228),
        (20.0, 2.3383, 0.14474),
        (30.0, 4.2431, 0.24336),
    ],
)
def test_svp_and_slope_match_fao56_table(
    temp: float, es_kpa: float, slope_kpa: float
) -> None:
    assert math.isclose(saturation_vapor_pressure(temp) / 1000.0, es_kpa, rel_tol=2e-3)
    assert math.isclose(slope_svp(temp) / 1000.0, slope_kpa, rel_tol=2e-3)


def test_slope_is_the_analytic_derivative_of_svp() -> None:
    # Δ ≈ d(e_s)/dT by finite difference (the slope is the exact derivative).
    t, h = 18.0, 1e-4
    numeric = (saturation_vapor_pressure(t + h) - saturation_vapor_pressure(t - h)) / (
        2 * h
    )
    assert math.isclose(slope_svp(t), numeric, rel_tol=1e-6)


# --- Penman–Monteith: pinned operating point + the floor --------------------
def test_penman_monteith_pinned_value() -> None:
    # Rn=200 W/m², VPD=1000 Pa, T=20 °C, r_a=50, r_s=70 s/m ⇒ ~6.16 mm/day (a
    # realistic summer potential transpiration). Pinned regression literal.
    tp = penman_monteith_transpiration(
        200.0, 1000.0, 20.0, aerodynamic_resistance=_RA, surface_resistance=_RS
    )
    assert math.isclose(tp, 6.158958394549651, rel_tol=1e-12)


def test_penman_monteith_zero_energy_zero_vpd_is_zero() -> None:
    # No available energy and no vapour deficit ⇒ no evaporative demand.
    tp = penman_monteith_transpiration(
        0.0, 0.0, 20.0, aerodynamic_resistance=_RA, surface_resistance=_RS
    )
    assert tp == 0.0


def test_penman_monteith_rejects_non_positive_aerodynamic_resistance() -> None:
    with pytest.raises(ValueError, match="aerodynamic_resistance"):
        penman_monteith_transpiration(
            200.0, 1000.0, 20.0, aerodynamic_resistance=0.0, surface_resistance=_RS
        )


# --- soil-water stress factor f_water: cardinal values + clamp --------------
@pytest.mark.parametrize(
    ("soil_water", "expected"),
    [
        (5.0, 0.0),  # below wilting
        (10.0, 0.0),  # at wilting
        (15.0, 0.25),  # quarter of the [10, 30] band
        (20.0, 0.5),  # midpoint
        (30.0, 1.0),  # at critical
        (40.0, 1.0),  # above critical
    ],
)
def test_water_stress_factor_cardinal_values(
    soil_water: float, expected: float
) -> None:
    assert math.isclose(
        water_stress_factor(soil_water, sw_wilting=10.0, sw_critical=30.0),
        expected,
        rel_tol=1e-12,
    )


def test_water_stress_factor_rejects_inverted_band() -> None:
    with pytest.raises(ValueError, match="sw_wilting < sw_critical"):
        water_stress_factor(20.0, sw_wilting=30.0, sw_critical=10.0)


# --- the assembled flows ----------------------------------------------------
_BIO = DomainId("biosphere")
_BOUNDARY = DomainId("boundary")
_SOIL_WATER = StockId("biosphere.soil_water")
_VAPOR = StockId("boundary.vapor")
_WATER_SOURCE = StockId("boundary.irrigation_supply")

_SW_WILT, _SW_CRIT = 10.0, 30.0


def _state(soil_water0: float) -> State:
    water = canonical_unit(Quantity.WATER)
    soil = Stock(
        id=_SOIL_WATER,
        domain=_BIO,
        quantity=Quantity.WATER,
        unit=water,
        amount=soil_water0,
        kind=StockKind.POOL,
    )
    vapor = Stock(
        id=_VAPOR,
        domain=_BOUNDARY,
        quantity=Quantity.WATER,
        unit=water,
        amount=0.0,
        kind=StockKind.BOUNDARY,
    )
    supply = Stock(
        id=_WATER_SOURCE,
        domain=_BOUNDARY,
        quantity=Quantity.WATER,
        unit=water,
        amount=1.0e9,
        kind=StockKind.BOUNDARY,
        unclamped=True,
    )
    return State(
        n=0,
        stocks={_SOIL_WATER: soil, _VAPOR: vapor, _WATER_SOURCE: supply},
        rng_seed=0,
    )


def _env(snapshot: State, dt: float, *, rn: float = 200.0, irrigation: float = 5.0):  # noqa: ANN202
    resolver = SourceResolver(
        forcings={
            "rn": constant(rn),
            "vpd": constant(1000.0),
            "temp": constant(20.0),
            "irrigation": constant(irrigation),
        }
    )
    return resolver.bind(snapshot, dt)


def _transpiration_flow(ground_area: float = 1.0) -> Transpiration:
    return Transpiration(
        id=FlowId("biosphere.transpiration"),
        priority=0,
        soil_water=_SOIL_WATER,
        vapor_sink=_VAPOR,
        rn_var="rn",
        vpd_var="vpd",
        temp_var="temp",
        params=_params(),
        ground_area=ground_area,
        sw_wilting=_SW_WILT,
        sw_critical=_SW_CRIT,
    )


def test_transpiration_leg_is_pm_times_fwater_times_area() -> None:
    # soil_water=25 ⇒ f_water=(25−10)/20=0.75; potential=6.1590 mm/day; area=1 m².
    state = _state(soil_water0=25.0)
    result = _transpiration_flow(ground_area=1.0).evaluate(state, _env(state, 1.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    potential = penman_monteith_transpiration(
        200.0, 1000.0, 20.0, aerodynamic_resistance=_RA, surface_resistance=_RS
    )
    expected = potential * 0.75 * 1.0
    assert math.isclose(legs[_SOIL_WATER], -expected, rel_tol=1e-12)
    assert math.isclose(legs[_VAPOR], expected, rel_tol=1e-12)
    # Cross-check against the pinned composed literal (kg/day at this point).
    assert math.isclose(expected, 4.619218795912238, rel_tol=1e-12)


def test_transpiration_is_water_balanced() -> None:
    state = _state(soil_water0=25.0)
    result = _transpiration_flow().evaluate(state, _env(state, 1.0), 1.0)
    assert_flow_balanced(result, state.stocks)


def test_transpiration_shuts_off_at_wilting_point() -> None:
    # soil_water at the wilting point ⇒ f_water=0 ⇒ no transpiration (structural
    # positivity: the pool cannot be drawn below wilting).
    state = _state(soil_water0=_SW_WILT)
    result = _transpiration_flow().evaluate(state, _env(state, 1.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    assert legs[_SOIL_WATER] == 0.0
    assert legs[_VAPOR] == 0.0


def test_transpiration_scales_with_ground_area() -> None:
    state = _state(soil_water0=25.0)
    one = next(
        leg.amount
        for leg in _transpiration_flow(1.0).evaluate(state, _env(state, 1.0), 1.0).legs
        if leg.stock == _VAPOR
    )
    triple = next(
        leg.amount
        for leg in _transpiration_flow(3.0).evaluate(state, _env(state, 1.0), 1.0).legs
        if leg.stock == _VAPOR
    )
    assert math.isclose(triple, 3.0 * one, rel_tol=1e-12)


def test_transpiration_scales_linearly_with_dt() -> None:
    state = _state(soil_water0=25.0)
    flow = _transpiration_flow()
    one = next(
        leg.amount
        for leg in flow.evaluate(state, _env(state, 1.0), 1.0).legs
        if leg.stock == _SOIL_WATER
    )
    half = next(
        leg.amount
        for leg in flow.evaluate(state, _env(state, 0.5), 0.5).legs
        if leg.stock == _SOIL_WATER
    )
    assert math.isclose(half, one * 0.5, rel_tol=1e-12)


def _irrigation_flow(ground_area: float = 2.0) -> Irrigation:
    return Irrigation(
        id=FlowId("biosphere.irrigation"),
        priority=0,
        water_source=_WATER_SOURCE,
        soil_water=_SOIL_WATER,
        irrigation_var="irrigation",
        ground_area=ground_area,
    )


def test_irrigation_leg_is_rate_times_area() -> None:
    # 5 mm/day over 2 m² ⇒ 10 kg/day into soil_water.
    state = _state(soil_water0=15.0)
    result = _irrigation_flow(2.0).evaluate(
        state, _env(state, 1.0, irrigation=5.0), 1.0
    )
    legs = {leg.stock: leg.amount for leg in result.legs}
    assert math.isclose(legs[_SOIL_WATER], 10.0, rel_tol=1e-12)
    assert math.isclose(legs[_WATER_SOURCE], -10.0, rel_tol=1e-12)


def test_irrigation_is_water_balanced() -> None:
    state = _state(soil_water0=15.0)
    result = _irrigation_flow().evaluate(state, _env(state, 1.0), 1.0)
    assert_flow_balanced(result, state.stocks)


# --- config boundary: load_transpiration_params -----------------------------
def test_transpiration_params_file_exists() -> None:
    assert TRANSPIRATION_PARAMS_PATH.is_file()


def test_load_transpiration_params_matches_committed_values() -> None:
    p = load_transpiration_params()
    assert isinstance(p, TranspirationParams)
    assert (p.aerodynamic_resistance, p.surface_resistance) == (_RA, _RS)


def _valid_transp() -> dict[str, Any]:
    return {
        "name": "winter_wheat",
        "process": "transpiration",
        "parameters": {
            "aerodynamic_resistance": {"value": 50.0, "unit": "s/m", "source": "[A]"},
            "surface_resistance": {"value": 70.0, "unit": "s/m", "source": "[A]"},
        },
    }


def _write_transp(tmp_path: Path, data: dict[str, Any]) -> Path:
    p = tmp_path / "transpiration.yaml"
    p.write_text(yaml.safe_dump(data), encoding="utf-8")
    return p


def test_transp_loader_round_trips_a_valid_file(tmp_path: Path) -> None:
    p = load_transpiration_params(_write_transp(tmp_path, _valid_transp()))
    assert p.aerodynamic_resistance == 50.0
    assert p.surface_resistance == 70.0


def test_transp_loader_rejects_a_wrong_unit(tmp_path: Path) -> None:
    data = _valid_transp()
    data["parameters"]["aerodynamic_resistance"]["unit"] = "min/m"  # wrong scale
    with pytest.raises(ValueError, match="aerodynamic_resistance"):
        load_transpiration_params(_write_transp(tmp_path, data))


@pytest.mark.parametrize("field", ["aerodynamic_resistance", "surface_resistance"])
def test_transp_loader_rejects_non_positive(tmp_path: Path, field: str) -> None:
    data = _valid_transp()
    data["parameters"][field]["value"] = 0.0
    with pytest.raises(ValueError, match=field):
        load_transpiration_params(_write_transp(tmp_path, data))


def test_transp_loader_rejects_a_missing_source(tmp_path: Path) -> None:
    data = _valid_transp()
    del data["parameters"]["surface_resistance"]["source"]
    with pytest.raises(ValidationError):
        load_transpiration_params(_write_transp(tmp_path, data))


def test_transp_loader_rejects_an_unknown_field(tmp_path: Path) -> None:
    data = _valid_transp()
    data["parameters"]["bogus"] = {"value": 1.0, "unit": "s/m", "source": "x"}
    with pytest.raises(ValidationError):
        load_transpiration_params(_write_transp(tmp_path, data))
