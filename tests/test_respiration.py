"""Phase-1 Step-6 tests: maintenance + growth respiration (the carbon sink flows).

The counterpart to Step 5's gross assimilation. Layers:

* **Rate laws** (``domains.biosphere.respiration``, pure stdlib): the ``Q10``
  temperature multiplier, maintenance respiration (∝ biomass × Q10), and the
  maintenance-first growth-respiration loss ``(1 − Yg)·max(0, GASS − MRES)``, checked
  against **independent hand-computed** literals and the ``MRES ≥ GASS`` → 0 clamp.
* **The assembled flows**: ``MaintenanceRespiration`` (carbon-balanced, dt-linear,
  rate set by the temperature forcing) and ``GrowthRespiration`` (a composed,
  carbon-balanced ``FlowResult`` recomputing GASS via the Step-4/5 canopy/FvCB stack
  and MRES via the shared maintenance helper; the dark-day → 0 clamp).
* **Config boundary** (``load_respiration_params``): the committed file loads to the
  expected params; bad units / out-of-range values / a missing source are rejected.
"""

import math
from pathlib import Path
from typing import Any

import pytest
import yaml
from pydantic import ValidationError

from domains.biosphere.canopy import CanopyParams
from domains.biosphere.loader import RESPIRATION_PARAMS_PATH, load_respiration_params
from domains.biosphere.photosynthesis import (
    PhotosynthesisParams,
    daily_canopy_assimilation,
)
from domains.biosphere.respiration import (
    GrowthRespiration,
    MaintenanceRespiration,
    RespirationParams,
    growth_respiration_flux,
    maintenance_respiration_flux,
    q10_factor,
)
from simcore.environment import SourceResolver, constant
from simcore.flow import assert_flow_balanced
from simcore.ids import DomainId, FlowId, StockId
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import State, Stock

# The committed winter-wheat provisional placeholders (mirror respiration.yaml).
_M_REF, _Q10, _T_REF, _YG = 0.02, 2.0, 25.0, 0.75

# FvCB + canopy placeholders (mirror photosynthesis.yaml / canopy.yaml; see those tests)
# — needed to drive the composed GrowthRespiration flow at a known operating point.
_VCMAX, _JMAX, _ALPHA, _THETA = 100.0, 180.0, 0.3, 0.7
_GAMMA_STAR, _KC, _KO, _O2 = 42.75, 404.9, 278.4, 210.0
_TMIN, _TOPT_LO, _TOPT_HI, _TMAX = 0.0, 15.0, 25.0, 35.0
_SLA_PER_MOL_C, _K = 0.5872044444444445, 0.6


def _resp() -> RespirationParams:
    return RespirationParams(
        maintenance_coef=_M_REF, q10=_Q10, t_ref=_T_REF, growth_efficiency=_YG
    )


def _photo() -> PhotosynthesisParams:
    return PhotosynthesisParams(
        vcmax=_VCMAX,
        jmax=_JMAX,
        quantum_yield=_ALPHA,
        theta=_THETA,
        gamma_star=_GAMMA_STAR,
        kc=_KC,
        ko=_KO,
        o2=_O2,
        t_min=_TMIN,
        t_opt_lo=_TOPT_LO,
        t_opt_hi=_TOPT_HI,
        t_max=_TMAX,
    )


def _canopy() -> CanopyParams:
    return CanopyParams(sla_per_mol_c=_SLA_PER_MOL_C, extinction_coef=_K)


# --- Q10 temperature multiplier: independent literals -----------------------
@pytest.mark.parametrize(
    ("temp", "expected"),
    [
        (25.0, 1.0),  # at t_ref
        (35.0, 2.0),  # +10 °C ⇒ ×q10
        (45.0, 4.0),  # +20 °C ⇒ ×q10²
        (15.0, 0.5),  # −10 °C ⇒ ÷q10
        (5.0, 0.25),  # −20 °C ⇒ ÷q10²
    ],
)
def test_q10_factor_known_values(temp: float, expected: float) -> None:
    assert math.isclose(
        q10_factor(temp, q10=_Q10, t_ref=_T_REF), expected, rel_tol=1e-12
    )


