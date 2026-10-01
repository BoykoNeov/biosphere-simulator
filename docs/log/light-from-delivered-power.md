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
* The user chose B (a lamp rule). Searched: a 1983 GE patent gives the FORM (switch the lamp,
  an interruptible load, off at a reserve); nothing gives a number, so B is lab-only, WHAT-IF.
  The ISS 35 % depth of discharge was read and rejected (a battery-life limit, needs a capacity).
* `sealed_station`'s battery drain is EXACTLY the lamp's energy (difference 0.0): the solar pays
  life support only. A shed lamp there never comes back on. Separate finding; not fixed here.
* Open with the user: switch-off with a latch, or gradual dimming. Nothing frozen moved.
