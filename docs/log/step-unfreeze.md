## **The step unfreeze** — `dt = 1` → `dt = ¼` (the sweep's decision, taken; the unit was the work, not the step)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per
> work item. The fuller record, with every table and every prediction scored, is
> [`../plans/post-roadmap-step-unfreeze.md`](../plans/post-roadmap-step-unfreeze.md).

**BUILT 2026-08-14 on the user's call ("quarter the step").** The successor to the step
sweep, which measured the axis and left the verdict open. This is the first unfreeze of a
**numerics** item rather than a science one, and the first to move the biosphere and station
contracts together. 13 goldens, both manifests, the native port and the Godot consumer.

**Why.** At `dt = 1` the **perennial** chamber's season-low CO₂ was **56.03 ppm** and the crop
kept fixing carbon there — below the **61.07 ppm** compensation point its own FvCB kinetics
make a hard shutoff. Truncation, not biology. At `¼`: **75.48 ppm**.

⚠ **CORRECTED 2026-08-14.** This read *"the sealed chamber's ... 57.9 ppm ... 76.82 ppm"*.
Both numbers are real and neither belongs to the other: `season.run_perennial` re-sows
**unconditionally** and the step sweep drove every scenario through it, while the sealed
chamber's golden uses plain `run_season`. In its own configuration the sealed chamber reads
**75.75 ppm at `dt = 1` — it never crossed**. The perennial and consumer chambers do re-sow,
their goldens run that way, and their crossing is real, so **the step move stands and only the
locus was wrong**. Corroborated two ways: the sweep's table reproduces cell for cell on today's
tree, and today's `dt = 1` `run_season` reproduces the pre-unfreeze committed golden
bit-exactly. Full detail in `src/domains/biosphere/step.py` and `docs/biosphere-reference.md`.

⚠ **The generalisable part is not the arithmetic.** Every write-up of this ceremony repeated
one scenario name, and the error travelled from the diagnosis into the *recommended fix* — the
successor band proposed below was written as *"the sealed chamber's season-low stays above the
compensation point"*, i.e. aimed at the only scenario that never had the problem, where it
would have passed on day one and caught nothing. **A guard inherits the locus of the diagnosis
that motivated it.** Also: a headline pair of before/after numbers is worth checking came from
the same run before it is quoted a fourth time — nothing here was arithmetically wrong, and
the pairing was wrong for three days across five files.

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

### The re-pinning pass, and the two things it found that the flip alone did not

~50 value pins across four test files went red at the new step. Most were arithmetic. Two
were not, and both are about *what a number in this suite means* rather than what it equals.

**1. A census margin is denominated in STEPS, not in time.** `test_acceptance_gate.py`
measures, per stock, `stock / demand-per-CALL` — the arbitration backstop's own `scale_s`.
Quartering the step quartered every per-call demand and so multiplied every biosphere margin
by ~4, while nothing physical moved. The evidence that the flip landed exactly where it was
aimed is the *split*: **nine of the nineteen census rows did not move at all** — power,
thermal, ECLSS and crew all keep their own steps.

The consequence is that several bounds in that file were claims about the integrator wearing
the clothes of claims about the world. `9.0 < soil_water margin < 10.0` is now
`9.0 < margin · BIO_DT < 10.0` and reads in **days**: *the root zone covers 9.31 days of peak
draw*, a number unchanged to sixteen digits across the step change. Four `1/(k·dt)` pins are
written as the formula rather than as the number it evaluates to — the same shape as the
`_is_one_over_k_dt` bug, where the `dt` was in the test's name and missing from its
arithmetic.

**2. ⚠⚠ The sealed station no longer has a plant-limited acceptance gate.** `sealed_station`
is the one sealed scenario that also runs a cabin, and the two registries draw on the same
`carbon_pool`. Its binding call was the **plant's** at 5.0232, tighter than the ECLSS
scrubber's `1/(k·dt) = 16.667`. A quarter-step quartered the plant's per-call draw, so it rose
to **19.0209** and crossed a cabin constant that did not move at all. Same stock, different
registry.

So the census's headline finding — *"the six smallest live gate margins in the roster are
`biosphere.carbon_pool`, in the six scenarios that seal a chamber"* — is now **five**, and the
station's row has fallen from 6th to 12th, behind four `power.battery` entries. Read plainly:
**the frozen contract's `rationed == 0` has stopped answering a question about the plant on
the station.** Nothing about the plant changed; the same trajectory, integrated more finely,
is simply never within a quarter-day of out-running the pool. That makes the acceptance test
weaker there than it was, and it is recorded rather than repaired — repairing it would mean
choosing a step to keep a gate, which is backwards.

