## **The step unfreeze** — the biosphere moves to `dt = ¼ day` (the step decision, taken and BUILT)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per
> work item. The fuller record, with every table and every prediction scored, is
> [`../plans/post-roadmap-step-unfreeze.md`](../plans/post-roadmap-step-unfreeze.md).

**BUILT 2026-08-14 on the user's call ("quarter the step").** The successor to the step
sweep, which measured the axis and left the verdict open. This is the first unfreeze of a
**numerics** item rather than a science one, and the first to move the biosphere and station
contracts together. 13 goldens, both manifests, the native port and the Godot consumer.

**Why.** At `dt = 1` the sealed chamber's season-low CO₂ was **57.9 ppm** and the crop kept
fixing carbon there — below the **61.07 ppm** compensation point its own FvCB kinetics make a
hard shutoff. Truncation, not biology. At `¼`: **76.82 ppm**.

### The finding that shaped the whole job: the step was small, the UNIT was the work

Moving `BIO_DT` is one line. What cost the effort is that every run length, reset period and
perturbation window was written in **days** and used as **steps** — and *no test at `dt = 1`
can tell a correct conversion from a wrong one, because the two are the same integer*. The
suite is blind to this class by construction.

So it landed in two commits, and **the split is the reusable part**: the routing first, proved
**byte-identical** on the full suite with no golden regenerated, then the step — so every byte
of the golden diff is attributable to the step alone. Correctness *in the new unit* was
established by a separate discriminating probe (flip the constants uncommitted, run, read the
failure **kind**, revert), never by the suite.

⚠ **"Provably inert" and "provably correct" were different claims here, and only one of them
was cheap.** Commit 1's green suite proved inertness and said *nothing* about whether each
conversion had been handed a genuine day count. Any future change whose correctness condition
is invisible at the shipped configuration needs its own discriminating probe; a green suite is
not evidence.

### Finding a unit bug: three passes, each failing differently — and a fourth found later

1. **By variable name** — missed every constant not named year-ish.
2. **Name-blind, by use site** (arithmetic trajectory subscripts, modulo against counters) —
   found two more, still not all.
3. **By CALLER of the function whose parameter carries the unit** — found the last one, whose
   constant is called `year_len` and whose third use inlines the day count directly into the
   call with no local to grep for. **A name can be anything; the callee cannot.**

⚠ **And that rule, stated once, was still applied too narrowly.** Method 3 was run against
`year_summaries` and no other callee, so it missed `with_station_leak`'s window in
`test_station_perturbations.py` — which at `¼` reads as **days 0.5–1.75 instead of 2–7**. The
Rust port caught it. **"Enumerate by callee" has to be repeated for EVERY callee carrying a
unit**, not just the one that produced the lesson.

### The asymmetry that nearly caused a new bug

`run_master_day` appends one state per master **day**, so station trajectories are
**day-indexed** while biosphere trajectories are **step-indexed**. Three station sites looked
identical to the broken ones and were already correct; converting them would have introduced
the very bug being removed. They carry DO-NOT-CONVERT comments now. *The same defect points
both ways, and the direction is a property of the producer, not of the call site.*

### Predicting the diff: the `n` pin did the work

Written before any constant moved (plan §4b), scored after (§4c). The sharpest check was the
cheapest: every state golden pins the integer step counter, `n` counts *slow steps*, so the
flip multiplies it by exactly 4 — no science, no judgement. **12 of 12 exact.** Also: the 12
biosphere-free goldens did not move; `git diff --stat` came back **279 insertions and 279
deletions**, an independent whole-tree proof that no array changed length; and in the lighting
seam every biosphere stock moved while `power.battery`, `boundary.waste_heat` and
`boundary.light_used` came back **bit-identical** — testing for the first time a docstring
claim that nothing had tested.

⚠ **A structural prediction is worth more than a value prediction.** The value rows needed
judgement and one came in 1 ppm off; the `n` rows were exact by construction and would have
caught any missed conversion instantly.

### Two guards that were correct BY ACCIDENT

* `sealed_reset`'s `n % season_days` with `n = 4·day` still fires on the right days **only
  because 305 is odd**. At `season_days = 304` it would re-sow four times a year.
* `test_perturbations.py` carried a surviving double conversion and a bare window literal.

*Correctness resting on an unwritten coprimality is not correctness.* Both converted
deliberately.

### A test kept by measuring rather than loosening

`o2_leak_is_absorbed_by_makeup_effort` pinned "the plant is untouched" as an absolute
`rel_tol = 1e-6`, and failed at **1.55e-5 in both ports identically** — which is itself the
evidence that it is the reference's behaviour, not a port defect. The cause is real: four slow
steps now run before any fast makeup, so the plant sees the intra-day O₂ drawdown at four
levels instead of one.

Loosening the tolerance would have been weakening a test to make it pass. Measured the
contrast instead — CARBON moves biomass **16.6 %**, O₂ **0.0015 %**, a factor of **10715** —
and asserted *that*. ⚠ **Strictly stronger than what it replaced:** the old absolute form also
passed if the carbon leak did nothing at all. *An absolute tolerance standing in for a
contrast is a pin waiting to break on the first scale change; write the contrast.*

### What the contracts did and did not catch

* **Biosphere: caught it, loudly and correctly.** `dt_days` is asserted against a hard-coded
  literal, and there is a *second* hand-maintained literal in the manifest generator. Missing
  the second one turned the assertion red — the design working. ⚠ Keep both as literals:
  comparing against `BIO_DT` would make the contract auto-follow the code, which is the
  opposite of a freeze.
* **Station: caught nothing.** Its step lives only in `numerics_note` prose, and that string
  is a literal inside the *generator*, compared against a manifest generated from it — so the
  two agree whatever the code does. ⚠ **Do not assume the loud gate on the biosphere side
  covers the station side.** Maintained by hand, deliberately.

### The gap that let the defect ship, which outlives the fix

It was never a guard failure: arbitration rationed zero times, every golden gate was green,
every `science_band` passed. **A band of the right shape — *"the sealed chamber's season-low
CO₂ stays above the compensation point"* — would have caught it on day one.** It did not exist
because the bands describe what the model *does* rather than cross-checking it against its own
kinetics. That band is now writable and is the natural successor — deliberately **not**
bundled here, because a band written in the same change that makes it pass is a restatement of
the run, not a contract on it.

### Open, and deliberately not closed here

* ⚠ **`step.py`'s convergence sequence and RK4 limit are STALE.** The measured 76.82 sits
  *above* the sweep's quoted limit of 76.29, which a finite-step Euler run should not — the
  sweep's limit was measured on a tree that has since gained stem reserves, soil layers and
  root coupling. Re-measuring is a refinement study, not part of this ceremony. **Do not
  re-quote those numbers as current.**
* `Γ*` is still a `TODO(cite)` entry, so the threshold cleared is provisional.
* The parked leaf-expansion merge stays excluded: its whole evidence base was measured at
  `dt = 1` and must be re-measured before the question is put again.
