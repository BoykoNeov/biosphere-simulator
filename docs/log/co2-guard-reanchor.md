## **The decade CO₂ guard, re-anchored** (the stem-only verdict's blocking contract question, answered by measurement rather than by choosing)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**COMPLETE (2026-08-10) — a `liveness_floors` `bound`+`source` unfreeze, EXACTLY 2 LINES of
the biosphere manifest; no value, golden, param hash or `src/` change. The smallest unfreeze
this contract has taken, and it is a TIGHTENING.**
`docs/plans/post-roadmap-co2-guard-reanchor.md`; probes `M:/temp/co2_guard/`. The question
was *"does `transient=2` fit a tree whose settling transient the CUE build measured at ~35
yr?"* with the trap named in the same breath (`transient=3` clears the floor, `5` clears
stationarity — picking a window because the subject goes green is the consumer-chamber-2× /
DPM-RPM / ruling-B / fractionation-seed-sweep shape, refused four times). **It was not
answered by choosing a window: it was answered by measuring that the window does no work.**
Discriminator used throughout — *would this justification survive if stem-only were deleted
from the record?* — and every claim below does. **FINDING 1 — the window is INERT on the
frozen tree, in BOTH assertions**: whole-run min **0.055175** (year 1) = **1.103×** the 0.05
floor and **no year dips below it**, so `non_collapsing(summaries)` and
`non_collapsing(summaries[2:])` are both True; `is_stationary` is True at `transient` = 0, 1
**and** 2 alike. ⇒ the slice constrained **nothing on the reference and only candidates** —
the one shape a frozen contract's guard must not have, a knob whose entire effect is on the
things the contract judges. Removal is a **strict tightening** (`non_collapsing(whole)` ⟹
`non_collapsing(sliced)`), so the teeth cannot decrease — a proof, not a measurement.
**FINDING 2 — the comment the window carried described a tree that no longer exists**: *"the
year-2 CO2 minimum dips to ~0.039 … before settling to ~0.055"* is pre-CUE on both numbers
(current: nothing below 0.05, settles **0.0733**). **The CUE build restated four guards and
this one was not among them**, so its assertions kept passing while its justification
rotted. ⚠ The sweep was **run, not assumed**: three live sites quoted pre-split CO₂ numbers
as present tense — this comment, `test_senescence_form.py`'s docstring (which *validated
itself against* this comment) and `test_biosphere_stress.py`'s `CO2min 0.039` — all
corrected in place; plan-doc tables recording what was measured *then* left alone, because
those are the record. **⚠⚠ FINDING 3 — THE GUARD DOES NOT DETECT WHAT ITS COMMENT SAID, AND
THE REFUTATION IS THE TEETH.** The comment claimed it shows *"closure is not slowly draining
the atmosphere into biomass"*, so the natural teeth check is to starve the loop — and **both
obvious levers make the trough SHALLOWER**: `microbial_respiration_rate` ÷2 (the drain
mechanism itself) gives **0.057797** and `chamber_co2_mol0` −20 % gives **0.058757**, both
*above* the frozen 0.055175, because everything downstream self-limits (less carbon reaching
the plant grows a smaller plant, which draws less). What trips it is the **BUFFER**: the jar
shrunk **0.8×** at fixed composition (Ci₀ and x_O₂ invariant) fails at **0.044941**, and
**0.7×** fails at 0.045871 **while stationarity PASSES** — the "clean attractor in the wrong
place" failure the level check exists for, now witnessed by a mutation that is **not a
candidate science change**, where that claim previously rested on stem-only, i.e. on the
very change the guard's verdict was being used to refuse. ⇒ it is a **buffer-vs-peak-demand
guard on `biosphere.carbon_pool`** — the acceptance-gate census's binding stock, and the
chamber-scale *"the atmosphere is a buffer of hours"* as a committed assertion; the
chamber-scale diagnosis reached independently for the **sixth** time. The negative result is
**committed as a test, not left in a probe** (`docs/retired/mineralization.yaml`'s reason: a
stale negative suppresses the next search, and a **counterintuitive** one suppresses it
hardest — a reader reaching for the recycling rate gets a green bar and concludes the guard
is toothless). **FINDING 4 — the anchor**: run to 50 yr the trough converges to
**0.07329124** (flat to 6 dp over the last eight years) = **1.466×** the floor, so the bound
sits below a *measured* attractor rather than beside a passing 15-yr reading (the CUE
build's own idiom for the `> 0.55` floor, applied to its sibling); **and the deepest year of
the fifty is year 1, INSIDE the frozen horizon** — without which removing the slice could
have traded one blind spot for another. **FINDING 5 — `_TRANSIENT` was NOT touched globally,
and the measurement is why**: it is load-bearing for both leaf pins (`transient` 0/1 →
**False**) and inert only here. Removing it from *this* stationarity call was drafted and
**dropped** — the binding same-phase diff (0.013618, **90 % of bound**) sits at index 2 and
is **not dropped by `transient=2` anyway**, so removal buys an *identical* constraint while
spending the remaining 10 % of headroom. Inertness justified removing the slice that was
hiding a failure; **nothing hides behind this one**, and `_TRANSIENT` keeps meaning one
thing across all four uses. **FINDING 6 — stem-only's verdict is UNCHANGED** (0.046065 at
year 2, failing the floor inside *and* outside the removed window, and failing stationarity
at fixed indices 2–3) ⚠ **and that is a fact about stem-only, NOT evidence the re-anchor is
right** — the justification is findings 1 + 3 and stands either way. The contract question
it poses is now *sharper* (frozen transient 1.103× the floor vs stem-only's 0.921×) and **is
still the user's**; re-deciding it inside the commit that moved its guard is the refused
shape. **Science-gate report**: `science_bands` structurally untouched (none on
`perennial_long_horizon`; `open_season`'s three unaffected, golden hash unmoved); every
other floor passes unchanged; no bound value moved. **Teeth verified by MUTATION**: dropping
the `science_gate` marker takes `test_frozen_science_gates_are_complete` **and**
`test_science_gate_bounds_name_a_literal_present_at_their_locus` red. Station manifest
untouched (`grep -l` confirmed it names no `perennial_long_horizon` gate). `git diff src/`
empty; ruff + pyright clean.
