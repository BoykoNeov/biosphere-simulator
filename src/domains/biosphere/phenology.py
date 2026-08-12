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

**Drought acceleration (Soltani & Sinclair Ch. 15, Eqn 15.8 — the THIRD modifier, and
the odd one out).** ``WSFD = (1 − WSFG)·WSSD + 1`` speeds development up under water
deficit. It joined the other two once the soil water regime was re-based on ``FTSW``
(``docs/plans/post-roadmap-soil-water-rebasing.md``), which is what gave it a ``WSFG``
to be defined through — before that there was no fraction to compute it from. ⚠ It is
**not** a ``[0, 1]`` limitation factor and **not** phase-gated; see
:func:`drought_development_factor` and :class:`ThermalTimeAccumulation`. ⚠ The successor
list that named it called it *"`WSSD` (phenology, 0.40)"* under a heading about
"different **thresholds**" — ``WSSD`` is a **coefficient**, and mis-reading it as a
threshold prices the mechanism as a second ``FTSW`` comparison it does not need
(``docs/plans/post-roadmap-water-stress-curves.md``).

Pure stdlib only. Citations: McMaster, G.S. & Wilhelm, W.W. (1997), "Growing
degree-days: one equation, two interpretations", Agricultural and Forest Meteorology
87:291–300 (the degree-day rate); van Keulen, H. & Wolf, J., eds. (1986), *Modelling of
Agricultural Production: Weather, Soils and Crops*, PUDOC, Wageningen (the
development-stage / thermal-sum (DVS/TSUM) concept).
"""

from collections.abc import Mapping
from dataclasses import dataclass

from domains.biosphere.transpiration import soil_water_stress
from simcore.auxiliary import AuxId
from simcore.environment import Environment
from simcore.ids import StockId
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


@dataclass(frozen=True)
class PhotoperiodParams:
    """Photoperiod (daylength) parameters — Soltani & Sinclair (2012) Ch. 7, Eqn 7.6.

    Wheat is a **long-day** plant: development toward flowering is fastest under long
    days and slows linearly below a critical photoperiod. Like
    :class:`VernalizationParams` this is optional — a day-neutral crop carries none and
    gets the plain degree-day rate byte-for-byte.
    """

    cpp: float  # CPP, critical photoperiod (h); at/above it there is no slowdown
    ppsen: float  # photoperiod sensitivity coefficient (1/h)


@dataclass(frozen=True)
class DroughtDevelopmentParams:
    """Drought-acceleration parameters — Soltani & Sinclair (2012) Ch. 15, Eqn 15.8.

    The third optional development-rate modifier, alongside
    :class:`VernalizationParams` and :class:`PhotoperiodParams`. A crop with no cited
    ``WSSD`` carries none and gets the unmodified rate byte-for-byte.

    ⚠ **Why soil geometry rides along in a phenology dataclass.** ``WSFD`` is not
    defined on ``FTSW`` directly — it is defined on ``WSFG`` (Eqn 15.8), which is
    :func:`transpiration.water_stress_factor` on ``FTSW = ATSW/TTSW``. So the modifier
    cannot be evaluated without the same root-zone geometry the three existing ``WSFG``
    consumers use, and it must use *identical* values or it silently disagrees with them
    about the stress state inside one step. The alternative — a second, independently
    parameterized ``FTSW`` — is precisely the disagreement
    :func:`transpiration.soil_water_stress` exists to prevent. ``RootDepthExtension``
    carries the same three fields flat for the same reason.

    ⚠ **``wssd`` is a COEFFICIENT, not a threshold**, and the distinction is easy to
    lose because its two table-mates are thresholds. Table 15.1's caption: "Threshold
    FTSW for leaf area development (WSSL) and growth (WSSG), **and a coefficient of
    phenological development response to drought (WSSD)**". There is no ``FTSW``
    comparison against
    ``wssd``; it scales an already-computed ``WSFG``.
    """

    wssd: float  # WSSD, the drought development-response coefficient (dimensionless)
    wssg: float  # WSSG, the growth FTSW threshold WSFD is defined through (Eqn 15.3)
    soil_extractable_water: float  # EXTR (mm mm⁻¹), for TTSW = DEPORT · EXTR
    ground_area: float  # m², the scenario footprint TTSW is expressed over


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


def photoperiod_factor(daylength_h: float, *, cpp: float, ppsen: float) -> float:
    """Development-rate multiplier ``ppfun ∈ [0, 1]`` — Soltani & Sinclair Eqn 7.6.

    The **long-day** form (wheat, barley, oat, rye, rapeseed):
    ``ppfun = 1 − ppsen·(CPP − PP)`` below the critical photoperiod ``cpp`` and **1**
    at/above it, **clamped to [0, 1]** — the source is explicit that a negative value is
    replaced by zero, "because phenological development is only a forward process and
    cannot be negative".

    Unlike :func:`vernalization_factor` this reads an *instantaneous* driver, not an
    accumulator: photoperiod has **no memory**, so the factor rises and falls with the
    season rather than saturating once. That difference is what distinguishes the two
    mechanisms in the oracle trajectory (see
    ``docs/plans/post-roadmap-oracle-match.md``): a saturating factor cannot reproduce
    a multiplier that keeps climbing after the cold requirement is met.

    Raises ``ValueError`` unless ``cpp > 0`` and ``ppsen >= 0`` (a negative sensitivity
    would make short days *accelerate* a long-day plant).
    """
    if not cpp > 0.0:
        raise ValueError(f"cpp must be > 0, got {cpp!r}")
    if ppsen < 0.0:
        raise ValueError(f"ppsen must be >= 0, got {ppsen!r}")
    if daylength_h >= cpp:
        return 1.0
    return max(0.0, min(1.0, 1.0 - ppsen * (cpp - daylength_h)))


def drought_development_factor(wsfg: float, *, wssd: float) -> float:
    """Development-rate multiplier ``WSFD`` — Soltani & Sinclair Eqn 15.8.

    ``WSFD = (1 − WSFG)·WSSD + 1``, where ``WSFG`` is the growth/transpiration deficit
    factor (:func:`transpiration.water_stress_factor`, Eqn 15.3). Unstressed
    (``WSFG = 1``) gives **exactly 1.0** — the identity that keeps every
    non-water-limited scenario byte-for-byte unchanged. Fully stressed (``WSFG = 0``)
    gives ``1 + WSSD``, so ``WSSD`` reads directly as the maximum fractional change in
    development rate: [F]'s own worked example is "if WSSD is 0.4, the maximum value of
    WSFD at WSFG = 0 is equal to 1.4".

    ⚠ **This is the first modifier in this module that may exceed 1**, and that is the
    citation, not a slip: drought *hastens* development in most species (Table 15.2 —
    "acceleration of development rates is more common"). ``verfun`` and ``ppfun`` are
    limitation factors on ``[0, 1]``; this one is a ratio on ``[0, 1 + WSSD]``.

    **Negative ``WSSD`` is [F]'s own provision** for species drought *delays*: "Eqn 15.8
    can still be used with negative values for WSSD. For example, if WSSD is −0.4, then
    WSFD will be 0.6 when FTSW and hence WSFG reach 0." At ``WSSD = −1`` development is
    fully arrested under maximum stress.

    Raises ``ValueError`` unless ``wssd >= -1`` — below that ``WSFD`` goes negative and
    development would run *backwards*, which the source rules out in the same words it
    uses for photoperiod ("phenological development is only a forward process and cannot
    be negative"). A cited bound, not a defensive clamp: the arrest at ``−1`` is the
    physical limit of the form.
    """
    if wssd < -1.0:
        raise ValueError(
            f"wssd must be >= -1 (development is forward-only), got {wssd!r}"
        )
    return (1.0 - wsfg) * wssd + 1.0


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

    **Drought acceleration (optional; the THIRD modifier).** When ``drought``,
    ``soil_water`` and ``rooted_depth_aux`` are all supplied, the rate is additionally
    scaled by :func:`drought_development_factor` ([F] Eqn 15.8), evaluated on the
    ``WSFG`` the rest of the tree computes from the step-entry ``soil_water`` amount and
    step-entry rooted depth. ⚠ **It differs from its two neighbours in both directions,
    and both differences are the source's:**

    * **It is NOT phase-gated.** ``verfun``/``ppfun`` sit inside the ``DVS < 1`` branch
      because wheat is insensitive to cold and daylength past anthesis. [F] Box 16.2
      gates ``WSFD`` on ``CTU > tuEMR`` *only* — after emergence, all the way to
      maturity — and this accumulator **starts** at emergence
      (``thermal_time = 0 ⇒ DVS = 0``), so the gate is satisfied by construction and the
      factor must run through grain filling too.
    * **It may exceed 1.** Drought hastens development; see
      :func:`drought_development_factor`.

    It is applied **last**, after both vegetative modifiers, mirroring Box 16.2's
    ``DTU = (TP1D - TBD) * tempfun`` … ``DTU = DTU * WSFD`` — ``WSFD`` scales the
    fully-modified daily temperature unit, not the bare degree-day rate.
    """

    id: AuxId
    accumulator: str  # the aux name written, e.g. "thermal_time"
    temp_var: str  # forcing var name read via env.get
    params: PhenologyParams
    vernalization: VernalizationParams | None = None
    vernalization_accumulator: str | None = None  # the aux name read, e.g. "vern_days"
    photoperiod: PhotoperiodParams | None = None
    daylength_var: str | None = None  # forcing var name read, e.g. "daylength_s"
    drought: DroughtDevelopmentParams | None = None
    soil_water: StockId | None = None  # the sibling stock read for ATSW
    rooted_depth_aux: str | None = (
        None  # the aux name read for DEPORT, e.g. "rooted_depth"
    )

    def evaluate(
        self, snapshot: State, env: Environment, dt: float
    ) -> Mapping[str, float]:
        temp_c = env.get(self.temp_var)
        rate = daily_thermal_time(
            temp_c, t_base=self.params.t_base, t_cap=self.params.t_cap
        )
        if self._is_vegetative(snapshot):
            # Both modifiers are vegetative-only and MULTIPLY (the source's Eqn 7.4
            # "biological day": BD = tempfun · ppfun, extended by Eqn 8.2's verfun).
            # Either may be absent; each contributes 1 when it is.
            rate *= self._vernalization_factor(snapshot)
            rate *= self._photoperiod_factor(env)
        # Applied OUTSIDE the vegetative branch and LAST — [F] gates WSFD on emergence
        # only (Box 16.2) and applies it to the already-modified DTU.
        rate *= self._drought_factor(snapshot)
        return {self.accumulator: rate * dt}

    def _is_vegetative(self, snapshot: State) -> bool:
        """``DVS < 1``. Both modifiers are gated here — wheat is insensitive to cold
        and to daylength at/after anthesis, so past it the plain rate is exact."""
        return (
            development_stage(
                snapshot.aux.get(self.accumulator, 0.0),
                tsum_anthesis=self.params.tsum_anthesis,
                tsum_maturity=self.params.tsum_maturity,
            )
            < 1.0
        )

    def _vernalization_factor(self, snapshot: State) -> float:
        """The Eqn-8.6 multiplier — 1 when vernalization is not configured."""
        if self.vernalization is None or self.vernalization_accumulator is None:
            return 1.0
        return vernalization_factor(
            snapshot.aux.get(self.vernalization_accumulator, 0.0),
            vsen=self.vernalization.vsen,
            vdsat=self.vernalization.vdsat,
        )

    def _photoperiod_factor(self, env: Environment) -> float:
        """The Eqn-7.6 multiplier — 1 when photoperiod is not configured.

        Reads the daylength forcing in **seconds** (the repo's canonical
        ``daylength_s``, produced by ``weather.daylength_seconds``) and converts to the
        hours the source's ``CPP``/``ppsen`` are expressed in.
        """
        if self.photoperiod is None or self.daylength_var is None:
            return 1.0
        return photoperiod_factor(
            env.get(self.daylength_var) / 3600.0,
            cpp=self.photoperiod.cpp,
            ppsen=self.photoperiod.ppsen,
        )

    def _drought_factor(self, snapshot: State) -> float:
        """The Eqn-15.8 multiplier — 1 when drought acceleration is not configured.

        Reads ``soil_water`` off ``snapshot.stocks`` and the rooted depth off
        ``snapshot.aux`` — the same two step-entry reads ``RootDepthExtension`` makes,
        routed through the same :func:`transpiration.soil_water_stress` the three
        existing consumers use. Sharing that one function is what stops this fourth
        consumer from disagreeing with the other three about ``FTSW`` inside a step.
        """
        if (
            self.drought is None
            or self.soil_water is None
            or self.rooted_depth_aux is None
        ):
            return 1.0
        wsfg = soil_water_stress(
            snapshot.stocks[self.soil_water].amount,
            snapshot.aux.get(self.rooted_depth_aux, 0.0),
            soil_extractable_water=self.drought.soil_extractable_water,
            ground_area=self.drought.ground_area,
            threshold=self.drought.wssg,
        )
        return drought_development_factor(wsfg, wssd=self.drought.wssd)


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
