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
"""

from __future__ import annotations

from authoring.errors import AuthoringError
from authoring.interpreter import BuiltScenario
from simcore.events import Event
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.state import State

_INTEGRATORS = {"euler": EulerIntegrator, "rk4": Rk4Integrator}


def run_scenario(
    built: BuiltScenario,
) -> tuple[list[State], int, tuple[Event, ...]]:
    """Step ``built`` to completion; return ``(states, total_rationed, events)``.

    ``states`` has length ``steps + 1`` (initial + one per step). An unknown
    integrator name is an ``AuthoringError``.
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
    return states, total_rationed, tuple(events)
