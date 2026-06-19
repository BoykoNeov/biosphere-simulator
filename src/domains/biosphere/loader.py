"""Load the biosphere demo parameters from YAML (the config boundary).

This is the **only** biosphere module that imports the outer config stack (pydantic
+ pint via ``config``); ``flows`` and ``demo`` stay stdlib-pure so the simulation
spine runs headless. The loader schema-validates the param file (pydantic) and
unit-validates the conserved-quantity amounts (pint, against the canonical-unit
table), then hands ``build_demo`` a plain ``DemoParams`` in canonical units.

``params/demo.yaml`` is the single source of truth for the demo coefficients —
``DemoParams`` carries no inline defaults (no hardcoded coefficients, per the
project's "parameters are data" invariant).
"""

from __future__ import annotations

from pathlib import Path

from pydantic import BaseModel, ConfigDict, Field

from config import convert, load_yaml, to_canonical
from domains.biosphere.canopy import CanopyParams
from domains.biosphere.demo import DemoParams
from domains.biosphere.photosynthesis import PhotosynthesisParams
from domains.biosphere.respiration import RespirationParams
from simcore.quantities import Quantity

# The committed canonical demo params.
DEMO_PARAMS_PATH: Path = Path(__file__).parent / "params" / "demo.yaml"
# The committed winter-wheat canopy (Beer–Lambert) params (Phase-1 Step 4).
CANOPY_PARAMS_PATH: Path = Path(__file__).parent / "params" / "canopy.yaml"
# The committed winter-wheat FvCB photosynthesis params (Phase-1 Step 5).
PHOTOSYNTHESIS_PARAMS_PATH: Path = (
    Path(__file__).parent / "params" / "photosynthesis.yaml"
)
# The committed winter-wheat respiration params (Phase-1 Step 6).
RESPIRATION_PARAMS_PATH: Path = Path(__file__).parent / "params" / "respiration.yaml"

# --- kg dry-matter <-> mol carbon boundary conversion (Phase-1 Step 1) -------
# Crop biomass is conventionally reported in kg dry matter, but our CARBON currency
# is mol C (golden-locked; see ``simcore.quantities.CANONICAL_UNIT``). The carbon
# fraction of dry matter (kg C per kg DM, crop-specific data, typically ~0.40-0.48)
# bridges the two. This is *crop data*, not generic dimensional analysis: pint
# treats mol ([substance]) and kg ([mass]) as incompatible without a molar mass, so
# this lives here as explicit cited arithmetic rather than a ``to_canonical``
# conversion. Build-ahead infra — no Phase-1 param uses it until allocation (Step 9).
#
# M_C = 12.011 g/mol = 0.012011 kg/mol (IUPAC conventional standard atomic weight of
# carbon, [12.0096, 12.0116]).
MOLAR_MASS_CARBON_KG_PER_MOL: float = 0.012011


def _check_carbon_fraction(carbon_fraction: float) -> None:
    """The carbon fraction is kg C per kg DM; it must lie in ``(0, 1]``."""
    if not (0.0 < carbon_fraction <= 1.0):
        raise ValueError(
            f"carbon_fraction must be in (0, 1] (kg C per kg DM), got {carbon_fraction}"
        )


def dry_matter_kg_to_carbon_mol(mass_kg: float, *, carbon_fraction: float) -> float:
    """Convert kg dry matter to mol carbon: ``mass_kg * f_C / M_C``."""
    _check_carbon_fraction(carbon_fraction)
    return mass_kg * carbon_fraction / MOLAR_MASS_CARBON_KG_PER_MOL


def carbon_mol_to_dry_matter_kg(mol_c: float, *, carbon_fraction: float) -> float:
    """Inverse of :func:`dry_matter_kg_to_carbon_mol`: ``mol_c * M_C / f_C``."""
    _check_carbon_fraction(carbon_fraction)
    return mol_c * MOLAR_MASS_CARBON_KG_PER_MOL / carbon_fraction


# Explicit field -> conserved Quantity map (explicit, not inferred): each amount's
# declared unit is validated against *this* quantity's canonical unit.
_AMOUNT_QUANTITIES: dict[str, Quantity] = {
    "atmospheric_c0": Quantity.CARBON,
    "plant_c0": Quantity.CARBON,
    "outside_c0": Quantity.CARBON,
    "light": Quantity.ENERGY,
}


