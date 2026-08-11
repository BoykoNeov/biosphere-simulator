# The context budget — why `CLAUDE.md` is small, and what keeps it small

**Status: rules 1–3 BUILT 2026-08-12** (rule 3's gate shipped with a vacuous assertion,
caught on advisor review the same day and fixed — see rule 3). **Rule 4 (splitting the
record file) is diagnosed, priced, and NOT BUILT — the user deferred it deliberately.**

Every other contract in this repo has a paired test. This one did not, and it failed
inside 24 hours. That is the whole reason this document exists.

## The measurement that forced it

On 2026-08-11 the post-roadmap record was moved out of `CLAUDE.md`, cutting the file
from **213 KB to 14.4 KB**. By the end of *that same day* — eleven commits — it was back
to **17,740 bytes**:

| Commit (2026-08-11) | `CLAUDE.md` bytes |
|---|---|
| `255da30` move the record out, leaving an index | 14,458 |
| `4e93299` O₂ regulator's reversal | 14,855 |
| `68909a2` direction gate | 15,115 |
| `8d81612` citation-pin wrapping | 15,352 |
| `00c3d9a` second authored habitat | 15,591 |
| `c16e88a` potato | 15,941 |
| `afd29e9` wheat partition backfill | 16,669 |
| `de5b2c8` + `0585c3f` root coupling | 17,147 / 17,259 |
| `b04b61e` rooted depth | 17,286 |
| `d86d9c8` soil layers | 17,740 |

**+3.3 KB in one day, ~300 bytes per piece of work, monotonically.** Extrapolated at the
observed rate, the file is back over 50 KB within a few weeks and back to 200 KB inside
a year. The 2026-08-11 fix changed *where the record lived*; it did not change the rule
that manufactures rows. That is the finding: **a relocation is not a discipline.**

## What the bytes were actually spent on

Measured on `CLAUDE.md` at 17,715 bytes, immediately before this change:

| Section | Bytes | Share |
|---|---|---|
| "Phase status" (the two index tables) | 8,888 | **50.2 %** |
| Everything else (invariants, layout, commands, contracts, style) | 8,827 | 49.8 % |

Inside that half:

- **The post-roadmap index: 32 rows, 7,661 bytes, mean 239 bytes/row.** The rule in
  force said "*one line* each". A 239-byte row is a paragraph. The rule was being
  violated on every commit that cited it.
- **The Phase 0–9 table: 11 rows, 692 bytes.** Every row read `COMPLETE`. It had not
  changed in months and never would again. Pure sunk cost, paid every session.
- **`MEMORY.md` indexes the same ledger a second time**, in 40 lines — 28 under
  "Biosphere science" plus 12 under "Platform & ports" — against the table's 32 rows.
  Two unconditionally-loaded indexes over one ledger, so a work item is paid for twice
  in every session, whether or not the session touches it.

> **A correction, kept because it is the same mistake this document is about.** The
> first pass reported *30* rows, 7,293 bytes, and "~27 of the 30 also in `MEMORY.md`".
> The 30 came from counting rows that matched `post-roadmap-`, which silently drops the
> two rows pointing at `docs/test-suite-runtime.md` and
> `tests/test_authoring_export_fidelity.py`; the ~27 was an eyeball, never measured. The
> same blind spot then went into rule 3's first draft (see below), where it made the
> parity check vacuous. **A filter is a claim about what you are counting** — the number
> it returns cannot tell you what it excluded.

Total unconditional load at the time: 17,715 (project map) + 8,234 (global preferences)
+ 9,842 (memory index) ≈ **36 KB, ~9k tokens, before the user types anything.**

## The rules

### 1. The retirement rule (the one that was missing)

> A work item leaves the always-loaded map when it is **finished** *and* its lesson
> exists somewhere else. The map then keeps a **pointer**, never a summary.

Nothing in the previous discipline ever removed a row. Rows were added forever and
condensed only in a crisis, by hand, which is exactly the operation the log's own
preamble warns is how a finding gets silently dropped. A retirement criterion makes
removal routine and mechanical instead of exceptional and lossy.

Applied on 2026-08-12 this retired the entire Phase 0–9 table (11 rows) and all 32
post-roadmap rows — **both moved content-verbatim, nothing rewritten:**

- the phase table → `docs/phase-index.md`
- the post-roadmap index → the top of `docs/post-roadmap-log.md`, above the record it
  indexes, where a reader who opens the file for one row can find the other 31

"Verbatim" here is a claim about bytes, so it was **measured, not asserted**: every one
of the 13 phase-table lines and all 34 index-table lines were compared character-for-
character against `d86d9c8:CLAUDE.md`, the commit before the move, and are identical.
Line endings were normalized to each destination file, so the precise claim is
**content-verbatim, destination line endings** — not byte-identical. The phase table,
which is genuinely frozen, is now content-pinned by sha-256 in the test so it stays
that way.

### 2. One unconditional index, not two

The two indexes differ in **load discipline**, and that is what decides which survives:

- `MEMORY.md` lines are the **matching surface** for recall — the text that decides
  whether a memory file gets pulled in. Delete a line and the memory becomes
  unreachable. It must keep one line per memory.
- The `CLAUDE.md` table was **pure navigation**, and the thing it navigated can host it
  at zero unconditional cost.

So the duplicate copy is the one in `CLAUDE.md`, and that is the one that moved. This
generalises: **when two always-loaded files describe the same thing, the one whose copy
does no work at load time loses it.**

