"""The author-selectable frozen-flow surface (the thing Step 7 freezes).

A scenario file selects frozen ``Flow`` types by name; this module is the explicit
mapping from an authoring type name → the frozen class + its wiring/param shape.

**Explicit, not introspected — by design.** A stock-id field is a plain ``str``
alias at runtime (``StockId = NewType("StockId", str)``), so field-type
introspection cannot tell a wiring field from any other string field. More to the
point, this registry *is* the authoring contract — the declared set of frozen
primitives a scenario may compose, and the wiring names it exposes for each. It
mirrors frozen constructor signatures; the signatures it mirrors are frozen, so the
"duplication" is a stable, deliberately-curated public surface (frozen in Step 7),
not incidental drift.

Step 0 registers the standalone **Crew** flows (the composition anchor). Later steps
grow this to the rest of the frozen flow set. Parameter *sets* are named separately
(:data:`PARAM_LOADERS`) so "this flow type takes a params object" (a fixed fact of
the class) is decoupled from "which param set" (a per-flow authoring choice); a
flow type either takes params (``takes_params=True``) or does not.
"""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass

from domains.crew.flows import (
    FoodMetabolism,
    OxygenConsumption,
    WaterBalance,
)
from domains.crew.loader import load_crew_params
from simcore.flow import Flow


@dataclass(frozen=True)
class FlowTypeSpec:
    """How one authoring flow-type name lowers to a frozen ``Flow`` constructor.

    ``cls`` is the frozen flow dataclass (typed ``Callable[..., Flow]`` because
    ``Flow`` is a structural ``Protocol``, not a nominal base — each concrete
    dataclass is a callable that returns a ``Flow``-conforming instance);
    ``wiring_fields`` the exact set of constructor keyword fields that take a
    ``StockId`` (the interpreter requires the scenario's ``wiring`` dict to match
    this set exactly); ``takes_params`` whether the constructor takes a ``params``
    object (looked up in :data:`PARAM_LOADERS`). Every frozen flow shares the
    ``(id, priority, *wiring[, params])`` shape.
    """

    cls: Callable[..., Flow]
    wiring_fields: tuple[str, ...]
    takes_params: bool


# The author-selectable frozen-flow surface. Keys are stable authoring type names
# (ASCII, dotted-namespace, matching the flows' own ids by convention). Step 0: the
# three standalone Crew flows.
FLOW_TYPES: dict[str, FlowTypeSpec] = {
    "crew.oxygen_consumption": FlowTypeSpec(
        cls=OxygenConsumption,
        wiring_fields=("o2_store", "o2_consumed"),
        takes_params=False,
    ),
    "crew.food_metabolism": FlowTypeSpec(
        cls=FoodMetabolism,
        wiring_fields=("food_store", "exhaled_co2", "fecal_waste"),
        takes_params=True,
    ),
    "crew.water_balance": FlowTypeSpec(
        cls=WaterBalance,
        wiring_fields=("water_store", "crew_humidity", "urine"),
        takes_params=True,
    ),
}


# Named frozen param sets. A flow type with ``takes_params=True`` references one of
# these by name in its ``params`` field; the interpreter calls the loader to get the
# exact frozen params object (so the param contribution is byte-identical to the
# frozen build). Inline/override parameter *packs* are Step 1.
PARAM_LOADERS: dict[str, Callable[[], object]] = {
    "crew": load_crew_params,
}
