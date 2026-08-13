# The CO₂ setpoint controller, priced — Step 0 axis 2 of the direction plan

**DIAGNOSED 2026-08-13, NOTHING BUILT.** Probe-only, on frozen `main` (`36d0ae5`).
`git diff src/` empty, no golden regenerated, no schema touched. Harness:
`M:/claud_projects/temp/co2-fragility/probe_controller.py`.

This is axis 2 of Step 0 in [`post-roadmap-direction.md`](post-roadmap-direction.md).
Its own §2 says the controller *"may dissolve this whole gate"* and made the step
recommendation **conditional** on this measurement. Question 1 in §7 (which step) was
withdrawn pending it.

---

## 1. What was being tested, and why it was blocking

`log/co2-enrichment-margin.md` closed with an aside: a chamber CO₂ *controller* — real
enrichment **holds** a concentration rather than charging the chamber once — *"would make
this whole fragility disappear."* Read literally that sentence attacks both halves of the
case for a finer integration step:

- **The science half.** The shipped step drives the sealed chamber's season-low CO₂ to
  57.9 ppm, below the 61.07 ppm compensation point where FvCB assimilation is exactly
  zero. A controller holding 1000–1200 ppm would never approach that floor, so the
  crossing would **stop existing** at `dt = 1`.
- **The numerical half.** `k·h = (rate · h) / stock`. A setpoint pins `stock` high while
  assimilation, saturating in `Ci`, rises far less than proportionally — so `k·h` should
  *fall*, and the parked leaf branch's 1.055 might clear with no step change at all.

If both held, the largest ceremony this project has run (three freeze contracts) would
have been bought to fix what the next realism item removes anyway. **Unpriced is why it
could not be assumed downstream** — hence a probe rather than a deferral.

## 2. The probe, and its two controls

A **dawn top-up clamp**, not a flow: the `reset` hook of `run_season` (`(n, state) ->
state`, consulted before each step) sets `biosphere.carbon_pool` to the setpoint and moves
the difference to or from a probe boundary reservoir. Nothing authored, nothing that can
reach a golden.

Three design points that were not free:

- **Composition.** `biosphere.carbon_pool` carries `{CARBON: 1.0, OXYGEN: 2.0}`. The
  reservoir is built by hand rather than via `boundary.source`, which takes no
  `composition` and would default to `{CARBON: 1}` — silently creating or destroying
  oxygen on every injection.
- **Composed with, not replacing, the calendar.** The hook calls `annual_reset` and then
  clamps, following the `resow_water_return` precedent.
- **`conservation.assert_conserved` runs across the hook** on every step, unchanged.

**Control A — the plumbing is inert.** With `setpoint=None` the replace path still runs
with a deficit of exactly 0.0, and the whole trajectory must be bit-identical to
`run_perennial`. A digest over hex-floats of every stock at every step, skipping the probe
reservoir:

| scenario | `run_perennial` | clamp, no-op | identical | frozen margin | frozen min CO₂ |
|---|---|---|---|---|---|
| `sealed_chamber` | `231be2442c6df87b` | `231be2442c6df87b` | **yes** | 1.3072 @ step 499 | 57.9 ppm |
| `perennial_chamber` | `f175ea6648ff5c63` | `f175ea6648ff5c63` | **yes** | 1.5528 @ step 805 | 56.0 ppm |
| `consumer_chamber` | `ecb5f24dde058056` | `ecb5f24dde058056` | **yes** | 2.0000 @ step 2 | 73.3 ppm |

**Control B — the clamp actually binds.** With it live the binding day must *migrate*, or
the probe is measuring the frozen run under a different name. It does: 499 → 262,
805 → 262, 2 → 262.

The margin is arbitration's own arithmetic, traced **unclamped** by monkeypatching
`arb._scale_factors` — the same instrumentation `allocation-headroom` used. `> 1` is clean;
`< 1` is a hard error under RK4 and silent rationing under Euler.

---

## 3. FINDING 1 — the ambient control inverts the hypothesis

Before testing enrichment, hold the chamber at the level it already starts at (357 ppm).
This should be nearly a no-op. It is the opposite:

| `sealed_chamber`, Euler `dt = 1` | margin | season-low CO₂ | rationing firings |
|---|---|---|---|
| frozen (no controller) | **1.3072** | 57.9 ppm | **0** |
| held at ambient, 357 ppm | **0.2977** | **10.6 ppm** | **270** |

Holding the chamber steady at its own starting concentration is **four times worse** for
the margin than letting it deplete, and fires the backstop 270 times where the frozen run
fires it none. `perennial_chamber` reproduces it (1.5528 → 0.2978, 270 firings);
`consumer_chamber` too (2.0000 → 0.5393, 147).

**The mechanism, and it is not subtle in hindsight.** `k·h = (rate · h) / stock`. The
uncontrolled chamber **self-limits**: the pool depletes, `Ci` falls, assimilation falls
with it, and the ratio stays bounded. A controller at ambient removes that negative
feedback — it keeps `stock` small *and* keeps `rate` high all season, which is the worst
combination available. **The controller argument needs the stock to be genuinely large,
not merely steady.** "Holding" is not the operative mechanism; only "enriching" could be.

