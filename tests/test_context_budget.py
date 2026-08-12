"""The context budget's paired gate — the ceiling on what loads into *every* session.

The companion to ``docs/context-budget.md`` (the human-readable rules). Every other
contract in this repo has a paired test; this one did not, and it failed inside 24
hours: on 2026-08-11 ``CLAUDE.md`` was cut 213 KB → 14,458 B, and eleven commits later
*the same day* it was back to 17,740 B — ~300 bytes per finished piece of work,
monotonically, with nothing ever retiring. This module exists so that the next such
regrowth is a red test rather than a slow tax nobody re-measures.

**What this gate owns: the ceiling and the index/record parity.** It does *not* judge
whether a row deserved to be retired, or whether a memory file really captured the
lesson — it bounds the blast radius, it does not supply judgement. Same standing the
freeze manifests have (they own *completeness*, the goldens own *values*, neither owns
"is the science right").

The assertions:

1. ``CLAUDE.md`` is under a hard byte ceiling. Raising ``MAX_CLAUDE_MD_BYTES`` is then a
   deliberate, reviewable, git-visible act — which is exactly what the 2026-08-11
   regrowth never had to be.
2. **No status ledger creeps back in.** ``CLAUDE.md`` must contain no table row naming a
   ``docs/plans/post-roadmap-*.md``. The ceiling alone would let 30 rows return as long
   as something else shrank; this names the specific shape that actually regrew.
3. **Index ↔ record parity** inside ``docs/post-roadmap-log.md``, by **row count** —
   what stops the retirement rule from decaying into a *deletion* rule. The obvious
   version of this check (compare the plan docs each table names) is **vacuous for any
   row that names something else**, and the very first row written under the new
   discipline was such a row: deleting it from the index alone was measured green. Row
   counting is blind to what a row names, which is the property that check lacked. Both
   are kept — the set comparison is sharper where it applies.
4. **Pointer ↔ file parity**, since rule 4 moved the record out to one file per work
   item in ``docs/log/``: every pointer row names a file that exists, and every file is
   named by a row. Same job as (3), now spanning the table and the disk.
5. **No record file is one giant line.** The defect rule 4 fixed was *shape*: one work
   item was one physical line of a markdown table, up to 54,343 characters, which
   defeats ``Grep``, ``Read`` and ``git diff`` alike. Without this cap, a split that
   moves the bytes and leaves them on one line is a **relocation, not a discipline** —
   this module's own headline finding, applied to the fix for it.
6. **Completeness:** every ``docs/plans/post-roadmap-*.md`` on disk is named in the log.
   Aimed where the ceiling is blind: a plan doc written and then never indexed is
   invisible to a byte count.
7. **The moved Phase 0-9 table is content-pinned**, because "moved verbatim" is a claim
   about bytes and this repo has been bitten before by claims that were only ever
   re-read, never measured.

**What is deliberately NOT pinned here: the content of the record.** The 32 migrated
files were verified character-for-character against the table they came out of, and both
sha-256 digests are recorded in ``docs/context-budget.md`` and the migration commit —
but as a **one-shot proof**, not an assertion. A content pin is right for the phase
table, which is frozen forever, and wrong for a living record that every work item
appends to: it would go red on the next legitimate row, and the fix would be "bump it",
which trains precisely the reflex this module exists to prevent.

Byte counts are **newline-normalized** (the house convention, see
``test_freeze_manifest.py``): the repo is developed on Windows and CI runs Linux, so a
raw count would differ by one byte per line across platforms for no semantic reason.

⚠ **One assertion here is green-by-skip on CI, deliberately and visibly.** ``MEMORY.md``
lives in the user's ``~/.claude`` profile, not in the repo, so the memory-index
assertion skips wherever that path is absent — every CI run. This repo has already been
bitten once by a check that was green *because it never ran*
(the PDF-backed citation pins), so it is stated here rather than discovered later: **CI
green does not mean the memory index is under its ceiling.** The three in-repo
assertions above are the ones CI actually enforces.
"""

from __future__ import annotations

import hashlib
import re
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]

CLAUDE_MD = REPO_ROOT / "CLAUDE.md"
LOG = REPO_ROOT / "docs" / "post-roadmap-log.md"
RECORDS = REPO_ROOT / "docs" / "log"
PLANS = REPO_ROOT / "docs" / "plans"
PHASE_INDEX = REPO_ROOT / "docs" / "phase-index.md"

MEMORY_INDEX = (
    Path.home()
    / ".claude"
    / "projects"
    / "M--claud-projects-space-station"
    / "memory"
    / "MEMORY.md"
)

