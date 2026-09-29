# Post-roadmap — plants and cabin trade gas within the day

**Opened 2026-09-29**, on the user's call *"start implementing"* the 2026-09-29 review proposal.
This is that proposal's first step. Record: `docs/log/intraday-gas-exchange.md`. The crew-loop
diagnosis this step answers: `docs/plans/post-roadmap-crew-coupled-loop.md` §9.

**Scope of this session: the lab slices only (1, 2, 3 and 5).** Nothing frozen is touched. The
adopt-or-not decision (slice 4) is the user's, and this document stops in front of it.

## 1. The defect

The station's two-rate driver runs a whole master day of plant steps, then a whole day of
cabin steps (`rust/crates/station/src/driver.rs`, `advance_one_master_day`):

```
today:     P P P P  c c c ... c        (4 plant quarter-days, then 1440 one-minute cabin steps)
proposed:  P c..c   P c..c   P c..c   P c..c     (360 cabin steps after each plant step)
```

So all four of the crop's quarter-day steps draw on the cabin CO₂ as it stood at dawn. The crew's
exhalation for that day arrives only afterwards. The scrubber holds the cabin pool at an amount
(about 3.8 mol C) that is a small fraction of what the crew exhales in a day, so a crop sized to
the crew is rationed by the schedule, not by the air.

## 2. What is built

* **A second day order, lab-only, in `driver.rs`.** One function runs a master day in either
  order: slow-first (today's) or interleaved. It reports rationing split by side (plant and
  cabin), because the reference driver sums the two into one number and the crew-loop record
  showed that sum cannot say which side rationed. An observer sees each step's before and after
  state, tagged plant, cabin or re-sow, so the lab can keep books per side.
* **The reference function is not edited.** No runner, session or bridge calls the new one.
* **Refusals.** The interleaved order requires the cabin's steps per day to divide evenly by the
  plant's; it refuses otherwise. Both orders keep the reference's two day-length guards.
* **The lab example `station/examples/intraday_exchange.rs`** does the measuring (slices 2, 3, 5).

## 3. Controls — each must hold before any measurement is read

1. **The slow-first path is the reference, bit for bit.** Same final state (hex-float snapshot),
   same rationing total, same events, on the greenhouse (7 days) and on the sealed station across a
   re-sow. Otherwise the lab is measuring its own copy.
2. **No plants, no change.** Interleaved with an empty plant registry must match the reference
   bit for bit. This measures the step-counter risk (the cabin now sees the counter move four
   times a day) instead of assuming it away. Also checked by reading: no cabin-side flow reads the
   step count, and every cabin-side forcing in the four plant-bearing scenarios is a constant.
3. **The interleaved order is not a no-op.** On the greenhouse it must differ from the reference.
4. **The area-scaling transform, rebuilt in Rust.** Scaling the ground area and every extensive
   starting amount (air included) by A on the standalone sealed chamber must reproduce A times the
   unscaled run, every stock at every step, to round-off. The crew-loop record measured this in
   Python (1.7e-14). If it fails here, that is a finding and slice 3 stops until it is understood.
   For the station runs the **air is not scaled** (the crew-loop record, finding 8(c)): the air
   fields are `chamber_air_capacity_mol`, `chamber_co2_mol0`, `chamber_o2_mol0`, `water_vapor0`
   and `condensate0`.
5. **The rationing probe names what the integrator counted.** For each plant step, the lab
   re-evaluates the plant flows and computes the backstop's own scale factors. The probe's count
   must equal the driver's plant-side count, step by step.
6. **The carbon books close.** Over a run, the cabin CO₂ pool's change must equal the plant-side
   change plus the cabin-side change plus any re-sow change, to round-off. The crew's exhalation,
   derived as the cabin-side change plus what the scrubber removed, must match the respired
   fraction times the food eaten.

## 4. Measurements

* **Slice 2 — the frozen roster.** The four scenarios that run on the two-rate driver and carry a
  plant: `greenhouse`, `lighting`, `harvest` (7 days each) and `sealed_station` (4 years). For
  each, every stock at the end of the run, reference order against interleaved.
