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
    rate_class: str = "fast"
    """The **rate class**: which side of the multi-rate partition this flow steps on.

    ``"fast"`` (the default) sub-steps ``n_sub`` times at ``dt/n_sub`` *inside* each
    master step; ``"slow"`` steps once at the master ``dt`` (as two ``dt/2`` halves
    under Strang splitting). The legal values are validated by the interpreter, not here
    (:data:`authoring.interpreter._RATE_CLASSES`) — the ``integrator``/``quantity``/
    ``kind`` discipline, where an unknown value is an ``AuthoringError`` rather than a
    pydantic error.

    **Spelled ``rate_class``, not ``rate``, deliberately.** ``KineticsSpec.rate`` — one
    nesting level down, on the same flow — is the *rate law* (how fast the flow runs);
    this is the *rate class* (which cadence it is integrated on). A flow may carry both,
    so a bare ``rate:`` here would put two senses of the word in one block. The name
    also matches what the concept is called everywhere else it appears:
    :data:`authoring.interpreter._RATE_CLASSES` and the manifest's ``rate_classes``.

    **This is a *property*, not a *reference* — which is exactly why it lives on the
    flow.** The rate class travels *with* the flow, inside a bundle, and ``compose``'s
    ``{bundle, prefix}`` rewrite cannot touch it, because prefixing rewrites **ids**.
    The rejected alternative — a top-level ``fast: [flow-id, …]`` list — would have been
    a list of id *references*, i.e. a new rewrite surface in ``compose.apply_includes``
    that silently mis-fires the moment someone forgets it. This shape has zero
    referential-integrity surface.

    **Defaulting to ``"fast"`` is load-bearing, not a convenience.** ``rate_class``
    defaulting to fast + ``n_sub`` defaulting to 1 ⇒ an **empty slow set** ⇒ a file with
    no multi-rate keys lowers to today's single-rate trajectory *by construction* (the
    identity path, measured in ``tests/test_authoring_multirate_identity.py``).
    Defaulting to ``"slow"`` could not do this: with an empty fast set, Strang would run
    two ``dt/2`` half-steps, which is **not** one full Euler step.
    """

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


class IncludeSpec(BaseModel):
    """A **prefixed** bundle include — a bundle instance under a namespace (Step 6c).

    The bare-string include form (``- bundles/battery.domain.yaml``) merges a bundle's
    ids verbatim; a mapping form (``{bundle: <path>, prefix: <name>}``) **namespaces**
    every id the bundle declares — each stock id, flow id and forcing key becomes
    ``<prefix>.<id>``, and every reference to them (``wiring`` values, ``stoichiometry``
    keys, and the ``stock(...)``/``forcing(...)`` refs inside a ``kinetics`` rate) is
    rewritten to match (:func:`authoring.compose.apply_includes`). This is what lets the
    **same** bundle be included more than once without the id collision a bare double
    include hits (:func:`authoring.compose` documents the surface).

    **Only disjoint / kinetics-style bundles are multi-instanceable.** A frozen flow
    that binds a forcing by a *hardcoded* name (the crew flows read ``crew_o2_intake``
    etc. from a frozen module constant, not through wiring) cannot find a namespaced
    forcing key — a prefixed crew include would raise a resolve-time ``KeyError``.
    Prefixing a forcing-bound frozen bundle is therefore an unsupported (documented)
    case, the crew analogue of the greenhouse's hardcoded ``CARBON_POOL`` blocker.
    Bundle **parameter** namespacing is likewise deferred (the only param-bearing
    bundle, crew, is blocked for the forcing reason anyway) — two prefixed instances of
    a param-bearing bundle collide on the parameter name, the honest boundary.
    """

    model_config = ConfigDict(extra="forbid")

    bundle: str
    prefix: str = Field(min_length=1)


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
    the run length; ``rng_seed`` the state seed (default 0). The interpreter builds a
    no-reset graph from this (annual-reset/phenology remains out of scope).

    ``dt`` + ``n_sub`` + each flow's ``rate_class`` are the **multi-rate** knob
    (post-roadmap): ``dt`` is the **coupling cadence** at which a step commits and peers
    read; the fast flow set sub-steps internally at ``dt/n_sub``. Omitting both keys is
    the single-rate path — an empty slow set at ``n_sub = 1`` — which reproduces the
    pre-multi-rate trajectory bit-for-bit. The one refused combination is ``n_sub = 1``
    *with* a non-empty slow set: it buys no rate separation and no performance win, yet
    silently moves the answer (see :func:`authoring.interpreter.interpret`).

    ``parameters`` (Step 3) is the **template contract**: named scalars with default
    values that an instantiation may override (``interpret(..., overrides=…)``), and
    that a stock ``amount`` / forcing ``const`` expression may read via ``param('…')``.
    Empty (the Step-0 form) means a fully-literal scenario with no knobs.

    ``includes`` (Step 6) is a list of bundle includes, resolved relative to the
    scenario file's directory. Each element is either a bare **bundle-file path** (a
    verbatim merge — each a :class:`BundleSpec`) or a :class:`IncludeSpec`
    (``{bundle, prefix}`` — a **namespaced** instance, Step 6c). Each included bundle's
    parameters/stocks/flows/forcings merge into this scenario's
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
    n_sub: int = Field(default=1, ge=1)
    """How many sub-steps the **fast** flow set takes inside one master ``dt`` step.

    This is the knob that turns ``dt`` into a **coupling cadence** — what a neighbouring
    domain sees exported — rather than what the fast flows solve at, which becomes the
    **effective sub-step** ``dt/n_sub``. Default 1 (with the default all-fast partition)
    is the single-rate path, bit-identical to the pre-multi-rate engine.

    ``ge=1`` mirrors ``simcore.multirate.multirate_step``'s own ``n_sub >= 1`` guard at
    the file boundary, so a nonsensical value is a schema error on the author's file
    rather than a ``ValueError`` surfacing from the core mid-run.

    **``n_sub`` does not by itself make a step safe** — it only makes ``dt/n_sub`` the
    number that must satisfy ``k·dt < 1``. Measured: ``n_sub=2`` at ``dt=3600`` still
    wrecks the cabin (36.0 against a truth of 8.0). Multi-rate is the *performance
    enabler*, not the hazard closer; the build-time precondition on the **effective**
    sub-step is a later step.
    """
    rng_seed: int = 0
    includes: list[str | IncludeSpec] = Field(default_factory=list)
    parameters: dict[str, float] = Field(default_factory=dict)
    stocks: list[StockSpec] = Field(default_factory=list)
    flows: list[FlowSpec] = Field(default_factory=list)
    forcings: dict[str, ForcingSpec] = Field(default_factory=dict)
