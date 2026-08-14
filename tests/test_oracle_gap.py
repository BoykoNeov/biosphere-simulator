"""Bucket 3: the oracle gap — RE-PINNED after scope (B) increment 1 (vern+photoperiod).

The plan of record is ``docs/plans/post-roadmap-oracle-match.md`` (increment 1);
``post-roadmap-validation.md`` is scope (A), the diagnosis this file originally pinned.

**This file went red on purpose and has been re-pinned, not deleted** (``CLAUDE.md``:
never weaken a test to make it pass; the ``lab.fit_order`` / BVAD idiom — pin the
*current* known-wrong behavior as a number, so a future fix turns it red again). Scope
(A) pinned a model whose canopy never bootstrapped and whose phenology ran ~1.6x fast.
Increment 1 added the two missing sciences (vernalization; photoperiod, both clean-room
from Soltani & Sinclair 2012), and **most of what this file used to pin is now FIXED or
FALSIFIED**:

* the canopy now closes — **95.6 % light interception at peak, up from 5.0 %** (the
  dominant, "structural", scope-(A) failure is essentially gone, with **no canopy
  science written** — it was downstream of the phenology error; see the plan's finding
  2);
* phenology no longer runs fast — anthesis is now ~34 days *late*, and **maturity lands
  within 2 days of the oracle** (day 294 vs 292);
* the "two independent problems" premise is **falsified** — fixing phenology fixed the
  canopy, because ``Allocation`` reads DVS (a one-directional coupling scope (A) called
  symmetric);
* the matched-day storage confound **dissolved** — with timing near-correct, both the
  matched-day and matched-DVS reads now agree (see ``test_method_*``).
  ⚠ **2026-08-12: it came BACK, from an unrelated cause** — the stem reserve keeps
  feeding grain past DVS 2.0, so matched-day now reads "we over-allocate" while
  matched-DVS still reads "we under-allocate". The method verdict is unchanged and
  strengthened (matched-day is invalid; it was only ever incidentally harmless), and it
  raises an undecided design question: should remobilization stop at maturity? Both are
  recorded in ``test_method_matched_day_confound_dissolved_with_the_phenology_fix``.

The residual is the ``tsum`` phase partition: our reproductive phase is ~43 days vs the
oracle's 75. **Scope-B ceremony 2 (2026-07-20) investigated it and moved NO value** —
the outcome is a finding, not a recalibration:

* Both tsum values are **already literature-centred** (Penning de Vries 1989, Tables
  12 & 15, first-hand): ``tsum_maturity = 750`` is dead-centre of the winter-wheat range
  [727, 784] °C·day; ``tsum_anthesis = 1100`` sits inside [1026, 1333]. The oracle's
  implied TSUM2 (~1207) is ~1.5× above these — a longer-grain-fill *cultivar*, not a
  calibration error. Matching it would leave the cited range = backfitting, forbidden by
  ruling B (the oracle is a diagnostic, never a fit target).
* The partition is **calendar-bounded anyway**: at our (ratified) day-251 anthesis only
  ~54 fixture-days remain, so ``tsum_maturity > ~912`` never even matures.
* The maturity "match" below (day 294 vs 292) is **two errors cancelling**, not
  validation — see ``test_fixed_maturity_lands_within_two_days``.

So these tests still pin the *same numbers* (nothing moved); only the story changed —
from "the target of a deferred recalibration" to "a recorded cultivar-variation +
double-modulation finding". Plan of record: ``docs/plans/post-roadmap-oracle-match.md``
("Ceremony 2").

PCSE-free: both fixtures are read as JSON; ``lab.oracle_match`` is stdlib.
"""

import dataclasses
import json
import math
from pathlib import Path

import pytest

