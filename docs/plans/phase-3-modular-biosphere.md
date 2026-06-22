# Phase 3 — Modular Biosphere / Consumers

**Status: Steps 1–5 COMPLETE; Steps 6–7 not started.** Phases 0, 0.5, 1, and 2
are complete and regression-pinned (`docs/plans/phase-{0-engine-skeleton,0.5-numerical-foundations,1-single-producer,2-closed-chamber}.md`).
This plan **locks the load-bearing Phase-3 decision** (the subsystem-hierarchy
representation, P3.1) and **enumerates** the process steps as forward-pointers, each to
be designed just-in-time — the Phase-1/2 rhythm. Per that rhythm and the advisor's
"design the representation in full and review it before enumerating process steps," the
foundation (Step 1) is designed in detail below; the science steps are sketched only.
**Step 1 landed** the subsystem hierarchy: stocks re-domained into leaf compartments
(label-only — goldens regenerated with **domain-label-only diffs, byte-identical
amounts**), the domain-side parent map + `descendant_stocks` view, and the
per-compartment boundary-ledger diagnostic. Exit evidence: `git diff src/simcore/`
**empty** (clean Option B), 820 tests pass. **Step 2 (reusable compartment builders, P3.2)
is COMPLETE** — `season.py` split into `scenario` / `stocks` / `atmosphere` / `soil` /
`plants` builder modules (`water` deferred to Step 3); both goldens **byte-identical
WITHOUT regeneration** (the proof the restructure was safe); `git diff src/simcore/` still
**empty**; 836 tests pass incl. the new `test_builders` — the **disjoint-ownership** and
**no-cross-import** checks the snapshot goldens genuinely can't see (the `build_season`
union dedups by id; imports are pure source structure), plus a per-builder leaf-stamp
check that *localizes* a mis-stamp the goldens — which serialize `domain` — would also
catch. **Step 3 (close the water cycle, P3.3) is COMPLETE** — the sealed chamber's last
boundary leak is closed: `Transpiration` now feeds an in-system `water_vapor` (ATMOSPHERE)
→ `Condensation` → `condensate` (the WATER leaf's first stocks) → `Recycling` →
`soil_water`, a genuinely closed WATER ring; sealed drops `Irrigation` + `water_source`
(the nonzero irrigation flux would otherwise pump water in). Open golden **byte-identical
WITHOUT regeneration** (water leaf empty there); sealed golden **regenerated** (only WATER
moved — carbon/O₂/N + thermal_time byte-identical); `git diff src/simcore/` **empty**; 856
tests pass incl. the new `test_water_cycle` (WATER-scoped per-compartment ledger, behavioral
ring-wiring, closed-loop conservation, `rationed == 0`). **Step 4 (closure-preserving
mortality + annual reset, P3.4) is COMPLETE** — the last closure gap (an annual plant locks
its grain forever) and the prerequisite for emergent behaviour (a one-shot plant has no
sustained dynamics). A pure `annual_reset` driver transform applied at each year boundary
(`run_perennial`) zeroes `thermal_time` + redistributes the old plant's organ/grain carbon
(seedling retained from grain, the rest → `litter_carbon`, the loss-sink never touched) so a
new `PERENNIAL_CHAMBER_SCENARIO` shows **sustained multi-year oscillation** — DVS reaches
maturity every year, a stable emergent period-2 limit cycle — genuinely closed (carbon
loss-sink 0.0, `events == ()`), `rationed == 0`, four-quantity-conserved incl. across the
discrete resets. Open + sealed goldens **byte-identical** (no regeneration); new perennial
golden pinned; `git diff src/simcore/` **empty**; 870 tests pass. **Step 5 (modular
ecosystem assembly + per-compartment diagnostics, P3.1 ledger discharge) is COMPLETE** —
the deferred half of P3.1 landed as a **diagnostics + verification** step (no new science,
no behavior change, no new golden): the per-compartment boundary ledger now balances
**every step / every quantity / every compartment** (boundary included, no whitelist) on
the committed `PERENNIAL_CHAMBER_SCENARIO` — including the four annual-reset boundary steps
(the before-step re-derived with `annual_reset`, mirroring `run_perennial`'s predicate
verbatim; legs reconstructed test-side under Euler + `rationed == 0`, the Step-3
precedent). The extinction exception (a balanced non-flow change the legs can't see: `+r`
on the organ's compartment, `−r` on `boundary`) is discharged by a **hand-built
deterministic** unit test + a small domain-side `expected_extinction_residuals` helper
(probe: the sealed run does **not** go extinct over its horizon, so the optional sealed
full-ledger test was dropped, as designed). Plus the emergent cross-compartment
demonstration — CARBON genuinely *cycles* through PLANTS/SOIL/ATMOSPHERE (both crossing
directions) and the chamber CO₂ pool draws down then recovers within each year (the
reset→litter→decomposition→CO₂→regrowth cascade, direction-only). All three goldens
**byte-identical** (no regeneration); the helper lives in `compartments.py` so
`git diff src/simcore/` stays **empty**; 877 tests pass incl. the new
`test_compartment_ledger`. **Next: Step 6.**

**Goal (roadmap lines 270–303):** *Assemble a complete ecosystem from reusable
compartments.* The headline is an **architectural upgrade**, not new physics: introduce a
**hierarchical subsystem structure**

```
Biosphere
  ├ Atmosphere
  ├ Soil
  ├ Plants
  └ Water
```

where **each subsystem owns its stocks and flows** but **the integrator stays global**
(one clock, one ledger, one conservation gate). The roadmap's load-bearing claim:
*"A biosphere subsystem and a station domain are the same pattern at different scales. The
registry and resolver built here generalize directly to sibling domains."* Phase 3 is
where the **compartment-composition pattern** is born and proven on the biosphere — the
*same* pattern Phase 5 will reuse to compose Power / Thermal / Atmosphere / Crew into a
station. On top of the structure: close the **water cycle** (the one cycle still open),
make mortality **closure-preserving**, and stand up a **perturbation harness** that shows
cascades emerge for free. **Exit: a genuinely closed ecosystem exhibiting emergent
behavior** (decade-scale stability + freeze-as-reference is Phase 4, held out of scope).

**Source of truth for the phase sequence:** `roadmap_extracted.txt` (Phase 3 = lines
270–303). Phases 0/0.5/1/2 are complete; their APIs are frozen. Phase 2 changed **exactly
one** core surface (P2.1, the composition fold); **Phase 3 targets zero core *changes* —
at most one *additive* registry descriptor** (the parent map), defaulted backward-compatibly.

**Reuse/licensing:** `docs/reuse-and-licenses.md`. Clean-room from primary literature;
PCSE/WOFOST stay offline oracles only.

---

## Relationship to the roadmap + the central reframing

Roadmap Phase-3 **compartments**: Plant chamber, Atmosphere, Soil, Decomposition, Water
recycling, Optional consumers. **Architectural upgrade**: hierarchical systems (the tree
above), each subsystem owning stocks + flows, integrator global. **New cycles**: water,
carbon, nitrogen (+ optional P/S/methane), long-term soil change. **Perturbation testing**:
drought, nutrient shock, species removal/addition, pathogens, lighting failure, atmospheric
leak. **Exit**: a genuinely closed ecosystem exhibiting emergent behavior.

The non-obvious reframing — confirmed in review — is that **Phase 3 is an
organizational/reporting upgrade, not an engine change.** Three facts make this true, and
they drive the locked decisions:

1. **The multi-domain machinery already exists; Phase 3 *uses* it.** The registry already
   indexes stocks by `Stock.domain` and a flow may already name a source stock in one
   domain and a sink in another (`registry.py` docstring); the resolver already lets a flow
   read a sibling's shared stock indistinguishably from forcing (#16). So splitting the
   monolithic `biosphere` domain into `atmosphere`/`soil`/`plants`/`water` compartments is
   **mostly a re-namespacing plus a hierarchy view** — the cross-compartment flows
   (transpiration, condensation, mineralization) become genuine cross-domain flows through
   shared stocks. → **P3.1 (the representation) + P3.2 (reusable builders).**
2. **Re-namespacing collides with bit-identity — and the collision is avoidable.** Stock
   ids are canonically sorted on every float reduction (#15). If subsystems meant *renaming*
   ids (`biosphere.leaf_c` → `biosphere.plants.leaf_c`), the sort order — hence the float
   reduction order — would change, drifting goldens in **amounts**, not just labels, and
   breaking "bit-identical within a build." → **P3.1 locks: move only the `domain` *label*
   (which no float reduction keys on — verified: `conservation.py` sorts by stock id /
   quantity name; `domain` is used only in set-valued groupings), keep stock/flow ids
   byte-identical.** "Amounts unchanged, only labels moved" is then the self-check that the
   restructure was behavior-preserving.
3. **"Genuinely closed" forbids routing death to the loss-sink.** The numerical-loss sink
   is a BOUNDARY; an organism dying into it leaks mass out of a sealed chamber. The
   ecological death pathway must route an organism's carbon to **litter** (in-system,
   decomposable). → **P3.4: death-routes-to-litter; the loss-sink stays the numerical guard
   only.** This is the same fix as the Phase-2-deferred `storage_c` residue seam.

---

## Locked decisions (Phase 3)

New decisions, numbered `P3.n`, carrying the Phase-0/0.5/1/2 invariants they depend on.

### P3.1 — A subsystem is a **domain namespace in a parent tree**; the integrator stays **global** (no co-simulation); **stock/flow ids are NOT renamed**. *(The load-bearing decision — designed in full below; advisor-reviewed before any process step.)*

- **A compartment = a `DomainId`** (a leaf namespace) that owns a set of stocks and the
  flows over them. The hierarchy `Biosphere ├ Atmosphere ├ Soil ├ Plants └ Water` is a
  **parent map** `DomainId → DomainId | None` over leaf domains; `domain_index` already
  gives each domain its `frozenset[StockId]`. A subsystem is "a namespace + its stock/flow
  membership + a parent," **not a rich class and never a sub-solver.**
- **One global integrator, one ledger (roadmap "One Integrator, One Ledger").**
  Compartments are a *grouping* over the flat union of all stocks and flows. The integrator
  never sees a compartment; it steps the whole state at once. **Co-simulation — separate
  solvers per compartment exchanging values — is forbidden in the core** (it breaks
  conservation at the boundary and determinism through exchange order). State this as a hard
  invariant: it is easy to violate while "modularizing."
- **Ids stay byte-identical; only the `domain` label moves.** Re-assign each existing stock
  to its leaf compartment via the existing `Stock.domain` field (`biosphere.leaf_c`'s
  *domain* becomes `biosphere.plants`; its *id* stays `biosphere.leaf_c`). No float
  reduction keys on `domain` (verified), so amounts are **bit-identical**; goldens
  regenerate with **domain-label-only diffs**. The id prefix becomes historical naming, not
  a structural claim. *(Renaming ids — the rejected alternative — would reorder reductions
  and drift amounts.)*
- **Per-compartment boundary reporting is a *diagnostic*, not a second gate — and it is
  flux *reporting*, not a wiring check.** The roadmap wants `Inputs = Outputs + ΔStored` to
  hold *per subsystem*, yet also "conservation enforced globally, never per-domain."
  Reconciled: the **global** every-step gate stays the only *enforcement*; a per-compartment
  **boundary ledger** (for each compartment, sum the *crossing* flow legs in/out + its
  ΔStored) is computed from the same ledger and surfaced in reports / asserted in tests,
  never aborting a step. **Its honest value is two things:** (1) **per-boundary flux
  reporting** — "net carbon plants→atmosphere this step," the payoff that makes a
  4-compartment system debuggable; (2) **local apply-integrity** — it catches a
  *balanced-but-misapplied* delta (compensating misapplications across compartments that net
  to zero globally). **It does NOT catch a flow wired into the wrong compartment:**
  `crossing-in − crossing-out = ΔStored` is an *identity by construction* (a stock's
  after−before *is* the sum of legs touching it; classify both sides by `stock.domain` and
  they move together), so it holds for *any* wiring. **Wiring correctness is a separate
  *behavioral* assertion** — the plants↔atmosphere boundary exchanges only these quantities,
  in these directions — written per cross-compartment flow (Step 3+), not a conservation
  identity.
- **Core change budget: zero *modifications*, ZERO *additions* — RESOLVED to domain-side
  (Option B), advisor-reviewed.** The parent map and the rollup helper that consumes it
  **both live outside `simcore`** (under `domains/`, `sim_io/`, or a new reporting module),
  reading `registry.domain_index` off the public API — never the reverse. **Why B over the
  `simcore.registry(parents=...)` alternative (A): asymmetric reversibility.** B→A later
  (if Phase 5 wants the hierarchy structurally in core) is *additive* — add the defaulted
  `parents=` kwarg when the in-core consumer actually exists. A→B is a *breaking removal*
  from a frozen surface (the `observation.py` `kind`-cut / `StepReport.ledger`-refusal
  norm). With no in-core consumer in Phase 3 (per-compartment enforcement, integrator-sees-
  compartments, and compartment-keyed multirate are all ruled out/deferred), take the
  reversible branch. The roadmap's "registry generalizes to sibling domains" is about the
  *pattern* (registry + resolver), not the parent map being a registry field — it does not
  force A.
- **Coupling constraint (the joint rule, do NOT split the difference):** "parent map
  domain-side" is only coherent if its *consumer* is also outside `simcore`. If the rollup /
  boundary-ledger helper is built inside `simcore` (e.g. in `observation.py`), core code
  would read a domain-side parent map — an inversion worse than either clean option, and a
  de-facto core reporting surface (which would argue A). Decide both jointly: map AND helper
  outside `simcore`. **If during Step 1 you want the helper in `simcore`, stop and reconsider
  A — never split (map out, consumer in).**
- **Acceptance check for clean B:** after Step 1, `git diff src/simcore/` shows **zero new
  symbols** — no `parents`, no `descendant_stocks`, no hierarchy method on `Registry`. If
  that holds, "Phase 3 = zero core changes" stays literally true. The frozen
  `Integrator.step`, `Flow`, `Stock` shape, conservation gate, arbitration, and resolver are
  **untouched**.

### P3.2 — Reusable compartment **builders**: the composition pattern. *(Refactor is behavior-preserving and separate from new science.)*

- Split the monolithic `season.py` assembly into **compartment builder modules** — each
  (atmosphere, soil, plants, water, decomposition) a pure function returning its
  `(stocks, flows, aux, resolver-wiring)` — and **compose** the ecosystem from them. This
  *is* the "assemble from reusable compartments" deliverable, and the *same* composition
  pattern Phase 5 uses to compose sibling station domains.
- **Restructure ships goldens bit-identical (amounts).** The compartment refactor is a pure
  re-organization: same stocks, same flows, same ids, same wiring — only the assembly code's
  shape changes. The open-field and sealed goldens regenerate with at most domain-label
  diffs and **identical amounts**. New science (new flows / stocks / goldens) lands in
  *separate* steps. **Never mix a restructure with a behavior change in one step** — that
  forfeits the proof that the restructure was safe.

### P3.3 — Cross-compartment coupling is **shared-stock flows**; the **water cycle** is the proving cycle.

- "Domains meet only at shared stocks. The shared stock is the entire interface." A
  cross-compartment flow has legs whose stocks live in different compartments; no compartment
  imports or calls another. The water cycle is the cleanest first exerciser **and** the one
  cycle still open: Phase 1/2 transpiration drains `soil_water` to a `vapor_sink` BOUNDARY.
  Phase 3 closes it: **Plants** (transpiration → a real `water_vapor` stock in
  **Atmosphere**) → **Atmosphere** (condensation → `condensate` in **Water**) → **Water**
  (recycling → `soil_water` in **Soil**). Four compartments, a real closed cycle, emergent
  from stock coupling with no control code — the structural validation that the hierarchy
  works before consumers/perturbation pile on.
- Carbon and nitrogen cycles are already closed (Phase 2 Steps 3–6); Phase 3 only
  **re-homes** their stocks/flows into compartments (P3.2) and adds the water cycle.

### P3.4 — **Closure-preserving mortality**: death routes to **litter**, not the loss-sink; annual reset enables sustained oscillation.

- An organism's death (plant senescence-to-death; consumer death) routes its remaining organ
  carbon to **`litter_carbon`** (in-system, decomposable) — keeping a sealed chamber closed.
  The numerical-loss sink stays the **guard only** (extinction round-off, #6). This is the
  same mechanism that drains the Phase-2-deferred **`storage_c` residue** (a dead plant's
  grain currently leaves carbon locked forever).
- **Annual phenology reset / re-sowing** (the explicit Phase-2 → Phase-3 forward-pointer):
  Phase 2's probe found the plant dies after year 1 (DVS monotone, no vernalization, no
  re-sowing) → no *sustained* oscillation. A scenario-level annual reset (re-sow / reset DVS
  on a schedule) gives sustained multi-year dynamics — the prerequisite for "emergent
  behavior" and for the species-removal/addition perturbations. **Closure caveat:** in a
  *sealed* chamber re-sowing must **not** inject seed biomass from outside (that conjures
  carbon — the death-to-loss-sink trap in reverse); the new seedling's carbon must come from
  an **in-system pool** (retained `storage_c`/grain → seedling). The death-to-litter
  principle, run backwards.

### P3.5 — **Perturbation testing** via the events/scenario harness; cascades emerge for free.

- Build a perturbation harness on the existing `events.py` + forcing/scenario seams and ship
  **2–3 representative perturbations**: **drought** (cut irrigation / drop soil-water input),
  **lighting failure** (PAR → 0 over a window), **atmospheric leak** (a boundary leak flow
  draining the chamber gas pool). Each must show a **cascade with no cascade code** (e.g.
  lighting failure → photosynthesis weakens → O₂ production drops → CO₂ rises) and **keep
  conservation + `rationed == 0`** through the perturbation. Species removal/addition,
  nutrient shock, and pathogens are deferred to a representative subset / later (see Scope).

### Carried Phase-0/0.5/1/2 invariants that constrain Phase 3
- **Core purity (#11):** `simcore` stays stdlib-only; the parent map (if it lands in the
  registry) is a plain mapping. Compartment builders/flows/params live in `domains/biosphere/`.
- **One integrator, one ledger; no co-simulation (roadmap):** compartments never sub-solve.
- **Canonical order on every reduction (#15) / determinism — bit-identical (#7):** ids are
  **not** renamed, so reductions are byte-stable; restructure goldens keep identical amounts;
  new goldens use hex-float; registration-order-independent.
- **Every-step global conservation gate (#13):** unchanged. Per-compartment boundary
  accounting is an *additional diagnostic*, never a replacement.
- **Extinction conserves mass (#6):** POPULATION death routes to **litter** ecologically
  (P3.4); the loss-sink remains the numerical guard. POOL gas/water stocks never
  zeroed-with-loss.
- **Arbitration backstop Euler-only and rare (#3):** goldens assert `rationed == 0`;
  positivity from kinetics, including through perturbations.

---

## Scope

### In scope (Phase 3)
- **Foundation (P3.1):** the subsystem-hierarchy representation — parent map + re-domaining
  the existing stocks to `atmosphere`/`soil`/`plants`/`water` leaf compartments (no id
  renames); the per-compartment boundary-ledger diagnostic; goldens domain-label-only diff.
- **Reusable compartment builders (P3.2):** refactor `season.py` into composable
  per-compartment builders; goldens bit-identical (amounts).
- **Water cycle closure (P3.3):** `water_vapor` (Atmosphere) + condensation + recycling to
  `soil_water` (Soil); replace the `vapor_sink` BOUNDARY. The first real cross-compartment
  cycle.
- **Closure-preserving mortality + annual reset (P3.4):** death-to-litter, `storage_c`
  residue routed at death; annual phenology reset / re-sowing → sustained multi-year
  oscillation.
- **Modular sealed-ecosystem scenario:** assemble the full compartmentalized biosphere;
  per-compartment boundary diagnostics; a multi-year run showing emergent cross-compartment
  dynamics (short of decade-scale).
- **Perturbation harness + 2–3 perturbations (P3.5):** drought, lighting failure,
  atmospheric leak; cascade-for-free; conservation + `rationed == 0` through perturbation.

### Explicitly deferred (do NOT build in Phase 3)
- **Decade-scale conservation stability + freeze-as-reference + the golden biosphere
  scenario suite** → **Phase 4** (the proving-ground capstone). Phase 3 demonstrates
  emergence; Phase 4 hardens and freezes it.
- **Full trophic webs / multiple consumer levels** → a *single minimal consumer* is an
  optional stretch step (proves the trophic pattern: graze plant biomass → consumer biomass
  → respiration + death-to-litter); full webs deferred.
- **Optional P / S / methane cycles** → architecture-ready (single-currency flows, the
  Phase-2 nitrogen-loop pattern); not built.
- **Pathogens, species removal/addition as a full sizing axis, nutrient-shock calibration**
  → representative perturbations only; the rest deferred.
- **Multirate sub-stepping** → biosphere is uniform Euler-daily; multirate is a Phase 5/6
  concern (fast power/thermal vs slow soil). Do **not** pull it in.
- **Energy closure** → still ENERGY-as-diagnostic (decision #8); Phases 5/6.

---

## API additions (additive only — zero core *modifications*)

```python
# --- domain-side parent map + hierarchy view (Option B — RESOLVED; simcore UNTOUCHED) -
#   The parent map `{biosphere.plants -> biosphere, ...}` and the read-only hierarchy view
#   + `descendant_stocks(domain)` helper live OUTSIDE simcore (domains/ | sim_io/ | a new
#   reporting module), reading `registry.domain_index` off the public API. NOT a Registry
#   field. (Rejected Option A — `Registry(parents=...)` — would be a breaking removal if
#   Phase 5 doesn't need it in core; B→A is additive when an in-core consumer appears.)
#   Acceptance: `git diff src/simcore/` shows zero new symbols after Step 1.

# --- a per-compartment boundary ledger (diagnostic; reporting layer, NOT the gate) ----
#   For each compartment: sum cross-compartment flow legs (in/out) + ΔStored over its
#   stocks; assert per-quantity balance. Computed from the same StepReport/ledger the
#   global gate uses. Surfaced in reports / asserted in tests; never aborts a step.

# --- domains/biosphere : compartment builders + new science --------------------------
#   atmosphere.py / soil.py / plants.py / water.py / (decomposition stays) — each a pure
#   builder returning (stocks, flows, aux, resolver-wiring); season.py composes them.
#   water cycle: water_vapor stock + Condensation/Recycling flows (replace vapor_sink).
#   mortality: death-to-litter routing + annual reset/re-sow scenario seam.
#   perturbations: drought / lighting-failure / atmospheric-leak scenario events.
```

No change to `Integrator.step`, `Flow`, `Stock` shape, `Quantity`, `StockKind`, the
conservation gate, arbitration, or the resolver.

---

## Step sequence

**Foundation — designed in full (P3.1) and reviewed/built before the process steps, the
Phase-1/2 rhythm.**

1. **✅ COMPLETE — The subsystem-hierarchy representation (P3.1).** Re-domained the existing
   stocks into `atmosphere`/`soil`/`plants`/`water` leaf compartments (**no id renames** —
   only the `Stock.domain` label moved); the domain-side parent map + `descendant_stocks`
   hierarchy view (Option B — `simcore` untouched); the per-compartment boundary-ledger
   diagnostic (`compartment_boundary_ledger` → `CompartmentFlux`, in
   `domains/biosphere/compartments.py`). **Behavior-preserving, verified: goldens
   regenerated with domain-label-only diffs and byte-identical amounts** (19 changed lines,
   all `"domain":`); `git diff src/simcore/` **empty** (clean Option B acceptance check); the
   ledger's apply-integrity residual documented as exact only on clean steps (post-arbitration
   legs, no non-flow routing — extinction/scaling are Step-5 live-wiring exceptions). Pure
   infra — no new science. Landed in two commits (re-domain+hierarchy; boundary ledger).

**Process steps — enumerated now, each designed just-in-time (the Phase-1/2 rhythm).**

2. **✅ COMPLETE — Reusable compartment builders (P3.2).** Split the monolithic
   `build_season` into per-compartment builder modules — `scenario.py` (the
   `SeasonScenario`), `stocks.py` (the id/var catalog + `STOCK_DOMAIN` spec + the
   `CompartmentBuild`/`ChamberWiring` types + `chamber_wiring` + `organ_stock`/`pool_stock`),
   and `atmosphere.py` / `soil.py` / `plants.py` (each `build_<x>(scenario, wiring) ->
   CompartmentBuild`; `_carbon_context` moved into `plants`). `season.py` is now the thin
   composition (`_compartments` aggregator → both `build_season` and `weather_resolver`)
   plus re-exports of the full symbol surface (no test import path changed). **Both goldens
   byte-identical WITHOUT regeneration**; `git diff src/simcore/` empty; 836 tests pass
   (new `test_builders.py`: disjoint+complete partition and the P3.3 no-cross-import guard
   — **genuinely golden-blind** (the `build_season` union dedups by id; imports are pure
   source structure) — plus a per-builder leaf-stamp check that *localizes* a mis-stamp the
   snapshot goldens, which serialize `domain`, would also catch). `test_sealed_chamber`'s
   f_O2 patch seam
   followed the moved loaders to `plants`/`soil`. The composition pattern Phase 5 will reuse
   for sibling domains. **Designed in full below (§ "Step 2 design") — advisor-reviewed.**
3. **Close the water cycle (P3.3)** — `water_vapor` (Atmosphere) + condensation +
   recycling to `soil_water` (Soil); replace the `vapor_sink` BOUNDARY. The first real
   cross-compartment cycle; new golden. Verify the per-compartment boundary ledger balances
   (WATER-scoped) for every compartment the cycle touches. **Designed in full below
   (§ "Step 3 design") — advisor-reviewed.**
4. **✅ COMPLETE — Closure-preserving mortality + annual reset (P3.4)** — `annual_reset`
   (a pure driver transform: zero `thermal_time` + redistribute the old plant's organ +
   grain carbon to `litter_carbon`, retaining the seedling from the grain) applied each
   year boundary by `run_perennial`; the new `PERENNIAL_CHAMBER_SCENARIO` shows sustained
   multi-year oscillation (DVS = 2 every year, emergent period-2 cycle), genuinely closed
   (loss-sink stays 0.0). New behavior + golden; open + sealed goldens byte-identical.
   **Designed in full below (§ "Step 4 design") — advisor-reviewed; oscillation de-risked by
   a scratch probe before the design was committed.**
5. **✅ COMPLETE — Modular sealed-ecosystem assembly + per-compartment diagnostics
   (P3.1 ledger discharge)** — the assembly was already built (`PERENNIAL_CHAMBER_SCENARIO`
   *is* the full closed 4-leaf ecosystem), so this was a **diagnostics + verification** step
   (no new science, no behavior change, no new golden): the per-compartment boundary ledger
   now balances `net crossing == ΔStored` **every step / every quantity / every compartment**
   (boundary included) on the perennial run incl. the four reset-boundary steps; the
   extinction exception discharged by a hand-built deterministic test + the domain-side
   `expected_extinction_residuals` helper (the sealed run does not go extinct — probed — so
   the optional sealed full-ledger test was dropped); plus the emergent cross-compartment
   demonstration (CARBON cycles through every leaf both directions; chamber CO₂ draws down
   then recovers per year). Legs reconstructed test-side (Euler + `rationed == 0`); all three
   goldens byte-identical; `git diff src/simcore/` empty; new `test_compartment_ledger`.
   **Designed in full below (§ "Step 5 design") — advisor-reviewed; the every-step residuals
   (incl. boundary steps) and the no-extinction sealed finding de-risked by a scratch probe
   before any assertion was pinned.**
6. **Perturbation harness + representative perturbations (P3.5)** — drought, lighting
   failure, atmospheric leak; assert cascade-for-free + conservation + `rationed == 0`
   through each.
7. **(Optional / stretch) A minimal consumer** — one herbivore proving the trophic pattern
   (graze plant biomass → consumer biomass → respiration CO₂ + death-to-litter); full
   trophic webs deferred. Only if Steps 1–6 land with budget to spare.

---

## Step 1 design — the subsystem-hierarchy representation (P3.1)

*Realizes P3.1. Tightest constraint first: every later compartment builder and
cross-compartment flow depends on the hierarchy representation being fixed and
behavior-preserving. Pure infra — built and proven bit-identical before any new science,
exactly as Phase-1 Step 2 (the aux channel) and Phase-2 Step 1 (the composition fold) were.*

**The change, minimized.**
- **Leaf compartments via the existing `Stock.domain` field.** Define the leaf `DomainId`s
  — `biosphere.atmosphere`, `biosphere.soil`, `biosphere.plants`, `biosphere.water` (and
  keep `boundary`) — and re-assign each existing stock's `domain` to its compartment
  (`leaf_c`/`stem_c`/`root_c`/`storage_c`/`plant_n` → plants; `soil_water`/`soil_n`/`litter_*`
  → soil; `carbon_pool`/`o2_pool` → atmosphere; etc.). **Stock and flow ids are unchanged**
  — only the label moves. The mapping is a small, explicit table in the scenario.
  **Build-time guard:** grep for `.split(".")` / id-prefix logic first — the relabel creates
  a deliberate id-prefix↔domain mismatch (`biosphere.leaf_c` with domain `biosphere.plants`),
  harmless *unless* some code derives a domain by splitting the id; confirm nothing does
  before committing the relabel.
- **The parent map.** `{biosphere.atmosphere: biosphere, biosphere.soil: biosphere,
  biosphere.plants: biosphere, biosphere.water: biosphere}` — a flat-for-now two-level tree
  under a `biosphere` root. **Lands domain-side (Option B — RESOLVED), NOT as a Registry
  field**; with its `descendant_stocks(domain)` hierarchy view (union of a node's subtree's
  stock sets) in the same outside-`simcore` module, reading `registry.domain_index` off the
  public API. **Coupling rule:** the helper that consumes the map must also live outside
  `simcore` — if Step 1 pulls it into `simcore`, stop and reconsider Option A. Acceptance:
  `git diff src/simcore/` shows zero new symbols.
- **The per-compartment boundary ledger (diagnostic).** A reporting helper: given a
  `StepReport`'s flow legs + the before/after stocks, for each compartment classify legs as
  *internal* (both endpoints inside) vs *crossing* (one endpoint outside) and report the
  per-quantity **crossing flux** in/out (the debuggability payoff) alongside ΔStored. The
  identity **crossing-in − crossing-out = ΔStored** (internal flows cancel) holds *by
  construction*, so asserting it is a **local apply-integrity** check (it trips on a
  balanced-but-misapplied delta that nets to zero globally) — **not** a check that a flow was
  wired into the right compartment (both sides move with the mislabel). Uses the same
  composition fold the global gate uses (a CO₂ leg books CARBON + OXYGEN). **Diagnostic
  only** — surfaced in reports / asserted in tests, never aborts a step. *Wiring correctness
  — that the plants↔atmosphere boundary carries only the expected quantities/directions — is
  a separate **behavioral** assertion (Step 3+), not this conservation identity.*

**Why this is behavior-preserving (the central correctness point).** No float reduction keys
on `domain`: `conservation.py` sorts by stock id (`sorted(before.stocks)`) and quantity name;
the registry sorts flows/aux by id; `domain` appears only in set-valued groupings
(`domain_index`, `flow.py`'s `frozenset` of touched domains, `observation.py`'s diagnostic
grouping). Therefore moving a stock's `domain` label changes **no** amount in **any**
reduction. The open-field and sealed goldens regenerate with **domain-label-only diffs and
byte-identical amounts** — and *that identity is the proof* the restructure introduced no
behavior change. (If any amount drifts, an id was renamed or a reduction silently keyed on
`domain` — a bug to fix, not to absorb into the golden.)

**Test plan.**
- **Bit-identity (the headline):** re-domaining produces **identical amounts** to the
  pre-Step-1 run for both the open-field season and the sealed chamber — assert against the
  existing goldens with only the serialized `domain` labels changed (a targeted golden
  regeneration; amounts compared hex-exact).
- **Hierarchy view:** the parent map yields the expected tree; `descendant_stocks(biosphere)`
  == the union of all leaf compartments' stocks == every biosphere stock; flat default
  (no parents) reproduces today's `domain_index` behavior.
- **Per-compartment boundary ledger:** for a hand-built two-compartment transfer, the
  reported crossing flux matches the transfer and `crossing-in − crossing-out = ΔStored`
  holds per quantity (the apply-integrity identity); a single-compartment internal flow
  contributes zero crossing flux; a **balanced-but-misapplied** delta (a stock changed by an
  amount no leg accounts for, two errors netting to zero globally) **trips** the identity —
  its real catch. *(A flow attached to the **wrong compartment** is deliberately NOT tested
  here — both sides of the identity move with the mislabel. That is a separate
  behavioral/expected-flux assertion, written per cross-compartment flow at Step 3+.)*
- **Determinism / order-independence:** registration-order-independence holds unchanged
  (ids unchanged); the hierarchy view is canonical (sorted by `DomainId`).
- **Purity + frozen surface:** the `simcore` AST purity gate stays green; `Integrator.step`,
  `Flow`, the conservation gate, arbitration, and the resolver are untouched (the additive
  `parents` arg is defaulted; every existing call site compiles unchanged).

---

## Step 2 design — reusable compartment builders (P3.2)

*Realizes P3.2. A **behavior-preserving** refactor: split the monolithic `build_season`
assembly into per-compartment builder modules and recompose. No new stock / flow / aux /
id / amount — the existing open-field and sealed goldens must pass **byte-identical, with
no regeneration** (the proof the restructure is safe). New science (water cycle, mortality,
perturbations) lands in the *separate* later steps; never mixed with this restructure.*

**Why "no regeneration" (stronger than Step 1's "domain-label diffs").** Step 1 already
moved the `domain` labels into both goldens; Step 2 only reshapes the *assembly code* that
produces the same stocks/flows/state. The serializer emits stocks **sorted by id**
(`sim_io/snapshot.py`: `for sid in sorted(state.stocks)` — verified), so the union order
across builders is irrelevant to the bytes. Identical stocks + identical amounts + same
sort ⇒ the committed goldens are reproduced **exactly**. `test_regression_season` /
`test_regression_sealed_season` byte-compares are the gate — **if a golden needs
regenerating, stop and find the bug (a dropped flow, a mis-wired id); that is a drift to
fix, not a diff to absorb.**

### The shape — three builder modules + a thin composition layer

A **compartment builder** is a pure function
`build_<compartment>(scenario, wiring) -> CompartmentBuild` returning its own stocks, flows,
aux, and resolver shared-stock wiring. `season.build_season` becomes a thin **composition**:
call each builder, union the parts, add the cross-cutting loss-sink, hand the flat union to
`Registry` (which re-sorts flows by id — so builder/union order is behaviorally inert).

```python
@dataclass(frozen=True)
class CompartmentBuild:
    stocks: tuple[Stock, ...]
    flows: tuple[Flow, ...]
    aux: tuple[AuxProcess, ...]
    shared: Mapping[str, StockId]   # forcing-var -> live stock (the resolver #16 seam)
```

**Module layout (new files; clean bottom-up DAG, no import cycles).**
- `scenario.py` — `SeasonScenario` + `DEFAULT_SCENARIO`, extracted from `season.py` so a
  builder can take a `scenario` argument **without importing `season`** (`season` imports
  the builders; the reverse would cycle).
- `stocks.py` — the **stock-id catalog** (every `StockId` + forcing-var-name constant + the
  `STOCK_DOMAIN` declared partition) plus the two small composition types `ChamberWiring`
  and `CompartmentBuild` and the `chamber_wiring(sealed)` factory. Imports only `simcore` +
  `compartments` (the leaf `DomainId`s). This is the **shared interface** every builder
  reads; it is *not* "a compartment," so reading ids from it does not violate "no
  compartment imports another."
- `atmosphere.py` / `soil.py` / `plants.py` — the three builder modules. **`water.py` is
  deferred to Step 3** (it owns no stocks/flows yet — its first flow is Step-3 recycling;
  shipping an empty module now would be noise).
- `season.py` — slimmed to `_compartments(scenario)` (the aggregator), `build_season`,
  `weather_resolver`, `run_season`, and **re-exports** of every symbol the tests import from
  it today (the ids, `STOCK_DOMAIN`, `SeasonScenario`, `SEALED_CHAMBER_SCENARIO/YEARS`) — so
  **no test import path changes**.

**No builder imports another** (P3.3). Cross-compartment *forked* ids travel through
`ChamberWiring`; stable cross-compartment ids are read from the `stocks.py` catalog. A
builder never imports `atmosphere`/`soil`/`plants`. *(Alternative considered and rejected:
keep `SeasonScenario`/ids in `season.py` and have builders type-reference them under
`TYPE_CHECKING`. It avoids two new modules but creates a conceptual `season`↔builder cycle —
extraction gives the clean DAG a Phase-5-reuse foundation deserves.)*

### The `sealed` fork — one `ChamberWiring`, computed once

The open-vs-sealed difference reduces to a handful of stock **ids whose identity depends on
`sealed`**, threaded into flows that live in different compartments. Capture exactly those:

```python
@dataclass(frozen=True)
class ChamberWiring:
    carbon_source: StockId         # CARBON_POOL (sealed) | CO2_ATMOS (open)
    resp_sink: StockId             # CARBON_POOL (sealed, == source) | CO2_RESP (open)
    o2_pool: StockId | None        # O2_POOL (sealed) | None (open)
    litter_carbon_target: StockId  # LITTER_CARBON (sealed) | LITTER_SINK (open)
```

`chamber_wiring(scenario.sealed)` computes it once (a pure selection over catalog ids — no
two-phase resolve, confirmed in review). It is consumed **entirely by the plants builder**
(the carbon-budget flows + `Senescence`); the **atmosphere**/**soil** builders *build the
stock objects* those ids point at. That asymmetry — wiring read by plants, the stocks built
by atmosphere/soil — is the shared-stock interface (P3.3) made concrete. Each builder
additionally self-selects its sealed-only content off `scenario.sealed` (atmosphere: the gas
pools vs the open `co2_atmos`/`co2_resp` boundaries; soil: the decomposer + N-return
pools/flows). Stable cross-compartment targets that always have the same id when they exist
(e.g. `NitrogenSenescence → litter_n`) are read from the catalog, not added to the wiring,
to keep it lean.

### Ownership — the enumerate-and-assign checklist

Every stock, flow, and aux in today's `build_season`, assigned to **exactly one** owner
(**process-home**: the compartment whose biology drives it). The union must equal the
current set — that completeness *is* what the byte-identical golden verifies.

| Owner | Modeled stocks | Boundary stocks | Flows | Aux | `shared` |
|---|---|---|---|---|---|
| **plants** | `leaf_c` `stem_c` `root_c` `storage_c` `plant_n` | `vapor_sink`; `litter_sink` (open) | `Allocation` `GrowthRespiration` `MaintenanceRespiration` `Senescence` `NitrogenUptake` `Transpiration`; `NitrogenSenescence` (sealed) | `ThermalTimeAccumulation` | — |
| **soil** | `soil_water` `soil_n`; `litter_carbon` `litter_n` `microbial_carbon` (sealed) | `water_source` `n_source` | `Irrigation` `Fertilization`; `Decomposition` `MicrobialRespiration` `Mineralization` (sealed) | — | `soil_water → SOIL_WATER` |
| **atmosphere** | `carbon_pool` `o2_pool` (sealed) | `co2_atmos` `co2_resp` (open) | — | — | `co2_pool → CARBON_POOL` (sealed) |
| **composition (`build_season`)** | — | carbon **loss-sink** (cross-cutting, extinction #6) | — | — | — |

- **Process-home resolves the two judgment calls.** **Transpiration → plants** (a canopy
  flux; P3.3 itself frames transpiration as a plants→atmosphere flow — putting it in `water`
  now would *fight* Step 3's recycling, not pre-position it). **NitrogenSenescence → plants**
  (symmetry with carbon `Senescence`: both shed plant matter to soil litter through the
  wiring; `Mineralization` stays soil). Flow ownership is behaviorally inert (Registry
  re-sorts; the integrator sees a flat union) — the value is *organizational* clarity for
  Step 3+.
- **Boundary stocks** are built by the compartment of the flow that drives them
  (`vapor_sink`/`litter_sink` → plants; `water_source`/`n_source` → soil;
  `co2_atmos`/`co2_resp` → atmosphere). The **carbon loss-sink** spans compartments
  (extinction routing for any POPULATION carbon) → **`build_season` adds it at composition
  level**, not a builder.
- The plant carbon-budget context (`CarbonContext` / `_carbon_context`) is plant-internal →
  moves into `plants.py`.

### The resolver `shared`-map seam (decided, not deferred)

The `shared` map (#16) is structural (sealed-dependent), so it is a **builder output**, not
inline-forked in `weather_resolver`. Both `build_season` and `weather_resolver` route through
the single `_compartments(scenario)` aggregator: `build_season` unions stocks/flows/aux;
`weather_resolver` merges `b.shared` for the live-stock wiring (forcings still come from the
weather table). One source of truth, reproducing today's map exactly (open
`{soil_water → SOIL_WATER}`; sealed adds `{co2_pool → CARBON_POOL}`). #16 makes
shared-vs-forcing indistinguishable, so this is golden-safe.

### `STOCK_DOMAIN` — kept as the declared partition spec

Builders stamp `domain` as a **literal** (the plants builder stamps `PLANTS`, etc.) — domain
assignment becomes structural, so the `_stock_domain` lookup is **removed**. `STOCK_DOMAIN`
is **retained** (moved into `stocks.py`, re-exported from `season`) as the declared partition
**spec**: `test_compartments.test_relabel_partitions_…` already asserts
`built.domain == STOCK_DOMAIN[sid]`, which now binds the literal stamps to the spec (the
drift guard). No `test_compartments` rewrite — only an import path the re-export preserves.
*(Alternative — dissolve the table and assert the partition structurally — rejected: more
churn, loses the one-glance partition, no gain, and CLAUDE.md says re-express invariants, not
weaken them.)*

### Test plan

- **Goldens byte-identical, no regeneration** (the headline proof): `test_regression_season`
  + `test_regression_sealed_season` pass **unchanged**.
- **Full suite green** unchanged: `test_compartments` (partition spec + hierarchy view),
  `test_sealed_chamber`, `test_season`, `test_oracle_smoke` all import `season`'s re-exported
  surface as today.
- **New structural test** (`test_builders.py`): each builder returns only stocks carrying its
  own leaf domain (atmosphere→`ATMOSPHERE`, soil→`SOIL`, plants→`PLANTS`); the union of
  builder stocks/flows/aux equals `build_season`'s; and **no builder module imports another**
  (an import-graph / source guard encoding the P3.3 rule).
- **Purity gate** stays green; **`git diff src/simcore/` stays empty** (all new code under
  `domains/biosphere/`).

### Acceptance gate

Both goldens byte-identical **without regeneration** + full suite green + `git diff
src/simcore/` empty. **If a golden drifts, stop and find the bug — do not regenerate.**

---

## Step 3 design — close the water cycle (P3.3)

*Realizes P3.3 — the **first real cross-compartment cycle** and the one cycle still open.
A **behavior change** (sealed only) with a **new sealed golden**; the open-field golden
stays **byte-identical**. Carbon (Phase-2 Steps 2–5) and nitrogen (Step 6) are already
closed; water is the last leak. Advisor-reviewed before building, the Step-1/2 rhythm.*

**The leak, today.** The sealed chamber is not water-closed: `Transpiration` drains
`soil_water (soil) → vapor_sink` (a BOUNDARY, water *out*) and `Irrigation` refills it
`water_source → soil_water` (a BOUNDARY, water *in*). Two boundary crossings ⇒ the
chamber exchanges water with the outside. Step 3 closes both.

### The closed loop — four compartments, no control code

Sealed only (open field keeps both boundaries unchanged — see "the fork"):

```
soil_water (soil) --Transpiration--> water_vapor (atmosphere)
water_vapor (atmosphere) --Condensation--> condensate (water)
condensate (water) --Recycling--> soil_water (soil)
```

A genuinely closed WATER loop: total water `soil_water + water_vapor + condensate` is
conserved (each leg is a balanced 1:1 WATER transfer), distributed around the ring. It is
**emergent from stock coupling** — no compartment imports another; each flow names a
sibling's shared stock and the resolver (#16) cannot tell shared from forcing. This is the
**structural validation** that the hierarchy works before consumers/perturbation pile on.

- **Transpiration** (owned by **plants**, already): retargets its vapor leg from
  `vapor_sink` to `water_vapor` via a new `ChamberWiring.vapor_target` field (`WATER_VAPOR`
  sealed | `VAPOR_SINK` open) — exactly the `carbon_source` / `litter_carbon_target`
  pattern. **Its legs touch soil + atmosphere (sealed) — neither is the plants
  compartment** (flow ownership is organizational, not where the legs land; the integrator
  sees a flat union, the ledger classifies by `stock.domain`).
- **Condensation** (new flow, owned by **atmosphere**): `water_vapor → condensate`.
- **Recycling** (new flow, owned by the new **water** builder): `condensate → soil_water`.

### Drop irrigation + `water_source` in sealed (genuine closure)

For a genuinely closed cycle **both** boundary crossings must go, mirroring how carbon
dropped `co2_atmos`/`co2_resp` for the finite `carbon_pool`. **The nitrogen "inert
boundary" precedent does NOT transfer:** N kept `n_source`/`Fertilization` because
`fertilization_kg_m2_day == 0` (a zero-flux, harmless boundary), but
`irrigation_mm_day == 2.0` is **nonzero** — left in, it would pump water into the chamber
every step. So sealed **removes** `Irrigation` + `water_source` (soil builds them
**open-only**, symmetric with plants building `vapor_sink`/`litter_sink` open-only).
Consequence: `IRRIGATION_VAR` becomes a **provided-but-unread** forcing in the sealed
resolver — harmless (`SourceResolver`/`Environment.get` resolve lazily on read; they do
**not** assert every forcing is consumed — *confirm in Step 3*). No `shared`-map change.

### Kinetics — first-order donor control (engineered condenser framing)

Both new flows are first-order in their donor stock, the decomposition/mineralization
template — `condensation = k_cond · water_vapor`, `recycling = k_rec · condensate`
(kg day⁻¹). Structural positivity: each → 0 as its pool → 0, so `k·dt < 1` keeps the Euler
backstop unfired (`rationed == 0`) with no `max(0, …)` clamp.

**Citation framing (the honest part).** First-order means vapor condenses *regardless of
humidity* — **wrong for natural atmospheric condensation** (which needs supersaturation;
that is the deferred saturation/dew-point refinement, needing chamber volume + T-coupling
with zero architectural payoff here), but **right for an engineered condenser + water-
recovery loop** — a dehumidifier / condensing heat-exchanger at fixed clearance, which is
what a sealed bioregenerative life-support chamber actually has (CELSS; Biosphere 2
condensate management). Rates ship as `TODO(cite)` literature-typical placeholders pending
a later validation gate — consistent with the season's documented "machinery, not
validated behaviour" honesty and the decomposition/mineralization first-order precedent.

### New stocks, modules, params (zero core change)

- **Stocks** (both `{WATER: 1}`, single-currency ⇒ the conservation gate folds them
  identically to Phase-1, **no core change**): `water_vapor` → **ATMOSPHERE**,
  `condensate` → **WATER** (the `water` leaf compartment's first stocks — declared empty
  since P3.1). Add both ids to the `stocks.py` catalog and to `STOCK_DOMAIN`. Initial
  amounts `water_vapor0` / `condensate0` (scenario, default `0.0`, sealed-only — the
  `litter_carbon0` precedent).
- **`water_cycle.py`** (new process module): the `Condensation` + `Recycling` flow classes
  + a `WaterCycleParams` dataclass (two first-order rates), mirroring how
  `mineralization.py` holds two coupled flows for one loop. The flows read only
  `snapshot.stocks` — **no `env.get`, no new forcing**.
- **`water.py`** (new compartment builder, deferred here from Step 2): builds `condensate`
  + the `Recycling` flow (sealed only; empty `CompartmentBuild` when open). Names
  `soil_water` from the **catalog** (stable id; no builder imports another, P3.3).
- **`atmosphere.py`**: sealed adds the `water_vapor` stock + the `Condensation` flow
  (names `condensate` from the catalog).
- **`water_cycle.yaml`** + `load_water_cycle_params()`: one file, two rates
  (`condensation_rate` / `recycling_rate`, both `1/day`), loaded by **both** atmosphere
  and water — the `mineralization.yaml` precedent (same file, two builders, separate
  objects). Value/unit/source schema, exact-string unit guard, non-negative bound.
- **`ChamberWiring`**: add `vapor_target: StockId`; `chamber_wiring()` selects
  `WATER_VAPOR` (sealed) | `VAPOR_SINK` (open). Keep `Transpiration`'s `vapor_sink`
  dataclass **field name** so the open flow object is byte-identical post-refactor.

### Sequencing — isolate "refactor safe" from "new science" (the Step-1/2 discipline)

1. **✅ DONE — Refactor only:** added `vapor_target: StockId` to `ChamberWiring`
   (`chamber_wiring()` selects `VAPOR_SINK` **unconditionally** — sealed flips to
   `WATER_VAPOR` only in substep 2, once that stock exists); `Transpiration` now reads
   `vapor_sink=wiring.vapor_target` instead of the hardcoded `VAPOR_SINK` (the dataclass
   **field name** `vapor_sink` is kept, so the flow object is identical). **Both goldens
   byte-identical WITHOUT regeneration** (`wiring.vapor_target` resolves to the same
   `StockId("boundary.vapor_sink")` in both chambers; `ChamberWiring` is never
   serialized); `git diff src/simcore/` empty; full suite green (836 passed); ruff +
   pyright clean. Pure indirection — no new stock/flow/science.
2. **✅ DONE — New science:** added the `water_vapor`/`condensate` stocks,
   `Condensation`/`Recycling` flows (new `water_cycle.py` + `water.py` builder + the
   atmosphere `Condensation`), `chamber_wiring()` flipped `vapor_target` to `WATER_VAPOR`
   when sealed, and dropped sealed `Irrigation` + `water_source`. **Probed the sealed run
   before regenerating the golden:** `events == ()` (no extinction — the ledger test stays
   WATER-clean), min `soil_water ≈ 980.9` (margin ≈ 920 above `sw_critical = 60`, so
   `f_water ≡ 1` — the carbon/O₂/N trajectory is unperturbed), and `rationed == 0`. With
   `k_cond = k_rec = 0.5` (`k·dt = 0.5 < 1`) all three hold, so the Euler backstop never
   fires. *Then* regenerated the sealed golden (diff confirmed **only** WATER moved:
   `condensate`/`water_vapor` added, `vapor_sink`/`water_source` removed, `soil_water`
   amount changed; carbon/O₂/N + thermal_time **byte-identical**). Open golden untouched.

### Test plan

- **Open golden byte-identical** (`test_regression_season`) — the refactor proof.
- **Sealed golden regenerated** (`test_regression_sealed_season`) — the new closed-water
  behaviour, hex-float exact.
- **Per-compartment boundary ledger — WATER-scoped** (the advisor's load-bearing catch):
  assert the `compartment_boundary_ledger` residual ≈ 0 **for `Quantity.WATER`** on every
  step of the sealed run, for soil / atmosphere / water (the compartments the cycle
  touches). **Scoped to WATER deliberately:** the residual identity holds only on a *clean*
  step (`rationed == 0` **and** no extinction routing), and the sealed producer *may* go
  extinct — but extinction routes **CARBON** to the loss-sink and touches **no WATER
  stock**, so WATER stays clean even if the plant dies. The full-ledger-every-step
  assertion (handling the extinction exception) is **Step 5's** job — do not pull it
  forward.
- **Behavioral wiring assertion** (the check the ledger identity *cannot* do — both sides
  move with a mislabel): the three cycle flows carry **only WATER**, in the ring directions
  `soil → atmosphere → water → soil`. Written per cross-compartment flow (P3.1's
  "wiring correctness is a separate behavioral assertion").
- **Closed-loop conservation:** `soil_water + water_vapor + condensate` constant (to tol)
  across the sealed run — the closure proof beyond the every-step global gate.
- **`rationed == 0`** through the sealed run (structural positivity).
- **`test_builders`** extended: `water.py` returns only `WATER`-domain stocks, imports no
  sibling builder; the union still equals `build_season`'s.
- **`test_compartments`**: `STOCK_DOMAIN` partition + `descendant_stocks(biosphere)` now
  include `water_vapor`/`condensate`; the `water` leaf is no longer empty.
- **Purity gate** green; **`git diff src/simcore/` empty** (all new code under
  `domains/biosphere/`).

### Acceptance gate

Open golden byte-identical (no regeneration) + sealed golden regenerated & pinned + the
WATER-scoped per-compartment ledger balances every step + behavioral wiring + closed-loop
conservation + `rationed == 0` + full suite green + `git diff src/simcore/` empty.

---

## Step 4 design — closure-preserving mortality + annual reset (P3.4)

*Realizes P3.4 — the **last closure gap** (an annual plant locks its grain forever) and
the prerequisite for **emergent behaviour** (a plant that dies after year 1 has no
sustained dynamics). A **new behaviour** on a **new scenario** with a **new golden**; the
open-field and sealed goldens stay **byte-identical** (no regeneration — the Phase-2
three-act capstone is preserved). The oscillation was **de-risked by a scratch probe
before this design was committed** (advisor-directed); the probe result is the load-bearing
evidence below. Advisor-reviewed, the Step-1/2/3 rhythm.*

**The two gaps, today (probed, not assumed).** A scratch driver applying the reset below
to the canonical sealed run shows the baseline plant (`docs` Phase-2 finding, re-confirmed):
DVS climbs to **2.0 (maturity) by the end of year 1 and stays pinned there forever**
(`thermal_time` keeps accumulating but DVS caps at 2) — no regrowth, no oscillation. And
**~1.885 mol C locks permanently in `storage_c`** (grain), filled in year 1, never recycled.
That locked grain is **in-system and conserved — a dead-end, NOT a boundary leak** (it is a
`POPULATION` carbon stock, not a BOUNDARY; the conservation gate still balances). It only
*bites* when you want sustained cycling — which is exactly the new perennial scenario — so
the fix lives there, and the Phase-2 sealed scenario keeps its locked grain (and its golden)
untouched.

### The load-bearing decision (P3.4-mech) — the annual reset lives in the **driver**, not a flow, not the aux process

The annual phenology reset / re-sow is a **scheduled scenario intervention** (a sowing /
harvest *calendar* event), exactly as P3.4 frames it ("a scenario-level annual reset on a
schedule"). It is **not** emergent dynamics and **not** a rate law. Three candidate homes,
two rejected:

- **REJECTED — a reset-aware `ThermalTimeAccumulation`** that reads `snapshot.n` and
  self-zeroes on the year boundary. Smuggles a *calendar schedule* into a *physics
  accumulator* and pollutes the one clean aux process — exactly the "resist a second
  accumulator for symmetry" discipline P2 set, inverted.
- **REJECTED — a discrete flow firing on `snapshot.n`.** A flow returns `rate·dt` legs
  (dt-linear, carried unchanged across RK4 stages); a once-a-year discrete transfer breaks
  dt-linearity and the RK4 stage semantics, and "fires on calendar day N" is control-flow in
  a rate law.
- **CHOSEN — a pure `annual_reset(state, scenario) -> State` applied by the run driver at
  year boundaries.** Both halves of the reset (zero `thermal_time`; redistribute carbon) in
  one visible scenario-layer place. `thermal_time` is **aux — invisible to the conservation
  gate** (`conservation` reasons only over `stocks`), so zeroing it is free and touches no
  conserved mass. The carbon redistribution is **conserving by construction** (the litter
  leg is the balancing residual — the senescence/maintenance idiom), and the driver
  **re-asserts `conservation.assert_conserved(before, after)` across the reset** so
  "conserved at every point" stays literally true even though the reset is not a flow-step.

**The cost taken on vs the flow path** (which gets arbitration's non-negativity net for
free): `annual_reset` must keep amounts ≥ 0 itself. Trivially safe — the seedling (0.16 mol
C) is dwarfed by the grain reserve (~1.9 mol), and the guard `grain ≥ seedling_total` is
asserted. All of this is **domain-side**: `git diff src/simcore/` stays **empty** (the
acceptance check).

### `annual_reset` — the conserving redistribution (carbon-only)

At each year boundary the **old plant dies/harvests entirely to litter, except the seedling
carbon retained from the grain (the seed bank)**, and `thermal_time` resets so the new
seedling develops from DVS 0:

```
new leaf_c / stem_c / root_c := scenario seedling amounts (leaf_c0, stem_c0, root_c0)
storage_c (grain)            := 0
litter_carbon                += old_veg + grain − seedling_total     # the balancing residual
thermal_time                 := 0
```

where `old_veg = old(leaf_c+stem_c+root_c)`, `seedling_total = leaf_c0+stem_c0+root_c0`. The
litter leg is computed as the residual so carbon balances exactly (verified: drift
≈ 5.8e-15 over a 5-year run). **Grain → 0 every year is what prevents seed-bank
accumulation** (the damped-cascade trap: if the seed bank only shed `seedling_total` it
would grow unboundedly, draining the active cycle). The dumped grain → `litter_carbon` →
`microbial_carbon` → CO₂ (the Step-4/5 decomposition chain) refuels the next year's
photosynthesis — **this is the carbon-recycling that makes the oscillation sustained, not
emergent control code.**

- **`t=0` sowing vs mid-run re-sow.** The initial state's organ amounts are a **legitimate
  initial seed** (real biomass placed at sowing). Only **mid-run** re-sows must draw from an
  in-system pool (the grain) — the closure caveat P3.4 names. `annual_reset` never injects
  carbon from outside; the seedling comes from `storage_c`.
- **Carbon-only; the N *windfall* (a documented deferral, framed honestly).** The reset
  moves only CARBON. The plant's `plant_n` persists across the "death" — so the new seedling
  **inherits the full standing N pool against ~1/6 the biomass**, and N *concentration jumps*
  at the reset (it is an N **windfall**, not a neutral no-op). This is harmless **only
  because N stays non-limiting** (`f_N ≡ 1`; `plant_n` probed in [0.15, 0.5], a high
  concentration against the small reset biomass — the jump pushes `f_N` further into its
  saturated `== 1` region, so it has zero effect on the carbon trajectory). The
  **continuous** `NitrogenSenescence`/`Mineralization` loop keeps N cycling regardless. A
  full N-reset (dump `plant_n` → `litter_n`, re-seed seedling N) is a deferred refinement —
  it would matter only once `f_N < 1` (a Phase-3 N-limited regime, itself deferred).
- **Keep the continuous `Senescence` flow as-is.** It handles **in-season** leaf/stem/root
  turnover (the slow relative-death shed to litter); the **discrete annual** redistribution
  handles the **death/harvest/re-sow boundary**. Two distinct mechanisms, not one extended.
- **The loss-sink stays the numerical guard only (the closure headline) — but this is
  *scenario-dependent*, not mechanism-guaranteed.** It holds because the reset catches the
  organs (sets them to the seedling) before they decay to the extinction threshold, so **no
  extinction fires** (`events == ()`) and the **carbon loss-sink stays exactly 0.0** (probed).
  But that is a property of the *sizing*: a leaner scenario (smaller carbon pool, slower
  decomposition, a thin year) could let an organ decay **below threshold mid-year**, before
  the reset — and the **core** extinction pass would then route a residual to the BOUNDARY
  loss-sink, **silently breaking "genuinely closed" while every per-step conservation test
  still passes** (the loss-sink balances the ledger). **Consequence (the pre-golden gate, see
  Sequencing):** on the *exact committed* `PERENNIAL_CHAMBER_SCENARIO`, assert
  `events == () AND carbon_loss_sink == 0.0` **first**, before regenerating the golden — that
  is the line between "closed" and "closed for these knobs." Death routes to **litter**
  (in-system), never to the BOUNDARY loss-sink (decision #6: the loss-sink is the round-off
  guard, not the death pathway).

### Probe result — sustained oscillation is REAL (the de-risk)

A scratch driver (`annual_reset` at each `n % len(weather) == 0`, `n > 0`) over **5 years**
on the sealed chamber shows, with **`rationed == 0` and `events == ()` throughout**:

- **DVS reaches maturity (2.0) every year** (0→2→0→2…; one step after each reset DVS ≈ 0.015)
  — regrowth, not a one-shot.
- A **stable emergent period-2 limit cycle** (overcompensation, classic in discrete
  annual-plant models): biomass peaks alternate ≈ 0.73 / ≈ 1.00 mol C, grain ≈ 1.9 / 2.1,
  CO₂ pool ≈ 0.72 / 0.32 — neither damped to zero nor exploding. **Genuine sustained
  multi-year dynamics** — the P3.4 deliverable, and a clean "emergent behaviour for free"
  demonstration (no cascade code).
- **All four quantities conserved** across the whole run incl. the resets (CARBON drift
  ≈ 5.8e-15; OXYGEN/WATER/NITROGEN ≤ 1e-12).

The phenomenon does **not** depend on the O₂-poor capstone sizing — a clean ample-O₂ sealed
chamber sustains identically (the O₂ crash is a year-1 transient orthogonal to the carbon
oscillation).

### Scope — a new scenario; both existing goldens untouched

- **`PERENNIAL_CHAMBER_SCENARIO`** (new, in `scenario.py`): a sealed chamber sized to cycle
  enough carbon for a clean oscillation (ample O₂ so the perennial carbon story is not
  muddied by the O₂-depletion drama; a modest seeded litter pile to fuel year-1 growth — the
  exact knobs pinned at implementation off the probe). Its **own** new golden.
- **Open-field + Phase-2 sealed goldens stay byte-identical (no regeneration).** Substep 1 is
  pure indirection; the perennial behaviour lives only on the new scenario + the new
  driver-with-reset path. The Phase-2 sealed capstone keeps its locked grain (in-system
  dead-end, documented) and its three-act narrative.

### Sequencing — refactor-safe → new-science (the Step-1/2/3 discipline)

1. **✅ DONE — Refactor-only — the driver reset hook.** `run_season` gained an optional
   **schedule-agnostic** `reset: Callable[[int, State], State] | None = None`, consulted
   before each step: it returns `state` unchanged (same object) on a non-reset step or a new
   `State` on a reset boundary; the `n % year` calendar lives **inside the caller's closure**
   (scheduling is a scenario/caller concern). When a reset is applied the driver re-asserts
   `conservation.assert_conserved(before, after)`. **Default `None` ⇒ the loop is identical**
   — open + sealed goldens pass **byte-identical WITHOUT regeneration** (proven before
   substep 2); `git diff src/simcore/` empty; full suite green (856). Pure indirection.
2. **✅ DONE — New science — `annual_reset` + perennial scenario + golden + tests.**
   `annual_reset(state, scenario)` (the carbon redistribution above) + `run_perennial`
   (`run_season` with the reset scheduled every `year` steps); `PERENNIAL_CHAMBER_SCENARIO`
   (+ `PERENNIAL_CHAMBER_YEARS = 5`, the ample-O₂ sibling of the sealed scenario). **Ran the
   pre-golden gate on the committed scenario via the production path first:** `events == ()`,
   `carbon_loss_sink == 0.0` (max over the run), `rationed == 0`, DVS = 2.0 every year,
   biomass peak > 0.5 every year, all four quantities conserved (CARBON drift ≈ 3e-15) —
   *then* pinned the new golden (the gate is baked into its `_final_state` generator). Open +
   sealed goldens **byte-identical** (no regeneration). New `test_perennial_chamber`
   (sustained-oscillation, genuine-closure, conservation-incl.-resets, determinism +
   `annual_reset` unit tests) and `test_regression_perennial_season` (the third golden). Full
   suite green (870); `git diff src/simcore/` empty; no new stock/flow/compartment.

### Test plan

- **Open + sealed goldens byte-identical** (no regeneration) — the refactor proof
  (substep 1).
- **Perennial golden** regenerated & pinned (hex-float exact), Euler-daily — the new
  closed-loop perennial behaviour (substep 2).
- **Sustained oscillation** (the headline): **every year reaches DVS = 2.0** (regrowth, not
  a one-shot) **and every year's biomass peak exceeds a floor** (e.g. `> 0.5` mol C) — that
  is exactly "sustained, not damped." **Do NOT assert period-matching** (`year[N] ≈
  year[N+2]`): the cycle is a period-2 attractor still in its transient at year 5 (odd years
  ≈ 0.717→0.727→0.730 rising, even ≈ 1.021→1.007 falling), so an equality/convergence
  assertion would be flaky. (Cosmetic: the driver stores **pre-reset** states, so no test
  should expect a DVS ≈ 0 entry at the exact year-boundary index — the reset instant is not
  in the stored trajectory.)
- **`annual_reset` conserves carbon** (a unit test on the pure function): total CARBON
  before == after; only CARBON moves (O₂/N/water deltas zero); post-reset amounts ≥ 0;
  `thermal_time == 0`; the seed-bank guard (`grain ≥ seedling_total`) holds.
- **Genuine closure** (the P3.4 point): over the perennial run the **carbon loss-sink stays
  exactly 0.0** and `events == ()` — death routes to litter, never to the BOUNDARY loss-sink.
- **Conservation every step incl. across resets**: all four quantities invariant over the
  whole multi-year run (the driver re-assert + the per-step gate together).
- **`rationed == 0`** through the perennial run (structural positivity survives the discrete
  reset — verified by the probe).
- **`test_compartments` / `test_builders`** unchanged (no new stock/flow/compartment — the
  reset reuses existing stocks; `annual_reset` is a driver helper, not a builder).
- **Purity gate** green; **`git diff src/simcore/` empty** (all new code under
  `domains/biosphere/`).

### Acceptance gate

Open + sealed goldens byte-identical (no regeneration) + perennial golden regenerated &
pinned + sustained-oscillation behavioural test + `annual_reset` conservation/closure unit
test + loss-sink stays 0.0 + every-step four-quantity conservation incl. resets +
`rationed == 0` + full suite green + `git diff src/simcore/` empty. **If an existing golden
drifts, stop and find the bug — do not regenerate.**

---

## Step 5 design — modular sealed-ecosystem assembly + per-compartment diagnostics (P3.1 ledger discharge)

*Realizes the **deferred half of P3.1** — the per-compartment boundary ledger asserted
**every step, every quantity, every compartment** (Step 1 built the diagnostic and proved
it on a hand transfer; Step 3 ran it live but **WATER-scoped**, explicitly deferring the
full-ledger + extinction-exception handling "to Step 5") — plus the **exit demonstration**:
a multi-year run exhibiting **emergent cross-compartment dynamics**. **No new science, no
behavior change, no new golden** — this is a **diagnostics + verification** step. The
oscillation/closure are already pinned (Step 4); Step 5 proves the *architecture* claim
(four compartments, one ledger, balance localized per compartment). Advisor-reviewed, the
Step-1/2/3/4 rhythm.*

**The assembly is already built — name it, do not rebuild it.** `build_season` already
composes the four compartment builders into one flat union (the integrator stays global —
P3.1), and `PERENNIAL_CHAMBER_SCENARIO` already activates **all four leaves
non-trivially**: ATMOSPHERE (`carbon_pool`, `o2_pool`, `water_vapor`), SOIL (`soil_water`,
`soil_n`, `litter_carbon`, `litter_n`, `microbial_carbon`), PLANTS (the `leaf_c`/`stem_c`/
`root_c`/`storage_c` organs, `plant_n`), WATER (`condensate`). The carbon cycle (Phase-2),
nitrogen cycle
(Phase-2), water cycle (Step 3), and mortality/reset (Step 4) all run in it, genuinely
closed (`events == ()`, loss-sink 0.0). **So "compose the full compartmentalized biosphere"
is the *perennial scenario as-is*** — Step 5 adds **no new scenario, no alias, and no new
golden** (a new bit-exact surface for zero modeling gain — rejected). The deliverable is to
*verify and report on* that already-assembled ecosystem per compartment.

### The value over the global gate (state it, or a reviewer asks "why not just the gate?")

The every-step global conservation gate (#13) already proves *total* mass per quantity is
invariant. The per-compartment ledger is **strictly stronger, not a duplicate**: it is
**localized apply-integrity** — for each `(compartment, quantity)` the identity
`net crossing flux == ΔStored` (internal flows cancel) **trips on a balanced-but-misapplied
delta that nets to zero globally** (two compensating misapplications across compartments
the global gate cannot see). It is **not** a wiring check (a flow mislabeled into the wrong
compartment moves both sides of the identity together — that is the separate *behavioral*
assertion, Step 3's `test_three_cycle_flows_carry_only_water_in_ring_order`). Asserting it
**every step, every compartment, every quantity** is the architectural proof that the
4-compartment split is apply-correct, and the **per-boundary flux** it reports
("net carbon plants→atmosphere this step") is the debuggability payoff of the hierarchy.

### The load-bearing decision (P3.5-ledger) — leg reconstruction is **test-side**, **no driver change**

`StepReport` exposes `(state, events, rationed)` but **not** the per-step flow legs, which
the ledger needs for the crossing flux. Three ways to get them; one chosen:

- **REJECTED — expose legs from the integrator (`StepReport.flow_results`).** A `simcore`
  *modification* of a frozen surface — blows the "zero core changes" budget
  (`git diff src/simcore/` must stay empty). Off the table.
- **REJECTED — an `observe` hook on `run_season`** (the Step-4 `reset`-hook shape: a
  defaulted `observe: Callable | None` consulted each step, byte-identical when `None`). It
  *works* and keeps the loop single-source, but it is **speculative generality** — a new
  driver surface + a substep-1 byte-identical proof for a *consumer that does not exist*
  (the only consumer is the Step-5 test). Against the plan's minimal-surface ethos. Revisit
  only if a need arises that test-side reconstruction genuinely cannot serve.
- **CHOSEN — reconstruct the legs test-side, the Step-3 precedent extended.** For Euler +
  `rationed == 0`, a single `flow.evaluate(state, env.bind(state, dt), dt)` at the
  start-of-step state **equals the applied legs** (no arbitration scaling ⇒
  `_reduce(scaled) == _reduce(raw)`; the #16 seam binds the resolver to that same state).
  Step 3 already does exactly this WATER-scoped; Step 5 generalizes it. **Zero production
  code touched** beyond a small domain-side helper (below); `git diff src/simcore/` stays
  trivially empty.

**Handling the annual reset without a driver change (the perennial wrinkle).** The reset is
a **pure, deterministic, schedule-known** transform, so the reconstruction re-derives the
post-reset pre-step state itself — **no need to capture it live.** For transition
`i → i+1` the before-step state is

```
before_step = annual_reset(states[i], scenario)   if i > 0 and i % year == 0
            = states[i]                            otherwise
```

**mirroring `run_perennial`'s predicate verbatim** (so it cannot drift). Computing the
ledger over `(before_step → states[i+1])` puts the reset **outside** the transition — the
ledger then sees **only flow legs (+ extinction)**, never the reset's non-flow carbon
redistribution. (The reset's own conservation is already discharged: the driver re-asserts
`assert_conserved` across it, and `annual_reset` is unit-tested.) On the perennial run
`events == ()`, so after this split **every** step is clean → all four residuals ≈ 0 with a
**zero** non-flow correction. That is the headline: *the genuinely-closed ecosystem balances
per compartment, every step, every quantity, with no correction term.*

**Euler-only precondition (state it).** Leg reconstruction (single evaluation == applied
legs) is sound **only** under Euler + `rationed == 0`; RK4 has no single "applied legs" (the
⅙-combine of four stage derivatives). The biosphere is Euler-daily and every golden pins
`rationed == 0`, so the precondition holds — but it is a precondition, asserted in the test
(`rationed == 0`) before the ledger loop.

### The extinction exception — a small named helper + a **hand-built** primary discharge

Extinction (#6) is a **balanced non-flow change** the legs cannot see: a sub-threshold
POPULATION organ snaps to 0 (its compartment loses `r`) and the residual routes to the
**`boundary`-domain** loss-sink (`boundary.loss.<q>`, gains `r`). So on an extinction step
the ledger residual is `+r` for the organ's compartment and `−r` for `boundary` — **not a
bug, an expected exception** (the Step-1 `CompartmentFlux` docstring already flags this as
"Step 5's job").

- **The helper (keep the ledger flow-only).** A new domain-side
  `expected_extinction_residuals(before, events) -> dict[(DomainId, Quantity), float]`
  (in `compartments.py`, beside `compartment_boundary_ledger`): for each `ExtinctionEvent`
  it books `+residual` to `(before.stocks[event.stock].domain, event.quantity)` and
  `−residual` to `(BOUNDARY_DOMAIN, event.quantity)`. The full check asserts
  `abs(entry.residual − expected.get((entry.domain, entry.quantity), 0.0)) <= tol`. **Do
  NOT fold this into `compartment_boundary_ledger`** — its "residual holds by construction
  on a clean step" property is exactly what makes a *nonzero* residual diagnostic; the
  non-flow correction is a separate, named, testable concern. The helper is reusable by
  Step 6 (perturbations may drive extinctions).
- **Per-step events by bucketing.** `run_season` returns events as one flat tuple over the
  run; each `ExtinctionEvent` carries `.n`, so per-step events are `[e for e in events if
  e.n == before_step.n]` — no per-step return needed.
- **PRIMARY discharge — a hand-built deterministic extinction step (the Step-1 idiom), not
  "hope the sealed run dies."** Whether `SEALED_CHAMBER_SCENARIO` actually fires extinction
  is incidental and possibly vacuous; hinging the deliverable on it is fragile. Instead a
  unit test constructs a tiny two-compartment state with one POPULATION stock below
  threshold + one clean crossing flow, steps it once (Euler), and asserts the raw ledger
  residual is `+r` / `−r` on the organ-compartment / `boundary`, **and** that
  `expected_extinction_residuals` zeroes both. This *deterministically* discharges the
  forward-pointer.
- **OPTIONAL realism — the sealed full run, probe-gated.** *If* a probe confirms
  `SEALED_CHAMBER_SCENARIO` fires ≥ 1 extinction, add a full-ledger-every-step test on it
  asserting `residual == expected_extinction_residuals` within per-quantity tol, **with a
  non-vacuity assert** (≥ 1 step has a nonzero correction). If it does not die, drop this
  test — nothing is lost (the hand-built test is the real discharge). No silent reliance.

### The full ledger check — **all** domains incl. `boundary`, **per-quantity** tol

- **No whitelist.** Step 3 whitelisted `{SOIL, ATMOSPHERE, WATER}` (WATER-scoped). The full
  check must iterate **every** `(domain, quantity)` the ledger returns — **including
  `boundary`** (where extinction's `−r` lands) and PLANTS. An *unexpected* nonzero anywhere
  is precisely what this is meant to catch; whitelisting would silently skip it.
- **Per-quantity tolerance, not flat 1e-7.** Reuse the table `test_perennial` already pins:
  CARBON `1e-12`, OXYGEN `1e-11`, WATER `1e-7`, NITROGEN `1e-9`. A flat `1e-7` is far too
  loose for the O(1) CARBON amounts and would hide real misapplication.
- **Non-vacuity.** Assert the loop **saw real crossing flux** (some `entry.crossing_in` or
  `crossing_out > 0` for CARBON, OXYGEN, WATER) — otherwise a frozen run would pass the
  residual check trivially (the Step-3 `saw_crossing` precedent, per quantity).

### The emergent cross-compartment demonstration — robust gate, loose narrative

Heed `test_perennial`'s own lesson (*do NOT assert period-matching* — the period-2 cycle is
still in transient at year 5). Two tiers:

- **Gate (robust) — genuine cycling, no pure source/sink.** Over the run, each of
  PLANTS / SOIL / ATMOSPHERE has **both** summed `crossing_in > 0` **and** summed
  `crossing_out > 0` for CARBON. That is the "emergent cross-compartment dynamics" claim
  turned into a check: carbon genuinely *cycles* through every leaf (photosynthesis draws
  atmosphere→plants; respiration + the reset push plants→atmosphere/soil; decomposition
  pushes soil→atmosphere), not a one-way drain. The reporting payoff (net per-boundary flux)
  *is* this assertion.
- **Narrative (loose) — direction only, never magnitude/timing.** The
  reset → `litter_carbon` → decomposition → `microbial_carbon` → CO₂ → regrowth cascade is
  the "cascade for free" story: assert only **direction** — `carbon_pool` (ATMOSPHERE) draws
  **down** within a growing year, then **recovers** before the next reset (decomposition
  refuels it) — never a magnitude or a day index. This shows cross-compartment coupling with
  no cascade code, the P3 exit theme, without a flaky equality.

### Sequencing — probe → verify (no refactor/science split; there is no behavior change)

Unlike Steps 2–4 there is **no behavior change and no driver change**, so there is no
"refactor-safe vs new-science" split — but the **Step-4 probe discipline still applies**:

1. **Probe FIRST (gates the assertions, not the design).** Run the reconstruction over the
   committed `PERENNIAL_CHAMBER_SCENARIO` trajectory and confirm **all four** residuals sit
   within their per-quantity tol on **every** step — *including the four reset-boundary
   steps* (the post-reset-state handling is the part most likely to have a bug). If a
   boundary step shows a residual, the `annual_reset(states[i])` reconstruction has a bug to
   **find**, not a tol to loosen. Also probe whether `SEALED_CHAMBER_SCENARIO` fires
   extinction (gates the optional sealed test).
2. **Then build:** the `expected_extinction_residuals` helper + the hand-built extinction
   unit test + the perennial full-ledger-every-step test + the cross-compartment cycling
   gate + the cascade-direction narrative test (+ the optional probe-gated sealed test). All
   three existing goldens stay **byte-identical** (no behavior touched anywhere).

### Test plan

- **Perennial full per-compartment ledger, every step** (the headline): for every
  `(domain, quantity)` incl. `boundary`, `abs(residual) <= tol[quantity]` on every step,
  with the verbatim-mirrored reset reconstruction and a zero extinction correction
  (`events == ()` pinned); per-quantity tol table; CARBON/OXYGEN/WATER non-vacuity.
- **Extinction-exception discharge (hand-built, deterministic):** a one-step
  below-threshold POPULATION + clean crossing flow yields ledger residual `+r` /`−r` on the
  organ-compartment / `boundary`, and `expected_extinction_residuals` zeroes both.
- **`expected_extinction_residuals` unit test:** an `ExtinctionEvent` on a known stock maps
  to `{(organ_domain, q): +r, (boundary, q): −r}`; empty events ⇒ empty dict.
- **Cross-compartment cycling gate:** PLANTS/SOIL/ATMOSPHERE each have summed
  `crossing_in > 0` and `crossing_out > 0` for CARBON over the run.
- **Cross-compartment cascade (direction only):** `carbon_pool` draws down then recovers
  within a year (no magnitude/timing).
- **(Optional, probe-gated) sealed full-ledger every step:** `residual ==
  expected_extinction_residuals` within tol, with a non-vacuity assert (≥ 1 nonzero
  correction). Dropped if the sealed run does not go extinct.
- **All three goldens byte-identical** (open / sealed / perennial) — **no regeneration, no
  new golden** (Step 5 changes no behavior).
- **`test_compartments` / `test_builders`** unchanged (no new stock/flow/compartment).
- **Purity gate** green; **`git diff src/simcore/` empty** (the helper lives in
  `domains/biosphere/compartments.py`).

### Acceptance gate

Perennial full per-compartment ledger balances every step / every quantity / every domain
(incl. `boundary`) within the per-quantity tol + extinction exception discharged by the
hand-built unit test + `expected_extinction_residuals` unit-tested + cross-compartment
CARBON cycling demonstrated (both directions, all three active leaves) + cascade-direction
narrative + all three existing goldens byte-identical (**no regeneration, no new golden**) +
full suite green + `git diff src/simcore/` empty. **The probe (perennial residuals within
tol on every step incl. the boundary steps) must pass before any assertion is pinned — if a
boundary step shows a residual, find the reconstruction bug, do not loosen the tol.**
