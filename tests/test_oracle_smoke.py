"""Phase-1 Step-11 oracle smoke check (5) — QUALITATIVE, behavioural, deferred-gate.

The clean-room crop model is reimplemented from primary literature with independently-
sourced ``TODO(cite)`` placeholder params, so it **cannot** reproduce the WOFOST/PCSE
oracle bit-for-bit (P5). Per the Step-11 decision (the tight **quantitative** gate is
**deferred**), this is a *qualitative* smoke check against the committed reference
trajectory under the **same** NASAPower weather: right *shape*, recorded *gap*.

What it asserts (shape — the model behaves like an annual crop, as the oracle does):
  * development ``DVS`` is monotone non-decreasing and completes the ``0 → 2`` cycle
    for **both** our season and the oracle; their aligned ``DVS`` series agree within a
    loose band (``lab.oracle_match.nrmse``; loose because our placeholder phenology
    runs faster — no vernalization, the documented overrun).
  * ``LAI`` is **unimodal** (emerges low, peaks mid-season, senesces low) for both.
  * grain forms (our ``storage_c`` / the oracle ``TWSO`` both end positive).

What it RECORDS as a documented finding (NOT a pass it pretends to achieve): the
**magnitude gap** — the oracle's peak LAI (~6) is ~2 orders of magnitude above ours
(~0.09), driven by the uncalibrated placeholders + the phenology overrun. The test pins
that the gap is *large and in the known direction* so a future reader cannot mistake the
committed season for a validated match; closing it (literature-range calibration +
vernalization) is the deferred quantitative gate.

PCSE-free: both committed fixtures are read as JSON; ``lab.oracle_match`` is stdlib.
"""

import json
from pathlib import Path

from domains.biosphere.canopy import leaf_area_index
from domains.biosphere.loader import load_canopy_params
from domains.biosphere.phenology import development_stage
from domains.biosphere.season import (
    LEAF_C,
    STORAGE_C,
    build_season,
    run_season,
    weather_resolver,
)
from lab.oracle_match import nrmse
from simcore.integrator import EulerIntegrator
from simcore.state import State

_ORACLE_DIR = Path(__file__).parent / "oracle"
_REFERENCE = _ORACLE_DIR / "winter_wheat_reference.json"
_WEATHER = _ORACLE_DIR / "winter_wheat_weather.json"
_TSUM_ANTHESIS, _TSUM_MATURITY = 1100.0, 750.0


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER.read_text(encoding="utf-8"))["weather"]


def _reference() -> list[dict[str, float]]:
    return json.loads(_REFERENCE.read_text(encoding="utf-8"))["trajectory"]


def _season_states() -> list[State]:
    state, registry = build_season()
    resolver = weather_resolver(_weather())
    states, _, _ = run_season(
        EulerIntegrator(registry), state, resolver, 1.0, len(_weather())
    )
    return states


def _our_dvs(states: list[State], n: int) -> list[float]:
    return [
        development_stage(
            states[i].aux["thermal_time"],
            tsum_anthesis=_TSUM_ANTHESIS,
            tsum_maturity=_TSUM_MATURITY,
        )
        for i in range(n)
    ]


def _our_lai(states: list[State]) -> list[float]:
    cp = load_canopy_params()
    return [
        leaf_area_index(
            s.stocks[LEAF_C].amount, sla_per_mol_c=cp.sla_per_mol_c, ground_area=1.0
        )
        for s in states
    ]


def _is_monotone_nondecreasing(series: list[float]) -> bool:
    return all(b >= a - 1e-9 for a, b in zip(series, series[1:], strict=False))


def _is_unimodal(series: list[float]) -> bool:
    """Emerges low, peaks in the interior, senesces low (a single hump)."""
    peak = max(series)
    peak_idx = series.index(peak)
    return (
        0 < peak_idx < len(series) - 1  # interior peak
        and series[0] < 0.5 * peak  # starts well below the peak
        and series[-1] < 0.5 * peak  # ends well below the peak (senescence)
    )


# --- shape: both develop 0 → 2 monotonically; aligned within a loose band ---
def test_development_completes_for_both() -> None:
    ref = _reference()
    our = _our_dvs(_season_states(), len(ref))
    oracle = [r["DVS"] for r in ref]
    assert _is_monotone_nondecreasing(our)
    assert _is_monotone_nondecreasing(oracle)
    assert our[-1] >= 1.9 and max(oracle) >= 1.9  # both reach maturity (DVS → 2)


def test_development_within_loose_band() -> None:
    # Loose: our placeholder phenology runs faster (anthesis earlier, no vernalization),
    # so the aligned DVS series differ in *timing* but follow the same 0 → 2 arc — a
    # wide, documented qualitative band, not the deferred tight gate.
    ref = _reference()
    our = _our_dvs(_season_states(), len(ref))
    oracle = [r["DVS"] for r in ref]
    assert nrmse(oracle, our) < 0.5


# --- shape: LAI is a single mid-season hump for both ------------------------
def test_lai_is_unimodal_for_both() -> None:
    our_lai = _our_lai(_season_states())
    oracle_lai = [r["LAI"] for r in _reference()]
    assert _is_unimodal(our_lai)
    assert _is_unimodal(oracle_lai)


# --- shape: grain forms in both ---------------------------------------------
def test_grain_forms_in_both() -> None:
    states = _season_states()
    assert states[-1].stocks[STORAGE_C].amount > 0.0
    assert max(r["TWSO"] for r in _reference()) > 0.0


# --- recorded finding: the magnitude gap (the deferred quantitative gate) ----
def test_magnitude_gap_is_large_and_documented() -> None:
    # NOT a match we pretend to achieve: the oracle's peak LAI is ~2 orders of magnitude
    # above ours (uncalibrated placeholders + phenology overrun). Pinning the gap's
    # direction + scale stops a reader mistaking the committed season for validated.
    our_peak = max(_our_lai(_season_states()))
    oracle_peak = max(r["LAI"] for r in _reference())
    assert our_peak > 0.0  # the canopy did form (liveness)
    assert oracle_peak > 10.0 * our_peak  # the known, deferred-calibration gap
