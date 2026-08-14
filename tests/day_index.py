"""Reading a STEP-indexed trajectory by physical DAY — one implementation, not five.

**Why this module exists, and it is the batch that created it that argues for it.**
A run's trajectory holds one entry per integration *step*; almost everything a test
compares it against — an oracle reference table, a milestone written in days, a
per-year window — is in *days*. Those were the same list index while
``BIO_DT`` was 1.0, and stopped being it on 2026-08-14. The step unfreeze converted
every run's *length* (``run_season(..., steps_for(days))``) and left every index that
**reads the trajectory back**, which was 18 red tests across five files and one that
failed in a way that reads as nonsense rather than as a wrong number.

``domains.biosphere.step.day_of`` already described this idiom in prose —
*"``states[steps_for(d)]`` is the state at the start of day ``d``"* — and provided no
function for it. So when five files needed it, five files invented it, in three
different shapes, one of them an inlined generator expression with no helper at all.
That is the same class as a hand-copied constant, and this batch's own record says
what to do about it: *re-measuring a copied constant fixes one occurrence; tying it to
its source fixes the class.* The sixth file to need this now has a canonical thing to
import.

⚠ **Not every trajectory index belongs in days, and converting one that does not is a
regression.** A test that asks for the state *at development stage 0.5* compares
against no day at all; its step-indexed series and index are self-consistent AND finer,
and converting it moved a measurement by sampling up to three steps late. **An index
has to be in days only when something it is compared against is.**

⚠ Station trajectories from ``station.driver.run_master_day`` are ALREADY day-indexed —
the driver appends one state per master day. Do not use this on those.
"""

from __future__ import annotations

from domains.biosphere.step import day_of, steps_for


def days_in[T](states: list[T]) -> int:
    """How many physical days a step-indexed trajectory covers — NOT ``len(states)``.

    ⚠ The reason this is a named function rather than an inline expression: the
    comparison it feeds is usually ``min(len(reference), days_in(states))``, and the
    bug it replaces was ``min(len(reference), len(states))`` — which returned
    ``min(305, 306) = 305`` and was right, and returns ``min(305, 1221) = 305`` and is
    wrong, **without changing value**. A ``min`` over two quantities in different units
    cannot announce itself; only the name can.
    """
    return day_of(len(states) - 1) + 1


def by_day[T](states: list[T], days: int | None = None) -> list[T]:
    """The entries at the START of each of the first ``days`` physical days.

    ``days=None`` takes every whole day the trajectory covers (:func:`days_in`).

    At ``dt = 1`` this is the identity, so converting a call site to it is a no-op on
    the old step — which is what a correct unit conversion looks like, and is the cheap
    check that a conversion has not changed a measurement it was only meant to re-index.

    ⚠ **Any index DERIVED from a series built here is itself a DAY index** — the return
    of a first-crossing search, an ``argmax`` — and must be used against a list from
    this function, never against the raw trajectory. That second half was six of the
    sites in the batch that motivated this module, and none of them would have been
    found by converting the series alone.
    """
    return [
        states[steps_for(d)] for d in range(days_in(states) if days is None else days)
    ]
