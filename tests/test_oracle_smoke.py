"""Phase-1 Step-11 oracle smoke check (5) — QUALITATIVE, behavioural, deferred-gate.

The clean-room crop model is reimplemented from primary literature with independently-
sourced ``TODO(cite)`` placeholder params, so it **cannot** reproduce the WOFOST/PCSE
oracle bit-for-bit (P5). Per the Step-11 decision (the tight **quantitative** gate is
**deferred**), this is a *qualitative* smoke check against the committed reference
trajectory under the **same** NASAPower weather: right *shape*, recorded *gap*.

What it asserts (shape — the model behaves like an annual crop, as the oracle does):
  * development ``DVS`` is monotone non-decreasing and completes the ``0 → 2`` cycle
    for **both** our season and the oracle; their aligned ``DVS`` series agree within a
    loose band (``lab.oracle_match.nrmse``; nrmse ~0.09 since increment 1 — kept loose,
    not tightened, as the tight gate is the deferred recalibration's business).
  * ``LAI`` forms a mid-season hump for both — the oracle fully unimodal, ours
    bootstrapping from a low emergence but not yet fully senescing by season end (the
    short-reproductive-phase residual; see the re-pinned test below).
  * grain forms (our ``storage_c`` / the oracle ``TWSO`` both end positive).

⚠ **Updated by post-roadmap scope (B) increment 1 (2026-07-20).** This file used to
record a **magnitude gap of ~2 orders** (oracle peak LAI ~6 vs ours ~0.09). Increment 1
added vernalization + photoperiod (clean-room from Soltani & Sinclair 2012), and the gap
**closed to ~1.22x** — the canopy now bootstraps (see ``test_oracle_gap.py``, the
quantitative pin, and ``docs/plans/post-roadmap-oracle-match.md``). The two tests that
recorded the *old* magnitude gap and the *old* full-senescence LAI shape are re-pinned
below to the new reality: the gap is small, and our LAI hump does not fully senesce by
season end because the residual reproductive phase is too short (~43 d vs ~75 d — a
``tsum`` calibration item, the deferred recalibration increment's target).

PCSE-free: both committed fixtures are read as JSON; ``lab.oracle_match`` is stdlib.
"""

import json
from pathlib import Path

import pytest

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
    # Since increment 1 (vernalization + photoperiod) the aligned DVS series are much
    # closer — nrmse ~0.09, well inside this loose qualitative band (it was the
    # phenology overrun that used to make it loose; maturity now lands within ~2 days).
    # Kept as a loose band, not tightened: the tight quantitative gate is the deferred
    # recalibration increment's business, and this file stays qualitative.
    ref = _reference()
    our = _our_dvs(_season_states(), len(ref))
    oracle = [r["DVS"] for r in ref]
    assert nrmse(oracle, our) < 0.5


# --- shape: LAI is a single mid-season hump for both ------------------------
def test_lai_hump_forms_oracle_fully_unimodal_ours_incompletely_senesced() -> None:
    # ⚠ Re-pinned by increment 1. The oracle LAI is fully unimodal (emerges low, peaks,
    # senesces low). Ours now forms a real interior hump too (it bootstraps — was a
    # day-32 collapse in scope A), BUT does not end below half-peak: the residual
    # ~43-day reproductive phase (vs the oracle's ~75) leaves the canopy incompletely
    # senesced at season end (~62 % of peak). That incomplete senescence is the
    # tsum-partition residual surfacing in LAI shape; a recalibration that lengthens
    # grain fill turns this red.
    our_lai = _our_lai(_season_states())
    oracle_lai = [r["LAI"] for r in _reference()]
    assert _is_unimodal(oracle_lai)  # the oracle: a clean single hump
    # Ours: a genuine interior peak, rising from a low emergence (the bootstrap).
    our_peak_idx = our_lai.index(max(our_lai))
    assert 0 < our_peak_idx < len(our_lai) - 1
    assert our_lai[0] < 0.5 * max(our_lai)
    # ...but NOT fully senesced by season end — the pinned residual.
    assert our_lai[-1] > 0.5 * max(our_lai)


# --- shape: grain forms in both ---------------------------------------------
def test_grain_forms_in_both() -> None:
    states = _season_states()
    assert states[-1].stocks[STORAGE_C].amount > 0.0
    assert max(r["TWSO"] for r in _reference()) > 0.0


# --- recorded finding: the magnitude gap CLOSED (increment 1) ----------------
def test_magnitude_gap_closed_to_a_small_residual() -> None:
    # ⚠ Re-pinned by increment 1. This asserted `oracle_peak > 10.0 * our_peak` — a ~2
    # order-of-magnitude gap — until 2026-07-20. Vernalization + photoperiod closed it:
    # the oracle's peak LAI is now only ~1.22x ours (6.34 vs 5.19), with NO canopy
    # science written (the gap was downstream of the phenology error). Pinned as a small
    # residual in the known direction; a recalibration that closes it further turns this
    # red.
    our_peak = max(_our_lai(_season_states()))
    oracle_peak = max(r["LAI"] for r in _reference())
    assert our_peak > 4.0  # the canopy now genuinely closes (was ~0.09 in scope A)
    ratio = oracle_peak / our_peak
    assert ratio == pytest.approx(1.22, abs=0.15)  # small residual, oracle still higher
    assert ratio < 2.0
