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

from pydantic import BaseModel, ConfigDict, Field, model_validator


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
    amount: float | str
    """The initial amount: a literal float, or a **template expression** (Step 3).

    A string is a bounded-grammar expression over the scenario's declared
    ``parameters`` (e.g. ``"param('crew_count') * 1000.0"``), evaluated to a literal at
    interpret time (:func:`authoring.template.eval_numeric_field`). A bare float is the
    Step-0 form, unchanged — all-literal scenarios still lower identically.
    """
    composition: dict[str, float] | None = None
    unclamped: bool = False
    extinction_threshold: float = 0.0


class ParamPackRef(BaseModel):
    """A reference to a **parameter pack** — an alternate param file (Step 1).

    ``pack`` is a path to a param file in the frozen loader's own
    ``{value, unit, source}`` schema, resolved **relative to the scenario file's
    directory** (so a modder-shipped scenario+pack bundle relocates intact). The
    *same* frozen loader reads it, so a pack's values pass the frozen schema/bounds/
    unit validation — a pack is a param file, not a way around the guards. The
    "cultivar = param pack" species primitive: an author overrides a value (e.g. a
    metabolic fraction) by shipping a full pack file with the changed value.
    """

    model_config = ConfigDict(extra="forbid")

    pack: str


class KineticsSpec(BaseModel):
    """An **authored kinetics** flow: a rate expression × a stoichiometry (Step 2).

    The declarative alternative to selecting a frozen flow ``type``. ``rate`` is a
    bounded-grammar expression string (see
    :func:`authoring.expr_parser.parse_rate_expr`) giving the **instantaneous** rate —
    ``dt``-independent (the grammar has no ``dt``
    token, so RK4-order-safety is structural). ``stoichiometry`` maps each touched
    stock id → its (integer/rational) coefficient; the flow emits ``coeff · rate · dt``
    per leg, so it is **balanced by construction** for any rate value provided the
    coefficient vector balances per quantity — which the interpreter verifies against
    the stock compositions at build time (decision C).

    **Authored ≠ validated** (decision B): conservation + determinism are guaranteed,
    scientific validity is the author's responsibility — an authored-kinetics run
    carries no calibration claim, no golden, no manifest entry.
    """

    model_config = ConfigDict(extra="forbid")

    rate: str
    stoichiometry: dict[str, float]


class FlowSpec(BaseModel):
    """One flow: either a frozen flow *type* + wiring, or authored *kinetics*.

    Exactly one of ``type`` (a key into :data:`authoring.flow_registry.FLOW_TYPES`, the
    author-selectable frozen-flow surface — with ``wiring`` mapping the constructor's
    stock-id fields → declared stock ids) **xor** ``kinetics`` (an authored
    rate×stoichiometry flow, Step 2) must be given; the model validator enforces the
    exclusivity and that a kinetics flow carries no ``wiring`` (its stoichiometry names
    stocks directly). ``params`` selects the params object: a **string** names a
    frozen param set (a frozen flow type's default set — the Step-0 form; or, for a
    kinetics flow, a :data:`authoring.flow_registry.PARAM_LOADERS` key whose loaded
    values the rate's ``param("…")`` reads, so authored params still pass the frozen
    bounds/unit guards), or a :class:`ParamPackRef` supplies a parameter pack (Step 1).
    It must be omitted for a param-free flow. Partial-merge/bundling packs are deferred.
    """

    model_config = ConfigDict(extra="forbid")

    id: str
    type: str | None = None
    priority: int = 0
    wiring: dict[str, str] = Field(default_factory=dict)
    kinetics: KineticsSpec | None = None
    params: str | ParamPackRef | None = None

    @model_validator(mode="after")
    def _check_type_xor_kinetics(self) -> FlowSpec:
        if (self.type is None) == (self.kinetics is None):
            raise ValueError(
                f"flow {self.id!r}: give exactly one of 'type' (a frozen flow type) "
                f"or 'kinetics' (an authored rate×stoichiometry flow)"
            )
        if self.kinetics is not None and self.wiring:
            raise ValueError(
                f"flow {self.id!r}: an authored 'kinetics' flow takes no 'wiring' "
                f"(its stoichiometry names stocks directly)"
            )
        return self


class ForcingSpec(BaseModel):
    """A forcing schedule. Step 0 = constant forcings only (``const``).

    Computed schedules (the Power half-sine, biosphere weather) are a later step;
    the Step-0 composition anchor (crew) uses only constant forced rates. ``const``
    is a literal float or a **template expression** over the declared ``parameters``
    (Step 3), evaluated to a literal at interpret time (like a stock ``amount``).
    """

    model_config = ConfigDict(extra="forbid")

    const: float | str


class BundleSpec(BaseModel):
    """A reusable **domain / species bundle**: stocks + flows + forcings (Step 6).

    A *domain* is a named stock+flow bundle over a quantity set; a *species* is a
    flow-set + param-set + stock-template (a bundle whose stock ``amount``s are
    template expressions over its own ``parameters`` — e.g. the crew "species"
    parametrized by ``crew_count``). A scenario **includes** one or more bundle files
    (:attr:`ScenarioSpec.includes`); the interpreter merges each bundle's parameters/
    stocks/flows/forcings into the scenario's flat graph
    (:func:`authoring.compose.apply_includes`).

    A bundle carries **no run config** (integrator/dt/steps/name/rng_seed) and **no
    nested** ``includes`` — both are rejected by ``extra="forbid"`` (run config lives
    only in the top-level scenario; includes are flat, one level deep — Step 6 scope).
    """

    model_config = ConfigDict(extra="forbid")

    parameters: dict[str, float] = Field(default_factory=dict)
    stocks: list[StockSpec] = Field(default_factory=list)
    flows: list[FlowSpec] = Field(default_factory=list)
    forcings: dict[str, ForcingSpec] = Field(default_factory=dict)


class ScenarioSpec(BaseModel):
    """A whole authored scenario: run config + parameters + stocks + flows + forcings.

    ``integrator`` is "euler" or "rk4"; ``dt`` the step size (seconds); ``steps``
    the run length; ``rng_seed`` the state seed (default 0). The interpreter builds
    a single-rate, no-reset graph from this (Step 0 scope).

    ``parameters`` (Step 3) is the **template contract**: named scalars with default
    values that an instantiation may override (``interpret(..., overrides=…)``), and
    that a stock ``amount`` / forcing ``const`` expression may read via ``param('…')``.
    Empty (the Step-0 form) means a fully-literal scenario with no knobs.

    ``includes`` (Step 6) is a list of **bundle-file paths** (each a
    :class:`BundleSpec`), resolved relative to the scenario file's directory. Each
    included bundle's parameters/stocks/flows/forcings merge into this scenario's
    (:func:`authoring.compose.apply_includes`, run at the top of ``interpret``): a
    scenario is thus *composed* from reusable domain/species bundles + its own inline
    declarations. A duplicate id/key/parameter across any two sources is an
    ``AuthoringError`` (no silent override). Empty (the pre-Step-6 form) means a
    self-contained scenario.
    """

    model_config = ConfigDict(extra="forbid")

    name: str
    integrator: str
    dt: float
    steps: int
    rng_seed: int = 0
    includes: list[str] = Field(default_factory=list)
    parameters: dict[str, float] = Field(default_factory=dict)
    stocks: list[StockSpec] = Field(default_factory=list)
    flows: list[FlowSpec] = Field(default_factory=list)
    forcings: dict[str, ForcingSpec] = Field(default_factory=dict)
