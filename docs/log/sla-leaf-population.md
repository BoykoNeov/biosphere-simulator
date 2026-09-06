## **Table 19's leaf population, RETRIEVED** (the question had a false dichotomy in it — and our locus turned out to be the book's own)

**RETRIEVED 2026-09-06** — the September direction plan's §2.1 item 1, the last open question
about `specific_leaf_area`. Plan of record: `post-roadmap-sla-leaf-population.md`. The source
was **on our own shelf** (`sources/…Penning De Vries…pdf`), so unlike the FvCB page check this
was never blocked on a fetch. Read at 7× off page renders, never the text layer (this book's
OCR is mangled); folio offset fixed as *printed = PDF index − 19* off the number printed at the
foot of a page, not assumed.

**THE ANSWER, AND IT REFUTES THE QUESTION.** Table 19's caption, verbatim: the specific leaf
weight constants *"are **average values for the entire canopy**"*. The plan asked whether the
frozen number is the **young**-leaf or the **mature**-leaf value; the page offers neither
column. So the ±35 %/+75 % span the item was ranked on was never a reading the source
presented — it was a choice between two things that are not in the table. ⚠ *A question can be
answered by the page refusing its premise, and that is a finding, not a null result.*

**THE HALF NOBODY ASKED FOR, AND IT IS THE INTERESTING ONE.** The constant is not a season
average either. p. 99: it is *"a fair approximation provided that its value is measured at a
proper time, such as at the end of the phase when most assimilates go to the leaves (e.g.,
development stage 0.5 for rice)"*, and Table 19 gives values *"at about this stage"*. So it is a
**canopy-wide average sampled at one moment (~DVS 0.5)**, used as a constant — an approximation
stated *with* its precondition, which the file's `source:` did not carry.

