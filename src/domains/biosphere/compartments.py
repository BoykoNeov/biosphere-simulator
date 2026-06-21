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
(``season.py``) reads when it assigns each stock to its compartment.

Pure stdlib only (a plain mapping + a frozenset rollup).
"""

from collections.abc import Mapping

from simcore.ids import DomainId, StockId

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
