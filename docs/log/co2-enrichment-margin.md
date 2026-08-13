## **CO₂ enrichment** — the shipped step crosses the compensation point

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per
> work item. The fuller record, with every table, is
> [`../plans/post-roadmap-co2-enrichment-margin.md`](../plans/post-roadmap-co2-enrichment-margin.md).
> Written new rather than migrated, so there is no pre-split table cell behind it.

**DIAGNOSED 2026-08-13, NOTHING BUILT.** Taken as successor 3 of `allocation-headroom.md`,
which named it *"most likely to be needed soonest, because raising CO₂ is the cheapest
realism move on the board"* and left it with one sentence: under Euler the sealed chamber
*"loses roughly a fifth of its headroom per doubling"*, reaching margin 1.044 at ×4 — four
per cent of room. Measured on frozen `main`; probes in `M:/claud_projects/temp/co2-fragility/`,
`src/` untouched, no golden regenerated. The harness reproduces all nine recorded points
exactly before adding anything.

**FINDING 1 — THE TREND LINE WAS A CHORD THROUGH THREE POINTS.** A fine sweep is **not
monotone**: sealed-chamber Euler margin runs 1.3072 (×1) → 1.0963 (×2.80) → **1.1067
(×3.36, rising)** → 1.0441 (×4) → 0.9749 (×5). Two days compete for the run minimum — day
193 of year 1 and of year 2 — and the binding step jumps 498 → 803 across the crossover, so
sampling ×1/×2/×4 lands on one side of it and reads as smooth decay. None of the recorded
numbers is wrong; the line drawn through them was.

**FINDING 2 — THE CLIFF IS AT 1785 ppm, ABOVE WHERE A REALISM MOVE WOULD LAND.** Rationing
first fires between ×4 (1428 ppm, clean) and ×5 (1785 ppm, one firing); perennial between
×5 and ×6. At 1000 and 1200 ppm — the levels enrichment would actually use — the margin is
~1.10 with zero firings and every golden gate green. ⚠ *That "1000–1200 ppm is what
commercial enrichment uses" is **uncited**: it is the range usually quoted and it is not on
this shelf (`bvad-reference.md` carries no CO₂ concentration at all). It marks where on the
sweep a plausible move lands; it is not a parameter.*

**FINDING 3 — DEMAND *FALLS* WITH ENRICHMENT; THE POOL FALLS FASTER.** At the binding day
the daily demand drops 0.1688 → 0.0979 mol C from ×1 to ×5 while the pool drops 0.2206 →
0.1172. The intuitive story — richer air, hungrier crop, bigger draw — describes a
different run. Whatever spends the headroom is on the **supply** side of the ratio.

**FINDING 4 — ⚠⚠ THE CHAMBER HAS A PHYSICAL CARBON FLOOR AND THE SHIPPED STEP CROSSES IT.**
`gross_leaf_assimilation` is `max(0, min(Ac, Aj))` and both FvCB branches carry `(Ci − Γ*)`,
so assimilation is **exactly zero** at or below the compensation point: with `Γ* = 42.75`
and `Ci/Ca = 0.7`, the crop cannot draw the chamber below `Ca = 61.07 ppm`. Under **RK4**
the season-low CO₂ is **pinned at ~76 ppm at every enrichment level** (76.4 at ×1 through
77.1 at ×8; perennial 75.9–76.7; consumer 74.7–74.8) — the physics working as designed,
where the initial charge cannot move the floor. Under **Euler at `dt = 1` the same quantity
collapses 57.9 → 12.1 ppm** across ×1 → ×5. The crop fixes carbon where the model says it
fixes none. Single-step evidence: at ×4, step 803 starts the day holding 183.5 ppm and
withdraws 175.8 of it, crossing the shutoff mid-step in a step that never re-evaluates —
and the **frozen ambient** run does it too (220.6 ppm held, 168.8 withdrawn). ⚠ The
*structure* is cited (FvCB); the *level* is not — `Γ* = 42.75` is one of
`photosynthesis.yaml`'s 13 `TODO(cite)` entries, so "the step crosses the shutoff" is robust
(it crosses by 3×) while "the floor is 61.07 ppm" inherits a provisional value.

