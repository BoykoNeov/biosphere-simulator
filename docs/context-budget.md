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

> ⚠ **That file no longer exists.** S6 deleted the Python checker on 2026-08-27; the gate
> now lives — solely — in `rust/crates/repo_gates/tests/context_budget.rs`, where it had
> been mirrored since the reference flip's Stage 3. **Every `tests/test_context_budget.py`
> below is read as naming that file.** The Python paths are kept rather than rewritten
> because they are what the record said when it was written, and this document does not
> edit its own history; see the 2026-09-06 raise for why a deleted copy is a worse kind of
> stale than a drifted one.

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

#### It fired on 2026-08-15, and the remedy was a raise — here is why, measured

The prediction above was accurate: it arrived at **11,925 B over 70 lines (~170 B/line)**.
Decomposing the 1,856 B of growth against the 10,069 B / 62-line / 162 B-per-line baseline:

| cause | growth | share |
|---|---|---|
| **count** — 62 → 70 index lines at the old 162 B/line | +1,271 B | **69 %** |
| **length** — 162 → 170 B/line across 70 lines | +585 B | 31 % |

**Two-thirds of it is more distinct lessons**, which is a long project working as
intended. Merging to satisfy a count would have destroyed distinct findings to buy bytes
— the exact inversion of what the merge remedy is for. So the ceiling went **12,000 →
16,000 B** (~94 lines at budget, ~24 memories of headroom).

⚠ **But the raise shipped with a NEW assertion, not just a restated sentence.** A byte
ceiling cannot tell "the project learned eight new things" from "the hooks grew into
paragraphs" — it fires identically on both, and this document opens with a story about
exactly the second one hiding behind a quiet byte count. So
`MAX_MEMORY_BYTES_PER_LINE = 170` is now **asserted** in
`tests/test_context_budget.py::test_memory_index_ceiling`, and the two bounds have
opposite remedies on purpose:

- the **ceiling** fires on *more memories* → merge related files;
- the **per-line budget** fires on *fatter hooks* → shorten the hooks, pushing detail
  into the memory files.

Raising the ceiling buys more memories; raising the per-line budget buys longer lines,
and only the first is growth. **A restated budget that is only prose owns nothing** —
the same week's other finding (`d8f5583`) is five frozen values in prose superseded six
commits after they were written, while the assertion four lines below them fired on
schedule and was re-pinned.

#### It was raised again on 2026-08-26 — 16,000 → 20,000 — and the raise owed a THIRD bound

**Pre-emptively, while still green.** 15,932 B over 94 lines (169.5 B/line): 67 B under
the ceiling, 0.5 B/line under the budget. The raise was made because S5 batch D's own
index line could not otherwise be written — the ordering the discipline forces, not a
bypass of it.

The decomposition against the 2026-08-15 baseline (11,925 B / 70 lines / 170.4 B/line) is
the cleanest case this gate has produced:

| cause | growth | share |
|---|---|---|
| **count** — 70 → 94 index lines at the old 170.4 B/line | +4,089 B | **102 %** |
| **length** — 170.4 → 169.5 B/line across 94 lines | −82 B | −2 % |

Actual growth +4,007 B. **All of it is count, and the per-line budget did not merely
hold — it improved.** Last time the split was 69/31 and a raise was still right; here
there is no length component to argue about at all.

⚠ **But the two existing bounds have between them stopped being able to see the
documented failure mode, so this raise ships a third.** The per-line budget is a **mean**,
and a mean dilutes with every raise: one 400 B paragraph moves it +3.3 B/line at 70 lines
and +2.0 at 117. Measured the day this shipped, the longest single index line is **239 B
against a 169.5 B mean — 1.41×**. A hook can already be half again the typical one. That
is the 2026-08-15 finding one turn further along: **a mean cannot tell one fat hook from
94 slightly fatter ones**, exactly as a total cannot tell more memories from fatter ones.

