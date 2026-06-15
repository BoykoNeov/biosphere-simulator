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