# --- maintenance respiration: independent literals --------------------------
def test_maintenance_respiration_known_value_at_reference_temp() -> None:
    # At t_ref the Q10 factor is 1: MRES = 0.02 · 5 · 1 = 0.1 mol C/day.
    mres = maintenance_respiration_flux(5.0, 25.0, params=_resp())
    assert math.isclose(mres, 0.1, rel_tol=1e-12)


@pytest.mark.parametrize(("temp", "expected"), [(35.0, 0.2), (15.0, 0.05)])
def test_maintenance_respiration_tracks_q10(temp: float, expected: float) -> None:
    mres = maintenance_respiration_flux(5.0, temp, params=_resp())
    assert math.isclose(mres, expected, rel_tol=1e-12)


def test_maintenance_respiration_proportional_to_biomass() -> None:
    a = maintenance_respiration_flux(3.0, 20.0, params=_resp())
    b = maintenance_respiration_flux(6.0, 20.0, params=_resp())
    assert math.isclose(b, 2.0 * a, rel_tol=1e-12)


def test_maintenance_respiration_zero_biomass_is_zero() -> None:
    # Self-limiting: no tissue, no maintenance cost (positivity is structural).
    assert maintenance_respiration_flux(0.0, 30.0, params=_resp()) == 0.0


def test_maintenance_respiration_maturity_seam_scales_linearly() -> None:
    # The deferred development-stage/senescence down-scaling seam (default 1.0).
    full = maintenance_respiration_flux(5.0, 20.0, params=_resp())
    half = maintenance_respiration_flux(5.0, 20.0, params=_resp(), maturity=0.5)
    assert math.isclose(half, 0.5 * full, rel_tol=1e-12)


# --- growth respiration: independent literals + the clamp -------------------
def test_growth_respiration_known_value() -> None:
    # (1 − 0.75) · max(0, 1.0 − 0.2) = 0.25 · 0.8 = 0.2 mol C/day.
    assert math.isclose(
        growth_respiration_flux(1.0, 0.2, growth_efficiency=_YG), 0.2, rel_tol=1e-12
    )


@pytest.mark.parametrize(("gross", "maintenance"), [(0.2, 1.0), (1.0, 1.0)])
def test_growth_respiration_clamps_when_maintenance_meets_or_exceeds_gross(
    gross: float, maintenance: float
) -> None:
    # MRES ≥ GASS ⇒ no growth ⇒ no growth respiration (never a carbon-creating deposit).
    assert growth_respiration_flux(gross, maintenance, growth_efficiency=_YG) == 0.0


# --- the assembled MaintenanceRespiration flow ------------------------------
_BIO = DomainId("biosphere")
_PLANT_C = StockId("biosphere.plant_c")
_CO2 = StockId("boundary.co2")


def _state(plant_c0: float) -> State:
    carbon = canonical_unit(Quantity.CARBON)
    plant = Stock(
        id=_PLANT_C,
        domain=_BIO,
        quantity=Quantity.CARBON,
        unit=carbon,
        amount=plant_c0,
        kind=StockKind.POPULATION,
        extinction_threshold=0.0,
    )
    co2 = Stock(
        id=_CO2,
        domain=DomainId("boundary"),
        quantity=Quantity.CARBON,
        unit=carbon,
        amount=1.0e9,
        kind=StockKind.BOUNDARY,
        unclamped=True,
    )
    return State(n=0, stocks={_PLANT_C: plant, _CO2: co2}, rng_seed=0)


def _env(snapshot: State, dt: float, *, par: float = 800.0, temp: float = 20.0):  # noqa: ANN202
    resolver = SourceResolver(
        forcings={
            "par": constant(par),
            "ci": constant(400.0),
            "temp": constant(temp),
            "daylength_s": constant(43200.0),
        }
    )
    return resolver.bind(snapshot, dt)


