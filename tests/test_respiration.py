"""Phase-1 Step-6 tests: maintenance + growth respiration (the carbon sink flows).

The counterpart to Step 5's gross assimilation. Layers:

* **Rate laws** (``domains.biosphere.respiration``, pure stdlib): the ``Q10``
  temperature multiplier, maintenance respiration (∝ biomass × Q10), and the
  maintenance-first growth-respiration loss ``(1 − Yg)·max(0, GASS − MRES)``, checked
  against **independent hand-computed** literals and the ``MRES ≥ GASS`` → 0 clamp.
* **Config boundary** (``load_respiration_params``): the committed file loads to the
  expected params; bad units / out-of-range values / a missing source are rejected.

The assembled flows (``MaintenanceRespiration``, ``GrowthRespiration``) moved to
``domains.biosphere.carbon_budget`` at Step 11 (the buffer dissolution); they are tested
in ``test_carbon_budget.py``. The maintenance / growth-respiration rate laws are here.
"""

import math
from pathlib import Path
from typing import Any

import pytest
import yaml
from pydantic import ValidationError

from domains.biosphere.loader import RESPIRATION_PARAMS_PATH, load_respiration_params
from domains.biosphere.respiration import (
    RespirationParams,
    growth_respiration_flux,
    maintenance_respiration_flux,
    q10_factor,
)

# The committed winter-wheat provisional placeholders (mirror respiration.yaml).
_M_REF, _Q10, _T_REF, _YG = 0.02, 2.0, 25.0, 0.75
_O2_KSAT = 0.001  # committed O₂ half-saturation (mole fraction) for the Step-7 f_O2


def _resp() -> RespirationParams:
    return RespirationParams(
        maintenance_coef=_M_REF,
        q10=_Q10,
        t_ref=_T_REF,
        growth_efficiency=_YG,
        o2_half_saturation=0.001,  # unused here (rate-law tests don't touch O₂)
    )


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


# --- config boundary: load_respiration_params -------------------------------
def test_respiration_params_file_exists() -> None:
    assert RESPIRATION_PARAMS_PATH.is_file()


def test_load_respiration_params_matches_committed_values() -> None:
    p = load_respiration_params()
    assert isinstance(p, RespirationParams)
    assert (
        p.maintenance_coef,
        p.q10,
        p.t_ref,
        p.growth_efficiency,
        p.o2_half_saturation,
    ) == (
        _M_REF,
        _Q10,
        _T_REF,
        _YG,
        _O2_KSAT,
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
            "o2_half_saturation": {
                "value": 0.001,
                "unit": "mol/mol",
                "source": "[C]",
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
