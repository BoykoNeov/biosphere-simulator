"""File composition: merge included **domain/species bundles** into a scenario (Step 6).

A scenario ``includes`` a list of bundle-file paths
(:class:`authoring.schema.BundleSpec`); :func:`apply_includes` reads each and merges its
``parameters`` / ``stocks`` / ``flows`` / ``forcings`` into the scenario's, producing a
single **flat** :class:`ScenarioSpec` the interpreter lowers exactly as before. This is
the "authored, not programmed" payoff one level up: a station is *composed* from
reusable bundles (a crew species, a battery domain) rather than re-declared inline.

**Merge semantics** (cross-port — Step 6b's Rust port + Step 7's freeze inherit them):

- **Order:** included bundles first (in ``includes`` order), then the scenario's own
  inline declarations. So a scenario that is *only* an include reproduces the bundle's
  declaration order exactly (the single-bundle byte-identity anchor).
- **No silent override:** a duplicate stock id, flow id, forcing key, or parameter name
  across any two sources is an :class:`AuthoringError` (``extra="forbid"``, a level up).
  Disjoint-domain composition (crew + battery) needs no id-namespacing.
- **Multi-instance id-namespacing** (Step 6c): an include may be a
  :class:`~authoring.schema.IncludeSpec` (``{bundle, prefix}``) instead of a bare path.
  A ``prefix`` namespaces every id the bundle declares — each stock id, flow id and
  forcing key becomes ``<prefix>.<id>`` — and every *reference* is rewritten to match:
  ``wiring`` values (stock ids), ``stoichiometry`` keys (stock ids), and the
  ``stock(...)``/``forcing(...)`` references inside a ``kinetics`` rate (parsed to the
  AST, prefixed there via :func:`_prefix_expr_refs`, and re-emitted by
  :func:`~authoring.expr_parser.render_rate_expr`). This lets the **same** bundle
  be included more than once — two ``{bundle: battery.domain, prefix: bat_a|bat_b}``
  instances compose without the id collision a bare double-include hits. ``param(...)``
  refs are **never** rewritten: in a rate they name a *frozen* param set (two instances
  correctly share ``k``); bundle-**parameter** namespacing is deferred (the only
  param-bearing bundle, crew, is un-multi-instanceable for the forcing reason below).
  A bundle whose *frozen* flows bind a forcing by a **hardcoded** name (the crew flows'
  ``crew_o2_intake`` module constant) cannot find a namespaced forcing key — so a
  prefixed crew include fails at resolve time, the documented crew-forcing blocker (the
  greenhouse ``CARBON_POOL`` analogue); kinetics / disjoint bundles namespace cleanly.
- **Flat, one level deep:** a bundle carries no ``includes`` of its own (rejected by
  ``BundleSpec``'s ``extra="forbid"``) — nested includes are out of scope.
- **Run config lives only in the top-level scenario** (a bundle has no integrator/dt/
  steps — also enforced by ``BundleSpec``).

**Parameter packs inside an included bundle are deferred** (advisor-endorsed scope,
matching Step 1's "full-file packs only" and the Rust port's "packs deferred"): a pack
path in a bundle would have to resolve against *the bundle's* directory, which needs
per-flow source-dir threading no Step-6 anchor exercises. A ``{pack: …}`` reference on a
bundle flow is a clean :class:`AuthoringError`; a bundle flow names a frozen param set
instead. (Top-level scenario flows resolve packs against the scenario dir — Step 1.)
"""

from __future__ import annotations

from pathlib import Path

from authoring.errors import AuthoringError
from authoring.expr_parser import parse_rate_expr, render_rate_expr
from authoring.schema import (
    BundleSpec,
    FlowSpec,
    ForcingSpec,
    ParamPackRef,
    ScenarioSpec,
    StockSpec,
)
from config import ConfigError, load_yaml
from simcore.expr import BinOp, Expr, ForcingRef, Neg, StockRef
from simcore.ids import StockId


