# CO₂ enrichment — how much room does the shipped configuration really have?

**Taken 2026-08-13 on the user's call**, as successor 3 of
[`post-roadmap-allocation-headroom.md`](post-roadmap-allocation-headroom.md) §10, which
named it the one *"most likely to be needed soonest, because raising CO₂ is the cheapest
realism move on the board"* and left it with a single sentence of evidence:

> Under **Euler** the sealed chamber loses roughly a fifth of its headroom per doubling of
> chamber CO₂ — 1.307 → 1.146 → **1.044** at ×4. Four per cent of room.

Probes: `M:/claud_projects/temp/co2-fragility/` (`probe_sweep.py`, `probe_trace.py`,
`probe_pool.py`). Everything below is measured on frozen `main`; **nothing in this
document changed a line of `src/`**, and no golden was regenerated.

---

## 0. What the harness had to prove before anything else

`probe_sweep.py` re-derives the margin the way `allocation-headroom` defined it — the
unclamped minimum of `stock.amount / demand` inside `_scale_factors`, arbitration's own
arithmetic — and reproduces all nine recorded points **exactly**: sealed 1.3072 / 1.1458 /
1.0441, perennial 1.5528 / 1.2946 / 1.0789, consumer 2.0000 / 1.7794 / 1.3269. The
predecessor's numbers stand; everything new here is what the three points did not show.

The scenario knob is `chamber_co2_mol0`, and it is quoted here in **ppm** as well as in
multiples, because the realism move it prices — greenhouse CO₂ enrichment — is quoted in
ppm by everyone who does it. ×1 is `Ca = 357 ppm` (`Ci₀ = 250`), so ×2.80 is 1000 ppm and
×3.36 is 1200 ppm.

> ⚠ **"1000–1200 ppm is what commercial enrichment uses" is UNCITED.** It is the range
> usually quoted, it is not on this project's shelf (`docs/bvad-reference.md` carries no
> CO₂ concentration at all), and no build should rest on it until it is sourced. It is used
> below only to mark *where on the sweep a plausible realism move would land*, which is a
> navigation aid, not a parameter.

## 1. FINDING 1 — the "fifth per doubling" is a chord through three points; the curve is not monotone

Sealed chamber, Euler, 3 yr, worst margin over the run:

| CO₂ | ppm | margin | binding step | | CO₂ | ppm | margin | binding step |
|---|---|---|---|---|---|---|---|---|
| ×1.00 | 357 | 1.3072 | 498 | | ×3.36 | 1200 | **1.1067** | 498 |
| ×2.00 | 714 | 1.1458 | 498 | | ×4.00 | 1428 | 1.0441 | 803 |
| ×2.80 | 1000 | **1.0963** | 498 | | ×5.00 | 1785 | **0.9749** | 803 |

The margin **rises** from ×2.80 to ×3.36 (1.0963 → 1.1067) and the step that owns the
minimum jumps from 498 to 803. Two different days compete for the minimum — day 193 of
year 1 and day 193 of year 2 — and they respond to enrichment differently, so the envelope
is not monotone. A three-point fit through ×1/×2/×4 reads as a smooth decay because all
three happen to be sampled on one side of the crossover.

**Nothing in the predecessor's numbers is wrong; the trend line drawn through them was.**

## 2. FINDING 2 — the cliff is at 1785 ppm, above where a realism move would land

Under Euler the sealed chamber first rations between ×4 and ×5 — margin 1.0441 (clean) at
1428 ppm, **0.9749 with one firing** at 1785 ppm. Perennial crosses between ×5 and ×6
(1.0459 → 0.9753). A frozen biosphere golden asserts the rationing count is **0**, so a
scenario that crossed this line would go red in the suite rather than pass quietly — this
is not a silent failure *for the frozen scenarios*. It is silent only for a run nobody
gates.

