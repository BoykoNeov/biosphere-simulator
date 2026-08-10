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

import dataclasses
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
from domains.biosphere.microbial_respiration import MicrobialRespiration
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
from simcore.registry import Registry
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


@pytest.mark.science_gate(
    scenario="perennial_long_horizon",
    field="liveness_floors",
    quantity="annual peak leaf carbon (mol C)",
    bound="non_collapsing(floor=0.05)",
    source="self — the calibrated attractor, not a cited value",
)
@pytest.mark.science_gate(
    scenario="consumer_long_horizon",
    field="liveness_floors",
    quantity="annual peak leaf carbon (mol C)",
    bound="non_collapsing(floor=0.05)",
    source="self — the calibrated attractor, not a cited value",
)
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


@pytest.mark.science_gate(
    scenario="perennial_long_horizon",
    field="liveness_floors",
    quantity="converged peak-leaf fixed point (mol C)",
    bound="max(tail) > 0.55",
    source="self — anchored on the MEASURED equilibrium 0.594984 (reached ~yr 45), not "
    "on the 15-yr reading; 2.2x the 0.253 dead baseline. Second move: >1.0 -> >0.9 "
    "(decomposer calibration) -> >0.55 (humification split)",
)
def test_perennial_leaf_cycle_is_a_fixed_point(runs) -> None:
    # CHANGED by post-roadmap scope (B) increment 1 (vernalization + photoperiod). This
    # asserted a period-2 limit cycle ("gap ~0.07, ~28% of scale") until 2026-07-20.
    # That cycle was a property of the BROKEN CANOPY REGIME, not of the perennial
    # chamber: with the two phenology sciences the canopy closes (~95% light
    # interception vs ~5%), Beer-Lambert saturates, the year-to-year return map's slope
    # drops below 1, and the 2-cycle loses stability — converging UPWARD to a period-1
    # fixed point (peak leaf ~0.25 -> ~1.2, then ~0.99 after the scope-B decomposer
    # calibration). Same mechanism, same flip, and the same evidence as
    # test_biosphere_stress.py::test_stress_perennial_fixed_point_sustained and
    # docs/plans/post-roadmap-oracle-match.md. Flipped, not weakened: still a discrete
    # structural pin, still fails on a period BREAK, plus a liveness floor so a
    # degenerate fixed point at a dead plant cannot pass where the oscillator used to.
    states, _, _ = runs[("perennial", "euler")]
    summaries = year_summaries(states, _YEAR, _peak_leaf)
    assert not is_period_2(summaries, transient=_PERIOD_TRANSIENT)
    tail = summaries[_PERIOD_TRANSIENT:]
    # ⚠ RESTATED by the humification split (2026-08-10), and the restatement is the
    # finding. This asserted ``gap < 1e-3 * max(tail)`` -- "the branches have merged
    # into
    # a fixed point" -- which was true while the chamber settled in ~3 years. The
    # humification split does NOT destabilise the chamber; it lengthens the settling
    # transient by an order of magnitude, from ~3 years to ~35, because the humus pool
    # fills on its own ~5-yr turnover. The attractor is still there and is still
    # period-1 -- it is measured at 0.594984 by year ~45 in
    # ``test_the_perennial_decline_has_a_floor_beyond_the_frozen_horizon`` below -- but
    # it is NOT REACHED inside the frozen 15-year horizon, so an equality-shaped pin
    # here would be asserting something false.
    #
    # What is true at 15 years, and is what a still-converging monotone approach means:
    diffs = [tail[k + 1] - tail[k] for k in range(len(tail) - 1)]
    assert all(d < 0.0 for d in diffs)  # monotone decline, no oscillation
    assert all(
        abs(diffs[k + 1]) < abs(diffs[k]) for k in range(len(diffs) - 1)
    )  # and DECELERATING -- converging, not running away
    assert abs(diffs[-1]) < 1e-2 * max(tail)  # the approach is already slow
    # Liveness floor. ⚠ This is the SECOND time this floor has moved to accommodate a
    # smaller plant (1.0 -> 0.9 when the scope-B decomposer calibration shrank it ~19%;
    # now 0.9 -> 0.55), and saying so plainly is the point -- a floor whose manifest
    # entry
    # reads "self -- the calibrated attractor, not a cited value" guards CONTINUITY with
    # the current calibration, not plausibility, so a deliberate recalibration is
    # supposed to move it. What must NOT happen is moving it to just below whatever the
    # run produced. It is anchored instead on the MEASURED EQUILIBRIUM (0.594984) rather
    # than on the 15-yr reading (0.634352), so the bound does not depend on the horizon,
    # and 0.55 sits below the equilibrium while staying 2.2x the 0.253 DEAD baseline --
    # which is the quantity a liveness floor exists to separate the plant from.
    assert max(tail) > 0.55


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
    # ⚠ RESTATED alongside the perennial pin (the humification split, 2026-08-10) and
    # for the same reason: the settling transient now outruns the frozen horizon, so at
    # year 15 this chamber is still converging rather than converged. The NEGATIVE claim
    # this test exists for -- that the herbivore damps the producer oscillation, so the
    # attractor is period-1 and not period-2 -- is untouched and is asserted above.
    diffs = [tail[k + 1] - tail[k] for k in range(len(tail) - 1)]
    assert all(d < 0.0 for d in diffs)
    assert all(abs(diffs[k + 1]) < abs(diffs[k]) for k in range(len(diffs) - 1))
    assert abs(diffs[-1]) < 1e-2 * max(tail)


