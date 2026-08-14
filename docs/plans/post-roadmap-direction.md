# Post-roadmap direction — one plan for what is next

**Written 2026-08-13, on frozen `main` (`80db320`), nothing built.** The roadmap ended at
Phase 9; everything since has been chosen one item at a time from the successors the
previous item named. This doc does the thing that has never been done: reads the whole
open queue at once and puts it in one order.

It is a *plan*, not a record. When an item here is finished it earns a line in
`docs/post-roadmap-log.md`, a file in `docs/log/`, and a memory — and it leaves this doc.

**Discharged so far (2026-08-13, same day):** the documented allowance (§4, first bullet;
`36d0ae5`), Step 0 **axis 2** — the controller probe (§2, and it came back **negative**,
which changed three other sections), and §7 **question 2**, the science-vs-product fork
(answered: stay on science). **And 2026-08-14:** Step 0 **axis 1** — the step sweep (§2;
`dt = ½` clears everywhere measurable, and it found two costs this doc did not price).
Struck-through text below is kept deliberately: the reasoning that turned out to be wrong is
the most useful part of a plan doc to leave visible.

⚠ **Step 0 is complete. The one open item in this doc is §7 question 1 — the step decision,
which is the user's.**

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
| **Euler, `dt = ½`** | clean and correct on everything tested | clears (`k·h ≈ 0.53`) | **2×** step count |
| **Euler, `dt = ¼`** | converged | clears with room (`k·h ≈ 0.26`) | 4× |
| **RK4, `dt = 1`** | converged | **still blocked** (`k·h = 1.055`) | ~4× derivative evaluations |
| **RK4, `dt = ½`** | converged | clears | ~8× |
| **Kinetic saturation, `dt = 1`** | unmeasured | would clear by construction | needs a *cited* form |
| **CO₂ setpoint controller, `dt = 1`** | ❌ **MEASURED 2026-08-13 — fails, worse than the status quo** | ❌ cannot clear (margin 0.47 at 1200 ppm) | ~~no step unfreeze~~ — needs `dt = ¼` |

⚠ **The `dt = ½` / `dt = ¼` choice is not a runtime question, it is a *ceremony-count*
question.** The scarce resource is the unfreeze, not the CPU. `dt = ½` clears the parked leaf
branch by less than 2× against a bound that finding 7 says is **emergent** — no load-time
check can ever guard it — so the next mechanism that raises assimilation rate eats that margin
and buys the three-contract ceremony a second time. Frame the choice as *"the minimum step
that clears today"* versus *"a step with room for the next two mechanisms."*

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
- ⚠ **Regenerating 25 hex-float goldens on this box is a known CI hazard, not a theoretical
  one.** `ci-python-job-red-on-linux` — goldens are UCRT-generated, CI is Linux, libm differs
  by ULPs, and locally-minted goldens have gone red on CI before. Price it next to the Rust
  parity re-measure.
- ⚠ The existing multi-rate machinery is **not** a cheap version of this: `simcore.multirate`
  freezes aux by design, so phenology would stop advancing.

### ~~⚠⚠ The controller may dissolve this whole gate~~ — MEASURED 2026-08-13, IT DOES NOT

This subsection asked whether a chamber CO₂ *controller* — real enrichment **holds** a
concentration rather than charging once — removes the reason to touch the step at all. It
argued (a) a setpoint near 1000–1200 ppm never approaches the 61.07 ppm compensation point,
so the science defect **stops existing**, and (b) `k·h = (rate · h)/stock` *falls* when
`stock` is pinned high while assimilation saturates in `Ci`, so the numerical defect
plausibly clears too. **Both were measured and both are wrong.** Full record:
[`../log/co2-controller.md`](../log/co2-controller.md).

- **(a) is wrong, and in the *opposite* direction.** The crossing was never about the
  initial charge — it is a **truncation error** that a high, sustained assimilation rate
  *amplifies*. Holding the sealed chamber at the 357 ppm it already starts from is **four
  times worse** than letting it deplete (margin 1.3072 → 0.2977, season-low CO₂
  57.9 → 10.6 ppm, rationing firings 0 → 270), because the uncontrolled chamber
  **self-limits** and a controller removes that feedback. At 1000–1200 ppm the season-low CO₂
  is still **39–77 % below the floor** on all three chambers (−75 % on the two plant-only
  ones; the consumer chamber, with the crew supplying carbon, is the shallow end at −39 %).
