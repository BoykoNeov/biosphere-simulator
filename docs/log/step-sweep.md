## **The step sweep** (the direction plan's Step 0, axis 1 — the last input to the step decision)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per
> work item. The fuller record, with every table, is
> [`../plans/post-roadmap-step-sweep.md`](../plans/post-roadmap-step-sweep.md).

**MEASURED 2026-08-14, NOTHING BUILT; the step decision is the user's and is still open.**
Axis 1 of Step 0 in the **direction plan** (the forward-looking plan doc named in this log's
own header — deliberately not in its index and deliberately not named by filename here,
because a record file naming it would make the index↔record plan-doc parity red). Axis 2 came
back negative on 2026-08-13, so the gate is no longer *whether* but *which step*. Probe only,
on frozen `main`: `git diff src/` empty, no golden regenerated, no schema touched. Harness
`M:/claud_projects/temp/step-sweep/{sweep_biosphere,sweep_co2,sweep_station}.py`; the parked
leaf branch was measured from a **git worktree**, so `main` was never checked out of. **Every
inherited anchor reproduced exactly before any new row was trusted** — sealed RK4 margin
1.3747 → `k·h` 0.727, perennial 1.3909 → 0.719, open field 9.1352, peak leaf 0.9215/0.8867/
0.8446, season-low CO₂ 57.89/76.41 ppm, the leaf branch's two `BREAK`s at `k·h` 1.055/1.034,
the ×1→×5 collapse 57.89 → 12.13 ppm — plus an independent cross-check the harness had never
seen: the ECLSS margin reads **16.667 = 1/0.06**, exactly the `k·dt = 0.06` design point that
the allocation-headroom diagnosis's finding 7 cites.

**SCOPE FINDING 1 — the axis is a REFINEMENT FACTOR, not an absolute `dt`.** Every forcing in
this tree is a pure function of the integer step `n`, never of `n·dt`; audited at all five
sites (`season.py:263` weather table, `power/system.py:153` half-sine, `environment.py:62`
constant, the two `perturbations.py` window bounds). Power ships at `dt = 3600 s`/24 steps a
day, the biosphere at `dt = 1 day`, so "`dt = ½`" only means anything against each scenario's
own step — and holding the forcing fixed takes a **different rule per site**: day-tiling for
the weather table, `steps_per_day ×= sub` for the half-sine (preserving
`dt · steps_per_day == 86400`), nothing for a constant.

**SCOPE FINDING 2 — ⚠⚠ FOUR STATION GOLDENS CANNOT TAKE A FINER BIOSPHERE STEP WITHOUT A CODE
CHANGE, AND THE PLAN DOES NOT PRICE IT.** `greenhouse`, `lighting`, `harvest` and
`sealed_station` run through `station.driver.run_master_day`, which takes **exactly one slow
biosphere step per master day** and hard-requires `fast_dt · steps_per_day == 86400`. So the
unfreeze is not a contract-and-goldens ceremony alone: carrying a finer step through that seam
needs a real change to `src/station/driver.py`, which is **engine code**. Left as-is, the same
domain would integrate at `dt = ½` standalone and `dt = 1` inside every station assembly — a
split the station freeze *delegates* to the biosphere freeze, and which would no longer be
honest. Reported as a ceremony cost, not measured (a probe may not change `src/`).

