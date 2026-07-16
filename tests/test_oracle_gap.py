"""Bucket 3: the oracle gap, DIAGNOSED and pinned as numbers with their causes.

The plan of record is ``docs/plans/post-roadmap-validation.md``. This is the "honest
first increment" (scope A): it **moves no golden and unfreezes nothing** — it measures
the committed season against the committed oracle fixture and pins *what is wrong,
by how much, and why*, so the gap is documented rather than merely known.

**What this file is for.** ``test_oracle_smoke.py`` records the *magnitude* gap (peak
LAI ~0.09 vs ~6.3), attributing it to "uncalibrated placeholders + the phenology
overrun". That attribution is **incomplete**, and magnitude is not the most diagnostic
signal. The
measurement behind this file (recorded in the plan doc) found the dominant cause is
structural and invisible to the suite: **our canopy peaks on day 32 of ~305 and
collapses before anthesis**, which ``test_oracle_smoke``'s ``_is_unimodal`` cannot see
— it asks only for an interior peak with both ends below half-peak, and a day-32 peak
satisfies that.

**These tests PIN KNOWN-WRONG BEHAVIOR.** This is the ``lab.fit_order`` / BVAD
``test_rq_structural_prediction`` idiom the project already uses: *measure* the known
structural error as a number rather than assert an aspiration. A green run here means
"the model is still wrong in exactly the documented way", **not** "the model is right".
Anyone who fixes a gap below will see this file go red — that is the design. **Update
the number and the docs; do not delete the test** (``CLAUDE.md``: never weaken a test
to make it pass).

The three columns, kept visibly separate (the ``test_bvad_validation.py`` precedent):

* **Shape** (``test_shape_*``) — what genuinely holds. Both trajectories complete the
  DVS 0 → 2 arc and form grain. This is the part the clean-room model gets right.
* **The pinned gap** (``test_gap_*``) — THE PAYLOAD. Three independent causes, each a
  number: the canopy never bootstraps; phenology runs ~1.6x fast; and (mildly)
  allocation is root-heavy. The first two are **missing science, not wrong numbers** —
  which is why the deferred "quantitative oracle match" was mis-sequenced as a
  calibration task.
* **Method** (``test_method_*``) — the confound guard. Organ fractions must be compared
  at matched **DVS**, never at matched calendar day; the matched-day comparison gives a
  qualitatively wrong answer (it reverses the sign of the storage finding). Pinned so
  the mistake cannot quietly return.

PCSE-free: both fixtures are read as JSON; ``lab.oracle_match`` is stdlib.
"""

import json
import math
from pathlib import Path

import pytest

from domains.biosphere.canopy import leaf_area_index
from domains.biosphere.loader import (
    load_canopy_params,
    load_phenology_params,
    load_senescence_params,
)
from domains.biosphere.phenology import development_stage
from domains.biosphere.season import (
    LEAF_C,
    ROOT_C,
    STEM_C,
    STORAGE_C,
    build_season,
    run_season,
    weather_resolver,
)
from simcore.integrator import EulerIntegrator
from simcore.state import State

_ORACLE_DIR = Path(__file__).parent / "oracle"
_REFERENCE = _ORACLE_DIR / "winter_wheat_reference.json"
_WEATHER = _ORACLE_DIR / "winter_wheat_weather.json"

_GROUND_AREA = 1.0  # m² — the Phase-1 winter-wheat PP plot (SeasonScenario default)


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


def _our_lai(states: list[State]) -> list[float]:
    cp = load_canopy_params()
    return [
        leaf_area_index(
            s.stocks[LEAF_C].amount,
            sla_per_mol_c=cp.sla_per_mol_c,
            ground_area=_GROUND_AREA,
        )
        for s in states
    ]


def _our_dvs(states: list[State], n: int) -> list[float]:
    # TSUM comes from phenology.yaml (the frozen params), not a test-local constant.
    pp = load_phenology_params()
    return [
        development_stage(
            states[i].aux["thermal_time"],
            tsum_anthesis=pp.tsum_anthesis,
            tsum_maturity=pp.tsum_maturity,
        )
        for i in range(n)
    ]


def _light_intercepted(lai: float) -> float:
    """Beer-Lambert fraction of incident light the canopy intercepts: 1 − e^(−k·LAI)."""
    return 1.0 - math.exp(-load_canopy_params().extinction_coef * lai)


def _first_day_at(series: list[float], threshold: float) -> int | None:
    for i, v in enumerate(series):
        if v >= threshold:
            return i
    return None


def _organ_fractions(state: State) -> dict[str, float]:
    organs = {
        "leaf": state.stocks[LEAF_C].amount,
        "stem": state.stocks[STEM_C].amount,
        "root": state.stocks[ROOT_C].amount,
        "storage": state.stocks[STORAGE_C].amount,
    }
    total = sum(organs.values())
    return {k: v / total for k, v in organs.items()}


def _oracle_fractions(row: dict[str, float]) -> dict[str, float]:
    organs = {
        "leaf": row["TWLV"],
        "stem": row["TWST"],
        "root": row["TWRT"],
        "storage": row["TWSO"],
    }
    total = sum(organs.values())
    return {k: v / total for k, v in organs.items()}


