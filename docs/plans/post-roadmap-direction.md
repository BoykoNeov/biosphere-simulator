# Post-roadmap direction — one plan for what is next

**Written 2026-08-13, on frozen `main` (`80db320`), nothing built.** The roadmap ended at
Phase 9; everything since has been chosen one item at a time from the successors the
previous item named. This doc does the thing that has never been done: reads the whole
open queue at once and puts it in one order.

It is a *plan*, not a record. When an item here is finished it earns a line in
`docs/post-roadmap-log.md`, a file in `docs/log/`, and a memory — and it leaves this doc.

---

## 1. Where the project stands

Roadmap Phases 0–9 are all COMPLETE. Four freeze contracts hold (biosphere science,
station assembly, native-port tolerance, authoring platform). Two independent ports agree
to a measured tolerance. The suite is green, 7m05s at `-n 12`.

Since the roadmap ended, ~38 pieces of work have shipped, and **almost all of them are one
thread**: making the biosphere's science more faithful, one mechanism at a time, each one
either built from a primary source or *refused* on one. The refusals are the valuable half
— the record is full of mechanisms that were priced, measured, and turned down because the
citation did not actually say what a summary of it said.

**The two most recent items both ended DIAGNOSED, NOTHING BUILT, and both landed on the
same wall.** That is new. Up to now the queue branched; now it converges.

---

## 2. The gate: the biosphere's integration step

Everything in section 3 sits behind one decision. The biosphere freeze's *first* item is
`Euler / dt = 1` — forward Euler, one-day steps. Two separate investigations, asking
unrelated questions, both concluded the step is the binding constraint.

### What is measured (all on frozen `main`, both records reproduce their inputs exactly)

| Quantity | Value | Source |
|---|---|---|
| Frozen science convergence under RK4, `dt = 1` | **0.06 %** over a 4× refinement — already converged | `log/allocation-headroom.md` finding 6(a) |
| Shipped Euler `dt = 1` truncation error, peak leaf carbon | **3.2 %**, baked into every biosphere golden | ibid. finding 6(b) |
| Shipped Euler `dt = 1` error, sealed chamber season-low CO₂ | **24 %** (57.9 ppm vs converged 76.3) | `log/co2-enrichment-margin.md` finding 6(a) |
| CO₂ compensation point (assimilation is *exactly* zero below it) | **61.07 ppm** (`Γ*/ci_ratio`; ⚠ `Γ*` is `TODO(cite)`) | ibid. finding 4 |
| Does the frozen tree cross it? | **Yes** — by 5 % at ambient, by **3×** at 4× enrichment | ibid. findings 4, 6(b) |
| Arbitration headroom, RK4 `dt = 1` | open field **9.14×**, sealed chamber **1.375×**, perennial **1.391×** | `log/allocation-headroom.md` finding 2 |
| Same, as the tree's own `k·h < 1` bound | frozen sealed **0.727**; parked leaf branch sealed **1.055** — over the bound | ibid. finding 7 |

Two things follow, and they are different in kind:

1. **A numerical one.** The sealed chamber passes by ~37 % of margin, not by construction.
   The parked leaf mechanism pushes the one stock in the tree whose rate constant is
   *emergent* rather than a parameter past the `k·h < 1` bound the rest of the tree
   enforces in three other places. Under RK4 that is a hard error; under shipped Euler it
   is **silent rationing**, which is worse.
2. **A scientific one, independent of leaves and present today.** The shipped step makes
   the crop fix carbon at concentrations where the model says it fixes none. This is not a
   guard going off — every gate is green — it is the answer being wrong. And it runs the
   wrong way for realism: at 4× enrichment the shipped integrator *overstates* peak plant
   carbon by 4.3 %.

⚠ The convergence check is what separates (2) from arithmetic. Margin scales as `1/dt`
near-tautologically, so a guard going quiet at a finer step proves nothing
(`log/leaf-expansion.md` finding 9, with the sign flipped). The season-low CO₂ does not
scale — it **converges**, 57.9 → 75.1 → 75.8 → 76.0 through four refinements, from both
integrators, approaching from opposite sides. That is a statement about the answer.

### The menu, with the two knobs separated