## 4. FINDING 2 — no realistic setpoint clears, on any chamber

Sweep the setpoint, Euler `dt = 1`, 3 years. Both criteria are **within-run**: season-low
pool CO₂ above the 61.07 ppm floor, and unclamped margin above 1.

`sealed_chamber`:

| setpoint | season-low CO₂ | vs floor | margin | binding | firings | peak leaf C | injected | vented | verdict |
|---|---|---|---|---|---|---|---|---|---|
| 357 | 10.6 | −82.7 % | 0.2977 | @262 | 270 | 4.1770 | 90.30 | 52.57 | fail |
| 500 | 12.5 | −79.5 % | 0.3014 | @262 | 285 | 6.2847 | 132.10 | 74.90 | fail |
| 700 | 15.0 | −75.4 % | 0.3320 | @235 | 279 | 9.1107 | 187.99 | 104.84 | fail |
| **1000** | 15.7 | −74.4 % | **0.4121** | @235 | **252** | 12.8626 | 263.40 | 145.52 | **fail** |
| **1200** | 17.4 | −71.6 % | **0.4744** | @235 | **210** | 14.9058 | 305.53 | 168.37 | **fail** |
| 1500 | 27.0 | −55.8 % | 0.5728 | @845 | 150 | 17.3710 | 356.56 | 196.15 | fail |
| 2000 | 31.4 | −48.6 % | 0.7417 | @845 | 66 | 20.2070 | 405.61 | 222.83 | fail |
| **3000** | 319.6 | +423.3 % | **1.0797** | @845 | 0 | 21.9083 | 429.38 | 235.26 | **pass** |
| 5000 | 2257.0 | +3595.6 % | 1.7559 | @845 | 0 | 22.6803 | 440.72 | 240.89 | pass |

`perennial_chamber` is the same curve to three digits (357: 0.2978 / 8.2 ppm; 1200:
0.4744 / 15.9; 3000: 1.0797 / 318.5, first pass). `consumer_chamber` clears earlier
because the crew is a second carbon source (1200: 0.9515 / 37.3 ppm / 3 firings — still a
fail; 2000: 1.4846 / 698.9 ppm — first pass).

**The band the direction plan assumed a controller would run at — 1000–1200 ppm — is deep
in the failing region on every chamber.** Margin 0.41–0.51, hundreds of silent rationing
firings, and a season-low CO₂ that is *~75 % below the compensation point* — worse than the
frozen run's 57.9 ppm, not better. The first setpoint that clears both criteria is
**~3000 ppm** on two of the three chambers, above even the 1785 ppm cliff the predecessor
record measured and far above any plausible enrichment level.

## 5. FINDING 3 — the discriminator: the controller *doubles* the step price

⚠ Two numbers at `dt = 1` cannot settle this. The science half of the case against the
shipped step is a **truncation error**, not a threshold crossing, so a controlled run needs
its own convergence check: is controlled `dt = 1` converged against controlled `dt = ⅛`?
All comparisons are **within one setpoint** — a controlled run's peak carbon is never
compared against the frozen run's.

At **1200 ppm** (the realistic setpoint), `sealed_chamber`:

| integrator | `dt` | season-low CO₂ | margin | firings | peak leaf C | peak plant C |
|---|---|---|---|---|---|---|
| Euler | 1 | 17.4 | 0.4744 | 210 | 14.905793 | 82.700371 |
| Euler | ½ | 39.0 | 0.9478 | 6 | 19.104806 | 103.868697 |
| Euler | ¼ | 586.9 | 1.8962 | 0 | 19.147571 | 104.070496 |
| Euler | ⅛ | 893.6 | 3.7928 | 0 | 19.155403 | 104.146461 |
| RK4 | 1 | — | **0.7326 → hard error** | — | — | — |
| RK4 | ½ | — | **0.9528 → hard error** | — | — | — |
| RK4 | ¼ | 623.7 | 1.1381 | 0 | 18.740695 | 101.838064 |
| RK4 | ⅛ | 901.2 | 3.0055 | 0 | 18.977047 | 103.139594 |

**Controlled `dt = 1` is 22.2 % low on peak leaf carbon and 20.6 % low on peak plant carbon
against its own `dt = ⅛` limit.** The frozen tree's equivalent error is 3.2 %. The
controller makes the step error **seven times worse**, because holding the pool high keeps
assimilation at a rate the day-long step cannot resolve.

**The price, stated the way the decision needs it:**

| tree | clears both criteria at |
|---|---|
| frozen, no controller | `dt = ½` (75.1 ppm > floor, margin ~2.6) |
| controlled at 1200 ppm | `dt = ¼` — at `dt = ½` it **still fails both** (39.0 ppm, margin 0.9478, 6 firings) |

