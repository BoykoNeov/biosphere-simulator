"""Phase-1 Step-10 tests: nitrogen uptake + limitation (NITROGEN flows + f_N).

The last of the seven processes — the NITROGEN-currency mirror of Step 7 (water).
Layers (mirroring Steps 5/6/7):

* **Rate laws** (``domains.biosphere.nitrogen``, pure stdlib): the soil-N availability
  factor (uptake supply side) and the plant-N stress factor ``f_N`` (the photosynthesis
  limiter) against independent literals + their band clamps and guards.
* **The assembled flows**: ``NitrogenUptake`` (N-balanced, dt-linear, self-limiting via
  soil-N availability on the step-entry soil N) and ``Fertilization`` (N-balanced,
  tracking the N-application forcing).
* **Config boundary** (``load_nitrogen_params``): the committed file loads to the
  expected params with the kg N/kg DM → kg N/mol C carbon-fraction fold applied; bad
  units / out-of-range values / an inverted band / a missing source are rejected.
"""

import math
from pathlib import Path
from typing import Any

import pytest
import yaml
from pydantic import ValidationError

from domains.biosphere.loader import (
    MOLAR_MASS_CARBON_KG_PER_MOL,
    NITROGEN_PARAMS_PATH,
    load_nitrogen_params,
)
from domains.biosphere.nitrogen import (
    Fertilization,
    NitrogenParams,
    NitrogenUptake,
    nitrogen_stress_factor,
    soil_n_availability,
)
from simcore.environment import SourceResolver, constant
from simcore.flow import assert_flow_balanced
from simcore.ids import DomainId, FlowId, StockId
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import State, Stock

# The committed winter-wheat provisional placeholders (mirror nitrogen.yaml).
_MAX_UPTAKE = 0.0015  # kg N m⁻² day⁻¹
_N_RESIDUAL_KG_KG = 0.005  # kg N/kg DM
_N_CRITICAL_KG_KG = 0.015  # kg N/kg DM
_CARBON_FRACTION = 0.45


# --- soil-N availability: cardinal values + clamp (the f_water shape) --------
@pytest.mark.parametrize(
    ("soil_n", "expected"),
    [
        (0.005, 0.0),  # below residual
        (0.01, 0.0),  # at residual
        (0.02, 0.25),  # quarter of the [0.01, 0.05] band
        (0.03, 0.5),  # midpoint
        (0.05, 1.0),  # at critical
        (0.07, 1.0),  # above critical
    ],
)
def test_soil_n_availability_cardinal_values(soil_n: float, expected: float) -> None:
    assert math.isclose(
        soil_n_availability(soil_n, sn_residual=0.01, sn_critical=0.05),
        expected,
        rel_tol=1e-12,
    )


def test_soil_n_availability_rejects_inverted_band() -> None:
    with pytest.raises(ValueError, match="sn_residual < sn_critical"):
        soil_n_availability(0.03, sn_residual=0.05, sn_critical=0.01)


# --- plant-N stress factor f_N: cardinal values, guard, clamp ---------------
# conc = plant_n / biomass_c (kg N / mol C). With biomass_c = 1000 mol C and the band
# [1e-4, 5e-4], plant_n = 0.1/0.3/0.5 kg N hit conc = 1e-4/3e-4/5e-4 ⇒ f_N = 0/0.5/1.
@pytest.mark.parametrize(
    ("plant_n", "expected"),
    [
        (0.05, 0.0),  # conc 5e-5, below residual
        (0.10, 0.0),  # conc 1e-4, at residual
        (0.20, 0.25),  # conc 2e-4, quarter of the [1e-4, 5e-4] band
        (0.30, 0.5),  # conc 3e-4, midpoint
        (0.50, 1.0),  # conc 5e-4, at critical
        (0.80, 1.0),  # conc 8e-4, above critical
    ],
)
def test_nitrogen_stress_factor_cardinal_values(
    plant_n: float, expected: float
) -> None:
    assert math.isclose(
        nitrogen_stress_factor(
            plant_n,
            1000.0,
            n_residual_per_mol_c=1e-4,
            n_critical_per_mol_c=5e-4,
        ),
        expected,
        rel_tol=1e-12,
    )


