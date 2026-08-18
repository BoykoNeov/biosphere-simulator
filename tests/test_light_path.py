"""The within-day light path, and the night gas exchange it makes reachable.

Two layers, mirroring ``test_gas_exchange.py``:

* **Path-level** — the two window means are exact partitions of the day they shape, so a
  day's photon dose is conserved **at any step size**, and a window in the dark returns
  exactly 0. These are the properties the whole change rests on: the daily dose being
  conserved is what makes this a redistribution rather than a recalibration, and the
  exact zero is what opens the night branch.
* **Integration (the sealed season)** — the observable the charge is about, which **no
  test in this tree asserted before**: the chamber's CO₂ rises through the night and
  falls through the day, with O₂ its exact mirror at PQ=1. Plus the three verifications
  the plan's stage 1 owes now that ``MaintenanceRespiration``'s biomass-burning branch
  actually runs (it never did before): the shortfall is positive in the dark and zero in
  full light, and the inherited ``f_O2`` deferral still holds.

⚠ **What these tests deliberately do NOT assert.** They do not claim the canopy the
light
path produces is right — the converged peak LAI sits *below* the frozen band's floor,
and
that finding is argued in ``docs/plans/post-roadmap-gross-net-gas-exchange.md`` (finding
14) and carried as a documented allowance in ``docs/biosphere-reference.md``, not
papered
over here.

Pure-stdlib data path (reads the committed JSON weather fixture; no PCSE).
"""

import json
import math

import pytest

from config.paths import WINTER_WHEAT_WEATHER
from domains.biosphere.chamber import oxygen_limitation_factor
from domains.biosphere.light_path import (
    SECONDS_PER_DAY,
    half_sine_window_mean,
    top_hat_window_mean,
)
from domains.biosphere.loader import load_respiration_params
from domains.biosphere.scenario import SEALED_CHAMBER_SCENARIO
from domains.biosphere.season import (
    build_season,
    run_season,
    weather_resolver,
)
from domains.biosphere.step import BIO_DT, STEPS_PER_DAY, steps_for
from domains.biosphere.stocks import CARBON_POOL, O2_POOL, PAR_VAR
from simcore.integrator import EulerIntegrator

_WEATHER = WINTER_WHEAT_WEATHER


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER.read_text(encoding="utf-8"))["weather"]


# --- the path itself ---------------------------------------------------------------
@pytest.mark.parametrize("steps_per_day", [1, 2, 4, 8, 32, 96])
@pytest.mark.parametrize("daylength_h", [0.0, 4.0, 9.5, 12.0, 16.5, 24.0])
def test_the_day_dose_is_conserved_at_every_step_size(
    steps_per_day: int, daylength_h: float
) -> None:
    """⚠ THE LOAD-BEARING PROPERTY: the light path redistributes, it never recalibrates.

    Σ over the day of ``window_mean · dt`` must equal the flat daytime mean × the
    daylight fraction — the same photon dose ``weather.incident_par`` already computes.
    If this failed the change would be a silent re-scaling of IRRAD, and the golden diff
    would be un-attributable. It holds *at any step* because the window means are an
    exact partition of one integral, which is precisely why the instantaneous sampling
    of the same sinusoid was refused (it holds only in the limit).
    """
    dt = 1.0 / steps_per_day
    mean_par = 400.0
    daylength_s = daylength_h * 3600.0
    dose = sum(
        half_sine_window_mean(k * dt, dt, mean_par, daylength_s) * dt
        for k in range(steps_per_day)
    )
    expected = mean_par * daylength_s / SECONDS_PER_DAY
    assert dose == pytest.approx(expected, rel=1e-12, abs=1e-12)


@pytest.mark.parametrize("steps_per_day", [1, 4, 8, 32])
@pytest.mark.parametrize("photoperiod_h", [0.0, 8.0, 16.0, 24.0])
def test_the_lamps_dose_is_conserved_at_every_step_size(
    steps_per_day: int, photoperiod_h: float
) -> None:
    """The same property for the lamp's top-hat — the grow-lamp seams' half.

    Before the light path the lit chamber got its dose from ``PAR × photoperiod`` as a
    *multiplier on the daily total*; now the photoperiod is the shape of the day. The
    dose is the same number either way, which is what this pins.
    """
    dt = 1.0 / steps_per_day
    on_par = 800.0
    photoperiod_s = photoperiod_h * 3600.0
    dose = sum(
        top_hat_window_mean(k * dt, dt, on_par, photoperiod_s) * dt
        for k in range(steps_per_day)
    )
    assert dose == pytest.approx(on_par * photoperiod_s / SECONDS_PER_DAY, rel=1e-12)


