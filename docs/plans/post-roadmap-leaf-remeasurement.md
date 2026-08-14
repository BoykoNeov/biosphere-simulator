# The parked leaf mechanism, re-measured at the step that would ship it

**DIAGNOSED 2026-08-14. NOTHING BUILT.** `git diff src/` empty on `main`; no golden
regenerated, no manifest touched. All measurement in a git worktree
(`M:/claud_projects/temp/leaf-worktree`, branch `leaf-expansion-rebase`), so `main` was
never checked out of. Probes: `M:/claud_projects/temp/leaf-remeasure/`. **The ship/refuse
decision is the user's and is untaken.**

## Charge

`docs/log/leaf-expansion.md` finding 11 left the mechanism at:

> **not refuted, route identified, evidence base pending re-measurement at the step that
> would ship it.**

and the direction plan queued it to ride the step ceremony. The step shipped
(`dt = 1 → ¼`) and the ceremony **did not carry it**. So the re-measurement finding 11
named is now the whole of what stands between this mechanism and a decision.

⚠ Finding 11's own warning governs this document: *"the reversal is not an endorsement —
read finding 9 in the mirror."* What cleared at the finer step is the **guard**. Every
measurement that got the mechanism accepted was Euler-at-one-day, and re-measuring them is
the point.

---

## Method — separate rebase error from step error, and pay one run to know which

The branch was 35 commits behind. Textual churn on its 9 files was one file (`season.py`,
19+/4−) and the rebase was clean — but **that measures conflict, not correctness**. The
branch's 847 lines were written when a step *was* a day, and at `dt = ¼` any rate written
per-step rather than per-day runs 4× fast **with nothing to make it red**, because at
`dt = 1` the two are the same integer. That is the conflation class the step unfreeze needed
four passes to find.

So: rebase in the worktree, patch `BIO_DT = 1.0 / STEPS_PER_DAY = 1` **there**, and
reproduce the branch's own anchors before flipping. A discrepancy at the old step is a
rebase bug; a discrepancy only after the flip is the step.

## FINDING 1 — the rebase is clean, and the mechanism was written step-independently ON PURPOSE

At the patched `dt = 1` every recorded anchor came back **exactly**:

| anchor | recorded | reproduced |
|---|---|---|
| `open_season` peak LAI | 5.2533 | **5.2533** |
| frozen derived peak LAI | 5.4624 | **5.4624** |
| Greenwood peak W | 13.6717 | **13.6717** |
| rationing, all eleven scenarios | 0 | **0** |
| leaf thickness, max | 1.20–1.21× | **1.2054–1.2144×** |
| node/source unclamped share | 39–45 % | **39–45 %** |

The audit agrees with the measurement: `main_stem_nodes` reads the thermal-time **state**,
not a step counter; `node_area_growth_rate` deliberately **differentiates** [F]'s day-over-day
difference (*"a day-over-day difference is a `dt = 1` object"*) to hold the aux channel's
`dt`-independence contract; `evaluate` returns `rate·dt`. The one deliberate exception, the
envelope projection, is documented as such. **Nothing in the 847 lines carries a step-unit
bug** — which is worth recording precisely because it was the thing most likely to be wrong.

## FINDING 2 — the evidence base at the shipped step: three of five pieces moved

| evidence | `dt = 1` | `dt = ¼` | verdict |
|---|---|---|---|
| `open_season` peak LAI | 5.2533 | **5.5572** | moved +5.8 % |
| Greenwood peak W (gate < 14.4248) | 13.6717 (5.2 % margin) | **14.0581 (2.5 %)** | margin halved |
| leaf thickness max (ceiling 1.1765) | 1.2054–1.2144 | **1.1817–1.1836** | **improved** |
| node/source unclamped share | 39–45 % | **38–46 %** | **holds** |
| rationing | 0 | **0** | holds ⚠ Euler-blind |

⚠ **The thickness result was PRE-REGISTERED and held.** The overshoot exists because the
envelope reads leaf carbon at *step entry* while the delta lands after it, so it should
shrink ~4× at a quarter step. Excess over the ceiling went 0.0384 → 0.0058–0.0076, i.e.
**4–6×**. The mechanism's own recorded exposure (iii) is now ~0.5 % rather than ~3 %.