@pytest.mark.science_gate(
    scenario="consumer_long_horizon",
    field="liveness_floors",
    quantity="year-end consumer carbon (mol C)",
    bound="non_collapsing(floor=5e-4)",
    source="self — the calibrated attractor, not a cited value",
)
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


@pytest.mark.science_gate(
    scenario="perennial_long_horizon",
    field="liveness_floors",
    quantity="annual minimum chamber CO2 pool (mol C)",
    bound="non_collapsing(floor=0.05)",
    source="self — anchored on the MEASURED trough attractor 0.0732912 "
    "(converged well before yr 50, 1.47x the floor), not on a 15-yr reading; "
    "teeth witnessed by a mutation independent of any candidate science change "
    "(the jar shrunk 0.8x at fixed composition trips it at 0.044941). "
    "Window removed: floor[2:] -> floor",
)
def test_decade_min_carbon_pool_stationary(runs) -> None:
    # Chamber CO2 pool (the producer's only carbon source when sealed): its per-year
    # minimum stays bounded + non-amplifying, and never approaches exhaustion.
    #
    # ⚠ WHAT THIS GUARD ACTUALLY DETECTS — measured, not inherited (2026-08-10). This
    # comment used to say "closure is not slowly draining the atmosphere into biomass".
    # That is false as a description of what the floor catches: the drain mechanism is
    # the recycling loop, and slowing it moves this trough the WRONG WAY — see
    # ``test_the_co2_floor_fires_on_the_buffer_not_on_the_carbon_supply``, which pins
    # that negative result so the next reader does not conclude the guard is toothless
    # from a green bar. What the floor tracks is the chamber's BUFFER against the crop's
    # peak demand: the same ``biosphere.carbon_pool`` the acceptance-gate census found
    # binding in all six sealed scenarios.
    #
    # ⚠ THE ``[_TRANSIENT:]`` WINDOW WAS REMOVED FROM THE FLOOR (2026-08-10), and
    # this is a TIGHTENING: ``non_collapsing(whole)`` implies
    # ``non_collapsing(sliced)``, so the teeth cannot decrease. It was added by the
    # scope-B decomposer calibration under the comment "the year-2 CO2 minimum dips
    # to ~0.039 during soil establishment before settling to ~0.055" — both numbers
    # belong to the PRE-humification-split tree. Measured on the current tree: the
    # whole-run minimum is 0.055175 (year 1) = 1.103x the floor and NO year dips
    # below it, so the slice constrained nothing on the frozen tree. A window that is
    # inert on the reference and load-bearing only on candidates is the one shape a
    # frozen contract's guard must not have.
    #
    # ``transient=_TRANSIENT`` STAYS in the stationarity call, deliberately: its
    # binding same-phase diff (0.013618, 90 % of bound) sits at index 2 and is NOT
    # dropped by the window, so removing it there buys an identical constraint at
    # the cost of the
    # remaining headroom. Inertness justified removing a slice that was hiding a
    # candidate's failure; nothing is hidden behind this one.
    states, _, _ = runs[("perennial", "euler")]
    summaries = year_summaries(states, _YEAR, _min_carbon_pool)
    diffs = same_phase_diffs(summaries, period=2)
    scale = max(summaries)
    assert is_stationary(
        diffs, bound=0.2 * scale, slope_tol=0.02 * scale, transient=_TRANSIENT
    )
    # The floor is anchored on the trough's MEASURED attractor (0.0732912, 1.47x it),
    # not on this horizon's reading — see the beyond-horizon test below, which also
    # pins that the deepest year of a 50-year run lies INSIDE the frozen 15, so this
    # window sees the worst case rather than assuming it.
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


