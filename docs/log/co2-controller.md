## **The CO₂ setpoint controller, priced** (the direction plan's Step 0, axis 2)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per
> work item. The fuller record, with every table, is
> [`../plans/post-roadmap-co2-controller.md`](../plans/post-roadmap-co2-controller.md).

**DIAGNOSED 2026-08-13, NOTHING BUILT.** Axis 2 of Step 0 in the consolidated **direction
plan** (the forward-looking plan doc named in this log's own header — deliberately not in
its index, and deliberately not named by filename here, because a record file naming it
would make the index↔record plan-doc parity red), which called the controller the item that
*"may dissolve this whole gate"* and made its step recommendation conditional on this
measurement. Probe
only, on frozen `main`: `git diff src/` empty, no golden regenerated, no schema touched.
Harness `M:/claud_projects/temp/co2-fragility/probe_controller.py` — a **dawn top-up clamp**
in `run_season`'s `reset` hook, composed with `annual_reset` rather than replacing it, with
the probe reservoir carrying `carbon_pool`'s own `{CARBON: 1, OXYGEN: 2}` composition
because `boundary.source` would have defaulted to `{CARBON: 1}` and silently moved oxygen.
Two controls before any result: the no-op clamp is **bit-identical** to `run_perennial` on
all three chambers (digests `231be2442c6df87b` / `f175ea6648ff5c63` / `ecb5f24dde058056`),
and the live clamp **migrates the binding day** (499 → 262, 805 → 262, 2 → 262), so the
probe is neither inert nor measuring the frozen run under another name.

**FINDING 1 — ⚠⚠ THE AMBIENT CONTROL INVERTS THE HYPOTHESIS.** Holding the sealed chamber at
the concentration it already starts from (357 ppm) should be nearly a no-op. It is four
times *worse* than letting it deplete: margin **1.3072 → 0.2977**, season-low CO₂
**57.9 → 10.6 ppm**, rationing firings **0 → 270**. Reproduced on the perennial chamber
(1.5528 → 0.2978) and the consumer chamber (2.0000 → 0.5393). The mechanism is the ratio
itself — `k·h = (rate · h) / stock`: the uncontrolled chamber **self-limits**, because the
pool depletes, `Ci` falls and assimilation falls with it, whereas a controller at ambient
keeps `stock` small *and* `rate` high all season, the worst combination available. **The
controller argument requires the stock to be genuinely large, not merely steady** —
"holding" is not the operative mechanism, only "enriching" could have been.

**FINDING 2 — NO REALISTIC SETPOINT CLEARS, ON ANY CHAMBER.** Both criteria within-run
(season-low pool CO₂ above the 61.07 ppm compensation floor, unclamped margin above 1),
Euler `dt = 1`, 3 yr. Sealed chamber: 357 → 0.2977/10.6 ppm/270 firings; 700 →
0.3320/15.0/279; **1000 → 0.4121/15.7/252**; **1200 → 0.4744/17.4/210**; 2000 →
0.7417/31.4/66; **3000 → 1.0797/319.6/0, the first pass**; 5000 → 1.7559/2257.0/0. The
perennial chamber matches to three digits; the consumer chamber clears earlier (first pass
at 2000) because the crew is a second carbon source, but 1200 still fails it
(0.9515/37.3 ppm/3 firings). **The 1000–1200 ppm band the direction plan assumed a
controller would run at is deep in the failing region on every chamber** — season-low CO₂
~75 % *below* the compensation point, worse than the frozen run's 57.9 ppm, with hundreds of
silent rationing firings. The first passing setpoint is ~3000 ppm, above even the 1785 ppm
cliff the predecessor measured.

