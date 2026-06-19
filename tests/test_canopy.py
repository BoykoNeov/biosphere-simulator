"""Phase-1 Step-4 tests: Beer–Lambert canopy light interception (Monsi & Saeki 1953).

The first biological process — a **pure canopy diagnostic**, not a flow and not an
aux accumulator. Two layers:

* **Physics** (``domains.biosphere.canopy``, pure stdlib): the leaf-area-index and
  intercepted-fraction free functions, checked against **independent hand-computed**
  known values (not internal restatements of the formula), plus the limits
  (LAI=0 → 0; large k·LAI → 1), monotonicity, and the ``ground_area > 0`` guard.
* **Config boundary** (``load_canopy_params`` + ``config.convert``): the structured
  ``value/unit/source`` param file loads to the folded ``sla_per_mol_c`` (m² per mol
  leaf C), the per-area SLA unit is pint-validated/converted, and bad units / out-of-
  range coefficients are rejected.
"""

import math
from pathlib import Path
from typing import Any

import pytest
import yaml
from pydantic import ValidationError

from config import UnitValidationError, convert
from domains.biosphere.canopy import (
    CanopyParams,
    intercepted_fraction,
    leaf_area_index,
)
from domains.biosphere.loader import (
    CANOPY_PARAMS_PATH,
    MOLAR_MASS_CARBON_KG_PER_MOL,
    load_canopy_params,
)


# --- Beer–Lambert intercepted fraction: independent known values ------------
@pytest.mark.parametrize(
    ("lai", "k", "expected"),
    [
        # Hand-computed 1 - e^(-k·LAI), asserted against a literal (not 1-exp(...),
        # which would just restate the implementation).
        (3.0, 0.6, 0.8347011117784134),
        (2.0, 0.5, 0.6321205588285577),
    ],
)
def test_intercepted_fraction_known_values(
    lai: float, k: float, expected: float
) -> None:
    assert math.isclose(
        intercepted_fraction(lai, extinction_coef=k), expected, rel_tol=1e-12
    )


def test_intercepted_fraction_zero_lai_is_zero() -> None:
    # No leaf area intercepts no light: 1 - e^0 = 0 exactly.
    assert intercepted_fraction(0.0, extinction_coef=0.6) == 0.0


def test_intercepted_fraction_saturates_to_one() -> None:
    # A closed canopy (large k·LAI) intercepts ~all incident PAR, asymptoting to 1
    # from below — never reaching or exceeding it.
    f = intercepted_fraction(50.0, extinction_coef=0.7)
    assert 0.0 < f < 1.0
    assert math.isclose(f, 1.0, abs_tol=1e-10)


def test_intercepted_fraction_monotone_increasing_in_lai() -> None:
    # More leaf area intercepts strictly more light (the extinction law is monotone).
    fractions = [intercepted_fraction(lai, extinction_coef=0.5) for lai in range(0, 8)]
    assert all(b > a for a, b in zip(fractions, fractions[1:], strict=False))


# --- leaf area index --------------------------------------------------------
def test_leaf_area_index_known_value() -> None:
    # LAI = leaf_carbon · sla_per_mol_c / ground_area = 100 · 0.5 / 2 = 25 (exact).
    assert leaf_area_index(100.0, sla_per_mol_c=0.5, ground_area=2.0) == 25.0


def test_leaf_area_index_zero_carbon_is_zero() -> None:
    assert leaf_area_index(0.0, sla_per_mol_c=0.5, ground_area=2.0) == 0.0


def test_leaf_area_index_scales_inversely_with_ground_area() -> None:
    # Same leaf carbon over twice the ground gives half the LAI (the divisor role).
    small = leaf_area_index(100.0, sla_per_mol_c=0.5, ground_area=1.0)
    large = leaf_area_index(100.0, sla_per_mol_c=0.5, ground_area=2.0)
    assert large == small / 2.0


@pytest.mark.parametrize("bad_area", [0.0, -1.0])
def test_leaf_area_index_rejects_non_positive_ground_area(bad_area: float) -> None:
    with pytest.raises(ValueError, match="ground_area"):
        leaf_area_index(100.0, sla_per_mol_c=0.5, ground_area=bad_area)


# --- the canopy diagnostic composed: carbon -> LAI -> intercepted fraction --
def test_canopy_diagnostic_composes() -> None:
    # The full Step-4 diagnostic chain a Step-5 flow will call: leaf carbon to LAI to
    # the intercepted fraction that scales incident PAR. Closed canopy => near-full
    # interception; an independent literal pins the composition.
    lai = leaf_area_index(200.0, sla_per_mol_c=0.0587204, ground_area=1.0)
    assert math.isclose(lai, 11.74408, rel_tol=1e-5)
    f = intercepted_fraction(lai, extinction_coef=0.6)
    assert math.isclose(f, 0.99911, rel_tol=1e-4)


# --- config.convert: the general boundary unit conversion -------------------
def test_convert_identity() -> None:
    assert convert("22.0 m^2/kg", "m^2/kg") == 22.0


def test_convert_compatible_unit() -> None:
    # 0.0022 ha/kg is the same SLA as 22 m²/kg (the WOFOST-conventional ha/kg unit
    # converts cleanly to the core's m²/kg).
    assert math.isclose(convert("0.0022 ha/kg", "m^2/kg"), 22.0, rel_tol=1e-12)


