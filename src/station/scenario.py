"""The Station scenario — the coupled Power → Thermal heat-closure run (P6.1).

The station analogue of ``domains.power.scenario`` / ``domains.thermal.scenario``, one
level up: a station scenario is **not** a new set of coefficients — it *references* the
already-validated sibling scenarios and adds only the cross-domain wiring choices. For
Step 1 the only sibling with tunable run data is Power (the diurnal microgrid); Thermal
contributes its ``radiator.yaml`` params (loaded, not scenario data) and no run data of
its own — the radiator is the restoring force, so it needs no forcing schedule.

**What Step 1 couples.** The standalone Power domain dumped every degraded joule into a
terminal ``boundary.waste_heat`` sink (a deliberate seam). The standalone Thermal domain
received a *forced* ``heat_load`` stand-in for "Power's dissipation" into its node. Step
1 removes both stand-ins and lets them meet at one shared stock: Power's dissipation
legs now deposit into ``thermal.node`` directly, and the Stefan-Boltzmann radiator
rejects that **real** load to deep space. The seam is pure sink re-wiring
(``system.build_station`` passes ``thermal.node``'s id where the Power flows took
``waste_heat``) — zero domain change, zero core change (finding #1).

**The node's initial heat is DERIVED from Power's actual output, not hand-set.** Because
Power runs daily-balanced (``load_fraction = 1`` ⇒ SOC returns to ``battery0`` every
day), in steady state *all* supplied solar energy ends up as heat in the node
(charge-conversion loss ``(1−η_c)·S`` + the 100 %-dissipative load ``η_c·S`` = ``S`` per
day). The mean dissipated power sets an emergent equilibrium node temperature;
``build_station`` starts the node at the corresponding ``Q_eq``
(``system.equilibrium_node_heat``), so the run begins near the attractor the *actual*
dissipation implies. That the equilibrium is set by dissipation independent of the
initial condition is proved **non-circularly** by the two-start convergence test (two
``node0`` values under identical Power forcing converge to one band — the radiator alone
governs the difference), not by starting there.

**Time unit / step come from the Power scenario** (``dt = 3600 s``, ``steps_per_day =
24``) — Thermal standalone used the same ``dt``, so there is no rate mismatch to
reconcile (the increment-form flows are dt-linear anyway, #multi-rate-safe). The horizon
is a day count, like Power's ``BOUNDED_SOC_DAYS``.

Pure stdlib only (a frozen dataclass wrapping the Power scenario).
"""

from dataclasses import dataclass

from domains.biosphere.scenario import SeasonScenario
from domains.power.scenario import BOUNDED_SOC_SCENARIO, PowerScenario


@dataclass(frozen=True)
class StationScenario:
    """Station run data: which sibling scenarios the coupled station is assembled from.

    Thin by design — it references the already-validated ``PowerScenario`` rather than
    re-declaring its fields, so the coupled run cannot drift from the standalone one it
    reuses (the battery trajectory stays bit-identical; see ``test_station_run.py``).
    The radiator params are loaded separately (``radiator.yaml``), like the charge
    param, and the node's initial heat is derived in ``system.build_station`` from
    Power's actual dissipation — neither is scenario data here. Later steps (crew /
    ECLSS / biosphere) add their own scenario references to this struct as their seams
    are built.
    """

    # The Power sub-scenario driving the station: the daily-balanced microgrid whose
    # dissipation the Thermal node now receives. Reused verbatim so the coupled battery
    # SOC matches standalone Power to the bit (coupling is pure sink re-wiring).
    power: PowerScenario = BOUNDED_SOC_SCENARIO


# Module-level default (immutable, frozen dataclass) — the canonical Step-1 station.
DEFAULT_STATION_SCENARIO: StationScenario = StationScenario()