The freeze names one thing (`Euler / dt = 1`) that is really two: the *integrator* and the
*step*. They have been discussed as one and they are not.

| Option | Science | Leaf mechanism | Cost |
|---|---|---|---|
| **Euler, `dt = 1`** (status quo) | 24 % low on chamber CO₂; crosses the shutoff | blocked (silently rations) | zero |
| **Euler, `dt = ½`** | clean and correct on everything tested | clears | **2×** step count |
| **RK4, `dt = 1`** | converged | **still blocked** (`k·h = 1.055`) | ~4× derivative evaluations |
| **RK4, `dt = ½`** | converged | clears | ~8× |
| **Kinetic saturation, `dt = 1`** | unmeasured | would clear by construction | needs a *cited* form |

The row that is easy to miss: **RK4 at `dt = 1` does not unblock the leaf mechanism.** The
leaf branch's over-draw is a step-size problem, not an integrator-order problem. So
"switch to RK4" is not a cheaper substitute for "halve the step" — it is a different,
more expensive purchase that buys less.

**The unfreeze price is the same for every option that moves a golden** — this is the part
that argues for deciding once:

- **Biosphere reference** — `Euler / dt = 1` is its first frozen item; 7 scenarios → goldens.
- **Station reference** — 13 scenarios, biosphere **delegated**, so they move with it.
- **Native-port reference** — freezes *measured* cross-port tolerance bands. ⚠ It never
  names a step, so **nothing in it would go red**; its dependence on `dt = 1` is entirely
  implicit. A contract whose dependence on the thing being unfrozen is unstated is the one
  left out of the price. Requires a Rust parity re-measure.
- 20 of the 25 goldens carry the cross-port tier contract; all of them regenerate.
- ⚠ The existing multi-rate machinery is **not** a cheap version of this: `simcore.multirate`
  freezes aux by design, so phenology would stop advancing.

### Recommendation

**Euler at `dt = ½`.** It is the cheapest option that is both numerically clean and
scientifically correct on everything measured; it moves the *answer* toward the converged
limit rather than merely quieting a guard; and it does not touch the integrator contract,
the arbitration backstop's Euler-only scope, or any form. RK4 costs more and solves less.
Kinetic saturation is attractive in principle — it keeps `dt = 1` and touches no integrator
contract — but its half-saturation constant has to come from a source, and tuned instead it
is `a-clamp-hides-a-wrong-amount` wearing a mechanism costume. Keep it alive as a shelf
search, not as a plan.

⚠ **This is the user's call, not mine.** Three freeze contracts is the largest ceremony
this project has run.

### Step 0 — the measurement pass that should happen either way

Before any ceremony, and costing none of it: one probe harness, `src/` untouched, no golden
regenerated — exactly how the last two items were run.

Sweep `{Euler, RK4} × dt ∈ {1, ½, ¼}` across every scenario on disk, plus the parked
`leaf-expansion-blocked` branch, plus the CO₂ enrichment sweep ×1…×5, recording per run:
unclamped arbitration margin, rationing firings, season-low chamber CO₂ against 61.07 ppm,
peak leaf carbon drift, harvest, and wall-clock.

Deliverable: a table that makes the decision **arithmetic instead of a judgement**, and a
measured answer to the two questions nobody can answer today — *is `dt = ½` enough on all
25 scenarios, or only on the three that were probed?* and *what does the suite runtime
actually become?* If `dt = ½` fails anywhere, the recommendation above is wrong and we
learn that for the price of a probe.

---

## 3. What is queued behind the gate

These should ride the *same* unfreeze, not pay for a second one:

1. **The parked leaf mechanism** (`leaf-expansion-blocked`, `cb668f6`). Standing is *"not
   refuted, route identified, evidence base pending re-measurement"* — **not** "unblocked,
   ships as-is". ⚠ Every piece of evidence that got it accepted (the Greenwood gate clearing
   by 5.2 %, thickness inside real wheat's range, rationing 0, the `WSFL` leverage cut) was
   measured at Euler `dt = 1`, and finding 6(b) gives a concrete reason to expect it to move.
   **Re-measure the evidence base at the shipping step before merging.**