class _Amounts(BaseModel):
    """Conserved-quantity amounts as unit-bearing strings ("1000.0 mol").

    Typed ``str``, not ``float``: a ``float`` field would reject the unit-bearing
    string before pint runs. The unit is validated/converted in ``to_canonical``.
    """

    model_config = ConfigDict(extra="forbid")

    atmospheric_c0: str
    plant_c0: str
    outside_c0: str
    light: str


class _Rates(BaseModel):
    """Dimensionless model coefficients (Scope A — not unit-validated).

    Strictly positive: a zero/negative first-order rate is meaningless here and
    would break the well-fed bound's sign assumptions.
    """

    model_config = ConfigDict(extra="forbid")

    k_photo: float = Field(gt=0)
    k_resp: float = Field(gt=0)
    k_harv: float = Field(gt=0)


class _DemoSchema(BaseModel):
    model_config = ConfigDict(extra="forbid")

    dt: float = Field(gt=0)
    amounts: _Amounts
    rates: _Rates


def load_demo_params(path: str | Path = DEMO_PARAMS_PATH) -> DemoParams:
    """Load, schema-validate, and unit-validate the demo params into ``DemoParams``.

    Raises ``ConfigError`` (incl. ``UnitValidationError``) on a bad file or unit, and
    ``pydantic.ValidationError`` on a schema violation (missing/extra field,
    non-positive rate/``dt``).
    """
    schema = _DemoSchema.model_validate(load_yaml(path))
    amounts = {
        field: to_canonical(quantity, getattr(schema.amounts, field))
        for field, quantity in _AMOUNT_QUANTITIES.items()
    }
    return DemoParams(
        atmospheric_c0=amounts["atmospheric_c0"],
        plant_c0=amounts["plant_c0"],
        outside_c0=amounts["outside_c0"],
        light=amounts["light"],
        k_photo=schema.rates.k_photo,
        k_resp=schema.rates.k_resp,
        k_harv=schema.rates.k_harv,
        dt=schema.dt,
    )


# --- canopy light interception (Phase-1 Step 4) -----------------------------
# The structured ``value/unit/source`` param format (docs/param-file-conventions.md,
# established at Step 3) — the first real crop param file. The schema is bespoke
# (hand-written, like ``_DemoSchema``): a generic structured-param loader is premature
# with one instance. Specific leaf area carries a per-area unit (m²/kg DM) that is
# *not* a conserved Quantity's canonical unit, so it is validated by ``config.convert``
# (Scope-A boundary discipline, generalized) rather than ``to_canonical``. The
# extinction coefficient and carbon fraction are dimensionless, schema-validated floats.

# The canonical target unit specific_leaf_area is converted into at the boundary.
_SLA_TARGET_UNIT: str = "m^2/kg"


class _CanopyValueUnit(BaseModel):
    """A single ``{value, unit, source}`` parameter entry (the Step-3 template).

    ``source`` is the clean-room provenance tag (required: every value cites its
    origin or carries a ``TODO(cite)`` provisional marker); it is recorded, not
    parsed.
    """

    model_config = ConfigDict(extra="forbid")

    value: float
    unit: str
    source: str


class _CanopyParameters(BaseModel):
    model_config = ConfigDict(extra="forbid")

    extinction_coef: _CanopyValueUnit
    specific_leaf_area: _CanopyValueUnit
    carbon_fraction: _CanopyValueUnit


class _CanopySchema(BaseModel):
    model_config = ConfigDict(extra="forbid")

    name: str
    process: str
    parameters: _CanopyParameters


def load_canopy_params(path: str | Path = CANOPY_PARAMS_PATH) -> CanopyParams:
    """Load, schema-validate, and unit-validate the canopy params into ``CanopyParams``.

    Folds the conventional specific leaf area (m²/kg dry matter) and the carbon
    fraction (kg C/kg DM) into ``sla_per_mol_c`` (m² per mol leaf C) so the pure
    ``canopy`` core never sees kg-DM or the molar-mass constant (the Step-1 lock):

        ``sla_per_mol_c = sla[m²/kg DM] · M_C[kg/mol] / carbon_fraction[kg C/kg DM]``

    (kg DM per mol C is ``M_C / carbon_fraction``). Raises ``ConfigError`` (incl.
    ``UnitValidationError``) on a bad file or unit, ``ValueError`` on an out-of-range
    carbon fraction or non-positive extinction coefficient, and
    ``pydantic.ValidationError`` on a schema violation.
    """
    schema = _CanopySchema.model_validate(load_yaml(path))
    params = schema.parameters

    carbon_fraction = params.carbon_fraction.value
    _check_carbon_fraction(carbon_fraction)

    extinction_coef = params.extinction_coef.value
    if not extinction_coef > 0.0:
        raise ValueError(
            f"extinction_coef must be > 0 (dimensionless), got {extinction_coef}"
        )

    sla_m2_per_kg = convert(
        f"{params.specific_leaf_area.value} {params.specific_leaf_area.unit}",
        _SLA_TARGET_UNIT,
    )
    if not sla_m2_per_kg > 0.0:
        raise ValueError(
            f"specific_leaf_area must be > 0 (m^2/kg), got {sla_m2_per_kg}"
        )
    sla_per_mol_c = sla_m2_per_kg * MOLAR_MASS_CARBON_KG_PER_MOL / carbon_fraction

    return CanopyParams(
        sla_per_mol_c=sla_per_mol_c,
        extinction_coef=extinction_coef,
    )