@pytest.mark.parametrize("biomass_c", [0.0, -1.0])
def test_nitrogen_stress_factor_zero_or_negative_biomass_is_neutral(
    biomass_c: float,
) -> None:
    # No biomass ⇒ no leaves ⇒ photosynthesis already 0 via LAI=0; f_N is neutral (1.0),
    # never a divide-by-zero.
    assert (
        nitrogen_stress_factor(
            0.0, biomass_c, n_residual_per_mol_c=1e-4, n_critical_per_mol_c=5e-4
        )
        == 1.0
    )


def test_nitrogen_stress_factor_rejects_inverted_band() -> None:
    with pytest.raises(ValueError, match="n_residual_per_mol_c < n_critical_per_mol_c"):
        nitrogen_stress_factor(
            0.3, 1000.0, n_residual_per_mol_c=5e-4, n_critical_per_mol_c=1e-4
        )


# --- the assembled flows ----------------------------------------------------
_BIO = DomainId("biosphere")
_BOUNDARY = DomainId("boundary")
_SOIL_N = StockId("biosphere.soil_n")
_PLANT_N = StockId("biosphere.plant_n")
_N_SOURCE = StockId("boundary.fertilizer_supply")

_SN_RESID, _SN_CRIT = 0.01, 0.05


def _params() -> NitrogenParams:
    fold = MOLAR_MASS_CARBON_KG_PER_MOL / _CARBON_FRACTION
    return NitrogenParams(
        max_uptake_capacity=_MAX_UPTAKE,
        n_residual_per_mol_c=_N_RESIDUAL_KG_KG * fold,
        n_critical_per_mol_c=_N_CRITICAL_KG_KG * fold,
    )


def _state(soil_n0: float, plant_n0: float = 0.2) -> State:
    nitrogen = canonical_unit(Quantity.NITROGEN)
    soil = Stock(
        id=_SOIL_N,
        domain=_BIO,
        quantity=Quantity.NITROGEN,
        unit=nitrogen,
        amount=soil_n0,
        kind=StockKind.POOL,
    )
    plant = Stock(
        id=_PLANT_N,
        domain=_BIO,
        quantity=Quantity.NITROGEN,
        unit=nitrogen,
        amount=plant_n0,
        kind=StockKind.POOL,
    )
    source = Stock(
        id=_N_SOURCE,
        domain=_BOUNDARY,
        quantity=Quantity.NITROGEN,
        unit=nitrogen,
        amount=1.0e9,
        kind=StockKind.BOUNDARY,
        unclamped=True,
    )
    return State(
        n=0,
        stocks={_SOIL_N: soil, _PLANT_N: plant, _N_SOURCE: source},
        rng_seed=0,
    )


def _env(snapshot: State, dt: float, *, fertilization: float = 0.001):  # noqa: ANN202
    resolver = SourceResolver(forcings={"fertilization": constant(fertilization)})
    return resolver.bind(snapshot, dt)


def _uptake_flow(ground_area: float = 1.0) -> NitrogenUptake:
    return NitrogenUptake(
        id=FlowId("biosphere.n_uptake"),
        priority=0,
        soil_n=_SOIL_N,
        plant_n=_PLANT_N,
        params=_params(),
        ground_area=ground_area,
        sn_residual=_SN_RESID,
        sn_critical=_SN_CRIT,
    )


