# Phase 3 — Modular Biosphere / Consumers

**Status: Steps 1–2 COMPLETE; Steps 3–7 not started.** Phases 0, 0.5, 1, and 2
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
**empty**; 836 tests pass incl. the new `test_builders` (per-builder leaf-domain stamp,
disjoint-and-complete partition vs `build_season`, and a no-cross-import guard — the
structural checks the amount-blind goldens cannot see). **Next: Step 3 — close the water
cycle (P3.3).**

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
   (new `test_builders.py`: per-builder leaf-domain stamp, disjoint+complete partition, the
   P3.3 no-cross-import guard — load-bearing because `domain` is amount-invariant so the
   goldens are blind to a mis-stamped leaf). `test_sealed_chamber`'s f_O2 patch seam
   followed the moved loaders to `plants`/`soil`. The composition pattern Phase 5 will reuse
   for sibling domains. **Designed in full below (§ "Step 2 design") — advisor-reviewed.**
3. **Close the water cycle (P3.3)** — `water_vapor` (Atmosphere) + condensation +
   recycling to `soil_water` (Soil); replace the `vapor_sink` BOUNDARY. The first real
   cross-compartment cycle; new golden. Verify the per-compartment boundary ledger balances
   for every compartment the cycle touches.
4. **Closure-preserving mortality + annual reset (P3.4)** — death routes organ +
   `storage_c` carbon to `litter_carbon`; annual phenology reset / re-sowing → sustained
   multi-year oscillation. New behavior + golden.
5. **Modular sealed-ecosystem assembly + per-compartment diagnostics** — compose the full
   compartmentalized biosphere; assert `Inputs = Outputs + ΔStored` per compartment every
   step; a multi-year run exhibiting emergent cross-compartment dynamics (the exit
   demonstration, short of decade-scale).
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
</content>
</invoke>
