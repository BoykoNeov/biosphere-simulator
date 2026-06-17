"""Conserved quantities, stock kinds, and the canonical-unit table.

This is part of the frozen Phase-0 API (see ``docs/plans/phase-0-engine-skeleton``).
Everything here is plain stdlib data — no third-party imports.
"""

from enum import Enum

from simcore.ids import UnitLabel


class Quantity(Enum):
    """A conserved quantity tracked by the ledger.

    The conservation gate balances each member independently every step. Add a
    member only when a real stock and the matching flow stoichiometry exist for
    it — every per-quantity loop and the canonical-unit table below must cover
    each member, so an unused quantity is dead weight.

    ``PHOSPHORUS`` is intentionally *reserved as a comment*, not a member: the
    design anticipates it, but it has no Phase-0 stock or unit. (Matches the
    frozen API, which lists it as ``# PHOSPHORUS reserved``.)
    """

    CARBON = "carbon"
    WATER = "water"
    NITROGEN = "nitrogen"
    OXYGEN = "oxygen"
    ENERGY = "energy"
    # PHOSPHORUS reserved — add when a phosphorus stock + flow actually exist.


class StockKind(Enum):
    """How a stock behaves under arbitration and extinction.

    - ``POOL``: a resource pool (e.g. atmospheric carbon, water). Never
      zeroed-with-loss; arbitration may throttle draws against it.
    - ``POPULATION``: absorbing-eligible biomass/population. May go extinct —
      below its ``extinction_threshold`` it snaps to 0 and the residual is routed
      to the numerical-loss boundary sink (decision #6).
    - ``BOUNDARY``: an "outside" reservoir. Its per-step delta *is* a ledger
      Input/Output (decision #13). May be flagged ``unclamped`` (e.g. solar) so
      arbitration's min-scaling never throttles it.
    """

    POOL = "pool"
    POPULATION = "population"
    BOUNDARY = "boundary"


# Canonical-unit table — the single shared source of truth (decision #9). The
# core carries only these *labels*; the outer ``config`` loader validates and
# converts incoming params against them with pint.
#
# PROVISIONAL (step-2 note): the specific unit chosen per quantity does NOT
# affect any Phase-0 invariant — conservation arithmetic only requires that every
# stock of a given quantity shares one consistent canonical unit, regardless of
# which. Picking the science-correct units (mol vs kg, dry-mass basis, ...) is a
# Phase-1 decision. Until then these are placeholders; do not read significance
# into mol-vs-kg here.
CANONICAL_UNIT: dict[Quantity, UnitLabel] = {
    Quantity.CARBON: UnitLabel("mol"),
    Quantity.WATER: UnitLabel("mol"),
    Quantity.NITROGEN: UnitLabel("mol"),
    Quantity.OXYGEN: UnitLabel("mol"),
    Quantity.ENERGY: UnitLabel("J"),
}


def canonical_unit(quantity: Quantity) -> UnitLabel:
    """Return the canonical-unit label for ``quantity``.

    Raises ``KeyError`` if a quantity has no table entry — which would be a bug,
    since every ``Quantity`` member must be covered (a test asserts totality).
    """
    return CANONICAL_UNIT[quantity]


# --- balance contract (shared by the step-3 flow check + step-8 ledger) ------
# The quantities whose per-flow / per-step balance is ASSERTED. ENERGY is exempt
# (decision #8: energy *closure* is Phase 5/6; here energy is a diagnostic only).
# This is the single source of truth for "what we assert" — co-located with the
# tolerances below ("how tightly") so the step-3 flow-balance helper and the
# step-8 conservation gate cannot drift apart.
ASSERTED_QUANTITIES: frozenset[Quantity] = frozenset(Quantity) - {Quantity.ENERGY}

# The balance test is ``abs(residual) <= BALANCE_ATOL + BALANCE_RTOL * scale``,
# where ``scale`` is the transfer magnitude (max ``|leg.amount|`` for the
# quantity). The relative term keeps the check meaningful over the varied
# magnitudes the step-8 every-step gate sees. Phase-0 defaults, verified adequate
# against the demo's real magnitudes (step-11 conservation-tol carry-forward:
# ``BALANCE_ATOL`` floors ``tol`` above the stored-rounding floor — see
# ``conservation.assert_conserved``). The *form* matters, not these exact values.
BALANCE_ATOL: float = 1e-9
BALANCE_RTOL: float = 1e-9
