# `specific_leaf_area` — what leaf population Table 19 reports

**The September direction plan's §2.1 item 1, taken 2026-09-06.** A retrieval, not a build:
the constant is cited and its value is not in question. What was owed is the *population* the
cited page describes, because the plan held that the same number read as a young-leaf or a
mature-leaf value spans a large range of peak LAI — and if the page turned out to describe an
age-specific population, a development-keyed form would be owed.

The source is on the shelf: `sources/Simulation Of Ecophysiological Processes Of Growth In --
F_ W_ T_ Penning De Vries, D_ M_ Jansen, H_ F_ M_ Ten Berge.pdf`. This item is therefore
**not** the blocked retrieval the FvCB page check is; nothing had to be fetched.

## 1. The question, restated before it was answered

`canopy.yaml` carries `specific_leaf_area: 23.53 m²/kg`, cited as `[B], Table 19 p.100,
'Wheat, winter' (425 kg/ha per unit LAI → 10000/425)`. The direction plan asked: *what leaf
population does that figure describe?* — offering **young** or **mature** as the two readings,
and predicting a form build if the answer were age-specific.

**Prediction written before the page was read:** the caption would name one of the two, and
the likely answer was a season- or canopy-average, which would close the item with no build.

## 2. Method

Page renders at 7× (`M:\claud_projects\temp\sla-page-check\`), never the text layer — this
book's OCR is mangled and the tree has a standing rule about it (`docs/log/canopy-magnitude.md`).
The PDF's folio offset is **printed page = PDF index − 19** (fixed off the page number printed
at the foot of PDF index 117 = printed 98).

Read before rendering, per the repo's own recurring lesson: `docs/log/` already carried three
close reads of this book's leaf-area section, and one of them (`leaf-expansion.md`) had already
quoted Table 20's winter-wheat row. **Two of the four findings below are checks on our own
records rather than new retrieval.**

## 3. What the pages say (all verbatim off renders)

**Table 19's caption, printed p. 100:**

> Table 19. The specific leaf weight constants for different crops are **average values for
> the entire canopy**. Leaf refers to leaf blades (one surface only) excluding petioles and
> leaf sheaths. The specific stem weight is an average for the growing season and includes
> stem proper, petioles, branches and leaf sheaths.

**Printed p. 99, the sentence that sets the constant up:**

> …this is a fair approximation provided that its value is **measured at a proper time, such
> as at the end of the phase when most assimilates go to the leaves** (e.g., development stage
> 0.5 for rice, Figure 34). Table 19 presents values of specific leaf weight constants for
> different crops **at about this stage**.

**Printed p. 98, §3.3.3 — the method Table 19 serves:**

> The simplest is to assume that the specific leaf weight is a crop characteristic and that it
> is constant in time and throughout the canopy. This value is called the specific leaf weight
> constant. **Leaf area is determined by dividing the weight of live leaves by the specific
> leaf weight** (e.g., van Keulen et al., 1982).

**Printed p. 99, the alternative:**

> A more realistic approximation … takes into account that new leaves formed early in the life
> of the plant are thinner than leaves formed later. The specific leaf weight **of new leaves**
> is then found by multiplying the specific leaf weight constant with a factor that depends on
> the development stage … **this method … enhances the simulated date of canopy closure by
> several days (or even as much as two weeks)** … **However, the effect on leaf biomass and on
> final yield tends to be small.**

**Table 20, printed p. 102, the `Wheat, winter` row** (the DVS-keyed factor, first of each pair
is the development stage):

> `FUNCTION SLT = 0.,1., 0.33,1.1, 0.36,1.06, 0.43,1.5, 0.53,1.05, 0.62,1., 0.77,0.85,
> 0.95,1.07, 1.14,1., 2.1,1.`

## 4. Findings

**FINDING 1 — the question contained a false dichotomy, and the caption says so in five
words.** Table 19's constants are *"average values for the entire canopy"*. Neither reading the
plan offered is the page's; there is no young-leaf and no mature-leaf column, and the ±35 %/
+75 % span the item was ranked on was never a choice the source presented.

**FINDING 2 — the constant carries a TIMING qualification the file does not record, and it is
the more interesting half.** It is not a season average: it is the canopy average *sampled at
about the stage when most assimilates go to the leaves* (~DVS 0.5). So the number is a
canopy-wide average **taken at one moment**, used as though constant — which is exactly the
approximation p. 98 licenses, stated with its own precondition.

**FINDING 3 — our LOCUS is the book's own, and this is what makes the answer a discharge
rather than a finding.** `science::leaf_area_index` is `leaf_carbon · sla_per_mol_c /
ground_area` — LAI from the **standing live leaf pool**, the single consumer of the constant
(`flows.rs:150, 476, 970, 2935`; `readouts.rs:278`). That is p. 98's *"dividing the weight of
live leaves by the specific leaf weight"*, verbatim. The value, the population and the
application point agree; nothing is owed.

**FINDING 4 — the book's development-keyed alternative is a DIFFERENT LOCUS, not a re-reading
of this constant.** `SLT` multiplies the constant for **new** leaves, which needs leaf-cohort
state; this tree has none by design (P2: *LAI is derived, not stored*). So adopting it is a
form build with a new state variable, and it is not what "read the constant the other way"
would have been.

**FINDING 5 — the winter-wheat `SLT` row does NOT have the shape the page's own narrative
describes, and this retires the plan's second argument for the item.** p. 99 motivates the
method with *"new leaves formed early … are thinner"*; winter wheat's row **starts at 1.0**,
rises to **1.5** at DVS 0.43, dips to **0.85** at 0.77 and returns to 1.0 — a bounded,
non-monotone wiggle, not a thin→thick ramp. (Spring wheat's row *does* have the ramp: 0.67 up
to DVS 0.55, then 1.0.) The direction plan argued that a "late-anchored reading" would also move
the pinned *"LAI peaks after anthesis"* defect the right way (DVS 1.37 → 0.96); the source's own
curve gives that reading no support. ⚠ Precisely: that figure was measured 2026-08-15 by keying
the **scalar constant** through a harness, not computed from Table 20's row, and it is separately
retired as pre-clamp by `log/mutual-shading-tolerance.md`. **The citation-side argument for the
late anchor is what this finding removes — not the measurement.**

**FINDING 6 — a check on our own record, and it HOLDS.** `docs/log/leaf-expansion.md` FINDING 4
cites Table 20's winter-wheat row as a two-sided envelope of **[0.85, 1.50]**. Both extremes are
confirmed off the render. ⚠ What that record calls an *envelope* is the range of the curve, and
the curve's own endpoints are 1.0 — so "the crop's leaves are thin early" is not a claim the
row supports, and the parked mechanism's floor is a mid-season value.

## 5. Outcome

* **§2.1 item 1 is DISCHARGED. No form build is owed and no value moves.**
* One **provenance-only unfreeze** on `canopy.yaml`: the `source:` string gains the population
  and the timing, and the file records the alternative it is not taking and why.
* Free corrections to `crops/potato/canopy.yaml` (outside the manifest by non-recursion): stale
  sentences from 2026-08-15 and a wrong ISBN digit — see the record for which leg of its
  cross-check died and which one stands.

### 5a. The unfreeze prediction, written BEFORE the edit was saved

1. `manifest_writer`'s byte compare goes **RED** as soon as `canopy.yaml` is saved and *before*
   the manifest is regenerated — C7's gate, which `log/extinction-coef-bound.md` recorded firing
   on exactly this shape. **Observed, not assumed:** the suite is run at that point on purpose.
2. The regenerated manifest diff is **exactly one line**, `param_files/canopy.yaml`'s hash.
3. `regen_goldens` reports **0 of 20 would change** — no value moves, so no trajectory moves.
4. No science band and no liveness floor moves, for the same reason.

## 6. What this does NOT close

The **form** question is now well-posed rather than answered: whether this tree should model
leaf-cohort thickness at all is a build with a new state variable, priced by nothing here, and
[B] itself says the effect on leaf biomass and final yield "tends to be small" while the effect
on canopy-closure date is days to weeks. **Silence in the source would not have licensed a form;
neither does this — the page describes what its own constant is, not what our model should be.**
