"""The season scenario — plot, initial amounts, soil/atmosphere/chamber knobs (P3.2).

Extracted from ``season.py`` (the Phase-3 Step-2 compartment-builder refactor) so a
compartment builder can take a :class:`SeasonScenario` argument **without importing
``season``** (``season`` imports the builders; the reverse would cycle). This is pure
scenario *data* — not flow-logic coefficients (those are crop params from YAML via
``loader.py``); every field is scenario wiring, defaulted to the Phase-1 winter-wheat
potential-production (PP) plot.

Pure stdlib only (a frozen dataclass).
"""

from dataclasses import dataclass


@dataclass(frozen=True)
class SeasonScenario:
    """Scenario data (not crop params): plot, initial amounts, soil/atmosphere knobs.

    Defaults are the Phase-1 winter-wheat PP plot (1 m² ground, a small
    sown seedling, N/water kept non-limiting — see the ``season`` module docstring). All
    are scenario wiring, not flow-logic coefficients (P4); crop coeffs come from the
    param files via the loaders.
    """

    ground_area: float = 1.0  # m²
    # seedling organ carbon (mol C) at sowing — small, nonzero (LAI ≈ 0.03 at emergence)
    leaf_c0: float = 0.05
    stem_c0: float = 0.03
    root_c0: float = 0.08
    storage_c0: float = 0.0
    # CO₂: an unclamped atmosphere (FvCB reads Ci forcing, not the stock) + a resp sink.
    # Started at 0 (it tracks cumulative net exchange, going negative) so amounts stay
    # O(1)–O(1e3) and the conservation gate's relative tolerance holds (a huge source
    # would swamp the small daily flux below float resolution; the demo's amounts note).
    co2_atmos0: float = 0.0
    ci: float = 250.0  # intercellular CO₂ (µmol mol⁻¹ ≈ 0.7·ambient for C3)
    # Sealed chamber (P2.2). ``sealed=False`` keeps the Phase-1 open field (unclamped
    # ``co2_atmos`` boundary + constant ``ci`` forcing; the regression golden is
    # untouched). ``sealed=True`` swaps in a finite ``carbon_pool`` POOL that
    # photosynthesis draws down, and derives Ci from it (the draw-down feedback). The
    # chamber air total + initial fill are sized (see the Step-2 design / probe) so Ci
    # falls meaningfully toward Γ* without exhausting the pool (rationed == 0). The
    # default fill reproduces the Phase-1 Ci=250 at t=0
    # (Ci0 = ci_ratio·co2_mol0/air_mol·1e6).
    sealed: bool = False
    chamber_air_mol: float = 1000.0  # total chamber air (mol); 0-D well-mixed
    # initial pool carbon (mol C); Ci0 = ci_ratio·co2_mol0/air_mol·1e6 ≈ 250 µmol mol⁻¹
    # (continuity with the Phase-1 constant Ci forcing). Sized (Step-2 probe) so the
    # draw-down spans ~40–60 days down toward Ci≈Γ* — Ci falls ~5×, gross assimilation
    # collapses ~4 orders — while withdrawals stay far from exhausting the pool
    # (rationed == 0; FvCB Ci-shutoff self-limits, never the Euler backstop).
    chamber_co2_mol0: float = 0.357
    ci_ratio: float = 0.7  # C3 Ci/Ca draw-down set point (Farquhar & Sharkey 1982)
    # O₂ counterpart pool (mol O₂; Step 3). Sized to a realistic chamber O₂ fraction
    # (~21% of ``chamber_air_mol``) — vastly larger than the O(0.1) mol C gas fluxes, so
    # it never approaches arbitration rationing and plant respiration needs no O₂
    # self-limitation (``f_O2``) yet. The depleting-O₂ regime (where ``f_O2`` becomes
    # load-bearing) arrives with microbial respiration (Step 5) and the O₂-depletion
    # validation (Step 7); a Step-3 test pins O₂ ≫ 0 to guard that deferral.
    # Photosynthesis deposits O₂ here (PQ=1) and respiration draws it, so it
    # anti-correlates with the CO₂ pool: ΔO₂ = −Δ(net CO₂), 2·(CO₂+O₂) conserved.
    chamber_o2_mol0: float = 210.0
    # Initial standing litter carbon (mol C) at sowing — the decomposer "soil organic
    # matter" seed. Default 0 (the PP sealed run starts with no litter; senescence makes
    # it). The Step-7 depleting run seeds a substantial litter pile so decomposition →
    # microbial respiration draws the (smaller) O₂ pool down a clear fraction toward its
    # floor (the Biosphere-2 soil-respiration O₂-depletion mechanism). Sealed-only.
    litter_carbon0: float = 0.0
    # Minimal consumer (P3 Step 7). ``consumer=False`` keeps every producer-only run
    # (open
    # field, the sealed/perennial chambers) byte-identical — the consumers leaf stays
    # empty and no consumer stock/flow is built. ``consumer=True`` (only meaningful with
    # ``sealed=True`` — the consumer reads the chamber's ``carbon_pool``/``o2_pool`` and
    # the soil's ``litter_carbon``) builds the one ``consumer_carbon`` POPULATION + the
    # grazing / consumer-respiration / mortality flows. ``consumer_c0`` is the herbivore
    # biomass at sowing (small, nonzero — a consumer present from t=0); first-order
    # grazing would refill it from leaf even from 0, but a positive seed reads honestly.
    consumer: bool = False
    consumer_c0: float = 0.01  # mol C (sealed + consumer only)
    # water (PP, non-limiting): a store sized to stay above the band all season
    soil_water0: float = 1000.0  # kg
    # Sealed water cycle (P3.3/Step 3): initial vapor + condensate (kg). Default 0 — the
    # closed ring fills them from ``soil_water`` by transpiration → condensation; the
    # whole-loop total ``soil_water + water_vapor + condensate`` is the conserved
    # invariant (== soil_water0 when these start at 0). Sealed-only (the
    # ``litter_carbon0`` precedent). The first-order condensation/recycling rates
    # (water_cycle.yaml) keep the in-flight water tiny so ``soil_water`` stays ≫
    # ``sw_critical`` — i.e. ``f_water ≡ 1``, so the carbon/O₂/N trajectory is
    # bit-identical to the pre-cycle sealed run.
    water_vapor0: float = 0.0  # kg
    condensate0: float = 0.0  # kg
    water_source0: float = 0.0  # kg (unclamped supply; tracks cumulative irrigation)
    sw_wilting: float = 20.0  # kg
    sw_critical: float = 60.0  # kg
    irrigation_mm_day: float = 2.0  # mm day⁻¹
    # nitrogen (PP, non-limiting): a generous plant-N reserve + ample soil supply
    soil_n0: float = 100.0  # kg N (>> sn_critical ⇒ availability = 1 all season)
    n_source0: float = 0.0  # kg N (unclamped supply; tracks cumulative fertilization)
    plant_n0: float = 0.5  # kg N — high conc ⇒ f_N = 1 all season (plant_n only grows)
    sn_residual: float = 1.0  # kg N (soil-N availability band, scenario/soil data)
    sn_critical: float = 50.0  # kg N
    fertilization_kg_m2_day: float = 0.0  # kg N m⁻² day⁻¹ (soil store already ample)
    # location (for the astronomical daylength); matches the oracle plot
    latitude: float = 52.0
    # Phenology modifiers (post-roadmap day-neutral crop). Both default True — the
    # frozen winter wheat carries vernalization (cold requirement) AND photoperiod
    # (long-day slowdown), so every frozen scenario keeps both and the goldens are
    # byte-identical. A **day-neutral** crop sets BOTH False: ``build_plants`` then
    # builds ``ThermalTimeAccumulation`` with neither modifier and omits
    # ``VernalizationAccumulation`` entirely, so DVS advances on thermal time alone (the
    # ``phenology.py`` optional-modifier seam — output is byte-identical to the plain
    # degree-day rate when both are absent). Independent bools because the two modifiers
    # are independent in the model (a photoperiod-only crop — vernalization off,
    # daylength on — stays expressible). Reuses the SAME cited winter-wheat crop params
    # (phenology.yaml); a day-neutral wheat is winter-wheat physiology with the cold/
    # daylength gates removed (ceremony 2: "vernalization is optional by design"), not a
    # new param file. See docs/plans/post-roadmap-day-neutral-crop.md.
    vernalization: bool = True
    photoperiod: bool = True


