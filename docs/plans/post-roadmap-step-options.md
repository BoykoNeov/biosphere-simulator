# Post-roadmap — three ways to take the step, priced against a fine-step answer (Step 2, slice 2)

**Opened 2026-09-30**, on the user's *"yes"* to *"Shall I start?"* — slice 2 of Step 2 in
`docs/plans/post-roadmap-review-2026-09-29.md` ("small air volumes against a big step"). Slice 1
(`docs/plans/post-roadmap-draw-census.md`) found that the only store under real pressure is a CO₂
pool drawn by the crop. The source search (`docs/plans/post-roadmap-co2-uptake-source.md`) found
that option C needs no new biology: it is a way of taking the step. Record: `docs/log/step-options.md`.

**Scope: lab only.** Nothing frozen is touched: no `simcore` edit, no frozen flow edited, no
golden regenerated, `readouts::step_draws` untouched. Nothing is decided. This slice builds the
three options side by side and scores them; slice 4 (the decision) is the user's.

## 1. The question

At the shipped quarter-day step, the sealed jar's crop takes up to 0.757 of the chamber's CO₂ in
one step, and the lab leaf form pushes that to 1.15 (the backstop rations). Three fixes were
proposed. For each: **how close does it come to the answer a much finer step gives, at what cost,
and how many frozen results would it move if adopted?** "The backstop stopped firing" does not
count as a pass: B and C make an overdraw impossible by construction.

## 2. The three options, as built

* **A — a uniformly finer step (⅛ day).** The existing runners with `dt = 1/8`, run for
  `305 · years · 8` steps, with the re-sow every `305 · 8` steps. ⚠ `steps_for_years` and
  `season_steps()` carry `STEPS_PER_DAY = 4` inside them and are **not** used off ¼ day: a wrong
  count still runs cleanly, over the wrong number of days (the step-unfreeze's own mistake).
* **B — split a step only when it is tight.** A lab driver in `domains`. Before each step it
  evaluates every flow on the step's starting state (the census probe) and takes the largest
  `withdrawal ÷ amount held` over the watched stores. Above a threshold θ the step is taken as
  two half-steps, and each half is judged again the same way, down to a depth limit (1/256 day).
  Still above 1 at the limit is a **refusal** (an error), never a ration. The rule reads only
  the state, so a run stays repeatable.
  * *How the half-steps see the right time of day.* The forcing schedules compute
    `t = n · dt` from the state's step counter. A half-step is taken on a copy of the state
    whose counter is renumbered to the finer grid (`2n`, `2n + 1` at `dt/2`), and the counter is
    set back to `n + 1` afterwards. The only other reader of the counter in the biosphere is the
    re-sow timing, which the driver applies on the coarse counter itself. No biosphere flow
    draws random numbers. The phenology counters are aux processes the integrator advances by
    `rate · dt` on every (sub-)step, so a split step advances them by the same total up to
    rounding — the open design question the review named, answered by the engine's own aux path.
  * Variants: the watched set is **the chamber CO₂ pool** (θ = 0.5, 0.25) or **every store**
    (θ = 0.25).
* **C — the crop's uptake taken against the air it leaves behind** (backward Euler on the
  crop's uptake alone). A lab flow `ImplicitUptake` wraps the frozen `biosphere.allocation` and
  solves `U = U_flow(C₀ − U)` for the step's uptake `U` by bisection on `[0, min(U_flow(C₀), C₀)]`.
  The inner flow is then evaluated once at `C₁ = C₀ − U`, so its legs stay balanced by
  construction. Every other flow still reads the start of the step (so the respiration returns
  stay explicit: an implicit/explicit split, stated as such).
  * Only the allocation flow is wrapped. Growth respiration is empty in a sealed chamber (source
    and sink are the same pool). Maintenance respiration reads assimilation only through its
    organ-burn shortfall `max(0, MRES − GASS)`, and the argument is that it is zero at both `C₀`
    and `C₁` whenever the crop takes anything. That argument is **asserted on every step**, not
    relied on (§3).
  * Where the season has no chamber pool (the open field), the wrapper passes the inner flow
    through untouched.

## 3. Controls — each must hold before any number is read

1. **The fine-step answer converges.** Plain Euler at ¼, ⅛, 1/16, 1/32, 1/64, 1/128 and 1/256
   day. Each metric's difference between successive step sizes must roughly halve (first
   order) over the last three halvings, and the 1/256 run must ration zero times. If it does not
   halve, some part of the model depends on the step size and there is no fine-step answer to
   score against — which would itself be the finding.
