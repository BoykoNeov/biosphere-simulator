# Phase 6 — Station Integration (cross-domain coupling)

**Status: IN PROGRESS — Steps 1–8 (P6.1–P6.8) COMPLETE.** Pre-plan investigation complete and
advisor-reviewed (two blocking checks run before this doc committed to anything; see
"Load-bearing findings" below). Steps 1–3 are designed concretely here; Steps 4–10 get
just-in-time design as Phase 5's siblings did ("each designed just-in-time"), because each
later seam's shape is only fully constrained once the seam before it exists. (Step 4 is
designed in full under "Steps" below; the plan's original one-line framing over-reached on
biosphere-transpiration coupling — corrected there.)

**Step 1 (P6.1) COMPLETE — the `src/station/` assembly layer, proven on Power → Thermal
heat closure.** New `src/station/` package (`scenario.py` / `system.py`, the assembly layer
that imports both siblings and owns the wiring; **no domain imports another**). The seam:
Power's dissipation legs are redirected from `boundary.waste_heat` into `thermal.node` by
passing `thermal.node`'s id where `SolarCharge`/`LoadDraw` took `waste_heat`, and Thermal's
forced `HeatInput` stand-in is dropped (Power's dissipation *is* the input now) — so
`boundary.waste_heat`/`boundary.heat_source` are **absent** from the station state (the
redirection is structural, not a shadow sink). `RadiatorReject` rejects that **real** load
to deep space. `build_station`/`station_resolver`/`run_station` — the harness every later
step reuses; `station_resolver` is exactly `power_resolver` (Thermal contributes no forcing
once `HeatInput` drops — multi-resolver merging deferred to the first step that needs it).
Single-quantity (ENERGY): the combined ledger balances every step over `solar_source +
battery + node + space` (the payload). **The node's initial heat is DERIVED from Power's
actual dissipation** (`equilibrium_node_heat` → `mean_dissipated_power` → the reused Thermal
`equilibrium_temperature`; all supplied solar becomes heat in daily balance ⇒ mean ≈ 316 W ⇒
`T_eq ≈ 160.1 K`), not a hand-set `heat_load`. **The two-start convergence test is the
non-circular core** (advisor): two `node0` values (`0.5·Q_eq`/`1.5·Q_eq`) under identical
Power forcing contract to one band over ~3 τ (τ ≈ 14.6 days) — the radiator alone governs
the difference (the no-radiator contrast keeps it constant), so the equilibrium is set by
dissipation *independent of IC* (starting at `Q_eq` alone would only show stability). The
node band is asserted within ~1 K of the **mean-power** `T_eq` (the true attractor sits
slightly below by the T⁴-convexity offset — honest, not pinned exact). Cheap corroboration:
`battery`+`solar_source` **bit-identical to standalone Power** (coupling is pure sink
re-wiring — the donor is unperturbed, verified empirically step-by-step); per-day `ΔSpace` ≈
Power's per-day heat generation (the radiator carries a *real* load, quantitatively); RK4 ≢
Euler on the node (nonlinear radiator, tolerance agreement) but bit-identical on the forced
battery; determinism; registration-order independence. **16 tests** (14 run + 2 golden);
additive **NON-frozen** golden `station_state.json` (pre-golden gate: `rationed == 0` /
`events == ()` / combined ENERGY closed every step / no shadow sink / node at the
dissipation-set equilibrium — an imbalanced/shadow-sink/runaway run is unpinnable). **Zero
core change** (`git diff src/simcore/` empty), **zero domain change** (`src/domains/`
untouched); full suite incl. `-m slow` + ruff + pyright green (**1221 passed**); **all
fourteen existing goldens byte-identical** (seven frozen + two demo + two Power + one
Thermal + one ECLSS + one Crew; no regen). `src/station` added to the wheel `packages` list.

**Step 2 (P6.2) COMPLETE — the Crew ↔ ECLSS cabin gas loop; OXYGEN closes via composition
CO₂ + a merged stoichiometric respiration flow.** New `src/station/cabin.py` (the second
assembly, mirroring `domains.*.system` one level up) + `src/station/flows.py`
(`CrewRespiration`, the first station-owned flow) + a `CabinScenario` in `station/scenario.py`.
The seam: ECLSS's forced `CrewMetabolism` stand-in and its three `metabolic_*` reservoirs are
dropped, and the standalone crew `o2_store` / `OxygenConsumption` are dropped too; the **real**
crew breathes cabin air via the merged **`CrewRespiration`** (`food_store + cabin_o2 →
cabin_co2 + fecal_waste` — the `biosphere.microbial_respiration` PQ = 1 template one trophic
level up, **forced**, 4-leg; the O₂ leg magnitude is `respired`, since only metabolized carbon
draws O₂, not egested feces) + the crew `WaterBalance` (`water_store → cabin_h2o + urine`);
ECLSS's three control loops (`CO2Scrubber` / `Condenser` / `O2Makeup`) carry over unchanged.
**Every CO₂-bearing stock across the loop is composition `{C:1,O:2}`** (`cabin_co2` *and* the
scrubber sink `co2_removed`) and every O₂ stock is `{O:2}` (`cabin_o2` *and* the makeup source
`o2_supply`) — built **inline in the station** (the `boundary` / `eclss` stock constructors take
no composition arg, and extending them would be a core change), **zero core change**. **The
payload is two non-vacuous gates** — per-quantity ledger balance is *trivial* (every flow
balances internally): (1) the **decoupled** build (pure-carbon `cabin_co2`) raises
`ConservationError` for OXYGEN on the **first step** (balance is evaluation-time — the gate is a
one-step run, not a construction) — composition is load-bearing, the "it bit" gate (the N-limited
`f_N` analogue); (2) **O₂ is genuinely drawn from the cabin** — `cabin_o2` starts at the setpoint
(makeup idle) and is pulled **below** to `o2_eq = setpoint − f_resp·food/k_makeup` (8.3 mol), the
makeup only topping up the deficit. **RQ = 1 is baked in by the PQ = 1 template** (O₂ consumption
= CO₂ production in one flow; a realistic RQ ≈ 0.75 needs metabolic-water / food-composition
machinery — deferred, matching the biosphere's `{CARBON:1}` biomass / PQ = 1 convention).
**Closure is the augmented / atom-conservation sense, NOT a closed O₂/CO₂ cycle** (O₂ still enters
from `o2_supply`, CO₂ still leaves to `co2_removed` — Thermal's permanent `boundary.space`
analogue; the recycled cabin cycle where plants return O₂ is **Step 3**). WATER stays decoupled
(`water_store → cabin_h2o`; metabolic water ignored — the plan's WATER scope boundary; food
carries no WATER composition). The cabin reaches emergent per-species steady states
(`cabin_steady_state` closed form — 8.3 mol O₂ / 3.4 mol CO₂ / 0.04 kg H₂O), while the crew
**stores run down** (forced, open-loop — the argument for Steps 4/6 closure), well-fed
(`rationed == 0`; CO₂/H₂O structural `k·dt<1`, `cabin_o2` + stores well-fed sizing). Forced stores
**RK4 ≡ Euler bit-identical**; state-dependent cabin species **RK4 ≢ Euler** (mid-transient
tolerance agreement — the Step-1 battery/node split, now stores/cabin). **dt = 60 s** (ECLSS's
constraint is binding — crew's dt = 3600 breaks `k_scrub·dt < 1`); reuses `crew.yaml` +
`eclss.yaml` params verbatim. **16 tests** (14 run + 2 golden); additive **NON-frozen** golden
`cabin_gas_state.json` (pre-golden gate: 3-quantity closure / `rationed == 0` / O₂ below setpoint
/ reached steady state — an imbalanced / undrawn / non-converged run is unpinnable). **Zero core
change** (`git diff src/simcore/` empty) + **zero domain change** (`src/domains/` untouched —
`CabinScenario` is additive in `station/scenario.py`); full suite incl. `-m slow` + ruff + pyright
green (**1237 passed**); **all fifteen existing goldens byte-identical** (seven frozen + two demo +
two Power + one Thermal + one ECLSS + one Crew + the Step-1 station; no regen). NEXT: Step 3 (P6.3)
— biosphere ↔ cabin (point the frozen biosphere's `ChamberWiring` at the cabin gas stocks; the
emergent crew↔plant CO₂/O₂ feedback with no control code).

Phases 0–5 are complete and regression-pinned. Phase 4 **froze the biosphere as THE
reference** (`docs/biosphere-reference.md` + manifest). Phase 5 built the four standalone
siblings — **Power / Thermal / ECLSS / Crew** in `src/domains/{power,thermal,eclss,crew}`
— each verified alone (conservation + determinism), each with an additive NON-frozen
golden. Phase 6 **closes the station, not just the biosphere**: the siblings meet the
biosphere, and each other, at shared stocks.

## Relationship to the roadmap (lines 326–348)

**Goal:** *Close the station, not just the biosphere.* Domains couple **only through
shared stocks** — electrical energy, waste heat, O₂ / CO₂, water, biomass / food. Joules
balance; heat-generated is accounted; radiators carry a real load. Cross-domain
perturbations (brownout, radiator failure, atmosphere leak, crew load spike, lighting
failure) produce cascades **with no cascade code**. Integrated consumption/production is
validated against published life-support references (NASA BVAD, BioSim) for at least one
crew configuration.

**Exit criteria (roadmap 347–348):** multi-year *sealed* station runs with **stable
conservation of matter AND energy**, stable numerics, believable cross-domain dynamics,
and **emergent failure cascades**.

## Load-bearing findings (pre-plan investigation, advisor-reviewed)

Three findings from reading the frozen biosphere and the four siblings *before* planning.
They determine the phase's whole shape, so they are settled here, not discovered mid-step.

1. **Coupling is stock-id re-wiring at a new assembly layer — target zero core change.**
   The core already carries the "Domain" primitive as `Registry.domain_index`
   (`DomainId → frozenset[StockId]`, derived from `Stock.domain`). A cross-domain
   interaction is just *a flow whose source stock belongs to one domain and whose sink
   belongs to another* — i.e. two domains' flows referencing one shared `StockId` in one
   combined `State`/`Registry`. Every sibling flow already takes its sink/source ids as
   **constructor args** (Power's `waste_heat: StockId`; the biosphere's `ChamberWiring`
   struct of ids), so coupling is purely an assembly-layer choice of *which id to pass*.
   **No domain imports another** — the new `src/station/` layer imports all of them and
   owns the wiring. (Roadmap: "The shared stock is the entire interface. If two domains
   need to talk, they share a stock. Nothing else.")

2. **The gas representation needs NO new core primitive — but the gas seam is a flow
   re-expression, not an annotation.** `composition` (`Stock.composition:
   Mapping[Quantity, float]`) is a **Phase-2 primitive (P2.1)**, folded into *both*
   `flow.py`'s internal-balance check and the `conservation.py` ledger. The frozen
   biosphere sealed chamber **already closes CARBON *and* OXYGEN** through it: `carbon_pool`
   is `{CARBON:1, OXYGEN:2}` (it *is* CO₂), `o2_pool` is `{OXYGEN:2}`, and
   `MicrobialRespiration` / plant maintenance close OXYGEN in **one** flow with biomass as
   pure CARBON (legs `(biomass −b, co2_pool +b, o2_pool −b)`: CARBON `−b+b=0`, OXYGEN
   `+2b−2b=0`, PQ=1). So "gas representation" is **not** Phase 6's one sanctioned core
   change (an earlier hypothesis, retracted).

   The catch the advisor sharpened: **annotating `cabin_co2` as `{C:1,O:2}` is necessary
   but not sufficient, and not benign.** The moment a crew flow *produces* a composition
   CO₂ stock, `flow.py`'s balance assert demands 2 mol O per mol CO₂ on that flow's input
   legs — and the current **decoupled** crew model (`FoodMetabolism` making pure-CARBON
   CO₂ + a separate `OxygenConsumption` draining O₂ to its own sink) supplies none, so the
   flow **fails to construct**. That refusal *is* the OXYGEN closure becoming real: the
   ledger will no longer accept the decoupled version. The resolution is a **single
   stoichiometric respiration flow** `food_C + O₂ → CO₂` (O₂ coupled to carbon metabolism
   via the respiratory quotient), copying `biosphere/microbial_respiration.py` /
   `respiration.py` verbatim in shape. And composition is a property of the **whole loop**:
   every stock CO₂ passes through (`cabin_co2`, the scrubber sink `co2_removed`, the
   biosphere pools) must be `{C:1,O:2}` or the scrubber unbalances OXYGEN. So the gas step
   is *"make every stock in the CO₂ loop composition-consistent + merge crew respiration
   into one flow,"* not *"annotate one stock."*

   **Scope boundary (settled):** crew **humidity** stays on the *separate* `water_store →
   cabin_h2o` path — metabolic water is ignored, exactly as the biosphere ignores biomass
   O/H — so WATER closes **independently** and food carries no WATER composition. (If
   humidity were meant to be metabolic, WATER could not be expressed without more currency;
   that is explicitly out of scope, not papered over.)

3. **The frozen biosphere is re-wirable beside its freeze — Steps 3/5 do not unfreeze it.**
   CO₂ reaches FvCB as a **live shared stock** (`co2_pool` forcing-var → `CARBON_POOL`,
   resolved through `env.get` — the P2.2 draw-down feedback), *not* an un-swappable
   constant forcing; and the gas-exchange flows are wired by explicit ids via
   `ChamberWiring` (`carbon_source` / `resp_sink` / `o2_pool` / …). A station assembly can
   therefore construct the **frozen** biosphere flow classes with a station `ChamberWiring`
   pointing at the shared cabin-gas stocks, and map `co2_pool` → the cabin CO₂ id in the
   station resolver — with **zero change to frozen flow classes, params, or the standalone
   biosphere goldens** (the biosphere's own sealed-chamber golden is a *separate* assembly
   that never runs coupled). Same pattern as Power's `waste_heat`, one level richer.
   Lighting (Step 5) is even cleaner: PAR stays a **forcing** var — the station computes it
   from lamp electrical draw instead of a weather table; a flow cannot tell forcing from a
   shared stock (#16), so the biosphere is untouched.

## Design invariants for the phase (carried from Phases 3–5)

- **New `src/station/` layer, outside `domains/`.** It imports every domain and owns all
  cross-domain wiring; **no domain imports another** (the Phase-3 "coupling machinery lives
  outside the coupled units" discipline, now cross-domain). `git diff src/simcore/` stays
  **empty** — zero core change is the target (the ENERGY-assert flip already happened in
  Phase 5 Step 1).
- **Each standalone domain golden stays byte-identical.** The station is a *separate*
  assembly; the seven frozen biosphere + two demo + four sibling goldens do not move.
  Station scenarios get their own **additive, NON-frozen** goldens (the Power-domain
  golden discipline), not entries in the biosphere freeze manifest.
- **Cascades emerge with no cascade code.** Cross-domain perturbations reuse the Phase-3
  `perturbations.py` discipline (compose the disturbance onto the assembled station inputs
  *outside* the builder); a brownout/leak/failure propagates through shared stocks alone.
- **The payload is physical closure, not "the ledger balances."** Per-quantity the combined
  ledger balances *trivially* (every flow is internally balanced). The non-vacuous gates
  are: OXYGEN closes **only** because CO₂ is a composition stock and respiration is one
  stoichiometric flow (Step 2/3); ENERGY closes because every dissipation leg names a real
  heat receiver (Step 1); and the **sealed multi-year run** neither drifts nor collapses
  (Step 7, the Phase-4 drift instrument reused at station scale).

## Steps (tightest-constraint-first: energy → gas → water → the rest)

**Step 1 (P6.1) — the station assembly layer, first proven on Power → Thermal heat
closure.** New `src/station/` (`system.py`: `build_station` = concatenate chosen domains'
stocks+flows into one `Registry`+`State`; a merged `SourceResolver`; `run_station`, the
`run_*` analogue; a station scenario). First seam: **redirect Power's dissipation legs from
`boundary.waste_heat` into `thermal.node`** (the receiver Thermal built standalone). Purely
the assembly passing `thermal.node`'s id where `SolarCharge`/`LoadDraw`/`SelfDischarge`
took `waste_heat` — **zero domain change, zero core change**. Single-quantity (ENERGY): the
cleanest possible first integration. The radiator now carries a **real** load (Power's heat,
not a constructed `heat_load`). Payload: ENERGY closes every step over the *combined*
ledger; `rationed==0`; `events==()`; the thermal node reaches an equilibrium set by the
*actual* power dissipation; determinism + registration-order independence. Additive
NON-frozen station golden. This step also establishes the station harness every later step
reuses.

**Step 2 (P6.2) — Crew ↔ ECLSS cabin: the gas loop closes for OXYGEN (composition +
merged respiration). COMPLETE (see the Step-2 block above).** Delete ECLSS's forced `CrewMetabolism` stand-in; wire the **real**
Crew flows to the cabin. The gas step per finding #2: `cabin_co2` and the scrubber sink
`co2_removed` become composition `{C:1,O:2}`; Crew's separate `OxygenConsumption` +
`FoodMetabolism`-CO₂ leg **merge** into one stoichiometric `food_store + cabin_o2 →
cabin_co2` respiration flow (the `MicrobialRespiration` template, PQ=1). WATER stays
decoupled: `water_store → cabin_h2o` (humidity) feeds the condenser; food carries no WATER
composition. Now **OXYGEN closes** across the crew↔cabin augmented loop (scrubber removes
CO₂; O₂ makeup tops up cabin_o2), and the ledger *refuses* the decoupled version — the
non-vacuous gate. The feces / urine splits keep their separate Phase-6 destinations.

**Step 3 (P6.3) — Biosphere ↔ cabin: crew and plants share the air (the emergent
feedback).** Couple the frozen biosphere's gas exchange to the Step-2 cabin's composition
CO₂/O₂ stocks. Plants draw CO₂ from and return O₂ to the cabin the crew breathes; the
CO₂/O₂ cross-domain feedback (crew exhales → cabin CO₂ rises → plants assimilate → cabin O₂
rises → crew breathes) emerges with **no control code**. Zero frozen-code / zero domain /
zero core change; the standalone biosphere golden is untouched.

*Design decisions (advisor-reviewed 2026-07-01; the plan text above under-specified two
load-bearing points — corrected here):*

- **SEAM DIRECTION REVERSED.** The naive seam ("point the biosphere's `ChamberWiring` at
  the cabin's CO₂/O₂ ids") is **blocked**: only `plants` consumes the wiring;
  `soil.MicrobialRespiration` (built for EVERY sealed chamber) and
  `consumers.ConsumerRespiration` read `CARBON_POOL`/`O2_POOL` from the **catalog,
  hardcoded** — so re-pointing the wiring redirects only plant gas (microbial gas would
  `KeyError` or leak one-way). Instead **keep the biosphere's `CARBON_POOL`/`O2_POOL`
  (already `{C:1,O:2}`/`{O:2}`) as the shared cabin air, and re-point the CABIN's five
  all-parameterized flows** (`CrewRespiration`/`CO2Scrubber`/`O2Makeup`/`Condenser`/
  `WaterBalance`) at those ids. This reuses `build_season(sealed)` wholesale (build_atmosphere
  included; the CARBON loss-sink + default `sealed` wiring + default
  `{CO2_POOL_VAR: CARBON_POOL}` map all just work), and is the correct closed-station
  physics (plants + microbes + crew share one cabin-air stock). `Ci`'s `air_mol` becomes the
  biosphere scenario's `chamber_air_mol`, set to the cabin's air moles (a NON-frozen
  greenhouse scenario).

- **A master-step driver, not `simcore.multirate`.** The biosphere is structurally `dt=1`
  **per-day** (weather indexed by `n`); the cabin is `dt=60 s` **per-second**
  (`k_scrub·dt<1`). `multirate_step` splits ONE shared master `dt` (`dt/n_sub`) — no single
  master dt serves both **units**, and it composes `substep` only, which by design freezes
  the biosphere's `thermal_time` aux (phenology). So the station owns a bespoke master-step
  driver: per day, cabin `substep(dt=60)`×1440 (keeps `n`; asserts conservation after each
  substep to preserve the every-step teeth) then biosphere `step_report(dt=1)`×1 (advances
  aux AND bumps `n`, so `n` stays the day count and the frozen `weather_resolver` works
  unchanged). All public methods → zero core change. Two disjoint registries over one shared
  stock dict + two integrators — multirate's model, orchestrated by hand for per-domain dt.

- **The non-vacuous gate is a net-fixation CONSERVATION IDENTITY, not a cabin-pool shift.**
  The fast ECLSS scrubber (τ≈1000 s) fully relaxes `CARBON_POOL` back to `P/k_scrub` between
  the once-daily biosphere lumps (86 τ per day), so the regulated pools are identical (to fp)
  at every day boundary — the plant's effect is *erased from the pool* and *conserved into*
  (a) biosphere biomass and (b) reduced ECLSS work. So the gate is: with-plants the plant
  fixes net carbon (`bio_organic_C` grows), the scrubber removes LESS CO₂
  (`co2_removed_with < co2_removed_no`) and the makeup supplies LESS O₂
  (`o2_supply_with > o2_supply_no`) — **the plant offloads life support** — and the three
  agree to tolerance (`Δco2_removed ≈ bio_gain ≈ Δo2_supply`, RQ=1; catastrophic-cancellation
  floor ~1e-10, so `atol≈1e-8`, not bit-exact). Booleans carry the sign robustly; an
  un-biting (net-source) run flips them → correctly unpinnable. The seedling starts DVS 0 in
  the growth phase so it is net-assimilating. Step 2's composition-failure gate does NOT
  re-run (`{C:1,O:2}` is frozen inside `build_atmosphere`; Step 3 adds no new composition
  requirement). Illustrative scale (crew dominates ~3400×); calibration deferred to Step 9.

**Step 4 (P6.4) — the water loop. COMPLETE.** Built on the **cabin** (not the greenhouse):
the crew's two WATER disposal sinks (`humidity_condensate` / `urine`) are re-pointed into a
new `recovered_water` buffer POOL (the crew analogue of the biosphere's `condensate`), and a
station-owned `WaterRecovery` flow (`recovered_water → water_store (+η_w) + brine (+(1−η_w))`,
donor-controlled `k_rec`, the `SolarCharge`/`carbon_split` η-split on WATER) returns the
recovered fraction to `crew.water_store`, venting only the unrecoverable remainder to a
`brine` sink. The crew's finite `water_store` — open-loop and monotonically depleting
standalone/cabin — becomes **regenerative up to the recovery efficiency** (net drain drops
from the full intake to `(1−η_w)·intake`, fully closed only at η_w = 1; `brine` is the honest
remaining WATER boundary, the Thermal `boundary.space` analogue). New `src/station/water.py`
+ the **first station-owned params** (`station/params/water_recovery.yaml` + `station/loader.py`;
`k_rec` 1/s, `η_w` dimensionless, illustrative `TODO(cite)` — NOT NASA/ISS numbers). **Zero
domain / zero core change** (assembly-level id re-pointing; the `Condenser`/`WaterBalance`
flow classes are untouched — a buffer pool + a new flow, not a split at the condenser).

*Scope decision (advisor-reviewed): closure ≠ humidity unification.* The plan's first framing
("Crew humidity + **biosphere transpiration** → cabin_h2o → …") over-reached — coupling the
biosphere's transpiration into the cabin humidity is **not** a closure requirement and is
**deferred** (to Step 7 / out of scope). The biosphere's internal water ring
(`soil_water → water_vapor → condensate → soil_water`) is **already closed and sealed
independently** (`test_biosphere_internal_water_loop_closed`); the crew loop closes
independently the moment recovery is added. So station WATER conserves as (closed biosphere
ring) + (crew loop closed up to brine); unifying the two humid-air stocks is a *fidelity
refinement*, not needed to close WATER. Building on the cabin (not the greenhouse) also keeps
the biosphere — Euler-locked by its freeze — out of the assembly, so the **RK4 ≢ Euler
cross-check runs**: recovery makes `water_store` state-dependent (its inflow ∝ the buffer
level), **breaking** the forced RK4 ≡ Euler bit-identity the cabin stores had (the "it earned
its keep" signal, the `SelfDischarge` analogue), while the forced `food_store` stays
bit-identical.

*The payload — a conservation identity (the Step-3 offload analogue).* The `recovered_water`
dynamics and the forced intake are **both independent of η_w** (η_w only splits the processor's
output), so the water returned to the store equals **exactly** η_w times the water the open-loop
(η_w = 0) baseline sends to `brine`: `water_store_with − water_store_without ≈ η_w · brine_without`
(to ~1e-13). The "it bit" gate is with-vs-without recovery (η_w = 0 reproduces the open-loop
drain from the *same* topology) + this identity; the two WATER pools reach emergent steady states
(`cabin_h2o → f_ins·intake/k_cond`, `recovered_water → intake/k_rec`); WATER's total is invariant
(`brine` the only terminal WATER sink); `rationed == 0` (structural `k_rec·dt < 1` + well-fed);
`events == ()`. **17 tests** (15 run — incl. the pre-golden gate — + 2 golden) + an additive **NON-frozen**
golden `water_recovery_state.json`. Full suite incl. `-m slow` + ruff + pyright green (**1265
passed**); all seventeen existing goldens byte-identical (no regen).

**Step 5 (P6.5) — Power → biosphere lighting (energy enters biology). COMPLETE.** A
station-owned **`Lamp`** flow (`station.flows`): `power.battery → light_used + waste_heat`,
3-leg SolarCharge η-split, forced. The station resolver computes the biosphere's `par`
**forcing** from the same lamp-draw schedule (`PAR = photon_efficacy·lamp_power_w/
ground_area`), replacing the weather-table PAR — the phase's **one non-shared-stock
coupling** (finding #3 / #16: Power and the biosphere share *no* stock; the schedule feeds
both the ENERGY ledger and the PAR forcing). Zero frozen / zero domain / zero core change
(PAR stays a forcing).

*Design decisions (advisor-reviewed; the plan's one-line framing under-specified three
load-bearing points — corrected here):*

- **The daylength coupling is the correctness crux.** `incident_par` returns a
  *daytime-mean* flux and FvCB re-multiplies by `daylength_s` (daily dose = PAR ×
  daylength), so overriding PAR alone silently corrupts the dose. **Both** `PAR_VAR` and
  `DAYLENGTH_VAR` come from the lamp (`daylength_s = photoperiod_hours·3600`). Verified the
  *only* runtime `DAYLENGTH_VAR` consumer is photosynthesis (phenology / transpiration /
  net-radiation don't read it), so "day = lamp photoperiod" is consistent everywhere.

- **Scope: Power + biosphere only; the `waste_heat` leg lands in `boundary.waste_heat`, NOT
  `thermal.node`.** The plan's parenthetical `(→ thermal.node)` would only re-test Step-1's
  node seam for no new thesis; the inward move is deferred to the sealed-station step (the
  "boundary now, inward later" rhythm Power's own dissipation followed — the Steps-3/4
  precedent for correcting the plan's first framing).

- **One lamp param, the ENERGY split derived (not two accountings).** `lamp.yaml` carries
  only `photon_efficacy` (µmol/J); the radiant fraction `η_lamp = photon_efficacy·
  PAR_PHOTON_ENERGY_J_PER_UMOL = photon_efficacy / 4.57` is *derived* via the inverse of
  the biosphere's own McCree PAR constant (a σ/CODATA-style module constant, not a param).
  So efficacy and the radiant fraction are two accountings of one device, consistent by
  derivation; the loader guards `photon_efficacy ∈ (0, PAR_UMOL_PER_J]` (the physical
  ceiling η_lamp = 1). Illustrative LED value 2.5 µmol/J (η_lamp ≈ 0.55; Kusuma/Bugbee
  2020), `TODO(cite)`.

- **The frozen-`n` fast domain forces a daily-average lamp draw.** `substep` **keeps** `n`,
  so a within-day top-hat is not an `n`-schedule; the biosphere carries the photoperiod
  *internally* via `daylength_s`, so Power draws the constant daily-average
  `lamp_power_w·photoperiod/24` — its daily **energy** (and the `light_used`/`waste_heat`
  legs) is exact, only the unobserved intra-day instantaneous power is smeared. PAR uses
  the on-window intensity.

- **The two-rate driver is EXTRACTED (second instance).** `station/driver.py`
  `run_master_day` generalizes `run_greenhouse`'s body (slow domain once/day via
  `step_report`, fast domain ×`steps_per_day` via `substep` + per-substep conservation
  assert); `run_greenhouse` refactored to a thin wrapper (greenhouse golden byte-identical),
  `run_lighting` another. **Minimal Power** (battery POOL + Lamp only — no SolarCharge/
  LoadDraw; the battery is a finite energy store draining, the Crew-store pattern).

- **The payload — the signed "it bit" gate (Euler-only, biosphere frozen).** Lamp-on ⇒
  `bio_organic_C` grows (+0.11 mol over 7 days); lamp-off (PAR = 0) ⇒ it declines
  (respiration only) — the lamp genuinely carries the energy driving fixation. Plus: the PAR
  factor reconstructed (`PAR = photon_efficacy·lamp_power_w/ground_area`), ENERGY closed
  every step (battery drains by exactly `lamp_power_w·photoperiod·3600·days`; `light_used`/
  `waste_heat` name the η-split), the biosphere internal water/N loops still close,
  `rationed == 0`, `events == ()`, `battery > 0` (well-fed), Power⊥biosphere stock sets
  disjoint. For Step 8: a schedule-derived PAR won't see brownout rationing automatically —
  flagged, not solved here. **26 tests** (unit: `lamp_energy_split` / `Lamp` legs+balance /
  loader bounds; run: the gate above + determinism); additive **NON-frozen** golden
  `lighting_state.json` (pre-golden gate: every quantity closed / battery drained by the
  lamp energy / lit grew while dark declined). **Zero core change** (`git diff src/simcore/`
  empty) + **zero domain change** (`src/domains/` untouched); full suite incl. `-m slow` +
  ruff + pyright green (**1291 passed**); **all eighteen existing goldens byte-identical**
  (seven frozen + two demo + two Power + one Thermal + one ECLSS + one Crew + Step-1 station
  + Step-2 cabin-gas + Step-3 greenhouse + Step-4 water-recovery; no regen —
  `lighting_state.json` is the nineteenth). NEXT: Step 6 (P6.6) — the biomass / food loop.

**Step 6 (P6.6) — the biomass / food loop — COMPLETE (both seams; the trophic CARBON ring
is closed).**

*Seam 2 execution log (feces → litter):* the design's "crew-scale feces DOMINATES the
litter dynamics (~3400×)" premise was **confirmed** (feces ≈ 363 mol C / 7 days vs seedling
litter 0.0135), but a **regime spike (advisor-flagged) inverted its "Δlitter ≈ feces"
identity**: at ``x_O2 ≈ 10/9500`` the microbes are **active, not throttled** — litter → ~342
mol, microbial biomass → ~20 mol, so microbes consume ~21 mol of the routed carbon and
``Δlitter ≈ feces`` does **not** hold. So the seam-2 gate is **per-quantity closure +
``FECAL_WASTE`` sink absent (no shadow sink) + litter grows materially (with vs without) +
``rationed == 0`` + ``events == ()``**, NOT a three-way identity. Wired via a
default-preserving ``fecal_waste_target: StockId = FECAL_WASTE`` param on
``build_greenhouse``/``_cabin_flows`` (drops the ``FECAL_WASTE`` sink when re-pointed;
station-layer change, zero domain/core) + a ``close_feces: bool`` knob on ``build_harvest``
(default ``True`` = closed ring). Both ``fecal_waste`` and ``litter_carbon`` verified pure
``{CARBON:1}`` (clean swap). **Finding:** closing feces perturbs the plant's grain/food only
at the **fp round-off level** (rel ~1e-15) — microbial CO₂ enters the shared ``CARBON_POOL``
the plant reads for Ci, but the ECLSS scrubber holds the pool at setpoint (the Step-3
regulator-erasure physics), so the two seams are near-orthogonal but **not** bit-identical.
The ``harvest_state.json`` golden was regenerated to the closed ring (Step 6's own additive
golden; the other nineteen stay byte-identical). +3 seam-2 tests
(litter-grows/no-shadow-sink/near-orthogonal), full suite **1305 passed**, ruff + pyright +
``-m slow`` green, ``git diff src/{simcore,domains}/`` empty.

*Seam 1 execution log:* the go/no-go grain-fills spike + the coupled
``k_harvest`` probe both **passed on the recommended path** — ``thermal_time0 = 1300`` (DVS
1.27, past anthesis), ``harvest_rate = 1e-5`` /s (``k·dt = 6e-4``). Seam 1 landed
``station/{flows.py:Harvest, loader.py:load_harvest_params, scenario.py:HarvestScenario,
harvest.py, params/harvest.yaml}`` + ``tests/{test_harvest_run.py (9), test_regression_harvest.py
(2) + golden/harvest_state.json}``. Grain settles to a **positive quasi-steady** (day-boundary
min ~7e-4–1.4e-3 mol; captures ~89 % of the ~1.3e-2 mol/7-day fill), the **two-way identity**
``Δfood_store = Δstorage_c = cumulative harvest`` holds to ~1.8e-9 (the ~1580-mol food-store
cancellation floor; the ~1.2e-2 signal sits 7 orders above), ``rationed == 0``, ``events == ()``,
every mass quantity closes every master day. **Zero core / zero domain change**
(``git diff src/{simcore,domains}/`` empty), ruff + pyright + full suite incl. ``-m slow`` green
(**1302 passed**), all nineteen existing goldens byte-identical (``harvest_state.json`` is the
twentieth). The **exact-identity properties were verified before building** (advisor-flagged):
(1) only ``annual_reset`` reads ``storage_c`` and it doesn't fire ≤7 days, and ``Allocation``'s
``FO·DMI`` fill leg is independent of ``storage_c``'s level → harvest doesn't perturb fill;
(2) ``CrewRespiration`` is forced (independent of ``food_store``) → the regenerated store doesn't
perturb the cabin gas; (3) ``Harvest`` touches neither ``CARBON_POOL`` nor a photosynthesis
input. The driver is **slow-first** (biosphere refills grain, then 1440 cabin sub-steps drain it),
so the day-boundary snapshot is the intra-day *minimum* ``storage_c``. **Seam 2 (the
``fecal_waste → litter_carbon`` re-pointing) is the next increment** — the design below is its
spec.

**Step 6 (P6.6) — the biomass / food loop — DESIGN (just-in-time).**
Biosphere harvest → crew `food_store` (regenerative food); crew feces → soil/waste. Close
CARBON through the trophic seam (the crew's finite `food_store`, open-loop standalone,
becomes regenerative). Built on the **greenhouse** (Step 3): it is the only assembly where a
live plant already shares the cabin air, so it is where biomass can flow into food. The
CARBON analogue of Step 4's `WaterRecovery`, one trophic level, run under the existing
two-rate `run_master_day` driver.

- **The two seams (both id re-pointings + one new flow at the station layer; zero core, zero
  domain change).**
  1. **Harvest — the new station-owned flow.** `Harvest(storage_c → food_store)`,
     donor-controlled `k_harvest · storage_c · dt` — structurally the biosphere's `Grazing`
     (`herbivory.py`, `leaf_c → consumer_carbon`) one seam over, and functionally the CARBON
     twin of `WaterRecovery`. Both pools are `{CARBON:1}` ⇒ **single-currency transfer, no
     composition fold, no core change**. `rationed == 0` is **structural** (`k·dt < 1`,
     donor-controlled, self-limiting to 0 as grain empties — the `SelfDischarge`/`Grazing`
     positivity, *not* the forced "well-fed sizing" `CrewRespiration` leans on). Lives in the
     **cabin/fast registry** (60 s, alongside `CrewRespiration`); it reads a biosphere stock
     (`storage_c`) but writes a crew stock (`food_store`) — a cross-domain flow, so it belongs
     in `station/flows.py`, never in `domains.*`. The frozen biosphere registry
     (`build_season` verbatim) is **untouched**.
  2. **Feces return — an id re-pointing, no new flow.** Crew `fecal_waste` (today a terminal
     boundary CARBON sink produced by `CrewRespiration`) is re-pointed into the biosphere's
     `litter_carbon` pool — the soil pool `soil.MicrobialRespiration` already consumes back to
     CO₂. Both must be `{CARBON:1}` — **verify `fecal_waste`'s composition before re-pointing**
     (asserted here, not yet confirmed by exploration); if it carries anything else the
     re-pointing is a composition mismatch, not a clean id swap. The redirection is structural
     (the orphaned `fecal_waste` boundary sink is **absent** from the station state, the Step-1
     "no shadow sink" property). This is what makes the loop *closed*: `food_store → respired
     CO₂ (→ cabin air = biosphere `carbon_pool`) + feces (→ litter → microbes → CO₂)`, and the
     plant fixes that cabin CO₂ back into biomass → grain → harvest → `food_store`. Full
     trophic CARBON ring.

- **EXECUTION SEQUENCING — land the two seams as SEPARATE increments (advisor-flagged).** They
  are independent, so bundling them means a problem in one masks the other. **First** land the
  Harvest flow (seam 1) and pass its food-regeneration "it bit" gate; **then**, as a separate
  increment, add the `fecal_waste → litter_carbon` re-pointing (seam 2). Isolation matters
  because crew-scale feces flux dumped into a seedling-scale `litter_carbon` will **dominate**
  the litter→microbial→CO₂ dynamics (the same ~3400× mismatch) — fine for conservation, but it
  must be understood on its own, not conflated with the harvest signal.

- **The load-bearing crux — the harvest source must be non-empty in the run window (the "it
  bit" precondition).** `biosphere.storage_c` (grain / storage organ) is the right source —
  `allocation.py` isolates it as *"harvested, not shed"* (excluded from maintenance
  respiration, senescence, and the `f_N` biomass sum; a pure `Allocation` sink), so it is the
  one plant pool that accumulates without being clawed back, and its only existing consumer is
  `annual_reset` (a year-boundary transform that does **not** fire inside a ≤7-day run). **But
  `storage_c` only fills after anthesis (`FO > 0` requires `DVS > 1`), and a fresh seedling
  sits at `DVS < 1` with `storage_c0 = 0`** — so the default greenhouse plant would give a
  zero harvest source and the loop would not bite. Resolution: a `HARVEST_BIO_SCENARIO`
  (**scenario data only** — the additive, non-frozen `N_LIMITED` / `WATER_BITING` precedent;
  no new flow / aux / param, frozen goldens stay byte-identical) that puts the plant in the
  reproductive phase so grain is actively *filling* while harvest drains it — a genuinely
  regenerative source, not a static reservoir being emptied. **Recommended:** initialise the
  biosphere `thermal_time` aux past the anthesis threshold (`DVS > 1` from day 0 ⇒ `FO > 0` ⇒
  grain fills under lighting). Fallback if that proves awkward: seed `storage_c0 > 0` (a
  standing grain stock the harvest draws down — demonstrates the transfer + CARBON closure but
  the "regenerative" story is weaker, since photosynthesis isn't replenishing the source
  during the run).

- **EXECUTION STEP 0 — the grain-fills spike (go/no-go, BEFORE any test/golden scaffolding;
  advisor-flagged as load-bearing).** Whether a given `thermal_time0` actually yields `FO > 0`
  and *meaningfully filling* `storage_c` in the ≤7-day window is an empirical fact about the
  **frozen** partition table + DVS calc — it **cannot be assumed**, it must be measured first.
  It is a **joint** search, not one knob: `thermal_time0` + horizon must *simultaneously*
  satisfy (a) grain fills at a rate that clears the ledger round-off floor (so the signed "it
  bit" gate measures signal, not noise), (b) the plant **persists** (a post-anthesis plant may
  reach maturity/senescence), (c) `events == ()` (senescence/extinction must not fire), and
  (d) `rationed == 0`. If grain-fill is impossibly slow at 1 m² over the window, **pivot to the
  `storage_c0 > 0` static-reservoir fallback here** — before building the five artifacts on
  sand. This spike decides recommended-vs-fallback; everything downstream assumes it passed.

- **The payload — a with-vs-without-harvest CARBON conservation identity** (the Step-3 /
  Step-4 "it bit" discipline). The baseline arm (`k_harvest = 0`, or `with_harvest=False`)
  reproduces today's open-loop greenhouse: `food_store` depletes at the full crew rate.
  The coupled arm: `food_store` depletes **slower** (regenerated by grain), grain is drawn
  down vs the un-harvested baseline, and litter grows by the routed feces — the three agree to
  tolerance (`Δfood_store ≈ grain_removed`, `Δlitter ≈ feces_routed`; a signed gate — an
  un-biting run flips the sign). Every conserved quantity closes over the augmented ledger
  **every master day**; `rationed == 0`; `events == ()`.

- **Honest scope / deferrals.** *Magnitude:* the 1 m² seedling's grain-fill rate is ~1e-4×
  the ~345 mol C/day crew food draw (the Step-3 ~3400× mismatch), so `food_store` still
  net-depletes — this is a **direction + conservation demo, not a self-sufficient loop**;
  crew-vs-plant magnitude calibration is **deferred to Step 9** (the Step-3 precedent). *RQ:*
  PQ = 1 / pure-CARBON biomass keeps RQ = 1 (metabolic-water / food-composition machinery
  stays deferred, matching the biosphere and Step 2). *`annual_reset`:* its seed-bank guard
  (`grain ≥ seedling_total`) is not exercised in a ≤7-day run, but a harvest that drains grain
  must not starve the re-sow — **flagged for the multi-year sealed run (Step 7)**, not solved
  here. *Numerics:* the greenhouse biosphere is **Euler-locked by its freeze**, so this is an
  Euler-only run (no RK4 cross-check) — the `WaterRecovery` "state-dependent breaks RK4≡Euler"
  signal does not apply on this two-rate, biosphere-coupled build.

- **The five artifacts to create** (the established per-step skeleton — `WaterRecovery` /
  Step 4 is the closest template; `greenhouse.py` / Step 3 supplies the two-rate assembly).
  1. `src/station/flows.py`: a `Harvest` flow class (`@dataclass(frozen=True)`, fields =
     stock ids + `priority` + a `HarvestParams`) + a `HarvestParams` frozen dataclass. No
     η-split (a single-fate transfer, unlike `WaterRecovery` / `Lamp`).
  2. `src/station/params/harvest.yaml` (one param `harvest_rate` `k_harvest`, unit `1/s`,
     `k ≥ 0`, illustrative `TODO(cite)` — **NOT** NASA/BVAD) + `load_harvest_params` in
     `loader.py` (the `{value, unit, source}` + exact-string unit-guard + bound-check
     discipline; reuse the generic `_ValueUnitSource` schema).
  3. `src/station/scenario.py`: a `HarvestScenario` (extending / referencing
     `GreenhouseScenario`) with the reproductive `HARVEST_BIO_SCENARIO` bio field + a
     `HARVEST_DAYS` horizon.
  4. `src/station/harvest.py`: `build_harvest` (re-uses `build_season` for the biosphere,
     builds the cabin/fast registry with the new `Harvest` flow and the `fecal_waste →
     litter_carbon` re-pointing, asserts the two flow-registries' id sets disjoint over the
     shared stock dict) / `harvest_resolver` (delegates to `greenhouse` resolvers) / a thin
     `run_harvest` wrapper over `run_master_day` / an optional `harvest_steady_state` /
     module-level `HARVEST = FlowId("station.harvest")` id constants. Baseline arm via a
     `with_harvest: bool`.
  5. `tests/test_harvest_run.py` (validation: every-master-day CARBON/OXYGEN/WATER closure;
     the signed with-vs-without "it bit" gate; the conservation identity; `rationed == 0`;
     `events == ()`; the orphaned `fecal_waste` sink absent; determinism +
     registration-order independence — no RK4 arm, Euler-locked) and
     `tests/test_regression_harvest.py` + `tests/regression/golden/harvest_state.json` (the
     two-test golden with a pre-golden gate that bakes in `rationed == 0` / every-day closure /
     harvest actually moved carbon / `food_store` above the un-harvested baseline — a
     degenerate/un-biting run is unpinnable; additive **NON-frozen**, not in the freeze
     manifest, `__main__` regen).

- **Exit criteria (same as every Phase-6 step):** `git diff src/simcore/` empty (zero core
  change), `src/domains/` untouched (zero domain change), full suite incl. `-m slow` + ruff +
  pyright green, and **all nineteen existing goldens byte-identical** (no regen — the new
  `harvest_state.json` is the twentieth). Then update the plan doc, `CLAUDE.md`, and memory;
  commit + push to `main`.

**Step 7 (P6.7) — the sealed station: multi-year matter+energy stability — COMPLETE
(executed; all spikes passed, all tiers delivered).** New `src/station/sealed.py`
(`build_sealed_station` composing every Phase-6 seam over one shared stock dict + two
registries — biosphere-slow + everything-fast ~11 flows; `sealed_bio_resolver` /
`sealed_fast_resolver` / `run_sealed` / `sealed_reset`) + `SealedStationScenario` + a new
`slow_reset` hook on `station.driver.run_master_day` (the annual re-sow machinery the ≤7-day
runs never fired) + shared `tests/sealed_tier2_helper.py` + a session-scoped
`sealed_tier2_run` conftest fixture; three test files (stability / landmine / regression) +
two additive **NON-frozen** goldens (`sealed_station_state.json`,
`sealed_energy_drift_summary.json`). **Zero core + zero domain change** (`git diff
src/{simcore,domains}/` empty). **Execution findings vs the design (advisor-guided):** (1)
the load-bearing spike PASSED — the coupled biosphere under pinned CO₂ (scrubber-held Ci≈258)
is **period-1** (grain-at-re-sow byte-identical every year) with a **converging** decomposer
pool (peak total-organic-C 29.10→29.196→29.196, diffs shrinking ~450×), re-sows cleanly,
`rationed==0`; (2) the `annual_reset` hook was the real work item (a plumbing gap, not a
physics finding — the discriminator the advisor flagged); (3) **harvest DROPPED from Tier 2**
(measured: it drains `storage_c` to 0.011<0.16 seed bank → starves the re-sow; its food-loop
conservation is pinned in Step 6); (4) Power runs **constant daily-average** solar/load in the
fast lane (`substep` freezes `n` ⇒ diurnal shape inexpressible — the Step-5 lamp-average
precedent; the diurnal SOC swing + node attractor are Tier 1's job via single-rate
`run_station`); (5) the `drift.py` **absolute** bounds do NOT transfer (station stocks 1e0–1e10;
OXYGEN's `o2_supply` reaches ~2e5 while its conserved total is ~27) — normalize by the **max
single-stock magnitude** (`quantity_scale`), not `total(0)`, giving horizon-invariant relative
drift ~1e-11 abs / ~1e-14 slope; (6) the biomass watch uses **total organic C** (not peak
leaf_c, which hides the decomposer) and asserts the year-over-year diffs **SHRINK** (genuine
convergence, stronger than `is_stationary`'s non-amplifying clause which passes a linear ramp);
(7) **Tier 3 landmine** (`close_feces=True`) sharper than designed — litter is quasi-steady
(~2600) but `microbial_carbon` grows unbounded and **`rationed>0` from day 25**; the *same*
`is_stationary(bound=1.0)` that passes Tier 2 (~0.09 diffs) FAILS the landmine (~1e4 diffs, via
the bound clause) — the symmetric discriminator. **Deliverables: Tier 1** (energy decade,
`run_station` 15 yr, node period-1 fixed point at `T_eq≈160.08 K`, SOC daily-periodic, ENERGY
relative drift flat) + its **drift-summary golden**; **Tier 2** (3-season combined-ledger run,
~915 days × 1440 sub-steps ≈ 1.3 M sub-steps ~3 min, every-quantity + ENERGY relative drift
flat, biomass bounded/converging, regulated pools stationary, `rationed==0`, feces open) + its
**final-State golden**; **Tier 3** (assertion-only landmine). Full suite incl. `-m slow` + ruff
+ pyright green; **all twenty existing goldens byte-identical** (the two new goldens are
additive, NOT in the freeze manifest). Original design (unchanged, retained for provenance):

**Step 7 (P6.7) — the sealed station: multi-year matter+energy stability — DESIGN
(just-in-time, advisor-reviewed).** The Phase-4 analogue at station scale: assemble the
fully-coupled sealed station, run multi-year, and prove **conservation of matter AND energy
holds, numerics are stable (no drift / no collapse), and the cross-domain dynamics are
believable**, reusing the `biosphere/drift.py` instrument across every conserved quantity
**and** ENERGY. Zero core, zero domain change (the Phase-6 discipline); a new
`src/station/` assembly + tests + additive NON-frozen goldens.

- **The thesis splits in two — keep them separate, never conflate (advisor-reviewed).**
  - **(A) Integration + longevity — the genuinely NEW thing Step 7 proves.** *Not*
    "combined matter+energy conservation" in a cross-quantity sense — energy and matter
    **share no stock** (the lamp couples them only through the PAR *forcing*; no flow
    converts energy↔matter), so per-quantity balance holds flow-by-flow and "the combined
    ledger conserves" reduces to "two disjoint per-quantity ledgers each conserve," which
    the short-horizon Steps 1–6 already showed. What is genuinely new is that the **full
    multi-seam assembly** — one `State`, ~11 flows across five domains, merged slow/fast
    resolvers, no stock- or flow-id collisions — **sustains** every-quantity conservation to
    round-off with **no drift** over **many annual cycles**: `drift.py` axis-(a)
    (`mass_drift_slope` ~ machine-ε, `max|d_q|` at the round-off floor, not the ceiling)
    flat across **every** quantity *and* ENERGY on the day-boundary trace. Integration +
    longevity, not a conversion-conservation the model does not contain.
  - **(B) Physical stationarity — a per-subsystem CHARACTERIZATION, not a whole-station
    claim.** **Energy** earns a genuine full-subsystem attractor-stationarity proof (a real
    `T_eq`, a permanent `boundary.space`, daily-periodic forcing — the clean Phase-4
    analogue). **Matter** earns conservation-to-round-off + numerical stability +
    regulated-*pool* stationarity (ECLSS / recovery hold CO₂/O₂/H₂O at their setpoints);
    whole-system matter stationarity is **deferred** — the open-loop crew stores deplete
    monotonically (provisioning is not closed) and the food/litter magnitude is uncalibrated
    (Step 9). **The golden's framing must never say "the station is stationary."**

- **The fork: SCOPE, do NOT calibrate (advisor-endorsed).** The plan's own Step-7 fork was
  "calibrate the crew-vs-plant scale before the multi-year run, OR scope to the subsystems the
  regulators hold stationary." Take **scope**: calibrating crew-vs-plant scale is the substance
  of **Step 9** (NASA BVAD, clean-room from primary literature); doing it now front-runs Step 9
  and picks numbers with no literature basis (just to pass a test). Concretely, **exclude the
  feces→litter coupling via `close_feces=False`** — the existing greenhouse default. That is
  *principled scoping, not an ad-hoc exclusion*: litter/microbial is the **one unregulated
  loop**, so turning it off **is** "scope to the loops the regulators hold." Matter is then open
  at the feces boundary — consistent with being open at store provisioning. No new machinery.

- **The driver is NOT the obstacle — it is a TWO-rate build (advisor corrected my first
  read).** Power is forced / dt-linear and Thermal's T⁴ radiator is *more* Euler-stable at
  smaller `dt`, so **both sit in the fast registry at `dt = 60 s`** alongside the cabin — the
  station is one slow domain (biosphere, 1 day) + one fast domain (everything else, 60 s),
  which the existing `run_master_day` already expresses. There is **no** three-time-scale
  problem. (`run_station`'s single-rate energy loop is untouched and needs only a horizon bump
  for Tier 1 below — zero new code there.) The two *real* gates to a single unified build are
  **compute** and **a unified scenario** (below).

- **Compute is the real lever — measured 2026-07-02 (`GREENHOUSE_SCENARIO`, 24 stocks).** The
  cabin sub-step costs **~76 µs** (5 flows + a full-ledger conservation assert each). So:
  - A unified **15-yr** run ≈ 1440·365·15 ≈ **7.9 M** sub-steps ≈ **~10 min** — too slow even
    for a marked-`slow` test (the Phase-4 100k stress is ~30 s). **15-yr unified is out.**
  - A unified **~2-yr** run ≈ 1440·365·2 ≈ **1.05 M** sub-steps ≈ **~80 s** — acceptable
    marked-`slow`. **This is the combined-ledger horizon.**
  - The **energy loop alone** (Power→Thermal, 24 steps/day) at 15 yr ≈ 131 k steps ≈ **seconds**
    — the decade-scale energy proof is cheap.
  - **Memory is a non-issue:** `run_master_day` retains only **day-boundary** states (≈730 for
    2 yr) and asserts conservation after **every** sub-step *internally*, so a completed run
    already proves per-step closure, and axis-(a) drift runs on the ~730-point day-boundary
    trace. No streaming/chunking needed (unlike the Phase-4 stress, which streamed only to
    bound a full-`list[State]` retention this driver never does).

- **The three-tier delivery (forced by the compute finding).**
  1. **Tier 1 — the energy decade proof (the clean Phase-4 analogue, cheap).** `run_station`
     (Power→Thermal) at **15 yr**, ENERGY only: a real emergent `T_eq` attractor, a permanent
     boundary, daily-periodic forcing. `drift.py` on ENERGY → `mass_drift_slope` flat
     (conservation to round-off), `node`/T **stationary** (period-1 fixed point,
     `is_stationary` + `non_collapsing`), SOC a period-1 daily cycle, `rationed == 0`,
     `events == ()`. **Only a scenario-horizon change** (a `SEALED_ENERGY_YEARS` alias); zero
     new engine code. This is the genuine full-subsystem attractor proof (B) promises for
     energy.
  2. **Tier 2 — the combined-ledger multi-year conservation (the NEW thing, (A)).** The unified
     fully-coupled two-rate sealed station: biosphere-slow (1 day) + everything-fast (60 s) —
     Power→Thermal (energy, waste-heat legs → `thermal.node`, the Step-1 inward seam) +
     biosphere ↔ cabin ↔ crew ↔ ECLSS (the greenhouse) + water-recovery + lighting (energy →
     biology) + harvest (biomass → food), **`close_feces=False`**. Run **~2 yr**, marked-`slow`.
     Proves the assembly sustains every-quantity + ENERGY conservation to round-off every
     sub-step over many annual cycles (A); regulated pools (CO₂/O₂/H₂O, node/T) stationary;
     `rationed == 0`; whole-system matter stationarity honestly deferred (stores drain —
     documented, not asserted stationary). **Critically, Tier 2 must WATCH the coupled
     biosphere's own biomass trajectory** (a year-boundary summary — peak `leaf_c` or total
     organic C — added to the axis-(a) drift check), because the coupled plant runs under a
     **pinned-CO₂** regime the freeze never validated (see the deferrals note): a
     slowly-growing coupled biosphere is **mass-conserving** (the scrubber tops `CARBON_POOL`
     back up), so axis-(a) alone stays flat and would silently mask it — the same failure
     mode Tier 3 catches for litter, which must not reappear unwatched in the plant.
     - **The unified scenario is the second real gate (a new composition, moderate).** One
       biosphere that both **breathes cabin air** (`CO2_POOL_VAR → CARBON_POOL`, the greenhouse
       reverse seam, unchanged) **and is lit by the lamp** (point `PAR_VAR` / `DAYLENGTH_VAR` at
       the Step-5 lamp schedule instead of the weather table). The fast registry then holds the
       5 cabin flows + `SolarCharge`/`LoadDraw` + `Lamp` + `RadiatorReject` + `WaterRecovery` +
       `Harvest` (~11 flows), all at `dt = 60 s`; the biosphere-slow registry is `build_season`
       verbatim. **Fallback if the lamp+cabin-air+harvest composition proves thorny:** drop to
       **greenhouse + energy + water-recovery** — the combined ledger *still* spans ENERGY (the
       Power/Thermal stocks are disjoint from the matter stocks, so per-quantity balance holds
       flow-by-flow) plus all matter; only the energy→biology *dynamic* coupling is lost, not
       the conservation payload.
  3. **Tier 3 — the landmine as a TEST, not a prose caveat (advisor addition).** A *separate*
     run with **`close_feces=True`** at illustrative scale, run long enough that `litter_carbon`
     grows unbounded → `drift.py` axis-(b) **correctly flags** the non-stationarity
     (`is_stationary` fails: `|same_phase_diff|` amplifies) and, run further, the once-daily
     microbial O₂ draw overtakes `O2Makeup`'s daily refill → **`rationed > 0`** (the Euler
     backstop firing). This is the drift instrument *earning its keep* and **empirically pinning
     the Step-9 calibration prerequisite** — far stronger than a comment. (An assertion test, no
     golden: it deliberately produces a non-stationary / rationing run.)

- **EXECUTION STEP 0 — go/no-go spikes, BEFORE any test/golden scaffolding (the Step-6
  discipline; each can force a scope pivot).**
  1. **Power + Thermal stable at `dt = 60`.** Confirm the nonlinear radiator Euler-steps
     cleanly at 60 s (τ grows 60× in step units ⇒ *more* stable, but verify), `rationed == 0`,
     ENERGY closed — and that the solar/load/lamp schedules are expressible per-60-s-substep
     (they are forced / dt-linear). If not, energy stays at its own `dt` in a *separate* fast
     lane (still two-rate) — but this should just work.
  2. **The unified lamp+cabin-air biosphere assembles + conserves for one master day.** A
     one-day run of Tier 2's build with every quantity + ENERGY closed after each sub-step (the
     Step-2 composition-mismatch-style gate). Decides recommended-vs-fallback for Tier 2.
  3. **The coupled biosphere completes ≥2 annual cycles cleanly under PINNED CO₂ — the
     first-order unknown (advisor-flagged).** This is the load-bearing spike. The coupled plant
     runs with `CARBON_POOL`/`O2_POOL` **held at ECLSS setpoints** (Ci ≈ 250 constant,
     regulator-erasure) — a *different atmospheric boundary condition* than the freeze ever
     tested (the freeze's sealed chamber has a self-swinging CO₂ pool, part of its period-2
     cycle). So the coupled biosphere's multi-year biomass trajectory + clean `annual_reset`
     re-sow are **empirical unknowns, MEASURED here, not inherited**. Confirm — **independent of
     harvest** — that over ≥2 yr the plant persists, `annual_reset` re-sows, biomass stocks stay
     **bounded** (no year-over-year growth/decay ramp), `events == ()`, `rationed == 0`. **Only
     then** layer harvest on as the *second-order* worry: `annual_reset`'s seed-bank guard needs
     `storage_c ≥ seedling_total`, and harvest drains `storage_c`, so a too-greedy `k_harvest`
     **starves the re-sow** → collapse. If the pinned-CO₂ base run drifts, that is the finding
     (a Step-9 / recalibration input, characterized like Tier 3); if only harvest starves the
     re-sow, **drop harvest from the Tier-2 run** (its food-loop conservation is already proven
     short-horizon) or size `k_harvest` to leave the seed bank.
  4. **The ~2-yr Tier-2 wall-clock is acceptable marked-`slow`** (re-measure on the *full* ~11-
     flow build; ~80 s is the ~5-flow estimate — the real build is heavier). If it blows the
     budget, shorten to the smallest horizon that (a) is ≥2 yr and (b) fires `annual_reset`
     ≥once, or thin the assert cadence with an env knob (the Phase-4 stress `slow` precedent).

- **Honest scope / deferrals.** *Calibration:* crew-vs-plant magnitude → **Step 9** (the
  feces→litter mismatch, the store-provisioning imbalance). *Matter closure:* open at the feces
  boundary (`close_feces=False`) and the store provisioning — a **characterization**, not a
  closed ecosystem. *Biosphere stationarity:* **NOT inherited from the freeze —
  MEASURED** (advisor-corrected). The freeze validated a *sealed chamber with a self-swinging
  CO₂ pool*; the coupled plant runs under a **pinned**-CO₂ regime (scrubber-held setpoints) the
  freeze never tested, so it is not the same dynamical system and its multi-year trajectory is
  an empirical unknown (see go/no-go spike #3 + the Tier-2 biomass-drift watch). What Tier 2
  *does* inherit is Euler-lock (⇒ Euler-only, below); what it must *measure* is the coupled
  biosphere staying bounded under pinned CO₂ over ≥2 annual cycles. Tier 2 does **not** re-derive
  the biosphere's period-2 cycle (only ~2 cycles fit — too few for `is_period_2`); that discrete
  period characterization was Phase-4's job (to 328 yr, under the *self-swinging* boundary). *Numerics:* the coupled build carries the
  frozen biosphere ⇒ **Euler-only** (no RK4 cross-check); Tier 1's energy loop *could* RK4-cross-
  check but the standalone Thermal already did, so it adds no decision value here.

- **The artifacts to create** (the established per-step skeleton; `system.py`/Step 1 supplies
  the energy assembly, `harvest.py`/Step 6 the maximal matter assembly + the two-rate driver).
  1. `src/station/sealed.py` (a non-reserved name — cf. the `aux.py`→`auxiliary.py` rule):
     `build_sealed_station` composing all seams over one shared stock dict + two registries
     (biosphere-slow / everything-fast), with a `close_feces: bool` and a `with_energy_coupling`
     (lighting) knob for the fallback; the unified fast/slow resolvers; a thin `run_sealed`
     wrapper over `run_master_day`; module-level `FlowId` constants for any new ids (none
     expected — all flows are reused). Assert the two registries' flow-id sets disjoint (the
     `build_harvest` guard) and the biosphere/cabin/energy stock-id sets disjoint.
  2. `src/station/scenario.py`: a `SealedStationScenario` referencing the sub-scenarios
     (greenhouse + Power + lamp + water-recovery + harvest) + horizons (`SEALED_STATION_YEARS`
     for Tier 2, `SEALED_ENERGY_YEARS` for Tier 1). No new params (all loaded from the sibling
     YAMLs).
  3. `tests/test_sealed_station_stability.py` (marked `slow`): Tier 1 (energy decade —
     `drift.py` ENERGY stationarity + closure + the drift-summary signature) and Tier 2 (~2-yr —
     per-sub-step closure across every quantity + ENERGY via the run completing, axis-(a) drift
     flat on the day-boundary trace for *each* quantity + ENERGY, **coupled-biosphere biomass
     bounded** (a year-boundary summary — the pinned-CO₂ watch, so a conservation-masked biomass
     ramp is caught), regulated-pool stationarity, `rationed == 0`, `events` handled, determinism
     + registration-order independence).
  4. `tests/test_sealed_station_landmine.py` (Tier 3, assertion-only, no golden): `close_feces=
     True`, `litter_carbon` grows unbounded, axis-(b) `is_stationary` **fails**, and (run
     further) `rationed > 0` — the instrument flagging the uncalibrated non-stationarity.
  5. `tests/test_regression_sealed_station.py` + `tests/regression/golden/sealed_station_state.
     json` (Tier-2 day-boundary final `State`). Pre-golden gate bakes in: `rationed == 0`,
     every-sub-step closure, energy at the dissipation-set `T_eq`, regulated pools at their
     setpoints, coupled biosphere biomass bounded, feces boundary open (Tier-2 scope). Additive
     **NON-frozen**, not in the freeze manifest, `__main__` regen. **The Phase-4 Step-4
     drift-*summary* stability-signature golden belongs on TIER 1** (energy, 15 yr — where a
     period class + per-year summaries are actually characterizable), **not Tier 2** (~2 yr = 2
     per-year points, no period class → it would pin noise). So Tier 1 gets the drift-summary
     golden (energy stationarity signature; mass-drift round-off deliberately not pinned); Tier
     2 gets only the final-State golden + gate above.

- **Exit criteria (same as every Phase-6 step):** `git diff src/simcore/` empty (zero core
  change), `src/domains/` untouched (zero domain change), full suite incl. `-m slow` + ruff +
  pyright green, and **all twenty existing goldens byte-identical** (the new
  `sealed_station_state.json` + drift-summary golden are additive). Then update the plan doc,
  `CLAUDE.md`, and memory; commit + push to `main`.

*Design-input landmine from Step 6 (advisor-flagged, retained for provenance — Tier 3 turns
it into a test):* Step 6's feces→litter loop is **not a bounded attractor** at illustrative
scales — the ~3400× crew-vs-plant mismatch means litter influx (~52 mol C/day) vastly exceeds
microbial consumption, so `litter_carbon` grows monotonically (342 mol by day 7, still
climbing) and `microbial_carbon` with it (→20 mol). Over 7 days `rationed == 0`, but at
multi-year scale (a) the once-daily microbial O₂ draw grows with the litter pile and
eventually exceeds `O2Makeup`'s daily refill → `rationed > 0`, and (b) the unbounded growth is
itself a non-stationarity `drift.py` flags — an artifact of the uncalibrated mismatch, not a
real instability. Scoped out of Tier 2 (`close_feces=False`) and *characterized* in Tier 3.

**Step 8 (P6.8) — cross-domain perturbation harness (cascades, no cascade code) —
DESIGN (just-in-time, advisor-reviewed 2026-07-02, spike-measured).** The Phase-3
`perturbations.py` discipline carried **cross-domain**: compose brownout / radiator
failure / atmosphere leak / crew load spike / lighting failure onto the *assembled*
station inputs **outside** the builder, and assert each cascade propagates through shared
stocks (or a shared *forcing*, #16) **alone** — conservation still holds, `rationed`
*behaves*, and the failure signature is the *emergent* one. Zero core / zero domain
change; **no golden** (a perturbation is a behavioural demonstration — the Phase-3
"diagnostics, no golden" precedent; determinism re-run stands in). All twenty existing
goldens stay byte-identical.

*The one genuinely-new thing vs Phase-3.* Phase-3's three perturbations were
**single-domain** (all inside the biosphere). Step 8's novelty is that a disturbance
applied to **one** domain cascades into **another** through a shared stock / shared
forcing, with no cascade code. So each shipped perturbation must demonstrate a *distinct*
cross-domain propagation — gated (Step-6/7 discipline) on a **go/no-go spike** showing a
distinct emergent signature; if two collapse to the same signature they merge (no silent
cap — the drop is logged).

*The load-bearing finding (advisor #1, spike-CONFIRMED): the station regulators ERASE the
naive pool-level signature.* Unlike Phase-3's un-regulated chamber, every station gas pool
is regulated, so in **every** matter perturbation the day-boundary `CARBON_POOL` / `O2_POOL`
/ `Ci` come back **identical to baseline** (the Step-3 regulator-erasure physics, now under
disturbance). The emergent signature is regulator **effort** + the sinks, **not** pool
level — and the two gas pools **do not fail the same way**, so each is spiked:
- **`CARBON_POOL` is only *removed* (`CO2Scrubber` is first-order donor-controlled — it
  cannot push CO₂ back *up*).** A leak genuinely lowers it *within* the window ⇒ Ci dips ⇒
  the plant assimilates **less** (`biomass↓`, spike: 0.280 vs 0.337 mol) and the scrubber
  does **less** work (`co2_removed↓`, spike: 2202 vs 2937). Signature = biology + scrubber
  effort down.
- **`O2_POOL` is actively *defended* (`O2Makeup` is demand-controlled toward a setpoint).**
  A leak barely moves `cabin_o2`; it shows up as `o2_supply` **effort** (spike: −5327 vs
  −2935, +81 % makeup) while biology is untouched. Signature = makeup effort up.

*The five perturbations — three seam-types, two substrates (all spike-measured 2026-07-02):*

| perturbation | seam-type | substrate | emergent signature (spiked) | `rationed` |
|---|---|---|---|---|
| **brownout** | forcing override (`solar_power`) | diurnal `run_station` | SOC↓, node **cools**; scales with severity | graceful arm 0 / deep arm **>0** |
| **radiator failure** | **windowed flow-scaler** (`RadiatorReject`) | diurnal `run_station` | node **heats** (T 176 vs 160), space monotonic | 0 (pool accumulation, not a shortfall) |
| **atmosphere leak** | added leak flow (`LeakFlow`) | short `run_sealed` | `leak_sink↑`; CARBON: `biomass↓`+`co2_removed↓`; O₂: `o2_supply` effort↑↑, `cabin_o2` flat | 0 |
| **crew load spike** | forcing override (`food_intake`) | short `run_sealed` | `co2_removed↑`+`o2_supply↑` (both regulators) + `food_store↓` faster | 0 |
| **lighting failure** | forcing override (`par` **and** `lamp_power`) | short `run_sealed` | `biomass↓` (growth stalls) + `battery` **saved** (energy↔biology, #16) | 0 |

- **The three seam-types.** (1) **Forcing override** — reuse Phase-3's generic
  `window_override` + `with_forcing` (pure schedule/resolver transforms, domain-agnostic;
  the station is the assembly layer, it imports them). Brownout scales `solar_power`; crew
  spike scales `food_intake`; lighting failure zeroes **both** `par` (biosphere resolver)
  **and** `lamp_power` (fast resolver) together — the #16 lamp is one intervention with an
  energy leg and a photon leg. (2) **Added leak flow** — reuse Phase-3's `LeakFlow` +
  `LEAK_SINK` (generic `pool → sink`, first-order, windowed, composition-mirroring so a
  `{C:1,O:2}` pool vents CARBON+OXYGEN in balance). (3) **Windowed flow-scaler** — the
  legitimately-new third seam-type (advisor-endorsed): a `ScaledFlow` that wraps an
  existing flow and multiplies **all** its legs by a windowed `health ∈ [0,1]` forcing, so
  the **whole flow scales** and stays internally balanced (the "arbitration scales the whole
  flow" invariant, applied as a perturbation; `health = 1` outside the window is
  **bit-identical** to baseline — `x·1.0 == x`). Radiator failure = scale `RadiatorReject`
  to ~0 over a window.

- **Two substrates (the Phase-3 "asymmetric assignment by design," now by physics + compute).**
  *Energy* perturbations (brownout, radiator failure) run on the **cheap single-rate diurnal
  `run_station`** (Power→Thermal, `dt = 3600 s`, seconds of wall-clock) — the diurnal SOC
  swing + the node attractor are only expressible where `n` advances. *Matter* perturbations
  (leak, crew spike, lighting) run on a **short two-rate `run_sealed`** (the maximal sealed
  build, so the cascade spans the most domains), `DAYS = 8`, window `[2, 7)` — ~1.25 s/arm,
  module-scoped fixtures so each run is paid once.

- **The `annual_reset` landmine — the short horizon is the FIX, so state it (advisor #2).**
  Phase-3 flags (and characterization-tests) that a *sustained/severe* perturbation starves
  grain → `annual_reset` raises `ValueError` at the season boundary. The sealed station's
  `slow_reset` fires at `n % 305 == 0`; a window inside year 1 (`n < 305`) **never reaches
  it**, so the landmine cannot bite — that is *why* the short horizon is correct, not merely
  a compute win. Windows stay inside year 1. (A sustained/severe arm asserting the raise is a
  *characterization* test, Phase-3 precedent — shipped only if it adds signal over Phase-3's,
  which already pins the biosphere-internal version; likely **deferred** as redundant.)

- **`rationed` *behaves* ≠ `rationed == 0` (advisor #3, spike-CONFIRMED).** Step 8's exit
  criterion is *emergent failure cascades*, so a deep/long brownout **should** empty the
  battery and produce `rationed > 0` — that is the payload, not a bug (the Tier-3 landmine
  precedent: rationing as deliberate characterization). Spiked: this Power sizing is tight
  enough that a ≥1-day full blackout empties the battery (`rationed` scales with severity —
  111 → 351 → 591), while a **short/shallow** cut (a ~5 h 50 % afternoon dip) stays graceful
  (`rationed == 0`, SOC dips to 6.6e6 > 0, node cools). So **brownout carries both regimes**
  — a graceful arm (`rationed == 0`, conservation + node-cool cascade) and a failure arm
  (`rationed > 0` **bounded**, still conserving — the Euler backstop conserves as it rations).
  The other four are graceful (`rationed == 0`).

- **Conservation — proven by the run completing, plus a relative day-boundary check
  (advisor #4).** `run_master_day` / the integrator assert conservation after **every**
  sub-step over the whole shared ledger, and every perturbation leg is internally balanced
  (the leak vents to `LEAK_SINK` in the shared stock dict, so the driver's assert folds it),
  so a **completed** perturbed run *is* the per-sub-step conservation proof (an unbalanced
  perturbation would raise). Phase-3's absolute `TOL` table does **not** transport (station
  stocks span 1e0–1e10, ENERGY ~1e9, no ENERGY entry) — the extra day-boundary drift teeth
  reuse `sealed_tier2_helper`'s **relative** `quantity_scale` / `relative_drift`, summing
  **incl. `LEAK_SINK`** (total conserved even as the chamber interior's closure breaks — the
  Phase-3 leak discipline). Do **not** assert closure / `loss_sink == 0` for the leak.

- **The artifacts to create.**
  1. `src/station/perturbations.py` — the station perturbation harness. Reuses Phase-3's
     `window_override` / `with_forcing` / `LeakFlow` / `LEAK_SINK` / `LEAK_VAR` (imported from
     `domains.biosphere.perturbations` — generic); adds the new `ScaledFlow` wrapper +
     `with_radiator_failure`, `with_brownout`, `with_crew_load_spike`,
     `with_lighting_failure`, and a two-registry `with_station_leak` (the sealed build has
     **two** registries + one shared stock dict — the leak lands in the **fast** registry,
     its `FlowId` kept **out of `bio_reg`** to satisfy the disjointness guard; `k_leak·dt < 1`
     is trivial at 60 s — spiked `k_leak = 1e-3` ⇒ `k·dt = 0.06`, the `k_scrub` scale). Small
     builder functions + the two shared helpers, **not** a `Perturbation` protocol (the
     Phase-3 "no speculative generality" call — 5 perturbations, additive if a real
     composition need appears).
  2. `tests/test_station_perturbations.py` — the cascade demonstrations. **Direction-only**
     asserts (the Phase-3/Step-4/5 anti-flakiness rule — never a magnitude or a day index;
     per-stock, never `State == State`), each a **perturbed vs baseline** contrast on the
     *emergent* signature (regulator effort + sinks, per the finding above), plus: the
     `rationed` split (graceful `== 0` / brownout-deep `> 0` bounded), conservation
     (completed run + relative day-boundary drift incl. `LEAK_SINK`), the orphaned baseline
     sink **absent** where re-pointed, and **determinism** re-runs (the no-golden insurance;
     fine on `run_station` despite `math.sin` — within-build bit-stable, the Power-golden
     precedent). Module-scoped fixtures compute each run once. Any ledger-reconstruction check
     binds the **perturbed** resolver (Phase-3's explicitly-flagged bug).

- **Exit criteria (same as every Phase-6 step):** `git diff src/simcore/` empty (zero core
  change), `src/domains/` untouched (zero domain change), full suite incl. `-m slow` + ruff +
  pyright green, and **all twenty existing goldens byte-identical** (no golden added). Then
  update the plan doc, `CLAUDE.md`, and memory; commit + push to `main`.

**Step 8 (P6.8) COMPLETE — EXECUTION (2026-07-02).** Built exactly as designed above; every
spike prediction held, no surprises forced a redesign. `src/station/perturbations.py` (the
station harness) + `tests/test_station_perturbations.py` (17 tests, **NOT** slow-marked —
each substrate is seconds). **Zero core change** (`git diff src/simcore/` empty) + **zero
domain change** (`src/domains/` untouched — the harness *imports* Phase-3's generic
`window_override` / `with_forcing` / `LeakFlow` / `LEAK_SINK` / `LEAK_VAR` from
`domains.biosphere.perturbations` and composes them at the station layer). **No golden**
(the Phase-3 "diagnostics, no golden" precedent; determinism re-runs are the insurance).

- **The five perturbations, three seam-types, two substrates — all shipped as designed.**
  (1) *Forcing override* (reused Phase-3 generics): `with_brownout` scales `solar_power`,
  `with_crew_load_spike` scales `food_intake`, `with_lighting_failure` zeroes **both** `par`
  and `lamp_power` together (the #16 lamp is one intervention, two legs). (2) *Added leak
  flow*: `with_station_leak` adds a windowed `LeakFlow` to the **fast** registry over the
  shared stock dict (its `FlowId` kept out of `bio_reg` for the disjointness guard), the
  `LEAK_SINK` composition **mirroring the pool** so a `{C:1,O:2}` pool vents CARBON+OXYGEN in
  balance. (3) *Windowed flow-scaler* — the new `ScaledFlow` (frozen dataclass wrapping an
  inner `Flow`, `id`/`priority` delegated, `evaluate` multiplies **all** legs by a windowed
  `health ∈ [0,1]` forcing); `with_radiator_failure` wraps `RadiatorReject` and rebuilds the
  `Registry` over `state.stocks`. **`health = 1` outside the window is bit-identical to
  baseline** (`x·1.0 == x`) — shipped as `test_radiator_failure_outside_window_is_baseline`.
- **The regulator-erasure finding held under every matter perturbation** — the emergent
  signature is regulator *effort* + sinks, so the tests assert on `co2_removed` / `o2_supply`
  / `biomass` / `food_store` / `leak_sink`, never the day-boundary pool level (which
  `test_crew_spike_pools_return_to_setpoint` + `test_lighting_failure_carbon_pool_returns_to_setpoint`
  positively pin **back to setpoint** via `math.isclose`, the finding as a teeth test).
- **`rationed` behaves, both regimes shipped:** `test_brownout_graceful_cools_node_without_rationing`
  (a ~5 h 50 % afternoon dip, `rationed == 0`, SOC dips, node cools) **and**
  `test_brownout_deep_emerges_rationing_still_conserving` (a multi-day full blackout,
  `rationed > 0` **bounded**, still conserving — the Euler backstop conserves as it rations).
  The other four arms are graceful (`rationed == 0`).
- **Conservation** via the completed-run proof (per-sub-step ledger assert) **plus** relative
  day-boundary drift teeth reusing `sealed_tier2_helper`'s `relative_drift` /
  `REL_DRIFT_BOUND` over `(CARBON, OXYGEN, WATER, NITROGEN)`, summing **incl. `LEAK_SINK`**
  (total conserved as the chamber-interior closure breaks — the Phase-3 leak discipline; no
  `loss_sink == 0` assert for the leak arms). Energy substrate: brownout/radiator on the
  cheap single-rate diurnal `run_station` (module-scoped `energy_baseline`, `_E_DAYS = 12`).
  Matter substrate: leak/crew/lighting on a short two-rate `run_sealed`
  (`SealedStationScenario(years=1, season_days=305)`, `_M_DAYS = 8`, window `[2, 7)` — inside
  year 1 so the `slow_reset` landmine cannot bite). Spiked `k_leak = 1e-3` (`k·dt = 0.06`).
- **Determinism** re-runs on both substrates (`test_radiator_failure_is_deterministic`,
  `test_matter_perturbation_is_deterministic`) stand in for the absent golden.
- **Verification:** full suite incl. `-m slow` + ruff + pyright green (**1338 passed, 1
  skipped**); **all twenty existing goldens byte-identical** (no regen). **Phase 6 continues
  → Step 9 (NASA BVAD / BioSim validation).**

**Step 9 (P6.9) — NASA BVAD validation (one crew configuration). COMPLETE** (2026-07-02;
executed as designed — the RQ ~11.8 % O₂ under-prediction is the measured headline; six
downstream goldens deliberately regenerated, frozen seven + Power/Thermal/ECLSS untouched;
sealed Tier-2 re-verified to converge; 1311 passed). DESIGN (advisor-reviewed,
just-in-time) below. Clean-room from **primary literature** under `docs/reuse-and-licenses.md` —
cite the reference, copy no dataset (the PCSE-oracle discipline). Validate integrated crew
consumption/production against NASA BVAD for one crew config, and bind the deferred crew
physiology params to that reference before Step 10 freezes the station.

**Primary source (recorded durably in `docs/bvad-reference.md`):** NASA/TP-2015-218570 **Rev 2**
(Feb 2022), Table **3-31** "Summary of Nominal Human Metabolic Interface Values", p. 58 — 82 kg
reference crewmember, RQ **0.860**. Key per-CM-day values: CO₂ load **1.085 kg** (= 24.654 mol),
O₂ consumed **0.895 kg** (= 27.970 mol), food solids dry **0.800 kg**, fecal solid dry **0.032 kg**,
resp+persp water **2.946 kg**, urine water **1.420 kg**, metabolic water **0.490 kg**, metabolic
heat **12.426 MJ** (≈ 143.8 W). BioSim is architecture-only (license unverified) — **numbers cite
BVAD, not BioSim** (`reuse-and-licenses.md`).

**The load-bearing framing (advisor): calibration ≠ validation; the ONE genuine structural output
is RQ.** The crew flows are *forced* (intake rates = scenario data) and the splits are params, so
every quantity we *set* matches BVAD **by construction** (vacuous). Three columns, kept visibly
separate in the test:
- **Calibrated (set → matches, "calibration checkpoints" NOT validation):** food-carbon intake,
  water intake, CO₂ production, feces, humidity, urine.
- **Structural prediction (genuine — can fail):** `CrewRespiration` is **PQ = 1 ⇒ RQ = 1.0** (one
  mol O₂ consumed per mol CO₂ produced), independent of the fraction values. Calibrate CO₂ to BVAD
  (24.654 mol) ⇒ model O₂ = 24.654 mol = 0.789 kg vs BVAD 0.895 kg. **RQ = 1 cannot hit both O₂ and
  CO₂** — it forces O₂ ≈ **11.8 % low** (or CO₂ ≈ 13.4 % high if O₂ is the calibrated one). Pinned as
  a **number, not a bound**: `model_O2 / bvad_O2 = 0.8814 ± tol` (the daily-effective molar RQ; a
  regression that silently changed RQ trips it — the `fit_order`/`nrmse` "measure the known error"
  discipline). **This ~12 % miss is the headline result of the step.**
- **Not modeled (honest gaps, documented):** metabolic water (0.490 kg/CM-d — our `WaterBalance` is
  intake-split only, no oxidation water); metabolic heat (143.8 W/CM — crew is not an ENERGY source
  into `thermal.node`); RQ variation with activity (0.86 nominal / 0.95–0.96 exercise — single fixed
  RQ). All three are seams for later, not Step-9 scope.

**Recalibrate BOTH crew fractions (the deferred debt comes due here — Step 10 freezes next).** The
churn cost is fixed at first touch, so recalibrate both (advisor):
- `insensible_water_fraction` 0.4 → **0.675** = 2.946/(2.946+1.420) — humidity vs urine water, clean
  from Table 3-31 (fecal water 0.101 kg excluded — our model has no fecal-water fate; note the
  modeling boundary so "water intake" is not read as BVAD's potable figure).
- `respired_carbon_fraction` 0.85 → **0.949** from the carbon balance `C_food = C_CO₂ + C_feces`:
  C_CO₂ = 24.654 mol, C_feces = 0.032 kg × **0.50** (carbon fraction of dry feces, 44–55 % — **Rose
  et al. 2015** CREST review citing Feachem 1983; *the same Rose 2015 BVAD Table 3-31 cites for its
  fecal numbers*) = 1.332 mol ⇒ f_resp = 24.654/25.986 = 0.949 (range [0.944, 0.955] over the 44–55 %
  uncertainty). Both `crew.yaml` values get BVAD/Rose citations, `TODO(cite)` removed.
- **Equipment rate-constants stay illustrative** (`eclss.yaml` k_scrub/k_cond/k_makeup, harvest/
  recovery rates): BVAD publishes no first-order τ, only steady-state *throughput* (= crew
  production), which the closure check already validates — and keeping them still leaves their
  goldens byte-identical.

**Deliberate golden regen (stated loudly — the departure from "byte-identical every step").** The
`crew.yaml` change moves the **6 non-frozen goldens downstream of the crew fractions**: `crew_state`,
`cabin_gas_state`, `greenhouse_state`, `harvest_state`, `water_recovery_state`, `sealed_station_state`
(regen via each test's `__main__`, pre-golden gates re-assert). **Byte-identical (the invariant that
matters, called out):** the **seven frozen biosphere goldens** (crew doesn't touch them) + Power ×2 +
Thermal + ECLSS (its own forced `CrewMetabolism` stand-in, verified independent of `crew.yaml`) + the
two demo + `n_limited`/`water_biting` + `lighting` + `station` + `sealed_energy_drift_summary`
(energy-only Tier-1, no crew). `crew.yaml` is **not** in the freeze manifest (verified).

**The deliverable — `tests/test_bvad_validation.py`, run on the CABIN assembly** (the "integrated"
word: crew respiring into ECLSS, not standalone-crew arithmetic). A BVAD-calibrated `CabinScenario`
built **in the test** (N-CM crew load = N × per-CM rates; N and per-CM values folded into the two
intake rates — doesn't touch the shipped `CABIN_GAS_SCENARIO`/its golden). Run to steady state, read
actual flow fluxes, assert:
1. **Calibration checkpoints** (tight band, labeled as such): per-CM CO₂ production, feces (→ dry via
   0.50), humidity, urine reproduce Table 3-31.
2. **Structural payload (pinned, can fail):** `model_O2_consumption / bvad_O2 = 0.8814 ± tol`.
3. **Closure (what "integrated" buys):** at steady state, ECLSS scrubber removal flux = crew CO₂
   production flux; O₂ makeup flux = crew O₂ consumption flux (throughput matches load).
No new golden (a validation-against-reference test, the `oracle_match` precedent — computed
comparison, not a regression pin).

**Verify-before-commit (advisor):** the Tier-2 sealed run converged at the *old* fractions; f_resp
0.85→0.949 (more carbon to CO₂ ⇒ higher scrubber load) and f_ins 0.4→0.675 shift the loads, so
**re-run the ~1.3 M-substep sealed Tier-2 stability test and confirm it still converges + regen its
golden** (the regulators should absorb the shift — an assumption until the run passes). Also re-run
the Step-8 perturbation suite (no golden; qualitative assertions) and the Tier-3 landmine.

**Zero core + zero domain change target:** `crew.yaml` is a Phase-5 domain *param* (a data/citation
change, not a `simcore/` or flow-class change) — `git diff src/simcore/` stays empty; the only
`src/domains/` touch is the two `crew.yaml` values + citations. Everything else is station-layer
(the new test) + regenerated goldens + doc.

**Step 10 (P6.10) — whole-station golden capture + freeze the station.** The Phase-4
analogue: pin the sealed-station scenario's final `State` + a stability signature as
additive NON-frozen goldens, then a station freeze contract + manifest (the multi-domain
reference Phase 7's native port will target). Just-in-time design.

## Sequencing rationale

Energy first (Step 1): single-quantity, verified re-wirable, the cleanest seam, and it
stands up the station harness every later step needs. Gas second (Steps 2–3): the one
non-trivial representation decision (composition across the CO₂ loop + merged respiration),
now settled, done before anything builds against `cabin_co2`'s shape to avoid rework. Water
/ lighting / food (Steps 4–6): the remaining shared-stock loops, each just-in-time. Then
the sealed multi-year run (7), perturbation cascades (8 — needs enough seams to cascade,
which 1–6 provide), literature validation (9), and freeze (10) last, mirroring Phase 4's
"stabilize, then validate, then freeze" close.