At the levels a realism move would actually pick, the margin is ~1.10 (sealed at both 1000
and 1200 ppm) and ~1.10–1.16 (perennial). **On the arbitration measure alone, enrichment
to greenhouse levels is survivable with ~10 % of room.** §4 is why that sentence is a trap.

## 3. FINDING 3 — demand FALLS with enrichment, and the pool falls faster

`probe_trace.py`, sealed chamber, Euler, at the binding day (year 1, day 193):

| CO₂ | pool (mol C) | one day's demand | margin |
|---|---|---|---|
| ×1.00 | 0.2206 | 0.1688 | 1.3072 |
| ×2.80 | 0.1588 | 0.1449 | 1.0963 |
| ×4.00 | 0.1322 | 0.1167 | 1.1334 |
| ×5.00 | 0.1172 | 0.0979 | 1.1973 |

The intuitive story — *more CO₂ → faster growth → a bigger daily draw → less room* — is
**not what happens**. The daily demand at the tight day *falls* by 42 % across that range.
The margin closes because the **pool** falls by 47 %. Whatever is spending the headroom is
on the supply side of the ratio, not the demand side, and every explanation that starts
with "the crop gets hungrier" is describing a different run than this one.

## 4. FINDING 4 — the chamber has a physical carbon floor, and the shipped step steps straight through it

`gross_leaf_assimilation` is `max(0, min(Ac, Aj))` and both FvCB branches carry the factor
`(Ci − Γ*)`, so **assimilation is exactly zero at or below the CO₂ compensation point**.
With `Γ* = 42.75 µmol mol⁻¹` and the fixed `Ci/Ca = 0.7` draw-down, the crop cannot draw
the chamber below

    Ca_floor = Γ* / ci_ratio = 61.07 ppm    (0.0611 mol C in a 1000-mol chamber)

That is the model's own statement about where photosynthesis stops. Measure the
season-**minimum** chamber CO₂ against it (`probe_pool.py`):

| CO₂ charge | ppm | **RK4** season low | vs floor | **Euler** season low | vs floor |
|---|---|---|---|---|---|
| ×1.00 | 357 | 76.4 ppm | +25.1 % | 57.9 ppm | −5.2 % |
| ×2.00 | 714 | 76.3 ppm | +25.0 % | 33.8 ppm | −44.6 % |
| ×2.80 | 1000 | 76.3 ppm | +25.0 % | 25.2 ppm | −58.8 % |
| ×3.36 | 1200 | 76.3 ppm | +25.0 % | 25.7 ppm | −57.9 % |
| ×4.00 | 1428 | 76.4 ppm | +25.1 % | 19.1 ppm | −68.8 % |
| ×5.00 | 1785 | 76.5 ppm | +25.2 % | 12.1 ppm | −80.1 % |
| ×8.00 | 2856 | 77.1 ppm | +26.3 % | 14.5 ppm | −76.2 % |

**Under RK4 the season low is pinned at ~76 ppm no matter how much carbon you put in.**
That is the physics working exactly as designed: the crop draws the chamber down until
assimilation shuts off, the shutoff is a property of the kinetics, and the initial charge
cannot move it. Enrichment changes how much carbon ends up in biomass (peak plant carbon
2.07 → 3.60 mol C) and **not** where the chamber bottoms out.

Under Euler at `dt = 1` the same quantity **collapses from 57.9 ppm to 12.1 ppm**. The crop
is fixing carbon at concentrations where the model says it fixes none. Both other chambers
agree: perennial RK4 holds 75.9–76.7 ppm across the whole sweep while Euler falls 56.0 →
13.2; consumer RK4 holds 74.7–74.8 while Euler falls 73.3 → 34.5.

The single-step evidence is in the trace. At ×4, step 803: the pool holds **0.1835 mol C
(183.5 ppm)** at the start of the day and one day's withdrawal is **0.1758 mol C** — the
step starts three times above the floor and lands at ~8 ppm, crossing the shutoff in the
middle of a step that never re-evaluates. Even the **frozen ambient** run does it: 220.6 ppm
at the start of the day, 168.8 ppm withdrawn.