# Measured 2026-08-12, immediately after the retirement rule was applied: CLAUDE.md
# 9,520 B, MEMORY.md 9,842 B. The headroom is for a genuine new invariant landing
# without a same-commit ceiling bump — it is NOT budget for status rows (see
# ``test_no_status_ledger_in_claude_md``, which is the assertion that says so).
MAX_CLAUDE_MD_BYTES = 12_000
MAX_MEMORY_INDEX_BYTES = 12_000

# The Phase 0-9 table as it stood in ``d86d9c8:CLAUDE.md``, verified character-for-
# character after the move (see ``test_phase_table_survived_its_move``).
PHASE_TABLE_SHA256 = "5551a414e790ca0cbc7c5f80ad59cd5ac763ecbdfa3a41509b5cb683c864d434"

# The index legitimately carries MORE rows than the record, because an index line may
# point into a record row it shares. Exactly one such pair exists (measured 2026-08-12,
# on this gate's first real run): ``Stem reserves: the "model FORM found" lead`` — a lead
# retracted later the same day — is an index line whose record lives inside "The
# winter-wheat partition backfill", one plan doc
# (``post-roadmap-wheat-partition-backfill.md``) with two index lines. The delta is
# asserted exactly rather than as an inequality, so drift in EITHER direction is red.
INDEX_SURPLUS_ROWS = 1

# The index table names plan docs bare (``post-roadmap-x.md``) and the record names them
# path-qualified (``docs/plans/post-roadmap-x.md``) — an artefact of the index having
# been moved verbatim from CLAUDE.md rather than rewritten, which is the point. So the
# prefix is optional. Two hits must NOT count: ``memory/post-roadmap-direction.md`` (a
# file, excluded by the lookbehind) and the log's own name (excluded by _plan_docs).
_PLAN_DOC = re.compile(r"(?<!memory/)\b(post-roadmap-[a-z0-9-]+\.md)")
_LOG_SELF = "post-roadmap-log.md"

# A pointer row in the record table: ``| <work> | [the record](log/<slug>.md) |``.
_RECORD_LINK = re.compile(r"\[the record\]\(log/([a-z0-9-]+\.md)\)")

# Measured 2026-08-12 on the 33 files the split produced: the longest wrapped line is 94
# characters (the wrapper targets 92 and overshoots only when a single unbreakable token
# is longer — the longest in the record is an 84-character test name). The cap is NOT a
# style rule and must not be read as one; it exists so that "one work item is one
# physical line" — a 54,343-character row — cannot come back.
MAX_RECORD_LINE_CHARS = 120


def _plan_docs(text: str) -> set[str]:
    return {m for m in _PLAN_DOC.findall(text) if m != _LOG_SELF}


def _record_files() -> list[Path]:
    return sorted(RECORDS.glob("*.md"))


def _normalized_bytes(path: Path) -> int:
    """Byte length with CRLF collapsed to LF, so Windows and CI agree."""
    return len(path.read_text(encoding="utf-8").replace("\r\n", "\n").encode("utf-8"))


def _log_sections() -> tuple[str, str]:
    """The log's index half and record half, split on their headings."""
    text = LOG.read_text(encoding="utf-8").replace("\r\n", "\n")
    index_head = "\n## Index"
    record_head = "\n## The record"
    i, r = text.find(index_head), text.find(record_head)
    assert i != -1, f"{LOG.name} has no '## Index' section — has the index moved again?"
    assert r > i, f"{LOG.name} has no '## The record' section after the index"
    return text[i:r], text[r:]


def _data_rows(section: str) -> list[str]:
    """Table rows, excluding the header and the ``|---|`` separator.

    Deliberately blind to what a row *names* — that is the whole point. See
    ``test_log_index_and_record_have_the_same_row_count``.
    """
    return [
        line
        for line in section.splitlines()
        if line.startswith("| ") and not line.startswith("| Work |")
    ]


def test_claude_md_ceiling() -> None:
    """The always-loaded map stays small enough not to be worth re-measuring."""
    size = _normalized_bytes(CLAUDE_MD)
    assert size <= MAX_CLAUDE_MD_BYTES, (
        f"CLAUDE.md is {size} B, over the {MAX_CLAUDE_MD_BYTES} B ceiling. It is "
        f"loaded unconditionally, so this is a tax on every task including the ones "
        f"it cannot help. Retire something (docs/context-budget.md, rule 1) rather "
        f"than raising the ceiling reflexively — it last rose by accretion, 300 B at "
        f"a time, and that is the failure this test exists to make loud."
    )


def test_no_status_ledger_in_claude_md() -> None:
    """The specific shape that regrew: a per-work-item row pointing at a plan doc."""
    offenders = [
        line
        for line in CLAUDE_MD.read_text(encoding="utf-8").splitlines()
        if line.lstrip().startswith("|") and _PLAN_DOC.search(line)
    ]
    assert not offenders, (
        "CLAUDE.md has regrown a post-roadmap status ledger:\n  "
        + "\n  ".join(offenders)
        + "\nA finished piece of work earns a line in the log's index, a row in the "
        "log's record, and a memory file — not a row in the always-loaded map."
    )