@pytest.mark.parametrize(
    "value",
    [
        "22.0 kg/m^2",  # inverted dimension
        "22.0 m^2",  # missing the per-mass
        "22.0",  # dimensionless, no unit
        "22.0 wibble/kg",  # undefined unit
        "not a quantity",  # unparseable
    ],
)
def test_convert_rejects_incompatible_or_unparseable(value: str) -> None:
    with pytest.raises(UnitValidationError):
        convert(value, "m^2/kg")


# --- load_canopy_params: committed file -> folded canonical floats ----------
def test_canopy_params_file_exists() -> None:
    assert CANOPY_PARAMS_PATH.is_file()


def test_load_canopy_params_folds_sla_to_per_mol_c() -> None:
    params = load_canopy_params()
    assert isinstance(params, CanopyParams)
    assert params.extinction_coef == 0.6
    # Independent hand-computed fold: SLA 22 m²/kg · M_C 0.012011 kg/mol / f_C 0.45
    # = 0.587204444... m²/mol C. Does not reference the loader's arithmetic.
    assert math.isclose(params.sla_per_mol_c, 0.5872044444444445, rel_tol=1e-12)
    # And the molar mass it folds in is the pinned IUPAC value.
    assert MOLAR_MASS_CARBON_KG_PER_MOL == 0.012011


# --- the gate end-to-end through the real canopy loader ---------------------
def _valid_canopy() -> dict[str, Any]:
    return {
        "name": "winter_wheat",
        "process": "canopy_light_interception",
        "parameters": {
            "extinction_coef": {
                "value": 0.6,
                "unit": "dimensionless",
                "source": "[A]",
            },
            "specific_leaf_area": {
                "value": 22.0,
                "unit": "m^2/kg",
                "source": "[A]",
            },
            "carbon_fraction": {
                "value": 0.45,
                "unit": "dimensionless",
                "source": "[A]",
            },
        },
    }


def _write_canopy(tmp_path: Path, data: dict[str, Any]) -> Path:
    p = tmp_path / "canopy.yaml"
    p.write_text(yaml.safe_dump(data), encoding="utf-8")
    return p


def test_canopy_loader_round_trips_a_valid_file(tmp_path: Path) -> None:
    params = load_canopy_params(_write_canopy(tmp_path, _valid_canopy()))
    assert params.extinction_coef == 0.6
    assert math.isclose(params.sla_per_mol_c, 0.5872044444444445, rel_tol=1e-12)


def test_canopy_loader_converts_a_non_canonical_sla_unit(tmp_path: Path) -> None:
    data = _valid_canopy()
    data["parameters"]["specific_leaf_area"] = {
        "value": 0.0022,
        "unit": "ha/kg",  # == 22 m²/kg after conversion
        "source": "[A]",
    }
    params = load_canopy_params(_write_canopy(tmp_path, data))
    assert math.isclose(params.sla_per_mol_c, 0.5872044444444445, rel_tol=1e-12)


def test_canopy_loader_rejects_a_dimensionally_wrong_sla(tmp_path: Path) -> None:
    data = _valid_canopy()
    data["parameters"]["specific_leaf_area"]["unit"] = "kg/m^2"  # inverted
    with pytest.raises(UnitValidationError):
        load_canopy_params(_write_canopy(tmp_path, data))


@pytest.mark.parametrize("bad_fraction", [0.0, -0.1, 1.5])
def test_canopy_loader_rejects_a_bad_carbon_fraction(
    tmp_path: Path, bad_fraction: float
) -> None:
    data = _valid_canopy()
    data["parameters"]["carbon_fraction"]["value"] = bad_fraction
    with pytest.raises(ValueError, match="carbon_fraction"):
        load_canopy_params(_write_canopy(tmp_path, data))


@pytest.mark.parametrize("bad_k", [0.0, -0.5])
def test_canopy_loader_rejects_a_non_positive_extinction_coef(
    tmp_path: Path, bad_k: float
) -> None:
    data = _valid_canopy()
    data["parameters"]["extinction_coef"]["value"] = bad_k
    with pytest.raises(ValueError, match="extinction_coef"):
        load_canopy_params(_write_canopy(tmp_path, data))


def test_canopy_loader_rejects_an_unknown_field(tmp_path: Path) -> None:
    data = _valid_canopy()
    data["parameters"]["bogus"] = {"value": 1.0, "unit": "dimensionless", "source": "x"}
    with pytest.raises(ValidationError):
        load_canopy_params(_write_canopy(tmp_path, data))


def test_canopy_loader_rejects_a_missing_field(tmp_path: Path) -> None:
    data = _valid_canopy()
    del data["parameters"]["extinction_coef"]
    with pytest.raises(ValidationError):
        load_canopy_params(_write_canopy(tmp_path, data))


def test_canopy_loader_rejects_a_param_missing_its_source(tmp_path: Path) -> None:
    # Clean-room discipline: every value carries a source tag (extra="forbid" plus
    # required fields means an entry without `source` is rejected at the boundary).
    data = _valid_canopy()
    del data["parameters"]["extinction_coef"]["source"]
    with pytest.raises(ValidationError):
        load_canopy_params(_write_canopy(tmp_path, data))