**The controller does not cancel the step gate. It doubles the cost of clearing it.**

## 6. FINDING 4 — the controller couples the two knobs the plan had separated

RK4 at `dt = 1` **hard-errors under the controller at every setpoint tested** — margin
0.7326 at 1200 ppm, 0.7522 even at the passing 3000 ppm. The direction plan's menu treats
integrator and step as independent choices; under a controller the "RK4 at `dt = 1`" row
is not merely a poor buy, it is **unbuildable**. At 1200 ppm even RK4 at `dt = ½` raises.

## 7. FINDING 5 — the passing setpoint grazes, and it vents

Three things about 3000 ppm, the first setpoint that clears:

- **Margin 1.0797 is 8 % of headroom** against a bound that `allocation-headroom` finding 7
  established is **emergent** and unguardable at load time. By the direction plan's own
  criterion — *"a step with room for the next two mechanisms"* — the passing setpoint fails.
- **It vents 235 mol of the 429 injected**, more than a third of the injected carbon thrown
  out of a habitat whose stated purpose is closing the carbon cycle. That is an argument
  against the route on *science* grounds, independent of any numerics. (Reported as
  measured; the probe vents because a setpoint is two-sided, and a real controller might be
  injection-only — which would then fail to hold the setpoint at all after resowing.)
- **The step error moved rather than vanished.** Same two-numbers discipline as the frozen
  24 %/3 %: at 3000 ppm the headline converges (peak leaf +0.010 %, peak plant −0.236 %
  against `dt = ⅛`) while the season-low pool is still **−88.0 %**. It is no longer
  *decision-relevant* — 319.6 ppm is nowhere near the 61.07 floor — but the error is intact.

## 8. FINDING 6 — this generalizes from "the probe clamp" to "any controller"

⚠ The plan's own caveat bites here: *a probe clamp is not the controller.* What was
measured is a dawn top-up — a state edit before the step, then depletion for a whole day. A
reader will reasonably ask whether a continuous **flow-based** injector would score better.

It cannot, and the engine says so directly. `src/simcore/arbitration.py:75`:
`scale_s = min(1, stocks[sid].amount / demand_s)` with `available_s` the **start-of-step**
level — its own docstring, line 18: *"so withdrawals never draw against same-step
inflows."* An injector flow contributes **nothing** to the margin's numerator in the step it
fires. So a make-up flow is *strictly worse* for the margin than a state edit applied before
the step, and the dawn clamp is the **upper bound on what any controller can do for
arbitration headroom under this engine's semantics**. The verdict is about controllers, not
about this probe.

## 9. Explicit omission

**The parked leaf branch (`leaf-expansion-blocked`, `cb668f6`) was not probed.** It draws
*more* carbon than the frozen tree, so it cannot rescue a margin already measured below 1 on
the lighter tree at every realistic setpoint — the conclusion is monotone in the direction
that matters. Recorded as an omission with its reason rather than left as a silent cap.

---

## 10. Verdict, and what it does to the direction plan

**The controller does not dissolve the gate; it sharpens it.** Every conditional the
direction plan attached to this measurement resolves against the controller:

- §2's *"⚠⚠ the controller may dissolve this whole gate"* — **refuted**. Both of its
  bullets were wrong, and the science-half bullet was wrong in the *opposite* direction
  from the one anticipated: holding the chamber makes the compensation-point crossing worse
  at every setpoint below ~3000 ppm, because the crossing was never about the initial
  charge — it is a truncation error that a high, sustained assimilation rate amplifies.
- §2's recommendation was *"Euler at `dt = ½` — conditional on the controller probe coming
  back negative."* The probe came back negative. **The condition is discharged and the
  recommendation stands** — with the observation that it is also the *cheaper* branch, since
  the controller path needs `dt = ¼` on top of a make-up flow that is itself unpriced.
- §3 item 3 moved the controller out from behind the gate on the grounds it might cancel it.
  **It moves back behind the gate**, now for a measured reason rather than an unpriced one:
  a controller at a realistic setpoint is not shippable on any step the tree could adopt
  today, and at 1200 ppm it needs `dt = ¼`.
- §7 question 1 (which step) was withdrawn pending this. **It can be re-put** — but axis 1
  (the step sweep across all 25 scenarios) has not run, so the *prices* of the candidate
  steps are still unmeasured. Ask it with that stated, or run axis 1 first.

**Successors named:**

1. **Step 0 axis 1** — the step sweep. Unchanged, and now the only remaining input to the
   step decision.
2. **The controller, if it is ever built, is a `dt = ¼` object.** Filed with its price
   measured instead of assumed. Its other costs are untouched by this work: a make-up flow
   inherits the O₂ regulator's direction hazard, and `authored ≠ validated` applies.
3. ⚠ **A `science_bands` entry for the chamber's minimum CO₂ would not have caught this
   either** — the controlled runs are probe-only and no band evaluates them. The band
   proposed in the direction plan §3 still belongs behind the step decision, unchanged.
