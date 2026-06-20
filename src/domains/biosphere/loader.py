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
from domains.biosphere.allocation import (
    AllocationParams,
    PartitionRow,
    SenescenceParams,
)
from domains.biosphere.canopy import CanopyParams
from domains.biosphere.decomposition import DecompositionParams
from domains.biosphere.demo import DemoParams
from domains.biosphere.microbial_respiration import MicrobialRespirationParams
from domains.biosphere.nitrogen import NitrogenParams
from domains.biosphere.phenology import PhenologyParams
from domains.biosphere.photosynthesis import PhotosynthesisParams
from domains.biosphere.respiration import RespirationParams
from domains.biosphere.transpiration import TranspirationParams
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
# The committed winter-wheat Penman–Monteith transpiration params (Phase-1 Step 7).
TRANSPIRATION_PARAMS_PATH: Path = (
    Path(__file__).parent / "params" / "transpiration.yaml"
)
# The committed winter-wheat thermal-time phenology params (Phase-1 Step 8).
PHENOLOGY_PARAMS_PATH: Path = Path(__file__).parent / "params" / "phenology.yaml"
# The committed winter-wheat allocation (DVS-keyed partition table) params (Step 9).
ALLOCATION_PARAMS_PATH: Path = Path(__file__).parent / "params" / "allocation.yaml"
# The committed winter-wheat senescence (relative death rate) params (Phase-1 Step 9).
SENESCENCE_PARAMS_PATH: Path = Path(__file__).parent / "params" / "senescence.yaml"
# The committed winter-wheat nitrogen uptake + limitation params (Phase-1 Step 10).
NITROGEN_PARAMS_PATH: Path = Path(__file__).parent / "params" / "nitrogen.yaml"
# The committed chamber litter-decomposition (first-order decay) params (P2 Step 4).
DECOMPOSITION_PARAMS_PATH: Path = (
    Path(__file__).parent / "params" / "decomposition.yaml"
)
# The committed chamber microbial-respiration (first-order rate) params (P2 Step 5).
MICROBIAL_RESPIRATION_PARAMS_PATH: Path = (
    Path(__file__).parent / "params" / "microbial_respiration.yaml"
)

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


# --- Penman–Monteith transpiration (Phase-1 Step 7) -------------------------
# Same structured value/unit/source format as respiration. The aerodynamic and surface
# resistances (s/m) are NOT a conserved Quantity's canonical unit, so they are schema-
# validated, bound-checked floats whose declared ``unit`` is exact-string guarded (same
# discipline as FvCB/respiration). The weather FORCING (Rn, VPD, T) and the soil-water
# stress thresholds live in the scenario/resolver, not this crop param file.

# Expected canonical unit string per transpiration param (exact-match guard).
_TRANSP_UNITS: dict[str, str] = {
    "aerodynamic_resistance": "s/m",
    "surface_resistance": "s/m",
}


class _TranspValueUnit(BaseModel):
    """A single ``{value, unit, source}`` parameter entry (the Step-3 template).

    ``source`` is the required clean-room provenance tag (recorded, not parsed); the
    ``unit`` is exact-string validated against ``_TRANSP_UNITS`` in the loader.
    """

    model_config = ConfigDict(extra="forbid")

    value: float
    unit: str
    source: str


class _TranspParameters(BaseModel):
    model_config = ConfigDict(extra="forbid")

    aerodynamic_resistance: _TranspValueUnit
    surface_resistance: _TranspValueUnit


class _TranspSchema(BaseModel):
    model_config = ConfigDict(extra="forbid")

    name: str
    process: str
    parameters: _TranspParameters


def _transp_value(params: _TranspParameters, field: str) -> float:
    """Read a transpiration param's value, exact-string guarding its declared unit."""
    entry: _TranspValueUnit = getattr(params, field)
    expected = _TRANSP_UNITS[field]
    if entry.unit != expected:
        raise ValueError(
            f"{field} must be declared in {expected!r}, got {entry.unit!r}"
        )
    return entry.value