def test_a_window_in_the_dark_is_EXACTLY_zero_not_merely_small() -> None:
    """Exactly 0 — the night branch is gated on an inequality, and it must be crossed.

    ``shortfall = MRES − GASS > 0`` needs GASS to actually reach 0 at midnight (a tiny
    residual PAR would keep FvCB's light term alive and the branch shut in the very
    scenarios the charge is about). ``==`` rather than ``approx`` is the point.
    """
    daylength_s = 12.0 * 3600.0  # sunrise 0.25, sunset 0.75
    assert half_sine_window_mean(0.0, 0.125, 400.0, daylength_s) == 0.0
    assert half_sine_window_mean(0.875, 0.125, 400.0, daylength_s) == 0.0
    assert top_hat_window_mean(0.0, 0.125, 400.0, daylength_s) == 0.0
    # and the lit half is not zero, so the assertion above is not vacuous
    assert half_sine_window_mean(0.375, 0.125, 400.0, daylength_s) > 0.0


def test_polar_day_and_polar_night_are_both_expressible() -> None:
    """The geometry already clamps to 24 h / 0 h (FAO-56); the path must not raise
    there.

    ``weather.daylength_seconds`` clamps its ``arccos`` for polar latitudes, so the two
    degenerate days reach this module and must give a full day of light and none at all.
    """
    assert half_sine_window_mean(0.0, 0.25, 400.0, 0.0) == 0.0
    full = sum(
        half_sine_window_mean(k * 0.25, 0.25, 400.0, SECONDS_PER_DAY) * 0.25
        for k in range(4)
    )
    assert full == pytest.approx(400.0, rel=1e-12)


def test_a_window_that_crosses_midnight_is_a_build_bug_not_a_silent_answer() -> None:
    """The daily tables are one row per physical day, so a straddling window has no
    single day's weather to read. ``step.py`` guarantees it cannot happen; this pins
    that
    the guard is loud rather than quietly answering from the wrong day."""
    with pytest.raises(ValueError, match="within one day"):
        half_sine_window_mean(0.9, 0.25, 400.0, 43200.0)
    with pytest.raises(ValueError, match="dt must be > 0"):
        half_sine_window_mean(0.0, 0.0, 400.0, 43200.0)


def test_the_peak_is_the_pi_over_two_multiple_of_the_daytime_mean() -> None:
    """The one constant in the form, checked against its own derivation.

    A half-sine integrates to ``(2/π)·peak·D``, so ``peak = (π/2)·mean`` is what
    conserves the dose. Measured at solar noon with a window short enough to be
    effectively instantaneous.
    """
    daylength_s = 12.0 * 3600.0
    noon = half_sine_window_mean(0.5 - 1e-6, 2e-6, 400.0, daylength_s)
    assert noon == pytest.approx((math.pi / 2.0) * 400.0, rel=1e-9)


# --- the season: the observable the charge is about --------------------------------
def _sealed_run(dt: float = BIO_DT, steps_per_day: int = STEPS_PER_DAY):
    weather = _weather()
    state, registry = build_season(SEALED_CHAMBER_SCENARIO)
    resolver = weather_resolver(weather, SEALED_CHAMBER_SCENARIO)
    states, rationed, events = run_season(
        EulerIntegrator(registry), state, resolver, dt, steps_for(len(weather))
    )
    assert rationed == 0
    assert events == ()
    return states, resolver


def test_the_sealed_chamber_BREATHES_co2_up_in_the_dark_and_down_in_the_light() -> None:
    """⚠ THE CHARGE'S OWN OBSERVABLE, and nothing in this tree asserted it before.

    "the plants MUST emit oxygen at least minute by minute and consume co2." Over the
    season, every step whose PAR is 0 must move the chamber's CO₂ **up**, and the
    brightest step of each day must move it **down**. Before the light path the pool
    fell
    monotonically within every day of the growing season, because gross assimilation
    could not reach zero.
    """
    states, resolver = _sealed_run()
    par = resolver.forcings[PAR_VAR]
    co2 = [s.stocks[CARBON_POOL].amount for s in states]
    dark_up = dark = 0
    for n in range(len(states) - 1):
        if par(n, BIO_DT) == 0.0:
            dark += 1
            dark_up += co2[n + 1] > co2[n]
    assert dark > 0, "no dark step in the season — the path is not shaping the day"
    assert dark_up == dark, f"{dark - dark_up} of {dark} dark steps did not raise CO2"
    # and the brightest step of a mid-season day draws it down again
    day = range(200 * STEPS_PER_DAY, 201 * STEPS_PER_DAY)
    lit = max(day, key=lambda n: par(n, BIO_DT))
    assert co2[lit + 1] < co2[lit]


