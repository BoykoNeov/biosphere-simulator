# Post-roadmap — why the quarter-day step loses the jar's harvest (Step 2, slice 3b)

**Opened 2026-09-30**, on the user's *"Go with your recommendation"*. The recommendation, given
with the step-options results (`docs/plans/post-roadmap-step-options.md`, record
`docs/log/step-options.md`), was: **find the cause first, then adopt a finer step.** This slice is
the first half only. Record: `docs/log/step-cause.md`.

**Scope: lab only.** No `simcore` edit, no frozen flow edited, no golden regenerated. **It stops
before any unfreeze**, even though the go-ahead named one: the whole point of measuring the cause
first was that it could change what the step change has to fix, so the result goes back to the
user before anything frozen moves.

## 1. The question

At the shipped quarter-day step the sealed jar's final harvest is **7.1 % short** of a 1/256-day
answer (open field 2.1 %). The step-options slice refuted two causes by removing them (the crop's
uptake outrunning the air inside a step; same-step returns). It named one candidate and did not
measure it: **the light.** The only forcing that varies within a day is PAR (temperature, day
length, net radiation and VPD are daily tables, `biosphere/system.rs::weather_forcings`), and it
reaches a step as the **mean over the step's window** of a half-sine day. The crop's light
response is curved and has a kink where assimilation meets maintenance
(`available = max(0, GASS − MRES)`) plus a night branch that burns tissue, so the mean of the
response over a quarter day is not the response to the mean light. Refining the step refines the
light and the state at once. This slice separates them.

## 2. The experiment: four cells

|                    | light in ¼-day windows | light in 1/256-day windows |
|--------------------|------------------------|----------------------------|
| **¼-day step**     | shipped (−7.1 %)       | **E2**                     |
| **1/256-day step** | **E1**                 | the answer (0)             |

* **E1 — fine step, blocked light.** The 1/256-day Euler run with PAR held at the quarter-day
  window mean for all 64 sub-steps of each quarter day. Built as a resolver wrapper:
  `par(n, dt) = base(⌊n·dt/Δ⌋, Δ)`, `Δ = ¼`. Every other forcing is a daily table and is
  already identical across the sub-steps. ⚠ This rebuilds the resolver from the public
  `weather_forcings`/`weather_shared` and so deliberately goes round `season_setup_composed`'s
  "the weather is not part of the seam" guard. The guard exists so a **mechanism** A/B cannot
  move the forcing; here the forcing's time resolution *is* the variable.
* **E2 — quarter-day step, light integrated inside it.** A lab flow wrapper that evaluates its
  inner flow at `k` sub-window PARs (the base schedule at the renumbered counter `n·k + j` with
  step `dt/k`) on the **same** start-of-step state and averages the legs stock by stock. The
  average is linear, so each flow stays balanced. It wraps **all three flows that read the carbon
  budget** — allocation, growth respiration, maintenance respiration — so no two of them see
  different light (the scope trap C+returns hit). `k = 64` matches 1/256; `k = 4, 16` if cheap.
  The leaf-form jar is out of scope: its leaf-area aux process reads the budget too, and an aux
  cannot be wrapped the same way.

The **interaction** is `shipped − (E1 + E2 − 0)` measured on the same scale: whatever the two
halves do not add up to.

Cases: the **sealed jar** (frozen, 3 seasons, no re-sow — its golden's drive) and the **open
field** (1 season), scored against Euler 1/256 at every quarter-day boundary: final harvest, peak
LAI, max and mean CO₂ error (jar), evaluations per day, wall time. Plus a diagnostic the last
slice's findings suggest: **vegetative carbon (leaf + stem + root) at the jar's peak LAI**, to see
whether a cell that loses harvest locked more carbon into the canopy first.

## 3. Controls — each must hold before a number is read

1. **E1 with `Δ = dt` is the plain run, bit for bit**, at ¼ and at 1/256 — the rebuilt resolver
   is the stock one.
2. **E2 with `k = 1` is the shipped run, bit for bit** (leg order, no `−0.0` creeping in).
3. **The sub-window doses partition the window's dose**: `Σⱼ par(nk + j, dt/k)·dt/k` equals
   `par(n, dt)·dt` to 1e-12 relative, on every lit step of a season.
4. **Nothing outside the three wrapped flows reads PAR** — a probing environment records every
   `par` read by every flow and aux process over a season; the set must be exactly those three.
5. **The known answer on the open field.** The light-path record measured the canopy's step
   sensitivity as the curved light response (its finding 4). The open field's peak LAI is +11 %
   at ¼ against 1/256 today. **E1 must reproduce most of that on the open field** (it keeps the
   curvature bias and removes the step), or the instrument is wrong and the jar's answer is not
   read.

## 4. Predictions — written 2026-09-30, before any code

The sign is not obvious, which is why it is measured. The curved response says a quarter-day
window fixes **more** carbon per unit of light (the open field's canopy +11 %). The jar ends with
**less** harvest. What connects them, if the light is the cause: the jar is carbon-limited (it
holds about 2 days of carbon), so carbon that the coarse window pushes into leaves and stems early
is carbon that is not in the air, or in the grain, later. The open field already shows the
pattern in miniature: +11 % peak LAI, −2.1 % harvest.

* **Q1. E1 carries most of the jar's loss.** E1's final harvest error is between −8 % and −5 %
  (the shipped step's is −7.1 %).
* **Q2. E2 recovers most of it.** E2 at `k = 64` is within ±2 % on final harvest.
* **Q3. The halves add up.** `|shipped − E1 − E2| < 2` percentage points on final harvest.
* **Q4. The open field agrees.** E1's open-field peak LAI error is at least +8 % (of today's
  +11 %); E2's is within ±3 %.
* **Q5. The mechanism.** In the cells that lose harvest (shipped, E1), vegetative carbon at the
  jar's peak LAI is **higher** than the answer's.
* **Q6. The price.** E2 at `k = 64` costs about 3 × 64 extra flow evaluations per step on three of
  the flow set's flows; its wall time is under a third of A at 1/16 (which reaches −1.4 %).
* **Q7. Every control in §3 holds on the first run.**

If Q1–Q2 are both refuted the other way (E1 ≈ 0, E2 ≈ −7 %), the loss is in the stepping of the
state and not the light, and the next suspect has to be named from the stepping (C has already
excluded the crop's CO₂ draw). A result that splits the loss is reported as a split.

## 5. Results

*(Not yet run.)*
