## **The draw census** (the 2026-09-29 review's Step 2, slice 1 — only a crop drawing CO₂ comes near its limit)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**MEASURED 2026-09-30 on the user's call** (*"step 2's first slice - work on it"*). Slice 1 of
Step 2 ("small air volumes against a big step") of the 2026-09-29 review proposal. Measurement
only: nothing frozen touched, no option priced, nothing decided. Plan, predictions (committed
before any code, `6aa7329`), grading and the full table:
`docs/plans/post-roadmap-draw-census.md`.

## What was built

* `rust/crates/station/examples/draw_census.rs` — one lab report. For every frozen run and
  every store the Euler backstop guards: the most any one step takes out of the store, as a
  fraction of what it holds when the step starts, plus the step, the day, the flow that took the
  most, the mean, and how many steps took more than 0.1 and 0.5. One probe, three drivers (the
  season loop with the re-sow applied before the probe, the single-rate loop, and both sides of
  the station's two-rate day). The roster is every golden file on disk except
  `state_snapshot.json`, a hand-written format fixture. The 5- and 15-season runs are measured
  separately.
* Nothing else. `readouts::step_draws` and the Step 1 example are unedited. No test added: the
  one figure worth pinning, the jar's 0.757, already has its gate.

## Controls — all four held on the first run

* 18 state goldens byte-exact; the 15-year heat-closure run bit-identical to the reference
  runner's end state; `drift_summary.json` is folded from the two 15-season runs that matched.
* The jar's pinned 0.756662 at step 777, to the last printed digit.
* About 1.9 million steps re-applied bit for bit from the probe's own sums. This is the
  control that can fail on a run that never rations.
* Firing counts agree everywhere. Because every frozen run reads `0 == 0`, it was also run on
  the lab leaf-form jar, which rations: the probe names CO₂, worst 1.1465, 135 steps over 0.5.

## Findings

**1. Only a CO₂ pool drawn by the crop comes near its limit.** Every draw above 0.1 that is not
a fixed rate or the weather is the crop's uptake on a CO₂ pool. The jar: worst **0.757**, over
0.5 on **52** steps and over 0.1 on 632, of 3660. The perennial chamber 0.262, the consumer
chamber 0.230, the sealed station's crop on the cabin air **0.078**.

**2. Every other store's worst is rate × step or the weather, and cannot reach 1 by
construction.** Condensate 0.125 (0.5/day × ¼ day), cabin CO₂ on the cabin side 0.060
(1e-3/s × 60 s), recovered water 0.060, cabin water 0.030, stem reserve 0.025, flat on every
step. The vapour store reads **0.447 in every sealed chamber on the same step**: a 16.7 °C →
9.7 °C night lowers the humidity target and the condenser takes the whole excess at once.

**3. ⚠ The four 7-day station goldens cannot see the station crop pull on the cabin air.** Their
crops are seedlings (0.002 and 0.0016). Only the 4-year sealed station grows one. A sizing
question about the station's plants, Step 3's included, has to be measured there.

**4. ⚠ My station prediction was off by a factor of two, and the cause was a unit.** Predicted
≈ 0.16, measured 0.078. The basis (the crew-loop record's 6.31× headroom) is the crop's biggest
**daily net** gain against the pool, from when the plant step was a whole day; I read it as a
per-step draw. Redone per quarter-day step it gives about 0.07. The same days-against-steps
mistake `log/step-unfreeze.md` records.

**5. For Step 3 (the reason the review said to read this slice first):** no water store is
within a factor of ten of its limit except the vapour store, and that one is set by a cold
night, not by transpiration. Step 3's +21 % water use is not expected to squeeze a water store —
expected, not measured.

## Predictions

Nine of eleven held. P5 held except its vapour ceiling (0.4 predicted, 0.447 measured, the
weather). P7 was refuted (finding 4).

## Next

Step 2 slice 2: price options A (a finer step everywhere), B (split a step only where a store
is pulled hard) and C (uptake that falls as CO₂ runs low), judged by convergence against a much
finer step. Finding 1 bears on all three: every store at risk is drawn by the same flow. The
user wants C built regardless; it needs a published form or it is WHAT-IF.
