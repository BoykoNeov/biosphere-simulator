"""Phase-3 P3.1 tests: the subsystem-hierarchy representation (domain-side).

Covers ``domains.biosphere.compartments`` — the leaf-compartment parent map, the
``descendant_stocks`` rollup, and the **per-compartment boundary ledger** diagnostic —
plus the invariant that the Phase-3 relabel partitions the *modeled* biosphere stocks
across the four leaf compartments while leaving boundary stocks in the ``boundary``
namespace.

The **bit-identity** half of P3.1 (the relabel changes only ``domain`` labels — goldens
regenerate with domain-label-only diffs and byte-identical amounts) is pinned by the
existing ``test_regression_season`` / ``test_regression_sealed_season`` byte-compares;
this file covers the new *structure* (the hierarchy view + the stock partition) and the
boundary-ledger diagnostic.
"""

from collections.abc import Mapping

from domains.biosphere.compartments import (
    ATMOSPHERE,
    BIOSPHERE,
    BIOSPHERE_PARENTS,
    PLANTS,
    SOIL,
    WATER,
    CompartmentFlux,
    compartment_boundary_ledger,
    descendant_stocks,
)
from domains.biosphere.season import (
    CONDENSATE,
    SEALED_CHAMBER_SCENARIO,
    STOCK_DOMAIN,
    WATER_VAPOR,
    build_season,
)
from simcore.boundary import BOUNDARY_DOMAIN
from simcore.flow import FlowResult, Leg
from simcore.ids import DomainId, StockId
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import State, Stock

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


# --- the relabel partition (sealed = all fourteen modeled stocks) -----------
# (12 carbon/N/water modeled stocks from Phases 1/2 + the two Step-3 water-cycle stocks
# ``water_vapor`` (atmosphere) and ``condensate`` (water).)


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
    # All four leaves are populated in the sealed chamber (water gained ``condensate``
    # in Step 3).
    for leaf in (PLANTS, SOIL, ATMOSPHERE, WATER):
        assert descendant_stocks(di, BIOSPHERE_PARENTS, leaf) == di[leaf]


def test_water_compartment_populated_when_sealed_empty_when_open() -> None:
    # Step 3 closes the water cycle, so the ``water`` leaf — declared empty since P3.1 —
    # gains ``condensate`` in the sealed chamber (``water_vapor`` lands in atmosphere).
    sealed, sealed_reg = build_season(SEALED_CHAMBER_SCENARIO)
    di = sealed_reg.domain_index
    assert di[WATER] == frozenset({CONDENSATE})
    assert descendant_stocks(di, BIOSPHERE_PARENTS, WATER) == frozenset({CONDENSATE})
    assert (
        sealed.stocks[WATER_VAPOR].domain == ATMOSPHERE
    )  # vapor is an atmosphere stock
    # The open field does not close the cycle, so the water leaf stays empty: absent
    # from the index, it rolls up to nothing — without error.
    _, open_reg = build_season()
    open_di = open_reg.domain_index
    assert WATER not in open_di
    assert descendant_stocks(open_di, BIOSPHERE_PARENTS, WATER) == frozenset()


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


# --- the per-compartment boundary ledger (diagnostic) -----------------------

_P = StockId("test.plant_c")
_A = StockId("test.atmos_c")
_POOL = StockId("test.co2_pool")
_S1 = StockId("test.soil_c1")
_S2 = StockId("test.soil_c2")


def _carbon(sid: StockId, domain: DomainId, amount: float) -> Stock:
    """A 1:1 CARBON POOL stock (composition defaults to ``{CARBON: 1.0}``)."""
    return Stock(
        id=sid,
        domain=domain,
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=amount,
        kind=StockKind.POOL,
    )


def _co2_pool(sid: StockId, domain: DomainId, amount: float) -> Stock:
    """A CO₂ POOL carrying ``{CARBON: 1, OXYGEN: 2}`` (the composition-fold case)."""
    return Stock(
        id=sid,
        domain=domain,
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=amount,
        kind=StockKind.POOL,
        composition={Quantity.CARBON: 1.0, Quantity.OXYGEN: 2.0},
    )


def _state(*stocks: Stock) -> State:
    return State(n=0, stocks={s.id: s for s in stocks}, rng_seed=0)


def _by_key(
    ledger: tuple[CompartmentFlux, ...],
) -> dict[tuple[DomainId, Quantity], CompartmentFlux]:
    return {(e.domain, e.quantity): e for e in ledger}