def test_log_index_and_record_have_the_same_row_count() -> None:
    """Parity that actually holds: same number of rows, whatever they name.

    The plan-doc check below is **not** sufficient and was caught being vacuous on the
    first row written under the new discipline. Several rows point at something that is
    not a plan doc — ``docs/test-suite-runtime.md``,
    ``tests/test_authoring_export_fidelity.py``, ``docs/context-budget.md`` — so for
    those, both sides contribute nothing to the plan-doc sets and parity holds *because
    neither table mentions them*. Deleting such a row from one table alone was measured
    green. Counting rows is blind to what a row names, which is exactly the property
    the plan-doc check lacks.
    """
    index, record = _log_sections()
    n_index, n_record = len(_data_rows(index)), len(_data_rows(record))
    assert n_index - n_record == INDEX_SURPLUS_ROWS, (
        f"{LOG.name}: the index table has {n_index} rows, the record table has "
        f"{n_record} — a surplus of {n_index - n_record}, expected "
        f"{INDEX_SURPLUS_ROWS}. New work appends to BOTH tables (one line to the "
        f"index, the full row to the record), and retirement never removes a row from "
        f"only one side. If this is a genuine new many-to-one — an index line whose "
        f"record lives inside another row — raise INDEX_SURPLUS_ROWS deliberately and "
        f"name the pair in the comment there, the same way the ceiling is raised."
    )


def test_every_pointer_row_names_a_record_file_and_vice_versa() -> None:
    """Rule 4 moved the record to one file per work item; the table is now pointers.

    Same job as the row count above, but spanning the table and the disk — a record file
    deleted, renamed, or written and never pointed at is invisible to a row count.
    """
    _, record = _log_sections()
    rows = _data_rows(record)
    pointed = [_RECORD_LINK.search(row) for row in rows]
    unlinked = [row[:80] for row, m in zip(rows, pointed, strict=True) if m is None]
    assert not unlinked, (
        f"record-table rows in {LOG.name} with no `[the record](log/...)` link:\n  "
        + "\n  ".join(unlinked)
    )
    named = {m.group(1) for m in pointed if m is not None}
    on_disk = {p.name for p in _record_files()}
    assert named == on_disk, (
        f"the record table and docs/log/ disagree:\n"
        f"  pointed at but missing: {sorted(named - on_disk)}\n"
        f"  on disk but pointed at by nothing: {sorted(on_disk - named)}\n"
        "New work adds one index line, one pointer row, AND one file in docs/log/."
    )


def test_each_record_file_is_headed_by_its_own_row() -> None:
    """The file's ``##`` heading is the pointer row's Work cell, verbatim.

    Two jobs. It keeps a file and its row from drifting into describing different work —
    a pointer table whose labels no longer match what they point at is worse than no
    table. And it is what licenses the heading's exemption from the line cap below: a
    heading cannot be wrapped, so it is only safe to exempt because it is a *copy of a
    bounded cell* rather than free prose.
    """
    _, record = _log_sections()
    mismatches: list[str] = []
    for row in _data_rows(record):
        m = _RECORD_LINK.search(row)
        if m is None:
            continue
        work = row[2:].split(" | ")[0]
        path = RECORDS / m.group(1)
        head = path.read_text(encoding="utf-8").split("\n", 1)[0].rstrip()
        if head != f"## {work}":
            mismatches.append(f"{path.name}\n    row:  {work}\n    head: {head}")
    assert not mismatches, (
        "record file headings that are not their row's Work cell:\n  "
        + "\n  ".join(mismatches)
    )


def test_no_record_file_is_one_giant_line() -> None:
    """The shape defect rule 4 fixed, kept fixed.

    The record used to be 32 rows of a markdown table — one work item per *physical
    line*, the longest 54,343 characters. `Grep` returns such a line as "one match",
    `Read` cannot page into it, and `git diff` rewrites the whole line for a one-word
    edit. Moving those bytes into their own files without breaking the lines would have
    been a **relocation, not a discipline**: the same failure this module documents, in
    the fix for it. So the cap is asserted, not assumed.
    """
    offenders: list[str] = []
    for path in _record_files():
        lines = path.read_text(encoding="utf-8").replace("\r\n", "\n").split("\n")
        # Line 1 is the ``##`` heading — unwrappable by construction, and pinned to its
        # pointer row by ``test_each_record_file_is_headed_by_its_own_row``, which is
        # what makes skipping it safe rather than a hole.
        for n, line in enumerate(lines[1:], 2):
            if len(line) > MAX_RECORD_LINE_CHARS:
                offenders.append(f"{path.name}:{n} is {len(line)} chars")
    assert not offenders, (
        f"record files over the {MAX_RECORD_LINE_CHARS}-char line cap:\n  "
        + "\n  ".join(offenders)
        + "\nThis is not a style rule. One work item per physical line is the defect "
        "rule 4 removed; wrap the prose instead of raising the cap."
    )


