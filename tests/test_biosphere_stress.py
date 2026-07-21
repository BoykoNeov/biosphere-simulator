"""Phase-4 Step-3 (P4.1): the 100k-step closed-biosphere stress — the real slow-drift
detector.

The decade probe (``test_decade_stability.py``, 15 yr) is **horizon-blind**: axis (b)
cannot see drift slower than the run, and axis (a)'s round-off accumulation sits only
~4 orders below its bound at decade scale. Roadmap line 211 asks for *"100k+
steps, no drift"* — so this gate runs the two closed scenarios to **328 whole years
(100,040 steps, ~328 yr)** and re-asserts the three P4.1 drift axes at that horizon, the
length where a slow leak or a creeping amplitude drift Step 1 could not resolve becomes
visible.

**Euler-only — the locked reference integrator.** Step 1 locked Euler *with evidence*
(the decade RK4 structural cross-check); Step 3 stresses the **locked** integrator. RK4
at 100k is ~4x the cost for zero decision value, so it is deliberately not re-run here.

**Streaming-chunked, memory-safe.** ``run_perennial`` retains the full ``list[State]``
(length ``steps + 1``) — at 100k that is ~0.8 GB per scenario. This gate does **not**
need the states, only the per-step ``total_quantity`` fold (axis a), one per-year
``peak leaf_c`` / year-end ``consumer_carbon`` summary (axis b), and the closure flags
(axis c). So it drives the run **one year per chunk**, carrying the exact ``State``
forward (``state.n`` is absolute and the forcing tables are indexed by it, so a chunked
run is **bit-identical** to a continuous one — verified), folds the floats out, and
discards the states. Memory is ~4 traces x 100,040 floats ~= 11 MB per scenario, not
0.8 GB. Each chunk's ``YEAR + 1`` states (carried boundary + a full year) is *exactly*
one :func:`year_summaries` segment, so the streamed summaries match the decade test's
semantics.

**The three axes at 328 yr (measured, 2026-06-30 — all hold):**

* **(a) Mass-conservation drift — the slow-drift detector.** Two tiers. The structural
  **ceiling** (``max|d_q| <= N * BALANCE_ATOL ~= 1e-4``) and the **detector** (the
  derived round-off-scale bounds ``MASS_DRIFT_ABS_BOUND`` / ``MASS_DRIFT_SLOPE_BOUND``).
  At 100k the worst ``max|d_q| ~= 7.4e-11`` (WATER) and worst ``|slope| ~= 7.5e-16`` —
  the **slope stayed flat at ~3x machine-eps** (it did *not* grow over the 22x-longer
  run), while ``max|d_q|`` grew ~linearly *at that machine-eps slope* (``slope * N ~=
  7.5e-16 * 1e5 ~= 7.4e-11 ~= max|d_q|``). That is the signature of deterministic
  round-off in the ``total_quantity`` fold, **not** a leak: a real ~1e-9/step leak would
  put ``max|d_q|`` at ~1e-4 (the ceiling) and the slope at ~1e-9 (2 orders over the
  bound). Both bounds hold with margin (~13x under the abs bound, ~4.5 orders under the
  slope bound) — see drift.py's PROVENANCE, now spanning both horizons.
* **(b) Limit-cycle stationarity — now confirmed, not merely "holding".** Over 328 yr
  the per-year ``peak leaf_c`` is **bounded + non-amplifying** (``is_stationary``) and
  **non-collapsing**; the period class is **sustained for the full horizon**. **BOTH**
  scenarios now hold a **period-1 fixed point**: the **consumer** always did (the
  herbivore damps the producer oscillation; settled adjacent gap ~3.4e-5 ~= 1e-4 of
  scale), and the **perennial** joined it in post-roadmap scope (B) increment 1.

  ⚠ *This paragraph read "the perennial holds a genuine period-2 cycle (320 yr of
  strict alternation, gap ~0.07 ~= 28% of scale)" until 2026-07-20.* That cycle turned
  out to be a property of the **broken canopy regime**, not of the perennial chamber:
  adding the two missing phenology sciences (vernalization + photoperiod) dissolved it,
  and **either one alone** is sufficient. The plant converged **upward** — peak leaf
  0.253 -> 1.222, ~4.8x (then -> 0.994, ~3.9x, after the 2026-07-21 scope-B decomposer
  calibration shrank the closed-chamber plant ~19%) — so this is damping by canopy
  closure, not collapse. Table and
  mechanism in ``test_stress_perennial_fixed_point_sustained`` below and
  ``docs/plans/post-roadmap-oracle-match.md``. This is the horizon where "the attractor
  holds" becomes a *measurement* — including when the attractor itself changes class.
* **(c) Closure carried over the full horizon.** ``rationed == 0`` (kinetics, not the
  Euler backstop), ``events == ()`` (no extinction), carbon loss-sink ``0.0`` on *every*
  one of all 100,040 steps — the strongest closure statement, over ~328 years.

**Gating.** The whole module is marked ``slow`` (the two 100k runs ~= 30 s total are the
cost) — an opt-*out* handle: a bare ``uv run pytest`` runs it (so it is not theater); a
fast loop deselects with ``-m "not slow"`` or shrinks it with
``STATION_BIOSPHERE_STRESS_YEARS=15 uv run pytest`` (the long-horizon axis-(b) checks
``skipif`` below the decade scale, where they cannot be meaningful; axes (a)/(c) are
scale-robust and always run). The measured wall-clock + the round-off slope vs the
ceiling are **logged** (the plan's "slowness logged, not silently capped").

Assertion-only — **no** golden here (Step 4 captures the *decade* hex snapshot + the
drift-summary golden; a 100k hex snapshot is not a deliverable). Pure-stdlib data path
(committed JSON weather; no PCSE). ``git diff src/simcore/`` stays empty: ``drift.py``
is a domain module and this file is under ``tests/``.
"""