# The Step-1 validation scenario: Power's daily-balanced microgrid feeding the Thermal
# node, the node started at the equilibrium its mean dissipation implies. ENERGY
# conserved every step over the combined ledger (solar_source + battery + node + space),
# ``rationed == 0``, ``events == ()``, the battery bit-identical to standalone Power,
# the node bounded near the predicted equilibrium, ``boundary.space`` monotonic
# (carrying the real load). The defaults already encode the sizing; this alias names the
# canonical run shared by the validation test and the golden so they cannot drift.
HEAT_CLOSURE_SCENARIO: StationScenario = DEFAULT_STATION_SCENARIO

# The golden / bounded-node horizon (days). Started at Q_eq the node stays within a
# tight band (~0.1 K over a week), so a short horizon suffices to pin a bounded
# near-equilibrium endpoint (the Power ``BOUNDED_SOC_DAYS`` length; cheap +
# deterministic).
HEAT_CLOSURE_DAYS: int = 7

# The two-start convergence horizon (days). The relaxation time τ ≈ 14.6 days (long, set
# by the large radiator heat capacity), so this is ~3 τ — enough for two bracketing
# ``node0`` starts to contract to a small fraction of their initial gap (the emergent
# attractor, visible; 7 days would show only ~0.48 τ ≈ 62 % of the gap remaining). Its
# own horizon, longer than the golden's — the attractor claim needs the length, the
# golden does not.
CONTRACTION_DAYS: int = 45


@dataclass(frozen=True)
class CabinScenario:
    """Step-2 gas-loop run data: the crew↔ECLSS cabin-air coupling (P6.2).

    A *fresh* scenario (not a reference to a sibling's, unlike ``StationScenario``
    wrapping ``PowerScenario``): the coupled cabin merges two siblings whose standalone
    scenarios made **incompatible** choices — Crew ran ``dt = 3600 s`` (monotone store
    depletion, dt-agnostic forced draws), ECLSS ran ``dt = 60 s`` (needed for
    ``k_scrub·dt < 1`` structural positivity). The coupled step must be **one** ``dt``,
    and ECLSS's constraint is the binding one, so the gas loop adopts ``dt = 60 s``. The
    forced crew intake rates are re-sized here (not reused from either standalone
    scenario) so the emergent cabin steady states are positive and well-fed under the
    shared ECLSS control-loop params (``eclss.yaml``) + crew split fractions
    (``crew.yaml``).

    **What the coupling changes vs the two standalones.** Standalone ECLSS drove the
    cabin with a *forced* ``CrewMetabolism`` stand-in (independent O₂/CO₂/H₂O rates, RQ
    ≈ 0.75); standalone Crew drew O₂ from a separate ``crew.o2_store`` into a decoupled
    sink and exhaled *pure-carbon* CO₂. Coupled, the real crew ``CrewRespiration``
    (``station.flows``) breathes **cabin** O₂ and exhales into the **cabin** CO₂ pool at
    **RQ = 1** (the PQ = 1 template), so O₂ consumption = CO₂ production is no longer a
    free rate — it is ``respired_carbon_fraction · food_intake``. WATER stays on the
    separate crew ``WaterBalance → cabin_h2o`` path (metabolic water ignored; food
    carries no WATER composition — the phase-6 scope boundary).

    **The cabin reaches a steady state; the stores run down (the hybrid, honestly).**
    The three ECLSS control loops (scrubber / condenser / O₂ makeup) are the restoring
    forces, so each cabin species relaxes to an emergent steady state
    (``cabin.cabin_steady_state``); but ``crew.food_store`` / ``crew.water_store`` are
    still *forced* draws with no resupply, so they deplete monotonically (open-loop,
    like standalone Crew). Step 2 closes the gas loop's **atom coupling** (OXYGEN closes
    across the augmented crew↔cabin loop), **not** the provisioning — regenerating the
    stores is Steps 4 (water) / 6 (food). Sizing keeps every store well-fed over the
    horizon (``rationed == 0``).

    Fields: the initial cabin inventories (a clean cabin at the O₂ setpoint, so the crew
    draw pulls O₂ down and CO₂/H₂O rise from 0 — the ECLSS monotone-relaxation
    demonstration), the finite crew store inventories, the two forced crew intake rates,
    and the shared ``dt``. ``o2_intake_rate`` is **absent** — O₂ consumption is derived
    from ``food_intake`` via RQ = 1 (the whole point of the merge).
    """

    # Initial cabin inventories (canonical units: O₂/CO₂ mol, H₂O kg). cabin_o2_0 starts
    # AT the O₂ setpoint (matches ``eclss.yaml`` o2_setpoint = 10 mol) so the regulator
    # starts idle and the crew draw pulls O₂ down to ``o2_eq``; CO₂/H₂O start at 0 (a
    # clean cabin) and rise to their eq — the cleanest monotone steady-state
    # demonstration.
    cabin_o2_0: float = 10.0
    cabin_co2_0: float = 0.0
    cabin_h2o_0: float = 0.0
    # Finite provisioned crew stores (crew POOLs; canonical units: food carbon mol,
    # water kg). Sized so each stays well-fed over the horizon — food depletes ~22 %
    # (216 mol of 1000), water ~14 % (2.7 kg of 20): a material drawdown that never
    # rations. There is NO o2_store — the crew now breathes cabin O₂ (the inward move;
    # ``crew.o2_store`` and ``OxygenConsumption`` are dropped in the coupled assembly).
    food_store0: float = 1000.0
    water_store0: float = 20.0
    # Forced constant crew intake rates (the standalone stand-in for the crew's real
    # schedule; illustrative, NOT NASA BVAD). With f_resp = 0.85 the food intake gives
    # CO₂/O₂ production = 0.85·4e-3 = 3.4e-3 mol/s ⇒ cabin_co2_eq = 3.4 mol, cabin_o2_eq
    # = 10 − 3.4e-3/2e-3 = 8.3 mol; with f_ins = 0.4 the water intake gives humidity
    # 0.4·5e-5 = 2e-5 kg/s ⇒ cabin_h2o_eq = 0.04 kg. All comfortably positive
    # (``cabin.cabin_steady_state``).
    food_intake_rate: float = 4.0e-3  # mol/s (carbon), food drawn from the food store
    water_intake_rate: float = 5.0e-5  # kg/s, water drawn from the water store
    # Integration step (s). 60 s — ECLSS's dt, the binding constraint (k_scrub·dt = 0.06
    # < 1). No steps_per_day: constant crew load ⇒ monotone relaxation, no diurnal
    # cycle.
    dt_seconds: float = 60.0