@pytest.mark.slow
def test_the_perennial_decline_has_a_floor_beyond_the_frozen_horizon() -> None:
    """The claim the two restated pins above rest on, measured rather than asserted.

    The humification split leaves the perennial chamber still declining at the frozen
    15-year horizon, which is why those pins had to be restated. The restatement is only
    honest if the decline actually *converges* — a plant walking to zero would also be
    "monotone and decelerating" over a short enough window. So the beyond-horizon run is
    a test, not a paragraph.

    Beyond-horizon is DIAGNOSTIC, never a gate: nothing here is frozen, and the frozen
    contract's horizon is unchanged at 15 years. What this pins is that the attractor
    the 15-yr window is approaching exists, is positive, and is where the liveness floor
    was anchored.
    """
    years = 50
    weather = _weather() * years
    state, registry = build_season(PERENNIAL_CHAMBER_SCENARIO)
    resolver = weather_resolver(weather, PERENNIAL_CHAMBER_SCENARIO)
    states, rationed, events = run_perennial(
        EulerIntegrator(registry),
        state,
        PERENNIAL_CHAMBER_SCENARIO,
        resolver,
        1.0,
        len(weather),
        year=_YEAR,
    )
    assert rationed == 0 and events == ()  # closure holds the whole way there
    summaries = year_summaries(states, _YEAR, _peak_leaf)
    settled = summaries[-5:]
    # Converged: the last five years are the same number to 1e-6.
    assert max(settled) - min(settled) < 1e-6
    assert settled[-1] == pytest.approx(0.594984, abs=1e-5)
    # And that equilibrium is what the 0.55 liveness floor is anchored below.
    assert settled[-1] > 0.55


@pytest.mark.slow
def test_the_chamber_co2_trough_has_an_attractor_beyond_the_frozen_horizon() -> None:
    """The anchor under the 0.05 floor — measured, not read off the frozen horizon.

    The sibling of ``test_the_perennial_decline_has_a_floor_beyond_the_frozen_horizon``,
    written for the same reason and against the same hazard: the humification split
    lengthened the chamber's settling transient to ~35 years, so a bound justified by
    a 15-year reading is a bound justified by a number the tree has not finished
    producing. ``test_decade_min_carbon_pool_stationary``'s floor is anchored here.

    Beyond-horizon is DIAGNOSTIC, never a gate: nothing here is frozen and the frozen
    contract's horizon is unchanged at 15 years.

    Two claims, and the second is the one that licenses the in-horizon guard:

    * the per-year CO2 trough **converges**, to 0.0732912 — 1.47x the floor, so the
      floor is anchored below a measured attractor rather than beside a passing reading;
    * the deepest year of a 50-year run lies **inside** the frozen 15, so the frozen
      window sees the worst case instead of assuming it. Without this the removed
      ``[_TRANSIENT:]`` slice could have been trading one blind spot for another.
    """
    years = 50
    weather = _weather() * years
    state, registry = build_season(PERENNIAL_CHAMBER_SCENARIO)
    resolver = weather_resolver(weather, PERENNIAL_CHAMBER_SCENARIO)
    states, rationed, events = run_perennial(
        EulerIntegrator(registry),
        state,
        PERENNIAL_CHAMBER_SCENARIO,
        resolver,
        1.0,
        len(weather),
        year=_YEAR,
    )
    assert rationed == 0 and events == ()
    summaries = year_summaries(states, _YEAR, _min_carbon_pool)
    settled = summaries[-5:]
    assert max(settled) - min(settled) < 1e-6  # converged
    assert settled[-1] == pytest.approx(0.0732912, abs=1e-6)
    assert settled[-1] / 0.05 > 1.4  # and the floor sits well below the attractor

    # The worst year of the fifty is the sow-in year, INSIDE the frozen horizon. This is
    # what makes the 15-year floor a check on the deepest draw rather than on whichever
    # part of the trajectory the horizon happens to include.
    worst = min(range(len(summaries)), key=lambda i: summaries[i])
    assert worst < DECADE_YEARS
    assert summaries[worst] == pytest.approx(0.055175, rel=1e-3)