- **(b) is wrong.** At 1200 ppm the margin is **0.4744** with 210 silent rationing firings on
  the sealed and perennial chambers, and **0.9515** with 3 firings on the consumer chamber —
  a fail on every one. The first setpoint that clears both criteria is **~3000 ppm** (1500 on
  the consumer chamber), above the 1785 ppm cliff the enrichment record already measured —
  and it clears by 8 %, against a bound finding 7 calls *emergent*.
- **And the price runs the wrong way.** The discriminator is a convergence check *within*
  the setpoint, because the case against the step is a truncation error rather than a
  threshold crossing. Controlled `dt = 1` at 1200 ppm is **22 % low on peak leaf carbon
  against its own converged limit** — **6.9×** the frozen tree's 3.2 %, read at the same 4×
  refinement (the two figures were not the same statistic as first written; the controlled run
  is converged by `dt = ¼`, so the ratio survives the correction). Where the sealed chamber
  clears at `dt = ½` uncontrolled, controlled at 1200 ppm it needs **`dt = ¼`**. ⚠ *"Clears at
  `dt = ½`"* is itself a **sealed-chamber** number from the enrichment record — establishing
  it across all 25 scenarios is what **axis 1** is for, and axis 1 has not run. **The
  controller does not cancel the step gate; it doubles the cost of clearing it.**
- ⚠ **It also makes "RK4 at `dt = 1`" unbuildable** (hard error at every setpoint tested),
  coupling the two knobs this section had just separated.
- ⚠ **This generalizes past the probe.** Arbitration scales against the **start-of-step**
  level and credits no same-step inflows (`simcore/arbitration.py:75`), so a make-up *flow*
  is strictly worse for the margin than the dawn state-edit clamp that was measured — which
  makes that clamp the **upper bound on what any controller can do here**.

### Recommendation

**Euler at `dt = ½`. The condition below is DISCHARGED — the controller probe came back
negative (2026-08-13), so this recommendation stands, and is also the cheaper branch.** Of the
step options it is the cheapest that is both numerically clean and scientifically correct on
everything measured; it moves the *answer* toward the converged limit rather than merely
quieting a guard; and it does not touch the integrator contract, the arbitration backstop's
Euler-only scope, or any form. RK4 costs more and solves less. Kinetic saturation is
attractive in principle — it keeps `dt = 1` and touches no integrator contract — but its
half-saturation constant has to come from a source, and tuned instead it is
`a-clamp-hides-a-wrong-amount` wearing a mechanism costume. Keep it alive as a shelf search,
not as a plan. ~~⚠ If the controller probe clears both criteria at `dt = 1`, this
recommendation is **withdrawn** and the question becomes "controller or step".~~ **It did
not clear; the question is "which step" after all.** ⚠ One thing the probe *added* to this
recommendation: the controller path is not merely "not a substitute" — it is a **`dt = ¼`
object**, so choosing it would buy the step ceremony *and* an unpriced make-up flow.

⚠ **The step decision is the user's call, not mine.** Three freeze contracts is the largest
ceremony this project has run.

⚠ **Do not oversell the 24 %.** It is a season **minimum** — a tail statistic on the most
sensitive observable in the run. The headline outputs move ~3 % (peak leaf carbon 3.2 %;
harvest barely). Both numbers belong in any sentence that asks the user to authorize a
ceremony; quoting 24 % alone hands them a surprise when harvest moves 3 %.

### Step 0 — the measurement pass that should happen either way

Before any ceremony, and costing none of it: one probe harness, `src/` untouched, no golden
regenerated — exactly how the last two items were run.

**~~Axis 1 — the step.~~ ✅ DONE 2026-08-14 — `dt = ½` clears everywhere measurable, and the
ceremony is bigger than this doc prices.** Record: [`../log/step-sweep.md`](../log/step-sweep.md);
design and every table: [`post-roadmap-step-sweep.md`](post-roadmap-step-sweep.md). Ran as
specified — every scenario on disk, the parked branch (from a **worktree**, so `main` was
never checked out of), the ×1…×5 enrichment sweep, all the named observables — plus three
things the specification did not have:

* ⚠ **The axis is a refinement factor, not an absolute `dt`.** Every forcing here is a
  function of the integer step `n`, never `n·dt` (all five sites audited), so `dt ∈ {1, ½, ¼}`
  is only meaningful against each scenario's own shipped step, and holding forcing fixed takes
  a different rule per site.
* ⚠⚠ **Four station goldens cannot take a finer biosphere step at all** without a change to
  `src/station/driver.py` — `run_master_day` pins the biosphere to one step per master day.
  **The ceremony is engine code plus three freeze contracts, not three freeze contracts.**
* ⚠⚠ **`water_biting` converges to two different answers under the two integrators** (Euler
  harvest 0.73, RK4 0.005, both stable under 8× refinement) — a **new argument against every
  RK4 row on the menu**, and not caused by this work.

Headline: `dt = ½` clears the 61.07 ppm compensation point on all 9 biosphere scenarios and
all 5 enrichment levels; the tail statistic moves **24 %** and harvest **0.7 %**; the parked
leaf branch is worse at `dt = 1` than its own record showed (a 2.2× crossing, not 5 %) and
clears at 2.1× headroom; the station/physics half is nowhere near the wall and should not be
moved at all. **The decision itself remains the user's and is untaken.**

**~~Axis 2 — the controller, and this is the one that can cancel axis 1.~~ ✅ DONE
2026-08-13 — it does not cancel axis 1.** Record: [`../log/co2-controller.md`](../log/co2-controller.md);
design and every table: [`post-roadmap-co2-controller.md`](post-roadmap-co2-controller.md).
Ran as specified plus two things the specification did not have and needed: an **ambient
control** (which is what inverted the hypothesis) and a **convergence check within the
setpoint** (because the case against the step is a truncation error, not a threshold
crossing, so two numbers at `dt = 1` could not have settled it). Deliberate omission: the
parked leaf branch was **not** probed — it draws more than the frozen tree, so it cannot
rescue a margin already below 1 on the lighter tree.

Deliverable: one table that makes the decision **arithmetic instead of a judgement**, and a
measured answer to the three questions nobody can answer today:

1. ~~Is `dt = ½` enough on all 25 scenarios, or only on the three that were probed?~~
   **ANSWERED 2026-08-14: enough on every scenario the method can measure** — 9 biosphere
   scenarios including both 15-year horizons, all 5 enrichment levels, the parked branch, and
   9 station/physics scenarios. ⚠ **Four station goldens are excluded for a structural reason,
   not silently** (`run_master_day`), and `water_biting`'s margin is uninformative by
   construction (a self-clamping flow), though its Euler trajectory is stable under 8×.
2. ~~What does the suite runtime actually become?~~ **ANSWERED 2026-08-14, as a bound:**
   simulation work 2×; the suite today measures **6 m 12 s** (superseding the documented
   7 m 05 s) and `dt = ½` lands between that and ~12 m 24 s. Bounded, not pinned — under
   `-n 12` the wall clock is set by the longest worker, not by total work.
3. ~~**Does a controlled chamber clear both criteria at `dt = 1`?**~~ **ANSWERED 2026-08-13:
   no, at no realistic setpoint — and the controlled tree needs `dt = ¼` where the frozen
   tree needs `dt = ½`.**

~~If `dt = ½` fails anywhere the recommendation above is wrong, and we learn that for the price
of a probe.~~ It did not fail anywhere measurable. ⚠ **Step 0 is now complete on both axes,
and every input to the step decision is in. The decision itself is the user's and has not
been taken.**

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
   one. ⚠ It is **red on the frozen tree today**, so the *band* cannot land before the step
   decision — but the **documented allowance can, and should, land immediately**; see
   section 4.

**And one that was moved out from behind the gate, measured, and moved back:**