# Module-level default (immutable, frozen dataclass) — used as the param default so the
# signatures don't call CabinScenario() in their defaults (ruff B008).
DEFAULT_CABIN_SCENARIO: CabinScenario = CabinScenario()

# The Step-2 validation scenario: the real crew respiring into / breathing from the
# ECLSS cabin, each species relaxing to an emergent steady state where its ECLSS control
# loop balances the crew load. All three quantities (CARBON / OXYGEN / WATER) conserved
# every step over the augmented crew↔cabin ledger (the payload); OXYGEN closes ONLY
# because cabin_co2 is a {C:1,O:2} composition stock and respiration draws O₂ from
# cabin_o2 (the non-vacuous gate — the decoupled version is unpinnable). rationed == 0
# (well-fed + structural), events == () (no POPULATION stock), sinks monotonic, stores
# deplete but stay well-fed. The defaults already encode the sizing; this alias names
# the canonical run shared by the validation test and the golden so they cannot drift.
CABIN_GAS_SCENARIO: CabinScenario = DEFAULT_CABIN_SCENARIO

# The steady-state-run horizon (steps). Same params as standalone ECLSS, so the slowest
# loop is H₂O (τ = 1/k_cond = 2000 s ≈ 33 steps of dt = 60 s); 900 steps = 54000 s = 15
# h ≈ 27 τ_H2O drives every species to within e^-27 of its steady state. A plain step
# count (no day structure). Long enough to reach the cabin steady states while the
# stores stay well-fed (food ~22 %, water ~14 % depleted).
CABIN_GAS_STEPS: int = 900


# --- Step 3 (P6.3): biosphere ↔ cabin greenhouse ------------------------------------

