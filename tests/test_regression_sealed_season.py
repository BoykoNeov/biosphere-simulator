"""Step-7 regression-snapshot gate: the golden sealed-chamber multi-year run.

Pins the Phase-2 capstone — the canonical sealed chamber (``SEALED_CHAMBER_SCENARIO``,
the season weather tiled ``SEALED_CHAMBER_YEARS×``, Euler-daily) — bit-exactly. The
**final State** is serialized via the ``sim_io`` hex-float serializer and byte-compared
to a committed golden, so any bit change in the sealed output (a flow law, the f_O2
factor, a param YAML, the reduction order, the chamber sizing) surfaces here. The first
**sealed** golden (the open field's is ``test_regression_season.py``); the validation
phenomena (O₂ depletion, the exact O₂↔CO₂ anti-correlation, f_O2 load-bearing,
conservation, determinism) are pinned behaviourally in ``test_sealed_chamber.py``.

Mirrors ``test_regression_season.py``: full ``State`` via ``sim_io.dumps`` (hex-float,
incl. the aux ``thermal_time`` and every boundary stock), Euler only (the crop scenario
selects Euler; P3), and regeneration is a separate explicit ``__main__`` action.

**Bit-stability caveat** (as for the open-field golden): the season uses transcendentals
(``exp``/``pow``/``sin``) which IEEE-754 does not mandate correctly-rounded, so this
golden is bit-identical **within a build** (determinism #7;
``test_sealed_run_is_deterministic`` confirms) but cross-platform last-ULP differences
are tolerance territory (the cross-port concern). Regenerate (review the diff) if the
toolchain moves.
"""

import json
from pathlib import Path

import sim_io
from domains.biosphere.season import (
    SEALED_CHAMBER_SCENARIO,
    SEALED_CHAMBER_YEARS,
    build_season,
    run_season,
    weather_resolver,
)
from golden_platform import windows_golden_only
from simcore.integrator import EulerIntegrator
from simcore.state import State

GOLDEN_DIR = Path(__file__).parent / "regression" / "golden"
GOLDEN_PATH = GOLDEN_DIR / "sealed_chamber_state.json"

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


def _final_state() -> State:
    """Run the canonical sealed multi-year season (Euler); return the final State.

    The single source of truth for the committed golden and the load-back test.
    Re-asserts the run is non-arbitrating + extinction-free — the golden comes from a
    ``rationed == 0`` / no-extinction trajectory by construction (Step 7), even though
    O₂ depletes ~99 % (f_O2 self-limits the draw).
    """
    weather = _weather() * SEALED_CHAMBER_YEARS
    state, registry = build_season(SEALED_CHAMBER_SCENARIO)
    states, rationed, events = run_season(
        EulerIntegrator(registry),
        state,
        weather_resolver(weather, SEALED_CHAMBER_SCENARIO),
        1.0,
        len(weather),
    )
    assert rationed == 0, "golden sealed run must be well-fed (no arbitration firing)"
    assert events == (), "golden sealed run must be extinction-free"
    return states[-1]


@windows_golden_only
def test_sealed_golden_bytes_match() -> None:
    # Byte-exact compare against the committed golden — any bit change in the sealed
    # output fails here (within-build; see the transcendental caveat in the module doc).
    expected = sim_io.dumps(_final_state()).encode("utf-8")
    assert expected == GOLDEN_PATH.read_bytes()


@windows_golden_only
def test_sealed_golden_loads_back() -> None:
    # The committed golden round-trips back to the exact final State (it routes through
    # the core constructors, so a tampered golden fails to load).
    text = GOLDEN_PATH.read_text(encoding="utf-8")
    assert sim_io.loads(text) == _final_state()


def _regenerate() -> None:
    """Rewrite the committed sealed golden from the current engine output.

    A deliberately separate, explicit action — NOT reachable from a test run. Run via::

        uv run python tests/test_regression_sealed_season.py

    Review the diff before committing: a change here means the sealed output moved.
    """
    GOLDEN_PATH.write_bytes(sim_io.dumps(_final_state()).encode("utf-8"))
    print(f"wrote {GOLDEN_PATH}")


if __name__ == "__main__":
    _regenerate()