⚠ **The share was re-measured LIKE-FOR-LIKE, and that took work.** A first pass counted
*which branch was entered* (67.6 %, step-invariant) and would have been reported as a change
against a recorded number that means *which branch decided* — a different quantity with a
different denominator. The original probe's exact tally (`floor%`/`free%`/`cap%`, all steps,
both branches) was re-run against the worktree instead, reproducing 39–45 % at `dt = 1`.
**A number compared against a differently-defined number is the locus error this project
keeps recording**; the coincidence that the wrong measure landed at the top of the right
range is exactly how it would have survived review.

## FINDING 3 — ⚠⚠ THE CONTROL INVERTS THE READING: THE MECHANISM CONVERGES TOWARD INERT, AND THE MARGIN EROSION IS `main`'S

Peak W looked like the mechanism eating a science gate's margin. It is not. Run the frozen
form (`plant_density=None`) through the same refinement:

| `dt` | mechanism LAI | frozen LAI | mechanism W | frozen W |
|---|---|---|---|---|
| 1 | 5.2533 | 5.4624 | 13.6717 | 13.9391 |
| ½ | 5.5139 | 5.5360 | 13.9774 | 14.0521 |
| **¼ (shipped)** | **5.5572** | **5.5719** | **14.0581** | **14.1077** |
| ⅛ | 5.5818 | 5.5896 | 14.1023 | 14.1350 |

Two things follow and they point opposite ways:

1. **The mechanism sits BELOW the frozen tree on both gated observables at every step.** At
   the shipped step it *improves* the Greenwood margin (2.5 % vs the frozen tree's 2.20 %).
   The margin narrowing is the **step's**, and it is already shipped — see finding 5.
2. **But the gap is closing fast.** The mechanism's effect on `open_season` peak LAI runs
   −3.8 % → −0.40 % → **−0.26 %** → −0.14 %. On the open field this mechanism is converging
   toward doing **nothing**, and the step that would ship it is most of the way there.

⇒ **The honest summary is not "the evidence base survived".** It is: *the mechanism is
becoming inert exactly where its headline evidence was measured, while remaining substantial
where it was not.* It still raises canopy **1.27–1.32×** over the derived form on the three
chambers and 1.62× on `day_neutral` — carbon-limited runs, where [F]'s node branch has
something to add — and the node branch still decides its 38–46 % of days.

⚠ **One corroboration weakens and must not be quoted unqualified.** The record's *"[F]'s own
working model reports max LAI 5.15; ours is 5.2533 (+2.0 %) with the envelope"* reads
**+7.9 %** at the shipped step. But the frozen tree is +8.2 %, so the envelope still moves us
toward [F] — the *tree* moved away from [F], not the mechanism. Quoting the +2.0 % today
would be quoting a number from a tree that no longer exists.

## FINDING 4 — the RK4 blocker is gone, and that is NOT the evidence

`tests/test_decade_stability.py` returned **31 errors** (`ArbitrationError`, `scale_f =
0.9668362828371428`) when the mechanism was parked. At the shipped step, on the rebased
branch: **34 passed**.

⚠ **Recorded as a fact, not as an argument.** Finding 11 already answered this in advance:
shrinking `dt` until `check_no_overdraw` goes quiet is finding 9's error with the sign
flipped, and *"what cleared at the finer step is the guard."* The evidence is finding 2 and
finding 3; this row is the removal of an obstacle, not a reason.

## FINDING 5 — ⚠ A STALE NARRATIVE ON `main`, FOUND WHILE MEASURING SOMETHING ELSE

Independent of leaves entirely, measured on `main`: `open_season`'s frozen Greenwood peak W
is **14.107660**, a margin of **2.20 %** under the 14.4248 crossing. The comment in
`tests/test_nitrogen_form.py` says **13.939142** and **3.4 %** — the `dt = 1` values.

**Nothing went red**, because the pin is a band (`13.9 < peak_w < 14.4248`) that spans both
and a ratio guard set at 0.85 while the quantity lives at 0.978. The step ceremony re-pinned
every test that *failed*; this one passed, so it was never re-read.

⇒ **A characterization pin wide enough to survive the change it characterizes is not a
tripwire, it is decoration.** Shipped separately (its own commit): the narrative is
re-measured and the ratio guard tightened 0.85 → 0.97, ~1 % below today's value. Tightening a
tripwire toward the measured value is the opposite of the retune the freeze discipline
forbids — the sourced half, the 14.4248 crossing, is untouched. ⚠ Also fixed there: the
test's docstring quoted a band (`12.0 < peak_w < 13.0`) two revisions dead, *in the file whose
subject is prose going stale.*

## FINDING 6 — the CO₂ band prices the mechanism, which is what it was built for

The compensation-point band landed earlier the same day
(`post-roadmap-co2-compensation-band.md`). Run against the mechanism:

| scenario | `main` | with the mechanism | Δ |
|---|---|---|---|
| `sealed_chamber` | 1.2579× | 1.2056× | −4.2 % |
| `perennial_chamber` | 1.2359× | 1.1810× | −4.4 % |
| **`consumer_chamber`** | 1.2186× | **1.1718×** | −3.8 % |

**All five still clear the floor**, and a ~30 % larger chamber canopy costs ~4 % of the
band's margin. ⇒ the band answered a question about a *candidate* mechanism in the contract's
own units, one day after being written, which is precisely the job it was proposed for. ⚠ The
tightest is `consumer_chamber` at 71.56 ppm against 61.07 — the scenario finding 2 of the
band record showed nobody had measured.

---

## What shipping would cost (not paid, and larger than the record prices it)

| | |
|---|---|
| **A test suite that does not exist** | the branch adds `leaf_area.py` (392 lines) and **no test file** — its entire evidence base lives in probe scripts outside the repo, now demonstrably measuring a tree that no longer exists |
| Goldens | 7 biosphere + the greenhouse-bearing station ones |
| Manifests | biosphere (`aux_set`, `flow_set`, param files) + station |
| Native port | a hand-mirrored new aux process, plus a tier-band re-measure |
| Contracts | the same three the step unfreeze paid |

That is **larger than the ceremony just run for the band**, and the leaf record does not price
it.

⚠ **And one tripwire it would land close to, checked rather than left to be discovered.**
Finding 5 tightened `test_open_season_peak_w_margin_to_the_crossing`'s ratio guard to 0.97,
~1 % below the *frozen* tree's 0.978. The mechanism's own ratio at the shipped step is
`14.0581 / 14.4248 = 0.9746` — it clears, by **0.47 %**. So shipping the leaf branch does not
trip the guard, but it halves the headroom to it, and the guard was set on the frozen tree
one commit earlier. Recorded here because a threshold set while measuring one thing and met
while shipping another is exactly the pairing nobody re-reads.

## The decision, stated as options rather than taken

1. **Ship it.** Pay the ceremony above. Buys: the chamber canopies (1.27–1.32×), the node
   branch's 38–46 % of days, a slightly better Greenwood margin than `main` has today.
2. **Refuse and revert.** The open-field case has converged to ~0.26 % and is heading to
   ~0.14 %; a mechanism that is inert where its evidence was taken is a defensible refusal
   — ⚠ but it is **not** inert in the chambers, which is where this project's subject
   actually lives, so the refusal must be argued on the chambers, not on `open_season`.
3. **Ship it behind the chambers only.** Not currently expressible — `plant_density=None`
   is per-scenario, not per-condition, and making it conditional on carbon limitation is a
   new mechanism nobody has priced. Recorded so it is not rediscovered as free.

⚠ **What this document does NOT establish.** That the mechanism is *correct* at the shipped
step — only that its own recorded evidence was re-measured there. The science judgements in
`docs/log/leaf-expansion.md` (findings 4–8: the envelope's provenance, the locus exposure,
[E] vs [F] on drought sensitivity) are untouched by this work and stand exactly as recorded.

## Housekeeping — declared, not left

The worktree `M:/claud_projects/temp/leaf-worktree` and the branch `leaf-expansion-rebase`
(a clean rebase of `leaf-expansion-blocked` onto `main`) are **kept deliberately**, so a
decision does not have to redo the rebase. `step.py` is restored to the shipped `BIO_DT =
0.25` there and `git status` is clean. ⚠ `leaf-expansion-blocked` is unchanged and still
points at `cb668f6`; the rebase is a *second* branch, not a rewrite of the parked one.