def load_transpiration_params(
    path: str | Path = TRANSPIRATION_PARAMS_PATH,
) -> TranspirationParams:
    """Load, schema- and bound-check the PM params into ``TranspirationParams``.

    Each param carries a declared unit (exact-string guarded) and a required ``source``
    tag (clean-room discipline). Both resistances are strictly positive (a zero ``r_a``
    would divide by zero in the PM combination equation). Raises
    ``pydantic.ValidationError`` on a schema violation, ``ValueError`` on a bad unit or
    non-positive value.
    """
    schema = _TranspSchema.model_validate(load_yaml(path))
    params = schema.parameters
    values = {field: _transp_value(params, field) for field in _TRANSP_UNITS}

    for field in ("aerodynamic_resistance", "surface_resistance"):
        if not values[field] > 0.0:
            raise ValueError(f"{field} must be > 0, got {values[field]}")

    return TranspirationParams(
        aerodynamic_resistance=values["aerodynamic_resistance"],
        surface_resistance=values["surface_resistance"],
    )


# --- thermal-time phenology (Phase-1 Step 8) --------------------------------
# Same structured value/unit/source format as transpiration. The cardinal temperatures
# (degC) and the thermal-time sums (degC*day) are NOT a conserved Quantity's canonical
# unit, so they are schema-validated, bound-checked floats whose declared ``unit`` is
# exact-string guarded (same discipline as FvCB/respiration/transpiration). NOTE:
# ``"degC*day"`` is deliberately NOT pint-parseable (pint cannot multiply the offset
# unit degC), so it is validated by exact-string equality only — never routed through
# ``config.convert``. Air-temperature FORCING lives in the scenario/resolver, not here.

# Expected canonical unit string per phenology param (exact-match guard).
_PHENOLOGY_UNITS: dict[str, str] = {
    "t_base": "degC",
    "t_cap": "degC",
    "tsum_anthesis": "degC*day",
    "tsum_maturity": "degC*day",
}


class _PhenologyValueUnit(BaseModel):
    """A single ``{value, unit, source}`` parameter entry (the Step-3 template).

    ``source`` is the required clean-room provenance tag (recorded, not parsed); the
    ``unit`` is exact-string validated against ``_PHENOLOGY_UNITS`` in the loader.
    """

    model_config = ConfigDict(extra="forbid")

    value: float
    unit: str
    source: str


class _PhenologyParameters(BaseModel):
    model_config = ConfigDict(extra="forbid")

    t_base: _PhenologyValueUnit
    t_cap: _PhenologyValueUnit
    tsum_anthesis: _PhenologyValueUnit
    tsum_maturity: _PhenologyValueUnit


class _PhenologySchema(BaseModel):
    model_config = ConfigDict(extra="forbid")

    name: str
    process: str
    parameters: _PhenologyParameters


def _phenology_value(params: _PhenologyParameters, field: str) -> float:
    """Read a phenology param's value, exact-string guarding its declared unit."""
    entry: _PhenologyValueUnit = getattr(params, field)
    expected = _PHENOLOGY_UNITS[field]
    if entry.unit != expected:
        raise ValueError(
            f"{field} must be declared in {expected!r}, got {entry.unit!r}"
        )
    return entry.value


def load_phenology_params(
    path: str | Path = PHENOLOGY_PARAMS_PATH,
) -> PhenologyParams:
    """Load, schema- and bound-check the phenology params into ``PhenologyParams``.

    Each param carries a declared unit (exact-string guarded) and a required ``source``
    tag (clean-room discipline). The cardinal temperatures must bracket a valid degree-
    day ramp (``t_base < t_cap``); both thermal sums are strictly positive (they are
    divisors in :func:`~domains.biosphere.phenology.development_stage`). ``t_base`` is
    otherwise sign-unconstrained (winter wheat's development base is ≈ 0 °C). Raises
    ``pydantic.ValidationError`` on a schema violation, ``ValueError`` on a bad unit or
    out-of-range value.
    """
    schema = _PhenologySchema.model_validate(load_yaml(path))
    params = schema.parameters
    values = {field: _phenology_value(params, field) for field in _PHENOLOGY_UNITS}

    if not values["t_base"] < values["t_cap"]:
        raise ValueError(
            "cardinal temperatures must satisfy t_base < t_cap, "
            f"got ({values['t_base']}, {values['t_cap']})"
        )
    for field in ("tsum_anthesis", "tsum_maturity"):
        if not values[field] > 0.0:
            raise ValueError(f"{field} must be > 0, got {values[field]}")

    return PhenologyParams(
        t_base=values["t_base"],
        t_cap=values["t_cap"],
        tsum_anthesis=values["tsum_anthesis"],
        tsum_maturity=values["tsum_maturity"],
    )