# The greenhouse biosphere: a **sealed** chamber whose CO₂/O₂ pools ARE the cabin air.
# Under the reverse seam (see ``station.greenhouse``) the biosphere's ``CARBON_POOL`` /
# ``O2_POOL`` (already composition ``{C:1,O:2}`` / ``{O:2}``) are the shared cabin-gas
# stocks the crew + ECLSS flows also act on, so their initial fills +
# ``chamber_air_mol`` are sized to the CABIN, not the standalone 1 m² chamber: -
# ``chamber_o2_mol0 = 10``: at the ECLSS O₂ setpoint (``eclss.yaml`` o2_setpoint = 10
# mol) so the makeup regulator starts idle (a wildly different scale would make the
# proportional regulator run backwards, dumping O₂ — the standalone 210 mol is
# incompatible with a 10 mol setpoint). - ``chamber_co2_mol0 = 3.4``: at the crew-driven
# scrubber steady state ``P/k_scrub = (f_resp·food_intake)/k_scrub = 3.4e-3/1e-3`` mol,
# so both the with- and without-plant runs start at the crew equilibrium and the plant's
# net draw is the only departure (the "it bit" contrast). - ``chamber_air_mol = 9500``:
# sized so ``Ci = ci_ratio·CARBON_POOL/air_mol·1e6`` is ≈ 250 µmol mol⁻¹ at the
# crew-driven CO₂ (continuity with the frozen chamber's fill), i.e. the plant
# photosynthesises in a healthy Ci regime. (The resulting O₂ mole fraction ``x_O2 =
# 10/9500`` is low vs 21 %; that only *weakens* respiration's O₂ draw via ``f_O2``,
# strengthening the net-sink signal — an honest artefact of the illustrative,
# uncalibrated ECLSS scales, calibration deferred to Step 9.) - ``litter_carbon0 = 0``:
# no seeded soil organic matter, so microbial respiration (a CO₂ *source*) stays minimal
# and the growing seedling is cleanly net-assimilating over the window (the advisor's
# sign requirement). ``consumer=False`` (default): a producer-only greenhouse — the
# consumer adds no gas-seam novelty and complicates the net-sink sign. A NON-frozen
# scenario; the frozen biosphere goldens are untouched.
GREENHOUSE_BIO_SCENARIO: SeasonScenario = SeasonScenario(
    sealed=True,
    chamber_o2_mol0=10.0,
    chamber_co2_mol0=3.4,
    chamber_air_mol=9500.0,
    litter_carbon0=0.0,
)


# The greenhouse crew stores, re-sized for the multi-DAY horizon. The Step-2
# ``CabinScenario`` stores (food 1000 mol / water 20 kg) are sized for its 900-step (~15
# h) run; the crew draw is ``rate·time`` (dt-independent), so a 7-day greenhouse draws
# ~345 mol C/day ⇒ ~2419 mol food and ~30 kg water over the horizon. Sized to a ~60 %
# drawdown (food 4000 mol, water 50 kg) — a material, honest depletion that never
# rations (``rationed == 0`` by well-fed sizing). Intake rates + initial humidity are
# the Step-2 values (reused verbatim, so the cabin steady states match).
GREENHOUSE_CABIN_SCENARIO: CabinScenario = CabinScenario(
    food_store0=4000.0,
    water_store0=50.0,
)