**SCOPE FINDING 3 — the margin measure is blind to one scenario, by construction.**
`water_biting` reads margin **exactly 1.0000 at every refinement under both integrators**,
which is a tell, not a knife edge: `soil_layers.py:134` is
`flux = demand if demand < available else available`, so the root-deepening flow clamps itself
to the stock (a cited form, [F] Eqn 14.10's `min`) and its ratio is pinned at 1. The
export-fidelity shape again — **a flow that pre-clamps is invisible to the backstop's
arithmetic.**

**FINDING 1 — `dt = ½` CLEARS THE COMPENSATION POINT EVERYWHERE TESTED.** Season-low chamber
CO₂ against the 61.07 ppm floor: sealed **57.89 → 75.06**, perennial **56.03 → 74.91**,
consumer 73.29 → 74.36, against RK4-converged 76.28/75.75/74.64. **One halving moves the
crossing from 5 % below the floor to 23 % above it**, and every further refinement moves it
~1 ppm. It holds at **every enrichment level, which `dt = 1` does not**: across ×1…×5 the
shipped step collapses 57.89 → 33.83 → 25.18 → 19.07 → **12.13 ppm** while `dt = ½` sits at
75.06 → 74.74 → 74.54 → 74.39 → **74.31**. ⚠ **New, and not in the record: at ×5 the shipped
integrator's margin is 0.975 — already below 1**, so enrichment past ×4 rations on the step
that ships today; at `dt = ½` the same run sits at 2.65.

**FINDING 2 — ⚠ BOTH NUMBERS BELONG IN THE DECISION.** Sealed chamber, Euler, against the
converged limit: season-low CO₂ is **−24 %** at `dt = 1` and −1.6 % at `dt = ½`; peak leaf
carbon **+4.0 %** → +1.5 %; **harvest — the headline output — only −0.7 %** → −0.3 %. The tail
statistic that motivated the whole gate moves 24 %; the number anyone would quote as the
result moves under 1 %. Quoting either alone misleads, in opposite directions.

**FINDING 3 — THE PARKED LEAF MECHANISM IS WORSE AT `dt = 1` THAN ITS OWN RECORD SHOWS.**
Recorded as *silent rationing under Euler*; it is also a **much deeper compensation-point
crossing** — the perennial chamber hits **28.02 ppm against a 61.07 floor, a 2.2× crossing**,
where the frozen tree crosses by 5 %. Same defect, same unfreeze, but a stronger argument for
moving the step than the record carries. `dt = ½` clears it at `k·h` 0.471–0.483 — **2.1× of
headroom**, matching the plan's "clears by less than 2×" framing; `dt = ¼` leaves 4.8×.

**FINDING 4 — THE PRICE HALF IS CHEAP, AND ITS ONE ANOMALY IS NOT INTEGRATION ERROR.** Nothing
rations at any refinement on any of the 9 station/physics scenarios; margins run 10× to 1550×,
and drift at `dt = ½` is round-off (1e-13) or truncation (4e-06) everywhere except the power
family's 4.3e-03. ⚠ **That one is forcing quadrature, not accuracy**: the half-sine is a
Riemann sum over `steps_per_day` samples, and refining it *changes the energy delivered*
(2.7345e7 → 2.7463e7 → 2.7492e7 → 2.7502e7 J/day at 24/48/96/converged) — **the shipped
24-sample sum under-delivers solar by 0.21 %**, so refining power's step changes what the
scenario physically *is*. None of it is forced: those are separate scenarios with their own
steps and a biosphere change leaves them untouched. **Recommendation: do not move them.**

**FINDING 5 — ⚠⚠ FLAGGED, NOT CAUSED HERE: `water_biting` CONVERGES TO TWO DIFFERENT ANSWERS.**
On a shipped golden, Euler settles at peak leaf 0.70 / harvest 0.73 while RK4 settles at
**0.065 / 0.005** with chamber CO₂ pinned at ambient 357.00 ppm — the crop is dead from the
first step. Both are **stable under an 8× refinement** and disagree ~10× on leaf and ~50× on
harvest, so this is **not truncation error** and a finer step does not fix it. Prime suspect
(a hypothesis, not a diagnosis): the self-clamping `min` at `soil_layers.py:134`, evaluated
independently at each of RK4's four stage states — the composition of four clamped stages is
not the clamp of the composition, the `a-clamp-hides-a-wrong-amount` shape, and scope finding
3 says the margin measure cannot see it. **Decision-relevant: this is a new argument against
every RK4 row on the menu** — choosing RK4 would change that golden *qualitatively*. Named as
a successor, not opened.

**FINDING 6 — THE WALL CLOCK, MEASURED AND BOUNDED.** Raw simulation work scales **2.00× at
`dt = ½`, 4.00× at `dt = ¼`**. Today's full suite (`-n 12`, tree clean at `e4f50d1`):
**2330 passed, 5 skipped in 371.73 s = 6 m 12 s** — which **supersedes the 7 m 05 s** in
`docs/test-suite-runtime.md`; the suite is now *faster* than its documented figure despite
having grown, so that headline is stale. ⚠ **A subset run is not a share**: the 71 test files
touching the biosphere (1493 tests) took **9 m 57 s**, *longer than the whole suite*, because
a subset defeats the conftest's xdist grouping — recorded only so nobody re-derives and quotes
it. Under `-n 12` wall clock is set by the longest worker, not total work, so `dt = ½` is
bounded below by 6 m 12 s and above by ~12 m 24 s. A bound, not a prediction.

**RECOMMENDATION (⚠ the decision is the user's and is NOT taken here): Euler at `dt = ½`,
biosphere only, leaving power/thermal/ECLSS/crew alone.** Cheapest option that is numerically
clean and scientifically correct on everything measured; it moves the answer *toward* the
converged limit rather than quieting a guard, and touches neither the integrator contract nor
the backstop's Euler-only scope. Two caveats the measurement adds: **`½` leaves only 2.1× of
headroom on an emergent bound no load-time check can guard** (`¼` leaves 4.8×), and **the
ceremony is bigger than the plan prices** — a `src/station/driver.py` change on top of the
three freeze contracts.
