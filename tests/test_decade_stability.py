"""Phase-4 Step-1 (P4.1): the decade-scale Euler probe — the de-risk gating Phase 4.

A **measurement** step. Runs the two closed scenarios (``PERENNIAL_CHAMBER_SCENARIO``
and ``CONSUMER_CHAMBER_SCENARIO``) Euler-daily to a **15-year horizon** (>= the
decade-scale target; the budgeted 15-20 yr working horizon) and asserts the three P4.1
drift axes, then runs the **same** scenarios under ``Rk4Integrator`` as a one-shot
**structural cross-check** that decides Euler-holds-vs-escalate *on evidence*:

* **(a) Mass-conservation drift** — two tiers. The structural **ceiling**
  (``max|d_q| <= N * BALANCE_ATOL``, the triangle-inequality worst case) AND the
  **detector** (``max|d_q| <= MASS_DRIFT_ABS_BOUND`` and ``|drift_slope| <=
  MASS_DRIFT_SLOPE_BOUND`` — the derived round-off-scale bounds, the real test that no
  systematic leak accumulates). The measured trace JITTERS at sqrt(N) round-off (~3e-12
  worst), it does not trend — conservation holds with ~6-9 orders of margin under the
  ceiling.
* **(b) Limit-cycle stationarity** — the per-year ``peak leaf_c`` (and, for the
  consumer, ``year-end consumer_carbon``) summaries are **bounded + non-amplifying**
  past the transient (``is_stationary``) and **non-collapsing** (``non_collapsing`` —
  alive, the mandatory level check that ``is_stationary`` is blind to). The lock does
  NOT require a reached attractor — a still-converging cycle is freezable. The discrete
  ``is_period_2`` check characterizes the settled attractor. ⚠ Since post-roadmap scope
  (B) increment 1 **both** scenarios are period-1 fixed points: adding vernalization +
  photoperiod closed the canopy, so the perennial's period-2 cycle (an artifact of the
  broken canopy regime) lost stability and converged upward to a fixed point — see
  ``test_perennial_leaf_cycle_is_a_fixed_point`` below and
  ``docs/plans/post-roadmap-oracle-match.md``. The consumer was always period-1 (the
  herbivore damps the producer oscillation).
* **(c) Closure carried over the full horizon** — ``rationed == 0``, ``events == ()``,
  carbon loss-sink ``0.0`` on **every** step of the run, for **both** integrators.

**The decide-on-evidence core.** Euler and RK4 differ by O(truncation), so their
attractors will NOT match numerically — agreement is **qualitative/structural**
(same period class, all stationary, bounded, closed). The RK4 run also
**empirically retires** the two preconditions the plan flagged (rather than assuming
them): that RK4
survives the discrete ``annual_reset`` x multistage interaction (it completes without
raising) and that no needed arbitration scale fires (``rationed == 0``; under RK4 a
needed scale is a hard error).

Outcome (measured): Euler holds — conservation rock-solid, cycle stationary, closure
held, Euler/RK4 structurally agree → **lock Euler, with evidence**. Zero ``simcore``
change: ``drift.py`` is a domain module, the RK4 run instantiates the already-shipped
``Rk4Integrator``, no new golden (capture is Step 4). Pure-stdlib data path (committed
JSON weather; no PCSE).
"""

import json
from pathlib import Path

import pytest

from domains.biosphere.drift import (
    MASS_DRIFT_ABS_BOUND,
    MASS_DRIFT_SLOPE_BOUND,
    drift_slope,
    is_period_2,
    is_stationary,
    mass_drift_trace,
    max_abs,
    non_collapsing,
    same_phase_diffs,
    year_summaries,
)
from domains.biosphere.season import (
    CARBON_POOL,
    CONSUMER_CARBON,
    CONSUMER_CHAMBER_SCENARIO,
    LEAF_C,
    LONG_HORIZON_YEARS,
    PERENNIAL_CHAMBER_SCENARIO,
    build_season,
    run_perennial,
    weather_resolver,
)
from simcore.boundary import loss_sink_id
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import BALANCE_ATOL, Quantity
from simcore.state import State

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


