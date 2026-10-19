## **Comments and docs that still spoke of Python as live** (Step 8 of the 2026-09-29 review — the record named a deleted port as the reference, and the freeze docs prescribed its tools)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED AND BUILT 2026-10-19** on the user's *"1, 2 and 3 — do them"*. This is Step 8, slices 1 and
2; slice 3 (a "what this is today" page per domain) was not offered and is not built. Plan and record:
`docs/plans/post-roadmap-stale-record-cleanup.md`.

**Comments and docs only.** ~300 Rust comment lines in 43 files, 47 param-file comment lines in 23
files, ~100 lines of live text in the four contract docs. No value, golden, manifest, string literal
or `source:` string moved. Every deleted Python name can be read at `4f7168e^` (S6's commit); a note
at each crate root says so once, so ~500 "ported from" lines stay as written.

* **The quoted false rule was already gone** — *"Python is right by definition"* left with `e15b9f3`.
  The review's 648-line count was a different rule's; 873 match today, 55 of them string literals.
* **The worst stale text was procedure, not pointers.** The freeze docs' unfreeze steps prescribed
  `ruff`, `pyright` and `-m slow`, said an undocumented unfreeze *"fails CI"* (no CI runs the reference
  since 2026-10-10), and the authoring one said *"Land it on BOTH ports."* All now name the Rust gates.
* **Six gaps found, none fixed here** (plan §4): a station manifest note names a deleted test and no
  Rust test took its check over; three manifest notes and four `source:` strings name deleted files;
  BVAD's seven bookkeeping checks were deleted unported; two authoring gates have no Rust successor by
  name; a rooting-cap branch lost its season-long pin.
* **Lesson: "mirrors X" ages into a false claim the day X is deleted.** A port's comments describe a
  live twin in the present tense; once the twin is gone, every one reads as a sync obligation.
* **Lesson: a comment sweep finds the tests nobody ported.** Five of the six gaps surfaced only because
  a comment named the Python test that used to own the check.
