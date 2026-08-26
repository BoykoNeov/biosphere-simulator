//! The context budget's paired gate — the ceiling on what loads into *every* session.
//!
//! The companion to `docs/context-budget.md` (the human-readable rules), ported from
//! `tests/test_context_budget.py` by the reference flip's Stage 3 (§5y decision 1). Every
//! other contract in this repo has a paired test; this one did not, and it failed inside 24
//! hours: on 2026-08-11 `CLAUDE.md` was cut 213 KB → 14,458 B and eleven commits later *the
//! same day* it was back to 17,740 B — ~300 bytes per finished piece of work, monotonically,
//! with nothing ever retiring.
//!
//! **What this gate owns: the ceiling and the index/record parity.** It does *not* judge
//! whether a row deserved to be retired or whether a memory file really captured the lesson
//! — it bounds the blast radius, it does not supply judgement. The same standing the freeze
//! manifests have (they own *completeness*, the goldens own *values*, neither owns "is the
//! science right").
//!
//! ⚠ **One assertion here does not run when the memory index is absent, deliberately and
//! visibly.** `MEMORY.md` lives in the user's profile, not the repo, so on CI it is missing.
//! Rust has no `skip`, so the test prints to stderr and returns — the analogue of pytest's
//! *loud* skip rather than `#[ignore]`, which would be opt-in and therefore silent locally
//! too (the lesson §5u recorded). **CI green does not mean the memory index is under its
//! ceiling.**
//!
//! ⚠ **What is deliberately NOT pinned: the content of the record.** A content pin is right
//! for the frozen phase table and wrong for a living record that every work item appends to
//! — it would go red on the next legitimate row, and the fix would be "bump it", which
//! trains precisely the reflex this module exists to prevent.

use std::collections::BTreeSet;

use repo_gates::{
    data_rows, log_sections, memory_index, normalised_bytes, plan_docs, read_normalised,
    record_files, record_link, repo_root,
};

/// Measured 2026-08-12, immediately after the retirement rule was applied. The headroom is
/// for a genuine new invariant landing without a same-commit ceiling bump — it is NOT
/// budget for status rows (see `no_status_ledger_in_claude_md`, the assertion that says so).
const MAX_CLAUDE_MD_BYTES: usize = 12_000;

/// ⚠ RAISED 2026-08-15, 12_000 → 16_000, with the per-line budget restated in the same
/// commit. Two-thirds of that growth was *more distinct lessons* (62 → 70 lines) and one
/// third was line drift — so the ceiling buys more memories and the per-line budget below
/// owns the half the discipline actually controls.
///
/// ⚠⚠ RAISED AGAIN 2026-08-26, 16_000 → 20_000 — **and this copy was MISSED for a whole
/// commit.** The raise landed in `tests/test_context_budget.py` with its decomposition, its
/// controls and its new bound, and this mirror kept the old ceiling; it went red on the
/// next workspace run, on the first memory line the raise existed to make room for. Nothing
/// is wrong with either gate — there are simply TWO COPIES of one rule, and a raise that
/// edits one of them is half a raise. *A rule with two copies has one that is stale*, and
/// this repo has logged that before; the ceiling ceremony is now itself an instance.
///
/// The reasoning is not restated here — it belongs in one place. `docs/context-budget.md`
/// ("the memory side") carries the decomposition (count +4,089 B, 102 %; length −82 B,
/// −2 %), the five controls and the cadence note. What this file owes is the SAME NUMBERS,
/// which is what being a mirror means.
const MAX_MEMORY_INDEX_BYTES: usize = 20_000;
const MAX_MEMORY_BYTES_PER_LINE: usize = 170;

/// ⚠ The THIRD bound, added 2026-08-26 alongside the raise. The per-line budget is a MEAN,
/// and a mean dilutes with every raise — one 400 B paragraph moves it +3.3 B/line at 70
/// lines and +2.0 at 117. Measured the day this shipped: longest index line 239 B against a
/// 169.5 B mean, 1.41×. So a mean cannot tell ONE fat hook from 94 slightly fatter ones,
/// exactly as a total cannot tell more memories from fatter ones. Pinned AT the measurement
/// (239 → 240) rather than above it, and never raised — same standing as the budget above.
const MAX_MEMORY_INDEX_LINE_BYTES: usize = 240;

/// The Phase 0–9 table as it stood in `d86d9c8:CLAUDE.md`, verified character-for-character
/// after the move.
const PHASE_TABLE_SHA256: &str = "5551a414e790ca0cbc7c5f80ad59cd5ac763ecbdfa3a41509b5cb683c864d434";

