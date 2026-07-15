# Phase 1 — Single Producer

**Status: COMPLETE — Steps 1–11 done; Phase 1 exits.** (The one exit item, the
*quantitative* oracle match, is explicitly DEFERRED by user decision — see the
Step-11 block.) **Steps 1 (units + area basis), 2 (the non-conserved aux
channel), 3 (the PCSE oracle harness + clean-room param discipline), 4
(Beer–Lambert light interception — the first biological process), 5 (FvCB
photosynthesis — the first carbon-source flow), 6 (maintenance + growth
respiration — the carbon-sink flows), 7 (Penman–Monteith transpiration + root
uptake — the first WATER-currency flows), and 8 (thermal-time phenology — the aux
accumulator's rate function, the first consumer of the Step-2 channel), and 9
(leaf/stem/root biomass allocation + senescence — the first internal-redistribution
carbon process and multi-organ stock structure), and 10 (nitrogen uptake + limitation —
the NITROGEN-currency mirror of Step 7, supplying the last `f_N` limiter) are
implemented, tested, and committed** — see their per-step `RESOLVED` blocks below. **All
seven biological processes (Steps 4–10) are now complete; the foundation (Steps 1–3)
was complete earlier. Step 11 (integration) is complete: the season is assembled
(`domains/biosphere/season.py`), conserves every step with `rationed == 0` by
construction, is liveness-checked + golden-pinned, and qualitatively matches the oracle's
shape — the `plant_c` buffer was dissolved (the load-bearing deviation, see the Step-11
RESOLVED block) and CO₂ split into `co2_atmos`/`co2_resp`. The one remaining exit item,
the *quantitative* oracle match, is explicitly DEFERRED (a user decision); the
single-producer machinery is done.** This plan **locks the three
foundation decisions** (the non-conserved aux channel, single-currency-flow coupling,
and units/area + the Euler-daily/gate split) and **enumerates** the seven biological
process steps as forward-pointers. Per the working-style rule and the advisor's
"tightest-constraint-first" sequencing, the foundation (Steps 1–3) is designed in full
and is reviewed/built **before** any of the seven process sections are written in
detail — seven FvCB/PM/allocation designs on top of an architecture that might shift
in review is wasted writing.

**Goal (roadmap exit, lines 220–246):** *"Build a research-grade crop physiology
model."* A single producer (one crop, well-mixed 0-D) driven by external forcing
(light, temperature, humidity, CO₂, irrigation, nutrient availability) — all read
**through the source resolver**, never hardwired as "external" — whose biomass
trajectories, gas exchange (carbon), water use, and nitrogen dynamics **reproduce
reference behavior** (growth-chamber literature + the WOFOST/PCSE oracle) for at
least one crop. **No closed-loop feedback yet** (that is Phase 2); forcing is
imposed from outside.

**Source of truth for the phase sequence:** `roadmap_extracted.txt` (Phase 1 =
lines 220–246). Phases 0 and 0.5 are complete and their APIs are frozen
(`docs/plans/phase-0-engine-skeleton.md`, `docs/plans/phase-0.5-numerical-foundations.md`).
Phase 1 is **additive** to those surfaces — it must not break them.

**Reuse/licensing:** `docs/reuse-and-licenses.md`. Clean-room from primary
literature; PCSE/WOFOST are **offline oracles only**, never imported or ported.

---

## Relationship to the roadmap + the central reframing

The roadmap's Phase-1 "Required Processes" (lines 233–239): Beer–Lambert light
interception; Farquhar–von Caemmerer–Berry photosynthesis; maintenance + growth
respiration; Penman–Monteith transpiration + root uptake; thermal-time phenology;
leaf/stem/root biomass allocation; nitrogen uptake + limitation. Each "exists as
flow, unit test, parameter file, and documentation **before** integration"
(line 232).

The non-obvious architectural reframing — settled in review — is **how biology maps
onto the frozen stock-and-flow core**, which was proven only on trivial first-order
transfers. Three tensions drove the locked decisions below:

1. **Currencies are molecular-species pools, not elements.** Photosynthesis
   CO₂ + H₂O → CH₂O + O₂ conserves *atoms*, but our OXYGEN currency (O₂) would be
   *created* with no O₂ withdrawal (the O atoms are booked under CARBON/WATER). A
   single-producer system is **open**, so O₂ is non-limiting and has no feedback —
   that is the Phase-2 closed chamber. → **P1: single-currency flows, multiplicative
   coupling.**
2. **Phenology has no balanced counterparty.** Thermal-time / development-stage is
   *state that evolves* but is **not** a conserved quantity, so it cannot ride the
   flow → reduce → apply path or pass the conservation gate. → **P2: a parallel
   non-conserved aux integration channel.**
3. **Research-grade crop physiology is Euler-daily** (so is the oracle), and its
   daily-integrated canopy flux uses `dt` non-linearly → it is **not dt-refinable**.
   → **P3: Euler-daily biology + split numerical gates + science units & area.**

---

## Locked decisions (Phase 1)

New decisions, numbered `P1…`, carrying the Phase-0/0.5 invariants they depend on.
They constrain the implementation and must not silently drift.

### P1 — Phase-1 flows are **single-currency**; coupling is multiplicative rate-limitation, not multi-quantity stoichiometry.
Every Phase-1 flow transfers **exactly one** conserved quantity (CARBON, WATER, or
NITROGEN). Cross-process coupling is expressed as **dimensionless limitation
factors**:

  `actual_rate = potential_rate · Π_i f_i`,  each `f_i ∈ [0, 1]`

where each `f_i` reads a stock or env var (light, temperature, soil-water fraction,
plant-N status). This is exactly how WOFOST couples; **every flow balances trivially
in its one currency**, and it dodges the O₂-creation problem entirely.

- **OXYGEN is NOT tracked in Phase 1** (no O₂ stock, no O₂ flow). Confirmed: all
  seven roadmap processes are single-currency (assimilation/respiration/allocation =
  carbon; transpiration/uptake = water; nutrients = nitrogen).
- The genuine **multi-quantity stoichiometric flow** (decision #2 — one atomic flow
  moving several quantities at fixed ratios) is **deferred to Phase 2** (closed-
  chamber gas exchange, where O₂/CO₂ coupling first matters and first has feedback).
  It is *filed, not built* here. The Phase-0 leg/`FlowResult` data shape already
  supports it (multiple legs across quantities); Phase 1 simply does not exercise it.
- A limitation factor reading a *sibling* value goes through `env.get` (the source
  resolver) so the same flow code runs standalone (forcing) and coupled (shared
  stock) unchanged — decision #16. Reading the *plant's own* state (e.g. leaf N
  concentration) is a direct snapshot read inside `evaluate` (decision #16's same
  immutable snapshot).

### P2 — Phenology/structure is a **non-conserved auxiliary integration channel**, parallel to stocks. *(The load-bearing decision — reviewed before any process step.)*
Thermal-time / DVS is integrator-advanced state with **no balanced counterparty**,
so it is **not** a `Flow`. Phase 1 adds a parallel channel: **non-conserved scalar
accumulators**, each with its own rate function, advanced by the integrator and
**exempt from the conservation gate**.

- **Minimize the surface.** Phase 1 needs essentially **one** accumulator —
  *thermal time* (°C·day). Everything else is **derived, not stored**:
  `DVS = f(thermal_time)` (development stage), `LAI = f(leaf_carbon)` (leaf area via
  specific leaf area). Derived quantities are computed where needed (a diagnostic
  helper / inside `evaluate`), never integrated. So the channel is "non-conserved
  scalar accumulators," not a general aux soup.
- **Euler-accumulated, read piecewise-constant within a step** — exactly how forcing
  already behaves (`BoundEnvironment` reads `snapshot.n`; aux reads `snapshot.aux`).
  Aux is advanced by **one explicit-Euler rate evaluation at the step-entry
  snapshot**, independent of the stock integrator's scheme, and is **never
  sub-staged through RK4**. (Under RK4, aux is *kept* across stages like `n` — only
  stock amounts perturb — so flows that read aux see a within-step constant. This is
  why Euler-daily and the aux channel reinforce each other: P3 means the stock
  scheme is Euler too, so the whole step is uniform.)
- **The frozen-State ripple is explicit** (additive; new field with empty default,
  so existing call sites and goldens are unaffected):
  - `State` gains `aux: Mapping[str, float]` (default empty `MappingProxyType`,
    wrapped like `stocks`; keys are stable, canonical-sortable names).
  - `simcore.conservation.compute_ledger`'s "before/after share the same **stock**
    key set" assertion is unchanged — aux lives **outside** `stocks`, so it is
    invisible to the ledger by construction (no conserved-quantity surface to drift).
  - `sim_io.snapshot` serializes `aux` (hex-float values, sorted by key — the same
    exactness/canonical-order discipline as `stocks`); a schema-version bump is
    required (goldens regenerate — see Step 2).
  - **`Observation` is NOT extended in Phase 1** — *no named consumer*. The repo's
    norm is "no field without a named consumer" (`observation.py` cut `kind`/`totals`
    for exactly this); Phase-1 validation reads `State.aux` **directly** in tests and
    round-trips it via `sim_io`, neither of which needs `observe(aux)`. The aux
    projection is **deferred** until a real consumer (a UI/telemetry growth-stage
    readout) appears — additive and cheap then, speculative now.
- **Alternative weighed + rejected:** a conservation-*exempt pseudo-quantity* that
  reuses `Stock`/integrator machinery "for free" was rejected — a one-legged
  "thermal-time flow" violates "every flow is a balanced transfer" (the Phase-0
  invariant), and exempting a `Quantity` from the gate adds a classification surface
  to the very code whose lack of a sign/classification surface keeps it bug-resistant
  (`conservation.py` docstring). A clean parallel channel is conceptually honest.

### P3 — Phase-1 biology runs **Euler at a fixed daily step**; the numerical gates **split**.
- **Why Euler-daily:** research-grade crop physiology integrates **daily** with an
  intra-day (and intra-canopy) Gaussian integration of instantaneous assimilation —
  and **PCSE/WOFOST are themselves Euler-daily**, so *matching the oracle requires
  the same numerics*. (A sub-daily diurnal-forcing run under RK4 would also collide
  with the Phase-0.5 deferral of sub-stage time-varying forcing — N-stage evaluation
  at `(n+cᵢ)·dt` is still out of scope.)
- **Consequence — the convergence gate cannot run on the biology.** A daily-
  integrated canopy flow uses `dt` **non-linearly** (the day's Gaussian integration
  is inside `evaluate`), so it **forfeits RK4 order** (the Phase-0 increment-form
  contract) and is **not dt-refinable**. Therefore Phase 0.5's "results converge as
  `dt`→0" gate **does not apply to Phase-1 biology**. The gates split cleanly:
  - **Engine numerics** (Euler→1, RK4→4 convergence; multi-rate→2; stability) stay
    proven on the **analytic** Phase-0.5 scenarios (decay, LV) — untouched.
  - **Phase-1 biology** is validated **against the oracle** (behavioral trajectory
    match, P5) and by **conservation + golden regression**, *not* by dt-convergence.
  - **RK4 stays available** for any dt-linear flow and for the engine gates; the
    crop scenario **selects Euler** and documents why.
- **Time-varying forcing becomes real.** Phase 0.5 scenarios were autonomous;
  Phase 1's diurnal/seasonal forcing schedules genuinely depend on `n`. Euler +
  `BoundEnvironment` already supports this (`schedule(snapshot.n, dt)`); it was
  simply never exercised with a non-constant schedule. Phase 1 adds time-varying
  `Schedule`s (e.g. a daily weather table) and tests them.