**...and the pin that should have caught it had been reading one registry twice.** The helper
that splits a scenario's calls by registry did it by *frequency* — "the most frequent
signature" versus "all the rest" — on the reasoning that the fast registry supplies almost all
the calls. True, but a fast registry has **phases** too: before a crew store comes online its
demand set is a strict subset, so `greenhouse` has one such cabin call and `harvest` three,
and those sat in "all the rest" alongside the biosphere's. Since a cabin call is far tighter
there, `min()` returned the cabin's number *as the biosphere's*, and a docstring said the two
registries "happen to coincide (16.667 either way)".

They do not, and — measured, not inferred — they never did. Checking out the pre-unfreeze tree
and applying the corrected split gives **308.16** for `greenhouse` and **376.41** for
`harvest`, against the cabin's 16.667; the old helper returns `(16.667, 16.667)` there too.
⚠ The inference (divide the new 1150.75 by four) would have reached the right verdict here for
a reason that is not always available — the two columns are *not* a clean 4× — so it was run
rather than argued. The split is now by what a call **demands**, with a test asserting the
discriminator against the driver's own call-count ratio.

⚠ **The generalisable pair:** *a cross-domain ranking can be re-ordered by a change to one
domain's clock alone*; and *a pin that does not move when one of its two inputs moves 4× was
never reading that input*.

### A guard re-RUN, not re-tuned

`test_the_co2_floor_fires_on_the_buffer_not_on_the_carbon_supply` shrinks the chamber until
the `floor=0.05` liveness guard fires. At `dt = ¼` the whole per-year CO₂ trough series sits
~35 % higher (frozen run 0.0559766 → **0.0754757**), so the absolute floor is a third further
away and the old 0.8× and 0.7× jars no longer reach it. The question is not *what do the old
factors read now* but *does any factor still trip the floor while stationarity passes*, so it
was answered by sweeping the factor and leaving the floor alone:

    0.80x 0.0600926 pass   0.68x 0.0514049 pass
    0.70x 0.0528486 pass   0.65x 0.0492366 TRIP   0.60x 0.0456069 TRIP

**The guard keeps its teeth and they are blunter**: the crossing moved from above 0.8× to
between 0.68× and 0.65×, so the jar must now be shrunk by about a third where a fifth used to
do. Stationarity passes at *every* factor swept, so the "the level check sees what
`is_stationary` cannot" claim is now witnessed more broadly, and is asserted on both tripping
rungs rather than one. The two supply-side probes both **faded by an order of magnitude**
(probe 1's effect 8.7 % → 0.7 %), and probe 2's sign **flipped back** to what it was born with
at a fifteenth of the magnitude it had inverted at two days earlier. *A claim that cannot hold
a sign across a step change is noise about a mechanism, not a mechanism* — so the magnitude
bound, not the ordering, is what now carries that argument.

This moved the biosphere manifest: the `liveness_floors` justification for
`perennial_long_horizon` cites the probe **by value**, so its anchor went `0.0736681`/1.47×
→ `0.0758448`/1.52× and its witness `0.8×`/`0.0481100` → `0.65×`/`0.0492366`. The floor itself
did **not** move, deliberately — re-anchoring 0.05 upward every time the reference moves is
how a floor becomes a restatement of the current run.

⚠ One justification **inverted and the code was left alone on purpose**. The argument for
keeping `transient=_TRANSIENT` in that test's stationarity call was *"its binding same-phase
diff sits at index 2 and is NOT dropped by the window"*. Re-measured, the binding diff is at
**index 0** — which the window *does* drop — and is 4.8 % of bound where it was 90 %. Removing
the window would now tighten the check and still pass with 20× to spare. It stays on its
original merit (the dropped diff is the sow-in, and `_TRANSIENT` is shared with two sibling
gates), and the measurement is recorded so the next reader decides with it in hand.

### Two defects this batch introduced and filed as somebody else's

⚠ **A BOM.** `tests/test_stem_reserves.py` gained a `U+FEFF` byte-order mark in `8b914ff`.
`ast.parse` rejects it, which broke `collect_science_gates()`, which is why **four manifest
gates were red** — and every consumer of that collector had been *failing rather than
checking* for five commits. It was filed as pre-existing on the strength of checking out
`8b914ff`'s own tests and re-running: **that confirms the failure exists at that commit, not
that it predates it.** Check out the commit *before* the suspect one. The same file also
carried an unformatted block and a `pyright` error from the same commit.

⚠ **An index line that was never added.** The log carried a paragraph exempting this
ceremony's plan doc from the index *"while it is in progress"*, ending with instructions to
delete itself on landing. The landing commit added the pointer row and the record file and
**not** the index line, and left the paragraph standing — so three context-budget checks
stayed red for five commits and were read as pre-existing too. *An exemption written for a
temporary state is a thing someone must remember to delete, and the deletion is part of the
work it exempts.*

### Stale prose the flip created in `src/`

Nine statements in shipped modules still described the biosphere as stepping **once per master
day** — `station/driver.py`'s own opening line, and every caller of it (`greenhouse`,
`harvest`, `lighting`, `sealed`, plus two scenario docstrings), `season.py`'s "Euler at
`dt = 1 day`", and `power/scenario.py`'s "the biosphere's `dt = 1 day` analogue". The driver's
*body* had been converted correctly and gained `slow_steps_per_day`; its prose and its callers'
had not. All corrected, prose only. `drift.py`'s provenance block keeps its `dt = 1.0` as the
historical record of how those bounds were derived, now saying so explicitly and noting that
the bounds were **re-run and not re-derived**.