2. **B never splitting is the reference run.** B with θ = ∞ must end bit-identical to
   `run_season` / `run_perennial` at ¼ day (which the golden gate ties to the committed golden).
3. **B always splitting is the finer uniform run.** B forced to split every step once must be
   bit-identical to plain Euler at ⅛ day at every coarse step; forced twice, to 1/16. This is the
   only check that proves the counter renumbering (forcing times, phenology counters, re-sow).
4. **C on the open field is the golden run**, bit for bit (no pool, pass-through).
5. **C converges to the same answer.** C at 1/256 must sit far closer to Euler at 1/256 than the
   two sit at ¼ (both are first-order schemes with one limit).
6. **C's solve, every step:** the bracket brackets (`g(0) ≤ 0 ≤ g(hi)`), and the returned uptake
   satisfies `U = U_flow(C₀ − U)` to 1e-12 of the pool. The largest residual is printed.
7. **C's scope, every step:** maintenance and growth respiration give bit-identical legs at `C₀`
   and at `C₁`. A single step where they differ means C as built is not "the crop's uptake
   only", and the result is not read until that is understood.

## 4. The scorecard

Cases, each driven the way its golden drives it:

* **the sealed jar**, frozen science, 3 seasons, no re-sow (the golden run);
* **the sealed jar under the lab leaf form** (`LeafAreaForm::NodeEnvelope`), same horizon — the
  case that rations at ¼ day.

Against the 1/256 Euler run, compared at every quarter-day boundary (a grid every option shares):

* **CO₂ error in chamber ppm, absolute** — the largest and the mean over the run. (Relative error
  is refused: near the compensation point the pool is small and a relative error there would
  dominate the table.)
* **Yield** — storage carbon at each season's end, the largest relative error over the seasons.
* **Peak leaf area index**, relative error.
* **Cost** — full flow-set evaluations per simulated day (B's probe is a full extra evaluation
  per step, counted), plus C's extra allocation evaluations, plus wall time.
* **Backstop firings.**

Across all six frozen biosphere runs (open field, jar, perennial and consumer chambers at 5 and
15 seasons), at ¼ day: **would the frozen result move?** A: every run by construction. B: the
count of steps its rule splits (zero means that golden would not move). C: whether the end state
differs from the frozen run.

⚠ **The station is not measured here.** Its crop draws on the cabin air through the two-rate
driver, which none of these runners drive. Nothing in this slice says what B or C would do to a
station golden, and the census's 0.078 is not a stand-in for measuring it.

## 5. Predictions — written 2026-09-30, before any code

* **P1. The fine-step answer converges, first order.** On both jar cases the successive
  differences halve (ratio 1.6–2.5) from 1/16 day down. The 1/256 run never rations.
* **P2. The shipped step is visibly off on the jar, but not badly on yield.** Plain Euler at ¼
  day against 1/256: yield within 5 % every season; peak leaf area within 5 %; CO₂ error largest
  in the drawdown phase, tens of ppm at worst.
* **P3. A halves the error at twice the cost.** Every A error is 0.4–0.6 of Euler-¼'s. A does not
  ration on the leaf-form jar (the 1.15 draw becomes roughly 0.6).
* **P4. C is first order too, and errs the other way on CO₂.** Euler reads uptake at the full
  pool and over-draws; C reads it at the emptied pool and under-draws. So during the drawdown C's
  CO₂ sits **above** the fine-step answer where Euler's sits **below** it. C's CO₂ error at ¼ day
  is of the same size as Euler's (between 0.5× and 2×); it is **not** expected to be much closer.
  C never rations (by construction — not evidence).
* **P5. B's CO₂-only rule fixes most of the tight steps cheaply in steps, not in evaluations.**
  B(CO₂, θ = 0.25) gets within 1.5× of A's CO₂ error. It splits on fewer than a fifth of the jar's
  steps. But because its probe is a full extra evaluation on every step, its evaluation count is
  about 2× Euler's, the same as A's. That is a price of the frozen engine (the probe cannot be
  shared with the step), not of the option.
