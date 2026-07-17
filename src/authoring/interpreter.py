"""Lower a validated :class:`ScenarioSpec` to a runnable engine graph.

The interpreter is the boundary act at the heart of Phase 9: it turns declarative
data into ``(State, Registry, resolver)`` **by calling the frozen constructors** —
``simcore.boundary`` / ``Stock`` / the frozen ``Flow`` classes / ``SourceResolver``.
Through Steps 0–2 it did *no* float arithmetic, so the parity risk was purely
*structural* (same ids, same quantities/units, same param values, same wiring).

**Step 3 amends that (deliberately, see** :mod:`authoring.template` **).** A
*template* scenario declares ``parameters`` and may write a stock ``amount`` / forcing
``const`` as an **expression** over them (``param('crew_count') * 1000.0``); the
interpreter now evaluates those expressions to literals at build time (IEEE
``+ − ×``). This is the only float math the boundary does, it is deterministic, and
the frozen engine stays untouched — but it is a new cross-port *boundary-eval* surface
Step 4's Rust interpreter must match (not merely structural parity).

Everything decidable from the file structure alone is checked here and raised as an
:class:`AuthoringError` (unknown flow type, wiring that does not match the flow
type's fields, a missing/spurious param-set reference). A *well-formed* scenario
that wires a flow badly interprets cleanly and surfaces as a runtime
``ConservationError`` on the first step (the safety property, not this layer's job).
"""

from __future__ import annotations

from dataclasses import asdict, dataclass, is_dataclass
from pathlib import Path

from authoring.compose import apply_includes
from authoring.errors import AuthoringError
from authoring.expr_parser import parse_rate_expr
from authoring.flow_registry import FLOW_TYPES, PARAM_LOADERS, load_param_set
from authoring.schema import FlowSpec, ParamPackRef, ScenarioSpec, StockSpec
from authoring.template import eval_numeric_field, resolve_parameters
from config import load_yaml
from simcore.environment import SourceResolver, constant
from simcore.expr import (
    BinOp,
    Const,
    DeclarativeFlow,
    Expr,
    ForcingRef,
    Monod,
    Neg,
    ParamRef,
    StepN,
    StockRef,
)
from simcore.flow import Flow
from simcore.ids import DomainId, FlowId, StockId
from simcore.quantities import (
    BALANCE_ATOL,
    BALANCE_RTOL,
    Quantity,
    StockKind,
    canonical_unit,
)
from simcore.registry import Registry
from simcore.state import State, Stock

_RATE_CLASSES: tuple[str, ...] = ("fast", "slow")
"""The legal :attr:`authoring.schema.FlowSpec.rate_class` values — the author-visible
**rate-class vocabulary**.

The single source of truth in the manifest-derivation sense (like ``run._INTEGRATORS``):
``tests/test_authoring_freeze_manifest.py`` reads *this* rather than transcribing it,
so a class added here but exercised by nothing still trips the completeness gate.

**Closed at two, by the core's own signature** — ``multirate_step`` takes exactly two
``Substepper``s (decision N3: the driver takes the two pre-built integrators and does
not infer the partition). So unlike the flow-type registry, which is explicitly
expected to grow, this vocabulary cannot grow without a ``simcore`` change.
"""


_SLOW_STEP_DIVISOR: float = 2.0
"""The divisor giving the SLOW set's effective step under the pinned split: ``dt/2``.

**This tracks ``authoring.run._SPLIT``, and the coupling is the point.** Strang runs the
slow set as two ``dt/2`` half-steps around the fast block (``simcore.multirate``:
``ops = [(slow, dt/2), *fast_ops, (slow, dt/2)]`` ), so ``dt/2`` — *not* ``dt/n_sub`` —
is the step a slow flow is actually integrated at. Under **Lie** the slow set would step
at the full ``dt``, making this divisor too permissive by 2×. It is not imported from
``run`` because ``run`` imports *this* module;
``tests/test_authoring_rate_precondition.py`` asserts ``run._SPLIT is Split.STRANG``
instead, so flipping the split goes red **here** rather than silently loosening the
check.
"""


