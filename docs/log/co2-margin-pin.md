## **The five CO₂ margins, re-owned in Rust** (the recheck's own candidate, taken — and the band it was written for was already built)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md), written
> under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per work item.
> The plan doc this row is filed under is
> [`../plans/post-roadmap-direction.md`](../plans/post-roadmap-direction.md), whose §3 item 2,
> §6 items 2 and 3 and §7 question 1 this work strikes — see FINDING 6.

**BUILT 2026-08-31.** One source file changed — `rust/crates/domains/src/biosphere/science_gates.rs`,
a new `#[cfg(test)] mod margins` — plus the docs below. No param, golden, manifest entry, science
band or gate bound moved: `cargo run --release -q -p station --example regen_goldens` reports
**19 of 19 goldens run; 0 would change**, and `git status` outside the docs names that one file.

**FINDING 1 — half the item was already built, and the list that offered it is stale for the third
time.** The work was taken as *"the CO₂ safety check plus the margin record"*. The safety check
exists: five `science_gates!` rows named `..._stays_above_the_compensation_point`, shipped with the
step unfreeze. The direction plan's §3 item 2 still describes that band as unwritten and *"red on
the frozen tree today"* — stale twice over, because the step change **fixed** the crossing and the
band then **landed**. That is the same shape [`co2-band-recheck.md`](co2-band-recheck.md) logged the
day before, and it named itself the third instance: *a forward-looking list is written once and read
many times, and nothing re-checks it.* §3 item 2 is struck in place, with both reasons.

**What was actually missing was the margin.** The five gates are one-sided (`min > floor`) **on
purpose** — they must survive the next mechanism's golden movement without a re-pin — and a one-sided
claim degrades silently: a change halving every margin leaves all five green. The goldens do not
close that gap either, being final-state snapshots (`perennial_chamber_state.json` is the state at
`n = 6100`) with the trough not among the pinned quantities.

**BUILT: `margins::the_five_margins_are_pinned_not_merely_positive`.** A plain `#[test]`, reusing the
cached `runs::` trajectories the gates already build.

⚠ **Deliberately NOT a `science_gates!` row.** A value there would be a second, tighter copy of a
frozen claim — a manifest entry and an unfreeze ceremony — and would take the one-sidedness the band
was written to have. This is a characterisation pin: it holds the numbers the contract cannot, and
re-pinning it is an ordinary edit.

⚠ **The floor is computed, never written.** `floor_ppm()` is `photosynthesis.gamma_star / ci_ratio`,
and `gamma_star` is one of the live `TODO(cite)` params the direction plan lists as a candidate. A
literal `61.071429` here would leave the pin reading a dead denominator on the day that citation
lands, while all five gates moved — *a rule with two copies has one that is stale.*

**FINDING 2 — measured off the instrument, not transcribed from either place that already held the
numbers.** The pin was written with placeholder expectations of `1.0` so its first run had to fail
and print the actuals. Three sources agreed afterwards, and the agreement is the result rather than
the input: the deleted Python constants, the prose table in `docs/biosphere-reference.md`, and the
tree. Measured on `12e2161` (Euler `dt = ¼`):

| scenario | margin (season-low ÷ floor) | the deleted Python pin | gap |
|---|---|---|---|
| `sealed_chamber` | 1.169709 | 1.1671 | 0.22 % |
| `perennial_chamber` | 1.150335 | 1.1543 | 0.35 % |
| `consumer_chamber` | 1.200866 | 1.2086 | 0.64 % |
| `perennial_long_horizon` | 1.150335 | 1.1543 | 0.35 % |
| `consumer_long_horizon` | 1.200866 | 1.2086 | 0.64 % |

⚠ **So the pin restored verbatim would have been green.** Nothing moved these margins between the
checker's deletion on 2026-08-27 and this restoration — which is the honest size of what the four
unowned days cost, stated because the alternative is to imply a rescue. The reason to transcribe
nothing anyway is [`co2-band-recheck.md`](co2-band-recheck.md)'s FINDING 2 one level down: *a pointer
does not inherit its target's corrections.*

⚠ **The two `*_long_horizon` rows are not duplicates and are pinned separately.** They read
identically to their 5-year siblings only because each trough falls inside the shorter horizon
(perennial's in year 2, consumer's in year 5). If they ever diverge, the trough has moved past the
short run's end — a claim changing, not a duplicate drifting. A comment says so, because otherwise a
future reader deletes two rows that look redundant.

**FINDING 3 — the coverage claim is demonstrated, not argued, and it is smaller than it first looks.**
`vcmax` 100 → 130 in `photosynthesis.yaml` (reverted; the file is byte-identical) tightens every
margin by 4.4 % — sealed 1.169709 → 1.118447, perennial 1.150335 → 1.103184, consumer 1.200866 →
1.144956 — and **all five one-sided gates stay green while the pin reddens.** That is the exact
degradation the pin exists to catch, produced rather than described.

