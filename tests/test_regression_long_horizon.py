"""Phase-4 Step-4 (P4.2): the canonical long-horizon goldens — capture, not invention.

Phase 4 invents no new ecosystem. It **pins** the existing closed scenarios at the
**decade-scale horizon** (``LONG_HORIZON_YEARS`` = 15 yr, the stability-validated
length, Step 1) as the canonical reference the freeze contract (Step 5) points at. Three
goldens, two kinds:

* **Long-horizon State snapshots** (``perennial_long_horizon_state.json`` /
  ``consumer_long_horizon_state.json``) — the final ``State`` of each 15-yr run,
  serialized via the ``sim_io`` hex-float serializer and byte-compared, exactly as the
  Phase-3 5-yr goldens (``test_regression_{perennial,consumer}_season.py``) do at 5 yr.
  The marginal value over the 5-yr goldens is real: a post-yr-5 regression in any stock
  the per-year summaries don't surface (soil_water, the N pools, the ``thermal_time``
  aux) shows up here.
* **The drift-summary golden** (``drift_summary.json``) — the *stability* signature, the
  genuinely new Phase-4 artifact. It pins the **per-year limit-cycle summaries** (peak
  ``leaf_c`` for both scenarios; year-end ``consumer_carbon`` for the consumer) as
  hex-float vectors **plus the period class** (perennial period-2, consumer period-1).
  This catches a regression in *stability* — a change to the cycle *shape* over the
  horizon — that a single final-state snapshot cannot: two trajectories with different
  mid-horizon amplitudes can share an endpoint, but not a per-year vector.

**What is deliberately NOT pinned: the mass-drift ``max|d_q|`` / ``slope`` numbers.**
They are round-off-scale (~1e-11) — byte-pinning them would catch only toolchain
round-off-pattern shifts (exactly the noise a golden must be insensitive to), not a real
regression: a mass-drift bug big enough to matter (a leak) perturbs the pools → perturbs
``peak_leaf`` → moves the vectors *and* the State snapshot. The meaningful axis-(a)
regression is already caught by the vectors + snapshots here and the *bound* assertions
in the decade / stress tests. Pin substance, not noise.

**This is the pinning layer only — it does NOT re-litigate the stability bounds.**
``test_decade_stability.py`` already runs both scenarios at exactly 15 yr and asserts
``is_stationary`` / ``is_period_2`` / ``non_collapsing`` / closure; this file just
freezes the bytes. The four Phase-3 goldens are **re-affirmed** (byte-identical, their
own tests), **not** regenerated — Step 2 (integrator escalation) was skipped, so nothing
forced a regen; a diff in any of the four would be a bug to investigate, not a regen.

**Pre-golden closure gate (the load-bearing discipline).** Each golden derives from a
run asserted ``rationed == 0`` / ``events == ()`` / carbon loss-sink ``== 0.0`` *before*
the bytes are pinned — the line between "closed" and "closed for these knobs" (already
proven to hold at 15 and 328 yr). **Bit-stability caveat:** the season uses
transcendentals (``exp``/``pow``/``sin``), not IEEE-754-correctly-rounded, so these
goldens are bit-identical **within a build** (the decade probe's
``test_decade_run_is_deterministic`` is the determinism evidence); cross-platform
last-ULP differences are tolerance territory (the cross-port concern). Regenerate
(review the diff) if the toolchain moves — run this module as ``__main__`` (below).

Mirrors the existing golden discipline: hex-float exactness (here via the stdlib
``float.hex`` / ``float.fromhex`` primitive ``sim_io`` itself uses — not a new
``sim_io`` API, which would be the speculative generality this codebase rejects), a
load-back test (a tampered golden fails), and a separate explicit ``__main__``
regeneration action. A module-scoped fixture runs each 15-yr scenario exactly once.
"""

import json
from collections.abc import Sequence
from pathlib import Path

import pytest

