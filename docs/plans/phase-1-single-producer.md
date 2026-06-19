# Phase 1 — Single Producer

**Status:** IN PROGRESS. **Steps 1 (units + area basis), 2 (the non-conserved aux
channel), 3 (the PCSE oracle harness + clean-room param discipline), and 4
(Beer–Lambert light interception — the first biological process) are implemented,
tested, and committed** — see their per-step `RESOLVED` blocks below. **The
foundation (Steps 1–3) is complete; Step 5 (FvCB photosynthesis) is next** — the
carbon-source flow that consumes the Step-4 canopy diagnostic, designed just-in-time. This plan **locks the three
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
   (CO₂ → plant carbon), gated by `f_light·f_temp·f_water·f_N` (limiters default to
   1.0 until their step lands — each process standalone first, roadmap line 232).
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
| Root water uptake (7) | WATER | soil-water POOL → plant water | ✓ |
| Transpiration (7) | WATER | plant/soil water → boundary vapor sink | ✓ |
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

---

## Exit criteria (Phase 1 — "research-grade single producer")

- [x] **Foundation locked:** science units + area basis (P4); the non-conserved aux
      channel (P2) — additive to frozen State, serialized, observed, outside the
      conservation gate; PCSE oracle harness behind a marker, clean-room param
      discipline (P5). *(Steps 1–3 — complete.)*
- [ ] **Seven processes**, each shipped as **flow/aux-rate + unit test + param file
      + documentation** before integration (roadmap line 232): Beer–Lambert, FvCB,
      respiration, Penman–Monteith, phenology, allocation, nitrogen. *(Steps 4–10.)*
- [ ] **Single-currency + multiplicative coupling (P1)** throughout; **OXYGEN not
      tracked**; every flow internally balanced in its one currency; the every-step
      conservation gate holds over a full season with `rationed == 0` (self-limiting
      kinetics; Euler-daily, P3).
- [ ] **Behavioral oracle match (P5)** — biomass trajectories, carbon gas exchange,
      water use, and nitrogen dynamics **reproduce reference behavior** (growth-
      chamber literature + WOFOST/PCSE) for the chosen crop, within tolerance (not
      bit-exact). *(Step 11.)*
- [ ] **Determinism + golden regression:** the crop scenario is bit-identical within
      a build and registration-order-independent; a committed hex-float golden pins
      it. *(Step 11.)*
- [ ] **Engine invariants still hold:** core purity, determinism, frozen Phase-0/0.5
      API (additions only); the Phase-0/0.5 gates (incl. analytic convergence/order
      and 100k stability) stay green — the **biology is validated against the oracle,
      not by dt-convergence** (the gate split, P3).
```
