"""The within-day light path (post-roadmap, 2026-08-14): PAR varies inside the day.

Before this module the model's only statement about the sun setting was the factor
``× daylength_s`` inside :func:`photosynthesis.daily_canopy_assimilation`: one
daytime-mean PAR, one multiplication, one lump of carbon per step. That form cannot
express night, and **night is the thing the charge is about** — a real plant consumes O₂
and returns CO₂ to the air in the dark. In this tree the mechanism that does that is
already built, already cited and already conservation-balanced
(:class:`carbon_budget.MaintenanceRespiration`'s biomass-sourced *shortfall*), and it
can never fire, because a day-averaged PAR makes ``GASS > MRES`` at every step of every
scenario. **The gate is the forcing, not a missing flow.**

**The path is cited, and so is the direction of its effect.** [E] (Penning de Vries,
Jansen, Ten Berge & Bakema, *Simulation of Ecophysiological Processes of Growth in
Several Annual Crops*), on the canopy-photosynthesis scheme it adopts from Goudriaan /
Spitters: *"The path of radiation intensity during the day is assumed to be
sinusoidal."* And its exercise answer T6, on what that does to the daily total:
*"The increase in canopy photosynthesis per unit increase of radiation decreases
continuously. **Splitting an amount of radiation in unequal portions over a day leads to
a lower daily total.**"* Our constant-mean-PAR evaluation is the *equal portions* case,
so assimilation is expected to fall — the same concavity (Jensen) bias that
``daily_canopy_assimilation``'s own docstring has confessed since Step 5.

**The window mean, and why it is not the instantaneous value (MEASURED, 2026-08-14).**
A forcing schedule is a function of the integer step (``simcore.environment.Schedule``,
decision #14) and is piecewise-constant across a step. There are two ways to hand a
sinusoid to such a schedule, and they are not close at the step this project ships:

* **instantaneous** — sample the sine at the step-entry instant ``t = n·dt``. Sampling
  luck decides the day's carbon. Measured against today's daily total, at ``dt = ¼``
  day and peak LAI: **0.916** at a 13.3 h day but **1.039** at a 16.5 h day — an 8 %
  loss and a 4 % *gain* from the same form, and at ``dt = 1`` it returns **exactly
  zero** (the one sample lands at midnight). The sign is not even stable, so a golden
  diff would record the sampling grid rather than the science.
* **window mean** (this module) — the analytic mean of the sine over ``[t, t+dt)``.
  The day's photon dose is conserved **exactly, at any step size**, because the window
  means are an exact partition of the same integral; a step that is half night carries
  half the light. Measured at ``dt = ¼`` and peak LAI: **0.9935–0.9965**, converging
  monotonically to the 60 s answer **0.9899** as the step shrinks.

The window mean is therefore what this module supplies. It is a *first-order-in-dt*
rendering of the cited sinusoidal path (it converges to it), and it keeps the golden
diff attributable to the form rather than to the grid. ⚠ It also means **the Jensen
correction is step-limited**: at ``dt = ¼`` roughly half of the converged 1.0 % loss at
peak canopy is resolved, and the rest arrives only at a finer step. That is a statement
about the step, not about the science, and it must not be reported as "the cited
direction did not appear".

**⚠ What the night branch actually needs is a DIM step, not a dark one — and the two
were confused here for an afternoon.** The gate is ``shortfall = MRES − GASS > 0``, so
dawn and dusk steps cross it as readily as midnight ones. A count of *fully dark* steps
is therefore a lower bound on when the mechanism runs, never a measure of it: over the
committed 305-day fixture at latitude 52°, ``dt = ¼`` has no fully-dark step at all on
133 days — and the sealed chamber still breathes on those days, with a diurnal CO₂ swing
within 1 % of the ``dt = 1/32`` answer. What the finer step buys is the **canopy**, not
the breathing: peak LAI is still moving 15 % between ``¼`` and ``1/32``. Measured in
``docs/plans/post-roadmap-gross-net-gas-exchange.md``, findings 13–15.

**Phase convention.** ``t`` is a fraction of the day with ``t = 0`` at midnight and
solar noon at ``t = 0.5``; the daylight span is centred there
(``sunrise = ½ − D/2``, ``sunset = ½ + D/2`` with ``D`` the daylength in days). Both
paths share it, so a lamp photoperiod and an orbital day are centred alike and a
scenario that switches between them moves only the shape.

Pure stdlib only. Citations:
  * Penning de Vries, F.W.T., Jansen, D.M., ten Berge, H.F.M. & Bakema, A. (1989),
    "Simulation of Ecophysiological Processes of Growth in Several Annual Crops",
    Pudoc/IRRI — the sinusoidal radiation path and the T6 direction ([E]).
  * The daylight geometry that supplies ``daylength_s`` is FAO-56 eq. 24/25/34, cited
    in :func:`weather.daylength_seconds`.
"""