# =====================================================================================
# Column 1 — SHAPE: what the clean-room model genuinely gets right
# =====================================================================================


def test_shape_both_complete_the_development_arc() -> None:
    """Both reach maturity (DVS → 2). The phenology *arc* is right; only its rate is
    wrong (see ``test_gap_phenology_runs_fast``)."""
    ref = _reference()
    our = _our_dvs(_season_states(), len(ref))
    oracle = [r["DVS"] for r in ref]
    assert our[-1] >= 1.9
    assert max(oracle) >= 1.9


def test_shape_grain_forms_in_both() -> None:
    """Storage organs fill in both — the reproductive phase is structurally present."""
    assert _season_states()[-1].stocks[STORAGE_C].amount > 0.0
    assert max(r["TWSO"] for r in _reference()) > 0.0


# =====================================================================================
# Column 2 — THE PINNED GAP (the payload). Each is KNOWN-WRONG behavior, pinned as a
# number. A fix turns these RED on purpose: update the number, do not delete the test.
# =====================================================================================


def test_gap_canopy_never_bootstraps_light_interception() -> None:
    """CAUSE 1 (DOMINANT) — source-limited from day 1; the canopy never establishes.

    At the sown seedling's LAI (~0.029) Beer-Lambert interception is ~1.75 %, so gross
    assimilation is tiny, so the daily structural increment to leaf (``fl·DMI``, with
    ``fl = 0.55`` — the allocation table is NOT the constraint here) is smaller than the
    2 %/day leaf death rate. Leaf shrinks → intercepts less light → fixes less carbon.
    A death spiral: peak interception is 5 % on day 32, then collapse.

    The oracle reaches **97.8 %** interception at peak LAI. THIS IS THE HEADLINE NUMBER.

    Note the initial condition is NOT the problem: our LAI₀ (0.029) and the oracle's
    (0.034) agree. The growth dynamics are. Closing this needs a juvenile
    canopy-expansion phase (temperature-driven rather than assimilate-limited) — **new
    science, clean-room from primary literature**, deferred to bucket B.
    """
    our_lai = _our_lai(_season_states())
    oracle_lai = [r["LAI"] for r in _reference()]

    assert _light_intercepted(our_lai[0]) == pytest.approx(0.0175, abs=5e-4)
    assert _light_intercepted(max(our_lai)) == pytest.approx(0.050, abs=5e-3)
    assert _light_intercepted(max(oracle_lai)) == pytest.approx(0.978, abs=5e-3)

    # The gap, stated the way it matters: we harvest ~5 % of the light they do at peak.
    assert _light_intercepted(max(our_lai)) < 0.10
    assert _light_intercepted(max(oracle_lai)) > 0.95


def test_gap_canopy_peaks_absurdly_early_the_timing_teeth() -> None:
    """CAUSE 1, the signal ``test_oracle_smoke`` is BLIND to.

    Our LAI peaks on **day 32** of ~305 and collapses *before* anthesis (day 138); the
    oracle's peaks on **day 212**, at anthesis, which is what a wheat canopy does.
    ``_is_unimodal`` passes on both — it asks only for an interior peak with ends below
    half-peak — so the suite recorded a 74x magnitude gap while the more diagnostic
    *timing* failure underneath it went unremarked. These are the teeth.
    """
    our_lai = _our_lai(_season_states())
    oracle_lai = [r["LAI"] for r in _reference()]
    our_peak_day = our_lai.index(max(our_lai))
    oracle_peak_day = oracle_lai.index(max(oracle_lai))

    assert our_peak_day == 32
    assert oracle_peak_day == 212

    # The canopy peaks before anthesis and is gone by it — the collapse, as a number.
    our_dvs = _our_dvs(_season_states(), len(_reference()))
    anthesis_day = _first_day_at(our_dvs, 1.0)
    assert anthesis_day is not None
    assert our_peak_day < anthesis_day
    # At flowering the canopy is at ~19 % of its own (already tiny) peak.
    assert our_lai[anthesis_day] < 0.25 * max(our_lai)


def test_gap_phenology_runs_fast() -> None:
    """CAUSE 2 — INDEPENDENT of cause 1, and also missing science, not a wrong number.

    Winter wheat sown 1 Oct reaches anthesis in **mid-February** (day 138 vs the
    oracle's 217 — 79 days early); maturity 74 days early. ``development_stage``
    accumulates
    thermal time alone: there is **no vernalization term anywhere in src/**
    (``phenology.py`` documents it as a deferred second state accumulator with a derived
    ``VERNFAC ∈ [0,1]``), so our crop races through the winter the oracle sits dormant
    through.

    DVS is driven by thermal time, **independent of biomass** — so fixing the canopy
    does not fix this, and fixing this does not fix the canopy. Two problems.
    """
    ref = _reference()
    our = _our_dvs(_season_states(), len(ref))
    oracle = [r["DVS"] for r in ref]

    our_anthesis, oracle_anthesis = _first_day_at(our, 1.0), _first_day_at(oracle, 1.0)
    our_maturity, oracle_maturity = _first_day_at(our, 2.0), _first_day_at(oracle, 2.0)
    assert our_anthesis is not None and oracle_anthesis is not None
    assert our_maturity is not None and oracle_maturity is not None
    assert (our_anthesis, oracle_anthesis) == (138, 217)
    assert (our_maturity, oracle_maturity) == (218, 292)
    assert oracle_anthesis - our_anthesis == 79
    assert oracle_maturity - our_maturity == 74