# Module-level default (immutable, frozen dataclass) — used as the param default so the
# signatures don't call SeasonScenario() in their defaults (ruff B008).
DEFAULT_SCENARIO: SeasonScenario = SeasonScenario()

# The canonical Phase-2 Step-7 sealed run: a deliberately **O₂-poor** chamber (2 mol O₂
# in 1000 mol air ≈ 0.2 % — a scale choice, like the Step-2 ``air_mol`` probe, so the
# tiny 1 m²-seedling gas fluxes can deplete O₂ non-vacuously) seeded with **3 mol C of
# standing litter** (the "soil organic matter"). Decomposition + microbial respiration
# draw O₂ down ~99 % to an acute trough while ``f_O2`` self-limits the draw (so
# ``rationed == 0`` survives the depleting pool — the Biosphere-2 O₂-depletion failure
# mode); the live producer then transiently refills O₂ before it dies, after which
# the chamber settles CO₂-rich (Ci ≈ 1140). Sized empirically (probe; ``f_N ≡ 1`` here —
# N stays non-limiting, so the N-limited regime is deferred). Run multi-year by tiling
# the season weather ``SEALED_CHAMBER_YEARS×``. Shared by the validation test and the
# regression golden so they cannot drift on the sizing.
SEALED_CHAMBER_SCENARIO: SeasonScenario = SeasonScenario(
    sealed=True,
    chamber_o2_mol0=2.0,
    litter_carbon0=3.0,
)
SEALED_CHAMBER_YEARS: int = 3