So `MAX_MEMORY_INDEX_LINE_BYTES = 240` is asserted, **pinned at the measurement** (239,
plus one byte) rather than set somewhere comfortable above it — a max bound with room in
it rots the way a ceiling with room in it rots. Its remedy is the per-line budget's aimed
at a single line: shorten *that* hook. Like the per-line budget, it is not raised.

⚠ **The honest scope, because two of the five controls measured it and "the new gate went
red" would have been true for the wrong reason.** Appending a 241 B hook *today* reddens
the **mean**, not the new bound — and so does a 239 B hook the new bound would allow,
because the mean has 0.5 B of slack. The new bound fires **alone** only once the index has
slack again: padded to 119 lines, the same fat hook reddens it and nothing else, and the
same 119-line index without the fat hook is green. **Its bite is in the regime this raise
creates**, which is the right place for it and is not the same as biting now.

⚠⚠ **THE RAISE HAS TO BE MADE TWICE, AND THE FIRST ATTEMPT ONLY MADE IT ONCE.**
**[RETRACTED AS AN INSTRUCTION 2026-09-06 — there is only one copy now; see below. The
finding it records still stands; the ceremony it prescribes does not.]** This gate
has **two copies** — `tests/test_context_budget.py` and
`rust/crates/repo_gates/tests/context_budget.rs`, the mirror the reference flip built. The
2026-08-26 raise edited the Python one, with its decomposition, its controls and its new
bound; the Rust one kept `16_000` and went red on the next full workspace run — on the very
first memory line the raise existed to make room for. Nothing was wrong with either gate:
there are simply two copies of one rule, and *a rule with two copies has one that is stale*,
which this repo has logged before on a different subject. The ceiling ceremony is now
itself an instance of it.

Both copies now carry the same three bounds and were controlled **together**: a 241 B hook,
a 239 B hook and 40 padding rows each produce the identical verdict and fire the identical
bound on both sides, and a byte-exact revert returns both to green. When this gate is raised
again, **raise both, and control them side by side** — the numbers live here, and each copy
owes the same numbers rather than its own reasoning.

⚠⚠ **The cadence is itself a measurement, and it is the user's call, not this document's.**
The 2026-08-15 raise bought "~24 memories of headroom". Those 24 index lines arrived in
**eleven days**. 20,000 B is ~118 lines at budget — the same ~24 memories, so the same
~11 days. If the ceremony keeps firing on that period, the finding is no longer "the index
grew"; it is that a fortnightly raise ritual is the wrong instrument for an index growing
this fast. Raised on precedent and **flagged**, not decided.

> **The flag was called correctly, to the day.** The next firing came 2026-09-06 — 11 days,
> for the third time running. See the section below, where it was decided rather than
> flagged again.

#### It was raised a THIRD time on 2026-09-06 — 20,000 → 40,000 — and this time the CADENCE was the finding

The two raises above each bought ~11 days. This one arrived after ~11 more, at **19,981 B
over 119 index lines (167.9 B/line) — 19 bytes of headroom**, which is to say the next
thing the project learned could not be written down. The decomposition against the
2026-08-26 baseline (15,932 B / 94 lines / 169.5 B/line):

| cause | growth | share |
|---|---|---|
| **count** — 94 → 119 index lines at the old 169.5 B/line | +4,237 B | **105 %** |
| **length** — 169.5 → 167.9 B/line across 119 lines | −190 B | −5 % |

Actual growth +4,049 B. **All count, and the per-line budget improved for the second raise
running.** That is now three firings, three all-count decompositions, three times with both
line bounds green.

⚠ **So the finding is about the bound, not about the index: this ceiling has never once
fired on the failure mode it was built for.** Fat hooks are owned by the per-line budget and
the per-line maximum, and those have held every single time. What the total has actually
been measuring, three times over, is **project age** — and answering that fortnightly, 4 KB
at a time, is the ritual this document itself predicted would become the wrong instrument.
The prediction named the period (~11 days) and named the owner (the user). Both were right.

