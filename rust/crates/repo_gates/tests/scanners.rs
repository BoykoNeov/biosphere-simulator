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

use repo_gates::{doc_bounds, memory_link, plan_docs, record_link, usize_consts};

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

/// ⚠ Pinned for the reason at the top of this file, which bites the memory parity gate
/// harder than the others: it is a set comparison **against the disk**, so a scanner that
/// silently reads fewer index lines shrinks only the indexed side and reports every one of
/// the files it stopped seeing as UNREACHABLE. That is loud. But the mirror case is silent
/// — a scanner that read *nothing* would make both differences empty and the gate green,
/// which is the "inert by construction" failure this file exists to stop. The first two
/// assertions are that mirror case.
#[test]
fn memory_link_reads_the_index_line_and_rejects_everything_else() {
    assert_eq!(
        memory_link("- [Soil layers: BUILT](soil-layers-built.md) — the expensive design")
            .as_deref(),
        Some("soil-layers-built.md")
    );
    // A hook containing its own parentheses must not shorten the match: the link is the
    // FIRST `](`, and the close is the first `)` after it.
    assert_eq!(
        memory_link("- [N-cycle, all four options](nitrogen-cycle-form.md) — (A)+(B) BUILT")
            .as_deref(),
        Some("nitrogen-cycle-form.md")
    );
    // Not links, not `.md`, and no target at all — each None rather than a partial match.
    assert!(memory_link("## Direction & posture").is_none());
    assert!(memory_link("- [Title](notes.txt) — hook").is_none());
    assert!(memory_link("- plain text with no link").is_none());
}

/// The scanners' shared number reader, exercised through both of them because the two sides
/// spell one bound differently: Rust groups with `_`, English prose groups with `,`. A reader
/// that handled only one would make the doc↔code comparison red for every four-digit bound
/// and green for `170` — i.e. it would look like it worked.
#[test]
fn both_scanners_read_underscore_and_comma_grouping() {
    assert_eq!(
        usize_consts("const MAX_A: usize = 12_000;"),
        vec![("MAX_A".to_string(), 12_000)]
    );
    assert_eq!(
        doc_bounds("| `MAX_A` | 12,000 | bytes | a thing |"),
        vec![("MAX_A".to_string(), 12_000)]
    );
}

#[test]
fn usize_consts_takes_only_module_level_usize_declarations() {
    // The three neighbours of a real bound in `context_budget.rs`, each of which must NOT
    // become a row the document owes: a `&str` const, an indented one, and the word `const`
    // inside prose. The last is not hypothetical — that file's doc comments discuss its own
    // constants at length, and a scanner matching them would demand table rows for sentences.
    let source = concat!(
        "const KEPT: usize = 240;\n",
        "const PHASE_TABLE_SHA256: &str = \"5551a414\";\n",
        "    const INDENTED: usize = 7;\n",
        "/// See const MAX_A: usize = 1; in the module above.\n",
    );
    assert_eq!(usize_consts(source), vec![("KEPT".to_string(), 240)]);
}

#[test]
fn doc_bounds_requires_the_name_to_be_the_whole_cell() {
    // ⚠ The exclusion that keeps this scanner safe on the file it actually reads.
    // `docs/context-budget.md` opens with a table whose first cell is a backticked commit
    // hash followed by prose, and closes with one whose first cell is a backticked FILE name
    // — eleven rows in the shape `| `x` … | <number> |`. Matching them would invent bounds
    // that no constant can ever satisfy, so the doc↔code comparison would be permanently red
    // and the only available "fix" would be to weaken it.
    assert!(doc_bounds("| `255da30` move the record out, leaving an index | 14,458 |").is_empty());
    assert!(doc_bounds("| `CLAUDE.md` | 17,715 B | 9,520 B (ceiling 12,000) |").is_empty());
    // A digits-only hash cannot pass for a constant either: the name must start with a letter.
    assert!(doc_bounds("| `1234567` | 14,458 |").is_empty());
}

#[test]
fn doc_bounds_ignores_a_row_whose_value_cell_is_prose() {
    // A well-formed name with an unparseable value is silently skipped rather than read as
    // zero. Pinned because "0" would be a plausible-looking bound that no code declares.
    assert!(doc_bounds("| `MAX_A` | see below | bytes | a thing |").is_empty());
}
