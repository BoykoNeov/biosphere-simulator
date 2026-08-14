"""The station's two-rate master-step driver (P6.3+): one slow domain + one fast domain.

The shared stepping harness for every station seam that couples a **day-scale** domain
(the biosphere: ``dt = BIO_DT`` day, ``STEPS_PER_DAY`` sub-steps per master day,
weather indexed by physical time ``n · dt``, a ``thermal_time`` phenology aux that
must advance) to a **second-scale** domain
(the cabin / Power: ``dt = 60`` s / ``dt = 3600`` s, no aux). ``simcore.multirate``
cannot bridge these — it splits ONE shared master ``dt`` (``dt/n_sub``), and no single
master ``dt`` serves both *time units*; and it composes ``substep`` only, which by
design freezes the biosphere's phenology aux. So the station does the operator split
**by hand**, calling each domain's own integrator with its own ``dt`` (extracted here as
the **second** two-rate instance — the greenhouse was the first, the
bespoke-until-second rhythm).

**The split (Lie, slow-first).** Per master day: the **slow** domain takes
``slow_steps_per_day`` ``step_report`` calls at ``slow_dt`` (advancing its aux **and**
``n``; its own conservation gate covers each sub-operation), then the **fast** domain
takes ``steps_per_day`` ``substep`` calls at ``fast_dt`` (keeping ``n``). ``substep``
deliberately skips the conservation assert, so the driver re-asserts it after **each**
fast sub-step over the whole shared ledger — preserving the "every step conserves"
teeth. Requires ``fast_dt · steps_per_day == 86400`` (one day) and
``slow_dt · slow_steps_per_day == 1`` (one day), so both operators advance the same
wall-clock day and ``n`` advances by the slow domain's own step count.

⚠ **``n`` is NOT the day count.** It was, while the biosphere's step was structurally
one day; it is now ``day · slow_steps_per_day``. Nothing here needs it to be a day
count: the weather resolver indexes physical time (``int(n · dt)``), not ``n``. Any
caller that computes a calendar from ``n`` — the ``slow_reset`` closure especially —
must convert through ``domains.biosphere.step.steps_for``.

Two disjoint registries over **one** shared stock dict + two integrators — exactly
``simcore.multirate``'s model, orchestrated by hand for per-domain ``dt`` + aux. The two
domains may share stocks (the greenhouse: the biosphere's gas pools ARE the cabin air)
or share **none** (lighting: Power and the biosphere are coupled only by a forcing
schedule, decision #16) — either way the combined ledger balances per-quantity, since
each flow touches only its own domain's stocks.

All public integrator methods → **zero core change**. Pure stdlib only.
"""

from collections.abc import Callable

from simcore import conservation
from simcore.environment import SourceResolver
from simcore.events import Event
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.state import State

MasterStepIntegrator = EulerIntegrator | Rk4Integrator

# Seconds in one master day — what ``fast_dt · steps_per_day`` must advance, so the fast
# operator covers exactly the same wall-clock span as the slow one.
SECONDS_PER_DAY: float = 86400.0

# Days in one master day — what ``slow_dt · slow_steps_per_day`` must advance. The slow
# domain's own unit is the day, so this is 1: the two operators are split over the SAME
# interval, whatever each one's internal step size.
DAYS_PER_MASTER_DAY: float = 1.0