**So it was put to the user as the question three raises had been answering without asking:
how much always-loaded index are you willing to pay for, per session?** With the sizes
priced — 20 KB ≈ 5k tokens today, ~370 B/day measured — the answer on 2026-09-06 was
**40,000 B (~10k tokens, ~2 months of headroom at the measured pace)**: sized to a horizon
instead of to the next fortnight. Recorded as a decision, not as precedent.

##### Two remedies were considered and REFUSED, both with a measurement

- **Merging, the remedy the ceiling's own failure message prescribes, cannot buy its way
  out of this — and the reason is a bound in this same gate.** A merged line has to carry
  the distinguishing terms of every memory it absorbs, or recall for those subjects dies;
  the per-line **maximum is 240 B and the longest line already measures 239 B**. So a merged
  line holds two subjects' terms, not five, and the realistic true-merges on the index
  (the four `s5-batch-*`, the two `python-checker-*`, a handful of genuine pairs) buy
  **629 B — 1.7 days.** ⚠ **The merge remedy and the max-line bound are in direct tension**,
  and at one byte of headroom that tension is live rather than theoretical.

  > ⚠ **That figure was "~1,300–1,800 B, four or five days" when this section was first
  > written, and it was an eyeball, not a measurement — in a document whose own rule 4 says
  > *a number quoted in three places is a number nobody re-measures*, quoted in three
  > places.** Measured afterwards on the actual index: the four `s5-batch-*` lines are
  > 691 B and collapse to one 241 B line (saves 450 B); the two `python-checker-*` lines
  > are 336 B (saves 95 B); the two `stem-only-*` lines are 325 B (saves 84 B). **629 B
  > total, 3.7 memories, 1.7 days at the measured 367 B/day — between a half and a third of
  > what was claimed.** The estimate was wrong in the direction that *flattered the option
  > being refused*, so measuring strengthened the refusal rather than overturning it. That
  > is luck, not method: the number was load-bearing for a decision and was carried
  > unmeasured through three documents first. Merging remains legitimate housekeeping; it is not an answer to a ceiling.
  Doing it *now*, with 20 KB of fresh headroom, would be merging to satisfy a count — the
  exact inversion the 2026-08-15 entry above warns against.

- **Archiving — one index line per finished thread, the ~31 reference-flip slice files
  left on disk unindexed — was proposed and refused.** It is not this document's merge; it
  is this document's headline failure wearing the merge's name. *Deleting an index line is
  not available*: the lines are the matching surface that decides whether a memory is
  recalled at all, so an unindexed file is a lost memory, not a saved one. **A relocation
  is not a discipline**, and an archive that unindexes 31 files is a relocation.

##### The raise owes a bound, and this one is a HOLE, not a new idea

Each raise so far has shipped a new assertion, because a raise widens the regime the
existing bounds are blind in. This one ships
`every_memory_index_line_names_a_file_and_vice_versa`, and its provenance is worth stating
plainly: **it closes a gap the memory side has had since it was written, which the archive
proposal above is what exposed.**

The ceiling's failure message prescribes *merge two files into one file with one line*.
That is two deletions — a file and a line — and doing only one of them fails invisibly:

- a file left on disk with **no index line** is unreachable, and it makes the index
  *smaller*, so **every byte bound in this gate reads the loss as an improvement**;
- an index line naming a **deleted file** is a dead recall target, spending bytes in every
  session to point at nothing.

The docs side has held exactly this invariant since rule 4
(`every_pointer_row_names_a_record_file_and_vice_versa`). The memory side never got it —
which is precisely why the archive design could be proposed at all: **it would have
orphaned 31 files and turned every bound in this file green while doing it.** Measured the
day it shipped: 119 index lines, 119 files, exact parity.

##### Eight controls, each predicted before it was run

The prediction is part of the control here, because at 239/240 and 19 B two bounds were one
byte apart and "it went red" would have been true for the wrong reason.

