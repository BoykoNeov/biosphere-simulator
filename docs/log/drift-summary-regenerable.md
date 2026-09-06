## **`drift_summary.json` becomes regenerable** (the last frozen value the reference could not produce)

The biosphere contract's last Python-authored value. One golden, one golden hash, one
`_authority` row; no science, no parameter, no other golden — and the whole of it was
predicted before it was run.

Plan: `docs/plans/post-roadmap-direction-2026-09.md` §3.2 (the OPEN item *"unregenerable by
any path"*), which named it as the last entry on the "no decision needed" list that was
neither blocked nor re-ranked. Built 2026-09-06.

### The defect: an unfreeze rule with a missing step

`docs/biosphere-reference.md`'s unfreeze discipline, step 3, says *"regenerate the affected
goldens and review the byte diff"*. For one of this contract's seven goldens that instruction
had pointed at **nothing** since 2026-08-27, when slice S6 deleted the Python checker: the
program that folded `drift_summary.json` went with it, and the reference had never authored
the file. So a frozen contract carried a value that **no tool could reproduce** — an unfreeze
whose ceremony could not be performed.

⚠ **This is a worse shape than a stale value, and the reason is that nothing goes red.** A
wrong number has a test that fails. A missing regeneration step fails only when somebody
tries to use it, which by construction is the one moment there is no time to build it. Every
automatic gate on this file was green throughout: the byte compare
(`domains/tests/golden_regression.rs`) never ran on it, because it was on the
*not-reference-authored* roster; the manifest hash matched, because nothing had moved it.

### Why it was left behind, and why that reason had expired

Not inertia — slice C5 measured it. C5 ported `drift.py`'s whole fold kit into
`domains::biosphere::drift` and converted the *station's* drift emitter to emit its summary
directly, and then measured this one: folding the Rust series moves **4 of its 45 values**
(≤7 ULP, consumer years 3–4). Python would therefore have needed an entry on
`golden_platform.PYTHON_DIVERGES`, and
`test_golden_provenance.py::test_every_diverging_scenario_keeps_a_byte_gated_sibling` refused
one — `emit_drift` served exactly one golden, so under that gate's key (the emitter program)
the entry had no byte-gated sibling. Widening a gate from inside the slice that needs it
widened is the co-adaptation this repo refuses, so authorship was deferred to its own
ceremony (plan §5h).

**That gate was a property of the Python comparator, and S6 deleted the comparator.** The
blocker did not weaken, it ceased to have a subject. What it left behind was the missing
regeneration step above. ⚠ Both halves of this are worth carrying: *deferring to avoid
co-adaptation was right*, and *a deferral outlives the thing it was deferring to* — the
record has to be re-read when the blocker's world changes, because nothing re-reads it on
its own. `regen.rs`'s assertion had actually noticed (*"a reason that has since dissolved"*)
and the item still sat open for ten days.

### What was built

* **`domains::goldens::drift_summary`** — the two 15-yr chamber trajectories (perennial +
  consumer), folded with the C5 kit: `year_summaries` for all three vectors (peak for the
  two leaf series, the segment's last value for `consumer_carbon`), then `is_period_2` on
  the folded 15-element vectors with `transient = 8` years and `min_rel_gap = 1e-3`.
* **`emit_drift.rs`** collapses to a one-line wrapper, like every other emitter since S2.
  Its header, which said the conversion was blocked and must not be finished, is rewritten
  rather than left standing — a stale instruction not to do the thing that was just done is
  worse than no instruction.
* Roster: `DOMAINS` gains the entry (`Transcendental` / `Cheap` / `FoldedSummary`), so the
  byte compare in `domains/tests/golden_regression.rs` now covers it **by iteration** — it
  is not a new test that could be forgotten. Roster count 19 → 20.
* `NOT_REFERENCE_AUTHORED` drops to one entry (`state_snapshot.json`), and
  `every_frozen_golden_is_one_the_reference_authors` now asserts the unauthored-frozen set is
  **empty**, kept non-vacuous by the `frozen.len() == 20` control already above it.
* The manifest's `scenarios/drift_summary/golden_sha256` authority moves `python` → `rust`.
  It was the last `python` key in this contract.

### The measurement, predicted first

Written down before the fold was run (the `soil-layers-built` discipline), because the
figures being reused were **twenty days old** and much science had landed in between:

```
consumer.peak_leaf        yr 3   ulp=7   rel=9.955e-16
consumer.peak_leaf        yr 4   ulp=1   rel=1.493e-16
consumer.consumer_carbon  yr 3   ulp=2   rel=4.389e-16
consumer.consumer_carbon  yr 4   ulp=2   rel=2.289e-16
```

**Measured: exactly those four, to every digit.** All 15 perennial peaks byte-identical, the
other 26 consumer values byte-identical, `horizon_years` unchanged, and **both `is_period_2`
booleans held `false`** — Tier-0 exact, no stability-class change.

⚠ The staleness risk was checked rather than assumed, and the check is one command:
`git log -- rust/data/golden/` has a single commit since the tree moved (2026-08-18), and the
two byte-gated long-horizon goldens drawn from these same two runs are green. So the
trajectories had not moved and the dated figures were still live. **A dated figure is not
automatically wrong; it is automatically unverified.** Reusing one without that check is the
failure this repo has recorded three times on `tiers.json` alone.

⚠ **The agreement is also the control that makes this a claim about the ports rather than
about a new fold.** The port hazard here is segmentation, not arithmetic: every float in the
file is a *selected* value (a segment max, or a segment's last element), never a computed
one, so a wrong inclusive boundary or a period passed in days instead of steps would move
**values**, not last bits, and would move the perennial too. Getting exactly the four
predicted ULPs is evidence the segmentation is right in a way a green byte compare against a
file you just wrote could never be.

The cause is C5's diagnosis unchanged: a 1-ULP transcendental divergence appears at step
4095, 1750 of 18301 steps then differ, and the contracting attractor damps it back to a
**bit-identical final state** by year 15. The final-state golden stays byte-gated and green
*because* the attractor contracts; these per-year peaks are the only artifact that samples
the trajectory while the difference is still alive.

### ⚠ What changed in KIND, not in value

Those four ULPs were a **deviation absorbed by a Tier-2 band**. They are now the **golden's
own bytes**. Plan §5h wrote the generalization and it is the one to keep: *a tolerance band
hides a difference until the flip makes one side the author — then the same number is a
value, not a deviation.* This is why the item was an unfreeze rather than the re-anchoring
C1/C8/C9 were, even though nothing scientific moved.

### Deliberately not done

* **`rust/data/tiers.json`'s `drift_summary` evidence string** still reads *"Rust-vs-Python
  bit-exact locally (max_rel_dev 0.0)"*, dated P7.4 and measured at 9.955e-16 on 2026-08-17.
  It belongs to the **native-port** contract (`docs/native-port-reference.md`), which has its
  own unfreeze ceremony; §5h ruled it out of this work when C5 was written and it stays out.
  Named here so the record is the thing that carries it, not a memory.
* **The `_authority` block's remaining `python` key** in the *authoring* contract
  (`parity_vectors/*`) — a different contract and a different reason (generator scaffolding).
  The biosphere contract is now fully reference-anchored; the file-level statement in
  `CLAUDE.md` that a `python` key is *"a queue, not a classification"* is unchanged and still
  true of the one that remains.

### ⚠ Side-finding: the `Cost::Cheap` threshold was stale, and not because of this entry

Classifying the new roster row forced a number, and the first one written here was
**inferred, not measured** (*"~3 s in release"*) — the exact move this repo refuses. Measuring
it produced a finding about the axis rather than about the entry.

`Cost::Cheap` said *"Under ~4 s. Runs in every `cargo test`."* But `cargo test` builds
**unoptimized**, and the debug numbers are:

| golden | runs | debug |
|---|---|---|
| `perennial_long_horizon_state.json` (a `Cheap` entry since P7.4) | 1 | **11.7 s** |
| `drift_summary.json` (new) | 2 | **17.4 s** |

⚠ **The control is what makes this a fact about the axis.** An eleven-second entry was
already sitting inside a four-second bound, so the threshold was false *before* the row that
measured it — the new entry is 1.5x a sibling already on that side of the line, not a new
category. Reclassifying here would have been reading one entry's cost as if it set the
boundary, which is the widen-the-gate-from-inside move; the doc line was corrected to the
measurement instead. **Nothing moved sides, and no gate reads that prose** — `Expensive` is
the side with teeth (`#[ignore]` + `the_ignored_set_is_exactly_the_expensive_roster`), and
its ~100 s IS measured, three optimization levels deep, in the same header.

The figure arrived **with the port** (S2, 2026-08-19) and was never re-measured against a
Rust debug run. Checked rather than assumed: the date is *after* `dt=1/4`, so the obvious
story — that the step change invalidated it — is wrong; it was simply carried across the
language boundary unexamined. That is [[posture-landed-in-claude-md]]'s finding a **third**
time (C3's stale `dt=1`; C7's stale *"nothing catches this"*; now a stale threshold) — and
the third instance widens it, because the first two were in `CLAUDE.md` and this one is a
doc-comment in the reference itself. **The shape is not "the always-loaded file is
unaudited"; it is "prose is unaudited wherever it lives."**

⚠ The absolute seconds are the weak half: the box was carrying four other projects' cargo
builds throughout, and the same binary measured 25.2 s in one batch and 17.4 s in another.
**The pair was measured in one invocation, so the ratio is the durable claim** and the
seconds are illustration. Re-timing was declined rather than repeated — the contention is
other sessions and does not clear.

### The generalization

**A rule with a step that cannot be performed is invisible to every gate that watches the
rule's subject.** The freeze machinery here is unusually thorough — completeness gates, byte
gates, a manifest that hashes its own goldens — and all of it watched the *values*. Nothing
watched whether the *procedure* still existed. The gate that eventually noticed was a prose
string inside an assertion message, which is to say: a person had to read it.
