## **Which uncited numbers matter** (Step 5 of the 2026-09-29 review — "60 citation-needed markers" was a list nobody could rank, so no one knew which unknown to chase)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**BUILT 2026-10-19** on the user's *"1, 2 and 3 — do them"* (item 3). A measurement: no value, golden,
manifest or `source:` string moved. Plan and full tables: `docs/plans/post-roadmap-uncited-triage.md`.

* **The roster is 45 values, not 60 markers.** 60 counted the text `TODO(cite)`; it missed every
  `DESIGN` value. 40 measured; the chamber's five `DESIGN` values cannot be handed to its builder.
* **Two lab examples**, `domains/examples/uncited_triage.rs` and `station/examples/uncited_switch.rs`:
  each value at ±10 % (±1 °C for temperatures), and again at the range its own `source:` records.
* **The five worth chasing:** the scrubber rate (at the real cadence the chamber ends with 8–30× the
  CO₂ — a form decision, not a citation); maintenance respiration (its recorded range crosses the yield
  tripwire); growth efficiency and quantum yield (+10 % crosses it); the leaf death rate (its own
  cited table's value crosses it).
* **Safe to stop chasing:** two values no frozen build reads (`surface_resistance`, `recycling_rate`);
  the hot-end temperatures (never reached); nitrogen uptake (inert even at its recorded 6× error);
  water recovery (÷1000 costs the crew 0.23 % of its water).
* **Found:** the fixed-point gate's `source` string quotes 0.578137; the reading is 1.029832 since the
  leaf-shedding unfreeze. A ceremony to correct, not done here.
* **Lesson: ±10 % ranks sensitivity, not error.** Nitrogen uptake and the scrubber both read as quiet
  at ±10 %; only a run at the recorded error told the inert one from the one that moves CO₂ 30×.
* **Lesson: a value that moves nothing has two causes — unread, or unreached.** Read the code before
  calling it inert: two of the "zero" rows are dead parameters, and one was measured on a build that
  never switches its flow on.
