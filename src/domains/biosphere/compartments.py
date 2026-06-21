"""The biosphere subsystem hierarchy (Phase-3 P3.1) — domain-side, ``simcore``-free.

Phase 3 splits the monolithic ``biosphere`` domain into four **leaf compartments** —
``atmosphere`` / ``soil`` / ``plants`` / ``water`` — under a ``biosphere`` root::

    biosphere
      |- biosphere.atmosphere
      |- biosphere.soil
      |- biosphere.plants
      `- biosphere.water

A compartment is **a ``DomainId`` namespace plus its stock membership and a parent**
— *not* a rich class and *never* a sub-solver (the integrator stays global; one clock,
one ledger, one conservation gate — see ``docs/plans/phase-3-modular-biosphere.md``
P3.1). The split moves only each stock's ``Stock.domain`` **label** (no stock/flow id
is renamed), so every float reduction — which keys on stock id / quantity name, never
``domain`` — is byte-stable and the goldens regenerate with **domain-label-only diffs
and bit-identical amounts** (the proof the restructure was behavior-preserving).

**Why this lives outside ``simcore`` (the resolved P3.1 build decision — Option B).**
The parent map and the hierarchy view that reads it are **pure reporting metadata**:
the integrator, conservation gate, arbitration, and resolver never touch them. Holding
them domain-side (rather than as a ``Registry(parents=...)`` field) keeps ``simcore``
literally untouched and is the reversible branch — adding the map to the registry later,
*if* an in-core consumer ever appears, is additive; removing it from a frozen surface
would be breaking. **Coupling rule:** the boundary-ledger helper that consumes this map
must also live outside ``simcore`` (it does — below); never split the map out but its
consumer in. Acceptance: ``git diff src/simcore/`` shows zero new symbols.

The leaf-``DomainId`` constants are the single source of truth the scenario
(``season.py``) reads when it assigns each stock to its compartment. This module also
hosts the **per-compartment boundary ledger** — the diagnostic that consumes the
hierarchy (the resolved coupling rule: the map *and* its consumer stay outside
``simcore``).

Pure stdlib only (a plain mapping, a frozenset rollup, and a per-step delta fold).
"""

from collections.abc import Iterable, Mapping
from dataclasses import dataclass

from simcore.flow import FlowResult
from simcore.ids import DomainId, StockId
from simcore.quantities import Quantity
from simcore.state import State

# --- the hierarchy nodes ----------------------------------------------------
# The root grouping node. After the Phase-3 relabel NO stock carries this domain
# directly — ``biosphere`` owns its stocks transitively, through the four leaves.
BIOSPHERE: DomainId = DomainId("biosphere")
# The four leaf compartments. ``water`` is declared up front but holds no stocks
# until Step 3 closes the water cycle (``water_vapor`` / ``condensate``).
ATMOSPHERE: DomainId = DomainId("biosphere.atmosphere")
SOIL: DomainId = DomainId("biosphere.soil")
PLANTS: DomainId = DomainId("biosphere.plants")
WATER: DomainId = DomainId("biosphere.water")

# The parent map: leaf-domain -> parent-domain. A flat two-level tree for now; deepens
# as compartments gain sub-structure (and the helpers below are already transitive, so
# a deeper tree needs no code change). This is the descriptor that, under the rejected
# Option A, would be ``Registry(parents=...)``; here it is plain domain-side data.
BIOSPHERE_PARENTS: Mapping[DomainId, DomainId] = {
    ATMOSPHERE: BIOSPHERE,
    SOIL: BIOSPHERE,
    PLANTS: BIOSPHERE,
    WATER: BIOSPHERE,
}


def descendant_stocks(
    domain_index: Mapping[DomainId, frozenset[StockId]],
    parents: Mapping[DomainId, DomainId],
    root: DomainId,
) -> frozenset[StockId]:
    """Union of the stock sets of ``root`` and **all its transitive descendants**.

    ``domain_index`` is ``Registry.domain_index`` (read off the public API — never the
    reverse); ``parents`` is a leaf->parent map (e.g. :data:`BIOSPHERE_PARENTS`). With
    the flat default ``parents={}`` a domain has no children, so this reduces to
    ``domain_index.get(root, frozenset())`` — i.e. today's pre-hierarchy behavior.

    The result is a ``frozenset`` (order-independent), so it is deterministic regardless
    of mapping iteration order (#15). A domain absent from ``domain_index`` (e.g. an
    empty compartment like ``water`` before Step 3) contributes nothing.
    """
    # parent -> children adjacency, derived once.
    children: dict[DomainId, list[DomainId]] = {}
    for child, parent in parents.items():
        children.setdefault(parent, []).append(child)

    collected: set[StockId] = set()
    # Iterative DFS over the subtree rooted at ``root`` (guards against a malformed
    # cycle in ``parents`` via the visited set, rather than recursing unboundedly).
    stack: list[DomainId] = [root]
    visited: set[DomainId] = set()
    while stack:
        node = stack.pop()
        if node in visited:
            continue
        visited.add(node)
        collected |= domain_index.get(node, frozenset())
        stack.extend(children.get(node, ()))
    return frozenset(collected)