def run_master_day(
    slow_integrator: MasterStepIntegrator,
    fast_integrator: MasterStepIntegrator,
    state: State,
    slow_resolver: SourceResolver,
    fast_resolver: SourceResolver,
    *,
    days: int,
    steps_per_day: int,
    slow_dt: float,
    fast_dt: float,
    slow_steps_per_day: int = 1,
    slow_reset: Callable[[int, State], State] | None = None,
) -> tuple[list[State], int, tuple[Event, ...]]:
    """Step ``days`` master days (slow ×``slow_steps_per_day`` + fast), slow-first.

    Per day: the ``slow_integrator`` runs ``slow_steps_per_day`` ``step_report`` calls
    at ``slow_dt`` (advancing any aux **and** ``n`` — its own conservation gate runs on
    each), then ``fast_integrator`` runs ``steps_per_day`` ``substep`` calls at
    ``fast_dt`` (``n`` kept). ``substep`` skips the conservation gate, so the driver
    asserts it after **each** fast sub-step over the full shared ledger — keeping the
    every-step teeth. ``states`` holds one entry per **master day**, not per slow step
    (length ``days + 1``; a golden pins the final one) — so a station trajectory stays
    day-indexed even when the slow domain sub-steps. ⚠ Callers that slice or modulo a
    station trajectory therefore work in **days**, unlike biosphere trajectories, which
    are step-indexed.

    ``slow_steps_per_day`` defaults to ``1``, which reproduces the original
    one-slow-step-per-day driver byte-for-byte.
    ``total_rationed`` sums both integrators' Euler-backstop firings (validation asserts
    ``== 0``); ``events`` are extinction events (empty on the well-fed station seams).

    **Scheduled slow-domain reset (P6.7).** ``slow_reset`` is the two-rate analogue of
    :func:`domains.biosphere.season.run_season`'s ``reset`` hook — a schedule-agnostic
    ``(n, state) -> state`` consulted **once per master day, before that day's first
    slow sub-step**. It returns ``state`` *unchanged* on a non-reset day or a new
    ``State`` on a boundary; the calendar lives in the caller's closure. ⚠ ``n`` is the
    slow domain's **step** count, not the day count (see the module note), so that
    closure's period must be in steps — ``n % steps_for(year) == 0``, never
    ``n % year == 0``. When a reset fires the driver re-asserts the conservation gate
    across it, so "conserved at every point" stays literally true even across the
    discrete intervention (``annual_reset`` moves CARBON only between in-system stocks,
    touching no other quantity — the assert is the teeth that proves it in the *coupled*
    ledger). This is what the ≤7-day greenhouse / lighting / harvest runs never needed
    (sub-seasonal, so ``annual_reset`` never fired) but the multi-year sealed station
    does — without it the biosphere never re-sows. **Default ``None`` ⇒ byte-identical
    to the pre-P6.7 driver** (the greenhouse / lighting / harvest goldens unaffected).

    Requires ``fast_dt · steps_per_day == 86400`` s **and**
    ``slow_dt · slow_steps_per_day == 1`` day, so the two operators are split over the
    same interval; else a ``ValueError``.
    """
    if fast_dt * steps_per_day != SECONDS_PER_DAY:
        raise ValueError(
            f"fast_dt*steps_per_day must equal one day ({SECONDS_PER_DAY} s) so the "
            f"fast operator covers one master day, got {fast_dt}*{steps_per_day} = "
            f"{fast_dt * steps_per_day}"
        )
    if slow_dt * slow_steps_per_day != DAYS_PER_MASTER_DAY:
        raise ValueError(
            f"slow_dt*slow_steps_per_day must equal one day "
            f"({DAYS_PER_MASTER_DAY}) so the slow operator covers one master day, got "
            f"{slow_dt}*{slow_steps_per_day} = {slow_dt * slow_steps_per_day}"
        )
    states = [state]
    total_rationed = 0
    events: list[Event] = []
    for _day in range(days):
        # Scheduled slow-domain reset (the re-sow hook), applied once per master day at
        # a slow-step boundary. ⚠ n is a STEP count, so the closure's period must be in
        # steps. Conservation re-asserted across it (the run_season idiom): annual_reset
        # is CARBON-conserving, so this proves it in the coupled ledger.
        if slow_reset is not None:
            reset_state = slow_reset(state.n, state)
            if reset_state is not state:
                conservation.assert_conserved(state, reset_state)
                state = reset_state
        # Slow operator: slow_steps_per_day sub-steps covering one day. step_report (not
        # substep) advances the phenology aux and bumps n by 1 each time. The weather
        # resolver indexes physical time int(n*dt), so it reads the right row whatever
        # the step size. Each call runs its own conservation gate.
        for _ in range(slow_steps_per_day):
            slow_report = slow_integrator.step_report(state, slow_resolver, slow_dt)
            state = slow_report.state
            total_rationed += slow_report.rationed
            events.extend(slow_report.events)
        # Fast operator: steps_per_day sub-steps at fast_dt (n kept). substep skips the
        # conservation assert, so we own it here — after each sub-step, over the full
        # shared ledger — keeping the every-step teeth.
        for _ in range(steps_per_day):
            before = state
            fast_report = fast_integrator.substep(state, fast_resolver, fast_dt)
            state = fast_report.state
            conservation.assert_conserved(before, state)
            total_rationed += fast_report.rationed
            events.extend(fast_report.events)
        states.append(state)
    return states, total_rationed, tuple(events)
