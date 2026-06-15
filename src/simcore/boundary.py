"""The Boundary domain: "outside" reservoir stocks (decision #13).

External forcing (irrigation adds water, harvest removes carbon, solar adds
energy) is *unbalanced* against the modeled stocks — which the flow invariant
(#2: ``Σ legs == 0``) forbids. Resolution: model "outside" as explicit BOUNDARY
reservoir stocks. Then every flow is internally balanced, and a boundary stock's
per-step delta *is* the ledger's Input/Output for that quantity (#13).

Three roles, all BOUNDARY-kind reservoirs, built by the constructors here so call
sites cannot misconfigure ``kind`` / ``unit`` / ``unclamped``:

  * ``source`` — an "outside" supply (e.g. solar, the outside atmosphere acting as
    a source). Flagged ``unclamped`` so arbitration's min-scaling never throttles
    it (#13). **An unclamped source MAY go negative by design** — the
    non-negativity invariant guards POOL stocks (via arbitration) and POPULATION
    stocks (via extinction), *not* unclamped sources, whose magnitude is pure
    ledger bookkeeping.
  * ``sink`` — an "outside" disposal reservoir (receives outputs; never withdrawn
    from, so min-scaling — which only touches withdrawals — never applies).
  * ``loss_sink`` — the numerical-loss reservoir that extinction routes a snapped
    POPULATION residual into (#6), keeping the per-quantity ledger balanced. One
    per conserved (mass) quantity, since a ``Stock`` holds exactly one ``Quantity``
    and conservation is per-quantity.

These constructors only *build* the stocks. They must be placed into the initial
``State`` (step 10's init) for step 7's extinction deposit and step 8's ledger to
resolve them — a missing loss-sink surfaces as a ``KeyError`` at the first step
(referential integrity is the apply path's job, step 5/6), not a build error.

Pure stdlib only.
"""

from collections.abc import Iterable

from simcore.ids import DomainId, StockId
from simcore.quantities import (
    ASSERTED_QUANTITIES,
    Quantity,
    StockKind,
    canonical_unit,
)
from simcore.state import Stock

# The canonical Boundary domain id — the namespace the rest of the codebase
# already uses for outside reservoirs (e.g. ``boundary.outside_c``).
BOUNDARY_DOMAIN: DomainId = DomainId("boundary")

# Numerical-loss sink ids share this prefix so diagnostics and the step-8 ledger
# can separate routed numerical-loss deltas from legitimate boundary exchange
# (see ``is_loss_sink``). ASCII-only — Python's str sort then matches the future
# Rust UTF-8 byte sort for canonical ordering (#15).
LOSS_SINK_PREFIX = "boundary.loss."


def loss_sink_id(quantity: Quantity) -> StockId:
    """Canonical, deterministic id for ``quantity``'s numerical-loss sink."""
    return StockId(f"{LOSS_SINK_PREFIX}{quantity.value}")


def is_loss_sink(stock_id: StockId) -> bool:
    """True iff ``stock_id`` names a numerical-loss sink (see ``loss_sink_id``)."""
    return stock_id.startswith(LOSS_SINK_PREFIX)


def loss_sink(quantity: Quantity, amount: float = 0.0) -> Stock:
    """A numerical-loss BOUNDARY reservoir for ``quantity`` (decisions #6/#13).

    Accumulates the snapped residual when a POPULATION stock goes extinct so the
    per-quantity ledger still balances. ``amount`` defaults to 0 — it is an
    accumulator; its running total is the tracked numerical loss (a diagnostic).
    Never withdrawn from, so it is left clamped (``unclamped=False``).
    """
    return Stock(
        id=loss_sink_id(quantity),
        domain=BOUNDARY_DOMAIN,
        quantity=quantity,
        unit=canonical_unit(quantity),
        amount=amount,
        kind=StockKind.BOUNDARY,
    )


def loss_sinks(
    quantities: Iterable[Quantity] = ASSERTED_QUANTITIES,
) -> dict[StockId, Stock]:
    """Build one loss-sink per quantity, keyed by id (ready to merge into State).

    Defaults to ``ASSERTED_QUANTITIES`` — the mass quantities the ledger balances.
    ENERGY gets no loss-sink (it is balance-exempt, decision #8, and POPULATION
    biomass is carbon anyway). Built in canonical (quantity-name) order so the
    result is deterministic regardless of the input set's iteration order.
    """
    ordered = sorted(quantities, key=lambda q: q.name)
    return {s.id: s for s in (loss_sink(q) for q in ordered)}


def source(
    stock_id: StockId,
    quantity: Quantity,
    amount: float,
    *,
    unclamped: bool = True,
) -> Stock:
    """An "outside" supply reservoir (e.g. solar, outside atmosphere as a source).

    ``unclamped`` defaults to True so arbitration's min-scaling never throttles
    the supply (decision #13) — the easy-to-get-wrong default, encoded here. Pass
    ``unclamped=False`` for a finite, throttleable boundary supply. ``unit`` is
    derived from ``quantity`` (the canonical-unit table is the single source of
    truth, #9), so it cannot drift from the stock's quantity.
    """
    return Stock(
        id=stock_id,
        domain=BOUNDARY_DOMAIN,
        quantity=quantity,
        unit=canonical_unit(quantity),
        amount=amount,
        kind=StockKind.BOUNDARY,
        unclamped=unclamped,
    )


def sink(stock_id: StockId, quantity: Quantity, amount: float = 0.0) -> Stock:
    """An "outside" disposal reservoir (e.g. harvested carbon leaving the system).

    Receives outputs and is never withdrawn from, so arbitration's min-scaling —
    which only touches withdrawals — never applies; it stays clamped (the
    default). ``amount`` defaults to 0 (an accumulator of cumulative output).
    ``unit`` is derived from ``quantity`` (single source of truth, #9).
    """
    return Stock(
        id=stock_id,
        domain=BOUNDARY_DOMAIN,
        quantity=quantity,
        unit=canonical_unit(quantity),
        amount=amount,
        kind=StockKind.BOUNDARY,
    )
