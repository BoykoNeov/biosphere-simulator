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

import pytest

from config.paths import WINTER_WHEAT_WEATHER
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
from domains.biosphere.step import BIO_DT, steps_for
from simcore.boundary import loss_sink_id
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import BALANCE_ATOL, Quantity
from simcore.registry import Registry
from simcore.state import State

_WEATHER_FIXTURE = WINTER_WHEAT_WEATHER


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


_YEAR_DAYS = len(_weather())  # season length in DAYS (the tiling + reset period, ~305)
# ...and in integration steps. ⚠ The two are separate names deliberately: the weather
# table is one row per DAY, while ``year_summaries`` and the reset period index the
# STEP-indexed trajectory. They coincide only while the step is a day.
_YEAR = steps_for(_YEAR_DAYS)
# The budgeted 15-yr working horizon (>= the decade-scale 10-yr target), shared as the
# single source of truth with the long-horizon golden + the freeze manifest
# (scenario.py).
DECADE_YEARS = LONG_HORIZON_YEARS
_STEPS = _YEAR * DECADE_YEARS
_QUANTITIES = (Quantity.CARBON, Quantity.OXYGEN, Quantity.NITROGEN, Quantity.WATER)
# ⚠ 2 -> 3 on 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor). The
# settling transient lengthened by one same-phase step: the consumer chamber's third
# diff is −0.09337 against a bound of 0.09124 (RK4), i.e. over by 2.3 %.
# ⚠ This is a TRANSIENT LENGTH, not a tolerance — the claim being made is "the
# same-phase differences stop amplifying and trend to zero", and they do: the series
# runs −0.097, −0.124, −0.090, −0.065, −0.048, −0.035, … −0.005, monotonically decaying
# after its peak on BOTH integrators. Nothing was widened to accommodate it.
# ⚠ And Euler was passing at 2 by 3 % (−0.09027 against 0.09313), so the two integrators
# sat either side of a knife edge on a value neither had a principled claim to. The
# monotone-decay assertion added below is the durable form of the claim; the transient
# is just where we start reading it.
_TRANSIENT = 3  # same-phase diffs to drop before the non-amplifying trend (the sow-in)
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
        BIO_DT,
        steps_for(len(weather)),
        year=_YEAR,  # already steps
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


# ⚠ The ``@pytest.mark.science_gate`` marker was REMOVED here in slice C4 of the
# reference flip (2026-08-18). The claim did not go away and was not weakened — it
# moved to the reference, where the roster row and the test that executes it are ONE
# declaration: rust/crates/domains/src/biosphere/science_gates.rs::perennial_decade_le
# af_cycle_is_stationary_and_alive +
# consumer_decade_leaf_cycle_is_stationary_and_alive (TWO markers on this one
# parametrized test; in the reference the row IS the test, so it is two tests with two
# loci). The biosphere manifest’s science_bands / liveness_floors are generated from
# there, so this function is the CHECKER’s copy of the assertion and no longer the
# contract’s locus. Deleting it is Stage 3’s call, not a free consequence of C4.
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


# ⚠ The ``@pytest.mark.science_gate`` marker was REMOVED here in slice C4 of the
# reference flip (2026-08-18). The claim did not go away and was not weakened — it
# moved to the reference, where the roster row and the test that executes it are ONE
# declaration: science_gates.rs::perennial_leaf_cycle_is_a_fixed_point. The biosphere
# manifest’s science_bands / liveness_floors are generated from there, so this
# function is the CHECKER’s copy of the assertion and no longer the contract’s locus.
# Deleting it is Stage 3’s call, not a free consequence of C4.
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


# ⚠ The ``@pytest.mark.science_gate`` marker was REMOVED here in slice C4 of the
# reference flip (2026-08-18). The claim did not go away and was not weakened — it
# moved to the reference, where the roster row and the test that executes it are ONE
# declaration: science_gates.rs::decade_consumer_biomass_is_stationary_and_alive. The
# biosphere manifest’s science_bands / liveness_floors are generated from there, so
# this function is the CHECKER’s copy of the assertion and no longer the contract’s
# locus. Deleting it is Stage 3’s call, not a free consequence of C4.
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