@dataclass(frozen=True)
class GreenhouseScenario:
    """Step-3 biosphere ↔ cabin run data (P6.3): plants + crew share the cabin air.

    References the two sibling scenarios it couples (the ``StationScenario``-wrapping-
    ``PowerScenario`` rhythm): a **sealed** :class:`SeasonScenario` whose CO₂/O₂ pools
    are the cabin air (:data:`GREENHOUSE_BIO_SCENARIO`) and the Step-2
    :class:`CabinScenario` for the crew stores + intake rates + initial cabin humidity
    (the gas initial fills come from ``bio``, so ``cabin.cabin_o2_0`` /
    ``cabin.cabin_co2_0`` are unused here).

    **The two-rate driver (see ``station.greenhouse.run_greenhouse``).** The biosphere
    is structurally ``dt = 1`` day (weather indexed by the step count) and the cabin
    ``dt = 60 s`` (ECLSS ``k_scrub·dt < 1``) — two different *time units*, which
    ``simcore.multirate`` (one shared master ``dt``, aux-freezing ``substep``) cannot
    bridge. So each master step is one day: the cabin sub-steps ``steps_per_day`` times
    at ``cabin_dt`` (keeping ``n``), then the biosphere takes one ``step_report`` at
    ``bio_dt`` (advancing phenology aux **and** ``n``, so ``n`` stays the day count and
    the frozen weather resolver is reused unchanged).
    """

    # The sealed biosphere whose gas pools are the cabin air (cabin-sized fills).
    bio: SeasonScenario = GREENHOUSE_BIO_SCENARIO
    # The crew stores (re-sized for the multi-day horizon) + intake rates + initial
    # humidity; the gas initial fills come from ``bio``, so ``cabin.cabin_o2_0`` /
    # ``cabin.cabin_co2_0`` are unused here.
    cabin: CabinScenario = GREENHOUSE_CABIN_SCENARIO
    # Horizon in master steps (days). Short and safely inside the seedling's growth
    # phase (the sealed chamber's Ci draw-down spans ~40–60 days), so the biosphere is
    # net- assimilating (a CO₂ sink) throughout — the signed feedback the demo shows.
    # Cheap: the cabin fully relaxes within the first day, so a week already exhibits
    # the shift.
    days: int = 7
    # Cabin sub-steps per biosphere day: 86400 s / 60 s = 1440 (the physical day at the
    # ECLSS dt). ``cabin_dt·steps_per_day == 86400`` (one day) is required for the day
    # mapping; enforced in ``build``/``run``.
    steps_per_day: int = 1440
    cabin_dt: float = 60.0  # s — ECLSS's binding dt (k_scrub·dt = 0.06 < 1)
    bio_dt: float = 1.0  # day — the frozen biosphere's structural step


# Module-level default (immutable) — the canonical Step-3 greenhouse.
DEFAULT_GREENHOUSE_SCENARIO: GreenhouseScenario = GreenhouseScenario()

# The Step-3 validation scenario: the frozen sealed biosphere breathing the crew's cabin
# air. Every quantity (CARBON / OXYGEN / WATER / NITROGEN) conserved every sub-step over
# the combined ledger (the payload); the plant is a **net CO₂ sink / O₂ source** vs a
# no-plant baseline (the signed "it bit" gate); the biosphere's internal water + N loops
# still close; ``rationed == 0`` (well-fed + kinetic self-limits); the crew stores run
# down (open-loop, argument for Steps 4/6). The defaults encode the sizing; this alias
# names the canonical run shared by the validation test and the golden so they cannot
# drift.
GREENHOUSE_SCENARIO: GreenhouseScenario = DEFAULT_GREENHOUSE_SCENARIO


# --- Step 4 (P6.4): the crew water-recovery loop -------------------------------------

# The Step-4 validation scenario reuses the Step-2 cabin sizing VERBATIM (the same crew
# stores + intake rates + dt = 60 s). Step 4 changes only the WATER *plumbing* — the two
# terminal disposal sinks (``humidity_condensate`` / ``urine``) become a
# ``recovered_water`` buffer POOL feeding a ``WaterRecovery`` flow back to the store —
# not the crew load or the gas loop, so the cabin O₂/CO₂/H₂O steady states are the same
# and reusing the scenario keeps them aligned. Step 4 is built on the **cabin**, not the
# greenhouse (the biosphere is Euler-locked by its freeze, so RK4 ≢ Euler — the "it
# earned its keep" signal that recovery made ``water_store`` state-dependent — is only
# cross-checkable below the greenhouse). The ``water_store`` now REGENERATES (net drain
# ``(1−η_w)·intake`` instead of the full intake), staying even more well-fed;
# ``food_store`` depletes as before.
WATER_RECOVERY_SCENARIO: CabinScenario = CABIN_GAS_SCENARIO

