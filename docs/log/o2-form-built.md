## **The live-O₂ FvCB form is BUILT, both halves, lab-only** (every prediction held — and the one finding is that a "saturates" claim had been carrying a scope it was never measured at)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md), written
> under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per work item.
> Plan of record: `post-roadmap-o2-form.md`. Filed under the September direction plan
> (referred to by that name here, never by filename — see the log's exemption note).

**BUILT 2026-09-06 on the user's call ("so build both").** That answered *which halves*; the
repo's own precedent answered the rest, so this took the shape the Q10 temperature form took —
a cited alternative reachable only from the lab, with the loader always selecting the frozen
reference. **No unfreeze: no param, golden, manifest key, band or floor moved.** Adoption is a
separate decision and is **not** taken here.

**FINDING 1 — every prediction in the plan's §5 held, including the one written to fail.**
Measured with `science_switch -- o2form=live_pool --long`:

| row | frozen | live O₂ | predicted |
|---|---|---|---|
| `open_season` peak LAI | 6.022837 | **6.022837** (+0.000 %) | 0.000 %, by construction |
| `open_season` peak W | 13.379084 | **13.379084** (+0.000 %) | 0.000 %, by construction |
| `sealed_chamber` low CO₂ | 71.435803 | **7.294541** (−89.789 %) | order 10 ppm or below |
| `perennial_chamber` low CO₂ | 70.252606 | 70.351292 (+0.140 %) | < 0.5 % |
| `consumer_chamber` low CO₂ | 73.338613 | 73.425099 (+0.118 %) | < 0.5 % |
| `perennial_long_horizon` peak-leaf | 0.578137 | 0.577999 (−0.024 %) | ~0.1 %, **not** 0.417240 |

⚠ **The cross-check is what makes this a form and not a coincidence.** The plan wrote down
**7.183490 ppm** before the run — the 2026-09-02 *global* substitution at the jar's charge
fraction — with the rule *"if the form lands far from ~7 ppm, the wiring is wrong, not the
science."* It landed at **7.294541**, 1.5 % away, and the residual is exactly what the two
things differ by: the global column pinned O₂ at the charge fraction, the form tracks the
stock down through the season.

⚠ **And the perennial row is the harder half of that same check.** The global substitution put
that fixed point at **0.417240 — below its 0.55 liveness floor**. The form moves it
**0.024 %**. A form wired globally would have reproduced the counterfactual; this one
reproduces the *scenario shape*, which is the whole difference between the two.

**FINDING 2 — a recorded "saturates" claim was carrying a scope it had never been measured
at, and the test that assumed it went red.** `log/o2-coupling-measured.md` reads: *"the
denominator **saturates by ~2 mmol/mol** (`o2=2` and `o2=0.033` agree to six figures), so all
of the form's time-dependence lives in Γ* and none in the term the gap was written about."*
Six figures is true of the **whole-run gated observable**. It is **not** true of the
leaf-level rate the sentence is phrased as being about: at `Ci = 300`, `Ac` moves
**36.344563 → 36.492040** between the jar's charge and its end, a relative change of
**4.06e-3** — three orders of magnitude coarser than "six figures".

Both readings are defensible descriptions of a real saturation; only one is what was measured.
The run-level agreement is the chamber being **carbon-limited**, not the arithmetic being
flat. The test now pins the claim that is actually true of the arithmetic — the term moves
~140× less below the jar's charge than it does between the atmosphere and that charge — and
pins `Ac`'s 4e-3 in a *band* (`1e-4..1e-2`) so neither the six-figure reading nor a
"large" one can be quoted from it. *A finding is scoped to the observable it was measured on,
and a sentence that names a different one is a new claim.*

**FINDING 3 — the guard against a silent baseline could not be put in the code, so it is a
test.** The form is only half a switch: it rides the params object, but the value it reads is
a **stock**, supplied per step by `CarbonContext::o2_pool_var`. A scenario with no O₂ pool
falls through to the frozen constant — correct for the open field, which breathes the
atmosphere, and **indistinguishable from having forgotten to wire the field at all**. That is
exactly the failure `lab::biosphere_with`'s own header names: patch the wrong symbol, get a
clean run, read the baseline back, report "no effect" as a finding. Nothing in the code can
separate those two, so `the_sealed_chamber_moves_under_the_live_form` asserts the jar moves by
more than half its value. If that ever passes by reading the baseline, every other test in the
file passes too.

**FINDING 4 — the mutation that matters is aimed at Γ\*, and the denominator half may have
none.** Named before the build rather than discovered after. Because the denominator flattens
below ~2 mmol/mol and the jar spends its whole season below that, a form that coupled **only**
the denominator would be nearly invisible on every gated row, while one that dropped Γ\* would
be loud. So the two halves are pinned at the *arithmetic* level —
`gamma_star_is_linear_in_the_oxygen_fraction` and
`the_rubisco_denominator_saturates_above_the_jars_charge` — rather than left to a whole-run
mutation the saturation would swallow. This is `s5-batch-f`'s *a redundant guard has no
mutation that reddens*, anticipated instead of found.

**FINDING 5 — the report REFUSES the floor rather than printing it stale, and that is a
one-line change with a real trap behind it.** `min > Γ*/ci_ratio (61.07 ppm)` is written
against a **constant** floor. Under this form Γ\* tracks a stock, so the claim becomes
pointwise — a different assertion, not a re-tuned one. **The frozen guard is left exactly as
it is**; re-posing it would be an unfreeze in a build whose whole claim is that it unfreezes
nothing. But the params object still holds 42.75, because the substitution happens per step
*inside the flow*, so the report's floor line would have printed **61.071429** and invited a
reader to divide the jar's 7.29 by it. `Column::floor_ppm` is now an `Option`, and the live
column prints `n/a` **with the reason in the cell**. *A number in prose acquires no owner* —
so the fix is to make the number unavailable, not to annotate it.

**What is now owed, and it is a decision, not a task.** Whether to adopt. The jar's science is
the only thing at stake (one row of one scenario), and adoption would require re-posing the
compensation guard as a pointwise claim **first** — argued in writing, before the run, never
retuned to fit. Nothing in the queue waits on it.

**Gates, run on the committed tree, counts read off the whole output and written after the
run.** `cargo run --release -q -p station --example regen_goldens`: **20 of 20 goldens run; 0
would change.** `cargo clippy --all-targets -- -D warnings`: **exit 0.**
`cargo test --workspace --no-fail-fast` from `rust/`: **1144 passed, 1 failed, 68 result
lines** — and the red is not green, so it is written down as what it is.

The red is `godot_bridge::cross_boundary::the_perturbed_brownout_crosses_the_boundary`, at
`cross_boundary.rs:260` — the *marker-parse* site. Godot exited without printing a report at
all, so it is a **no-report** failure, not a value mismatch: nothing compared, nothing
disagreed. Three things bound it, and none of them is "it passed the second time":

- **Unreachable from this change.** The loader always selects `O2Form::Constant`; the only
  edit since this same suite ran **1145 / 0 / 68 green on a tree already carrying this build**
  is the NaN guard inside `o2_coupled`, and only `LivePool` reaches it.
- **A stronger value-level check covers the same scenarios.** `regen_goldens` re-ran all 20
  goldens — the station rows this test asserts against included — and **0 would change**.
- That binary took **595.74 s against its own 600 s limit**, and it was the first run after
  `domains` changed, so the cdylib was genuinely being rebuilt underneath it.

⚠ **The parallel failure is UNREPRODUCED and its cause UNMEASURED.** The isolated re-run
(`-p godot_bridge --test cross_boundary -- --test-threads=1`) was **19 / 0 in 233.34 s**, this
test included — but that is a *different experiment*, not a repeat, so it licenses no verdict
of "flaky". And the capture filter kept only `^test result:` lines, which **discarded the
panic's own stdout/stderr dump** — the one artifact that would have said whether Godot failed
to load the extension or died some other way. *A grep that selects for the summary throws away
the diagnostic, and you find that out only when there is something to diagnose.*
