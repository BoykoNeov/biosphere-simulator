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
grow this to the rest of the frozen flow set. A flow type's **param set** (its
``param_set`` name, a key into :data:`PARAM_LOADERS`, or ``None`` for a param-free
flow) is a fixed fact of the class — it names *which frozen loader* produces the
flow's params object. "Which concrete param values" is the per-flow authoring choice
(Step 1): the named default committed file, or a **parameter pack** — a param file in
the same ``{value, unit, source}`` schema that the *same frozen loader* reads (so a
pack's values are validated by the frozen bounds/unit guards, never bypassing them).
"""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path

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
    this set exactly); ``param_set`` names the frozen loader that produces this
    flow's params object (a key into :data:`PARAM_LOADERS`), or ``None`` for a
    param-free flow. Every frozen flow shares the ``(id, priority, *wiring[,
    params])`` shape.
    """

    cls: Callable[..., Flow]
    wiring_fields: tuple[str, ...]
    param_set: str | None


# The author-selectable frozen-flow surface. Keys are stable authoring type names
# (ASCII, dotted-namespace, matching the flows' own ids by convention). Step 0: the
# three standalone Crew flows.
FLOW_TYPES: dict[str, FlowTypeSpec] = {
    "crew.oxygen_consumption": FlowTypeSpec(
        cls=OxygenConsumption,
        wiring_fields=("o2_store", "o2_consumed"),
        param_set=None,
    ),
    "crew.food_metabolism": FlowTypeSpec(
        cls=FoodMetabolism,
        wiring_fields=("food_store", "exhaled_co2", "fecal_waste"),
        param_set="crew",
    ),
    "crew.water_balance": FlowTypeSpec(
        cls=WaterBalance,
        wiring_fields=("water_store", "crew_humidity", "urine"),
        param_set="crew",
    ),
}


# Named frozen param loaders. Each takes an **optional path** (defaulting to the
# committed frozen param file). A flow references its set by name (``params: crew``
# → the loader's default file) or supplies a **pack** (``params: {pack: …}`` → the
# same loader called with the pack's path), so a pack's values flow through the
# frozen loader's schema/bounds/unit validation — a pack is a param file, not a way
# around the guards.
PARAM_LOADERS: dict[str, Callable[..., object]] = {
    "crew": load_crew_params,
}


def load_param_set(param_set: str, pack_path: Path | None) -> object:
    """Load a flow type's params: the committed default, or a pack file.

    ``param_set`` is the flow type's :attr:`FlowTypeSpec.param_set`; ``pack_path``
    is ``None`` for the committed default or an already-resolved path to a pack file
    (a param file in the frozen loader's own ``{value, unit, source}`` schema). The
    pack is read by the *same frozen loader*, so its values are validated identically.
    """
    loader = PARAM_LOADERS[param_set]
    return loader() if pack_path is None else loader(pack_path)