* **Slice 3 — the crew-sized crops.** The sealed station at **14.09 m²** (one crew member's crop
  by BVAD) and **187.45 m²** (the station's whole crew). Sized as the crew-loop record did it:
  ground area and every extensive plant and soil amount × A, not the air; lamp × A, solar × A and
  battery × A; one season; harvest on. Both drivers, **re-measured today** — the record's
  numbers came from deleted Python, before the quarter-day step and the humidity setting.
  Reported per driver: plant-side and cabin-side rationing, the stock that forces each plant-side
  firing, the worst single-step draw per stock, peak leaf area, and the season's crop carbon.
* **Slice 5 — the closure readout.** Per run: what the crew exhaled, what the scrubber removed,
  what the plant side drew from the air net, and what harvest returned to the food store. The
  closure share is the plant side's net draw over the crew's exhalation.

## 5. Predictions — written 2026-09-29, before any code ran

**Which frozen results can move if interleaving is adopted:**

* **Correction to the review proposal.** It lists five goldens that would move. Two cannot:
  * `lighting` — the lamp and the crop share no stock; the only link is the lamp schedule, a
    forcing. Interleaving must leave it **bit-identical**.
  * `sealed_energy_drift` — it has no plant and runs on the single-rate driver, not this one.
* **Sixteen of the twenty goldens never touch the two-rate driver** and cannot move: the seven
  biosphere ones, the five sibling-domain ones (`crew`, `eclss`, `power`,
  `power_self_discharge`, `thermal`), and `cabin_gas`, `water_recovery`, `station`,
  `sealed_energy_drift`. Only `greenhouse`, `lighting`, `harvest` and `sealed_station` do.
* So adoption would move **three** goldens: `greenhouse`, `harvest`, `sealed_station`.

**How much they move:**

* `greenhouse` and `harvest` are 7-day runs of a seedling. The plant draws little, so plant stocks
  move by less than 1 part in 1,000. The cabin pool ends each day back near the scrubber's level
  in both orders, so cabin stocks move by less than that.
* `sealed_station` (1 m², 4 years): **the crop gets more carbon, not less.** Today its lit
  quarter-days after the first see a pool the earlier ones have drawn down; interleaved, each sees
  a refilled pool. Predicted size: season crop carbon up by **0.3 % to 5 %**. Cabin-side stocks
  (battery, heat, water) move by less than 1 part in 10,000.

**The crew-sized crops (slice 3):**

* The lamp is on 16 hours centred on midday, so the four quarter-days get 2, 6, 6 and 2 hours of
  light. The two middle ones carry 6/16 of the day's photosynthesis each.
* Scaling the crew-loop record's per-area daily **net** gain (its demand figure), the heaviest
  quarter-day asks for about **0.84×** the refilled pool at 14.09 m² and **11×** at 187.45 m².
  Net gain is a lower bound on what the crop draws, so both are floors.
* The scrubber's time constant is about 17 minutes against a 6-hour window, so under
  interleaving each plant step sees the pool fully re-settled.
* **14.09 m²:** reference order rations on most lit days. Interleaved, rationing **falls by at
  least 90 %**. Whether it reaches zero is not predicted — 0.84× is a floor, not an estimate.
* **187.45 m²:** reference order rations; interleaved still rations on most lit days, because
  11× cannot be closed by a factor-of-four refill. The stock forcing it is the cabin CO₂ pool.
* **Cabin-side rationing is 0** in every run, both orders (the power is sized with the lamp).
* **Closure share:** 1 m², well under 1 % in both orders. At 187.45 m², interleaving raises it by
  a factor between 2 and 4 — the crop gets up to four draws a day instead of one.

**The step-counter risk:** control 2 passes, and so does the no-plant run.

## 6. Grading

*(Filled in after the run.)*

## 7. The decision this stops in front of

**Adopt interleaving into the reference, or keep it lab-only?** Adoption is a station unfreeze:
the ceremony, the reference function replaced (not kept as a switch), the session switched in the
same commit, and the moved goldens regenerated.
