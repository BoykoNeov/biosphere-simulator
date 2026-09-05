## **The tolerance the inert term was holding** (the temperature item's own closing question, taken — and the hypothesis it was taken on is REFUTED)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per
> work item.

**MEASURED 2026-09-05 — lab-only, nothing built.** No param file, golden, manifest digest or
gate bound moved; every column is a `domains::lab` substitution rewritten in memory.
Subject: `rust/crates/domains/tests/mutual_shading_tolerance.rs` (5 tests). Probes and the
written-first predictions: `M:/claud_projects/temp/sla-ladder/`.

The question came out of `log/temperature-kinetics.md` FINDING 5, which measured the Van
Keulen & Seligman 5 %/day mutual-shading loss as **exactly inert** in the frozen tree while it
absorbed 7.31 of LAI under an alternative kinetics form — *"a mechanism that has never done
anything in any recorded run is now the thing that sets peak LAI."* The hypothesis taken from
it, and tested here: **a loss that pins the canopy at 6-point-something makes the
`5.0 < peak < 8.0` ceiling unreachable, so the band tests only its lower half.**

**It does not.** The ceiling is reachable, and what the loss actually buys the contract is
tolerance and an *ordering* of its detectors.

**One knob**, chosen so the answer is about shading and nothing else: `specific_leaf_area`,
the linear carbon→area conversion (`require_positive`, no upper bound), swept ×0.68 → ×8.0
against `shade_rate = 0.0`. ⚠ The other candidate, `quantum_yield`, was rejected before any
run: it is capped at ×3.33 by its own frozen bound *and* changes which photosynthetic branch
limits as it rises (FINDING 2/8 of the kinetics record), a confound with nothing to do with
shading.

---

## FINDING 1 — *"exactly inert"* is a claim about ONE observable, and the sibling quantity of the same run moves

At the frozen params, with the loss switched off:

| `open_season` | loss ON | loss OFF | |
|---|---|---|---|
| peak LAI | 6.022837 | 6.022837 | **bit-identical** |
| peak W (t/ha) | 13.379084 | **13.406590** | **+0.206 %** |

The recorded reason for the zero is exact and still holds — the canopy crosses the threshold
*at* its summit, so the loss only ever acts on the way down. But `peak W` is reached **later**
than `peak LAI`, by which time the loss has been shedding leaf carbon for days. So the term is
inert on the observable that was measured and live on the one that was not.

⚠ The kinetics record's sentence is true as written (its 2×2 is a peak-LAI table) and false as
it will be read. `senescence.yaml`'s own 2026-07-27 note — *"BIT-IDENTICALLY inert added to the
frozen form"* — is from when the canopy peaked at 5.19, under the threshold, and the term
genuinely never fired; it has been stale since the layered-canopy commit and nothing said so.

## FINDING 2 — THE HYPOTHESIS IS REFUTED: the ceiling is reachable, and the loss buys ~1.8× of tolerance

Bisected crossings of the `5.0 < peak < 8.0` ceiling, in multiples of the cited
`specific_leaf_area`:

| | loss OFF | loss ON | absorption |
|---|---|---|---|
| peak LAI crosses 8.0 | **×1.138** | **×2.014** | **1.77×** |

So the band's upper half is **not blind — it is tolerant**, and the tolerance is the loss's
doing rather than the canopy's. A tree whose leaf-thickness constant is wrong by 14 % reddens
the ceiling with the loss off; it takes a **twofold** error with the loss modelled.

⚠ This is the same shape as `log/partition-leaf-direction.md`'s *"the frozen table absorbs a
threefold error before the contract goes red"*, and it arrives at the opposite verdict about
the same band: there the tolerance was the partition table's, here it is the loss's. **The
`5.0 < peak < 8.0` band has now been measured, from two independent directions, to be a
decade-check rather than a discriminator** — which is what `log/canopy-magnitude.md` said in
prose about a 3.5× amplifier, now with numbers on both sides of it.

