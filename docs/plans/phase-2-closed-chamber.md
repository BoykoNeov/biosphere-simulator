# Phase 2 — Closed Chamber / Producer + Decomposer

**Status:** IN PROGRESS — **plan drafted and advisor design-reviewed; the load-bearing
foundation design (P2.1, the element-composition core change) is written below and ready to
implement (Step 1).** The review corrected two things, now folded in: (1) the composition fold
has **two** mandatory sites, not one — `flow.py` (legs) *and* `conservation.py` (state
deltas); missing the second falsely trips the OXYGEN gate every photosynthesis step; (2) an
O₂ self-limitation mirror (Michaelis in O₂) is required so respiration keeps `rationed == 0`
on a depleting O₂ pool (flagged for steps 3/5). PQ=1 with pure-carbon biomass is confirmed
the correct, water-untouched formulation. No Phase-2 code yet. Phase 1 (single producer) is
complete
and regression-pinned (`docs/plans/phase-1-single-producer.md`); Phase 2 is **additive to
the biology** but is the **first deliberate modification of the frozen Phase-0 conservation
core** — a user-approved exception, scoped as tightly as the science allows (see P2.1).

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

1. **Element-composition core change (P2.1)** — `Stock.composition`, the
   `per_quantity_residual` fold, `sim_io` serialization + schema bump, goldens regenerate.
   Property tests: 1:1 default reproduces Phase-1 exactly; a 2-quantity flow
   (CO₂→biomass+O₂) balances CARBON and OXYGEN; a deliberately mis-stoichiometric flow
   trips the gate. **Pure infra — no biology yet.**

**Process steps — enumerated now, each designed just-in-time (Phase-1 rhythm).**

2. **Finite chamber atmosphere + the `Ci`-from-stock seam (P2.2)** — CO₂/O₂ POOL stocks;
   flip `Ci` from forcing to the live CO₂-pool read; verify FvCB self-limits the finite
   pool (`rationed == 0`). The first emergent feedback.
3. **Gas exchange as multi-quantity flows** — rework assimilation (`CO₂ → biomass + O₂`)
   and plant respiration (`biomass + O₂ → CO₂`) onto the finite pools with CARBON+OXYGEN
   legs (PQ=1). Re-examines the Phase-1 carbon-budget rewiring against the now-clamped CO₂.
4. **Litter + decomposition (CARBON)** — `litter_sink` → live `litter_carbon` POOL; senescence
   feeds it; first-order/Michaelis decomposition to microbial biomass + CO₂ (cited kinetics).
5. **Microbial respiration (CARBON+OXYGEN)** — `microbial_C + O₂ → CO₂`; the P2.1-dependent
   decomposer gas flux that draws O₂ down (the Biosphere-2 O₂-sink mechanism).
6. **Mineralization (NITROGEN)** — litter/microbial N → `soil_n`; closes the N loop that
   Phase 1 fed externally.
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

## Exit criteria (Phase 2 — "closed chamber / producer + decomposer")

- [ ] **Element-composition core (P2.1)** landed: stocks carry composition, the gate folds
      it, OXYGEN is genuinely asserted; the Phase-1 1:1 behavior is preserved (goldens
      regenerate only for the serialized field).
- [ ] **The genuine multi-quantity stoichiometric flow** (P1's filed deferral) exists:
      gas exchange balances CARBON *and* OXYGEN in one flow at PQ=1.
- [ ] **Emergent feedback (P2.2), no control code:** photosynthesis draws down a finite CO₂
      pool → `Ci` falls → assimilation falls; CO₂ draw-down/oscillation + O₂↔CO₂
      anti-correlation appear with no special code; `rationed == 0` on the finite pool
      (FvCB Ci-shutoff).
- [ ] **Decomposer loop (P2.3):** litter → microbial biomass → CO₂; microbial respiration
      draws O₂; mineralization returns N to `soil_n`.
- [ ] **Sealed multi-year run:** stable every-step conservation (all of CARBON/OXYGEN/WATER/
      NITROGEN), stable numerics, qualitatively believable dynamics incl. at least one
      closed-system phenomenon (O₂ depletion / Biosphere-2-style failure).
- [ ] **Determinism + golden + engine invariants:** bit-identical, order-independent,
      hex-float golden; purity + Phase-0/0.5 gates stay green.
