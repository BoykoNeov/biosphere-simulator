# Post-roadmap direction — one plan for what is next

> ⚠⚠ **SUPERSEDED 2026-09-02 by `post-roadmap-direction-2026-09.md`.** Kept whole as the
> record of a plan and of every place it went stale: four re-reads found stale spans here,
> and this file's own header says why (*a forward-looking doc goes stale from the front, and
> nothing here re-checks itself*). The September doc is the live queue, and it is the one
> `repo_gates` holds to the record table. Nothing below is current advice; read it for the
> reasoning that was priced and refused, which is still the most useful part.

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

~~⚠ **Step 0 is complete. The one open item in this doc is §7 question 1 — the step decision,~~
~~which is the user's.**~~

⚠ **STALE since 2026-08-14, marked 2026-08-31: the step decision was TAKEN** (`dt = ¼`, the
user's *"quarter the step"*), so this doc has no open item left in the sense this line means.
Read §4 and §6b as a **menu that other work has been eating from**, not as a live queue: its
first bullet was struck below after its premise was found sixteen days dead. Nothing here
re-checks itself, and a forward-looking doc goes stale from the front.

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

Headline: `dt = ½` clears the 61.07 ppm compensation point on all 4 sealed biosphere
configurations and all 5 enrichment levels; the tail statistic moves **24 %**, peak leaf
carbon **4.0 %** and harvest **0.7 %** — all three belong in the ask; the parked
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
   **ANSWERED 2026-08-14: enough on every scenario the method can measure** — 8 biosphere
   scenario configurations including both 15-year horizons, all 5 enrichment levels, the parked
   branch, and 9 station/physics scenarios. ⚠ Scope it precisely: the compensation-point
   criterion **only exists where the chamber is sealed** — 4 of the 8, and all 4 clear at
   `dt = ½`; the other two are open-field; the station/physics half has no plant and is judged
   on margin and drift instead. ⚠ **Four station goldens are excluded for a structural reason,
   not silently** (`run_master_day`), and `water_biting`'s margin is uninformative by
   construction (a self-clamping flow), though it clears the floor at every step and its Euler
   trajectory is stable under 8×.
2. ~~What does the suite runtime actually become?~~ **ANSWERED 2026-08-14, as a bound:**
   simulation work 2×; the suite measured **6 m 12 s** today (one run — same order as the
   documented 7 m 05 s, and not asserted as a correction to it) and `dt = ½` lands between that
   and ~12 m 24 s. Bounded, not pinned — under `-n 12` the wall clock is set by the longest
   worker, not by total work.
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
2. ~~**A science band for the chamber's minimum CO₂.** `science_bands` in both manifests~~
   ~~already give assertions contract standing; *"the sealed chamber's season-low CO₂ stays~~
   ~~above the compensation point"* is exactly that shape and would have caught this on day~~
   ~~one. ⚠ It is **red on the frozen tree today**, so the *band* cannot land before the step~~
   ~~decision — but the **documented allowance can, and should, land immediately**; see~~
   ~~section 4.~~

   ⚠⚠ **STRUCK 2026-08-31: this bullet is stale twice over, and the two staleness causes are
   opposite.** The band it asks for **shipped** with the step unfreeze — five
   `..._stays_above_the_compensation_point` rows in `rust/crates/domains/src/biosphere/science_gates.rs`,
   one per sealed scenario — and the *"red on the frozen tree today"* that was supposed to block
   it stopped being true the same day, because the step change (`dt = 1 → ¼`) **fixed** the
   crossing rather than documenting it. So the item was simultaneously already done and no longer
   blocked, and neither fact reached the line. ⚠ **The third instance of the shape
   [`../log/co2-band-recheck.md`](../log/co2-band-recheck.md) named the day before** — *a
   forward-looking list is written once and read many times, and nothing re-checks it* — and the
   second one found in this section. What the band could not hold, because it is written one-sided
   on purpose, is now held next to it: [`../log/co2-margin-pin.md`](../log/co2-margin-pin.md).

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

- ~~⚠ **The documented allowance — do this first, regardless of both answers in section 7.**~~
  ~~A known-deviation note in `docs/biosphere-reference.md`: *the shipped step puts the sealed~~
  ~~chamber's season-low CO₂ below the compensation point (57.9 ppm measured against 61.07),~~
  ~~and 24 % below the converged 76.3*, with the measured numbers and the pointer to~~
  ~~`log/co2-enrichment-margin.md`. **The freeze's prose half is ungated, so this moves no hash~~
  ~~and needs no ceremony** — it is free. It converts a silent wrong answer into a visible one~~
  ~~while the decision is pending, which is the honest version of what the tree does today.~~
  ~~⚠ Being free is exactly why it is easy to keep deferring; it is listed first on purpose.~~

  ⚠⚠ **STRUCK 2026-08-31: there is no deviation to document, and writing the note would have
  put a false statement into the frozen reference.** Taken as the next item on the user's call,
  and the first thing the target file said was that it was already discharged. Three reasons,
  each independently fatal to the bullet:

  1. **Its premise ended the day after it was written.** *"While the decision is pending"* — the
     §7 step decision landed 2026-08-14 (`dt = 1 → ¼`) and *fixed* the crossing rather than
     documenting it. This bullet says it holds *"regardless of both answers in section 7"*, and
     the §7 answer is exactly what discharged it. **An item written to be answer-independent was
     the one the answer cancelled** — because it was scoped to the waiting, not to the defect.
  2. **Its numbers name the one scenario that never crossed.** `57.9 ppm` is the sealed chamber
     driven through an unconditional re-sow no golden performs; the crossing was the *perennial*
     chamber's, at 56.03. Corrected in the freeze doc on 2026-08-14 — this bullet inherited the
     locus error and outlived the correction by seventeen days.
  3. **Measured, not assumed** (shipped tree `7f60442`): sealed 71.44, perennial 70.25, consumer
     73.34, both long horizons 70.25 / 73.34, against a 61.07 floor. All five clear, all five
     identical to the freeze doc's `cc44b41` table, and all five gated in Rust.

  ⚠ **What the visit did find, and it is the free work this bullet was reaching for:** the
  freeze doc's band section pointed at `tests/test_co2_compensation_band.py` and offered
  *"`git diff src/` empty"* as its evidence — both naming a tree S6 deleted on 2026-08-27 — and
  the **five-margin pin did not survive that deletion**, so nothing in `rust/` now records how
  near any of the five sits to its floor. Marked in place, with the re-measurement, at
  `docs/biosphere-reference.md` (*"RE-CHECKED 2026-08-31"*). The pin itself is a **candidate,
  not built**: a new assertion on a frozen observable is not a prose correction.
- **`Γ*`'s citation.** The floor this whole diagnosis is measured against is one of
  `photosynthesis.yaml`'s 13 `TODO(cite)` entries. The measurement survives any plausible
  re-value (a 3× crossing is robust), but the *number* 61.07 ppm should not appear in a
  science claim until it is sourced. It is a standard FvCB quantity — one targeted retrieval
  attempt, not a reopening of the citation bucket.
- **Citation debt generally.** ~~79 `TODO(cite)` across 37 files.~~ Bucket 3(C) closed after 7
  rounds as *blocked on retrieval, not effort*, with the residual risk documented. Don't
  reopen it wholesale; take single params when a finding leans on one.

  ⚠ **The count was the deleted Python tree's; re-measured on the reference 2026-09-01:
  60 `TODO(cite)` across 20 files in `domains` and 4 across 3 in `station` — the biosphere
  alone is 53 across 15.** The claim stands; only the number was stale.
- **Contract hygiene — ~~two known holes, both cheap~~ one, and the other was falsified:**
  - ~~A **provenance-only edit is an unfreeze that nothing catches** (the per-file sha-256 is~~
    ~~recorded but never compared). Currently honor-system.~~

    ⚠⚠ **STRUCK 2026-09-01 — false since C7 (2026-08-18), and this time it was RUN rather
    than cited.** `CLAUDE.md` has carried the correction for two weeks; this bullet never got
    it. The reference now *writes* each manifest from the files it compiles in and
    `manifest_writer.rs` byte-compares the committed one, so a `source:`-only edit leaves the
    manifest stale and **red**. Control: appending `(control probe)` to one `source:` string
    in `canopy.yaml` fails `the_committed_manifest_is_what_the_reference_writes` by name, with
    both hashes printed; reverted byte-for-byte, gate green again. Record:
    [`../log/co2-margin-pin.md`](../log/co2-margin-pin.md) FINDING 10. **What is still
    honor-system is the ceremony, not the regeneration.**
  - **The freeze's prose half is ungated** — the manifest gate equates manifest↔tree; the
    doc is not a side. ✅ **Re-checked 2026-09-01: stands.**
- **Potato stage 2** — the Rust habitat mirror, deferred at stage 1. ✅ **Re-checked
  2026-09-01: stands.** The params crossed with the flip
  (`params/biosphere/crops/potato/`, 4 files), but `system.rs:1977` records that *"the Rust
  roster has no potato build at all"* — a param move is not the habitat mirror.
- ~~**The canopy regulator** — DIAGNOSED, not built; fixes the canopy and is bit-identically~~
  ~~inert on every frozen scenario. Cheap, and its inertness makes it a low-risk unfreeze.~~

  ⚠⚠ **STRUCK 2026-09-01: it is BUILT, and the bullet is wrong twice more as a description.**
  `science::mutual_shading_rate` is called from two sites in `flows.rs`, parameterised in
  `senescence.yaml` and gated by
  `the_vks_mutual_shading_regime_is_modelled_not_merely_avoided` — the Van Keulen & Seligman
  5 %/day-above-LAI-6 rule that [`../log/canopy-regulator.md`](../log/canopy-regulator.md)
  FINDING 1 identified. It shipped as *"the mutual-shading loss the pair forced"* inside
  [`../log/layered-canopy.md`](../log/layered-canopy.md), so **no row ever said "the canopy
  regulator is built"**. Beyond that: it is *inert on the chambers and flips the canopy on
  `open_season`* (`../log/acceptance-gate-standing.md`), not inert everywhere; and
  `../log/gross-net-gas-exchange.md` FINDING 5 **eliminated it by measurement** as a fix,
  since it is a 5 %/day *loss* and the canopy offered to it had fallen to 4.71. **Fourth
  instance** of *a forward-looking list is written once and read many times, and nothing
  re-checks it*.

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
2. ~~**The step decision** — the user's call, informed by (1). ⚠ Which question it even is~~
   ~~depends on (1). **Settled: it is "which step". ← THE PROJECT IS HERE.**~~

   ✅ **TAKEN 2026-08-14: `dt = 1` → `dt = ¼`** ([`../log/step-unfreeze.md`](../log/step-unfreeze.md)).
   ⚠⚠ **The arrow is struck 2026-08-31, seventeen days and eight work items late, and it was
   the single most misleading string in this file** — a reader landing in §6 was told the
   project was waiting on a decision that had already been taken.
3. ~~**Then: one ceremony**, carrying the step change + the leaf mechanism with a re-measured~~
   ~~evidence base + the chamber-CO₂ science band. Three contracts, one unfreeze, one Rust~~
   ~~parity re-measure, one CI-goldens hazard — ⚠ **plus a `src/station/driver.py` change**,~~
   ~~which axis 1 found and this list did not have: `run_master_day` pins the biosphere to one~~
   ~~step per master day, so four station goldens cannot take a finer step without it. (*The~~
   ~~alternative branch — "the controller path, no step unfreeze" — was measured out; a~~
   ~~controller needs a finer step than the frozen tree does.*)~~

   ⚠⚠ **STRUCK 2026-08-31: the bundle never happened, and all three parts shipped
   separately.** The step change went alone; the leaf mechanism *"stays excluded"* by that
   record's own closing line and its ship/refuse call is still the user's
   ([`../log/leaf-remeasurement.md`](../log/leaf-remeasurement.md)); the band landed on its own
   with the unfreeze; and the margin pin the band could not carry landed as a fourth piece
   ([`../log/co2-margin-pin.md`](../log/co2-margin-pin.md)). Written in the future tense about
   a plan that reality had already routed around, and left that way for seventeen days.
4. **In parallel, independent of all of the above:** `Γ*`'s citation, the two contract
   hygiene holes, the canopy regulator, potato stage 2.
5. **Separately and explicitly: the product-track fork** in section 5.

---

## 6b. OPENED 2026-08-14 by the light path: **the canopy this tree cannot grow**

The within-day light path shipped (`post-roadmap-gross-net-gas-exchange.md`), and it left
one question behind that is now the biggest open science item in the queue.

**The finding.** With an honest sun — the day's photons arriving unevenly, as the source
says they do — `open_season`'s converged peak LAI is **4.71**, against a frozen band of
`5.0 < peak < 8.0` sourced as *"real wheat peaks at ~5–8"*. It passes at the shipped step
(5.38) only because the observable is still moving 15 % between `dt = ¼` and `dt = 1/32`.
⇒ **the band was clearing against a diurnally biased assimilation**, and what the light
path exposes is a canopy this tree cannot grow.

**What is already eliminated, by measurement, so it is not re-proposed:**

- **The canopy regulator.** It is a 5 %/day leaf-area **loss** above LAI 6 — it can only
  push a canopy down, and we are now further below its threshold than before.
- **The parked leaf mechanism** (`leaf-expansion-blocked`). Sink-limited expansion sits
  *below* the frozen tree on both gated observables at every step
  (`log/leaf-remeasurement.md`) — it makes this worse, not better.
- **Re-tuning the bound.** Refused three times in this project already; the bound is
  sourced and the model is what moved.

**Candidate directions, none measured:** ~~the *intra-canopy* half of the Jensen bias is
still open (this work closed only the diurnal half — a sunlit/shaded or multi-layer canopy
is the cited next step and can move canopy assimilation *up*)~~ — ⚠ **STRUCK 2026-08-15:
MEASURED, AND THE SIGN IS BACKWARDS** (`post-roadmap-canopy-magnitude.md`). `Ag` is
**concave** in PAR — `photosynthesis.py` has said so since Step 5 — so resolving the canopy
into depths redistributes the same photons onto a concave response and can only *lower* the
sum. Measured: **0 of 2598 lit calls** above 1.0, and closed-loop peak LAI **5.3806 →
5.0314** at the shipped step, **4.7132 → 4.4169** converged. The clause above was written
without reading the function it was about, and it is left struck rather than deleted because
the way it was wrong is the finding. Refused as a *fix*; not refused as science. The
specific-leaf-area
constant has no DVS keying anywhere (`canopy-regulator-diagnosed` finding 2 measured it as
a single constant, which makes LAI strictly linear in leaf carbon); and the partition
table was **fitted** against the biased assimilation
(`wheat-partition-backfill-refused`), so it is a suspect in exactly the way the band is.
⚠ **The SLA candidate was measured 2026-08-15 and is not one direction but two.** Keying the
constant to development spans peak LAI **3.0446 (−35 %)** to **8.2391 (+75 %)** depending on
whether the frozen 22.0 m²/kg is read as the *young*-leaf or the *mature*-leaf value —
which `canopy.yaml` does not say. So it is a **citation** question, not a modelling one, and
the late-anchored reading also moves the independently-pinned "LAI peaks after anthesis" gap
the right way (DVS 1.373 → 0.958). The partition table remains **unmeasured**.

⚠ **Read the liveness floors before starting.** Two narrowed under the light path, and
perennial's converged peak-leaf sits **3.2 % over its floor on its settled value** — the
third move in the same direction from unrelated causes. A mechanism that raises canopy
assimilation moves that number the safe way; one that lowers it may take the floor red.
Both bands and floors are tabulated in `docs/biosphere-reference.md`. ⚠ **That warning was
discharged for the layered candidate on 2026-08-15 and the answer was "inert"**: perennial's
`max(tail)` moves 0.603679 → 0.603540 under it, because the correction scales with canopy
**closure** and the chambers' canopies never close — the canopy regulator's inertness
finding, reached again from an unrelated mechanism. Do not assume it generalises to a
candidate that works through leaf *area* rather than through shading.

⚠ **And one thing this section cannot do, found the same day:** peak LAI compounds through
interception at **~3.5×** (a 5 % nudge to leaf-area-per-carbon or to assimilation buys 18 %
of peak LAI) — ⚠ that being the elasticity to a **uniform** perturbation; a closure-weighted
one amplifies only ~1.14× — while three of its inputs are `TODO(cite)` provisional
literals. A band whose
subject amplifies every parameter error 3.5× can say the tree is in the right decade and
**cannot arbitrate between mechanisms** — so "the canopy this tree cannot grow" is a
provenance finding before it is a mechanism finding.

---

## 7. The open questions for the user

1. ✅ **ANSWERED 2026-08-14: Euler `dt = ¼`** — the user's *"quarter the step"*, shipped the
   same day ([`../log/step-unfreeze.md`](../log/step-unfreeze.md)). ⚠ **The paragraph below is
   the QUESTION as it stood, kept for the options it priced and struck 2026-08-31**, because a
   reader landing in §7 does not see the struck status header at the top of this file and would
   read a settled decision as live. Everything from here to the end of this item is history.

   ~~**The step. Fully unblocked as of 2026-08-14** — both axes have run and neither removes the~~
   ~~reason to ask.~~ The live options were: Euler `dt = ½`, Euler `dt = ¼` (more headroom for the
   next mechanism, one ceremony instead of two), kinetic saturation (needs a *cited* form),
   RK4 `dt = ½`, or hold and accept the now-documented deviation with the leaf mechanism
   refused. ~~the controller at `dt = 1`~~ is off the menu — measured. ⚠ **RK4 now carries an
   argument against it, and is deliberately NOT struck**: axis 1 found `water_biting`
   converging to a *qualitatively different* answer under RK4 (the crop dies), stable under 8×
   refinement, so any RK4 row would change a shipped golden's science. But the cause is a
   **hypothesis, not a settled diagnosis**, so it is reported, not used to delete a row —
   removing an option is part of the decision, and the decision is the user's. ⚠ When it is
   put: quote **three** numbers, not two — **24 %** (season minimum), **4.0 %** (peak leaf
   carbon), **0.7 %** (harvest). The "~3 % headline" written above splits into the last two,
   and quoting the smallest alone undersells exactly as quoting the largest oversells. ⚠ And
   say that the ceremony includes an engine-code change (`src/station/driver.py`), not three
   freeze contracts alone.
2. ✅ **The fork — ANSWERED 2026-08-13: stay on biosphere science.** The Godot/Rust product
   track stays dormant by decision now rather than by default. Section 5 stands as the record
   of what is parked; re-open it when the science thread reaches a natural stop.
