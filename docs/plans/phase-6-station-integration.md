# Phase 6 — Station Integration (cross-domain coupling)

**Status: IN PROGRESS — Steps 1–5 (P6.1–P6.5) COMPLETE.** Pre-plan investigation complete and
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

**Step 6 (P6.6) — the biomass / food loop.** Biosphere harvest → crew `food_store`
(regenerative food); crew feces → soil/waste. Close CARBON through the trophic seam (the
crew's finite `food_store`, open-loop standalone, becomes regenerative). Just-in-time
design.

**Step 7 (P6.7) — the sealed station: multi-year matter+energy stability.** The Phase-4
analogue at station scale — assemble the fully-coupled sealed station, run multi-year, and
prove **conservation of matter AND energy holds, numerics are stable (no drift / no
collapse), and the cross-domain dynamics are believable**. Reuse the `biosphere/drift.py`
instrument (mass-drift ceiling, stationarity, period class) across every conserved quantity
+ ENERGY. Just-in-time design.

**Step 8 (P6.8) — cross-domain perturbation harness (cascades, no cascade code).** The
Phase-3 `perturbations.py` discipline, cross-domain: compose brownout / radiator failure /
atmosphere leak / crew load spike / lighting failure onto the assembled station inputs
*outside* the builder, and assert the cascade propagates through shared stocks alone
(conservation still holds; `rationed` behaves; the failure signature is the *emergent*
one). Just-in-time design.

**Step 9 (P6.9) — NASA BVAD / BioSim validation (one crew configuration).** Clean-room from
**primary literature** under `docs/reuse-and-licenses.md` — cite the reference, copy no
dataset (the PCSE-oracle discipline). Validate integrated consumption/production for at
least one crew config. Just-in-time design.

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