def _effective_step(dt: float, n_sub: int, *, multirate: bool, slow: bool) -> float:
    """The step size a flow is ACTUALLY integrated at — the number ``k`` multiplies.

    Three cases, and conflating them is the trap this function exists to name:

    * **single-rate** → ``dt``. The whole registry steps once per master step.
    * **multi-rate, fast** → ``dt/n_sub``. The point of the cadence knob.
    * **multi-rate, slow** → ``dt/2`` (:data:`_SLOW_STEP_DIVISOR`), *independent of*
      ``n_sub``, because Strang splits the slow set into two half-steps.

    **The plan for this step specified ``k·(dt/n_sub) < 1`` for every flow, and that is
    measured WRONG for the slow set** — a false PASS in the unsafe direction. With
    ``eclss.co2_scrubber`` classed slow at master ``dt=3600``, ``n_sub=60``, the formula
    reports ``k·h = 0.06`` (safe) while the flow truly steps at 1800 s → ``k·h = 1.8``:
    the run rations 24 times and empties ``cabin_co2`` to exactly 0.0. A check that
    greenlights that is worse than no check, because it reads as a guarantee.

    Note ``dt/n_sub`` coincides with ``dt`` at ``n_sub=1``, so the single-rate case is
    not a special case of the formula so much as the same arithmetic — the slow case is
    the one that genuinely differs. This is the **same Strang fact** that turned Step
    4's predicted 60× Thermal saving into a measured 30×; it has now bitten two separate
    claims in this phase, from the same blind spot: reasoning about ``n_sub`` as though
    it governed both sets.
    """
    if not multirate:
        return dt
    return dt / _SLOW_STEP_DIVISOR if slow else dt / n_sub


def _rate_precondition_message(
    flow_id: str,
    flow_type: str,
    param: str,
    k: float,
    h: float,
    dt: float,
    *,
    slow: bool,
    multirate: bool,
) -> str:
    """The ``AuthoringError`` text — the remedy is conditional on the flow's rate class.

    "Increase ``n_sub``" is honest for a **fast** flow and actively misleading for a
    **slow** one, which steps at ``dt/2`` however large ``n_sub`` grows: an author who
    followed that advice would raise ``n_sub``, watch nothing change, and conclude the
    check is broken. Mirrors ``run._rationed_message`` 's conditional structure — the
    same hazard, named at build time instead of run time.

    Every remedy quotes a **concrete number** the author can act on (the dt ceiling, the
    minimum ``n_sub``) rather than restating the inequality: the author already knows
    ``k*h < 1`` failed — what they need is the value that satisfies it.
    """
    if not multirate:
        where = f"dt={h!r}"
        remedy = (
            f"Reduce dt below {1.0 / k!r} s, or adopt a multi-rate cadence: keep dt as "
            f"the export cadence neighbours see and add 'n_sub' > {k * dt!r} so this "
            f"flow sub-steps at dt/n_sub"
        )
    elif slow:
        where = f"dt/2={h!r} (the slow set's Strang half-step, NOT dt/n_sub)"
        remedy = (
            f"This flow is rate_class 'slow', so it steps at dt/2 REGARDLESS of n_sub "
            f"(Strang splitting) — raising n_sub will NOT help it. Either reduce dt "
            f"below {2.0 / k!r} s, or re-class this flow 'fast' so that n_sub governs "
            f"its step too"
        )
    else:
        where = f"dt/n_sub={h!r} (master dt={dt!r})"
        remedy = (
            f"Increase n_sub past {k * dt!r} (i.e. to at least "
            f"{int(k * dt) + 1}), which leaves the master export cadence untouched, "
            f"or reduce dt"
        )
    return (
        f"flow {flow_id!r} ({flow_type}): param {param!r} = {k!r} /s is a first-order "
        f"rate constant, and this scenario integrates the flow at {where}, giving "
        f"k*h = {k * h!r} >= 1. The step is too large for this flow's frozen rate: "
        f"over one step the law removes more than the whole stock (or, for a "
        f"demand-controlled flow, overshoots its setpoint and oscillates). k*h < 1 is "
        f"the platform's EXPORT-FIDELITY bound, deliberately stricter than the "
        f"textbook stability bound k*h < 2: this engine couples domains, so a "
        f"neighbour must be able to USE the exported value, not merely watch it "
        f"converge eventually. "
        f"{remedy}. See 'The dt constraint' in docs/authoring-reference.md. To build "
        f"the unsafe scenario anyway — to STUDY it, not to make it work — pass "
        f"allow_unsafe_step=True."
    )


