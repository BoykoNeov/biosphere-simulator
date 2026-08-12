## **Sink-limited leaf expansion** — BUILT, BOUNDED by a second source, then BLOCKED under RK4

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per
> work item. The fuller record is the plan doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**BUILT 2026-08-12** — the successor `water-stress-curves.md` finding 2 named: a sink-limited
leaf-expansion phase, *"never 'the missing `WSFL` multiply'"*.
`docs/plans/post-roadmap-leaf-expansion.md`; probes `M:/claud_projects/temp/leaf-expansion/`.
Leaf area becomes the **fourth aux accumulator**, which **reverses the P2 lock "LAI is derived,
not stored"** on purpose: `WSFL` scales an expansion *rate*, so area a drought withholds must
stay withheld. **FINDING 1 — A DENSITY SWEEP RUN BEFORE ANY CODE, SO THE BAND COULD NOT BE
REVERSE-ENGINEERED.** Peak canopy is linear in `PDEN`, which [F] does **not** put in its
parameter table; it crosses the 5.0 science floor at `PDEN` 170.9, and `PDEN = 300` was read
off [F]'s own Run sheet before that table existed. Two more things fell out of measuring first:
[F]'s literal bare-degree-day clock is **refused by measurement** (peak LAI 11.92 against a 6.0
ceiling — the node phase runs on our development clock), and the LAI-as-state vs
additive-offset question is a **non-question** (0.1 % apart), settled instead on a defect only
the offset has — it drives the winter canopy negative behind a `max(0, ·)` clamp. **FINDING 2 —
THE FIRST BUILD WAS CORRECT AND UNSHIPPABLE, AND THE GATE THAT CAUGHT IT WAS THE RATIONING
GATE.** [F] Ch. 9 is scoped in its own opening sentence to *"non-limiting water and nutrients"*
and Ch. 12 is titled *"A Model for **Potential** Production"*; [F] has **no mechanism by which
the atmosphere runs out of carbon**, because in its world it cannot. Six of our eleven
scenarios are growth-limited, and there the node branch makes **area without carbon**: leaf
thinness 2–15× nominal (worst 29.46×) and three scenarios rationing (27 / 41 / 83 firings). ⚠
**The discriminator is carbon limitation — a STATE, not a scenario property**: the worst run
(`n_limited`) is an **open field**, which kills any "wire it in the field, not the jar" rule.
And the defect was sharper than "the leaves get thin": **six scenarios peaked at exactly
2.9068** — different volumes, different nitrogen, horizons of 1 to 15 years, agreeing to four
decimals — because 2.9068 is where the node curve ends and below TLM there is **no carbon
feedback whatsoever**. A thinness cap would have bounded the symptom and left the missing
feedback intact. **FINDING 3 — `min` ON THE RATE IS NOT `min` ON THE STATE, AND CONFLATING THEM
COST A WHOLE REFUSAL.** The hybrid `GLAI = min(node_rate, carbon_rate)` ([F] p. 103, describing
Boote et al. 1998) collapsed the roster (`open_season` 5.4624 → 1.2682) and was recorded as
"the hybrid is refuted" — too broad. A per-day `min` **ratchets** (the integral of the
pointwise minimum is below both paths), and the carbon rate is **itself a function of LAI**, so
the ratchet is self-reinforcing: a death spiral, not a bound. A bound on the **state** has
neither property. ⚠ **The sentence that draft used to dismiss Boote belongs to a different
model** — [F]'s *"little experimental evidence to support such a complex view"* closes the
paragraph on **Kropff & van Laar's** phase switch. Transcribed correctly, attached to the wrong
model: **a locus error inside a correct quote, committed in our own record.** **FINDING 4 —
EVERY SOURCE ON THE SHELF THAT MODELS LEAF AREA UNDER *ANY* LIMITATION COMPUTES IT AS MASS ÷
THICKNESS.** [E] is `ALV = WLV/SLA`; Teh is `LAI += W_leaf·SLA`. So the "additional model that
behaves where [F] doesn't" is not exotic — **it is the form already frozen in this tree**, and
the build's real job was composing the two. No source carries a *maximum* SLA (the four
candidates and why each fails are tabulated in the plan doc §7.2), but **[E] Table 20 carries a
`Wheat, winter` row**: `SLT` = specific leaf **weight** as a fraction of the Table 19 constant,
range **[0.85, 1.50]** — a *two-sided envelope*, a better object than the one-sided maximum
being looked for. ⚠ **The fractions invert**: weight is mass per area, so the *minimum*
fraction is the *thinnest* leaf and hence the *ceiling* on area. Centre corroborated three
ways: [E] 23.5 m²/kg, [F] 21, our own frozen 22 — and our 22 is a bare `TODO(cite)`, so **[E]'s
row is better provenance than the constant this tree already ships**. **FINDING 5 — THE FLOOR
IS NOT DECORATION; IT IS WHAT STOPS THE SPIRAL, AND ONLY MEASUREMENT SHOWS IT.** [F]'s node
curve starts at 1 cm²/plant, *smaller* than the seedling's carbon-derived area, so a
**ceiling-only** bound lets `min` pull LAI below what the standing mass already supports:
`open_season` 4.4300 against the frozen form's own 5.4624 — **the combined model coming out
worse than the model it was meant to improve**. And applying [E]'s curve *instantaneously*
rather than as its extremes is worse still (2.0403), because `SLT = 1.5` makes the bound
tighter than nominal mid-season: **the envelope is the RANGE of thickness the crop exhibits,
not the thickness it typically has.** **RESULT** — rationing **0 on all eleven scenarios** (was
27/41/83); leaf thickness **1.03–1.18× nominal median, 1.20–1.21× max** (was 0.87–3.72 /
29.46), inside real wheat's range; **the 2.9068 signature is gone**; [F]'s node branch still
decides **~39–45 % of days**, so the sink limitation survives. ⚠ **THE GREENWOOD GATE PASSES,
AND IT PASSES DOWNWARD**: §6.6 recorded `open_season` peak W failing by 0.9 % (14.5554 vs
14.4248) with the *other* phyllochron locus passing, and refused to switch for that reason.
With the envelope it is **13.6717 — clearing by 5.2 % at the same `PHYL = 112`.** The gate was
resolved by a mechanism, not by choosing the constant that passes. **Corroboration nobody
fitted:** [F]'s own working model reports max LAI 5.15; ours is 5.2533 (+2.0 %) with the
envelope and 5.9129 (+14.8 %) without — the envelope came from [E] with no reference to [F]'s
output and moved us *toward* it. **FINDING 6 — THE ENVELOPE DECIDES A DISAGREEMENT BETWEEN TWO
SHELVED SOURCES, AND THAT MUST NOT BE FILED AS A BOUND'S SIDE EFFECT.** [F] Table 15.1 makes
leaf expansion **more** drought-sensitive than growth (`WSSL` 0.40 > `WSSG` 0.30) — drought
thickens leaves, which is the entire reason [F] gives leaf area a factor of its own. [E] p. 100
reports that irrigation and fertilization *"have little effect on specific leaf weight"*. **The
envelope makes [E] win**, measurably: on `water_biting`, forcing `WSFL = 1` moves peak LAI
**+215.0 % under [F] alone and +0.5 % under the envelope** — the drought curve's leverage cut
~400× in the one run that exercises it, and `WSFL` was the reason the mechanism was built. ⚠
The corroborating sentence is also **weaker than it looks**: its loci are **potato** irrigation
and **maize** density, not wheat, and [E] flags the generalization itself (*"may also be small
in other crops; they are disregarded here"*). **FINDING 7 — BEFORE READING THAT AS "DROUGHT
DOES NOT MATTER": FIVE OF SEVEN SCENARIOS NEVER EXERCISE `WSFL` AT ALL.** `FTSW` never reaches
0.40 in `open_season`, `day_neutral`, `n_limited`, `sealed_chamber` — **or in the one named
`drought`** (consistent with `drought-defence-is-the-mechanism-working`: the accelerated
phenology lets the crop escape the deficit). Only `water_biting` fires it hard (below 1 on
**100 %** of days, minimum 0.1250). So the small numbers are a **coverage** fact on most of the
roster and a physiological one on exactly one scenario — recorded separately because they have
different successors. **FINDING 8 — THE TWO NEW NUMBERS ARE UNREFERENCED, AND THE ANSWER
SURVIVES IT FOR A REASON THAT WAS MEASURED RATHER THAN ASSERTED.** [E] Table 19's winter-wheat
row shows no reference (*"obtained from colleagues at CABO, Wageningen"*), and [E] says of
these relations *"should be used carefully and checked whenever possible"* — **the same
provenance class as `tu_tlm` and as the `fstr = 0.40` that got stem reserves refused.** The
difference is in *kind*: `fstr` **set** an amount; this pair **bounds** one, and where it binds
the model reduces to the frozen carbon-derived form, so **a wrong envelope fails toward the
science already in the tree**. Swept from `(1.00, 1.00)` to `(0.75, 2.00)`: every envelope
clears Greenwood with zero rationing and moves `open_season` by ≤ 19 %, while **removing** it
moves `n_limited` by **30×** and fails the gate. The numbers pick a point inside a flat region.
**THREE EXPOSURES, STATED NOT SETTLED:** (i) ⚠ **locus, with its direction** — [E] applies
`SLT` to **new** leaf area, we apply it to the **canopy average**, and a mixture varies less
than its newest component, so our envelope is **wider than [E]'s own**, i.e. biased *generous
to [F]*; the honest envelope would clip more. (ii) ⚠ **the projection is not `dt`-independent,
deliberately** — clamped, `evaluate` returns `ceiling − LAI`, which is the correct
discretisation of a bound *on the state* (it never violates the envelope at any step size,
which a `dt`-independent rate cannot promise); same shape as `root_depth`'s cap, and written
into the docstring because **the Rust mirror carries the rule, not the rationale**. (iii) ⚠
**"inside the envelope by construction" would be ~3 % false** — the bound reads leaf carbon at
*step entry*, so the canopy overshoots by one step's growth (measured 1.20–1.21× against a
1.176 ceiling); the same lag is why the degenerate `(1.00, 1.00)` envelope lands **10.1 % low**
instead of reproducing the frozen form, a control kept precisely because it puts a number on
the lag. **AND THE CLAMP LESSON, ANSWERED BEFORE IT IS RAISED:** `a-clamp-hides-a-wrong-amount`
is about a clamp standing in for an **amount nobody measured**; this one is a cited
physiological bound whose binding branch is the tree's own reference form, and the sweep above
measures what happens when the number moves. **FINDING 9 — ⚠ BLOCKING, AND IT INVALIDATES THE
EVIDENCE BASE ABOVE RATHER THAN ADDING TO IT: EVERY MEASUREMENT IN THIS RECORD WAS
EULER-ONLY.** Paying §6.7's debt (the full suite) returned **105 failed, 31 errors** — and all
31 errors are one file, `test_decade_stability.py`, none of them an assertion:
`ArbitrationError: flow #0 would over-draw a stock (scale_f=0.9668 < 1) under a higher-order
scheme`. Eleven scenarios report `rationed == 0` under Euler *across every envelope in the
sweep*, which is exactly what that guard cannot see. **It is ours**: the frozen derived form
runs the same decade cleanly under RK4; `[F]` alone errors at `scale_f = 0.6982`; `[F]` + the
envelope at **0.9668** — the envelope cuts the overdraw from 30 % to 3.3 % and does not remove
it. Mechanism: the chamber's peak leaf carbon rises 0.8446 → 0.9677 (**+15 %**), and
`chamber-scale-diagnosed` already measured that the jar holds two days of carbon. Two
diagnostics, **measured and NOT adopted**: (a) the failure is *marginal in the bound* — at
[E]'s cited `slw_min = 0.85` it errors, at 0.90 it is clean — and **0.90 is refused**, because
`biosphere-reference.md` step 5 forbids retuning a bound so a change fits, and it would be
doubly tempting since exposure (i) above independently argues for a *narrower* envelope; that
argument is earned by running [E]'s mixture model, not by trying values until RK4 goes quiet.
(b) ⚠ **THE FIRST VERSION OF THIS FINDING GOT THE MECHANISM WRONG, AND THE ERROR IS THE PART
WORTH KEEPING.** It argued that because years 3/5/10/15 all fail with `scale_f = 0.966836`
*identical to six decimals*, this must be "one recurring day from the second sowing onward",
pointing at the `annual_reset` × re-sow interaction. **Unsound:** the run raises on the first
violation and *aborts*, so every horizon ≥ 3 executes an identical prefix and dies on the
identical day — identical `scale_f` is a tautology of fail-fast, and a drifting failure would
give those same four numbers; `years = 1` never reaches a reset at all, so "year 1 clean"
discriminates nothing. **A prediction written in the grammar of a measurement**, one section
after this record congratulated itself for measuring rather than asserting. **The actual
discriminator** — instrument the raise: flow #0 is **`biosphere.allocation`** (the flow that
spends carbon to build tissue), at step **502**, **day 197 of season 1, 108 days from any
season boundary**. Mid-season. The re-sow hypothesis is **refuted**; under RK4
`check_no_overdraw` runs at each *perturbed stage state*, so a canopy 15 % larger asks
`allocation` for carbon the stage state no longer holds. It is the **sealed chamber's carbon
inventory**. ⚠⚠ **Which makes this blocker NOT independent of finding 6's science judgement, as
first reported** — the envelope's generosity (LAI up to 1.176× the carbon-derived area) and the
overdraw are one fact, so the successor that narrows the envelope on [E]'s own terms addresses
the RK4 break and exposure (i) together. ⚠ Separately cleared, on the integrator's own
contract: the clamp's `dt`-dependence is **not** the cause — aux advances once per step at the
step-entry state, `_perturb` shifts stock amounts only so `State.aux` is identical across
k1–k4, and every stage receives the full `dt`. ⚠ §6.7's own scope estimate was also short by
**3×** — it named five files; the reach is seventeen, including `test_soil_fractionation` (15)
and `test_crew_coupled_loop` (2), which it did not anticipate. **NOTHING WAS REGENERATED; no
golden, no manifest.** The tree is red, the mechanism is complete and measured, and the
successor is named: **derive [E]'s canopy-average envelope properly, or refuse.** The durable
lesson is the single-integrator one — a guard that is blind by construction cannot be the
evidence that a mechanism is safe, and any successor measures both integrators from the first
probe.

