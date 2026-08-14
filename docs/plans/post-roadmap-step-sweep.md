# Step 0, axis 1 — the step sweep (2026-08-14)

**Status: MEASURED 2026-08-14, NOTHING BUILT. The step decision is the user's and is
still open.**

**Charge.** Axis 1 of Step 0 in the consolidated *direction plan* (the forward-looking
plan doc named in this log's own header — deliberately not in its index, and deliberately
not named here by filename, because a plan doc naming it would put it on the wrong side of
the index↔record parity check). Axis 2 — the CO₂ setpoint controller — ran 2026-08-13 and
came back negative, so the gate is no longer *"whether"* but *"which step"*. This axis
measures the price and the payoff of each candidate so that the decision is arithmetic
rather than a judgement.

**Method.** One probe harness, `src/` untouched, no golden regenerated, no schema moved —
the same shape as the last two items.
`M:/claud_projects/temp/step-sweep/{sweep_biosphere,sweep_co2,sweep_station}.py`, with the
raw output in `{frozen,leaf,co2,station}.txt`, the full write-up in `RESULTS.md` and the
wall-clock denominator in `suite-baseline.txt`. The parked leaf branch was measured from a
**git worktree**, so `main` was never checked out of; the worktree was removed on
completion and `git worktree list` shows one entry.

**Harness validation — every inherited anchor reproduced exactly before any new row was
trusted.** Sealed RK4 margin 1.3747 (record 1.375) → `k·h` 0.727; perennial 1.3909 → 0.719;
open field 9.1352; peak leaf carbon Euler 0.9215 / RK4 0.8867 / perennial 0.8446; season-low
CO₂ 57.89 ppm Euler and 76.41 RK4 (record 57.9 / 76.4); the leaf branch's RK4 `BREAK` at
margin 0.9481 → `k·h` 1.055 and perennial 0.9668 → 1.034; the ×1→×5 Euler collapse
57.89 → 12.13 ppm (record 57.9 → 12.1); ×4 Euler margin 1.044. Independent cross-check
against a source the harness had never seen: the ECLSS margin reads **16.667 = 1/0.06**,
exactly the `k·dt = 0.06` design point that the allocation-headroom diagnosis's finding 7
cites.

**Observable definitions are inherited, not re-derived** (from
`M:/claud_projects/temp/allocation-headroom/RESULTS.md`): `margin` is the minimum over
(step, RK4 stage, clamped stock) of `amount / demand`, taken **unclamped** by monkeypatching
`simcore.arbitration._scale_factors`; `k·h = 1 / margin`; `rationed` is the Euler backstop's
firing count; `lowCa` is the season-low chamber CO₂ in ppm, against the 61.07 ppm
compensation point (`Γ*/ci_ratio = 42.75/0.7`), and is reported **only for sealed
scenarios** — unsealed, `biosphere.carbon_pool` is an unclamped boundary reservoir whose
minimum is not a concentration anything is limited by.

---

## ⚠ Three scope facts that change what "sweep every scenario" means

### 1. The axis is a REFINEMENT FACTOR, not an absolute `dt`

Every forcing in this tree is a pure function of the integer step count `n`, never of
`n·dt`. Audited at all five sites: `season.py:263` (the weather table, `values[min(n, last)]`),
`power/system.py:153` (the half-sine, `(n % steps_per_day)/steps_per_day`), `environment.py:62`
(a constant), and the two `perturbations.py` window bounds. Power ships at `dt = 3600 s`,
24 steps/day; the biosphere ships at `dt = 1 day`. So "`dt = ½`" is only meaningful against
each scenario's *own* shipped step, and holding the forcing fixed under refinement takes a
different rule per site:

* **weather table** — day-tiling (repeat each weather day `sub` times, `dt = 1/sub`, and
  scale the annual reset period `year` with it). Inherited from `probe7.py`.
* **half-sine** — `steps_per_day ×= sub` alongside `dt /= sub`, preserving the identity
  `dt · steps_per_day == 86400`.
* **constant** — nothing; it is `dt`-invariant.

The forcing total per physical day is unchanged in every case.

### 2. ⚠⚠ Four station goldens cannot take a finer biosphere step without a code change

`greenhouse`, `lighting`, `harvest` and `sealed_station` run through
`station.driver.run_master_day`, which takes **exactly one slow (biosphere) `step_report`
per master day** and hard-requires `fast_dt · steps_per_day == 86400`
(`src/station/driver.py:49-96`). Their biosphere step is pinned *by the driver*, not by the
scenario.

