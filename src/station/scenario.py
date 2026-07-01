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

**Time unit / step come from the Power scenario** (``dt = 3600 s``,
``steps_per_day = 24``) — Thermal standalone used the same ``dt``, so there is no rate
mismatch to reconcile (the increment-form flows are dt-linear anyway, #multi-rate-safe).
The horizon is a day count, like Power's ``BOUNDED_SOC_DAYS``.

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


# --- Step 3 (P6.3): biosphere ↔ cabin greenhouse
# ------------------------------------

# The greenhouse biosphere: a **sealed** chamber whose CO₂/O₂ pools ARE the cabin
# air. Under the reverse seam (see ``station.greenhouse``) the biosphere's
# ``CARBON_POOL`` / ``O2_POOL`` (already composition ``{C:1,O:2}`` / ``{O:2}``) are
# the shared cabin-gas stocks the crew + ECLSS flows also act on, so their initial
# fills + ``chamber_air_mol`` are sized to the CABIN, not the standalone 1 m²
# chamber:
# - ``chamber_o2_mol0 = 10``: at the ECLSS O₂ setpoint (``eclss.yaml`` o2_setpoint
# = 10 mol) so the makeup regulator starts idle (a wildly different scale would
# make the proportional regulator run backwards, dumping O₂ — the standalone 210
# mol is incompatible with a 10 mol setpoint).
# - ``chamber_co2_mol0 = 3.4``: at the crew-driven scrubber steady state
# ``P/k_scrub = (f_resp·food_intake)/k_scrub = 3.4e-3/1e-3`` mol, so both the
# with- and without-plant runs start at the crew equilibrium and the plant's net
# draw is the only departure (the "it bit" contrast).
# - ``chamber_air_mol = 9500``: sized so ``Ci = ci_ratio·CARBON_POOL/air_mol·1e6``
# is ≈ 250 µmol mol⁻¹ at the crew-driven CO₂ (continuity with the frozen
# chamber's fill), i.e. the plant photosynthesises in a healthy Ci regime. (The
# resulting O₂ mole fraction ``x_O2 = 10/9500`` is low vs 21 %; that only
# *weakens* respiration's O₂ draw via ``f_O2``, strengthening the net-sink
# signal — an honest artefact of the illustrative, uncalibrated ECLSS scales,
# calibration deferred to Step 9.)
# - ``litter_carbon0 = 0``: no seeded soil organic matter, so microbial
# respiration (a CO₂ *source*) stays minimal and the growing seedling is cleanly
# net-assimilating over the window (the advisor's sign requirement).
# ``consumer=False`` (default): a producer-only greenhouse — the consumer adds no
# gas-seam novelty and complicates the net-sink sign. A NON-frozen scenario; the
# frozen biosphere goldens are untouched.
GREENHOUSE_BIO_SCENARIO: SeasonScenario = SeasonScenario(
    sealed=True,
    chamber_o2_mol0=10.0,
    chamber_co2_mol0=3.4,
    chamber_air_mol=9500.0,
    litter_carbon0=0.0,
)


# The greenhouse crew stores, re-sized for the multi-DAY horizon. The Step-2
# ``CabinScenario`` stores (food 1000 mol / water 20 kg) are sized for its 900-step
# (~15 h) run; the crew draw is ``rate·time`` (dt-independent), so a 7-day
# greenhouse draws ~345 mol C/day ⇒ ~2419 mol food and ~30 kg water over the
# horizon. Sized to a ~60 % drawdown (food 4000 mol, water 50 kg) — a material,
# honest depletion that never rations (``rationed == 0`` by well-fed sizing). Intake
# rates + initial humidity are the Step-2 values (reused verbatim, so the cabin
# steady states match).
GREENHOUSE_CABIN_SCENARIO: CabinScenario = CabinScenario(
    food_store0=4000.0,
    water_store0=50.0,
)


@dataclass(frozen=True)
class GreenhouseScenario:
    """Step-3 biosphere ↔ cabin run data (P6.3): plants + crew share the cabin air.

    References the two sibling scenarios it couples (the
    ``StationScenario``-wrapping- ``PowerScenario`` rhythm): a **sealed**
    :class:`SeasonScenario` whose CO₂/O₂ pools are the cabin air
    (:data:`GREENHOUSE_BIO_SCENARIO`) and the Step-2 :class:`CabinScenario` for
    the crew stores + intake rates + initial cabin humidity (the gas initial fills
    come from ``bio``, so ``cabin.cabin_o2_0`` / ``cabin.cabin_co2_0`` are unused
    here).

    **The two-rate driver (see ``station.greenhouse.run_greenhouse``).** The
    biosphere is structurally ``dt = 1`` day (weather indexed by the step count)
    and the cabin ``dt = 60 s`` (ECLSS ``k_scrub·dt < 1``) — two different *time
    units*, which ``simcore.multirate`` (one shared master ``dt``, aux-freezing
    ``substep``) cannot bridge. So each master step is one day: the cabin
    sub-steps ``steps_per_day`` times at ``cabin_dt`` (keeping ``n``), then the
    biosphere takes one ``step_report`` at ``bio_dt`` (advancing phenology aux
    **and** ``n``, so ``n`` stays the day count and the frozen weather resolver is
    reused unchanged).
    """

    # The sealed biosphere whose gas pools are the cabin air (cabin-sized fills).
    bio: SeasonScenario = GREENHOUSE_BIO_SCENARIO
    # The crew stores (re-sized for the multi-day horizon) + intake rates + initial
    # humidity; the gas initial fills come from ``bio``, so ``cabin.cabin_o2_0`` /
    # ``cabin.cabin_co2_0`` are unused here.
    cabin: CabinScenario = GREENHOUSE_CABIN_SCENARIO
    # Horizon in master steps (days). Short and safely inside the seedling's growth
    # phase (the sealed chamber's Ci draw-down spans ~40–60 days), so the biosphere
    # is net- assimilating (a CO₂ sink) throughout — the signed feedback the demo
    # shows. Cheap: the cabin fully relaxes within the first day, so a week already
    # exhibits the shift.
    days: int = 7
    # Cabin sub-steps per biosphere day: 86400 s / 60 s = 1440 (the physical day at
    # the ECLSS dt). ``cabin_dt·steps_per_day == 86400`` (one day) is required for
    # the day mapping; enforced in ``build``/``run``.
    steps_per_day: int = 1440
    cabin_dt: float = 60.0  # s — ECLSS's binding dt (k_scrub·dt = 0.06 < 1)
    bio_dt: float = 1.0  # day — the frozen biosphere's structural step


# Module-level default (immutable) — the canonical Step-3 greenhouse.
DEFAULT_GREENHOUSE_SCENARIO: GreenhouseScenario = GreenhouseScenario()

# The Step-3 validation scenario: the frozen sealed biosphere breathing the crew's
# cabin air. Every quantity (CARBON / OXYGEN / WATER / NITROGEN) conserved every
# sub-step over the combined ledger (the payload); the plant is a **net CO₂ sink /
# O₂ source** vs a no-plant baseline (the signed "it bit" gate); the biosphere's
# internal water + N loops still close; ``rationed == 0`` (well-fed + kinetic
# self-limits); the crew stores run down (open-loop, argument for Steps 4/6). The
# defaults encode the sizing; this alias names the canonical run shared by the
# validation test and the golden so they cannot drift.
GREENHOUSE_SCENARIO: GreenhouseScenario = DEFAULT_GREENHOUSE_SCENARIO
