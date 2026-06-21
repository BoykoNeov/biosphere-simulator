# Phase 2 — Closed Chamber / Producer + Decomposer

**Status: COMPLETE — all 7 steps landed (Steps 1–6 below; Step 7 the capstone).** The
closed chamber runs: a finite atmosphere photosynthesis draws down (feedback emerges from
stock coupling, no control code), a producer + decomposer carbon/oxygen/nitrogen loop, and
a multi-year sealed run that reproduces the **Biosphere-2 O₂-depletion failure mode** with
every-step conservation of all four quantities and `rationed == 0` from kinetics. Phase 2
changed **exactly one** core surface (P2.1, the element-composition fold); everything else
is additive. Exit criteria all met (see the bottom of this doc). All gates green (805
passed, 1 skipped; ruff/pyright clean). The phase summary lives in `MEMORY.md`; the next
phase is Phase 3 (the subsystem hierarchy / multi-compartment structure).

**Step 1 (P2.1, the element-composition core change) is COMPLETE
and landed.** Stocks now carry a `composition` map; the conservation gate folds it at both
mandatory sites (`flow.py` legs + `conservation.py` state deltas); OXYGEN is genuinely
asserted; the Phase-1 1:1 behaviour is bit-identical (goldens regenerated for the additive
`composition` block + schema v2→v3 only — zero amount drift). New `tests/test_composition.py`
pins both gate paths, the POPULATION-single-quantity invariant, construction-time validation,
and determinism; the multi-key round-trip is pinned in `test_sim_io_snapshot.py`. All gates
green (686 passed, ruff/pyright clean).

**Step 2 (P2.2, the `Ci`-from-stock seam + finite chamber) is COMPLETE and landed.** The
first emergent feedback runs with no control code: a sealed-chamber scenario sources
photosynthesis from a finite `carbon_pool` POOL (replacing the open field's *unclamped*
boundary), and FvCB derives `Ci` from the pool's live draw-down via
`chamber.ci_from_co2_pool`, read as a shared stock (#16) — the same mechanism `f_water`
uses for `soil_water`. Mechanically: the pool draws down → `Ci` falls → assimilation
collapses toward Γ\*. Sized empirically (`air_mol=1000`, `co2_mol0=0.357`, Ci₀≈250) so the
draw-down is **non-vacuous** — Ci falls ~5× (250→48), GASS collapses ~4 orders — while
`rationed == 0` holds *purely from FvCB's Ci-shutoff*, never the Euler backstop (the central
P2.2 numerical check; Phase 1 dodged a clamped pool, Step 2 re-clamps on purpose). New
`tests/test_chamber.py` (20 tests) pins the conversion, the forcing-vs-pool seam + its
all-or-nothing guard, the monotone draw-down, the meaningful Ci fall, the GASS collapse +
liveness, and total-carbon conservation; the open-field season + its regression golden are
**bit-identical** (the chamber is a `sealed` scenario variant — `build_season` is
parametrized, not forked). All gates green (707 passed, ruff/pyright clean). **Three
deliberate scope refinements vs the plan wording, all advisor-reviewed (see the Step-2
design section): (1)** the pool is named honestly as a single-currency `carbon_pool`
(`{CARBON:1}`), *not* a CO₂ stock — it is promoted to `{CARBON:1, OXYGEN:2}` + an O₂
counterpart at Step 3, when flows go multi-quantity (the `Ci` read needs no rework — it
reads the mol-carbon amount, unchanged by the promotion); **(2)** the O₂ POOL stock is
deferred to Step 3 (an inert O₂ pool at Step 2 would be dead weight); **(3)** the
deliverable is honestly the **draw-down decline**, not oscillation / O₂↔CO₂
anti-correlation — respiration still drains to `co2_resp`, so the chamber is not yet closed
(the return loop is Step 3).