_YEAR = len(_weather())  # season length in steps (the tiling + reset period, ~305)
# The budgeted 15-yr working horizon (>= the decade-scale 10-yr target), shared as the
# single source of truth with the long-horizon golden + the freeze manifest
# (scenario.py).
DECADE_YEARS = LONG_HORIZON_YEARS
_STEPS = _YEAR * DECADE_YEARS
_QUANTITIES = (Quantity.CARBON, Quantity.OXYGEN, Quantity.NITROGEN, Quantity.WATER)
_TRANSIENT = 2  # same-phase diffs to drop before the non-amplifying trend (the sow-in)
_PERIOD_TRANSIENT = 8  # years to drop before the period check — reach settled tail


def _run(scenario, integrator_cls) -> tuple[list[State], int, tuple]:
    weather = _weather() * DECADE_YEARS
    state, registry = build_season(scenario)
    resolver = weather_resolver(weather, scenario)
    return run_perennial(
        integrator_cls(registry),
        state,
        scenario,
        resolver,
        1.0,
        len(weather),
        year=_YEAR,
    )


@pytest.fixture(scope="module")
def runs() -> dict[tuple[str, str], tuple[list[State], int, tuple]]:
    """All four decade runs (2 scenarios × {Euler, RK4}), each executed exactly once.

    A raised exception here is a real failure signal: it means RK4 did NOT survive the
    discrete ``annual_reset`` x multistage interaction or a needed arbitration scale hit
    the hard-error path — the preconditions this probe exists to retire.
    """
    scenarios = {
        "perennial": PERENNIAL_CHAMBER_SCENARIO,
        "consumer": CONSUMER_CHAMBER_SCENARIO,
    }
    integrators = {"euler": EulerIntegrator, "rk4": Rk4Integrator}
    return {
        (sname, iname): _run(scenario, icls)
        for sname, scenario in scenarios.items()
        for iname, icls in integrators.items()
    }


# --- per-year summary functions (reference the domain stock ids) -------------


def _peak_leaf(segment) -> float:
    return max(s.stocks[LEAF_C].amount for s in segment)


def _min_carbon_pool(segment) -> float:
    return min(s.stocks[CARBON_POOL].amount for s in segment)


def _year_end_consumer(segment) -> float:
    return segment[-1].stocks[CONSUMER_CARBON].amount


# --- axis (a): mass-conservation drift — ceiling + detector ------------------


@pytest.mark.parametrize("scenario", ["perennial", "consumer"])
@pytest.mark.parametrize("quantity", _QUANTITIES)
def test_decade_conservation_ceiling(runs, scenario, quantity) -> None:
    # The structural ceiling: the triangle-inequality worst case. If it ever trips, the
    # flow legs themselves are unbalanced — a hard bug. Loose (~N*1e-9, ~4.6e-6).
    states, _, _ = runs[(scenario, "euler")]
    trace = mass_drift_trace(states, quantity)
    assert max_abs(trace) <= _STEPS * BALANCE_ATOL


@pytest.mark.parametrize("scenario", ["perennial", "consumer"])
@pytest.mark.parametrize("quantity", _QUANTITIES)
def test_decade_conservation_detector(runs, scenario, quantity) -> None:
    # The REAL test (the teeth): the round-off-scale derived bounds. ``max|d_q|`` is the
    # directly-interpretable accumulation bound; ``drift_slope`` is the systematic-leak
    # signature (a leak is linear in n; round-off is not). A drift bug at ~1e-9/step
    # would breach both, orders below the loose ceiling. Both must hold over the decade.
    states, _, _ = runs[(scenario, "euler")]
    trace = mass_drift_trace(states, quantity)
    assert max_abs(trace) <= MASS_DRIFT_ABS_BOUND
    assert abs(drift_slope(trace)) <= MASS_DRIFT_SLOPE_BOUND


# --- axis (b): limit-cycle stationarity --------------------------------------