* **P6. B watching every store hits a floor on cold nights.** On a cold night the condenser takes
  the whole humidity excess in one step, so the vapour store's draw stays near 0.37 at any step
  size. B(all stores, θ = 0.25) therefore splits to its depth limit on every cold night, in every
  chamber run, and would move every chamber golden. Cost stays small (a split that bottoms out
  costs about one evaluation per level, not 2^depth). B(CO₂, θ = 0.5) does not see it.
* **P7. Frozen results that would move.** A: all six runs. B(CO₂, 0.5): the sealed jar only (52
  steps above 0.5 in the census). B(CO₂, 0.25): the jar and the two perennial-chamber runs (census
  worst 0.262), not the consumer chambers (0.230) and not the open field. C: every sealed run (the
  jar, both perennial, both consumer), not the open field.
* **P8. The leaf-form jar is further off than the frozen jar** at ¼ day, on every metric, and
  every option improves on plain Euler there by more than on the frozen jar.
* **P9. Every control in §3 holds on the first run.** If one fails, that is a finding, and no
  number is read until it is understood.

## 6. Results — measured 2026-09-30

`cargo run --release -q -p domains --example step_options` (about 7 minutes). Full output kept
outside the repo at `W:\temp\claude\step-options\run6.txt`. Tests:
`rust/crates/domains/tests/step_options.rs` (6, the cheap controls on one season).

### ⚠ One variant was added after the predictions

The first run showed C no better than plain Euler at ¼ day. The candidate cause was that CO₂
returned to the chamber during a step (soil and plant respiration) reaches the crop only on the
next step. So a second form of C was built **after** the predictions and results above were
written: **C+returns** (`Scope::WithInflows`), where the crop's whole carbon budget
(allocation, growth and maintenance respiration) reads one end-of-step pool
`X = C₀ + I + N(X)`, with `I` what every other flow returns over the same step. Its first run
tripped control 7: with returns counted, maintenance read at the start of the step disagreed
with allocation read at the end (the crop both grew and burned tissue for want of air in one
step). That is why the budget is solved as one. It carries no prediction, and it was built to
test a cause, not as a candidate.

### Controls

1. **The fine-step answer converges, but not cleanly at first order.** Plain Euler's
   difference to the next step size down (frozen jar): CO₂ 206 → 32 → 11 → 2.5 → 1.27 →
   0.56 ppm; final harvest 4.75 → 1.15 → 1.00 → 0.23 → 0.12 → 0.03 %. The ratios run 6.4, 2.9,
   4.4, 2.0, 2.3 (CO₂) and 4.1, 1.15, 4.4, 2.0, 4.2 (harvest). Over the last two halvings they are
   about 2 or better, and the 1/256 run never rations. So the fine-step answer's own error is
   about the last difference: **0.6 ppm and 0.03 % on the frozen jar, 2.7 ppm and 0.03 % on the
   leaf-form jar.** Peak leaf area stops improving at about 0.01 %, a floor, so its ratios mean nothing
   below that. On the leaf-form jar the CO₂ ratios over the last three halvings are 1.87, 1.46
   and 1.67, slower than halving. Any option error below about 3 ppm there is inside the answer's
   own uncertainty.
2. **B never splitting is the reference run**: bit-identical to Euler ¼ at all 3,661
   quarter-days of the jar and all 6,101 of the perennial chamber.
3. **B always splitting is the finer uniform run**: forced once, bit-identical to Euler ⅛; forced
   twice, to Euler 1/16, at every quarter-day of both runs, **across four re-sows**. The counter
   renumbering is proved. A test (`b_always_splitting_once_is_the_eighth_day_run_across_a_resow`)
   holds it, and a deliberate break of the renumbering turns it red.
4. **C on the open field is the reference run**, bit for bit, with 0 solves.
5. **C and Euler share one limit.** Max |C − Euler|: 34.2 ppm at ¼, 2.7 ppm at 1/64. C+returns:
   29.4, then 2.3.