@dataclass(frozen=True)
class BuiltScenario:
    """The interpreted graph plus its run config — everything a run needs.

    ``integrator`` is the requested kind ("euler"/"rk4"); the run harness
    (:mod:`authoring.run`) constructs the matching integrator(s) and steps ``steps``
    times at ``dt``.

    **Three registries, deliberately.** ``registry`` is the whole flow set (the
    single-rate path, and the structural-equality surface tests compare against);
    ``slow_registry`` / ``fast_registry`` are its **disjoint partition over the same
    stock dict** (decision N3), which is what ``multirate_step`` consumes. They are
    always built, so the shape does not depend on whether multi-rate was declared;
    ``slow ∪ fast == registry`` and ``slow ∩ fast == ∅`` by construction. Keeping the
    full ``registry`` is what lets the run harness take **today's single-rate code
    path** verbatim when no partition is declared, rather than leaning on the
    (measured, but load-bearing) ``n_sub=1`` identity.
    """

    name: str
    state: State
    registry: Registry
    slow_registry: Registry
    fast_registry: Registry
    resolver: SourceResolver
    integrator: str
    dt: float
    steps: int
    n_sub: int = 1
    has_authored_kinetics: bool = False
    """True if any flow is an authored :class:`~simcore.expr.DeclarativeFlow`.

    The **"authored ≠ validated"** marker (decision B): conservation + determinism are
    guaranteed for such a run, scientific validity is not. The display projection
    (Godot, a later step) surfaces this; carrying it on the built scenario is where the
    fact originates.
    """

    @property
    def is_multirate(self) -> bool:
        """True if this scenario declared a multi-rate cadence — i.e. needs the driver.

        Either a sub-stepped fast set (``n_sub > 1``) *or* a non-empty slow partition.
        The two are not independent: ``interpret`` refuses a slow set at ``n_sub == 1``,
        so on any **built** scenario this is equivalent to ``n_sub > 1``. The robust
        form is written anyway — the equivalence is a consequence of that refusal, not a
        property of multi-rate, and hard-coding it here would silently mislower a
        scenario the day the refusal is relaxed.
        """
        return self.n_sub > 1 or bool(self.slow_registry.flows)


def _build_stock(spec: StockSpec, params: dict[str, float]) -> Stock:
    """Lower one :class:`StockSpec` to a frozen ``Stock``.

    ``unit`` is derived from ``quantity`` via the canonical-unit table (the single
    source of truth — never authored), so an authored stock cannot carry a
    mislabelled unit. An unknown quantity/kind value surfaces as an
    ``AuthoringError`` (rather than a raw ``ValueError`` from the enum lookup). The
    ``amount`` is a literal or a **template expression** over ``params`` (Step 3),
    lowered to a float here.
    """
    amount = eval_numeric_field(spec.amount, params, where=f"stock {spec.id!r} amount")
    try:
        quantity = Quantity(spec.quantity)
    except ValueError as exc:
        raise AuthoringError(
            f"stock {spec.id!r}: unknown quantity {spec.quantity!r}"
        ) from exc
    try:
        kind = StockKind(spec.kind)
    except ValueError as exc:
        raise AuthoringError(f"stock {spec.id!r}: unknown kind {spec.kind!r}") from exc
    composition: dict[Quantity, float] = {}
    if spec.composition is not None:
        for qname, coeff in spec.composition.items():
            try:
                composition[Quantity(qname)] = coeff
            except ValueError as exc:
                raise AuthoringError(
                    f"stock {spec.id!r}: unknown composition quantity {qname!r}"
                ) from exc
    # ``Stock.__post_init__`` enforces the deeper invariants (finite amount,
    # composition includes the stock's own quantity, POPULATION single-quantity,
    # unclamped-only-on-BOUNDARY); a violation there is a genuine (Value)Error we let
    # propagate — the authored file asked for an impossible stock.
    return Stock(
        id=StockId(spec.id),
        domain=DomainId(spec.domain),
        quantity=quantity,
        unit=canonical_unit(quantity),
        amount=amount,
        kind=kind,
        extinction_threshold=spec.extinction_threshold,
        unclamped=spec.unclamped,
        composition=composition,
    )