# --- leaf/stem/root allocation (Phase-1 Step 9) -----------------------------
# A NEW schema shape: a DVS-keyed partition TABLE (a list of {dvs, fl, fs, fr} rows),
# not the flat value/unit/source scalars the other process files use. The fractions are
# dimensionless (no pint), so the loader's job is the structural discipline that keeps
# the every-step conservation gate from hard-failing: each row's fl+fs+fr == 1 (within
# tol — else the organ legs would not sum to the structural increment), the DVS knots
# strictly increase (so interpolation has a well-defined bracket), and every fraction
# lies in [0, 1]. The single shared-breakpoint table is sum-1 *everywhere* by linearity.

# Tolerance for the per-row fl+fs+fr == 1 check (the fractions are authored decimals;
# this is a typo guard, not a floating-point-accumulation bound).
_PARTITION_SUM_ATOL: float = 1e-9


class _PartitionRowSchema(BaseModel):
    model_config = ConfigDict(extra="forbid")

    dvs: float
    fl: float
    fs: float
    fr: float
    fo: float


class _PartitionTable(BaseModel):
    model_config = ConfigDict(extra="forbid")

    source: str  # the clean-room provenance tag for the partition curve (recorded)
    rows: list[_PartitionRowSchema]


class _AllocationParameters(BaseModel):
    model_config = ConfigDict(extra="forbid")

    partition_table: _PartitionTable


class _AllocationSchema(BaseModel):
    model_config = ConfigDict(extra="forbid")

    name: str
    process: str
    parameters: _AllocationParameters


def load_allocation_params(
    path: str | Path = ALLOCATION_PARAMS_PATH,
) -> AllocationParams:
    """Load, schema- and structurally-check the DVS-keyed partition table.

    Requires at least two rows (interpolation needs a bracket), strictly increasing
    ``dvs`` knots, every fraction in ``[0, 1]``, and each row summing to 1
    (``fl + fs + fr + fo``) within tol — the last is load-bearing: a row that does not
    sum to 1 would make the
    allocation flow's organ legs miss the structural increment and trip the every-step
    conservation gate. ``source`` is required (clean-room discipline). Raises
    ``pydantic.ValidationError`` on a schema violation, ``ValueError`` on a structural
    violation.
    """
    schema = _AllocationSchema.model_validate(load_yaml(path))
    rows = schema.parameters.partition_table.rows
    if len(rows) < 2:
        raise ValueError(f"partition table needs >= 2 rows, got {len(rows)}")
    for prev, row in zip(rows, rows[1:], strict=False):
        if not prev.dvs < row.dvs:
            raise ValueError(
                f"partition table dvs knots must strictly increase, got "
                f"{prev.dvs} then {row.dvs}"
            )
    for row in rows:
        for name, frac in (
            ("fl", row.fl),
            ("fs", row.fs),
            ("fr", row.fr),
            ("fo", row.fo),
        ):
            if not (0.0 <= frac <= 1.0):
                raise ValueError(
                    f"partition fraction {name} must be in [0, 1] at dvs={row.dvs}, "
                    f"got {frac}"
                )
        total = row.fl + row.fs + row.fr + row.fo
        if abs(total - 1.0) > _PARTITION_SUM_ATOL:
            raise ValueError(
                f"partition fractions must sum to 1 at dvs={row.dvs}, got {total}"
            )
    return AllocationParams(
        table=tuple(
            PartitionRow(dvs=r.dvs, fl=r.fl, fs=r.fs, fr=r.fr, fo=r.fo) for r in rows
        )
    )