# The Phase-3 Step-4 (P3.4) perennial chamber: the sealed scenario plus an **annual
# phenology reset / re-sow** (applied by ``season.run_perennial`` at each year
# boundary), giving **sustained multi-year oscillation** instead of the one-shot "plant
# dies after year 1" baseline. The ample-O₂ sibling of ``SEALED_CHAMBER_SCENARIO``
# (``chamber_o2_mol0`` at the default 210, not the O₂-poor 2.0): the O₂-depletion drama
# is a Phase-2 capstone concern orthogonal to the perennial carbon oscillation, left out
# here so the oscillation is the clean headline. The 3 mol seeded litter fuels year-1
# growth; thereafter the closed carbon loop (organs/grain → litter at each reset →
# microbial → CO₂ → regrowth) sustains it. Probed (5 yr): DVS reaches maturity every
# year, a stable emergent period-2 limit cycle, ``rationed == 0``, ``events == ()`` (the
# carbon loss-sink stays 0.0 — genuinely closed), all four quantities conserved. Shared
# by the validation test and the regression golden so they cannot drift on the sizing.
PERENNIAL_CHAMBER_SCENARIO: SeasonScenario = SeasonScenario(
    sealed=True,
    litter_carbon0=3.0,
)
PERENNIAL_CHAMBER_YEARS: int = 5

# The Phase-3 Step-7 minimal-consumer chamber: the perennial sealed chamber plus **one
# herbivore** (``consumer=True``) proving the trophic pattern (graze ``leaf_c`` →
# consumer
# biomass → respiration CO₂ + death-to-litter). The consumer composes onto the *same*
# closed perennial ecosystem (``annual_reset`` stays plant-only — the herbivore persists
# across the re-sow), so it inherits the sustained multi-year oscillation and the
# genuine
# closure (loss-sink 0.0). Sized (probe, the Step-4 rhythm) so the consumer **persists**
# (consumer* = grazing·leaf/(respiration+mortality) tracks the leaf), the plant still
# **fills grain** so ``annual_reset`` never trips its seed-bank guard (the recoverable
# regime), and ``rationed == 0`` / ``events == ()`` / four-quantity conservation all
# hold.
# Its own new golden. The producer-only goldens (open / sealed / perennial) stay
# byte-identical (``consumer`` defaults False everywhere else).
# Chamber ENLARGED 2x (post-roadmap scope (B) increment 1). The vernalization +
# photoperiod sciences produce a ~5x larger, correctly-developing plant, and the
# herbivore grazes leaf so the plant regrows by drawing MORE from the CO2 pool -- this
# chamber's carbon throughput exceeds the herbivore-free perennial's. At the original
# 0.357 mol / 1000 mol air it over-drew the pool at step 196 (1.29x), tripping the Euler
# backstop (rationed == 1) and RK4's hard ArbitrationError (scale_f 0.9506). All three
# gas quantities scale by the SAME factor so BOTH intensive variables are invariant:
# Ci0 = ci_ratio*co2/air*1e6 = 250 and x_O2 = o2/air = 0.21 both unchanged -- a bigger
# chamber holding the same gas, not a different atmosphere. The factor is the smallest
# round one past the ~1.5x exhaustion threshold (carbon draw-down is scale-INVARIANT
# above it: FvCB Ci-shutoff pins Ci to a fixed fraction toward Gamma*), ~2x peak-draw
# headroom. SEALED and PERENNIAL keep their frozen sizing -- neither rations, and
# SEALED's O2-depletion drama depends on its coupled O2/litter tuning (see
# docs/plans/post-roadmap-oracle-match.md). CONSEQUENCE, recorded: this is no longer
# literally "the perennial chamber + one herbivore" but a LARGER chamber that also holds
# a herbivore, because the herbivore raises carbon demand.
CONSUMER_CHAMBER_SCENARIO: SeasonScenario = SeasonScenario(
    sealed=True,
    litter_carbon0=3.0,
    consumer=True,
    chamber_air_mol=2000.0,
    chamber_co2_mol0=0.714,
    chamber_o2_mol0=420.0,
)
CONSUMER_CHAMBER_YEARS: int = 5