/// The index legitimately carries MORE rows than the record, because an index line may
/// point into a record row it shares. Exactly one such pair exists (measured 2026-08-12):
/// the retracted stem-reserve "model FORM found" lead, whose record lives inside the
/// winter-wheat partition backfill. Asserted exactly rather than as an inequality, so drift
/// in EITHER direction is red.
const INDEX_SURPLUS_ROWS: usize = 1;

/// Measured 2026-08-12 on the 33 files the split produced: the longest wrapped line is 94
/// characters. The cap is NOT a style rule and must not be read as one; it exists so that
/// "one work item is one physical line" — a 54,343-character row — cannot come back.
///
/// ⚠ Counted in **characters, not bytes**, matching Python's `len(line)`. The record files
/// are full of `⚠` and `—`; a byte count would silently tighten the cap by a third on the
/// very lines that carry the findings.
const MAX_RECORD_LINE_CHARS: usize = 120;

#[test]
fn claude_md_ceiling() {
    let root = repo_root();
    let size = normalised_bytes(&root.join("CLAUDE.md"));
    assert!(
        size <= MAX_CLAUDE_MD_BYTES,
        "CLAUDE.md is {size} B, over the {MAX_CLAUDE_MD_BYTES} B ceiling. It is loaded \
         unconditionally, so this is a tax on every task including the ones it cannot help. \
         Retire something (docs/context-budget.md, rule 1) rather than raising the ceiling \
         reflexively — it last rose by accretion, 300 B at a time, and that is the failure \
         this test exists to make loud."
    );
}

#[test]
fn no_status_ledger_in_claude_md() {
    let root = repo_root();
    let text = read_normalised(&root.join("CLAUDE.md"));
    let offenders: Vec<&str> = text
        .lines()
        .filter(|l| l.trim_start().starts_with('|') && !plan_docs(l).is_empty())
        .collect();
    assert!(
        offenders.is_empty(),
        "CLAUDE.md has regrown a post-roadmap status ledger:\n  {}\nA finished piece of work \
         earns a line in the log's index, a row in the log's record, and a memory file — not \
         a row in the always-loaded map.",
        offenders.join("\n  ")
    );
}

#[test]
fn log_index_and_record_have_the_same_row_count() {
    // Parity that actually holds: same number of rows, whatever they name. The plan-doc
    // check below is NOT sufficient and was caught being vacuous on the first row written
    // under the new discipline — several rows point at something that is not a plan doc, so
    // both sides contribute nothing and parity holds because neither table mentions them.
    let root = repo_root();
    let log = read_normalised(&root.join("docs").join("post-roadmap-log.md"));
    let (index, record) = log_sections(&log);
    let (n_index, n_record) = (data_rows(index).len(), data_rows(record).len());
    assert!(
        n_index >= n_record,
        "post-roadmap-log.md: the record table has more rows ({n_record}) than the index \
         ({n_index}) — new work adds one line to BOTH."
    );
    assert_eq!(
        n_index - n_record,
        INDEX_SURPLUS_ROWS,
        "post-roadmap-log.md: the index table has {n_index} rows, the record table has \
         {n_record}, expected a surplus of {INDEX_SURPLUS_ROWS}. New work appends to BOTH \
         tables, and retirement never removes a row from only one side. If this is a genuine \
         new many-to-one, raise INDEX_SURPLUS_ROWS deliberately and name the pair in the \
         comment there, the same way the ceiling is raised."
    );
}

#[test]
fn every_pointer_row_names_a_record_file_and_vice_versa() {
    // Same job as the row count, but spanning the table and the disk — a record file
    // deleted, renamed, or written and never pointed at is invisible to a row count.
    let root = repo_root();
    let log = read_normalised(&root.join("docs").join("post-roadmap-log.md"));
    let (_, record) = log_sections(&log);
    let rows = data_rows(record);

    let unlinked: Vec<String> = rows
        .iter()
        .filter(|row| record_link(row).is_none())
        .map(|row| row.chars().take(80).collect())
        .collect();
    assert!(
        unlinked.is_empty(),
        "record-table rows with no `[the record](log/...)` link:\n  {}",
        unlinked.join("\n  ")
    );

    let named: BTreeSet<String> = rows.iter().filter_map(|row| record_link(row)).collect();
    let on_disk: BTreeSet<String> = record_files(&root)
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        named,
        on_disk,
        "the record table and docs/log/ disagree:\n  pointed at but missing: {:?}\n  on disk \
         but pointed at by nothing: {:?}\nNew work adds one index line, one pointer row, AND \
         one file in docs/log/.",
        named.difference(&on_disk).collect::<Vec<_>>(),
        on_disk.difference(&named).collect::<Vec<_>>(),
    );
}

