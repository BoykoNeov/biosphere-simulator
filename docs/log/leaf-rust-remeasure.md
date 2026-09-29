## **The parked leaf mechanism, re-measured in Rust** (the "cheap" step was a port — and it breaks the jar the finer step had cleared)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**MEASURED 2026-09-29 on the user's call** (*"go with your recommendation"* — the re-measuring
step of the third direction plan's §2.1 only). Plan, predictions and the full table:
`docs/plans/post-roadmap-leaf-rust-remeasure.md`. Predecessors: `docs/log/leaf-expansion.md`,
`docs/log/leaf-remeasurement.md`.

## The price was wrong before anything ran

The recommendation called re-measuring *"cheap, changes nothing"*. The mechanism existed only
as Python on a parked branch, and S6 deleted `src/`, so re-measuring meant **porting it far
enough to run**. Said to the user before building. The half that stayed true: it is a
**lab-only form** (`LeafAreaForm::NodeEnvelope`, loader sets `Derived`), its seven numbers are
cited consts in `science.rs` on the `TEH_Q10_*` precedent (a `leaf_area.yaml` would have joined
the frozen param census, i.e. an unfreeze), and **20 of 20 goldens are identical**, no manifest
byte moved.

## What the port had to get right to measure anything

Five places a straight port would have corrupted the measurement, each closed:

* the peak-LAI readout derived LAI from leaf carbon — under the form it reads the stored state;
* the aux key is seeded only under the form (the branch seeded it always);
* the re-sow: `annual_reset` now **refuses** a state storing leaf area, and
  `annual_reset_with` / `run_perennial_with` reset it — the branch without it rationed 85×;
* mutual shading still reads derived LAI, as on the branch — **a recorded decision, unmeasured**;
* the `dt` contract: analytic node rate, and the envelope as a projection on the state.

## Findings

**FINDING 1 — THE JAR BREAKS, UNDER BOTH INTEGRATORS.** `sealed_chamber` rations **5** times
under Euler (frozen: 0) and RK4 raises at step 773, day 193 of season 1, `scale_f = 0.978`.
August's *"the RK4 blocker is gone at `dt = ¼`"* does not hold on today's tree. The day is
close to August's day 197 — recorded as a coincidence of timing, **not** a cause. Two changes
since August bear on the jar (live O₂ adopted; mutual shading made live by the layered canopy);
no cause is claimed, because naming one needs a control nobody's decision needs yet.

**FINDING 2 — THE CHAMBER CASE HOLDS, AND IT COSTS A THIRD OF THE CO₂ HEADROOM.** Perennial
and consumer chambers: peak canopy **1.291× / 1.303×**, zero rationing, RK4 clean. The CO₂
compensation ratio is a FLOOR; its margin falls 0.150 → 0.100 and 0.201 → 0.140 (**−33 % /
−30 %**). The plan predicted "< 10 % of margin" and is falsified — August's "~4 %" was a change
in the ratio, and the plan's sentence mixed the two.

**FINDING 3 — NOT INERT IN THE OPEN FIELD ANY MORE.** `open_season` peak LAI −5.3 % (6.02 →
5.70). August measured −0.26 % at the same step on a tree that no longer exists. Greenwood peak
W is a ceiling and moves down, i.e. more headroom.

**FINDING 4 — THE ENVELOPE HOLDS AT STEP ENTRY, NOT AT THE SAMPLE.** Sampled thinness reaches
0.628 against a 0.667 floor: every extreme is the bound divided by that step's leaf-carbon
change, exactly (day 10.75, +3.6–6.1 %). August recorded only the ceiling-side overshoot; the
floor side is bigger because a seedling grows fastest. The gate asserts the step-entry form,
and dropping the floor clamp reddens it.

**FINDING 5 — DROUGHT IS UNMEASURED.** The leaf drought factor fired on **0** steps of any run.
The two scenarios that exercised it were retired by C6, and drought response was the reason
this mechanism was built.

## What was deliberately not done

No retune (narrowing the envelope or refining the step until the jar goes quiet is what
`leaf-expansion.md` finding 9 refused); no cause-hunting control; no adoption. The form stays
reachable from `science_switch -- leafform=node_envelope` and moves nothing frozen. **Whether
the rewrite continues is the user's decision.**

## The jar control (2026-09-29, the same day) — an INTERACTION, and a thin reference margin

User: *"the next step is the control run that finds out why the jar fails"*. Plan §5, predictions
written first, all five held. Instrument: `examples/jar_control.rs`.

