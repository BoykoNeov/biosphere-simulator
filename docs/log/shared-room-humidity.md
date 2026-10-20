## **One moisture store for a shared room** (the crew-size item's measured side effect — where crew and plants share one room, two vapour stocks held by two condensers add past saturation)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED 2026-10-20** on the user's *"3"*, then *"1"*. ⚠ It reverses their 2026-10-03 decision to keep
two stocks and record the flaw; they were told before choosing. A plan with predictions; nothing built,
no value, golden, manifest or `source:` string moved. Plan: `docs/plans/post-roadmap-shared-room-humidity.md`.

* **Measured today** (a throwaway probe outside the repo, one sealed season): the room's two vapour
  stocks sum above saturation on 2670 of 2690 warm plant steps (mean 1.21) and on all 2190 cold ones
  (mean 2.23). Two causes: the targets add (75 % + the crew's 46 %), and the crew's target assumes 22 °C
  while the room spends 45 % of the season at 4 °C, where the crew's vapour ALONE is 150 % of
  saturation. The record's 116 % predates the cold program and the four crew.
* **The form:** one vapour stock (the plants'), both condensers on one target at the temperature the
  plants' water reads, and the crew's condenser on the plants' frozen law (all of the excess, every
  minute). The ECLSS first-order law would let the plants' condenser sweep about a third of the crew's
  water into the soil. Each side keeps its own water loop; no re-plumbing.
* **Two leaks, quantified before building:** the minute's residue, 0.131 kg/d (160 kg over the
  1220-day run); and the target falling between plant steps, 5.68 kg a season measured (the cold
  switches), estimated 1–15 kg over the 7-day greenhouse and harvest runs, whose water reads the
  within-day air. Accepted as two coils in one room.
* **The setting is the user's:** 75 % (the plants' value, already in the tree; BVAD's top crew limit)
  or 40 % (the crew's nominal; drier air, more transpiration). Priced lab-only first.
* **MEASURED, lab-only** (`station/examples/shared_room_humidity.rs`; plan §7–§8). A `RoomVapour` switch on the
  greenhouse and sealed scenarios, default two stocks (the suite's goldens pass unchanged). At 75 % on the
  window thermostat the room still passed saturation 143 minutes a season (max 1.23), at the warm ↔ cold
  switches: the chamber cools inside a plant step while the target keeps the step's start. With the crew's
  condenser on the chamber's LIVE temperature: max 0.7601, never above 1, and only the minute's residue
  crosses (39.9 kg a season, as predicted). Carbon +0.0000 %; crew store −36.5 kg; transpiration −1.5 %.
* **Missed:** the leak lands in the soil and subsoil (+22 %, +19 %, small stores), not the condensate; the
  battery moved 0.04 % (bound 0.01 %). At 40 %: transpiration +112 %, yet grain peaks ROSE 1.6–3.6 % and the
  soil was not drawn down; the heater pays the evaporation (battery −9.9 %). Why grain rose is not measured.
* **1220 days** (plan §9): the live thermostat never passed saturation (max 0.7603); the plants' condenser
  caught 159.7 kg (predicted 160); every minute above "setting + one minute's breath" had the chamber
  cooling, which supports (not isolates) the cooling-inside-the-minute cause of the 0.7601 miss. The window
  thermostat passed saturation 573 minutes. **40 % drains the battery**: lowest charge 16 % of the start,
  against 51 % with two stocks, and it may still be falling at the end. Grain rose again at 40 %; still not
  explained. ⚠ Open: at 40 % the crop on the last day is 34 % / 41 % below two stocks while both 75 % cases
  are unmoved that day; a later sowing or slower growth, neither measured.
* **Corrected (plan §7, §10):** the greenhouses' "never above 0.754" read the same temperature the
  condensers enforce. Read minute by minute off the outdoor air curve, the max is 0.821, never above 1. A
  live greenhouse thermostat is buildable (an earlier line here said it was not); not built.
* **Corrected (plan §4, §11):** `sealed_energy_drift_summary` runs Power → Thermal with no room in it;
  fresh prediction: byte-identical at adoption.
* **The user's call:** *"75%, live"* (live is the sealed station's only; the greenhouses have no chamber).
* **ADOPTED 2026-10-20** (plan §12): a station unfreeze. The lab saved the three goldens' own builds at the
  chosen room, after the two-stock side reproduced each committed golden; the regenerated `greenhouse`,
  `harvest` and `sealed_station` are `cmp`-identical to those files, the other 17 identical (the energy-drift
  golden included, as predicted). Manifest: 3 hashes and `flow_set` + `Condensation`, as predicted. Moved:
  crew condensate −3.1 kg in the greenhouses, the sealed crew store −146 kg; no carbon bit in the
  greenhouses. Not predicted: the sealed top soil layer on the last day, 8.8 → 28.5 kg (not traced).
  Separate air builds two stocks itself; the plant-step tests and `air_split.rs`'s shared baseline keep two
  stocks (a plant-step build cannot hold one); two node pins re-pinned as measured, bands unchanged (the
  structure walls 188.70 → 188.59 K; the 1220-day run's mean 170.06 → 170.03 K, found by `--ignored`).
* **The decision summary left something out.** It said that at 75 % "nothing else moved"; a side note
  pointed out the ~40 kg a season of crew breath the plants' condenser catches, which ends in the plants'
  soil. The user was told, and asked to keep it, remove it or undo.
* **A pipe, built and reverted unread:** the user first chose "send the plants' collected water to the
  crew's recovery" (the plants' condenser booked to the recycler); before any result, the user asked
  instead for *"a system that balanced water needs"* with *"a tank with xtra water to serve as a buffer"*.
* **The user's design (plan §13):** the crew's 20 t tank is the station's reserve; the plants' tank keeps
  one full watering and drains the rest through the crew's recycler; a short watering is topped up from
  the crew's tank. The level was first written from the soil's depth (119 kg) and amended to the deepest
  roots' (103.3 kg) after the probe: no watering can be bigger.
* **The probe before building:** on the reference the top-up would never fire (the tank always held
  enough) and the plants' water climbed 18.3 kg a cycle. **Measured, lab-only:** 143.7 kg returned, the
  plants' water flat at 208.9 kg, top-up 4.3 kg, carbon unmoved; missed: the level it settles at (the
  drain starts in the first cycle, not the second).
* **ADOPTED 2026-10-21** (plan §14): only `sealed_station` moved, `cmp`-identical to the lab's save;
  `flow_set` + `TankDrain`. ⚠ `ReserveTopUp` runs on the plant step, in the slow registry the station
  manifest excludes, so no freeze record lists it (predicted to appear: half-missed). Found at the flip, not
  predicted: a dead cooler's hot room keeps being watered from the reserve and never stops evaporating
  (140 kg over 60 days); `chamber_heat.rs`'s node bound pins `Off`, and that behaviour has its own test.
  `air_split.rs`'s two-room baseline pins `Off` too.