6. **C's solve, every step**: the bracket brackets and the largest residual is 2.2e-16 mol (C)
   and 2.0e-12 mol (C+returns). ⚠ The first version of the rewritten solve refused a step with
   `h(hi) = −3e-17`: when the crop takes nothing, `hi` is itself the root and is zero only up to
   rounding. The bracket is now judged to the same tolerance as the residual.
7. **C's scope, every step.** On the frozen jar, no explicit process gives a different answer at
   the end-of-step pool, in either scope. **On the leaf-form jar, the leaf-area process
   (`biosphere.leaf_area_index`, an aux accumulator) does, on 270–342 steps**: it reads the
   carbon budget too, and neither scope reaches it. That is a named gap in C as built for the
   lab leaf form, not a failure of the check.

### ⚠ Yield: the FINAL harvest, not the season-1 end (corrected after the first write-up)

The jar is never re-sown, and at day 305 (season 1's end) its grain is **still filling**:
storage grows another 46 % before it stops (1/256 day: 0.924 → 1.346 → 1.346 mol C). The first
write-up and commit `c384fa1` scored "yield" as the largest error across the three season ends,
which is day 305, and headlined **−15 %**. That figure measures **when** the grain fills, not
how much it holds. The coarse step fills later. The **final harvest** is **−7.1 %**. The advisor
caught it (the pattern: peak leaf area within 0.5 % while "yield" was 15 % off). Both are
reported below, and the final harvest is the headline.

| Euler | day 305 | final (day 915) |
|---|---|---|
| ¼ (shipped) | 0.7848 (−15.1 %) | 1.2495 (**−7.1 %**) |
| ⅛ | 0.8877 (−3.9 %) | 1.3119 (−2.5 %) |
| 1/16 | 0.9037 (−2.2 %) | 1.3272 (−1.4 %) |
| 1/64 | 0.9219 (−0.2 %) | 1.3437 (−0.1 %) |
| 1/256 | 0.9241 | 1.3456 |

### The scorecard, frozen jar (3 seasons; errors against Euler 1/256)

| Option | max ΔCO₂ ppm | mean ΔCO₂ ppm | **final harvest** | day 305 | peak LAI | evaluations / day | rationed |
|---|---|---|---|---|---|---|---|
| Euler ¼ (shipped) | 241 | 42.6 | **−7.1 %** | −15.1 % | −0.5 % | 4 | 0 |
| **A**: Euler ⅛ | 39 | 14.1 | **−2.5 %** | −3.9 % | +0.9 % | 8 | 0 |
| Euler 1/16 | 15 | 7.5 | −1.4 % | −2.2 % | +0.1 % | 16 | 0 |
| B, CO₂ only, θ = 0.5 | 241 | 42.6 | −7.1 % | −15.1 % | −0.7 % | 8.2 | 0 |
| B, CO₂ only, θ = 0.25 | 193 | 24.3 | −3.5 % | −8.7 % | +0.2 % | 10.2 | 0 |
| B, CO₂ only, θ = 0.1 | 160 | 16.1 | −2.2 % | −4.4 % | +0.1 % | 15.7 | 0 |
| B, every store, θ = 0.25 | 193 | 24.2 | −3.4 % | −8.6 % | +0.2 % | 10.7 | 0 |
| **C** ¼ | 217 | 41.8 | **−7.5 %** | −15.3 % | −1.3 % | 4 + 20 allocation | 0 |
| C ⅛ | 29 | 13.3 | −2.7 % | −4.0 % | +0.2 % | 8 + 17 allocation | 0 |
| C+returns ¼ | 223 | 42.5 | **−7.4 %** | −15.2 % | −1.2 % | 4 + 40 budget | 0 |
| C+returns ⅛ | 29 | 13.4 | −2.7 % | −4.0 % | +0.2 % | 8 + 35 budget | 0 |
| Euler 1/64 | 1.6 | 0.8 | −0.1 % | −0.2 % | 0.0 % | 64 | 0 |

The chamber air ranges 9–2,131 ppm in the fine-step answer. Every harvest error is **negative**
(the coarse step grows less), and every mean CO₂ bias at ¼ day is **positive** (the coarse step
leaves more CO₂ in the air). B's evaluations include its probe, one full extra evaluation on
every step. B split 55, 439, 605 and 519 of 3,660 steps. The every-store rule bottomed out at
the depth limit 9 times.

**The leaf-form jar**, final harvest: Euler ¼ −7.2 % (day 305 −16.1 %), 304 ppm, peak leaf area
−4.2 %, 5 firings. A −2.6 % with none. B θ = 0.5 −6.4 %; θ = 0.25 −2.3 %; θ = 0.1 −1.9 %. C −7.5 %,
C+returns −7.4 %. None of B, C or C+returns ration. **Every option leaves peak leaf area at about
−4.2 % except A** (−0.2 %): B's splits do not reach it.

**The diagnostic: the open field, with no chamber pool**, is −2.1 % on its (one-season) final
harvest at ¼ day and +11.0 % on peak leaf area.

### Would a frozen result move? (frozen science, ¼ day)

| Run | B CO₂ 0.5 | B CO₂ 0.25 | B CO₂ 0.1 | B every 0.25 | C | C+returns |
|---|---|---|---|---|---|---|
| open field | 0 | 0 | 0 | 0 | no pool | no pool |
| sealed jar | 55 splits | 439 | 605 | 519 | moves | moves |
| perennial chamber, 5 and 15 seasons | 0 | 2 | 171 / 346 | 142 / 422 | moves | moves |
| consumer chamber, 5 and 15 seasons | 0 | 0 | 56 / 140 | 140 / 420 | moves | moves |

A moves all of them. A B run that split nothing is asserted bit-identical to the frozen run, and
one that split is asserted to differ. ⚠ **The station is not measured** (§4).

### Predictions, graded

* **P1 — HALF HELD.** It converges, and the 1/256 run never rations. But it is not cleanly first
  order: the successive ratios at coarse steps run 1.1–6.4, and the leaf-form jar's CO₂ ratios
  over the last three halvings are 1.5–1.9. The fine-step answer is usable because its own error
  (0.6–2.7 ppm, 0.03 % harvest) is far below the ¼-day errors being scored.
* **P2 — REFUTED.** The shipped step is **7.1 % low on the jar's final harvest** (15 % behind at
  day 305), not within 5 %. Its CO₂ is off by up to 241 ppm, not tens. Peak leaf area held
  (−0.5 %).