**Step 3 (gas exchange as multi-quantity CARBON+OXYGEN flows) is COMPLETE and landed.**
The sealed `carbon_pool` is promoted to a true CO₂ stock (`{CARBON:1, OXYGEN:2}`) with an
O₂ counterpart POOL (`{OXYGEN:2}`), and the **gas loop is closed**: respiration returns
CO₂ to the pool instead of a boundary sink. Photosynthesis is now the genuine
multi-quantity flow `CO₂ → biomass + O₂` (`Allocation` deposits an O₂ leg = the carbon
fixed) and plant maintenance is `biomass + O₂ → CO₂` (the shortfall consumes O₂ = the
carbon burned), each balancing CARBON **and** OXYGEN in one flow at PQ=1 — the P1-filed
deferred multi-quantity stoichiometric flow, now real. **Key realization:** in the closed
chamber only carbon transfers that change *organ* carbon carry net O₂; the
"immediately-respired" carbon (growth respiration; the *covered* part of maintenance) is a
CO₂→CO₂ round trip with the O₂ release reconsumed — a net no-op. The flows detect
`co2_atmos == co2_resp` (closed chamber) and net those round trips away (which also avoids
`FlowResult`'s duplicate-leg guard), leaving the pool's *withdrawal* = DMI only, so
`rationed == 0` is preserved even more safely than Step 2's GASS-bounded draw. **Empirical
(305-day sealed run):** `rationed == 0`, no extinction; total OXYGEN = `2·(CO₂+O₂)` =
420.714 mol conserved to 1.7e-13 (float-exact, exercising conservation fold site 2); total
CARBON conserved to 4.4e-16; the CO₂ pool is **no longer monotone** (159 refill steps —
the closed loop made observable) but still nets a non-vacuous draw-down (Ci falls ~2.7× to
min ~92.8); O₂ stays ≫ 0 (210 → 210.2). The open-field season + its regression golden are
**bit-identical** (the closed branch is `o2_pool`/source==sink-gated; open paths preserve
exact float accumulation). New `tests/test_gas_exchange.py` (15 tests) pins per-flow
CARBON+OXYGEN balance, the closed-loop no-ops, exact OXYGEN conservation, the PQ=1
anti-correlation, and the O₂≫0 deferral guard; three Step-2 sealed tests were revised from
the superseded open-loop behavior (monotone pool / Ci, GASS collapse) to the closed-loop
reality. All gates green (724 passed, ruff/pyright clean). **One scope refinement vs the
plan, advisor-reviewed: O₂ self-limitation (`f_O2`) is DEFERRED** — at a realistic O₂ fill
(~210 mol vs O(0.1) mol fluxes) plant respiration never approaches rationing, so `f_O2`
is not yet load-bearing; it lands where O₂ actually depletes (microbial respiration, Step
5; the O₂-depletion validation, Step 7). A test pins O₂ ≫ 0 to guard that precondition.
**The gas loop is closed; the carbon loop is still open** (senescence leaks organ carbon
to `litter_sink` until the decomposer, Step 4).

**Step 4 (litter + decomposition, CARBON only) is COMPLETE and landed.** The Phase-1
`litter_sink` BOUNDARY is promoted to a finite `litter_carbon` POOL (senescence-fed,
exactly as Step 2 promoted `co2_atmos` → the finite `carbon_pool`), and a new
`Decomposition` flow `litter_carbon → microbial_carbon` transfers decaying litter into a
pure-carbon `microbial_carbon` POPULATION via first-order donor-controlled decay
(`k·litter`, Olson 1963; self-limiting like senescence, `k·dt = 0.02 ≪ 1`). It is
**deliberately single-currency CARBON** — the central scope decision, surfaced by the
user's "decomposition should also consume O₂, no?" and **confirmed correct**: aerobic
decomposition *is* microbial respiration and genuinely consumes O₂, and in this model
that coupling is not optional but **gate-forced** (CO₂ into the `{CARBON:1, OXYGEN:2}`
pool drags 2 oxygens pure-carbon litter cannot supply → the P2.1 gate would hard-fail),
so it is **sequenced to Step 5** (`microbial_C + O₂ → CO₂`), a pure *addition* that tears
up nothing here. **Empirical (305-day sealed run):** `rationed == 0`, no extinction;
litter accumulates then drains (0 → 0.106 → 0.013 mol C — non-vacuous, the emergent
behaviour); `microbial_carbon` grows monotonically (0 → 0.329, the intentional
intermediate — nothing withdraws it until Step 5); **total CARBON conserved float-exact
(2.2e-16)** (the sealed chamber has no boundary carbon source/sink — decomposition is an
internal CARBON transfer), and **OXYGEN stays exactly conserved** (decomposition never
touches the gas system — the Step-4/Step-5 split guard). The open-field season + its
regression golden are **bit-identical** (the decomposer is `sealed`-gated; Senescence's
target is parametrized `litter_carbon`/`litter_sink` exactly like `carbon_source`/
`resp_sink`). New `tests/test_decomposition.py` (16 tests) pins the rate law, the
CARBON-only leg balance, dt-linearity, the litter accumulate-then-drain, the monotone
microbial growth, exact CARBON + OXYGEN conservation, `rationed == 0`, and the
untouched open field. All gates green (741 passed, ruff/pyright clean; one pre-existing
E501 in `carbon_budget.py:60` trimmed in passing). **One scope refinement vs the plan
wording, advisor-reviewed (Option A; see the Step-4 design): the Step-4 one-liner's "+
CO₂" is ahead of itself — it needs O₂, hence Step 5; Step 4 is the carbon-only transfer.**
**Deferred seams:** microbial death / turnover (`microbial_C → litter_C` recycling),
microbe-explicit Michaelis kinetics, and the O₂-coupled microbial respiration (Step 5).

**Step 5 (microbial respiration, CARBON+OXYGEN) is COMPLETE and landed.** A new
`MicrobialRespiration` flow `microbial_carbon + O₂ → CO₂` burns microbial biomass back to
CO₂ via first-order respiration (`m_resp·microbial_C`, self-limiting; `m_resp·dt = 0.05 ≪
1`), the genuine multi-quantity (CARBON+OXYGEN) decomposer gas flux — the decomposer's
mirror of plant maintenance respiration's biomass-burned shortfall, and the chamber's
decomposer O₂ sink (the Biosphere-2 mechanism). **The carbon loop is now closed:** litter
→ microbial → CO₂ → photosynthesis (Step 4 left `microbial_carbon` a sink-only
intermediate; Step 5 gives it the CO₂-returning, O₂-consuming sink the Step-4 doc deferred
here — gate-forced, since CO₂ into the `{CARBON:1,OXYGEN:2}` pool drags 2 oxygens
pure-carbon microbes cannot supply). The three legs `(microbial −b, co2_pool +b, o2_pool
−b)` balance CARBON and OXYGEN at PQ=1 in one flow; it is **simpler** than the plant
shortfall it mirrors — sealed-only, always three legs, no `source == sink` netting
(`microbial ≠` the pool). **Empirical (305-day sealed run):** `rationed == 0`, no
extinction; total CARBON conserved float-exact (2.2e-16; microbial → pool is an internal
transfer) **and** total OXYGEN conserved float-exact (the CO₂'s 2 oxygens come from the
consumed O₂ — exercising fold site 2 through the new flux); `microbial_carbon` is now
**non-monotone** (peaks ~0.040 at day 93, drained to ~0.012; 212 draw-down steps — the
Step-4 monotone pile-up is superseded, microbial now ≈ decomposition-in/respiration-out
balanced ~0.04 vs Step-4's 0.329), and the CO₂ pool refills further so end-of-season GASS
**partially recovers** (~4% of peak, vs the mid-season collapse to 0) — the closed-carbon-
loop signature. The open-field season + its regression golden are **bit-identical** (the
flux is `sealed`-gated/appended, like `Decomposition`). **One scope refinement vs the
plan, advisor-reviewed: O₂ self-limitation (`f_O2`) is DEFERRED to Step 7** (a magnitude
bet): at the ~210 mol O₂ fill the standing microbial biomass is O(0.01) mol C, so over the
season O₂ falls only ~0.0065 mol — measured min(O₂) ≈ 0.99997·fill, ~4 orders from
rationing — exactly the Step-3 deferral logic; `f_O2` lands at Step 7's depleting
multi-year run (applied to **both** microbial and plant maintenance respiration), guarded
by `test_gas_exchange`/`test_microbial_respiration`'s O₂ ≫ 0 checks. New
`tests/test_microbial_respiration.py` (15 tests) pins the rate law, the 3-leg
CARBON+OXYGEN balance, dt-linearity, the self-limit, loader guards, and the integration
(microbial drained, CARBON+OXYGEN conserved, `rationed == 0`, no extinction, O₂ ≫ 0); two
superseded Step-4 tests (microbial monotone growth; the "decomposition never touches the
gas system" oxygen framing) were revised to the closed-loop reality, and one Step-2/3
chamber test (`test_sealed_assimilation_rises_then_declines`) was strengthened to assert
the collapse at the post-peak *trough* (GASS hits 0) with an end-of-season *recovery*. All
gates green (757 passed, ruff/pyright clean). **Deferred seams (unchanged):** microbial
death/turnover recycling, microbe-explicit Michaelis substrate kinetics, and the `f_O2`
above.

**Step 6 (mineralization, NITROGEN — the nitrogen return loop) is COMPLETE and landed.**
The carbon decomposer loop (Steps 4–5) gets its **nitrogen mirror**, closing the N cycle
Phase 1 left open. Phase 1 drained `soil_n → plant_n` by uptake, refilled `soil_n` from an
*external* `n_source` (fertilization), and left `plant_n` **monotone-growing** (nothing
withdrew it). Step 6 returns plant N to the soil internally via two single-currency
NITROGEN flows + a finite `litter_n` POOL (the N analogue of `litter_carbon`): **(1)
`NitrogenSenescence` `plant_n → litter_n`** (first-order `n_sen·plant_n`, self-limiting —
the N counterpart of carbon senescence; drains `plant_n` — no longer monotone-growing, the
consumption side the open loop lacked) and **(2) `Mineralization` `litter_n → soil_n`** (first-order
donor-controlled net mineralization `k_min·litter_n`, Stanford & Smith 1972). The loop
`soil_n → plant_n → litter_n → soil_n` now closes with no external supply (the inert
`n_source`/fertilization stays wired at rate 0). Both are **single-currency NITROGEN** (all
pools `{NITROGEN:1}`), so the gate folds them exactly like Phase 1 — **no core change**.
**Empirical (305-day sealed run):** `rationed == 0`, no extinction; `litter_n` accumulates
then drains (0 → 0.110 at day 67 → 0.058 mol — non-vacuous, the emergent N behaviour);
`plant_n` is now **drained** (0.5 → 0.166, reversing the Phase-1 monotone growth);
**total NITROGEN conserved float-exact (5.7e-14; the cycle is entirely internal)**.
**Two scope refinements vs the plan wording, advisor-reviewed: (1) DIRECT
`litter_n → soil_n`** net mineralization, **deferring** the microbe-mediated path the plan's
"litter/**microbial** N" wording implies (immobilization `litter_n → microbial_n` then
`microbial_n → soil_n` via turnover) — first-order net mineralization is the standard
minimal soil-N treatment (mirrors Step 4 deferring microbe-explicit kinetics; the
C:N-ratio-driven immobilization is the advanced seam). **(2) Mechanism, not feedback —
`f_N ≡ 1`, the carbon trajectory is bit-identical** (the f_O2-deferral mirror): at the PP
fill `plant_n` stays ~1000× above the critical-N concentration (measured min conc 0.604 vs
critical 0.0004 kg N/mol C), so `f_N` is **exactly 1.0 every step** and the N loop has
**zero effect on photosynthesis / carbon** — every prior sealed test
(`test_chamber`/`test_gas_exchange`/`test_decomposition`/`test_microbial_respiration`)
passes **unchanged** (the bit-identical proof). The deliverable is honestly **"N mass
cycles internally and is conserved,"** *not* "emergent N feedback"; the N-limited regime
(`f_N < 1` biting) is **deferred to Step 7's sized run** (exactly as Step 2 shipped
draw-down-not-oscillation and Steps 3/5 deferred `f_O2`) — and *verified* not asserted
(`test_mineralization` recomputes `f_N` each step and asserts `== 1.0`). The open-field
season + its regression golden are **bit-identical** (the loop is `sealed`-gated/appended,
like the carbon decomposer; no sealed golden exists — the sealed run is pinned
behaviourally). N-senescence is first-order in `plant_n` (the litter C:N is **emergent**
from two independent rates), not tied to the carbon-senescence flux at a fixed tissue N:C
(which would *control* litter C:N — a documented refinement, with N resorption). New
`tests/test_mineralization.py` (21 tests) pins both rate laws, the single-currency NITROGEN
balance (no C/O/W residual), dt-linearity, self-limits, loader guards (both rates'
negative/bad-unit rejection), and the integration (litter_n accumulate-then-drain, plant_n
drained, total N float-exact, `rationed == 0`, no extinction, the `f_N == 1.0` decoupling,
open field grows no `litter_n`). All gates green (779 passed, ruff/pyright clean).
**Deferred seams:** microbe-mediated N (immobilization), C:N-ratio-driven
immobilization/mineralization, N resorption, and the N-limited `f_N < 1` regime (Step 7).

**Step 7 (sealed-chamber integration + validation — the Phase-2 capstone) is COMPLETE
and landed, in two commits.** **7a — `f_O2`, the deferred O₂ self-limit** (Steps 3/5 → here):
a Monod factor in the chamber O₂ mole fraction (`chamber.oxygen_limitation_factor`,
`f_O2 = x_O2/(K_O2 + x_O2)`) throttling the O₂-consuming respiration fluxes (plant
maintenance shortfall + microbial respiration) toward 0 as O₂ → 0 — the respiratory mirror
of FvCB's Ci-shutoff. Conservation-safe (it scales the whole flow; every leg still balances
CARBON+OXYGEN — advisor-verified). Open field byte-identical (`o2_pool=None`); each
O₂-consuming flow carries its own `air_mol` basis + an `o2_half_saturation` param. `K_O2 =
1e-4` mol/mol (low/sharp — terminal-oxidase O₂ affinity, no soil-diffusion limit in a
well-mixed chamber), so `f_O2 ≈ 1` at the ~21 % PP fill (the prior sealed tests stay green)
and it is load-bearing only near anoxia. **7b — the canonical depleting multi-year run +
validation + the first sealed golden.** `SEALED_CHAMBER_SCENARIO`: a deliberately **O₂-poor**
chamber (**2 mol O₂ in 1000 mol air**, a scale choice so the tiny 1 m²-seedling fluxes
deplete O₂ non-vacuously — the Step-2 `air_mol`-probe rhythm) seeded with **3 mol C of
standing litter**, run **3 years** by tiling the season weather (`_table` reads repeated
rows in order). **The emergent three-act story (no control code):** (1) the seeded litter
decomposes → microbial respiration draws O₂ down **~99 %** to an acute trough (min O₂ ≈
0.004 of the 2-mol fill) — the **Biosphere-2 soil-respiration O₂-depletion failure mode** —
while `f_O2` self-limits the draw so **`rationed == 0` holds on the depleting pool** (the
central check; an un-throttled `K_O2 = 0` control **rations 77×** — `f_O2` is genuinely
load-bearing); (2) the live producer photosynthesises at the elevated CO₂ (Ci ≈ 1600
µmol mol⁻¹) and **transiently refills O₂** before it matures and dies — a coupled
producer × `f_O2` swing (respiration backs off at the trough, letting photosynthesis
out-pace it); (3) the plant dead, decomposition wins and the chamber settles **CO₂-rich**
(Ci ≈ 1140). **Empirical (915-step run):** all four quantities (CARBON/OXYGEN/WATER/NITROGEN)
conserved float-exact every step; `rationed == 0`; no extinction; O₂ depletes ≥ 95 % yet
stays strictly positive (the `f_O2` floor); the O₂↔CO₂ anti-correlation is exact
(`CO₂_mol + O₂_mol = const` — the OXYGEN-conservation law made *visible* by the 99 % swing,
not an independent invariant); deterministic; `f_N ≡ 1` (N stays non-limiting, verified —
the N-limited regime is deferred). New `tests/test_sealed_chamber.py` (12 tests) +
`tests/test_regression_sealed_season.py` + `sealed_chamber_state.json` (the first **sealed**
hex-float golden; the open field's golden is bit-identical). **Frozen-surface honesty:
Step 7 is ZERO `simcore` changes** (`f_O2` is domain-side, the scenario is `season.py`, the
golden is tests) — Phase 2 changed exactly one core surface (P2.1), as promised.
**Sustained multi-year oscillation is DEFERRED** (the probe found the plant dies after year 1
— DVS is monotone, no vernalization/re-sowing, and past-maturity partitioning sends no
carbon to leaves; sustained oscillation needs annual phenology reset — a Phase-3 scenario
seam). **Deferred seams:** the N-limited `f_N < 1` regime (a separate low-N sizing axis),
annual phenology reset / re-sowing (→ sustained oscillation), plant-extinction-as-failure
(brings in the boundary loss-sink, breaking strict closure — a Phase-3 multi-compartment
concern), and the permanent `storage_c` carbon residue (grain pays no maintenance / never
senesces, so a dead plant leaves locked carbon).

The design review's two corrections were folded in before
build: (1) the composition fold has **two** mandatory sites, not one — `flow.py` (legs) *and*
`conservation.py` (state deltas); missing the second falsely trips the OXYGEN gate every
photosynthesis step; (2) an O₂ self-limitation mirror (Michaelis in O₂) is required so
respiration keeps `rationed == 0` on a depleting O₂ pool (flagged for steps 3/5). PQ=1 with
pure-carbon biomass is confirmed the correct, water-untouched formulation. Phase 1 (single
producer) is complete and regression-pinned (`docs/plans/phase-1-single-producer.md`); Phase 2
is **additive to the biology** but Step 1 was the **first deliberate modification of the frozen
Phase-0 conservation core** — a user-approved exception, scoped as tightly as the science
allows (see P2.1).

**Goal (roadmap lines 247–269):** *move the system boundaries inward — external forcing
shrinks, internal feedback grows.* Seal the producer into a chamber with a finite
atmosphere and a decomposer loop, so the headline principle holds: **feedback emerges
automatically from stock coupling, with no hand-written control code** — photosynthesis
draws down a *finite* CO₂ stock, which lowers `Ci`, which weakens photosynthesis; litter
decomposes and respires CO₂ back. Validation is **qualitative reproduction of known
closed-system phenomena** (CO₂ draw-down/oscillation, O₂↔CO₂ anti-correlation, O₂
depletion, Biosphere-2-style failures). Exit: **multi-year sealed runs with stable
conservation, stable numerics, believable dynamics.**

**Source of truth for the phase sequence:** `roadmap_extracted.txt` (Phase 2 = lines
247–269). Phases 0/0.5/1 are complete and their APIs are otherwise frozen; Phase 2 changes
**exactly one** core surface (the stock→quantity multiplicity, P2.1) and is additive
everywhere else.

**Reuse/licensing:** `docs/reuse-and-licenses.md`. Clean-room from primary literature;
PCSE/WOFOST stay offline oracles only.

---

## Relationship to the roadmap + the central reframing

Roadmap Phase-2 **new stocks**: CO₂, O₂, water vapor, litter carbon, microbial biomass,
plant biomass. **New processes**: decomposition, atmospheric mixing, gas exchange,
microbial respiration, mineralization. **New principle**: no special feedback code —
feedback emerges through stock coupling.

The non-obvious architectural reframing — the thing Phase 1 explicitly *filed for Phase 2*
(P1: "the genuine multi-quantity stoichiometric flow … one atomic flow moving several
quantities at fixed ratios … deferred to Phase 2 (closed-chamber gas exchange, where
O₂/CO₂ coupling first matters and first has feedback)") — is **how gas-exchange
stoichiometry conserves more than one quantity in a single flow**. Three tensions drive the
locked decisions:

1. **Species are not conserved across reactions — only elements are.** Photosynthesis and
   respiration interconvert molecular species (CO₂, O₂, biomass), but the conservation gate
   asserts **per-quantity balance every step**. A single-currency flow cannot host
   `CO₂ ⇌ biomass + O₂`. → **P2.1: stocks gain an element-composition map; the gate folds
   it, so one flow balances CARBON *and* OXYGEN simultaneously.** This is the deferred
   multi-quantity stoichiometric flow, now built.
2. **Closed-loop feedback must emerge, not be coded.** Phase 1 read CO₂ as *forcing*
   (`ci_var` through a `Schedule`); a closed chamber reads it from a *finite stock* that
   photosynthesis draws down. → **P2.2: flip the `Ci` seam from forcing to a live CO₂-pool
   read.** No controller; the coupling is the feedback.
3. **Decomposition is single-currency and reuses Phase-1 machinery.** Litter/microbial
   carbon + mineralized nitrogen are CARBON/NITROGEN flows of exactly the Phase-1 kind;
   Phase 1's `litter_sink` BOUNDARY becomes a **live `litter_carbon` POOL** feeding them.
   → **P2.3: decomposition/microbial/mineralization add no core change** (microbial
   respiration *consuming O₂* is the one process that re-touches P2.1 — a noted dependency).

---

## Locked decisions (Phase 2)

New decisions, numbered `P2.n`, carrying the Phase-0/0.5/1 invariants they depend on.

### P2.1 — Stocks gain an **element-composition map**; the conservation gate asserts per-element balance over it. *(The load-bearing decision — the deliberate frozen-core change. Designed in full below; advisor-reviewed before any process step.)*

A `Stock` today maps **1:1** to one `Quantity` (`Stock.quantity`). Phase 2 generalizes this
to a **composition**: how much of each conserved quantity one canonical unit of the stock
*contributes* to the ledger.

  `composition: Mapping[Quantity, float]`   (default `{self.quantity: 1.0}` — 1:1, so every
  Phase-1 stock is unchanged)

- **The gas-phase carbon system carries CARBON *and* OXYGEN.** Per mol:
  - CO₂ → `{CARBON: 1, OXYGEN: 2}`
  - O₂  → `{OXYGEN: 2}`
  - plant/litter/microbial biomass → `{CARBON: 1}` (**pure carbon — unchanged from Phase 1;
    organs are already mol C with no oxygen booked**)
- **WATER and NITROGEN are untouched 1:1 molecular pools** (`{WATER: 1}` / `{NITROGEN: 1}`),
  still fully gate-asserted. *(This is the decisive simplification — see "Photosynthetic
  quotient = 1" below. The Step-1 Phase-1 lock chose `WATER = kg`, molecular, "with the
  Phase-2 molar-gas-stoichiometry cost understood"; P2.1 honors that — water never enters
  the gas stoichiometry, so it never has to be re-based to a molar element basis.)*
- **The composition fold has TWO mandatory sites** (corrected from an earlier "one change"
  claim — the every-step gate reasons over *state deltas*, a path that does **not** reuse the
  leg path, by the `conservation.py` docstring's deliberate design):
  1. **`flow.py` — leg side:** `per_quantity_residual` and the `scale` loop in
     `assert_flow_balanced`. A leg `(stock, amount)` contributes `amount · composition[q]`
     to **each** quantity `q` it carries, instead of all of `amount` to one quantity.
  2. **`conservation.py` — state-delta side:** `compute_ledger`'s `bucket[b.quantity] +=
     delta` and `assert_conserved`'s `scale[b.quantity]`. **A per-stock Δamount folds the
     same way** — a CO₂ stock's delta books to **both** CARBON and OXYGEN. *Miss this and the
     per-step OXYGEN gate falsely trips on every photosynthesis step* (CO₂'s 2 oxygens book as
     CARBON only, while O₂'s 2 oxygens reach OXYGEN → spurious residual).

  Both sites keep the per-quantity, canonical-order, tolerance-gated machinery **unchanged** —
  only the stock→quantity fan-out generalizes from 1→1 to 1→many. CARBON-only stocks fold
  identically to today.
- **Extinction routing stays single-quantity (a constraint, not a fold).** `integrator.py`
  routes an extinct POPULATION stock's residual via `loss_sink_id(stock.quantity)` (one
  nominal quantity). Multi-quantity stocks (CO₂, O₂) are **POOL/BOUNDARY, never POPULATION**,
  and biomass/microbial POPULATION stocks are pure carbon (`{CARBON: 1}`), so extinction never
  touches a multi-quantity composition. **P2.1 invariant: a POPULATION stock must be
  single-quantity** (else loss-sink routing needs its own composition fold — deferred, not
  needed). `Observation`'s `quantity=stock.quantity` (`observation.py`) is a **diagnostic
  grouping**, not a conservation fold — it keeps the nominal quantity (Observation is not
  extended in Phase 2, the aux precedent).
- **Photosynthetic quotient = 1 (why water drops out, the load-bearing simplification).**
  Real photosynthesis `CO₂ + H₂O → CH₂O + O₂` sources O₂'s oxygen from *water*. But because
  our biomass currency is **pure carbon** (mol C, no oxygen booked — the Phase-1 organ
  model), the **net** oxygen budget closes through CO₂↔O₂ *directly*, with no water term:
  - Photosynthesis `CO₂ → organ_C + O₂`: CARBON `−1+1=0` ✓; OXYGEN `−2 + 0 + 2 = 0` ✓
  - Respiration `organ_C + O₂ → CO₂`: CARBON `0` ✓; OXYGEN `−2 + 2 = 0` ✓

  i.e. **1 mol CO₂ fixed ⟷ 1 mol O₂ released** (photosynthetic/respiratory quotient = 1).
  This is the standard ecosystem-box-model treatment and the correct fidelity for "O₂
  depletion / CO₂ buildup" dynamics; the trace stoichiometric water (≪ transpiration) is
  *not* tracked, so **WATER stays a conserved molecular quantity** (no Phase-1 regression).
  **Rejected alternative (the "water declares oxygen" path):** booking O₂'s oxygen against
  consumed water requires water to carry OXYGEN composition *and* makes the WATER *molecule*
  non-conserved (photosynthesis would destroy water with no counterpart, tripping the WATER
  gate) — forcing WATER off kg or out of the asserted set. Pure-carbon biomass + PQ=1 avoids
  all of it. *(This supersedes an earlier in-conversation sketch; recorded so it is not
  re-litigated.)*
- **OXYGEN joins `ASSERTED_QUANTITIES` in practice** by getting real stocks/legs (it was
  always a `Quantity` member with canonical unit `mol`, just unused → trivially balanced).
- **Frozen-core honesty.** This *is* the "additions only" exception the user approved. It is
  scoped to: one new `Stock` field (defaulted backward-compatibly), the **two** residual-fold
  sites (`flow.py` legs + `conservation.py` state deltas), and the `sim_io` serialization of
  composition (schema bump; goldens regenerate by
  their explicit actions). The frozen `Flow`, `Integrator.step`, `Quantity`, `StockKind`,
  resolver, and arbitration are **untouched** — arbitration still scales whole flows; a
  multi-quantity flow scales all its legs together (it already supports multi-leg results).

### P2.2 — Closed-loop feedback is the **`Ci` seam flipped from forcing to a live CO₂-pool read**. *(No controller — coupling is the feedback.)*
- Phase 1's `co2_atmos` was an **unclamped BOUNDARY source** and `Ci` was a constant forcing
  `Schedule`. Phase 2 makes the chamber atmosphere a **finite POOL** (CO₂ and O₂ pools); the
  FvCB rate's `Ci` is **derived from the live CO₂-pool concentration** (mol CO₂ ÷ chamber
  air volume/amount → partial pressure → `Ci`), read through the source resolver as a
  *shared stock* (#16) — the same mechanism Phase 1 used for `f_water` reading `soil_water`.
- **Emergence, mechanically:** photosynthesis withdraws CO₂ (and deposits O₂) from finite
  pools → CO₂ concentration falls → `Ci` falls → FvCB assimilation falls. Respiration +
  decomposition return CO₂ (and consume O₂). The CO₂ draw-down/oscillation and O₂↔CO₂
  anti-correlation are **emergent**, asserted by no special code.
- **Positivity stays from kinetics (invariant #3 / P3).** A *finite* CO₂ pool that
  photosynthesis draws against must keep `rationed == 0`: the FvCB rate →0 as `Ci`→Γ\* (the
  `(Ci − Γ*)` factor, already clamped in `gross_leaf_assimilation`), so the draw is
  self-limiting → the Euler backstop stays the rare guard, not the ecological mechanism.
  *(Phase 1 dissolved the `plant_c` buffer to source carbon from an **unclamped** boundary
  precisely to dodge the backstop; Phase 2 re-clamps CO₂ on purpose and relies on FvCB's
  Ci-shutoff for self-limitation. This is the central numerical thing to verify in the
  gas-exchange step.)*
- **The O₂ mirror — respiration must self-limit on a depleting O₂ pool (JIT, steps 5/7).**
  CO₂'s self-limit (Ci-shutoff) has a mirror obligation: **respiration consuming O₂ from a
  finite, depleting O₂ pool will hit arbitration and fire the backstop** — and "O₂ depletion"
  is an *explicit* validation target, so the collision is guaranteed, not hypothetical. Plant
  **and microbial** respiration need an **O₂ self-limitation** — a Michaelis/Monod factor in
  O₂ concentration that →0 as O₂→0 (`f_O2 = O2 / (K_O2 + O2)`, cited from microbial-respiration
  literature) — exactly mirroring Ci-shutoff, so `rationed == 0` holds as O₂ depletes. Flagged
  here; designed at step 5 (microbial respiration) and applied to plant respiration in step 3.

### P2.3 — Decomposition / microbial respiration / mineralization are **single-currency flows reusing Phase-1 machinery**; `litter_sink` becomes a live `litter_carbon` POOL.
- **CARBON:** litter decomposition (`litter_carbon → microbial_carbon` + `→ CO₂` via
  microbial *growth* + *respiration*), with first-order/Michaelis kinetics from primary
  decomposition literature (cited; clean-room). These are exactly the Phase-1 single-currency
  carbon-flow kind — no core change.
- **NITROGEN:** mineralization releases litter/microbial N to a soil-mineral-N pool
  (`soil_n`), closing the loop that Phase 1 fed from an external `n_source`. Single-currency
  NITROGEN flows.
- **The one P2.1 dependency:** **microbial respiration consumes O₂** (`microbial_C + O₂ →
  CO₂`), so it is a multi-quantity (CARBON+OXYGEN) flow under P2.1 — built *after* the
  composition core lands. Decomposition's carbon/nitrogen transfers themselves need no O₂.
- **Atmospheric mixing** (roadmap) is, in a well-mixed 0-D chamber, **trivial/degenerate**
  (one well-mixed atmosphere pool); explicit mixing flows are deferred until multi-compartment
  structure exists (Phase 3's hierarchy). Phase 2 keeps 0-D well-mixed — noted, not built.

### Carried Phase-0/0.5/1 invariants that constrain Phase 2
- **Core purity (#11):** `simcore` stays stdlib-only, incl. the P2.1 composition change.
  Crop/decomposition flows/params/loaders stay in `domains/biosphere/`.
- **Canonical order on every reduction (#15):** the composition fold sums per quantity in
  canonical `Quantity` order; new flows/stocks register in canonical id order.
- **Determinism — bit-identical within a build (#7):** sealed runs are bit-identical and
  registration-order-independent; new goldens use hex-float.
- **Every-step conservation gate (#13):** unchanged in *form*; P2.1 only generalizes the
  leg→quantity fan-out. The gate now genuinely asserts OXYGEN (real stocks exist).
- **Extinction conserves mass (#6):** POPULATION biomass (plant, microbial) below threshold
  → 0 with residual to the loss-sink; POOL stocks (CO₂/O₂/litter) never zeroed-with-loss.
- **Arbitration backstop is Euler-only and rare (#3):** golden asserts `rationed == 0`;
  positivity from kinetics (FvCB Ci-shutoff on the now-finite CO₂ pool — P2.2).

---

## Scope

### In scope (Phase 2)
- **Foundation (P2.1):** stock element-composition + the gate fold; the genuine
  multi-quantity stoichiometric gas-exchange flow.
- **A finite chamber atmosphere:** CO₂ + O₂ POOL stocks; the `Ci`-from-stock seam (P2.2);
  emergent CO₂ draw-down / O₂↔CO₂ anti-correlation.
- **Gas exchange:** photosynthesis (`CO₂ → biomass + O₂`) and plant respiration
  (`biomass + O₂ → CO₂`) reworked as multi-quantity flows sourcing/sinking the finite pools.
- **Decomposer loop:** `litter_carbon` POOL + microbial biomass; decomposition, microbial
  respiration (CARBON+OXYGEN), mineralization (NITROGEN → `soil_n`).
- **Sealed-chamber scenario + validation:** a multi-year sealed run; conservation +
  `rationed == 0` every step; qualitative match to known closed-system phenomena; hex-float
  golden.

### Explicitly deferred (do NOT build in Phase 2)
- **Full element re-basing (C/H/O/N in mol), molar water, hydrogen tracking** → not needed
  under PQ=1 + pure-carbon biomass (P2.1); a noted future enhancement only if a process
  requires water/hydrogen stoichiometry.
- **Multi-compartment hierarchy / explicit atmospheric mixing / water recycling as cycles**
  → Phase 3 (the subsystem hierarchy). Phase 2 stays 0-D well-mixed.
- **Consumers / herbivory / trophic levels** → Phase 3+.
- **Energy closure** → still ENERGY-as-diagnostic (decision #8); Phases 5/6.
- **Typed per-leg dimensional signatures on `Flow`** → still deferred (P4); composition is a
  per-*stock* element map, not a per-leg dimensional check.

---

## API additions (additive except the P2.1 core fold)

```python
# --- simcore/quantities.py or state.py : Stock gains element composition (P2.1) -----
@dataclass(frozen=True)
class Stock:
    ...                                   # frozen Phase-0 fields unchanged
    composition: Mapping[Quantity, float] = <default {quantity: 1.0}>
    # moles of each conserved quantity per canonical unit of this stock. Default keeps
    # the 1:1 Phase-1 behavior (every existing stock unchanged). CO2={CARBON:1,OXYGEN:2},
    # O2={OXYGEN:2}, biomass={CARBON:1}. Validated: keys ⊆ Quantity; finite; the
    # self-quantity key present.

# --- TWO fold sites (the every-step gate reasons over state deltas, a separate path) ----
# (1) simcore/flow.py : per_quantity_residual + assert_flow_balanced.scale (leg side)
#   for leg in result.legs:
#       for q, coeff in stocks[leg.stock].composition.items():   # was: q = stock.quantity
#           residual[q] += leg.amount * coeff                     #       residual[q] += amount
# (2) simcore/conservation.py : compute_ledger buckets + assert_conserved.scale (delta side)
#   for q, coeff in before.stocks[sid].composition.items():      # was: bucket[b.quantity]
#       bucket[q] = bucket.get(q, 0.0) + delta * coeff           #       += delta
# Both keep per-quantity / canonical order / BALANCE_ATOL/RTOL untouched; only the
# stock->quantity fan-out generalizes. compute_ledger does NOT reuse the leg path
# (its docstring is explicit) — it needs its own fold or OXYGEN falsely trips.

# --- sim_io/snapshot.py : serialize composition (key-sorted, hex-float coeffs); schema bump
# --- domains/biosphere : GasExchange flow(s) — multi-quantity legs (CARBON+OXYGEN);
#     finite CO2/O2 pools; litter_carbon + microbial stocks; decomposition/mineralization.
```

No change to `Flow`, `Integrator.step`, `Quantity` membership, `StockKind`, the resolver, or
arbitration.

---

## Step sequence

**Foundation — designed in full (P2.1) and reviewed/built before the process steps, the
Phase-1 rhythm.**

1. ~~**Element-composition core change (P2.1)**~~ — **DONE.** `Stock.composition` (default
   1:1, `hash=False`, validated incl. POPULATION-single-quantity), both fold sites
   (`flow.per_quantity_residual`/`assert_flow_balanced` + `conservation.compute_ledger`/
   `assert_conserved`), `sim_io` serialization (key-sorted hex-float) + schema v2→v3, four
   goldens regenerated (diff = additive `composition` block + version only). Tests
   (`tests/test_composition.py`): 1:1 default reproduces Phase-1 exactly; CO₂→biomass+O₂
   balances CARBON and OXYGEN on **both** gate paths; mis-stoichiometric O₂ trips the gate;
   order-independence; validation guards. **Pure infra — no biology yet.**

**Process steps — enumerated now, each designed just-in-time (Phase-1 rhythm).**

2. **Finite chamber atmosphere + the `Ci`-from-stock seam (P2.2)** — CO₂/O₂ POOL stocks;
   flip `Ci` from forcing to the live CO₂-pool read; verify FvCB self-limits the finite
   pool (`rationed == 0`). The first emergent feedback.
3. ~~**Gas exchange as multi-quantity flows**~~ — **DONE.** Assimilation
   (`CO₂ → biomass + O₂`) and plant maintenance (`biomass + O₂ → CO₂`) reworked onto the
   finite CO₂/O₂ pools with CARBON+OXYGEN legs (PQ=1); the gas loop closed (respiration
   returns CO₂ to the pool). The assimilate-respired round trips (growth respiration; the
   *covered* maintenance) net to no-ops (`source == sink` detection). Total OXYGEN
   float-exact; `rationed == 0`; open-field golden bit-identical. `f_O2` O₂ self-limitation
   deferred (O₂ ≫ rationing at the realistic fill; lands at Step 5/7), guarded by a test.
4. ~~**Litter + decomposition (CARBON)**~~ — **DONE.** `litter_sink` → live
   `litter_carbon` POOL (senescence-fed); first-order donor-controlled decay (Olson 1963)
   `litter_carbon → microbial_carbon` (pure-carbon POPULATION). **Single-currency CARBON**
   — the CO₂-releasing, O₂-consuming microbial respiration is **Step 5** (gate-forced: CO₂
   into the multi-quantity pool needs O₂; Option A, advisor-reviewed). `rationed == 0`,
   total CARBON + OXYGEN float-exact, open-field golden bit-identical; 16 tests.
5. ~~**Microbial respiration (CARBON+OXYGEN)**~~ — **DONE.** `MicrobialRespiration` flow
   `microbial_carbon + O₂ → CO₂` (first-order `m_resp·microbial_C`, self-limiting), the
   multi-quantity decomposer gas flux closing the carbon loop (litter → microbial → CO₂ →
   photosynthesis) and the chamber's decomposer O₂ sink. Sealed-only, always three legs,
   no `source == sink` netting. `rationed == 0`, total CARBON + OXYGEN float-exact,
   open-field golden bit-identical; microbial is now non-monotone (respired); 15 tests.
   `f_O2` O₂ self-limitation **deferred to Step 7** (O₂ stays ≈ 0.99997·fill — ~4 orders
   from rationing; the Step-3 deferral logic), guarded by an O₂ ≫ 0 test.
6. ~~**Mineralization (NITROGEN)**~~ — **DONE.** A finite `litter_n` POOL (the N analogue
   of `litter_carbon`) + two single-currency NITROGEN flows: `NitrogenSenescence`
   (`plant_n → litter_n`, first-order, self-limiting — the N counterpart of carbon
   senescence) and `Mineralization` (`litter_n → soil_n`, first-order donor-controlled net
   mineralization, Stanford & Smith 1972). Closes `soil_n → plant_n → litter_n → soil_n`
   internally (Phase 1 fed it from the external `n_source`). **DIRECT `litter_n → soil_n`**
   (microbe-mediated N deferred) and **`f_N ≡ 1` / carbon bit-identical** (mechanism not
   feedback; the N-limited regime deferred to Step 7 — the f_O2-deferral mirror, verified
   per-step). `rationed == 0`, total NITROGEN float-exact, open-field golden bit-identical;
   21 tests.
7. **Sealed-chamber integration + validation** — assemble the multi-year sealed season;
   conservation + `rationed == 0`; qualitative closed-system phenomena (CO₂ oscillation,
   O₂↔CO₂ anti-correlation, an O₂-depletion failure mode); hex-float golden; frozen-surface
   notes; `MEMORY.md`/docs.

---

## Step 1 design — the element-composition core change (P2.1)

*Realizes P2.1. Tightest constraint first: every gas-exchange and microbial-respiration flow
depends on the gate folding composition. Pure infra — built and tested with synthetic stocks
before any Phase-2 biology, exactly as Phase-1 Step 2 (the aux channel) was.*

**The change, minimized.**
- **`Stock.composition: Mapping[Quantity, float]`** — additive field, default
  `MappingProxyType({self.quantity: 1.0})` set in `__post_init__` when not supplied, so
  positional construction and every Phase-1 call site/golden are unchanged. Validated:
  every key is a `Quantity`; every coeff is finite (the `Stock.amount` isfinite discipline);
  the stock's own `quantity` key is present with a positive coeff (so a stock always
  contributes to its nominal quantity). Read-only, detached from the caller dict (like
  `stocks`/`aux`).
- **Fold site 1 — `per_quantity_residual` + `scale` (flow.py, leg side).** Today:
  `residual[stocks[leg.stock].quantity] += leg.amount`. After: loop the leg's stock
  composition, `residual[q] += leg.amount * coeff`, summing in canonical `Quantity` order
  (#15). `scale[q]` (the tolerance denominator in `assert_flow_balanced`) folds the same way
  (`max(scale[q], abs(leg.amount * coeff))`). The per-quantity loop over
  `ASSERTED_QUANTITIES` and the `BALANCE_ATOL/RTOL` tolerance are byte-for-byte the same.
- **Fold site 2 — `compute_ledger` + `assert_conserved.scale` (conservation.py, state-delta
  side) — the site the "one change" claim missed.** This module **deliberately does not reuse
  the flow path** (its docstring: "reasons about state deltas, not flows"), so it needs its
  **own** fold: today `bucket[b.quantity] += delta` (line 93) and `scale[b.quantity]` (line
  142); after, loop `before.stocks[sid].composition` and add `delta * coeff` / fold the scale
  per quantity, still in sorted stock-id order (#15, the bit-identity reduction). The
  before/after **stock key-set** assertion is unchanged (composition lives *inside* a stock,
  not a new stock). **Without this fold a CO₂ stock's Δamount books entirely as CARBON and
  the per-step OXYGEN gate falsely trips every photosynthesis step** — the central correctness
  point of Step 1.
- **`sim_io.snapshot`** — serialize `composition` as a key-sorted object of hex-float coeffs
  (same exactness/order discipline as `aux`/amounts); `SCHEMA_VERSION` bump; goldens
  regenerate via their explicit `_regenerate`/`__main__` actions (diff = only the new
  `composition` block + version; no amount drifts — a 1:1 stock serializes
  `{"<its quantity>": "0x1p+0"}`).

**Why this is safe / behavior-preserving for Phase 1.** Every Phase-1 stock defaults to
`{quantity: 1.0}`, so `leg.amount * 1.0` to the one quantity == today's behavior exactly;
the demo + season goldens are bit-identical apart from the additive serialized `composition`
block (an explicit regeneration, the established golden discipline). The frozen `Flow`,
`Integrator`, `arbitration`, and resolver never see composition — it is read only inside the
conservation residual/ledger fold.

**Test plan.**
- **1:1 equivalence:** a stock with default composition produces byte-identical residual/
  ledger results to the pre-P2.1 path (the Phase-1 goldens, regenerated only for the
  serialized field, are otherwise unchanged).
- **Multi-quantity balance — BOTH gate paths.** (a) *Leg side:* a synthetic
  `CO₂ → biomass + O₂` flow (CO₂`{C:1,O:2}`, biomass`{C:1}`, O₂`{O:2}`) balances **both**
  CARBON and OXYGEN via `assert_flow_balanced`; a respiration flow reverses it. (b) *State-
  delta side:* a `before → after` where the CO₂ pool drops and the O₂ pool rises by the PQ=1
  amounts passes `assert_conserved` for **both** CARBON and OXYGEN — the test that pins fold
  site 2 (a regression here would be the false-OXYGEN-trip bug). A control with the O₂ rise
  perturbed off-stoichiometry trips the OXYGEN gate.
- **Mis-stoichiometry trips the gate:** an O₂ leg of the wrong coefficient (e.g. 0.5 mol O₂
  per mol CO₂) leaves OXYGEN unbalanced → `ConservationError` (the gate doing its job — the
  whole point of element accounting over a silent factor).
- **Determinism / order independence:** the composition fold is canonical-`Quantity`-ordered;
  residuals are registration-order-independent (Hypothesis).
- **Purity + frozen surface:** the AST purity gate stays green; Phase-0/0.5 analytic
  convergence/order + 100k stability gates untouched (P2.1 is conservation-side only).

---

## Step 2 design — the finite chamber atmosphere + the `Ci`-from-stock seam (P2.2)

*Realizes P2.2 (designed JIT, the Phase-1 rhythm; advisor-reviewed before build). The first
emergent feedback: flip `Ci` from a constant forcing to a live read of a finite carbon pool
photosynthesis draws down — coupling **is** the feedback, no controller.*

**The change, minimized.**
- **`chamber.ci_from_co2_pool(co2_mol, *, air_mol, ci_ratio)`** (new, pure stdlib) — the
  amount→`Ci` conversion: `Ca = co2_mol / air_mol · 1e6` (chamber CO₂ mole fraction,
  µmol mol⁻¹), `Ci = ci_ratio · Ca` (the fixed C3 `Ci/Ca ≈ 0.7` stomatal set point;
  Farquhar & Sharkey 1982). The resolver's `#16` shared-stock read returns the raw stock
  *amount*, so the transform must live in domain code (it cannot live in the resolver).
- **`CarbonContext` Ci-source seam** — three additive fields (`co2_pool_var`,
  `chamber_air_mol`, `ci_ratio`), all defaulting `None`. `_ci(env)`: when `co2_pool_var` is
  None (open field) it returns the `ci_var` forcing read (**Phase-1 behaviour exactly — the
  regression golden is unchanged**); when set (sealed) it derives `Ci` from the pool amount
  read via `co2_pool_var`. An `__post_init__` guard makes the triple all-or-nothing (a
  partial wiring is a build bug). `budget` calls `self._ci(env)` in place of
  `env.get(self.ci_var)`.
- **`build_season` parametrized** (not forked) — `SeasonScenario.sealed` + chamber fields.
  Sealed swaps the unclamped `co2_atmos` BOUNDARY source for a finite `carbon_pool` POOL
  (`{CARBON:1}`), wires every gas-exchange flow's carbon source to it (by id — the
  single-currency legs are byte-identical, only the source's *clamping* changes), passes the
  Ci-source triple into the context, and adds `co2_pool → carbon_pool` to the resolver's
  `shared` map. Open field is untouched.

**Why `rationed == 0` is non-vacuous (the central numerical check).** Phase 1 sourced carbon
from an *unclamped* boundary precisely to dodge the Euler backstop. Step 2 re-clamps on
purpose: the finite POOL is throttleable, so a withdrawal exceeding the start-of-step amount
*would* ration. It never does — FvCB's `(Ci − Γ*)` shutoff drives gross assimilation (and
thus the carbon draw) → 0 as `Ci → Γ*`, so the pool self-limits at its floor and is never
over-drawn. A chamber sized so large that `Ci` barely moves would pass `rationed == 0`
trivially and verify nothing, so the pool is **sized empirically** (a probe sweep over
`air_mol`) to land in the regime where the draw-down is real: `air_mol = 1000`,
`co2_mol0 = 0.357` (Ci₀ ≈ 250 for continuity with the Phase-1 forcing). Over the committed
weather the pool falls 0.357 → 0.069 mol C, `Ci` 250 → 48 (~5×, near the Γ\*≈42.75 floor),
GASS collapses ~4 orders, `rationed == 0`, no extinction. (Too-small `air_mol≈50` Euler-
overshoots `Ci` below Γ\* in one daily step; too-large barely dents `Ci` — both rejected by
the probe.)

**Three deliberate scope refinements vs the plan's "CO₂/O₂ POOL stocks at step 2" wording**
(advisor-reviewed; recorded so they are not re-litigated):
1. **Honest naming.** The pool is a single-currency `carbon_pool` (`{CARBON:1}`), not a
   molecular CO₂ stock. It is promoted to `{CARBON:1, OXYGEN:2}` with an O₂ counterpart at
   **Step 3**, when the flows go multi-quantity. `ci_from_co2_pool` reads the pool's
   mol-carbon amount, which the promotion leaves unchanged → **the seam needs no rework.**
2. **O₂ pool deferred to Step 3.** An inert O₂ pool at Step 2 (nothing reads/writes it)
   would be dead weight; it lands where it is first used (microbial/plant respiration).
3. **Draw-down decline, not oscillation.** Respiration still drains to the `co2_resp` sink
   (no return path), so the chamber is **not** closed — the pool is a *monotonic draw-down*.
   The O₂↔CO₂ anti-correlation / oscillation phenomena need Step 3's return loop and are
   **not** claimed here.

**Test plan (`tests/test_chamber.py`, 20 tests, all green).** (a) *Conversion:* hand value,
linearity, zero-carbon→Ci 0, and rejection of degenerate chamber / negative / non-finite
inputs. (b) *Seam:* open-field reads the `ci` forcing; sealed derives `Ci` from the pool;
the partial-triple guard. (c) *Integration (the sealed season):* `rationed == 0` across the
draw-down (the non-vacuous self-limit), no extinction, pool monotone non-increasing with a
net draw-down, `Ci` monotone non-increasing and `< 0.5 · Ci₀` (meaningful), GASS peaks
(liveness) then collapses `< 1e-3 · peak` (the feedback), total CARBON invariant across all
stocks (the sealed chamber has no boundary carbon *source*), and the open-field path keeps
`co2_atmos` / grows no `carbon_pool` (golden untouched).

---

## Step 4 design — litter + decomposition (CARBON only; the first decomposer, P2.3)

*Realizes the CARBON half of P2.3 (designed JIT, the Phase-1 rhythm; advisor-reviewed
before build). The producer half (gas exchange) now has its mirror: dead organic carbon
re-enters the cycle — but **only the carbon-only transfer** this step; the O₂-coupled
respiration is Step 5.*

**The user's question — and why it decides the scope.** The user asked "decomposition
should also consume some oxygen, no?" — and they are exactly right. Aerobic decomposition
*is* microbial respiration (`organic-C + O₂ → CO₂`). In this model the coupling is not a
realism nicety but **gate-forced**: `litter_carbon`/`microbial_carbon` are pure carbon
(`{CARBON:1}`), the chamber `carbon_pool` is `{CARBON:1, OXYGEN:2}`, so **any CO₂
deposited into the pool drags 2 oxygens the litter cannot supply** — they can only come
from the O₂ pool, exactly like plant maintenance respiration (`organ_C + O₂ → CO₂`). The
P2.1 composition gate hard-fails otherwise. So the user's intuition is honored by
*sequencing* the O₂ consumption to Step 5, not by skipping it.

**The fork + the locked choice (Option A, advisor-reviewed).** The plan labels Step 4
"(CARBON)" and Step 5 "(CARBON+OXYGEN)", and states "Decomposition's carbon/nitrogen
transfers themselves need no O₂" — yet the Step-4 one-liner says "to microbial biomass +
CO₂". Those conflict, because CO₂-into-the-pool needs O₂. Three resolutions:
- **(A) Step 4 = litter→microbial carbon transfer only** (first-order `k·litter`, pure
  CARBON). All CO₂ release + O₂ consumption is Step 5's `microbial_C + O₂ → CO₂`. **CHOSEN.**
- (B) Step 4 splits decay into CUE→microbial + (1−CUE)→a single-currency `{CARBON:1}`
  boundary CO₂ sink, rewired into pool+O₂ at Step 5. Matches "+ CO₂" literally but ships
  scaffolding torn up one step later (the open→closed rework the producer already did,
  Steps 2→3).
- (C) Step 4 already consumes O₂ for the respired fraction. Pulls Step 5 forward,
  contradicts the deliberate split + the `f_O2` deferral. **Rejected.**

The discriminator is **additive-vs-rework into Step 5**: under A, Step 5 *purely adds*
the respiration; Step 4 ships nothing Step 5 demolishes. A also matches the authoritative
P2.3 sentence verbatim. The "+ CO₂" in the Step-4 one-liner is the part that is ahead of
itself. (Consequence: `microbial_carbon` only **grows** this step — decay deposits,
nothing withdraws until Step 5 — an intentional intermediate, like `plant_n` only growing
in Phase 1; mirrors the producer rhythm Step 2 open draw-down → Step 3 closed loop.)

**The change, minimized.**
- **`decomposition.py`** (new, pure stdlib) — `decomposition_flux(litter_c, k) = k·litter_c`
  (Olson 1963 first-order donor-controlled decay; self-limiting → 0 as litter → 0, so
  positivity is structural — the senescence/respiration pattern) + the `Decomposition`
  flow `litter_carbon → microbial_carbon` (single-currency CARBON; both pools `{CARBON:1}`,
  so the gate folds it identically to Phase 1 and no O₂ appears). `flux = daily·dt`.
- **`params/decomposition.yaml` + `load_decomposition_params`** — the first-order rate `k`
  (1/day), the value/unit/source template + exact-string unit guard + non-negative bound
  (the senescence loader discipline). Provisional `k = 0.02` (turnover ~50 days),
  TODO(cite) Olson 1963.
- **`season.py` (sealed-gated, golden-safe)** — add `litter_carbon` (POOL, 0) +
  `microbial_carbon` (pure-carbon POPULATION, 0, `threshold = 0` — sink-only this step,
  so `0 < 0` never snaps; the existing CARBON loss-sink already covers it once Step 5 makes
  it a source). Senescence's destination is parametrized `litter_target = LITTER_CARBON if
  sealed else LITTER_SINK` (exactly like `carbon_source`/`resp_sink`); `LITTER_SINK` moves
  into the open-field-only branch; the `Decomposition` flow is appended only when sealed
  (the Registry sorts by id → order-independent). **Open field is byte-identical** — the
  regression golden is untouched.

**Why `rationed == 0` (the backstop guard).** First-order `k·litter_c·dt` withdraws against
the **start-of-step** litter amount; senescence's same-step inflow doesn't count (the
arbitration memory), but `k·dt = 0.02 ≪ 1` so the draw never exceeds the pool — the Euler
backstop stays unfired, mirroring senescence's `rate·organ_c`. (Microbe-explicit Michaelis
kinetics would need a microbial seed + active decomposers; deferred — first-order donor
control is the right minimal Step-4 pick.)

**Test plan (`tests/test_decomposition.py`, 16 tests, all green).** (a) *Rate law:*
first-order in litter, zero at zero litter. (b) *Flow:* litter→microbial transfer of the
same amount, CARBON-only balance (no OXYGEN/WATER/NITROGEN residual), dt-linearity,
self-limit at zero litter. (c) *Loader:* committed rate, negative-rate + bad-unit
rejection. (d) *Integration (sealed season):* litter accumulates then drains (non-vacuous),
microbial grows monotonically, **total CARBON conserved exactly**, **OXYGEN still exactly
conserved** (the Step-4/Step-5 split guard — an O₂ leak into Step 4 would break it),
`rationed == 0`, no extinction, and the open field grows no decomposer stocks (keeps
`litter_sink`).

---

## Step 5 design — microbial respiration (CARBON+OXYGEN; the decomposer gas flux, P2.3)

*Realizes the O₂-coupled half of P2.3 (designed JIT, the Phase-1 rhythm; advisor-reviewed
before build). Step 4 left `microbial_carbon` a sink-only intermediate; Step 5 gives it
the CO₂-returning, O₂-consuming sink, **closing the carbon loop** (litter → microbial →
CO₂ → photosynthesis) — a pure addition that tears up nothing in Step 4.*

**The flow, minimized.** A new `microbial_respiration.py` (pure stdlib): the rate law
`microbial_respiration_flux(microbial_c, m_resp) = m_resp · microbial_c` (mol C day⁻¹;
first-order in standing microbial biomass, self-limiting → 0 as microbial → 0 — the
maintenance/decomposition positivity pattern) + the `MicrobialRespiration` flow
`microbial_carbon + o2_pool → carbon_pool`. The three legs `(microbial −b, co2_pool +b,
o2_pool −b)` balance CARBON (`−b + b = 0`) **and** OXYGEN (the pool's `+2b` vs the O₂
pool's `−2b`, via the P2.1 composition fold) at PQ=1 — the decomposer's mirror of plant
maintenance respiration's biomass-burned shortfall, but **simpler**: microbial biomass and
the CO₂/O₂ pools exist only when sealed, so the flow is sealed-only, **always three legs**,
with no `source == sink` netting (`microbial ≠` the pool, unlike the plant *covered*
CO₂→CO₂ round trip). `params/microbial_respiration.yaml` (provisional `m_resp = 0.05`/day,
turnover ~20 days, `TODO(cite)`) + `load_microbial_respiration_params` (exact-unit guard,
non-negative — the decomposition loader discipline). `season.py` appends it sealed-only
(like `Decomposition`; Registry sorts by id → order-independent).

**The rate-law fork (first-order vs CUE), locked.** First-order microbial respiration
(maintenance/turnover) is chosen over a carbon-use-efficiency split on the decomposition
flux: CUE `(1−CUE)→CO₂` would **rework Step 4** (the rejected Option B — Step 4 deposits
100% of decay into microbial biomass), whereas first-order respiration is a **pure
addition** draining the standing pool. It also matches the plan's `microbial_C + O₂ → CO₂`
verbatim. Microbe-explicit Michaelis substrate kinetics (`Vmax·microbial·litter/(K_m+
litter)`) stay deferred (need a microbial seed; first-order donor/standing control is the
right minimal pick — the Step-4 rhythm).

**The `f_O2` deferral (the magnitude bet, advisor-reviewed; one scope refinement vs the
plan's "designed at step 5").** P2.2 flags an O₂ Michaelis factor `f_O2 = O2/(K_O2+O2)` so
respiration keeps `rationed == 0` on a *depleting* O₂ pool. Microbial respiration's O₂ draw
`m_resp·microbial_C` is **not** self-limiting on the O₂ pool, so a small enough fill *would*
ration — but at the realistic ~210 mol O₂ fill the standing microbial biomass is O(0.01)
mol C, so over the 305-day season O₂ falls only ~0.0065 mol (**measured min(O₂) ≈
0.99997·fill, ~4 orders from rationing**). So `f_O2` would be ≈ 1 throughout and
untestable-in-anger here — **deferred to Step 7**, where the multi-year run is *sized to
deplete* O₂ (the explicit O₂-depletion target) and `f_O2` is applied to **both** microbial
and plant maintenance respiration. This exactly mirrors the Step-3 deferral (the plan's
"designed at step 5" wording predates the Step-3 magnitude discovery); it is guarded by the
O₂ ≫ 0 checks in `test_gas_exchange`/`test_microbial_respiration` — if a future change
pushes O₂ toward its floor, those break and flag `f_O2` has become load-bearing.

**Test plan (`tests/test_microbial_respiration.py`, 15 tests, all green).** (a) *Rate law:*
first-order in microbial biomass, zero at zero. (b) *Flow:* the 3-leg `microbial → CO₂ +
O₂-consumed` transfer of the same amount, CARBON **and** OXYGEN balance (no WATER/NITROGEN
residual), dt-linearity, self-limit at zero microbial. (c) *Loader:* committed rate,
negative-rate + bad-unit rejection. (d) *Integration (sealed season):* microbial respired
(a strict draw-down — the Step-4 monotone claim superseded), **total CARBON + OXYGEN
conserved float-exact** (the closed loop; OXYGEN through the new sink), `rationed == 0`, no
extinction, and O₂ ≫ 0 (the `f_O2`-deferral guard). Two superseded Step-4 tests (microbial
monotone growth; the "decomposition never touches the gas system" oxygen framing) and one
Step-2/3 chamber test (GASS collapse asserted at the *end* — now a mid-season *trough* with
an end-of-season *recovery*, the closed-carbon-loop signature) were revised, not weakened.
**Open field is byte-identical** — the flux is `sealed`-gated/appended; the regression
golden is untouched.

---

## Step 6 design — mineralization (NITROGEN; the nitrogen return loop, P2.3)

*Realizes the NITROGEN half of P2.3 (designed JIT, the Phase-1 rhythm; advisor-reviewed
before build). The carbon decomposer loop (Steps 4–5) gets its nitrogen mirror: the cycle
`soil_n → plant_n → litter_n → soil_n` closes internally, replacing the external `n_source`
Phase 1 fed it from.*

**The central scope realization — N must first ENTER an N pool.** `litter_carbon` and
`microbial_carbon` are **pure CARBON** (`{CARBON:1}`) — they hold no nitrogen. So
"mineralization (litter/microbial N → `soil_n`)" is **vacuous** unless organic N first
exists. In Phase 1 `plant_n` only **grew** (uptake fed it; nothing withdrew it). So a
faithful Step 6 needs both a **source** (the plant must shed N when it senesces) and a
**return** (mineralization). This is irreducibly two transfers + one intermediate pool —
larger than Steps 4/5, but it is the nature of closing the loop.

**The change, minimized.** A new `mineralization.py` (pure stdlib) holding **both** halves
of the N return loop (the `nitrogen.py` precedent — it holds uptake + fertilization):
- **`nitrogen_senescence_flux(plant_n, n_sen) = n_sen·plant_n`** + the `NitrogenSenescence`
  flow `plant_n → litter_n` (first-order in whole-plant N, self-limiting → 0 as `plant_n`
  → 0; the carbon-senescence positivity pattern). **Drains** `plant_n` (no longer
  monotone-growing) — the consumption side the open Phase-1 N loop lacked.
- **`mineralization_flux(litter_n, k_min) = k_min·litter_n`** + the `Mineralization` flow
  `litter_n → soil_n` (first-order donor-controlled net mineralization, Stanford & Smith
  1972; self-limiting). The DIRECT release of mineral N back to soil.
- Both **single-currency NITROGEN** (`litter_n`/`soil_n`/`plant_n` are all `{NITROGEN:1}`),
  so the gate folds them exactly like Phase 1 — **no core change**.
- **`params/mineralization.yaml` + `load_mineralization_params`** — both first-order rates
  (`n_senescence_rate`, `mineralization_rate`, 1/day) in one honestly-named file (the N
  return loop; keeps `senescence.yaml`/`SenescenceParams` byte-untouched — the surgical /
  golden-safe choice), each value/unit/source-templated, exact-unit-guarded, non-negative
  bound (the decomposition/microbial-respiration loader discipline). Provisional
  `n_sen = 0.01`, `k_min = 0.03`, `TODO(cite)`.
- **`season.py` (sealed-gated, golden-safe)** — add `litter_n` (POOL, 0) sealed-only; append
  `NitrogenSenescence` + `Mineralization` sealed-only (id order; Registry sorts → order-
  independent). The carbon `Senescence` flow is **untouched** (N-senescence is a *separate*
  flow, not an N leg bolted onto it). Open field byte-identical.

**The two scope refinements (advisor-reviewed; recorded so they are not re-litigated).**
1. **DIRECT `litter_n → soil_n`, deferring microbe-mediated N.** The plan says "litter/
   **microbial** N → soil_n"; this ships the direct first-order net mineralization and
   defers the microbe-mediated path (N immobilization `litter_n → microbial_n` during
   decomposition, then `microbial_n → soil_n` via microbial turnover). First-order net
   mineralization is the standard minimal soil-N treatment — exactly as Step 4 chose
   first-order donor decay over microbe-explicit Michaelis kinetics; the C:N-ratio-driven
   immobilization is the advanced refinement seam.
2. **Mechanism, not feedback — `f_N ≡ 1`, the carbon trajectory bit-identical (the f_O2
   mirror).** With the chamber sized for potential production (PP, non-limiting N),
   `plant_n` stays ~1000× above the critical-N concentration (measured min conc 0.604 vs
   critical 0.0004 kg N/mol C), so `f_N` is **exactly 1.0 every step** and the N loop is a
   **parallel cycle with zero effect on the carbon / plant trajectory**. So the deliverable
   is honestly **"nitrogen mass cycles internally and is conserved,"** *not* "emergent N
   feedback" (cf. Step 2's draw-down-not-oscillation, Steps 3/5's `f_O2` deferral). The
   N-limited regime (`f_N < 1` throttling photosynthesis) is **deferred to Step 7**'s sized
   multi-year run. The decoupling is **verified** — `test_mineralization` recomputes `f_N`
   each step and asserts `== 1.0`, and the bit-identical carbon run is *additionally* pinned
   by the **unchanged** prior sealed tests.

**Why `rationed == 0` (the backstop guard).** Each first-order draw (`n_sen·plant_n·dt`,
`k_min·litter_n·dt`, both `rate·dt ≪ 1`) withdraws against the **start-of-step** donor;
same-step inflows don't count (the arbitration memory) but the draw never exceeds the pool
— the senescence/decomposition self-limiting pattern. `litter_n` starts at 0, so on step 1
mineralization sources 0 (reads start-of-step 0) and the pool accumulates first, then drains
— exactly the `litter_carbon` → decomposition rhythm.

**Why first-order in `plant_n` (the litter C:N is emergent — a deferred refinement).**
N-shedding is a plain first-order relative rate on the whole-plant `plant_n` POOL. The
alternative — shedding N in lockstep with the carbon-senescence flux at a fixed tissue N:C
— would *control* the litter C:N ratio; here the litter C:N (`litter_carbon / litter_n`) is
instead **emergent** from two independent rates. The simpler decoupled pick is the
JIT-minimal Step-6 choice; the N:C-coupled shedding and N resorption before abscission
(this model sheds all standing plant N) are documented refinement seams.

**Test plan (`tests/test_mineralization.py`, 21 tests, all green).** (a) *Rate laws:*
each flux first-order in its donor, zero at zero. (b) *Flow:* `plant_n → litter_n` /
`litter_n → soil_n` transfers of the same amount, single-currency NITROGEN balance (no
CARBON/OXYGEN/WATER residual), dt-linearity, self-limit at zero donor. (c) *Loader:*
committed rates, both rates' negative + bad-unit rejection. (d) *Integration (sealed
season):* `litter_n` accumulate-then-drain (non-vacuous), `plant_n` drained (declines from
start — reversing the Phase-1 monotone growth), **total NITROGEN conserved float-exact**
(`abs_tol 1e-9`, soil_n-dominated — the oxygen-test magnitude, not carbon's `1e-12`),
`rationed == 0`, no extinction, the **`f_N == 1.0` decoupling** (recompute
`nitrogen_stress_factor` per state), and the open field grows no `litter_n`. **Open field
is byte-identical** — the loop is `sealed`-gated/appended; no sealed golden exists (the
sealed run is pinned behaviourally), and the open-field regression golden is untouched.

---

## Step 7 design — sealed-chamber integration + validation (the Phase-2 capstone)

*Closes Phase 2 (designed JIT, the Phase-1 rhythm; advisor-reviewed before build). Steps
1–6 built every piece (the composition core, the finite gas pools, the multi-quantity gas
exchange, the carbon decomposer loop, the nitrogen return loop); Step 7 **assembles a
multi-year sealed run, applies the deferred kinetic self-limiters (`f_O2`, and `f_N`
where it bites), and validates a closed-system phenomenon** against a hex-float golden.
Zero `simcore` changes — `f_O2` is a domain-side pure function, the scenario is
`season.py`, the golden is tests. Phase 2 thus changes **exactly one** core surface
(P2.1), as promised.*

**The probe that reframed the deliverable (the advisor's "look before you design").** The
plan's headline phenomenon was "CO₂ oscillation across years." A cheap 3-year tiled run
at the PP sizing (`bench/probe_multiyear.py`, throwaway) showed it is **not achievable**
in this model: DVS is monotone (it only accumulates, never resets — no vernalization, no
re-sowing) and caps at 2.0, where the partition table sends **`fl = 0` carbon to leaves**
(all to storage/grain). So the plant matures mid-year-1, leaf carbon then decays to ~0
(senescence drains it; nothing refills it), LAI → 0, and **photosynthesis stops after
year 1** — no summer recovery, hence no sustained oscillation. The run instead settles to
a **dead steady state** (organic carbon respired back to CO₂ except a permanent
`storage_c` residue that pays no maintenance and never senesces). `rationed == 0`,
`events == ()` hold across all 915 steps. **Sustained multi-year oscillation is therefore
DEFERRED** (it needs annual phenology reset / re-sowing / vernalization — a Phase-3
scenario seam), exactly as Step 2 shipped *draw-down, not oscillation*. What is real and
already present is the **within-year draw-down → partial recovery** (Step 5) — a single
seasonal cycle, not a sustained one.

**The spine (exit-criteria; must land) vs the gravy (assert-if-it-falls-out, else defer)
— the advisor's scoping.**
- **Spine.** (1) The `f_O2` kinetic factor (a pure addition; applied to both plant
  maintenance shortfall and microbial respiration). (2) **One canonical multi-year sealed
  run** — all four quantities (CARBON/OXYGEN/WATER/NITROGEN) conserved float-exact,
  `rationed == 0`, no extinction, deterministic, **hex-float golden** (the first sealed
  golden). (3) **At least one closed-system phenomenon clearly asserted: O₂ depletion**
  (the Biosphere-2 headline) — with the **O₂↔CO₂ anti-correlation riding along exact**
  (`CO₂_mol + O₂_mol = const`, since only CO₂/O₂ carry OXYGEN and `2·(CO₂+O₂)` is
  conserved). (4) Docs + `MEMORY.md` + the zero-`simcore`-change frozen-surface note.
- **Gravy.** **`f_N < 1` biting** is a *separate sizing axis* (low total N) from the
  O₂/litter axis, and is **not** an exit-criteria phenomenon (those are all gas-side). It
  is asserted **if the canonical depleting run happens to dilute N below critical**;
  otherwise a small targeted test pins it, and the fully N-limited multi-year regime is a
  documented Phase-3 seam. Do not let N-tuning block the gas-phase capstone.

**`f_O2` — the deferred O₂ self-limitation (Steps 3/5 → here).** A Monod factor in the O₂
**mole fraction** (consistent with the `Ci` treatment, chamber-size-aware):
`f_O2 = x_O2 / (K_O2 + x_O2)`, `x_O2 = o2_mol / air_mol`, a new pure
`chamber.oxygen_limitation_factor(o2_mol, *, air_mol, k_o2)` → 0 as O₂ → 0, → 1 as
O₂ ≫ K_O2. It multiplies the **O₂-consuming respiration fluxes only**:
- **Microbial respiration** (`microbial_C + O₂ → CO₂`): `flux *= f_O2(o2_pool)`.
- **Plant maintenance shortfall** (`biomass + O₂ → CO₂`, the only O₂-consuming plant leg
  in the closed chamber — the *covered* maintenance is a CO₂→CO₂ no-op): the organ-burn
  is scaled by `f_O2`. The budget's `MRES`/`available` (driving allocation + growth
  respiration) are **unchanged** — only the realized burn shrinks, so the unmet
  maintenance carbon simply **stays in biomass** (an O₂-limited-respiration model, the
  respiratory mirror of FvCB's Ci-shutoff). **Conservation-safe**: `f_O2` scales the whole
  flow, every leg still balances CARBON and OXYGEN.
- **Open field is byte-identical** (`o2_pool=None` → `f_O2` not applied; the regression
  golden is untouched). **PP-sealed is behaviourally green**: `K_O2` is **low/sharp**
  (aerobic respiration is genuinely O₂-saturated until near-anoxia), so `f_O2 ≈ 1` at the
  ~21% PP fill (the existing sealed behavioural tests still pass — conservation is
  leg-balanced regardless, draws only shrink) and `f_O2` is **load-bearing only as O₂
  approaches its floor** — i.e. in the depleting canonical run. `K_O2` is cited from the
  aerobic-respiration O₂-half-saturation literature (clean-room, `TODO(cite)` provisional).
  Placement: a per-process `o2_half_saturation` in `respiration.yaml` and
  `microbial_respiration.yaml` (each respiration process declares its own kinetics — the
  established per-process param pattern), read into the maintenance context and the
  microbial flow respectively.

**The canonical run — sized to deplete O₂ (the empirical heart, the Step-2 `air_mol`-probe
rhythm). COMMITTED SIZING (probed):** `SEALED_CHAMBER_SCENARIO` = a deliberately **O₂-poor**
chamber **2 mol O₂ in 1000 mol air** (≈ 0.2 % — at the PP ~210 mol fill depletion is
invisible, ~0.026 %; shrinking O₂ is the scale choice that makes the phenomenon
non-vacuous, exactly the Step-2 `air_mol` logic) seeded with **3 mol C of standing litter**,
run **`SEALED_CHAMBER_YEARS = 3`** years. **`K_O2 = 1e-4` mol/mol** (retuned from the 7a
placeholder 0.001): low/sharp enough that `f_O2` starts at **0.95** (a healthy-but-O₂-poor
chamber, *not* ≈ 1 — the fill is deliberately low) and bites hard at the trough
(`f_O2 ≈ 0.04`). The seeded litter's decomposition + microbial respiration draw O₂ down
**~99 %** to an acute trough where `f_O2` self-limits the draw and **`rationed == 0` still
holds** (the positivity-from-kinetics invariant on a genuinely depleting pool — the thing
Steps 3/5 deferred; an un-throttled `K_O2 = 0` control rations **77×**, the load-bearing
proof). Weather is **tiled** (the daily table repeated `n_years×`; `_table` already reads
rows in order, so a repeated list cycles the seasonal forcing — no new schedule code).

**The emergent three-act trajectory (richer than first sketched — no control code).**
(1) *Acute O₂ crash* (~day 120): litter → microbes → CO₂ draws O₂ 2.0 → ~0.004,
`f_O2` → 0.04, CO₂ rises (Ci → ~1650). (2) *Producer-driven recovery* (to ~day 305): a
**coupled producer × `f_O2`** swing — respiration backs off at the anoxic trough, so the
still-living plant (photosynthesising at the high CO₂) **out-paces it and refills O₂** to
~1.6 (the control, with no `f_O2`, just crashes O₂ to zero). (3) *Secular CO₂-rich end
state*: the plant matures and dies (leaf → 0), decomposition wins, the chamber settles
O₂-depleted / CO₂-rich (Ci ≈ 1140). One hardening fix surfaced in the build:
`oxygen_limitation_factor` **clamps a float-dust-negative O₂ to the anoxic floor**
(`f_O2 = 0`) rather than raising, so full depletion self-limits smoothly to 0.

**Failure mode = O₂ depletion, NOT extinction (advisor-affirmed).** Extinction routes a
POPULATION's residual to the **boundary** loss-sink (`integrator.py`, #6) — total CARBON
*including* the boundary stays conserved (the gate passes), but carbon leaving the chamber
to a boundary contradicts the *sealed/closed* narrative and injects a discontinuity. So
the canonical run asserts **`events == ()`** (the self-limiting first-order draws + `f_O2`
keep the POPULATION pools off zero naturally — the assertion is a guard that should hold).
Plant-extinction-as-failure is a documented seam (it belongs with Phase-3 multi-compartment
structure, where a chamber boundary is explicit).

**Test plan (`tests/test_sealed_chamber.py`, the integration capstone).** (a) *`f_O2`
factor* (in `test_chamber.py` or here): Monod hand value, → 1 as O₂ ≫ K_O2, → 0 as
O₂ → 0, degenerate-input guards; `f_O2 ≈ 1` at the PP fill (the deferral-guard, now
*satisfied* not just asserted). (b) *Conservation (the canonical run):* each of CARBON,
OXYGEN, WATER, NITROGEN invariant float-exact every step (the every-step gate end-to-end
through `f_O2`-throttled fluxes). (c) *Stability:* `rationed == 0`, `events == ()` across
the full multi-year horizon (`f_O2` is what makes `rationed == 0` survive the depleting
O₂ pool — verify it is load-bearing by also showing an *un-throttled* control would
ration / over-draw). (d) *The phenomenon:* O₂ depletes a clear fraction (assert `min(O₂) <
α·fill` for a committed α) and the **O₂↔CO₂ anti-correlation is exact** (`CO₂_mol +
O₂_mol == const` to float — pinned tight, not qualitative). (e) *Determinism:* the
canonical run is bit-identical on a re-run (registration-order-independent). (f) *Golden:*
byte-exact hex-float final-state snapshot (`test_regression_sealed_season.py`, the
`_regenerate`/`__main__` discipline of the open-field golden) + round-trip load-back.
(g) *`f_N` (gravy):* assert `f_N < 1` somewhere if the run dilutes N below critical, else
a targeted low-N test pins the bite and the rest is a documented seam. **Open field
remains byte-identical** (the existing season golden is untouched).

---

## Exit criteria (Phase 2 — "closed chamber / producer + decomposer")

- [x] **Element-composition core (P2.1)** landed: stocks carry composition, the gate folds
      it, OXYGEN is genuinely asserted; the Phase-1 1:1 behavior is preserved (goldens
      regenerate only for the serialized field).
- [x] **The genuine multi-quantity stoichiometric flow** (P1's filed deferral) exists
      (Step 3): gas exchange balances CARBON *and* OXYGEN in one flow at PQ=1
      (`CO₂ → biomass + O₂` / `biomass + O₂ → CO₂`); per-flow + every-step OXYGEN balance
      pinned, total OXYGEN float-exact.
- [x] **Emergent feedback (P2.2), no control code:** **landed (Steps 2+3):**
      photosynthesis draws a finite CO₂ pool → `Ci` falls → assimilation falls, and (Step
      3) respiration returns CO₂ → the pool refills, so the CO₂↔O₂ anti-correlation is
      exact (ΔO₂ = −Δ net CO₂, `2·(CO₂+O₂)` conserved) — no special code; `rationed == 0`
      on the finite pools (FvCB Ci-shutoff; O₂ far from its floor), non-vacuous (Ci falls
      ~2.7×). *(Sustained multi-year oscillation awaits the decomposer return of litter
      carbon, Steps 4–7.)*
- [x] **Decomposer loop (P2.3):** litter → microbial biomass → CO₂; microbial respiration
      draws O₂; mineralization returns N to `soil_n`. **Landed (Steps 4–6):** Step 4 the
      `litter_carbon` POOL + first-order `litter → microbial` decay (CARBON-only); **Step 5
      the O₂-drawing microbial respiration `microbial_C + O₂ → CO₂`**, closing the carbon
      loop (litter → microbial → CO₂ → photosynthesis) with CARBON + OXYGEN float-exact;
      **Step 6 the nitrogen return loop** — a `litter_n` POOL with `NitrogenSenescence`
      (`plant_n → litter_n`) + `Mineralization` (`litter_n → soil_n`), closing
      `soil_n → plant_n → litter_n → soil_n` internally (DIRECT net mineralization;
      microbe-mediated N deferred), total NITROGEN float-exact, `rationed == 0`. All
      mechanism-level; `f_N ≡ 1` keeps it carbon-decoupled (the N-limited regime is the
      Step-7 sized run).
- [x] **Sealed multi-year run (Step 7):** the canonical `SEALED_CHAMBER_SCENARIO` (O₂-poor
      chamber + litter seed, tiled 3 years) holds **stable every-step conservation of all
      four quantities** (CARBON/OXYGEN/WATER/NITROGEN, float-exact), stable numerics
      (`rationed == 0`, no extinction), and a clear closed-system phenomenon: the
      **Biosphere-2 O₂-depletion failure mode** (O₂ drawn down ~99 % to an acute trough,
      `f_O2` self-limiting the draw — load-bearing, a control rations 77×) with the exact
      O₂↔CO₂ anti-correlation and an emergent producer × `f_O2` recovery swing.
      (`tests/test_sealed_chamber.py`.) *Sustained oscillation is deferred — the plant dies
      after year 1; it needs annual phenology reset / re-sowing, a Phase-3 seam.*
- [x] **Determinism + golden + engine invariants (Step 7):** the sealed run is bit-identical
      on re-run and **registration-order-independent** (the Registry sorts by id; pinned by
      `test_sealed_is_flow_registration_order_independent` on the sealed path), with the
      first **sealed** hex-float golden (`sealed_chamber_state.json`); purity + Phase-0/0.5
      gates stay green, and **Step 7 added zero `simcore` changes** — Phase 2 changed
      exactly one core surface (P2.1).
