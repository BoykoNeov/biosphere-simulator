## **Tooling: the record split** (that rule set's own rule 4, taken as its own piece of work)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md).
> Written here directly, under the shape rule 4 established — unlike the 32 files
> beside it, which were moved verbatim out of that table and carry a
> `<!-- record-body -->` marker saying so.

**COMPLETE 2026-08-12**, the deferred half of the context-budget work, taken on the user's
call. **The defect was shape, not size.** `docs/post-roadmap-log.md` held 255,567 bytes of
record in **32 physical lines** — one markdown table row per work item — and the nitrogen
row alone was **54,343 characters on one line**. That defeats every tool that reads the
file: `Grep` returns a 54 KB line as "one match", `Read` cannot page into a row, and
`git diff` rewrites the entire line for a one-word edit. **A record nobody can read is a
worse defect than an index nobody needs**, which is why this outranked anything else the
budget work left open.

**What shipped:** the 32 Status cells are now 32 files in `docs/log/`, one per work item,
and the table is a pointer per row. **Moved verbatim, and measured rather than asserted** —
the transformation is *only* the choice of which separator spaces become newlines, so
`"\n".join(body_lines).replace("\n", " ")` reproduces the original cell character-for-
character by construction. The generator asserts that per file before writing; the
sha-256 of the 32 original cells and of the 32 reconstructions are both
`96bffdcb896cceafb7985f326b0d9fc186c8617320d3d3d6106e7aabd8c5e658`, and the first of those
was computed by a separate script reading the pre-split file, so the two are not the same
arithmetic done twice. The index half of the log is byte-identical across the change
(`7afb080f431557b349ed5fee33cadf88c57be47b70d60acd2b6ae64e73cb65b2` before and after), and
`git diff --numstat` on the log reads **41 insertions / 33 deletions** — the record section
and nothing else.

⚠ **The advisor caught the design about to repeat this project's own headline finding.**
My plan was to pin a sha-256 of the reconstructed record in the standing test, mirroring
`test_phase_table_survived_its_move`. That is right for the phase table, which is frozen
forever, and **wrong for a living record**: the pin goes red on the next legitimate append,
and the fix becomes "bump the hash" — training exactly the reflex the ceiling test exists
to prevent. The digests are therefore a **one-shot migration proof** (here, and in the
commit message), not an assertion. And the standing gate gained the assertion I had
omitted, which is the one that matters most: **a maximum line length on `docs/log/*.md`.**
Without it, a split that moves 54 KB into its own file and leaves it as one line is a
*relocation, not a discipline* — this work item's own predecessor, applied to itself.
Falsified: re-joining any record file to a single line turns it red.

**Two smaller corrections, same shape as the ones before them.** "55,289 characters" was
**bytes**, not characters — `awk length()` in the C locale counts bytes and Python's
`len()` says 54,343 for the same row; the 946-byte gap is the em-dashes and `₂`/`→`/`⚠`.
And "line 30" was stale by one commit: the row moved to line 82 when the index landed
above it. Both numbers were quoted in `docs/context-budget.md` and in the memory file. A
number carried across three documents is a number nobody re-measures.

**What the split deliberately did not do:** condense anything. The log's own preamble
already establishes that hand-condensing rows is how a finding gets silently dropped, and
it is right. Not one word of record prose was rewritten — the only editorial content added
is the four-line back-pointer at the top of each file. **Relocate and point; never
summarise.**

**Cascade:** 33 new files in `docs/log/`, the log's record table replaced by pointers,
`tests/test_context_budget.py` re-pointed from table↔table parity to table↔disk parity
plus the line-length cap, `docs/context-budget.md` rule 4 moved from *diagnosed and not
built* to **BUILT**. No value, golden, manifest or param was touched; `git diff src/` is
empty. `docs/context-budget.md`
