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

from authoring.errors import AuthoringError
from authoring.flow_registry import FLOW_TYPES, PARAM_LOADERS
from authoring.schema import FlowSpec, ScenarioSpec, StockSpec
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


def _build_flow(spec: FlowSpec) -> Flow:
    """Lower one :class:`FlowSpec` to a frozen ``Flow`` via the flow-type registry.

    Validates the wiring dict against the flow type's declared fields (exact match)
    and resolves the param set for a params-taking flow — both as
    :class:`AuthoringError` (decidable from the file structure).
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
    if type_spec.takes_params:
        if spec.params is None:
            raise AuthoringError(
                f"flow {spec.id!r} ({spec.type}): this flow type requires a "
                f"'params' set (one of {sorted(PARAM_LOADERS)})"
            )
        loader = PARAM_LOADERS.get(spec.params)
        if loader is None:
            raise AuthoringError(
                f"flow {spec.id!r}: unknown param set {spec.params!r} "
                f"(known: {sorted(PARAM_LOADERS)})"
            )
        kwargs["params"] = loader()
    elif spec.params is not None:
        raise AuthoringError(
            f"flow {spec.id!r} ({spec.type}): this flow type takes no params, "
            f"but a 'params' set {spec.params!r} was given"
        )
    return type_spec.cls(FlowId(spec.id), spec.priority, **kwargs)


def interpret(spec: ScenarioSpec) -> BuiltScenario:
    """Build the runnable ``(State, Registry, resolver)`` graph from a scenario spec.

    Stocks are lowered and keyed by id (a duplicate id is an ``AuthoringError``);
    flows are lowered via the registry (``Registry`` re-sorts them into canonical
    order, so authoring order is inert); forcings become constant schedules. Single
    ``State`` at ``n=0`` with the authored seed.
    """
    stocks: dict[StockId, Stock] = {}
    for stock_spec in spec.stocks:
        stock = _build_stock(stock_spec)
        if stock.id in stocks:
            raise AuthoringError(f"duplicate stock id {stock.id!r}")
        stocks[stock.id] = stock
    state = State(n=0, stocks=stocks, rng_seed=spec.rng_seed)
    flows = [_build_flow(flow_spec) for flow_spec in spec.flows]
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
    """
    return interpret(ScenarioSpec.model_validate(load_yaml(path)))