**FINDING 10 — THE NAMED SUCCESSOR WAS TAKEN 2026-08-12 AND IT DISCHARGED THE EXPOSURE BY
REFUTING IT, LEAVING THE BLOCKER EXACTLY WHERE IT STOOD.** §8.5 named one successor —
*"derive [E]'s canopy-average envelope properly, or refuse"* — and §7.9(1) had priced it as
un-derivable *"without running [E]'s mixture model"*. Run. [E] Listing 3 lines 88-92 give
new area as `GLV/SLN`, area **loss at the canopy average** (`LLA = LLV/SLA`, and [E]'s prose
on printed p. 101 says so in words), and the canopy average itself as **emergent**
(`SLA = WLV/ALV`). For `S ≡ W/A` those four lines reduce to **`dS/dt = (GLV/A)·(1 − S/SLN)`,
in which every senescence term cancels exactly.** With `GLV ≥ 0` always, `S` is pushed *up*
below `min SLN` and *down* above `max SLN`, so `SLC·[0.85, 1.50]` is **forward-invariant for
any growth history whatsoever** — and tight, since `dS → 0` only as `S → SLN`. ⚠ **THE
SHIPPED PAIR IS THEREFORE THE MIXTURE CALCULATION'S OWN ANSWER, NOT A LOOSE STAND-IN FOR
ONE**, and §7.9(1)'s *"the honest envelope would clip more"* is **REFUTED**: it conflated the
range the average **visits** with the range it can **reach**, and a bound is about the
second. The one narrowing that would have been legitimate — restricting the hull to the
stages at which leaf mass is actually created, which has zero free parameters — is
unavailable, because `allocation.yaml` gives leaf a nonzero share across the whole of
DS [0, 2.0), containing both DS 0.43 (`SLT` 1.50) and DS 0.77 (`SLT` 0.85). ⚠⚠ **AND THE
MEASUREMENT MADE THE TRAP CONCRETE RATHER THAN THEORETICAL.** The mixture trajectory,
integrated on nine scenarios, stays strictly inside the interval everywhere — but its lower
extreme is **strongly scenario-dependent**: 0.9449 on the open field and *exactly* 1.0000 on
all four carbon-limited runs, which never dip below the derived form. A ceiling fitted to
`open_season`'s 0.9449 would be 1.058× instead of 1.176× — and §8.4(a) had already measured
that **0.95 clears RK4**. So the one derivation that would have *looked* principled lands
inside the very retune window that was refused; §9.3's numbers were written **before** that
table was produced, which is the only thing separating the two. ⚠ A first, area-form
reconstruction of the mixture read `f_min = 0.79` on `consumer_chamber` — *below* the
invariant — because it charged herbivory's mass loss to senescence and removed too little
shadow area; integrating the **average** instead drops every loss term by construction and
the artifact vanishes. **A reconstruction artifact about to be reported as a violated
invariant is finding 9(b)'s shape again, caught before it reached the record this time.**
**ALL THREE PRE-REGISTERED PREDICTIONS HELD**: no parameter moved, no golden or manifest was
touched, and `test_decade_stability.py` still errors at `scale_f = 0.9668362828371428` — the
same value to all ten digits. ⚠ **§8.5's "the ONE successor that addresses both" is
FALSIFIED**: the locus exposure and the RK4 break were never one fact, and finding 9's own
closing sentence welded them together via an argument that holds only if a narrower envelope
is derivable. **The mechanism is complete, correct, fully measured and unshippable, and the
only measured route to green remains the retune this project refuses.** The verdict is the
user's, not mine.