def _resolve_params(spec: FlowSpec, param_set: str, base_dir: Path) -> object:
    """Resolve a params-taking flow's params object (named default or a pack).

    ``spec.params`` is a **string** naming the flow type's frozen default set (must
    equal ``param_set``) or a :class:`ParamPackRef`; a pack path is resolved relative
    to ``base_dir`` (the scenario file's directory). The frozen loader reads both, so
    a pack's values pass the frozen bounds/unit validation.
    """
    if spec.params is None:
        raise AuthoringError(
            f"flow {spec.id!r} ({spec.type}): this flow type requires 'params' "
            f"(the set name {param_set!r} or a {{pack: …}} reference)"
        )
    if isinstance(spec.params, ParamPackRef):
        pack_path = base_dir / spec.params.pack
        return load_param_set(param_set, pack_path)
    # A bare string names the flow type's default committed set.
    if spec.params != param_set:
        raise AuthoringError(
            f"flow {spec.id!r} ({spec.type}): param set {spec.params!r} does not "
            f"match this flow type's set {param_set!r}"
        )
    return load_param_set(param_set, None)


def _kinetics_param_map(spec: FlowSpec) -> dict[str, float]:
    """Resolve an authored-kinetics flow's params to a ``name -> float`` map.

    ``spec.params`` is a **string** naming a :data:`PARAM_LOADERS` set (whose frozen
    loader produces a params dataclass — flattened here to floats the rate's
    ``param("…")`` reads, so authored params still pass the frozen bounds/unit guards),
    or ``None`` for a param-free rate. Parameter **packs** for authored flows are
    deferred (a pack needs an explicit set name to pick the loader, which the
    frozen-type path derives from ``FlowTypeSpec`` but a kinetics flow has not) — a
    :class:`ParamPackRef` here is an :class:`AuthoringError`, consistent with Step 1's
    "full-file packs only" deferrals.
    """
    if spec.params is None:
        return {}
    if isinstance(spec.params, ParamPackRef):
        raise AuthoringError(
            f"flow {spec.id!r}: parameter packs for authored 'kinetics' flows are "
            f"deferred; name a param set (a {sorted(PARAM_LOADERS)} key) instead"
        )
    if spec.params not in PARAM_LOADERS:
        raise AuthoringError(
            f"flow {spec.id!r}: unknown param set {spec.params!r} "
            f"(known: {sorted(PARAM_LOADERS)})"
        )
    params_obj = load_param_set(spec.params, None)
    # Frozen param loaders return a flat frozen dataclass of floats (ChargeParams,
    # SelfDischargeParams, …); flatten it to the name→float map the rate's param("…")
    # reads. ``is_dataclass`` + not-a-type narrows the loader's ``object`` return.
    if not is_dataclass(params_obj) or isinstance(params_obj, type):
        raise AuthoringError(
            f"flow {spec.id!r}: param set {spec.params!r} did not load a dataclass"
        )
    return {name: float(value) for name, value in asdict(params_obj).items()}


def _collect_refs(node: Expr, params: set[str], stocks: set[StockId]) -> None:
    """Recursively collect ``param``/``stock`` reference names from a rate AST.

    ``forcing`` references are intentionally *not* collected — like the frozen flows'
    ``env.get``, a forcing var's referential integrity is resolve-time (a ``KeyError``
    at the first step if unwired), not a build check.

    **Exhaustive over the ``Expr`` union, deliberately.** This walk used to end in an
    implicit fall-through, which silently skipped build-time referential validation of
    any unhandled node's subtree — an unknown ``param``/``stock`` inside it would then
    surface as a runtime ``KeyError`` instead of a clean ``AuthoringError``. Found when
    Tier 2 added ``Monod``; the ref-free nodes are now named explicitly and an unknown
    node raises, so the next grammar addition cannot slip. Structural mirror of
    :func:`authoring.compose._prefix_expr_refs`.
    """
    if isinstance(node, ParamRef):
        params.add(node.name)
    elif isinstance(node, StockRef):
        stocks.add(node.stock)
    elif isinstance(node, ForcingRef):
        pass  # resolve-time (env.get), by design
    elif isinstance(node, Neg):
        _collect_refs(node.operand, params, stocks)
    elif isinstance(node, BinOp):
        _collect_refs(node.left, params, stocks)
        _collect_refs(node.right, params, stocks)
    elif isinstance(node, Monod):
        _collect_refs(node.substrate, params, stocks)
        _collect_refs(node.half_saturation, params, stocks)
    elif isinstance(node, (Const, StepN)):
        pass  # no references
    else:  # pragma: no cover - exhaustive over the Expr union
        raise AuthoringError(f"cannot validate unknown expression node {node!r}")