# The horizon (steps). Reuses ``CABIN_GAS_STEPS`` (900 × 60 s ≈ 15 h): long enough for
# ``recovered_water`` (τ = 1/k_rec ≈ 1000 s ≈ 17 steps) and ``cabin_h2o`` (τ ≈ 33 steps)
# to reach their steady states while both stores stay well-fed, and for a MATERIAL
# with-vs-without-recovery gap in ``water_store`` to accumulate (the "it bit" signal).
WATER_RECOVERY_STEPS: int = CABIN_GAS_STEPS


# --- Step 5 (P6.5): Power → biosphere lighting ---------------------------------------

# The lighting biosphere: a **sealed, self-contained** chamber (its own CO₂/O₂
# atmosphere — NOT the cabin air; lighting couples Power to the biosphere, not the
# crew). The plain sealed default (``chamber_co2_mol0 = 0.357`` ⇒ Ci ≈ 250 µmol mol⁻¹,
# DVS 0 at sowing), whose Ci draw-down spans ~40–60 days, so over the short lighting
# horizon the seedling is cleanly **net-assimilating** under the lamp — the signed "it
# bit" contrast (lamp-on grows, lamp-off/PAR = 0 declines to maintenance respiration).
# ``litter_carbon0 = 0`` (the default) keeps microbial respiration minimal so the
# growing plant is unambiguously the net carbon sink. A NON-frozen scenario; the frozen
# biosphere goldens are untouched.
LIGHTING_BIO_SCENARIO: SeasonScenario = SeasonScenario(sealed=True)


@dataclass(frozen=True)
class LightingScenario:
    """Step-5 Power → biosphere lighting run data (P6.5): the lamp carries energy into
    biology.

    The phase's **one non-shared-stock coupling** (finding #3 / #16): Power and the
    biosphere share *no* stock. The single interface is the **lamp-draw schedule**,
    which drives both the ``station.flows.Lamp`` flow (the ENERGY it withdraws from
    ``power.battery``) and the biosphere's PAR forcing (``station.lighting.lamp_par`` =
    ``photon_efficacy · lamp_power / ground_area``). So this scenario carries a **sealed
    self-contained** :class:`SeasonScenario` (the biosphere, breathing its own chamber
    air) plus the lamp's electrical schedule + the provisioned battery — the lamp
    coefficient itself (``photon_efficacy``) is a loaded param (``lamp.yaml``), not
    scenario data.

    **The daylength coupling (the correctness crux).** ``incident_par`` returns a
    *daytime-mean* photon flux and the FvCB aggregator re-multiplies by ``daylength_s``
    for the daily photon dose — so PAR and daylength are coupled. Under the lamp
    **both** come from the schedule: ``PAR = photon_efficacy·lamp_power_w/ground_area``
    and ``daylength_s = photoperiod_hours·3600``. (The only runtime ``daylength_s``
    consumer is photosynthesis, so "day = lamp photoperiod" is consistent everywhere it
    is read.) The chamber's non-light forcings — temperature, VPD, net radiation — stay
    weather-driven (reused from the winter-wheat fixture, the greenhouse precedent); a
    fully controlled-environment chamber (setpoint temp/humidity) is a deferred
    refinement, not a Step-5 requirement.

    **The two-rate driver (see ``station.driver.run_master_day``).** The biosphere is
    structurally ``dt = 1`` day; Power runs sub-daily (``power_dt = 3600`` s ×
    ``steps_per_day = 24``) so the lamp draws over a top-hat photoperiod window and the
    battery trajectory is meaningful. Per master day the biosphere ``step_report`` runs
    once (advancing phenology aux **and** ``n``), then Power ``substep`` ×24 (``n``
    kept) — exactly the greenhouse rhythm with Power as the fast domain instead of the
    cabin.
    """

    # The sealed self-contained biosphere the lamp lights (its own chamber air).
    bio: SeasonScenario = LIGHTING_BIO_SCENARIO
    # Initial provisioned battery energy (J). Sized well-fed: the lamp draws
    # ``lamp_power_w·photoperiod_hours·3600`` J/day = 1.152e7 J/day, so over the horizon
    # the battery depletes materially (~40 %) but never approaches 0 (``rationed == 0``
    # by well-fed sizing — the ``LoadDraw`` / Crew-store way; the lamp is forced, not
    # donor-controlled). No solar recharge here (that re-shows Step-1 machinery with no
    # new thesis) — the battery is a finite energy store draining via the lamp.
    battery0: float = 2.0e8  # J
    # The lamp's on-window electrical power (W). With ``photon_efficacy = 2.5`` µmol/J
    # and ``ground_area = 1`` m², this gives PAR = 2.5·200/1 = 500 µmol m⁻² s⁻¹ — a
    # healthy crop PPFD (typical grow range 200–800). Illustrative, not a rated fixture.
    lamp_power_w: float = 200.0
    # Photoperiod (integer hours of lamp-on per day; an integer number of the 24
    # sub-steps so the top-hat aligns to sub-step boundaries). 16 h — a standard
    # horticultural photoperiod with an 8 h dark period (maintenance respiration runs
    # regardless).
    photoperiod_hours: int = 16
    # Horizon in master steps (days). 7 days: safely inside the seedling's growth phase
    # (the sealed Ci draw-down spans ~40–60 days), so the plant is net-assimilating
    # throughout — the signed lamp-on/off contrast. Matches the greenhouse horizon.
    days: int = 7
    # Power sub-steps per biosphere day + the sub-step dt (s). 24 × 3600 = 86400 (one
    # day) is required for the day mapping (enforced in the driver); the lamp draws its
    # top-hat over the first ``photoperiod_hours`` of the 24.
    steps_per_day: int = 24
    power_dt: float = 3600.0  # s
    bio_dt: float = 1.0  # day — the frozen biosphere's structural step