import json
import os
import time
from pathlib import Path

import pytest

from domains.biosphere.drift import (
    MASS_DRIFT_ABS_BOUND,
    MASS_DRIFT_SLOPE_BOUND,
    drift_slope,
    is_period_2,
    is_stationary,
    max_abs,
    non_collapsing,
    same_phase_diffs,
    total_quantity,
)
from domains.biosphere.season import (
    CONSUMER_CARBON,
    CONSUMER_CHAMBER_SCENARIO,
    LEAF_C,
    PERENNIAL_CHAMBER_SCENARIO,
    SeasonScenario,
    build_season,
    run_perennial,
    weather_resolver,
)
from simcore.boundary import loss_sink_id
from simcore.integrator import EulerIntegrator
from simcore.quantities import BALANCE_ATOL, Quantity

# The two 100k runs are the cost; mark the whole module slow so `-m "not slow"` skips it
# while a bare `uv run pytest` still runs it (opt-OUT, not deselect-by-default — the
# latter would make the gate theater with no CI running `-m slow`). Mirrors
# test_stability.py (the Phase-0.5 *engine* 100k gate; this is the *biosphere* one).
pytestmark = pytest.mark.slow

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


_YEAR = len(_weather())  # season length in steps (the tiling + reset period, ~305)
# 328 whole years (328 * 305 = 100,040 steps) — "100k+" framed as WHOLE years so every
# chunk is exactly one year-summary (100000 is not a multiple of 305 → a partial final
# year, a spurious summary + a boundary edge case; whole years dissolve it). Env-
# overridable for a fast iteration loop (the long-horizon axis-(b) checks skip below the
# decade scale — see _DECADE_YEARS).
_YEARS = int(os.environ.get("STATION_BIOSPHERE_STRESS_YEARS", "328"))
_STEPS = _YEAR * _YEARS
_QUANTITIES = (Quantity.CARBON, Quantity.OXYGEN, Quantity.NITROGEN, Quantity.WATER)
_CARBON_LOSS = loss_sink_id(Quantity.CARBON)

_TRANSIENT = 2  # same-phase diffs to drop before the non-amplifying trend (the sow-in)
_PERIOD_TRANSIENT = 8  # years to drop before the period check — reach the settled tail
_DECADE_YEARS = 12  # axis (b) needs a real horizon; below this it cannot be meaningful


