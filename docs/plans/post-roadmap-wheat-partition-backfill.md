# The winter-wheat partition backfill — TAKEN and REFUSED (2026-08-11)

**Verdict: REFUSED.** The frozen winter-wheat partition table stays uncited. The cited
replacement was transcribed, derived, and **measured**, and it drives the canopy to
**peak LAI 2.201** against a contract-standing band of **5.0–8.0** (real wheat). Nothing
in `src/` moved; `git diff src/` is empty. The table, the derivation and every number
below are kept here so nobody re-attempts this expecting a different answer.

The successor work is **root functional coupling**, not a citation hunt. See "What this
actually blocks" below.

## Why it was taken

`docs/plans/post-roadmap-stem-reserves.md` named the uncited partition table *"the real
successor"* — the largest uncited object left in the frozen biosphere reference. The
potato exercise then established that [E] Table 18 carries cited, per-species, DVS-keyed
partition curves, and `params/crops/potato/allocation.yaml` records in its own header that
this was *"NOT a licence to backfill wheat's table from the same source, which would be an
unfreeze with its own ceremony."* This is that ceremony, run to its conclusion.

⚠ **One premise of the framing was wrong, and it was wrong before any measurement.** The
backfill was expected to unblock stem-reserve remobilization. It cannot. Allocation
distributes `DMI = Yg·max(0, GASS − MRES)`, which is **≥ 0**, among fractions in `[0,1]`;
no combination of fractions can move mass *out* of an organ. The derived table's stem
share **falling** 0.702 → 0.623 between DVS 0.77 and 0.95 is the stem growing more slowly,
not the stem feeding the grain. `post-roadmap-stem-reserves.md`'s refusal — *"a data table
is not a model form"* — survives this exercise untouched. **The real lead is recorded at
the bottom of this file**, and it is not a partition table.

## The source, and the transcription

[E] Table 18, **"Wheat, winter"**, PDF page 111 = printed page 91, **read off a rendered
page image** (`pdftoppm -r 600`), never the text layer. Non-negotiable, and this entry
demonstrates why: `pdftotext` renders its CASST line as

```
PUNCTION CASST= 0:,0:53:0.33,0.5; 0.53,0:75,1.,1.; 2.1%
```

The page image gives:

```
CALVT = 0.,0.90, 0.33,0.85, 0.43,0.83, 0.53,0.75, 0.62,0.56, 0.77,0.20,
        0.95,0.09, 1.14,0.05, 1.38,0., 2.1,0.
CASTT = 0.,0.10, 0.33,0.15, 0.43,0.17, 0.53,0.25, 0.62,0.44, 0.77,0.80,
        0.95,0.64, 1.14,0.62, 1.38,0., 2.1,0.
CASST = 0.,0.5, 0.33,0.5, 0.53,0.75, 1.,1., 2.1,1.
```

⚠ **Weaker provenance than potato's rows.** [E] gives this entry **no cultivar label and
no per-row attribution** — it is "Wheat, winter" plain, where the potato rows are
"cv Mara (late)" traced to van Heemst (1986). Recorded because it would have gone into the
frozen file as a citation of equal standing, and it is not.

## The derivation and the table it produced

Same conversion as the potato file, at the same source table:
`fr = 1 − CASST`, `fl = CASST·CALVT`, `fs = CASST·CASTT`,
`fo = CASST·(1 − CALVT − CASTT)`, evaluated at the **union of the three curves' own
knots** — `{0, 0.33, 0.43, 0.53, 0.62, 0.77, 0.95, 1.0, 1.14, 1.38}`, no invented knot.

**The rounding rule, stated rather than left to luck.** Exact rational arithmetic, written
at 12 decimal places, residual `(1 − Σ)` added to the **largest** fraction of the row. Both
halves are load-bearing: the loader's tolerance is `1e-9` and **six decimals — potato's
precedent — MISSES it at dvs 0.62** (sum 1.000001); and putting the residual on a fixed
column invents a `1e-12` grain share at 0.62 where [E] has `CALVT + CASTT = 1.000000`
exactly. Worst deviation from the exact value: **7.0e-13**.

