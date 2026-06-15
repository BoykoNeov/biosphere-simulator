"""The flow Registry: build-once structural config (decisions #7/#15).

Because balance and referential integrity are *evaluation-time* (see
``simcore.flow``), the registry's job is purely structural:

  * holds the flow set and **rejects duplicate ``FlowId``**;
  * exposes **canonical id-sorted** iteration — this is the
    registration-order-independence guarantee (#7/#15): shuffling the
    registration list yields bit-identical iteration;
  * derives a **domain index** ``DomainId -> frozenset[StockId]`` from
    ``Stock.domain`` over the initial stocks. This index *is* Phase-0's "Domain"
    primitive — a namespace plus its stock membership, not a rich class.

Built here (step 3), injected into the integrator at construction (step 6); the
frozen ``Integrator.step(state, env, dt)`` takes no registry argument.

The registry does **not** validate that flow legs reference known stocks — it
cannot, since legs do not exist until ``evaluate`` (that check is the apply
path's, step 5/6).

Pure stdlib only.
"""

from collections.abc import Iterable, Iterator, Mapping
from types import MappingProxyType

from simcore.flow import Flow
from simcore.ids import DomainId, FlowId, StockId
from simcore.state import Stock


class Registry:
    """An immutable, build-once set of flows plus a stock domain index."""

    def __init__(self, flows: Iterable[Flow], stocks: Mapping[StockId, Stock]) -> None:
        # Canonical id-sorted order (#15): the registration-order-independence
        # guarantee. ASCII ids → Python's str sort matches the future Rust UTF-8
        # byte sort (cross-port); keep flow ids ASCII.
        ordered = sorted(flows, key=lambda flow: flow.id)
        seen: set[FlowId] = set()
        for flow in ordered:
            if flow.id in seen:
                raise ValueError(f"Registry has a duplicate FlowId {flow.id!r}")
            seen.add(flow.id)
        self._flows: tuple[Flow, ...] = tuple(ordered)

        index: dict[DomainId, set[StockId]] = {}
        for stock in stocks.values():
            index.setdefault(stock.domain, set()).add(stock.id)
        self._domain_index: Mapping[DomainId, frozenset[StockId]] = MappingProxyType(
            {domain: frozenset(ids) for domain, ids in index.items()}
        )

    @property
    def flows(self) -> tuple[Flow, ...]:
        """The flows in canonical id-sorted order."""
        return self._flows

    @property
    def domain_index(self) -> Mapping[DomainId, frozenset[StockId]]:
        """Read-only ``DomainId -> frozenset[StockId]`` over the initial stocks."""
        return self._domain_index

    def __iter__(self) -> Iterator[Flow]:
        return iter(self._flows)

    def __len__(self) -> int:
        return len(self._flows)