class _StressResult:
    """The folded result of one streamed 328-yr run — floats only, states discarded.

    ``drift[q]`` is ``[total_q(n) - total_q(0)]`` over the whole trajectory (the axis-a
    accumulation trace); ``peak_leaf`` / ``consumer`` are the per-year axis-b summaries
    (one scalar per year); ``rationed`` / ``events`` / ``max_loss_sink`` are the axis-c
    closure flags; ``elapsed`` is the wall-clock (logged, not capped).
    """

    def __init__(
        self,
        drift: dict[Quantity, list[float]],
        peak_leaf: list[float],
        consumer: list[float],
        rationed: int,
        events: tuple,
        max_loss_sink: float,
        elapsed: float,
    ) -> None:
        self.drift = drift
        self.peak_leaf = peak_leaf
        self.consumer = consumer
        self.rationed = rationed
        self.events = events
        self.max_loss_sink = max_loss_sink
        self.elapsed = elapsed


def _stream(scenario: SeasonScenario, *, has_consumer: bool) -> _StressResult:
    """Drive ``scenario`` Euler-daily for ``_YEARS`` years, one year per chunk.

    Reuses ``run_perennial`` per chunk, carrying the exact ``State`` forward (so the run
    is bit-identical to a continuous ``run_perennial(steps=_STEPS)`` — verified) while
    retaining only the folded floats. The forcing tables are indexed by the absolute
    ``state.n`` (clamped at the table end), so the resolver is built **once** over the
    fully tiled weather and is valid for every chunk.
    """
    weather = _weather() * _YEARS  # full tiled forcing table (resolver indexes abs n)
    resolver = weather_resolver(weather, scenario)
    state, registry = build_season(scenario)
    integrator = EulerIntegrator(registry)

    totals: dict[Quantity, list[float]] = {q: [] for q in _QUANTITIES}  # rebased below
    peak_leaf: list[float] = []
    consumer: list[float] = []
    rationed = 0
    events: tuple = ()
    max_loss_sink = 0.0
    first = True
    start = time.perf_counter()
    for _ in range(_YEARS):
        states, r, e = run_perennial(
            integrator, state, scenario, resolver, 1.0, _YEAR, year=_YEAR
        )
        # Each chunk returns YEAR+1 states (carried boundary + a full year) == exactly
        # one year_summaries segment. For the per-step total_q trace, the carried
        # boundary state[0] duplicates the previous chunk's last state — skip it past 0.
        segment = states if first else states[1:]
        for q in _QUANTITIES:
            totals[q].extend(total_quantity(s, q) for s in segment)
        peak_leaf.append(max(s.stocks[LEAF_C].amount for s in states))
        if has_consumer:
            consumer.append(states[-1].stocks[CONSUMER_CARBON].amount)
        rationed += r
        events = events + e
        max_loss_sink = max(
            max_loss_sink, max(s.stocks[_CARBON_LOSS].amount for s in segment)
        )
        state = states[-1]
        first = False
    elapsed = time.perf_counter() - start

    drift = {q: [v - totals[q][0] for v in totals[q]] for q in _QUANTITIES}
    return _StressResult(
        drift, peak_leaf, consumer, rationed, events, max_loss_sink, elapsed
    )


@pytest.fixture(scope="module")
def runs() -> dict[str, _StressResult]:
    """Both 100k streamed runs (perennial, consumer), each executed exactly once.

    Module-scoped: each run is ~15 s but retains only ~11 MB of floats, so caching both
    is cheap (unlike the retain-all states it folds away).
    """
    return {
        "perennial": _stream(PERENNIAL_CHAMBER_SCENARIO, has_consumer=False),
        "consumer": _stream(CONSUMER_CHAMBER_SCENARIO, has_consumer=True),
    }


# --- axis (a): mass-conservation drift — the slow-drift detector at 328 yr ----


@pytest.mark.parametrize("scenario", ["perennial", "consumer"])
@pytest.mark.parametrize("quantity", _QUANTITIES)
def test_stress_conservation_ceiling(runs, scenario, quantity) -> None:
    # The structural ceiling: the triangle-inequality worst case (~N*1e-9 ~= 1e-4). If
    # it ever trips, the flow legs themselves are unbalanced — a hard bug, not drift.
    trace = runs[scenario].drift[quantity]
    assert max_abs(trace) <= _STEPS * BALANCE_ATOL


