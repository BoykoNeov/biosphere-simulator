## **The layered canopy + the leaf-thickness anchor** (the refused candidate, BUILT as honest physics on the user's call — and the mutual-shading loss the pair forced)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per
> work item. The fuller record is the plan doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**BUILT 2026-08-15 — an UNFREEZE: 7 biosphere + 4 station goldens, both manifests, one
science band restated, the native port mirrored.**
`docs/plans/post-roadmap-canopy-magnitude.md` §7b; probes
`M:/claud_projects/temp/canopy-layers/`. Authorized by the user on the diagnosis's open
decision (*"build the layered canopy anyway as honest physics (it's coherent alongside the
thickness fix, and that combination is the only one where it doesn't eat the margin)"*),
then twice more as the work uncovered its price: *"Build canopy + move constant to 23.53"*
and *"Build the shading loss too, restate the check"*. ⚠⚠ **THE PREDECESSOR'S HEADLINE
NUMBER WAS RETRACTED BY THE BUILD ITSELF, AND THE RETRACTION IS THE FIRST FINDING.**
`canopy-magnitude`'s finding 2 — "the cited cheap scheme is worse than the expensive one" —
was measured with a probe whose docstring said *3-point Gaussian* and whose arithmetic was a
**midpoint rectangle rule**: three equally-weighted samples at equally-spaced depths. The
real Goudriaan scheme (abscissae `0.5 ± 0.5·√0.6`, weights `5/18, 8/18, 5/18`) tracks the
100-layer reference to **better than 0.2 %** where the midpoint rule sat **−5.2 %** low, so
there is no quadrature error to point anywhere and the generalisation drawn from it was
backwards. ⚠ **A probe that NAMES a scheme is not evidence that it IMPLEMENTED that scheme**
— the name and the arithmetic sat three lines apart, nothing compared them, and the number
travelled into a plan and was used to argue about a mechanism. Related in shape to
`stem-reserve-form-is-on-the-shelf` (the stale artifact was our own record) but sharper: here
the artifact was *self-inconsistent at the moment it was written*. **WHAT SHIPPED, three
mechanisms.** (i) `photosynthesis.canopy_assimilation` now integrates over canopy **depth**
with the Goudriaan scheme, absorbed PAR at depth `L` being `k·I₀·exp(−k·L)`; the big-leaf
aggregator (one call at layer-mean light × intercepted fraction) is gone, and both halves of
the Jensen bias the module docstring has confessed since Step 5 are now closed — the
within-day half by the light path on 08-14, the within-canopy half here. (ii)
`specific_leaf_area` **22.0 → 23.53 m²/kg**, bound to Penning de Vries et al. (1989) Table 19
p. 100, *"Wheat, winter"* (425 kg/ha per unit LAI ⇒ 10000/425), read off the page render
because the text layer is unusable. ⚠ This retired a `TODO(cite)` **and moved the value
+7.0 %**, so it is a calibration inside the goldens, not the honour-system provenance-only
shape. (iii) `allocation.mutual_shading_rate` — an extra **5 %/day** leaf-area loss above
LAI 6, per Van Keulen & Seligman 1987 via [B] p. 101. ⚠⚠ **THE THIRD MECHANISM WAS NOT
CHOSEN; IT WAS FORCED BY THE FIRST TWO.** The pair took `open_season`'s peak LAI to
**6.0228**, which cleared the "5.0 < peak < 8.0" band handsomely (§7 had predicted 5.0314, a
0.6 % margin — wrong, because it was computed with the retracted midpoint rule) but tripped a
*second*, independently sourced check at **peak < 6.0**, by 0.38 %. Moving 6.0 to fit 6.0228
is the co-adaptation refused three times in this same file, so instead the **mechanism the
threshold was standing in for** was built and the check restated as *"peak below 6.0 **or**
the 5 %/day mutual-shading loss is MODELLED"*. ⚠ **Honest reading, and it is in the test
name and the manifest: the loss is currently INERT** — the peak is 6.0228 bit-identically
with and without it, because the crossing is too brief at the shipped step for a 5 %/day rate
to bite, and no chamber's canopy comes within an order of magnitude of closing. What changed
is that the regime is **represented** rather than **avoided**; a restated band that says so
is stronger than a numeric bound that merely happened to hold. ⚠ Provenance caveat carried in
`senescence.yaml`: the underlying monograph is a **spring** wheat crop, generalised to
"wheat" by Penning de Vries; we are winter wheat. **A LATENT DEFECT THE BUILD SURFACED**:
`NitrogenSenescence` recomputes the shed-carbon flux independently of the senescence flow, so
adding the shading term to one and not the other silently decouples litter C from litter N.
Caught by an existing test written for exactly that
(`test_shed_nitrogen_uses_the_same_carbon_flux_as_the_senescence_flow`) — the paired-
recomputation test earned its keep years after it was written. **CORROBORATION, NEITHER
SOUGHT NOR FITTED**: our peak-LAI gap to the LINTUL3 validation oracle closed from **22 % to
6.4 %**; the oracle was not consulted while choosing, and nothing was tuned toward it. And
the anchor was already **on our own shelf** — `tests/test_potato_crop.py` cites the *same*
Table 19 for potato (300 kg/ha ⇒ 33.33 m²/kg). Wheat's row was one line away, under a
`TODO(cite)`, for the whole life of the project. ⚠ Third instance of the
`canopy-regulator-diagnosed` shape: **check your own shelf before treating a value as
missing science.** ⚠⚠ **A PREMISE OF THE PREDECESSOR PLAN IS FALSE FOR OUR CROP.** §6 rested
on "every SLA table in the literature declines with development", which is what produced the
±35–75 % swing and the "2.7× range" framing. Table 19's specific-leaf-weight column for
winter wheat is **non-monotonic and returns to its starting value**; the probe's "+74.8 %
late-anchored ramp" was effectively *spring* wheat's table. So the shipped constant is
deliberately a **scalar**, not a DVS-keyed ramp — and a second reason reinforces it: the
frozen tree applies leaf area at **state** level, and a rate-level table applied at state
level is a form error this project has made before. **WHAT MOVED**: all 7 biosphere goldens
and the 4 plant-bearing station goldens (the plant-free ones byte-identical, the structural
check that nothing leaked); `canopy.yaml` + `senescence.yaml` param hashes; the `open_season`
science band restated; `biosphere_params.txt` +3 lines for the cross-port transfer. ~20
descriptive pins re-measured with dated notes. `git diff src/simcore/` empty. **THE GATE
REPORT — all 15 science gates GREEN**, including the four frozen liveness floors and all five
CO₂ compensation-point bands. Two acceptance-gate facts moved and are recorded rather than
smoothed: the station's plant-side margin **loosened 11.8868 → 12.2894** (the layered canopy
draws slightly less carbon, so the plant still binds but the gap to the cabin's 16.6667
narrowed — this pair has now crossed twice in two days, so the *direction* is not a stable
property), and the perennial chamber's 5-year/15-year margin identity held **bit-identically
a third time** across a change that moved the value +8.4 %, which promotes "the perennial
chamber's tightest carbon moment is inside five years" from coincidence toward property.
⚠ **NOT re-anchored**: the 50-year perennial peak-leaf attractor now settles at **0.5437,
BELOW the 0.55 figure** the manifest floor is anchored on. That figure is a *probe*, not a
manifest bound — the four frozen 15-year floors all still pass — so the number is recorded
and the assertion inverted with its reasoning, rather than the floor being moved to fit.
⚠⚠ **AND A SECOND, INDEPENDENT INSTANCE OF THE SAME SHAPE, FOUND ON REVIEW: THE CROSS-
PORT TOLERANCE GATE HAD GONE VACUOUS.** The native-port contract requires each Tier-2
band to sit above a *measured* ±1-ULP sensitivity, re-derived from the tree by
`test_crossport.py` on every run — a design chosen precisely so the number could not go
stale. The probe shimmed `domains.biosphere.canopy`'s `math`, because
`intercepted_fraction`'s `exp` was the assimilation path's only transcendental; this
build moved that `exp` into `photosynthesis.canopy_assimilation`, so the shim perturbed
a function the carbon path no longer calls and **both biosphere rows measured exactly
0.0**. `band > sensitivity` then held **against zero** — and the same document names a
reading of 0.0, three paragraphs earlier, as *"a same-libm artifact, not a cross-libm
measurement"*. The gate accepted as proof the very thing its own rule forbids. Repaired
both ways: the probe shims **both** modules holding a Beer-Lambert `exp` (3.5e-15 on the
15-yr chambers, 2.8e-16 on the 7-day greenhouse, both re-recorded in `docs/native-port-
reference.md`), and both band tests now **reject a zero sensitivity outright**, so a
probe that stops perturbing the trajectory goes red rather than quiet. ⚠ Same lesson as
the retraction above, reached from the opposite direction: **a probe is validated by
what it PERTURBS, not by what it is named after — and a gate that re-derives its own
input is only as live as its handle on the tree.** Two of the four freeze contracts'
prose halves were touched by this build; the third (`docs/station-reference.md`) was
checked and carries none of the moved numbers.
**NOT DONE**: the partition table (still a live suspect, still fitted against the biased
assimilation), and the two surviving `TODO(cite)` literals under the 3.5× amplifier
(`extinction_coef`, `carbon_fraction`) — now flagged in `canopy.yaml`'s header as the
highest-leverage provenance work left on this observable.