| dvs | fl | fs | fr | fo |
|---|---|---|---|---|
| 0.00 | 0.45 | 0.05 | 0.5 | 0.0 |
| 0.33 | 0.425 | 0.075 | 0.5 | 0.0 |
| 0.43 | 0.51875 | 0.10625 | 0.375 | 0.0 |
| 0.53 | 0.5625 | 0.1875 | 0.25 | 0.0 |
| 0.62 | 0.446808510639 | 0.351063829787 | 0.202127659574 | 0.0 |
| 0.77 | 0.175531914894 | 0.702127659574 | 0.122340425532 | 0.0 |
| 0.95 | 0.087606382979 | 0.622978723404 | 0.026595744681 | 0.262819148936 |
| 1.00 | 0.079473684211 | 0.634736842105 | 0.0 | 0.285789473684 |
| 1.14 | 0.05 | 0.62 | 0.0 | 0.33 |
| 1.38 | 0.0 | 0.0 | 0.0 | 1.0 |
| 2.00 | 0.0 | 0.0 | 0.0 | 1.0 |

(The 2.00 row relabels [E]'s unreachable 2.1 knot to our DVS cap, exactly as potato's does.
Every row sums to exactly 1; conservation is exact by linearity, so the every-step
conservation gate could never have been the thing that caught this.)

## What the cited table says that the placeholder did not

* **Roots STOP at anthesis** (`fr = 0` from DVS 1.0). The placeholder kept them at 0.20 at
  anthesis and 0.10 at maturity.
* **The stem peaks hard and early** — 0.702 at DVS 0.77 against the placeholder's 0.50
  maximum. The placeholder's straight line averaged the stem-elongation phase away.
* ⚠ **The grain starts BEFORE anthesis.** [E]'s `CALVT + CASTT` is exactly 1 up to DVS
  0.77 and drops below it after (0.73 at 0.95), so `fo` is strictly positive past 0.77 — a
  quarter of a development unit before flowering. The frozen header asserts *"FO is 0
  before anthesis"*. **That sentence is the placeholder's shape, not a science invariant**
  — and it is worth knowing that **no test encodes it**; it lives only in file prose. It
  stands only because the refusal keeps the placeholder.

## The measurement that refused it

Contract-standing gates only (`-m science_gate`, the `science_bands` + `liveness_floors`
that gained standing 2026-08-09), run **without `-x`** so the result is a survey and not a
first-failure:

> **9 of 10 pass. Exactly one fails**: `test_frozen_open_season_canopy_is_physical`,
> peak LAI **2.2005** against `5.0 < peak < 8.0`.

Everything else clears: the Van Keulen & Seligman mutual-shading ceiling (`peak < 6.0`,
trivially), the Greenwood peak-W band (`< 14.4248 t/ha` — the crop got *smaller*, so the
margin widened), and **all four liveness floors** including both decade-stability
attractors. The whole suite shows 26 red, but the other 25 are pinned diagnostic
measurements that move under *any* allocation change; they are not gates. **Distinguishing
the two is the whole point of the 2026-08-09 standing work, and this is its first use in
anger.**

Miss factor on the one that matters: **2.36×**. Not marginal, not a tolerance question.

## The cause, isolated

Four runs of `open_season`, one year, Euler, patching only the partition table:

| table | peak LAI | final LAI |
|---|---|---|
| placeholder (frozen, uncited) | **5.191** | 3.193 |
| cited [E] Table 18 | **2.201** | 0.803 |
| cited, but the placeholder's ROOT share (shoot split rescaled) | **5.339** | 1.642 |
| cited, post-0.53 leaf collapse removed (control) | 13.259 | 13.259 |