**FINDING 11 — ⚠⚠ THE VERDICT WAS HANDED BACK ON A PREMISE THAT IS FALSE, AND THE
MEASUREMENT THAT FALSIFIED IT IS ENTIRELY ON THE FROZEN TREE.** Finding 10 closed with *"the
RK4 blocker stands, with NO route on the shelf"* — meaning the only measured way to green was
the bound retune step 5 forbids, so refuse-and-revert was the recommendation. **The premise
was never measured; it was inferred from the one knob that had been swept.** Asked instead as
a question about the *tree* rather than about leaves — does the frozen tree pass RK4 by
margin or by construction? — it answers on frozen `main` in one afternoon of probes, and it
answers against the recommendation. The record is
[`allocation-headroom.md`](allocation-headroom.md) and its plan doc; the headline: **the
blocker is a step-size bound, not a science defect.** A crop far larger than this mechanism
ever grew runs clean; the parked branch runs clean at a finer step with **no parameter moved
and no form changed**; and the wall is the sealed jar's carbon turnover time falling below
one day, on the one withdrawal in this tree whose rate constant is emergent and therefore has
no `k·h < 1` precondition — the bound three other places in the tree enforce. **So `refuse
and revert` is no longer supported by its own argument.** ⚠ It survives as a *scope*
judgement, and that is a different claim needing a different case. ⚠⚠ **AND THE REVERSAL IS
NOT AN ENDORSEMENT — READ FINDING 9 IN THE MIRROR.** Finding 9's durable lesson is that a
guard blind by construction cannot be evidence a mechanism is safe; shrinking `dt` until
`check_no_overdraw` goes quiet is **the same error with the sign flipped**. What cleared at
the finer step is the *guard*. Every measurement that got this mechanism accepted — the
Greenwood gate, the leaf thickness, the node branch's day share, the zero rationing, the
`WSFL` leverage cut — is **still Euler-only at `dt = 1`**, exactly the defect finding 9
named, and the frozen tree's own trajectory moves measurably under that refinement. **The
standing is: not refuted, route identified, evidence base pending re-measurement at the step
that would ship it.** The successor is not a leaf successor at all — it is the `dt = 1`
contract for a sealed chamber, which prices out at **three** freeze contracts, and a shelf
search for a cited supply-limited assimilation form that would keep `dt = 1` instead.
