"""Step-11 regression-snapshot gate: the golden single-producer season run.

Pins the assembled winter-wheat season (``domains.biosphere.season``) bit-exactly. The
full season — ``build_season`` + the NASAPower-weather resolver, Euler-daily for
``len(weather)`` steps — is run and its **final State** serialized via the ``sim_io``
hex-float serializer and **byte-compared** to a committed golden. Any bit change in the
season output (a flow law, the budget recompute, the reduction order, a param YAML, a
weather conversion) surfaces here.

Design mirrors ``test_regression_demo.py``:

* **Snapshot the full ``State`` via ``sim_io.dumps``** (not ``observe``) — bit-exact
  hex-float bytes, including the aux ``thermal_time`` accumulator (schema v2) and the
  boundary stocks (``co2_resp`` total respiration, the loss-sink) the guard watches.
* **Euler only.** The crop scenario *selects* Euler (P3: crop physiology is Euler-daily;
  the daily-integrated canopy flux is not RK4-refinable). RK4 stays for the engine
  gates, not the crop season — so there is one golden, not two.
* **Regeneration is a separate, explicit ``__main__`` action**, never a test side effect
  (a verify run is strictly read-only).

**Bit-stability caveat (vs. the demo golden).** Unlike the demo (only basic arithmetic,
IEEE-correctly-rounded, platform-stable), the season uses **transcendentals**: ``exp``
in FvCB / Penman-Monteith / SVP, ``sin``/``acos`` in daylength, ``pow`` in ``Q10`` —
which IEEE-754 does **not** mandate correctly-rounded; they are libm-dependent. So this
golden is bit-identical **within a build** (determinism invariant #7: same
interpreter/libm gives identical bytes; ``test_season_is_deterministic`` confirms) but
cross-platform last-ULP differences are **tolerance territory** — exactly the cross-port
(Rust) concern the project gates by tolerance, not bytes. Regenerate (and review the
diff) if the toolchain moves.
"""

import json
from pathlib import Path

import sim_io
from domains.biosphere.season import build_season, run_season, weather_resolver
from simcore.integrator import EulerIntegrator
from simcore.state import State

GOLDEN_DIR = Path(__file__).parent / "regression" / "golden"
GOLDEN_PATH = GOLDEN_DIR / "season_euler_state.json"

# The committed raw-weather fixture drives the canonical season (read as JSON, no PCSE).
_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


def _final_state() -> State:
    """Run the canonical season (Euler, full weather length); return the final State.

    The single source of truth for the committed golden and the verify tests. Re-asserts
    the run is well-fed + extinction-free — the golden comes from a non-arbitrating,
    non-extinction trajectory by construction (the Step-11 ``rationed == 0`` invariant).
    """
    weather = _weather()
    state, registry = build_season()
    states, rationed, events = run_season(
        EulerIntegrator(registry), state, weather_resolver(weather), 1.0, len(weather)
    )
    assert rationed == 0, "golden season must be well-fed (no arbitration firing)"
    assert events == (), "golden season must be extinction-free"
    return states[-1]


def test_season_golden_bytes_match() -> None:
    # Byte-exact compare against the committed golden — any bit change in the season
    # output fails here (within-build; see the transcendental caveat in the module doc).
    expected = sim_io.dumps(_final_state()).encode("utf-8")
    assert expected == GOLDEN_PATH.read_bytes()


def test_season_golden_loads_back() -> None:
    # The committed golden round-trips back to the exact final State (it routes through
    # the core constructors, so a tampered golden fails to load).
    text = GOLDEN_PATH.read_text(encoding="utf-8")
    assert sim_io.loads(text) == _final_state()


def _regenerate() -> None:
    """Rewrite the committed season golden from the current engine output.

    A deliberately separate, explicit action — NOT reachable from a test run. Run via::

        uv run python tests/test_regression_season.py

    Review the diff before committing: a change here means the season output moved.
    """
    GOLDEN_PATH.write_bytes(sim_io.dumps(_final_state()).encode("utf-8"))
    print(f"wrote {GOLDEN_PATH}")


if __name__ == "__main__":
    _regenerate()
