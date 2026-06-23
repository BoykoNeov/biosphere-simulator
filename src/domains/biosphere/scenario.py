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
CONSUMER_CHAMBER_SCENARIO: SeasonScenario = SeasonScenario(
    sealed=True,
    litter_carbon0=3.0,
    consumer=True,
)
CONSUMER_CHAMBER_YEARS: int = 5

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