import math

SECONDS_PER_DAY: float = 86400.0


def _daylight_span(daylength_s: float) -> tuple[float, float]:
    """``(sunrise, sunset)`` as fractions of the day, centred on solar noon (½).

    Raises on a negative or super-diurnal daylength (a meaningless geometry); a
    daylength of exactly 0 (polar night) and of exactly one day (polar day) are both
    legal and give an empty / full span.
    """
    if daylength_s < 0.0 or daylength_s > SECONDS_PER_DAY:
        raise ValueError(
            f"daylength_s must be within [0, {SECONDS_PER_DAY}] s, got {daylength_s!r}"
        )
    half = daylength_s / SECONDS_PER_DAY / 2.0
    return 0.5 - half, 0.5 + half


def _check_window(t0: float, dt: float) -> None:
    """The window must lie inside one day — the schedule's own precondition.

    A window that crossed midnight would need two days' weather rows, and the daily
    tables are one row per physical day (``season._table``). ``domains.biosphere.step``
    already guarantees an integer number of steps per day at a negative power of two, so
    this is a build-bug guard, not a runtime condition.
    """
    if dt <= 0.0:
        raise ValueError(f"dt must be > 0 days, got {dt!r}")
    if t0 < 0.0 or t0 + dt > 1.0 + 1e-12:
        raise ValueError(
            f"light-path window [{t0!r}, {t0 + dt!r}) must lie within one day; the "
            "step must divide the day (see domains.biosphere.step)"
        )


def half_sine_window_mean(
    t0: float, dt: float, daytime_mean_par: float, daylength_s: float
) -> float:
    """Mean PAR over ``[t0, t0+dt)`` of the sinusoidal day ([E]) — µmol m⁻² s⁻¹.

    The path is ``PAR(t) = peak · sin(π·(t − sunrise)/D)`` over the daylight span and 0
    outside it, with ``peak = (π/2)·daytime_mean_par`` — the value that makes the day's
    photon dose **identical to the flat daytime mean it replaces**, since a half-sine
    integrates to ``(2/π)·peak·D``. So no parameter is introduced, no radiation is
    created, and ``weather.incident_par`` needs no recalibration: only the distribution
    within the day changes.

    The mean is the exact integral over the window divided by the **full** window length
    (not by the daylight overlap): a step that is half night carries half the light,
    which is what makes the day's dose conserved when the window means are summed.

    ``t0`` and ``dt`` are fractions of a day (``t0`` measured from midnight). Returns 0
    for a window wholly in the dark, and for a zero daylength or zero mean PAR.
    """
    _check_window(t0, dt)
    if daytime_mean_par < 0.0:
        raise ValueError(f"daytime_mean_par must be >= 0, got {daytime_mean_par!r}")
    sunrise, sunset = _daylight_span(daylength_s)
    lo = max(t0, sunrise)
    hi = min(t0 + dt, sunset)
    if hi <= lo or daytime_mean_par == 0.0:
        return 0.0
    span = sunset - sunrise  # daylength as a fraction of the day
    peak = (math.pi / 2.0) * daytime_mean_par
    # ∫ sin(π(t−s)/D) dt = (D/π)·[cos(π(a−s)/D) − cos(π(b−s)/D)]
    integral = (span / math.pi) * (
        math.cos(math.pi * (lo - sunrise) / span)
        - math.cos(math.pi * (hi - sunrise) / span)
    )
    return peak * integral / dt


def top_hat_window_mean(
    t0: float, dt: float, on_par: float, photoperiod_s: float
) -> float:
    """Mean PAR over ``[t0, t0+dt)`` of a lamp's on/off day — µmol m⁻² s⁻¹.

    The grow-lamp path: a constant ``on_par`` inside a photoperiod window centred on
    solar noon, 0 outside it. Like the sinusoid, the day's dose is conserved exactly
    (``on_par · photoperiod_s``) — this replaces the ``× photoperiod`` multiplication
    the lit chamber used to get from ``DAYLENGTH_VAR``, moving the photoperiod from a
    *multiplier on the daily total* to a *shape within the day*, which is what makes the
    lamp's dark hours real dark hours the plant respires through.

    ⚠ The lamp is a **top-hat, not a sinusoid**, and that is physical rather than a
    simplification: a lamp on a timer has no diurnal intensity path. A scenario wanting
    a dimming ramp is the authoring platform's ``table`` kind, not this function.
    """
    _check_window(t0, dt)
    if on_par < 0.0:
        raise ValueError(f"on_par must be >= 0, got {on_par!r}")
    sunrise, sunset = _daylight_span(photoperiod_s)
    lo = max(t0, sunrise)
    hi = min(t0 + dt, sunset)
    if hi <= lo:
        return 0.0
    return on_par * (hi - lo) / dt
