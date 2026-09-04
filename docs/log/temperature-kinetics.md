## **The temperature form of the FvCB kinetics** (the science switch's first scientific pair — and the band the tree passes is held by a term measured to be INERT)

**2026-09-04.** Plan: `docs/plans/post-roadmap-temperature-kinetics.md`, written and committed
**before** any code, carrying numeric predictions for all eight gated quantities so the run
could score them. The item is the September direction plan's §2.3.1, ranked there as *"the
highest-leverage item in this list, and the least predictable"*.

**Standing: lab-only.** No frozen param file moved, no golden moved, no manifest moved, no
unfreeze. `KineticsForm::Cardinal` is the frozen body verbatim and is bit-identical on all
eight measured quantities.

---

## What was built

`PhotosynthesisParams` gains one non-YAML field naming **which temperature form its twelve
constants are read under**:

* **`Cardinal`** — the frozen reference. 25 °C constants, whole rate scaled by the [B]
  cardinal/TMPFTB multiplier.
* **`Q10Teh`** — [D] Teh ch. 6: per-constant Q10s on `Vcmax` (2.4), `Kc` (2.1), `Ko` (1.2) and
  `Γ*` (**derived**, `1/Q10(τ) = 1.754`, from eq. 6.19 `Γ* = O/(2τ)` — the shelf tabulates `τ`,
  not `Γ*`), eq. 6.23's quantum-efficiency shape applied to our `quantum_yield` as a **ratio**,
  and **no** whole-rate multiplier.

Reached through `lab::biosphere_with_form`, one column of the shared report
(`science_switch form=q10_teh`), three controls in
`rust/crates/domains/tests/temperature_kinetics.rs`.

**Why the params object and not three flow replacements.** `Allocation`, `GrowthRespiration`
and `MaintenanceRespiration` each hold a `CarbonContext` and each calls `budget()`. Replacing
one leaves a step whose growth respiration is computed off the frozen assimilation and whose
allocation is not — internally inconsistent and entirely plausible-looking in a report.
Replacing all three means rebuilding their contexts, i.e. a **second assembly body**, the
defect `lab/mechanism.rs` exists to prevent. The form rides the one funnel
`tests/param_funnel.rs` already gates, and all three flows follow.

---

## FINDING 1 — the retrieval this item was ranked behind was never needed: the source
was on our own shelf, cited by name

The direction plan blocked the pricing of this item on the Bernacchi PDF, saying of the
Vcmax temperature response: *"that the temperature form moves Vcmax at all is **understood,
not retrieved** — no Vcmax Arrhenius parameters are on the shelf."*

**Teh (2006) Table 6.2, p. 130 tabulates Vcmax's Q10 at 2.4**, alongside Kc, Ko and τ. The
book has been in `sources/` throughout, and `science_gates.rs` has cited it *by constant*
since the FvCB item — `TEH_SPECIFICITY_FACTOR = 2600` is a number off that same table. The
blocker was a claim about our shelf that nobody re-checked against the shelf. ⚠ **This is the
fourth time a "missing" source has been found already in the tree** — see
`canopy-regulator`, `canopy-provenance`, `nitrogen-cycle-form`. *Any "the shelf has no X" is
a claim about a search someone did once, and it is dated.*

⚠ Both the table and eq. 6.23 were read off **rendered page images**, not the PDF text layer,
which is mangled on exactly those pages: it gives Ko's unit as `mol mol⁻¹` (the page says
µmol mol⁻¹) and eq. 6.23's coefficients as `0.0000537 / 0.0000197` (the page says
`0.000053 / 0.000019`). The FvCB item was burned binding constants without a page; this one
had the page, and the text layer would have put a wrong unit into a cited constant.

## FINDING 2 — every prior pricing of this item reasoned from the branch that does not bind

§2.1 item 4 and §2.3.1 were both written off a `vcmax` ladder, and the o2-coupling item's
crossover analysis (*"the crossover sits between 80 and 90, ~15 % below the shipped Vcmax"*)
is the same reasoning. Measured at the real `open_season` forcing (Ci = 250, the sinusoidal
within-day PAR at `dt = ¼`, the committed 305-day temperature series):