## FINDING 3 — prediction 3 was wrong, and the miss is the finding: `peak W` SATURATES

Written first: *"the biomass cap breaks FIRST, at ×1.1–1.2"*, reasoned from the frozen
headroom (7.8 %). Measured:

| | loss OFF | loss ON | absorption |
|---|---|---|---|
| peak W crosses 14.4248 | ×1.170 | **×3.81** | **3.26×** |

The prediction never asked whether `peak W` *saturates*. It does — with the loss on it crests
at **14.4435 at ×4.5**, which is **0.13 %** above the recorded cap, and falls away beyond
(14.1450 at ×8.0).

**WHY it saturates was measured, not derived, because the obvious reading is the wrong one.**
The crest sits 0.13 % above the cap, and that near-agreement invites a nitrogen explanation
which this tree's own files supply in their own words: `nitrogen.yaml` records that the flat
`n_critical` threshold and the Greenwood dilution curve *"[coincide] only at W ≈ 14.44 t/ha"*,
and `senescence.yaml` records 14.4248 as the point where `f_N` **first bites in a frozen
scenario**. Two numbers from unrelated derivations agreeing to one part in 800 is not something
to write up as an accident — and if the crop were pinned there by its own nitrogen limitation,
*"the cap is unfalsifiable in this direction"* would mean something else entirely: an
observable held **at the gate's own bound by a mechanism inside the model**.

**It is not nitrogen.** At the crest, dropping `n_critical` 0.015 → 0.010 and doubling
`max_uptake_capacity` each leave `peak W` **bit-identical** at 14.443512 — `f_N` is not biting
there at all, and the 0.13 % agreement is a coincidence of two unrelated numbers.

**It is light interception, and this is the 2×2 that says so** rather than arithmetic off the
extinction coefficient:

| `peak W` | frozen canopy | at the crest (×4.5) |
|---|---|---|
| `extinction_coef` 0.60 | 13.379084 | 14.443512 |
| `extinction_coef` 0.45 | **10.015729 (−25.1 %)** | **14.600939 (+1.1 %)** |

A 25 % cut in light capture costs the frozen canopy a quarter of its biomass and costs the
crest **nothing** — the signature of interception that is already complete, so more leaf area
buys no carbon and eventually costs maintenance respiration. ⚠ *The first draft of this
paragraph asserted the light explanation from `exp(-0.6 × 6.02) ≈ 0.027`, i.e. reasoned and then
written as measured. Both halves above exist because a causal claim earns the experiment that
removes the cause — and the one that was ASSUMED away turned out to be the refutable one.*

**So while the loss is modelled the biomass cap is nearly unfalsifiable in this direction**:
it can be exceeded at all only between about ×3.8 and the crest, and then by a tenth of a
percent. ⚠ Do not read that as "the cap is a weak gate" in general — the kinetics form broke it
by **27 %** (18.366). It is a statement about *this knob*, and the difference between the two
is exactly why an absorption factor has to name its direction.

## FINDING 4 — the loss does not blind the contract, it ORDERS it

With the loss off, the two open-field bounds break within 2.8 % of each other (×1.138 and
×1.170) — **near-redundant detectors**, two gates saying one thing. With it on they separate
by 1.9×, and the LAI ceiling becomes the one that fires first, at ×2.014.

That inverts the framing this item was taken on. The loss is not a clamp hiding a defect from
the band; it is what gives the contract two *different* detectors instead of two copies of
one.

## FINDING 5 — the second gate's chamber half, swept for the first time

`the_vks_mutual_shading_regime_is_modelled_not_merely_avoided` also asserts `chambers < 1.0` —
that the three chambers are carbon-limited by design and cannot reach the shading regime at
all. No knob had ever been moved against it. Measured:

| | frozen | ×2.5 | ×3.5 |
|---|---|---|---|
| `sealed_chamber` | 0.5425 | 0.9305 | **1.2061** |
| `perennial_chamber` | 0.4927 | 0.8439 | **1.0908** |
| `consumer_chamber` | 0.5849 | 0.9634 | **1.1873** |

