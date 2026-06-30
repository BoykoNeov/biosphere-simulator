"""Flows: structured, atomic, stoichiometric transfers (decisions #1/#2).

A ``Flow`` exposes only ``evaluate`` — legs exist **only after** evaluation
against a snapshot (the key step-3 correction). So balance and referential
integrity are *evaluation-time* properties, not registration-time:

  * **Balance** is a property of an evaluated ``FlowResult``: per conserved
    ``Quantity``, the legs sum to ~0 (every mole withdrawn from one carbon stock
    lands in another carbon stock, boundary reservoirs included). Resolving a
    leg's quantity needs a stock lookup, so the check is a pure helper over
    ``(result, stocks)`` — used here and **reused by the step-8 every-step
    conservation gate** (hence the shared ``ASSERTED_QUANTITIES`` / ``BALANCE_*``
    contract living in ``simcore.quantities``).
  * **Referential integrity** (every produced ``Leg.stock`` is a real stock) is
    *not* checked here — it is an assertion in the apply path (step 5/6), which
    runs before the conservation gate. A typo'd stock id surfaces as a
    ``KeyError`` at the first step, by design.

``ENERGY`` is **now asserted** like the mass quantities — it joined the conserved
set in Phase 5 (the Power domain; was exempt through Phase 4 under the original
decision #8). ``per_quantity_residual`` reports every quantity present (it always
did); ``assert_flow_balanced`` iterates ``ASSERTED_QUANTITIES``, which now includes
``ENERGY``. Electricity→heat is a transmutation of *form* within the one ENERGY
currency (see ``simcore.quantities`` / phase-5 plan), not a cross-quantity flow.

Pure stdlib only.
"""

import math
from collections.abc import Mapping
from dataclasses import dataclass
from typing import Protocol, runtime_checkable

from simcore.environment import Environment
from simcore.ids import DomainId, FlowId, StockId
from simcore.quantities import (
    ASSERTED_QUANTITIES,
    BALANCE_ATOL,
    BALANCE_RTOL,
    Quantity,
)
from simcore.state import State, Stock


class ConservationError(Exception):
    """A flow/ledger failed per-quantity balance within tolerance.

    Such a failure is an engine bug (decisions #2/#13), not a recoverable
    condition. It gets a dedicated type — not ``ValueError`` (brittle to
    message-match) and not ``AssertionError`` (stripped under ``python -O``) — so
    the step-8 every-step gate can catch/assert *this specific* failure.
    """


@dataclass(frozen=True)
class Leg:
    """One stock touched by a flow.

    ``amount`` is per dt in the stock's canonical unit: ``>0`` deposits into the
    stock, ``<0`` withdraws from it.
    """

    stock: StockId
    amount: float

    def __post_init__(self) -> None:
        # NaN/Inf are forbidden anywhere in state (determinism/serialization).
        if not math.isfinite(self.amount):
            raise ValueError(
                f"Leg on {self.stock!r} amount is not finite: {self.amount!r}"
            )


@dataclass(frozen=True)
class FlowResult:
    """The requested transfer from one ``evaluate`` against a snapshot.

    At most one leg per ``StockId`` — a flow nets its own touches on a stock into
    a single leg. This keeps arbitration's per-(stock, flow) ``demand``/``scale``
    a clean scalar (step 7). Empty ``legs`` is a valid no-op. ``legs`` is coerced
    to a tuple so the result stays hashable/comparable (the purity test compares
    two results for equality); the annotation stays ``tuple[Leg, ...]`` so the
    checker still flags non-tuples at call sites.
    """

    legs: tuple[Leg, ...]

    def __post_init__(self) -> None:
        legs = tuple(self.legs)
        object.__setattr__(self, "legs", legs)
        seen: set[StockId] = set()
        for leg in legs:
            if leg.stock in seen:
                raise ValueError(
                    f"FlowResult has a duplicate leg for stock {leg.stock!r}; "
                    "a flow must net its own touches on a stock into one leg"
                )
            seen.add(leg.stock)


