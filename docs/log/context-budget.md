## **Tooling: the context budget** (the 2026-08-11 CLAUDE.md cut, re-opened because it did not hold)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**COMPLETE 2026-08-12** — `docs/context-budget.md`, `tests/test_context_budget.py`. **The
finding: a relocation is not a discipline.** The 2026-08-11 commit cut `CLAUDE.md` 213 KB →
14,458 B by moving the record here; eleven commits *the same day* put it back to 17,740 B,
~300 B per finished piece of work, monotonically, because nothing in the rule ever *retired*
a row. Measured before the fix: the status section was **8,888 of 17,715 B — 50.2 %** of the
always-loaded map; the post-roadmap index was **32 rows at a mean 239 B** against a rule
that said "*one line* each"; the Phase 0–9 table was **11 rows, 692 B, every one reading
COMPLETE** and unchanged for months; and `MEMORY.md` indexed the same ledger a second time
in **40 lines** (28 "Biosphere science" + 12 "Platform & ports") against those 32 rows — two
unconditionally-loaded indexes over one ledger. Total unconditional load was ~36 KB / ~9k
tokens before the user typed anything. **Three rules built.** (1) *Retirement*: a work item
leaves the always-loaded map when it is finished AND its lesson exists elsewhere; the map
keeps a pointer, never a summary. Applied, this retired the phase table (→
`docs/phase-index.md`) and the 32-row index (→ the `## Index` section of this file, directly
above the record it indexes) — **both moved content-verbatim — verified line-by-line against
`d86d9c8`, line endings normalized to the destination; no row was condensed, summarised, or
dropped**, and the phase table is now content-pinned by sha-256. (2) *One unconditional
index*: the tie is broken by **load discipline**, not by which reads better — `MEMORY.md`
lines are the matching surface that decides whether a memory file is recalled at all, so
deleting one makes a memory unreachable; the `CLAUDE.md` table was pure navigation and the
file it navigated could host it at zero unconditional cost, so that is the copy that moved.
(3) *A paired gate*, because every other contract here has one and this one had re-bloated
inside 24 h without it: a hard byte ceiling on `CLAUDE.md`, a guard on the specific shape
that regrew (a table row naming a plan doc), index↔record parity so retirement cannot decay
into silent deletion of one half, and plan-doc completeness. **The gate shipped with a
broken assertion, caught on advisor review the same day and fixed in the follow-up commit.**
Parity was first written as "do both tables name the same plan docs" — vacuous for any row
naming something else (`docs/test-suite-runtime.md`,
`tests/test_authoring_export_fidelity.py`, `docs/context-budget.md`), and **the first row
written under the new discipline was such a row**: deleting it from the index alone was
measured green. Replaced by a row-count delta, which is blind to what a row names; running
it immediately surfaced a real property nothing had stated — the index legitimately carries
**one row more** than the record, because "Stem reserves: the model FORM found" is a lead
whose record lives inside the wheat-backfill row. Two method findings, both kept in the doc:
**a check that reads correctly can still be testing nothing** (falsify on the case you care
about), and **a green falsification attempt is a result to distrust** — the first attempt
here passed only because a PowerShell string was indexed as a char array, so the mangle
never happened. Every assertion is now falsified: a re-added status row, a blown ceiling, an
index row deleted from one side, and an edited phase row each turn it red. Two headline
numbers were also corrected on review — the index was **32 rows at ~239 B**, not 30 at 243;
the "30" came from a filter matching `post-roadmap-`, which silently dropped the two rows
pointing elsewhere. **A filter is a claim about what you are counting.** ⚠ The `MEMORY.md`
ceiling is **green-by-skip on CI** — that path is in the user's profile, not the repo —
stated in the module docstring rather than left to be discovered, since this repo has
already been bitten once by a check that was green because it never ran. Result: `CLAUDE.md`
17,715 → 9,520 B, the status-ledger share 50.2 % → 0 %, unconditional indexes 2 → 1,
enforcement honour-system → tested. **Rule 4 is diagnosed, priced, and NOT BUILT on the
user's call:** this file is 255 KB in 52 physical lines — one work item per line, the
nitrogen row **55,289 characters on line 30** — which defeats `Grep` (one 55 KB "match"),
`Read` (cannot page into a row) and `git diff` (a one-word edit rewrites the line). The fix
is a byte-preserving mechanical split, one file per work item; any version that reads as
"condense the record" must be refused, for the reason this file's own preamble already
gives.