# The Phase-4 (P4.2) **decade-scale** horizon: the run length at which the closed
# biosphere's emergent limit cycle is stability-validated (Step 1 probe) and pinned as
# the canonical *long-horizon* golden (Step 4) — the run the freeze contract (Step 5)
# points at. 15 (>= the decade-scale 10-yr target): both scenarios are fully settled by
# ~yr 10, and 15 yr affords enough same-phase differences to characterize the attractor
# (10 yr gives only ~4 per branch — too thin). Single source of truth shared by the
# long-horizon golden, the decade probe, and the freeze manifest, so the frozen horizon
# cannot drift.
LONG_HORIZON_YEARS: int = 15

# --- Additive dormant-machinery scenarios (NOT frozen reference scenarios) -----------
# Two **additive, non-reference** scenarios that deliberately drive code paths the seven
# frozen scenarios leave dormant: the ``f_N`` photosynthesis limiter (every frozen
# scenario keeps ``f_N ≡ 1`` — verified by ``test_*_f_n_stays_one``) and the sealed
# water cycle's ``f_water`` (tuned **inert** in the frozen chambers — ``soil_water``
# stays far above the stress band, so ``f_water ≡ 1``). Purpose: flush latent bugs in
# the never-run-hot limiter integration before Phase 5 builds on it. **These are NOT
# part of the freeze reference** (not in ``biosphere-reference.manifest.json``): they
# add no flow/aux/param — only new scenario *data* + their own goldens — so every frozen
# trajectory stays byte-identical. Sized by probe (the Step-4 rhythm); see each note.

# **N-limiting** (open field, single season): N-limitation **by dilution** — the primary
# mechanism ``nitrogen.py`` names. A deliberately small fixed plant-N reserve
# (``plant_n0`` ~ the f_N critical concentration times the seedling biomass) puts the
# whole-plant N concentration ``plant_n / (leaf+stem+root)`` inside the
# ``(n_residual, n_critical)`` band at sowing; as biomass grows the concentration falls
# *through* the band, so ``f_N`` ramps below 1 and N-limits gross assimilation (probe:
# ``f_N`` reaches ~0.55, biting on ~66 of 305 steps, then recovers as the plant dies
# back). Uptake is shut **off**: ``soil_n0`` below the default ``sn_residual=1.0``,
# so ``soil_n_availability ≡ 0`` and ``NitrogenUptake`` yields a structural zero leg
# every step — which keeps ``plant_n`` constant so the bite is pure dilution,
# unconfounded by uptake. (The ``soil_n_availability`` *middle* ramp cannot be
# co-exercised arbitration-free **with this dilution bite**: it pins ``plant_n`` in
# the tiny f_N band, where the frozen ``max_uptake_capacity = 0.0015`` kg N/m2/day is
# ~15x that band per day, so any in-band uptake either floods ``plant_n`` past the f_N
# band or exhausts ``soil_n`` in one step -> the Euler backstop. The ramp IS traversable
# in a *healthy-plant* run with a narrow high soil-N band, but that would not make f_N
# bite — a different experiment, out of this scenario's two-scenario scope. So this
# scenario owns the f_N concentration ramp + the uptake-shutoff path; the availability
# *middle* ramp stays an integrated never-run-hot path, unit-tested in
# ``test_nitrogen.py``.) Open field (the only place with no N return loop), single
# season, ``rationed == 0`` / ``events == ()`` / loss-sink ``0.0``.
N_LIMITED_SCENARIO: SeasonScenario = SeasonScenario(
    plant_n0=6e-5,  # kg N — tiny reserve ⇒ conc in the f_N band, diluted by growth
    soil_n0=0.5,  # kg N < sn_residual (1.0) ⇒ availability ≡ 0, uptake off (dilution)
)
N_LIMITED_YEARS: int = 1

