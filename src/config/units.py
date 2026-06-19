"""Unit validation at the config boundary (decision #9).

The pure core (``simcore``) stores plain floats plus a canonical-unit *label* and
never imports pint. This module is the boundary that validates an incoming param's
declared unit against the quantity's canonical unit and converts to it, handing the
core a plain float in the canonical unit.

**Scope A** (see the Step-11 design in ``docs/plans/phase-0-engine-skeleton.md``):
only conserved-quantity *amounts* are unit-validated here. Rate coefficients are
dimensionless model parameters validated by the schema; full rate-law dimensional
closure is a Phase-1 concern (it would require per-leg dimensional signatures on the
about-to-be-frozen ``Flow`` protocol).

This validates **absolute** amounts only (e.g. a soil-water stock in ``kg``).
**Per-area** physiological rate params (mm day⁻¹, µmol m⁻² s⁻¹) are NOT routed here
— they are dimensionally incompatible with the absolute canonical unit (length/area
vs mass) and are instead schema-validated floats multiplied by a scenario
``ground_area`` (m²) inside the flow's ``evaluate``. See the Phase-1 Step-1 design
(``docs/plans/phase-1-single-producer.md``) for that absolute-vs-per-area split.
"""

from __future__ import annotations

import math

import pint

from config.errors import UnitValidationError
from simcore.quantities import Quantity, canonical_unit

# One module-level registry. pint quantities must not cross registries; we use
# exactly one, built once at import.
_UREG = pint.UnitRegistry()


def convert(value: str, target_unit: str) -> float:
    """Validate a unit-bearing param string and convert it to an explicit unit.

    The general boundary conversion: ``value`` is a ``"<magnitude> <unit>"`` string
    (e.g. ``"0.0022 ha/kg"``) and ``target_unit`` is a pint-parseable unit
    expression (e.g. ``"m^2/kg"``). The value must parse and be dimensionally
    compatible with ``target_unit``; its magnitude is converted into that unit and
    returned as a plain float.

    This is the same Scope-A boundary discipline as :func:`to_canonical`, but for
    params whose unit is **not** a conserved ``Quantity``'s canonical unit — e.g.
    specific leaf area in m²/kg (Phase-1 Step 4). It is **not** the deferred per-leg
    ``Flow`` dimensional check (P4): it validates one declared param against one
    declared target unit, not a rate law's full dimensional signature.

    Note on notation: pint reads ``"m^2/kg"`` / ``"m**2/kg"`` but **not** ``"m2
    kg-1"`` (it parses ``kg-1`` as ``kg minus 1``). Param files use the ``^``/``/``
    form (see ``docs/param-file-conventions.md``).

    Raises ``UnitValidationError`` if ``value`` is unparseable, carries no unit or an
    incompatible one, or converts to a non-finite magnitude.
    """
    try:
        parsed = _UREG.Quantity(value)
    except Exception as exc:  # pint raises several error types on malformed input
        raise UnitValidationError(
            f"param {value!r} is not a parseable quantity"
        ) from exc
    try:
        converted = parsed.to(target_unit)
    except pint.DimensionalityError as exc:
        raise UnitValidationError(
            f"param {value!r} is dimensionally incompatible with target unit "
            f"{target_unit!r}"
        ) from exc
    magnitude = float(converted.magnitude)
    if not math.isfinite(magnitude):
        raise UnitValidationError(f"param {value!r} converts to a non-finite magnitude")
    return magnitude


def to_canonical(quantity: Quantity, value: str) -> float:
    """Validate and convert a unit-bearing param string to a canonical-unit float.

    ``value`` is a ``"<magnitude> <unit>"`` string (e.g. ``"1000.0 mol"``). It must
    parse and be dimensionally compatible with ``quantity``'s canonical unit; the
    magnitude is then converted into that unit and returned as a plain float.

    Raises ``UnitValidationError`` if ``value`` is unparseable, carries no unit or an
    incompatible one, or converts to a non-finite magnitude.
    """
    target = canonical_unit(quantity)
    try:
        parsed = _UREG.Quantity(value)
    except Exception as exc:  # pint raises several error types on malformed input
        raise UnitValidationError(
            f"{quantity.value} param {value!r} is not a parseable quantity"
        ) from exc
    try:
        converted = parsed.to(target)
    except pint.DimensionalityError as exc:
        raise UnitValidationError(
            f"{quantity.value} param {value!r} is dimensionally incompatible with "
            f"its canonical unit {target!r}"
        ) from exc
    magnitude = float(converted.magnitude)
    if not math.isfinite(magnitude):
        raise UnitValidationError(
            f"{quantity.value} param {value!r} converts to a non-finite magnitude"
        )
    return magnitude
