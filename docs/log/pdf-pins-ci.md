## **Tooling: the PDF-backed citation pins are green-by-skip on CI**

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**FOUND + FIXED 2026-08-11**, while closing out the direction gate. Three assertions in
`tests/test_nitrogen_throttle.py` went red on a tree that changed nothing near them. Cause:
they check a **quoted phrase as a literal substring of `pdftotext` output**, and poppler
hard-wraps — so `"stimulate the decomposition of low-N plant residue" in text` is really an
assertion about *where the extractor broke the line*. A poppler upgrade (25.09.1) re-wrapped
two phrases and truncated the two RothC bibliography entries mid-line. Every phrase is still
in the papers: verified by flattening whitespace, so the fix (`_flat`) **preserves each
assertion's subject exactly** and is not a weakening. ⚠ **The finding is the class, not the
three tests.** `sources/` is gitignored, so on CI every PDF-backed test `skip`s — **CI green
here means nothing was checked**, and the pins can only rot locally, silently, on a
toolchain upgrade nobody connects to them. That is `memory/ci-python-job-red-on-linux.md`
with the arrow reversed (*local* green ≠ CI green there; *CI* green ≠ local green here), and
it applies to all 5 other PDF-backed modules — measured green on this poppler, so the
exposure is recorded rather than assumed. **What was NOT flattened:** the two *absence*
claims ("RothC never mentions nitrogen outside its bibliography") need line structure to
tell an entry from a body sentence, so they keep the exact count of 2 and read one line of
lookahead, with the widening refused in the test's own comment. Also added the length floor
the other extraction tests carry to the one test that lacked it — `_pdf_text` skips only on
a nonzero exit, so a short read would have turned its quotations into vacuous passes.
