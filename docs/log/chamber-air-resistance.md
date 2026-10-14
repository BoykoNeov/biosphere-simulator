## **Pricing a chamber air resistance** (opened from Step 6's water-gap search — a canopy air resistance of ~9 s/m instead of the uncited 50 would close the gap alone, on the trial's 20 m² basis)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED AND PRICED 2026-10-14** on the user's call (*"2"*: price changing the air-resistance value),
after Step 6's water gap (`post-roadmap-real-world-checks.md` §11–§12). Plan:
`docs/plans/post-roadmap-chamber-air-resistance.md`. **Nothing adopted; main keeps 50 everywhere.**

* **The user's choices:** a spread of values, 50 → 30 → 15 → 9 s/m, over the fitted 9 alone; the open
  field keeps 50; priced on two scopes — the lamp-lit chambers (2 goldens) and every sealed chamber
  (9 goldens; the lamp-lit ones are sealed too). No value below 50 has a source.
* **Checked first:** `ra` reaches the model in one place, the crop's transpiration; the station has no
  latent heat; the station builders read params inside `build_season`, so a scope must be carried by the
  scenario — `ra` is a property of the room.
* **Measured on a local branch** (`wip/chamber-ra-price`, never pushed; an environment-read override,
  a pricing device and not an adoption shape): every cell moves exactly the predicted goldens, and
  **only water** — not one carbon, nitrogen, oxygen, energy or phenology leaf, even at 9. The watering
  keeps the soil above every stress threshold.
* **Refuted:** "the end-state change grows with the value". The multi-year sealed runs end at different
  points of a watering cycle, so their end states swing in sign (the sealed station's soil +36 %, +18 %,
  −17 %). They show *that* water moves, not *how much*; the size is the arithmetic's ×1.16–1.73.
* **One behaviour red** at 9: the perennial chamber's below-root store settles in two re-sow cycles
  instead of one (still a cycle, not a ratchet). Two reds belong to the branch device; the rest are the
  goldens and their bands, as predicted.
* **The value is the open question.** 9 matches TM 102788 only on its 20 m² basis; on the 16 m² of trays
  the match needs ~1.5, which no chamber plausibly has, so matching the trial is not evidence for the
  value. Adopting any value needs a source or the user setting aside the WHAT-IF rule; the TM scorecard
  (an open build) sits outside both scopes.

**Lesson:** an end state of a run with a watering cycle is a phase reading — size a water change by
its throughput, not by a snapshot.