It holds to ×2.5 and all three break it by ×3.5, so it is the **second** detector on this
knob — after the LAI ceiling (×2.01), before the biomass cap (×3.81). It is not the loss's
doing: the chambers never reach the threshold, so the term cannot act there at any rung run.

⚠ **The shared lab report cannot show this, and adding a row is not the fix.**
`ReadoutSpec::informs` resolves a gate *under the same scenario*, and this gate's scenario is
`open_season` — so a chamber row could not declare the gate it serves. **A gate that reads four
scenarios is representable in the report by exactly one of them.** Recorded, not rebuilt.

## FINDING 6 — the direction plan's `specific_leaf_area` span is a PRE-CLAMP
number, and git says so rather than an argument

The September direction plan ranks §2.1 item 1 on a span from `log/canopy-magnitude.md`:
*"keying the constant spans peak LAI 3.04 (−35 %) to 8.24 (+75 %)"*. Whether that was measured
with the mutual-shading loss was the open question.

**It was not, and this is checkable rather than inferable:** `git log -S` shows `shade_rate`
and `23.53` were introduced by the **same commit** — `a436a96`, 2026-08-15, *"the layered
canopy + the leaf-thickness anchor — an UNFREEZE"* — while the span was measured on the
diagnosis that preceded it, against a baseline of 22.0. **The parameter did not exist in the
tree when the span was measured.**

Re-measured at the current anchor, on the rung that *is* reproducible — the ramp's own top,
a uniform ×1.318 (29.0/22.0):

| ×1.318 of the cited SLA | peak LAI | vs frozen |
|---|---|---|
| loss OFF | 10.285864 | **+70.8 %** |
| loss ON | 6.272467 | **+4.1 %** |

The loss-OFF column reproduces the recorded +74.8 % to within a few points, which is the
control that says the old number is a no-clamp number rather than a different measurement.
**With the loss modelled the same relative change is worth +4.1 %**, so the 2.7× span the item
is ranked on is retired on its high side.

⚠ **The low side is NOT re-measured and must not be quoted from here.** The old figure came
from a DVS *ramp* (22.0 → 15.0); a uniform ×0.682 is a harsher change and gives **−87.0 %**
(peak LAI 0.785534) against the recorded −35.4 %. The two are not the same experiment. What
survives is that the loss is **one-sided**: at that rung the two arms are bit-identical, since
the term cannot act below its threshold.

---

## The predictions, written before the first run and scored

`M:/claud_projects/temp/sla-ladder/PREDICTION.md`, committed to before any column was run.

| # | predicted | measured | |
|---|---|---|---|
| 1 | loss-OFF crosses 8.0 at ×1.25–1.35 | ×1.138 | ❌ |
| 2 | loss-ON crosses 8.0 at ×1.6–2.2, *not never* | ×2.014 | ✅ |
| 3 | the biomass cap breaks FIRST, at ×1.1–1.2 | ×3.81, **last** | ❌ |
| 4 | the chambers reach 1.0 at about ×1.6 | between ×2.5 and ×3.0 | ❌ |
| 5 | the recorded span's high end is a no-clamp number | confirmed, by git | ✅ |

**2 of 5.** All three misses share one cause: each was reasoned from the frozen point's
*local* headroom, and every one of the three observables **saturates** before it reaches its
bound. A headroom percentage is not a distance when the curve flattens.

## Controls, and what they caught

* **The one-sided control.** At ×0.682 both arms are bit-identical on *both* observables, so
  the OFF arm is switching off the cited mechanism and nothing else.
* **Crossings are bisected, not bracketed.** The first draft asserted
  `let absorption = 2.00 / 1.14; assert!(absorption > 1.75)` — **constant arithmetic that no
  mutation could redden**. Replaced with a 10-step bisection, so the headline number is
  measured by the test that asserts it. *An assertion over two literals is a comment with a
  semicolon.*
