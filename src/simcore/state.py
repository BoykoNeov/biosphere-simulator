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
from dataclasses import dataclass, field
from types import MappingProxyType

from simcore.ids import DomainId, StockId, UnitLabel
from simcore.quantities import Quantity, StockKind

# Shared immutable empty default for ``State.aux`` (the non-conserved channel,
# Phase-1 P2). A ``MappingProxyType`` is not in dataclass's mutable-default
# blacklist (list/dict/set) and is effectively immutable, so one shared instance
# is a safe default — every ``State`` re-wraps its own copy in ``__post_init__``.
_EMPTY_AUX: Mapping[str, float] = MappingProxyType({})

# Shared "not supplied" sentinel for ``Stock.composition`` (P2.1). Empty means
# "default to the 1:1 ``{quantity: 1.0}`` map" — filled in ``__post_init__`` once
# the stock's own quantity is known. Same safe-shared-MappingProxyType idiom as
# ``_EMPTY_AUX`` (not in dataclass's mutable-default blacklist).
_EMPTY_COMPOSITION: Mapping[Quantity, float] = MappingProxyType({})


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
    # Element composition (P2.1): moles of each conserved quantity that one
    # canonical unit of this stock contributes to the conservation ledger. The
    # empty-sentinel default fills to the 1:1 ``{quantity: 1.0}`` map in
    # ``__post_init__``, so every Phase-1 stock is unchanged. A gas-phase stock
    # carries several: CO2={CARBON:1, OXYGEN:2}, O2={OXYGEN:2}; biomass stays
    # {CARBON:1}. ``hash=False`` because a ``Mapping`` is unhashable — but it
    # stays in ``__eq__`` so a dropped/garbled composition is caught by the
    # snapshot round-trip. The conservation gate (``flow``/``conservation``)
    # folds this map; nothing else (arbitration, integrator, observation) reads it.
    composition: Mapping[Quantity, float] = field(
        default=_EMPTY_COMPOSITION, hash=False
    )

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
        # Element composition (P2.1). Empty (the not-supplied sentinel) → the 1:1
        # ``{quantity: 1.0}`` default. Validate: every key a ``Quantity``, every
        # coeff finite, and the stock's own quantity present with a positive coeff
        # (so a stock always contributes to its nominal quantity). Detach from the
        # caller's mapping and freeze read-only — same discipline as ``stocks``/``aux``.
        composition = dict(self.composition) or {self.quantity: 1.0}
        for q, coeff in composition.items():
            if not isinstance(q, Quantity):
                raise ValueError(
                    f"Stock {self.id!r} composition key {q!r} is not a Quantity"
                )
            if not math.isfinite(coeff):
                raise ValueError(
                    f"Stock {self.id!r} composition[{q.name}] is not finite: {coeff!r}"
                )
        self_coeff = composition.get(self.quantity)
        if self_coeff is None or self_coeff <= 0.0:
            raise ValueError(
                f"Stock {self.id!r} composition must include its own quantity "
                f"{self.quantity.name} with a positive coeff (got {self_coeff!r})"
            )
        # A POPULATION stock must be single-quantity (P2.1 invariant): extinction
        # routes its residual via ``loss_sink_id(stock.quantity)`` — one nominal
        # quantity — so a multi-quantity POPULATION would leak the non-nominal
        # quantities' mass at extinction. Reject at construction.
        if self.kind is StockKind.POPULATION and set(composition) != {self.quantity}:
            raise ValueError(
                f"Stock {self.id!r} is POPULATION but multi-quantity "
                f"(composition keys {sorted(k.name for k in composition)}); a "
                "POPULATION stock must be single-quantity (extinction loss-sink "
                "routing is single-quantity, P2.1)"
            )
        object.__setattr__(self, "composition", MappingProxyType(composition))


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

    ``aux`` is the **non-conserved auxiliary channel** (Phase-1 P2): scalar
    accumulators (thermal time, …) advanced by the integrator in parallel to
    ``stocks`` but **outside** the conservation gate — they have no balanced
    counterparty, so they are not flows and are invisible to
    ``conservation.compute_ledger`` (which reasons only over ``stocks``). The
    empty default keeps every pre-P2 call site (and the Phase-0/0.5 goldens, modulo
    the schema bump) unchanged. Keys are stable, canonical-sortable names; values
    are advanced by ``simcore.auxiliary`` processes (see the integrator).
    """

    n: int
    stocks: Mapping[StockId, Stock]
    rng_seed: int
    aux: Mapping[str, float] = _EMPTY_AUX

    def __post_init__(self) -> None:
        if self.n < 0:
            raise ValueError(f"State.n must be >= 0, got {self.n}")
        # Each mapping key must equal its stock's own id — a cheap guard against
        # a stock filed under the wrong key (which would corrupt id-keyed lookups
        # and canonical-order reductions).
        for key, stock in self.stocks.items():
            if key != stock.id:
                raise ValueError(f"State.stocks key {key!r} != stock.id {stock.id!r}")
        # NaN/Inf are forbidden anywhere in state (determinism/serialization) — the
        # same isfinite-only discipline as ``Stock.amount``; aux values are plain
        # accumulators, so no further coercion.
        for name, value in self.aux.items():
            if not math.isfinite(value):
                raise ValueError(f"State.aux[{name!r}] is not finite: {value!r}")
        # Detach from the caller's dicts and forbid post-construction mutation.
        object.__setattr__(self, "stocks", MappingProxyType(dict(self.stocks)))
        object.__setattr__(self, "aux", MappingProxyType(dict(self.aux)))
