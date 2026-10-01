## **A sealed chamber's crop transpires against the chamber's own air** (the 2026-09-29 review's Step 3, slice 3b — the crop's water loss reads the chamber's humidity, not the weather's)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**FROZEN 2026-10-01** — on the user's *"work on step 3"*, after 3a, then *"freeze it"*. Plan,
form, predictions (committed before any code and again before the write) and results:
`docs/plans/post-roadmap-chamber-dryness.md`.

* Form: the chamber's vapour pressure from its vapour store, by the room-gas identity the model
  already uses; no new number. Sealed builds only; the open field keeps the weather's figure.
* Control held: with the weather reading, 20 of 20 goldens are identical. The weather reading is
  kept, lab-only (`lab::biosphere_with_vpd_read`).
* Exactly the 9 sealed goldens moved, and only their water stocks; the written bytes match the
  measurement run's copies. Potential transpiration +20.55 %; the stress factor stays exactly 1
  at every step. 9 `golden_sha256` rows moved (biosphere 5, station 4) and nothing else in any
  manifest; `tiers.json` and `simcore` untouched.
* Missed: `drift_summary` did not move (predicted it would; it folds carbon only, and no carbon
  moved). The re-sow test's first-year transient went 3.39e-4 -> 1.203e-3, over its 1e-3 bound,
  while the cycle still settles to 1e-12; the user raised the bound to 4e-3 (~3x, the rule that
  set 1e-3).
* Reach: authoring cannot build a transpiration flow, so authored habitats do not move; the
  Godot palette's `greenhouse` and `sealed` sessions read the chamber's air from the freeze on.