@pytest.mark.parametrize("scenario", ["perennial", "consumer"])
@pytest.mark.parametrize("quantity", _QUANTITIES)
def test_stress_conservation_detector(runs, scenario, quantity) -> None:
    # The REAL slow-drift test at 328 yr (the teeth). Over a 22x-longer run than the
    # decade probe, max|d_q| stayed ~7.4e-11 (still ~13x under the abs bound) and the
    # slope stayed flat at ~machine-eps (~4.5 orders under the slope bound): no
    # systematic leak emerged across ~328 years. A ~1e-9/step leak would breach BOTH
    # (max|d_q| ~ 1e-4, slope ~ 1e-9), orders below the loose ceiling.
    trace = runs[scenario].drift[quantity]
    assert max_abs(trace) <= MASS_DRIFT_ABS_BOUND
    assert abs(drift_slope(trace)) <= MASS_DRIFT_SLOPE_BOUND


# --- axis (b): limit-cycle stationarity — confirmed over the long horizon -----


@pytest.mark.skipif(
    _YEARS < _DECADE_YEARS, reason="axis (b) needs >= decade scale to be meaningful"
)
@pytest.mark.parametrize("scenario", ["perennial", "consumer"])
def test_stress_leaf_cycle_is_stationary(runs, scenario) -> None:
    # Per-year peak leaf carbon over 328 yr: bounded + non-amplifying past the transient
    # (no creep toward blow-up / annual_reset raising) AND non-collapsing (alive — the
    # level check is_stationary is blind to). Bounds are relative to the summary scale.
    summaries = runs[scenario].peak_leaf
    diffs = same_phase_diffs(summaries, period=2)
    scale = max(summaries)
    assert is_stationary(
        diffs, bound=0.1 * scale, slope_tol=0.01 * scale, transient=_TRANSIENT
    )
    assert non_collapsing(summaries, floor=0.05)


@pytest.mark.skipif(_YEARS < _DECADE_YEARS, reason="period check needs >= decade scale")
def test_stress_perennial_fixed_point_sustained(runs) -> None:
    # CHANGED by post-roadmap scope (B) increment 1 (vernalization + photoperiod). This
    # test previously asserted a period-2 limit cycle (`is_period_2`, gap ~0.07, ~28%
    # of scale). That cycle was NOT a robust property of the perennial chamber — it was
    # property of the BROKEN CANOPY REGIME underneath it, and adding the two missing
    # phenology sciences dissolved it. The test is flipped, not weakened: it still
    # pins a discrete structural property over the horizon and still fails on a period
    # break — the branch it pins is simply the other one now.
    #
    # Measured, isolating each term (docs/plans/post-roadmap-oracle-match.md). NOTE: the
    # peak leaf below is pre-2026-07-21, at the OLD decomposer rates; the scope-B
    # calibration (docs/plans/post-roadmap-decomposer-calibration.md) later shrank the
    # shipped fixed point to 0.994. The qualitative point (either phenology term alone
    # -> fixed point) holds; only the magnitudes shifted with slower carbon recycling.
    #
    #   config                  peak leaf   max adjacent gap
    #   both inert (baseline)     0.2530    7.157e-02  (28.28% — reproduces the old pin)
    #   vernalization only        1.0171    4.44e-16   (fixed point)
    #   photoperiod only          1.0795    0.0        (fixed point)
    #   both (shipped, old k)     1.2215    1.55e-15   (fixed point; now 0.994 at new k)
    #
    # EITHER term alone collapses it, so this is not photoperiod entrainment (the first
    # hypothesis, measured and REJECTED). The mechanism is canopy closure flattening the
    # year-to-year return map: at baseline the starved canopy sits at ~5% light
    # interception, where Beer-Lambert is still nearly LINEAR in LAI, so a good year
    # begets a bad one with gain > 1. Slowing development lets the canopy close (~95.6%
    # interception), where Beer-Lambert SATURATES — a change in starting leaf barely
    # moves intercepted light, the map's slope at the fixed point drops below 1, and the
    # 2-cycle loses stability. Same damping story as the consumer chamber below, where a
    # herbivore rather than light saturation supplies the damping.
    summaries = runs["perennial"].peak_leaf
    assert not is_period_2(summaries, transient=_PERIOD_TRANSIENT)
    tail = summaries[_PERIOD_TRANSIENT:]
    gap = max(abs(tail[k + 1] - tail[k]) for k in range(len(tail) - 1))
    assert gap < 1e-3 * max(tail)  # the branches have merged → a fixed point
    # ...and it converged UP, not by collapsing: the plant is ~3.9x healthier than the
    # oscillating baseline (0.253 -> 0.9942). It was 1.2215 with the pre-calibration
    # fast decomposers; the scope-B decomposer calibration (decomp 0.02->0.011, micro
    # 0.05->0.016; docs/plans/post-roadmap-decomposer-calibration.md) slows the
    # recycled-CO2 loop and shrinks the closed-chamber plant ~19%, so the sustained
    # fixed point drops to 0.9942 -- still a robustly-alive plant (CO2min 0.039, storage
    # 0.308 >> 0.16 seed), ~3.9x the 0.253 dead baseline. A degenerate "fixed point" at
    # a dead plant would pass the two assertions above; this one cannot. Floor 0.9 keeps
    # the alive-not-dead guard with margin over 0.253.
    assert max(tail) > 0.9


