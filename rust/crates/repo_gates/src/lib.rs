//! Helpers for the repository's own gates — path resolution, newline-normalised byte
//! counts, and the three tiny scanners the context-budget contract needs.
//!
//! # Why hand-rolled scanners rather than a regex crate
//!
//! The Python original used three regular expressions. Two of them are literal-prefix
//! searches and the third is a character-class run with one negative lookbehind; all three
//! are a few lines each by hand. Pulling a regex dependency into a crate whose whole job is
//! to read four markdown files would be the larger change, and the house style in this
//! workspace is a hand-rolled reader (`simcore` carries zero third-party dependencies and
//! the config crate hand-rolls its own sha-256 and JSON).
//!
//! ⚠ The scanners are therefore **tested against the strings that actually broke them**
//! (`tests/scanners.rs`), not assumed equivalent to the regexes they replace. A port of a
//! regex is exactly the place a silent narrowing hides: the lookbehind below excludes one
//! path prefix, and dropping it would make `test_every_plan_doc_is_indexed` pass on a doc
//! that is only ever named by a memory file.

use std::path::{Path, PathBuf};

/// The repository root — this crate lives at `<root>/rust/crates/repo_gates`.
///
/// Resolved from `CARGO_MANIFEST_DIR` at compile time but used at run time, so a moved
/// crate fails loudly here rather than silently reading nothing.
pub fn repo_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("crates/repo_gates is three levels below the repository root")
        .to_path_buf();
    assert!(
        root.join("CLAUDE.md").is_file(),
        "repo_root() resolved to {} , which has no CLAUDE.md — has this crate moved?",
        root.display()
    );
    root
}

/// Read a file with CRLF collapsed to LF.
///
/// The house convention (see `test_freeze_manifest.py`): the repo is developed on Windows
/// and CI runs Linux, so a raw byte count would differ by one byte per line for no
/// semantic reason.
pub fn read_normalised(path: &Path) -> String {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    raw.replace("\r\n", "\n")
}

/// Newline-normalised byte length.
pub fn normalised_bytes(path: &Path) -> usize {
    read_normalised(path).len()
}

/// The record files, `docs/log/*.md`, sorted by name (the Python original sorts too, and
/// the parity assertions compare sets, so order only matters for message stability).
pub fn record_files(root: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(root.join("docs").join("log"))
        .expect("docs/log/ is missing")
        .map(|e| e.expect("unreadable entry in docs/log/").path())
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .collect();
    files.sort();
    files
}

/// The log's index half and record half, split on their headings.
///
/// Panics with the same question the Python original asks — "has the index moved again?" —
/// because it has moved twice already and a silently-empty section would make every parity
/// assertion below vacuously true.
pub fn log_sections(log: &str) -> (&str, &str) {
    let i = log
        .find("\n## Index")
        .expect("post-roadmap-log.md has no '## Index' section — has the index moved again?");
    let r = log
        .find("\n## The record")
        .expect("post-roadmap-log.md has no '## The record' section");
    assert!(
        r > i,
        "post-roadmap-log.md: '## The record' precedes '## Index'"
    );
    (&log[i..r], &log[r..])
}

/// Table rows, excluding the header and the `|---|` separator.
///
/// Deliberately blind to what a row *names* — that is the whole point of the row-count
/// assertion, which caught the plan-doc comparison being vacuous.
pub fn data_rows(section: &str) -> Vec<&str> {
    section
        .lines()
        .filter(|l| l.starts_with("| ") && !l.starts_with("| Work |"))
        .collect()
}

/// The log's own file name, which never counts as a plan doc it names.
const LOG_SELF: &str = "post-roadmap-log.md";

/// Every `post-roadmap-*.md` named in `text`, excluding the log itself and any hit
/// prefixed by `memory/`.
///
/// The Python original is `(?<!memory/)\b(post-roadmap-[a-z0-9-]+\.md)`. Both exclusions
/// are load-bearing and both are asserted in `tests/scanners.rs`:
///
/// * the **lookbehind** keeps `memory/post-roadmap-direction.md` (a memory file, not a plan
///   doc) out of the set;
/// * the **word boundary** keeps a longer identifier ending in the same text from matching.
pub fn plan_docs(text: &str) -> std::collections::BTreeSet<String> {
    const NEEDLE: &str = "post-roadmap-";
    let bytes = text.as_bytes();
    let mut found = std::collections::BTreeSet::new();
    let mut from = 0usize;
    while let Some(rel) = text[from..].find(NEEDLE) {
        let start = from + rel;
        from = start + NEEDLE.len();

        // `\b` — the character before must not be a word character. `-` and `/` are not
        // word characters, which is why the `memory/` exclusion needs its own test below.
        let boundary = start == 0 || {
            let c = bytes[start - 1] as char;
            !(c.is_ascii_alphanumeric() || c == '_')
        };
        if !boundary {
            continue;
        }
        // `(?<!memory/)` — the lookbehind, applied at the same position as `\b`.
        if text[..start].ends_with("memory/") {
            continue;
        }
        // `[a-z0-9-]+` greedy, then a literal `.md`.
        let mut end = start + NEEDLE.len();
        while end < bytes.len() {
            let c = bytes[end] as char;
            if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' {
                end += 1;
            } else {
                break;
            }
        }
        if end == start + NEEDLE.len() || !text[end..].starts_with(".md") {
            continue;
        }
        let name = &text[start..end + 3];
        if name != LOG_SELF {
            found.insert(name.to_string());
        }
    }
    found
}

/// The record file a pointer row names: `[the record](log/<slug>.md)`.
///
/// The Python original is `\[the record\]\(log/([a-z0-9-]+\.md)\)`.
pub fn record_link(row: &str) -> Option<String> {
    const OPEN: &str = "[the record](log/";
    let start = row.find(OPEN)? + OPEN.len();
    let rest = &row[start..];
    let close = rest.find(')')?;
    let name = &rest[..close];
    let stem = name.strip_suffix(".md")?;
    if stem.is_empty()
        || !stem
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return None;
    }
    Some(name.to_string())
}

/// The user's memory index, which lives in the profile rather than the repo.
///
/// `None` when the home directory is not discoverable; the caller decides what an absent
/// file means (on CI it means the assertion does not run, and says so out loud).
pub fn memory_index() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)?;
    Some(
        home.join(".claude")
            .join("projects")
            .join("M--claud-projects-space-station")
            .join("memory")
            .join("MEMORY.md"),
    )
}
