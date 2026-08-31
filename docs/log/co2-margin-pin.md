## **The five CO₂ margins, re-owned in Rust** (the recheck's own candidate, taken — and the band it was written for was already built)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md), written
> under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per work item.
> The plan doc this row is filed under is
> [`../plans/post-roadmap-direction.md`](../plans/post-roadmap-direction.md), whose §3 item 2
> this work strikes.

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

**NOT TAKEN, and named rather than skipped quietly: the visibility half.** The dead pin had two jobs.
*Detection* is now re-owned for all five. *Visibility* is still four of five: `consumer_long_horizon`
is a scenario in `GATES` with no `runs()` entry and no `SPECS` row in `lab::report`, so the lab report
prints the margin for four of the five and nothing requires anyone to run it at unfreeze time. The
report's own guard `every_spec_names_a_scenario_that_is_actually_run` forces the two to be added
together, which means **a second 15-year run on every long-report invocation** — a runtime cost worth
deciding on its own rather than folding into this item. Left to the user with the price attached.

**Gates run:** `cargo test --release --no-fail-fast` — **321 tests across 30 binaries, all green**,
including `manifest_writer` on all three contracts and both `golden_regression` suites;
`cargo clippy --all-targets -- -D warnings` clean; `rustfmt` on the one changed file, never bare
`cargo fmt`. The `repo_gates` context-budget gates are inside that run.

**Docs touched:** this file, the index and pointer rows in `docs/post-roadmap-log.md`,
`docs/plans/post-roadmap-direction.md` (§3 item 2 struck), and `docs/biosphere-reference.md`, whose
*"RE-CHECKED 2026-08-31"* subsection said in so many words that nothing in `rust/` recorded how near
the five sit to their floor — a sentence that is now false and is corrected at its locus rather than
left to be read as current.
