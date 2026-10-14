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
* **Refuted:** "the end-state change grows with the value" — the multi-year end states swing in sign
  (the sealed station's soil +36 %, +18 %, −17 %). Their first-written cause, "a snapshot inside a watering
  cycle", was untested; measured on run means it holds for the re-sown chambers only. The lamp-lit station's
  water moves steadily out of the condensate (−9 / −16 / −39 % on average) into the deep store (+6 / +11 /
  +25 %), cause not established; the never-re-sown chamber's means swing too.
* **The size, measured** (root-zone outflow over each run): +20 / +54 / +77 % through the lamp-lit station,
  +4 to +19 % through the sunlit sealed chambers, whose crop is small or dead much of the time.
* **What "water only" cannot see:** the model gives evaporation no heat and the dehumidifier no power
  for what it condenses, so the energy side of this price is unmeasured, not zero.
* **One behaviour red** at 9: the perennial chamber's below-root store settles in two re-sow cycles
  instead of one (still a cycle, not a ratchet). Two reds belong to the branch device; the rest are the
  goldens and their bands, as predicted.
* **The value is the open question.** 9 matches TM 102788 only on its 20 m² basis; on the 16 m² of trays
  the match needs ~1.5, which no chamber plausibly has, so matching the trial is not evidence for the
  value. Adopting any value needs a source or the user setting aside the WHAT-IF rule; the TM scorecard
  (an open build) sits outside both scopes.

**Lesson:** an end-state change is not the size of a change, and a cause written for it is a claim until a
run tests it — the "watering cycle" explanation held for two builds and failed for two.

**CLOSED 2026-10-14 on the user's call (*"use 50"*): nothing adopted.** Every room keeps the uncited 50; the
price stands as the record. Keeping 50 decides the model, not the trial's water gap, which stays open
(`post-roadmap-real-world-checks.md` §13).