@pytest.mark.parametrize("scenario", ["perennial", "consumer"])
def test_decade_leaf_cycle_is_stationary(runs, scenario) -> None:
    # Peak leaf carbon per year: bounded + non-amplifying past the transient (not
    # creeping toward blow-up / annual_reset raising), and non-collapsing (alive — the
    # level check is_stationary cannot see). Bounds are relative to the summary scale, a
    # direction-of-trend test, not a magic equality (anti-flakiness).
    states, _, _ = runs[(scenario, "euler")]
    summaries = year_summaries(states, _YEAR, _peak_leaf)
    diffs = same_phase_diffs(summaries, period=2)
    scale = max(summaries)
    assert is_stationary(
        diffs, bound=0.1 * scale, slope_tol=0.01 * scale, transient=_TRANSIENT
    )
    assert non_collapsing(summaries, floor=0.05)  # peak leaf never collapses to ~0


def test_perennial_leaf_cycle_is_a_fixed_point(runs) -> None:
    # CHANGED by post-roadmap scope (B) increment 1 (vernalization + photoperiod). This
    # asserted a period-2 limit cycle ("gap ~0.07, ~28% of scale") until 2026-07-20.
    # That cycle was a property of the BROKEN CANOPY REGIME, not of the perennial
    # chamber: with the two phenology sciences the canopy closes (~95% light
    # interception vs ~5%), Beer-Lambert saturates, the year-to-year return map's slope
    # drops below 1, and the 2-cycle loses stability — converging UPWARD to a period-1
    # fixed point (peak leaf ~0.25 -> ~1.2). Same mechanism, same flip, and the same
    # evidence as test_biosphere_stress.py::test_stress_perennial_fixed_point_sustained
    # and docs/plans/post-roadmap-oracle-match.md. Flipped, not weakened: still a
    # discrete structural pin, still fails on a period BREAK, plus a liveness floor so a
    # degenerate fixed point at a dead plant cannot pass where the oscillator used to.
    states, _, _ = runs[("perennial", "euler")]
    summaries = year_summaries(states, _YEAR, _peak_leaf)
    assert not is_period_2(summaries, transient=_PERIOD_TRANSIENT)
    tail = summaries[_PERIOD_TRANSIENT:]
    gap = max(abs(tail[k + 1] - tail[k]) for k in range(len(tail) - 1))
    assert gap < 1e-3 * max(tail)  # branches merged → a fixed point
    assert max(tail) > 1.0  # converged UP, not collapsed (liveness)


def test_consumer_leaf_converges_to_a_fixed_point(runs) -> None:
    # The CONSUMER chamber is period-1, NOT period-2: adding the herbivore DAMPS the
    # producer oscillation to a fixed point. Past the transient the per-year peak_leaf
    # converges to a single value (measured adjacent gap ~3e-5, ~1e-4 of scale), so the
    # branch gap collapses and is_period_2 correctly returns False. Assert both the
    # negative (not period-2) and the positive characterization (a settled fixed point:
    # consecutive years nearly equal). This corrects the over-general "period-2" claim.
    states, _, _ = runs[("consumer", "euler")]
    summaries = year_summaries(states, _YEAR, _peak_leaf)
    assert not is_period_2(summaries, transient=_PERIOD_TRANSIENT)
    tail = summaries[_PERIOD_TRANSIENT:]
    gap = max(abs(tail[k + 1] - tail[k]) for k in range(len(tail) - 1))
    assert gap < 1e-3 * max(tail)  # a fixed point: the branches have merged


def test_decade_consumer_biomass_is_stationary_and_alive(runs) -> None:
    # The consumer trophic level persists and its standing biomass reaches a stationary,
    # non-collapsing attractor over the decade — neither blowing up nor starving.
    states, _, _ = runs[("consumer", "euler")]
    summaries = year_summaries(states, _YEAR, _year_end_consumer)
    diffs = same_phase_diffs(summaries, period=2)
    scale = max(summaries)
    assert is_stationary(
        diffs, bound=0.2 * scale, slope_tol=0.02 * scale, transient=_TRANSIENT
    )
    assert non_collapsing(summaries, floor=5e-4)  # consumer carbon stays well above 0