@pytest.mark.slow
def test_the_co2_floor_fires_on_the_buffer_not_on_the_carbon_supply() -> None:
    """A committed NEGATIVE result: the two obvious ways to starve the loop fail.

    ``test_decade_min_carbon_pool_stationary``'s floor reads like a carbon-supply
    guard, so the natural way to check its teeth is to cut the supply — start the
    chamber with less CO2, or slow the microbial return that recycles litter back
    into it. **Both make the trough SHALLOWER**, because everything downstream
    self-limits: less carbon reaching the plant grows a smaller plant, which draws
    less. A reader who tried either lever would get a green bar and conclude the
    guard is toothless.

    That is worth a test rather than a paragraph for the reason ``docs/retired/
    mineralization.yaml`` exists: a stale negative result suppresses the next search,
    and a counterintuitive one suppresses it hardest.

    What DOES move the trough is the buffer against peak demand — shrink the jar at
    fixed composition (the consumer-chamber idiom run backwards, so Ci0 and x_O2 are
    invariant and it is a smaller chamber holding the same atmosphere) and the floor
    fires. The
    0.7x case fires **while stationarity passes**, which is the "the level check catches
    what ``is_stationary`` is blind to" claim witnessed by a mutation that is not a
    candidate science change — so the guard's teeth do not rest on the one change its
    verdict is being used to refuse.
    """

    def trough(
        scenario, *, micro_factor: float = 1.0
    ) -> tuple[list[float], bool, bool]:
        weather = _weather() * DECADE_YEARS
        state, registry = build_season(scenario)
        if micro_factor != 1.0:
            flows, hits = [], 0
            for f in registry.flows:
                if isinstance(f, MicrobialRespiration):
                    rate = f.params.microbial_respiration_rate * micro_factor
                    flows.append(
                        dataclasses.replace(
                            f,
                            params=dataclasses.replace(
                                f.params, microbial_respiration_rate=rate
                            ),
                        )
                    )
                    hits += 1
                else:
                    flows.append(f)
            assert hits == 1, "the mutation is a no-op — the probe proves nothing"
            registry = Registry(flows, state.stocks, registry.aux_processes)  # type: ignore[arg-type]
        resolver = weather_resolver(weather, scenario)
        states, _, _ = run_perennial(
            EulerIntegrator(registry),
            state,
            scenario,
            resolver,
            1.0,
            len(weather),
            year=_YEAR,
        )
        summaries = year_summaries(states, _YEAR, _min_carbon_pool)
        scale = max(summaries)
        return (
            summaries,
            non_collapsing(summaries, floor=0.05),
            is_stationary(
                same_phase_diffs(summaries, period=2),
                bound=0.2 * scale,
                slope_tol=0.02 * scale,
                transient=_TRANSIENT,
            ),
        )

    def shrink(factor: float):
        p = PERENNIAL_CHAMBER_SCENARIO
        return dataclasses.replace(
            p,
            chamber_air_mol=p.chamber_air_mol * factor,
            chamber_co2_mol0=p.chamber_co2_mol0 * factor,
            chamber_o2_mol0=p.chamber_o2_mol0 * factor,
        )

    frozen, frozen_floor, _ = trough(PERENNIAL_CHAMBER_SCENARIO)
    assert min(frozen) == pytest.approx(0.055175, rel=1e-3) and frozen_floor

    # (1) Halve the microbial CO2 return — the actual drain mechanism. The trough RISES.
    slow_return, slow_floor, _ = trough(PERENNIAL_CHAMBER_SCENARIO, micro_factor=0.5)
    assert min(slow_return) > min(frozen)
    assert min(slow_return) == pytest.approx(0.057797, rel=1e-3)
    assert slow_floor, "slowing the recycling loop does NOT trip the floor"

    # (2) Start with 20 % less CO2 in the same jar. The trough RISES here too.
    lean = dataclasses.replace(
        PERENNIAL_CHAMBER_SCENARIO,
        chamber_co2_mol0=PERENNIAL_CHAMBER_SCENARIO.chamber_co2_mol0 * 0.8,
    )
    less_co2, less_floor, _ = trough(lean)
    assert min(less_co2) > min(frozen)
    assert min(less_co2) == pytest.approx(0.058757, rel=1e-3)
    assert less_floor, "starting the chamber CO2-poor does NOT trip the floor"

    # (3) The buffer. A 0.8x jar at the same composition trips it...
    small, small_floor, _ = trough(shrink(0.8))
    assert min(small) == pytest.approx(0.044941, rel=1e-3)
    assert not small_floor

    # ...and at 0.7x it trips WHILE STATIONARITY PASSES — a clean attractor in the
    # wrong place, which is exactly the failure ``is_stationary`` cannot see.
    # Witnessed by a jar-size mutation, so the claim is independent of any candidate.
    smaller, smaller_floor, smaller_stationary = trough(shrink(0.7))
    assert min(smaller) == pytest.approx(0.045871, rel=1e-3)
    assert not smaller_floor
    assert smaller_stationary
