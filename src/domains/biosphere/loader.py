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