def test_oxygen_is_the_exact_mirror_of_carbon_through_the_night() -> None:
    """PQ = 1, step for step, in the dark — the respiration half of the gas loop.

    The daylight half was already pinned (``test_gas_exchange``); this is the branch
    that
    could not fire before. Every mol of carbon returned to the pool consumes one mol of
    O₂, so the two deltas sum to zero exactly (the same ``organ_burn`` sum sources both
    legs).
    """
    states, resolver = _sealed_run()
    par = resolver.forcings[PAR_VAR]
    checked = 0
    for n in range(len(states) - 1):
        if par(n, BIO_DT) != 0.0:
            continue
        c0, c1 = (states[i].stocks[CARBON_POOL].amount for i in (n, n + 1))
        o0, o1 = (states[i].stocks[O2_POOL].amount for i in (n, n + 1))
        assert (c1 - c0) + (o1 - o0) == pytest.approx(0.0, abs=1e-15)
        checked += 1
    assert checked > 100


def test_the_night_branch_is_what_moved_and_the_daylight_branch_is_untouched() -> None:
    """The gate is ``shortfall = MRES − GASS > 0`` — dark steps cross it, bright ones do
    not.

    ⚠ This is the finding-5 gate measured on the running tree rather than argued: before
    the light path the shortfall was identically zero at **every step of every
    scenario**
    (daily GASS exceeded MRES 20–200×). ⚠ And note what it is NOT gated on: a *fully
    dark* step. A dim dawn step crosses it too, which is why the chamber breathes even
    at
    a step size that has no dark window at midsummer.
    """
    states, resolver = _sealed_run()
    par = resolver.forcings[PAR_VAR]
    co2 = [s.stocks[CARBON_POOL].amount for s in states]
    # the brightest step of a mid-season day: assimilation dominates, pool falls
    day = range(200 * STEPS_PER_DAY, 201 * STEPS_PER_DAY)
    brightest = max(day, key=lambda n: par(n, BIO_DT))
    assert par(brightest, BIO_DT) > 0.0
    assert co2[brightest + 1] < co2[brightest]
    # a step at the same day's darkest PAR: respiration dominates, pool rises
    darkest = min(day, key=lambda n: par(n, BIO_DT))
    assert co2[darkest + 1] > co2[darkest]


def test_the_f_O2_throttle_bites_and_that_is_NOT_the_light_paths_doing() -> None:
    """⚠ THE INHERITED CLAIM IS FALSE, AND IT WAS FALSE BEFORE THIS CHANGE.

    ``MaintenanceRespiration``'s docstring recorded "at the PP fill it is ≈ 1", written
    about a branch that never executed. It executes now, so the throttle was measured —
    and its season minimum is **0.854**, a 15 % suppression of the night burn, not ≈ 1.

    ⚠ **The control says it is not ours.** The same measurement on the committed tree at
    ``82d965c`` gives **0.847** — very slightly *worse*. The dip is a property of the
    sealed chamber's provisioned O₂ fill (the minimum lands at the START of the season;
    the pool then *rises* 2.0 → 2.245 mol as the crop out-produces its own respiration),
    so what the light path did was expose a stale prose claim, not create a limitation.
    Recorded rather than fixed by re-wording alone: the docstring is corrected too.

    What this pins is the property that actually matters — the throttle stays far from
    shutoff, so the night branch is O₂-limited only marginally and the season's diurnal
    swing is respiration's own, not the throttle's.
    """
    states, _ = _sealed_run()
    resp = load_respiration_params()
    air_mol = SEALED_CHAMBER_SCENARIO.chamber_air_mol
    assert air_mol is not None
    worst = min(
        oxygen_limitation_factor(
            s.stocks[O2_POOL].amount, air_mol=air_mol, k_o2=resp.o2_half_saturation
        )
        for s in states
    )
    # ⚠ 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor): 0.8541 ->
    # 0.8776. The worst oxygen limitation EASES — a smaller-mass crop respires less at
    # the fill, so the O2 pool is drawn down less hard. Moving AWAY from shutoff.
    assert 0.86 < worst < 0.89, worst  # measured 0.8776 (was 0.8541 / 0.8466)
    # far from shutoff, and rising: the fill is the worst point, not the season's end
    final = oxygen_limitation_factor(
        states[-1].stocks[O2_POOL].amount, air_mol=air_mol, k_o2=resp.o2_half_saturation
    )
    assert final > worst