def _check_stoichiometry_balanced(
    flow_id: str,
    stoichiometry: tuple[tuple[StockId, float], ...],
    stocks: dict[StockId, Stock],
) -> None:
    """Verify the coefficient vector balances per quantity (decision C, build time).

    Computes ``Σ(coeff · composition[q])`` over the stoichiometry for each conserved
    quantity and requires it within ``assert_flow_balanced``'s **relative** tolerance
    (exact for integer coefficients like ``−1/+1``; tolerance-backed for fractional
    split coefficients ``f/(1−f)``, exactly as ``charge_split`` relies on). Because the
    single scalar ``rate·dt`` multiplies every leg, a balanced coefficient vector keeps
    ``Σ legs = 0`` for *any* rate/state — so an unbalanced authored flow is rejected
    here, before it can run, rather than only surfacing at the every-step gate.
    """
    residual: dict[Quantity, float] = {}
    scale: dict[Quantity, float] = {}
    for stock_id, coeff in stoichiometry:
        for quantity, comp in stocks[stock_id].composition.items():
            residual[quantity] = residual.get(quantity, 0.0) + coeff * comp
            scale[quantity] = max(scale.get(quantity, 0.0), abs(coeff * comp))
    for quantity in sorted(residual, key=lambda q: q.name):
        tol = BALANCE_ATOL + BALANCE_RTOL * scale.get(quantity, 0.0)
        if abs(residual[quantity]) > tol:
            raise AuthoringError(
                f"flow {flow_id!r}: authored stoichiometry is not balanced for "
                f"{quantity.name} (Σ coeff·composition = {residual[quantity]!r}, "
                f"tolerance {tol!r}); an authored flow must conserve every quantity"
            )


def _build_declarative_flow(
    spec: FlowSpec, stocks: dict[StockId, Stock], base_dir: Path
) -> Flow:
    """Lower a :class:`FlowSpec` with ``kinetics`` to a :class:`DeclarativeFlow`.

    Parses the rate expression, coerces the stoichiometry coefficients to ``float``,
    resolves the param map, then applies the build-time structural checks
    (referential integrity of ``param``/``stock`` reads, non-empty stoichiometry over
    known stocks, and balance-by-construction). Any structural failure is an
    :class:`AuthoringError`; a *well-formed* but physically-nonsensical rate runs (it
    still conserves) — "authored ≠ validated" (decision B).
    """
    kinetics = spec.kinetics
    assert kinetics is not None  # guaranteed by the caller's branch
    rate = parse_rate_expr(kinetics.rate)
    stoichiometry = tuple(
        (StockId(stock_id), float(coeff))
        for stock_id, coeff in kinetics.stoichiometry.items()
    )
    if not stoichiometry:
        raise AuthoringError(f"flow {spec.id!r}: kinetics 'stoichiometry' is empty")
    param_map = _kinetics_param_map(spec)

    ref_params: set[str] = set()
    ref_stocks: set[StockId] = set()
    _collect_refs(rate, ref_params, ref_stocks)
    for name in sorted(ref_params):
        if name not in param_map:
            raise AuthoringError(
                f"flow {spec.id!r}: rate references param {name!r} not in its param "
                f"set (available: {sorted(param_map)})"
            )
    for stock_id in sorted(ref_stocks) + [sid for sid, _ in stoichiometry]:
        if stock_id not in stocks:
            raise AuthoringError(
                f"flow {spec.id!r}: references unknown stock {stock_id!r}"
            )
    _check_stoichiometry_balanced(spec.id, stoichiometry, stocks)

    return DeclarativeFlow(
        id=FlowId(spec.id),
        priority=spec.priority,
        rate=rate,
        stoichiometry=stoichiometry,
        params=tuple(sorted(param_map.items())),
    )