from domains.biosphere.canopy import leaf_area_index
from domains.biosphere.loader import (
    load_canopy_params,
    load_phenology_params,
)
from domains.biosphere.phenology import development_stage
from domains.biosphere.scenario import DEFAULT_SCENARIO
from domains.biosphere.season import (
    LEAF_C,
    ROOT_C,
    STEM_C,
    STEM_RESERVE_C,
    STORAGE_C,
    build_season,
    run_season,
    weather_resolver,
)
from domains.biosphere.step import BIO_DT, day_of, steps_for
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
        EulerIntegrator(registry), state, resolver, BIO_DT, steps_for(len(_weather()))
    )
    return states


def _days_in(states: list[State]) -> int:
    """How many physical days the trajectory covers — NOT ``len(states)``."""
    return day_of(len(states) - 1) + 1


def _by_day(states: list[State], days: int) -> list[State]:
    """The state at the START of each of the first ``days`` physical days.

    ⚠⚠ **THE UNIT SEAM OF THIS FILE.** The oracle reference is one row per
    **day**; the trajectory is one entry per **step**. Those were the same list index
    while the step was a day, and stopped being it on 2026-08-14. The step unfreeze
    converted every run's *length* here (``run_season`` is correctly given
    ``steps_for(...)``) and left every index that *reads the trajectory back* —
    including, invisibly, ``n = min(len(ref), len(states))``, which was
    ``min(305, 306) = 305`` and right, and became ``min(305, 1221) = 305`` and wrong
    **without changing value**. *A ``min`` over two quantities in different units cannot
    announce itself.*

    ``steps_for(d)`` is the documented inverse of ``day_of``
    (``domains.biosphere.step``) and is the identity at ``dt = 1``, so every use of this
    helper is a no-op on the old step — which is what a correct conversion looks like.
    """
    return [states[steps_for(d)] for d in range(days)]


def _our_lai(states: list[State], n: int) -> list[float]:
    # ⚠ ``n`` is a DAY count, and was not a parameter before 2026-08-14: this walked
    # the whole trajectory, so at ``dt = ¼`` its argmax was a STEP index being
    # compared against the oracle's day 212.
    cp = load_canopy_params()
    return [
        leaf_area_index(
            s.stocks[LEAF_C].amount,
            sla_per_mol_c=cp.sla_per_mol_c,
            ground_area=_GROUND_AREA,
        )
        for s in _by_day(states, n)
    ]