**The entire failure is the root share.** Swapping only the roots back — keeping [E]'s
early leaf peak, its hard stem takeover, and its pre-anthesis grain — **recovers the
canopy above the floor** (5.339 vs the placeholder's 5.191). The late leaf collapse is not
the culprit; the control that removes it overshoots to 13.3, confirming it is what stops
the canopy, but it is not what starves it.

And the divergence is **concentrated in the first third of development**:

| DVS | placeholder `fr` | cited `fr` |
|---|---|---|
| 0.00 | 0.35 | **0.50** |
| 0.33 | ~0.30 | **0.50** |
| 0.50 | 0.275 | 0.2875 |

By DVS 0.5 the two root shares have **nearly converged**. The whole effect is bought in
DVS 0–0.33 — the phase where leaf area compounds into light capture into more leaf area.
**Peak canopy is set by assimilate diverted during the compounding phase; diversions after
it are nearly free.** That is the general result, and the pre-anthesis grain is its own
proof: `fo` opening at 0.77 costs essentially nothing, because 0.77 is past the compounding
phase.

## ⚠ The refusal does NOT make the placeholder better science

Say this out loud, because the shape of the outcome invites the opposite reading. Against
the **oracle's own implied root share** (WOFOST winter wheat, `TWRT` increments as a
fraction of new biomass — facts only, ruling B, a diagnostic and never a fit target):

| window | WOFOST | placeholder | cited [E] |
|---|---|---|---|
| DVS 0–0.2 | 0.474 | ~0.34 | **0.50** ← [E] is closer |
| DVS 0–0.33 | 0.380 | ~0.325 | 0.50 ← WOFOST sits *between* them |
| DVS 0.5–1.0 | 0.067 | ~0.24 | **~0.115** ← [E] is closer |

**Neither table dominates.** WOFOST agrees with [E] that root share *starts* near 0.5; it
disagrees about how fast it falls, dropping to 0.170 by DVS 0.33–0.53 where [E]'s coarse
four-knot CASST is still declining linearly to anthesis. (Caveat: `TWLV` is *live* leaf, so
late windows understate leaf growth and flatter the late root share.)

**The placeholder passes the band because it was fitted to it, not because it is more
faithful.** The honest statement of the outcome is: *an uncited, fitted value was retained
over a cited one because the cited one fails an independent cited gate, and the reason it
fails is a missing mechanism on our side.*

## What this actually blocks — the successor is root functional coupling

`ROOT_C` is read in exactly one place outside plumbing: `nitrogen.py:256`, where it is
summed into a biomass total for nitrogen **demand**. **There is no uptake function.** Root
carbon buys nothing — and in `open_season`, nitrogen and water are non-limiting anyway, so
there is nothing for it to buy. Senescence (`rdr_root = 0.01/day`) then bleeds it away.

So carbon sent below ground is, in our model, **dead weight by construction**. A source
that allocates 50 % of early assimilate to roots is describing a plant whose roots pay for
themselves; ours cannot. The frozen table's canopy physicality **rests on a fitted root
share compensating for a missing mechanism**, and that is the finding this exercise
bought.

This is a **LOCUS** failure in the project's established sense — the citation is faithful
and the value is transcribed correctly; what is wrong is the model context it lands in.
Consistent with `bucket3-scope-c-citation`'s rule that citations fail on locus, not
transmission.

**Do not re-attempt this backfill until roots do work.** It will fail the same way.

## ⚠ A committed claim this measurement corrects — the potato canopy attribution

`post-roadmap-potato-crop.md` and `tests/test_potato_crop.py` record the potato canopy
shortfall and the tuber over-fill as **"one cause, two symptoms"** — both downstream of the
early tuber onset. Wheat has now reproduced a canopy shortfall of the same magnitude class
with a storage organ that opens at DVS **0.77**, not 0.15, and traced it entirely to roots.
That made the potato attribution worth testing rather than assuming. Same harness, same
scenario, patching only the table:

| potato intervention | peak LAI | day | gap to oracle (8.885) | closed |
|---|---|---|---|---|
| as committed (control) | 3.184 | 34 | 2.79× | — |
| **tuber held to anthesis** | **5.406** | 31 | 1.64× | **39 %** |
| roots from the fitted wheat | 3.478 | 31 | 2.55× | 5 % |
| roots removed entirely (extreme control) | 8.067 | 31 | 1.10× | 86 % |