# Module-level default (immutable) — the canonical Step-5 lighting scenario.
DEFAULT_LIGHTING_SCENARIO: LightingScenario = LightingScenario()


# --- Step 6 (P6.6): the biomass/food loop --------------------------------------------

# The harvest horizon (master days). 7 days — the greenhouse horizon; the plant,
# started past anthesis (see ``HarvestScenario.thermal_time0``), fills grain throughout
# and stays short of maturity (DVS 1.27 → ~1.40, far below the 2.0 senescence point),
# so grain is a genuinely regenerative source the harvest drains — not a static
# reservoir emptied.
HARVEST_DAYS: int = 7

# The greenhouse the harvest is built on: the Step-3 greenhouse VERBATIM (the sealed
# cabin-sized chamber ``GREENHOUSE_BIO_SCENARIO`` + the multi-day crew stores
# ``GREENHOUSE_CABIN_SCENARIO``), only the horizon named for Step 6. Reusing the
# greenhouse scenario keeps the cabin gas loop identical — Step 6 adds the ``Harvest``
# flow + a reproductive plant (via ``thermal_time0``), nothing else. There is no
# separate ``HARVEST_BIO_SCENARIO``: the reproductive phase is set by the
# ``thermal_time0`` aux injection (a station field), not a ``SeasonScenario`` field
# (adding one would be a domain change, forbidden by the Phase-6 exit criterion).
HARVEST_GREENHOUSE_SCENARIO: GreenhouseScenario = GreenhouseScenario(days=HARVEST_DAYS)


