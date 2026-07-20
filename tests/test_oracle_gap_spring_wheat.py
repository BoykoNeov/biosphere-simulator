"""The day-neutral crop vs the LINTUL3 spring-wheat oracle — a DIAGNOSTIC, never a fit.

Plan of record: ``docs/plans/post-roadmap-day-neutral-crop.md``. This is the "second
wheat" ceremony 2 left open — a warm-habitat crop with no cold requirement — diagnosed
against PCSE's bundled ``lintul3_springwheat`` oracle (offline, license-clean).

**Read the framing before the numbers (advisor).** Our DVS is the **same model family**
as LINTUL3 — both are linear growing-degree-day thermal time with a two-phase
``TSUM1``/``TSUM2`` DVS. So a phenology "match" is **near-tautological**: it tests our
*param choice*, not our model. The genuine cross-model signal lives where the
families differ — **canopy (LAI) dynamics** and **biomass** (LINTUL3's
light-use-efficiency vs our FvCB), where we **cannot fit and do not** (ruling B: the
oracle is a diagnostic, never a fit target). So this file does **not** claim "we
validated our crop against an oracle"; it pins *a literature-cited, sane day-neutral
crop with its gaps to LINTUL3 measured*.

The day-neutral crop reuses the **same cited winter-wheat phenology params**
(``phenology.yaml``; ``tsum_anthesis=1100``/``tsum_maturity=750``, Penning de Vries
1989)
with vernalization + photoperiod **structurally off** — no clean primary spring-wheat
``TSUM`` exists on the shelf, and reusing our own cited values (rather than copying
LINTUL3's ``TSUM1=800``/``TSUM2=1030``, which would be reverse-engineering PCSE) is the
ruling-B-clean choice: use independently-justified params, let DVS timing fall where it
falls, and record the gap.

**The headline finding corroborates ceremony 2 across a SECOND, independent oracle.**
Our ``tsum`` partition is **vegetative-heavy** (1100/750); *both* the WOFOST
winter-wheat oracle (ceremony 2) and now the LINTUL3 spring-wheat oracle (800/1030) are
**reproductive-heavy**. Two independent oracles agree wheat's partition is more
grain-fill-weighted than our winter-wheat ``tsum`` — strengthening ceremony 2's
"cultivar variation, not our error" reading, and it is exactly the same
two-errors-cancelling maturity coincidence (see below).

Our-side day/LAI numbers carry ``pytest.approx`` bands: LAI + organ carbon flow through
FvCB (``exp``/``sqrt``), so they are last-ULP libm-sensitive off the Windows/UCRT
platform (the ``tests/golden_platform.py`` trap). Oracle-side values are exact
(committed JSON).
PCSE-free: both fixtures are read as JSON.
"""

import json
from pathlib import Path

import pytest

from domains.biosphere.canopy import leaf_area_index
from domains.biosphere.loader import load_canopy_params, load_phenology_params
from domains.biosphere.phenology import development_stage
from domains.biosphere.scenario import DAY_NEUTRAL_SCENARIO
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
_REFERENCE = _ORACLE_DIR / "spring_wheat_reference.json"
_WEATHER = _ORACLE_DIR / "spring_wheat_weather.json"

_GROUND_AREA = 1.0  # m² — the open-field plot (SeasonScenario default)


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER.read_text(encoding="utf-8"))["weather"]


def _oracle() -> list[dict[str, float]]:
    """The post-emergence oracle rows (DVS not null) — day 0 == emergence, aligned to
    our thermal_time = 0 at emergence."""
    traj = json.loads(_REFERENCE.read_text(encoding="utf-8"))["trajectory"]
    return [r for r in traj if r["DVS"] is not None]


def _our_states(n: int) -> list[State]:
    """Run the day-neutral crop (vern+photoperiod off) under spring weather for ``n``
    steps — same forcing as the oracle."""
    state, registry = build_season(DAY_NEUTRAL_SCENARIO)
    resolver = weather_resolver(_weather(), DAY_NEUTRAL_SCENARIO)
    states, rationed, events = run_season(
        EulerIntegrator(registry), state, resolver, 1.0, n
    )
    # Conservation/determinism sanity — a day-neutral crop is still a well-behaved run.
    assert rationed == 0
    assert events == ()
    return states


def _our_dvs(states: list[State], n: int) -> list[float]:
    pp = load_phenology_params()
    return [
        development_stage(
            states[i].aux["thermal_time"],
            tsum_anthesis=pp.tsum_anthesis,
            tsum_maturity=pp.tsum_maturity,
        )
        for i in range(n)
    ]


def _our_lai(states: list[State], n: int) -> list[float]:
    cp = load_canopy_params()
    return [
        leaf_area_index(
            states[i].stocks[LEAF_C].amount,
            sla_per_mol_c=cp.sla_per_mol_c,
            ground_area=_GROUND_AREA,
        )
        for i in range(n)
    ]