⚠ **Stated at its real size: the goldens redden on that same mutation.** So the pin is not the only
detector, and claiming it re-owns detection outright would be the overstatement. What it adds is the
*quantity*: a golden diff says a run moved, the pin says **which** number moved and **by how much**,
in the unit the frozen claim is written in — the number the next unfreeze's gate report quotes.

⚠ **And the tolerance is a re-read trigger, not a golden.** `vcmax` 100 → 105 moves every margin by
less than 2 %, so the pin stays green where the goldens go red. 2 % is kept from the Python pin
because it was **measured to fire**: the within-day light path moved three of the five past it on
2026-08-14 while every one-sided gate stayed green.

**FINDING 4 — the pin's own non-vacuity control.** `the_tolerance_rejects_a_margin_that_actually_moved`
asserts that an exact match and a 1 % shift pass, and that a 3 % shift and a **halved** margin fail.
Written because the subject is a comparison that is easy to make inert — a `TOLERANCE` fat-fingered by
a factor, an `.abs()` on the wrong side — and every real margin would go on passing either way, so the
pin would read green for the rest of its life without a single one of its checks doing work.

**FINDING 5 — the pin's own roster was the failure it was written to prevent, one level down.**
As first written, `PINNED` was five literal tuples and `measured()` five hand-written calls, and
the length check compared **one of my lists to my own other list**. A sixth sealed scenario would
have got its one-sided band and no margin, silently — *a census ported as a LIST is the failure it
prevents*, in the guard written against exactly that drift. Fixed by deriving the roster from
`GATES`: `every_banded_scenario_has_a_pinned_margin` compares the set of scenarios carrying the
band's `quantity` string against `PINNED`.

⚠ **Controlled, not asserted.** A temporary sixth `gate` row (`open_season`, the same quantity)
reddens the tie and leaves both other margin tests **green** — which is the point: the length check
could not see it, so the two guards do not overlap. Probe reverted; the file is byte-identical to
its pre-probe state.

**FINDING 6 — striking one stale bullet left three more armed in the same file, and this item is
what found them.** §3 item 2 was struck for being stale; a re-read of the rest of
`docs/plans/post-roadmap-direction.md` found the same defect three more times, all reading as
current seventeen days after the fact:

* **§6 item 2** still carried `← THE PROJECT IS HERE` against a step decision taken 2026-08-14,
  with eight work items landed since — the most misleading string in the document.
* **§6 item 3** described *"one ceremony"* bundling the step change, the leaf mechanism and the
  CO₂ band, in the future tense. All three shipped **separately**, and the margin pin is a fourth
  piece. The plan was written around a bundle reality routed past.
* **§7 question 1** still lists the step options as live. The struck header at the top of the file
  covers it — but only for a reader who starts at the top, which is not how a reader lands in §7.

⚠ **This is why the item taken here was half-built already**, and it upgrades the recheck's own
closing note from an observation to a measurement: **four spans, one file, nothing re-checking any
of them.** All four are now struck in place with their reasons. The file has no gate and this pass
is not one — it is a re-read, and the next one is owed the next time an item is taken off it.

**~~NOT TAKEN~~, and named rather than skipped quietly: the visibility half.** The dead pin had two jobs.
*Detection* is now re-owned for all five. *Visibility* is still four of five: `consumer_long_horizon`
is a scenario in `GATES` with no `runs()` entry and no `SPECS` row in `lab::report`, so the lab report
prints the margin for four of the five and nothing requires anyone to run it at unfreeze time. The
report's own guard `every_spec_names_a_scenario_that_is_actually_run` forces the two to be added
together, which means **a second 15-year run on every long-report invocation** — a runtime cost worth
deciding on its own rather than folding into this item. ~~Left to the user with the price attached.~~

---

## The visibility half — **TAKEN AND BUILT 2026-09-01**, and the price was one second

Same item, second half, so this is an amendment rather than a new row: no index line, no pointer
row, no second record file. One source file changed —
`rust/crates/domains/src/lab/report.rs` — one `SPECS` row, one `runs()` entry, two doc comments.
No param, golden, manifest entry, science band or gate bound moved.

**FINDING 7 — the price that parked it is ~1.2 s, and the default report pays nothing.** The
paragraph above deferred this on a runtime cost it *described* but never *measured*: "a second
15-year run on every long-report invocation… worth deciding on its own." Measured, release build,
min-of-5, with the before and after binaries built separately and **alternated in one window**:

| invocation (baseline + 1 variant) | before | after |
|---|---|---|
| default (short) report | 1.12 s | 1.09 s — **unchanged** |
| `--long` | 2.28 s | 3.60 s — **+1.3 s** |

