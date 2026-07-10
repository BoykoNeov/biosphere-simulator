"""Step-7 regression-snapshot gate: the golden minimal-consumer multi-year run.

Pins the Phase-3 Step-7 capstone — the perennial sealed chamber plus **one herbivore**
(``CONSUMER_CHAMBER_SCENARIO``, the season weather tiled ``CONSUMER_CHAMBER_YEARS×``,
Euler-daily, with :func:`season.run_perennial`'s annual phenology reset / re-sow) —
bit-exactly. The **final State** is serialized via the ``sim_io`` hex-float serializer
and byte-compared to a committed golden, so any bit change in the consumer output (a
flow
law — grazing / consumer respiration / mortality — a herbivory param, the reset, the
reduction order, or the consumer sizing) surfaces here. The fourth golden (open field
``test_regression_season.py``; sealed ``test_regression_sealed_season.py``; perennial
``test_regression_perennial_season.py``); the validation phenomena (consumer
persistence,
the leaf↓/CO₂↑ cascade, genuine closure, conservation, the per-compartment ledger,
determinism) are pinned behaviourally in ``test_consumer.py``.

Mirrors the perennial golden: full ``State`` via ``sim_io.dumps`` (hex-float, incl. the
aux ``thermal_time`` and every boundary stock), Euler only, regeneration a separate
explicit ``__main__`` action. The generator bakes in the **pre-golden closure gate**
(the
Step-4 rhythm): it asserts ``rationed == 0``, ``events == ()`` AND the carbon loss-sink
==
0.0 on this exact scenario — death (plant *and* consumer) routes to ``litter_carbon``,
never to the BOUNDARY loss-sink, so "genuinely closed" holds for *these* committed knobs
—
before the bytes can be pinned.

**Bit-stability caveat** (as for the other goldens): the season uses transcendentals
(``exp``/``pow``/``sin``) which IEEE-754 does not mandate correctly-rounded, so this
golden is bit-identical **within a build** (determinism #7; the determinism test
confirms)
but cross-platform last-ULP differences are tolerance territory. Regenerate (review the
diff) if the toolchain moves.
"""

import json
from pathlib import Path

import sim_io
from domains.biosphere.season import (
    CONSUMER_CHAMBER_SCENARIO,
    CONSUMER_CHAMBER_YEARS,
    build_season,
    run_perennial,
    weather_resolver,
)
from golden_platform import windows_golden_only
from simcore.boundary import loss_sink_id
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.state import State

GOLDEN_DIR = Path(__file__).parent / "regression" / "golden"
GOLDEN_PATH = GOLDEN_DIR / "consumer_chamber_state.json"

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


def _final_state() -> State:
    """Run the canonical consumer multi-year season (Euler); return the final State.

    The single source of truth for the committed golden and the load-back test. Bakes in
    the **pre-golden closure gate**: the golden comes from a ``rationed == 0`` /
    no-extinction / loss-sink-empty trajectory by construction — the consumer's
    mortality
    routes to ``litter_carbon`` (in-system, decomposable), never to the BOUNDARY
    loss-sink, so "genuinely closed" holds for *these* committed knobs (the consumer is
    sized to persist while the plant still fills grain for ``annual_reset`` — the
    recoverable regime).
    """
    year = len(_weather())
    weather = _weather() * CONSUMER_CHAMBER_YEARS
    state, registry = build_season(CONSUMER_CHAMBER_SCENARIO)
    states, rationed, events = run_perennial(
        EulerIntegrator(registry),
        state,
        CONSUMER_CHAMBER_SCENARIO,
        weather_resolver(weather, CONSUMER_CHAMBER_SCENARIO),
        1.0,
        len(weather),
        year=year,
    )
    assert rationed == 0, "golden consumer run must be well-fed (no arbitration)"
    assert events == (), "golden consumer run must be extinction-free"
    carbon_loss_sink = loss_sink_id(Quantity.CARBON)
    assert all(s.stocks[carbon_loss_sink].amount == 0.0 for s in states), (
        "golden consumer run must be genuinely closed (carbon loss-sink stays 0.0 — "
        "the consumer's death routes to litter, not the BOUNDARY loss-sink)"
    )
    return states[-1]


@windows_golden_only
def test_consumer_golden_bytes_match() -> None:
    # Byte-exact compare against the committed golden — any bit change in the consumer
    # output fails here (within-build; see the transcendental caveat in the module doc).
    expected = sim_io.dumps(_final_state()).encode("utf-8")
    assert expected == GOLDEN_PATH.read_bytes()


@windows_golden_only
def test_consumer_golden_loads_back() -> None:
    # The committed golden round-trips back to the exact final State (it routes through
    # the core constructors, so a tampered golden fails to load).
    text = GOLDEN_PATH.read_text(encoding="utf-8")
    assert sim_io.loads(text) == _final_state()


def _regenerate() -> None:
    """Rewrite the committed consumer golden from the current engine output.

    A deliberately separate, explicit action — NOT reachable from a test run. Run via::

        uv run python tests/test_regression_consumer_season.py

    Review the diff before committing: a change here means the consumer output moved.
    """
    GOLDEN_PATH.write_bytes(sim_io.dumps(_final_state()).encode("utf-8"))
    print(f"wrote {GOLDEN_PATH}")


if __name__ == "__main__":
    _regenerate()