# --- biomass senescence (Phase-1 Step 9) ------------------------------------
# Same structured value/unit/source format as respiration. The per-organ relative death
# rates (1/day) are NOT a conserved-Quantity canonical unit, so they are schema-
# validated, bound-checked floats whose declared ``unit`` is exact-string guarded (the
# respiration discipline). A zero rate is valid (no turnover); a negative is rejected.

# Expected canonical unit string per senescence param (exact-match guard).
_SENESCENCE_UNITS: dict[str, str] = {
    "rdr_leaf": "1/day",
    "rdr_stem": "1/day",
    "rdr_root": "1/day",
}


class _SenescenceValueUnit(BaseModel):
    """A single ``{value, unit, source}`` parameter entry (the Step-3 template)."""

    model_config = ConfigDict(extra="forbid")

    value: float
    unit: str
    source: str


class _SenescenceParameters(BaseModel):
    model_config = ConfigDict(extra="forbid")

    rdr_leaf: _SenescenceValueUnit
    rdr_stem: _SenescenceValueUnit
    rdr_root: _SenescenceValueUnit


class _SenescenceSchema(BaseModel):
    model_config = ConfigDict(extra="forbid")

    name: str
    process: str
    parameters: _SenescenceParameters


def _senescence_value(params: _SenescenceParameters, field: str) -> float:
    """Read a senescence param's value, exact-string guarding its declared unit."""
    entry: _SenescenceValueUnit = getattr(params, field)
    expected = _SENESCENCE_UNITS[field]
    if entry.unit != expected:
        raise ValueError(
            f"{field} must be declared in {expected!r}, got {entry.unit!r}"
        )
    return entry.value


def load_senescence_params(
    path: str | Path = SENESCENCE_PARAMS_PATH,
) -> SenescenceParams:
    """Load, schema- and bound-check the senescence params into ``SenescenceParams``.

    Each param carries a declared unit (exact-string guarded) and a required ``source``
    tag (clean-room discipline). The relative death rates must be non-negative (0 = no
    turnover of that organ; negative would create carbon). Raises
    ``pydantic.ValidationError`` on a schema violation, ``ValueError`` on a bad unit or
    negative value.
    """
    schema = _SenescenceSchema.model_validate(load_yaml(path))
    params = schema.parameters
    values = {field: _senescence_value(params, field) for field in _SENESCENCE_UNITS}

    for field in _SENESCENCE_UNITS:
        if values[field] < 0.0:
            raise ValueError(f"{field} must be >= 0, got {values[field]}")

    return SenescenceParams(
        rdr_leaf=values["rdr_leaf"],
        rdr_stem=values["rdr_stem"],
        rdr_root=values["rdr_root"],
    )


# --- nitrogen uptake + limitation (Phase-1 Step 10) -------------------------
# Same structured value/unit/source format as respiration. The per-area uptake capacity
# (kg/m^2/day) and the concentration thresholds (kg/kg = kg N/kg DM) are NOT a
# conserved-Quantity canonical unit, so they are schema-validated, bound-checked floats
# whose declared ``unit`` is exact-string guarded (the respiration discipline). The two
# concentration thresholds are folded from kg N/kg DM to kg N/mol C via the carbon
# fraction (the ``sla_per_mol_c`` precedent — the pure core never holds the molar mass),
# so ``carbon_fraction`` lives here too. NOTE: it MUST equal ``canopy.yaml``'s value
# (Step-11 transition checklist item 3) — divergence models a silently inconsistent
# plant; the dedup of the duplicated entry is the Step-4 deferred nicety. The soil-N
# availability thresholds, ``ground_area``, and the N-application FORCING are scenario
# data in the resolver, not this crop param file.

# Expected canonical unit string per nitrogen param (exact-match guard).
_NITROGEN_UNITS: dict[str, str] = {
    "max_uptake_capacity": "kg/m^2/day",
    "n_residual": "kg/kg",
    "n_critical": "kg/kg",
    "carbon_fraction": "dimensionless",
}


class _NitrogenValueUnit(BaseModel):
    """A single ``{value, unit, source}`` parameter entry (the Step-3 template)."""

    model_config = ConfigDict(extra="forbid")

    value: float
    unit: str
    source: str


