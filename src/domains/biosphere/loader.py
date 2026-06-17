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

from config import load_yaml, to_canonical
from domains.biosphere.demo import DemoParams
from simcore.quantities import Quantity

# The committed canonical demo params.
DEMO_PARAMS_PATH: Path = Path(__file__).parent / "params" / "demo.yaml"

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