import sim_io
from domains.biosphere.drift import is_period_2, year_summaries
from domains.biosphere.season import (
    CONSUMER_CARBON,
    CONSUMER_CHAMBER_SCENARIO,
    LEAF_C,
    LONG_HORIZON_YEARS,
    PERENNIAL_CHAMBER_SCENARIO,
    SeasonScenario,
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
PERENNIAL_GOLDEN = GOLDEN_DIR / "perennial_long_horizon_state.json"
CONSUMER_GOLDEN = GOLDEN_DIR / "consumer_long_horizon_state.json"
DRIFT_SUMMARY_GOLDEN = GOLDEN_DIR / "drift_summary.json"

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"

_PERIOD_TRANSIENT = 8  # years to drop before the period check — reach the settled tail


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


def _run(scenario: SeasonScenario) -> list[State]:
    """Run ``scenario`` Euler-daily to ``LONG_HORIZON_YEARS``; return the trajectory.

    The single source of truth for both golden kinds (final-state snapshot + per-year
    summaries). Bakes in the **pre-golden closure gate**: the goldens come from a
    ``rationed == 0`` / no-extinction / loss-sink-empty trajectory by construction —
    death routes to ``litter_carbon`` (in-system), never to the BOUNDARY loss-sink, so
    "genuinely closed" holds for *these* committed knobs at the decade horizon, not just
    in the abstract.
    """
    year = len(_weather())
    weather = _weather() * LONG_HORIZON_YEARS
    state, registry = build_season(scenario)
    states, rationed, events = run_perennial(
        EulerIntegrator(registry),
        state,
        scenario,
        weather_resolver(weather, scenario),
        1.0,
        len(weather),
        year=year,
    )
    assert rationed == 0, "golden long-horizon run must be well-fed (no arbitration)"
    assert events == (), "golden long-horizon run must be extinction-free"
    carbon_loss_sink = loss_sink_id(Quantity.CARBON)
    assert all(s.stocks[carbon_loss_sink].amount == 0.0 for s in states), (
        "golden long-horizon run must be genuinely closed (carbon loss-sink stays 0.0, "
        "death routes to litter, not the BOUNDARY loss-sink)"
    )
    return states


def _trajectories() -> dict[str, list[State]]:
    """Both 15-yr trajectories (the closure gate fires inside :func:`_run`)."""
    return {
        "perennial": _run(PERENNIAL_CHAMBER_SCENARIO),
        "consumer": _run(CONSUMER_CHAMBER_SCENARIO),
    }


@pytest.fixture(scope="module")
def trajectories() -> dict[str, list[State]]:
    """Run each 15-yr scenario exactly once for the whole module."""
    return _trajectories()


# --- per-year summary functions (reference the domain stock ids) -------------


def _peak_leaf(segment: Sequence[State]) -> float:
    return max(s.stocks[LEAF_C].amount for s in segment)


def _year_end_consumer(segment: Sequence[State]) -> float:
    return segment[-1].stocks[CONSUMER_CARBON].amount


# --- the drift-summary: the stability signature ------------------------------


def _drift_summary_floats(trajectories: dict[str, list[State]]) -> dict[str, object]:
    """The canonical drift summary as plain floats / bools (the source data).

    Per-year peak ``leaf_c`` for both scenarios + year-end ``consumer_carbon`` for the
    consumer, plus the period class. ``year_summaries`` segments the trajectory exactly
    as the decade probe does, so these are the same numbers ``test_decade_stability.py``
    asserts on — this file just freezes them.
    """
    year = len(_weather())
    p_leaf = year_summaries(trajectories["perennial"], year, _peak_leaf)
    c_leaf = year_summaries(trajectories["consumer"], year, _peak_leaf)
    c_biomass = year_summaries(trajectories["consumer"], year, _year_end_consumer)
    return {
        "horizon_years": LONG_HORIZON_YEARS,
        "perennial": {
            "peak_leaf": p_leaf,
            "is_period_2": is_period_2(p_leaf, transient=_PERIOD_TRANSIENT),
        },
        "consumer": {
            "peak_leaf": c_leaf,
            "consumer_carbon": c_biomass,
            "is_period_2": is_period_2(c_leaf, transient=_PERIOD_TRANSIENT),
        },
    }


def _to_hex(value: object) -> object:
    """Recursively convert floats to hex-float strings — the same lossless, bit-exact,
    cross-port discipline ``sim_io`` applies to stock amounts (``float.hex``)."""
    if isinstance(value, float):
        return value.hex()
    if isinstance(value, list):
        return [_to_hex(v) for v in value]
    if isinstance(value, dict):
        return {k: _to_hex(v) for k, v in value.items()}
    return value


def _drift_summary_dumps(trajectories: dict[str, list[State]]) -> str:
    """Serialize the drift summary to canonical JSON (hex-float, byte-stable).

    Matches ``sim_io.dumps``' format exactly (``indent=2, sort_keys=True`` + trailing
    newline) so the byte-compare discipline is identical across the project's goldens.
    """
    data = _to_hex(_drift_summary_floats(trajectories))
    return json.dumps(data, indent=2, sort_keys=True) + "\n"


# --- the long-horizon State snapshots ----------------------------------------


@windows_golden_only
def test_perennial_long_horizon_golden_bytes_match(trajectories) -> None:
    # Byte-exact: any bit change in the 15-yr perennial output fails here (within-build;
    # see the transcendental caveat in the module doc).
    expected = sim_io.dumps(trajectories["perennial"][-1]).encode("utf-8")
    assert expected == PERENNIAL_GOLDEN.read_bytes()


@windows_golden_only
def test_perennial_long_horizon_golden_loads_back(trajectories) -> None:
    # The committed golden round-trips back to the exact final State (it routes through
    # the core constructors, so a tampered golden fails to load).
    text = PERENNIAL_GOLDEN.read_text(encoding="utf-8")
    assert sim_io.loads(text) == trajectories["perennial"][-1]


@windows_golden_only
def test_consumer_long_horizon_golden_bytes_match(trajectories) -> None:
    expected = sim_io.dumps(trajectories["consumer"][-1]).encode("utf-8")
    assert expected == CONSUMER_GOLDEN.read_bytes()


@windows_golden_only
def test_consumer_long_horizon_golden_loads_back(trajectories) -> None:
    text = CONSUMER_GOLDEN.read_text(encoding="utf-8")
    assert sim_io.loads(text) == trajectories["consumer"][-1]


# --- the drift-summary golden ------------------------------------------------


@windows_golden_only
def test_drift_summary_golden_bytes_match(trajectories) -> None:
    # Byte-exact: a regression in the limit-cycle *shape* (per-year peak_leaf /
    # consumer_carbon) or the period class fails here — the stability regression
    # catcher a single final-state snapshot cannot provide.
    assert (
        _drift_summary_dumps(trajectories).encode("utf-8")
        == DRIFT_SUMMARY_GOLDEN.read_bytes()
    )


@windows_golden_only
def test_drift_summary_golden_loads_back(trajectories) -> None:
    # The committed golden's hex vectors decode (float.fromhex) to exactly the freshly
    # computed summary floats, and the period booleans match — a tampered golden fails.
    # Recomputes the vectors directly (not via the object-typed summary dict) so the
    # decode is checked against the live engine output.
    parsed = json.loads(DRIFT_SUMMARY_GOLDEN.read_text(encoding="utf-8"))
    year = len(_weather())
    p_leaf = year_summaries(trajectories["perennial"], year, _peak_leaf)
    c_leaf = year_summaries(trajectories["consumer"], year, _peak_leaf)
    c_bio = year_summaries(trajectories["consumer"], year, _year_end_consumer)
    assert parsed["horizon_years"] == LONG_HORIZON_YEARS
    assert [float.fromhex(h) for h in parsed["perennial"]["peak_leaf"]] == p_leaf
    assert [float.fromhex(h) for h in parsed["consumer"]["peak_leaf"]] == c_leaf
    assert [float.fromhex(h) for h in parsed["consumer"]["consumer_carbon"]] == c_bio
    assert parsed["perennial"]["is_period_2"] == is_period_2(
        p_leaf, transient=_PERIOD_TRANSIENT
    )
    assert parsed["consumer"]["is_period_2"] == is_period_2(
        c_leaf, transient=_PERIOD_TRANSIENT
    )


def test_drift_summary_period_class_is_pinned() -> None:
    # ⚠ Re-pinned by post-roadmap scope (B) increment 1. Both chambers are now period-1
    # fixed points: the perennial's old period-2 cycle was an artifact of the broken
    # canopy regime, and closing the canopy (vernalization + photoperiod) flattened the
    # year-to-year return map below unit gain — see
    # test_biosphere_stress.py::test_stress_perennial_fixed_point_sustained and
    # docs/plans/post-roadmap-oracle-match.md. The consumer was always period-1. They
    # are no longer DISTINCT in period class; the golden encodes both as False.
    parsed = json.loads(DRIFT_SUMMARY_GOLDEN.read_text(encoding="utf-8"))
    assert parsed["perennial"]["is_period_2"] is False
    assert parsed["consumer"]["is_period_2"] is False


def _regenerate() -> None:
    """Rewrite all three committed long-horizon goldens from the current engine output.

    A deliberately separate, explicit action — NOT reachable from a test run. Run via::

        uv run python tests/test_regression_long_horizon.py

    Review the diff before committing: a change means the decade-scale output moved.
    """
    trajectories = _trajectories()
    PERENNIAL_GOLDEN.write_bytes(
        sim_io.dumps(trajectories["perennial"][-1]).encode("utf-8")
    )
    CONSUMER_GOLDEN.write_bytes(
        sim_io.dumps(trajectories["consumer"][-1]).encode("utf-8")
    )
    DRIFT_SUMMARY_GOLDEN.write_bytes(_drift_summary_dumps(trajectories).encode("utf-8"))
    for path in (PERENNIAL_GOLDEN, CONSUMER_GOLDEN, DRIFT_SUMMARY_GOLDEN):
        print(f"wrote {path}")


if __name__ == "__main__":
    _regenerate()
