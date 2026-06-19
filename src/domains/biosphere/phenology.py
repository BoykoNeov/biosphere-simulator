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
    """

    id: AuxId
    accumulator: str  # the aux name written, e.g. "thermal_time"
    temp_var: str  # forcing var name read via env.get
    params: PhenologyParams

    def evaluate(
        self, snapshot: State, env: Environment, dt: float
    ) -> Mapping[str, float]:
        temp_c = env.get(self.temp_var)
        rate = daily_thermal_time(
            temp_c, t_base=self.params.t_base, t_cap=self.params.t_cap
        )
        return {self.accumulator: rate * dt}