def test_decade_min_carbon_pool_stationary(runs) -> None:
    # Chamber CO2 pool (the producer's only carbon source when sealed) never runs dry
    # and its per-year minimum reaches a stationary attractor — closure is not slowly
    # draining the atmosphere into biomass.
    states, _, _ = runs[("perennial", "euler")]
    summaries = year_summaries(states, _YEAR, _min_carbon_pool)
    diffs = same_phase_diffs(summaries, period=2)
    scale = max(summaries)
    assert is_stationary(
        diffs, bound=0.2 * scale, slope_tol=0.02 * scale, transient=_TRANSIENT
    )
    assert non_collapsing(summaries, floor=0.05)


# --- axis (c): closure carried over the full horizon, BOTH integrators -------


@pytest.mark.parametrize("scenario", ["perennial", "consumer"])
@pytest.mark.parametrize("integrator", ["euler", "rk4"])
def test_decade_closure_held(runs, scenario, integrator) -> None:
    # The Phase-3 closure asserts, now held for the ENTIRE 15-yr horizon, for BOTH
    # integrators: no extinction, the carbon loss-sink stays 0.0 each step (death
    # routes to the in-system litter POOL, not the boundary), so the chamber stays
    # genuinely closed at decade scale.
    states, _, events = runs[(scenario, integrator)]
    assert events == ()
    carbon_loss_sink = loss_sink_id(Quantity.CARBON)
    assert all(s.stocks[carbon_loss_sink].amount == 0.0 for s in states)


# --- the decide-on-evidence core: RK4 precondition retirement + agreement ----


@pytest.mark.parametrize("scenario", ["perennial", "consumer"])
def test_rk4_preconditions_retired(runs, scenario) -> None:
    # The evidence in "lock Euler WITH evidence" — not a throwaway. The RK4 decade run
    # (a) completed without raising (the fixture would have errored otherwise, so it
    # survives the discrete annual_reset x multistage boundary), and (b) never needed an
    # arbitration scale: under RK4 a needed scale is a HARD ERROR, so rationed == 0 with
    # no exception is positive proof the first-order donor-controlled kinetics stay
    # positive under
    # the multistage integrator. Euler's backstop also never fires (rationed == 0).
    _, euler_rationed, _ = runs[(scenario, "euler")]
    _, rk4_rationed, rk4_events = runs[(scenario, "rk4")]
    assert euler_rationed == 0
    assert rk4_rationed == 0
    assert rk4_events == ()


@pytest.mark.parametrize("scenario", ["perennial", "consumer"])
def test_euler_rk4_structural_agreement(runs, scenario) -> None:
    # Agreement is QUALITATIVE / structural, NOT "within X": Euler and RK4 differ by
    # O(truncation), so the attractors do not match numerically (asserted: the final
    # states differ — the cross-check integrated differently). What must agree is the
    # STRUCTURE — same period class (period-2 perennial, period-1 consumer), both
    # stationary, both non-collapsing. This is the one check that
    # distinguishes "Euler is fine" from "Euler's truncation produced a stably-WRONG
    # attractor": if RK4 disagreed on the period class, the lock would not hold.
    euler_states, _, _ = runs[(scenario, "euler")]
    rk4_states, _, _ = runs[(scenario, "rk4")]
    assert euler_states[-1] != rk4_states[-1]  # genuinely different integration

    structure = []
    for states in (euler_states, rk4_states):
        summaries = year_summaries(states, _YEAR, _peak_leaf)
        diffs = same_phase_diffs(summaries, period=2)
        scale = max(summaries)
        assert is_stationary(
            diffs, bound=0.1 * scale, slope_tol=0.01 * scale, transient=_TRANSIENT
        )
        assert non_collapsing(summaries, floor=0.05)
        structure.append(is_period_2(summaries, transient=_PERIOD_TRANSIENT))
    assert structure[0] == structure[1]  # Euler & RK4 agree on the period class


def test_decade_run_is_deterministic(runs) -> None:
    # Bit-identical on a re-run at decade scale (the golden's premise; the reset closure
    # and both integrators are pure).
    states, rationed, events = runs[("perennial", "euler")]
    states2, rationed2, events2 = _run(PERENNIAL_CHAMBER_SCENARIO, EulerIntegrator)
    assert states2[-1] == states[-1]
    assert (rationed2, events2) == (rationed, events)
