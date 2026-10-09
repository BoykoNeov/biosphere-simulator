## **The plant chamber's dehumidifier stops drying the air below its setting** (opened from the soil-evaporation supply decision — the condenser drew vapour below its humidity setting, which a real dehumidifier does not)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED AND ADOPTED 2026-10-09** on the user's call (*"Fix the dehumidifier. Then step 1"*). A biosphere
unfreeze. Plan, predictions committed before code, and grading: `docs/plans/post-roadmap-chamber-dehumidifier.md`.

* **The fault.** The sealed chamber's condenser removed the whole excess over its humidity setting **plus**
  `condensation_rate · dt · min(v, target)` below it, a first-order draw that a dehumidifier does not make. The
  parameter file had recorded it as a flaw since 2026-09-29; the cabin's condenser was fixed for the same
  fault on 2026-10-03.
* **The fix.** The condenser removes `max(0, v − target)` and nothing else. The rate had no other job and was
  deleted: one uncited DESIGN number fewer. Not copied from the cabin's first-order draw on the excess: here the
  excess was already removed whole, and slowing it could leave the air above saturation after a cooling step.
* **What moved.** 8 goldens, water only. The five biosphere chambers moved only at rounding (≤ 5e-14): wherever a
  living crop feeds the air, the old draw was refilled from the same flux, so the two laws agree. `greenhouse` /
  `harvest`: condensate −0.25 %. `lighting` did not move (predicted to; not traced). Manifests: one param-file
  digest and 8 golden digests.
* **What it was hiding.** In the lab's never-re-sown sealed chamber (dead crop, soil evaporation on, watering in
  events) the dead-phase air against frozen rose 0.605 → **0.981** under the chosen root-zone supply (success
  test S4 now passes) and 0.176 → 0.704 under the old top-layer cap (still fails). Most of the "dry air" had been
  the dehumidifier.
* Tests restated, none weakened: the condenser's first-order unit claim retired and replaced by its opposite;
  the headroom tests pin `max(0, target − v)`; one ring-order test needed a second state above the target.
* Noticed, not caused here: the repo's memory-index gate is red since the 2026-10-08 memory-index compression
  (several links per line; the gate reads one).
