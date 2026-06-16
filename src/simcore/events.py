"""Simulation events: discrete, logged state transitions (step 7).

Events are the diagnostic record of a *non-flow* state change the engine makes
during a step. Phase 0 has exactly one: extinction. A POPULATION stock that falls
below its ``extinction_threshold`` is snapped to 0 and its residual mass is routed
into the numerical-loss boundary sink (decision #6); the ``ExtinctionEvent``
records that this happened, where, and how much mass moved.

Events are returned from the integrator in a ``StepReport`` (see
``simcore.integrator``) — they are *not* carried in ``State`` (whose frozen fields
are ``n``/``stocks``/``rng_seed``) and the engine holds no mutable event log, so
the core stays pure and re-runnable. The caller (a scenario/golden harness)
accumulates events as it sees fit.

Pure stdlib only.
"""

from dataclasses import dataclass

from simcore.ids import StockId
from simcore.quantities import Quantity


@dataclass(frozen=True)
class ExtinctionEvent:
    """A POPULATION stock went extinct (decision #6).

    Recorded by the integrator's extinction pass when a POPULATION stock's amount
    is below its ``extinction_threshold`` (and not already exactly 0): the stock is
    set to 0 and ``residual`` mass is routed to the quantity's numerical-loss sink
    so the ledger still balances.

      * ``n`` — the step count of the *post-apply* state at which extinction was
        observed (i.e. the ``n`` of the ``State`` the integrator is producing).
      * ``stock`` — the POPULATION stock that went extinct.
      * ``quantity`` — its conserved quantity (the loss-sink that received the
        residual is ``boundary.loss_sink_id(quantity)``).
      * ``residual`` — the snapped amount routed to the loss-sink (the stock's
        pre-snap amount). Normally positive; a negative value is a defensively
        conserved RK4 round-off excursion (positivity under RK4 is the kinetics'
        job, not a guard).
    """

    n: int
    stock: StockId
    quantity: Quantity
    residual: float


# The Phase-0 event union is a single type. When more event kinds appear (e.g. a
# scenario reintroduction), widen this to ``ExtinctionEvent | ...`` in one place;
# everything annotating ``Event`` (notably ``StepReport.events``) broadens with it.
Event = ExtinctionEvent