def _first_day_at(series: list[float], threshold: float) -> int | None:
    for i, v in enumerate(series):
        if v >= threshold:
            return i
    return None


def _our_root_fraction(state: State) -> float:
    organs = {
        "leaf": state.stocks[LEAF_C].amount,
        "stem": state.stocks[STEM_C].amount,
        "root": state.stocks[ROOT_C].amount,
        "storage": state.stocks[STORAGE_C].amount,
    }
    return organs["root"] / sum(organs.values())


def _oracle_root_fraction(row: dict[str, float]) -> float:
    # LINTUL3 organ weights (g m⁻²); green + dead leaf, stem, root, storage.
    organs = {
        "leaf": row["WLVG"] + row["WLVD"],
        "stem": row["WST"],
        "root": row["WRT"],
        "storage": row["WSO"],
    }
    return organs["root"] / sum(organs.values())


# =====================================================================================
# Column 1 — the POSITIVE: shape + the canopy magnitude genuinely agrees
# =====================================================================================


def test_shape_both_complete_the_development_arc() -> None:
    """Both reach maturity (DVS → 2) — day-neutral develops without a cold cue."""
    oracle = _oracle()
    our = _our_dvs(_our_states(len(oracle)), len(oracle))
    assert our[-1] >= 1.9
    assert max(r["DVS"] for r in oracle) >= 1.9


def test_peak_lai_magnitude_is_a_realistic_canopy_for_both() -> None:
    """The canopy CLOSES to a realistic wheat height — but read this carefully.

    Peak LAI is **5.60 (ours) vs 5.73 (LINTUL3)** — a ratio of **1.02×**, tighter than
    the winter-wheat oracle's 1.22× (``test_oracle_gap.py``). The day-neutral crop's
    canopy bootstraps and closes with **no canopy science and no param fit**. But two
    *independently-parameterized* canopy models both landing near a realistic wheat
    peak is "both are sane", **not** cross-validation (the same caution as the phenology
    tautology, one level over). Recorded because the canopy closes rather than
    collapsing; the real cross-model signal is the *timing* of that peak (next test).
    """
    oracle = _oracle()
    n = len(oracle)
    our_peak = max(_our_lai(_our_states(n), n))
    oracle_peak = max(r["LAI"] for r in oracle)
    # FvCB-derived (exp/sqrt) ⇒ last-ULP libm-sensitive off Windows/UCRT; bands set
    # defensively wide (winter oracle-gap precedent — approx-bands absorb cross-libm).
    assert our_peak == pytest.approx(5.60, abs=0.5)  # a real, closed canopy
    assert our_peak > 4.5
    assert oracle_peak / our_peak == pytest.approx(1.02, abs=0.12)


# =====================================================================================
# Column 2 — THE GAPS (the payload), each a pinned number. Ruling B: documented, never
# fit. A recalibration or a canopy-timing fix turns these red — update, do not delete.
# =====================================================================================


def test_gap_partition_is_vegetative_heavy_corroborating_ceremony_2() -> None:
    """THE HEADLINE — our ``tsum`` partition is veg-heavy vs a SECOND oracle.

    Emergence → anthesis is **~94 days (ours) vs 74 (LINTUL3)**; anthesis → maturity is
    **~41 days (ours) vs 61**. Our vegetative phase is too long and grain fill too short
    — the *identical direction* ceremony 2 found against the WOFOST winter-wheat oracle
    (there: 251 vs 217 veg, 43 vs 75 repro). Two independent oracles of different model
    families (WOFOST assimilation; LINTUL3 light-use-efficiency) agree wheat's partition
    is more reproductive-heavy than our winter-wheat ``tsum`` (1100/750). Per ceremony 2
    this is **cultivar variation**, recorded not fitted (ruling B) — and the
    cross-oracle agreement is new evidence for that reading.
    """
    oracle = _oracle()
    n = len(oracle)
    our = _our_dvs(_our_states(n), n)
    oracle_dvs = [r["DVS"] for r in oracle]

    our_ant, our_mat = _first_day_at(our, 1.0), _first_day_at(our, 2.0)
    oracle_ant, oracle_mat = (
        _first_day_at(oracle_dvs, 1.0),
        _first_day_at(oracle_dvs, 2.0),
    )
    assert our_ant is not None and our_mat is not None
    assert oracle_ant is not None and oracle_mat is not None

    # Oracle side: exact (committed JSON, emergence-relative days).
    assert (oracle_ant, oracle_mat) == (74, 135)
    assert oracle_mat - oracle_ant == 61  # oracle reproductive phase
    # Our side: ±3 days (thermal-time threshold crossings).
    assert our_ant == pytest.approx(94, abs=3)
    our_repro = our_mat - our_ant
    assert our_repro == pytest.approx(41, abs=4)
    # The finding: our vegetative phase is longer, our reproductive phase shorter.
    assert our_ant > oracle_ant
    assert our_repro < oracle_mat - oracle_ant


