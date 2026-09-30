## **Plants and cabin trade gas within the day: ADOPTED** (a station unfreeze moving three goldens, each one the lab's interleaved end state to the last printed digit)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**BUILT 2026-09-30**, on the user's *"1. adopt"*. The measurement that argued for it is the
previous item, `log/intraday-gas-exchange.md`. Plan, predictions (committed before any code,
`de23392`) and grading: `docs/plans/post-roadmap-intraday-gas-exchange.md` §8. Unfreeze entry:
`docs/station-reference.md`, 2026-09-30.

## What changed

* **The reference master day is interleaved.** `station::driver::advance_one_master_day` runs
  one plant quarter-day, then that quarter's 360 cabin minutes, four times. It ran all four
  plant steps, then all 1440 minutes. The runners, the session and the Godot bridge all call
  that one function, so none of them needed an edit of its own.
* **A new refusal**, `driver::fast_steps_per_slow_step`: a cabin-step count that does not divide
  evenly by the plant-step count is refused in the per-day function, `run_master_day` and
  `SimSession::two_rate`. Before, nothing needed the split; now an integer division would drop
  the remainder's cabin time without a word. No shipped scenario trips it, and authored
  scenarios cannot reach this driver.
* **The retired order stays in the lab** (`DayOrder::SlowFirst`). The plan's §7 said to delete
  the lab section on adoption. On review that would have orphaned the example that reproduces
  every finding of the previous item. §7's objection was to a switch *in the reference*, and
  nothing in the reference can reach the lab section.

## Results

* **All six predictions held.** 3 goldens of 20 changed. Each matched the lab's interleaved end
  state to every printed digit: harvest's grain store +37.53 %, humus +19.05 %; sealed
  station's crop carbon +1.23 % over 4 years; greenhouse ≤ 4.7e-4. The manifest moved by
  exactly 3 hashes plus the hand-written `numerics_note`, and no `simcore` byte changed.
* **The bit-identity controls were turned round, not dropped.** They now hold the lab's
  *interleaved* arm to the reference on the greenhouse, the harvest ring and across a re-sow.
  The lab example's full-length control (the 4-year sealed station with its real re-sow) does
  the same.
* **The Tier-2 band basis moved and still holds**: greenhouse ±1-ULP sensitivity went from
  2.76e-16 to 3.49e-16, against a band of 1e-11.
* **Gates**: the full suite (74 test binaries, 0 failed), the ignored long runs, and clippy
  `-D warnings` all pass.

## Findings

**1. Prose had named the old order in ten places, and nothing would have caught any of
them.** They were the driver, the session, the greenhouse, the palette, the bridge's time
control, a Godot smoke script, the ULP probe, the native-port doc, the tier evidence and the
station contract. The Tier-2 argument ("the regulators hold the pools at setpoint between the
**once-daily** biosphere lumps") had been stale since the quarter-day step made the lumps four
back-to-back steps. It is now literally true for the first time: the lumps are quarter-day ones
with the regulators running between each. One session comment still called `n` "the day
count", which is wrong since `dt = ¼`. Found only by searching for every way the old order
could be described.

**2. The manifest's `numerics_note` did not say the order at all.** It is the contract's only
record of how the station steps, and it would have read the same before and after. It now
names the order. It is still hand-maintained prose that nothing checks, and that caveat on its
own row still stands.

**3. The review's golden count was wrong on arrival, and the plan's deletion instruction was
wrong on arrival.** Neither was an error in the reasoning. Each was written before the
measurement it depended on, which is why both were re-checked against the tree rather than
followed.

## Named on the way out

* **The whole crew's crop is still starved**: at 187.45 m², the heaviest step asks 7.26× the
  scrubbed pool. That is the review's Step 2 (small stores against a big step), whose first
  slice is a measurement.
* **The user wants Step 2's option C built regardless of which option wins**: plants take less
  CO₂ as it runs low. It is a change of form, so it needs a published source, or it is
  WHAT-IF and lab-only. Recorded under Step 2 of the 2026-09-29 review proposal (a
  forward-looking plan, deliberately not named here by file — see the log's exemption note).