**The plan does not price this.** The step unfreeze is therefore not a
contract-and-goldens ceremony alone: carrying a finer biosphere step through the master-day
seam needs a real change to `src/station/driver.py`, which is engine code. Left as-is, the
same domain would integrate at `dt = ½` standalone and at `dt = 1` inside every station
assembly — a split the station freeze *delegates* to the biosphere freeze, and which would
no longer be honest. Not measured here, because a probe may not change `src/`; reported as
a ceremony cost.

### 3. The margin measure is blind to one scenario, by construction

`water_biting` reads margin **exactly 1.0000 at every refinement, under both integrators**
— which is not a knife edge, it is a tell. `soil_layers.py:134` reads
`flux = demand if demand < available else available`: the root-deepening flow clamps
*itself* to the stock (a cited form — [F] Eqn 14.10's `min`), so its demand can never exceed
supply and the ratio is pinned at 1. Same shape as the export-fidelity finding: **a flow
that pre-clamps is invisible to the backstop's arithmetic.**

---

## 1. The science answer: `dt = ½` clears the compensation point everywhere tested

Season-low chamber CO₂ (ppm) against the **61.07 ppm** compensation point. `X` marks a run
below it — i.e. the crop is fixing carbon where the model says it fixes none.

| scenario | Euler `dt=1` | `dt=½` | `dt=¼` | `dt=⅛` | RK4 converged |
|---|---|---|---|---|---|
| sealed_chamber | **57.89 X** | 75.06 ✓ | 75.82 ✓ | 76.03 ✓ | 76.28 |
| perennial_chamber | **56.03 X** | 74.91 ✓ | 75.48 ✓ | 75.65 ✓ | 75.75 |
| consumer_chamber | 73.29 ✓ | 74.36 ✓ | 74.42 ✓ | 74.54 ✓ | 74.64 |

**One halving moves the crossing from 5 % below the floor to 23 % above it**, and every
further refinement moves it about 1 ppm. This is the *converging* observable — a statement
about the answer, not about the guard's arithmetic.

### It also holds at every enrichment level — which `dt = 1` does not

Season-low CO₂, sealed chamber, across the CO₂ ×1…×5 sweep:

| enrichment | Euler `dt=1` | Euler `dt=½` | Euler `dt=¼` | RK4 |
|---|---|---|---|---|
| ×1 (357 ppm) | **57.89 X** | 75.06 ✓ | 75.82 ✓ | 76.41 |
| ×2 (714 ppm) | **33.83 X** | 74.74 ✓ | 75.50 ✓ | 76.33 |
| ×3 (1071 ppm) | **25.18 X** | 74.54 ✓ | 75.05 ✓ | 76.33 |
| ×4 (1428 ppm) | **19.07 X** | 74.39 ✓ | 74.86 ✓ | 76.37 |
| ×5 (1785 ppm) | **12.13 X** | 74.31 ✓ | 74.76 ✓ | 76.47 |

⚠ **New, and not in the record:** at ×5 the shipped integrator's margin is **0.975 — already
below 1**, so enrichment past ×4 rations on the step that ships today. At `dt = ½` the same
run sits at 2.65.

## 2. The cost side: how much the answer actually moves

Sealed chamber, Euler, against the converged (`dt = ⅛` RK4) value:

| observable | `dt=1` | `dt=½` | `dt=¼` | converged | error at `dt=1` | at `dt=½` |
|---|---|---|---|---|---|---|
| season-low CO₂ | 57.89 | 75.06 | 75.82 | 76.29 | **−24 %** | −1.6 % |
| peak leaf carbon | 0.9215 | 0.8995 | 0.8923 | 0.8860 | **+4.0 %** | +1.5 % |
| harvest (storage C) | 0.7189 | 0.7219 | 0.7234 | 0.7240 | **−0.7 %** | −0.3 % |

⚠ **Both numbers belong in the decision.** The headline output — harvest — moves **0.7 %**.
The tail statistic that motivated the whole gate moves **24 %**. Quoting either one alone
misleads, in opposite directions.

## 3. The parked leaf mechanism: `dt = ½` clears it, and `dt = 1` is worse than recorded

Measured on the `leaf-expansion-blocked` branch, from a worktree:

| scenario | `dt=1` Euler | `dt=1` RK4 | `dt=½` | `dt=¼` |
|---|---|---|---|---|
| sealed | `k·h` 0.746, **lowCa 39.73 X** | **BREAK**, `k·h` 1.055 | `k·h` 0.483, lowCa 72.34 ✓ | 0.214 |
| perennial | `k·h` 0.863, **lowCa 28.02 X** | **BREAK**, `k·h` 1.034 | `k·h` 0.471, lowCa 71.75 ✓ | 0.207 |
| consumer | `k·h` 0.526, lowCa 70.70 ✓ | clean, 0.577 | 0.282 | 0.131 |

⚠ **A finding the plan does not have.** The leaf mechanism's problem at `dt = 1` was recorded
as *silent rationing under Euler*. It is also a **much deeper compensation-point crossing**:
28.0 ppm against a 61.07 floor is a **2.2× crossing**, where the frozen tree crosses by 5 %.
The two blockers are the same defect and genuinely ride the same unfreeze — but **the leaf
branch is a stronger argument for moving the step than its own record shows.**

`dt = ½` clears it at `k·h` 0.471–0.483, i.e. **2.1× of headroom** to the bound, which
matches the direction plan's "clears by less than 2×" framing. `dt = ¼` leaves 4.8×.

## 4. The price half: the station/physics goldens are nowhere near the wall

Nothing rations at any refinement anywhere; margins run 10× to 1550×. `drift` is the largest
relative move of any final stock against that scenario's own shipped-step Euler run.

| scenario | margin `dt=1` | drift at `dt=½` | what the drift is |
|---|---|---|---|
| crew | 388.6 | 2.9e-14 | round-off |
| eclss | 16.67 | 2.9e-13 | round-off |
| cabin_gas | 16.67 | 2.9e-13 | round-off |
| water_recovery | 16.67 | 3.9e-13 | round-off |
| thermal | 257.7 | 4.0e-06 | truncation |
| demo_coupled | 10.00 | 1.3e-02 | truncation (a toy fixture) |
| power | 11.30 | **4.3e-03** | ⚠ **not truncation — see below** |
| power_self_discharge | 11.09 | **4.3e-03** | ⚠ same |
| station_heat_closure | 11.30 | **4.3e-03** | ⚠ same |

⚠ **The power family's drift is forcing quadrature, not integration error.** The half-sine
solar schedule is a Riemann sum over `steps_per_day` samples, so refining the step changes
the energy actually delivered: **2.7345e7 → 2.7463e7 → 2.7492e7 → 2.7502e7 J/day** at
24 / 48 / 96 / converged samples. The shipped 24-sample sum **under-delivers solar energy by
0.21 %**. Refining power's step is therefore a change to *what the scenario physically is*,
not an accuracy improvement — a distinction worth having before anyone reads the drift
column as an argument for moving it.

**But none of it is forced.** Power, thermal, ECLSS, crew, cabin and water-recovery are
separate scenarios with their own steps; a biosphere step change leaves them untouched.
Their goldens move only if we *choose* to move them. **Recommendation: do not.**

## 5. ⚠⚠ FLAGGED — `water_biting` converges to two different answers under the two integrators

Not caused by this work, not fixed by refinement, and it sits on a shipped golden:

| | `dt=1` | `dt=½` | `dt=¼` | `dt=⅛` |
|---|---|---|---|---|
| Euler peak leaf C | 0.7087 | 0.6979 | 0.6976 | 0.7046 |
| **RK4** peak leaf C | **0.0662** | **0.0655** | **0.0651** | **0.0649** |
| Euler harvest | 0.7309 | 0.7274 | 0.7267 | 0.7250 |
| **RK4** harvest | **0.0054** | **0.0052** | **0.0053** | **0.0173** |

Both are **stable** under an 8× refinement and they disagree by ~10× on leaf carbon and ~50×
on harvest. Under RK4 the chamber CO₂ never leaves ambient (357.00 ppm at every step) — the
crop is effectively dead from the start. Refinement does not close the gap, so this is **not
truncation error**, and it is not something a finer step fixes.

**Prime suspect — a hypothesis, not a settled diagnosis:** the self-clamping `min` at
`soil_layers.py:134`. A clamp *inside* a flow is evaluated independently at each of RK4's
four stage states, and the composition of four clamped stages is not the clamp of the
composition. This is the `a-clamp-hides-a-wrong-amount` shape, and §3 above says the margin
measure cannot see it.

**Decision-relevant consequence: this is a new argument against every RK4 row on the menu.**
Choosing RK4 would change `water_biting`'s golden *qualitatively* — the crop dies. Euler at a
finer step does not touch it. It deserves its own work item either way, and has not been
opened as one.

## 6. Wall clock — a measured baseline and a bounded projection

Raw simulation time scales as the step count, measured: **2.00× at `dt = ½`, 4.00× at
`dt = ¼`** (the biosphere Euler golden runs total ~2.5 s → ~5.1 s → ~10.2 s in-probe).

**Today's full-suite baseline** (`uv run pytest -n 12 -q`, tree at `e4f50d1`, clean):
**2330 passed, 5 skipped in 371.73 s = 6 m 12 s**, exit 0. This supersedes the 7 m 05 s in
`docs/test-suite-runtime.md` (measured 2026-08-10 at ~160 fewer tests) — the suite is now
**faster** than the documented figure despite having grown. That doc's headline figure is
therefore stale; it is left alone here because updating it is not this item's business, but
the number above is the current one.

⚠ **Probe seconds are not suite seconds, and a subset run is not a share.** The 7 biosphere
golden test files alone run in 6.35 s at `-n 12`; the **71** test files that touch the
biosphere (1493 tests) run in **9 m 57 s** — *longer than the whole 6 m 12 s suite*, because
running a subset defeats the conftest's xdist grouping. That figure is **discarded**, and is
recorded here only so nobody re-derives it and quotes it as the biosphere's share.

**Honest projection:** biosphere simulation work doubles, but under `-n 12` the wall clock is
set by the longest worker rather than by total work, so the suite at `dt = ½` is bounded
below by **6 m 12 s** and above by **~12 m 24 s**. That is a bound, not a prediction.
Narrowing it means actually halving the step and running the suite, which is ceremony work.

---

## The three questions Step 0 was posed to answer

1. **Is `dt = ½` enough on all 25 scenarios, or only on the three that were probed?**
   **Enough on every scenario this method can measure** — 9 biosphere scenarios (including
   both 15-year horizons), all 5 CO₂ enrichment levels, the parked leaf branch, and 9
   station/physics scenarios. **Four station goldens are excluded for a structural reason
   (§2), not silently.** `water_biting`'s margin is uninformative by construction (§3), but
   its Euler trajectory is stable across an 8× refinement.
2. **What does the suite runtime become?** Simulation work 2×. The suite today is
   **6 m 12 s**; at `dt = ½` it lands between that and ~12 m 24 s. Bounded, not pinned — §6
   says why, and why the 9 m 57 s subset figure was discarded.
3. **Does a controlled chamber clear both criteria at `dt = 1`?** Answered 2026-08-13 by
   axis 2: **no**, and the controller is worse than the instability it was meant to fix.

## Recommendation — ⚠ THE DECISION IS THE USER'S, AND IS NOT TAKEN HERE

**Euler at `dt = ½`, biosphere only, leaving the power/thermal/ECLSS/crew steps alone.** It
is the cheapest option that is numerically clean and scientifically correct on everything
measured, it moves the answer *toward* the converged limit rather than quieting a guard, and
it touches neither the integrator contract nor the arbitration backstop's Euler-only scope.

Two things the measurement adds to that recommendation, both of which cut against taking it
lightly:

* **`dt = ½` vs `dt = ¼` is still a ceremony-count question, and `½` leaves only 2.1× of
  headroom on an emergent bound that no load-time check can guard** (the chamber carbon
  pool's `k` is emergent — the allocation-headroom diagnosis's own point). `¼` leaves 4.8×.
* **The ceremony is bigger than the plan prices.** It includes a `src/station/driver.py`
  change for the master-day seam (§2) — engine code, not contract text — on top of the three
  freeze contracts the plan already counts.

## Successors named, none taken

* **The `water_biting` Euler/RK4 divergence** (§5) — a shipped golden whose two integrators
  disagree ~50× on harvest, with a named prime suspect and a measure that is blind to it.
  Not opened.
* **`docs/test-suite-runtime.md`'s headline figure is stale** (§6) — 7 m 05 s against a
  measured 6 m 12 s. A tooling fix, not science.
* **The master-day seam** (§2) — whether `run_master_day` should carry a slow-step
  refinement at all is a design question that outlives this decision.