# --- FvCB photosynthesis (Phase-1 Step 5) -----------------------------------
# Same structured value/unit/source format as canopy. Per config/units.py, per-area
# rates (µmol m⁻² s⁻¹) and mole-fraction concentrations are NOT a conserved Quantity's
# canonical unit, so they are NOT routed through ``to_canonical``/``convert``: they are
# schema-validated, bound-checked floats whose declared ``unit`` is exact-string
# guarded (catches a mis-declared unit without invoking pint on awkward affine-degC /
# mole-fraction units). The full per-leg dimensional check stays deferred (P4). The Ci,
# PAR, temperature, and photoperiod FORCING live in the scenario/resolver, not here.

# Expected canonical unit string per FvCB param (exact-match guard at the boundary).
_PHOTO_UNITS: dict[str, str] = {
    "vcmax": "umol/m^2/s",
    "jmax": "umol/m^2/s",
    "quantum_yield": "mol/mol",
    "theta": "dimensionless",
    "gamma_star": "umol/mol",
    "kc": "umol/mol",
    "ko": "mmol/mol",
    "o2": "mmol/mol",
    "t_min": "degC",
    "t_opt_lo": "degC",
    "t_opt_hi": "degC",
    "t_max": "degC",
}


class _PhotoValueUnit(BaseModel):
    """A single ``{value, unit, source}`` parameter entry (the Step-3 template).

    ``source`` is the required clean-room provenance tag (recorded, not parsed); the
    ``unit`` is exact-string validated against ``_PHOTO_UNITS`` in the loader.
    """

    model_config = ConfigDict(extra="forbid")

    value: float
    unit: str
    source: str


class _PhotoParameters(BaseModel):
    model_config = ConfigDict(extra="forbid")

    vcmax: _PhotoValueUnit
    jmax: _PhotoValueUnit
    quantum_yield: _PhotoValueUnit
    theta: _PhotoValueUnit
    gamma_star: _PhotoValueUnit
    kc: _PhotoValueUnit
    ko: _PhotoValueUnit
    o2: _PhotoValueUnit
    t_min: _PhotoValueUnit
    t_opt_lo: _PhotoValueUnit
    t_opt_hi: _PhotoValueUnit
    t_max: _PhotoValueUnit


class _PhotoSchema(BaseModel):
    model_config = ConfigDict(extra="forbid")

    name: str
    process: str
    parameters: _PhotoParameters


def _photo_value(params: _PhotoParameters, field: str) -> float:
    """Read a FvCB param's value, exact-string guarding its declared unit."""
    entry: _PhotoValueUnit = getattr(params, field)
    expected = _PHOTO_UNITS[field]
    if entry.unit != expected:
        raise ValueError(
            f"{field} must be declared in {expected!r}, got {entry.unit!r}"
        )
    return entry.value


