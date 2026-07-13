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

from dataclasses import dataclass
from pathlib import Path

from authoring.errors import AuthoringError
from authoring.flow_registry import FLOW_TYPES, load_param_set
from authoring.schema import FlowSpec, ParamPackRef, ScenarioSpec, StockSpec
from config import load_yaml
from simcore.environment import SourceResolver, constant
from simcore.flow import Flow
from simcore.ids import DomainId, FlowId, StockId
from simcore.quantities import Quantity, StockKind, canonical_unit
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


def _build_flow(spec: FlowSpec, base_dir: Path) -> Flow:
    """Lower one :class:`FlowSpec` to a frozen ``Flow`` via the flow-type registry.

    Validates the wiring dict against the flow type's declared fields (exact match)
    and resolves the params object (named default or a pack, relative to ``base_dir``)
    for a params-taking flow — structural failures raise :class:`AuthoringError`.
    """
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
    flows = [_build_flow(flow_spec, base_dir) for flow_spec in spec.flows]
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
