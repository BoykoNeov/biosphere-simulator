## **The parked leaf mechanism, re-measured at the shipped step** (finding 11's named condition, discharged)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per
> work item. The fuller record is the plan doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**DIAGNOSED 2026-08-14, NOTHING BUILT; the ship/refuse decision is the user's and is
untaken.** `docs/plans/post-roadmap-leaf-remeasurement.md`; probes
`M:/claud_projects/temp/leaf-remeasure/`. `git diff src/` empty on `main`, no golden
regenerated, no manifest touched — every measurement in a **git worktree** so `main` was
never checked out of. `docs/log/leaf-expansion.md` finding 11 left the mechanism *"not
refuted, route identified, evidence base pending re-measurement at the step that would ship
it"*, and the step shipped without carrying it, so this re-measurement is all that stands
between the mechanism and a decision. **METHOD — separate rebase error from step error and
pay one run to know which.** The branch was 35 commits behind; textual churn on its nine
files was one file and the rebase was clean, but **that measures conflict, not
correctness**: its 847 lines were written when a step *was* a day, and at `dt = ¼` a rate
written per-step runs 4× fast **with nothing to make it red**, because at `dt = 1` the two
are the same integer. So the worktree was patched back to `dt = 1` and the branch's own
anchors reproduced **before** flipping. **FINDING 1 — THE REBASE IS CLEAN, AND THE
MECHANISM WAS WRITTEN STEP-INDEPENDENTLY ON PURPOSE.** At the patched old step every anchor
came back exactly: peak LAI **5.2533**, frozen derived **5.4624**, Greenwood peak W
**13.6717**, rationing **0** on all eleven, thickness max **1.2054–1.2144×**, unclamped
share **39–45 %**. The audit agrees with the measurement — `main_stem_nodes` reads the
thermal-time **state** not a step counter, `node_area_growth_rate` deliberately
*differentiates* [F]'s day-over-day difference (*"a day-over-day difference is a `dt = 1`
object"*) to hold the aux `dt`-independence contract, and `evaluate` returns `rate·dt`.
**Nothing in the 847 lines carries a step-unit bug**, which is worth recording precisely
because it was the thing most likely to be wrong. **FINDING 2 — THREE OF THE FIVE EVIDENCE
PIECES MOVED.** At `dt = ¼`: peak LAI **5.2533 → 5.5572** (+5.8 %); Greenwood peak W
**13.6717 → 14.0581**, i.e. the margin under the 14.4248 crossing **halved, 5.2 % → 2.5 %**;
leaf thickness max **1.2054–1.2144 → 1.1817–1.1836** against a 1.1765 ceiling — **improved**,
and improved by the **pre-registered** factor (the overshoot is the envelope reading leaf
carbon at *step entry*, so it should shrink ~4× at a quarter step; excess went 0.0384 →
0.0058–0.0076, i.e. **4–6×**, cutting the recorded exposure (iii) from ~3 % to ~0.5 %); the
unclamped share **39–45 % → 38–46 %, holds**; rationing **0**, ⚠ Euler-blind by
construction. ⚠ **The share took work to compare honestly**: a first pass counted *which
branch was ENTERED* (67.6 %, step-invariant) and would have been reported as a change
against a recorded number meaning *which branch DECIDED* — different quantity, different
denominator. The original probe's exact tally was re-run against the worktree instead,
reproducing 39–45 % at the old step. **A number compared against a differently-defined
number is the locus error this project keeps recording**, and the coincidence that the wrong
measure landed at the top of the right range is exactly how it would have survived review.
**FINDING 3 — ⚠⚠ THE CONTROL INVERTS THE READING: THE MECHANISM IS CONVERGING TOWARD INERT,
AND THE MARGIN EROSION IS `main`'S.** Peak W looked like the mechanism eating a science
gate. Run the frozen form through the same refinement and it moves the same way — frozen
peak W **13.9391 → 14.0521 → 14.1077 → 14.1350** across `dt` 1, ½, ¼, ⅛, against the
mechanism's 13.6717 → 13.9774 → 14.0581 → 14.1023. **The mechanism sits BELOW the frozen
tree on both gated observables at every step**, so at the shipped step it *improves* the
Greenwood margin (2.5 % vs the frozen tree's 2.20 %) — but **the gap is closing fast**: its
effect on `open_season` peak LAI runs **−3.8 % → −0.40 % → −0.26 % → −0.14 %**. ⇒ the honest
summary is **not** "the evidence base survived"; it is *the mechanism is becoming inert
exactly where its headline evidence was measured, while remaining substantial where it was
not* — still **1.27–1.32×** the derived canopy on the three chambers, 1.62× on
`day_neutral`, carbon-limited runs where [F]'s node branch has something to add. ⚠ One
corroboration weakens: the record's *"[F]'s own model reports max LAI 5.15; ours is 5.2533
(+2.0 %)"* reads **+7.9 %** today — but the frozen tree is **+8.2 %**, so the envelope still
moves us toward [F] and it is the *tree* that moved away. Quoting +2.0 % now would be
quoting a tree that no longer exists. **FINDING 4 — THE RK4 BLOCKER IS GONE, AND THAT IS NOT
THE EVIDENCE.** `test_decade_stability.py` went from **31 errors** (`ArbitrationError`,
`scale_f = 0.9668362828371428`) to **34 passed**. Recorded as a fact, not an argument:
finding 11 answered it in advance — shrinking `dt` until `check_no_overdraw` goes quiet is
finding 9's error with the sign flipped, and *"what cleared at the finer step is the
guard."* **FINDING 5 — ⚠ A STALE NARRATIVE ON `main`, FOUND WHILE MEASURING SOMETHING ELSE,
AND SHIPPED SEPARATELY.** Independent of leaves: `open_season`'s **frozen** Greenwood peak W
is **14.107660**, a margin of **2.20 %**, while `test_nitrogen_form.py`'s comment says
13.939142 and 3.4 % — the old-step values. **Nothing went red**, because the pin is a band
spanning both and the ratio guard sat at 0.85 while the quantity lives at 0.978; the step
ceremony re-pinned every test that *failed*, so this one was never re-read. ⇒ **a
characterization pin wide enough to survive the change it characterizes is not a tripwire,
it is decoration** — the narrative is re-measured and the guard tightened **0.85 → 0.97**,
which is the opposite of the retune the discipline forbids (the sourced half, the 14.4248
crossing, is untouched). ⚠ Also fixed: the test's docstring quoted a band two revisions dead
**in the file whose subject is prose going stale**. **FINDING 6 — THE CO₂ BAND PRICES THE
MECHANISM ONE DAY AFTER BEING WRITTEN.** Against the band landed the same day, all five
scenarios still clear the floor and every margin tightens ~4 %: sealed 1.2579 → 1.2056,
perennial 1.2359 → 1.1810, **consumer 1.2186 → 1.1718** (71.56 ppm against 61.07). A ~30 %
larger chamber canopy costs ~4 % of the band's margin — the band answering a question about
a *candidate* mechanism in the contract's own units, which is the job it was proposed for.
**WHAT SHIPPING WOULD COST, NOT PAID AND LARGER THAN THE LEAF RECORD PRICES IT**: a **test
suite that does not exist** (the branch adds 392 lines and **no test file** — its whole
evidence base is probe scripts outside the repo, now measuring a tree that no longer
exists), 7 biosphere + the greenhouse-bearing station goldens, both manifests, a hand-mirrored
new aux process in Rust with a tier-band re-measure — the same three contracts the step
unfreeze paid. **THE THREE OPTIONS, STATED NOT TAKEN**: ship and pay; refuse and revert —
⚠ but the refusal must be argued **on the chambers**, where the mechanism is *not* inert,
not on `open_season` where it has converged to ~0.26 %; or ship behind carbon limitation
only, which is **not currently expressible** (`plant_density=None` is per-scenario, not
per-condition) and is a new unpriced mechanism, recorded so it is not rediscovered as free.
⚠ **What this does NOT establish**: that the mechanism is *correct* at the shipped step —
only that its own recorded evidence was re-measured there. The science judgements in
`leaf-expansion.md` (the envelope's provenance, the locus exposure, [E] vs [F] on drought
sensitivity) are untouched and stand as recorded. **HOUSEKEEPING, DECLARED NOT LEFT**: the
worktree and the branch `leaf-expansion-rebase` are **kept deliberately** so a decision need
not redo the rebase; `step.py` is restored to the shipped `BIO_DT = 0.25` there and the tree
is clean; `leaf-expansion-blocked` is **unchanged** at `cb668f6` — the rebase is a second
branch, not a rewrite of the parked one.
