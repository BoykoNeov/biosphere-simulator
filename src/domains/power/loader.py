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
from domains.power.flows import ChargeParams

# The committed Power charge-efficiency params (P5.2).
CHARGE_PARAMS_PATH: Path = Path(__file__).parent / "params" / "charge.yaml"

# Expected canonical unit string per charge param (exact-match guard at the boundary).
# A dimensionless efficiency is not a conserved-Quantity canonical unit, so it is
# schema-validated and exact-string guarded rather than routed through pint
# (the growth_efficiency / decomposition_rate discipline).
_CHARGE_UNITS: dict[str, str] = {
    "charge_efficiency": "dimensionless",
}


class _ChargeValueUnit(BaseModel):
    """A single ``{value, unit, source}`` parameter entry (the biosphere template).

    ``source`` is the required clean-room provenance tag (recorded, not parsed); the
    ``unit`` is exact-string validated against ``_CHARGE_UNITS`` in the loader.
    """

    model_config = ConfigDict(extra="forbid")

    value: float
    unit: str
    source: str


class _ChargeParameters(BaseModel):
    model_config = ConfigDict(extra="forbid")

    charge_efficiency: _ChargeValueUnit


class _ChargeSchema(BaseModel):
    model_config = ConfigDict(extra="forbid")

    name: str
    process: str
    parameters: _ChargeParameters


def _charge_value(params: _ChargeParameters, field: str) -> float:
    """Read a charge param's value, exact-string guarding its declared unit."""
    entry: _ChargeValueUnit = getattr(params, field)
    expected = _CHARGE_UNITS[field]
    if entry.unit != expected:
        raise ValueError(
            f"{field} must be declared in {expected!r}, got {entry.unit!r}"
        )
    return entry.value


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
