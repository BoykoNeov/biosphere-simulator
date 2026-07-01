"""Load the Station-owned parameters from YAML (the config boundary).

The station analogue of ``domains.*.loader``: the **only** ``station`` module that
imports the outer config stack (pydantic via ``config``); ``system`` / ``cabin`` /
``greenhouse`` / ``water`` / ``flows`` stay stdlib-pure so the simulation spine runs
headless. Same structured ``{value, unit, source}`` format and exact-string unit guard
the Power / Thermal / ECLSS / Crew / biosphere param files use — a generic
structured-param loader stays premature (the bespoke-schema-per-file discipline; a
shared loader waits for enough instances to justify it).

Until Step 4 the station reused sibling params (``CrewRespiration`` took the crew's
``respired_carbon_fraction``); ``station/params/water_recovery.yaml`` is the **first**
station-owned param file — the single source of truth for the two water-recovery
coefficients (``WaterRecoveryParams`` carries no inline defaults, per the "parameters
are data" invariant). The forced crew intake rates + initial cabin inventories remain
scenario data (``scenario.py``), not params.
"""

from __future__ import annotations

from pathlib import Path

from pydantic import BaseModel, ConfigDict

from config import load_yaml
from station.flows import WaterRecoveryParams

# The committed station water-recovery param file (Step 4 crew water loop).
WATER_RECOVERY_PARAMS_PATH: Path = (
    Path(__file__).parent / "params" / "water_recovery.yaml"
)

# Expected canonical unit string per param (exact-match guard at the boundary). Neither
# is a conserved-Quantity canonical unit, so each is schema-validated and exact-string
# guarded, not routed through pint (the charge_efficiency / EclssParams discipline).
_WATER_RECOVERY_UNITS: dict[str, str] = {
    "recovery_rate": "1/s",
    "recovery_efficiency": "dimensionless",
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


class _WaterRecoveryParameters(BaseModel):
    model_config = ConfigDict(extra="forbid")

    recovery_rate: _ValueUnitSource
    recovery_efficiency: _ValueUnitSource


class _WaterRecoverySchema(BaseModel):
    model_config = ConfigDict(extra="forbid")

    name: str
    process: str
    parameters: _WaterRecoveryParameters


def _wr_value(params: _WaterRecoveryParameters, field: str) -> float:
    """Read a water-recovery param's value, exact-string guarding its declared unit."""
    entry: _ValueUnitSource = getattr(params, field)
    expected = _WATER_RECOVERY_UNITS[field]
    if entry.unit != expected:
        raise ValueError(
            f"{field} must be declared in {expected!r}, got {entry.unit!r}"
        )
    return entry.value


def load_water_recovery_params(
    path: str | Path = WATER_RECOVERY_PARAMS_PATH,
) -> WaterRecoveryParams:
    """Load, schema- and bound-check the water-recovery params.

    Each param carries a declared unit (exact-string guarded) and a required ``source``
    tag (clean-room discipline). Bounds: ``recovery_rate`` (k_rec) must be ``≥ 0`` (a
    zero rate disables recovery — no draw of the buffer; the ``k_rec·dt < 1`` structural
    bound is a scenario-``dt`` concern, checked in the run, not here);
    ``recovery_efficiency`` (η_w) must lie in ``[0, 1]`` (a fraction split across two
    fates; η_w = 0 is the open-loop baseline the "it bit" gate compares against, η_w = 1
    perfect closure). Raises ``pydantic.ValidationError`` on a schema violation,
    ``ValueError`` on a bad unit or out-of-range value.
    """
    schema = _WaterRecoverySchema.model_validate(load_yaml(path))
    recovery_rate = _wr_value(schema.parameters, "recovery_rate")
    recovery_efficiency = _wr_value(schema.parameters, "recovery_efficiency")
    if recovery_rate < 0.0:
        raise ValueError(f"recovery_rate must be >= 0, got {recovery_rate}")
    if not (0.0 <= recovery_efficiency <= 1.0):
        raise ValueError(
            f"recovery_efficiency must be in [0, 1], got {recovery_efficiency}"
        )
    return WaterRecoveryParams(
        recovery_rate=recovery_rate,
        recovery_efficiency=recovery_efficiency,
    )