**FINDING 5 — THE CONVERGENCE CHECK, WHICH IS THE HALF THAT IS *NOT* ARITHMETIC.**
`allocation-headroom` finding 5 warned that `margin ∝ 1/dt` is near-tautological and that
shrinking `dt` until a guard goes quiet is finding 9 with the sign flipped. That warning
holds for the margin column here (1.3072 → 2.5638 → 5.4574, a clean doubling). **The
season-low CO₂ does not scale — it converges:** 57.9 → 75.1 → 75.8 ppm at ×1, and 12.1 →
74.3 → 74.8 at ×5, onto the same ~75 ppm the already-converged RK4 run reports, from every
enrichment level. A statement about the **answer**, not the backstop: at `dt = ½` the
shipped integrator resolves the compensation point and at `dt = 1` it does not.

**FINDING 6 — THE FROZEN AMBIENT SCENARIO ALREADY CROSSES THE FLOOR, AND THE ERROR INFLATES
THE PAYOFF.** At ×1 the shipped, golden-pinned run bottoms at **57.9 ppm against a converged
75.8 — 24 % low, and below the model's own shutoff**. `allocation-headroom` finding 6(b)
put the shipped step's truncation error at **3.2 % on peak leaf carbon**; the same error on
the chamber's minimum CO₂ is **24 %**, and 3–6× under enrichment — same phenomenon, far more
sensitive observable, and this one has a physical reading rather than a percentage. Not a
contract defect (`Euler / dt = 1` is the first item in the biosphere freeze), but not
previously measured. ⚠ And it runs the wrong way for a realism claim: at ×4 the shipped
integrator reports peak plant carbon **2.7649 against RK4's 2.6502 (+4.3 %)**. **The
integrator that cannot survive enrichment is the one that overstates its benefit.**

**FINDING 7 — THE INTEGRATOR INVERSION GOES BOTH WAYS.** The predecessor flagged Euler being
*thinner* than RK4 at ×4 (1.044 vs 1.490) as unexpected; the opposite case also exists — at
×8 the consumer chamber **breaks under RK4 (0.9222) while Euler runs clean at 1.1277**.
Neither is a paradox: by ×4 the two integrators are on different trajectories, because
Euler's crop has been fed carbon RK4's crop never got. **A margin comparison between
integrators at one `dt` compares two different runs**, and settles nothing about which is
safer.

**THE VERDICT, AND WHAT IT DOES TO THE THREE ROUTES.** *"Raise the chamber CO₂" fails
scientifically about 600 ppm before it fails numerically, and the margin is the wrong
alarm* — at 1000–1200 ppm the guard is quiet and every gate is green while the chamber's
minimum CO₂ is wrong by a factor of three. A team watching the guard would ship it. So the
recorded "four per cent of room" is real but was named for the wrong thing: not a
carbon-supply limit being approached, but **a fixed integration error being amplified** —
already present at ambient, multiplied by enrichment. **On the shipped step no enrichment
level is scientifically clean, including none at all**; at `dt = ½` every level tested is
clean *and* correct. Route **(B)**, the finer step, therefore gains a benefit entry that is
neither about leaves nor about the guard — its three-contract price is unchanged. Route
**(C)** is not refused but its discriminator needs a clause: the kinetics already contain a
hard shutoff, so the failure is one of **resolution, not steepness**, and the shelf search
should ask for a form that limits the rate *before* a threshold rather than *at* one. Route
**(A)** is untouched. Successors named: the `dt` contract decision (unchanged, now with a
second and independent argument); `Γ*`'s citation; ⚠ **a chamber CO₂ *controller* is a
different object from a bigger initial charge** — real enrichment holds a level rather than
charging once, which would make this fragility disappear while introducing a make-up flow
with the O₂ regulator's direction hazard, and nothing here prices it; and a `science_bands`
entry for the chamber's minimum CO₂, which is exactly that contract's shape and would have
caught this on day one — ⚠ but which is **red on the frozen tree today**, so it cannot land
without the step decision or an explicitly documented allowance.