### 3. A paired mechanical gate

`tests/test_context_budget.py`. The assertions:

1. `CLAUDE.md` is under a **hard byte ceiling** (`MAX_CLAUDE_MD_BYTES`). Raising it is a
   deliberate, reviewable, git-visible act — not the silent accretion the table above
   documents.
2. `MEMORY.md`'s index is under its own ceiling, for the same reason.
3. **Index ↔ record parity** in `docs/post-roadmap-log.md`, by **row count** — what
   stops the retirement rule from becoming the *deletion* rule. The index carries
   exactly one row more than the record (`INDEX_SURPLUS_ROWS`), because "Stem reserves:
   the model FORM found" is a lead whose record lives inside the wheat-partition-backfill
   row; the delta is asserted exactly, so drift in *either* direction is red.
4. **Completeness:** every `docs/plans/post-roadmap-*.md` on disk is named in the log,
   and the log names no plan doc that does not exist.
5. **The moved phase table is content-pinned** by sha-256, so "moved verbatim" stays a
   measured fact rather than a sentence.

> **Rule 3 shipped broken, and the bug is worth keeping on the record.** Its first
> version compared the *plan docs each table names*. That is vacuous for any row naming
> something else — `docs/test-suite-runtime.md`,
> `tests/test_authoring_export_fidelity.py`, and `docs/context-budget.md` — because
> neither side contributes those to the comparison. **The very first row written under
> the new discipline was such a row**, and deleting it from the index alone was measured
> green: the gate built to enforce the rule waved through the rule's own first
> application. Row counting is blind to what a row names, which is exactly the property
> the original check lacked. Both checks are kept; the set comparison is sharper where
> it applies. The finding generalises: **a check that reads correctly can still be
> testing nothing — falsify it on the case you actually care about, not on a convenient
> one.** (The first falsification attempt here also passed spuriously, because a
> PowerShell string was indexed as a char array and the mangle never happened. A green
> falsification attempt is a result to distrust, not a pass.)

Running the row-count check for the first time immediately surfaced the shared-record
pair above — a real property of the ledger that nothing had previously stated.

The ceiling is not a target to grow into. Headroom exists so a genuine new invariant can
land without a same-commit ceiling bump; it is not budget for status rows.

### The memory side — what retirement means there

Rule 1 retires rows from the docs side. **The memory index has no such rule, and its
ceiling shipped without a legitimate escape hatch** — the failure message's original
advice was "tighten the hooks", which is hand-condensing, the exact operation this
document argues is how findings get dropped. A gate whose only remedy contradicts the
document it enforces is not finished. So, stated before it fires:

`MEMORY.md` is 10,069 B over 62 lines (~162 B/line) against a 12,000 B ceiling — roughly
twelve more memories, a few weeks at this project's pace. When it fires:

- **The remedy is to MERGE, not to delete.** Two memories on one subject become one file
  with one index line, the detail preserved *inside* the file. That is structurally the
  same move as the docs side — push detail down, keep one pointer — and it is not a
  condense, because nothing is rewritten away.
- **Deleting an index line is not available.** Those lines are the matching surface that
  decides whether a memory file is recalled at all; delete one and the file becomes
  unreachable, which is worse than the bytes it saved. This is the asymmetry that decided
  rule 2.
- **Raising the ceiling is allowed, but must restate the per-line budget** (bytes ÷
  lines) in the same commit. Raising it without that is how the ceiling stops meaning
  anything — which is the whole failure documented at the top of this file.

### What the gate deliberately does NOT check

It cannot tell whether a row *should* have been retired, or whether a memory file
actually captures the lesson. It bounds the blast radius; it does not supply judgement.
This is the same standing the freeze manifests have: they own **completeness**, the
goldens own **values**, and neither owns "is this science right".

## Rule 4, diagnosed and not built

`docs/post-roadmap-log.md` is **255 KB in 52 physical lines** — one giant markdown table
where a single work item is one line. The nitrogen row is **55,289 characters on line
30**.

This defeats every tool that reads it: `Grep` returns a 55 KB line as "one match", `Read`
cannot page into a row, and `git diff` rewrites the whole line for a one-word edit. **The
record being unusable is a worse defect than the index being big.**

The fix is a **mechanical split that preserves bytes exactly** — one file (or one
multi-line block) per work item, moved verbatim, with the table reduced to pointers.
Any version of this that reads as "condense the record" must be refused: the log's own
preamble already establishes that hand-condensing rows is how findings get dropped, and
it is right. **Relocate and point; never summarise.**

Deferred by the user on 2026-08-12 as its own piece of work, since it touches 255 KB and
is independent of the rules above.

## Result

| | Before | After |
|---|---|---|
| `CLAUDE.md` | 17,715 B | 9,520 B (ceiling 12,000) |
| Share that was a status ledger | 50.2 % | 0 % |
| Indexes over the post-roadmap ledger, loaded unconditionally | 2 | 1 |
| Enforcement | honour system (failed in 24 h) | `tests/test_context_budget.py` |

**No record was condensed, summarised, or dropped.** Both tables moved content-verbatim,
verified line-by-line against `d86d9c8`; the phase table is content-pinned from here on.

Every assertion in the gate was **falsified before being believed** — a re-added status
row, a blown ceiling, an index row deleted from one table only, and an edited phase row
each turn it red. That mattered: the first version of the parity check passed all of it
while testing nothing (see rule 3 above).