### Open, and deliberately not closed here

* ~~⚠ **`step.py`'s convergence sequence and RK4 limit are STALE** — measured on a tree that
  has since gained stem reserves, soil layers and root coupling.~~ **CLOSED 2026-08-14, and
  the explanation was wrong.** The tree never changed: the sweep's table reproduces cell for
  cell today. The sequence is the **re-sown** run's and the 76.82 is the **no-re-sow** run's,
  and separated, each converges monotonically from below to its own RK4 limit — the paradox
  was the comparison, not the numbers. Both sequences are now tabulated in `step.py`.
  ⚠ **The instruction survives its own reason**: do not quote a convergence figure without
  saying which run it belongs to. And a first explanation that *fits* is not a measurement —
  this one was already contradicted by evidence in hand (peak leaf agreed to 6 s.f. across
  the two, which a changed tree cannot do), and nobody looked.
* ~~⚠ **Not checked:** whether the older `57.9 ppm` headlining `log/co2-enrichment-margin.md`
  and the CO₂-controller work measured the sealed chamber through the same unconditional
  re-sow.~~ **CLOSED 2026-08-14 — it did.** Measured directly at `dt = 1`: the sealed chamber
  reads **75.75 ppm** without the re-sow (minimum in year 1, step 195) and **57.89 ppm** with
  it (minimum in year 3, step 805). Both records quote 57.9, which is the re-sown number and
  is nowhere near the frozen one. Per-year under the re-sow at `dt = 1`: 75.75 / 61.65 /
  **57.89** / **57.93** / 69.33 / 71.04 / 66.32 / 63.63 — so it crosses in years 3–4 only, and
  even that figure is a year-3 reading of a configuration the sealed golden does not run. At
  `dt = ¼` with the re-sow it never falls below ~75.8. ⚠ **Nothing in those two records is
  arithmetically wrong and no conclusion moves** — the crossing they diagnose is real on the
  chambers that re-sow. What changes is that their sealed-chamber column is the re-sown run,
  which is the same locus correction this ceremony already made once. *A figure quoted across
  three documents inherits the run configuration of whoever measured it first.*
* **Measured 2026-08-14 at the user's request (measure only, nothing changed): the frozen
  sealed golden re-sows in none of its three years, and only year 1 is a growing season.**

      no re-sow (frozen)          with the annual re-sow
      yr  CO2 min  CO2 end  leaf  |  CO2 min  CO2 end  leaf
       1    76.82   111.60  0.892 |    76.82   111.60  0.892
       2   111.60  2195.52  0.190 |    75.82   109.65  0.887
       3  2195.52  2356.71  0.000 |    75.96   109.59  0.831

  Years 2–3 of the frozen run are a matured crop respiring into a sealed jar: peak leaf falls
  0.892 → 0.190 → 0.0002 and the pool climbs to **2357 ppm**, ~6× ambient and 31× its own
  year-1 low. **The state that golden pins is a chamber with essentially no plant in it.** With
  the re-sow it is a genuine multi-year closed loop that settles by ~yr 12 to a stationary
  cycle (min ~76.2, max ~1396, year-end ~109.6 ppm, peak leaf → 0.638). ⚠ Recorded as a
  measurement, not a proposal: whether the sealed scenario *should* re-sow is a contract
  question and is the user's.
* **Gaps the re-pin opened deliberately, each named in the test that carries them:** no running
  test separates the DVS form from the frozen one on closure (RK4 stopped hard-erroring on it,
  and the Euler half was always silent); both stem-only reserve-off controls (15 yr and 50 yr)
  no longer collapse, so no running test witnesses that collapse; and the non-vacuity control
  in `test_stem_reserves.py` is still dead. ⚠ *A refusal that rests on a transient is resting
  on a step size* — none of the three refusals is reopened, but each has lost its last measured
  evidence.
* `Γ*` is still a `TODO(cite)` entry, so the threshold cleared is provisional.
* The parked leaf-expansion merge stays excluded: its whole evidence base was measured at
  `dt = 1` and must be re-measured before the question is put again.