**AND THE LOCUS MATCHES, WHICH IS WHAT MAKES THIS A DISCHARGE.** p. 98 §3.3.3 defines the method
the constant serves: *"Leaf area is determined by dividing the weight of **live leaves** by the
specific leaf weight."* Our `science::leaf_area_index` is `leaf_carbon · sla_per_mol_c /
ground_area` — the standing live pool, and the constant's only consumer (`flows.rs:150, 476,
970, 2935`; `readouts.rs:278`). Value, population and application point agree. ⚠ This check was
run **before** the page was read, on the advisor's instruction, and it is what gave the answer
teeth: without it, "whole-canopy average" is a fact with nothing to be right or wrong about.

**FINDING — the book's development-keyed alternative is a DIFFERENT LOCUS, not a re-reading.**
Table 20's `SLT` multiplies the constant for **new** leaves, which needs leaf-cohort state; this
tree has none by design (P2: *LAI is derived, not stored*). So "key it to development" was never
available as a re-interpretation of the frozen number — it is a form build with a new state
variable, and the item's own framing hid that.

**FINDING — the winter-wheat `SLT` row does not have the shape the page's narrative describes,
and that retires the item's second argument.** p. 99 motivates the method with *"new leaves
formed early … are thinner"*. Winter wheat's row (read off the render) is
`0.,1., 0.33,1.1, 0.36,1.06, 0.43,1.5, 0.53,1.05, 0.62,1., 0.77,0.85, 0.95,1.07, 1.14,1., 2.1,1.`
— it **starts at 1.0**, peaks *thick* at 1.5 mid-vegetative, dips to 0.85, returns to 1.0. A
bounded non-monotone wiggle, not a thin→thick ramp. (Spring wheat's row *is* the ramp: 0.67 to
DVS 0.55, then 1.0.) The plan argued a "late-anchored reading" would also move the pinned *"LAI
peaks after anthesis"* defect the right way (DVS 1.37 → 0.96); **the source's own curve gives
that reading no support — it assumes a monotone ramp this crop's row does not have.** ⚠ Stated
precisely, because the imprecise version is this repo's own logged failure shape: that 1.37 →
0.96 figure was measured 2026-08-15 by keying the *scalar constant* through a harness, NOT
computed from Table 20's row, and it is separately retired as pre-clamp by
`mutual-shading-tolerance.md`. What died here is the **citation-side** argument for the late
anchor, not the measurement. ⚠ *A general narrative in a source's prose is not a claim about the
row you are using.*

**A CHECK ON OUR OWN RECORD, AND IT HOLDS.** `leaf-expansion.md` FINDING 4 cites this row as a
two-sided envelope of **[0.85, 1.50]**; both extremes are confirmed. ⚠ But the *curve's*
endpoints are 1.0, so the parked mechanism's floor is a mid-season value, not a seedling one —
"thin early" is not a claim this row supports.

**THE UNFREEZE.** Provenance-only, the shape `extinction_coef` took the same day: one `source:`
string, one manifest hash, **0 goldens, 0 values, 0 bands, 0 floors**. Predicted before running
and confirmed: the manifest diff is exactly one line, `param_files/canopy.yaml`.

**FREE CORRECTIONS NEXT DOOR, AND WHY THEY WERE FREE.** `crops/potato/canopy.yaml` is one of the
four overrides the manifest census excludes **by non-recursion**, so editing it moves no hash and
is not an unfreeze. Its sentences died on 2026-08-15 without anything going red: it says potato's
SLA is cited *"where the reference crop's SLA remains TODO(cite)"* (bound that day), and it
offers its own conversion as *"a useful check in two directions"* against *"the frozen file's
placeholder 22.0"*. ⚠ **Exactly ONE of those two directions died, and saying which is the whole
point of a file whose virtue is labelling what is and is not justified.** The check that potato's
~1.5× thinner leaf *"is the source's, not an artefact of our conversion"* **stands** — potato is
the 300 kg/ha row, wheat the 425 one, two distinct rows. The check that the conversion
*corroborates* the reference's number is now **tautological**: that number IS 10000/425 off the
same table, so it agrees with itself. Also a wrong ISBN check digit (`-2`; the book's own
colophon reads `90-220-0937-8`). ⚠ *When the other side adopts your source, the leg of a
cross-check that compared them stops being evidence — and nothing goes red, because both files
are then right.*

**TWO DEAD POINTERS FOUND, ONE FIXED, ONE LEFT ON PURPOSE.** Four comments in the params still
name `tests/test_potato_crop.py`, deleted by S6 on 2026-08-27. The one in
`crops/potato/canopy.yaml` was a **live claim** about where a cross-crop constraint is pinned,
so it now names `tests/potato_crop.rs` (the pin itself never lapsed — stage 2 rebuilt it). Left
untouched and recorded here instead: `crops/potato/allocation.yaml:85` (same shape, a different
file, and scope discipline says a free fix is still a separate edit) and `canopy.yaml:33`, which
mentions the Python file as a **dated historical instance** rather than a pointer. ⚠ *A comment
naming a deleted file is not a broken reference to any tool* — nothing in this repo can go red
on one, which is why they outlived their target by ten days.

**THE MEMORY INDEX HIT ITS CEILING, AND THE FIX IS RECORDED HERE BECAUSE NOTHING ELSE CAN HOLD
IT.** This item's memory line pushed `MEMORY.md` to 20,118 B against its 20,000 B ceiling. The
remedy is the gate's own, quoted in its assertion message — *"MERGE related memory files (two
files become one file with one line, the detail preserved inside), not … condense"* — so
`extinction-coef-bound.md` was absorbed into `canopy-provenance-split.md` (that name survived: it
had eight inbound links against one), its index line removed, the survivor's hook rewritten to
carry both lessons, and the one backlink re-pointed. ⚠ **The memory tree is not in git**, so this
paragraph is the only durable record of that merge — and the standing after it is **19,981 B and
167.9 B/line against ceilings of 20,000 and 170.0**. *The next memory line of any normal length
turns that gate red: the next item owes a merge, not a new line.*

**WHAT THIS DOES NOT CLOSE.** Whether this tree should model leaf-cohort thickness at all is a
form question with a new state variable, priced by nothing here. [B] itself says the effect is
days-to-weeks on canopy-closure date and *"small"* on leaf biomass and final yield. **The page
describing its own constant does not license a form — silence would not have, and neither does
an answer.**
