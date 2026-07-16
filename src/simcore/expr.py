"""The bounded kinetics expression VM: a plain-data AST + evaluator + DeclarativeFlow.

**This module is the one deliberate, one-time extension of the frozen ``simcore``
core since the biosphere/station freeze** (Phase 9, decision A / decision D). It is
purely *additive* — no existing ``simcore`` file changes — so every frozen golden
stays byte-identical; the "zero core change" streak is broken **once, on purpose**,
to add a single frozen engine primitive (like adding a new integrator), never
per-scenario code.

Why it lives in the core (not the boundary): an authored flow's rate expression is
evaluated **per step, inside the integrator** (once per Euler step, per stage under
RK4), so the evaluator must be pure stdlib and deterministic like every other flow.
*Parsing* a scenario file's text into this AST is a one-time boundary act and stays
in ``src/authoring`` (decision A). Only AST→``float`` evaluation is core.

The grammar is **bounded, closed, and deterministic** (decision D) — a fixed, finite
set of primitives, no user functions / recursion / loops / I/O. Step 2 ships the
unambiguous arithmetic core exercised by the re-expression anchor (``SelfDischarge``):

  * literals (``Const``);
  * reads: a stock amount by id (``StockRef``), a param by name (``ParamRef``), a
    forcing by name (``ForcingRef``, resolved through ``env.get`` — #16), and the
    integer step ``n`` (``StepN``);
  * binary ``+ - *`` (``BinOp``) and unary ``-`` (``Neg``);
  * saturating kinetics ``monod`` (``Monod``) — the post-roadmap Tier-2 unfreeze; see
    that node's docstring for the frozen flow that forced its definition.

**Deliberately deferred** until a real frozen flow forces the semantic definition
(the ``_DemoSchema``/``_CanopySchema`` "bespoke until a second instance justifies it"
discipline, applied to the grammar):

  * ``/`` — division is IEEE-unambiguous *except* at ``x/0`` (Python raises
    ``ZeroDivisionError``; Rust f64 yields ``inf``). ``monod`` (the first divider)
    has now landed **without** lifting this: it guards its own denominator, so it
    resolves ``x/0`` *internally* and never exposes the raw form. Bare ``/`` would
    re-introduce exactly the hazard this defers, and no frozen flow forces it;
  * the rest of the closed **function set** — ``exp ln pow sqrt abs min max clamp`` +
    bounded conditionals. ``clamp``/``ifpos`` still carry a real definitional choice
    (inclusive bounds? ``>0`` vs ``≥0``?), and a faithful transcendental anchor
    additionally needs a **named-constant** surface (Stefan-Boltzmann's σ is a CODATA
    *module constant*, not a param — see ``domains.thermal``), unresolved here. Kept
    explicit so the Rust port does not assume the grammar is complete.

**No ``dt`` token, by construction.** Exposing ``dt`` to the rate grammar would let an
author write ``coeff·dt·f(dt)`` — non-linear in ``dt`` — silently forfeiting RK4 order
(the very thing the increment-form contract in ``simcore.flow`` protects). The rate
expression is the **instantaneous rate** (``dt``-independent); :class:`DeclarativeFlow`
supplies the single ``× dt`` that makes the per-step increment. So RK4-order-safety is
*structural* for every authored flow, not a documented hope. ``n`` stays readable
(``dt``-independent, safe).

Pure stdlib only. The AST is plain data (frozen dataclasses of ``float``/``str``/nested
nodes), so the Step-4 Rust port is a mechanical mirror and cross-port parity is
tolerance-gated exactly like every other transcendental (Phase-7 3-tier contract): a
transcendental-free authored flow is Tier-1 bit-exact.
"""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass

from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.state import State

# --- the AST (plain data) ----------------------------------------------------
# Every node is a frozen dataclass of primitives / nested nodes, so the whole tree
# is immutable + hashable + equatable and ports mechanically to Rust. ``Expr`` is the
# closed node union (Step 2's arithmetic core); the evaluator dispatches on it.


@dataclass(frozen=True)
class Const:
    """A numeric literal (a ``float`` — the parser has already coerced it)."""

    value: float


@dataclass(frozen=True)
class StockRef:
    """A read of a stock's current amount, by id, from the evaluation snapshot.

    Reads the snapshot **directly** (``snapshot.stocks[id].amount``), the
    donor-controlled idiom (``SelfDischarge`` reads ``battery`` this way), *not* via
    ``env`` — a stock read and a forcing read are distinct grammar forms.
    """

    stock: StockId


@dataclass(frozen=True)
class ParamRef:
    """A read of a flow param by name (from the flow's own param map)."""

    name: str