def test_log_index_and_record_name_the_same_plan_docs() -> None:
    """The sharper half of parity, for the rows that do name a plan doc.

    Re-pointed by rule 4: the record side is now the union of the files in ``docs/log/``
    rather than a column of the table.
    """
    index, _ = _log_sections()
    in_index = _plan_docs(index)
    in_record = _plan_docs(
        "\n".join(p.read_text(encoding="utf-8") for p in _record_files())
    )
    assert in_index, "the log's index table names no plan docs at all"
    assert in_index == in_record, (
        f"the log's index and docs/log/ disagree:\n"
        f"  indexed but not recorded: {sorted(in_index - in_record)}\n"
        f"  recorded but not indexed: {sorted(in_record - in_index)}\n"
        "New work adds one index line naming its plan doc AND one record file naming "
        "the same one."
    )


def test_every_plan_doc_is_indexed() -> None:
    """Completeness — the gap a byte ceiling is structurally blind to."""
    on_disk = {p.name for p in PLANS.glob("post-roadmap-*.md")}
    named = _plan_docs(
        "\n".join(
            [LOG.read_text(encoding="utf-8")]
            + [p.read_text(encoding="utf-8") for p in _record_files()]
        )
    )
    assert on_disk <= named, (
        f"plan docs on disk but named nowhere in {LOG.name}: {sorted(on_disk - named)}"
    )
    assert named <= on_disk, (
        f"{LOG.name} points at plan docs that do not exist: {sorted(named - on_disk)}"
    )


def test_phase_table_survived_its_move() -> None:
    """The Phase 0-9 table was retired by *moving* it — not by summarising.

    A row count would pass on 11 *rewritten* rows, so this pins content. The hash was
    taken after checking the moved table character-for-character against the table in
    ``d86d9c8:CLAUDE.md`` (the commit before the move) — all 13 lines identical, line
    endings normalized to the destination file. That check is the reason the hash is
    trustworthy; the hash is what keeps it true from here on.

    Every roadmap phase is COMPLETE and none will change again, so unlike the log's
    index this content is genuinely frozen and a pin costs nothing in maintenance. If
    this fails, a phase row was edited — which is an unfreeze-shaped event, not a typo
    fix.
    """
    rows = [
        line
        for line in PHASE_INDEX.read_text(encoding="utf-8")
        .replace("\r\n", "\n")
        .split("\n")
        if line.startswith("|")
    ]
    assert len(rows) == 13, (
        f"docs/phase-index.md has {len(rows)} table lines, expected 13 "
        "(header + separator + phases 0, 0.5, 1-9)."
    )
    digest = hashlib.sha256("\n".join(rows).encode("utf-8")).hexdigest()
    assert digest == PHASE_TABLE_SHA256, (
        f"the moved Phase 0-9 table has changed (sha-256 {digest}, pinned "
        f"{PHASE_TABLE_SHA256}). It was moved verbatim from CLAUDE.md and every row "
        f"reads COMPLETE — an edit here means content was rewritten, which is what "
        f"'moved verbatim' was supposed to rule out."
    )


def test_memory_index_ceiling() -> None:
    """Green-by-skip on CI, and said out loud in this module's docstring."""
    if not MEMORY_INDEX.exists():
        pytest.skip(
            f"{MEMORY_INDEX} not present (expected on CI — the memory index lives in "
            "the user's profile, not the repo). This assertion did NOT run."
        )
    size = _normalized_bytes(MEMORY_INDEX)
    lines = [
        ln
        for ln in MEMORY_INDEX.read_text(encoding="utf-8").splitlines()
        if ln.startswith("- [")
    ]
    assert size <= MAX_MEMORY_INDEX_BYTES, (
        f"MEMORY.md is {size} B over {len(lines)} lines, past the "
        f"{MAX_MEMORY_INDEX_BYTES} B ceiling ({size // max(len(lines), 1)} B/line). "
        f"Its lines are the matching surface for recall, so deleting one makes a "
        f"memory unreachable — the remedy is to MERGE related memory files (two files "
        f"become one file with one line, the detail preserved inside), which is the "
        f"same move as the docs side, not a condense. Raising the ceiling is allowed "
        f"but must come with a restated per-line budget. See docs/context-budget.md, "
        f"'the memory side'."
    )
