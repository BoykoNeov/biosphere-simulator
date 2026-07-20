"""Thermal-time phenology (Phase-1 Step 8; the first non-conserved aux process).

The first real consumer of the Step-2 auxiliary channel (P2). Crop development is
driven by accumulated temperature ("thermal time", °C·day), which *evolves* but is
**not** a conserved quantity — it has no balanced counterparty, so it cannot ride the
flow → reduce → apply path or pass the conservation gate. It is therefore neither a
``Flow`` (no balanced leg) nor a Step-4-style pure diagnostic (it accumulates): it is
exactly the :class:`simcore.auxiliary.AuxProcess` that P2 was built for.

* **Thermal time is the one accumulator.** :class:`ThermalTimeAccumulation` writes the
  single accumulator name ``thermal_time`` in *increment form*
  (``{name: daily_thermal_time(T)·dt}``, like a ``Flow``'s ``dt·rate``), advanced by
  one explicit-Euler evaluation at the step-entry snapshot and carried unchanged across
  RK4 stages (P2/P3). The rate is dt-independent, so the accumulation is dt-linear.

* **Development stage (DVS) is derived, NOT stored** (the P2 lock —
  "``DVS = f(thermal_time)``"). :func:`development_stage` is a pure function computed on
  demand by consumers (Step 9 allocation; the Step-6 maintenance ``maturity`` seam),
  never integrated. Resisting a second "DVS accumulator for symmetry" is the phenology
  analogue of Step 4's "LAI is derived, not an aux" — so the channel stays the one
  accumulator P2 minimized to.

**Degree-day rate (cardinal-cap form; McMaster & Wilhelm 1997).**
:func:`daily_thermal_time` is 0 at/below a base temperature ``t_base``, the linear
``T − t_base`` between, and capped at ``t_cap − t_base`` at/above an upper cap ``t_cap``
— the growing-degree-day form with an upper cap (the WOFOST ``DTSMTB`` idiom). Monotone
non-decreasing in ``T`` and bounded.

**Development stage (DVS; the WOFOST ``TSUM1``/``TSUM2`` two-phase idiom).**
:func:`development_stage` ramps DVS linearly 0 → 1 over the vegetative thermal-time sum
``tsum_anthesis`` (emergence → anthesis), then 1 → 2 over the reproductive sum
``tsum_maturity`` (anthesis → maturity), capped at 2.0. Stage points: 0 emergence,
1 anthesis/flowering, 2 maturity. The accumulator starts at emergence
(``thermal_time = 0 ⇒ DVS = 0``); the sowing→emergence sub-phase (``TSUMEM``) and *when*
the accumulator starts/resets are scenario concerns deferred to the Step-11 season.

This raw-accumulator + derived-piecewise-DVS form is mathematically **equivalent** to
WOFOST's phase-wise DVS integration **because the base/cap response is phase-invariant**
(the same daily °C·day rate feeds both phases, normalized by ``TSUM1`` vs ``TSUM2``).

**Deferred refinements (documented Step-11 seams, like Step 5's Arrhenius / Step 6's
maturity).** Winter wheat genuinely needs both, and a plain degree-day model overruns
development through a mild winter — but they are *structurally different* deferrals:

* **Photoperiod** is a pure astronomical function (latitude + day-of-year) read via
  ``env.get`` — a development-rate multiplier with **no accumulator**; it slots onto
  :func:`daily_thermal_time` as a factor (the FvCB ``f_temp`` shape).
* **Vernalization** (the cold requirement) is WOFOST-style a **second state
  accumulator** (vernalization-days) with a derived ``VERNFAC ∈ [0, 1]`` that
  down-scales the rate only in the vegetative phase. A second accumulator is an
  **extension** of the channel, not a violation: P2 says *essentially* one accumulator
  and names the channel "non-conserved scalar accumulator**s**" (plural). The
  ``evaluate(snapshot, env, dt)`` signature already carries the snapshot, so a future
  vernalization-aware rate can read ``snapshot.aux["thermal_time"]``, derive the current
  DVS, and gate ``VERNFAC`` to the vegetative phase with no API change — the seam exists
  structurally even though the plain rate here does not read ``snapshot``.

Pure stdlib only. Citations: McMaster, G.S. & Wilhelm, W.W. (1997), "Growing
degree-days: one equation, two interpretations", Agricultural and Forest Meteorology
87:291–300 (the degree-day rate); van Keulen, H. & Wolf, J., eds. (1986), *Modelling of
Agricultural Production: Weather, Soils and Crops*, PUDOC, Wageningen (the
development-stage / thermal-sum (DVS/TSUM) concept).
"""