class _NitrogenParameters(BaseModel):
    model_config = ConfigDict(extra="forbid")

    max_uptake_capacity: _NitrogenValueUnit
    n_residual: _NitrogenValueUnit
    n_critical: _NitrogenValueUnit
    carbon_fraction: _NitrogenValueUnit


class _NitrogenSchema(BaseModel):
    model_config = ConfigDict(extra="forbid")

    name: str
    process: str
    parameters: _NitrogenParameters


def _nitrogen_value(params: _NitrogenParameters, field: str) -> float:
    """Read a nitrogen param's value, exact-string guarding its declared unit."""
    entry: _NitrogenValueUnit = getattr(params, field)
    expected = _NITROGEN_UNITS[field]
    if entry.unit != expected:
        raise ValueError(
            f"{field} must be declared in {expected!r}, got {entry.unit!r}"
        )
    return entry.value


def load_nitrogen_params(
    path: str | Path = NITROGEN_PARAMS_PATH,
) -> NitrogenParams:
    """Load, schema- and bound-check the nitrogen params into ``NitrogenParams``.

    Folds the conventional N-concentration thresholds (kg N/kg DM) to kg N/mol C via the
    carbon fraction (``threshold · M_C / carbon_fraction`` — the ``sla_per_mol_c``
    precedent) so the pure ``nitrogen`` core compares ``plant_n / biomass_c`` against
    plain floats. Each param carries a declared unit (exact-string guarded) and a
    required ``source`` tag (clean-room discipline). ``max_uptake_capacity`` is strictly
    positive; ``carbon_fraction`` ∈ (0, 1]; the residual concentration is non-negative
    and strictly below the critical one (a valid stress band). Raises
    ``pydantic.ValidationError`` on a schema violation, ``ValueError`` on a bad unit or
    out-of-range value.
    """
    schema = _NitrogenSchema.model_validate(load_yaml(path))
    params = schema.parameters
    values = {field: _nitrogen_value(params, field) for field in _NITROGEN_UNITS}

    if not values["max_uptake_capacity"] > 0.0:
        raise ValueError(
            f"max_uptake_capacity must be > 0, got {values['max_uptake_capacity']}"
        )
    carbon_fraction = values["carbon_fraction"]
    _check_carbon_fraction(carbon_fraction)
    n_residual, n_critical = values["n_residual"], values["n_critical"]
    if n_residual < 0.0:
        raise ValueError(f"n_residual must be >= 0 (kg N/kg DM), got {n_residual}")
    if not n_residual < n_critical:
        raise ValueError(
            "N-concentration thresholds must satisfy n_residual < n_critical, "
            f"got ({n_residual}, {n_critical})"
        )

    # kg N/kg DM -> kg N/mol C: × (kg DM per mol C) = × M_C / carbon_fraction.
    fold = MOLAR_MASS_CARBON_KG_PER_MOL / carbon_fraction
    return NitrogenParams(
        max_uptake_capacity=values["max_uptake_capacity"],
        n_residual_per_mol_c=n_residual * fold,
        n_critical_per_mol_c=n_critical * fold,
    )


# --- litter decomposition (Phase-2 Step 4) ----------------------------------
# Same structured value/unit/source format as senescence. The first-order decay rate
# (1/day) is NOT a conserved-Quantity canonical unit, so it is a schema-validated,
# bound-checked float whose declared ``unit`` is exact-string guarded (the senescence /
# respiration discipline). A zero rate is valid (no decomposition); negative rejected.

# Expected canonical unit string per decomposition param (exact-match guard).
_DECOMPOSITION_UNITS: dict[str, str] = {
    "decomposition_rate": "1/day",
}


class _DecompositionValueUnit(BaseModel):
    """A single ``{value, unit, source}`` parameter entry (the Step-3 template)."""

    model_config = ConfigDict(extra="forbid")

    value: float
    unit: str
    source: str


class _DecompositionParameters(BaseModel):
    model_config = ConfigDict(extra="forbid")

    decomposition_rate: _DecompositionValueUnit


class _DecompositionSchema(BaseModel):
    model_config = ConfigDict(extra="forbid")

    name: str
    process: str
    parameters: _DecompositionParameters


