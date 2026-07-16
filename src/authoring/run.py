"""Run an interpreted scenario (single-rate, no-reset) — the Step-0 harness.

Mirrors the standalone ``run_crew`` / ``run_power`` / ``run_eclss`` drivers: step
the requested integrator ``steps`` times, returning the full trajectory (including
the initial state), the summed arbitration-backstop firings, and any extinction
events. **No reset hook** and **single-rate only** — the two-rate master-day driver
(``station.driver``) and phenology/annual-reset are a later Phase-9 step, exactly as
the Rust side layered them after the single-rate session.

The every-step conservation gate runs inside ``integrator.step_report``, so a
completed run is itself proof the authored graph balanced every step (an authored
mis-wiring surfaces here as a ``ConservationError`` — the safety property).

**Rationing is a hard error here (post-roadmap; see ``RationedError``).** Conservation
is *not* sufficient: the ``dt`` hazard produces a run that balances every step and
still asphyxiates the cabin, because the backstop clamps the over-draw at zero rather
than going negative. So this harness gates on ``total_rationed == 0`` too — the second
half of "the authored graph is sound".
"""

from __future__ import annotations

from authoring.errors import AuthoringError, RationedError
from authoring.interpreter import BuiltScenario
from simcore.events import Event
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.state import State

_INTEGRATORS = {"euler": EulerIntegrator, "rk4": Rk4Integrator}


def run_scenario(
    built: BuiltScenario,
    *,
    allow_rationing: bool = False,
) -> tuple[list[State], int, tuple[Event, ...]]:
    """Step ``built`` to completion; return ``(states, total_rationed, events)``.

    ``states`` has length ``steps + 1`` (initial + one per step). An unknown
    integrator name is an ``AuthoringError``.

    **Raises ``RationedError`` if the Euler backstop fired at all** (see that class:
    on an authored graph, rationing means the ``dt`` is wrong, and the failure is
    otherwise silent). Pass ``allow_rationing=True`` to opt back in to the old
    return-and-trust-the-caller behavior — for deliberately studying a rationed run
    (``tests/test_authoring_dt_hazard.py``), not for making a scenario "work".
    """
    integrator_cls = _INTEGRATORS.get(built.integrator)
    if integrator_cls is None:
        raise AuthoringError(
            f"unknown integrator {built.integrator!r} (known: {sorted(_INTEGRATORS)})"
        )
    integrator = integrator_cls(built.registry)
    state = built.state
    states = [state]
    total_rationed = 0
    events: list[Event] = []
    for _ in range(built.steps):
        report = integrator.step_report(state, built.resolver, built.dt)
        state = report.state
        states.append(state)
        total_rationed += report.rationed
        events.extend(report.events)
    if total_rationed > 0 and not allow_rationing:
        raise RationedError(
            f"the arbitration backstop fired {total_rationed} time(s) at dt="
            f"{built.dt!r} over {built.steps} step(s). On an authored graph this "
            f"means dt is too large for some flow's frozen rate constant: the "
            f"over-draw was clamped at zero, so the run still conserved every "
            f"quantity and still finished — but a clamped stock is an emptied one "
            f"(a cabin with no oxygen conserves mass perfectly). Reduce dt and "
            f"re-run; each flow type's constraint is tabulated under 'The dt "
            f"constraint' in docs/authoring-reference.md (e.g. ECLSS's frozen "
            f"rates want dt <= ~60 s). To inspect the rationed run instead of "
            f"failing, pass allow_rationing=True."
        )
    return states, total_rationed, tuple(events)
