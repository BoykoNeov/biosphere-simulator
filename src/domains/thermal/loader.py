"""Load the Thermal domain parameters from YAML (the config boundary).

The Thermal analogue of ``domains.power.loader``: the **only** Thermal module that
imports the outer config stack (pydantic via ``config``); ``stocks`` and ``flows`` stay
stdlib-pure so the simulation spine runs headless. Same structured ``{value, unit,
source}`` format and exact-string unit guard the Power / biosphere param files use — a
generic structured-param loader stays premature (the bespoke-schema-per-file discipline;
a shared loader waits for enough instances to justify it).

``params/radiator.yaml`` is the single source of truth for the radiator coefficients —
``ThermalParams`` carries no inline defaults (no hardcoded coefficients, per the
"parameters are data" invariant). The Stefan-Boltzmann constant σ is **not** here — it
is a physics constant in ``flows.py``, not a tunable param.
"""

from __future__ import annotations

from pathlib import Path

from pydantic import BaseModel, ConfigDict

from config import load_yaml
from domains.thermal.flows import ThermalParams

# The committed Thermal param file (Step 5 radiator).
RADIATOR_PARAMS_PATH: Path = Path(__file__).parent / "params" / "radiator.yaml"

# Expected canonical unit string per Thermal param (exact-match guard at the boundary).
# None is a conserved-Quantity canonical unit (those are CARBON=kg, ENERGY=J, ...), so
# each is schema-validated and exact-string guarded rather than routed through pint (the
# charge_efficiency / growth_efficiency discipline). Absolute K (not degC) — the T⁴ law.
_RADIATOR_UNITS: dict[str, str] = {
    "emissivity": "dimensionless",
    "radiator_area": "m^2",
    "heat_capacity": "J/K",
    "space_temperature": "K",
}


class _ValueUnitSource(BaseModel):
    """A single ``{value, unit, source}`` parameter entry (Power/biosphere template).

    ``source`` is the required clean-room provenance tag (recorded, not parsed); the
    ``unit`` is exact-string validated against the loader's expected-unit map.
    """

    model_config = ConfigDict(extra="forbid")

    value: float
    unit: str
    source: str


class _RadiatorParameters(BaseModel):
    model_config = ConfigDict(extra="forbid")

    emissivity: _ValueUnitSource
    radiator_area: _ValueUnitSource
    heat_capacity: _ValueUnitSource
    space_temperature: _ValueUnitSource


class _RadiatorSchema(BaseModel):
    model_config = ConfigDict(extra="forbid")

    name: str
    process: str
    parameters: _RadiatorParameters


def _radiator_value(params: _RadiatorParameters, field: str) -> float:
    """Read a radiator param's value, exact-string guarding its declared unit."""
    entry: _ValueUnitSource = getattr(params, field)
    expected = _RADIATOR_UNITS[field]
    if entry.unit != expected:
        raise ValueError(
            f"{field} must be declared in {expected!r}, got {entry.unit!r}"
        )
    return entry.value


def load_thermal_params(path: str | Path = RADIATOR_PARAMS_PATH) -> ThermalParams:
    """Load, schema- and bound-check the radiator params into ``ThermalParams``.

    Each param carries a declared unit (exact-string guarded) and a required ``source``
    tag (clean-room discipline). Bounds: ``emissivity`` ∈ (0, 1] (1 = black body; 0 = no
    rejection path, rejected); ``radiator_area`` > 0; ``heat_capacity`` > 0;
    ``space_temperature`` >= 0 (a physical absolute temperature). Raises
    ``pydantic.ValidationError`` on a schema violation, ``ValueError`` on a bad unit or
    out-of-range value.
    """
    schema = _RadiatorSchema.model_validate(load_yaml(path))
    emissivity = _radiator_value(schema.parameters, "emissivity")
    radiator_area = _radiator_value(schema.parameters, "radiator_area")
    heat_capacity = _radiator_value(schema.parameters, "heat_capacity")
    space_temperature = _radiator_value(schema.parameters, "space_temperature")
    if not (0.0 < emissivity <= 1.0):
        raise ValueError(f"emissivity must be in (0, 1], got {emissivity}")
    if radiator_area <= 0.0:
        raise ValueError(f"radiator_area must be > 0, got {radiator_area}")
    if heat_capacity <= 0.0:
        raise ValueError(f"heat_capacity must be > 0, got {heat_capacity}")
    if space_temperature < 0.0:
        raise ValueError(f"space_temperature must be >= 0, got {space_temperature}")
    return ThermalParams(
        emissivity=emissivity,
        radiator_area=radiator_area,
        heat_capacity=heat_capacity,
        space_temperature=space_temperature,
    )
