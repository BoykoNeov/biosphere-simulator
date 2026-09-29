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