**The attribution is supported in direction and overstated in strength.** The early tuber
onset is real and is the larger of the two terms *for potato* — but it accounts for
**~39 %** of the canopy gap, so "one cause" is too strong. Corrected in the potato doc and
in the test comment rather than left to rot.

**The two crops also invert which term dominates** — roots for wheat, the tuber for potato
— which is what makes the unifying statement the right one to carry forward: it is not
"roots" and not "the storage organ", it is **early diversion, whatever the organ**.

⚠ **Deliberate, stated deferral:** the 39 % is recorded in prose here and not pinned as a
test. The repo's own memory (`direction-gate-built`) says a claim about a measured quantity
is not checkable by re-reading it. Pinning it needs the four-way harness lifted into
`tests/`, which is real work and outside this exercise's scope. Flagged, not silently
skipped.

## The stem-reserve lead — a SEPARATE work item, and the reason the thread was picked

The user chose this thread partly because it was expected to unblock stem reserves. It does
not (see "Why it was taken"). But the `pdftotext` sweep used to locate Table 18 surfaced
the actual blocker on **[E] p. 93** (PDF 113), §3.2.4 "Formation of shielded reserves":

> "A simple way to deal with the formation of shielded reserves is to assume that **a
> certain fraction of the increase in stem weight will be available for redistribution
> after flowering** (Listing 3 Lines 17, 35). This fraction is assumed to consist only of
> starch. **Some data on the magnitude of the remobilizable fraction are given Table 7**."

That is a **model form, plus a magnitude table, plus two CSMP listings** (Listings 3 and 4,
lines 32–33, 37–38 for the accumulation half) — in a source already on the shelf and
already first-hand for four other rows. `post-roadmap-stem-reserves.md` refused on *"a data
table is not a model form"*; this is the form.

**Not folded into this exercise.** Recorded as the potato lesson repeating itself: **a
recorded blocker is dated — re-check the artifact.** It has now caught us twice.

> ⚠⚠ **RETRACTED 2026-08-12 — the two paragraphs above are wrong, and dated rather than
> deleted because the error is the finding.** §3.2.4 was **not** a discovery: the
> stem-reserve exercise of 2026-08-10 had already quoted this exact passage
> (`post-roadmap-stem-reserves.md` §"§3.2.4 'Formation of shielded reserves' (p. 93)"),
> **implemented** the form it programs, and **measured** it — its own comparison table
> lists *"§3.2.4 growth fraction — what [A] programs (Listing 3 Lines 17, 35)"* at stem
> shape **0.985×** and harvest index **1.151×** the oracle. `[A]` there and `[E]` here are
> the same book. So *"a data table is not a model form"* was a statement about **Table 7**,
> never a claim that the form was missing; this section read it as the latter and filed a
> re-discovery as a discharged blocker. **The lesson inverts:** the artifact re-checked was
> the *source*, and the thing that was stale was **our own record of what we had already
> done**. Re-read the predecessor's measurement table before believing a successor's
> framing of it. What genuinely blocks stem reserves is unchanged and is in that document:
> the book's programmed form **overshoots** the reference harvest index by 15 %, the form
> that hits it is one **we constructed** with no listing line in the source, and the one
> number that moves anything (`fstr` = 0.40, Table 7's wheat row) is **CABO unpublished**.
> Neither is a missing-form problem, so nothing here unblocks it — it is a provenance
> judgement, and the verdict is the user's.

## Sources

* **[E]** Penning de Vries, F.W.T., Jansen, D.M., ten Berge, H.F.M. & Bakema, A. (1989),
  *Simulation of Ecophysiological Processes of Growth in Several Annual Crops*, Simulation
  Monographs 29, PUDOC/IRRI. ISBN 90-220-0937-2. **Table 18, p. 91** ("Wheat, winter");
  **p. 88** (the patterns are derived from observed biomass increments); **p. 93, §3.2.4 +
  Table 7** (the stem-reserve form). Page images, not the text layer.
* **The oracle (facts only, never params)**: WOFOST winter wheat, `tests/oracle/
  winter_wheat_reference.json`. Used as a third witness on root share; never a fit target.