def test_gap_maturity_coincides_but_it_is_two_errors_cancelling() -> None:
    """Maturity lands on day 135 for BOTH — and, as in ceremony 2, that is a trap.

    It is **two errors cancelling**: our anthesis is ~20 days late and our grain fill
    ~20 days short, so the maturity *date* nets to ~0 error while the partition beneath
    is wrong (previous test). The mechanism is a total-magnitude near-coincidence — our
    thermal time at maturity is ~1855 °C·day (= our 1100+750 ``tsum``), and LINTUL3's is
    ~1830 (800+1030), so both cross maturity on nearly the same day under this weather.
    Pinned so the coincidence is never mistaken for an endpoint validation (the
    ungated-prose-half pattern, caught in the assertion's own narrative).
    """
    oracle = _oracle()
    n = len(oracle)
    our = _our_dvs(_our_states(n), n)
    oracle_dvs = [r["DVS"] for r in oracle]
    our_mat = _first_day_at(our, 2.0)
    oracle_mat = _first_day_at(oracle_dvs, 2.0)
    assert oracle_mat == 135  # exact (committed JSON)
    assert our_mat == pytest.approx(135, abs=3)


def test_gap_lai_peaks_after_anthesis_not_before() -> None:
    """The genuine cross-model signal — canopy TIMING diverges (not tautological).

    A wheat canopy should peak at or just before anthesis and decline through grain
    fill: the oracle peaks on **day 72, ~2 days BEFORE its anthesis (day 74)**. Ours
    peaks on **day ~107, ~13 days AFTER our anthesis (~day 94)** — a senescence/
    allocation timing residual, the same direction as the winter-wheat oracle
    (``test_oracle_gap.py``). This is where the two model families genuinely differ
    (FvCB allocation + relative-death senescence vs LINTUL3's LUE + DVS-driven leaf
    death), so it is a *real* finding, not a same-family artefact.
    """
    oracle = _oracle()
    n = len(oracle)
    states = _our_states(n)
    our_lai = _our_lai(states, n)
    oracle_lai = [r["LAI"] for r in oracle]
    our_dvs = _our_dvs(states, n)
    anthesis = _first_day_at(our_dvs, 1.0)
    assert anthesis is not None

    assert oracle_lai.index(max(oracle_lai)) == 72  # oracle: exact, before its anthesis
    # ±12 days: argmax over a broad flat hump (libm-sensitive winner off Windows/UCRT).
    # The finding is "AFTER anthesis" (the next assert), not the exact day.
    assert our_lai.index(max(our_lai)) == pytest.approx(107, abs=12)
    assert our_lai.index(max(our_lai)) > anthesis  # the slip: peak AFTER anthesis


def test_gap_oracle_allocates_more_to_root_early() -> None:
    """A partition-model difference — LINTUL3 is far more root-heavy early.

    At matched DVS 0.5 the oracle's root fraction is **0.55 vs our 0.31** — the
    *opposite* sign of the winter-wheat oracle finding (there WE were root-heavy),
    because LINTUL3's own ``FRTTB`` front-loads roots (60 % at emergence). By anthesis
    (DVS 1.0) the two converge (**0.25 vs 0.25**). A model-partition difference recorded
    at matched DVS (never matched day), not a defect to fit.
    """
    oracle = _oracle()
    n = len(oracle)
    states = _our_states(n)
    our_dvs = _our_dvs(states, n)
    oracle_dvs = [r["DVS"] for r in oracle]

    # DVS 0.5 — the oracle is much more root-heavy than us. Our-side fractions are
    # FvCB-derived ⇒ libm-sensitive: banded wide (the winter-precedent). Oracle: exact.
    our_i = _first_day_at(our_dvs, 0.5)
    oracle_i = _first_day_at(oracle_dvs, 0.5)
    assert our_i is not None and oracle_i is not None
    assert _our_root_fraction(states[our_i]) == pytest.approx(0.31, abs=3e-2)
    assert _oracle_root_fraction(oracle[oracle_i]) == pytest.approx(0.548, abs=1e-3)
    assert _oracle_root_fraction(oracle[oracle_i]) > _our_root_fraction(states[our_i])

    # DVS 1.0 — they converge.
    our_j = _first_day_at(our_dvs, 1.0)
    oracle_j = _first_day_at(oracle_dvs, 1.0)
    assert our_j is not None and oracle_j is not None
    assert _our_root_fraction(states[our_j]) == pytest.approx(0.25, abs=3e-2)
    assert _oracle_root_fraction(oracle[oracle_j]) == pytest.approx(0.246, abs=1e-3)