* **What overdraws:** `biosphere.allocation` draws more CO₂ (`biosphere.carbon_pool`) than the
  jar holds, days 193–197 of season 1, once a day; RK4's flow #0 is the same flow.
* **The 2×2** (leaf form × O₂ form): only lab × live O₂ rations. Frozen × live O₂, frozen ×
  constant O₂ and lab × constant O₂ are all 0 firings, RK4 clean. **Neither change alone breaks
  the jar.** Mutual shading measured uninvolved (LAI ≤ 0.67 against 6).
* **Why:** at 0.2 % O₂ the live form removes photorespiration, so the crop pulls the jar's CO₂
  to ~7 ppm instead of stopping at ~70. The reference jar's tightest step (777) already draws
  **76 %** of the pool; the lab form, at the same step, draws **115 %** — its crop asks **15 %
  more** and its pool arrives **24 % lower** (drawn down earlier). ⚠ First written as "~50 %
  more demand"; a ratio of ratios, corrected in review.
* **⚠ Outlives the leaf form:** on the reference run step 777 withdraws 76 % of the CO₂ it
  starts with, and **no gate measures that distance** — searched, not asserted: the jar's trough
  is pinned (×10.674948, 2 %), but to the compensation floor; `rationed == 0` is binary; no
  test in `rust/crates` bounds a step's draw on `biosphere.carbon_pool`. Left for the user.

## The jar's step draw, PINNED (2026-09-29, the same day)

User: *"Add a test that tracks how close the jar's worst step comes to running out of CO2."*

* **The pin:** `science_gates::margins::the_jars_tightest_co2_step_is_pinned_by_its_headroom`
  holds the reference jar's tightest step at **0.756662** of its CO₂ pool (step 777, day 194.25
  of season 1). Read off the test's own red with a deliberately wrong pin first; the control's
  0.757 was the prediction and it held.
* **The tolerance is on the headroom (`1 − draw` = 0.243), ±2 %, i.e. ±0.005 on the draw** —
  not 2 % of the draw, which would let ~6 % of what is left go unnoticed (the P7 ratio-vs-margin
  mix again). Mutation: pinning 0.762 (0.7 % on the ratio, 2.2 % of the headroom) reddens it.
* **The instrument:** `readouts::step_draws` evaluates every flow at each step's entry state and
  divides each clamped stock's summed withdrawals by what it holds. It asserts every step that
  its firing call matches the backstop's own `scale_factors` AND the integrator's `rationed` —
  the demand sum is a copy of simcore's private one, and that assertion is what keeps the copy
  honest. `examples/jar_control.rs` now uses the same `withdrawal_demand` and reproduces §5a's
  table exactly.
* **Control:** `leaf_form.rs`'s jar test reads the lab form's CO₂ draw through the same function
  and asserts `> 1` on the run that rations, so the pin is not a probe that never sees a squeeze.
* Not a `science_gates!` row (that is a manifest entry, an unfreeze) and not in `PINNED` (tied
  to the compensation-band roster). A characterisation pin; re-pinning it is an ordinary edit.

## Which part squeezes the jar (2026-09-29, the same day) — the SEEDLING phase, carbon-starved

User: *"continue work on the leaf mechanism"*. Plan §6/§6a, predictions first; instrument
`examples/jar_squeeze.rs`.

* **My design's premise was wrong, and the run's first line said so:** the leaf-growth cutoff is
  at day 225.5, the squeeze at days 193–197. The one whole-run control ("cap the area after the
  cutoff") therefore never engaged — **uninformative by construction**, not "held".
* **Early the lab crop is SMALLER** (plant carbon −36 % at day 162.5): the seedling rule keeps
  leaf area below what its carbon implies, so it grows slower and leaves the jar's CO₂ in place
  ~12 days longer. **At the squeeze it is BIGGER** (+18.5 % plant, +30 % leaf carbon), with the
  same total carbon in the jar sitting less in the soil.
* **During the squeeze** leaf carbon falls in a CO₂-starved jar while the seedling rule (thermal
  time, not carbon) keeps pushing area up; the thin-leaf ceiling (1.18×) is the only coupling.
* **Split at step 777:** removing the extra area alone → 0.93 of the pool (one-step
  counterfactual, not a run); lower CO₂ and more leaf carbon roughly cancel; O₂ nothing.
* Graded: Q3–Q5 held; Q1's side held but its reason was falsified; Q2 falsified as a story;
  Q6 uninformative. **Open, a source check not a run:** does [F]'s seedling phase limit area
  growth by carbon supply, a rule the port may have omitted with the envelope standing in?
