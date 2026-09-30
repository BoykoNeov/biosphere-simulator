## **Why the quarter-day step loses the jar's harvest** (the 2026-09-29 review's Step 2, slice 3b — it is the light's time resolution, not the step, and resolving the light inside a quarter-day step recovers it)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**MEASURED 2026-09-30 on the user's call** (*"Go with your recommendation"* — the recommendation
being "find the cause first, then a finer step"). Lab only: nothing frozen touched, no golden
regenerated. Plan, predictions (committed before any code, `b1487ad`), full tables and grading:
`docs/plans/post-roadmap-step-cause.md`. It stops before the step change on purpose: the cause
was measured first because it could change what the step change has to fix, and it did.

## What was built

* `rust/crates/domains/src/lab/step_cause.rs`:
  * **E1**, `run_uniform_blocked_light`: any step, with PAR held at the mean of a coarser window
    (the resolver rebuilt from the public weather forcings, PAR wrapped as
    `base(⌊n·dt/Δ⌋, Δ)`). It goes round the "weather is not part of the seam" guard on purpose:
    here the forcing's time resolution is the variable.
  * **E2**, `LightQuadrature`: a flow wrapper that evaluates its frozen flow at `k` sub-window
    lights (the schedule at the renumbered counter `n·k + j`, step `dt/k`) on the same
    start-of-step state and averages the legs stock by stock. `build_season_light_quadrature`
    wraps all three flows that read the carbon budget.
  * `par_readers` and `dose_partition_worst`: E2's scope and dose controls.
* `rust/crates/domains/examples/step_cause.rs`: the four-cell table. It panics on any failed
  control.
* `rust/crates/domains/tests/step_cause.rs`: 5 tests. Two deliberate breaks (E2's renumbering,
  E1's block) each turned exactly one red.

## Controls

Each instrument at its identity setting (a light block as long as the step; one sub-window) is
the plain run bit for bit at every quarter-day, on the jar and the open field. The sub-windows
partition the dose to 8e-16. Nothing outside the wrapped flows reads PAR (the plan said "exactly
those three"; in the jar growth respiration returns empty before reading it, so the wording was
corrected on the first run, not the instrument). On the open field E1 keeps the light-path
record's canopy bias (+10.6 % of +11.0 %). And a check nobody planned: the water stores, which
read no light, end identical to the answer under E1 and identical to the shipped run under E2.

## Findings

**1. The jar's 7 % harvest loss is the light's time resolution, all of it.** Final harvest against
the 1/256-day answer: shipped −7.14 %; a 1/256 step with quarter-day light −7.19 %; a quarter-day
step with the light resolved to 1/256 inside it −0.02 %. The halves add up to within 0.07 points.
The light sweep (⅛, 1/16, 1/64) reproduces the step sweep's harvest curve to within 0.04 points. The
step-options record's "cause not identified" is discharged.

**2. The missing grain is carbon left in the air.** The shipped jar ends with 0.096 mol C less
grain, 0.072 mol C more CO₂ in the chamber (O₂ the exact mirror) and about 0.03 in the soil
pools. Vegetative carbon at peak LAI is within ±0.5 % in every cell, so it is not carbon parked
in the canopy (prediction Q5, refuted: that is the open field's pattern, not the jar's).

**3. Why the sign flips, named and not measured.** A quarter-day mean of a curved light response
fixes more carbon (the open field's bigger canopy). The jar, living near the CO₂ compensation
point, fixes less. The reading that fits is the growth kink `max(0, GASS − MRES)`, which a
quarter-day mean light can put a whole window on the wrong side of. A top-hat light run would
separate the two.

**4. The open field splits:** light −1.5 points of harvest and nearly all the canopy bias;
stepping −0.9 points and −1.9 % canopy, which E2 exposes once the light no longer masks it.

**5. A new option for the step decision.** Resolving the light inside the budget flows reaches the
jar's answer at a quarter-day step: `k = 16` gets −0.17 % for 1.7× the shipped run time, `k = 64`
gets −0.02 % for 4×, where a 1/16 step costs 4× and stays 1.4 % short. It is a lab instrument, not
a scheme, and adopting its shape would change the frozen science's form (how the budget flows
take the day's light), moving every plant-bearing golden. It leaves the within-step CO₂ swing
(about 31 ppm at worst, the size C addresses). The station is not measured.

**6. Predictions: 7 graded.** Q1–Q4 held; Q5 refuted (above); Q6 refuted as stated (E2 at `k = 64`
cost about what the 1/16 step costs, not a third of it, but for 60 times the accuracy); Q7 half
held (the control-4 wording).

## Lessons

* **When refining a step helps, ask what else the refinement refined.** Here the forcing was
  resolved on the step's own grid, so "a finer step" and "a finer light" were one knob. Pulling
  them apart took a wrapper on each side and a four-cell table; the step sweep's harvest curve
  turned out to be a light-resolution curve all along.
* **A store the variable cannot reach is a free control.** The water stores read no light, so
  they had to follow the step and ignore the light. They did, to every digit, which is a stronger
  proof than any designed control that the two instruments separate what they claim.

## Open

* **The step decision (Step 2, slice 4) is the user's**, now with three shapes: a finer step
  (A), the light resolved inside the budget flows (E2's shape), and C for the within-step CO₂
  swing, alone or combined.
* **Curvature vs the growth kink** (finding 3): the top-hat run.
* **The leaf-form jar and the station:** unmeasured.