# ⚠ The ``@pytest.mark.science_gate`` marker was REMOVED here in slice C4 of the
# reference flip (2026-08-18). The claim did not go away and was not weakened — it
# moved to the reference, where the roster row and the test that executes it are ONE
# declaration: science_gates.rs::decade_min_carbon_pool_stationary. The biosphere
# manifest’s science_bands / liveness_floors are generated from there, so this
# function is the CHECKER’s copy of the assertion and no longer the contract’s locus.
# Deleting it is Stage 3’s call, not a free consequence of C4.
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
    # whole-run minimum is 0.0754757 (year 1) = 1.510x the floor and NO year dips
    # below it, so the slice constrained nothing on the frozen tree. A window that is
    # inert on the reference and load-bearing only on candidates is the one shape a
    # frozen contract's guard must not have. (0.055175 / 1.103x pre-step-unfreeze.)
    #
    # ⚠⚠ **THE ARGUMENT FOR KEEPING ``transient=_TRANSIENT`` IN THE STATIONARITY CALL
    # INVERTED ON 2026-08-14 (the step unfreeze), and the code is UNCHANGED while the
    # reasoning is replaced.** It used to read: "its binding same-phase diff (0.013618,
    # 90 % of bound) sits at index 2 and is NOT dropped by the window, so removing it
    # buys an identical constraint at the cost of the remaining headroom."
    #
    # Re-measured at dt = 1/4, both halves of that are false. The binding diff is now
    # -0.00072449 at **index 0** — which the window DOES drop — and it is **4.8 % of
    # bound**, not 90 %. The largest surviving diff is 0.00024763 at index 3, 1.6 % of
    # bound. So the window is no longer inert here, and the headroom it was protecting
    # is no longer scarce: removing it would TIGHTEN the check and still pass with 20x
    # to spare.
    #
    # It stays anyway, on its original merit rather than on the retired arithmetic: the
    # dropped diff is the sow-in settling, which is what a transient window is for, and
    # ``_TRANSIENT`` is shared with the two sibling stationarity gates. Recorded rather
    # than acted on, so the next reader decides with the measurement in hand instead of
    # inheriting a justification that stopped being true.
    states, _, _ = runs[("perennial", "euler")]
    summaries = year_summaries(states, _YEAR, _min_carbon_pool)
    diffs = same_phase_diffs(summaries, period=2)
    scale = max(summaries)
    assert is_stationary(
        diffs, bound=0.2 * scale, slope_tol=0.02 * scale, transient=_TRANSIENT
    )
    # The floor is anchored on the trough's MEASURED attractor (0.0758448, 1.52x it —
    # it was 0.0732912 / 1.47x), not on this horizon's reading — see the beyond-horizon
    # test below, which also pins that the deepest year of a 50-year run lies INSIDE
    # the frozen 15, so this window sees the worst case rather than assuming it.
    #
    # ⚠ 2026-08-14, the step unfreeze: the trough series rose ~35 % and the floor did
    # NOT follow it. That is deliberate — this guard's justification prose is inside
    # the biosphere manifest, and re-anchoring 0.05 upward every time the reference
    # moves is how a floor becomes a restatement of the current run. It is recorded
    # instead: the gap the floor has to cross widened from 1.10x to 1.51x of the
    # whole-run minimum, and the mutation needed to trip it grew from a 20 % jar
    # shrink to a 35 % one.
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
        # ⚠ The claim `_TRANSIENT` is a reading-offset INTO, asserted directly so that a
        # future lengthening of the transient cannot be absorbed by nudging the offset
        # again: past its peak, the same-phase differences shrink in magnitude every
        # single step, on both integrators. That is settling, and it does not depend on
        # where one starts reading.
        tail = diffs[_TRANSIENT:]
        assert all(abs(b) < abs(a) for a, b in zip(tail, tail[1:], strict=False)), diffs
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
        BIO_DT,
        steps_for(len(weather)),
        year=_YEAR,  # already steps
    )
    assert rationed == 0 and events == ()  # closure holds the whole way there
    summaries = year_summaries(states, _YEAR, _peak_leaf)
    settled = summaries[-5:]
    # Converged: the last five years are the same number to 1e-6.
    # ⚠ 2026-08-12 (stem reserves): the absolute spread over the last five years is
    # 1.2169e-6, just over the old 1e-6. Re-expressed as a RELATIVE bound rather than
    # nudged: 'flat to 5 parts per million' is a claim about convergence that does not
    # silently change meaning when the level moves, which is exactly how this pin came
    # to be marginal. The bound is TIGHTER than the old one in relative terms
    # (1e-6/0.594984 = 1.7e-6) only if the level falls; it is stated so the reader can
    # see which.
    assert (max(settled) - min(settled)) / settled[-1] < 5e-6
    # 0.594984 -> 0.593864 (the build) -> 0.593883 (its cessation window) -> this
    # (2026-08-14, the step unfreeze: dt = 1/4). A 2.8 % fall, and it is a genuine
    # re-integration of the same attractor rather than a rescaling — this summary is a
    # STOCK LEVEL in mol C, so unlike a census margin it carries no per-step
    # denominator and nothing about it should have moved by a factor of the step.
    assert settled[-1] == pytest.approx(
        0.543748,
        abs=1e-5,  # ⚠ 2026-08-15 canopy 0.567715 -> 0.543748
    )  # ⚠ 2026-08-14 (light path), was 0.577062
    # And that equilibrium is what the 0.55 liveness floor is anchored below. ⚠ The
    # headroom NARROWED, and by more than the level moved: 0.593883 sat 8.0 % above the
    # floor, 0.577062 sits 4.9 % above it. Recorded, not re-anchored — moving 0.55
    # because the level moved is the co-adaptation this project refuses. It is worth
    # watching: two more moves of this size and the floor stops being a floor.
    #
    # ⚠⚠ **AND IT DID — 2026-08-15, ON THE VERY NEXT MOVE.** The line above predicted
    # "two more moves of this size"; one sufficed. The 50-year attractor is now
    # **0.543748, BELOW the 0.55 floor it is supposed to sit above.** The assertion is
    # INVERTED to say what is true rather than deleted, because what it measures is a
    # fact about the tree and the fact changed.
    #
    # ⚠ **What this does and does NOT mean.** It is NOT a gate failure: this run is
    # 50 years and the frozen contract's horizon is 15, where all four liveness floors
    # still pass (measured 2026-08-15, `test_perennial_leaf_cycle_is_a_fixed_point` and
    # the three `test_decade_*` gates, all green). What it means is that **the 15-year
    # gate now passes because 15 years is short of convergence, not because the tree
    # settles above the floor** — the anchor the floor was placed under has crossed it.
    # A floor whose own anchor is below it is no longer measuring what it was built to
    # measure.
    #
    # ⚠ The cause is a UNIT confusion waiting to happen, and worth stating plainly: this
    # summary is peak leaf **CARBON (mol C)**, not leaf AREA. The same change that took
    # `open_season`'s peak LAI UP 12 % (a bigger canopy) takes leaf carbon DOWN, because
    # binding `specific_leaf_area` to its source made each mol of leaf carbon buy 7 %
    # more area — so the plant reaches its canopy on less carbon. More leaf area and
    # less leaf mass are the same event. See `docs/log/canopy-magnitude.md`.
    #
    # ⇒ NOT re-anchored here. Moving 0.55 to fit is the refused co-adaptation, and
    # choosing between re-anchoring, accepting a shorter-lived perennial, and treating
    # the drift as a defect is a science call that belongs upstream of a test file.
    assert settled[-1] < 0.55, "the attractor is now BELOW the floor — see above"
    assert settled[-1] > 0.50, settled[-1]  # ...but still well clear of collapse


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
        BIO_DT,
        steps_for(len(weather)),
        year=_YEAR,  # already steps
    )
    assert rationed == 0 and events == ()
    summaries = year_summaries(states, _YEAR, _min_carbon_pool)
    settled = summaries[-5:]
    assert max(settled) - min(settled) < 1e-6  # converged
    # ⚠ 0.0732912 -> 0.0736507 -> 0.0736681 (2026-08-12: the stem-reserve build, then
    # its cessation window) -> this (2026-08-14, the step unfreeze). The CLAIM — the
    # trough reaches an attractor comfortably above the 0.05 floor — is unchanged and
    # re-run each time; the ratio assertion below carries it and has gone 1.47x ->
    # 1.52x across the step change, i.e. the trough moved AWAY from the floor.
    assert settled[-1] == pytest.approx(
        0.072238,
        abs=1e-6,  # ⚠ 2026-08-15 canopy 0.073326 -> 0.072238
    )  # ⚠ 2026-08-14 (light path), was 0.0758448
    assert settled[-1] / 0.05 > 1.4  # and the floor sits well below the attractor

    # The worst year of the fifty is the sow-in year, INSIDE the frozen horizon. This is
    # what makes the 15-year floor a check on the deepest draw rather than on whichever
    # part of the trajectory the horizon happens to include.
    worst = min(range(len(summaries)), key=lambda i: summaries[i])
    assert worst < DECADE_YEARS
    # ⚠ 0.055175 -> 0.0559766 (2026-08-12, stem reserves) -> this (2026-08-14, the step
    # unfreeze). ⚠⚠ +34.9 %, the largest single move this number has made, and it is
    # what re-priced the whole CO2-floor probe below: the entire per-year trough series
    # rose by about a third, so the absolute 0.05 floor is a third further away than it
    # was. Measured index of the worst year is 1 (0.0754757, vs year 0's 0.0762602).
    assert summaries[worst] == pytest.approx(
        0.070253,
        rel=1e-3,  # ⚠ 2026-08-15 canopy 0.070492 -> 0.070253
    )  # ⚠ 2026-08-14 (light path), was 0.0754757


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
    fires **while stationarity passes**, which is the "the level check catches
    what ``is_stationary`` is blind to" claim witnessed by a mutation that is not a
    candidate science change — so the guard's teeth do not rest on the one change its
    verdict is being used to refuse.

    ⚠⚠ **RE-RUN 2026-08-14 (the step unfreeze), NOT re-tuned, and both halves moved.**
    At ``dt = 1/4`` the whole per-year trough series sits ~35 % higher (the frozen run's
    minimum went 0.0559766 -> 0.0754757), so the absolute 0.05 floor is a third further
    away and the old shrink factors no longer reach it: 0.8x now lands at 0.0600926 and
    0.7x at 0.0528486, **both passing**. The question this probe exists to answer is not
    "what do the old factors read now" but "does any factor still trip the floor while
    stationarity passes", so it was answered by sweeping the factor and leaving the
    floor alone:

        0.80x 0.0600926 pass   0.68x 0.0514049 pass
        0.70x 0.0528486 pass   0.65x 0.0492366 TRIP   0.60x 0.0456069 TRIP
                               0.62x 0.0470611 TRIP   0.55x 0.0419631 TRIP

    **The guard keeps its teeth, and they are blunter.** The crossing moved from above
    0.8x to between 0.68x and 0.65x: the jar must now be shrunk by about a third where a
    fifth used to do it. Stationarity passes at *every* factor swept, including all four
    that trip — so the "``is_stationary`` is blind to this" claim is now witnessed more
    broadly than it was, not less. The two rungs below were re-chosen to bracket the
    re-measured crossing, which is the probe's independent variable; the 0.05 floor is
    untouched, and lowering it to keep 0.7x red would have been the fitted cut this file
    refuses elsewhere in exactly these words.
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
            BIO_DT,
            steps_for(len(weather)),
            year=_YEAR,  # already steps
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
    # ⚠ 0.055175 -> 0.0559766 (2026-08-12, stem reserves) -> this (2026-08-14, dt=1/4).
    # ⚠ and again 2026-08-14 (the light path): 0.0754757 -> 0.0704924.
    assert (
        min(frozen) == pytest.approx(0.070253, rel=1e-3) and frozen_floor
    )  # ⚠ 2026-08-15 canopy 0.0704924 -> 0.070253

    # (1) Halve the microbial CO2 return — the actual drain mechanism. The trough RISES.
    slow_return, slow_floor, _ = trough(PERENNIAL_CHAMBER_SCENARIO, micro_factor=0.5)
    assert min(slow_return) > min(frozen)
    # ⚠ 0.057797 -> 0.0608579 (2026-08-12, stem reserves) -> this (2026-08-14, dt=1/4).
    # The CLAIM — halving the microbial return RAISES the trough, so the floor fires on
    # the buffer and not on the carbon supply — is asserted on the line above and is
    # unchanged through both moves. ⚠ Its MAGNITUDE collapsed, though: the gap was 8.7 %
    # of the trough and is now 0.7 %. Probe 1 still points the same way; it points much
    # more faintly, for the same reason probe 2 below flipped back.
    assert min(slow_return) == pytest.approx(
        0.071129,
        rel=1e-3,  # ⚠ 2026-08-15 canopy 0.071864 -> 0.071129
    )  # ⚠ 2026-08-14 (light path), was 0.0760228
    assert slow_floor, "slowing the recycling loop does NOT trip the floor"

    # (2) Start with 20 % less CO2 in the same jar. The trough RISES here too.
    lean = dataclasses.replace(
        PERENNIAL_CHAMBER_SCENARIO,
        chamber_co2_mol0=PERENNIAL_CHAMBER_SCENARIO.chamber_co2_mol0 * 0.8,
    )
    less_co2, less_floor, _ = trough(lean)
    # ⚠⚠ **PROBE 2'S SIGN HAS FLIPPED TWICE IN THREE DAYS, AND THAT IS THE FINDING.**
    #   2026-08-11 and earlier  ``>``  0.058757 — CO2-poor RAISES the trough (+5.0 %).
    #   2026-08-12 (stem build) ``<``  0.0552887 against 0.0559766 (-1.2 %).
    #   2026-08-14 (dt = 1/4)   ``>``  0.0755337 against 0.0754757 (+0.077 %).
    #
    # It is back to the sign it was born with, at a fifteenth of the magnitude it
    # inverted at, under a change that is not about carbon supply at all. Read together
    # with probe 1's collapse from 8.7 % to 0.7 %, the two supply-side probes are now
    # both an order of magnitude fainter than they were: at a quarter-day step the
    # chamber's CO2 trough is close to indifferent to how much carbon is in the loop.
    #
    # ⚠ THE CONCLUSION HOLDS AND IS STRONGER THAN IT WAS. The original claim was "the
    # floor fires on the buffer, not on the carbon supply"; 2026-08-12 weakened it to
    # "carbon supply has a small, correctly-signed influence". The influence has now
    # shrunk to 0.08 % and cannot hold a sign across a step change, so it is back to
    # being noise about a mechanism rather than a mechanism. The ORDERING is asserted
    # below because it is what actually got measured — but the magnitude bound is the
    # line that carries the argument, and it is tightened from 2 % to 0.5 % to match.
    assert min(less_co2) > min(frozen), "probe 2 flipped back — see the note"
    assert min(less_co2) == pytest.approx(
        0.070345,
        rel=1e-3,  # ⚠ 2026-08-15 canopy 0.070616 -> 0.070345
    )  # ⚠ 2026-08-14 (light path), was 0.0755337
    assert abs(min(less_co2) - min(frozen)) / min(frozen) < 0.005, "and it is small"
    assert less_floor, "starting the chamber CO2-poor still does NOT trip the floor"

    # (3) The buffer. ⚠ THE FACTORS ARE 0.65x / 0.60x, WERE 0.8x / 0.7x — see the
    # docstring's sweep. They were re-chosen to bracket the crossing after it was
    # MEASURED to have moved (0.7x and even 0.68x now pass), not adjusted until the
    # assertions went green, and the 0.05 floor did not move.
    #
    # First rung: just past the crossing, which is between 0.68x (0.0514049, passes)
    # and here.
    small, small_floor, small_stationary = trough(shrink(0.65))
    # was 0.044941 (2026-08-12) -> 0.0481100 (its cessation window), both at 0.8x
    assert min(small) == pytest.approx(
        0.045342,
        rel=1e-3,  # ⚠ 2026-08-15 canopy 0.045217 -> 0.045342
    )  # ⚠ 2026-08-14 (light path), was 0.0492366
    assert not small_floor

    # ...and clear of it at 0.60x, WHILE STATIONARITY PASSES — a clean attractor in the
    # wrong place, which is exactly the failure ``is_stationary`` cannot see.
    # Witnessed by a jar-size mutation, so the claim is independent of any candidate.
    smaller, smaller_floor, smaller_stationary = trough(shrink(0.60))
    # was 0.045871 -> 0.0477959 (2026-08-12, stem reserves), both at 0.7x
    assert min(smaller) == pytest.approx(
        0.041853,
        rel=1e-3,  # ⚠ 2026-08-15 canopy 0.041755 -> 0.041853
    )  # ⚠ 2026-08-14 (light path), was 0.0456069
    assert not smaller_floor
    assert smaller_stationary
    # ⚠ STRENGTHENED 2026-08-14: stationarity is now asserted on BOTH tripping rungs,
    # not just the deeper one. The sweep found it true at every factor it visited, so
    # the "the level check sees what is_stationary cannot" claim is not a property of
    # one hand-picked jar size — and pinning it twice is what would catch it becoming
    # one.
    assert small_stationary
