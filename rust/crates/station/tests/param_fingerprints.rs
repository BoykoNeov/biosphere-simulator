//! Step 4a's control, on the real files: the manifests' param fingerprints follow what the
//! files MEAN, not their text (`docs/plans/post-roadmap-meaning-fingerprint.md`).
//!
//! The rule itself is unit-tested in `config::provenance` on a fixture. This file runs it
//! against every frozen param file both manifests span — the 15 biosphere files and the 10
//! station ones — because a fixture cannot show that the real files' shapes (the `{value,
//! unit, source}` entries, `allocation.yaml`'s table, the uncited `demo.yaml`-free census)
//! are all read the way the fixture is.

use config::provenance::param_fingerprints;

/// Every file the two manifests record, by basename.
fn all_frozen_param_files() -> Vec<(&'static str, &'static str)> {
    let mut files = domains::biosphere::params::param_files();
    files.extend(domains::params::param_files());
    files.extend(station::params::param_files());
    assert_eq!(files.len(), 25, "the two censuses are 15 + 10 files");
    files
}

/// `text` with every whole-line comment and blank line removed, and every trailing comment
/// outside a quoted scalar cut. A rough stripper, deliberately not the parser's own: it is
/// the edit an author makes, and the fingerprint must not see it.
fn strip_comments(text: &str) -> String {
    text.lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }
            // Cut a ` #` comment only on lines with no quote, so a `#` inside a cited
            // string is never touched.
            if !line.contains('"') && !line.contains('\'') {
                if let Some(at) = line.find(" #") {
                    return Some(line[..at].trim_end().to_string());
                }
            }
            Some(line.to_string())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn deleting_every_comment_moves_no_frozen_fingerprint() {
    for (name, text) in all_frozen_param_files() {
        let stripped = strip_comments(text);
        assert!(
            stripped.len() < text.len(),
            "{name}: the stripper removed nothing, so this case proves nothing"
        );
        assert_eq!(
            param_fingerprints(&stripped, name).unwrap(),
            param_fingerprints(text, name).unwrap(),
            "{name}: a comment-only edit moved a fingerprint"
        );
    }
}

#[test]
fn a_citation_edit_moves_only_the_sources_fingerprint() {
    let mut cited = 0;
    for (name, text) in all_frozen_param_files() {
        let Some(at) = text.find("source: \"") else {
            continue;
        };
        cited += 1;
        let at = at + "source: \"".len();
        let edited = format!("{}EDITED {}", &text[..at], &text[at..]);
        let (base, moved) = (
            param_fingerprints(text, name).unwrap(),
            param_fingerprints(&edited, name).unwrap(),
        );
        assert_eq!(
            moved.values, base.values,
            "{name}: a citation edit moved the values"
        );
        assert_ne!(
            moved.sources, base.sources,
            "{name}: a citation edit went unseen"
        );
    }
    // Every frozen file cites something; one that did not would be a census change.
    assert_eq!(cited, 25, "a frozen param file carries no quoted source");
}

#[test]
fn a_value_edit_moves_only_the_values_fingerprint() {
    for (name, text) in all_frozen_param_files() {
        // The first `value:` entry, or a table row's first number.
        let key = ["value: ", "fl: "]
            .into_iter()
            .find(|k| {
                text.contains(&format!("\n    {k}")) || text.contains(&format!("\n        {k}"))
            })
            .unwrap_or_else(|| panic!("{name}: no value line to edit"));
        let at = text.find(key).unwrap() + key.len();
        let edited = format!("{}9{}", &text[..at], &text[at..]);
        let (base, moved) = (
            param_fingerprints(text, name).unwrap(),
            param_fingerprints(&edited, name).unwrap(),
        );
        assert_ne!(
            moved.values, base.values,
            "{name}: a value edit went unseen"
        );
        assert_eq!(
            moved.sources, base.sources,
            "{name}: a value edit moved the sources"
        );
    }
}
