"""Lower a validated :class:`ScenarioSpec` to a runnable engine graph.

The interpreter is the boundary act at the heart of Phase 9: it turns declarative
data into ``(State, Registry, resolver)`` **by calling the frozen constructors** —
``simcore.boundary`` / ``Stock`` / the frozen ``Flow`` classes / ``SourceResolver``
— and does no float arithmetic itself. So the parity risk is purely *structural*
(same ids, same quantities/units, same param values, same wiring), and the pure
engine is untouched.

Everything decidable from the file structure alone is checked here and raised as an
:class:`AuthoringError` (unknown flow type, wiring that does not match the flow
type's fields, a missing/spurious param-set reference). A *well-formed* scenario
that wires a flow badly interprets cleanly and surfaces as a runtime
``ConservationError`` on the first step (the safety property, not this layer's job).
"""

from __future__ import annotations

from dataclasses import asdict, dataclass, is_dataclass
from pathlib import Path

from authoring.errors import AuthoringError
from authoring.expr_parser import parse_rate_expr
from authoring.flow_registry import FLOW_TYPES, PARAM_LOADERS, load_param_set
from authoring.schema import FlowSpec, ParamPackRef, ScenarioSpec, StockSpec
from config import load_yaml
from simcore.environment import SourceResolver, constant
from simcore.expr import (
    BinOp,
    DeclarativeFlow,
    Expr,
    ForcingRef,
    Neg,
    ParamRef,
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


@dataclass(frozen=True)
class BuiltScenario:
    """The interpreted graph plus its run config — everything a run needs.

    ``integrator`` is the requested kind ("euler"/"rk4"); the run harness
    (:mod:`authoring.run`) constructs the matching integrator from ``registry`` and
    steps ``steps`` times at ``dt``.
    """

    name: str
    state: State
    registry: Registry
    resolver: SourceResolver
    integrator: str
    dt: float
    steps: int
    has_authored_kinetics: bool = False
    """True if any flow is an authored :class:`~simcore.expr.DeclarativeFlow`.

    The **"authored ≠ validated"** marker (decision B): conservation + determinism are
    guaranteed for such a run, scientific validity is not. The display projection
    (Godot, a later step) surfaces this; carrying it on the built scenario is where the
    fact originates.
    """


def _build_stock(spec: StockSpec) -> Stock:
    """Lower one :class:`StockSpec` to a frozen ``Stock``.

    ``unit`` is derived from ``quantity`` via the canonical-unit table (the single
    source of truth — never authored), so an authored stock cannot carry a
    mislabelled unit. An unknown quantity/kind value surfaces as an
    ``AuthoringError`` (rather than a raw ``ValueError`` from the enum lookup).
    """
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
        amount=spec.amount,
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
    # Const, StepN: no references.


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


def interpret(spec: ScenarioSpec, base_dir: Path | None = None) -> BuiltScenario:
    """Build the runnable ``(State, Registry, resolver)`` graph from a scenario spec.

    Stocks are lowered and keyed by id (a duplicate id is an ``AuthoringError``);
    flows are lowered via the registry (``Registry`` re-sorts them into canonical
    order, so authoring order is inert); forcings become constant schedules. Single
    ``State`` at ``n=0`` with the authored seed. ``base_dir`` is the directory that
    any parameter-pack path is resolved against (defaults to the process CWD — set it
    to the scenario file's parent via :func:`load_scenario`).
    """
    if base_dir is None:
        base_dir = Path()
    stocks: dict[StockId, Stock] = {}
    for stock_spec in spec.stocks:
        stock = _build_stock(stock_spec)
        if stock.id in stocks:
            raise AuthoringError(f"duplicate stock id {stock.id!r}")
        stocks[stock.id] = stock
    state = State(n=0, stocks=stocks, rng_seed=spec.rng_seed)
    flows = [_build_flow(flow_spec, stocks, base_dir) for flow_spec in spec.flows]
    registry = Registry(flows, stocks)
    resolver = SourceResolver(
        forcings={name: constant(f.const) for name, f in spec.forcings.items()}
    )
    return BuiltScenario(
        name=spec.name,
        state=state,
        registry=registry,
        resolver=resolver,
        integrator=spec.integrator,
        dt=spec.dt,
        steps=spec.steps,
        has_authored_kinetics=any(f.kinetics is not None for f in spec.flows),
    )


def load_scenario(path: str) -> BuiltScenario:
    """Read a scenario YAML file, validate its schema, and interpret it.

    ``load_yaml`` is the safe (``yaml.safe_load``, top-level-mapping) read shared
    with the param loaders; ``ScenarioSpec.model_validate`` applies the schema
    (``extra="forbid"`` + float coercion — the pyyaml numeric-string backstop).
    Parameter-pack paths are resolved relative to ``path``'s directory.
    """
    spec = ScenarioSpec.model_validate(load_yaml(path))
    return interpret(spec, base_dir=Path(path).parent)