from collections.abc import Mapping
from dataclasses import dataclass

from simcore.auxiliary import AuxId
from simcore.environment import Environment
from simcore.state import State


@dataclass(frozen=True)
class PhenologyParams:
    """Loader-produced thermal-time phenology parameters in core-ready form.

    Mirrors ``TranspirationParams``/``RespirationParams``: declared data, no magic
    numbers in the physics. All values are provisional literature-typical placeholders
    pending the Step-11 validation gate (see ``params/phenology.yaml``).
    """

    t_base: float  # base temperature for development (°C; no degree-days below it)
    t_cap: float  # upper cap temperature (°C; degree-days plateau at/above it)
    tsum_anthesis: float  # TSUM1, thermal time emergence → anthesis (°C·day)
    tsum_maturity: float  # TSUM2, thermal time anthesis → maturity (°C·day)


@dataclass(frozen=True)
class VernalizationParams:
    """Cold-requirement (vernalization) parameters — Soltani & Sinclair (2012) Ch. 8.

    Separate from :class:`PhenologyParams` because vernalization is **optional**: a crop
    without a cold requirement carries none, and a scenario that supplies none gets the
    plain degree-day rate byte-for-byte (see :class:`ThermalTimeAccumulation`). The four
    cardinal temperatures parameterize the Eqn-8.3 response; ``vsen``/``vdsat``
    parameterize the Eqn-8.6 saturation curve.
    """

    t_base_v: float  # TBV, base temperature for vernalization (°C)
    t_opt_lower_v: float  # TP1V, lower optimum (°C; full effect at/above)
    t_opt_upper_v: float  # TP2V, upper optimum (°C; full effect at/below)
    t_ceiling_v: float  # TCV, ceiling temperature (°C; no effect at/above)
    vsen: float  # sensitivity coefficient of development rate to vernalization (1/day)
    vdsat: float  # VDSAT, vernalization days that saturate the response (day)


def daily_thermal_time(temp_c: float, *, t_base: float, t_cap: float) -> float:
    """Daily thermal-time increment (°C·day per day) — the cardinal-cap GDD rate.

    The growing-degree-day rate with an upper cap (McMaster & Wilhelm 1997; the WOFOST
    ``DTSMTB`` idiom): **0** at/below ``t_base``, the linear ``temp_c − t_base`` on
    ``(t_base, t_cap)``, and the plateau ``t_cap − t_base`` at/above ``t_cap``. Monotone
    non-decreasing in ``temp_c`` and bounded above. Raises ``ValueError`` unless
    ``t_base < t_cap`` (a non-positive band has no valid ramp).
    """
    if not t_base < t_cap:
        raise ValueError(f"require t_base < t_cap, got ({t_base!r}, {t_cap!r})")
    if temp_c <= t_base:
        return 0.0
    if temp_c >= t_cap:
        return t_cap - t_base
    return temp_c - t_base


def vernalization_day(
    temp_c: float,
    *,
    t_base_v: float,
    t_opt_lower_v: float,
    t_opt_upper_v: float,
    t_ceiling_v: float,
) -> float:
    """Vernalization days per calendar day (day/day) — Soltani & Sinclair Eqn 8.3.

    The 3-segment linear cold response with four cardinal temperatures (base ``TBV``,
    lower optimum ``TP1V``, upper optimum ``TP2V``, ceiling ``TCV``): **0** at/below
    ``t_base_v``, a linear ramp up to **1** at ``t_opt_lower_v``, the full-effect
    plateau **1** across the optimum band, a linear ramp back down to **0** at
    ``t_ceiling_v``, and **0** at/above it. Bounded in ``[0, 1]`` and unimodal in
    ``temp_c`` — the inverted-plateau mirror of :func:`daily_thermal_time`'s monotone
    cap.

    Raises ``ValueError`` unless the four cardinals are non-decreasing with a strictly
    positive ramp on each side (``t_base_v < t_opt_lower_v`` and
    ``t_opt_upper_v < t_ceiling_v``) — both are divisors below — and the optimum band is
    well-ordered (``t_opt_lower_v <= t_opt_upper_v``).
    """
    if not t_base_v < t_opt_lower_v:
        raise ValueError(
            f"require t_base_v < t_opt_lower_v, got ({t_base_v!r}, {t_opt_lower_v!r})"
        )
    if not t_opt_lower_v <= t_opt_upper_v:
        raise ValueError(
            "require t_opt_lower_v <= t_opt_upper_v, got "
            f"({t_opt_lower_v!r}, {t_opt_upper_v!r})"
        )
    if not t_opt_upper_v < t_ceiling_v:
        raise ValueError(
            "require t_opt_upper_v < t_ceiling_v, got "
            f"({t_opt_upper_v!r}, {t_ceiling_v!r})"
        )
    if temp_c <= t_base_v or temp_c >= t_ceiling_v:
        return 0.0
    if temp_c < t_opt_lower_v:
        return (temp_c - t_base_v) / (t_opt_lower_v - t_base_v)
    if temp_c <= t_opt_upper_v:
        return 1.0
    return (t_ceiling_v - temp_c) / (t_ceiling_v - t_opt_upper_v)


