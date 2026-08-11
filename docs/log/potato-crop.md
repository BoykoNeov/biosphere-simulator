## **Potato — the first SECOND species** (stage 1 of 2)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**COMPLETE 2026-08-11.** User chose "both, staged" + "potato only" after being shown the
shelf audit; the validated Python crop landed, the **Rust habitat mirror is stage 2,
deferred not dropped**. Until now the tree could hold exactly **one crop** —
`_carbon_context`/`build_plants` called every loader argument-free, so the frozen
`params/*.yaml` defaults *were* the crop (the day-neutral crop is not a counterexample; by
its own record it is the same wheat files with the gates off, "not a new param file"). **The
branch-deciding fact was a stale reading, and checking it was the whole exercise's hinge:**
the day-neutral plan recorded that only `lintul3_springwheat` shipped offline and the WOFOST
oracle needed an unlicensed param repo + network, which made *any* new species authored-only
by construction. False as of PCSE 6.0.13 — `pcse/tests/test_data/pcse_dump.sql` (loaded at
`import pcse`) ships **6 crops with complete parameter sets** (winter wheat, grain maize,
spring barley, **potato**, winter rapeseed, sunflower), calendars, 2 yr of grid-31031
weather, site, soil, **and `wofost_unittest_benchmarks`** (pre-computed daily
DVS/LAI/TAGP/TWSO for all 6, in `pp` **and** `wlp`); `pcse.start_wofost` is a supported API
over it. Licence discipline unchanged: output + provenance committed, **never a parameter
value**. **The seam:** `SeasonScenario.crop` + `loader.crop_param_set` — a crop is the
**eight plant-side param files resolved as a set**, with `overridden`/`shared`
*partitioning* the vocabulary so "potato shares wheat's photosynthesis" is a **pinned
assertion, not a comment** (the flattering-direction failure this guards).
Additive/default-preserving, proven: 7 goldens + both manifests byte-identical, 2146 green.
**Layout is a written decision, not a side-effect:** the freeze gate globs `params/*.yaml`
**non-recursively**, so `params/crops/potato/` does not trip it while a sibling file would —
correct scoping (a species is deliberately wired into no golden; freezing a set we
simultaneously call unvalidated would be incoherent), recorded in the loader header and
pinned. **Params clean-room from [E] Penning de Vries 1989 read OFF PAGE IMAGES** (its text
layer garbles exactly the table digits); the method **self-checked** — [E]'s winter-wheat
rows read back as the values our frozen file already carries. Cardinals come from [E] Table
13's potato row `7.,0.01, 18.,1.0, 29.,0.01` → `t_base` 7, `t_cap` **at the optimum** 18 (a
cap at the 29 °C upper zero would accumulate 22 °C·day/day where the source says development
has *stopped*). **Two rows land BETTER CITED than the frozen reference** — the DVS-keyed
partition table (wheat's is `TODO(cite)`, and `post-roadmap-stem-reserves.md` called it "the
real successor") and specific leaf area — **without touching the freeze**; backfilling wheat
from the same source remains an unfreeze with its own ceremony, deliberately not done.
Day-neutral **by the source's own marking** ([E] Table 12's daylength column is "–",
legended "not relevant"), not by analogy to the wheat gates. **THE HEADLINE FINDING IS A
DISAGREEMENT BETWEEN SOURCES, NOT A DEFECT:** [E]/van Heemst's curve starts filling the
tuber essentially at emergence (**day 7, DVS 0.19**) while WOFOST holds it at **exactly 0
until day 46 (DVS 1.03)** — two cited parameterizations of the same organ of the same crop
differing **qualitatively**, across ~39 d of a 96 d season. **One cause, two symptoms:** the
canopy (peak LAI **3.18 vs 8.88, 2.79× low**) and the tuber (**~14.3 vs 7.25 t/ha, 1.97×
high**) are both downstream of it — so "fix the canopy" and "fix the yield" are **one**
question, recorded because treating them as two invites two wrong calibrations. Nothing
moved (ruling B). Phenology (same-family, so it tests param choice) anthesis d33 vs 44,
maturity d108 vs 96; roots 0.36 vs 0.20 at DVS 0.5 — **opposite sign** to the day-neutral
crop's LINTUL3 root finding, so our allocation carries no standing bias. **A canopy
agreement is not a property of our canopy:** the day-neutral crop matched peak LAI within 2
%, this one is 2.79× low — **both survive only because neither was fitted**, the strongest
evidence yet that ruling B does real work. **The FvCB "gap" was a false alarm worth the
audit:** all 12 FvCB params are `TODO(cite)` tagged *"literature-typical C3"* — **never
wheat-specific** — so sharing them with a second C3 crop is exactly as justified as their
current use. General lesson: **before calling a shared parameter a compromise, check whether
it was ever specific to anything.** Where reuse *is* weak the **direction of the error is
written down**: potato's extinction coefficient is plausibly biased **LOW** (broad-leaved
planophile ~0.8–1.0 vs the carried 0.6). **All four advisor-flagged traps got pins, none was
a discovery:** `carbon_fraction` agreement now spans a **crop boundary** (potato overrides
canopy but not nitrogen) and is checked for every crop set; the seed-bank guard never fires
(tuber ≈ 430× the seedling); the **sealed chamber does NOT over-draw** on the larger crop
under either integrator (FvCB's Ci-shutoff self-limits before the backstop); and the `t_base
≠ 0` caveat came out **sharper and reversed** — our cap sits *at* the optimum, so above 18
°C we accumulate development where [E]'s response declines (a live warm-window over-run on
the Andalusian season, not a cold-window softness). **Honest residuals:** van Heemst (1986)
itself was **not opened** (the LOCUS check that caught Dunn 2011 has not been run on it);
[E] Table 19's potato row is a CABO **personal communication**; the partition derivation
multiplies-then-interpolates where [E] interpolates-then-multiplies (bounded by using the
union of all three curves' own knots; conservation exact by linearity); `ci_ratio` left at
the frozen 0.7 though [E] Table 22 offers potato 0.67–0.69.
`docs/plans/post-roadmap-potato-crop.md`