**The light-limited branch binds 99 % of lit steps** — 10 of 876 are Rubisco-bound at LAI 3.
So a change to the Rubisco constants is nearly invisible and a change to the light branch is
nearly everything, the reverse of how the item was written. The whole season runs
−1.8 → 22.2 °C, mean 10.67, **entirely below 25 °C**.

## FINDING 3 — the effect decomposes, and "it is just the multiplier's deletion" is false

Season-integrated leaf-level gross rate at fixed LAI = 3, as a ratio to frozen:

| variant | ratio | share of the log-effect |
|---|---|---|
| delete the whole-rate multiplier only | 1.243 | 49 % |
| Teh's Q10 kinetics only (multiplier kept) | 1.150 | 31 % |
| Teh's quantum-yield shape only | ~1.09 | 20 % |
| **the built form** | **1.561** | 100 % |
| Teh's parameterization taken whole | 1.746 | — |

Two things this settles that were argued rather than measured:

* **The flat-`Jmax` gap is worth ~3 %, not ~40 %.** Giving `Jmax` the cardinal multiplier
  moves 1.561 → 1.511. At this canopy's absorbed PAR (~40 µmol m⁻² s⁻¹ mid-depth) the branch
  is quantum-yield-limited and `Jmax` is nearly inert, so its missing temperature response
  cannot be the story. Leaving it alone is honest rather than convenient.
* **The mixing is worth ~12 %, and it is almost entirely `Vcmax`** (Teh's 200 against our
  100). ⚠ `science_gates.rs` warns that *"Teh's companion constants disagree with ours … mixing
  them would be the co-adaptation this project refuses"*. That warning is about swapping a
  **value**; taking a **response shape** and leaving every 25 °C value alone is a different
  act — but it is still a mixing, so it was priced instead of waved through.

## FINDING 4 — the predictions scored 2 of 4, and the two misses are the findings

Written before the run, scored after:

| quantity | predicted | actual |
|---|---|---|
| `open_season` peak LAI | ~8.0 (7.3–8.8), ceiling RED | **6.232730** (+3.5 %) |
| `open_season` peak W (t/ha) | ~17.4 (15.5–19.0), cap breached | **18.365566** (+37.3 %) |
| `perennial_long` peak-leaf (mol C) | ~0.70 (0.62–0.80), rising | **0.304503** (−47.3 %) |
| the five CO₂ minima (ppm) | fall to ~62 sealed (50–68), ≥1 cross | **54.16 / 54.67 / 63.63** |

**❌ Peak LAI: badly wrong** — a +33 % move predicted, +3.5 % measured.
**✅ Peak W: in range**, and the 14.4248 cap is over by 27 %.
**❌ The liveness floor: wrong SIGN** — predicted to gain clearance, it halved and went red.
**✅ The CO₂ minima: in range**, and three of the five cross the recorded floor.

*A prediction that lands is worth less than one that misses.* Both misses share a cause: they
were computed from the leaf-level carbon supply, and both observables are set by something
downstream of supply.

## FINDING 5 — peak LAI barely moved because a loss term EXACTLY INERT in the frozen
tree absorbs the whole gain

Run as a 2×2, because *a causal claim earns the experiment that removes the cause* and a
one-sided run cannot tell "the step caps this form" from "the step caps everything":

| peak LAI | mutual-shading loss ON | loss OFF (`shade_rate = 0`) | released |
|---|---|---|---|
| `Cardinal` | 6.022837 | **6.022837** | **0.000000** |
| `Q10Teh` | 6.232730 | **13.544978** | **7.312248** |

The 5 %/day mutual-shading loss above LAI 6 (Van Keulen & Seligman, via [A] p. 101) is
**exactly inert** in the frozen tree — the peak sits a hair over the threshold and is reached
before the term can bite — and under the new form it holds back **more than a doubling** of
the canopy.

⚠⚠ **So the Q10 column's apparent compliance with `5.0 < peak < 8.0` is not the canopy
clearing the band; it is a loss term clamping it there.** A mechanism that has never done
anything in any recorded run is now the thing that sets peak LAI. ⚠ And it re-frames the
second recorded bound — *"`peak < 6.0` **or** the 5 %/day mutual-shading loss is MODELLED"* —
which has always been satisfied by its second clause while the clause was measurably doing
nothing. It is doing something now.

## FINDING 6 — the two red gates are red for different reasons, and one is
uninterpretable as science

