## **Authoring can reach plants** (Step 7 of the 2026-09-29 review — nothing ran the authored scenarios, and the authoring blocks hold no plant flow)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED 2026-10-18** on the user's *"1 and 2"* — Steps 4 and 7 of the review proposal written
2026-09-29. This is Step 7. Plan and record: `docs/plans/post-roadmap-authored-plants.md`.

**7a BUILT.** `rust/crates/authoring/tests/authored_content.rs` enumerates `scenarios/*.yaml` on disk
and holds each to the platform's promise for an authored file: it builds, runs its full horizon under
`run_scenario` (conservation every step, rationing and reversal are failures), and two runs end
bit-identical. Not a golden; nothing is pinned. ~7 s for both files' full years.

**Its first run found a broken habitat, not a stale sentence.** `bioregenerative_station.yaml` was
written against the design `o2_setpoint` of 10.0 mol with an 8-mol cabin. The 2026-09-06 citation moved
the setpoint to 1995.0 and nothing ran the file: for six weeks the regulator drained the 500-mol tank into
the cabin every hour — **122,640 backstop firings a year**, every step conserving. Fixed by sizing, not
tuning: the regulator's offset (0.23725 mol) is set by the crew's draw and the rate constant, not the
setpoint, so the cabin now starts at 1993.0 and every other book runs as written. Re-measured on the fixed
file: rationed 0, O₂ 1994.76275 from hour 2, tank 500 → 148.198 (the file's stated 148.2), biomass and
feed at their stated 5× and 4.5×, the thermal node at its stated Q_eq. `algae_habitat.yaml` was sound and
ends at its stated fixed point. A second false sentence came out: "every ECLSS value is a DESIGN
placeholder", falsified by two citations.

**7b DESIGNED, not built** — three questions for the user (the plants' counters and re-sow, which of a
bed's stocks an author may connect, how the two step sizes are declared), and the parked O₂-setpoint
authoring decision, which is what broke the habitat above.

* **Lesson: the unvalidated half breaks silently.** A frozen value move is a ceremony on the reference
  side and a free fall on the authored side. A test that only RUNS the content — no values — caught a
  six-week-old failure on its first run.
* **Lesson: run before you read.** The plan expected one false sentence; reading would have fixed the
  sentence and left a habitat that empties its oxygen tank 14 times an hour.