def _decomposition_value(params: _DecompositionParameters, field: str) -> float:
    """Read a decomposition param's value, exact-string guarding its declared unit."""
    entry: _DecompositionValueUnit = getattr(params, field)
    expected = _DECOMPOSITION_UNITS[field]
    if entry.unit != expected:
        raise ValueError(
            f"{field} must be declared in {expected!r}, got {entry.unit!r}"
        )
    return entry.value


def load_decomposition_params(
    path: str | Path = DECOMPOSITION_PARAMS_PATH,
) -> DecompositionParams:
    """Load, schema- and bound-check the decomposition params (``DecompositionParams``).

    The first-order decay rate carries a declared unit (exact-string guarded) and a
    required ``source`` tag (clean-room discipline). The rate must be non-negative
    (0 = no decomposition; negative would create carbon). Raises
    ``pydantic.ValidationError`` on a schema violation, ``ValueError`` on a bad unit or
    negative value.
    """
    schema = _DecompositionSchema.model_validate(load_yaml(path))
    params = schema.parameters
    values = {
        field: _decomposition_value(params, field) for field in _DECOMPOSITION_UNITS
    }

    if values["decomposition_rate"] < 0.0:
        raise ValueError(
            f"decomposition_rate must be >= 0, got {values['decomposition_rate']}"
        )

    return DecompositionParams(
        decomposition_rate=values["decomposition_rate"],
    )


# --- microbial respiration (Phase-2 Step 5) ---------------------------------
# Same structured value/unit/source format as decomposition. The first-order microbial
# respiration rate (1/day) is NOT a conserved-Quantity canonical unit, so it is a
# schema-validated, bound-checked float whose declared ``unit`` is exact-string guarded
# (the decomposition / senescence discipline). A zero rate is valid (no respiration —
# microbial biomass only grows, the Step-4 behaviour); negative is rejected.

# Expected canonical unit string per microbial-respiration param (exact-match guard).
_MICROBIAL_RESPIRATION_UNITS: dict[str, str] = {
    "microbial_respiration_rate": "1/day",
}


class _MicrobialRespirationValueUnit(BaseModel):
    """A single ``{value, unit, source}`` parameter entry (the Step-3 template)."""

    model_config = ConfigDict(extra="forbid")

    value: float
    unit: str
    source: str


class _MicrobialRespirationParameters(BaseModel):
    model_config = ConfigDict(extra="forbid")

    microbial_respiration_rate: _MicrobialRespirationValueUnit


class _MicrobialRespirationSchema(BaseModel):
    model_config = ConfigDict(extra="forbid")

    name: str
    process: str
    parameters: _MicrobialRespirationParameters


def _microbial_respiration_value(
    params: _MicrobialRespirationParameters, field: str
) -> float:
    """Read a microbial-respiration param's value, exact-string guarding its unit."""
    entry: _MicrobialRespirationValueUnit = getattr(params, field)
    expected = _MICROBIAL_RESPIRATION_UNITS[field]
    if entry.unit != expected:
        raise ValueError(
            f"{field} must be declared in {expected!r}, got {entry.unit!r}"
        )
    return entry.value


def load_microbial_respiration_params(
    path: str | Path = MICROBIAL_RESPIRATION_PARAMS_PATH,
) -> MicrobialRespirationParams:
    """Load, schema- and bound-check the microbial-respiration params.

    The first-order respiration rate carries a declared unit (exact-string guarded) and
    a required ``source`` tag (clean-room discipline). The rate must be non-negative
    (0 = no respiration; negative would create carbon). Raises
    ``pydantic.ValidationError`` on a schema violation, ``ValueError`` on a bad unit or
    negative value.
    """
    schema = _MicrobialRespirationSchema.model_validate(load_yaml(path))
    params = schema.parameters
    values = {
        field: _microbial_respiration_value(params, field)
        for field in _MICROBIAL_RESPIRATION_UNITS
    }

    if values["microbial_respiration_rate"] < 0.0:
        raise ValueError(
            "microbial_respiration_rate must be >= 0, got "
            f"{values['microbial_respiration_rate']}"
        )

    return MicrobialRespirationParams(
        microbial_respiration_rate=values["microbial_respiration_rate"],
    )
