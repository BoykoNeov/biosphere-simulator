"""Step-11 tests: the config boundary — YAML load, unit validation, demo params.

The config layer (``config/`` + the biosphere ``loader``) is the only place that
imports pydantic / pint / yaml; ``simcore`` and the simulation spine stay
stdlib-pure. These gates cover the three responsibilities of that boundary:

  * ``to_canonical`` — a param's declared unit is validated against the quantity's
    canonical unit (decision #9) and converted, or rejected as a ``ConfigError``;
  * ``load_yaml`` — a safe read that wraps IO / parse / shape failures as
    ``ConfigError``;
  * ``load_demo_params`` — the committed ``params/demo.yaml`` loads to the exact
    canonical floats the step-10 demo used (single source of truth), and the full
    schema + unit gate fires end-to-end through the real loader.
"""

from pathlib import Path
from typing import Any

import pytest
import yaml
from pydantic import ValidationError

from config import ConfigError, UnitValidationError, load_yaml, to_canonical
from domains.biosphere.loader import DEMO_PARAMS_PATH, load_demo_params
from simcore.quantities import Quantity


# --- to_canonical: accept (identity + dimensionally-compatible conversion) ---
@pytest.mark.parametrize(
    ("quantity", "value", "expected"),
    [
        (Quantity.CARBON, "1000.0 mol", 1000.0),  # identity (declared in canonical)
        (Quantity.CARBON, "0.0 mol", 0.0),
        (Quantity.CARBON, "1.0 kmol", 1000.0),  # kmol -> mol, exact
        (Quantity.ENERGY, "1.0 J", 1.0),  # identity
        (Quantity.ENERGY, "1.0 kJ", 1000.0),  # kJ -> J, exact
    ],
)
def test_to_canonical_converts_compatible_units(
    quantity: Quantity, value: str, expected: float
) -> None:
    assert to_canonical(quantity, value) == expected


# --- to_canonical: reject (wrong dimension, missing unit, unparseable) -------
@pytest.mark.parametrize(
    ("quantity", "value"),
    [
        (Quantity.CARBON, "1000.0 kg"),  # mass is not amount-of-substance
        (Quantity.CARBON, "1.0 J"),  # energy is not amount-of-substance
        (Quantity.CARBON, "1000.0"),  # dimensionless: no unit at all
        (Quantity.ENERGY, "1.0 mol"),  # amount is not energy
        (Quantity.ENERGY, "1.0 kg"),  # mass is not energy
        (Quantity.CARBON, "1.0 wibblewobble"),  # undefined unit
        (Quantity.CARBON, "not a quantity"),  # unparseable
    ],
)
def test_to_canonical_rejects_incompatible_or_unparseable(
    quantity: Quantity, value: str
) -> None:
    with pytest.raises(UnitValidationError):
        to_canonical(quantity, value)


def test_unit_validation_error_is_a_config_error() -> None:
    # The loader's callers catch ConfigError broadly; the unit error must be caught
    # by that same handler.
    assert issubclass(UnitValidationError, ConfigError)


# --- load_yaml: safe read + shape guard -------------------------------------
def test_load_yaml_reads_a_mapping(tmp_path: Path) -> None:
    p = tmp_path / "ok.yaml"
    p.write_text("a: 1\nb: two\n", encoding="utf-8")
    assert load_yaml(p) == {"a": 1, "b": "two"}


def test_load_yaml_missing_file_raises_config_error(tmp_path: Path) -> None:
    with pytest.raises(ConfigError):
        load_yaml(tmp_path / "does_not_exist.yaml")


def test_load_yaml_non_mapping_raises_config_error(tmp_path: Path) -> None:
    p = tmp_path / "list.yaml"
    p.write_text("- 1\n- 2\n", encoding="utf-8")  # a sequence, not a mapping
    with pytest.raises(ConfigError):
        load_yaml(p)


def test_load_yaml_invalid_yaml_raises_config_error(tmp_path: Path) -> None:
    p = tmp_path / "bad.yaml"
    p.write_text("a: [unterminated\n", encoding="utf-8")
    with pytest.raises(ConfigError):
        load_yaml(p)


# --- load_demo_params: committed params -> exact canonical floats -----------
def test_demo_params_file_exists() -> None:
    assert DEMO_PARAMS_PATH.is_file()


def test_load_demo_params_yields_canonical_floats() -> None:
    params = load_demo_params()
    # Exact (==): the YAML declares amounts already in canonical units, so the
    # conversion is identity and these are bit-identical to the step-10 inline
    # defaults — determinism / the well-fed bound are unchanged.
    assert params.atmospheric_c0 == 1000.0
    assert params.plant_c0 == 100.0
    assert params.outside_c0 == 0.0
    assert params.light == 1.0
    assert params.k_photo == 0.2
    assert params.k_resp == 0.1
    assert params.k_harv == 0.05
    assert params.dt == 0.5


# --- the gate end-to-end through the real loader ----------------------------
def _valid_demo() -> dict[str, Any]:
    return {
        "dt": 0.5,
        "amounts": {
            "atmospheric_c0": "1000.0 mol",
            "plant_c0": "100.0 mol",
            "outside_c0": "0.0 mol",
            "light": "1.0 J",
        },
        "rates": {"k_photo": 0.2, "k_resp": 0.1, "k_harv": 0.05},
    }


def _write_demo(tmp_path: Path, data: dict[str, Any]) -> Path:
    p = tmp_path / "demo.yaml"
    p.write_text(yaml.safe_dump(data), encoding="utf-8")
    return p


def test_loader_round_trips_a_valid_file(tmp_path: Path) -> None:
    params = load_demo_params(_write_demo(tmp_path, _valid_demo()))
    assert params.atmospheric_c0 == 1000.0
    assert params.dt == 0.5


def test_loader_converts_a_non_canonical_amount(tmp_path: Path) -> None:
    data = _valid_demo()
    data["amounts"]["atmospheric_c0"] = "1.0 kmol"  # 1000 mol after conversion
    params = load_demo_params(_write_demo(tmp_path, data))
    assert params.atmospheric_c0 == 1000.0


def test_loader_rejects_a_dimensionally_wrong_amount(tmp_path: Path) -> None:
    data = _valid_demo()
    data["amounts"]["atmospheric_c0"] = "1000.0 kg"  # mass, not amount-of-substance
    with pytest.raises(UnitValidationError):
        load_demo_params(_write_demo(tmp_path, data))


def test_loader_rejects_a_non_positive_rate(tmp_path: Path) -> None:
    data = _valid_demo()
    data["rates"]["k_photo"] = -0.1  # gt=0 in the schema
    with pytest.raises(ValidationError):
        load_demo_params(_write_demo(tmp_path, data))


def test_loader_rejects_a_non_positive_dt(tmp_path: Path) -> None:
    data = _valid_demo()
    data["dt"] = 0.0  # gt=0 in the schema
    with pytest.raises(ValidationError):
        load_demo_params(_write_demo(tmp_path, data))


def test_loader_rejects_an_unknown_field(tmp_path: Path) -> None:
    data = _valid_demo()
    data["bogus"] = 1  # extra="forbid"
    with pytest.raises(ValidationError):
        load_demo_params(_write_demo(tmp_path, data))


def test_loader_rejects_a_missing_field(tmp_path: Path) -> None:
    data = _valid_demo()
    del data["amounts"]["light"]
    with pytest.raises(ValidationError):
        load_demo_params(_write_demo(tmp_path, data))
