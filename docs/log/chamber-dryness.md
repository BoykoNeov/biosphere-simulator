## **A sealed chamber's crop transpires against the chamber's own air** (the 2026-09-29 review's Step 3, slice 3b — the crop's water loss reads the chamber's humidity, not the weather's)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**BUILT AND MEASURED 2026-10-01, NOT FROZEN** — on the user's *"work on step 3"*, after 3a. Plan,
form, predictions (committed before any code) and results: `docs/plans/post-roadmap-chamber-dryness.md`.

* Form: the chamber's vapour pressure from its vapour store, by the room-gas identity the model
  already uses; no new number. Sealed builds only; the open field keeps the weather's figure.
* Control held: with the weather reading, 20 of 20 goldens are identical.
* Under the chamber reading, exactly the 9 sealed goldens move, and only their water stocks.
  Potential transpiration +20.55 %; the stress factor stays exactly 1 at every step.
* Missed: `drift_summary` did not move (predicted it would); and the re-sow test's transient
  bound (1e-3) is crossed, 3.39e-4 -> 1.203e-3, while the cycle still settles to 1e-12.
* The loader still reads the weather; freezing is the user's call, with that test's bound.
