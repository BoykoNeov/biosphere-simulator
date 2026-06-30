"""Regression-snapshot gate: the golden N-limited open-field run (additive scenario).

Pins ``N_LIMITED_SCENARIO`` (open field, single season, Euler-daily) bit-exactly — the
dormant-machinery scenario that drives the ``f_N`` photosynthesis limiter below 1
(N-limitation by dilution; see ``test_n_limited.py`` for the phenomena). The **final
State** is serialized via the ``sim_io`` hex-float serializer and byte-compared to a
committed golden, so any bit change in the N-limited trajectory (the limiter wiring, a
flow law, the reduction order, a param YAML, the scenario sizing) surfaces here.

**This is an additive, NON-reference golden** — not in
``docs/biosphere-reference.manifest.json``. It locks a deliberately-stressed code path
so the never-run-hot ``f_N`` integration cannot silently change under Phase 5; the seven
frozen reference goldens are untouched and byte-identical.

Mirrors ``test_regression_season.py`` (the open-field discipline): full ``State`` via
``sim_io.dumps``, Euler only, regeneration a separate explicit ``__main__`` action. The
generator bakes in a **pre-golden gate that is specific to this scenario's purpose**: it
asserts the run is well-fed / extinction-free AND that ``f_N`` *actually bit* (min < the
bite ceiling) before the bytes can be pinned — so the golden is **impossible to
regenerate from a non-biting run** (a sizing regression that restored ``f_N ≡ 1``
would fail the gate, not silently re-freeze a degenerate trajectory).

**Bit-stability caveat** (as for the other goldens): the season uses transcendentals
(``exp``/``pow``/``sin``) which IEEE-754 does not mandate correctly-rounded, so this
golden is bit-identical **within a build** but cross-platform last-ULP differences are
tolerance territory. Regenerate (review the diff) if the toolchain moves.
"""

import json
from pathlib import Path

import sim_io
from domains.biosphere.loader import load_nitrogen_params
from domains.biosphere.nitrogen import nitrogen_stress_factor
from domains.biosphere.scenario import N_LIMITED_SCENARIO, N_LIMITED_YEARS
from domains.biosphere.season import (
    LEAF_C,
    PLANT_N,
    ROOT_C,
    STEM_C,
    build_season,
    run_season,
    weather_resolver,
)
from simcore.boundary import loss_sink_id
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.state import State

GOLDEN_DIR = Path(__file__).parent / "regression" / "golden"
GOLDEN_PATH = GOLDEN_DIR / "n_limited_state.json"

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"

# The golden MUST come from a genuinely N-limited trajectory: f_N has to bite below this
# ceiling, or the scenario has regressed to potential production and the golden is
# meaningless. Encodes the scenario's whole purpose into its provenance.
_BITE_CEILING = 0.9


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


def _final_state() -> State:
    """Run the canonical N-limited season (Euler); return the final State.

    The single source of truth for the committed golden and the load-back test. Bakes in
    the **pre-golden gate**: the golden comes from a ``rationed == 0`` / extinction-free
    / loss-sink-empty trajectory in which ``f_N`` *actually bit* (min < bite ceiling)
    — so a future sizing regression that restored ``f_N ≡ 1`` fails here rather than
    silently re-freezing a non-biting (degenerate) run.
    """
    weather = _weather() * N_LIMITED_YEARS
    state, registry = build_season(N_LIMITED_SCENARIO)
    states, rationed, events = run_season(
        EulerIntegrator(registry),
        state,
        weather_resolver(weather, N_LIMITED_SCENARIO),
        1.0,
        len(weather),
    )
    assert rationed == 0, "golden N-limited run must be well-fed (no arbitration)"
    assert events == (), "golden N-limited run must be extinction-free"
    carbon_loss_sink = loss_sink_id(Quantity.CARBON)
    assert all(s.stocks[carbon_loss_sink].amount == 0.0 for s in states), (
        "golden N-limited run must keep the carbon loss-sink 0.0 (stress, not a kill)"
    )
    nitro = load_nitrogen_params()
    f_n_min = min(
        nitrogen_stress_factor(
            s.stocks[PLANT_N].amount,
            s.stocks[LEAF_C].amount + s.stocks[STEM_C].amount + s.stocks[ROOT_C].amount,
            n_residual_per_mol_c=nitro.n_residual_per_mol_c,
            n_critical_per_mol_c=nitro.n_critical_per_mol_c,
        )
        for s in states
    )
    assert f_n_min < _BITE_CEILING, (
        f"golden N-limited run must actually bite (min f_N {f_n_min} < {_BITE_CEILING})"
        " — a non-biting trajectory must not be pinnable as this golden"
    )
    return states[-1]


def test_n_limited_golden_bytes_match() -> None:
    # Byte-exact compare against the committed golden — any bit change in the N-limited
    # output fails here (within-build; see the transcendental caveat in the module doc).
    expected = sim_io.dumps(_final_state()).encode("utf-8")
    assert expected == GOLDEN_PATH.read_bytes()


def test_n_limited_golden_loads_back() -> None:
    # The committed golden round-trips back to the exact final State.
    text = GOLDEN_PATH.read_text(encoding="utf-8")
    assert sim_io.loads(text) == _final_state()


def _regenerate() -> None:
    """Rewrite the committed N-limited golden from the current engine output.

    A deliberately separate, explicit action — NOT reachable from a test run. Run via::

        uv run python tests/test_regression_n_limited_season.py

    Review the diff before committing: a change here means the N-limited output moved.
    """
    GOLDEN_PATH.write_bytes(sim_io.dumps(_final_state()).encode("utf-8"))
    print(f"wrote {GOLDEN_PATH}")


if __name__ == "__main__":
    _regenerate()
