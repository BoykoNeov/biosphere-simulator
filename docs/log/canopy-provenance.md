## **The canopy's two uncited literals** (that build's own closing line — one was cited a file away, the other is a value question the shelf disagrees with itself about)

Plan of record: `post-roadmap-canopy-provenance.md`.

**`carbon_fraction` BUILT, `extinction_coef` MEASURED-NOT-BUILT, 2026-08-15** — the
predecessor named both as "the highest-leverage provenance work left on this observable";
the item split into three and only one was a retrieval problem. ⚠⚠ **`carbon_fraction`
WAS NEVER UNCITED.** `nitrogen.yaml` has carried the identical 0.45 bound to Raimanova et
al. (2024) since the 2026-07-16 citation round, under a **MUST-EQUAL** constraint both
files document, and `crops/potato/canopy.yaml` has stated since 2026-08-11 that *"the
reference value IS cited, but to a measurement of WHEAT grain/straw"* — describing the
reference file, which still read `TODO(cite)`. **A crop OVERRIDE was the record that the
reference was sourced.** Fourth instance of *check your own shelf*, and the fourth where
the stale artifact was **our own record** rather than the shelf. Shipped as a
provenance-only unfreeze on the honour-system ceremony (advisor review → regenerate the
manifest → unfreeze-log entry): **one sha-256 moved, the value did not, nothing could go
red by construction.** ⚠ What was NOT a copy is the **basis**: `nitrogen.yaml` applies the
fraction whole-plant and carries a root delta (roots 34.9 %, ~10 points below shoots,
overstating root C ~29 %), while `canopy.yaml` applies it to **leaf blade**, so the paper's
straw figure (45.66 %) is the nearer measurement and the delta does **not** transfer — the
same citation binds *more* tightly here than where it was first written, and copying the
source string verbatim would have imported a caveat that does not apply.
⚠ **`extinction_coef` is a VALUE question, not a retrieval one — the shelf answered three
times and disagreed with itself**, all three verified off page renders: **0.60** (Penning
de Vries 1989 p. 36, *"about 0.6 for a canopy with erect leaves and 0.8 for one with
horizontal leaves (Goudriaan, 1977)"* — an architecture-class PAR value, already source
[B] of the file, and Goudriaan is the author of the depth quadrature shipped the day
before), **0.65** (Soltani & Sinclair 2012 Table 10.1, the "Wheat" row, for the *identical*
equation `FINT = 1 − exp(−KPAR·LAI)`), **0.68** (ibid. Fig. 10.8a, wheat cv. Zagros, but
explicitly *unpublished data*). ⚠ **"Crop-specific" does not settle it**: Table 10.1's own
caption lists Penning de Vries 1989 among its sources, so 0.65 is partly derived rather
than an independent measurement — there is a coherence argument for 0.60 that has nothing
to do with which value is free, and picking the free one *because* it is free is the
co-adaptation this project has refused four times. ⚠ **The predecessor's 3.5× amplifier
does NOT transfer to this knob and this is where that was established**: it is the
elasticity to a *uniform* perturbation, while `k` raises absorption at the canopy top and
lowers penetration to the base, so it self-cancels in a closed canopy — +8.3 % on `k` buys
**+0.8 %** of peak LAI at the shipped step, not +30 %. Measured on `cc44b41`, `src/`
untouched: `open_season` peak LAI 6.0228 → 6.0700 shipped and **5.4273 → 5.8233 (+7.3 %)
converged**, harvest +1.0 %, rationing 0 — and the **LAI peak moves 0.4 DVS earlier, from
after anthesis (1.306) to before it (0.909)**, a second independently-pinned defect moving
the right way, corroborated by two oracles of different model families. ⚠ **The shipped
column barely moves because a mechanism is doing its job**: the Van Keulen & Seligman
5 %/day mutual-shading loss above LAI 6, shipped **inert** the day before, is above its
threshold for 1.8 days at `k = 0.65` and clips the peak — the mechanism a threshold forced
into the tree is what stops a bigger `k` running away. ⚠⚠ **THE TWO GATE FAMILIES MOVE IN
OPPOSITE DIRECTIONS FOR ONE REASON, AND READING EITHER ALONE GIVES THE WRONG ANSWER.** All
five frozen chamber CO₂ bands get **looser** at `k = 0.65` (sealed 1.1697× → 1.1913×,
perennial 1.1503× → 1.1723×, consumer 1.2009× → 1.2102×) — not a safety margin improving
but the chamber crop doing **less**: more interception per unit leaf is a faster draw on a
**fixed** carbon inventory, so the sealed crop limits earlier and ends smaller, which is
precisely why its CO₂ minimum rises. Meanwhile the perennial **liveness floor** clears by
**0.40 %** where it cleared by 5.12 % (`max(tail)` 0.578137 → 0.552202 against 0.55), and
it clears *only because* the statistic is a maximum over the tail — the trajectory declines
monotonically and its **final year, 0.525448, is already below the bound**. The gate is
green, in CI too, and is one small mechanism from red. `test_decade_stability.py` at
`k = 0.65`: **31 passed, 3 failed**, all three exact-value characterization pins that any
value move breaks; no floor and no band red. Three options priced (bind 0.60 at the
provenance-only price / bind 0.65 for the full 13-golden ceremony / bind 0.60 and record
the disagreement as a measured risk — ⚠ with the asymmetry running the *unsafe* way, since
both alternatives sit **above** ours, so the honest risk statement is *"our canopy may
intercept ~8 % less light than the crop-specific literature says"*); 0.68 is not
recommended, because unpublished data cannot retire a `TODO(cite)` in a tree whose rule is
*cite the primary literature*. **The decision is the user's and is untaken.**
⚠ **One audit spin-off:** `extinction_coef` has exactly one production consumer
(`photosynthesis.py:272`), which is what made the loader patch complete — and the same
audit found **`intercepted_fraction` now has NO caller in `src/` at all**, surviving only
in tests and the Rust mirror. It is the dead `exp` behind `cc44b41`'s vacuous cross-port
gate, still exported from the frozen surface.
⚠⚠ **AND A SPIN-OFF THAT WAS SHIPPED SEPARATELY** (`d8f5583`, a correction to the previous
two builds rather than to this one): **five frozen values in prose had been superseded six
commits after they were written.** The freeze doc's band table and
`tests/test_co2_compensation_band.py`'s docstrings quoted 76.82 / 75.48 / 74.42 ppm; the
tree reads 71.44 / 70.25 / 73.34 — the light path moved them 4–7 % *the same day the band
landed*. **The pin written for exactly this event DID fire**:
`test_the_five_margins_are_pinned_not_merely_positive` went red at the light path and was
re-pinned in `a0ef98b` **with its own comment updated to say so**, four lines below
docstrings that kept the old numbers. So the gap is not a missing guard — **a value in an
assertion acquires an owner the moment it goes red; a value in prose acquires none.** ⚠ The
**ranking inverted** too: the consumer chamber's docstring argues *"THE TIGHTEST OF THE
FIVE … enumerate the roster, not the discussion"* and it is now the **loosest**, because
both builds that moved these act through canopy **closure** and the consumer chamber's crop
is the one the crew's CO₂ keeps furthest from closing. The lesson survives, its subject
moved: **a ranking is a claim about a moment — re-derive it, never quote it.** Two things
deliberately not restated: the 50-yr claims (75.84 / 75.06) are dated and carried as an
open question rather than given digits they have not earned, and the **15-yr = 5-yr
identity held bit-equal through both unfreezes** — a *shape* outliving the values it was
measured on.
