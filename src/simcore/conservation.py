"""Conservation ledger + the every-step balance gate (decision #13, step-alg #7).

The augmented system (modeled stocks + boundary reservoirs) is **closed**: every
Phase-0 state change is balanced — a flow has ``Σ legs == 0`` per quantity
(arbitration scales *whole* flows, preserving that), and extinction's loss-sink
routing is a balanced *non-flow* state change. So over the augmented system, per
asserted quantity the **total mass across all stocks is unchanged step-to-step**,
within tolerance::

    residual_q = Σ_{stocks s of quantity q} (after[s].amount − before[s].amount) ≈ 0

This is the **bug-resistant form** of the plan's ``inputs − outputs − ΔStored = 0``:
they are equal up to sign (``inputs − outputs = −boundary_delta``, so
``inputs − outputs − ΔStored = −(boundary_delta + stored_delta) = −residual``), but
the total-mass residual has **no sign/classification surface in the pass/fail** —
the ``StockKind`` partition feeds only the diagnostic decomposition.

This module reasons about **state deltas, not flows**. It deliberately does *not*
reuse ``flow.assert_flow_balanced`` over a synthetic "whole-step delta" flow, which
would mis-state the step-3 caveat that extinction's loss-sink routing is a balanced
*non-flow* change. It shares with ``simcore.flow`` only the contract constants
(``ASSERTED_QUANTITIES``, ``BALANCE_ATOL``/``BALANCE_RTOL``, ``ConservationError``) —
which is the real drift risk, and is covered.

Pure stdlib only.
"""

from dataclasses import dataclass

from simcore.flow import ConservationError
from simcore.quantities import (
    ASSERTED_QUANTITIES,
    BALANCE_ATOL,
    BALANCE_RTOL,
    Quantity,
    StockKind,
)
from simcore.state import State


@dataclass(frozen=True)
class QuantityLedger:
    """Per-quantity conservation accounting for one step (decision #13).

    - ``boundary_delta`` — ``Σ Δamount`` over BOUNDARY stocks of the quantity. This
      *is* the ledger's net Input/Output; the net flux *into* the modeled system is
      ``−boundary_delta``.
    - ``stored_delta`` — ``Σ Δamount`` over POOL + POPULATION stocks (the modeled
      ΔStored).
    - ``residual`` — ``boundary_delta + stored_delta`` (≡ total-mass Δ); ~0 when the
      step conserved the quantity. The decomposition is exact by construction (the
      residual is defined as the sum of the two partition deltas, not a separate
      interleaved sum), so ``boundary_delta + stored_delta == residual`` always holds.

    Clean accounting only — no ``scale`` field; the relative-tolerance basis is a
    tolerance detail computed in ``assert_conserved``.
    """

    quantity: Quantity
    boundary_delta: float
    stored_delta: float
    residual: float


def compute_ledger(before: State, after: State) -> tuple[QuantityLedger, ...]:
    """Per-quantity ledger for the step ``before → after`` (canonical quantity order).

    Covers every quantity *present* in the stocks, ``ENERGY`` included as a
    diagnostic (balance-exempt, decision #8 — mirroring ``flow.per_quantity_residual``).
    Per-stock deltas are accumulated within each ``StockKind`` partition in **sorted
    stock-id order** (decision #15: "every reduction"; float sums are non-associative,
    so this — not mapping iteration order — is what makes the ledger bit-identical
    under registration shuffle). The residual is the sum of the two partition deltas,
    so the decomposition is exact.

    Raises ``ValueError`` if ``before``/``after`` do not share the same stock-id key
    set: Phase 0 never adds or removes stocks mid-run, so a mismatch is an engine bug
    (``ValueError``, not ``assert`` — the latter is stripped under ``python -O``).
    """
    if before.stocks.keys() != after.stocks.keys():
        raise ValueError(
            "conservation ledger requires before/after to share the same stock ids; "
            "Phase 0 never adds/removes stocks mid-run "
            f"(before-only={set(before.stocks) - set(after.stocks)!r}, "
            f"after-only={set(after.stocks) - set(before.stocks)!r})"
        )
    boundary: dict[Quantity, float] = {}
    stored: dict[Quantity, float] = {}
    for sid in sorted(before.stocks):
        b = before.stocks[sid]
        delta = after.stocks[sid].amount - b.amount
        bucket = boundary if b.kind is StockKind.BOUNDARY else stored
        # Element-composition fold (P2.1) — the state-delta mirror of the leg fold
        # in ``flow.per_quantity_residual``. This module deliberately does NOT
        # reuse the leg path (it reasons over deltas, see the docstring), so it
        # needs its own fold: a per-stock Δamount books to *each* quantity the
        # stock carries (a CO2 pool's drop books to both CARBON and OXYGEN).
        # Without this, the per-step OXYGEN gate would falsely trip on every
        # photosynthesis step. A 1:1 stock folds ``delta · 1.0`` to its single
        # quantity — bit-identical to the pre-P2.1 path.
        for q, coeff in b.composition.items():
            bucket[q] = bucket.get(q, 0.0) + delta * coeff
    quantities = sorted(set(boundary) | set(stored), key=lambda q: q.name)
    return tuple(
        QuantityLedger(
            quantity=q,
            boundary_delta=boundary.get(q, 0.0),
            stored_delta=stored.get(q, 0.0),
            residual=boundary.get(q, 0.0) + stored.get(q, 0.0),
        )
        for q in quantities
    )


