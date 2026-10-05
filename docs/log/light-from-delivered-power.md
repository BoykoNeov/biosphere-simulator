## **The crop's light follows the power the lamp actually gets** (the 2026-09-29 review's Step 3, slice 3a — built lab-only: the lamp is switched off at a reserve, and the crop feels what the lamp got)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**BUILT LAB-ONLY 2026-10-01** on the user's *"work on step 3"*, then *"Dimming rule"*, then
*"Switch off"*. Plan, findings, predictions and results:
`docs/plans/post-roadmap-light-from-delivered-power.md`.

* The lamp gets less than nameplate only when `min_scaling` fires: a numerical guard that
  golden runs assert never fires, and that cuts life support and the lamp by the same share.
* No frozen run shorts the lamp; `sealed_station`'s battery falls 2.0e10 → 5.95e9 J over the
  run (measured below: exactly the lamp's energy). No brownout has ever run against a lit crop.
* A station-side wrapper can dim `par` with no `simcore` or biosphere edit; it moves the
  station flow-set rows, so any form of 3a is a station unfreeze. The lag is 1/16 day.
* The user chose B (a lamp rule). Searched: a 1983 GE patent gives the FORM (switch the lamp,
  an interruptible load, off at a reserve); nothing gives a number, so B is lab-only, WHAT-IF.
  The ISS 35 % depth of discharge was read and rejected (a battery-life limit, needs a capacity).
* `sealed_station`'s battery drain is EXACTLY the lamp's energy (difference 0.0): the solar pays
  life support only. A shed lamp there never comes back on. Separate finding; not fixed here.
* The user chose switch-off. Built in `station::lamp_shed` (7 tests, example `lamp_shed`):
  the lamp is off below a WHAT-IF reserve (24 h of life support); the crop reads the share
  of lamp power that actually arrived over the last power group (lag 1/16 day).
* Controls bit for bit: rule off, and rule on with the frozen battery, are the plain run.
* WHAT-IF result, 1.5e8 J battery, 3-day blackout: plain crop +0.000 % with 5140 rationings;
  lab crop -31.6 % with 0. The shed lamp never comes back (solar pays life support only).
* A first version counted what the rule predicted, not what arrived; a broken lamp still
  darkened the crop. Fixed. Nothing frozen moved; no golden, no manifest row.
* After the first commit: day-end states carried a one-group-stale share, invisible to a
  resumed crop (a day starts in the lamp's dark hours). Asserted directly, red, fixed.
* Patent quotes verified against the raw page. Only `par` follows the lamp: transpiration's
  net radiation is the weather file's, even in the frozen lamp-lit station (bears on 3b).

**Forward note, 2026-10-05 — the lit detector changed under this record** (room-temperature
plan §23g–§23h, slice 2b-ii). The sealed station's lamp light now heats the plant chamber, so
`boundary.light_used` is gone from the sealed build and the detector this record describes
("a sub-step counts as lit when light arrived in `boundary.light_used`") no longer exists.
`run_shedding` now counts a sub-step lit when the lamp, evaluated on the sub-step's starting
state, draws from the battery (`lamp_shed::lamp_draws`) — before arbitration, so a lamp the
backstop cut wholly would also count as lit (the tests assert `rationed == 0` where they rely
on it). Every figure `examples/lamp_shed.rs` prints is unchanged except its last column, now
the mean delivered share; cross-check: the blackout run's old light ratio 2.9212e7 / 5.0416e7 =
0.5794 = its new share.
