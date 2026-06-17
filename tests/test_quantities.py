"""Step-2 tests for quantities, stock kinds, and the canonical-unit table."""

from simcore.quantities import CANONICAL_UNIT, Quantity, StockKind, canonical_unit


def test_quantity_members_exact() -> None:
    assert {q.name for q in Quantity} == {
        "CARBON",
        "WATER",
        "NITROGEN",
        "OXYGEN",
        "ENERGY",
    }


def test_phosphorus_is_reserved_not_a_member() -> None:
    # PHOSPHORUS is reserved as a comment, not an enum member (frozen API).
    assert "PHOSPHORUS" not in Quantity.__members__


def test_stock_kind_members_exact() -> None:
    assert {k.name for k in StockKind} == {"POOL", "POPULATION", "BOUNDARY"}


def test_canonical_unit_table_is_total() -> None:
    # Every quantity must have a canonical unit — the conservation loop iterates
    # all quantities, so a missing entry would be a latent bug.
    assert set(CANONICAL_UNIT) == set(Quantity)


def test_canonical_unit_accessor_matches_table() -> None:
    for quantity in Quantity:
        assert canonical_unit(quantity) == CANONICAL_UNIT[quantity]


def test_canonical_units_are_nonempty_labels() -> None:
    assert all(isinstance(u, str) and u for u in CANONICAL_UNIT.values())


def test_canonical_units_are_science_correct() -> None:
    # Phase-1 Step-1 resolves the Phase-0 PROVISIONAL table to science-correct labels.
    #   CARBON=mol, ENERGY=J are GOLDEN-LOCKED (the committed demo goldens carry them;
    #   changing either forces regenerating those goldens).
    #   WATER=kg, NITROGEN=kg are mass-basis: kg H2O matches Penman-Monteith
    #   (mm/day = kg m^-2 day^-1); kg N is unambiguous element mass (WOFOST-native,
    #   unlike species-ambiguous "mol N").
    #   OXYGEN stays mol (untracked in Phase 1; molar keeps gas species consistent
    #   for the deferred Phase-2 stoichiometry).
    assert CANONICAL_UNIT[Quantity.CARBON] == "mol"
    assert CANONICAL_UNIT[Quantity.ENERGY] == "J"
    assert CANONICAL_UNIT[Quantity.WATER] == "kg"
    assert CANONICAL_UNIT[Quantity.NITROGEN] == "kg"
    assert CANONICAL_UNIT[Quantity.OXYGEN] == "mol"
