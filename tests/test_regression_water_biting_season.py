"""Regression-snapshot gate: the golden water-biting sealed run (additive scenario).

Pins ``WATER_BITING_SCENARIO`` (sealed chamber, single season, Euler-daily) bit-exactly:
the dormant-machinery scenario that drives the closed water cycle's ``f_water`` limiter
below 1 (the frozen chambers tune it inert; see ``test_water_biting.py``). The **final
State** is serialized via ``sim_io`` and byte-compared to a committed golden, so any bit
change in the water-biting trajectory surfaces here.

**This is an additive, NON-reference golden** — not in
``docs/biosphere-reference.manifest.json``. It locks the never-run-hot ``f_water``
integration in a sealed run so it cannot silently change under Phase 5; the seven frozen
reference goldens are untouched and byte-identical.

Mirrors ``test_regression_sealed_season.py`` (the sealed discipline): full ``State`` via
``sim_io.dumps``, Euler only, regeneration a separate explicit ``__main__`` action. The
generator bakes in a **pre-golden gate specific to this scenario's purpose**: it asserts
the run is well-fed / extinction-free / loss-sink-empty AND that ``f_water`` *actually
bit* (min < the bite ceiling) before the bytes can be pinned — so a sizing regression
that quietly restored ``f_water ≡ 1`` fails the gate rather than re-freezing a dead run.

**Bit-stability caveat** (as for the other goldens): transcendentals make this golden
bit-identical **within a build**; cross-platform last-ULP differences are tolerance
territory. Regenerate (review the diff) if the toolchain moves.
"""

import json
from pathlib import Path

import sim_io
from domains.biosphere.scenario import WATER_BITING_SCENARIO, WATER_BITING_YEARS
from domains.biosphere.season import (
    SOIL_WATER,
    build_season,
    run_season,
    weather_resolver,
)
from domains.biosphere.transpiration import water_stress_factor
from golden_platform import windows_golden_only
from simcore.boundary import loss_sink_id
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.state import State

GOLDEN_DIR = Path(__file__).parent / "regression" / "golden"
GOLDEN_PATH = GOLDEN_DIR / "water_biting_state.json"

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"

# The golden MUST come from a genuinely water-limited trajectory: f_water has to bite
# below this ceiling, or the sealed water cycle has regressed to inert and the golden is
# meaningless. Encodes the scenario's whole purpose into its provenance.
_BITE_CEILING = 0.9


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


def _final_state() -> State:
    """Run the canonical water-biting season (Euler); return the final State.

    The single source of truth for the committed golden and the load-back test. Bakes in
    the **pre-golden gate**: the golden comes from a ``rationed == 0`` / extinction-free
    / loss-sink-empty trajectory in which ``f_water`` *actually bit* (min < the bite
    ceiling), so a future sizing regression that restored ``f_water ≡ 1`` fails here.
    """
    weather = _weather() * WATER_BITING_YEARS
    state, registry = build_season(WATER_BITING_SCENARIO)
    states, rationed, events = run_season(
        EulerIntegrator(registry),
        state,
        weather_resolver(weather, WATER_BITING_SCENARIO),
        1.0,
        len(weather),
    )
    assert rationed == 0, "golden water-biting run must be well-fed (no arbitration)"
    assert events == (), "golden water-biting run must be extinction-free"
    carbon_loss_sink = loss_sink_id(Quantity.CARBON)
    assert all(s.stocks[carbon_loss_sink].amount == 0.0 for s in states), (
        "golden water-biting run must keep the carbon loss-sink 0.0 (stress not a kill)"
    )
    f_water_min = min(
        water_stress_factor(
            s.stocks[SOIL_WATER].amount,
            sw_wilting=WATER_BITING_SCENARIO.sw_wilting,
            sw_critical=WATER_BITING_SCENARIO.sw_critical,
        )
        for s in states
    )
    assert f_water_min < _BITE_CEILING, (
        f"golden water-biting run must actually bite (min f_water {f_water_min} < "
        f"{_BITE_CEILING}) — a non-biting trajectory must not be pinned as this golden"
    )
    return states[-1]


@windows_golden_only
def test_water_biting_golden_bytes_match() -> None:
    # Byte-exact compare against the committed golden — any bit change fails here
    # (within-build; see the transcendental caveat in the module doc).
    expected = sim_io.dumps(_final_state()).encode("utf-8")
    assert expected == GOLDEN_PATH.read_bytes()


@windows_golden_only
def test_water_biting_golden_loads_back() -> None:
    # The committed golden round-trips back to the exact final State.
    text = GOLDEN_PATH.read_text(encoding="utf-8")
    assert sim_io.loads(text) == _final_state()


def _regenerate() -> None:
    """Rewrite the committed water-biting golden from the current engine output.

    A deliberately separate, explicit action — NOT reachable from a test run. Run via::

        uv run python tests/test_regression_water_biting_season.py

    Review the diff before committing: a change here means the water-biting run moved.
    """
    GOLDEN_PATH.write_bytes(sim_io.dumps(_final_state()).encode("utf-8"))
    print(f"wrote {GOLDEN_PATH}")


if __name__ == "__main__":
    _regenerate()