@dataclass(frozen=True)
class HarvestScenario:
    """Step-6 biomass/food-loop run data (P6.6): grain → crew ``food_store``.

    References the Step-3 :class:`GreenhouseScenario` it is built on (the
    ``StationScenario``-wrapping-``PowerScenario`` rhythm) and adds the single Step-6
    knob: the biosphere phenology at ``t = 0``. The ``Harvest`` flow
    (:class:`station.flows.Harvest`) drains the biosphere's ``storage_c`` (grain) into
    the crew ``food_store`` in the cabin / fast registry, making the crew's finite food
    **regenerative** — the CARBON twin of Step 4's ``WaterRecovery``. The harvest rate
    is a loaded param (``harvest.yaml``), not scenario data.

    **The reproductive plant (the "it bit" precondition).** ``storage_c`` fills only
    after anthesis (``FO > 0`` needs ``DVS > 1``), and a fresh seedling sits at
    ``DVS < 1`` with ``storage_c0 = 0`` — so the default greenhouse plant would give a
    zero harvest source. :attr:`thermal_time0` starts the biosphere's ``thermal_time``
    accumulator **past** anthesis (``tsum_anthesis = 1100`` °C·day), so ``DVS > 1`` and
    grain is actively *filling* while harvest drains it (a regenerative source, not a
    static reservoir). This is a **station** field injected at :class:`State`
    construction in :func:`station.harvest.build_harvest` — the station owns the
    greenhouse ``State``'s aux dict, so it needs no change to ``SeasonScenario`` (adding
    a ``thermal_time0`` field there would be a domain change, forbidden by the Phase-6
    exit criterion).

    Runs under the two-rate :func:`station.driver.run_master_day` (the greenhouse
    rhythm: biosphere slow once/day, cabin fast ×``steps_per_day``), so all
    timing/horizon fields come from the embedded greenhouse scenario.
    """

    # The Step-3 greenhouse this is built on (sealed cabin-sized chamber + multi-day
    # crew stores + the two-rate timing), only the Step-6 horizon named.
    greenhouse: GreenhouseScenario = HARVEST_GREENHOUSE_SCENARIO
    # The biosphere ``thermal_time`` accumulator at ``t = 0`` (°C·day). Past anthesis
    # (``tsum_anthesis = 1100``) so ``DVS > 1`` ⇒ ``FO > 0`` ⇒ the plant fills grain
    # from day 0. 1300 (DVS 1.27) — the go/no-go spike's pick: actively filling with
    # maximal maturity headroom (final DVS ~1.40, far below the 2.0 senescence point),
    # grain-fill ~6 orders above the ledger round-off floor. A STATION field (see the
    # class docstring — NOT a ``SeasonScenario`` field), injected at ``State``
    # construction.
    thermal_time0: float = 1300.0


# Module-level default (immutable) — the canonical Step-6 harvest scenario.
DEFAULT_HARVEST_SCENARIO: HarvestScenario = HarvestScenario()

# The Step-6 validation scenario: the reproductive greenhouse plant filling grain that
# the harvest drains into the crew's food_store. Every quantity (CARBON / OXYGEN / WATER
# / NITROGEN) conserved every master day over the combined ledger (the payload); the
# with-vs-without-harvest two-way identity (Δfood_store = cumulative harvest =
# Δstorage_c — the signed "it bit" gate); the crew food_store depletes SLOWER than the
# no-harvest baseline (regenerated by grain); grain settles to a positive quasi-steady
# (daily harvest ≈ daily fill); ``rationed == 0`` (structural + well-fed); ``events ==
# ()``. Euler-only (the greenhouse biosphere is Euler-locked by its freeze). The
# defaults encode the sizing; this alias names the canonical run shared by the
# validation test and golden so they cannot drift.
HARVEST_SCENARIO: HarvestScenario = DEFAULT_HARVEST_SCENARIO

# The Step-5 validation scenario: the frozen sealed biosphere lit by a battery-powered
# grow lamp. ENERGY conserved every step over the Power ledger (battery + light_used +
# waste_heat — the lamp names every joule); the biosphere internal CARBON / OXYGEN /
# WATER / NITROGEN loops still close; the plant is a **net carbon sink under the lamp**
# (lamp-on ``bio_organic_C`` grows) but **not without it** (PAR = 0 ⇒ it declines) — the
# signed "it bit" gate; the battery depletes (open-loop, well-fed); ``rationed == 0``,
# ``events == ()``. The defaults encode the sizing; this alias names the canonical run
# shared by the validation test and the golden so they cannot drift.
LIGHTING_SCENARIO: LightingScenario = DEFAULT_LIGHTING_SCENARIO