The short report's zero is **structural, not luck**: `measure_composed` computes each run's
`needed` spec set and `continue`s when it is empty, so a `long: true` spec costs nothing on the
path that does not ask for it. The whole price falls on `--long`, and it is about a second per
column.

⚠ **The instrument had to be fixed before the number could be trusted, and the first three
attempts were wrong.** Single `cargo run` passes on this box disagreed by **3×** — 4-column
`--long` timed 9.8 s, 12.4 s, 23.5 s in three consecutive runs, and at one point the *before*
binary timed **slower than the after**, which is impossible and was the tell. `Get-Process` named
the cause: `MsMpEng` (the antivirus) was the top CPU consumer on the machine. *A wall clock on a
contended box is not a measurement, and a before/after pair taken in two different windows is not
a comparison.* Min-of-5, both binaries alternated in one window, reproduced across two passes.

**FINDING 8 — the guard's own hypothetical was discharged as a prediction, not merely as a fix.**
`every_spec_names_a_scenario_that_is_actually_run` carried this exact case in its doc comment:
*"`consumer_long_horizon` is already a scenario in `GATES` with no `runs()` entry, so the next spec
added under it is the one that would vanish."* The `SPECS` row was therefore landed **alone
first** — and the guard failed by name on it (`consumer_long_horizon/season-low chamber CO2 (ppm)
names no run`) before the `runs()` entry was added. So the control is the prediction being run,
and the comment now records a **spent** example rather than being reworded into a fresh imaginary
one.

**The fifth row agrees with the pin — and that is a wiring check, not a second route to the
number.** The report prints `consumer_long_horizon / season-low chamber CO2 (ppm)` frozen at
**73.338613 ppm**; the pin holds that scenario's margin at **1.200866**, and
`73.338613 / 61.071429 = 1.200866`. ⚠ Stated as exactly what it is, because this file's own
standard is *a matching boolean from different arguments is `[] == []` in a passing test's
clothes*: the pin folds `min_ppm` over `runs::consumer_long()` and the report folds `min_ppm` over
`consumer_chamber_scenario()` at `LONG_HORIZON_YEARS` — same scenario, same fold, two call sites.
What the agreement establishes is that the new row is wired to the **right run**. It is not
independent evidence about the value.

**One stale claim corrected at its locus:** `ReadoutSpec::long`'s doc said the 15-year horizon
costs *"minutes rather than seconds"*. It is seconds — the numbers above are written in, with the
contention caveat, because the next reader deciding whether to add a long row will read that field.

### The standing re-read this item owed — §4 of the direction plan

The half above closes with *"the next one is owed the next time an item is taken off it."* Taking
this one, §4's remaining bullets were checked **against the tree, not re-read as prose**. Two are
stale, one number is stale, three stand:

⚠ Written as a **list, not a table**: the record-file cap is 120 characters and a markdown table
row has no legal wrap point, so a table here would have forced the verdicts to be shortened to fit
the format — and the verdicts are the finding. (`7f60442` is the same cap catching the same shape.)

* **`Γ*`'s citation — STANDS.** `gamma_star` is still `TODO(cite)` in `photosynthesis.yaml`, and
  all five band gates say so in their own `source`.
* **Citation debt, "79 `TODO(cite)` across 37 files" — COUNT STALE, claim stands.** That count was
  the deleted Python tree's. Re-measured on the reference: **60 across 20 files** in `domains`,
  4 across 3 in `station`; the biosphere alone is 53 across 15.
* **"A provenance-only edit is an unfreeze that nothing catches" — FALSIFIED**, measured rather
  than argued. FINDING 10 below.
* **"The freeze's prose half is ungated" — STANDS.**
* **Potato stage 2 — STANDS.** The params came across with the flip
  (`params/biosphere/crops/potato/`, 4 files), but `system.rs:1977` says in so many words that
  *"the Rust roster has no potato build at all"*. A param move is not the habitat mirror.
* **The canopy regulator, "DIAGNOSED, not built" — BUILT.** FINDING 9 below.

**FINDING 9 — the canopy regulator shipped, under another item's name.** §4 still offers it as
cheap open work whose *"inertness makes it a low-risk unfreeze"*. It is in the tree:
`science::mutual_shading_rate`, called from two sites in `flows.rs`, parameterised in
`senescence.yaml`, and gated by `the_vks_mutual_shading_regime_is_modelled_not_merely_avoided`
— the Van Keulen & Seligman 5 %/day-above-LAI-6 rule that `canopy-regulator.md`'s FINDING 1
identified. It landed as *"the mutual-shading loss the pair forced"* inside
[`layered-canopy.md`](layered-canopy.md), so no row ever said "the canopy regulator is built".
⚠ And the bullet is wrong twice more even as a description: `acceptance-gate-standing.md` records
it as *inert on the chambers, **flipping** the canopy on `open_season`* — not "bit-identically
inert on every frozen scenario" — and `gross-net-gas-exchange.md`'s FINDING 5 **eliminated it by
measurement** as a fix, because it is a 5 %/day *loss* and the canopy it was offered to repair had
fallen to 4.71. **Fourth instance of the shape** the last two records named: *a forward-looking
list is written once and read many times, and nothing re-checks it.*