# --- the per-compartment boundary ledger (diagnostic) -----------------------
# The roadmap wants ``Inputs = Outputs + ΔStored`` to hold *per subsystem*, while
# conservation stays enforced *globally, never per-domain*. Reconciled (P3.1): the
# global every-step gate stays the only enforcement; this is a **diagnostic** computed
# from the same legs + state deltas, surfaced in reports / asserted in tests, never
# aborting a step. Its honest value is two things: (1) per-boundary **flux reporting**
# — "net carbon plants→atmosphere this step", the payoff that makes a 4-compartment
# system debuggable; (2) **local apply-integrity** — the identity below trips on a
# *balanced-but-misapplied* delta (compensating misapplications that net to zero
# globally). It does NOT catch a flow wired into the wrong compartment — that is a
# separate *behavioral* assertion (per cross-compartment flow, Step 3+), because both
# sides of the identity move together under a mislabel.


@dataclass(frozen=True)
class CompartmentFlux:
    """One ``(compartment, quantity)``'s boundary accounting for a step (diagnostic).

    - ``crossing_in`` / ``crossing_out`` — gross folded flux **in / out** of the
      compartment carried by *crossing* flows (those touching more than one domain);
      legs of fully-internal flows are excluded (they cancel, contributing nothing).
      These are **gross**: a multi-leg crossing flow that also redistributes *within*
      the compartment (e.g. ``Allocation`` shuffling plant organs) inflates both sides
      equally. The reporting payoff ("net carbon plants→atmosphere") is the **net**
      ``crossing_in − crossing_out``; the gross pair is the audit detail behind it.
    - ``stored_delta`` — folded ``Σ (after − before)`` over the compartment's stocks.
    - ``residual`` — ``(crossing_in − crossing_out) − stored_delta``, the
      **apply-integrity** check: ``net crossing == ΔStored`` because internal flows
      cancel, so a nonzero residual means a stock moved by an amount no leg accounts
      for. It is *not* a wiring check (a mislabel moves both sides together).

      **Holds only on a "clean" step** — ``flow_results`` are the **post-arbitration**
      legs (a backstop-scaled flow's *applied* delta is its scaled leg, not the raw
      ``evaluate()`` result) **and** no **non-flow** state change occurred. Extinction
      routing (POPULATION → loss-sink, #6) is a *balanced non-flow* change that
      ``after − before`` includes but **no leg** does, so an extinction step
      legitimately yields ``residual ≠ 0`` for that stock's compartment. Such steps are
      **expected exceptions to handle when this is wired live (Step 5)** — not bugs.
      The Phase-1/2 goldens are ``rationed == 0`` + extinction-free, so neither arises.

    Folds element composition exactly as the global gate (``compute_ledger`` /
    ``flow.per_quantity_residual``): a CO₂-pool leg books both CARBON and OXYGEN.
    """

    domain: DomainId
    quantity: Quantity
    crossing_in: float
    crossing_out: float
    stored_delta: float
    residual: float


def compartment_boundary_ledger(
    before: State,
    after: State,
    flow_results: Iterable[FlowResult],
) -> tuple[CompartmentFlux, ...]:
    """Per-``(compartment, quantity)`` boundary ledger for the step ``before → after``.

    ``flow_results`` are the step's evaluated flows (the integrator yields them in
    canonical flow-id order; cross-flow sums fold in that order, mirroring the engine's
    own ``_reduce``). Compartment membership and element composition are read from the
    ``before`` snapshot's stocks. Entries cover every ``(domain, quantity)`` present and
    are returned in canonical order (domain id, then quantity name) — deterministic
    regardless of leg/stock construction order (#15).

    **Diagnostic only** — it never raises; callers assert ``abs(residual) <= tol`` (the
    apply-integrity check) or read the crossing flux for reporting. That identity holds
    exactly only on a **clean step** (post-arbitration legs, no non-flow routing) — see
    :class:`CompartmentFlux` for when it legitimately does not hold. A leg naming an
    unknown stock raises ``KeyError`` (referential integrity is the apply path's job).
    """
    stocks = before.stocks
    crossing_in: dict[tuple[DomainId, Quantity], float] = {}
    crossing_out: dict[tuple[DomainId, Quantity], float] = {}
    for result in flow_results:
        footprint = frozenset(stocks[leg.stock].domain for leg in result.legs)
        if len(footprint) <= 1:
            continue  # a fully-internal flow crosses no compartment boundary
        # Fold legs in canonical stock-id order (#15); a leg books each quantity its
        # stock carries (composition fold), into in/out by the folded sign.
        for leg in sorted(result.legs, key=lambda leg: leg.stock):
            domain = stocks[leg.stock].domain
            for quantity, coeff in stocks[leg.stock].composition.items():
                folded = leg.amount * coeff
                key = (domain, quantity)
                if folded >= 0.0:
                    crossing_in[key] = crossing_in.get(key, 0.0) + folded
                else:
                    crossing_out[key] = crossing_out.get(key, 0.0) - folded
    stored: dict[tuple[DomainId, Quantity], float] = {}
    for sid in sorted(before.stocks):
        b = before.stocks[sid]
        delta = after.stocks[sid].amount - b.amount
        for quantity, coeff in b.composition.items():
            key = (b.domain, quantity)
            stored[key] = stored.get(key, 0.0) + delta * coeff
    keys = sorted(
        set(crossing_in) | set(crossing_out) | set(stored),
        key=lambda k: (str(k[0]), k[1].name),
    )
    return tuple(
        CompartmentFlux(
            domain=domain,
            quantity=quantity,
            crossing_in=crossing_in.get((domain, quantity), 0.0),
            crossing_out=crossing_out.get((domain, quantity), 0.0),
            stored_delta=stored.get((domain, quantity), 0.0),
            residual=(
                crossing_in.get((domain, quantity), 0.0)
                - crossing_out.get((domain, quantity), 0.0)
                - stored.get((domain, quantity), 0.0)
            ),
        )
        for domain, quantity in keys
    )
