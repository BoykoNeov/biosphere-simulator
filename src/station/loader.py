"""Load the Station-owned parameters from YAML (the config boundary).

The station analogue of ``domains.*.loader``: the **only** ``station`` module that
imports the outer config stack (pydantic via ``config``); ``system`` / ``cabin`` /
``greenhouse`` / ``water`` / ``flows`` stay stdlib-pure so the simulation spine runs
headless. Same structured ``{value, unit, source}`` format and exact-string unit guard
the Power / Thermal / ECLSS / Crew / biosphere param files use — a generic
structured-param loader stays premature (the bespoke-schema-per-file discipline; a
shared loader waits for enough instances to justify it).

Until Step 4 the station reused sibling params (``CrewRespiration`` took the crew's
``respired_carbon_fraction``); ``station/params/water_recovery.yaml`` was the **first**
station-owned param file, and ``station/params/lamp.yaml`` (Step 5, the grow-lamp photon
efficacy) is the second — each the single source of truth for its flow's coefficients
(``WaterRecoveryParams`` / ``LampParams`` carry no inline defaults, per the "parameters
are data" invariant). The forced crew intake rates, initial cabin inventories, and the
lamp power / photoperiod schedule remain scenario data (``scenario.py``), not params.
"""

from __future__ import annotations

from pathlib import Path

from pydantic import BaseModel, ConfigDict

from config import load_yaml
from domains.biosphere.weather import PAR_UMOL_PER_J
from station.flows import HarvestParams, LampParams, WaterRecoveryParams

# The committed station water-recovery param file (Step 4 crew water loop).
WATER_RECOVERY_PARAMS_PATH: Path = (
    Path(__file__).parent / "params" / "water_recovery.yaml"
)

# The committed station grow-lamp param file (Step 5 Power → biosphere lighting).
LAMP_PARAMS_PATH: Path = Path(__file__).parent / "params" / "lamp.yaml"

# The committed station grain-harvest param file (Step 6 biomass/food loop).
HARVEST_PARAMS_PATH: Path = Path(__file__).parent / "params" / "harvest.yaml"

# Expected canonical unit string per param (exact-match guard at the boundary). Neither
# is a conserved-Quantity canonical unit, so each is schema-validated and exact-string
# guarded, not routed through pint (the charge_efficiency / EclssParams discipline).
_WATER_RECOVERY_UNITS: dict[str, str] = {
    "recovery_rate": "1/s",
    "recovery_efficiency": "dimensionless",
}

# Expected canonical unit string for the lamp param (exact-match guard). Not a
# conserved-Quantity canonical unit, so schema-validated + exact-string guarded (not
# pint).
_LAMP_UNITS: dict[str, str] = {"photon_efficacy": "umol/J"}

# Expected canonical unit string for the harvest param (exact-match guard). Not a
# conserved-Quantity canonical unit, so schema-validated + exact-string guarded (not
# pint).
_HARVEST_UNITS: dict[str, str] = {"harvest_rate": "1/s"}


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


class _LampParameters(BaseModel):
    model_config = ConfigDict(extra="forbid")

    photon_efficacy: _ValueUnitSource


class _LampSchema(BaseModel):
    model_config = ConfigDict(extra="forbid")

    name: str
    process: str
    parameters: _LampParameters


def load_lamp_params(path: str | Path = LAMP_PARAMS_PATH) -> LampParams:
    """Load, schema- and bound-check the grow-lamp param.

    ``photon_efficacy`` carries a declared unit (exact-string guarded, ``umol/J``) and a
    required ``source`` tag (clean-room discipline). Bounds: it must be **strictly
    positive** (a lamp that emits nothing is inert / a wiring mistake) and **at most**
    ``PAR_UMOL_PER_J`` — the physical ceiling at which all electrical input becomes PAR
    photons (η_lamp = 1, the waste-heat leg exactly 0); a value above it would imply an
    over-unity lamp (radiant PAR energy exceeding the input). Raises
    ``pydantic.ValidationError`` on a schema violation, ``ValueError`` on a bad unit or
    out-of-range value.
    """
    schema = _LampSchema.model_validate(load_yaml(path))
    entry = schema.parameters.photon_efficacy
    expected = _LAMP_UNITS["photon_efficacy"]
    if entry.unit != expected:
        raise ValueError(
            f"photon_efficacy must be declared in {expected!r}, got {entry.unit!r}"
        )
    if not (0.0 < entry.value <= PAR_UMOL_PER_J):
        raise ValueError(
            f"photon_efficacy must be in (0, {PAR_UMOL_PER_J}] µmol/J (the physical "
            f"ceiling where all input becomes PAR photons), got {entry.value}"
        )
    return LampParams(photon_efficacy=entry.value)


class _HarvestParameters(BaseModel):
    model_config = ConfigDict(extra="forbid")

    harvest_rate: _ValueUnitSource


class _HarvestSchema(BaseModel):
    model_config = ConfigDict(extra="forbid")

    name: str
    process: str
    parameters: _HarvestParameters


def load_harvest_params(path: str | Path = HARVEST_PARAMS_PATH) -> HarvestParams:
    """Load, schema- and bound-check the grain-harvest param.

    ``harvest_rate`` carries a declared unit (exact-string guarded, ``1/s``) and a
    required ``source`` tag (clean-room discipline). Bound: it must be ``≥ 0`` (a zero
    rate disables harvest — the ``with_harvest=False`` baseline the "it bit" gate
    compares against; the ``k_harvest·dt < 1`` structural bound is a scenario-``dt``
    concern, checked in the run, not here — the ``recovery_rate`` discipline). Raises
    ``pydantic.ValidationError`` on a schema violation, ``ValueError`` on a bad unit or
    out-of-range value.
    """
    schema = _HarvestSchema.model_validate(load_yaml(path))
    entry = schema.parameters.harvest_rate
    expected = _HARVEST_UNITS["harvest_rate"]
    if entry.unit != expected:
        raise ValueError(
            f"harvest_rate must be declared in {expected!r}, got {entry.unit!r}"
        )
    if entry.value < 0.0:
        raise ValueError(f"harvest_rate must be >= 0, got {entry.value}")
    return HarvestParams(harvest_rate=entry.value)