* **P3 — REFUTED in the good direction.** A did better than halve the error: final harvest −7.1 →
  −2.5 % (×0.35), max CO₂ 241 → 39 ppm (×0.16), for twice the evaluations. It removes the
  leaf-form rationing (held).
* **P4 — HALF HELD.** C at ¼ day is first order with an error the same size as Euler's (final
  harvest −7.5 % vs −7.1 %): held. The sign is refuted: C does not sit on the other side of the
  answer. Both leave the air **above** it on average (bias +15.8 and +9.6 ppm), and both grow less.
  C never rations (by construction, not evidence).
* **P5 — REFUTED on CO₂.** B (CO₂, θ = 0.25) reaches 193 ppm against A's 39, not within 1.5×. On
  final harvest it is within it (−3.5 % vs −2.5 %). It split 12 % of steps, so the "fewer than a
  fifth" half held. Its cost was 2.5× Euler's, against A's 2×. Uniform refinement at the same
  cost beats every B row: B θ = 0.1 costs 15.7 evaluations a day for −2.2 %, and 1/16 costs 16 for
  −1.4 %.
* **P6 — HELD.** The every-store rule bottoms out on cold nights (9 times in the jar). It splits
  every chamber run where the CO₂-only rule does not (perennial 142 vs 2, consumer 140 vs 0), at
  almost no extra cost (10.7 vs 10.2 evaluations a day).
* **P7 — HELD exactly.** B θ = 0.5 moves the jar only. B θ = 0.25 moves the jar and both
  perennial runs (2 splits each) and nothing else. C moves every sealed run and not the open
  field. A moves all.
* **P8 — HALF HELD.** The leaf-form jar is further off on every metric, barely so on harvest
  (−7.17 vs −7.14 %), clearly on leaf area (−4.2 vs −0.5 %) and CO₂ (304 vs 241 ppm). "Every
  option helps more there" is refuted: C helps neither.
