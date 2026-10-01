## **The crop's light follows the power the lamp actually gets** (the 2026-09-29 review's Step 3, slice 3a — stopped before code: today only the emergency rationing can short the lamp)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED 2026-10-01 on the user's *"work on step 3"*, STOPPED BEFORE CODE.** Plan, findings
and predictions: `docs/plans/post-roadmap-light-from-delivered-power.md`.

* The lamp gets less than nameplate only when `min_scaling` fires: a numerical guard that
  golden runs assert never fires, and that cuts life support and the lamp by the same share.
* No frozen run shorts the lamp; `sealed_station`'s battery falls 2.0e10 → 5.95e9 J over the
  run (not balanced; not investigated). No brownout has ever run against a lit crop.
* A station-side wrapper can dim `par` with no `simcore` or biosphere edit; it moves the
  station flow-set rows, so any form of 3a is a station unfreeze. The lag is 1/16 day.
* Waiting on the user: A (the guard dims the lamp), B (a lamp dimming rule; no source on our
  shelf, so WHAT-IF unless one is found), or skip to 3b. Nothing frozen moved.
