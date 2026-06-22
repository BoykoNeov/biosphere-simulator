"""Step-4 regression-snapshot gate: the golden perennial-chamber multi-year run.

Pins the Phase-3 Step-4 (P3.4) capstone — the canonical perennial sealed chamber
(``PERENNIAL_CHAMBER_SCENARIO``, the season weather tiled ``PERENNIAL_CHAMBER_YEARS×``,
Euler-daily, with :func:`season.run_perennial`'s annual phenology reset / re-sow) —
bit-exactly. The **final State** is serialized via the ``sim_io`` hex-float serializer
and byte-compared to a committed golden, so any bit change in the perennial output (a
flow law, the reset redistribution, a param YAML, the reduction order, the chamber/reset
sizing) surfaces here. The third golden (open field ``test_regression_season.py``;
sealed ``test_regression_sealed_season.py``); the validation phenomena (sustained
oscillation, genuine closure / loss-sink == 0.0, conservation, determinism) are pinned
behaviourally in ``test_perennial_chamber.py``.

Mirrors the sealed golden: full ``State`` via ``sim_io.dumps`` (hex-float, incl. the aux
``thermal_time`` and every boundary stock), Euler only, regeneration a separate explicit
``__main__`` action. The generator bakes in the **pre-golden closure gate** (the
advisor's load-bearing catch): it asserts ``rationed == 0``, ``events == ()`` AND the
carbon loss-sink == 0.0 on this exact scenario — the line between "closed" and "closed
for these knobs" — before the bytes can be pinned.

**Bit-stability caveat** (as for the other goldens): the season uses transcendentals
(``exp``/``pow``/``sin``) which IEEE-754 does not mandate correctly-rounded, so this
golden is bit-identical **within a build** (determinism #7;
``test_perennial_is_deterministic`` confirms) but cross-platform last-ULP differences
are tolerance territory (the cross-port concern). Regenerate (review the diff) if the
toolchain moves.
"""

import json
from pathlib import Path

import sim_io
from domains.biosphere.season import (
    PERENNIAL_CHAMBER_SCENARIO,
    PERENNIAL_CHAMBER_YEARS,
    build_season,
    run_perennial,
    weather_resolver,
)
from simcore.boundary import loss_sink_id
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.state import State

GOLDEN_DIR = Path(__file__).parent / "regression" / "golden"
GOLDEN_PATH = GOLDEN_DIR / "perennial_chamber_state.json"

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


def _final_state() -> State:
    """Run the canonical perennial multi-year season (Euler); return the final State.

    The single source of truth for the committed golden and the load-back test. Bakes in
    the **pre-golden closure gate**: the golden comes from a ``rationed == 0`` /
    no-extinction / loss-sink-empty trajectory by construction — death routes to
    ``litter_carbon`` (in-system), never to the BOUNDARY loss-sink, so "genuinely
    closed" holds for *these* committed knobs (not just the mechanism in the abstract).
    """
    year = len(_weather())
    weather = _weather() * PERENNIAL_CHAMBER_YEARS
    state, registry = build_season(PERENNIAL_CHAMBER_SCENARIO)
    states, rationed, events = run_perennial(
        EulerIntegrator(registry),
        state,
        PERENNIAL_CHAMBER_SCENARIO,
        weather_resolver(weather, PERENNIAL_CHAMBER_SCENARIO),
        1.0,
        len(weather),
        year=year,
    )
    assert rationed == 0, "golden perennial run must be well-fed (no arbitration)"
    assert events == (), "golden perennial run must be extinction-free"
    carbon_loss_sink = loss_sink_id(Quantity.CARBON)
    assert all(s.stocks[carbon_loss_sink].amount == 0.0 for s in states), (
        "golden perennial run must be genuinely closed (carbon loss-sink stays 0.0 — "
        "death routes to litter, not the BOUNDARY loss-sink)"
    )
    return states[-1]


def test_perennial_golden_bytes_match() -> None:
    # Byte-exact compare against the committed golden — any bit change in the perennial
    # output fails here (within-build; see the transcendental caveat in the module doc).
    expected = sim_io.dumps(_final_state()).encode("utf-8")
    assert expected == GOLDEN_PATH.read_bytes()


def test_perennial_golden_loads_back() -> None:
    # The committed golden round-trips back to the exact final State (it routes through
    # the core constructors, so a tampered golden fails to load).
    text = GOLDEN_PATH.read_text(encoding="utf-8")
    assert sim_io.loads(text) == _final_state()


def _regenerate() -> None:
    """Rewrite the committed perennial golden from the current engine output.

    A deliberately separate, explicit action — NOT reachable from a test run. Run via::

        uv run python tests/test_regression_perennial_season.py

    Review the diff before committing: a change here means the perennial output moved.
    """
    GOLDEN_PATH.write_bytes(sim_io.dumps(_final_state()).encode("utf-8"))
    print(f"wrote {GOLDEN_PATH}")


if __name__ == "__main__":
    _regenerate()
