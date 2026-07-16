"""Template parameters + boundary-time parameter-expression evaluation (Step 3).

A **template** is a scenario file with a top-level ``parameters:`` block (named
scalars + defaults) whose values may be *instantiated* per run (``overrides``), and
whose numeric fields (a stock ``amount``, a forcing ``const``) may be **expressions
over those parameters** instead of bare literals — so one habitat template yields
many habitats (``crew_count`` = 1, 4, …).

**This is a deliberate amendment of decision A.** Through Steps 0–2 the interpreter
"did no float math → the parity risk is purely structural." Lowering
``param('crew_count') * 1000.0 → 4000.0`` makes the boundary do arithmetic, which
adds a new cross-port surface: **the Rust boundary (Step 4) must compute the same
float.** It is benign — the ops are IEEE-deterministic ``+ − ×`` and decimals
round-trip identically across correct-rounding parsers — but it is load-bearing for
Step-4 boundary-eval parity and Step-7's freeze, so it is stated, not slipped.

**The grammar is reused, the context is not.** Template expressions parse with the
*same* :func:`authoring.expr_parser.parse_rate_expr` bounded grammar as the kinetics
DSL (so Step 4 mirrors one parser, and precedence/associativity stay pinned in one
place). But they are evaluated **at build time**, where no ``State``/``env``/``n``
exists: only ``Const`` / ``ParamRef`` / ``Neg`` / ``BinOp`` are legal, and
``param('…')`` resolves against the **template parameter** namespace (an *overload* of
the ``param`` keyword, which in a *kinetics rate* reads the flow's own params — the
two contexts are disjoint; documented here so Step 7's freeze is honest). A
``stock``/``forcing``/``n`` reference is an :class:`AuthoringError`.

The arithmetic op-order mirrors :func:`simcore.expr.eval_expr` exactly (``left``
before ``right``; ``+ − ×``) so a boundary literal computed here is bit-identical to
what the engine VM would compute from the same AST — one op-order, both ports.
"""

from __future__ import annotations

from collections.abc import Mapping

from authoring.errors import AuthoringError
from authoring.expr_parser import parse_rate_expr
from simcore.expr import BinOp, Const, Expr, Monod, Neg, ParamRef


def resolve_parameters(
    declared: Mapping[str, float], overrides: Mapping[str, float] | None
) -> dict[str, float]:
    """Merge a scenario's declared parameter defaults with an instantiation's overrides.

    ``declared`` is ``ScenarioSpec.parameters`` (name → default); ``overrides`` the
    per-run instantiation values. An override of a name that is **not declared** is an
    :class:`AuthoringError` (a template's parameter set is its explicit contract — a
    typo'd override is caught, not silently accepted). Override values are coerced to
    ``float`` (an integer ``crew_count: 4`` is fine).
    """
    params = dict(declared)
    if overrides:
        for name, value in overrides.items():
            if name not in declared:
                raise AuthoringError(
                    f"override of undeclared parameter {name!r} "
                    f"(declared: {sorted(declared)})"
                )
            params[name] = float(value)
    return params


def eval_numeric_field(
    value: float | str, params: Mapping[str, float], *, where: str
) -> float:
    """Lower a numeric scenario field (stock ``amount`` / forcing ``const``) to a float.

    A bare ``int``/``float`` passes through (the Step-0 form — the existing all-literal
    scenarios are unchanged, so their goldens stay byte-identical). A **string** is a
    template expression over ``params``: parsed with the bounded grammar and evaluated
    here (build time), producing the literal the frozen constructor receives. ``where``
    labels the field in any error.
    """
    if isinstance(value, (int, float)):
        return float(value)
    ast = parse_rate_expr(value)  # AuthoringError on a malformed expression
    return _eval(ast, params, where=where, whole=value)


def _eval(node: Expr, params: Mapping[str, float], *, where: str, whole: str) -> float:
    """Evaluate a build-time-legal AST subtree against the template parameter map.

    Only ``Const`` / ``ParamRef`` / ``Neg`` / ``BinOp`` are legal at build time; a
    ``StockRef`` / ``ForcingRef`` / ``StepN`` (no ``State``/``env``/``n`` exists yet) or
    an undeclared ``param('…')`` is an :class:`AuthoringError`. The ``+ − ×`` op-order
    mirrors :func:`simcore.expr.eval_expr` so the boundary literal is bit-identical to
    the engine VM's evaluation of the same AST.
    """
    if isinstance(node, Const):
        return node.value
    if isinstance(node, ParamRef):
        if node.name not in params:
            raise AuthoringError(
                f"{where}: expression {whole!r} references undeclared parameter "
                f"{node.name!r} (declared: {sorted(params)})"
            )
        return params[node.name]
    if isinstance(node, Neg):
        return -_eval(node.operand, params, where=where, whole=whole)
    if isinstance(node, BinOp):
        left = _eval(node.left, params, where=where, whole=whole)
        right = _eval(node.right, params, where=where, whole=whole)
        if node.op == "+":
            return left + right
        if node.op == "-":
            return left - right
        if node.op == "*":
            return left * right
        raise AuthoringError(  # pragma: no cover - parser only emits + - *
            f"{where}: unsupported operator {node.op!r} in {whole!r}"
        )
    if isinstance(node, Monod):
        # Rate-only (Tier 2), and a *deliberate, reversible* deferral rather than a
        # limitation: no frozen flow forces a saturating initial condition, and the
        # build-time-legal node set is its own frozen surface. Rejected precisely — the
        # generic message below would claim the author wrote a stock/forcing/n they did
        # not, which is exactly the kind of lying error this platform does not ship.
        raise AuthoringError(
            f"{where}: monod(…) is a kinetics-rate form and is not available in a "
            f"template expression {whole!r} (which is arithmetic over param('…') only)"
        )
    # StockRef / ForcingRef / StepN — legal in a kinetics *rate* (evaluated per step
    # against a State/env), but there is no State/env/n at build time.
    raise AuthoringError(
        f"{where}: template expression {whole!r} may reference only template "
        f"parameters (param('…')); stock/forcing/n are not available at build time"
    )