def _our_dvs(states: list[State], n: int) -> list[float]:
    # TSUM comes from phenology.yaml (the frozen params), not a test-local constant.
    pp = load_phenology_params()
    return [
        development_stage(
            s.aux["thermal_time"],
            tsum_anthesis=pp.tsum_anthesis,
            tsum_maturity=pp.tsum_maturity,
        )
        for s in _by_day(states, n)
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
    """Organ shares of total plant carbon, on **WOFOST's organ basis**.

    ⚠ ``stem`` INCLUDES ``stem_reserve_c`` (2026-08-12, the stem-reserve build) for the
    same comparability reason as the spring-wheat module: WOFOST has no separate reserve
    pool, so the carbohydrate this model holds apart is inside the oracle's own
    ``TWST``.
    Leaving it out would shrink OUR denominator only and inflate every fraction on our
    side of a matched-DVS comparison — a pool the oracle does not have, read as a
    partition difference.
    """
    organs = {
        "leaf": state.stocks[LEAF_C].amount,
        "stem": state.stocks[STEM_C].amount + state.stocks[STEM_RESERVE_C].amount,
        "root": state.stocks[ROOT_C].amount,
        "storage": state.stocks[STORAGE_C].amount,
    }
    total = sum(organs.values())
    return {k: v / total for k, v in organs.items()}


def _season_states_without_reserve() -> list[State]:
    """The pre-stem-reserve tree, for attributing the 2026-08-12 re-pins."""
    scenario = dataclasses.replace(DEFAULT_SCENARIO, stem_reserves=False)
    state, registry = build_season(scenario)
    resolver = weather_resolver(_weather(), scenario)
    states, _, _ = run_season(
        EulerIntegrator(registry), state, resolver, BIO_DT, steps_for(len(_weather()))
    )
    return states


def _organ_fractions_bare(state: State) -> dict[str, float]:
    """``_organ_fractions`` for a tree with NO reserve stock (the control only)."""
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
# Column 1 — SHAPE + WHAT INCREMENT 1 FIXED: what the model now gets right
# =====================================================================================


def test_shape_both_complete_the_development_arc() -> None:
    """Both reach maturity (DVS → 2), and now on nearly the same schedule."""
    ref = _reference()
    our = _our_dvs(_season_states(), len(ref))
    oracle = [r["DVS"] for r in ref]
    assert our[-1] >= 1.9
    assert max(oracle) >= 1.9


def test_shape_grain_forms_in_both() -> None:
    """Storage organs fill in both — the reproductive phase is structurally present."""
    assert _season_states()[-1].stocks[STORAGE_C].amount > 0.0
    assert max(r["TWSO"] for r in _reference()) > 0.0


def test_fixed_the_canopy_now_bootstraps() -> None:
    """SCOPE (A)'s DOMINANT failure is GONE — and no canopy science was written.

    Scope (A) pinned a canopy that peaked at **5.0 %** light interception on day 32 and
    collapsed (``rdr_leaf`` outpacing a starved seedling's assimilate). With
    vernalization + photoperiod slowing development, the plant stays in the high-``fl``
    vegetative phase long enough for the canopy to close: **peak interception is now
    ~95.6 %** against the oracle's 97.8 % — a residual of ~1.22x in peak LAI, down from
    ~43x.

    This is the plan's finding 2, pinned: the "structural" canopy gap was a *downstream
    consequence* of the phenology error (``Allocation`` reads DVS), not an independent
    missing science. A fix that re-broke phenology would re-break this — turning it red.
    """
    our_lai = _our_lai(_season_states(), len(_reference()))
    oracle_lai = [r["LAI"] for r in _reference()]

    # Sowing interception is unchanged (same LAI₀) — the fix is in the growth dynamics.
    assert _light_intercepted(our_lai[0]) == pytest.approx(0.0175, abs=5e-4)
    # Peak interception now CLOSES the canopy (was < 0.10 in scope A).
    assert _light_intercepted(max(our_lai)) == pytest.approx(0.956, abs=1e-2)
    assert _light_intercepted(max(our_lai)) > 0.90
    assert _light_intercepted(max(oracle_lai)) == pytest.approx(0.978, abs=5e-3)
    # The residual peak-LAI ratio, pinned as a number (was ~43x).
    assert max(oracle_lai) / max(our_lai) == pytest.approx(1.22, abs=0.1)


def test_maturity_lands_within_two_days_but_it_is_two_errors_cancelling() -> None:
    """Maturity day 294 vs the oracle's 292 — but this is NOT validation (ceremony 2).

    It reads like a near-exact match, and that is the trap. It is **two errors
    cancelling**: our anthesis is ~34 days LATE (day 251 vs 217, the ratified
    vern+photoperiod overshoot) and our reproductive phase is ~32 days SHORT (43 d vs
    75), so the maturity *date* nets to ~0 error while the *partition* underneath is
    wrong. The assertion passes and stays pinned; only the story is corrected, so the
    coincidence is not mistaken for a validated endpoint (ungated-prose-half pattern,
    caught in the assertion's own narrative). See
    ``test_gap_phase_partition_is_wrong_vegetative_too_long`` for the partition itself.
    """
    ref = _reference()
    our = _our_dvs(_season_states(), len(ref))
    oracle = [r["DVS"] for r in ref]
    our_mat, oracle_mat = _first_day_at(our, 2.0), _first_day_at(oracle, 2.0)
    assert our_mat is not None and oracle_mat is not None
    assert oracle_mat == 292
    # ±3 days: our side is a thermal-time threshold crossing (libm-sensitive off the
    # Windows/UCRT generation platform — the tests/golden_platform.py trap).
    assert our_mat == pytest.approx(294, abs=3)
    assert abs(our_mat - oracle_mat) <= 5


# =====================================================================================
# Column 2 — THE PINNED GAP (the payload). The RESIDUAL is now cause 3 (calibration):
# the tsum phase partition, an LAI-peak timing slip, and a root-heavy bias. Each is a
# number. A recalibration (scope-B ceremony 2) turns these red — update, do not delete.
# =====================================================================================


def test_gap_phase_partition_is_wrong_vegetative_too_long() -> None:
    """THE PAYLOAD — the residual after both sciences: the tsum partition is off.

    Maturity date is right (~day 293) but the *split* is not: our vegetative phase
    (emergence → anthesis) is **251 days vs the oracle's 217**, and our reproductive
    phase (anthesis → maturity) is only **43 days vs 75**.

    **Ceremony 2 (2026-07-20) resolved this to a finding, not a recalibration** (both
    values are now CITED, no longer ``TODO(cite)``): ``tsum_maturity = 750`` is
    dead-centre of the first-hand winter-wheat range [727, 784] °C·day (Penning de
    Vries 1989), while the oracle's implied ~1207 is a longer-grain-fill *cultivar*.
    The gap is cultivar variation, not our error: closing it would leave the cited
    range (backfitting, forbidden by ruling B) and is calendar-impossible at our
    anthesis anyway. This test therefore pins a *permanent, explained* residual, not a
    to-do.
    """
    ref = _reference()
    our = _our_dvs(_season_states(), len(ref))
    oracle = [r["DVS"] for r in ref]
    our_ant, oracle_ant = _first_day_at(our, 1.0), _first_day_at(oracle, 1.0)
    our_mat, oracle_mat = _first_day_at(our, 2.0), _first_day_at(oracle, 2.0)
    assert our_ant is not None and oracle_ant is not None
    assert our_mat is not None and oracle_mat is not None

    # Oracle side: exact (committed JSON, cross-platform stable).
    assert (oracle_ant, oracle_mat) == (217, 292)
    assert oracle_mat - oracle_ant == 75  # oracle reproductive phase
    # Our side: ±3 days (thermal-time threshold crossings, libm-sensitive).
    assert our_ant == pytest.approx(251, abs=3)
    our_repro = our_mat - our_ant
    assert our_repro == pytest.approx(43, abs=4)
    # The finding: our reproductive phase is ~half the oracle's.
    assert our_repro < 0.7 * (oracle_mat - oracle_ant)


def test_gap_lai_peaks_slightly_after_anthesis() -> None:
    """A residual timing slip — the mirror image of scope (A)'s day-32 collapse.

    A wheat canopy should peak at or just before anthesis and decline through grain
    fill. Ours peaks on **day ~263, ~12 days AFTER our anthesis (~day 251)** — a
    senescence/allocation timing residual, not a failure to bootstrap. The oracle peaks
    on day 212, at its anthesis. Modest next to the partition error, but recorded so the
    recalibration has to account for it.
    """
    states = _season_states()
    our_lai = _our_lai(states, len(_reference()))
    oracle_lai = [r["LAI"] for r in _reference()]
    our_peak_day = our_lai.index(max(our_lai))
    oracle_peak_day = oracle_lai.index(max(oracle_lai))
    our_dvs = _our_dvs(states, len(_reference()))
    anthesis_day = _first_day_at(our_dvs, 1.0)
    assert anthesis_day is not None

    assert oracle_peak_day == 212  # oracle: exact (committed JSON)
    # ±8 days: argmax over a broad flat hump — libm-sensitive winner (golden_platform
    # trap). The finding is "just after anthesis", a ~12-day offset, not the exact day.
    assert our_peak_day == pytest.approx(263, abs=8)
    assert our_peak_day > anthesis_day  # the slip: peak is AFTER anthesis, not before


def test_gap_allocation_is_root_heavy() -> None:
    """CAUSE 3, unchanged in kind — a modest root-heavy bias at matched DVS.

    Compared at matched DVS (never matched day — see ``test_method_*``), root fraction
    is 0.30 vs the oracle's 0.26 at DVS 0.5, and 0.24 vs 0.15 at anthesis. A
    partition-table calibration item, milder than in scope (A) (the collapsing canopy
    had exaggerated it).
    """
    states = _season_states()
    ref = _reference()
    n = min(len(ref), _days_in(states))  # was `len(states)` — a STEP count
    days = _by_day(states, n)
    our_dvs = _our_dvs(states, n)
    oracle_dvs = [r["DVS"] for r in ref[:n]]

    for target, our_root, oracle_root in ((0.5, 0.302, 0.256), (1.0, 0.243, 0.146)):
        oi, ri = _first_day_at(our_dvs, target), _first_day_at(oracle_dvs, target)
        assert oi is not None and ri is not None
        # Our-side band 1.5e-2: the sample index is a DVS threshold crossing (libm).
        assert _organ_fractions(days[oi])["root"] == pytest.approx(our_root, abs=1.5e-2)
        assert _oracle_fractions(ref[ri])["root"] == pytest.approx(
            oracle_root, abs=1e-3
        )
        # The bias, as the comparison that matters: we still hold more root share.
        assert _organ_fractions(days[oi])["root"] > _oracle_fractions(ref[ri])["root"]


# =====================================================================================
# Column 3 — METHOD: the confound guard. The matched-day confound DISSOLVED with the
# phenology fix — pinned, because the dissolution is itself a finding.
# =====================================================================================


def test_method_matched_day_confound_dissolved_with_the_phenology_fix() -> None:
    """The scope-(A) matched-day storage confound is GONE — and that is the finding.

    Scope (A): because phenology ran fast, our plant sat at DVS 2 for ~87 extra days,
    inflating a matched-*day* storage read to 0.69 ("we over-allocate") while
    matched-*DVS* said 0.40 ("we under-allocate") — the sign reversed, so matched-day
    was invalid.

    Now maturity lands within 2 days of the oracle, so there is no DVS-2 plateau to
    inflate the matched-day read: **both** comparisons now agree we *under*-allocate
    storage (matched-day 0.39, matched-DVS 0.30, vs the oracle's 0.52). The confound was
    a
    *consequence* of the phenology overrun, and fixing phenology dissolved it.
    Matched-DVS remains the correct method on principle; this pins that the two now
    agree in sign.

    ⚠⚠ **2026-08-12: THE SIGN DISAGREEMENT IS BACK, AND ITS CAUSE IS A DIFFERENT ONE.**
    The stem-reserve build gives the plant a carbon source that keeps delivering to
    grain
    *after* anthesis, and ``StemRemobilization`` has a lower DVS trigger with **no upper
    bound** — so it keeps draining past DVS 2.0, all the way to the end of the fixture.
    Measured:

        matched-DAY  ours 0.544437  vs oracle 0.520755  -> we now OVER-allocate
        matched-DVS  ours 0.465762  vs oracle 0.520755  -> we still UNDER-allocate

    With ``stem_reserves=False`` both reads return to under-allocating (0.390068 /
    0.302037), so the reserve is the cause and not a coincidence.

    ⚠ **THE CONCLUSION THIS TEST DREW IS STILL RIGHT; ITS PIN WAS NEVER THE
    CONCLUSION.**
    The finding was never "the two reads agree" — it was *"matched-day is invalid as a
    method, and it happened to be harmless once phenology was fixed"*. That the two can
    disagree again, from a completely unrelated mechanism, is the **stronger** form of
    the original argument, not a refutation of it. So the sign assertions below are
    re-pointed at what is now true and the method verdict is unchanged.

    ⚠ **AND IT SURFACES AN OPEN DESIGN QUESTION I AM NOT DECIDING HERE:** should
    remobilization stop at maturity? [E] §3.2.4 describes it as a grain-FILL process
    (DVS 1 → 2), and our flow has no upper gate. The frozen ``open_season`` run
    continues
    past DVS 2.0, so the difference is reachable. Deciding it would change goldens
    inside
    the build that raised it — the refused shape — so it is recorded and named as a
    successor. It is written down in the plan doc, not only here.
    """
    states = _season_states()
    ref = _reference()
    n = min(len(ref), _days_in(states))  # was `len(states)` — a STEP count
    days = _by_day(states, n)

    matched_day_ours = _organ_fractions(days[n - 1])["storage"]
    matched_day_oracle = _oracle_fractions(ref[n - 1])["storage"]

    our_dvs = _our_dvs(states, n)
    oracle_dvs = [r["DVS"] for r in ref[:n]]
    oi, ri = _first_day_at(our_dvs, 2.0), _first_day_at(oracle_dvs, 2.0)
    assert oi is not None and ri is not None
    matched_dvs_ours = _organ_fractions(days[oi])["storage"]
    matched_dvs_oracle = _oracle_fractions(ref[ri])["storage"]

    # ⚠ THE SIGNS DISAGREE AGAIN (2026-08-12). Asserted as the disagreement it is,
    # rather than left as the agreement it was: matched-day says we over-allocate,
    # matched-DVS says we under-allocate, and matched-DVS is the one to believe.
    assert matched_day_ours > matched_day_oracle, "matched-day: we OVER-allocate"
    assert matched_dvs_ours < matched_dvs_oracle, "matched-DVS: we UNDER-allocate"

    # The numbers, so the disagreement is legible and not just asserted.
    # Was 0.39 / 0.30 (both under) before the reserve.
    assert matched_day_ours == pytest.approx(0.544, abs=0.03)
    assert matched_dvs_ours == pytest.approx(0.466, abs=0.03)
    assert matched_dvs_oracle == pytest.approx(0.52, abs=0.02)

    # ⚠ THE CONTROL, so the cause is named and not inferred: with the reserve off, both
    # reads go back to under-allocating at the values this test was written around.
    off = _season_states_without_reserve()
    n_off = min(len(ref), _days_in(off))  # was `len(off)` — a STEP count
    off_days = _by_day(off, n_off)
    off_day = _organ_fractions_bare(off_days[n_off - 1])["storage"]
    off_dvs_i = _first_day_at(_our_dvs(off, n_off), 2.0)
    assert off_dvs_i is not None
    off_dvs = _organ_fractions_bare(off_days[off_dvs_i])["storage"]
    assert off_day == pytest.approx(0.390, abs=0.005)
    assert off_dvs == pytest.approx(0.302, abs=0.005)
    assert off_day < matched_day_oracle and off_dvs < matched_dvs_oracle


def test_method_the_death_spiral_is_gone() -> None:
    """Scope (A)'s death-spiral mechanism no longer operates — pinned as its inverse.

    Scope (A) pinned ``leaf[-1] < 0.2 * leaf[peak]``: the starved canopy collapsed to a
    small fraction of an already-tiny peak. With the canopy closing, leaf now ends the
    season at a *substantial* fraction of a real peak (grain fill senesces it, but it
    does not spiral). The mechanism claim is inverted so a regression that re-starves
    the canopy turns this red.
    """
    states = _season_states()
    # ⚠ day-indexed: `peak_day` is used only to index `leaf`, so a step-indexed
    # series was self-consistent and stayed green — but the name says DAY, and this file
    # now carries one unit throughout, so a later comparison cannot pick the wrong one.
    leaf = [s.stocks[LEAF_C].amount for s in _by_day(states, len(_reference()))]
    peak_day = leaf.index(max(leaf))
    # No spiral: leaf ends well above the scope-(A) < 20 % collapse threshold.
    assert leaf[-1] > 0.4 * leaf[peak_day]
    # And the peak itself is a real canopy, not the ~0.15 mol C scope-(A) peak.
    assert leaf[peak_day] > 1.0