> ⚠ **The structure is cited; the level of the floor is not.** That a shutoff exists, and
> where it sits in the rate law, is Farquhar–von Caemmerer–Berry and is what
> `photosynthesis.py` implements. The **value** `Γ* = 42.75` is one of the 13 `TODO(cite)`
> entries in `photosynthesis.yaml` (*"provisional, CO₂ compensation point w/o Rd at
> ~25 °C; primary citation pending"*). So "the shipped step crosses the shutoff" is robust —
> it crosses by a factor of three — while "the floor is 61.07 ppm" inherits a provisional
> number and should be quoted with that attached.

## 5. FINDING 5 — the convergence check, and why it is not the identity the last record warned about

`allocation-headroom` finding 5 warned that `margin ∝ 1/dt` is near-tautological and that
shrinking `dt` until a guard goes quiet repeats finding 9 with the sign flipped. That
warning applies in full to the margin column here, and the identity is visible in it
(sealed, ×1: 1.3072 → 2.5638 → 5.4574 at `dt` = 1, ½, ¼ — a clean doubling).

**The season-low CO₂ does not behave that way, which is the whole point:**

| | `dt = 1` | `dt = ½` | `dt = ¼` |
|---|---|---|---|
| Euler, ×1.00 | 57.9 ppm | 75.1 ppm | **75.8 ppm** |
| Euler, ×2.80 | 25.2 ppm | 74.6 ppm | **75.1 ppm** |
| Euler, ×4.00 | 19.1 ppm | 74.4 ppm | **74.9 ppm** |
| Euler, ×5.00 | 12.1 ppm | 74.3 ppm | **74.8 ppm** |
| RK4, ×1.00 | 76.4 ppm | 76.3 ppm | 76.3 ppm |

It **converges** rather than scaling — onto the same ~75 ppm the already-converged RK4
solution reports, from every enrichment level, with a step refinement that is *not* enough
to move the guard's arithmetic anywhere near quiet. This is a statement about the
**answer**, not about the backstop: at `dt = ½` the shipped integrator resolves the
compensation point and at `dt = 1` it does not.

## 6. FINDING 6 — the frozen ambient scenario already crosses the floor, and the error inflates the payoff

At ×1 — the shipped, frozen, golden-pinned configuration — the sealed chamber's season low
is **57.9 ppm against a converged 75.8**, i.e. **24 % low**, and it is below the model's own
shutoff. `allocation-headroom` finding 6(b) recorded the shipped step's truncation error as
**3.2 % on peak leaf carbon**; the same error on the chamber's minimum CO₂ is **24 %**, and
under enrichment **3–6×**. Same phenomenon, a far more sensitive observable, and this one
has a physical interpretation rather than a percentage: *the crop assimilates below the
compensation point.*

This is **not a contract defect** — `Euler / dt = 1` is the first item in the biosphere
freeze and the goldens are its record, not a victim of it. It is a number anyone re-opening
the step contract needs, and it had not been measured.

⚠ It also runs the wrong way for a realism claim: at ×4 the shipped integrator reports peak
plant carbon **2.7649 mol C against RK4's 2.6502 (+4.3 %)**. The carbon fixed below the
shutoff is real biomass in the run. **The integrator that cannot survive enrichment is also
the one that overstates its benefit.**

## 7. FINDING 7 — the integrator inversion goes both ways, and is trajectory divergence

`allocation-headroom` finding 8 flagged as unexpected that at ×4 the *shipped* integrator is
thinner than RK4 (1.044 vs 1.490). The sweep adds the opposite case: at ×8 on the consumer
chamber, **RK4 breaks (`ArbitrationError`, margin 0.9222) while Euler runs clean at 1.1277**.

Neither is a paradox once §4 is in hand. The two integrators are not running the same
trajectory by ×4 — Euler's crop has been fed carbon that RK4's crop never got — so their
margins are measured on different states, and which one is tighter is not fixed by
integration order. **A margin comparison between integrators at the same `dt` compares two
different runs**, and should not be read as "RK4 is safer" or "Euler is safer".

## 8. What this does to the three routes the predecessor priced

- **(B) A finer biosphere step** gains a *scientific* justification it did not have.
  `allocation-headroom` finding 6(a) priced it as moving the margin without moving the
  answer (0.06 % on peak leaf carbon under RK4) — true, and measured under RK4. Under the
  integrator that **ships**, refining the step moves the sealed chamber's minimum CO₂ by
  24 % at ambient and by 3–6× under enrichment, and moves it from *below the model's own
  shutoff* to *above it*. The three-contract price in that document is unchanged; the
  benefit column now has an entry that is not about leaves and not about the guard.
- **(C) Positivity from kinetics** is *not* refuted, but its discriminator needs a clause.
  The premise was that the pool dependence "already exists via `Ci` and is simply not steep
  enough". What §4 shows is that the kinetics already contain a **hard shutoff**, and the
  failure is one of **resolution, not steepness** — a one-day step crosses it. A steeper,
  pool-saturating form would still help under Euler (it lowers the *start-of-step* rate,
  which is the only rate a step sees), but any form whose limiting happens *at* a threshold
  has this same problem at `dt = 1`. So the shelf search should now ask for a form that
  limits the rate **before** the threshold, not one that merely has a sharper one.
- **(A) Refuse the leaf mechanism** is untouched by this work.

## 9. The verdict on the question that was asked

**"Raise the chamber CO₂" fails scientifically about 600 ppm before it fails numerically,
and the margin is the wrong alarm.** At 1000–1200 ppm the arbitration guard is quiet
(margin ~1.10, zero rationing, every golden gate green) while the chamber's minimum CO₂ is
**wrong by a factor of three** and sits far below the concentration at which the model says
photosynthesis stops. A team watching the guard would ship it.

So the enrichment fragility recorded as *"four per cent of room"* is real but was named for
the wrong thing: it is not a carbon-supply limit being approached, it is a **fixed
integration error being amplified** — the error is already present at ambient, and
enrichment multiplies it. **On the shipped step there is no enrichment level that is
scientifically clean, including none at all.** At `dt = ½` every level tested is both clean
and correct.

**NOTHING WAS BUILT.** This is a measurement; `src/` is untouched, no golden moved, and the
`dt` decision it informs is a three-contract unfreeze that remains the user's.

## 10. Successors, named

1. **The `dt = 1` contract decision for a sealed chamber** — unchanged as an item, but it
   now has a second, independent argument, and this one is about the science rather than
   the guard. It is still the same three-contract price.
2. **`Γ*`'s citation.** The floor this whole record is measured against is a `TODO(cite)`
   value. The measurement does not depend on it (a factor-of-three crossing survives any
   plausible re-value), but the *number* 61.07 ppm should not appear in a science claim
   until the param is sourced. It is one of `photosynthesis.yaml`'s 13.
3. ⚠ **A chamber CO₂ *controller* is a different object from a bigger initial charge**, and
   is what real enrichment is. `chamber_co2_mol0` is a one-shot charge that the crop eats
   within a season; a controller holding 1000 ppm would keep the pool far above the floor
   and would make this fragility disappear — while introducing a make-up flow, which is
   the O₂ regulator's shape and inherits `post-roadmap-o2-makeup-reversal.md`'s finding
   about direction. **Nothing here prices that**, and it should not be assumed cheap.
4. **The chamber's minimum CO₂ deserves a science band.** `science_bands` in the two
   manifests already give assertions contract standing; "the sealed chamber's season-low
   CO₂ stays above the compensation point" is exactly that shape, is measurable on existing
   goldens, and would have caught this on the day it was introduced. ⚠ It would be **red on
   the frozen tree today** (57.9 vs 61.07), so it cannot be added without either the step
   decision in (1) or an explicitly documented allowance — which is the honest version of
   what the tree currently does silently.
