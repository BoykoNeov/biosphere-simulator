"""Load the Crew domain parameters from YAML (the config boundary).

The Crew analogue of ``domains.power.loader`` / ``domains.eclss.loader``: the **only**
Crew module that imports the outer config stack (pydantic via ``config``); ``stocks``
and ``flows`` stay stdlib-pure so the simulation spine runs headless. Same structured
``{value, unit, source}`` format and exact-string unit guard the Power / Thermal / ECLSS
/ biosphere param files use — a generic structured-param loader stays premature (the
bespoke-schema-per-file discipline; a shared loader waits for enough instances to
justify it).

``params/crew.yaml`` is the single source of truth for the two Crew metabolic-split
fractions — ``CrewParams`` carries no inline defaults (no hardcoded coefficients, per
the "parameters are data" invariant). The forced crew intake rates + initial store
inventories are scenario data (``scenario.py``), not params.
"""

from __future__ import annotations

from pathlib import Path

from pydantic import BaseModel, ConfigDict

from config import load_yaml
from domains.crew.flows import CrewParams

# The committed Crew param file (Step 7 metabolic-split fractions).
CREW_PARAMS_PATH: Path = Path(__file__).parent / "params" / "crew.yaml"

# Expected canonical unit string per Crew param (exact-match guard at the boundary).
# Neither fraction is a conserved-Quantity canonical unit, so each is schema-validated
# and exact-string guarded rather than routed through pint (the charge_efficiency
# discipline).
_CREW_UNITS: dict[str, str] = {
    "respired_carbon_fraction": "dimensionless",
    "insensible_water_fraction": "dimensionless",
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


class _CrewParameters(BaseModel):
    model_config = ConfigDict(extra="forbid")

    respired_carbon_fraction: _ValueUnitSource
    insensible_water_fraction: _ValueUnitSource


class _CrewSchema(BaseModel):
    model_config = ConfigDict(extra="forbid")

    name: str
    process: str
    parameters: _CrewParameters


def _crew_value(params: _CrewParameters, field: str) -> float:
    """Read a crew param's value, exact-string guarding its declared unit."""
    entry: _ValueUnitSource = getattr(params, field)
    expected = _CREW_UNITS[field]
    if entry.unit != expected:
        raise ValueError(
            f"{field} must be declared in {expected!r}, got {entry.unit!r}"
        )
    return entry.value


def load_crew_params(path: str | Path = CREW_PARAMS_PATH) -> CrewParams:
    """Load, schema- and bound-check the Crew params into ``CrewParams``.

    Each fraction carries a declared unit (exact-string guarded) and a required
    ``source`` tag (clean-room discipline). Bounds: both ``respired_carbon_fraction``
    and ``insensible_water_fraction`` must lie in ``[0, 1]`` (a fraction of an ingested
    quantity split across two output fates; endpoints collapse one leg to 0, a valid
    degenerate split). Raises ``pydantic.ValidationError`` on a schema violation,
    ``ValueError`` on a bad unit or out-of-range value.
    """
    schema = _CrewSchema.model_validate(load_yaml(path))
    f_resp = _crew_value(schema.parameters, "respired_carbon_fraction")
    f_ins = _crew_value(schema.parameters, "insensible_water_fraction")
    if not (0.0 <= f_resp <= 1.0):
        raise ValueError(f"respired_carbon_fraction must be in [0, 1], got {f_resp}")
    if not (0.0 <= f_ins <= 1.0):
        raise ValueError(f"insensible_water_fraction must be in [0, 1], got {f_ins}")
    return CrewParams(
        respired_carbon_fraction=f_resp,
        insensible_water_fraction=f_ins,
    )