* **The bisection's bracket is a real argument, not a convenience.** `peak W` with the loss on
  is **not monotone** — it crests at ×4.5 — so a bisection over `[1, 8]` would have returned a
  plausible wrong crossing. Both ends are asserted before the loop.
* **Absolute facts before the ratio.** Each crossing is pinned in its own right before
  `on / off` is asserted, because a ratio alone is satisfied by both arms moving together —
  which is precisely what disabling the loss produces.

### The mutation battery — and the run of it that was measuring NOTHING

⚠⚠ **The first battery was unreadable, and the tell was that the mutation's red set was
BYTE-IDENTICAL to the baseline's.** `sed -i` rewrote `science.rs`, cargo did not rebuild, and
both stages ran the *same* stale test binaries — left over from an earlier mutated build. It
reported nine reds at BASELINE on a tree `git status` called clean, and the single test named
in them passes on a forced rebuild. Every stage was re-run with a `touch` before each cargo
invocation and a **baseline-must-be-green guard that aborts the battery**, because a battery
whose baseline is red cannot attribute a single one of its reds.

*`CLAUDE.md` warns that a battery which greps only for reds cannot tell "no failures" from
"no run". This is the sibling failure: it cannot tell "the mutation did that" from "the
mutation was never compiled", and an identical red set across two different sources is what
gives it away.*

**Re-run clean: BASELINE green — 414 passed, 0 failed** across `--lib`,
`mutual_shading_tolerance`, `temperature_kinetics` and `golden_regression`. Then:

| this file's test | M1: the loss never fires | M2: the threshold is 0.5, not 6.0 |
|---|---|---|
| inert on peak LAI / live on peak W | **RED** | **RED** |
| **the LAI ceiling's absorption (the headline)** | **RED** | **RED** |
| the biomass cap's absorption | **RED** | **RED** |
| the loss is one-sided below its threshold | green — *correctly* | **RED** |
| the chamber half of the gate | green — *correctly* | **RED** |

**Every one of the five is reachable, and the two that survive M1 survive it for a reason
that is the test's own subject**: with the loss deleted the two arms are identical
everywhere, which is exactly what the one-sided control asserts below the threshold; and the
chambers never reach the threshold, so no change to the term can move them. M2 — which drops
the threshold under both — reddens both.

Outside this file, M1 reddens 6 more (two senescence flow tests, the step's own unit test,
`the_vks_mutual_shading_regime_is_modelled_not_merely_avoided`, the `open_season` golden, and
the kinetics 2×2 this item descends from); M2 reddens 10.

⚠ **The most informative result is a mutation that does NOT redden.**
`open_season_canopy_is_physical` — the `5.0 < peak < 8.0` gate itself — stays **green under
M1**: delete the loss entirely and the band this item was taken to investigate does not
notice. That is FINDING 1 and FINDING 2 arriving from a third direction, and it is the
cleanest single statement of what was measured here: **at the frozen params the band and the
loss are independent; the coupling between them is entirely off-frozen-point.**

---

## What this changes, and what it does not

**Nothing is proposed and no decision is discharged.** The five decisions on the September
direction plan's list are untouched.

Two things it *does* change:

1. **Retire the phrase "the band the tree passes is held by a term measured INERT"** — the
   headline of `log/temperature-kinetics.md` and of the commit that landed it. The term is
   inert at exactly one point, the frozen one, and on exactly one observable. Off that point
   it is the largest single source of the contract's tolerance, and it is what stops the two
   open-field gates being duplicates of each other. The sentence is right about the frozen
   tree and wrong about the tree.
2. **§2.1 item 1 of the direction plan is ranked on a retired number.** Its "+75 %" is a
   no-clamp figure; the tree it would be quoted about answers +4.1 %. The retrieval it asks
   for (what leaf population [B] Table 19 reports) is unaffected — but its *leverage* is not
   what the plan says, and the ranking in §4 was computed from it.