def assert_conserved(
    before: State,
    after: State,
    *,
    atol: float = BALANCE_ATOL,
    rtol: float = BALANCE_RTOL,
) -> None:
    """Raise ``ConservationError`` if any asserted quantity's mass is not conserved.

    The every-step engine gate (decision #7 / CLAUDE.md "conservation is asserted
    every step — a failure is a bug"). For each ``ASSERTED_QUANTITIES`` member (every
    ``Quantity`` except the exempt ``ENERGY``, #8), require
    ``abs(residual) <= atol + rtol * scale``, where ``scale`` is the transfer
    magnitude — ``max |per-stock Δ|`` for that quantity — matching
    ``flow.assert_flow_balanced``'s scale notion.

    What keeps this safe is **atol**, not the transfer magnitude. The Phase-0 demo
    *does* enter the watch-item's regime — ``Harvest`` fills the ``outside_c`` boundary
    reservoir to a large accumulated stock (~1e3) with shrinking deposits, so
    ``rtol·max|Δ|`` collapses toward the stored-rounding floor ``~eps·|amount|``. But
    ``tol >= atol`` (1e-9) sits ~3–4 orders above that floor, so the per-step ratio
    ``|residual|/tol`` is tiny: observed worst ≈ ``1.1e-4`` (a signed sum, so below the
    ceiling), analytic ceiling ``eps·Σ|amount| / atol`` ≈ ``2.4e-4`` —
    length-independent because conservation caps the total mass. An ``amount``-scaled
    basis (``Σ|amount|``) becomes the principled choice only once accumulated amounts
    approach ``atol/eps`` (~4.5e6), where ``eps·Σ|amount|`` overtakes ``atol`` — a
    flagged Phase-1 revisit, verified unnecessary for the Phase-0 demo (the step-11
    carry-forward gate in ``tests/test_biosphere_demo.py``).

    Thresholds the residual ``compute_ledger`` already computes (one residual
    computation; the ``ValueError`` key-set guard there also covers this entry point).
    """
    ledger = {ql.quantity: ql for ql in compute_ledger(before, after)}
    scale: dict[Quantity, float] = {}
    for sid, b in before.stocks.items():
        d = after.stocks[sid].amount - b.amount
        # Composition fold (P2.1), mirroring ``flow.assert_flow_balanced``'s scale:
        # the tolerance denominator for ``q`` is ``max |Δ · coeff_q|``. 1:1 folds
        # to ``abs(d)`` — unchanged.
        for q, coeff in b.composition.items():
            scale[q] = max(scale.get(q, 0.0), abs(d * coeff))
    # Sorted so the first reported failure is deterministic (a frozenset has no
    # defined iteration order) — cosmetic, but determinism everywhere is on-brand.
    for quantity in sorted(ASSERTED_QUANTITIES, key=lambda q: q.name):
        ql = ledger.get(quantity)
        if ql is None:
            continue  # quantity absent from the state — trivially conserved
        tol = atol + rtol * scale.get(quantity, 0.0)
        if abs(ql.residual) > tol:
            raise ConservationError(
                f"conservation violated for {quantity.name}: residual "
                f"{ql.residual!r} exceeds tolerance {tol!r} "
                f"(boundary_delta={ql.boundary_delta!r}, "
                f"stored_delta={ql.stored_delta!r})"
            )