@pytest.mark.skipif(_YEARS < _DECADE_YEARS, reason="period check needs >= decade scale")
def test_stress_consumer_fixed_point_sustained(runs) -> None:
    # The consumer chamber holds its period-1 fixed point for all 328 years (the
    # herbivore damps the producer oscillation): is_period_2 stays False and the settled
    # adjacent gap stays collapsed (~3.4e-5, ~1e-4 of scale) — no late re-emergence of
    # an oscillation, no slow drift off the fixed point.
    summaries = runs["consumer"].peak_leaf
    assert not is_period_2(summaries, transient=_PERIOD_TRANSIENT)
    tail = summaries[_PERIOD_TRANSIENT:]
    gap = max(abs(tail[k + 1] - tail[k]) for k in range(len(tail) - 1))
    assert gap < 1e-3 * max(tail)  # the branches have merged → a fixed point


@pytest.mark.skipif(
    _YEARS < _DECADE_YEARS, reason="axis (b) needs >= decade scale to be meaningful"
)
def test_stress_consumer_biomass_stationary_and_alive(runs) -> None:
    # The consumer trophic level persists over 328 yr and its standing biomass holds a
    # stationary, non-collapsing attractor — neither blowing up nor starving across the
    # full horizon.
    summaries = runs["consumer"].consumer
    diffs = same_phase_diffs(summaries, period=2)
    scale = max(summaries)
    assert is_stationary(
        diffs, bound=0.2 * scale, slope_tol=0.02 * scale, transient=_TRANSIENT
    )
    assert non_collapsing(summaries, floor=5e-4)


# --- axis (c): closure carried over the entire 100,040-step horizon ----------


@pytest.mark.parametrize("scenario", ["perennial", "consumer"])
def test_stress_closure_held(runs, scenario) -> None:
    # The Phase-3 closure, now held for the ENTIRE ~328-yr horizon: no extinction
    # (events == ()), the Euler backstop never fired (rationed == 0 — positivity from
    # first-order donor-controlled kinetics, not arbitration), and the carbon loss-sink
    # stayed 0.0 every step (death routes to the in-system litter POOL). The chamber
    # stays genuinely closed at 100k+.
    result = runs[scenario]
    assert result.rationed == 0
    assert result.events == ()
    assert result.max_loss_sink == 0.0


# --- the plan's "slowness logged, not silently capped" -----------------------


def test_stress_report_logged(runs, capsys) -> None:
    # Not an assertion gate — it surfaces the measured round-off (max|d_q| + slope) vs
    # the N*BALANCE_ATOL ceiling and the wall-clock, the plan's "report round-off slope
    # vs the ceiling" + "slowness logged, not silently capped". Visible with -s / on any
    # failure in this module.
    lines = [
        f"\n[biosphere stress] {_YEARS} yr = {_STEPS} steps; ceiling "
        f"N*BALANCE_ATOL = {_STEPS * BALANCE_ATOL:.3e}"
    ]
    for name, result in runs.items():
        worst = max(max_abs(result.drift[q]) for q in _QUANTITIES)
        worst_slope = max(abs(drift_slope(result.drift[q])) for q in _QUANTITIES)
        lines.append(
            f"  {name:9s} wall={result.elapsed:6.1f}s  worst max|d_q|={worst:.3e}  "
            f"worst |slope|={worst_slope:.3e}  rationed={result.rationed}"
        )
    with capsys.disabled():
        print("\n".join(lines))