| control | predicted | verdict | bound fired |
|---|---|---|---|
| A a memory file on disk with no index line | UNREACHABLE | RED | UNREACHABLE |
| B an index line whose file is gone | DANGLING | RED | DANGLING |
| C half a merge: index line deleted, file kept | UNREACHABLE | RED | UNREACHABLE |
| D one 241 B hook appended | MAXLINE **alone** | RED | MAXLINE |
| E one ordinary 168 B memory line | green | GREEN | — |
| F byte-exact revert of the index | green | GREEN | — |
| G control E's line against the **old 20,000 B** ceiling | RED / CEILING | RED | CEILING |
| H index untouched against the **old 20,000 B** ceiling | green (19 B spare) | GREEN | — |

Two of those are load-bearing beyond the new bound. **D fires the max-line bound and
nothing else** — the 2026-08-26 entry above predicted its bite would arrive "in the regime
this raise creates", and this raise created it: at 20,000 B the same hook reddened the
ceiling too. And **G is the counterfactual made a measurement rather than an assertion**:
the raise was *necessary*, not merely convenient — one ordinary memory line was red before
it and green after, with H showing the untouched index was still green, so G isolates the
new line and not some other drift. The index was restored byte-exactly (sha-256
`6494ec66…d024a23b` before and after all eight), and the gate file likewise.

⚠ **One mechanism correction, because this repo distinguishes them.** The new index line
written for this work item was 253 B and was cut to 239 — but it was caught by a
hand-written `assert` in the throwaway measuring script that *mirrors* the bound, not by
the gate, which never saw that line. The bound would have caught it on the next run; it
did not catch it. *A check that would have fired is not a check that fired.*

##### Which bound binds NEXT — the forward accounting, because "~2 months" is a CEILING claim

⚠ The 2026-08-26 entry did this arithmetic forward ("its bite is in the regime this raise
creates") and the first pass of *this* entry did not. Measured after the raise landed:
**20,268 B / 120 index lines / mean 168.90 / longest 239.**

The mean is `total ÷ line count`, so at 120 lines the budget allows 20,400 B — **132 B of
slack, 1.10 B/line.** Solving forward for `k` further lines of hook length `L`:

| next hooks are… | lines before a bound reddens | which bound |
|---|---|---|
| ≤ 169 B (the historical mean) | ~116 | **ceiling** — the ~2 months the raise bought |
| 200 B | 4 | **mean** |
| 239 B (both hooks written by *this* work item) | **1** | **mean** |

**So "~2 months of headroom" is true of the ceiling and only holds if hook length returns
to ~168 B.** This one work item spent 1.1 of the mean's 2.1 B/line slack, because both
lines it added sit at the max-line bound.

⚠⚠ **If the mean is what goes red next, that is NOT this ceremony firing again, and the
remedy is the opposite one: TRIM the fattest hooks, pushing detail into the memory files.
Do not reach for a fifth ceiling raise.** The whole point of keeping three bounds with
opposing remedies is that the failure message tells you which one you are in; read it.

##### The "raise both copies" ceremony is retracted — there is only one copy now

⚠⚠ The 2026-08-26 entry ends "when this gate is raised again, **raise both, and control
them side by side**". **Do not: `tests/test_context_budget.py` was deleted by S6 on
2026-08-27**, and `rust/crates/repo_gates/tests/context_budget.rs` is the whole gate.

The lesson survives its instructions going stale, one turn further along. *A rule with two
copies has one that is stale* — and when the stale copy is **deleted** rather than drifted,
the staleness lands in the **ceremony** rather than in a number, where **nothing goes red on
it**. A wrong constant went red on the next workspace run; a ceremony naming a deleted file
would have been followed, found impossible, and improvised around, and this document is the
only thing that catches that. `CLAUDE.md` was pointing at the same deleted file and was
corrected in the same commit.

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