@dataclass(frozen=True)
class ForcingRef:
    """A read of a forcing var by name, resolved through ``env.get`` (#16).

    Indistinguishable at evaluation time from a coupled sibling's shared stock —
    exactly ``env``'s contract; a forced authored flow reads its rate this way.
    """

    name: str


@dataclass(frozen=True)
class StepN:
    """A read of the integer step count ``n`` (as a ``float``); ``dt``-independent."""


@dataclass(frozen=True)
class Neg:
    """Unary negation ``- operand``."""

    operand: Expr


@dataclass(frozen=True)
class BinOp:
    """A binary arithmetic op ``left <op> right`` with ``op`` ∈ ``{"+", "-", "*"}``.

    ``left`` is evaluated before ``right`` and then combined — a fixed op-order the
    Rust port mirrors, so the result is bit-identical within a build.
    """

    op: str
    left: Expr
    right: Expr


@dataclass(frozen=True)
class Monod:
    """Saturating kinetics ``substrate / (substrate + half_saturation)``.

    The **one shape ``+ − ×`` cannot approximate**, and the most common functional form
    in the science this project models — Michaelis-Menten, Monod and Holling type II all
    share this algebra. Added by the post-roadmap Tier-2 grammar unfreeze
    (``docs/plans/post-roadmap-grammar-monod.md``).

    **A frozen flow forced the definition** (the grammar's own precondition): this is
    the kernel of ``domains.biosphere.chamber.oxygen_limitation_factor`` — frozen since
    Phase-2 Step 7, cited to Davidson et al. 2012, and used by three frozen flows. Both
    of the choices the grammar had to make were therefore *already made, and cited*:

    * **2-arg, dimensionless** (not a 3-arg ``Vmax·S/(S+K)``). The frozen ``f_O2`` is a
      dimensionless factor applied as ``daily · f_O2 · dt``, so ``Vmax`` arrives through
      the already-frozen ``*``. Arg order is ``monod(substrate, half_saturation)``,
      matching the frozen signature and Michaelis-Menten convention — a **frozen
      semantic choice**, pinned by a parse vector.
    * **``denom <= 0`` → ``0.0``**, mirroring the frozen line-for-line ("no O₂ ⇒ no
      respiration"). This makes the node **total**: for all finite inputs it returns a
      finite float — never NaN, never ±inf, never raising. So ``0/0`` cannot reach a
      hex-float golden, and the Python-raise-vs-Rust-``inf`` split never arises because
      no raw ``x/0`` is reachable.

    **Only the kernel is mirrored, not the frozen function's argument preparation.** The
    frozen ``max(0.0, o2_mol)/air_mol`` and its ``ValueError`` guards are *arg prep* for
    a depleting physical pool; for an authored rate that layer **is the sub-expressions
    the author composed**. Baking a silent ``max(0, ·)`` in here would change
    ``monod(stock("a") - stock("b"), k)`` invisibly — the silent-failure class this
    platform exists to avoid. Off the natural domain (``S ≥ 0``, ``K > 0``, where the
    result is the textbook ``[0, 1)`` and bit-identical to frozen ``f_O2``) the node
    stays total and yields conservation-closed nonsense the author owns, by design.

    **Tier-1 bit-exact, not Tier-2.** Division is an IEEE-754 *basic* operation —
    correctly-rounded and deterministic cross-port, exactly like ``+ − *``. The
    "a transcendental moves an authored flow Tier-1 → Tier-2" rule covers *libm* calls
    (``RadiatorReject``'s ``T⁴`` via ``pow``); this is not in that class.

    **RK4-order-safe.** ``S/(S+K)`` is C∞ on the natural domain, so RK4's convergence
    order survives — which the obvious cheap alternative ``min(k·S, Vmax)`` would
    destroy (its kink is non-differentiable). The ``denom <= 0`` branch is a derivative
    discontinuity only at the pathological boundary, which ``K > 0`` never reaches.
    """

    substrate: Expr
    half_saturation: Expr


Expr = Const | StockRef | ParamRef | ForcingRef | StepN | Neg | BinOp | Monod

# The op set Step 2 ships (unambiguous IEEE arithmetic). Division and the function
# set are deferred (see the module docstring); the parser and evaluator both reject
# anything outside this set, so an unimplemented op can never silently evaluate.
_BINARY_OPS: frozenset[str] = frozenset({"+", "-", "*"})


