"""The declarative scenario-file schema (pydantic, the config boundary).

Validates the shape of an authored scenario file into typed specs the interpreter
lowers to frozen engine objects. Step 0 = the **composition subset**: stocks (ICs),
flows (existing frozen types + wiring + a param-set reference), constant forcings,
and the run config (integrator/dt/steps). Every model is ``extra="forbid"`` — a
typo'd key is a schema error, not a silent drop (the ``config`` loader discipline).

**The pyyaml numeric hazard is handled here.** ``yaml.safe_load`` uses the YAML-1.1
float resolver, under which ``1e-3`` (no decimal point) parses as a *string*, not a
float. Every float field below is a pydantic ``float`` (string→float coercion is a
backstop), and scenario files are written with an explicit decimal point
(``1.0e-3``); the interpreter's anchor test additionally asserts the loaded ICs and
forcings equal the frozen reference values, so a silent string-parse fails loudly.
"""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class StockSpec(BaseModel):
    """One stock's declaration: id, namespace, quantity, kind, initial amount.

    ``quantity`` is a :class:`simcore.quantities.Quantity` value ("carbon",
    "water", "nitrogen", "oxygen", "energy"); ``kind`` a
    :class:`simcore.quantities.StockKind` value ("pool", "population",
    "boundary"). ``composition`` (optional) maps quantity values → coeffs for a
    multi-quantity stock (e.g. CO2 = {"carbon": 1, "oxygen": 2}); omitted means the
    frozen 1:1 default. ``unclamped`` / ``extinction_threshold`` mirror the frozen
    ``Stock`` fields (meaningful only for BOUNDARY sources / POPULATION stocks).
    """

    model_config = ConfigDict(extra="forbid")

    id: str
    domain: str
    quantity: str
    kind: str
    amount: float
    composition: dict[str, float] | None = None
    unclamped: bool = False
    extinction_threshold: float = 0.0


class FlowSpec(BaseModel):
    """One flow's declaration: a frozen flow *type* + wiring + optional params.

    ``type`` is a key into :data:`authoring.flow_registry.FLOW_TYPES` (the
    author-selectable frozen-flow surface). ``wiring`` maps the flow constructor's
    stock-id fields → stock ids declared in this scenario. ``params`` names a frozen
    param-set (:data:`authoring.flow_registry.PARAM_LOADERS`) for flow types that
    take a params object; it must be omitted for those that do not. Parameter
    *packs* (inline/override) are Step 1.
    """

    model_config = ConfigDict(extra="forbid")

    id: str
    type: str
    priority: int = 0
    wiring: dict[str, str]
    params: str | None = None


class ForcingSpec(BaseModel):
    """A forcing schedule. Step 0 = constant forcings only (``const``).

    Computed schedules (the Power half-sine, biosphere weather) are a later step;
    the Step-0 composition anchor (crew) uses only constant forced rates.
    """

    model_config = ConfigDict(extra="forbid")

    const: float


class ScenarioSpec(BaseModel):
    """A whole authored scenario: run config + stocks + flows + forcings.

    ``integrator`` is "euler" or "rk4"; ``dt`` the step size (seconds); ``steps``
    the run length; ``rng_seed`` the state seed (default 0). The interpreter builds
    a single-rate, no-reset graph from this (Step 0 scope).
    """

    model_config = ConfigDict(extra="forbid")

    name: str
    integrator: str
    dt: float
    steps: int
    rng_seed: int = 0
    stocks: list[StockSpec]
    flows: list[FlowSpec]
    forcings: dict[str, ForcingSpec] = Field(default_factory=dict)