def _slow_flow_ids(spec: ScenarioSpec) -> frozenset[FlowId]:
    """The declared **slow** partition, with the rate-class vocabulary validated.

    Must run over the **post-**:func:`~authoring.compose.apply_includes` spec, and that
    is the whole reason this is not a pydantic ``model_validator`` on ``ScenarioSpec``:
    a *bundle* may contribute ``rate_class: slow`` flows, and a schema-level validator
    sees only the scenario's own inline ``flows`` — it would miss them, and (worse)
    would miss them silently, lowering a partitioned scenario as if it were single-rate.
    """
    slow: set[FlowId] = set()
    for flow_spec in spec.flows:
        if flow_spec.rate_class not in _RATE_CLASSES:
            raise AuthoringError(
                f"flow {flow_spec.id!r}: unknown rate class {flow_spec.rate_class!r} "
                f"(known: {sorted(_RATE_CLASSES)})"
            )
        if flow_spec.rate_class == "slow":
            slow.add(FlowId(flow_spec.id))
    return frozenset(slow)


def _check_rate_preconditions(
    spec: ScenarioSpec,
    flows: list[Flow],
    slow_ids: frozenset[FlowId],
) -> None:
    """Refuse a scenario whose step is too large for a declared rate (Step 5).

    For every frozen flow type declaring
    :attr:`~authoring.flow_registry.FlowTypeSpec.rate_params`, read each ``k`` off the
    **pack-resolved** params object and require ``k · h < 1``, where ``h`` is that
    flow's :func:`_effective_step`.

    **Why build time, and why here rather than in ``run``** — a ``params: {pack: …}``
    may inflate a gain, and a pack's values exist only *after* this function's caller
    resolves them. ``run_scenario`` receives an already-built flow and cannot see what
    the pack asked for, so the pack case is checkable **only** at this point in the
    pipeline. That is the load-bearing reason, ahead of the (real, but secondary)
    convenience of failing before a long run rather than after it.

    **Read off the built flow, not re-resolved.** ``flows`` is in ``spec.flows`` order
    (the caller's list comprehension), so the zip is positional and exact; re-loading
    each pack would double the file reads and, worse, could disagree with what was
    actually built. Transcendental-free (``+ − × <``) ⇒ the Rust mirror is byte-safe.

    **What this honestly does NOT cover, by declaration rather than omission**: authored
    ``kinetics`` (the author wrote the rate law, so the platform cannot know its
    constant — decision B's "authored ≠ validated" boundary),
    ``thermal.radiator_reject`` (``τ ≫ dt`` is not a predicate), and
    ``eclss.crew_metabolism`` (``forced draw < stock`` is state-dependent). So the claim
    is "the platform catches the ``k·dt`` family", never "your dt is safe". See
    ``FlowTypeSpec.rate_params``.
    """
    multirate = spec.n_sub > 1 or bool(slow_ids)
    # strict=True asserts the positional correspondence this function's contract rests
    # on (`flows` is built by a comprehension over `spec.flows`). If that ever stops
    # holding, a silent zip-truncation would check the WRONG flow's params against the
    # wrong rate class — the failure would be a wrong verdict, not a crash, so it must
    # be loud.
    for flow_spec, flow in zip(spec.flows, flows, strict=True):
        if flow_spec.type is None:  # authored kinetics — structurally uncheckable
            continue
        type_spec = FLOW_TYPES[flow_spec.type]  # _build_flow already validated the name
        if not type_spec.rate_params:
            continue
        params_obj = getattr(flow, "params", None)
        if params_obj is None:  # pragma: no cover — a registry bug, not authored input
            raise AuthoringError(
                f"flow {flow_spec.id!r} ({flow_spec.type}): declares rate_params "
                f"{list(type_spec.rate_params)} but its built flow carries no params "
                f"object; the flow-type registry is inconsistent"
            )
        h = _effective_step(
            spec.dt,
            spec.n_sub,
            multirate=multirate,
            slow=FlowId(flow_spec.id) in slow_ids,
        )
        for param in type_spec.rate_params:
            k = float(getattr(params_obj, param))
            if k * h >= 1.0:
                raise AuthoringError(
                    _rate_precondition_message(
                        flow_spec.id,
                        flow_spec.type,
                        param,
                        k,
                        h,
                        spec.dt,
                        slow=FlowId(flow_spec.id) in slow_ids,
                        multirate=multirate,
                    )
                )


