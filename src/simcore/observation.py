"""The observation surface: ``observe(state) -> Observation`` (plain data).

This is the last of the three frozen core-surface functions (``init`` / ``step`` /
``observe``). ``observe`` is the **consumer-facing read** of a simulation — "what is
in the tanks at step ``n``" — decoupled from ``State``'s engine-internal shape so a
UI / telemetry / Godot front-end never has to reach into ``State.stocks`` and know
about the RNG seed, arbitration flags, or extinction thresholds.

**It is a projection, not an aggregate.** ``Observation`` re-exposes only the
*observable* subset of each ``Stock`` and deliberately **drops engine internals**:

  * ``State.rng_seed`` — internal RNG state, not an observation;
  * ``Stock.extinction_threshold`` — an engine *control* (when a population snaps),
    not a measurement;
  * ``Stock.unclamped`` — an arbitration *control* (decision #13), not a measurement.

The rule for which fields live here (and the contract a future reader should hold
the type to): **if you cannot say what a consumer observes with a field, it is an
engine internal and does not belong.** Each kept field passes that test:

  * ``StockObservation.id`` — *which* tank a measurement belongs to (without it the
    amounts are anonymous and cannot map to a series / UI element);
  * ``.domain`` — group/filter observations by namespace (biosphere vs boundary)
    without fragile id-string parsing;
  * ``.quantity`` — *what substance* (carbon/water/…); needed to group or colour by
    conserved quantity;
  * ``.unit`` — an amount is uninterpretable without its canonical-unit label;
  * ``.amount`` — the measurement itself;
  * ``.kind`` — descriptive *classification* (POOL / POPULATION / BOUNDARY): lets a
    consumer tell a modeled stock from an "outside" boundary reservoir. This is
    classification, not a tunable control — hence kept while ``threshold`` /
    ``unclamped`` are dropped.

**No aggregates (yet).** There is intentionally no per-quantity ``totals`` /
per-domain rollup field: nothing in Phase 0 consumes it (the golden snapshot
round-trips a full ``State`` via ``sim_io``; the API freeze reads neither), and
the project's norm is "no field without a named consumer" (cf. the ``StepReport``
``ledger`` refusal in step 8). A total is additive and cheap to add when a real
consumer appears, and the canonical-order reduction it would need already lives in
``conservation.compute_ledger``.

**Plain-data, hashable.** ``Observation`` and ``StockObservation`` are frozen
dataclasses of stdlib primitives + the core enums (the same "plain data" notion as
``QuantityLedger`` / ``ExtinctionEvent``). Carrying no ``Mapping`` field, an
``Observation`` is fully ``==``-comparable **and hashable** — unlike ``State``,
which trades hashability for its ``MappingProxyType`` stocks — which is exactly what
snapshot/equality assertions want. Stocks are emitted in **canonical id-sorted
order** (decision #15) so two observations of equal states compare equal regardless
of ``State.stocks`` insertion order.

Pure stdlib only.
"""

from dataclasses import dataclass

from simcore.ids import DomainId, StockId, UnitLabel
from simcore.quantities import Quantity, StockKind
from simcore.state import State


@dataclass(frozen=True)
class StockObservation:
    """The observable subset of one ``Stock`` (engine-internal fields dropped).

    See the module docstring for why each field is observable and why
    ``extinction_threshold`` / ``unclamped`` are not.
    """

    id: StockId
    domain: DomainId
    quantity: Quantity
    unit: UnitLabel
    amount: float
    kind: StockKind


@dataclass(frozen=True)
class Observation:
    """A plain-data, hashable read of a ``State`` (decoupled from its internals).

    ``n`` is the integer step count (time index; wall time is ``n * dt``, but ``dt``
    is not part of state — decision #14). ``stocks`` are the per-stock observations
    in canonical id-sorted order (#15).
    """

    n: int
    stocks: tuple[StockObservation, ...]


def observe(state: State) -> Observation:
    """Project ``state`` to a plain-data ``Observation`` (the consumer read surface).

    Re-exposes only the observable subset of each stock (dropping the RNG seed and the
    extinction/arbitration controls — see the module docstring) with stocks in
    **canonical id-sorted order** (#15), so equal states observe equal regardless of
    ``State.stocks`` insertion order.
    """
    stocks = tuple(
        StockObservation(
            id=stock.id,
            domain=stock.domain,
            quantity=stock.quantity,
            unit=stock.unit,
            amount=stock.amount,
            kind=stock.kind,
        )
        for stock in (state.stocks[sid] for sid in sorted(state.stocks))
    )
    return Observation(n=state.n, stocks=stocks)