@runtime_checkable
class Flow(Protocol):
    """A pure, deterministic stoichiometric transfer.

    ``evaluate`` reads the snapshot/env only, never mutates, and is deterministic
    in its inputs. ``priority`` is carried for declared-controller arbitration
    policies (decision #5) but is **unused** under the proportional default;
    canonical reduction order is always id-sorted (#15), never priority-sorted.

    **Increment-form contract (step 6).** ``evaluate`` returns the *per-step
    increment* — each leg amount is ``dt·rate(snapshot)``, not a bare rate — and
    ``rate`` must be **independent of ``dt``** (linear in ``dt``). This is what
    lets RK4's ⅙-combine reproduce classical RK4 exactly (see
    ``simcore.integrator``). A flow that uses ``dt`` non-linearly (an analytic
    sub-step, dt-gated logic) still conserves mass but silently forfeits RK4 order.

    ``id``/``priority`` are **read-only** (declared as properties) so that frozen
    flow implementations — the expected shape, matching the immutable-state model
    — satisfy the protocol; a mutable attribute satisfies a read-only member too.
    """

    @property
    def id(self) -> FlowId: ...

    @property
    def priority(self) -> int: ...

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult: ...


def per_quantity_residual(
    result: FlowResult, stocks: Mapping[StockId, Stock]
) -> dict[Quantity, float]:
    """Net leg sum per ``Quantity`` actually present in ``result`` (the diagnostic).

    Includes ``ENERGY`` (exempt from the asserted balance but reported as a
    diagnostic, decision #8). Quantities not touched are simply absent from the
    result (a missing quantity has a trivial 0 residual). Legs are folded in
    **canonical order** — sorted by stock id (decision #15) — so the sum is
    bit-identical regardless of leg construction order; this helper is the
    building block the step-8 every-step gate composes. (Phase-0 ids are ASCII,
    so Python's str sort agrees with the future Rust UTF-8 byte sort — keep ids
    ASCII.)

    A leg referencing an unknown stock id raises ``KeyError`` — referential
    integrity is the apply path's job (step 5/6), not this helper's.
    """
    residual: dict[Quantity, float] = {}
    for leg in sorted(result.legs, key=lambda leg: leg.stock):
        # Element-composition fold (P2.1): a leg contributes ``amount · coeff`` to
        # *each* quantity its stock carries (CO2 books both CARBON and OXYGEN),
        # instead of all of ``amount`` to one. A 1:1 stock folds ``amount · 1.0``
        # to its single quantity — bit-identical to the pre-P2.1 path. Each
        # quantity has its own accumulator, so inner iteration order is irrelevant;
        # the canonical-order (#15) guarantee comes from the outer stock-id sort.
        for quantity, coeff in stocks[leg.stock].composition.items():
            residual[quantity] = residual.get(quantity, 0.0) + leg.amount * coeff
    return residual


def assert_flow_balanced(
    result: FlowResult,
    stocks: Mapping[StockId, Stock],
    *,
    atol: float = BALANCE_ATOL,
    rtol: float = BALANCE_RTOL,
) -> None:
    """Raise ``ConservationError`` if any asserted quantity fails to balance.

    For each quantity in ``ASSERTED_QUANTITIES`` (every ``Quantity`` — ``ENERGY``
    joined the conserved set in Phase 5), require
    ``abs(residual) <= atol + rtol * scale`` where ``scale`` is the transfer
    magnitude (max ``|leg.amount|`` for that quantity).
    The relative term keeps the check meaningful for the step-8 gate over real,
    varied magnitudes; step-3 synthetic flows (residual ~0) pass on ``atol`` alone.
    """
    residual = per_quantity_residual(result, stocks)
    scale: dict[Quantity, float] = {}
    for leg in result.legs:
        # Same composition fold as the residual: the tolerance denominator for
        # quantity ``q`` is ``max |leg.amount · coeff_q|`` (P2.1). 1:1 folds to
        # ``abs(leg.amount)`` — unchanged.
        for quantity, coeff in stocks[leg.stock].composition.items():
            scale[quantity] = max(scale.get(quantity, 0.0), abs(leg.amount * coeff))
    # Iterate sorted so the first reported failure is deterministic (a frozenset
    # has no defined iteration order); which quantity fails first is cosmetic, but
    # determinism everywhere is cheap and on-brand.
    for quantity in sorted(ASSERTED_QUANTITIES, key=lambda q: q.name):
        r = residual.get(quantity, 0.0)
        tol = atol + rtol * scale.get(quantity, 0.0)
        if abs(r) > tol:
            raise ConservationError(
                f"flow not balanced for {quantity.name}: "
                f"residual {r!r} exceeds tolerance {tol!r}"
            )


def domains_touched(
    result: FlowResult, stocks: Mapping[StockId, Stock]
) -> frozenset[DomainId]:
    """Domains the evaluated ``result`` touches (>1 ⇒ the flow is cross-domain).

    A leg referencing an unknown stock id raises ``KeyError`` (see
    ``per_quantity_residual`` — referential integrity is deferred to the apply
    path, step 5/6).
    """
    return frozenset(stocks[leg.stock].domain for leg in result.legs)
