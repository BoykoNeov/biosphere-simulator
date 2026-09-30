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

## 5. Results — measured 2026-09-30

`cargo run --release -q -p domains --example step_cause` (about 2 minutes). Output kept outside
the repo at `W:\temp\claude\step-cause\run3.txt`. Code: `rust/crates/domains/src/lab/step_cause.rs`,
tests `rust/crates/domains/tests/step_cause.rs` (5; two deliberate breaks — E2's counter
renumbering, E1's block — each turned exactly one red).

### Controls

1. **E1 with a step-sized block is the plain run**, bit for bit at every quarter-day, at ¼ and at
   1/256, on both cases.
2. **E2 at `k = 1` is the plain run**, bit for bit at every quarter-day, on both cases.
3. **The sub-windows partition the dose**: worst relative gap 7.7e-16 at `k = 64`.
4. **Nothing outside the wrapped flows reads PAR** (877 lit steps probed per case). ⚠ The plan
   said "exactly those three", and the jar refuted the wording on the first run: in a sealed
   chamber growth respiration's source and sink are the same pool, so it returns empty before
   reading the budget (the step-options slice already knew this). The check is now "a subset,
   containing the allocation". The instrument was right; the sentence was not.
5. **The known answer holds on the open field.** E1 (fine step, quarter-day light) gives peak
   LAI **+10.6 %** against the shipped step's +11.0 %: it keeps the light-path record's canopy
   bias almost whole while removing the step.
6. **An unplanned cross-check, and the strongest one.** The four water stores read no light.
   At the end of the run they are **identical to the answer in E1** (fine step) and **identical
   to the shipped run in E2** (quarter-day step), **bit for bit** on both cases (first seen to
   six printed decimals; the bit check was added to the runner after an advisor challenge that
   six decimals is not "every digit"). Each instrument moves exactly the half it claims to.

### The four cells — final harvest against the 1/256-day answer

| | light in ¼-day windows | light in 1/256-day windows |
|---|---|---|
| **¼-day step** | **−7.143 %** (shipped) | **−0.023 %** (E2, `k = 64`) |
| **1/256-day step** | **−7.185 %** (E1) | 0 (the answer) |

Interaction `shipped − E1 − E2` = **+0.065 points**. On the open field: shipped −2.12 %, E1
−1.53 %, E2 −0.87 %, interaction +0.28 — a split there, see below.

**The jar's harvest loss is the light's time resolution, all of it.** Refining the step with the
light held in quarter-day blocks recovers nothing (−7.19 %). Leaving the step at a quarter day
and resolving the light inside it recovers everything (−0.02 %). The light sweep reproduces the
step sweep's whole harvest curve: E1 at light ⅛, 1/16, 1/64 gives −2.55, −1.39, −0.146 %;
plain Euler at those **steps** gave −2.51, −1.37, −0.143 % (step options §6) — within 0.04
points.

### The sweeps

Sealed jar, 3 seasons (secs = one run's wall time, a single measurement):

| cell | harvest % | day 305 % | peak LAI % | max ΔCO₂ ppm | mean ΔCO₂ ppm | budget evals / step | secs |
|---|---|---|---|---|---|---|---|
| shipped (step ¼, light ¼) | −7.143 | −15.08 | −0.47 | 240.7 | 42.62 | 3 | 0.21 |
| E1 step 1/256, light ¼ | −7.185 | −15.02 | −1.24 | 259.8 | 42.47 | 3 | 12.9 |
| E1 step 1/256, light ⅛ | −2.551 | −3.91 | +0.79 | 57.0 | 13.88 | 3 | 12.5 |
| E1 step 1/256, light 1/16 | −1.390 | −2.21 | +0.23 | 17.2 | 7.38 | 3 | 12.2 |
| E1 step 1/256, light 1/64 | −0.146 | −0.24 | +0.01 | 1.6 | 0.76 | 3 | 12.0 |
| E2 step ¼, light 1/16 (`k = 4`) | −1.405 | −2.27 | −0.12 | 23.3 | 8.55 | 12 | 0.23 |
| E2 step ¼, light 1/64 (`k = 16`) | −0.169 | −0.31 | −0.51 | 30.1 | 2.45 | 48 | 0.35 |
| E2 step ¼, light 1/256 (`k = 64`) | −0.023 | −0.07 | −0.53 | 31.1 | 1.74 | 192 | 0.83 |
| A: step 1/16 | −1.371 | −2.21 | +0.11 | 15.0 | 7.53 | 3 | 0.74 |

* **What E2 leaves behind is the within-step CO₂ swing**: its mean CO₂ error falls to 1.7 ppm
  but its largest stays at about 31 ppm, the size of the gap between C and plain Euler at a
  quarter day (34 ppm, step options control 5). That part is the stepping of the state, and it
  is not what costs the harvest.
* **Where the missing grain sits.** At the end, the shipped jar holds 0.096 mol C less grain than
  the answer, and **0.072 mol C more CO₂ in the air** (O₂ exactly the mirror), plus 0.017 in
  humus and about 0.01 in litter and microbes. E1 is the same to within 0.001. So the crop under
  quarter-day light **cannot pull the air down as far**, and the carbon it does not fix stays in
  the chamber. It is not carbon parked in the canopy (vegetative carbon at peak LAI is within
  ±0.5 % in every jar cell).

Open field, 1 season:

| cell | harvest % | peak LAI % |
|---|---|---|
| shipped | −2.12 | +11.02 |
| E1 light ¼ / ⅛ / 1/16 / 1/64 | −1.53 / −0.43 / +0.22 / +0.004 | +10.62 / +3.75 / +0.84 / +0.07 |
| E2 `k = 4` / 16 / 64 | −0.64 / −0.87 / −0.87 | −1.03 / −1.86 / −1.92 |
| A: step 1/16 | −0.01 | +0.29 |

The open field **splits**: the light carries −1.5 points of its harvest error and the whole
canopy bias (+10.6 of +11.0 %), and the stepping of the state carries about −0.9 points of
harvest and −1.9 % of canopy, which E2 exposes once the light no longer masks it. The open
field's shipped canopy is **bigger** than the answer (+0.77 mol C more leaf, +1.56 stem, +1.20
root at the end) and its grain smaller — the concave light response over-feeding the canopy,
as the light-path record found.

### Predictions, graded

* **Q1 — HELD.** E1 −7.19 % (predicted −8 to −5).
* **Q2 — HELD.** E2 at `k = 64` −0.02 % (predicted within ±2).
* **Q3 — HELD.** Interaction +0.07 points on the jar (+0.28 on the open field; predicted < 2).
* **Q4 — HELD.** Open-field E1 peak LAI +10.6 % (predicted ≥ +8); E2 −1.9 % (predicted ±3).
* **Q5 — REFUTED.** Vegetative carbon at the jar's peak LAI is +0.5 % (shipped) and −0.1 % (E1):
  the jar's loss is not carbon locked into the canopy first. It is carbon **left in the air**.
  The mechanism I wrote the prediction around is the open field's, not the jar's.
* **Q6 — REFUTED as stated.** E2 at `k = 64` took 0.83 s against A at 1/16's 0.74 s, not under a
  third. But it is 60 times more accurate on the jar's harvest at that cost, and `k = 16`
  (0.35 s, 1.7× the shipped run) reaches −0.17 %, eight times closer than A at 1/16 on the jar's
  harvest for under half its time. Wall times are single runs. ⚠ On the open field's harvest and
  canopy, and on the jar's worst CO₂ error, A at 1/16 is the better of the two (finding 3).
* **Q7 — HALF HELD.** Control 4's wording failed on the first run (above); the instrument did
  not, and every other control held first time.

## 6. Findings

1. **The jar's 7 % harvest loss is the light's time resolution, not the step's stepping of the
   state.** The four cells add up to within 0.07 points, the E1 light sweep reproduces the step
   sweep's harvest curve to within 0.04 points, and the water stores prove the two instruments
   separate what they claim. **The earlier record's "cause not identified" is discharged.**
2. **The sign puzzle narrows, but is not closed.** The concave light response makes a
   quarter-day window fix **more** carbon (the open field's canopy). The jar, living near the
   CO₂ compensation point, ends with more CO₂ in its air under the same windows: it fixes
   **less**. The reading that fits is the growth kink, `available = max(0, GASS − MRES)`: near
   compensation, a quarter-day mean light can put a whole window on the wrong side of it, where
   finely resolved light keeps the bright hours productive. **Named, not measured.** A top-hat
   light (the light-path record's own control) is the experiment that would separate them.
3. **This changes what the step decision has to fix.** On the **jar's harvest**, a finer step
   (option A) helps only because it also resolves the light. Resolving the light inside the
   budget flows (E2's shape) reaches the jar's harvest at a quarter-day step for 1.7–4× the
   shipped cost, where A at 1/16 costs 4× and stays 1.4 % short. ⚠ **Scoped to the jar's harvest.**
   Neither shape dominates: on the open field the 1/16 step is the better one (harvest −0.01 %
   against E2's −0.87 %, peak LAI +0.3 % against −1.9 %), and on the jar's worst CO₂ error too (15
   ppm against 31), because there the stepping of the state carries real error that only a finer
   step removes.
   E2's shape is **not a candidate as built**: it is a
   lab instrument, and adopting anything like it is a change to the frozen science's form (how
   the budget flows take the day's light), so it would move every plant-bearing golden, with the
   usual ceremony. It leaves the within-step CO₂ swing (31 ppm) that C addresses. The station
   is not measured.
4. **The open field splits** (light −1.5, stepping −0.9 points on harvest). There, both halves
   matter, and E2 alone leaves −0.9 %.

## 7. What this slice does not do

* It does not decide. The step decision (Step 2, slice 4) goes back to the user with a new
  option on the table.
* It does not measure the leaf-form jar (its leaf-area aux reads the budget, and E2 does not
  reach an aux) or the station.
* It does not separate the light's curvature from the growth kink (finding 2).
