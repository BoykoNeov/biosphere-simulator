"""Phase-3 P3.1 tests: the subsystem-hierarchy representation (domain-side).

Covers ``domains.biosphere.compartments`` — the leaf-compartment parent map and the
``descendant_stocks`` rollup — plus the invariant that the Phase-3 relabel partitions
the *modeled* biosphere stocks across the four leaf compartments while leaving boundary
stocks in the ``boundary`` namespace.

The **bit-identity** half of P3.1 (the relabel changes only ``domain`` labels — goldens
regenerate with domain-label-only diffs and byte-identical amounts) is pinned by the
existing ``test_regression_season`` / ``test_regression_sealed_season`` byte-compares;
this file covers the new *structure* (the hierarchy view + the stock partition).
"""

from collections.abc import Mapping

from domains.biosphere.compartments import (
    ATMOSPHERE,
    BIOSPHERE,
    BIOSPHERE_PARENTS,
    PLANTS,
    SOIL,
    WATER,
    descendant_stocks,
)
from domains.biosphere.season import (
    SEALED_CHAMBER_SCENARIO,
    STOCK_DOMAIN,
    build_season,
)
from simcore.boundary import BOUNDARY_DOMAIN
from simcore.ids import DomainId, StockId
from simcore.state import Stock

LEAVES = frozenset({ATMOSPHERE, SOIL, PLANTS, WATER})


def _modeled_ids(stocks: Mapping[StockId, Stock]) -> frozenset[StockId]:
    """Every non-boundary (modeled) stock id in a built state."""
    return frozenset(sid for sid, s in stocks.items() if s.domain != BOUNDARY_DOMAIN)


# --- the parent map ---------------------------------------------------------


def test_parent_map_is_flat_two_level_tree_under_biosphere() -> None:
    # Exactly the four leaves are declared, each parented to the biosphere root.
    assert set(BIOSPHERE_PARENTS) == set(LEAVES)
    assert set(BIOSPHERE_PARENTS.values()) == {BIOSPHERE}
    # The root is not itself a leaf (it owns stocks only transitively).
    assert BIOSPHERE not in BIOSPHERE_PARENTS


# --- the relabel partition (sealed = all twelve modeled stocks) -------------


def test_relabel_partitions_modeled_stocks_into_leaves() -> None:
    state, _ = build_season(SEALED_CHAMBER_SCENARIO)
    modeled = _modeled_ids(state.stocks)
    # The sealed build instantiates exactly the modeled stocks the table assigns.
    assert modeled == set(STOCK_DOMAIN)
    # Each modeled stock carries the leaf its table entry names; every leaf is declared.
    for sid in modeled:
        assert state.stocks[sid].domain == STOCK_DOMAIN[sid]
        assert STOCK_DOMAIN[sid] in LEAVES
    # No modeled stock keeps the bare ``biosphere`` root label, and boundary stocks are
    # untouched (still in the boundary namespace).
    assert all(state.stocks[sid].domain != BIOSPHERE for sid in modeled)
    boundary_ids = set(state.stocks) - modeled
    assert boundary_ids  # the build has boundary stocks
    assert all(state.stocks[sid].domain == BOUNDARY_DOMAIN for sid in boundary_ids)


# --- descendant_stocks rollup -----------------------------------------------


def test_descendant_stocks_root_unions_all_leaves() -> None:
    state, registry = build_season(SEALED_CHAMBER_SCENARIO)
    di = registry.domain_index
    root = descendant_stocks(di, BIOSPHERE_PARENTS, BIOSPHERE)
    # The root rolls up the union of every leaf's members ...
    leaves_union = frozenset().union(*(di.get(leaf, frozenset()) for leaf in LEAVES))
    assert root == leaves_union
    # ... which is exactly every modeled stock in the build.
    assert root == _modeled_ids(state.stocks)


def test_descendant_stocks_leaf_equals_its_own_members() -> None:
    _, registry = build_season(SEALED_CHAMBER_SCENARIO)
    di = registry.domain_index
    # A leaf has no children, so its rollup is just its own members (== domain_index).
    for leaf in (PLANTS, SOIL, ATMOSPHERE):
        assert descendant_stocks(di, BIOSPHERE_PARENTS, leaf) == di[leaf]


def test_water_compartment_is_empty_until_step3() -> None:
    _, registry = build_season(SEALED_CHAMBER_SCENARIO)
    di = registry.domain_index
    # No stock lives in ``water`` yet (water_vapor / condensate arrive in Step 3), so it
    # is absent from the index and rolls up to nothing — without error.
    assert WATER not in di
    assert descendant_stocks(di, BIOSPHERE_PARENTS, WATER) == frozenset()


def test_flat_default_reproduces_domain_index() -> None:
    _, registry = build_season(SEALED_CHAMBER_SCENARIO)
    di = registry.domain_index
    # With no parents every domain is childless, so the rollup degenerates to the raw
    # ``domain_index`` lookup — today's pre-hierarchy behavior.
    for leaf in (PLANTS, SOIL, ATMOSPHERE):
        assert descendant_stocks(di, {}, leaf) == di.get(leaf, frozenset())
    # The root owns no stocks directly, so flat-default it rolls up to nothing.
    assert descendant_stocks(di, {}, BIOSPHERE) == frozenset()


def test_descendant_stocks_open_field_is_a_subset() -> None:
    state, registry = build_season()  # open field: no atmosphere/decomposer pools
    di = registry.domain_index
    root = descendant_stocks(di, BIOSPHERE_PARENTS, BIOSPHERE)
    assert root == _modeled_ids(state.stocks)
    # The open field has no atmosphere stocks (carbon_pool/o2_pool are sealed only), so
    # the rollup is a strict subset of the full table.
    assert ATMOSPHERE not in di
    assert root < set(STOCK_DOMAIN)


# --- determinism / robustness ----------------------------------------------


def test_descendant_stocks_is_a_frozenset_and_order_independent() -> None:
    _, registry = build_season(SEALED_CHAMBER_SCENARIO)
    di = registry.domain_index
    result = descendant_stocks(di, BIOSPHERE_PARENTS, BIOSPHERE)
    assert isinstance(result, frozenset)
    # Shuffling the parent-map insertion order yields a bit-identical rollup (#15).
    reordered = dict(reversed(list(BIOSPHERE_PARENTS.items())))
    assert descendant_stocks(di, reordered, BIOSPHERE) == result


def test_descendant_stocks_tolerates_a_malformed_cycle() -> None:
    # A cyclic parent map (a wiring bug) must not hang the rollup — the visited guard
    # terminates it. Two leaves pointing at each other, with stocks in one.
    a, b = DomainId("x.a"), DomainId("x.b")
    di = {a: frozenset({StockId("x.s")})}
    assert descendant_stocks(di, {a: b, b: a}, b) == frozenset({StockId("x.s")})