* **`peak W` 18.366 against the 14.4248 t/ha cap is a real breach**, 27 % over.
* **The three CO₂ crossings are measured against a floor the form itself dissolves.** The
  gate reads `min > Γ*/ci_ratio`, frozen at 61.07 ppm off a **constant** 25 °C `Γ*`. Under the
  form `Γ*` moves with temperature and the floor sweeps the season:

  | T (°C) | −1.8 | 4.6 | 10.7 | 17.6 | 22.2 |
  |---|---|---|---|---|---|
  | floor (ppm) | 13.54 | 19.39 | 27.29 | 40.24 | **52.18** |

  It is **below 61.07 everywhere**, so a run reported as crossing the recorded floor at
  54.16 ppm may be nowhere near the form's own compensation point. **The correct reading is
  "the gate must be re-posed", not "the form fails."** ⚠ This is exactly the hazard
  `log/o2-coupling-measured.md` recorded for a live-`Γ*` build — *"a pointwise floor is a
  different assertion, not a re-tuned one"* — arriving through a different door: there it was
  O₂ that would make `Γ*` live, here it is temperature.

## FINDING 7 — the liveness collapse is the jar starving itself on its own productivity

`perennial_long`'s converged peak-leaf fixed point halves (0.578 → 0.305, under the 0.55
floor) **because** the crop got better: a bigger canopy draws the sealed chamber's CO₂ to
54.67 ppm and the chamber cannot refill it. *A field-scale improvement is not a chamber-scale
one* — scope (A) finding 11 and `crew-coupled-loop`'s refusal, from a third direction.

## FINDING 8 — the form promotes an uncited constant into a load-bearing position

Rubisco-bound share of lit steps at LAI 3: **1.1 % frozen → 9.1 % under the form.** `vcmax`
is `TODO(cite)` — *"provisional, literature-typical C3"* — so the form makes a number we
cannot defend matter nine times more often. This **raises** the value of the two retrievals
already owed (§2.2: the Bernacchi page, the Wullschleger wheat survey); it does not discharge
either.

---

## Controls, and what they caught

* **`Cardinal` is bit-identical to the loader on all eight quantities**, by two independent
  routes (`biosphere_with_form(&[], Cardinal)` parses the frozen file text;
  `params::biosphere()` goes through the ordinary loader). Not a round trip.
* **Three mutations, each reddening the intended test.** Making the `Q10Teh` branch return the
  Cardinal answer reddened the mis-target guard *and* the 2×2; flipping the loader's default
  to `Q10Teh` reddened the bit-identity control.
  ⚠ Those two left the 2×2's **own headline assertion** (`cardinal_off == cardinal_on`)
  uncovered — it carries "the term is inert in the frozen tree", and nothing run so far could
  have reddened it. Third mutation, aimed at it: lowering the LAI threshold by 1.0 makes the
  loss bite under `Cardinal` (peak 6.022837 → **5.087177**), and that assertion is what fails.
  *A control battery is only as good as the assertion nobody aimed at.*
* **The first draft of the test file hand-listed the six runs and its own folds, and silently
  measured 7 of the 8 quantities** — it had no fixed-point row. The count assertion caught it,
  and the file was rewritten to derive its roster from `report::SPECS` and use the report's
  own folds. *A roster copied by hand is the failure the copy was made to avoid.*
* **The 2×2's frozen release is exactly 0.000000**, so the test asserts two absolute facts
  rather than a ratio between them: a ratio against a zero denominator passes for a reason
  that has nothing to do with the form.

## What is now owed

**A decision, and it is the user's.** Recommendation: **refuse `Q10Teh` as a replacement for
the reference, keep it as a lab instrument.** It breaks the above-ground biomass cap outright,
collapses the chamber liveness floor, deletes a whole-rate multiplier by an editorial act Teh
does not license for *our* `Jmax`-shaped light branch, and promotes an uncited `vcmax`. What
it has produced is worth more than a form swap: **the peak-LAI band is being held by a loss
term measured to be inert at the frozen params**, which is a fact about the frozen tree and
does not depend on adopting anything.

Not done here, and each its own item: re-posing the CO₂ floor as a pointwise assertion; the
two `photosynthesis.yaml` retrievals; whether the mutual-shading step being load-bearing is
acceptable or is itself the next canopy question.