def _build_flow(spec: FlowSpec, stocks: dict[StockId, Stock], base_dir: Path) -> Flow:
    """Lower one :class:`FlowSpec` to a ``Flow``: a frozen type, or authored kinetics.

    A ``kinetics`` flow is lowered to a :class:`DeclarativeFlow`
    (:func:`_build_declarative_flow`). A frozen-``type`` flow validates its wiring dict
    against the flow type's declared fields (exact match) and resolves the params
    object (named default or a pack, relative to ``base_dir``) — structural failures
    raise :class:`AuthoringError`. (The schema validator has already guaranteed exactly
    one of ``type``/``kinetics`` is set.)
    """
    if spec.kinetics is not None:
        return _build_declarative_flow(spec, stocks, base_dir)
    assert spec.type is not None  # schema validator: type xor kinetics
    type_spec = FLOW_TYPES.get(spec.type)
    if type_spec is None:
        raise AuthoringError(
            f"flow {spec.id!r}: unknown flow type {spec.type!r} "
            f"(known: {sorted(FLOW_TYPES)})"
        )
    if set(spec.wiring) != set(type_spec.wiring_fields):
        raise AuthoringError(
            f"flow {spec.id!r} ({spec.type}): wiring keys {sorted(spec.wiring)} "
            f"do not match this flow type's fields {sorted(type_spec.wiring_fields)}"
        )
    kwargs: dict[str, object] = {
        field: StockId(spec.wiring[field]) for field in type_spec.wiring_fields
    }
    if type_spec.param_set is not None:
        kwargs["params"] = _resolve_params(spec, type_spec.param_set, base_dir)
    elif spec.params is not None:
        raise AuthoringError(
            f"flow {spec.id!r} ({spec.type}): this flow type takes no params, "
            f"but 'params' was given"
        )
    return type_spec.cls(FlowId(spec.id), spec.priority, **kwargs)


