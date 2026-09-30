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

## 6. Results

*(to be filled after the run)*
