## **Three ways to take the step, priced** (the 2026-09-29 review's Step 2, slice 2 — the shipped step leaves the sealed jar's harvest 7 % short, and only a finer step moves it)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**MEASURED 2026-09-30 on the user's call** (*"yes"* to starting Step 2's slice 2). Lab only:
nothing frozen touched, no golden regenerated, nothing decided. Plan, predictions (committed
before any code, `e585aac`), full tables and grading: `docs/plans/post-roadmap-step-options.md`.
⚠ The first commit of this record (`c384fa1`) headlined **−15 %**. That was the day-305 snapshot
of a jar whose grain is still filling, not its harvest. Corrected the same day, before
reporting, on an advisor challenge.

## What was built

* `rust/crates/domains/src/lab/step_options.rs`:
  * **A**, `run_uniform`: any power-of-two steps per day. It is also the source of the fine-step
    answer. Counts come from `SEASON_DAYS`, never from `steps_for_years`/`season_steps()`, which
    carry four steps a day inside them.
  * **B**, `run_split`: probe each step's withdrawals, and take a tight step as two halves,
    recursively, to 1/256 day. It refuses rather than rations at the limit. A half-step sees the
    right time of day because the state's counter is renumbered onto the finer grid (the
    schedules read `t = n·dt`), and set back afterwards.
  * **C**, `ImplicitUptake`: the crop's frozen flows read against an end-of-step pool solved by
    bisection. Two scopes: **crop only** (`X = C₀ − U(X)`, backward Euler on the uptake) and
    **with returns** (the crop's whole carbon budget against `X = C₀ + I + N(X)`, `I` what every
    other flow returns over the same step). The second was added after the first run, to test a
    cause.
* `rust/crates/domains/examples/step_options.rs`: the scorecard. It panics on any failed control.
* `rust/crates/domains/tests/step_options.rs`: 6 tests, the controls that arithmetic fixes in
  advance, on one season. Two deliberate breaks (the counter renumbering; C ignoring its end
  pool) each turned one of them red.

## Controls

B never splitting is the frozen run, bit for bit. B always splitting is Euler ⅛ (and, two deep,
1/16) bit for bit at every quarter-day, across four re-sows. C leaves the open field
bit-identical. C and Euler approach one limit (34 → 2.7 ppm apart from ¼ to 1/64). C's solve
satisfies its equation to 2e-12 mol on every step. The explicit processes never read the end
pool on the frozen jar. **On the leaf-form jar the leaf-area aux process does** (270–342 steps),
so C as built does not reach it. The fine-step answer (1/256 day) converges, but not cleanly at
first order at coarse steps. Its own error, estimated as the last halving, is 0.6–2.7 ppm and
0.03 % on harvest. As a cross-check, it reproduces the direction plan's recorded converged
`open_season` peak LAI to 0.05 % (5.4248 vs 5.4273).

## Findings

**1. The shipped quarter-day step leaves the sealed jar's final harvest 7.1 % short** of a
5-minute step (lab leaf form 7.2 %), with CO₂ errors of up to 241 ppm. At day 305 the jar is
15 % behind: the coarse step fills the grain later. The open field's one-season harvest is 2.1 %
short. The step sweep had measured sealed-chamber harvest at −0.7 % at `dt = 1`, before the
within-day light path. Its table does not say which run or horizon that was, so no trend is
claimed. The light-path record found the open field's canopy step-sensitive (15 %), and the
user's call there was to finish at the shipped step. It measured the jar's day-night swing, not
its harvest.

**2. It is not the crop's uptake outrunning the air inside a step.** C removes exactly that
error, and the harvest moves the wrong way (−7.5 %). C with same-step returns also lets the crop
spend what the chamber gives back during the step, and gives −7.4 %. Both candidate causes are
refuted by the experiment that removes them. The loss lives in the steps where the crop draws
hard: refining only those (B at θ = 0.1) recovers most of it. But that refines every process and
the light at once. **Cause not identified.** The named candidate, the light path's
curved-response bias, is unmeasured for the jar, and its sign does not follow simply.

**3. A moves the error the furthest per unit cost**: −2.5 % at ⅛ (2× the evaluations), −1.4 % at
1/16 (4×), −0.1 % at 1/64 (16×). It is also the only option that fixes the leaf-form jar's peak
leaf area (−4.2 % → −0.2 %). Its price is a step unfreeze of every biosphere golden, plus the
station's plant step, which is unmeasured.

**4. B, built without touching `simcore`, is dominated by uniform refinement at the same cost.**
Its probe cannot share the step's evaluation. θ = 0.25 costs 10.2 evaluations a day for −3.5 %
(A: 8 for −2.5 %). θ = 0.1 costs 15.7 for −2.2 % (1/16: 16 for −1.4 %). At θ = 0.5 it moves only
the jar's golden and buys nothing there. Watching every store, it bottoms out on cold nights,
where the condenser takes the whole humidity excess at any step size, and it moves every chamber
golden.

**5. C is a safety property, not an accuracy fix.** It can never overdraw, so the backstop never
fires on the crop's CO₂. On accuracy it matches the shipped step. It would move every sealed
golden and not the open field. For the user's standing wish (*"make plants take less as the CO₂
runs low"*): C buys no rationing, and no better harvest.

**6. Predictions: 9 graded.** Three held (P6, P7, and P9 for the pre-registered controls). Four
half held (P1, P4, P8). Three refuted: P2 is the finding; P3 refuted in the good direction; P5
refuted on CO₂. The sign in P4 was wrong: C does not sit on the other side of the answer from
Euler. Both leave more CO₂ in the air and grow less.

## Lessons

* **A season-end snapshot of a run that is never re-sown is not a harvest.** The jar's grain is
  still filling at day 305. A "max error over season ends" folded a timing difference into a
  yield headline. The give-away was in the table: peak leaf area within 0.5 % while "yield" was
  15 % off.
* **Grep the prior record before claiming a gap.** "The step decision never compared against a
  converged answer" was false. The step sweep did, and the record said so.

## Open

* **The decision (Step 2, slice 4) is the user's.** The options on the table: A at ⅛ or 1/16 (a
  step unfreeze); C for safety, alone or with A; nothing.
* **The harvest loss's cause.** A mechanism experiment is owed before any claim about it: for
  example, the jar at ¼ day with the light forcing taken from a finer window. → **Opened
  2026-09-30** on the user's *"Go with your recommendation"* (cause first, then the step):
  `docs/plans/post-roadmap-step-cause.md`. **Answered** (`docs/log/step-cause.md`): it is the
  light's time resolution, all of it — on the jar's harvest, "only a finer step moves it" in
  this record's heading holds only because a finer step also resolves the light (on the open
  field the stepping carries real error too).
* **The station**: none of the three options was run through the two-rate driver.
