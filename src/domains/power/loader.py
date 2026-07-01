"""Load the Power domain parameters from YAML (the config boundary).

The Power analogue of ``domains.biosphere.loader``: the **only** Power module that
imports the outer config stack (pydantic via ``config``); ``stocks`` and ``flows`` stay
stdlib-pure so the simulation spine runs headless. Same structured ``{value, unit,
source}`` format and exact-string unit guard the biosphere param files use — a generic
structured-param loader stays premature with one Power instance (the ``_DemoSchema`` /
``_CanopySchema`` precedent: bespoke schemas until a second instance justifies sharing).

``params/charge.yaml`` is the single source of truth for the charge coefficient —
``ChargeParams`` carries no inline default (no hardcoded coefficients, per the project's
"parameters are data" invariant).
"""

from __future__ import annotations

from pathlib import Path

from pydantic import BaseModel, ConfigDict

from config import load_yaml
from domains.power.flows import ChargeParams, SelfDischargeParams

# The committed Power param files (P5.2 charge, P5.5 self-discharge).
CHARGE_PARAMS_PATH: Path = Path(__file__).parent / "params" / "charge.yaml"
SELF_DISCHARGE_PARAMS_PATH: Path = (
    Path(__file__).parent / "params" / "self_discharge.yaml"
)

# Expected canonical unit string per Power param (exact-match guard at the boundary).
# Neither a dimensionless efficiency nor a first-order rate is a conserved-Quantity
# canonical unit, so both are schema-validated and exact-string guarded rather than
# routed through pint (the growth_efficiency / decomposition_rate discipline).
_CHARGE_UNITS: dict[str, str] = {
    "charge_efficiency": "dimensionless",
}
_SELF_DISCHARGE_UNITS: dict[str, str] = {
    "self_discharge_rate": "1/s",
}


class _ValueUnitSource(BaseModel):
    """A single ``{value, unit, source}`` parameter entry (the biosphere template).

    Generic across Power params (charge, self-discharge). ``source`` is the required
    clean-room provenance tag (recorded, not parsed); the ``unit`` is exact-string
    validated against the owning loader's expected-unit map.
    """

    model_config = ConfigDict(extra="forbid")

    value: float
    unit: str
    source: str


def _guarded_value(entry: _ValueUnitSource, field: str, units: dict[str, str]) -> float:
    """Read a param entry's value, exact-string guarding its declared unit."""
    expected = units[field]
    if entry.unit != expected:
        raise ValueError(
            f"{field} must be declared in {expected!r}, got {entry.unit!r}"
        )
    return entry.value


class _ChargeParameters(BaseModel):
    model_config = ConfigDict(extra="forbid")

    charge_efficiency: _ValueUnitSource


class _ChargeSchema(BaseModel):
    model_config = ConfigDict(extra="forbid")

    name: str
    process: str
    parameters: _ChargeParameters


def _charge_value(params: _ChargeParameters, field: str) -> float:
    """Read a charge param's value, exact-string guarding its declared unit."""
    return _guarded_value(getattr(params, field), field, _CHARGE_UNITS)


def load_charge_params(path: str | Path = CHARGE_PARAMS_PATH) -> ChargeParams:
    """Load, schema- and bound-check the charge params into ``ChargeParams``.

    ``charge_efficiency`` carries a declared unit (exact-string guarded) and a required
    ``source`` tag (clean-room discipline). It is the **one-way charge** efficiency and
    must lie in ``(0, 1]``: 1 = lossless charging (the heat leg collapses to 0); 0 (a
    battery that stores nothing) and out-of-range values are rejected. Raises
    ``pydantic.ValidationError`` on a schema violation, ``ValueError`` on a bad unit or
    out-of-range value.
    """
    schema = _ChargeSchema.model_validate(load_yaml(path))
    eta = _charge_value(schema.parameters, "charge_efficiency")
    if not (0.0 < eta <= 1.0):
        raise ValueError(f"charge_efficiency must be in (0, 1], got {eta}")
    return ChargeParams(charge_efficiency=eta)


class _SelfDischargeParameters(BaseModel):
    model_config = ConfigDict(extra="forbid")

    self_discharge_rate: _ValueUnitSource


class _SelfDischargeSchema(BaseModel):
    model_config = ConfigDict(extra="forbid")

    name: str
    process: str
    parameters: _SelfDischargeParameters


def load_self_discharge_params(
    path: str | Path = SELF_DISCHARGE_PARAMS_PATH,
) -> SelfDischargeParams:
    """Load, schema- and bound-check the self-discharge param.

    ``self_discharge_rate`` (k, 1/s) carries a declared unit (exact-string guarded) and
    a required ``source`` tag (clean-room discipline). It must be ``>= 0``: 0 is valid
    (an ideal leak-free cell — inert; the herbivory "zero rate is valid" precedent),
    negative (which would *create* energy on the leak) is rejected. Raises
    ``pydantic.ValidationError`` on a schema violation, ``ValueError`` on a bad unit or
    negative value.
    """
    schema = _SelfDischargeSchema.model_validate(load_yaml(path))
    k = _guarded_value(
        schema.parameters.self_discharge_rate,
        "self_discharge_rate",
        _SELF_DISCHARGE_UNITS,
    )
    if k < 0.0:
        raise ValueError(f"self_discharge_rate must be >= 0, got {k}")
    return SelfDischargeParams(self_discharge_rate=k)