# **Water-biting** (sealed chamber, single season): the sealed water cycle made to
# **bite** instead of run inert. The frozen chambers start ``soil_water0 = 1000`` kg, so
# the closed loop (``soil_water -> water_vapor -> condensate -> soil_water``) keeps
# ``soil_water`` far above ``sw_critical = 60`` and ``f_water ≡ 1``. Here
# ``soil_water0 = 50`` kg sits inside the ``(sw_wilting, sw_critical) = (20, 60)`` band:
# the conserved loop total is only 50 kg, so ``soil_water`` settles ~40 kg (transp.
# self-limits via its own ``water_stress_factor``, so the cycle reaches a stable fixed
# point well **above** wilting — the plant survives), and ``f_water`` holds ~0.5 every
# step (probe), water-limiting gross assimilation the whole season. Ample-O2 sibling of
# the perennial chamber (``litter_carbon0 = 3``, default O2 = 210) so the carbon story
# is the clean perennial one and the water bite is the only novelty. Single season, the
# water-loop total conserved to round-off (probe drift ~4e-14), ``rationed == 0`` /
# ``events == ()`` / loss-sink ``0.0`` by construction (the first-order donor-controlled
# water flows can never overdraw). Keeps ``f_N ≡ 1`` (default ``plant_n0``) — purely
# water.
WATER_BITING_SCENARIO: SeasonScenario = SeasonScenario(
    sealed=True,
    litter_carbon0=3.0,
    soil_water0=50.0,  # kg — inside (sw_wilting, sw_critical) ⇒ f_water < 1 all season
)
WATER_BITING_YEARS: int = 1


# The post-roadmap **day-neutral** crop (the "second wheat"): an open-field plot with
# BOTH phenology modifiers OFF (``vernalization=False``, ``photoperiod=False``), so
# development advances on **thermal time alone**. It is the warm-habitat crop ceremony 2
# left open (``docs/plans/post-roadmap-oracle-match.md``): a cold-requiring winter wheat
# would never flower in a warm, lamp-lit habitat, so the habitat needs a crop with no
# cold or daylight gate. It reuses the **same cited winter-wheat crop params**
# (phenology.yaml — a day-neutral wheat is winter-wheat physiology with the gates
# removed, not a new param file), so it is **additive scenario data + its own
# diagnostic**, NOT a frozen reference and NOT an unfreeze (the N_LIMITED/WATER_BITING
# precedent; every frozen scenario keeps both modifiers ON, so their goldens are
# byte-identical). Validated as a
# DIAGNOSTIC against the bundled LINTUL3 spring-wheat oracle (a light-use-efficiency
# model, a different family — never a fit target, ruling B); see
# ``docs/plans/post-roadmap-day-neutral-crop.md`` and
# ``tests/test_oracle_gap_spring_wheat.py``.
DAY_NEUTRAL_SCENARIO: SeasonScenario = SeasonScenario(
    vernalization=False,
    photoperiod=False,
)
DAY_NEUTRAL_YEARS: int = 1


# The Phase-3 Step-6 (P3.5) drought scenario: an **open-field** plot deliberately sized
# **water-lean** so the irrigation-cut perturbation actually bites. The default open
# field starts ``soil_water0 = 1000`` kg — a store so far above the stress band
# (``sw_critical = 60``) that cutting irrigation never drops ``f_water`` below 1, i.e.
# *no cascade* (the dead-band trap the advisor flagged and the Step-6 probe confirmed).
# Here ``soil_water0 = 70`` kg sits just above ``sw_critical``: with irrigation on,
# ``soil_water`` stays ≥ critical so baseline ``f_water ≡ 1`` (no spurious baseline
# stress); cut irrigation over a window and the small daily transpiration draws
# ``soil_water`` *below* the band, so ``f_water < 1`` and assimilation falls — the
# drought cascade, emergent with no cascade code. Open field (``sealed=False``) is the
# only scenario with irrigation to cut (the sealed chamber dropped it in Step 3 for
# genuine water closure), so drought necessarily lives here. All other fields default.
DROUGHT_SCENARIO: SeasonScenario = SeasonScenario(soil_water0=70.0)