def vernalization_factor(
    vernalization_days: float, *, vsen: float, vdsat: float
) -> float:
    """Development-rate multiplier ``verfun ∈ [0, 1]`` — Soltani & Sinclair Eqn 8.6.

    ``verfun = 1 − vsen·(vdsat − CUMVER)`` while cumulative vernalization days are below
    the saturation requirement ``vdsat``, and **1** at/above it, **clamped to [0, 1]**.

    The clamp is load-bearing, not defensive. With the cited winter-wheat values
    (``vsen = 0.033``, ``vdsat = 50``) the unclamped expression is ``−0.65`` at
    ``CUMVER = 0``: winter-Europe wheat is **qualitative** in the source's own
    terminology (Fig. 8.2) — development is *fully arrested* until ~19.7 vernalization
    days accumulate, rather than merely slowed. That arrest is a property of the cited
    parameterization, not a modeling choice here. A *quantitative* cultivar
    (``vsen·vdsat < 1``) never reaches the clamp.

    Raises ``ValueError`` unless ``vdsat > 0`` (a zero requirement has no curve) and
    ``vsen >= 0`` (a negative sensitivity would make cold *retard* development).
    """
    if not vdsat > 0.0:
        raise ValueError(f"vdsat must be > 0, got {vdsat!r}")
    if vsen < 0.0:
        raise ValueError(f"vsen must be >= 0, got {vsen!r}")
    if vernalization_days >= vdsat:
        return 1.0
    return max(0.0, min(1.0, 1.0 - vsen * (vdsat - vernalization_days)))


def development_stage(
    thermal_time: float, *, tsum_anthesis: float, tsum_maturity: float
) -> float:
    """Development stage ``DVS ∈ [0, 2]`` derived from thermal time (TSUM1/TSUM2 idiom).

    Two phase-linear ramps (van Keulen & Wolf 1986): ``DVS = tt / tsum_anthesis`` over
    ``[0, tsum_anthesis]`` (vegetative, emergence → anthesis, ending at DVS = 1), then
    ``DVS = 1 + (tt − tsum_anthesis) / tsum_maturity`` (reproductive, anthesis →
    maturity), **capped at 2.0**. Derived, never stored (the P2 lock). Raises
    ``ValueError`` unless both thermal sums are strictly positive (they are divisors).
    """
    if not tsum_anthesis > 0.0:
        raise ValueError(f"tsum_anthesis must be > 0, got {tsum_anthesis!r}")
    if not tsum_maturity > 0.0:
        raise ValueError(f"tsum_maturity must be > 0, got {tsum_maturity!r}")
    if thermal_time <= 0.0:
        return 0.0
    if thermal_time < tsum_anthesis:
        return thermal_time / tsum_anthesis
    reproductive = 1.0 + (thermal_time - tsum_anthesis) / tsum_maturity
    return min(2.0, reproductive)