**FINDING 3 — THE DISCRIMINATOR: THE CONTROLLER *DOUBLES* THE STEP PRICE.** ⚠ Two numbers at
`dt = 1` cannot settle this — the science case against the shipped step is a **truncation
error**, not a threshold crossing, so the controlled run needs its own convergence check,
all comparisons within one setpoint. At 1200 ppm, controlled Euler runs
17.4 → 39.0 → 586.9 → 893.6 ppm and margin 0.4744 → 0.9478 → 1.8962 → 3.7928 across
`dt = 1, ½, ¼, ⅛`. **Controlled `dt = 1` is 22.2 % low on peak leaf carbon and 20.6 % low on
peak plant carbon against its own `dt = ⅛` limit** — against the frozen tree's 3.2 %, seven
times worse, because holding the pool high sustains an assimilation rate the day-long step
cannot resolve. Stated as the decision needs it: **the frozen tree clears both criteria at
`dt = ½`; controlled at 1200 ppm it needs `dt = ¼`**, and at `dt = ½` it still fails both
(39.0 ppm, margin 0.9478, 6 firings). **The controller does not cancel the step gate — it
doubles the cost of clearing it.**

**FINDING 4 — THE CONTROLLER COUPLES THE TWO KNOBS THE PLAN HAD SEPARATED.** RK4 at `dt = 1`
**hard-errors under the controller at every setpoint tested** (margin 0.7326 at 1200 ppm,
0.7522 even at the passing 3000). The menu treats integrator and step as independent
choices; under a controller the "RK4 at `dt = 1`" row is not a poor buy, it is
**unbuildable** — and at 1200 ppm even RK4 at `dt = ½` raises (0.9528).

**FINDING 5 — THE PASSING SETPOINT GRAZES, AND IT VENTS.** Margin 1.0797 at 3000 ppm is
**8 % of headroom** against a bound `allocation-headroom` finding 7 established is
*emergent* and unguardable at load time — by the direction plan's own criterion, *"a step
with room for the next two mechanisms"*, the passing setpoint fails. It also **vents 235 of
the 429 mol injected**, over a third of the injected carbon thrown out of a habitat whose
purpose is closing the carbon cycle (reported as measured: the probe vents because a
setpoint is two-sided). And the step error **moved rather than vanished** — same two-numbers
discipline as the frozen 24 %/3 %: at 3000 ppm the headline converges (peak leaf +0.010 %,
peak plant −0.236 %) while the season-low pool is still **−88.0 %** against `dt = ⅛`. No
longer decision-relevant at 319.6 ppm; still intact.

**FINDING 6 — THIS GENERALIZES FROM "THE PROBE CLAMP" TO "ANY CONTROLLER".** ⚠ The plan's own
caveat — *a probe clamp is not the controller* — bites exactly here, and the engine closes it
rather than leaving it a caveat. `src/simcore/arbitration.py:75` scales against
`stocks[sid].amount`, the **start-of-step** level, and its docstring says why: *"so
withdrawals never draw against same-step inflows."* A make-up **flow** therefore contributes
nothing to the margin in the step it fires, making it *strictly worse* than a state edit
applied before the step. **The dawn clamp is the upper bound on what any controller can do
for arbitration headroom under this engine's semantics**, so the verdict is about
controllers, not about this probe.

**EXPLICIT OMISSION.** The parked leaf branch (`leaf-expansion-blocked`, `cb668f6`) was not
probed: it draws *more* carbon than the frozen tree, so it cannot rescue a margin already
below 1 on the lighter tree at every realistic setpoint. Recorded as an omission with its
reason rather than left as a silent cap.

**THE VERDICT, AND WHAT IT DOES TO THE PLAN.** *The controller does not dissolve the gate; it
sharpens it.* The §2 warning is **refuted**, and its science-half bullet was wrong in the
*opposite* direction from the one anticipated — the crossing was never about the initial
charge, it is a truncation error that a high sustained rate amplifies, so holding the chamber
makes it worse below ~3000 ppm. The conditional recommendation (Euler `dt = ½`) is
**discharged and stands**, and is now also the *cheaper* branch, since the controller path
needs `dt = ¼` on top of an unpriced make-up flow. The controller **moves back behind the
gate**, for a measured reason instead of an unpriced one. Successors: **Step 0 axis 1** (the
step sweep, now the only remaining input to the step decision, so question 1 should either
wait for it or be put with the candidate steps' prices stated as unmeasured); **the
controller filed as a `dt = ¼` object** with the O₂ regulator's direction hazard and
`authored ≠ validated` still attached; and ⚠ **a `science_bands` entry for the chamber's
minimum CO₂ would not have caught this either** — probe runs are evaluated by no band — so
that entry stays behind the step decision, unchanged.