- **Positivity / the backstop stays the rare guard.** An instantaneously-fine rate
  can **overshoot over a full day** (Euler), so self-limiting kinetics must cap the
  **daily flux** (smooth shutoff as a resource depletes — e.g. transpiration ∝
  relative soil water, assimilation → 0 as CO₂/light → 0). Nominal crop scenarios
  keep `rationed == 0` (the golden asserts it); the Euler min-scaling backstop
  remains the rare numerical guard, never the ecological mechanism (decision #3).

### P4 — Units become **science-correct**, plus an **area basis**.
Phase 0 left `CANONICAL_UNIT` **PROVISIONAL** ("the science pick is a Phase-1
decision" — `quantities.py`). Phase 1 resolves it:

- **Golden-locked vs free.** The committed regression goldens
  (`tests/regression/golden/demo_*`) contain only **CARBON** and **ENERGY** stocks,
  so `CARBON = mol` and `ENERGY = J` are **golden-locked** — changing either forces
  regenerating those goldens (avoid unless science demands it; `mol C` and `J` are
  adequate). **WATER and NITROGEN labels are free** to set to science-correct units
  now (they appear in no committed golden). Pick at Step 1 (candidates: WATER in
  `kg` or `mol`; NITROGEN in `kg` or `mol`) — the only hard constraint is **one
  consistent canonical unit per quantity** (conservation needs that, #9). Per-organ
  *biomass* is conventionally kg dry-matter; our currency is **carbon**, so a kg-DM
  ⇄ mol-C conversion (carbon fraction of dry matter) is a **boundary conversion**
  in the loader, not a core unit.
- **Area basis (the genuinely new thing).** Crop physiology is **per unit ground
  area** (FvCB in µmol CO₂ m⁻² s⁻¹; PM in mm = kg H₂O m⁻² day⁻¹). A flow's `Leg`
  is an **absolute** amount in the canonical unit, so each physiological flow
  converts per-area → absolute using a scenario **`ground_area`** parameter (m²)
  inside `evaluate`. `ground_area` is declared data (a scenario param), not core
  state. This is the convention that makes per-m² science integrate to absolute
  stock changes.
- **Rate-law dimensional closure** — `config/units.py` flags it Phase-1: full
  per-leg dimensional signatures on `Flow` are **still deferred** (they would touch
  the frozen `Flow` protocol). Phase 1's discipline is the **param-file boundary**:
  every parameter carries a declared unit, validated/converted by the loader
  (extending Scope-A unit validation to the new param sets), and each flow documents
  its rate-law dimensions in its docstring/param-file header. A typed per-leg
  dimensional check is a noted future enhancement.

### P5 — Clean-room from primary literature; PCSE oracle **behind a marker**; the oracle match is **behavioral**.
- **Reimplement from papers, cite the paper.** FvCB (Farquhar, von Caemmerer &
  Berry 1980); Penman–Monteith (Monteith 1965; Penman 1948); Beer–Lambert canopy
  interception (Monsi & Saeki 1953); thermal-time phenology (degree-day literature);
  allocation (DVS-keyed partitioning tables from cited crop-model literature). Param
  *values* come from **cited publications**, never copied from the unlicensed
  `WOFOST_crop_parameters` YAML (`docs/reuse-and-licenses.md`). Each param-file
  header and flow docstring cites its source.
- **PCSE is a dev/test dependency only.** It drags numpy + a large tree, so it lives
  behind a test marker (e.g. `oracle`) — **never** imported by `simcore`, never near
  the purity gate (`tests/test_simcore_purity.py` stays green by construction).
  Enable the commented-out `oracle = ["pcse"]` group in `pyproject.toml` at Step 3.
- **"Match the oracle" is BEHAVIORAL, not bit-exact.** Because params are
  independently literature-derived (we cannot copy WOFOST's), and our reimplemented
  equations differ in detail, the validation gate compares **trajectory shape and
  magnitude within tolerance** (biomass curve, gas-exchange and water-use
  magnitudes, N dynamics), not bit-for-bit. The exit criterion must be read that way.
- **License precondition.** `docs/reuse-and-licenses.md` says a `LICENSE` is chosen
  "before Phase 1, when real crop code lands." Adding one is a **user decision**
  (which license) — surfaced as an open item, not decided here (Step 1 precondition).

### Carried Phase-0 / 0.5 invariants that constrain Phase 1
- **Core purity (#11):** `simcore` (incl. any new biology *mechanism* placed in
  core) stays stdlib-only. Crop **flows/params/loaders** live in
  `domains/biosphere/` (flows are stdlib-pure; the loader is the config boundary) —
  the established split. The AST purity gate must keep passing.
- **Canonical order on every reduction (#15):** every new reduction (aux-increment
  sums, any multi-process aux touch) sorts by stable id/name.
- **Determinism — bit-identical within a build (#7):** crop runs are bit-identical
  and registration-order-independent; new golden(s) use hex-float.
- **Every-step conservation gate (#13):** unchanged and always-on; aux is outside it
  by construction (P2). Each crop flow is internally balanced in its one currency
  (P1), incl. the boundary reservoirs for water/N inputs and harvest/litter outputs.
- **Frozen Phase-0/0.5 API:** additions only. `State.aux` (new field, empty default)
  and the `sim_io`/`Observation`/aux-process additions are additive; the frozen
  `Integrator.step`, `Flow`, `Quantity`, `Stock` (other than the carried `aux`
  ripple), and the resolver are otherwise untouched.

---

## Scope

### In scope (Phase 1)
- **Foundation:** science units + area basis (P4); the non-conserved aux channel
  (P2); the PCSE oracle harness + clean-room param discipline (P5).
- **Seven biological processes**, each as **flow (or aux rate) + unit test + param
  file + documentation** (roadmap line 232): Beer–Lambert light interception; FvCB
  photosynthesis; maintenance + growth respiration; Penman–Monteith transpiration +
  root uptake; thermal-time phenology; leaf/stem/root allocation; nitrogen uptake +
  limitation.
- **Boundary reservoirs** for the new currencies' inputs/outputs: soil-water source
  (irrigation), N source (fertilizer/soil supply), CO₂/atmosphere as forcing-or-
  boundary, harvest/litter sinks — every crop flow internally balanced (#13).
- **Time-varying forcing** schedules (diurnal/seasonal weather; CO₂; irrigation;
  nutrient availability) read through the source resolver.
- **Integration scenario:** one crop, full season, assembled from the seven
  processes; **behavioral validation against the oracle** (P5) + a **golden
  regression snapshot** + season-long conservation with `rationed == 0`.

### Explicitly deferred (do NOT build in Phase 1)
- **Multi-quantity stoichiometric flows / O₂ tracking / gas-exchange coupling** →
  Phase 2 (closed chamber). Filed in P1.
- **Closed-loop feedback** (photosynthesis lowering CO₂, etc.) → Phase 2; Phase-1
  forcing is imposed from outside.
- **Decomposition, microbial biomass, litter dynamics** → Phase 2.
- **Sub-daily diurnal stepping under RK4 / sub-stage time-varying forcing** → still
  deferred (Phase-0.5 note); Phase-1 biology is Euler-daily.
- **Typed per-leg dimensional signatures on `Flow`** → noted future enhancement
  (P4); Phase 1 validates units at the param boundary.
- **Multiple crops / cultivar libraries / scenario authoring** → later phases; Phase
  1 needs **one** crop (roadmap line 244).

---

## API additions (additive; frozen Phase-0/0.5 surface otherwise untouched)

```python
# --- simcore/state.py : State gains a non-conserved aux channel (P2) ----------
@dataclass(frozen=True)
class State:
    n: int
    stocks: Mapping[StockId, Stock]
    rng_seed: int
    aux: Mapping[str, float] = <empty>   # NEW: non-conserved scalar accumulators
                                         # (thermal time, ...). Wrapped immutable;
                                         # outside the conservation gate. Empty
                                         # default ⇒ existing call sites unchanged.

# --- simcore aux process: parallel to Flow, but UN-balanced + single-valued ---
class AuxProcess(Protocol):              # (module: simcore/auxiliary.py — PURE core)
    id: AuxId
    def evaluate(self, snapshot: State, env: Environment, dt: float
                 ) -> Mapping[str, float]: ...
        # returns per-aux-name INCREMENTS dt·rate(snapshot) (increment form, like
        # Flow). NOT balanced (no conserved counterparty). Advanced by explicit
        # Euler at the step-entry snapshot, never sub-staged (P2/P3).

# integrator advances aux alongside stocks (Euler one-eval; canonical name order):
#   new_aux[name] = old_aux.get(name, 0) + Σ_processes increment[name]
# carried unchanged across RK4 stages (like n). Aux processes are a CONSTRUCTION
# dependency of the integrator (like Registry) — exact wiring settled at Step 2
# (lean: Registry gains an optional aux-process collection, default empty, so the
# integrator constructor stays Integrator(registry)).

# --- sim_io/snapshot.py : serialize aux (hex-float, key-sorted); schema bump ---
# --- aux advances in step_report ONLY; substep leaves aux untouched (see Step 2) -
# (Observation is NOT extended — no named Phase-1 consumer; deferred. P2.)
```

No change to `Flow`, `Quantity` (no new member — OXYGEN stays unused in Phase 1),
`StockKind`, the resolver, or the frozen `Integrator.step` signature.

---

## Step sequence

**Foundation — designed in full below; reviewed/built before the process steps.**

1. **Units + area basis (P4)** + LICENSE precondition.
2. **The non-conserved aux channel (P2)** — the load-bearing architecture step.
3. **PCSE oracle harness + clean-room param discipline (P5).**

**Biological processes — enumerated now, each *designed just-in-time* (the Phase-0/0.5
rhythm: a "Step N design" section settled with the advisor immediately before
implementing it). Each delivers flow/aux-rate + unit test + param file + doc.**

4. **Light — Beer–Lambert interception** (Monsi & Saeki 1953). *Auxiliary/derived,
   not a mass flow:* computes intercepted-PAR fraction from LAI (`= f(leaf_carbon)`),
   feeding photosynthesis. Establishes the canopy diagnostic.
5. **Photosynthesis — FvCB** (Farquhar et al. 1980). The carbon **source** flow
   (CO₂ → plant carbon), gated by `f_temp·f_water·f_N` (limiters default to
   1.0 until their step lands — each process standalone first, roadmap line 232).
   *Light coupling caveat (JIT, from Step 4):* the Step-4 intercepted fraction
   `f_int ∈ [0,1)` is **not** an independent `f_light` multiplier in the `Π fᵢ`
   product. Light enters FvCB through the **electron-transport / light-response
   curve**: absorbed PAR = incident PAR · `f_int` *drives* the assimilation curve.
   Wiring `f_int` *both* into the light response *and* as a standalone limiter would
   double-count light limitation — so the canopy diagnostic feeds **absorbed PAR**,
   not a separate `f_light` factor.
   *Carbon-vs-DM caveat (JIT, Steps 5/9):* WOFOST-style conversion efficiency (kg DM
   per kg CH₂O) **implicitly destroys mass** — in our framework that carbon must go
   somewhere explicit (gross assimilation → **growth respiration** to
   atmosphere/boundary → structural carbon, each a *balanced* carbon flow). A silent
   efficiency factor that drops carbon trips the every-step gate (the gate doing its
   job), so it shapes the stock/flow structure.
6. **Respiration — maintenance + growth** (carbon: plant → atmosphere/boundary).
7. **Water — Penman–Monteith transpiration + root uptake** (Monteith 1965). WATER-
   currency flows (soil water → plant → atmosphere); supplies `f_water`.
8. **Phenology — thermal-time progression** (degree-day literature). The aux
   accumulator's rate function (Step 2 channel); DVS drives allocation + senescence.
9. **Biomass allocation — leaf/stem/root** (DVS-keyed partition tables). CARBON
   flows among plant-organ stocks; needs the organ stocks + the senescence path.
   *Stock-structure note (JIT):* earlier carbon steps may use a single provisional
   `plant_c` POPULATION pool; Step 9 either introduces the leaf/stem/root organ pools
   from the first carbon step or makes the provisional-pool→split an explicit
   transition (a mid-phase stock-structure change to call out, not slip in).
10. **Nutrients — nitrogen uptake + limitation.** NITROGEN-currency flows (soil N →
    plant N); supplies `f_N`.
11. **Integration + validation.** Assemble the full single-producer season; oracle
    behavioral match (P5) for the chosen crop; golden regression snapshot;
    season-long conservation + `rationed == 0`; update the frozen-surface notes.

> **Crop choice (named, deferred — P5).** Validation needs **one** well-documented
> C3 crop with strong WOFOST/PCSE coverage and growth-chamber literature.
> Recommendation: **winter wheat** (the WOFOST reference crop) or **potato**.
> Confirm at Step 11 (the validation step); it threads into param files only there.

---

## Foundation — provisional stock/flow inventory (P1 single-currency validation)

*Concrete check that every Phase-1 flow closes in **one** currency before the seven
process sections are written (the advisor's "tightest constraint first" made
explicit). Stock kinds/ids are provisional and settle JIT; the point is currency
closure. Light interception and phenology are **aux/derived**, not flows.*

| Process (step) | Currency | Source → Sink | Closes? |
|---|---|---|---|
| Light interception (4) | — | *aux/derived:* intercepted-PAR fraction from `LAI=f(leaf_c)` | n/a (no leg) |
| Photosynthesis / FvCB (5) | CARBON | CO₂ boundary/forcing reservoir → plant structural C | ✓ 1-currency |
| Growth respiration (5/6) | CARBON | plant C → atmosphere/boundary CO₂ (the explicit conversion loss, item 4) | ✓ |
| Maintenance respiration (6) | CARBON | plant C → atmosphere/boundary CO₂ | ✓ |
| Root water uptake (7) | WATER | soil-water POOL → plant water | ✓ — *collapsed into transpiration; see Step-7 RESOLVED (3→2, no `plant_water`)* |
| Transpiration (7) | WATER | soil-water POOL → boundary vapor sink | ✓ |
| Irrigation (7) | WATER | boundary source → soil-water POOL | ✓ |
| Phenology (8) | — | *aux:* thermal-time accumulator; `DVS=f(thermal_time)` | n/a (no leg) |
| Allocation (9) | CARBON | plant C → leaf/stem/root C (internal redistribution) | ✓ |
| Senescence (9) | CARBON | leaf/stem/root C → litter boundary sink (Phase 1; litter dynamics = Phase 2) | ✓ |
| N uptake (10) | NITROGEN | soil-N source/POOL → plant N | ✓ |

Every leg-bearing row is single-currency ⇒ **P1 holds against the concrete process
list**. Coupling across currencies is *only* via the dimensionless limiters
`f_light·f_temp·f_water·f_N` (P1), never a shared leg. Boundary reservoirs needed:
CO₂ source/forcing, soil-water source (irrigation) + vapor sink, soil-N source,
litter sink, plus the carbon loss-sink already in the demo (extinction routing).

---

## Step 1 design — science units + area basis (P4)

*Realizes P4. Tightest constraint first: every later param file and flow depends on
the canonical units and the per-area↔absolute convention.*

### RESOLVED (2026-06-17) — the locks

- **Canonical units** (`simcore.quantities.CANONICAL_UNIT`):
  `CARBON = mol`, `ENERGY = J` (golden-locked, untouched); **`WATER = kg`**,
  **`NITROGEN = kg`** (mass basis — kg H₂O matches Penman–Monteith mm/day =
  kg m⁻² day⁻¹; kg N is unambiguous element mass, unlike species-ambiguous "mol N");
  `OXYGEN = mol` (untracked in P1; molar keeps gas species consistent for deferred
  Phase-2 stoichiometry). *WATER is a genuine kg-vs-mol toss-up (Phase-2 molar gas
  stoichiometry was the only real counter-argument); user chose kg with the cost
  understood.*
- **Plan-claim correction.** The earlier "WATER/NITROGEN appear in no committed
  golden" is true only for NITROGEN. **WATER appears in
  `tests/regression/golden/state_snapshot.json`** (a `bio.water` POOL stock), which
  is byte-pinned. The WATER→kg flip **regenerated that one golden** (mechanical —
  `_golden_state()` reads `canonical_unit(WATER)` dynamically; the demo goldens are
  carbon/energy-only and untouched). No `sim_io` schema-version bump: the unit is a
  label *value*, not a schema-structure change (the version bump is Step 2's aux
  field).
- **Ground-area basis — the absolute-vs-per-area split (written down, the real
  substance of the lock).** Per-area params (mm day⁻¹, µmol CO₂ m⁻² s⁻¹, kg N ha⁻¹)
  are *dimensionally incompatible* with the absolute canonical unit in pint (length
  vs mass; even kg m⁻² ≠ kg), so `to_canonical` neither can nor should touch them:
  - **Absolute amounts** (initial stock values, e.g. soil-water content in kg) →
    validated/converted by `to_canonical` against the quantity's canonical unit.
    This is what Step 1's unit-validation tests exercise (WATER/NITROGEN kg ↔ g
    convert; mol/L/etc. are rejected).
  - **Per-area rate-law params** (mm day⁻¹, µmol m⁻² s⁻¹) → the per-leg dimensional
    closure that P4 **defers**: schema-validated floats carrying a *declared unit in
    the param-file header*, multiplied by the scenario **`ground_area`** (m²) inside
    `evaluate` to yield an absolute leg in the canonical unit. `ground_area` is
    scenario data, not core state. A typed per-leg dimensional check stays a future
    enhancement (P4).
- **kg-DM ⇄ mol-C conversion** lives in the **biosphere loader**
  (`domains/biosphere/loader.py`), not `config/units.py`: it is crop-specific data
  (carbon fraction kg C / kg DM), not generic pint. Explicit cited arithmetic —
  `mol_C = mass_kg · f_C / M_C`, `M_C = 12.011 g/mol` (IUPAC) — with a round-trip
  test and an `f_C ∈ (0, 1]` guard. Build-ahead infra; first used at allocation
  (Step 9).
- **LICENSE precondition — resolved.** **Apache-2.0** added at repo root (`/LICENSE`);
  `docs/reuse-and-licenses.md` updated. Permissive keeps the core copyleft-free per
  the reuse rationale; Apache over MIT for the patent grant.

**Tasks.**
- **Resolve `CANONICAL_UNIT` for WATER and NITROGEN** to science-correct labels
  (CARBON/ENERGY stay `mol`/`J` — golden-locked). Add a totality test (already
  exists) coverage note. *Do not* change CARBON/ENERGY without regenerating the demo
  goldens — out of scope for Step 1.
- **Ground-area convention.** Document that physiological flows take a scenario
  `ground_area` (m²) param and convert per-area rates → absolute legs inside
  `evaluate`. No core change (it is flow/scenario data); the convention is written
  down so every process step follows it.
- **Extend the loader's unit validation** to the kg-DM ⇄ mol-C (carbon fraction) and
  any WATER/NITROGEN boundary conversions, keeping Scope-A discipline (amounts
  unit-validated; dimensionless coefficients schema-validated).
- **LICENSE precondition.** Surface the license choice to the user (a `LICENSE`
  before real crop code lands, per the reuse doc). Not decided in this plan.

**Test plan.** `CANONICAL_UNIT` totality holds (every `Quantity` covered); a
WATER/NITROGEN param in a compatible unit converts, an incompatible one raises
`UnitValidationError`; the kg-DM→mol-C boundary conversion round-trips a known
value; the Phase-0 demo goldens are **unchanged** (CARBON/ENERGY labels untouched).

## Step 2 design — the non-conserved aux channel (P2)

*Realizes P2 — the load-bearing step; reviewed before any process step.*

### RESOLVED (2026-06-17) — the locks (implemented + advisor-reviewed)

- **`State.aux: Mapping[str, float]`** — additive 4th field, default a shared empty
  `MappingProxyType` (re-wrapped per instance in `__post_init__`), so positional
  `State(n, stocks, rng_seed)` and every pre-P2 call site are unchanged. Values are
  **finiteness-validated** (NaN/Inf rejected) with the *isfinite-only* discipline of
  `Stock.amount` — no further coercion. The mapping is detached from the caller dict
  and read-only, exactly like `stocks`.
- **`simcore/auxiliary.py` (pure core)** — `AuxId = NewType(str)` and the `AuxProcess`
  Protocol (read-only `id` property + `evaluate(snapshot, env, dt) -> Mapping[str,
  float]` returning per-name **increments**, increment-form like `Flow`). No balance
  check (non-conserved by definition). `AuxProcess.id` is the *process* id (dedup +
  canonical order), distinct from the accumulator *names* it writes — several
  processes may write one shared name (summed).
  - **Module-name deviation (forced):** the plan's `aux.py` is **unusable** — `AUX`
    is a reserved Windows device name, so git's Win32 APIs can't commit/clone
    `aux.py` (and an eventual Rust `aux.rs` would hit the same wall). The file is
    **`auxiliary.py`**; the Python identifiers (`AuxProcess`, `AuxId`, `State.aux`,
    `aux_processes`) are unaffected — only the filename was reserved.
- **`Registry`** gained an optional `aux_processes` (default `None`), sorted by
  `AuxId` and duplicate-id-rejected — the *same* discipline as flows — so
  `Registry(flows, stocks)` and `Integrator(registry)` are unchanged.
- **Integrator** — aux advances in **`_apply`** (reached **only** by `step_report`):
  one explicit-Euler `_aux_increments` evaluation at the **step-entry** snapshot,
  summed per name across processes in canonical `AuxId` order (#15), folded into the
  single `n→n+1` commit. **RK4 carries aux for free** — `_perturb` only `replace`s
  `stocks`, so stage states keep aux like they keep `n`; a flow reading aux sees a
  within-step constant and aux advances exactly once per step regardless of scheme.
  `substep`/`_deltas` never touch aux.
- **aux × multirate — documented + placement-test guarded, NO runtime raise**
  (advisor-endorsed). `multirate_step` is typed against `Substepper`, which exposes
  no registry; a runtime guard would need `isinstance(_BaseIntegrator)` + reach into
  `.registry`, breaking the abstraction multirate is deliberately decoupled from, to
  defend a case Phase 1 explicitly defers — against the repo's anti-speculation norm.
  `substep` leaving aux untouched is the structural guard; `test_aux.py` pins it.
- **`sim_io.snapshot` — `SCHEMA_VERSION` 1→2**; `aux` serialized as a **key-sorted
  object of hex-float strings** (same exactness/canonical order as amounts). **The
  bump is NOT golden-only** (an earlier plan claim was wrong): two version-hardcoding
  tests were updated (`test_schema_version_constant_exposed` → 2;
  `test_unknown_schema_version_rejected` now asserts **v1 is rejected**). All **3
  goldens regenerated** via their explicit `_regenerate`/`__main__` actions (a
  `_regenerate()` block was added to `test_sim_io_snapshot.py`); the diff is clean —
  only `version` + an `aux` block changed, no stock amount drifted.
- **`conservation` untouched** — `compute_ledger` and its key-set guard reason over
  `stocks` only, so aux is invisible by construction (pinned: an aux-only change
  conserves trivially; an unbalanced *stock* change still trips the gate).
- **`Observation` NOT extended** — deferred, no named Phase-1 consumer (P2).

**Tasks.**
- `State.aux: Mapping[str, float]` (additive field, empty `MappingProxyType`
  default; `__post_init__` wraps/validates finiteness like amounts).
- `simcore/auxiliary.py` (PURE core): the `AuxProcess` protocol (`evaluate → per-name
  increments`, increment-form like `Flow`) + an `AuxId` type. No balance check
  (non-conserved by definition).
- **Integrator:** advance aux by **explicit Euler at the step-entry snapshot**
  (one evaluation; sum increments per name in **canonical name order**, #15);
  carry aux unchanged across RK4 stages (like `n`); apply at the single `n→n+1`.
  Wire aux processes as a construction dependency (lean: `Registry` gains an
  optional aux-process collection, default empty, so `Integrator(registry)` is
  unchanged). Settle the exact placement at implementation.
- **aux × multirate placement (decide explicitly — the spine is shared).** Aux
  advances in **`step_report` only**; **`substep` leaves aux untouched** (and `_deltas`
  must **not** carry aux). `simcore.multirate` composes `substep` `n_sub`× per master
  step while keeping `n` — so advancing aux in the shared `_deltas`/apply path would
  advance aux `n_sub`× per master step (wrong), and advancing only in `step_report`
  would make multirate silently drop aux. Phase 1 is single-rate, so **aux + multirate
  is out of scope and guarded/documented** (not exercised); the placement is pinned
  now because Step 2 edits the shared `_BaseIntegrator`.
- **Ripple:** `sim_io.snapshot` serializes `aux` (hex-float, key-sorted) + a schema
  bump (regenerate the demo goldens — an explicit, separate regeneration action, per
  the Phase-0 golden discipline). **`Observation` is not extended** (deferred — no
  named consumer, P2).
- **conservation:** confirm aux is invisible to `compute_ledger` (stocks-only key
  set) — a test pins that an aux-only change does **not** affect the ledger.

**Test plan.** Aux accumulates by Euler (a constant-rate aux process integrates to
`rate·n·dt`); aux is carried (not perturbed) across RK4 stages (a flow reading aux
sees a within-step constant); aux is **excluded** from the conservation gate (an
aux change alone conserves trivially; a deliberately unbalanced *stock* change still
trips it); snapshot round-trip preserves aux bit-exactly (hex-float); a multirate
`substep` leaves aux **untouched** while `step_report` advances it once (the placement
guard); determinism + registration/aux-order independence (Hypothesis); the
empty-default keeps every Phase-0/0.5 test green (only the goldens regenerate, by the
schema bump).

## Step 3 design — PCSE oracle harness + clean-room param discipline (P5)

*Realizes P5. Built early as infra (a target for the process steps); the full
behavioral-match gate is Step 11.*

### RESOLVED (2026-06-19) — the locks (advisor-reviewed; empirically grounded)

- **Crop/oracle = winter wheat / WOFOST** (user-confirmed at Step 3, knowing the
  licensing constraint below). Empirically verified with the installed
  `pcse==6.0.13`: `YAMLCropDataProvider` exposes `wheat → Winter_wheat_101…106`, and
  a full `Wofost72_PP` season (sow 2006-10-01, NL 52°N/5°E, NASAPower weather) runs
  **305 daily steps** emitting `DVS, LAI, TAGP, TWLV/TWST/TWRT, TWSO, TRA, SM`
  (final TAGP ≈ 20.4 t/ha, TWSO ≈ 11.5 t/ha — realistic). Winter wheat is the
  closest oracle for the WOFOST-family equations Phase 1 reimplements (FvCB-style
  assimilation, Penman–Monteith, DVS-keyed allocation, thermal time); LINTUL3 spring
  wheat (the only *bundled* crop) is a looser light-use-efficiency oracle.
- **The licensing safeguard (load-bearing — advisor-flagged blocker, now closed).**
  PCSE itself is EUPL → running it as an oracle is *mere use*, and its **output is
  facts** (`docs/reuse-and-licenses.md` lines 13/28). But winter-wheat **inputs**
  come from the **`WOFOST_crop_parameters` repo, which has no license = all rights
  reserved** ("Do NOT copy the files", reuse doc line 14). Winter wheat is **not
  bundled** in PCSE; `YAMLCropDataProvider` **downloads** that repo to the user's
  `~/.pcse` cache. Therefore the hard rule: **we commit ONLY the output trajectory
  + a provenance record — NEVER the param YAML.** The download is a private,
  transient local copy used to run the model (mere use → factual output); we
  redistribute nothing. This holds regardless of crop and is the design's central
  constraint.
- **Provenance record (committed beside the fixture)** makes the clean lineage
  auditable and regeneration reproducible: PCSE version (`6.0.13`), crop-parameter
  source + repo version/branch, weather source (NASAPower, lat/lon, date range),
  agromanagement (sow/harvest dates, variety), WOFOST model variant. A reviewer can
  see the fixture is derived output, not copied params.
- **Module placement (advisor-affirmed split).**
  - **PCSE-driving runner → `tests/oracle/runner.py`** — a **non-`test_` module name**
    so pytest's collection glob never imports it on a machine without `pcse` (no
    collection-time `ImportError`). It lives under `tests/` (NOT a shipped hatch
    package in `pyproject.toml`'s `packages=[…]`), so **EUPL PCSE code never enters
    `src/`**. The companion `tests/oracle/test_oracle.py` does the `oracle`-marked,
    `pytest.importorskip("pcse")` checks.
  - **Behavioral-match tolerance helper → `src/lab/oracle_match.py`** — **pure
    stdlib**, exactly like `lab/convergence.py:fit_order`. It compares two in-memory
    trajectories; it needs **no PCSE**. (`lab` *ships* in the wheel, so this stays
    PCSE-free by discipline — the AST purity gate only scans `simcore`; a one-line
    test asserts nothing in `lab` imports `pcse`.)
  - **Committed fixture decouples everything from PCSE:** the helper + its tests run
    against the committed winter-wheat reference trajectory with **zero PCSE
    dependency**. PCSE is needed *only* to regenerate the fixture.
- **Fixture format = plain decimal JSON, not hex-float.** The oracle match is
  **behavioral / within-a-band** (P5), not bit-exact — readable reference numbers
  beat exact-float goldens, and there is no conflict with determinism invariant #7
  (that governs *our* engine's bit-identity, not the oracle's). Variable set for the
  first fixture: `day, DVS, LAI, TAGP, TWLV, TWST, TWRT, TWSO, TRA, SM` (a
  `Wofost72_PP` potential-production season). N-limited variables and a
  water-limited (`WLP`) variant are **JIT extensions** added when the water (Step 7)
  and nitrogen (Step 10) processes land — Step 3 pins biomass + LAI + water-use now.
- **Tolerance helper shape** (mirrors `fit_order`'s discipline — a measurement, not a
  pass/fail policy): given a candidate trajectory and the reference, return a
  per-variable discrepancy metric (max relative deviation and/or normalized RMSE over
  aligned days) — pure float arithmetic. A **discriminating control test** proves it
  bites: a within-band perturbation accepts, an out-of-band one rejects (the
  synthetic check `fit_order` uses). The actual Phase-1 *gate* (which variables, what
  band) is wired at Step 11.
- **Clean-room param discipline — convention now, automated check deferred.** Step 3
  establishes the **param-file header template** (cite the primary publication per
  value) + a **review checklist** ("no value copied from the unlicensed WOFOST YAML")
  in docs. **No crop param files exist yet** (they land in Steps 4–10), so an
  automated header-presence check is premature — deferred until params arrive (the
  repo's anti-speculation norm).
- **Purity stays green by construction** — PCSE lives in `tests/oracle/`, outside all
  `src/` packages; the existing AST gate (`tests/test_simcore_purity.py`) re-asserts
  `simcore` imports nothing third-party. The `oracle` marker de/selects the oracle
  tests; default `uv run pytest` stays green without PCSE via `importorskip`.

**Tasks.**
- Enable the `oracle = ["pcse"]` dependency group (`pyproject.toml`); register an
  `oracle` (and/or reuse `slow`) pytest marker; confirm `simcore` purity is
  unaffected (PCSE is dev/test-only, never imported by core — re-assert via the
  existing AST gate).
- A thin **offline runner** (under `lab/` or `tests/oracle/`, **outside** `simcore`)
  that drives PCSE/WOFOST for one crop under a given weather/management forcing and
  captures a **reference trajectory** (biomass, LAI, gas-exchange/water-use
  magnitudes, N) as committed fixture data (numbers are facts, not PCSE code —
  license-clean per the reuse doc).
- The **behavioral-match** tolerance helper (trajectory shape/magnitude within a
  band), analogous to the Phase-0.5 `fit_order` discipline (a measurement helper,
  not a bit-compare).
- **Clean-room param discipline:** param-file header template that **cites the
  primary publication** for every value; a check (or review checklist) that no value
  is copied from the unlicensed WOFOST YAML.

**Test plan.** The oracle runner produces a deterministic reference fixture (pinned);
the behavioral-match helper accepts a within-band trajectory and rejects an
out-of-band one (a discriminating control, like `fit_order`'s synthetic check); the
purity gate stays green (PCSE not in `simcore`'s import closure); the marker
de/selects the oracle tests as intended.

## Step 4 design — Beer–Lambert light interception (Monsi & Saeki 1953)

*The first biological process, designed just-in-time. Establishes the **canopy
diagnostic** that Step 5 (FvCB) consumes — absorbed PAR = incident PAR × intercepted
fraction.*

### RESOLVED (2026-06-19) — the locks (advisor-reviewed)

- **It is a pure diagnostic, NOT a `Flow` and NOT an `AuxProcess`.** Both LAI and the
  intercepted fraction are **derived on demand** (P2: "LAI is derived, not stored"),
  so light interception adds **no leg, no accumulator, no `State` field, and no
  integrator wiring**. It is a standalone pure module (`domains/biosphere/canopy.py`),
  built and unit-tested in isolation now (roadmap line 232 — "exists before
  integration"); Step 5's carbon flow imports and calls it. The advisor flagged the
  one temptation to resist — making LAI an aux accumulator "for symmetry with thermal
  time"; it is not one.
- **Two free functions, params via a `CanopyParams` dataclass (the `DemoParams`
  idiom).** The physics is free functions so Beer–Lambert can be tested on a raw LAI
  without routing through a contrived carbon value:
  - `leaf_area_index(leaf_carbon, *, sla_per_mol_c, ground_area) = leaf_carbon ·
    sla_per_mol_c / ground_area` (dimensionless m²/m²).
  - `intercepted_fraction(lai, *, extinction_coef) = 1 − exp(−k·LAI)` ∈ [0, 1) — the
    Monsi & Saeki extinction law (`I/I₀ = exp(−k·LAI)` transmitted). 0 at no leaf
    area, → 1 as the canopy closes: the shape of a P1 limitation factor.
  A "Canopy object with methods" would couple the two; deferred until a Step-5
  consumer actually wants a carrier (anti-speculation).
- **`carbon_fraction` folds into `sla_per_mol_c` at the loader (the Step-1 lock).**
  Specific leaf area is conventionally m²/kg dry matter, but our currency is mol leaf
  C. The kg-DM⇄mol-C conversion lives at the config boundary, not core, so the loader
  computes `sla_per_mol_c = SLA[m²/kg] · M_C[kg/mol] / carbon_fraction[kg C/kg DM]`
  (kg DM per mol C = M_C / f_C) and `canopy.py` never holds the molar-mass constant.
  This is the **first consumer** of the carbon-fraction conversion (the plan expected
  Step 9 — harmless; note the future dedup with allocation's tissue carbon fraction).
- **`ground_area` (m²) is a scenario call-arg, NOT a crop param (P4).** It stays out
  of `canopy.yaml` and `CanopyParams`; the physics takes it as an argument (guarded
  `> 0`). Note its role here is a **divisor** (LAI = leaf area ÷ ground area) — the
  mirror of the per-area-rate **× ground_area** *multiply* that mass-bearing
  physiological flows use to turn a per-m² rate into an absolute leg.
- **First real param file → the Step-3 structured `value/unit/source` format**
  (`params/canopy.yaml`), loaded by a **bespoke** `load_canopy_params` (hand-written
  schema like `_DemoSchema`; a generic structured-param loader is premature with one
  instance). Specific leaf area carries a **per-area unit** (m²/kg) that is *not* a
  conserved `Quantity`'s canonical unit, so it is validated by a new general
  `config.convert(value, target_unit)` — the same Scope-A boundary discipline as
  `to_canonical`, generalized to an explicit target unit (still **not** the deferred
  per-leg `Flow` dimensional check). `extinction_coef` (k > 0) and `carbon_fraction`
  (∈ (0, 1]) are dimensionless, bound-checked floats.
- **pint notation correction (empirically forced).** The Step-3 convention doc's
  example notation `"m2 kg-1"` / `"umol m-2 s-1"` is **unparseable** by the installed
  pint: it reads `kg-1` as `kg minus 1` (`DimensionalityError`) and does not know
  `m2`. Param-file units must use `^`/`**` and `/` — `"m^2/kg"`, `"umol/m^2/s"`,
  `"mm/day"`. `docs/param-file-conventions.md` was corrected.
- **Clean-room (P5).** The extinction LAW is cited to Monsi & Saeki (1953); the
  numeric parameter VALUES are honest **provisional placeholders** (`TODO(cite)`,
  literature-typical) pending the Step-11 validation gate — never fabricated to a
  recalled citation and never backfilled from the unlicensed WOFOST YAML (the
  convention doc's sanctioned provisional path; values are placeholders until Step 11
  anyway).

**Test plan (`tests/test_canopy.py`).** Beer–Lambert known values against
**independent hand-computed literals** (not `1-exp(...)` restatements); the limits
(LAI=0 → 0; large k·LAI → 1 from below); monotonicity in LAI; the `ground_area > 0`
guard; the composed carbon→LAI→fraction chain; `config.convert` accepts a compatible
unit (`ha/kg` → `m²/kg`) and rejects an incompatible/unparseable one; the committed
`canopy.yaml` folds to the hand-computed `sla_per_mol_c`; the loader rejects a
dimensionally-wrong SLA, an out-of-range carbon fraction / extinction coef, and a
missing `source` tag (clean-room discipline at the boundary).

## Step 5 design — FvCB photosynthesis (Farquhar, von Caemmerer & Berry 1980)

*The first carbon **source** flow, designed just-in-time. Consumes the Step-4 canopy
diagnostic (absorbed PAR = incident PAR × intercepted fraction) and deposits gross
assimilated carbon into the plant pool.*

### RESOLVED (2026-06-19) — the locks (advisor-reviewed)

- **A single CARBON source `Flow` (`GrossAssimilation`), `boundary.co2 → plant_c`,
  internally balanced in carbon (P1).** New pure module
  `domains/biosphere/photosynthesis.py` (stdlib only), a `PhotosynthesisParams`
  dataclass (the `CanopyParams`/`DemoParams` idiom), a bespoke
  `load_photosynthesis_params` in `loader.py`, and `params/photosynthesis.yaml` in the
  structured `value/unit/source` format. Roadmap line 232: flow + unit test + param
  file + doc, standalone before integration.
- **Two layers, split deliberately (advisor seam).**
  - **Instantaneous leaf-level FvCB — the citable, exactly hand-checkable part**
    (tested against independent literals, Step-4 style):
    - Rubisco-limited `Ac = Vcmax·(Ci − Γ*) / (Ci + Kc·(1 + O/Ko))`.
    - Electron transport `J` from absorbed PAR via the **non-rectangular hyperbola**
      `θ·J² − (I₂ + Jmax)·J + I₂·Jmax = 0` (smaller root), `I₂ = α·absorbed_par`.
    - Light/RuBP-limited `Aj = J·(Ci − Γ*) / (4·Ci + 8·Γ*)`.
    - Gross leaf assimilation `Ag = max(0, min(Ac, Aj))`. The `Ac/Aj` form is
      gross-of-dark-respiration, **net-of-photorespiration** — the `(Ci − Γ*)` factor
      already books photorespiratory CO₂, so there is no hidden mass leak.
  - **Provisional canopy + diurnal aggregator** `daily_canopy_assimilation(...)` — a
    **free function the multilayer/diurnal Gaussian extends additively** (same
    signature). Big-leaf: absorbed canopy PAR (per ground) `= incident_par · f_int`;
    mean absorbed PAR per leaf area `= incident_par · f_int / LAI` drives the leaf
    curve (well-defined as LAI→0: `f_int ≈ k·LAI`, so the ratio → `k·incident_par`,
    finite; guarded `Ag = 0` at exactly `LAI = 0`); canopy rate `= Ag · LAI`; daily
    flux `= canopy_rate · daylength_s · ground_area · 1e-6 (µmol→mol) · f_temp`.
- **The Γ* clamp is load-bearing for a source flow.** When `Ci ≤ Γ*`, `(Ci − Γ*) ≤ 0`
  flips `Ag` negative — the flow would *withdraw* from `plant_c` (a source turned
  withdrawal, tripping positivity/extinction). `Ag = max(0, …)` clamps it to 0,
  matching P3's "assimilation → 0 as CO₂/light → 0." The `Ci ≤ Γ*` → 0 case is unit-
  tested.
- **f_temp correction — a latent plan bug, recorded here (Steps 1/2 set the
  precedent).** Plan line 329 lumps `f_temp` with `f_water`/`f_N` as "limiters default
  to 1.0 **until their step lands**." But the step sequence lands `f_water` at Step 7
  and `f_N` at Step 10 — **f_temp lands at no step** (Step-8 phenology drives DVS via
  thermal time, *not* the instantaneous assimilation temperature response). So
  "1.0 until its step lands" would silently mean "1.0 **forever**," and a
  temperature-independent FvCB over a winter-wheat season (Oct→Aug; sub-zero winter →
  20 °C+ summer) would assimilate near-max through winter — a **structural** mismatch
  the Step-11 oracle gate would catch. → **Step 5 wires a multiplicative `f_temp(T)`
  now** (temperature forcing via `env.get`; the WOFOST **TMPFTB** idiom — a cardinal-
  temperature response of the assimilation rate, citable to crop-model literature). A
  **piecewise-linear cardinal-temperature** factor (0 below `T_min`, ramp to 1 over
  `[T_min, T_opt_lo]`, plateau 1 on `[T_opt_lo, T_opt_hi]`, ramp down to 0 over
  `[T_opt_hi, T_max]`) — independently hand-checkable. `f_water`/`f_N` genuinely have
  later steps, so they **stay 1.0** with the `Π fᵢ` seam left in place (a documented
  multiplier in `evaluate`, additive at Steps 7/10).
- **FvCB params stay at a reference temperature (no Arrhenius now).** Full
  `Vcmax(T)/Jmax(T)/Γ*(T)` Arrhenius scaling is **out of scope** for Step 5; the
  multiplicative `f_temp` is the right altitude (the temperature response of AMAX, not
  re-deriving every kinetic constant). Folding Arrhenius into FvCB is a Step-11
  refinement if the oracle match needs it.
- **CO₂ pattern mirrors the demo's `light` (decision #16/#8 shape).** Intercellular CO₂
  `Ci` (µmol mol⁻¹), incident PAR, air temperature, and photoperiod `daylength_s` are
  read via `env.get` as **scalar drivers** (forcing or shared stock — the flow cannot
  tell). **Unlike the demo (rate ∝ the withdrawn stock's amount), here the rate is set
  by the `Ci` forcing, independent of `boundary.co2.amount`** — correct because CO₂ is
  an **unclamped, non-limiting** boundary source in the open Phase-1 single-producer
  system (P1; no rationing → `rationed == 0`). Using `Ci` directly as forcing is fine;
  stomatal `Ci/Ca` coupling is naturally Step 7's (water).
- **Light enters ONLY through absorbed PAR, never a separate `f_light` (Step-4
  caveat).** The Step-4 intercepted fraction `f_int` drives the electron-transport
  curve (absorbed PAR = incident · `f_int`); wiring `f_int` *also* as a standalone
  `Π fᵢ` factor would double-count light limitation.
- **Deposit gross, not net (carbon-vs-DM caveat).** Step 5 deposits `Ag` (gross
  min(Ac,Aj)). The WOFOST conversion-efficiency / **growth-respiration** carbon loss
  is an **explicit balanced leg in Step 6**, never a silent efficiency factor (a
  dropped-carbon factor trips the every-step gate — the gate doing its job). Dark
  respiration `Rd` (maintenance) is likewise Step 6.
- **Stock structure — single provisional `plant_c` POPULATION pool (Step-9 note).**
  The flow deposits into `biosphere.plant_c` and reads that **same** pool as
  `leaf_carbon` for the canopy diagnostic. The leaf/stem/root organ split is the
  explicit Step-9 transition; Step 5 does not pre-empt it.
- **Big-leaf high-bias — named, not assumed away (advisor flag).** `Ag` is **concave**
  in PAR (saturating `J`, then `min`), so a big-leaf on **daily-mean / canopy-mean**
  PAR overestimates the true daily integral (Jensen) — exactly why WOFOST does the
  intra-canopy/diurnal Gaussian, which P3 calls load-bearing for the oracle match.
  Step 5 accepts the bias **because** `daily_canopy_assimilation` is the additive seam
  for the Gaussian; **closing it is Step 11**, not an assumption that the big-leaf
  already matches.
- **P3-premise correction — the flow is dt-LINEAR, not dt-nonlinear (advisor-flagged
  divergence from a locked decision; recorded in the Steps-1/2 style).** P3 asserts
  Phase-1 biology "uses `dt` non-linearly (the day's Gaussian integration is inside
  `evaluate`) … and is **not dt-refinable**." That mechanism does **not** hold for what
  was built: `daily_canopy_assimilation` integrates over the **photoperiod
  `daylength_s`** — an astronomical *forcing* read via `env.get`, **decoupled from the
  integrator step `dt`** — so the daily rate is dt-independent and `flux = daily·dt` is
  exactly the increment form (the Phase-0 RK4 contract *holds*; `test_..._scales_
  linearly_with_dt` proves it). Decoupling the photoperiod from `dt` is the correct
  modeling choice (and cleaner than P3 anticipated; even the Step-11 Gaussian, if it
  integrates over `daylength_s`, stays dt-linear). **P3's conclusion stands, its
  rationale is corrected:** the crop scenario still **selects Euler-daily — to match
  the oracle's daily numerics (P3), not because the flow forfeits RK4 order.** (The
  engine-numerics gates remain proven on the analytic Phase-0.5 scenarios; the gate
  split is unaffected.)
- **Clean-room (P5).** The FvCB equations are cited to Farquhar, von Caemmerer & Berry
  (1980); the cardinal-temperature response to degree-day/crop-model literature. All
  numeric VALUES (`Vcmax, Jmax, α, θ, Γ*, Kc, Ko, O, T_*`) are honest **provisional
  literature-typical placeholders** (`TODO(cite)`), pending the Step-11 validation gate
  — never fabricated to a recalled citation, never backfilled from the unlicensed
  WOFOST YAML.

**Tasks.**
- `domains/biosphere/photosynthesis.py` (PURE stdlib): the leaf-level free functions
  (`rubisco_limited_rate`, `electron_transport_rate`, `light_limited_rate`,
  `gross_leaf_assimilation`), `temperature_factor`, the provisional
  `daily_canopy_assimilation` aggregator, the `PhotosynthesisParams` dataclass, and the
  `GrossAssimilation` flow.
- `params/photosynthesis.yaml` (structured `value/unit/source`) + `load_photosynthesis_params`
  in `loader.py` (bespoke schema like `_CanopySchema`; dimensionless coefficients
  bound-checked, the `Ci`/PAR/temperature *forcing* lives in the scenario/resolver, not
  the crop param file).
- Wire the `Π fᵢ` seam (`f_temp` populated; `f_water`/`f_N` = 1.0, documented additive).

**Test plan (`tests/test_photosynthesis.py`).** `Ac`, `Aj`, the non-rectangular-
hyperbola `J`, and `Ag = min(...)` against **independent hand-computed literals**;
the **`Ci ≤ Γ*` → `Ag = 0`** clamp; `Ag → 0` as PAR → 0; monotonicity in PAR and Ci
(below saturation); `temperature_factor` cardinal-point values (0 below `T_min`,
1 in the plateau, 0 above `T_max`, linear ramps) against literals; the `LAI → 0`
limit of `daily_canopy_assimilation` is finite (no division blow-up) and `= 0` at
`LAI = 0`; the assembled `GrossAssimilation.evaluate` produces a **carbon-balanced**
`FlowResult` (`assert_flow_balanced`) with the hand-computed daily mol-C leg; the
committed `photosynthesis.yaml` loads to the expected `PhotosynthesisParams`; the
loader rejects out-of-range/bad-unit params and a missing `source` tag.

## Step 6 design — maintenance + growth respiration (the carbon sink flows)

*The carbon **sink** counterpart to Step 5, designed just-in-time. Books the two
respiratory losses that turn Step 5's gross-deposited carbon into the net structural
increment — including the growth-respiration leg Step 5 flagged.*

### RESOLVED (2026-06-19) — the locks (advisor-reviewed)

- **Two single-currency CARBON sink `Flow`s, both `plant_c → boundary.co2`, each
  internally balanced in carbon (P1).** New pure module
  `domains/biosphere/respiration.py` (stdlib only), a `RespirationParams` dataclass
  (the `PhotosynthesisParams`/`CanopyParams` idiom), a bespoke `load_respiration_params`
  in `loader.py`, and `params/respiration.yaml` in the structured `value/unit/source`
  format. Roadmap line 232: flow + unit test + param file + doc, standalone before
  integration. Both respire into the **same** `boundary.co2` reservoir gross
  assimilation draws from (open/unclamped in the Phase-1 single-producer system); a
  separate atmosphere sink would be speculative.
- **Maintenance respiration** — `MRES = maintenance_coef · plant_c · Q10^((T−T_ref)/10)`
  (mol C day⁻¹). Proportional to standing biomass and rising with temperature via a
  `Q10` response (**unbounded above**, unlike the FvCB cardinal factor — correct for
  maintenance). Self-limiting in `plant_c` (→ 0 as biomass → 0), so positivity is
  **structural** (no backstop dependence; P3). Cite: McCree (1970); Penning de Vries
  et al. (1974); Thornley (1970); Amthor (2000).
- **Growth respiration — the maintenance-first paradigm (the load-bearing
  disambiguation; advisor-flagged, the plan prose was loose).** Plan line ~343's
  "gross assimilation → growth respiration → structural carbon" does **not** say
  whether maintenance is subtracted first. It is: growth respiration acts on the
  assimilate **remaining after maintenance** — `GRES = (1−Yg)·max(0, GASS − MRES)`
  (mol C day⁻¹), `Yg` = carbon growth-conversion efficiency. This is the
  McCree–de Vries–Thornley budget and exactly how the **Step-11 WOFOST oracle** budgets
  carbon (`ASRC = GPHOT − MRES`, then `× CVF`). The alternative (growth resp on gross,
  lighter dependencies) was **rejected**: it is a *different carbon budget* that would
  force an architecture shift mid-validation — A means Step 11 only tunes coefficients.
  The `max(0, …)` clamp is **load-bearing** (the Step-6 analogue of Step 5's `Γ*`
  clamp): when `MRES ≥ GASS` there is no growth, hence no growth respiration, and the
  flow never flips into a carbon-creating *deposit*. Net `plant_c` change across the
  three flows = `GASS − MRES − GRES = Yg·(GASS − MRES)` — the structural increment.
  Cite: Penning de Vries et al. (1974); Thornley (1970).
- **`GrowthRespiration` recomputes `GASS` and `MRES` — forced, not a smell.** Flows
  `evaluate` independently against the step-entry snapshot (a flow cannot read another
  flow's result pre-arbitration), so a **flux-coupled** quantity must recompute its
  inputs: `GASS` via the Step-5 `daily_canopy_assimilation` seam, `MRES` via the
  **same** `maintenance_respiration_flux` `MaintenanceRespiration` uses (so the two
  flows can never drift on the maintenance value — the one genuine DRY hazard). Folding
  growth resp into `GrossAssimilation` is foreclosed (it would break Step 5's committed
  gross-leg tests); a single combined budget flow loses per-process diagnostics. The
  long field list on `GrowthRespiration` (photosynthesis + canopy + respiration params,
  `ground_area`, the same forcing-var names as `GrossAssimilation`) is the **inherent
  cost** of a flux-coupled quantity in an independent-flow engine.
- **dt-linearity preserved.** The `max(0, …)` clamps a *dt-independent daily rate*, so
  `flux = const·dt` still holds (the RK4 increment-form contract carries over from
  Step 5; the `scales_linearly_with_dt` tests pass for both flows). The clamp is not a
  dt-nonlinearity.
- **Carbon-basis rates (the Step-1 lock).** `maintenance_coef` and `Yg` are expressed
  on a **carbon basis** (mol C per mol C biomass), so the pure physics never holds the
  kg-DM⇄mol-C carbon fraction — it folds into the placeholder values, not the equations.
- **Deferred seam — maturity/senescence (documented, like Step 5's f_water/f_N).**
  WOFOST scales maintenance *down* as tissue matures (a development-stage / senescence
  factor); that lands with phenology (Step 8) and the Step-11 oracle tuning. Standalone
  Step 6 is plain `Q10·biomass` with the multiplier seam (`maturity` argument, default
  1.0) in place, so Step 11 is a coefficient change, not a structural one.
- **Known provisional behavior — winter biomass decline (the Step-6 analogue of
  Step 5's documented big-leaf high-bias).** When `GASS < MRES` (dark, cold) the net
  `plant_c` change is `−(MRES − GASS) < 0`: standing biomass shrinks. Physically real
  for a respiring plant in carbon deficit; dormancy / vernalization is the Step-11
  refinement — a known provisional behavior, not a bug.
- **Self-limiting / positivity (P3).** Both fluxes → 0 as `plant_c` → 0 (maintenance
  ∝ `plant_c`; growth resp ∝ `max(0, GASS − MRES)` with `GASS` → 0 as LAI → 0), so
  positivity is structural. `rationed == 0` over a full Euler-daily season is a Step-11
  *scenario* concern (keep `maintenance_coef·Q10^max·dt` well under 1 — realistic
  ~0.06), not a Step-6 test.
- **Clean-room (P5).** The respiration paradigm is cited to the McCree–de Vries–Thornley
  literature; all numeric VALUES (`maintenance_coef`, `q10`, `t_ref`, `growth_efficiency`)
  are honest **provisional literature-typical placeholders** (`TODO(cite)`), pending the
  Step-11 validation gate — never fabricated to a recalled citation, never backfilled
  from the unlicensed WOFOST YAML.

**Tasks.**
- `domains/biosphere/respiration.py` (PURE stdlib): `q10_factor`,
  `maintenance_respiration_flux` (with the `maturity` seam), `growth_respiration_flux`
  (the clamped maintenance-first loss), the `RespirationParams` dataclass, and the
  `MaintenanceRespiration` + `GrowthRespiration` flows.
- `params/respiration.yaml` (structured `value/unit/source`) + `load_respiration_params`
  in `loader.py` (bespoke schema like `_PhotoSchema`; exact-string unit guard for the
  non-canonical `1/day`/`dimensionless`/`degC` units; `maintenance_coef`/`q10` > 0,
  `growth_efficiency` ∈ (0, 1]).

**Test plan (`tests/test_respiration.py`).** `q10_factor`, `maintenance_respiration_flux`,
and `growth_respiration_flux` against **independent hand-computed literals**; the
`MRES ≥ GASS` → 0 growth-resp clamp; maintenance ∝ biomass, → 0 at zero biomass, and the
`maturity` seam; the assembled `MaintenanceRespiration` and `GrowthRespiration` flows
produce **carbon-balanced** `FlowResult`s (`assert_flow_balanced`) with the hand-computed
/ composed daily mol-C legs, the dark-day (`PAR = 0`) growth-resp clamp, and `dt`-linear
scaling; the committed `respiration.yaml` loads to the expected `RespirationParams`; the
loader rejects out-of-range/bad-unit params and a missing `source` tag.

## Step 7 design — Penman–Monteith transpiration + root uptake (Monteith 1965)

*The first **WATER**-currency flows, designed just-in-time. Establishes the soil-water
state the season depletes/refills and the ``water_stress_factor`` (``f_water``) that
the Step-11 integration wires into assimilation.*

### RESOLVED (2026-06-19) — the locks (advisor-reviewed)

- **Two WATER `Flow`s over a single `soil_water` POOL — the 3→2 deviation from the
  inventory (recorded, like the Steps-5/6 corrections).** The foundation inventory
  (line ~382) lists **three** water flows (root uptake `soil→plant`, transpiration
  `plant/soil→vapor`, irrigation `source→soil`) implying a `plant_water` intermediate
  pool. **Step 7 builds only two** — `Transpiration` (`soil_water → vapor_sink`) and
  `Irrigation` (`water_source → soil_water`) — and **no `plant_water` pool**. The
  discriminating test is *"does any Phase-1 limiter read **plant** water?"* — **no**:
  `f_water` reads **soil** water availability, not plant water content. A `plant_water`
  pool would be written by uptake and drained by transpiration but **read by nothing**
  — a stock with no consumer, violating the repo norm that cut `Observation.kind/totals`
  and keeps LAI derived-not-stored. Root uptake **≡** transpiration in a single-bucket
  model (exactly the WOFOST `TRA` convention: one soil-water state, transpiration is
  both the uptake and the loss; the "how much the roots can extract" physics lives in
  `f_water`, not a separate leg). The inventory row already hedges ("plant/**soil**
  water → vapor sink"). Contrast **nitrogen** (Step 10), where a `plant_n` pool *is*
  justified because leaf-N concentration drives `f_N` — the asymmetry is principled,
  not a shortcut. `soil_water` is a **POOL** (throttleable, never zeroed-with-loss);
  `water_source`/`vapor_sink` are BOUNDARY reservoirs (irrigation supply / evaporative
  sink), so each flow is internally balanced in WATER (P1).
- **Penman–Monteith — the Step-5 split, with a deliberately *bounded* input interface
  (the advisor's "lock it or it sprawls").** New pure module
  `domains/biosphere/transpiration.py` (stdlib only), the citable hand-checkable
  free functions + the flows:
  - `saturation_vapor_pressure(temp_c)` = `A·exp(B·T/(T+C))` (Pa; Tetens / FAO-56
    form, `A=610.8, B=17.27, C=237.3`) and `slope_svp(temp_c)` = `B·C·e_s/(T+C)²`
    (Pa/°C — the analytic derivative of the same expression, so no separate `4098`
    magic constant). Both check against **published FAO-56 table values**
    (e_s(20 °C) ≈ 2.338 kPa, Δ(20 °C) ≈ 0.145 kPa/°C — genuine independent literals).
  - `penman_monteith_transpiration(rn, vpd, temp_c, *, r_a, r_s, soil_heat_flux=0)` —
    the combination equation `λE = [Δ·(Rn−G) + ρ_a·c_p·VPD/r_a] / [Δ + γ·(1 + r_s/r_a)]`
    (W m⁻²), then `→ mm day⁻¹` via `λE / λ_vap · 86400`. Returns **potential**
    transpiration (mm day⁻¹).
  - **Minimal forcing set (locked):** net radiation `Rn` (daily-average W m⁻²), vapor
    pressure deficit `VPD` (Pa), air temperature `T` (°C) — read via `env.get` as
    scalar drivers (#16). **`r_a` and `r_s` are crop params, NOT derived from
    wind+canopy-height+roughness** — that derivation is a Step-11 refinement that
    would balloon the forcing set; a fixed pair is hand-checkable and bounded. `G`
    defaults to 0 (negligible at the daily step).
  - **Physical constants are module-level cited values** (like `MICROMOL_TO_MOL`):
    `γ` (psychrometric, Pa/°C), `ρ_a` (air density), `c_p` (specific heat of air),
    `λ_vap` (latent heat of vaporization), `SECONDS_PER_DAY`. Universal psychrometry
    (FAO-56 standard values), **not** crop coefficients — clean-room-safe.
  - **WOFOST uses Penman + a crop factor, not full PM** (the advisor's named trap).
    Step 7 implements **what the plan cites** (PM, Monteith 1965) clean-room; the
    WOFOST-ET formulation reconciliation is an explicit **Step-11** behavioral-tuning
    concern, not a bend in the standalone physics now.
- **mm/day ≡ kg·m⁻²·day⁻¹ — the clean area conversion (no scaling factor).** At water
  density 1000 kg m⁻³, **1 mm depth over 1 m² = 1 kg**, so the absolute daily leg is
  `kg day⁻¹ = T[mm day⁻¹] · ground_area[m²]` directly — the per-area-rate × `ground_area`
  convention (P4), with the density identity stated as the module's documented
  assumption (the mirror of FvCB's µmol→mol `1e-6`). `ground_area` is a scenario flow
  field (not a crop param), exactly as in Steps 4–6.
- **`water_stress_factor` (`f_water`) — the self-limiting/positivity mechanism (P3).**
  `actual_transpiration = potential_PM · f_water(soil_water)` with
  `f_water = clamp01((soil_water − sw_wilting)/(sw_critical − sw_wilting))` — 0 at/below
  wilting, a linear ramp to 1 at/above a critical point (the available-water-fraction
  form; cite the primary soil-water-stress literature, `TODO(cite)` placeholders). As
  `soil_water → wilting`, `f_water → 0` and transpiration shuts off, so **positivity is
  structural** (the WATER analogue of Step 5's `Γ*` clamp and Step 6's `max(0,…)`).
  The `sw_wilting`/`sw_critical` thresholds are **scenario/soil data** (flow
  construction args, like `ground_area`), **not** in `transpiration.yaml` — they couple
  soil type + rooting depth, whose param-file home is deferred with soil modeling
  (anti-speculation). `f_water` stays a pure function so Step 11 wires it into the
  `GrossAssimilation`/respiration `Π fᵢ` `limitation` seam.
- **Two over-deliveries deferred to Step 11 (advisor-flagged).**
  1. **`f_water` is NOT wired into `GrossAssimilation.limitation` now.** The Step-5
     `f_temp` precedent does **not** transfer — `f_temp` was wired early *because it
     had no home step*; `f_water` has a home (this step produces it) and a consumer
     step (Step 11 wires all limiters). Standalone-first (line 232): deliver the helper
     + the water flows; leave the documented `limitation=1.0` seam untouched.
     **`photosynthesis.py`/`respiration.py` are not modified.**
  2. **No WLP / soil-moisture oracle fixture now.** The committed fixture is
     `Wofost72_PP` (potential production — water non-limiting; `TRA` present, no `SM`).
     A water-limited (`WLP`) variant feeds the Step-11 behavioral *gate* and needs PCSE
     to regenerate — deferred to Step 11 per the gate's scope. The standalone Step-7
     deliverable (flow + test + param + doc) needs no fixture.
- **dt-linearity preserved.** Both the PM daily rate and `f_water` are evaluated at the
  step-entry snapshot (forcing + `soil_water` amount), so `flux = daily·dt` is the
  increment form (the RK4 contract carries over; `scales_linearly_with_dt` holds). The
  `f_water` clamp is on a daily rate, not a dt-gate. Full-season `rationed == 0`
  (choosing `sw_critical ≫ sw_wilting` so the ramp is gentle) is a Step-11 *scenario*
  concern, like respiration's.
- **Clean-room (P5).** The PM equation is cited to Monteith (1965) / Penman (1948); the
  SVP/slope operational forms and standard psychrometric constants to FAO-56 (Allen et
  al. 1998, a public FAO document) and Tetens (1930); the water-stress form to primary
  soil-water literature. All crop VALUES (`r_a`, `r_s`) are honest provisional
  literature-typical placeholders (`TODO(cite)`), never backfilled from the unlicensed
  WOFOST YAML.

**Tasks.**
- `domains/biosphere/transpiration.py` (PURE stdlib): `saturation_vapor_pressure`,
  `slope_svp`, `penman_monteith_transpiration` (→ mm day⁻¹), `water_stress_factor`,
  the `TranspirationParams` dataclass (`r_a`, `r_s`), and the `Transpiration` +
  `Irrigation` flows.
- `params/transpiration.yaml` (structured `value/unit/source`: `r_a`, `r_s` in `s/m`)
  + `load_transpiration_params` in `loader.py` (bespoke schema like `_RespSchema`;
  exact-string unit guard; `r_a`, `r_s` > 0).

**Test plan (`tests/test_transpiration.py`).** `saturation_vapor_pressure` and
`slope_svp` against **FAO-56 published table values** (independent literals);
`penman_monteith_transpiration` at a pinned operating point (composed from the module
constants) and its `Rn=0, VPD=0 → 0` floor; `water_stress_factor` cardinal values
(0 at/below wilting, 1 at/above critical, linear midpoint) + the clamp; the assembled
`Transpiration` flow produces a **water-balanced** `FlowResult` (`assert_flow_balanced`)
with the hand-computed `PM · f_water · ground_area` daily kg leg, the `f_water → 0`
(dry-soil) shutoff, and `dt`-linear scaling; the `Irrigation` flow balances and tracks
the irrigation forcing; the committed `transpiration.yaml` loads to the expected
`TranspirationParams`; the loader rejects out-of-range/bad-unit params and a missing
`source` tag.

## Step 8 design — thermal-time phenology (degree-day literature)

*The aux accumulator's rate function — the **first real consumer of the Step-2
non-conserved aux channel** (P2). Designed just-in-time. Establishes thermal time
(°C·day) and the derived development stage `DVS = f(thermal_time)` that Step 9
(allocation) and the Step-6 maintenance ``maturity`` seam consume.*

### RESOLVED (2026-06-19) — the locks (advisor-reviewed)

- **It is an ``AuxProcess``, NOT a ``Flow`` and NOT a pure diagnostic — the first
  exercise of the Step-2 aux channel.** Thermal time *accumulates* with no balanced
  counterparty (it is not a conserved quantity), so it is neither a `Flow` (which
  would need a balanced leg, failing the conservation gate) nor a Step-4-style pure
  diagnostic (which is recomputed, not integrated). It is exactly the
  ``simcore.auxiliary.AuxProcess`` that P2 was built for: a new
  ``ThermalTimeAccumulation`` writes the **single** accumulator name ``thermal_time``
  (°C·day) in increment form ``{name: daily_thermal_time(T)·dt}``, advanced by one
  explicit-Euler evaluation at the step-entry snapshot and carried across RK4 stages
  (P2/P3). New pure module ``domains/biosphere/phenology.py`` (stdlib only).
- **DVS is derived, NOT stored (the P2 lock; resist the second-accumulator
  temptation).** ``development_stage(thermal_time, params)`` is a pure free function
  computed on demand by consumers (Step 9 allocation; the Step-6 ``maturity`` seam) —
  **not** a second accumulator. This is the phenology analogue of the Step-4 flag
  (LAI is derived, not an aux "for symmetry"); P2's "DVS = f(thermal_time)" names it
  explicitly. So the channel stays the **one** accumulator P2 minimized to.
- **Degree-day rate — cardinal-cap form (hand-checkable).**
  ``daily_thermal_time(temp_c, *, t_base, t_cap)`` (°C day⁻¹, i.e. °C·day per day):
  **0** at/below ``t_base``; the linear ``temp − t_base`` on ``(t_base, t_cap)``;
  capped at ``t_cap − t_base`` at/above ``t_cap``. The McMaster & Wilhelm (1997)
  growing-degree-day form **with an upper cap** (the WOFOST ``DTSMTB`` idiom — a
  piecewise-linear daily-temperature-sum response). Monotone non-decreasing in ``T``,
  bounded, and independently hand-computable (the Step-4/5/6/7 literal-test discipline).
- **DVS — two-phase TSUM scaling (the WOFOST ``TSUM1``/``TSUM2`` idiom).**
  ``development_stage(thermal_time, *, tsum_anthesis, tsum_maturity)``: on
  ``[0, tsum_anthesis]`` the vegetative ramp ``DVS = tt / tsum_anthesis ∈ [0, 1]``
  (emergence → anthesis); beyond, the reproductive ramp
  ``DVS = 1 + (tt − tsum_anthesis) / tsum_maturity``, **capped at 2.0** (anthesis →
  maturity). Stage points: **DVS = 0** emergence, **1** anthesis/flowering, **2**
  maturity. The accumulator starts at emergence (``tt = 0 ⇒ DVS = 0``); the
  sowing→emergence sub-phase (``TSUMEM``) and *when the accumulator starts/resets* are
  **scenario** concerns deferred to the Step-11 season assembly, not standalone-Step-8
  physics. Cite the DVS/TSUM **concept** to published crop-model literature (van Keulen
  & Wolf 1986; WOFOST methodology papers) — **never** the unlicensed WOFOST YAML;
  values stay ``TODO(cite)`` provisional.
- **WOFOST-equivalence (recorded so the Step-11 oracle match is auditable).** A *raw
  thermal-time accumulator* + a *derived piecewise DVS* is mathematically **equivalent**
  to WOFOST's phase-wise DVS integration **only because the base/cap response
  (``DTSMTB``) is phase-invariant** — the same daily °C·day rate feeds both phases, just
  normalized by ``TSUM1`` vs ``TSUM2``. This pre-empts the "why not integrate DVS
  directly like WOFOST" question: with a phase-invariant rate the two formulations
  coincide, and the single-accumulator form is the cleaner one.
- **Deferred seams — vernalization and photoperiod are STRUCTURALLY DIFFERENT
  (the load-bearing split; advisor-flagged, recorded like the Steps-5/6/7 deviations).**
  Winter wheat genuinely needs both — without them thermal time accrues through a mild
  winter and development runs far too fast (the Step-11 oracle gate would catch it).
  Standalone Step 8 is plain degree-day; both are documented Step-11 refinements with
  the seam left in place. But they are **not the same kind of deferral**:
  1. **Photoperiod** is a pure function of latitude + day-of-year (astronomical),
     read via ``env.get`` — a development-rate-modifying factor with **no accumulator**.
     Clean to add later as a multiplier on ``daily_thermal_time`` (the FvCB-``f_temp``
     shape), no state ripple.
  2. **Vernalization** (the cold requirement) is WOFOST-style a **second state
     accumulator** (vernalization-days), with a derived ``VERNFAC ∈ [0, 1]`` that
     down-scales the development rate **only in the vegetative phase**. A second
     accumulator rubs against P2's "essentially **one** accumulator" (line 104) — but
     P2 says *essentially* one and names the channel "non-conserved scalar
     accumulator**s**" (plural): "one" is the Phase-1 **estimate**, not a hard cap. A
     Step-11 vernalization-days accumulator is therefore an **extension** of the
     channel, not a violation of it — exactly the kind of growth P2's parallel-channel
     design anticipated.
  - **The single-accumulator/derived-DVS design *composes* with the deferred
    refinement (why the deferral is principled, not merely postponed).** A future
    vernalization-aware rate reads the snapshot ``evaluate`` already receives
    (``evaluate(snapshot, env, dt)``): it can read ``snapshot.aux["thermal_time"]``,
    derive the current DVS, and gate ``VERNFAC`` to the vegetative phase — so the
    seam exists structurally. Standalone ``ThermalTimeAccumulation.evaluate`` does
    **not** read ``snapshot`` (the rate depends only on forced temperature), but the
    signature carries it, so the refinement slots in without an API change.
- **Standalone — no existing flow is modified (the Step-7 ``f_water`` precedent).**
  DVS *drives* allocation (Step 9) and the maintenance ``maturity`` down-scaling
  (Step 6's documented seam), wired at those consumer steps. Step 8 delivers the
  accumulator process + the ``development_stage`` helper + tests + param + doc; like
  Step 7's ``water_stress_factor`` it ships its headline deliverable
  (``development_stage``) validated by literals, not yet by a consumer — that is the
  established standalone-first rhythm (roadmap line 232), not speculation.
- **Forcing via ``env.get`` (#16).** Air temperature is read as a scalar driver
  (``temp_var``, the same idiom as FvCB/transpiration; daily-mean temperature at the
  daily step). Increment form ``{thermal_time: daily_thermal_time(T)·dt}`` is dt-linear
  (the rate is dt-independent), advanced once per step at the step-entry snapshot and
  carried unchanged across RK4 stages (P2/P3) — so a flow reading ``thermal_time``
  would see a within-step constant.
- **``"degC*day"`` is the first exact-string-guarded unit that genuinely will NOT
  pint-parse (deliberate — recorded like the Step-4 pint-notation correction).** Offset
  units (``degC``) cannot be multiplied in pint, so ``degC*day`` raises in
  ``config.convert``. That is fine and intentional: the thermal-time params are
  validated by the ``_resp_value``-style **exact-string guard** (pure string equality,
  never invoking pint — exactly as ``"degC"``/``"1/day"``/``"dimensionless"`` already
  are), **not** routed through ``config.convert``. Recorded so a later reader does not
  "fix" it into a pint conversion. The cardinal temps carry ``"degC"``; the TSUM sums
  carry ``"degC*day"``.
- **Clean-room (P5).** The degree-day law is cited to McMaster & Wilhelm (1997); the
  DVS/TSUM development-stage concept to crop-model literature (van Keulen & Wolf 1986;
  WOFOST methodology). All numeric VALUES (``t_base``, ``t_cap``, ``tsum_anthesis``,
  ``tsum_maturity``) are honest **provisional literature-typical placeholders**
  (``TODO(cite)``), pending the Step-11 validation gate — never fabricated to a
  recalled citation, never backfilled from the unlicensed WOFOST YAML.

**Tasks.**
- ``domains/biosphere/phenology.py`` (PURE stdlib): ``daily_thermal_time`` (the
  cardinal-cap degree-day rate), ``development_stage`` (the two-phase DVS), the
  ``PhenologyParams`` dataclass, and the ``ThermalTimeAccumulation`` ``AuxProcess``.
- ``params/phenology.yaml`` (structured ``value/unit/source``: ``t_base``/``t_cap`` in
  ``degC``, ``tsum_anthesis``/``tsum_maturity`` in ``degC*day``) +
  ``load_phenology_params`` in ``loader.py`` (bespoke schema like ``_TranspSchema``;
  exact-string unit guard; ``t_base < t_cap``; ``tsum_* > 0``; ``t_base`` sign
  unconstrained — wheat's development base is ≈ 0 °C).

**Test plan (`tests/test_phenology.py`).** ``daily_thermal_time`` against **independent
hand-computed literals** (below base → 0; mid-band linear; at/above cap →
``t_cap − t_base``) + monotonicity and the cap; ``development_stage`` cardinal points
(``DVS = 0`` at ``tt = 0``; ``= 1`` at ``tt = tsum_anthesis``; ``= 2`` at
``tt = tsum_anthesis + tsum_maturity``) + the linear midpoints and the 2.0 cap;
``ThermalTimeAccumulation`` produces the increment-form ``{thermal_time: rate·dt}``,
reads temperature through ``env.get`` (#16), and is a ``runtime_checkable``
``AuxProcess``; integrated through the ``EulerIntegrator`` a constant-temperature
season accumulates to ``daily_thermal_time(T)·n·dt`` (the ``test_aux`` precedent); the
committed ``phenology.yaml`` loads to the expected ``PhenologyParams``; the loader
rejects an out-of-range value (``t_base ≥ t_cap``, non-positive ``tsum``), a bad unit
string, and a missing ``source`` tag (clean-room discipline at the boundary).

## Step 9 design — leaf/stem/root biomass allocation + senescence (DVS-keyed)

*The first **internal-redistribution** CARBON process and the first **multi-organ**
stock structure, designed just-in-time. Consumes the Step-8 derived ``DVS`` (allocation
fractions) and the Step-5/6 carbon budget (the net structural increment). Establishes
the leaf/stem/root organ pools the Step-11 season grows and the litter boundary sink
the season sheds into.*

### RESOLVED (2026-06-19) — the locks (advisor-reviewed)

- **Two new pure flows in one module ``domains/biosphere/allocation.py`` (stdlib only):
  ``Allocation`` (the multi-leg redistribution) and ``Senescence`` (the litter loss) —
  the respiration.py idiom (two related flows, one module).** Roadmap line 232: flow +
  unit test + param file + doc, **standalone before integration**. Per the Step-7/8
  precedent, **no committed flow is modified** here — except one *non-behavioral*
  extraction (below) — and the Step-5/6 rewiring is **deferred to Step 11 with an
  explicit checklist written down now** (the load-bearing mandate; see "the transition").
- **``Allocation`` is ONE multi-leg CARBON flow, recompute-DMI (the ``GrowthRespiration``
  pattern).** Source ``plant_c``, sinks ``leaf_c``/``stem_c``/``root_c``; it **recomputes**
  the daily structural increment ``DMI = Yg · max(0, GASS − MRES)`` from the step-entry
  snapshot (flows cannot read each other's results pre-arbitration), exactly as
  ``GrowthRespiration`` recomputes GASS/MRES. One flow, not three: a 3-organ multi-leg
  flow balances trivially (Σ legs = 0) and recomputes DMI **once**; three flows would
  triple the recompute and the field list. The long field list (photosynthesis + canopy +
  respiration + phenology params, ``ground_area``, the forcing-var names, the aux name,
  and the four organ/buffer stock ids) is the **inherent cost** of a flux-and-state-coupled
  quantity in an independent-flow engine — precedented by ``GrowthRespiration``.
- **Shared ``available_for_growth = max(0, GASS − MRES)`` helper — the one extraction
  (a non-behavioral refactor of ``respiration.py``).** ``growth_respiration_flux`` owns
  that expression today; if ``Allocation`` recomputed it independently the two would be a
  **3-way budget-drift hazard** (assimilation/growth-resp/allocation must agree on the same
  carbon budget — the ``ASRC = GPHOT − MRES`` invariant the Step-11 oracle budgets to).
  Extract ``available_for_growth(gross, maintenance) -> float`` in ``respiration.py``;
  ``growth_respiration_flux`` becomes ``(1 − Yg)·available_for_growth(...)`` (identical
  behavior — Step-6 tests stay green) and ``Allocation`` computes
  ``DMI = Yg·available_for_growth(...)`` — agreement by construction. Then net
  ``plant_c`` across the four carbon flows is ``GASS − MRES − GRES − DMI = 0`` when
  ``GASS ≥ MRES`` (the buffer passes growth through), confirming the budget closes.
- **Pure split functions, hand-checkable with a *given* DMI.** ``partition_fractions(dvs,
  table) -> (FL, FS, FR)`` (the DVS-keyed interpolation) and ``partition(dmi, dvs, table)
  -> (leaf, stem, root)`` are tested against independent literals with a supplied ``dmi``;
  the flow recomputes ``dmi`` and calls them (a ``Flow`` cannot take ``dmi`` as an arg —
  which is *why* the pure fn is tested standalone and the flow recomputes).
- **DISCRIMINATING CONSTRAINT — fractions must sum to 1 *under interpolation*, or the
  every-step conservation gate HARD-FAILS every step (a crash, not a silent drift).** If
  ``FL + FS + FR ≠ 1`` at the evaluated DVS, the organ legs don't sum to ``DMI`` and the
  flow doesn't balance. Designed out:
  - **A single DVS-keyed table of ``(dvs, fl, fs, fr)`` rows — NOT three independent
    FL/FS/FR tables.** Independent tables with different breakpoints interpolate to sums
    ≠ 1 *between* knots even when each knot sums to 1; a single shared-breakpoint table is
    sum-1 **everywhere** by linearity (``lerp(1, 1) = 1``).
  - **Loader enforces** (the one genuinely new schema shape — a *list of rows*, not the
    flat scalar ``value/unit/source`` entries): each row sums to 1 within tol, DVS knots
    strictly increasing, every fraction ∈ [0, 1], ``source`` present. ``dvs`` clamps to
    ``[rows[0].dvs, rows[-1].dvs]`` (a flat extrapolation outside the table).
  - **Belt-and-suspenders:** the flow sets the ``plant_c`` leg = ``−Σ(organ legs)`` so it
    **balances by construction** regardless; the loader sum-check enforces the *semantics*
    (drain exactly ``DMI``, not more or less).
- **THE TRANSITION — the Step-11 rewiring checklist (written down now; the gate will NOT
  catch these — they are physics, not balance, errors).** Once ``Allocation`` drains
  ``DMI`` from ``plant_c``, in the *integrated* model ``plant_c`` reframes from "the
  biomass" to a near-zero **labile carbohydrate buffer** (``+GASS −MRES −GRES −DMI = 0``
  per step). The checklist is **per-read, NOT per-flow** — every flow that *recomputes* a
  shared quantity must read the *same* stock, or the agreement-by-construction breaks
  silently (the budget stops telescoping; ``plant_c`` stops netting to 0):
  1. **Every GASS/LAI recompute reads ``leaf_c`` for leaf carbon** — ``GrossAssimilation``,
     ``GrowthRespiration``, **and** ``Allocation`` — else LAI ≈ 0 (the buffer is ~empty).
  2. **Every MRES recompute reads ``Σ(leaf_c + stem_c + root_c)`` for biomass** —
     ``MaintenanceRespiration``, ``GrowthRespiration``, **and** ``Allocation`` — else
     maintenance ≈ 0.
  - **The trap (why per-read):** ``GrowthRespiration.evaluate`` reads ``plant_c`` *once*
    into ``leaf_carbon`` and feeds it to **both** its LAI **and** its MRES recompute. A
    per-flow checklist tempts "fix the LAI read, done" — but its MRES read must switch to
    ``Σ`` organs too, or ``GrowthRespiration`` (GRES on the buffer's ~0 maintenance) and
    ``Allocation`` (DMI on ``Σ``-organ maintenance) no longer share
    ``available_for_growth``, GRES/DMI stop telescoping, and the shared-helper property
    (pinned by ``test_allocation_dmi_agrees_with_step6_growth_resp_budget``) dies.
  3. **When ``f_water``/``f_N`` land (Steps 7/10 → wired at 11), the ``limitation=``
     factor must be applied identically across all three GASS recomputes** — same
     divergence hazard if one flow limits and another does not.
  (Optional ``plant_c`` → ``labile_c`` rename is a Step-11 nicety, not now.) **Standalone
  ``Allocation`` already reads the post-transition-correct stocks** — ``leaf_c`` for its
  GASS/LAI recompute and ``Σ`` organs for its MRES recompute — so Step 11 only re-points
  the *other* flows to agree with it; allocation needs no change at integration.
- **``Senescence`` — a separate multi-leg flow ``{leaf_c, stem_c, root_c} → litter_sink``,
  relative death rate ∝ organ carbon.** ``senescence_flux(organ_c, rdr) = rdr · organ_c``
  (mol C day⁻¹) per organ; → 0 as the organ → 0, so positivity is **structural** (the
  Step-5/6/7 self-limiting pattern). ``litter_sink`` is a **BOUNDARY** sink **distinct
  from the numerical extinction loss-sink** (decision #6) — real shed biomass, not a
  rounding residual; Phase-2 litter/decomposition dynamics consume it later. DVS / leaf-age
  / self-shading keying of the death rate is a documented **Step-11 seam** (standalone is a
  plain per-organ constant relative rate), so Step 11 is a coefficient/keying change, not a
  structural one — the Step-6 ``maturity``-seam precedent.
- **Organ stocks: POPULATION, CARBON.** ``leaf_c``/``stem_c``/``root_c`` are absorbing-
  eligible biomass (extinction-eligible, decision #6); ``plant_c`` stays the
  (POPULATION) buffer; ``litter_sink`` is BOUNDARY. Standalone tests construct them
  ad hoc (the scenario wiring — initial organ amounts, the season's stock set — is
  Step 11). Each flow is internally balanced in CARBON (P1): ``Allocation`` is an
  internal redistribution (Σ legs = 0), ``Senescence`` is organ → boundary (Σ = 0).
- **Grain / storage organ deferred — flagged, NOT built (it blocks Step 11, not Step 9).**
  The title is leaf/stem/root; the table is structured so a **4th fraction ``FO``**
  (storage organ, the reproductive-phase sink at DVS > 1) is an **additive** column.
  **But this is not free at validation:** the committed oracle fixture has
  ``TWSO ≈ 11.5`` of ``TAGP ≈ 20.4 t/ha`` — over half the above-ground biomass is grain —
  so a 3-organ model **cannot** match the biomass curve at the Step-11 behavioral gate.
  Adding ``FO`` + a ``storage_c`` pool is therefore a **Step-11 precondition**, surfaced
  here so it is not a surprise.
- **Clean-room (P5).** The DVS-keyed partitioning concept is cited to crop-model
  literature (Penning de Vries et al. 1989; van Keulen & Wolf 1986; the WOFOST
  ``FLTB``/``FSTB``/``FRTB`` idiom); the relative-death-rate senescence concept likewise.
  All numeric VALUES (the partition fractions, the per-organ death rates) are honest
  **provisional literature-typical placeholders** (``TODO(cite)``), pending the Step-11
  validation gate — never fabricated to a recalled citation, never backfilled from the
  unlicensed WOFOST YAML.

**Tasks.**
- ``domains/biosphere/allocation.py`` (PURE stdlib): ``PartitionRow`` + ``AllocationParams``
  (the table) and ``SenescenceParams`` (per-organ rates) dataclasses; ``partition_fractions``
  + ``partition`` (the DVS-keyed split); ``senescence_flux``; the ``Allocation`` (recompute-DMI,
  multi-leg) and ``Senescence`` (multi-leg) flows.
- Extract ``available_for_growth`` in ``respiration.py`` (non-behavioral; ``growth_respiration_flux``
  delegates to it) and import it in ``allocation.py``.
- ``params/allocation.yaml`` (the row-table shape) + ``params/senescence.yaml`` (per-organ
  ``1/day`` rates) + ``load_allocation_params`` / ``load_senescence_params`` in ``loader.py``
  (the new row-table schema with the sum-1 / increasing-DVS / bound checks; the exact-string
  ``1/day`` guard for senescence, like ``_RespSchema``).

**Test plan (`tests/test_allocation.py`).** ``partition_fractions`` at the table knots and a
midpoint against **independent literals**, the sum-to-1 invariant across a DVS sweep, and the
out-of-range clamp; ``partition`` splits a given DMI exactly; ``senescence_flux`` ∝ organ
carbon and → 0 at zero; the assembled ``Allocation`` flow produces a **carbon-balanced** 4-leg
``FlowResult`` (``assert_flow_balanced``) whose organ legs equal the hand-computed
``DMI·fractions`` (DMI recomposed via the Step-4/5/6 stack + ``available_for_growth``) and whose
``plant_c`` leg is ``−ΣDMI``, the dark-day ``GASS < MRES`` → ``DMI = 0`` clamp, DVS read from
``snapshot.aux["thermal_time"]``, and ``dt``-linear scaling; the assembled ``Senescence`` flow
produces a carbon-balanced ``{organs → litter}`` ``FlowResult`` with the hand-computed legs and
``dt``-linear scaling; the committed ``allocation.yaml``/``senescence.yaml`` load to the expected
params; the loaders reject a non-sum-1 row, non-increasing DVS, an out-of-range fraction / death
rate, a bad unit, and a missing ``source`` tag (clean-room discipline at the boundary).

## Step 10 design — nitrogen uptake + limitation (the NITROGEN currency)

*The last of the seven processes, designed just-in-time. The **NITROGEN-currency**
analog of Step 7 (water): a depletable soil pool drained by a self-limiting uptake flow
and refilled by a supply flow, plus the ``f_N`` stress factor that Step 11 wires into
the photosynthesis ``Π fᵢ`` seam. Supplies the last populated limiter.*

### RESOLVED (2026-06-19) — the locks (advisor-reviewed)

- **Structural mirror of Step 7 — two flows + a delivered-but-unwired stress factor,
  in one module ``domains/biosphere/nitrogen.py`` (stdlib only).** Roadmap line 232:
  flow + unit test + param file + doc, **standalone before integration**. No committed
  flow is modified; the ``f_N`` wiring into photosynthesis is **deferred to Step 11**
  (the ``limitation=`` seam — exactly as Step 7 delivered ``water_stress_factor``
  without wiring ``f_water`` into ``GrossAssimilation``).
  - **``NitrogenUptake``** — NITROGEN flow ``soil_n (POOL) → plant_n (POOL)``, balanced
    in N. ``potential = max_uptake_capacity[kg N m⁻² day⁻¹] · ground_area ·
    soil_n_availability(soil_n)`` — self-limited as ``soil_n`` depletes (structural
    positivity, P3; the WATER analog of transpiration's ``water_stress_factor``).
  - **``Fertilization``** — NITROGEN flow ``n_source (BOUNDARY) → soil_n (POOL)``, a
    scheduled supply (kg N m⁻² day⁻¹ forcing → kg day⁻¹ × ``ground_area``) — the
    ``Irrigation`` mirror that refills the depleting pool so the season's N balance
    closes (#13).
  - **``nitrogen_stress_factor`` (= ``f_N``)** — a pure function reading the **plant's
    own** N status (a direct snapshot read, decision #16), delivered + unit-tested
    standalone, **not yet a flow input**.
- **Two DISTINCT factor functions (the genuine new structure vs Step 7).** Step 7's
  ``water_stress_factor`` did double duty (it limited transpiration *and* was
  ``f_water``). Here the two roles split because they read **different stocks**:
  - ``soil_n_availability(soil_n, *, sn_residual, sn_critical)`` limits **uptake**
    (supply side) — reads ``soil_n``; linear ramp 0→1 over ``[sn_residual,
    sn_critical]`` (the ``water_stress_factor`` shape). Its thresholds are
    **scenario/soil data — call-args like ``sw_wilting``/``sw_critical``**, not crop
    params.
  - ``nitrogen_stress_factor(plant_n, biomass_c, *, n_residual_per_mol_c,
    n_critical_per_mol_c)`` = ``f_N`` limits **photosynthesis** (plant status) — reads
    ``plant_n`` + biomass. Linear ramp on the **concentration** ``plant_n / biomass_c``
    (kg N / mol C). **Guard ``biomass_c <= 0`` → return 1.0** (neutral; photosynthesis
    is already 0 at LAI=0 — never ``== 0``).
- **Uptake is a max *capacity* gated by availability, NOT plant demand (the fixed-flux
  lock).** ``max_uptake_capacity`` ignores plant need **by construction**; N-limitation
  arises by **dilution** (biomass outgrows the fixed N supply → concentration falls →
  ``f_N`` drops) and by **soil depletion** (``soil_n_availability`` → 0). The discriminating
  reason to fix the flux now (over WOFOST's demand-deficit ``target_conc·biomass −
  plant_n``) is **coupling surface**: fixed-flux keeps ``NitrogenUptake`` reading **only
  ``soil_n``**, entirely out of the biomass-read consistency web that the transition
  checklist below exists to manage. Demand-deficit is a **strictly additive Step-11
  seam** — a changed ``potential`` formula + new reads inside ``evaluate`` (the
  ``maturity``-seam shape), introduced where the web is already being managed.
- **``f_N`` concentration in native currency units — the carbon-fraction fold lives at
  the loader (the ``sla_per_mol_c`` precedent).** ``plant_n`` is kg N; biomass is mol C;
  leaf-N concentration is conventionally kg N / kg DM. Rather than the pure core holding
  ``M_C``/``carbon_fraction``, the loader pre-converts the residual/critical thresholds
  ``kg N/kg DM → kg N/mol C`` via ``× M_C / carbon_fraction`` (identical in form to
  ``sla_per_mol_c = sla · M_C / carbon_fraction``). The core function compares
  ``plant_n / biomass_c`` against plain-float thresholds. So ``nitrogen.yaml`` carries a
  ``carbon_fraction`` entry — see the consistency requirement in the checklist.
- **Stocks: ``soil_n`` POOL, ``plant_n`` POOL, ``n_source`` BOUNDARY (all NITROGEN).**
  ``plant_n`` is a **POOL** — an N substance reservoir, **never zeroed-with-loss** (the
  POOL invariant), not extinction-eligible biomass. Each flow is internally balanced in
  NITROGEN (P1): uptake ``soil_n → plant_n`` (Σ legs = 0), fertilization
  ``n_source → soil_n`` (Σ = 0). Standalone tests construct the stocks ad hoc; the
  scenario wiring (initial amounts, the season's stock set) is Step 11.
- **Documented seams (NOT built — the established rhythm: simplest citable core now,
  WOFOST elaboration deferred).**
  - **Demand-deficit uptake** (WOFOST NDEMTO) — the fixed-flux refinement above.
  - **N translocation / orphaning on extinction.** ``plant_n`` POOL ⇒ if Step-11 organs
    go extinct (biomass → 0 with loss), their N stays orphaned in ``plant_n``. The same
    seam as senescence-N-translocation; senescence (Step 9) stays **carbon-only** in
    Phase 1. Noted, not built.
  - **Whole-plant N concentration, not leaf-specific.** One ``plant_n`` pool ⇒ ``f_N``
    is whole-plant; leaf-specific N would need per-organ N pools (deferred).
  - **Temperature / root-density limitation of uptake** — availability-only here.
- **Clean-room (P5).** The N-stress / critical-N-dilution concept is cited to crop-N
  literature (the WOFOST ``NMINSO``/``NMAXLV`` / critical-N-curve idiom; Greenwood et al.
  1990 critical-N dilution). All numeric VALUES (``max_uptake_capacity``, the N-concentration
  thresholds, ``carbon_fraction``) are honest **provisional literature-typical placeholders**
  (``TODO(cite)``), pending the Step-11 validation gate — never fabricated to a recalled
  citation, never backfilled from the unlicensed WOFOST YAML.

### THE TRANSITION — Step-10 additions to the Step-11 rewiring checklist

*Now that ``f_N``'s exact reads are known, the Step-9 checklist item 3 ("apply
``limitation=`` identically across all GASS recomputes") is concretized. All three below
are **physics-not-balance** errors — the every-step conservation gate will **not** catch
them (they shift magnitudes, not balance). They land in the written design now, while
``f_N``'s reads are in hand:*

1. **``f_N`` must enter ALL THREE GASS recomputes** — ``GrossAssimilation``,
   ``GrowthRespiration``, **and** ``Allocation`` — folded into the same ``limitation=``
   factor as ``f_water`` (Step 7). If it limits the deposit but not the GRES/DMI
   recomputes, the carbon budget stops telescoping and ``plant_c`` stops netting to 0
   (the drift ``test_allocation_dmi_agrees_with_step6_growth_resp_budget`` pins).
2. **``f_N``'s biomass denominator must be the *same* ``Σ(leaf_c + stem_c + root_c)``
   that MRES reads** — ``f_N`` is a **fourth consumer** of that exact biomass read
   (alongside ``MaintenanceRespiration``, ``GrowthRespiration``, ``Allocation`` — Step-9
   checklist item 2), not a new one. Step 11 points all four at one expression.
3. **``carbon_fraction`` in ``nitrogen.yaml`` MUST equal ``canopy.yaml``'s.** Both fold
   it (``sla → m²/mol C``; N-thresholds → ``kg N/mol C``); divergent values =
   a silently inconsistent plant. The Step-4 note defers the *dedup* of the duplicated
   entry; the **consistency requirement** is real now — a Step-11 assertion line, not an
   optional nicety.

**Tasks.**
- ``domains/biosphere/nitrogen.py`` (PURE stdlib): ``NitrogenParams``
  (``max_uptake_capacity`` + the two ``*_per_mol_c`` concentration thresholds, loader-folded);
  ``soil_n_availability`` + ``nitrogen_stress_factor`` (the two ramp functions);
  the ``NitrogenUptake`` (potential × availability) and ``Fertilization`` (scheduled supply)
  flows.
- ``params/nitrogen.yaml`` (value/unit/source: ``max_uptake_capacity`` in ``kg/m^2/day``,
  ``n_residual``/``n_critical`` in ``kg/kg`` = kg N/kg DM, ``carbon_fraction`` in
  ``dimensionless``) + ``load_nitrogen_params`` in ``loader.py`` (exact-string unit guards
  like ``_RespSchema``; fold the kg N/kg DM → kg N/mol C conversion via ``M_C/carbon_fraction``;
  bound checks: positive capacity, ``carbon_fraction ∈ (0, 1]``, ``n_residual < n_critical``).

**Test plan (`tests/test_nitrogen.py`).** ``soil_n_availability`` against independent
literals — the band limits (≤ residual → 0; ≥ critical → 1) and a midpoint ramp, and the
``sn_residual < sn_critical`` guard; ``nitrogen_stress_factor`` likewise on the
concentration, plus the ``biomass_c <= 0 → 1.0`` guard and the ``n_residual_per_mol_c <
n_critical_per_mol_c`` guard; the assembled ``NitrogenUptake`` flow produces a
**N-balanced** 2-leg ``FlowResult`` (``assert_flow_balanced``) equal to ``potential ·
availability · ground_area · dt`` with the ``plant_n`` leg ``+`` and ``soil_n`` leg ``−``,
the depleted-soil ``availability → 0`` shutoff, and ``dt``-linear scaling; the assembled
``Fertilization`` flow produces a balanced ``{n_source → soil_n}`` result with
``dt``-linear scaling; the committed ``nitrogen.yaml`` loads to the expected params with
the carbon-fraction fold applied (a known value round-trips); the loader rejects a bad
unit, an out-of-range ``carbon_fraction``, ``n_residual ≥ n_critical``, a non-positive
capacity, and a missing ``source`` tag (clean-room discipline at the boundary).

## Step 11 design — integration + behavioral validation (the capstone)

*Assembles the seven standalone processes into one single-producer season, applies the
Step-9/10 transition (rewiring), and validates. Tightest-constraint-first: the binding
constraint is the **always-on conservation + ``rationed == 0`` gate over the full
season**, NOT the oracle match. Built in ~6 commits in that order. No new core
machinery — Phase 1 is additive to the frozen Phase-0/0.5 surface (only ``State.aux``,
already landed at Step 2).*

### RESOLVED (2026-06-19) — the locks (advisor-reviewed; arbitration-grounded)

- **THE LOAD-BEARING DEVIATION — the ``plant_c`` "labile buffer" is DISSOLVED; every
  carbon fate is sourced from the unclamped ``co2_source`` boundary + the organ pools.**
  The Step-9 transition prose ("``plant_c`` reframes to a near-zero labile buffer; net
  ``+GASS −MRES −GRES −DMI = 0`` per step; Step 11 only re-points reads") was written
  **without checking it against arbitration semantics, and it does not survive contact.**
  The arbitration backstop computes ``scale_s = stocks[s].amount / demand_s`` against the
  **start-of-step** amount — "withdrawals never draw against same-step inflows"
  (``simcore/arbitration.py``). So a pool that nets to ~0 each step **cannot source any
  same-step withdrawal**: ``GASS``'s same-step deposit into the buffer does not count
  toward covering that step's ``MRES``/``GRES``/``DMI`` draws, so the backstop would fire
  **every step** (not merely in winter — the advisor's flag-1 generalized). The Step-9
  standalone tests passed only because they construct ``plant_c`` with an ad-hoc healthy
  amount. A standing-reserve workaround (init ``plant_c`` large) only **postpones** firing
  until an overwintering deficit exhausts the reserve — a design that *can* fire the
  backstop violates "the backstop is the rare numerical guard, never the mechanism"
  (decision #3) and the exit-criterion golden. **Rejected.** The correction:
  - **No ``plant_c`` stock.** Gross assimilation ``GASS`` becomes a **recomputed
    quantity** (a number computed inside ``evaluate`` from ``leaf_c`` + forcing), not a
    flow that deposits into a pool. The flow ``GrossAssimilation`` **dissolves**.
  - **``Allocation``**: ``co2_source -> {leaf_c, stem_c, root_c, storage_c}`` — deposits
    the structural increment ``DMI`` split by the DVS-keyed fractions. Source is the
    unclamped CO₂ boundary (skipped by arbitration), sinks are organs.
  - **``GrowthRespiration``**: ``co2_source -> co2_sink`` — the growth-conversion loss
    ``GRES = (1−Yg)·available_for_growth(GASS, MRES)``, assimilated-and-immediately-
    respired (never becomes biomass; correct booking).
  - **``MaintenanceRespiration``**: ``{co2_source(covered), organs(shortfall)} -> co2_sink``
    — see the maintenance-first split below.
  - **``Senescence``**: ``{leaf_c, stem_c, root_c} -> litter_sink`` (unchanged; storage
    excluded — see FO below).
  Every **clamped** carbon withdrawal is now from an organ pool (∝ that organ's amount,
  self-limiting → 0 as the organ → 0) or from the unclamped boundary — so the carbon-side
  backstop is **structurally impossible**, not merely tuned away. ``rationed == 0`` holds
  **by construction**. This is a RESOLVED deviation, documented like Step 7's 3→2 collapse
  and the ``auxiliary.py`` rename.

- **Maintenance-first split (the no-double-count lock).** ``MaintenanceRespiration``
  recomputes ``GASS`` (so it knows how much of today's assimilate covers maintenance) and
  draws ``MRES = maintenance_respiration_flux(Σ organs, T)`` thus:
  - ``covered = min(GASS, MRES)`` from ``co2_source`` (the maintenance paid out of today's
    fixed carbon),
  - ``shortfall = max(0, MRES − GASS)`` from the organ pools, **proportional to each
    organ's share** of ``Σ(leaf + stem + root)`` (biomass shrinks in deficit — WOFOST-
    faithful; the documented "winter biomass decline" now lands on real biomass, not the
    phantom buffer). Guard ``Σ organs == 0 ⇒ MRES = 0`` (no leaves, no maintenance).
  - ``co2_sink`` leg ``= +MRES`` (``= covered + shortfall``, balanced).
  This **preserves** ``available_for_growth = max(0, GASS − MRES)`` shared by
  ``GrowthRespiration`` (``GRES = (1−Yg)·available``) and ``Allocation``
  (``DMI = Yg·available``), so the budget telescopes and
  ``test_allocation_dmi_agrees_with_step6_growth_resp_budget`` survives. **Rejected
  alternative:** "draw the full ``MRES`` from organs while ``available`` stays
  ``GASS − MRES``" — this **double-charges maintenance** (once inside ``available``, once
  on the organs) and silently slows biomass; the conservation gate does NOT catch it (it
  balances). The min/shortfall split is the only structure that both conserves and budgets
  correctly.
  - *Trace to verify before coding (both conserve; neither touches a clamped stock beyond
    self-limiting draws):* **surplus day** (``GASS ≥ MRES``): ``co2_source`` net ``= −GASS``
    (``−DMI −GRES −MRES``, and ``DMI + GRES + MRES = (GASS−MRES) + MRES = GASS``); organs
    grow by ``DMI``; ``co2_sink`` gains ``GRES + MRES``. **deficit day** (``GASS < MRES``):
    ``available = 0`` so ``GRES = DMI = 0``; ``co2_source`` net ``= −covered = −GASS``;
    organs shrink by ``shortfall = MRES − GASS``; ``co2_sink`` gains ``MRES``.

- **Three GASS-recompute sites (UPDATES the Step-9/10 checklist — ``GrossAssimilation`` is
  gone).** With the source flow dissolved, the flows that recompute ``GASS`` are
  **``GrowthRespiration``, ``Allocation``, and ``MaintenanceRespiration``** (the last is a
  *new* GASS site, forced by the maintenance split). The Step-9 checklist named
  ``GrossAssimilation`` as a site; that line is superseded. The per-read discipline holds:
  - every **LAI/GASS** recompute reads ``leaf_c`` for leaf carbon (all three sites);
  - every **MRES** recompute reads ``Σ(leaf_c + stem_c + root_c)`` for biomass (all three
    sites + ``f_N``'s denominator — a fourth consumer of that exact read);
  - ``f_water · f_N`` enters the **same** ``limitation=`` factor identically across all
    three GASS recomputes (Step-5 seam). If one site limits and another does not, the
    budget stops telescoping and biomass drifts — a physics error the gate will NOT catch.

- **The per-read field split on the committed flow classes (a real class change, not just
  wiring).** ``GrowthRespiration`` today carries one ``plant_c`` field used for **both**
  its leg source **and** its LAI/MRES reads. Re-sourcing forces splitting it: a
  ``co2_source`` field (leg source) **plus** ``leaf_c`` and the organ ids (reads). Same for
  the maintenance/allocation flows. Standalone tests gain fields; **their hand-computed
  numbers are preserved** (never weaken a test to make it pass — fix the wiring).

- **``f_water`` / ``f_N`` wiring (the limiter seams close).** ``f_water`` reads
  ``soil_water`` via ``env.get`` (the shared-stock #16 path — soil water is a *sibling*
  stock); ``f_N`` reads ``plant_n`` (a POOL) + ``Σ organs`` (a direct snapshot read — the
  plant's *own* state). The flow takes the two limiter stock ids / var names and forms
  ``limitation = f_water · f_N`` inside ``evaluate``, passed to
  ``daily_canopy_assimilation(limitation=…)``. **``carbon_fraction`` parity:**
  ``nitrogen.yaml``'s ``carbon_fraction`` MUST equal ``canopy.yaml``'s (both fold it —
  ``sla → m²/mol C``; N-thresholds → ``kg N/mol C``); a Step-11 assertion line pins it
  (the dedup of the duplicated entry stays deferred).

- **FO / storage organ (the Step-9 precondition — now built).** The committed oracle
  fixture has ``TWSO`` (storage/grain) as ~half of ``TAGP``, so a 3-organ model cannot
  reproduce the biomass split. ``PartitionRow`` gains a 4th fraction ``fo``; the row sum
  becomes ``fl + fs + fr + fo == 1`` (loader-enforced, shared-breakpoint table ⇒ sum-1
  everywhere); ``partition`` returns ``(leaf, stem, root, storage)``; ``Allocation`` gains
  a ``storage_c`` leg. **``storage_c`` is POPULATION CARBON** (extinction-eligible biomass,
  like the others) but is **excluded from maintenance and senescence in Phase 1** (grain is
  harvested, not shed; grain maintenance is small and a documented refinement seam) — so
  ``storage_c`` is a pure allocation sink, kept **out of** the ``Σ organs`` maintenance/
  ``f_N`` biomass read and out of the ``Senescence`` flow. This keeps the consistency web
  at ``leaf/stem/root``. The pre-anthesis table rows have ``fo = 0`` (storage fills only at
  ``DVS > 1``). Existing 3-fraction ``allocation.yaml`` + tests migrate to the 4-fraction
  shape (the FO column added; standalone literals recomputed by hand).

- **``rationed == 0`` whole-season proof (the enumeration IS the proof — state it).** Every
  clamped stock's every withdrawal self-limits, so the Euler backstop never fires:
  - **organ pools** (``leaf_c``/``stem_c``/``root_c``, POPULATION): drawn by ``Senescence``
    (``rdr · organ``) and the maintenance ``shortfall`` (∝ organ share) — both ∝ the
    organ's amount, → 0 as it → 0. ``storage_c``: only deposited into (no withdrawal).
  - **``soil_water``** (POOL): drawn by ``Transpiration`` (``× water_stress_factor`` → 0 at
    ``sw_wilting``) — Step 7, structural.
  - **``soil_n``** (POOL): drawn by ``NitrogenUptake`` (``× soil_n_availability`` → 0 at
    ``sn_residual``) — Step 10, structural.
  - **``co2_source`` / ``water_source`` / ``n_source``** (unclamped BOUNDARY sources):
    withdrawals **skipped** by arbitration (decision #13) — impose no constraint.
  - **``co2_sink`` / ``vapor_sink`` / ``litter_sink`` / loss-sinks** (BOUNDARY): only
    **receive** (positive legs) — no withdrawal demand, never rationed.
  Carbon was the only gap (water/N were built self-limiting at Steps 7/10); the buffer
  dissolution closes it. The well-fed bound (``coef · dt < 1`` per first-order draw) is a
  scenario-param check, like the demo's.

- **Oracle match = QUALITATIVE / loose smoke check; the tight quantitative gate is
  DEFERRED (user decision, 2026-06-19).** Params are honest ``TODO(cite)`` placeholders;
  clean-room forbids backfitting to WOFOST. So Step 11 ships the assembled season +
  conservation + golden as the hard deliverable, and the oracle comparison is a **shape +
  order-of-magnitude** check (e.g. biomass rises then plateaus, LAI is a unimodal hump, DVS
  is monotone to ~2, peak TAGP is the right power of ten), using the committed
  ``lab/oracle_match.py`` metrics with a **loose, documented band**. The observed gap is
  recorded as a **finding**, not tuned away. A tight quantitative gate (with literature-
  range calibration) is a deferred follow-up. **Calibration must never block the
  deterministic commits** (FO, rewiring, assembly, golden).
  - **Honest comparison needs the oracle's INPUTS, not just its outputs.** The fixture is
    outputs (DVS/LAI/TAGP/…). Extend ``tests/oracle/runner.py`` to dump the **daily
    NASAPower weather** PCSE consumed (temperature, radiation, vapour pressure / VPD inputs,
    plus latitude/day-of-year for daylength) as a committed fixture — weather is **facts**,
    license-clean (the same rule as the output fixture; no WOFOST param YAML is committed).
    Our season is driven by **that** weather through a table-backed ``Schedule`` so the
    comparison is against the same forcing. Our clean-room equations derive PAR / Rn / VPD /
    daylength / Ci from the raw weather (documented, cited conversions); the PP (potential-
    production) oracle variant means ``f_water = f_N = 1`` (non-limiting) for the first
    season — soil water/N thresholds are set so the limiters stay at 1, matching ``_PP``.

- **Time-varying forcing becomes real (P3).** The season uses a **table-backed
  ``Schedule``** (``schedule(n, dt) = weather_table[n]``) — the first genuinely
  ``n``-dependent forcing (Phase-0/0.5 were autonomous/constant). ``Euler`` is selected
  (P3: Euler-daily biology; ``dt = 1 day``); RK4 stays available for the engine gates but
  the crop scenario documents why it picks Euler.

- **Golden + frozen-surface notes.** A committed **hex-float** snapshot pins the season
  bit-exactly (determinism #7 + registration/aux-order independence). The frozen-surface
  notes and the exit-criteria checkboxes below are updated; ``MEMORY.md`` + a memory file
  record the buffer-dissolution deviation so it is not re-litigated.

- **CO₂ is TWO stocks: ``co2_atmos`` (unclamped BOUNDARY source) + ``co2_resp`` (BOUNDARY
  sink) — supersedes the Step-6 "same reservoir" note (advisor-reconciled).** The Step-6
  ``respiration.py`` docstring chose a *single* ``boundary.co2`` reservoir — but that was
  correct only **because respiration's source was then ``plant_c``** (``plant_c → co2``
  moved real carbon). The buffer dissolution makes the source ``co2`` itself, so a single
  stock would make ``GrowthRespiration`` (``co2 → co2``) a **permanent no-op** and the
  covered-maintenance a wash — i.e. respiration would become a *silent* ``Yg`` factor, the
  very thing plan line 348 forbids ("a balanced carbon flow to atmosphere/boundary, not a
  silent factor"). The premise that justified one reservoir is exactly what dissolution
  removed, so it does not carry forward. Two stocks keep every flow real and balanced:
  - ``Allocation``: ``co2_atmos -> organs`` (DMI). ``GrowthRespiration``:
    ``co2_atmos -> co2_resp`` (GRES). ``MaintenanceRespiration``:
    ``{co2_atmos(covered), organs(shortfall)} -> co2_resp`` (MRES).
  - ``co2_atmos`` is the unclamped atmosphere (CO₂ non-limiting, P1 — the FvCB rate reads
    the ``ci_var`` *forcing*, never the stock amount). ``co2_resp`` is the carbon **mirror
    of the already-committed ``vapor_sink`` (transpiration) and ``litter_sink``
    (senescence)** — BOUNDARY sinks that accumulate and are read only in Phase 2; an
    unread-in-Phase-1 boundary sink is the **established discipline**, not anti-speculation
    debt. ``co2_resp`` strictly increasing is also a cheap liveness signal ("respiration
    happened"), complementing the organ-biomass liveness assertion.
  - **One-way door:** no Phase-1 deliverable distinguishes one stock from two (biomass /
    conservation / ``rationed == 0`` / golden / the biomass+water oracle match are
    bit-identical); the only difference is whether respiration is a visible flux or a wash.
    Decided here, sets the golden's stock set, not revisited in Phase 1.

- **Seedling bootstrap + the no-extinction guard (advisor-flagged; same root as liveness).**
  The season starts with **nonzero** ``leaf_c``/``stem_c``/``root_c`` (the oracle's day-0
  row has ``TWLV``/``TWST``/``TWRT`` > 0, ``TWSO`` = 0): a zero ``leaf_c`` ⇒ LAI = 0 ⇒
  GASS = 0 ⇒ the plant never grows (and the deterministic gates still pass — see liveness).
  Organ ``extinction_threshold = 0`` for the well-fed season (no spurious extinction from
  early senescence/maintenance snapping an organ below threshold), and the seedling is
  sized to grow.

- **Structural three-site agreement (advisor sharpening; commit 3).** Rather than *rely on
  discipline* for the three GASS sites to pass identical args, extract **one helper** that
  computes ``(GASS, MRES, available)`` from a (snapshot + params + ``limitation``) bundle;
  ``GrowthRespiration``, ``MaintenanceRespiration``, and ``Allocation`` all call it, so they
  **cannot drift** by passing different ``leaf_c``/``Σ organs``/``limitation``. Closes the
  MRES/limitation side the way ``daily_canopy_assimilation`` already closes the GASS side.

**Tasks (commit-sized, tightest-constraint-first).**
1. This design block (advisor-reviewed).
2. **FO/storage organ** — ``allocation.py`` 3→4 fractions + ``storage_c`` leg; loader sum-1
   over 4 fractions; ``allocation.yaml`` FO column; standalone tests green (hand literals).
3. **Carbon rewiring** — dissolve ``plant_c``; ``GrossAssimilation`` removed; re-source
   ``GrowthRespiration``/``MaintenanceRespiration``/``Allocation`` per the deviation; the
   per-read field split; ``f_water·f_N`` into ``limitation=`` across the three GASS sites;
   ``carbon_fraction`` parity assertion; standalone tests migrated (numbers preserved).
4. **Season assembly** — scenario module (all stocks + boundary reservoirs + loss-sinks,
   the four carbon flows + the two water + two N flows + the thermal-time aux, table-backed
   weather ``Schedule``); extend ``runner.py`` to dump the weather fixture; drive the full
   season; assert conservation + ``rationed == 0`` every step.
5. **Oracle smoke check** — loose qualitative match vs. the committed reference under the
   same weather; document the gap.
6. **Golden + docs** — hex-float season golden (determinism + order-independence); update
   frozen-surface notes, exit criteria, ``MEMORY.md``/docs; commit + push.

**Test plan.** Per-commit standalone tests stay green with hand-computed literals (FO split,
the maintenance covered/shortfall split traced by hand on a surplus and a deficit day, the
three-site ``limitation=`` parity); the assembled season conserves every step
(``assert_conserved``, always-on) with summed ``rationed == 0`` and no extinction events on
the well-fed scenario; a **liveness** assertion proves the plant actually grew (final organ
biomass > initial; LAI peaks above a floor; DVS advances past 1) — because conservation and
``rationed == 0`` both pass *trivially on a dead/null trajectory*, so liveness is what makes
"green" meaningful; the season is bit-identical under flow/aux registration shuffle and
matches the committed hex-float golden; the oracle smoke check passes the loose band and the
gap is recorded; the Phase-0/0.5 gates (analytic convergence/order, 100k stability, purity)
stay green — the biology is validated against the oracle, **not** by dt-convergence (P3).

---

## Exit criteria (Phase 1 — "research-grade single producer")

- [x] **Foundation locked:** science units + area basis (P4); the non-conserved aux
      channel (P2) — additive to frozen State, serialized, observed, outside the
      conservation gate; PCSE oracle harness behind a marker, clean-room param
      discipline (P5). *(Steps 1–3 — complete.)*
- [x] **Seven processes**, each shipped as **flow/aux-rate + unit test + param file
      + documentation** before integration (roadmap line 232): Beer–Lambert, FvCB,
      respiration, Penman–Monteith, phenology, allocation, nitrogen — all complete and
      committed. *(Steps 4–10.)* The Step-11 season **re-exercises** them assembled
      (the rewiring per the transition checklists); shipping each standalone is what
      this criterion gates.
- [x] **Single-currency + multiplicative coupling (P1)** throughout; **OXYGEN not
      tracked**; every flow internally balanced in its one currency; the every-step
      conservation gate holds over a full season with `rationed == 0` (self-limiting
      kinetics; Euler-daily, P3). *(Step 11 — `test_season`; `rationed == 0` is
      structural after the `plant_c` buffer dissolution.)*
- [~] **Behavioral oracle match (P5)** — **QUALITATIVE only; the quantitative gate is
      DEFERRED (user decision).** The assembled season reproduces the oracle's *shape*
      (DVS monotone 0→2, unimodal LAI, grain fills post-anthesis) under the same
      NASAPower weather (`test_oracle_smoke`); the **magnitude** runs ~2 orders below the
      oracle (uncalibrated `TODO(cite)` placeholders + the no-vernalization phenology
      overrun) — recorded as a documented finding, not a validated match. Closing the gap
      (literature-range calibration + vernalization) is a deferred follow-up. *(Step 11.)*
- [x] **Determinism + golden regression:** the crop scenario is bit-identical within
      a build and registration-order-independent; a committed hex-float golden pins it
      (`test_regression_season` + `test_season`; transcendentals ⇒ within-build
      bit-stable, cross-platform tolerance-gated). *(Step 11.)*
- [x] **Engine invariants still hold:** core purity, determinism, frozen Phase-0/0.5
      API (additions only); the Phase-0/0.5 gates (incl. analytic convergence/order
      and 100k stability) stay green — the **biology is validated against the oracle,
      not by dt-convergence** (the gate split, P3).

> **Phase-1 status:** All criteria met **except the quantitative oracle match, which is
> explicitly deferred** (the `[~]` above). The single-producer *machinery* is complete and
> regression-pinned; behavioural calibration to the oracle is the next phase of work.

---

## Consolidation audit (2026-06-20) — pre-Phase-2 fresh-eyes review

A timeboxed hardening pass before opening Phase 2, targeting the **gate-invisible
physics risks the plan names** — the ones the conservation gate and `rationed == 0`
pass *trivially* over (a wrong-but-balanced trajectory still conserves), so only a code
read catches them. **Result: clean — no behaviour-preserving fix met the bar; the audit
is the deliverable** (the season golden must stay bit-identical, so cosmetic churn is not
hardening). Tree green (670 passed, oracle test opt-in).

- **Three-site structural agreement — verified real.** `GASS`, `MRES`, and `available`
  each have **exactly one** computation site, `CarbonContext.budget`
  (`carbon_budget.py`); `limitation` (`f_water · f_N`) is computed *inside* `budget`,
  never passed by a flow. `Allocation`, `GrowthRespiration`, `MaintenanceRespiration`
  all route through `ctx.budget`, so they **cannot** drift on assimilation / maintenance
  / limitation (structural, not disciplinary). Grep confirmed
  `daily_canopy_assimilation` / `maintenance_respiration_flux` / `available_for_growth`
  have no other assembled call site.
- **Maintenance covered/shortfall split — traced by hand, matches the documented
  surplus/deficit arithmetic.** `covered = min(GASS, MRES)`,
  `shortfall = max(0, MRES − GASS)`; `co2_resp` deposit = Σ of *actual* withdrawals (not
  `MRES·dt`) so float-rounding in the organ shares cannot break conservation; `available
  = max(0, GASS − MRES)` is subtracted once ⇒ maintenance paid once (no double-charge).
  Net `co2_atmos` loss = `GASS` on both surplus and deficit days, as documented.
- **Storage-organ web — consistent.** `storage_c` is excluded from `Σ organs`
  (maintenance / `f_N` biomass) and from `Senescence`; it is a pure allocation sink.
- **`carbon_fraction` dedup** — a *conscious* deferral; the guard test
  `test_committed_nitrogen_carbon_fraction_matches_canopy` already makes silent drift
  impossible. Not an oversight; single-sourcing the two param files is a future option,
  not a debt.

The foundation is cleared to build Phase 2 on — see `phase-2-closed-chamber.md`.