def load_photosynthesis_params(
    path: str | Path = PHOTOSYNTHESIS_PARAMS_PATH,
) -> PhotosynthesisParams:
    """Load, schema- and bound-check the FvCB params into ``PhotosynthesisParams``.

    Each param carries a declared unit (exact-string guarded against the documented
    canonical unit) and a required ``source`` tag (clean-room discipline). Strictly
    positive kinetic constants; ``quantum_yield``/``theta`` ∈ (0, 1]; the cardinal
    temperatures strictly bracket a valid response (``t_min < t_opt_lo ≤ t_opt_hi <
    t_max``) so both ramp denominators are positive. Raises ``pydantic.ValidationError``
    on a schema violation, ``ValueError`` on a bad unit or out-of-range value.
    """
    schema = _PhotoSchema.model_validate(load_yaml(path))
    params = schema.parameters
    values = {field: _photo_value(params, field) for field in _PHOTO_UNITS}

    for field in ("vcmax", "jmax", "gamma_star", "kc", "ko", "o2"):
        if not values[field] > 0.0:
            raise ValueError(f"{field} must be > 0, got {values[field]}")
    for field in ("quantum_yield", "theta"):
        if not (0.0 < values[field] <= 1.0):
            raise ValueError(f"{field} must be in (0, 1], got {values[field]}")

    t_min, t_opt_lo, t_opt_hi, t_max = (
        values["t_min"],
        values["t_opt_lo"],
        values["t_opt_hi"],
        values["t_max"],
    )
    if not (t_min < t_opt_lo <= t_opt_hi < t_max):
        raise ValueError(
            "cardinal temperatures must satisfy t_min < t_opt_lo <= t_opt_hi < t_max, "
            f"got ({t_min}, {t_opt_lo}, {t_opt_hi}, {t_max})"
        )

    return PhotosynthesisParams(
        vcmax=values["vcmax"],
        jmax=values["jmax"],
        quantum_yield=values["quantum_yield"],
        theta=values["theta"],
        gamma_star=values["gamma_star"],
        kc=values["kc"],
        ko=values["ko"],
        o2=values["o2"],
        t_min=t_min,
        t_opt_lo=t_opt_lo,
        t_opt_hi=t_opt_hi,
        t_max=t_max,
    )


# --- maintenance + growth respiration (Phase-1 Step 6) ----------------------
# Same structured value/unit/source format as photosynthesis. Per P4 / config/units.py,
# the per-day relative rate (1/day), the dimensionless Q10 ratio and growth efficiency,
# and the reference temperature are NOT a conserved Quantity's canonical unit, so they
# are schema-validated, bound-checked floats whose declared ``unit`` is exact-string
# guarded (same discipline as FvCB). Air-temperature FORCING lives in the resolver.

# Expected canonical unit string per respiration param (exact-match guard).
_RESP_UNITS: dict[str, str] = {
    "maintenance_coef": "1/day",
    "q10": "dimensionless",
    "t_ref": "degC",
    "growth_efficiency": "dimensionless",
}


class _RespValueUnit(BaseModel):
    """A single ``{value, unit, source}`` parameter entry (the Step-3 template).

    ``source`` is the required clean-room provenance tag (recorded, not parsed); the
    ``unit`` is exact-string validated against ``_RESP_UNITS`` in the loader.
    """

    model_config = ConfigDict(extra="forbid")

    value: float
    unit: str
    source: str


class _RespParameters(BaseModel):
    model_config = ConfigDict(extra="forbid")

    maintenance_coef: _RespValueUnit
    q10: _RespValueUnit
    t_ref: _RespValueUnit
    growth_efficiency: _RespValueUnit


class _RespSchema(BaseModel):
    model_config = ConfigDict(extra="forbid")

    name: str
    process: str
    parameters: _RespParameters


def _resp_value(params: _RespParameters, field: str) -> float:
    """Read a respiration param's value, exact-string guarding its declared unit."""
    entry: _RespValueUnit = getattr(params, field)
    expected = _RESP_UNITS[field]
    if entry.unit != expected:
        raise ValueError(
            f"{field} must be declared in {expected!r}, got {entry.unit!r}"
        )
    return entry.value


def load_respiration_params(
    path: str | Path = RESPIRATION_PARAMS_PATH,
) -> RespirationParams:
    """Load, schema- and bound-check the respiration params into ``RespirationParams``.

    Each param carries a declared unit (exact-string guarded) and a required ``source``
    tag (clean-room discipline). ``maintenance_coef`` and ``q10`` are strictly
    positive; ``growth_efficiency`` ∈ (0, 1] (1 = no conversion loss; 0 would mean all
    assimilate is respired away, never structural). ``t_ref`` is an unconstrained
    reference temperature. Raises ``pydantic.ValidationError`` on a schema violation,
    ``ValueError`` on a bad unit or out-of-range value.
    """
    schema = _RespSchema.model_validate(load_yaml(path))
    params = schema.parameters
    values = {field: _resp_value(params, field) for field in _RESP_UNITS}

    for field in ("maintenance_coef", "q10"):
        if not values[field] > 0.0:
            raise ValueError(f"{field} must be > 0, got {values[field]}")
    if not (0.0 < values["growth_efficiency"] <= 1.0):
        raise ValueError(
            f"growth_efficiency must be in (0, 1], got {values['growth_efficiency']}"
        )

    return RespirationParams(
        maintenance_coef=values["maintenance_coef"],
        q10=values["q10"],
        t_ref=values["t_ref"],
        growth_efficiency=values["growth_efficiency"],
    )
