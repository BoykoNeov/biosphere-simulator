# Phase 6 — Station Integration (cross-domain coupling)

**Status: IN PROGRESS — Steps 1–2 (P6.1–P6.2) COMPLETE.** Pre-plan investigation complete and
advisor-reviewed (two blocking checks run before this doc committed to anything; see
"Load-bearing findings" below). Steps 1–3 are designed concretely here; Steps 4–10 get
just-in-time design as Phase 5's siblings did ("each designed just-in-time"), because each
later seam's shape is only fully constrained once the seam before it exists.

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
feedback).** Point the frozen biosphere's `ChamberWiring` (`carbon_source` / `resp_sink` /
`o2_pool`) at the **cabin's** composition CO₂/O₂ stocks, and map the `co2_pool` forcing-var
→ the cabin CO₂ id in the station resolver, so FvCB's `Ci` reads the live cabin CO₂
(finding #3). Plants draw CO₂ from and return O₂ to the cabin the crew breathes; the
CO₂/O₂ cross-domain feedback (crew exhales → cabin CO₂ rises → plants assimilate → cabin O₂
rises → crew breathes) emerges with **no control code**. Zero frozen-code change (station
re-wiring only); the standalone biosphere golden is untouched.

**Step 4 (P6.4) — the water loop.** Crew humidity + biosphere transpiration → cabin_h2o →
condenser → water recovery → crew `water_store` (+ urine → water recovery). Close WATER
across the station (the crew's finite `water_store`, open-loop standalone, becomes
regenerative). Just-in-time design.

**Step 5 (P6.5) — Power → biosphere lighting (energy enters biology).** A station **lamp**
flow: `power.battery → light-used + waste_heat` (→ `thermal.node`, ENERGY-balanced), and
the station resolver computes the biosphere's `par` **forcing** from the lamp's electrical
draw (replacing the weather-table PAR). Lighting is now powered; a later lighting/brownout
perturbation cascades into photosynthesis with no cascade code. Zero biosphere change (PAR
stays a forcing). Just-in-time design.

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
