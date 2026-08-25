//! The hand-rolled replacements for the Python original's three regular expressions,
//! tested against the strings that make them non-trivial.
//!
//! ⚠ **Why this file exists at all.** The gates in `context_budget.rs` are set comparisons.
//! A scanner that silently matches *less* than the regex it replaced makes both sides of
//! every one of those comparisons smaller, so the parity assertions still pass — the port
//! would read green while checking nothing. That is the failure mode Stage 3 keeps finding
//! (a gate inert by construction), and a set comparison cannot see it from the inside. So
//! the scanners are pinned here, on inputs chosen for the two exclusions that are easy to
//! drop: the `memory/` lookbehind and the word boundary.

use repo_gates::{plan_docs, record_link};

fn set(items: &[&str]) -> std::collections::BTreeSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

#[test]
fn plan_docs_finds_bare_and_path_qualified_names() {
    // The index names plan docs bare and the record names them path-qualified — an artefact
    // of the index having been moved verbatim from CLAUDE.md rather than rewritten, which is
    // the point. Both must land in the same set or the two halves can never agree.
    assert_eq!(
        plan_docs("see `post-roadmap-stem-reserves.md` and docs/plans/post-roadmap-soil-layers.md"),
        set(&[
            "post-roadmap-soil-layers.md",
            "post-roadmap-stem-reserves.md"
        ])
    );
}

#[test]
fn plan_docs_excludes_the_memory_prefixed_hit() {
    // ⚠ The lookbehind. `memory/post-roadmap-direction.md` is a MEMORY FILE, not a plan doc;
    // counting it would put a name on the record side that can never appear on the index
    // side, turning `log_index_and_record_name_the_same_plan_docs` permanently red — or, if
    // someone "fixed" that by indexing it, permanently wrong.
    assert!(plan_docs("memory/post-roadmap-direction.md").is_empty());
    // ...and the same name unprefixed is a real hit, so the exclusion is about the prefix
    // and not about the name.
    assert_eq!(
        plan_docs("post-roadmap-direction.md"),
        set(&["post-roadmap-direction.md"])
    );
    // Only the immediate prefix is excluded — a path that merely CONTAINS "memory" is a
    // normal hit. Pinned because "does the exclusion anchor at the match?" is exactly the
    // kind of thing a rewrite gets subtly wrong.
    assert_eq!(
        plan_docs("docs/memory-notes/post-roadmap-direction.md"),
        set(&["post-roadmap-direction.md"])
    );
}

#[test]
fn plan_docs_excludes_the_log_itself() {
    // The log names its own filename in its prose. Counting it would put a name in the index
    // set that no record file carries.
    assert!(plan_docs("this file, post-roadmap-log.md, is the index").is_empty());
}

#[test]
fn plan_docs_respects_the_word_boundary() {
    // `\b` — a longer identifier that merely ENDS in the needle is not a plan doc. `-` and
    // `/` are not word characters, so they stay boundaries; a letter or digit does not.
    assert!(plan_docs("xpost-roadmap-thing.md").is_empty());
    assert!(plan_docs("9post-roadmap-thing.md").is_empty());
    assert!(plan_docs("_post-roadmap-thing.md").is_empty());
    assert_eq!(
        plan_docs("-post-roadmap-thing.md"),
        set(&["post-roadmap-thing.md"])
    );
}

#[test]
fn plan_docs_requires_a_slug_and_the_md_suffix() {
    // `[a-z0-9-]+` is one-or-more, and the suffix is literal.
    assert!(plan_docs("post-roadmap-.md").is_empty());
    assert!(plan_docs("post-roadmap-thing.txt").is_empty());
    assert!(plan_docs("post-roadmap-thing").is_empty());
    // Uppercase is not in the class, so the run stops before it — and what is left has no
    // `.md`, so there is no hit rather than a truncated one.
    assert!(plan_docs("post-roadmap-Thing.md").is_empty());
    // A longer extension still yields the `.md` name, exactly as the greedy regex does.
    assert_eq!(
        plan_docs("post-roadmap-thing.mdx"),
        set(&["post-roadmap-thing.md"])
    );
}

#[test]
fn plan_docs_finds_every_hit_on_a_line_not_just_the_first() {
    // The index is one long table row per work item; a scanner that stopped at the first
    // match would under-count on exactly the rows that carry the most work.
    assert_eq!(
        plan_docs("post-roadmap-a.md then post-roadmap-b.md then post-roadmap-a.md"),
        set(&["post-roadmap-a.md", "post-roadmap-b.md"])
    );
}

#[test]
fn record_link_reads_the_pointer_and_rejects_everything_else() {
    assert_eq!(
        record_link("| Some work | [the record](log/reference-flip.md) |").as_deref(),
        Some("reference-flip.md")
    );
    // A row that names a plan doc but has no record pointer is the `unlinked` case the gate
    // reports separately — it must be None, not a partial match.
    assert!(record_link("| Some work | post-roadmap-x.md |").is_none());
    // Wrong directory, wrong suffix, and an empty slug are all not pointers.
    assert!(record_link("[the record](docs/log/x.md)").is_none());
    assert!(record_link("[the record](log/x.txt)").is_none());
    assert!(record_link("[the record](log/.md)").is_none());
    // The slug character class is the same one the plan-doc scanner uses.
    assert!(record_link("[the record](log/Ref.md)").is_none());
}