**FINDING 10 — "nothing catches a provenance-only edit" is false, and the control is three
commands.** `CLAUDE.md` has said since 2026-08-18 that C7 falsified it; §4 was never updated, and
neither was the sentence inside `canopy.yaml` itself (*"a provenance-only unfreeze, which the
manifest records and no test can see"*). Rather than cite the correction, it was **run**: append
`(control probe)` to one `source:` string in `canopy.yaml`, and
`the_committed_manifest_is_what_the_reference_writes` fails, naming the file and both hashes —

```
line 163:
  writes:    "canopy.yaml": "8a504ca856651b046d8cd5271add2204a009718f1bc7c4972442a6d4ac53ad78",
  committed: "canopy.yaml": "2e5a2f7f665886dff0f3c60f739b97163cb690213a6d30f6a850aa29411748fc",
```

The probe was reverted byte-for-byte (`git diff` on `params/` empty, the gate green again).

**Gates run for the visibility half, on the finished tree — one run, after every edit.** ⚠ Stated
that way on purpose: the first landing's gate sentence spanned two tree states and this one does
not. `cargo test --release --no-fail-fast` → **1101 passed, 0 failed, 4 ignored across 64 result
lines, exit 0**, with `manifest_writer`, `golden_regression` and `context_budget` confirmed by name
in the binary list rather than inferred. ⚠ The count is read off the **whole** file: the first
attempt was captured with `2>&1 > file`, which sends stderr to the *old* stdout and would have
dropped `error: test failed` — that run was discarded and re-run with `> file 2>&1`, because *an
instrument that cannot record a failure cannot report a pass*. `cargo clippy --all-targets -- -D
warnings` clean; `rustfmt` on the one changed file, never bare `cargo fmt`; and
`cargo run --release -q -p station --example regen_goldens` → **19 of 19 run, 0 would change**.
⚠ **`canopy.yaml`'s own stale sentence is deliberately NOT fixed**, and that is the rule working
rather than laziness: a comment-only edit to a param file moves its manifest hash and is itself an
unfreeze event, which this control just demonstrated. A prose fix is not worth a freeze ceremony —
the same reasoning `post-roadmap-log.md` records for `self_discharge.yaml`. What *is* honor-system
is still the **ceremony**, not the regeneration.

**Gates run, on the committed tree:** `cargo test --release --no-fail-fast` — **1101 tests across
64 result lines, all green, 0 failed**, with
`manifest_writer` (domains, station, authoring) and both `golden_regression` suites confirmed by
name in the output rather than inferred from the binary count; `cargo clippy --all-targets -- -D
warnings` clean; `rustfmt` on the one changed file, never bare `cargo fmt`; and
`regen_goldens` reporting **19 of 19 run, 0 would change**.

⚠⚠ **And its test COUNT was wrong, by the repo's own documented mechanism.** The first landing
said *"321 tests across 30 binaries"*. The real figure is **1101 across 64**: the count had been
summed from a `| grep … | tail -60` pipeline, so it totalled the last sixty lines of the run rather
than the run. Green either way, and the verdict was never in doubt — but *a battery that greps its
own output can report fewer tests than ran*, which is `CLAUDE.md`'s `--no-fail-fast` warning one
level up: there the instrument truncates the reds, here it truncated the greens. The rule is the
same — **read the count off the whole output, or do not quote a count.**

⚠ **The first landing's gate sentence spanned two tree states and is corrected here rather than
left standing.** Its 321-test run happened *before* the doc edits and `repo_gates` *after*, so no
single run covered what was committed. Nothing under `docs/` feeds any test but `repo_gates`, and
the reference edit is prose rather than `.manifest.json` — but that is reasoning, and the record
this item grew out of is the one that wrote down *a doc commit with a partial gate run and no note
is indistinguishable from a skipped one.*

**Docs touched:** this file, the index and pointer rows in `docs/post-roadmap-log.md`,
`docs/plans/post-roadmap-direction.md` (§3 item 2 struck), and `docs/biosphere-reference.md`, whose
*"RE-CHECKED 2026-08-31"* subsection said in so many words that nothing in `rust/` recorded how near
the five sit to their floor — a sentence that is now false and is corrected at its locus rather than
left to be read as current.