#[test]
fn each_record_file_is_headed_by_its_own_row() {
    // Two jobs. It keeps a file and its row from drifting into describing different work —
    // a pointer table whose labels no longer match what they point at is worse than no
    // table. And it is what licenses the heading's exemption from the line cap below: a
    // heading cannot be wrapped, so exempting it is only safe because it is a copy of a
    // bounded cell rather than free prose.
    let root = repo_root();
    let log = read_normalised(&root.join("docs").join("post-roadmap-log.md"));
    let (_, record) = log_sections(&log);

    let mut mismatches: Vec<String> = Vec::new();
    for row in data_rows(record) {
        let Some(name) = record_link(row) else {
            continue;
        };
        let work = row[2..].split(" | ").next().unwrap_or_default();
        let path = root.join("docs").join("log").join(&name);
        let head = read_normalised(&path)
            .split('\n')
            .next()
            .unwrap_or_default()
            .trim_end()
            .to_string();
        if head != format!("## {work}") {
            mismatches.push(format!("{name}\n    row:  {work}\n    head: {head}"));
        }
    }
    assert!(
        mismatches.is_empty(),
        "record file headings that are not their row's Work cell:\n  {}",
        mismatches.join("\n  ")
    );
}

#[test]
fn no_record_file_is_one_giant_line() {
    // The shape defect rule 4 fixed, kept fixed. The record used to be 32 rows of a markdown
    // table — one work item per PHYSICAL line, the longest 54,343 characters. Moving those
    // bytes into their own files without breaking the lines would have been a relocation,
    // not a discipline: the same failure this module documents, in the fix for it.
    let root = repo_root();
    let mut offenders: Vec<String> = Vec::new();
    for path in record_files(&root) {
        let text = read_normalised(&path);
        // Line 1 is the `##` heading — unwrappable by construction, and pinned to its
        // pointer row by `each_record_file_is_headed_by_its_own_row`, which is what makes
        // skipping it safe rather than a hole.
        for (n, line) in text.split('\n').enumerate().skip(1) {
            let chars = line.chars().count();
            if chars > MAX_RECORD_LINE_CHARS {
                let name = path.file_name().unwrap().to_string_lossy();
                offenders.push(format!("{name}:{} is {chars} chars", n + 1));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "record files over the {MAX_RECORD_LINE_CHARS}-char line cap:\n  {}\nThis is not a \
         style rule. One work item per physical line is the defect rule 4 removed; wrap the \
         prose instead of raising the cap.",
        offenders.join("\n  ")
    );
}

#[test]
fn log_index_and_record_name_the_same_plan_docs() {
    // The sharper half of parity, for the rows that do name a plan doc. The record side is
    // the union of the files in docs/log/ rather than a column of the table.
    let root = repo_root();
    let log = read_normalised(&root.join("docs").join("post-roadmap-log.md"));
    let (index, _) = log_sections(&log);
    let in_index = plan_docs(index);

    let joined: String = record_files(&root)
        .iter()
        .map(|p| read_normalised(p))
        .collect::<Vec<_>>()
        .join("\n");
    let in_record = plan_docs(&joined);

    assert!(
        !in_index.is_empty(),
        "the log's index table names no plan docs at all"
    );
    assert_eq!(
        in_index,
        in_record,
        "the log's index and docs/log/ disagree:\n  indexed but not recorded: {:?}\n  recorded \
         but not indexed: {:?}\nNew work adds one index line naming its plan doc AND one \
         record file naming the same one.",
        in_index.difference(&in_record).collect::<Vec<_>>(),
        in_record.difference(&in_index).collect::<Vec<_>>(),
    );
}

#[test]
fn every_plan_doc_is_indexed() {
    // Completeness — the gap a byte ceiling is structurally blind to. A plan doc written and
    // then never indexed is invisible to a byte count.
    let root = repo_root();
    let plans = root.join("docs").join("plans");
    let on_disk: BTreeSet<String> = std::fs::read_dir(&plans)
        .expect("docs/plans/ is missing")
        .map(|e| e.expect("unreadable entry in docs/plans/").file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .filter(|n| n.starts_with("post-roadmap-") && n.ends_with(".md"))
        .collect();

    let mut corpus = read_normalised(&root.join("docs").join("post-roadmap-log.md"));
    for path in record_files(&root) {
        corpus.push('\n');
        corpus.push_str(&read_normalised(&path));
    }
    let named = plan_docs(&corpus);

    assert!(
        on_disk.is_subset(&named),
        "plan docs on disk but named nowhere in post-roadmap-log.md: {:?}",
        on_disk.difference(&named).collect::<Vec<_>>()
    );
    assert!(
        named.is_subset(&on_disk),
        "post-roadmap-log.md points at plan docs that do not exist: {:?}",
        named.difference(&on_disk).collect::<Vec<_>>()
    );
}

#[test]
fn phase_table_survived_its_move() {
    // A row count would pass on 11 REWRITTEN rows, so this pins content. Every roadmap phase
    // is COMPLETE and none will change again, so unlike the log's index this content is
    // genuinely frozen and a pin costs nothing in maintenance. A failure here means a phase
    // row was edited — an unfreeze-shaped event, not a typo fix.
    let root = repo_root();
    let text = read_normalised(&root.join("docs").join("phase-index.md"));
    let rows: Vec<&str> = text.split('\n').filter(|l| l.starts_with('|')).collect();
    assert_eq!(
        rows.len(),
        13,
        "docs/phase-index.md has {} table lines, expected 13 (header + separator + phases 0, \
         0.5, 1-9).",
        rows.len()
    );
    let digest = config::provenance::sha256_hex(rows.join("\n").as_bytes());
    assert_eq!(
        digest, PHASE_TABLE_SHA256,
        "the moved Phase 0-9 table has changed. It was moved verbatim from CLAUDE.md and \
         every row reads COMPLETE — an edit here means content was rewritten, which is what \
         'moved verbatim' was supposed to rule out."
    );
}

#[test]
fn memory_index_ceiling() {
    // ⚠ Does not run when the memory index is absent — every CI run. Said out loud rather
    // than discovered later: this repo has already been bitten by a check that was green
    // BECAUSE IT NEVER RAN (the PDF-backed citation pins).
    let Some(path) = memory_index() else {
        eprintln!(
            "memory_index_ceiling: no HOME/USERPROFILE, so the memory index path is not \
             resolvable. THIS ASSERTION DID NOT RUN."
        );
        return;
    };
    if !path.is_file() {
        eprintln!(
            "memory_index_ceiling: {} not present (expected on CI — the memory index lives in \
             the user's profile, not the repo). THIS ASSERTION DID NOT RUN.",
            path.display()
        );
        return;
    }

    let text = read_normalised(&path);
    let size = text.len();
    let lines = text.lines().filter(|l| l.starts_with("- [")).count().max(1);
    assert!(
        size <= MAX_MEMORY_INDEX_BYTES,
        "MEMORY.md is {size} B over {lines} lines, past the {MAX_MEMORY_INDEX_BYTES} B ceiling \
         ({} B/line). Its lines are the matching surface for recall, so deleting one makes a \
         memory unreachable — the remedy is to MERGE related memory files (two files become \
         one file with one line, the detail preserved inside), not to condense. Raising the \
         ceiling is allowed but must come with a restated per-line budget.",
        size / lines
    );

    // ⚠ THE HALF THE DISCIPLINE ACTUALLY OWNS. The ceiling alone cannot tell "the project
    // learned 8 new things" (legitimate) from "the hooks grew into paragraphs" (the
    // documented failure mode) — it fires identically on both.
    let per_line = size as f64 / lines as f64;
    assert!(
        per_line <= MAX_MEMORY_BYTES_PER_LINE as f64,
        "MEMORY.md averages {per_line:.1} B over {lines} index lines, past the \
         {MAX_MEMORY_BYTES_PER_LINE} B/line budget — the hooks are growing into paragraphs, \
         which is the failure the byte ceiling is blind to. The remedy here is the OPPOSITE \
         of the ceiling's: SHORTEN the hooks, pushing detail into the memory files. Do NOT \
         raise this to make room — raising the ceiling buys more memories, raising this buys \
         longer lines, and only the first is growth."
    );

    // ⚠ The third bound. Both assertions above are blind to a SINGLE hook grown into a
    // paragraph: a ceiling sees only the total, and an average dilutes one long line
    // across every short one. Its remedy is the per-line budget's, aimed at one line.
    let longest = text
        .lines()
        .filter(|l| l.starts_with("- ["))
        .map(|l| l.len())
        .max()
        .unwrap_or(0);
    assert!(
        longest <= MAX_MEMORY_INDEX_LINE_BYTES,
        "MEMORY.md's longest index line is {longest} B, past the          {MAX_MEMORY_INDEX_LINE_BYTES} B per-line maximum — one hook has grown into a          paragraph, which BOTH bounds above are blind to. The remedy is that single hook:          SHORTEN it, pushing the detail into its memory file, and keep the distinguishing          terms — they are the recall matching surface, so trimming is not condensing. Do          NOT raise this bound; it is pinned at a measurement on purpose."
    );
}