3. **The chamber CO₂ *controller*.** Real enrichment holds a concentration; `chamber_co2_mol0`
   charges once and the crop eats it. It was filed here as *"the next realism move once the
   step is settled"*, then moved ahead of the gate on the grounds that it might cancel it.
   **Measured 2026-08-13 ([`../log/co2-controller.md`](../log/co2-controller.md)): it does
   not.** It belongs here after all — now for a *measured* reason instead of an unpriced one,
   and with a price attached: **a controller is a `dt = ¼` object**, needing a finer step than
   the frozen tree does, so it rides the same unfreeze rather than replacing it. ⚠ Its other
   costs are untouched by the probe: a make-up flow is the O₂ regulator's shape and inherits
   its direction hazard, `authored ≠ validated` applies, and a two-sided setpoint **vents**
   (235 of 429 mol at the passing setpoint), which is an odd thing for a habitat whose purpose
   is closing the carbon cycle.

---

## 4. What does not touch the gate and can move now

- ⚠ **The documented allowance — do this first, regardless of both answers in section 7.**
  A known-deviation note in `docs/biosphere-reference.md`: *the shipped step puts the sealed
  chamber's season-low CO₂ below the compensation point (57.9 ppm measured against 61.07),
  and 24 % below the converged 76.3*, with the measured numbers and the pointer to
  `log/co2-enrichment-margin.md`. **The freeze's prose half is ungated, so this moves no hash
  and needs no ceremony** — it is free. It converts a silent wrong answer into a visible one
  while the decision is pending, which is the honest version of what the tree does today.
  ⚠ Being free is exactly why it is easy to keep deferring; it is listed first on purpose.
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

✅ **ANSWERED 2026-08-13: stay on biosphere science.** The product track stays dormant, but
now **by decision rather than by default**, which was the whole point of writing this section.
It is not closed — re-open it at a natural stop in the science thread.

---

## 6. Proposed order

0. ✅ **The documented allowance** — DONE 2026-08-13 (`36d0ae5`), free, no hash moved.
1. ✅ **Step 0, the measurement pass — COMPLETE ON BOTH AXES.** Axis 2 (the controller) DONE
   2026-08-13, negative; **axis 1 (the step sweep) DONE 2026-08-14** — see section 2. Every
   input to the decision is now measured.
2. **The step decision** — the user's call, informed by (1). ~~⚠ Which question it even is
   depends on (1).~~ **Settled: it is "which step". ← THE PROJECT IS HERE.**
3. **Then: one ceremony**, carrying the step change + the leaf mechanism with a re-measured
   evidence base + the chamber-CO₂ science band. Three contracts, one unfreeze, one Rust
   parity re-measure, one CI-goldens hazard — ⚠ **plus a `src/station/driver.py` change**,
   which axis 1 found and this list did not have: `run_master_day` pins the biosphere to one
   step per master day, so four station goldens cannot take a finer step without it. (*The
   alternative branch — "the controller path, no step unfreeze" — was measured out; a
   controller needs a finer step than the frozen tree does.*)
4. **In parallel, independent of all of the above:** `Γ*`'s citation, the two contract
   hygiene holes, the canopy regulator, potato stage 2.
5. **Separately and explicitly: the product-track fork** in section 5.

---

## 7. The open questions for the user

1. **The step. Fully unblocked as of 2026-08-14** — both axes have run and neither removes the
   reason to ask. The live options are: Euler `dt = ½`, Euler `dt = ¼` (more headroom for the
   next mechanism, one ceremony instead of two), kinetic saturation (needs a *cited* form),
   ~~RK4 `dt = ½`~~, or hold and accept the now-documented deviation with the leaf mechanism
   refused. ~~the controller at `dt = 1`~~ is off the menu — measured. ⚠ **RK4 is now also
   argued against**: axis 1 found `water_biting` converging to a *qualitatively different*
   answer under RK4 (the crop dies), stable under 8× refinement, so any RK4 row changes a
   shipped golden's science. ⚠ When it is put: quote both the 24 % (season minimum) and the
   ~0.7 % (headline harvest), not the 24 % alone. ⚠ And say that the ceremony includes an
   engine-code change (`src/station/driver.py`), not three freeze contracts alone.
2. ✅ **The fork — ANSWERED 2026-08-13: stay on biosphere science.** The Godot/Rust product
   track stays dormant by decision now rather than by default. Section 5 stands as the record
   of what is parked; re-open it when the science thread reaches a natural stop.