def interpret(
    spec: ScenarioSpec,
    base_dir: Path | None = None,
    overrides: dict[str, float] | None = None,
    *,
    allow_unsafe_step: bool = False,
) -> BuiltScenario:
    """Build the runnable ``(State, Registry, resolver)`` graph from a scenario spec.

    Any ``includes`` are merged first (:func:`authoring.compose.apply_includes`):
    each referenced bundle file's parameters/stocks/flows/forcings are flattened into
    the scenario (bundles first, then inline; a duplicate across sources is an
    ``AuthoringError``), yielding a self-contained spec lowered exactly as a
    hand-flattened one — so composition adds no per-step surface.

    Template ``parameters`` are resolved next (defaults + ``overrides``; an override
    of an undeclared name is an ``AuthoringError``), then any stock ``amount`` /
    forcing ``const`` **expression** over them is evaluated to a literal (Step 3).
    Stocks are lowered and keyed by id (a duplicate id is an ``AuthoringError``); flows
    are
    lowered via the registry (``Registry`` re-sorts them into canonical order, so
    authoring order is inert); forcings become constant schedules. Single ``State`` at
    ``n=0`` with the authored seed. ``base_dir`` is the directory that any
    parameter-pack path is resolved against (defaults to the process CWD — set it to
    the scenario file's parent via :func:`load_scenario`).

    Finally the flows are **partitioned by rate class** into the two disjoint registries
    ``multirate_step`` consumes (N3), over the one shared stock dict. This runs *after*
    the include merge, so a bundle-contributed ``rate_class: slow`` flow is seen. Two
    combinations are refused here rather than honoured:

    * an unknown ``rate_class`` → ``AuthoringError`` (the ``quantity``/``kind`` rule);
    * ``n_sub = 1`` **with** a non-empty slow set → ``AuthoringError``. It buys no rate
      separation and no perf win while silently perturbing the trajectory, via the slow
      set's own two-half-step discretization *and* the mid-step coupling — both
      measured, over all three flow shapes, in
      ``tests/test_authoring_multirate_identity.py``.

    ``n_sub > 1`` with an **empty** slow set is deliberately **legal**: it is uniform
    sub-stepping, which decouples the export cadence from the solver step, and it is
    the configuration the measured payoff rests on (master ``dt=3600``, ``n_sub=60``
    lands on the same value as a single-rate ``dt=60`` run while exporting 60x less
    often).

    Finally the **rate precondition** (Step 5): every frozen flow type declaring
    ``rate_params`` must satisfy ``k · h < 1`` at its own effective step ``h``
    (:func:`_check_rate_preconditions`). This **moves the ``k·dt`` family's failure from
    run time to build time** — a donor-controlled flow that used to surface as
    ``run_scenario`` 's ``RationedError`` after a full run is now refused here, before
    any step. For the demand-controlled ``eclss.o2_makeup`` it is not a *move* but the
    **only** catch there has ever been: rationing structurally cannot see it.

    ``allow_unsafe_step=True`` skips that check — the
    ``run_scenario(allow_rationing=True)`` idiom, and for the same purpose: **studying**
    an unsafe run (``tests/test_authoring_export_fidelity.py`` exists to measure the
    oscillation the bound excludes, and cannot construct its own subject otherwise),
    never making a scenario "work". It does not make the step safe; it makes the
    platform stop objecting.
    """
    if base_dir is None:
        base_dir = Path()
    spec = apply_includes(spec, base_dir)
    params = resolve_parameters(spec.parameters, overrides)
    stocks: dict[StockId, Stock] = {}
    for stock_spec in spec.stocks:
        stock = _build_stock(stock_spec, params)
        if stock.id in stocks:
            raise AuthoringError(f"duplicate stock id {stock.id!r}")
        stocks[stock.id] = stock
    state = State(n=0, stocks=stocks, rng_seed=spec.rng_seed)
    flows = [_build_flow(flow_spec, stocks, base_dir) for flow_spec in spec.flows]
    registry = Registry(flows, stocks)
    slow_ids = _slow_flow_ids(spec)
    if spec.n_sub == 1 and slow_ids:
        raise AuthoringError(
            f"n_sub=1 with a non-empty slow set ({sorted(slow_ids)}): a partition at "
            f"n_sub=1 buys NO rate separation (the fast set takes one full-dt "
            f"sub-step) and no performance win — yet it is not inert. It does not "
            f"reproduce the single-rate trajectory: the slow set is split into two "
            f"dt/2 half-steps ((1-k*dt/2)^2 != (1-k*dt)) and, the dominant effect, the "
            f"fast flows read slow-updated stocks mid-step. So this is a "
            f"misconfiguration that would silently move the answer. Either raise n_sub "
            f"(the fast set then sub-steps at dt/n_sub, which is the point of the "
            f"partition), or drop the 'rate_class: slow' key(s) to run single-rate."
        )
    if not allow_unsafe_step:
        _check_rate_preconditions(spec, flows, slow_ids)
    # The disjoint partition over the SAME stock dict (N3). Built unconditionally: with
    # no authored rate_class keys the slow set is empty and `fast_registry` holds every
    # flow, so this is inert for every pre-multi-rate scenario.
    slow_registry = Registry([f for f in flows if f.id in slow_ids], stocks)
    fast_registry = Registry([f for f in flows if f.id not in slow_ids], stocks)
    resolver = SourceResolver(
        forcings={
            name: constant(
                eval_numeric_field(f.const, params, where=f"forcing {name!r} const")
            )
            for name, f in spec.forcings.items()
        }
    )
    return BuiltScenario(
        name=spec.name,
        state=state,
        registry=registry,
        slow_registry=slow_registry,
        fast_registry=fast_registry,
        resolver=resolver,
        integrator=spec.integrator,
        dt=spec.dt,
        steps=spec.steps,
        n_sub=spec.n_sub,
        has_authored_kinetics=any(f.kinetics is not None for f in spec.flows),
    )


def load_scenario(
    path: str,
    overrides: dict[str, float] | None = None,
    *,
    allow_unsafe_step: bool = False,
) -> BuiltScenario:
    """Read a scenario YAML file, validate its schema, and interpret it.

    ``load_yaml`` is the safe (``yaml.safe_load``, top-level-mapping) read shared
    with the param loaders; ``ScenarioSpec.model_validate`` applies the schema
    (``extra="forbid"`` + float coercion — the pyyaml numeric-string backstop).
    Parameter-pack paths are resolved relative to ``path``'s directory. ``overrides``
    instantiates a template's ``parameters`` (Step 3) — the "one template, many
    habitats" knob. ``allow_unsafe_step`` is forwarded to :func:`interpret` — threaded
    rather than omitted so that the file-loading surface an author actually calls is not
    the one place the study hatch is unavailable.
    """
    spec = ScenarioSpec.model_validate(load_yaml(path))
    return interpret(
        spec,
        base_dir=Path(path).parent,
        overrides=overrides,
        allow_unsafe_step=allow_unsafe_step,
    )
