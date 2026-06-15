"""Immutable state primitives: ``Stock`` and the per-step ``State`` snapshot.

Both are frozen (decision: the engine writes a *new* ``State`` each step via
``dataclasses.replace`` rather than mutating in place — see the reconciled Frozen
API note in the plan). ``State.stocks`` is wrapped in a ``MappingProxyType`` so a
snapshot cannot be mutated through its mapping after construction.

Caveat (per advisor): the proxy guards *mutation*, not *order*. Canonical order
for reductions comes from explicitly sorting by stock/flow id at each reduction
(later steps), never from mapping iteration order.

Pure stdlib only.
"""

import math
from collections.abc import Mapping
from dataclasses import dataclass
from types import MappingProxyType

from simcore.ids import DomainId, StockId, UnitLabel
from simcore.quantities import Quantity, StockKind


@dataclass(frozen=True)
class Stock:
    """A single well-mixed compartment holding an amount of one quantity.

    ``amount`` is in the quantity's canonical unit (the ``unit`` label records
    which; see ``simcore.quantities.CANONICAL_UNIT``). Frozen: updates produce a
    new ``Stock`` via ``dataclasses.replace``.
    """

    id: StockId
    domain: DomainId
    quantity: Quantity
    unit: UnitLabel
    amount: float
    kind: StockKind
    # POPULATION only: at/below this level the stock snaps to 0 and the residual
    # is routed to the numerical-loss sink (decision #6). Ignored for other kinds.
    extinction_threshold: float = 0.0
    # BOUNDARY source (e.g. solar): never throttled by arbitration min-scaling
    # (decision #13). Meaningful only for BOUNDARY stocks.
    unclamped: bool = False

    def __post_init__(self) -> None:
        # NaN/Inf are forbidden anywhere in state (determinism/serialization).
        if not math.isfinite(self.amount):
            raise ValueError(f"Stock {self.id!r} amount is not finite: {self.amount!r}")
        if not math.isfinite(self.extinction_threshold):
            raise ValueError(
                f"Stock {self.id!r} extinction_threshold is not finite: "
                f"{self.extinction_threshold!r}"
            )
        # `unclamped` is meaningful only for BOUNDARY sources (decision #13): it
        # tells arbitration's min-scaling never to throttle this stock. An
        # unclamped POOL/POPULATION would silently escape throttling and could go
        # negative — a conservation break — so reject it at construction. (Guard
        # added in step 4 alongside the Boundary domain module; it tightens this
        # step-2 primitive to match its own documented contract.)
        if self.unclamped and self.kind is not StockKind.BOUNDARY:
            raise ValueError(
                f"Stock {self.id!r} is unclamped but kind={self.kind.name}; "
                "unclamped is only valid on BOUNDARY stocks (decision #13)"
            )


@dataclass(frozen=True)
class State:
    """One immutable simulation snapshot.

    Time is the integer step count ``n`` (decision #14): wall time is ``n * dt``,
    evaluated, never accumulated. ``rng_seed`` is carried in state so draws are
    keyed by ``(seed, key, n)`` and stay order-independent (decision #12).

    Note: a ``State`` is **not hashable** — the ``MappingProxyType`` stocks field
    is unhashable, so ``hash(state)`` raises. Equality (by contents) works. If a
    later test needs states in a ``set``/dict-key, give ``State`` a custom
    ``__hash__`` then (e.g. over ``n`` + sorted stock ids); don't rely on it now.
    """

    n: int
    stocks: Mapping[StockId, Stock]
    rng_seed: int

    def __post_init__(self) -> None:
        if self.n < 0:
            raise ValueError(f"State.n must be >= 0, got {self.n}")
        # Each mapping key must equal its stock's own id — a cheap guard against
        # a stock filed under the wrong key (which would corrupt id-keyed lookups
        # and canonical-order reductions).
        for key, stock in self.stocks.items():
            if key != stock.id:
                raise ValueError(f"State.stocks key {key!r} != stock.id {stock.id!r}")
        # Detach from the caller's dict and forbid post-construction mutation.
        object.__setattr__(self, "stocks", MappingProxyType(dict(self.stocks)))
