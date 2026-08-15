## **The canopy this tree cannot grow** (the light path's own successor — the named candidate, measured and refused)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per
> work item. The fuller record is the plan doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**DIAGNOSED 2026-08-15, NOTHING BUILT; the decision is the user's and is untaken.**
`docs/plans/post-roadmap-canopy-magnitude.md`; probes
`M:/claud_projects/temp/canopy-layers/`. `git diff src/` empty, no golden regenerated, no
manifest touched, no parameter moved — every candidate is a monkeypatch in a throw-away
probe, and every control reproduces its inherited anchor to the printed digit before any
candidate number is believed (peak LAI 5.3806 shipped / 4.7132 converged; perennial
`max(tail)` peak leaf 0.603679). ⚠⚠ **THE DIRECTION PLAN'S NAMED CANDIDATE IS REFUTED ON
ITS SIGN, AND THE TREE HAD SAID SO SINCE PHASE 1.** §6b called a multi-layer / sunlit-shaded
canopy *"the cited next step … and can move canopy assimilation UP"*; `photosynthesis.py`'s
module docstring has read, unchanged since Step 5, that `Ag` is **concave** in PAR so the
big-leaf-at-mean-PAR form *overestimates* the depth integral. Concavity runs one way:
resolving depth **redistributes the same photons** onto a concave response and can only
lower the sum. Measured open-loop on the real `open_season` trajectory — **0 of 2598 lit
calls** came back above 1.0 (the pre-registered prediction, held exactly), season-integrated
**0.9433**; and closed-loop, peak LAI **5.3806 → 5.0314** at the shipped step and **4.7132 →
4.4169** converged, i.e. **~6.5 % the wrong way at both steps**, eating four fifths of the
margin the band is currently passing on. Refused **as a fix**, explicitly not as science —
the big leaf is a known high-bias and a depth-resolved canopy is the more faithful physics;
what is refuted is the plan's claim about which way it moves. ⚠⚠ **RETRACTED 2026-08-15 — "the cited CHEAP scheme is
worse than the expensive one" was an artefact of the probe, not a property of the scheme.**
The probe labelled "3-point Gaussian" implemented a **midpoint rectangle rule**; the real
Goudriaan scheme (abscissae `0.5 ± 0.5·√0.6`, weights `5/18, 8/18, 5/18`) tracks the
100-layer integral to better than **0.2 %**, where the midpoint rule sat 5.2 % low. The
withdrawn numbers were 0.9300 vs 0.9433 open-loop and 4.9634 vs 5.0314 closed-loop. The
lesson is the inverse of the one drawn: **a probe that names a scheme is not evidence that
it implemented that scheme** — the name was in the docstring, the arithmetic disagreed three
lines below, nothing compared them, and the number reached a plan. ⚠ **It would NOT have taken a liveness floor
red, and the reason is the canopy regulator's reason verbatim**: §6b warned that a
mechanism lowering assimilation might red perennial's peak-leaf floor (3.2 % over its bound
on its settled value), so it was run rather than assumed — `perennial_long_horizon` 15 yr
under the *worst* scheme moves `max(tail)` **0.603679 → 0.603540 (0.023 %)** and moves the
annual-min CO₂ floor *up* in the seventh digit, because the correction scales with canopy
**closure** and every chamber peaks at LAI 0.07–0.63 where the ratio is 0.99995–0.996. A
mutual-shading mechanism is inert in a habitat whose canopy never closes — measured for the
regulator in 2026-07-27 and now measured again for an unrelated mechanism arriving from an
unrelated direction; the damage is confined to `open_season`'s band. ⚠⚠ **THE FINDING THAT
REFRAMES THE ITEM: the band's subject is a 3.5× AMPLIFIER.** Closed-loop at the converged
step, ×1.05 on specific leaf area gives **+18.0 %** of peak LAI and ×1.05 on gross
assimilation **+17.1 %** — the canopy compounds through interception, so clearing the 6.1 %
gap to the floor costs about **+1.8 %** of either quantity. ⚠ **Both are the elasticity to a
UNIFORM perturbation and the scope is load-bearing**: a *closure-weighted* one amplifies far
less — the layered canopy's 5.7 % of season assimilation costs 6.5 % of peak LAI, i.e.
**~1.14×**, because its correction is near-zero while the canopy is open and only reaches
−8 % at closure, small exactly where the compounding would have multiplied it. So "+1.8 %
clears the gap" is a statement about a *uniform constant* (which `specific_leaf_area` is),
not a budget any mechanism can be held to. Three of the inputs that set
peak LAI are `TODO(cite)` provisional literals (`specific_leaf_area: 22.0`,
`extinction_coef: 0.6`, `vcmax`/`jmax`), so **a ±2 % provenance disagreement about any one
of them is the entire distance to the bound**: a band whose subject amplifies every
parameter error 3.5× can say the tree is in the right decade and cannot arbitrate between
mechanisms. Reading 4.71 as evidence *about a mechanism* reads it past its resolution —
which makes this a **provenance** finding, not a mechanism finding, and that is a different
queue. ⚠⚠ **THE SLA ROUTE HAS BOTH SIGNS, AND WHICH ONE YOU GET IS THE UNDOCUMENTED CONTENT
OF A `TODO(cite)`.** Every SLA table in the literature declines with development, so keying
the frozen constant is one mechanism — but it is ambiguous about something `canopy.yaml`
never says: what 22.0 m²/kg **is**. Two crude uncited ramps (a sensitivity test of the FORM,
run deliberately *before* any source hunt so a source hunt is not spent on a dead route)
bracket it: early-anchored (22.0 is the young value, 22.0 → 15.0) gives peak LAI **3.0446,
−35.4 %** — worse than the refused layered canopy; late-anchored (22.0 is the mature value,
29.0 → 22.0) gives **8.2391, +74.8 %** — clears the floor and overshoots the 8.0 ceiling.
**The same mechanism spans 3.04 to 8.24, a 2.7× range, on a question the parameter file does
not answer**, so "key SLA to development" is not a proposal until the anchor is sourced.
⚠ **And the late-anchored reading moves a SECOND, independently-pinned defect the right
way**: peak LAI on the frozen tree sits at **DVS 1.373**, *after* anthesis, which is already
pinned as a known gap against **two independent oracles of different model families**
(`test_gap_lai_peaks_after_anthesis_not_before` — LINTUL3 peaks 2 days *before* its own
anthesis, ours ~13 days after, same direction as the WOFOST winter-wheat oracle); the
late-anchored ramp moves the peak to **DVS 0.958, before anthesis**, where both oracles put
it. One candidate moving two symptoms recorded separately, one of them by an oracle the
candidate knows nothing about, is the opposite of the fitted-table shape
(`wheat-partition-backfill-refused`) — and is still only a **direction**, since the ramp is
crude, uncited and overshoots. **THE DECISION, LEFT OPEN**: (1) build the layered canopy as
fidelity anyway, accepting the band at a **0.6 % margin** at the shipped step and a deeper
converged failure; (2) take the SLA anchor as a targeted citation question against [A],
which is where finding 4 says the leverage actually is; (3) neither — the deviation is
already documented and the band still passes at the shipped step. ⚠ (1) and (2) are
**independent** and building both is coherent — it is the only combination in which the
layered canopy does not eat the band's margin, and it is also the largest. **NOT MEASURED**:
the partition table (§6b's third candidate, a live suspect because it was fitted against the
biased assimilation), and `open_season`'s other gates under either candidate — only the two
perennial liveness floors were run, because nothing is proposed for shipping.
⚠⚠ **DECISION TAKEN 2026-08-15: the user chose (1)+(2), and the build is
[`layered-canopy.md`](layered-canopy.md).** Read that row for what actually happened; §7b of
the plan doc records how far this diagnosis's own price list was from the measured outcome.
