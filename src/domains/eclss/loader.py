"""Load the ECLSS domain parameters from YAML (the config boundary).

The ECLSS analogue of ``domains.power.loader`` / ``domains.thermal.loader``: the
**only** ECLSS module that imports the outer config stack (pydantic via ``config``);
``stocks`` and ``flows`` stay stdlib-pure so the simulation spine runs headless. Same
structured ``{value, unit, source}`` format and exact-string unit guard the Power /
Thermal / biosphere param files use — a generic structured-param loader stays premature
(the bespoke-schema-per-file discipline; a shared loader waits for enough instances to
justify it).

``params/eclss.yaml`` is the single source of truth for the four ECLSS control-loop
coefficients — ``EclssParams`` carries no inline defaults (no hardcoded coefficients,
per the "parameters are data" invariant). The forced crew metabolic rates + initial
cabin inventories are scenario data (``scenario.py``), not params.
"""

from __future__ import annotations

from pathlib import Path

from pydantic import BaseModel, ConfigDict

from config import load_yaml
from domains.eclss.flows import EclssParams

# The committed ECLSS param file (Step 6 cabin-air control loops).
ECLSS_PARAMS_PATH: Path = Path(__file__).parent / "params" / "eclss.yaml"

# Expected canonical unit string per ECLSS param (exact-match guard at the boundary).
# None is a conserved-Quantity canonical unit (those are OXYGEN=mol, CARBON=mol,
# WATER=kg, ...), so each is schema-validated and exact-string guarded rather than
# routed through pint (the charge_efficiency / emissivity discipline). The three rates
# are 1/s; the setpoint is a cabin O₂ inventory in mol (the OXYGEN canonical unit, but
# here it is a control setpoint, not a stock amount — still exact-string guarded like
# the rest).
_ECLSS_UNITS: dict[str, str] = {
    "co2_scrub_rate": "1/s",
    "condense_rate": "1/s",
    "o2_makeup_gain": "1/s",
    "o2_setpoint": "mol",
}


class _ValueUnitSource(BaseModel):
    """A single ``{value, unit, source}`` parameter entry (the shared template).

    ``source`` is the required clean-room provenance tag (recorded, not parsed); the
    ``unit`` is exact-string validated against the loader's expected-unit map.
    """

    model_config = ConfigDict(extra="forbid")

    value: float
    unit: str
    source: str


class _EclssParameters(BaseModel):
    model_config = ConfigDict(extra="forbid")

    co2_scrub_rate: _ValueUnitSource
    condense_rate: _ValueUnitSource
    o2_makeup_gain: _ValueUnitSource
    o2_setpoint: _ValueUnitSource


class _EclssSchema(BaseModel):
    model_config = ConfigDict(extra="forbid")

    name: str
    process: str
    parameters: _EclssParameters


def _eclss_value(params: _EclssParameters, field: str) -> float:
    """Read an ECLSS param's value, exact-string guarding its declared unit."""
    entry: _ValueUnitSource = getattr(params, field)
    expected = _ECLSS_UNITS[field]
    if entry.unit != expected:
        raise ValueError(
            f"{field} must be declared in {expected!r}, got {entry.unit!r}"
        )
    return entry.value


def load_eclss_params(path: str | Path = ECLSS_PARAMS_PATH) -> EclssParams:
    """Load, schema- and bound-check the ECLSS params into ``EclssParams``.

    Each param carries a declared unit (exact-string guarded) and a required ``source``
    tag (clean-room discipline). Bounds: the three rates ``co2_scrub_rate`` /
    ``condense_rate`` / ``o2_makeup_gain`` must be ``> 0`` (a zero rate disables its
    control loop — no removal / no restoring force, which the standalone validation
    needs); ``o2_setpoint`` must be ``> 0`` (a positive target inventory). Raises
    ``pydantic.ValidationError`` on a schema violation, ``ValueError`` on a bad unit or
    out-of-range value.
    """
    schema = _EclssSchema.model_validate(load_yaml(path))
    co2_scrub_rate = _eclss_value(schema.parameters, "co2_scrub_rate")
    condense_rate = _eclss_value(schema.parameters, "condense_rate")
    o2_makeup_gain = _eclss_value(schema.parameters, "o2_makeup_gain")
    o2_setpoint = _eclss_value(schema.parameters, "o2_setpoint")
    if co2_scrub_rate <= 0.0:
        raise ValueError(f"co2_scrub_rate must be > 0, got {co2_scrub_rate}")
    if condense_rate <= 0.0:
        raise ValueError(f"condense_rate must be > 0, got {condense_rate}")
    if o2_makeup_gain <= 0.0:
        raise ValueError(f"o2_makeup_gain must be > 0, got {o2_makeup_gain}")
    if o2_setpoint <= 0.0:
        raise ValueError(f"o2_setpoint must be > 0, got {o2_setpoint}")
    return EclssParams(
        co2_scrub_rate=co2_scrub_rate,
        condense_rate=condense_rate,
        o2_makeup_gain=o2_makeup_gain,
        o2_setpoint=o2_setpoint,
    )