def _maintenance_flow() -> MaintenanceRespiration:
    return MaintenanceRespiration(
        id=FlowId("biosphere.maintenance_respiration"),
        priority=0,
        plant_c=_PLANT_C,
        co2_sink=_CO2,
        temp_var="temp",
        params=_resp(),
    )


def test_maintenance_flow_leg_is_the_hand_computed_daily_flux() -> None:
    state = _state(plant_c0=5.0)
    result = _maintenance_flow().evaluate(state, _env(state, 1.0, temp=20.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    expected = maintenance_respiration_flux(5.0, 20.0, params=_resp())
    assert math.isclose(legs[_PLANT_C], -expected, rel_tol=1e-12)
    assert math.isclose(legs[_CO2], expected, rel_tol=1e-12)


def test_maintenance_flow_is_carbon_balanced() -> None:
    state = _state(plant_c0=5.0)
    result = _maintenance_flow().evaluate(state, _env(state, 1.0), 1.0)
    assert_flow_balanced(result, state.stocks)


def test_maintenance_flow_scales_linearly_with_dt() -> None:
    state = _state(plant_c0=5.0)
    flow = _maintenance_flow()
    one = next(
        leg.amount
        for leg in flow.evaluate(state, _env(state, 1.0), 1.0).legs
        if leg.stock == _PLANT_C
    )
    half = next(
        leg.amount
        for leg in flow.evaluate(state, _env(state, 0.5), 0.5).legs
        if leg.stock == _PLANT_C
    )
    assert math.isclose(half, one * 0.5, rel_tol=1e-12)


# --- the assembled GrowthRespiration flow (composed) ------------------------
def _growth_flow() -> GrowthRespiration:
    return GrowthRespiration(
        id=FlowId("biosphere.growth_respiration"),
        priority=0,
        plant_c=_PLANT_C,
        co2_sink=_CO2,
        par_var="par",
        ci_var="ci",
        temp_var="temp",
        daylength_var="daylength_s",
        photo=_photo(),
        canopy=_canopy(),
        resp=_resp(),
        ground_area=1.0,
    )


def test_growth_flow_leg_is_the_composed_gass_minus_mres_loss() -> None:
    # Composed (Steps 4+5+6): GASS via the canopy/FvCB stack, MRES via the shared
    # maintenance helper, both at the same operating point the flow reads from forcing.
    state = _state(plant_c0=5.0)
    lai = 5.0 * _SLA_PER_MOL_C / 1.0
    gross = daily_canopy_assimilation(
        800.0,
        lai,
        400.0,
        20.0,
        43200.0,
        params=_photo(),
        canopy=_canopy(),
        ground_area=1.0,
    )
    mres = maintenance_respiration_flux(5.0, 20.0, params=_resp())
    expected = growth_respiration_flux(gross, mres, growth_efficiency=_YG)
    result = _growth_flow().evaluate(state, _env(state, 1.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    assert math.isclose(legs[_PLANT_C], -expected, rel_tol=1e-12)
    assert math.isclose(legs[_CO2], expected, rel_tol=1e-12)
    # Cross-check against the pinned composed literal (mol C/day at this point).
    assert math.isclose(expected, 0.32678769775306143, rel_tol=1e-12)


def test_growth_flow_is_carbon_balanced() -> None:
    state = _state(plant_c0=5.0)
    result = _growth_flow().evaluate(state, _env(state, 1.0), 1.0)
    assert_flow_balanced(result, state.stocks)


def test_growth_flow_clamps_to_zero_in_the_dark() -> None:
    # No light ⇒ GASS = 0 ⇒ MRES > GASS ⇒ growth respiration clamps to 0 (no deposit).
    state = _state(plant_c0=5.0)
    result = _growth_flow().evaluate(state, _env(state, 1.0, par=0.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    assert legs[_PLANT_C] == 0.0
    assert legs[_CO2] == 0.0


def test_growth_flow_scales_linearly_with_dt() -> None:
    state = _state(plant_c0=5.0)
    flow = _growth_flow()
    one = next(
        leg.amount
        for leg in flow.evaluate(state, _env(state, 1.0), 1.0).legs
        if leg.stock == _PLANT_C
    )
    half = next(
        leg.amount
        for leg in flow.evaluate(state, _env(state, 0.5), 0.5).legs
        if leg.stock == _PLANT_C
    )
    assert math.isclose(half, one * 0.5, rel_tol=1e-12)


# --- config boundary: load_respiration_params -------------------------------
def test_respiration_params_file_exists() -> None:
    assert RESPIRATION_PARAMS_PATH.is_file()


def test_load_respiration_params_matches_committed_values() -> None:
    p = load_respiration_params()
    assert isinstance(p, RespirationParams)
    assert (p.maintenance_coef, p.q10, p.t_ref, p.growth_efficiency) == (
        _M_REF,
        _Q10,
        _T_REF,
        _YG,
    )


def _valid_resp() -> dict[str, Any]:
    return {
        "name": "winter_wheat",
        "process": "respiration",
        "parameters": {
            "maintenance_coef": {"value": 0.02, "unit": "1/day", "source": "[A]"},
            "q10": {"value": 2.0, "unit": "dimensionless", "source": "[B]"},
            "t_ref": {"value": 25.0, "unit": "degC", "source": "[B]"},
            "growth_efficiency": {
                "value": 0.75,
                "unit": "dimensionless",
                "source": "[B]",
            },
        },
    }


def _write_resp(tmp_path: Path, data: dict[str, Any]) -> Path:
    p = tmp_path / "respiration.yaml"
    p.write_text(yaml.safe_dump(data), encoding="utf-8")
    return p


def test_resp_loader_round_trips_a_valid_file(tmp_path: Path) -> None:
    p = load_respiration_params(_write_resp(tmp_path, _valid_resp()))
    assert p.maintenance_coef == 0.02
    assert p.growth_efficiency == 0.75


def test_resp_loader_rejects_a_wrong_unit(tmp_path: Path) -> None:
    data = _valid_resp()
    data["parameters"]["maintenance_coef"]["unit"] = "1/s"  # right dim, wrong scale
    with pytest.raises(ValueError, match="maintenance_coef"):
        load_respiration_params(_write_resp(tmp_path, data))


@pytest.mark.parametrize("field", ["maintenance_coef", "q10"])
def test_resp_loader_rejects_non_positive_rates(tmp_path: Path, field: str) -> None:
    data = _valid_resp()
    data["parameters"][field]["value"] = 0.0
    with pytest.raises(ValueError, match=field):
        load_respiration_params(_write_resp(tmp_path, data))


@pytest.mark.parametrize("bad", [0.0, -0.1, 1.5])
def test_resp_loader_rejects_out_of_unit_interval_efficiency(
    tmp_path: Path, bad: float
) -> None:
    data = _valid_resp()
    data["parameters"]["growth_efficiency"]["value"] = bad
    with pytest.raises(ValueError, match="growth_efficiency"):
        load_respiration_params(_write_resp(tmp_path, data))


def test_resp_loader_rejects_a_missing_source(tmp_path: Path) -> None:
    data = _valid_resp()
    del data["parameters"]["q10"]["source"]
    with pytest.raises(ValidationError):
        load_respiration_params(_write_resp(tmp_path, data))


def test_resp_loader_rejects_an_unknown_field(tmp_path: Path) -> None:
    data = _valid_resp()
    data["parameters"]["bogus"] = {"value": 1.0, "unit": "dimensionless", "source": "x"}
    with pytest.raises(ValidationError):
        load_respiration_params(_write_resp(tmp_path, data))