* **P9 — HELD for the pre-registered controls.** Controls 2–7 held on their first run; control 1
  is P1. The two refusals this slice met were both in C code rewritten after the first run (the
  scope clash that shaped C+returns, and the rounding at the bracket's edge).

## 7. Findings

1. **The shipped quarter-day step leaves the sealed jar's final harvest 7.1 % short** of a
   5-minute step, and the lab leaf form 7.2 %. At day 305 the jar is 15 % behind, because the
   coarse step fills the grain later. The open field's one-season harvest is 2.1 % short. ⚠ Two
   corrections to my own first write-up:
   * it headlined the day-305 snapshot (−15 %) as the harvest (fixed above);
   * it said the step decision never compared against a converged answer. **That was false.**
     The step sweep (`docs/log/step-sweep.md` finding 2, 2026-08-14) measured sealed-chamber
     harvest at −0.7 % at `dt = 1` against an RK4-converged limit, before the within-day light
     path. Its table does not say which run or horizon its "harvest" is (0.72 mol, matching
     neither today's day-305 0.92 nor today's final 1.35). So **no trend is claimed** between
     that 0.7 % and today's 7.1 %: they may not be one observable.

   The light-path record (`docs/log/gross-net-gas-exchange.md` findings 4–6) found the open
   field's **canopy** moves 15 % between ¼ day and converged, and the user's call there was to
   finish at the shipped step. It measured the jar's day-night CO₂ swing (converged), not the
   jar's harvest. I found no record that measures the jar's harvest against a converged answer
   on today's tree.
2. **It is not the crop's uptake outrunning the air inside a step.** C removes exactly that
   error, and the final harvest moves the wrong way (−7.1 → −7.5 %). C+returns also lets the crop
   spend what the chamber gives back during the step, and gives −7.4 %. Both candidate causes are
   refuted by the experiment that removes them. **The cause is not identified.** What is measured:
   the loss lives in the steps where the crop draws hard (B at θ = 0.1, refining only those,
   recovers most of it). It is larger with a small pool than without one (7.1 % after three
   seasons vs 2.1 % after one; not one horizon). But refining a step refines every process and
   the light within it at once, so B does not say which. The candidate the record already holds
   is the light path's own: a quarter-day window mean of a curved light response, which the
   light-path record measured moving the open field's canopy by 15 %. It is **named, not
   measured**, for the jar. Its sign does not follow simply, either: that bias makes a coarse
   step fix *more* carbon per unit of light, and the jar's coarse step ends with *less* harvest.
3. **Option C is a safety property, not an accuracy fix.** It can never overdraw, so the backstop
   never fires on the crop's CO₂. On accuracy it matches the shipped step, slightly worse on final
   harvest (−7.5 vs −7.1 %). That is still the user's standing wish (*"make plants take less as
   the CO₂ runs low"*), and this slice says what it buys: no rationing, no better harvest. It
   would move every sealed golden (15–16 end values each), because it changes every step on
   which the crop draws.
4. **Option B, as buildable without touching `simcore`, is dominated by uniform refinement at
   the same cost.** Its probe cannot share the step's own evaluation, so every step pays twice.
   B θ = 0.25 costs 10.2 evaluations a day for −3.5 % (A: 8 for −2.5 %). B θ = 0.1 costs 15.7 for
   −2.2 % (1/16: 16 for −1.4 %). At θ = 0.5 it moves only the jar's golden and buys nothing there.
5. **Option A moves the error the furthest per unit cost**: final harvest −7.1 % → −2.5 % at ⅛
   (2× the evaluations), −1.4 % at 1/16 (4×), −0.1 % at 1/64 (16×). It is also the only option that
   fixes the leaf-form jar's peak leaf area (−4.2 % → −0.2 %). Its price is a step unfreeze: every
   biosphere golden, plus the station's plant step, which is unmeasured.
6. **C as built does not reach the lab leaf form's leaf-area process**, which reads the carbon
   budget as an aux accumulator. Any adoption of C alongside the leaf form has to reach it.

## 8. What this slice does not do

* It does not decide. Slice 4 (the decision) is the user's, on §7.
* It does not measure the station. B and C would need the two-rate driver. A would move the
  station's plant step, which is tied to the cabin's minutes by the interleaved day.
* It does not find the cause of the harvest loss. Finding 2 narrows it and names what was ruled
  out.