def eval_expr(
    node: Expr,
    snapshot: State,
    env: Environment,
    params: Mapping[str, float],
) -> float:
    """Evaluate ``node`` to a ``float`` against a snapshot/env/param context.

    Pure and deterministic in its inputs. Reference resolution mirrors the frozen
    flows: ``StockRef`` reads the snapshot directly (#16 donor read), ``ForcingRef``
    goes through ``env.get`` (#16 forcing/shared read), ``ParamRef`` reads the flow's
    param map, ``StepN`` reads ``snapshot.n``. A ``StockRef`` at a missing id raises
    ``KeyError`` and a ``ParamRef`` at a missing name raises ``KeyError`` — referential
    integrity is validated at *build* time by the interpreter (an ``AuthoringError``),
    so these are belt-and-suspenders, never the primary guard.

    ``dt`` is intentionally absent from the signature: the rate grammar has no ``dt``
    token (see the module docstring), so a rate expression cannot depend on ``dt``.
    """
    if isinstance(node, Const):
        return node.value
    if isinstance(node, StockRef):
        return snapshot.stocks[node.stock].amount
    if isinstance(node, ParamRef):
        return params[node.name]
    if isinstance(node, ForcingRef):
        return env.get(node.name)
    if isinstance(node, StepN):
        return float(snapshot.n)
    if isinstance(node, Neg):
        return -eval_expr(node.operand, snapshot, env, params)
    if isinstance(node, BinOp):
        left = eval_expr(node.left, snapshot, env, params)
        right = eval_expr(node.right, snapshot, env, params)
        if node.op == "+":
            return left + right
        if node.op == "-":
            return left - right
        if node.op == "*":
            return left * right
        # Unreachable for a well-formed AST (the parser only emits _BINARY_OPS and the
        # interpreter re-validates), but kept explicit so a malformed op is a loud
        # error rather than a silent wrong answer.
        raise ValueError(f"unsupported binary op {node.op!r}")
    if isinstance(node, Monod):
        # substrate before half_saturation — the same fixed left-to-right order BinOp
        # uses, mirrored by the Rust port.
        substrate = eval_expr(node.substrate, snapshot, env, params)
        half_saturation = eval_expr(node.half_saturation, snapshot, env, params)
        denom = substrate + half_saturation
        if denom <= 0.0:
            # The frozen f_O2's own choice, verbatim ("no O₂ ⇒ no respiration"): the
            # degenerate 0/0 returns 0 rather than NaN, which is what makes this node
            # total. Also catches a negative denominator (an author's negative K), where
            # S/denom would otherwise flip sign.
            return 0.0
        return substrate / denom
    raise TypeError(f"not an Expr node: {node!r}")  # pragma: no cover - exhaustive


@dataclass(frozen=True)
class DeclarativeFlow:
    """An authored ``Flow``: an instantaneous *rate* expression × a fixed stoichiometry.

    **Balanced by construction (decision C).** The flow emits one leg per
    ``(stock, coeff)`` pair, all sharing the single scalar ``increment = rate·dt``, so
    per conserved quantity ``Σ legs = rate·dt · Σ(coeff · composition)`` — which is
    ``0`` for *any* rate value **iff** the stoichiometry's coefficient vector balances.
    The author picks only the scalar rate and the (integer/rational) coefficients; they
    cannot vary the per-leg magnitude independently, so a *balanced* coefficient vector
    stays balanced at every step regardless of state. The interpreter validates that
    coefficient vector against the stock compositions at build time (relative tolerance,
    mirroring ``assert_flow_balanced``); the every-step conservation gate is then a
    redundant backstop. (For integer coefficients like ``−1/+1`` the balance is exact;
    for fractional split coefficients ``f/(1−f)`` it holds to floating tolerance, like
    the frozen ``charge_split`` — so "structural" is exact for integer coeffs, and
    tolerance-backed for fractional ones.)

    **Increment-form (RK4-order-safe).** ``rate`` is the ``dt``-independent
    instantaneous rate; ``evaluate`` forms ``increment = rate · dt`` and each leg is
    ``coeff · increment``. This is the standard ``flux → × dt`` split every frozen flow
    uses, so RK4's ⅙-combine reproduces classical RK4 exactly. ``rate`` has no ``dt``
    token (grammar-enforced), so ``dt``-linearity is structural.

    ``params`` is stored as a sorted ``(name, value)`` tuple (immutable + hashable, the
    frozen-flow idiom) and exposed to the evaluator as a mapping; ``stoichiometry`` is a
    ``(stock, coeff)`` tuple in author order (leg order does not affect the trajectory —
    each leg lands on a distinct stock and the integrator sums per stock).
    """

    id: FlowId
    priority: int
    rate: Expr
    stoichiometry: tuple[tuple[StockId, float], ...]
    params: tuple[tuple[str, float], ...]

    def _param_map(self) -> dict[str, float]:
        return {name: value for name, value in self.params}

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        rate = eval_expr(self.rate, snapshot, env, self._param_map())
        increment = rate * dt
        return FlowResult(
            legs=tuple(
                Leg(stock, coeff * increment) for stock, coeff in self.stoichiometry
            )
        )
