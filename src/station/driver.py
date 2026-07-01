"""The station's two-rate master-step driver (P6.3+): one slow domain + one fast domain.

The shared stepping harness for every station seam that couples a **day-scale** domain
(the biosphere: structurally ``dt = 1`` per-day, weather indexed by the integer step
``n``, a ``thermal_time`` phenology aux that must advance) to a **second-scale** domain
(the cabin / Power: ``dt = 60`` s / ``dt = 3600`` s, no aux). ``simcore.multirate``
cannot bridge these — it splits ONE shared master ``dt`` (``dt/n_sub``), and no single
master ``dt`` serves both *time units*; and it composes ``substep`` only, which by
design freezes the biosphere's phenology aux. So the station does the operator split
**by hand**, calling each domain's own integrator with its own ``dt`` (extracted here as
the **second** two-rate instance — the greenhouse was the first, the
bespoke-until-second rhythm).

**The split (Lie, slow-first).** Per master day: the **slow** domain takes ONE
``step_report`` at ``slow_dt`` (advancing its aux **and** ``n`` — so ``n`` stays the day
count and a frozen day-indexed weather resolver reads the right row; its own
conservation gate covers this sub-operation), then the **fast** domain takes
``steps_per_day`` ``substep`` calls at ``fast_dt`` (keeping ``n``). ``substep``
deliberately skips the conservation assert, so the driver re-asserts it after **each**
fast sub-step over the whole shared ledger — preserving the "every step conserves"
teeth. Requires ``fast_dt · steps_per_day == 86400`` (one day) so the slow domain's
once-daily step maps ``n`` to the day the weather table indexes.

Two disjoint registries over **one** shared stock dict + two integrators — exactly
``simcore.multirate``'s model, orchestrated by hand for per-domain ``dt`` + aux. The two
domains may share stocks (the greenhouse: the biosphere's gas pools ARE the cabin air)
or share **none** (lighting: Power and the biosphere are coupled only by a forcing
schedule, decision #16) — either way the combined ledger balances per-quantity, since
each flow touches only its own domain's stocks.

All public integrator methods → **zero core change**. Pure stdlib only.
"""

from simcore import conservation
from simcore.environment import SourceResolver
from simcore.events import Event
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.state import State

MasterStepIntegrator = EulerIntegrator | Rk4Integrator

# Seconds in one biosphere day — the ``fast_dt · steps_per_day`` one master step must
# advance so ``n`` stays the day count the day-indexed weather resolver reads.
SECONDS_PER_DAY: float = 86400.0


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
) -> tuple[list[State], int, tuple[Event, ...]]:
    """Step ``days`` master days (slow once + fast ×``steps_per_day`` each), slow-first.

    Per day: the ``slow_integrator`` runs one ``step_report`` at ``slow_dt`` (advancing
    any aux **and** ``n`` — its own conservation gate runs), then ``fast_integrator``
    runs ``steps_per_day`` ``substep`` calls at ``fast_dt`` (``n`` kept). ``substep``
    skips the conservation gate, so the driver asserts it after **each** fast sub-step
    over the full shared ledger — keeping the every-step teeth. ``states`` holds one
    entry per day boundary (length ``days + 1``; a golden pins the final one).
    ``total_rationed`` sums both integrators' Euler-backstop firings (validation asserts
    ``== 0``); ``events`` are extinction events (empty on the well-fed station seams).

    Requires ``fast_dt · steps_per_day == 86400`` (one day) so the slow domain's
    once-daily step maps ``n`` to the day a day-indexed weather resolver reads; else a
    ``ValueError``.
    """
    if fast_dt * steps_per_day != SECONDS_PER_DAY:
        raise ValueError(
            f"fast_dt*steps_per_day must equal one day ({SECONDS_PER_DAY} s) so n "
            f"stays the day count, got {fast_dt}*{steps_per_day} = "
            f"{fast_dt * steps_per_day}"
        )
    states = [state]
    total_rationed = 0
    events: list[Event] = []
    for _day in range(days):
        # Slow operator: one full day-step. step_report advances the phenology aux
        # (substep would not) and bumps n by 1 — so n counts days and a frozen
        # day-indexed weather table reads the right row. Its own conservation gate
        # covers this.
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