2. **A science band for the chamber's minimum CO₂.** `science_bands` in both manifests
   already give assertions contract standing; *"the sealed chamber's season-low CO₂ stays
   above the compensation point"* is exactly that shape and would have caught this on day
   one. ⚠ It is **red on the frozen tree today**, so it cannot land before the step decision
   without an explicitly documented allowance — which is the honest version of what the tree
   currently does silently.
3. **The chamber CO₂ *controller*.** Real enrichment holds a concentration; `chamber_co2_mol0`
   charges once and the crop eats it. A controller would make this whole fragility disappear
   — while introducing a make-up flow, which is the O₂ regulator's shape and inherits its
   direction hazard. ⚠ Nothing has priced it; do not assume it is cheap. This is the next
   *realism* move once the step is settled.

---

## 4. What does not touch the gate and can move now

- **`Γ*`'s citation.** The floor this whole diagnosis is measured against is one of
  `photosynthesis.yaml`'s 13 `TODO(cite)` entries. The measurement survives any plausible
  re-value (a 3× crossing is robust), but the *number* 61.07 ppm should not appear in a
  science claim until it is sourced. It is a standard FvCB quantity — one targeted retrieval
  attempt, not a reopening of the citation bucket.
- **Citation debt generally.** 79 `TODO(cite)` across 37 files. Bucket 3(C) closed after 7
  rounds as *blocked on retrieval, not effort*, with the residual risk documented. Don't
  reopen it wholesale; take single params when a finding leans on one.
- **Contract hygiene — two known holes, both cheap:**
  - A **provenance-only edit is an unfreeze that nothing catches** (the per-file sha-256 is
    recorded but never compared). Currently honor-system.
  - **The freeze's prose half is ungated** — the manifest gate equates manifest↔tree; the
    doc is not a side.
- **Potato stage 2** — the Rust habitat mirror, deferred at stage 1.
- **The canopy regulator** — DIAGNOSED, not built; fixes the canopy and is bit-identically
  inert on every frozen scenario. Cheap, and its inertness makes it a low-risk unfreeze.

---

## 5. The dormant track: product

`CLAUDE.md`'s stated end goal is *"a science-credible Godot station sim that runs the same
simulation headless"*, and the Rust-primary pivot (2026-07-20) opened a lane for exactly
that: **new content and gameplay are Rust-first, no Python mirror owed, Rust-native
conservation + determinism instead of cross-port goldens.**

**Nothing has used that lane since it was decided.** Every item in the ~3 weeks since has
been Python biosphere science under the unfreeze discipline. That is not a criticism of the
work — the science thread has been productive and the refusals are load-bearing — but it
means the product half of the project has been stationary for a month, and the decision to
leave it there has never been taken explicitly.

This is a genuine fork and it belongs in the plan as one:

- **Keep going on science.** The queue is deep and the thread is converging on a decision
  that will genuinely improve fidelity.
- **Wake the product track.** Godot front-end, authored habitats as content, whatever the
  station is supposed to *be* as a thing someone plays or watches.
- **Both, alternating.** The science thread has natural stopping points (each item ends
  with a record), so a habitat/UI item between science items costs nothing structurally.

No recommendation here — it depends on what the user wants the project to be, which is not
a technical question.

---

## 6. Proposed order

1. **Step 0, the measurement pass** — probes only, no ceremony, no risk. Makes the step
   decision arithmetic. *Do this regardless of which way the decision goes.*
2. **The step decision** — the user's call, informed by (1).
3. **If the step moves: one ceremony**, carrying the step change + the leaf mechanism with a
   re-measured evidence base + the chamber-CO₂ science band. Three contracts, one unfreeze,
   one Rust parity re-measure.
4. **The chamber CO₂ controller** — the next realism move, priced properly first.
5. **In parallel, independent of all of the above:** `Γ*`'s citation, the two contract
   hygiene holes, the canopy regulator, potato stage 2.
6. **Separately and explicitly: the product-track fork** in section 5.

---

## 7. The open questions for the user

1. **The step.** Euler `dt = ½` (recommended), RK4, kinetic saturation, or hold at `dt = 1`
   and accept a documented 24 % error on chamber CO₂ with the leaf mechanism refused?
2. **The fork.** Stay on biosphere science, wake the Godot/Rust product track, or alternate?
