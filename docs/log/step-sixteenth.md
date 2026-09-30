## **The biosphere moves to a 1/16-day step, and the crop's CO₂ uptake is taken against the air it leaves** (the 2026-09-29 review's Step 2, slice 4 — the user's "1 + 3", with the test suite held to its old run time)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**IN PROGRESS since 2026-09-30** on the user's call (*"1 + 3. But ensure the tests runtime would
not quadruple"*, then *"i want the test suite to be roughly the same time as now"*). Plan,
predictions (committed before any code) and per-slice results:
`docs/plans/post-roadmap-step-sixteenth.md`.

* Slice 0 (`99c256e`): the simulation crates are optimised in dev/test builds. The suite's run
  time went from 496 s to 110 s, byte-neutral.
* Slice 1: the biosphere step is 1/16 day (a biosphere and station unfreeze, 11 goldens). The
  whole suite runs in 254 s against 496 s before slice 0. Two user decisions on the way: the
  master day may run in equal groups (the lamp scenarios' 24 power hours over 16 plant steps),
  and the mutual-shading loss, which no frozen scenario reaches any more, is shown acting on a
  pushed run. Six measured claims moved and were restated; see the plan's slice 1 results.
* Slice 2 (C): open. This record is written out when it closes.
