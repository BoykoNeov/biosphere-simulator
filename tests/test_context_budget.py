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

The four assertions:

1. ``CLAUDE.md`` is under a hard byte ceiling. Raising ``MAX_CLAUDE_MD_BYTES`` is then a
   deliberate, reviewable, git-visible act — which is exactly what the 2026-08-11
   regrowth never had to be.
2. **No status ledger creeps back in.** ``CLAUDE.md`` must contain no table row naming a
   ``docs/plans/post-roadmap-*.md``. The ceiling alone would let 30 rows return as long
   as something else shrank; this names the specific shape that actually regrew.
3. **Index ↔ record parity** inside ``docs/post-roadmap-log.md``: the plan docs named in
   the index table and in the record table are the same set. This is what stops the
   retirement rule from decaying into a *deletion* rule — a row cannot quietly leave one
   table while the other keeps it.
4. **Completeness:** every ``docs/plans/post-roadmap-*.md`` on disk is named in the log.
   Aimed where the ceiling is blind: a plan doc written and then never indexed is
   invisible to a byte count.

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

import re
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]

CLAUDE_MD = REPO_ROOT / "CLAUDE.md"
LOG = REPO_ROOT / "docs" / "post-roadmap-log.md"
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

# The index table names plan docs bare (``post-roadmap-x.md``) and the record names them
# path-qualified (``docs/plans/post-roadmap-x.md``) — an artefact of the index having
# been moved verbatim from CLAUDE.md rather than rewritten, which is the point. So the
# prefix is optional. Two hits must NOT count: ``memory/post-roadmap-direction.md`` (a
# file, excluded by the lookbehind) and the log's own name (excluded by _plan_docs).
_PLAN_DOC = re.compile(r"(?<!memory/)\b(post-roadmap-[a-z0-9-]+\.md)")
_LOG_SELF = "post-roadmap-log.md"


def _plan_docs(text: str) -> set[str]:
    return {m for m in _PLAN_DOC.findall(text) if m != _LOG_SELF}


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


def test_log_index_and_record_name_the_same_plan_docs() -> None:
    """Parity: retirement must never become silent deletion of one half."""
    index, record = _log_sections()
    in_index = _plan_docs(index)
    in_record = _plan_docs(record)
    assert in_index, "the log's index table names no plan docs at all"
    assert in_index == in_record, (
        f"index/record disagree in {LOG.name}:\n"
        f"  indexed but not recorded: {sorted(in_index - in_record)}\n"
        f"  recorded but not indexed: {sorted(in_record - in_index)}\n"
        "New work appends to BOTH tables — one line to the index, the full row to the "
        "record."
    )


def test_every_plan_doc_is_indexed() -> None:
    """Completeness — the gap a byte ceiling is structurally blind to."""
    on_disk = {p.name for p in PLANS.glob("post-roadmap-*.md")}
    named = _plan_docs(LOG.read_text(encoding="utf-8"))
    assert on_disk <= named, (
        f"plan docs on disk but named nowhere in {LOG.name}: {sorted(on_disk - named)}"
    )
    assert named <= on_disk, (
        f"{LOG.name} points at plan docs that do not exist: {sorted(named - on_disk)}"
    )


def test_phase_table_survived_its_move() -> None:
    """The Phase 0-9 table was retired by *moving* it, verbatim — not by summarising."""
    rows = [
        line
        for line in PHASE_INDEX.read_text(encoding="utf-8").splitlines()
        if re.match(r"^\| [0-9]", line)
    ]
    assert len(rows) == 11, (
        f"docs/phase-index.md has {len(rows)} phase rows, expected 11 "
        "(0, 0.5, 1-9). The move was byte-for-byte; a row going missing means "
        "something was condensed."
    )


def test_memory_index_ceiling() -> None:
    """Green-by-skip on CI, and said out loud in this module's docstring."""
    if not MEMORY_INDEX.exists():
        pytest.skip(
            f"{MEMORY_INDEX} not present (expected on CI — the memory index lives in "
            "the user's profile, not the repo). This assertion did NOT run."
        )
    size = _normalized_bytes(MEMORY_INDEX)
    assert size <= MAX_MEMORY_INDEX_BYTES, (
        f"MEMORY.md is {size} B, over the {MAX_MEMORY_INDEX_BYTES} B ceiling. Its "
        f"lines are the matching surface for recall, so they cannot just be deleted — "
        f"tighten the hooks instead (docs/context-budget.md, rule 2)."
    )
