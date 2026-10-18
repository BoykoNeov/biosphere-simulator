## **A param file's fingerprint follows its meaning, not its text** (Step 4 of the 2026-09-29 review — a comment edit in a frozen param file moved a manifest hash, so comments known to be false were left in place)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED AND BUILT 2026-10-18** on the user's *"1 and 2"* — Steps 4 and 7 of the review proposal
written 2026-09-29. This is Step 4. Plan and record: `docs/plans/post-roadmap-meaning-fingerprint.md`.

**4a BUILT.** `config::provenance::param_fingerprints` parses a param file with the loaders' own reader
and returns `values` (the parsed document, each parameter's `source:` removed) and `sources` (those
strings, by parameter). Both manifests' `param_files` entries are now `{sources, values}`. A re-anchoring
ceremony on two manifests (the authoring manifest hashes no param file): **the predicted diff was
`param_files` and its `_authority` row in each, and that was all that moved** — no golden, set, claim,
horizon or census. Controls kept as tests (a fixture in `config`; all 25 real files in
`station/tests/param_fingerprints.rs`), each shown red by a mutation.

**4b DECIDED, not asked:** option 1, the eleven transcendental goldens keep regenerating on Windows only —
the user's 2026-10-10 "Linux is not a goal" settles it.

**The live control, a separate commit:** `self_discharge.yaml` pointed three times at Python files deleted
in S6. Corrected; both manifests regenerated **byte-identical**. Under the old rule that was an unfreeze.

* **Lesson: a scalar is its text, on purpose.** Hashing numbers by value (`1.0e7` = `10000000.0`) was
  suggested and not taken: the fingerprint would have to decide which strings are numbers, which the
  schema decides per field, and a wrong call is a *missed* change. A reformatted number costs a ceremony;
  a missed one costs the freeze.
* **Lesson: re-check a plan's premise on the day.** Of the two stale comments the plan named, one had
  already been fixed (2026-10-06). The other was still there.