@dataclass(frozen=True)
class ThermalTimeAccumulation:
    """``AuxProcess`` advancing the ``thermal_time`` accumulator (P2; the first one).

    Reads air temperature as a scalar driver through ``env.get`` (forcing or shared
    stock — the process cannot tell, #16; daily-mean temperature at the daily step) and
    returns the per-step increment ``{accumulator: daily_thermal_time(T)·dt}`` in the
    increment form the integrator advances by explicit Euler at the step-entry snapshot
    (carried unchanged across RK4 stages, P2/P3). The plain rate depends only on the
    forced temperature, so ``snapshot`` is unread here — but the signature carries it so
    a deferred vernalization-aware rate can read ``snapshot.aux["thermal_time"]``
    without an API change (the documented Step-11 seam).

    **Vernalization (optional; the seam above, taken).** When ``vernalization`` and
    ``vernalization_accumulator`` are both supplied, the degree-day rate is scaled by
    the Eqn-8.6 factor :func:`vernalization_factor`, read off the *second* accumulator
    on ``snapshot.aux`` — the deferred read this signature was kept for. The factor
    applies **only in the vegetative phase** (``DVS < 1``); at/after anthesis it is
    fixed at 1, per the source (wheat is insensitive beyond anthesis). Supplying neither
    leaves the plain degree-day rate **byte-for-byte unchanged**, which is what keeps a
    crop with no cold requirement — and every pre-vernalization scenario — exactly as it
    was.

    The factor scales the *thermal-time increment* rather than a DVS rate because our
    DVS is derived from ``thermal_time`` rather than integrated (the P2 lock); scaling
    the increment scales DVS's rate of advance identically. That is a faithful
    re-expression of the source's Eqn 8.2, recorded because the two forms are not
    obviously the same.
    """

    id: AuxId
    accumulator: str  # the aux name written, e.g. "thermal_time"
    temp_var: str  # forcing var name read via env.get
    params: PhenologyParams
    vernalization: VernalizationParams | None = None
    vernalization_accumulator: str | None = None  # the aux name read, e.g. "vern_days"

    def evaluate(
        self, snapshot: State, env: Environment, dt: float
    ) -> Mapping[str, float]:
        temp_c = env.get(self.temp_var)
        rate = daily_thermal_time(
            temp_c, t_base=self.params.t_base, t_cap=self.params.t_cap
        )
        rate *= self._vernalization_factor(snapshot)
        return {self.accumulator: rate * dt}

    def _vernalization_factor(self, snapshot: State) -> float:
        """The Eqn-8.6 multiplier: 1 unless configured *and* still vegetative."""
        if self.vernalization is None or self.vernalization_accumulator is None:
            return 1.0
        dvs = development_stage(
            snapshot.aux.get(self.accumulator, 0.0),
            tsum_anthesis=self.params.tsum_anthesis,
            tsum_maturity=self.params.tsum_maturity,
        )
        if dvs >= 1.0:  # insensitive at/after anthesis — the factor is fixed at 1
            return 1.0
        return vernalization_factor(
            snapshot.aux.get(self.vernalization_accumulator, 0.0),
            vsen=self.vernalization.vsen,
            vdsat=self.vernalization.vdsat,
        )


@dataclass(frozen=True)
class VernalizationAccumulation:
    """``AuxProcess`` advancing the ``vernalization_days`` accumulator (the second one).

    The exact structural mirror of :class:`ThermalTimeAccumulation`: it reads
    temperature through ``env.get`` (#16) and returns the per-step increment
    ``{accumulator: vernalization_day(T)·dt}`` in increment form. P2 names the channel
    "non-conserved scalar accumulator**s**" (plural), so this is an **extension** of the
    channel rather than a violation of the single-accumulator minimization — the same
    argument the deferral in this module's docstring made in advance.

    **Crown temperature.** The source prescribes crown temperature ``Tcr`` (the
    growing point sits below the soil surface) and notes soil-surface temperature *is*
    similar to air temperature except where **snow cover** makes them diverge. Air
    temperature is read here because no snow/precipitation forcing exists to represent
    the divergence — a documented simplification, not an oversight.

    **De-vernalization is not implemented.** The source's Eqn 8.5 reduces the
    accumulator when it is below 10 days and daily **maximum** temperature exceeds 30
    °C. The forcing carries daily-*mean* temperature only, so the term is
    *unimplementable* rather than merely omitted — and inert on the committed weather
    besides, whose seasonal maximum daily mean is 22.2 °C.
    """

    id: AuxId
    accumulator: str  # the aux name written, e.g. "vernalization_days"
    temp_var: str  # forcing var name read via env.get
    params: VernalizationParams

    def evaluate(
        self, snapshot: State, env: Environment, dt: float
    ) -> Mapping[str, float]:
        rate = vernalization_day(
            env.get(self.temp_var),
            t_base_v=self.params.t_base_v,
            t_opt_lower_v=self.params.t_opt_lower_v,
            t_opt_upper_v=self.params.t_opt_upper_v,
            t_ceiling_v=self.params.t_ceiling_v,
        )
        return {self.accumulator: rate * dt}
