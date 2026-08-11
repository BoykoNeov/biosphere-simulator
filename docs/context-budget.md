# The context budget — why `CLAUDE.md` is small, and what keeps it small

**Status: all four rules BUILT 2026-08-12.** Rules 1–3 first (rule 3's gate shipped with
a vacuous assertion, caught on advisor review the same day and fixed — see rule 3); rule 4
was deferred by the user at that point and taken later the same day as its own piece of
work.

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
6. **Three more, added by rule 4** once the record moved out to `docs/log/`: pointer↔file
   parity against the disk, each file's heading being its row's Work cell verbatim, and a
   line-length cap. See rule 4.

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

### 4. The record is one file per work item, not one *line* per work item

The first three rules are about *size*. This one is about **shape**, and the shape was the
worse defect. `docs/post-roadmap-log.md` held **255,567 bytes of record in 32 physical
lines** — one markdown table row per work item — and the nitrogen row alone was **54,343
characters on one line**.

That defeats every tool that reads the file: `Grep` returns a 54 KB line as "one match",
`Read` cannot page into a row, and `git diff` rewrites the whole line for a one-word edit.
**A record nobody can read is a worse defect than an index nobody needs.**

> **Two numbers here were wrong before this was built, in the same way twice.** It said
> "55,289 **characters** on line 30". That is a *byte* count — `awk length()` in the C
> locale counts bytes, and Python's `len()` gives 54,343 for the same row; the 946-byte
> gap is the em-dashes and `₂`/`→`/`⚠`. And "line 30" was stale by one commit: the row
> moved to line 82 when the index landed above it. Both numbers were carried in three
> documents. **A number quoted in three places is a number nobody re-measures** — the
> same failure as the 30-vs-32 row count above, which is why it is recorded rather than
> quietly corrected.

**The fix is a mechanical split that preserves the content exactly**, and "exactly" is
enforced by construction rather than by care. A table cell is one physical line, so the
*only* transformation applied is choosing which of its separator spaces become newlines:

```
"\n".join(body_lines).replace("\n", " ") == the original cell        # character-for-character
```

The generator asserts that per file before writing anything. It never breaks at the two
double-space sites in the record, and never where the next word would read as a markdown
list, heading, quote or rule at the start of a line.

**Measured, not asserted.** The sha-256 of the 32 original cells and of the 32
reconstructions are both
`96bffdcb896cceafb7985f326b0d9fc186c8617320d3d3d6106e7aabd8c5e658`. The first was computed
by a separate script reading the pre-split file, so those are not the same arithmetic done
twice; the second was recomputed a third time from the files on disk, in the pointer
table's order, after the falsification pass had finished mutating and restoring them. The
log's index half is byte-identical across the change
(`7afb080f431557b349ed5fee33cadf88c57be47b70d60acd2b6ae64e73cb65b2` before and after), and
`git diff --numstat` on the log reads **41 insertions / 33 deletions** — the record
section and nothing else.

**Not one word of record prose was rewritten.** Any version of this that reads as
"condense the record" must be refused: the log's own preamble already establishes that
hand-condensing rows is how findings get dropped, and it is right. **Relocate and point;
never summarise.** The only editorial content added is a four-line back-pointer at the top
of each file.

The record table became a pointer table, and the index was left alone. That is not the
tidiest possible design — merging the two tables would drop a duplicate — but the index
rows are *the ones moved verbatim from `CLAUDE.md`*, and rewriting them to add a column
would make the "moved verbatim, verified line-by-line against `d86d9c8`" claim above false
as written. Rule 2's argument against two indexes turns on *unconditional* load; the log
is not unconditionally loaded, so 32 short pointer rows cost nothing.

> **The gate got the assertion that matters only because the design was reviewed.** The
> plan was to pin a sha-256 of the reconstructed record in the standing test, mirroring
> the phase table. That is right for the phase table, which is frozen forever, and
> **wrong for a living record**: it goes red on the next legitimate append, and the fix
> becomes "bump the hash" — training precisely the reflex this whole document exists to
> prevent. So the digests above are a **one-shot migration proof**, and the standing test
> is structural only. In their place the gate gained **a maximum line length on
> `docs/log/*.md`**, which was missing: without it, a split that moves 54 KB into its own
> file and leaves it on one line is **a relocation, not a discipline** — this document's
> own headline finding, applied to the fix for it.

Going forward, a finished piece of work adds **one index line, one pointer row, and one
file in `docs/log/`**. Four of the gate's assertions hold that together: row-count parity,
pointer↔file parity against the disk, the file's heading being its row's Work cell
verbatim, and the line cap. Each was falsified before being believed — a deleted record
file, an unpointed one, a drifted heading, a file re-joined to one line, and a plan doc
named on only one side each turn it red.

## Result

| | Before | After |
|---|---|---|
| `CLAUDE.md` | 17,715 B | 9,520 B (ceiling 12,000) |
| Share that was a status ledger | 50.2 % | 0 % |
| Indexes over the post-roadmap ledger, loaded unconditionally | 2 | 1 |
| Longest physical line in the record | 54,343 chars | 94 chars (cap 120) |
| Files the record occupies | 1 | 33 (`docs/log/`) |
| Enforcement | honour system (failed in 24 h) | `tests/test_context_budget.py`, 10 tests |

**No record was condensed, summarised, or dropped.** Both tables moved content-verbatim,
verified line-by-line against `d86d9c8`; the phase table is content-pinned from here on;
the split record's 32 cells reconstruct to the same sha-256 as the originals.

Every assertion in the gate was **falsified before being believed** — a re-added status
row, a blown ceiling, an index row deleted from one table only, an edited phase row, a
deleted record file, an unpointed one, a drifted heading, and a record file put back on
one line each turn it red. That mattered: the first version of the parity check passed all
of it while testing nothing (see rule 3 above).