def test_uptake_leg_is_capacity_times_availability_times_area() -> None:
    # soil_n=0.03 ⇒ availability=(0.03−0.01)/0.04=0.5; capacity=0.0015; area=1 m².
    state = _state(soil_n0=0.03)
    result = _uptake_flow(ground_area=1.0).evaluate(state, _env(state, 1.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    expected = _MAX_UPTAKE * 1.0 * 0.5  # 0.00075 kg N/day
    assert math.isclose(legs[_PLANT_N], expected, rel_tol=1e-12)
    assert math.isclose(legs[_SOIL_N], -expected, rel_tol=1e-12)
    assert math.isclose(expected, 0.00075, rel_tol=1e-12)


def test_uptake_is_nitrogen_balanced() -> None:
    state = _state(soil_n0=0.03)
    result = _uptake_flow().evaluate(state, _env(state, 1.0), 1.0)
    assert_flow_balanced(result, state.stocks)


def test_uptake_shuts_off_at_residual_soil_n() -> None:
    # soil_n at the residual point ⇒ availability=0 ⇒ no uptake (structural positivity:
    # the pool cannot be drawn below residual).
    state = _state(soil_n0=_SN_RESID)
    result = _uptake_flow().evaluate(state, _env(state, 1.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    assert legs[_SOIL_N] == 0.0
    assert legs[_PLANT_N] == 0.0


def test_uptake_scales_with_ground_area() -> None:
    state = _state(soil_n0=0.03)
    one = next(
        leg.amount
        for leg in _uptake_flow(1.0).evaluate(state, _env(state, 1.0), 1.0).legs
        if leg.stock == _PLANT_N
    )
    triple = next(
        leg.amount
        for leg in _uptake_flow(3.0).evaluate(state, _env(state, 1.0), 1.0).legs
        if leg.stock == _PLANT_N
    )
    assert math.isclose(triple, 3.0 * one, rel_tol=1e-12)


def test_uptake_scales_linearly_with_dt() -> None:
    state = _state(soil_n0=0.03)
    flow = _uptake_flow()
    one = next(
        leg.amount
        for leg in flow.evaluate(state, _env(state, 1.0), 1.0).legs
        if leg.stock == _PLANT_N
    )
    half = next(
        leg.amount
        for leg in flow.evaluate(state, _env(state, 0.5), 0.5).legs
        if leg.stock == _PLANT_N
    )
    assert math.isclose(half, one * 0.5, rel_tol=1e-12)


def _fertilization_flow(ground_area: float = 2.0) -> Fertilization:
    return Fertilization(
        id=FlowId("biosphere.fertilization"),
        priority=0,
        n_source=_N_SOURCE,
        soil_n=_SOIL_N,
        fertilization_var="fertilization",
        ground_area=ground_area,
    )


def test_fertilization_leg_is_rate_times_area() -> None:
    # 0.001 kg N m⁻² day⁻¹ over 2 m² ⇒ 0.002 kg N/day into soil_n.
    state = _state(soil_n0=0.03)
    result = _fertilization_flow(2.0).evaluate(
        state, _env(state, 1.0, fertilization=0.001), 1.0
    )
    legs = {leg.stock: leg.amount for leg in result.legs}
    assert math.isclose(legs[_SOIL_N], 0.002, rel_tol=1e-12)
    assert math.isclose(legs[_N_SOURCE], -0.002, rel_tol=1e-12)


def test_fertilization_is_nitrogen_balanced() -> None:
    state = _state(soil_n0=0.03)
    result = _fertilization_flow().evaluate(state, _env(state, 1.0), 1.0)
    assert_flow_balanced(result, state.stocks)


# --- config boundary: load_nitrogen_params ----------------------------------
def test_nitrogen_params_file_exists() -> None:
    assert NITROGEN_PARAMS_PATH.is_file()


def test_load_nitrogen_params_applies_carbon_fraction_fold() -> None:
    p = load_nitrogen_params()
    assert isinstance(p, NitrogenParams)
    assert p.max_uptake_capacity == _MAX_UPTAKE
    fold = MOLAR_MASS_CARBON_KG_PER_MOL / _CARBON_FRACTION
    assert math.isclose(p.n_residual_per_mol_c, _N_RESIDUAL_KG_KG * fold, rel_tol=1e-12)
    assert math.isclose(p.n_critical_per_mol_c, _N_CRITICAL_KG_KG * fold, rel_tol=1e-12)


def test_committed_nitrogen_carbon_fraction_matches_canopy() -> None:
    # Transition checklist item 3: nitrogen.yaml's carbon_fraction MUST equal
    # canopy.yaml's, or the two folds (sla → m²/mol C; N-thresholds → kg N/mol C) model
    # a silently inconsistent plant.
    nitrogen = yaml.safe_load(NITROGEN_PARAMS_PATH.read_text(encoding="utf-8"))
    from domains.biosphere.loader import CANOPY_PARAMS_PATH

    canopy = yaml.safe_load(CANOPY_PARAMS_PATH.read_text(encoding="utf-8"))
    assert (
        nitrogen["parameters"]["carbon_fraction"]["value"]
        == canopy["parameters"]["carbon_fraction"]["value"]
    )


def _valid_nitrogen() -> dict[str, Any]:
    return {
        "name": "winter_wheat",
        "process": "nitrogen",
        "parameters": {
            "max_uptake_capacity": {
                "value": 0.0015,
                "unit": "kg/m^2/day",
                "source": "[A]",
            },
            "n_residual": {"value": 0.005, "unit": "kg/kg", "source": "[A]"},
            "n_critical": {"value": 0.015, "unit": "kg/kg", "source": "[A]"},
            "carbon_fraction": {
                "value": 0.45,
                "unit": "dimensionless",
                "source": "[A]",
            },
        },
    }


def _write_nitrogen(tmp_path: Path, data: dict[str, Any]) -> Path:
    p = tmp_path / "nitrogen.yaml"
    p.write_text(yaml.safe_dump(data), encoding="utf-8")
    return p


def test_nitrogen_loader_round_trips_a_valid_file(tmp_path: Path) -> None:
    p = load_nitrogen_params(_write_nitrogen(tmp_path, _valid_nitrogen()))
    assert p.max_uptake_capacity == 0.0015


def test_nitrogen_loader_rejects_a_wrong_unit(tmp_path: Path) -> None:
    data = _valid_nitrogen()
    data["parameters"]["max_uptake_capacity"]["unit"] = "kg/ha/day"  # wrong scale
    with pytest.raises(ValueError, match="max_uptake_capacity"):
        load_nitrogen_params(_write_nitrogen(tmp_path, data))


def test_nitrogen_loader_rejects_non_positive_capacity(tmp_path: Path) -> None:
    data = _valid_nitrogen()
    data["parameters"]["max_uptake_capacity"]["value"] = 0.0
    with pytest.raises(ValueError, match="max_uptake_capacity"):
        load_nitrogen_params(_write_nitrogen(tmp_path, data))


def test_nitrogen_loader_rejects_out_of_range_carbon_fraction(tmp_path: Path) -> None:
    data = _valid_nitrogen()
    data["parameters"]["carbon_fraction"]["value"] = 1.5
    with pytest.raises(ValueError, match="carbon_fraction"):
        load_nitrogen_params(_write_nitrogen(tmp_path, data))


def test_nitrogen_loader_rejects_inverted_concentration_band(tmp_path: Path) -> None:
    data = _valid_nitrogen()
    data["parameters"]["n_residual"]["value"] = 0.02  # >= n_critical (0.015)
    with pytest.raises(ValueError, match="n_residual < n_critical"):
        load_nitrogen_params(_write_nitrogen(tmp_path, data))


def test_nitrogen_loader_rejects_negative_residual(tmp_path: Path) -> None:
    data = _valid_nitrogen()
    data["parameters"]["n_residual"]["value"] = -0.001
    with pytest.raises(ValueError, match="n_residual"):
        load_nitrogen_params(_write_nitrogen(tmp_path, data))


def test_nitrogen_loader_rejects_a_missing_source(tmp_path: Path) -> None:
    data = _valid_nitrogen()
    del data["parameters"]["n_critical"]["source"]
    with pytest.raises(ValidationError):
        load_nitrogen_params(_write_nitrogen(tmp_path, data))


def test_nitrogen_loader_rejects_an_unknown_field(tmp_path: Path) -> None:
    data = _valid_nitrogen()
    data["parameters"]["bogus"] = {"value": 1.0, "unit": "kg/kg", "source": "x"}
    with pytest.raises(ValidationError):
        load_nitrogen_params(_write_nitrogen(tmp_path, data))
