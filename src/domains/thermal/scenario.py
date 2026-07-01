"""The Thermal scenario — initial heat, the forced load, the step (Step 5).

The Thermal analogue of ``domains.power.scenario``: pure scenario **data** (not the
radiative coefficients — those are ``params/radiator.yaml`` via ``loader.py``). Every
field is sizing / forcing wiring for the standalone equilibrium-temperature validation
run that ``domains.thermal.system`` assembles.

**The load is NOT derived (unlike Power) — the radiator is the restoring force.**
Power's two flows were both *forced*, so its SOC was a restoring-force-free accumulator
and a *bounded* SOC had to be *constructed* by an exactly-balanced derived load
(``balanced_load_w``). Thermal is different: the radiator ``RadiatorReject`` is
donor-controlled and monotone in temperature, so **any** constant ``heat_load_w`` yields
a **unique, stable emergent equilibrium temperature** ``T_eq`` where ``ε·σ·A·(T_eq⁴ −
T_space⁴) = heat_load_w``. So the forced load is plain scenario data (an amplitude, like
Power's ``solar_peak_w``), **not** derived — ``system.thermal_resolver`` is a simple
constant, with no coefficient coupling.

**The load-bearing sizing constraint: ``τ >> dt``.** The nonlinear radiator's positivity
near equilibrium is a *sizing* property, not structural (radiated ∝ ``T⁴``, not ∝
``Q``): Euler must not overshoot. The relaxation time ``τ = C / (4·ε·σ·A·T_eq³)`` (``C``
= ``heat_capacity``, a ``radiator.yaml`` param) must be many steps. With the committed
params (ε 0.85, A 10 m², T_space 2.7 K, C 1e7 J/K) and the defaults below, ``T_eq ≈
280.9 K`` and ``τ ≈ 2.34e5 s ≈ 65 steps`` of ``dt_seconds = 3600`` — comfortably ``>>
dt`` (a step rejects ≈ 0.4 % of the stored heat at equilibrium, far from overshoot), so
``rationed == 0`` holds by sizing (the Power ``LoadDraw`` well-fed discipline). See
``system.relaxation_time`` / ``system.equilibrium_temperature``.

**The node starts cold (``node0 = 0`` ⇒ ``T = T_space``) and warms to ``T_eq``.** A
clean emergent-attractor demonstration: the sensible heat rises monotonically from the
floor (where the radiator emits exactly 0) until rejection balances input. The
equilibrium heat is ``Q_eq = C·(T_eq − T_space) ≈ 2.78e9 J``.

**Time unit is seconds (SI), documented not implicit** — energy J, power W = J/s, as for
Power. There is **no diurnal structure** (constant forced load ⇒ monotone relaxation),
so — unlike Power — there is no ``steps_per_day`` / half-sine schedule; the horizon is a
plain step count (``EQUILIBRIUM_STEPS``) sized to reach equilibrium (~11 τ).

Pure stdlib only (a frozen dataclass).
"""

from dataclasses import dataclass


@dataclass(frozen=True)
class ThermalScenario:
    """Scenario data (not the radiator params): initial heat, forced load, the step.

    Defaults are an illustrative small-station thermal loop (not a calibrated system —
    Phase 5 ships machinery, not validated numbers, the biosphere/Power ``TODO(cite)``
    precedent). ``heat_load_w`` is the forcing amplitude; ``node0`` the initial sensible
    heat; ``dt_seconds`` the step. The equilibrium temperature is *emergent* from these
    plus the ``radiator.yaml`` params (see ``system.equilibrium_temperature``).
    """

    # Initial sensible heat Q = C·(T − T_space) (J). 0.0 ⇒ the node starts at T_space
    # (the radiator floor, emitting exactly 0) and warms monotonically to T_eq — the
    # cleanest emergent-equilibrium demonstration.
    node0: float = 0.0
    # Forced constant heat input (W) — the standalone stand-in for Power's electrical
    # dissipation. Plain scenario data (an amplitude); the equilibrium T_eq is emergent
    # from it + the radiator params (NOT derived/tuned — the radiator is the balance).
    # 3000 W with the committed params ⇒ T_eq ≈ 280.9 K (~8 °C radiator).
    heat_load_w: float = 3000.0
    # Integration step (s). 3600 s = 1 h, so the ~65-step relaxation time is well
    # resolved (τ >> dt — the sizing constraint that keeps rationed == 0). No
    # steps_per_day: there is no diurnal cycle (constant load ⇒ monotone relaxation, not
    # a periodic swing).
    dt_seconds: float = 3600.0


# Module-level default (immutable, frozen dataclass) — used as the param default so the
# signatures don't call ThermalScenario() in their defaults (ruff B008).
DEFAULT_THERMAL_SCENARIO: ThermalScenario = ThermalScenario()

# The standalone validation scenario (Step 5): a cold node under a constant heat load,
# warming to an emergent equilibrium temperature where Stefan-Boltzmann rejection
# balances the input. ENERGY conserved every step (augmented ledger: heat_source + node
# + space), rationed == 0 (τ >> dt sizing), events == () (no POPULATION stock), space
# sink monotonic, T converges to T_eq. The defaults already encode the validation
# sizing; this alias names the canonical run shared by the validation test (and the
# golden) so they cannot drift on the sizing.
EQUILIBRIUM_SCENARIO: ThermalScenario = DEFAULT_THERMAL_SCENARIO

# The equilibrium-run horizon (steps). τ ≈ 65 steps, so 720 steps ≈ 11 τ drives the node
# to within e^-11 ≈ 1.7e-5 of T_eq — tight enough to assert a narrow equilibrium band
# while keeping the monotone-warming shape visible over the first several τ. 720·3600 s
# = 30 days. A plain step count (no day structure), unlike Power's BOUNDED_SOC_DAYS.
EQUILIBRIUM_STEPS: int = 720
