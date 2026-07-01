"""The ECLSS scenario — initial cabin inventories, the forced crew load (Step 6).

The ECLSS analogue of ``domains.power.scenario`` / ``domains.thermal.scenario``: pure
scenario **data** (not the control-loop coefficients — those are ``params/eclss.yaml``
via ``loader.py``). Every field is sizing / forcing wiring for the standalone
steady-state validation run that ``domains.eclss.system`` assembles.

**The forced crew load is NOT derived (like Thermal, unlike Power) — each control loop
is the restoring force.** Power's two flows were both *forced*, so its SOC was a
restoring-force-free accumulator and a *bounded* SOC had to be *constructed* by an
exactly-balanced derived load. ECLSS is different: the three control flows
(``CO2Scrubber`` / ``Condenser`` / ``O2Makeup``) are donor-/demand-controlled and
monotone, so **any** constant crew load yields a **unique, stable steady state** per
species (``co2_eq = P_co2/k_scrub``, ``h2o_eq = P_h2o/k_cond``, ``o2_eq = o2_setpoint −
Con_o2/k_makeup`` — see ``system.steady_state``). So the crew rates are plain scenario
data (amplitudes, like Power's ``solar_peak_w``), **not** derived —
``system.eclss_resolver`` is a simple set of constants, no coefficient coupling.

**Linear ⇒ geometric contraction (the SelfDischarge idiom, per species).** Each species'
dynamics is first-order (or demand-controlled, also first-order in the amount), so —
unlike Thermal's nonlinear ``T⁴`` — two runs differing only in one species' initial
amount contract by the **exact** ``d_n = d_0·(1 − k·dt)^n`` law. The validation asserts
that exact law per species (not just monotone contraction).

**The load-bearing sizing constraints.** CO₂/H₂O positivity is **structural**
(``k_scrub·dt < 1``, ``k_cond·dt < 1`` — donor-controlled, self-limiting); O₂ positivity
is by **well-fed sizing** (``cabin_o2`` never empties — the depleting side is the crew
O₂ draw, and ``o2_eq > 0``). With the committed params (k_scrub 1e-3, k_cond 5e-4,
k_makeup 2e-3, o2_setpoint 10 mol) and the defaults below (dt = 60 s), the per-step
removal fractions are 0.06 / 0.03 / 0.12 (all < 1) and the steady states are ``co2_eq =
3.0 mol``, ``h2o_eq = 0.04 kg``, ``o2_eq = 8.0 mol`` — all comfortably positive, so
``rationed == 0``.

**The cabin starts clean and away from equilibrium, and each species relaxes to its
steady state.** ``cabin_co2_0 = 0`` / ``cabin_h2o_0 = 0`` rise to their eq; ``cabin_o2_0
= o2_setpoint`` (10 mol) is drawn down to ``o2_eq`` (8 mol). A clean monotone approach
per species (no periodic structure — constant crew load ⇒ monotone relaxation).

**Time unit is seconds (SI), documented not implicit** — the rates are 1/s and the crew
rates mol/s or kg/s, as for Power/Thermal. There is **no diurnal structure** (constant
crew load ⇒ monotone relaxation), so the horizon is a plain step count
(``STEADY_STATE_STEPS``) sized to reach steady state (many time constants), unlike
Power's ``steps_per_day``.

Pure stdlib only (a frozen dataclass).
"""

from dataclasses import dataclass


@dataclass(frozen=True)
class EclssScenario:
    """Scenario data (not coefficients): initial cabin inventories, crew load, step.

    Defaults are an illustrative small-cabin life-support loop (not a calibrated system
    — Phase 5 ships machinery, not validated numbers; ECLSS calibration against NASA
    BVAD / BioSim is Phase 6, the biosphere/Power/Thermal ``TODO(cite)`` precedent). The
    crew rates are the forcing amplitudes; the ``cabin_*_0`` the initial inventories;
    the steady states are *emergent* from these plus the ``eclss.yaml`` params (see
    ``system.steady_state``).
    """

    # Initial cabin inventories (canonical units: O₂/CO₂ mol, H₂O kg). cabin_o2_0
    # starts AT the O₂ setpoint (10 mol) so the regulator starts idle and the crew
    # draw pulls O₂ down to o2_eq; CO₂/H₂O start at 0 (a clean cabin) and rise to
    # their eq — the cleanest monotone steady-state demonstration per species.
    cabin_o2_0: float = 10.0
    cabin_co2_0: float = 0.0
    cabin_h2o_0: float = 0.0
    # Forced constant crew metabolic rates — the standalone stand-in for the Crew
    # domain. Plain scenario data (amplitudes); the steady states are emergent from
    # them + the eclss.yaml params (NOT derived/tuned — each control loop is the
    # balance). Illustrative (NOT NASA BVAD numbers): with the committed params
    # these give co2_eq = 3.0 mol, h2o_eq = 0.04 kg, o2_eq = 10 − 0.004/2e-3 = 8.0
    # mol.
    o2_consumption_rate: float = 0.004  # mol/s, crew O₂ intake
    co2_production_rate: float = 0.003  # mol/s, crew CO₂ output
    h2o_production_rate: float = 2.0e-5  # kg/s, crew humidity output
    # Integration step (s). 60 s = 1 min, so the fastest control loop (O₂, τ ≈ 500 s
    # ≈ 8 steps) is well resolved and k·dt < 1 for all three loops (0.06 / 0.03 /
    # 0.12). No steps_per_day: there is no diurnal cycle (constant crew load ⇒
    # monotone relaxation).
    dt_seconds: float = 60.0


# Module-level default (immutable, frozen dataclass) — used as the param default so the
# signatures don't call EclssScenario() in their defaults (ruff B008).
DEFAULT_ECLSS_SCENARIO: EclssScenario = EclssScenario()

# The standalone validation scenario (Step 6): a clean cabin under a constant crew load,
# each species relaxing to an emergent steady state where its control loop balances the
# crew load. All three quantities (CARBON / OXYGEN / WATER) conserved every step over
# the augmented ledger (the payload), rationed == 0 (structural for CO₂/H₂O, well-fed
# sizing for O₂), events == () (no POPULATION stock), co2_removed / humidity_condensate
# sinks monotonic, each species converging to its steady state. The defaults already
# encode the validation sizing; this alias names the canonical run shared by the
# validation test (and the golden) so they cannot drift on the sizing.
STEADY_STATE_SCENARIO: EclssScenario = DEFAULT_ECLSS_SCENARIO

# The steady-state-run horizon (steps). The slowest loop is H₂O (τ = 1/k_cond = 2000 s ≈
# 33 steps of dt = 60 s); 900 steps = 54000 s = 15 h ≈ 27 τ_H2O drives every species to
# within e^-27 of its steady state — tight enough to assert a narrow band while keeping
# the monotone-approach shape visible over the first several τ. A plain step count (no
# day structure), unlike Power's BOUNDED_SOC_DAYS.
STEADY_STATE_STEPS: int = 900
