"""The Crew scenario — initial store inventories, the forced crew intake (Step 7).

The Crew analogue of ``domains.power.scenario`` / ``domains.eclss.scenario``: pure
scenario **data** (not the metabolic-split fractions — those are ``params/crew.yaml``
via ``loader.py``). Every field is sizing / forcing wiring for the standalone
**mission-endurance** validation run that ``domains.crew.system`` assembles.

**Forced ⇒ NO restoring force, NO attractor — the stores just run down (like Power's
forced flows, unlike ECLSS/Thermal).** All three crew flows read a constant intake
*rate* from ``env`` and never read a store amount, so each store is a pure
linear-depletion accumulator: ``store(n) = store0 − n·rate·dt``. There is no emergent
bound — boundedness is simply that the **mission is shorter than the closed-form
time-to-depletion** ``store0 / rate`` (see ``system.depletion_times``). This is Power's
"forced accumulator" situation, but *monotone depletion* rather than a *balanced
oscillation* — Crew does not even construct a balance (there is no resupply standalone).
That standalone incompleteness is the argument for Phase-6 closure (the biosphere +
ECLSS regenerate what the crew consumes).

**The load-bearing sizing constraint — well-fed provisioning (the ``LoadDraw`` way).**
``rationed == 0`` holds because every store stays positive over the mission: the mission
duration is a fraction of each store's time-to-depletion, so no forced draw ever
over-draws a store (which would fire the Euler backstop, or hard-error under RK4). With
the defaults below (dt = 3600 s, a 7-day mission = 168 steps) each store depletes to ≈
70 % of its initial inventory — a **material** drawdown (so "the mission actually ran"
is unambiguous) that stays comfortably positive (so ``rationed == 0``). An over-long
mission that empties a store is the **brownout/starvation** analogue — a documented
seam, not the baseline golden.

**Forced ⇒ RK4 ≡ Euler bit-identical.** No flow reads a stock, so every RK4 stage
derivative is identical (``k1 = k2 = k3 = k4``) and RK4 reproduces Euler bit-for-bit —
the symmetric bookend to ECLSS/Thermal (which broke that identity). The validation
asserts the bit-identity, framed as the identity (not robustness), exactly as Power's
two-flow run did.

**Time unit is seconds (SI), documented not implicit** — the intake rates are mol/s or
kg/s, as for Power/ECLSS. There is **no diurnal structure** (constant crew load ⇒
monotone depletion), so the horizon is a plain mission length in days ×
``steps_per_day`` (hourly resolution), unlike Power's day/night ``steps_per_day``.

Pure stdlib only (a frozen dataclass).
"""

from dataclasses import dataclass


@dataclass(frozen=True)
class CrewScenario:
    """Scenario data (not coefficients): initial store inventories, crew intake, step.

    Defaults are an illustrative provisioned mission (not a calibrated crew — Phase 5
    ships machinery, not validated numbers; Crew calibration against NASA BVAD / BioSim
    is Phase 6, the biosphere/Power/Thermal/ECLSS ``TODO(cite)`` precedent). The intake
    rates are the forcing amplitudes; the ``*_store0`` the initial provisioned
    inventories. The split of each intake across output fates is set by the
    ``crew.yaml`` params (see ``flows``).
    """

    # Initial provisioned store inventories (canonical units: food/O₂ mol, water kg).
    # Sized (probe) so each store's time-to-depletion (store0/rate) is ~3.3× the 7-day
    # mission, i.e. each depletes to ≈ 70 % — a material drawdown that stays well-fed
    # (rationed == 0). food_store0 2·food; o2_store0 2000 mol O₂; water_store0 60 kg.
    food_store0: float = 1000.0
    water_store0: float = 60.0
    o2_store0: float = 2000.0
    # Forced constant crew intake rates — the standalone stand-in for the crew's real
    # consumption schedule. Plain scenario data (amplitudes); illustrative (NOT NASA
    # BVAD numbers). With the defaults each store's endurance is store0/rate = 2.0e6 s ≈
    # 23.1 days, so the 7-day mission leaves ≈ 70 % in every store.
    o2_intake_rate: float = 1.0e-3  # mol/s, crew O₂ drawn from the O₂ store
    food_intake_rate: float = 5.0e-4  # mol/s (carbon), food drawn from the food store
    water_intake_rate: float = 3.0e-5  # kg/s, water drawn from the water store
    # Integration step (s) and steps per 24 h day. 3600 s × 24 = 86400 s = one day.
    # Hourly resolution over a multi-day mission (there is no diurnal cycle — constant
    # crew load ⇒ monotone depletion — so ``steps_per_day`` is pure resolution, not a
    # day/night shape).
    dt_seconds: float = 3600.0
    steps_per_day: int = 24


# Module-level default (immutable, frozen dataclass) — used as the param default so the
# signatures don't call CrewScenario() in their defaults (ruff B008).
DEFAULT_CREW_SCENARIO: CrewScenario = CrewScenario()

# The standalone validation scenario (Step 7): a provisioned mission under a constant
# crew load, every store depleting monotonically but staying well-fed over the horizon.
# All three quantities (CARBON / OXYGEN / WATER) conserved every step over the augmented
# ledger (the payload), rationed == 0 (well-fed sizing), events == () (no POPULATION
# stock), output sinks monotonic, RK4 ≡ Euler bit-identical (forced-only). The defaults
# already encode the validation sizing; this alias names the canonical run shared by the
# validation test (and the golden) so they cannot drift on the sizing.
MISSION_SCENARIO: CrewScenario = DEFAULT_CREW_SCENARIO

# The mission length in days; the run horizon is MISSION_DAYS × steps_per_day steps. 7
# days (like Power's BOUNDED_SOC_DAYS) at hourly resolution = 168 steps — long enough
# for a material drawdown (≈ 30 % of each store) yet short of the ≈ 23-day endurance, so
# every store stays positive and rationed == 0. A plain mission length (no day/night
# structure).
MISSION_DAYS: int = 7