def apply_includes(spec: ScenarioSpec, base_dir: Path) -> ScenarioSpec:
    """Merge ``spec.includes`` bundle files into a flat :class:`ScenarioSpec`.

    Returns ``spec`` unchanged if it has no includes (the pre-Step-6 path, so existing
    scenarios and bare specs are untouched). Otherwise reads each bundle relative to
    ``base_dir`` (the scenario file's directory), validates it, and returns a new spec
    with the merged parameters/stocks/flows/forcings and an emptied ``includes`` — which
    the interpreter then lowers identically to a hand-flattened scenario.
    """
    if not spec.includes:
        return spec

    parameters: dict[str, float] = {}
    stocks: list[StockSpec] = []
    flows: list[FlowSpec] = []
    forcings: dict[str, ForcingSpec] = {}
    stock_ids: dict[str, str] = {}  # id -> source label (for the collision message)
    flow_ids: dict[str, str] = {}

    def merge(
        source: str,
        params: dict[str, float],
        sspecs: list[StockSpec],
        fspecs: list[FlowSpec],
        fcs: dict[str, ForcingSpec],
    ) -> None:
        for name, value in params.items():
            if name in parameters:
                raise AuthoringError(
                    f"duplicate parameter {name!r}: declared by both {source} and an "
                    f"earlier source (parameters share one flat namespace across "
                    f"includes; rename or use 'overrides')"
                )
            parameters[name] = value
        for st in sspecs:
            if st.id in stock_ids:
                raise AuthoringError(
                    f"duplicate stock id {st.id!r}: declared by both {source} and "
                    f"{stock_ids[st.id]}"
                )
            stock_ids[st.id] = source
            stocks.append(st)
        for fl in fspecs:
            if fl.id in flow_ids:
                raise AuthoringError(
                    f"duplicate flow id {fl.id!r}: declared by both {source} and "
                    f"{flow_ids[fl.id]}"
                )
            if source != "the scenario" and isinstance(fl.params, ParamPackRef):
                raise AuthoringError(
                    f"flow {fl.id!r} in {source}: parameter packs inside an included "
                    f"bundle are deferred (a bundle pack must resolve against the "
                    f"bundle's directory); name a frozen param set instead"
                )
            flow_ids[fl.id] = source
            flows.append(fl)
        for key, forcing in fcs.items():
            if key in forcings:
                raise AuthoringError(
                    f"duplicate forcing {key!r}: declared by both {source} and an "
                    f"earlier source"
                )
            forcings[key] = forcing

    for inc in spec.includes:
        path = inc if isinstance(inc, str) else inc.bundle
        prefix = None if isinstance(inc, str) else inc.prefix
        bundle = _load_bundle(base_dir / path, path)
        if prefix is not None:
            bundle = _namespaced_bundle(bundle, prefix)
            source = f"bundle {path!r} (prefix {prefix!r})"
        else:
            source = f"bundle {path!r}"
        merge(source, bundle.parameters, bundle.stocks, bundle.flows, bundle.forcings)
    merge("the scenario", spec.parameters, spec.stocks, spec.flows, spec.forcings)

    return spec.model_copy(
        update={
            "includes": [],
            "parameters": parameters,
            "stocks": stocks,
            "flows": flows,
            "forcings": forcings,
        }
    )


def _prefix_expr_refs(node: Expr, prefix: str) -> Expr:
    """Rewrite ``stock``/``forcing`` reference names in a rate AST under ``prefix``.

    ``stock(...)`` and ``forcing(...)`` args are bundle ids/keys → namespaced to
    ``<prefix>.<name>``. ``param(...)`` is **left untouched** — in a rate it names a
    *frozen* param set (two instances of the bundle share the same ``k``), not a
    namespaced bundle parameter. ``Const``/``StepN`` carry no refs. Structural mirror of
    :func:`authoring.interpreter._collect_refs`.
    """
    if isinstance(node, StockRef):
        return StockRef(StockId(f"{prefix}.{node.stock}"))
    if isinstance(node, ForcingRef):
        return ForcingRef(f"{prefix}.{node.name}")
    if isinstance(node, Neg):
        return Neg(_prefix_expr_refs(node.operand, prefix))
    if isinstance(node, BinOp):
        return BinOp(
            node.op,
            _prefix_expr_refs(node.left, prefix),
            _prefix_expr_refs(node.right, prefix),
        )
    return node  # Const, ParamRef (frozen set), StepN — unchanged.


def _namespace_flow(flow: FlowSpec, prefix: str) -> FlowSpec:
    """Return a copy of ``flow`` with its id, wiring targets, stoichiometry keys and
    kinetics-rate stock/forcing refs namespaced under ``prefix`` (Step 6c)."""
    update: dict[str, object] = {"id": f"{prefix}.{flow.id}"}
    if flow.wiring:
        update["wiring"] = {k: f"{prefix}.{v}" for k, v in flow.wiring.items()}
    if flow.kinetics is not None:
        rate_ast = parse_rate_expr(flow.kinetics.rate)
        update["kinetics"] = flow.kinetics.model_copy(
            update={
                "rate": render_rate_expr(_prefix_expr_refs(rate_ast, prefix)),
                "stoichiometry": {
                    f"{prefix}.{sid}": coeff
                    for sid, coeff in flow.kinetics.stoichiometry.items()
                },
            }
        )
    return flow.model_copy(update=update)


def _namespaced_bundle(bundle: BundleSpec, prefix: str) -> BundleSpec:
    """Return a copy of ``bundle`` with every declared id namespaced under ``prefix``.

    Stock ids, flow ids (+ their references) and forcing keys become ``<prefix>.<id>``;
    ``parameters`` are **not** prefixed (bundle-parameter namespacing deferred — a
    param-bearing bundle is un-multi-instanceable for the crew-forcing reason, so a
    second prefixed instance collides on the parameter name, the honest boundary).
    """
    return bundle.model_copy(
        update={
            "stocks": [
                st.model_copy(update={"id": f"{prefix}.{st.id}"})
                for st in bundle.stocks
            ],
            "flows": [_namespace_flow(fl, prefix) for fl in bundle.flows],
            "forcings": {
                f"{prefix}.{key}": forcing for key, forcing in bundle.forcings.items()
            },
        }
    )


def _load_bundle(path: Path, label: str) -> BundleSpec:
    """Read + validate one bundle (``extra="forbid"`` — no run config, no nesting)."""
    try:
        raw = load_yaml(str(path))
    except ConfigError as exc:
        raise AuthoringError(
            f"included bundle {label!r} could not be read from {path}: {exc}"
        ) from exc
    return BundleSpec.model_validate(raw)
