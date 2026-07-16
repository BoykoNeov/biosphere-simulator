"""Authoring-layer error type (the boundary's own failure surface).

An :class:`AuthoringError` marks a scenario-file that is *structurally* invalid at
interpret time — an unknown flow type, a wiring dict that does not match the flow
type's constructor fields, a missing/spurious param-set reference. It is the
authoring analogue of ``config.ConfigError`` (bad param YAML): raised in the
boundary layer, before any engine step runs.

It is **distinct from** a runtime ``simcore.flow.ConservationError``: a well-formed
scenario that *wires* a flow badly (e.g. a carbon flow's withdrawal leg pointed at
an oxygen stock) interprets cleanly and then surfaces as a ``ConservationError`` on
the first step — the "bad wiring surfaces, never silently fixed" safety property
(Phase-9 decision B). ``AuthoringError`` catches only what is decidable from the
file structure alone.

:class:`RationedError` is the module's **second, later** surface, added post-roadmap:
a *runtime* verdict on a run that already completed. It is deliberately **not** an
``AuthoringError`` subclass — nothing about it is decidable from the file structure
(the same file at a smaller ``dt`` is fine), and by the paragraph above that is
precisely the line ``AuthoringError`` does not cross.
"""

from __future__ import annotations


class AuthoringError(Exception):
    """A scenario file is structurally invalid at interpret time."""


class RationedError(Exception):
    """An authored run needed the Euler arbitration backstop — so its ``dt`` is wrong.

    **Why this is an error and not a statistic.** The backstop is a *rare numerical
    guard* (``simcore.arbitration``), not a mechanism: it scales an over-draw so no
    stock goes negative. On the frozen scenarios it never fires, and every golden
    asserts ``rationed == 0``; ``simcore.integrator.StepReport`` calls a nonzero count
    "a failing gate, not a warning"; under RK4 the identical condition is already a
    hard ``ArbitrationError``. This class simply brings **authored Euler runs** in line
    with a verdict the rest of the project had already reached — it is not a new policy.

    **What it is protecting against (the reason it must raise rather than report).**
    Every frozen rate constant was sized against the ``dt`` of its own frozen scenario,
    and that sizing is part of the flow's positivity argument — but an author picks
    ``dt``. At ``dt = 3600`` ``eclss.co2_scrubber``'s ``k·dt`` is ``3.6``: it demands
    3.6x the entire CO₂ pool in one step. The backstop clamps it, so the run **does not
    raise, conserves every quantity every step, and completes** — with the cabin oxygen
    at zero. *Mass conservation is not survival.* The only signal was the ``rationed``
    count, and ``states, _, _ = run_scenario(built)`` discards it. See "The dt
    constraint" in ``docs/authoring-reference.md`` and
    ``tests/test_authoring_dt_hazard.py``, which measured that silence.

    **Distinct from ``simcore.arbitration.ArbitrationError``**, which it deliberately
    does not reuse despite the near-identical trigger: that one aborts *mid-step* under
    RK4 and is documented as "not a recoverable condition", whereas this is a post-hoc
    verdict on a **completed** run and *is* recoverable — ``allow_rationing=True``
    returns the trajectory for inspection. Different lifetime, different recovery.

    **Not raised by the station/Godot path**, which reaches the same verdict in its own
    idiom: ``station.objectives`` scores a rationed run as ``survived = False`` (a
    blackout that rations ``power.load_draw`` is a *lost game*, not a crash). A player
    should see the failure; an author calling a library function gets an exception.
    """