def test_two_compartment_transfer_reports_crossing_flux_and_balances() -> None:
    # A balanced carbon transfer plants -> atmosphere (2 mol C). The crossing flux
    # matches the transfer on both sides and the apply-integrity residual is ~0.
    before = _state(_carbon(_P, PLANTS, 10.0), _carbon(_A, ATMOSPHERE, 5.0))
    after = _state(_carbon(_P, PLANTS, 8.0), _carbon(_A, ATMOSPHERE, 7.0))
    result = FlowResult(legs=(Leg(_P, -2.0), Leg(_A, 2.0)))

    ledger = compartment_boundary_ledger(before, after, [result])
    by = _by_key(ledger)
    plants = by[(PLANTS, Quantity.CARBON)]
    atmos = by[(ATMOSPHERE, Quantity.CARBON)]

    assert (plants.crossing_in, plants.crossing_out, plants.stored_delta) == (
        0.0,
        2.0,
        -2.0,
    )
    assert (atmos.crossing_in, atmos.crossing_out, atmos.stored_delta) == (
        2.0,
        0.0,
        2.0,
    )
    assert all(abs(e.residual) < 1e-12 for e in ledger)
    # Canonical order: domain id then quantity name (atmosphere before plants).
    assert [str(e.domain) for e in ledger] == [
        "biosphere.atmosphere",
        "biosphere.plants",
    ]


def test_internal_flow_contributes_zero_crossing_flux() -> None:
    # A flow with both legs inside soil crosses no boundary: zero crossing flux, and the
    # balanced internal transfer leaves the compartment's stored carbon unchanged.
    before = _state(_carbon(_S1, SOIL, 10.0), _carbon(_S2, SOIL, 0.0))
    after = _state(_carbon(_S1, SOIL, 7.0), _carbon(_S2, SOIL, 3.0))
    result = FlowResult(legs=(Leg(_S1, -3.0), Leg(_S2, 3.0)))

    ledger = compartment_boundary_ledger(before, after, [result])
    soil = _by_key(ledger)[(SOIL, Quantity.CARBON)]
    assert soil.crossing_in == 0.0 and soil.crossing_out == 0.0
    assert soil.stored_delta == 0.0
    assert abs(soil.residual) < 1e-12


def test_balanced_but_misapplied_delta_trips_the_identity() -> None:
    # The legs request a 2 mol plants->atmosphere transfer, but the apply moved only
    # 1.5 each way — two compensating errors that net to zero GLOBALLY (total carbon 15
    # preserved). The global gate would pass; the per-compartment apply-integrity
    # residual catches it.
    before = _state(_carbon(_P, PLANTS, 10.0), _carbon(_A, ATMOSPHERE, 5.0))
    after = _state(_carbon(_P, PLANTS, 8.5), _carbon(_A, ATMOSPHERE, 6.5))
    result = FlowResult(legs=(Leg(_P, -2.0), Leg(_A, 2.0)))

    ledger = compartment_boundary_ledger(before, after, [result])
    by = _by_key(ledger)
    # Global mass is still conserved ...
    assert abs(sum(e.stored_delta for e in ledger)) < 1e-12
    # ... yet each compartment's residual is nonzero (the local catch).
    assert abs(by[(PLANTS, Quantity.CARBON)].residual) == 0.5
    assert abs(by[(ATMOSPHERE, Quantity.CARBON)].residual) == 0.5


def test_composition_fold_books_both_carbon_and_oxygen() -> None:
    # A leg on a CO₂ pool books CARBON·1 AND OXYGEN·2 of crossing flux (the same fold
    # the global gate uses). The plant-organ side carries only CARBON.
    organ = StockId("test.leaf_c")
    before = _state(_carbon(organ, PLANTS, 10.0), _co2_pool(_POOL, ATMOSPHERE, 4.0))
    after = _state(_carbon(organ, PLANTS, 9.0), _co2_pool(_POOL, ATMOSPHERE, 5.0))
    result = FlowResult(legs=(Leg(organ, -1.0), Leg(_POOL, 1.0)))

    ledger = compartment_boundary_ledger(before, after, [result])
    by = _by_key(ledger)
    assert by[(ATMOSPHERE, Quantity.CARBON)].crossing_in == 1.0  # 1 · coeff_C(1)
    assert by[(ATMOSPHERE, Quantity.OXYGEN)].crossing_in == 2.0  # 1 · coeff_O(2) — fold
    assert abs(by[(ATMOSPHERE, Quantity.CARBON)].residual) < 1e-12
    assert abs(by[(ATMOSPHERE, Quantity.OXYGEN)].residual) < 1e-12
    # The plant organ carries no oxygen, so there is no (plants, OXYGEN) entry.
    assert (PLANTS, Quantity.OXYGEN) not in by
    assert (ATMOSPHERE, Quantity.OXYGEN) in by


def test_boundary_ledger_is_leg_order_independent() -> None:
    # Folded in canonical stock-id order: a shuffled leg tuple yields an equal ledger.
    before = _state(_carbon(_P, PLANTS, 10.0), _carbon(_A, ATMOSPHERE, 5.0))
    after = _state(_carbon(_P, PLANTS, 8.0), _carbon(_A, ATMOSPHERE, 7.0))
    forward = compartment_boundary_ledger(
        before, after, [FlowResult(legs=(Leg(_P, -2.0), Leg(_A, 2.0)))]
    )
    reversed_legs = compartment_boundary_ledger(
        before, after, [FlowResult(legs=(Leg(_A, 2.0), Leg(_P, -2.0)))]
    )
    assert forward == reversed_legs
