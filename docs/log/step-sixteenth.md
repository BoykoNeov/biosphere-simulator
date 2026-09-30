## **The biosphere moves to a 1/16-day step, and the crop's CO₂ uptake is taken against the air it leaves** (the 2026-09-29 review's Step 2, slice 4 — the user's "1 + 3", with the test suite held to its old run time)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**BUILT 2026-09-30** on the user's call (*"1 + 3. But ensure the tests runtime would not
quadruple"*, then *"i want the test suite to be roughly the same time as now"*). A biosphere and
station unfreeze in three attributable commits. Plan, predictions (each slice's committed before
its code), amendments, results and grading: `docs/plans/post-roadmap-step-sixteenth.md`.

## What was built

* **Slice 0, the runtime budget** (`99c256e`): `simcore`, `domains`, `station`, `authoring` and
  `config` build at `opt-level = 3` in dev/test (`rust/Cargo.toml`); the Godot binding stays
  unoptimised. Byte-neutral: every golden passed unregenerated, the ignored ones included.
* **Slice 1, the step** (`e15b9f3`): `BIO_DT` ¼ → 1/16, `STEPS_PER_DAY` 4 → 16, the hand-typed
  `dt_days` and station `numerics_note` moved by hand, 11 goldens regenerated (every `n` exactly
  4×). The station's master day now runs in equal groups (`driver::day_groups`) so the lamp
  scenarios' 24 power hours split over 16 plant steps.
* **Slice 2, C**: `science::Co2Read` (loader: `EndOfStep`) and `flows::end_of_step_pool`. In a
  sealed build the allocation flow reads the chamber CO₂ the step leaves (`X = C₀ − U(X)`),
  solved by a bracketed Illinois search that stops only when no float lies between its ends
  and returns the side where the draw cannot exceed the start-of-step pool. 10 goldens; the
  open field is byte-identical by construction.

## Findings

1. **The run time went down, not up**: whole suite 496 s → 251 s, the four ignored tests
   1649 s → 339 s.
2. **The mutual-shading loss was reached only through the quarter-day step's canopy bias.** At
   1/16 the open field peaks at LAI 5.44 (converged 5.43) and no frozen scenario enters the
   regime. On the user's call the science gate shows the loss acting on a pushed run (leaves
   10 % thinner: 6.88 off, 6.08 on). At the finer step the loss acts as a ceiling; the biomass
   cap's tolerance to leaf-thickness error fell (×3.8 → ×2.5).
3. **A rationing finding blamed on the backstop belonged to the jar.** The jar's season-low CO₂
   reverses as its room shrinks. At ¼ that coincided with the backstop firing; at 1/16 the
   backstop fires only below 17 % of the room, and the reversal is complete without it.
4. **The lab leaf form no longer rations the jar at 1/16** (tightest step 0.31 of the pool; 1.15
   at ¼). The jar-breaks reading that holds it back was taken at a retired step; reopening it
   is the user's call.
5. **C keeps the promise, as a test**: jars squeezed to a tenth and a fiftieth of their room
   ration 205 and 1403 times explicitly, 0 times under C. It lifts the jar's CO₂ low point most
   (compensation margin +21 %) and moves harvests by under 0.1 % in the chambers.
6. **Predictions: 14 graded** (8 for the step, P7 left open by design; 6 for C). Held: P2, P4, P5, P6, Q1. Half: P1, P3,
   P9, Q2–Q7. P8 refuted the good way (faster, not the same). The misses were all of one kind:
   a quantity whose SIZE was predicted from a different observable (the jar's margin from the
   harvest's, the solver's cost without its stop rule, the manifest without its hash rows).

## Lessons

* **When a refinement removes a behaviour, look for the tests whose subject was that
  behaviour, not only the ones pinning its numbers.** Four tests existed to show the jar
  rationing; under C they could not go red by a number, only by losing their subject.
* **A tolerance stop makes a solver's answer depend on its path.** The exact-bracket stop the
  advisor asked for costs nothing in accuracy; its first build cost 19.5 evaluations a step,
  and one probe of the neighbouring float brought that to 5–6.
* **Optimisation level is free determinism here**: Rust does no float contraction, so the test
  profile could be optimised with no golden moving, which bought the step its run time.

## Open

* **The lab leaf form** (finding 4): re-measure at 1/16 with C before any reconsideration.
* **The shading ceiling** (finding 2) is measured on one knob (leaf thickness) only.