def test_gap_allocation_is_root_heavy() -> None:
    """CAUSE 3 — real but MODEST, and the only one that is a genuine value error.

    Compared at matched DVS (never matched day — see ``test_method_*``), the partition
    shape is broadly right; the surviving signal is a root-heavy bias: root fraction
    0.39 vs the oracle's 0.26 at DVS 0.5, and 0.39 vs 0.15 at anthesis.

    This is a partition-table calibration item (bucket B), third in line behind two
    structural gaps. No amount of tuning it fixes a canopy that intercepts 1.75 % of
    light — which is exactly why the deferred oracle match is not a calibration task.
    """
    states = _season_states()
    ref = _reference()
    n = min(len(ref), len(states))
    our_dvs = _our_dvs(states, n)
    oracle_dvs = [r["DVS"] for r in ref[:n]]

    for target, our_root, oracle_root in ((0.5, 0.390, 0.256), (1.0, 0.385, 0.146)):
        oi, ri = _first_day_at(our_dvs, target), _first_day_at(oracle_dvs, target)
        assert oi is not None and ri is not None
        assert _organ_fractions(states[oi])["root"] == pytest.approx(our_root, abs=5e-3)
        assert _oracle_fractions(ref[ri])["root"] == pytest.approx(
            oracle_root, abs=5e-3
        )
        # The bias, as the comparison that matters: we hold ~1.5–2.6x their root share.
        assert _organ_fractions(states[oi])["root"] > _oracle_fractions(ref[ri])["root"]


# =====================================================================================
# Column 3 — METHOD: the confound guard. This one protects a *conclusion*, not the code.
# =====================================================================================


def test_method_matched_day_comparison_is_invalid() -> None:
    """Organ fractions MUST be compared at matched DVS, not matched calendar day.

    Because phenology runs ~1.6x fast (cause 2), at the final day our plant has sat at
    DVS 2 for ~87 days — senescing at 2 %/day, filling grain — while the oracle matured
    ~13 days earlier. A matched-*day* read therefore says "we OVER-allocate to storage,
    0.69 vs 0.52". At matched DVS the sign **REVERSES**: we *under*-allocate (0.40 vs
    0.52). The matched-day answer is an artifact of cause 2, not a partitioning finding.

    Pinned because the confound is easy to re-introduce and cheap to miss: it produced a
    plausible, wrong, and confidently-stated conclusion during this very diagnosis.
    """
    states = _season_states()
    ref = _reference()
    n = min(len(ref), len(states))

    # The INVALID comparison (matched final day) — reproduced here to pin that it lies.
    matched_day_ours = _organ_fractions(states[n - 1])["storage"]
    matched_day_oracle = _oracle_fractions(ref[n - 1])["storage"]
    assert matched_day_ours > matched_day_oracle  # says: we over-allocate. WRONG.

    # The VALID comparison (matched DVS = 2.0) — the sign reverses.
    our_dvs = _our_dvs(states, n)
    oracle_dvs = [r["DVS"] for r in ref[:n]]
    oi, ri = _first_day_at(our_dvs, 2.0), _first_day_at(oracle_dvs, 2.0)
    assert oi is not None and ri is not None
    matched_dvs_ours = _organ_fractions(states[oi])["storage"]
    matched_dvs_oracle = _oracle_fractions(ref[ri])["storage"]
    assert matched_dvs_ours < matched_dvs_oracle  # says: we under-allocate. CORRECT.

    # The numbers behind both reads, so the reversal is legible and not just asserted.
    assert matched_day_ours == pytest.approx(0.69, abs=0.02)
    assert matched_dvs_ours == pytest.approx(0.40, abs=0.02)
    assert matched_dvs_oracle == pytest.approx(0.52, abs=0.02)


def test_method_the_death_spiral_mechanism() -> None:
    """The mechanism behind cause 1, pinned so the *explanation* is verified, not just
    asserted in prose: at the peak, leaf growth can no longer outpace leaf death.

    ``rdr_leaf`` (2 %/day) applied to the standing leaf exceeds the assimilate the
    canopy can capture at ~5 % light interception, so the leaf turns over and declines.
    """
    states = _season_states()
    leaf = [s.stocks[LEAF_C].amount for s in states]
    peak_day = leaf.index(max(leaf))
    rdr_leaf = load_senescence_params().rdr_leaf

    assert rdr_leaf == pytest.approx(0.02, abs=1e-9)
    # Past the peak the canopy is in net decline: growth < death, every step.
    assert leaf[peak_day + 1] < leaf[peak_day]
    assert leaf[peak_day + 30] < leaf[peak_day]
    # And it declines to a small fraction of an already-tiny peak by anthesis.
    assert leaf[-1] < 0.2 * leaf[peak_day]
