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
* **The user's call:** 75 % or 40 %, and the sealed station's thermostat (live or window).
