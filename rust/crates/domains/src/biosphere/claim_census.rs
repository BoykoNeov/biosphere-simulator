//! The by-name claim census and the gate that keeps it honest — clause 3 of S5's exit
//! gate, built by slice S6 (2026-08-27) after seven consecutive batches deferred it.
//!
//! The census itself is [`CENSUS`], a committed TSV compiled in with `include_str!`; its
//! own header states what it is, how it was built and what each disposition means, and
//! that text is not repeated here. What lives here is the part that has to RUN: the live
//! inventory, read from the biosphere sources on disk at test time, and the assertions
//! that tie the two together.
//!
//! ⚠⚠ **Why the inventory is read from disk and the census is compiled in.** The census's
//! Python column is a snapshot of a tree the next slice DELETES, so it can never be
//! re-derived and must be frozen. The Rust column is the opposite: it must track the tree
//! or the gate rots the first time a test is renamed. Same split, same reason, as
//! `params::param_files` (compiled in) against `params::PARAMS_DIR` (read from disk).
//!
//! ⚠ **The scanner knows two shapes, and that is a measured requirement rather than
//! defensive breadth.** The science gates are declared through the `science_gates!` macro
//! as `gate <name> { … }`, not as bare `#[test] fn`. A scanner that knew only the second
//! silently missed sixteen of them — including three that census rows name as their
//! successor — and the first draft of this file did exactly that.

use std::collections::BTreeMap;

/// The committed census. See its own header for the format and the vocabulary.
pub const CENSUS: &str = include_str!("claim_census.tsv");

/// The biosphere surfaces the census is a census **of**, resolved at compile time against
/// this crate's own root — the same idiom as [`super::params::PARAMS_DIR`].
pub const SURFACES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/biosphere");

/// The four surfaces slice S5 landed its ported tests on. A test in one of these **must**
/// be claimed by a census row or declared as additional; a test in any other surface may
/// be, because the other surfaces descend from Python files outside S5's twenty.
pub const REQUIRED_SURFACES: [&str; 4] = ["science.rs", "flows.rs", "params.rs", "system.rs"];

/// The one `.rs` file in the directory that the census deliberately leaves out.
///
/// This module is the census's own gate. Requiring it to contain rows about itself makes
/// the census a mirror rather than a record, and every such row would have to be added in
/// the same commit that adds the test — a forcing function pointed at itself.
pub const EXCLUDED_SURFACE: &str = "claim_census.rs";

/// Every test the biosphere surfaces declare, as `(name, file)`.
///
/// Both shapes, in one pass: a `#[test]` attribute followed (possibly across further
/// attributes) by a `fn`, and the `gate <name> {` form the `science_gates!` macro expands
/// into a `#[test] fn`.
pub fn live_inventory() -> BTreeMap<String, String> {
    let mut found = BTreeMap::new();
    for file in surface_files() {
        let path = std::path::Path::new(SURFACES_DIR).join(&file);
        let text = std::fs::read_to_string(&path).expect("a readable biosphere surface");
        let lines: Vec<&str> = text.lines().collect();
        for (i, raw) in lines.iter().enumerate() {
            let line = raw.trim();
            if let Some(rest) = line.strip_prefix("gate ") {
                if let Some(name) = rest.split_whitespace().next() {
                    if rest.contains('{') {
                        found.insert(name.to_string(), file.clone());
                    }
                }
                continue;
            }
            if line != "#[test]" {
                continue;
            }
            // Skip any further attributes (`#[should_panic]`, `#[ignore]`, …) and the doc
            // comments that may sit between them, then take the `fn`'s identifier.
            for probe in lines.iter().skip(i + 1) {
                let p = probe.trim();
                if p.starts_with("#[") || p.starts_with("//") || p.is_empty() {
                    continue;
                }
                if let Some(rest) = p.strip_prefix("fn ") {
                    let name: String = rest
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    found.insert(name, file.clone());
                }
                break;
            }
        }
    }
    found
}

/// The `.rs` files under [`SURFACES_DIR`], read from disk, minus [`EXCLUDED_SURFACE`].
fn surface_files() -> Vec<String> {
    let mut files: Vec<String> = std::fs::read_dir(SURFACES_DIR)
        .expect("the biosphere source directory is readable")
        .map(|e| e.expect("a readable dir entry"))
        .filter(|e| e.path().is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".rs") && n != EXCLUDED_SURFACE)
        .collect();
    files.sort();
    files
}

/// One parsed census row.
pub struct Row<'a> {
    pub kind: &'a str,
    pub a: &'a str,
    pub b: &'a str,
    pub tag: &'a str,
    pub rest: Vec<&'a str>,
}

/// Parse [`CENSUS`], dropping comment lines.
pub fn rows() -> Vec<Row<'static>> {
    CENSUS
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            assert!(f.len() >= 5, "a census row has at least five columns: {l:?}");
            Row {
                kind: f[0],
                a: f[1],
                b: f[2],
                tag: f[3],
                rest: f[4..].to_vec(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// The closed vocabulary. A disposition outside it is a typo or an unrecorded kind.
    const DISPOSITIONS: [&str; 11] = [
        "ported",
        "owned",
        "folded",
        "compiler",
        "engine",
        "boundary",
        "decided-out",
        "no-subject",
        "retired-subject",
        "python-only",
        "open",
    ];

    fn successors<'a>(row: &Row<'a>) -> Vec<&'a str> {
        let raw = row.rest.last().copied().unwrap_or("-");
        if raw == "-" {
            Vec::new()
        } else {
            raw.split(',').collect()
        }
    }

    /// ⚠ THE ASSERTION THE WHOLE FILE EXISTS FOR: every claim that says it has a successor
    /// names one that is really there.
    ///
    /// It is deliberately the WEAKER of the two directions — a named test existing is not
    /// proof it carries the claim — and it is stated as such rather than left to read as
    /// coverage. The strong direction is the next test.
    #[test]
    fn every_named_successor_is_a_test_that_exists() {
        let live = live_inventory();
        let mut missing = Vec::new();
        for row in rows() {
            if row.kind != "P" {
                continue;
            }
            for s in successors(&row) {
                if !live.contains_key(s) {
                    missing.push(format!("{} -> {s}", row.b));
                }
            }
        }
        assert!(missing.is_empty(), "census names absent successors: {missing:#?}");
    }

    /// ⚠⚠ THE STRONG DIRECTION: every test in the four surfaces S5 landed on is either
    /// claimed by a Python row or declared as additional coverage.
    ///
    /// This is what makes the census a forcing function rather than a document. A test
    /// added to `science.rs`, `flows.rs`, `params.rs` or `system.rs` reddens this until
    /// somebody says, in the census, what claim it carries and where that claim came
    /// from. The other seven surfaces are exempt because they descend from Python files
    /// outside S5's twenty; their tests are still declared, but by an `A` row rather than
    /// by this rule.
    #[test]
    fn every_test_in_an_s5_surface_is_claimed_or_declared() {
        let live = live_inventory();
        let mut accounted: BTreeSet<String> = BTreeSet::new();
        for row in rows() {
            match row.kind {
                "P" => accounted.extend(successors(&row).into_iter().map(str::to_string)),
                "A" => {
                    accounted.insert(row.b.to_string());
                }
                other => panic!("unknown census row kind {other:?}"),
            }
        }
        let orphans: Vec<&String> = live
            .iter()
            .filter(|(name, file)| {
                REQUIRED_SURFACES.contains(&file.as_str()) && !accounted.contains(*name)
            })
            .map(|(name, _)| name)
            .collect();
        assert!(
            orphans.is_empty(),
            "tests in an S5 surface that no census row accounts for: {orphans:#?}"
        );
    }

    /// Every `A` row names a test that exists — the same check as for successors, for the
    /// other kind of row, so a renamed additional test cannot leave a stale declaration.
    #[test]
    fn every_declared_additional_test_exists() {
        let live = live_inventory();
        let stale: Vec<&str> = rows()
            .iter()
            .filter(|r| r.kind == "A" && !live.contains_key(r.b))
            .map(|r| r.b)
            .collect();
        assert!(stale.is_empty(), "declared but absent: {stale:#?}");
    }

    /// ⚠ Bare names must resolve. The census keys Rust tests by name alone, so two tests
    /// sharing one across surfaces would make a row resolve ambiguously and BOTH checks
    /// above would still pass. Measured at zero collisions when the census was built;
    /// asserted so it stays that way rather than recorded as a fact that was once true.
    #[test]
    fn no_two_biosphere_tests_share_a_bare_name() {
        let mut seen: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for file in surface_files() {
            let path = std::path::Path::new(SURFACES_DIR).join(&file);
            let text = std::fs::read_to_string(&path).expect("a readable biosphere surface");
            for (name, _) in single_file_inventory(&text) {
                seen.entry(name).or_default().push(file.clone());
            }
        }
        let dupes: Vec<_> = seen.iter().filter(|(_, v)| v.len() > 1).collect();
        assert!(dupes.is_empty(), "bare-name collisions: {dupes:#?}");
    }

    fn single_file_inventory(text: &str) -> Vec<(String, ())> {
        let lines: Vec<&str> = text.lines().collect();
        let mut out = Vec::new();
        for (i, raw) in lines.iter().enumerate() {
            let line = raw.trim();
            if let Some(rest) = line.strip_prefix("gate ") {
                if rest.contains('{') {
                    if let Some(name) = rest.split_whitespace().next() {
                        out.push((name.to_string(), ()));
                    }
                }
                continue;
            }
            if line != "#[test]" {
                continue;
            }
            for probe in lines.iter().skip(i + 1) {
                let p = probe.trim();
                if p.starts_with("#[") || p.starts_with("//") || p.is_empty() {
                    continue;
                }
                if let Some(rest) = p.strip_prefix("fn ") {
                    out.push((
                        rest.chars()
                            .take_while(|c| c.is_alphanumeric() || *c == '_')
                            .collect(),
                        (),
                    ));
                }
                break;
            }
        }
        out
    }

    /// The shape of the census, pinned as literals.
    ///
    /// ⚠ These are a FORCING FUNCTION, exactly like `param_files`'s three-place count, and
    /// the cost is stated in the census header rather than discovered: adding a biosphere
    /// test reddens this until the census declares it. The Python side can only ever
    /// shrink to zero — its tree is deleted by this slice's successor — so a change to the
    /// 455 means a row was dropped, which is the one thing the census exists to prevent.
    #[test]
    fn the_census_has_the_shape_it_was_built_with() {
        let all = rows();
        let (p, a): (Vec<_>, Vec<_>) = all.iter().partition(|r| r.kind == "P");
        assert_eq!(p.len(), 455, "Python test functions across S5's twenty files");
        assert_eq!(a.len(), 67, "Rust tests declared as additional coverage");
        for row in &p {
            assert!(
                DISPOSITIONS.contains(&row.tag),
                "unknown disposition {:?} on {}",
                row.tag,
                row.b
            );
        }
        for row in &a {
            assert!(
                ["outside-the-20", "no-ancestor-named"].contains(&row.tag),
                "unknown additional tag {:?} on {}",
                row.tag,
                row.b
            );
        }
        // Twenty Python files, and no row may name a twenty-first: the roster S5 batched
        // is closed, and a file appearing here would mean the extractor read a wider tree
        // than the one the batches covered.
        let files: BTreeSet<&str> = p.iter().map(|r| r.a).collect();
        assert_eq!(files.len(), 20, "S5's roster is twenty files");
    }

    /// ⚠⚠ THE NUMBER THAT MAKES THE CENSUS HONEST AT 455 ROWS: how many claims have no
    /// successor AND no earlier slice behind them.
    ///
    /// `open` means a gap — no successor and no recorded decision. `S6` as its evidence
    /// means this slice decided it today with nothing behind it. A row that is both is a
    /// claim nobody has reasoned about, and putting unsure rows THERE rather than giving
    /// them a plausible successor is the whole discipline: a census that says "three rows
    /// are my judgement today, unbacked" is worth more than one that says nothing and is
    /// quietly wrong about a dozen.
    ///
    /// The three, and why each is genuinely open rather than lazily filed:
    /// `test_every_scenarios_water_stores_are_geometric` and
    /// `test_the_harvest_injection_keeps_depth_and_water_together` are claims about the
    /// whole scenario ROSTER and about a STATION scenario, neither of which the biosphere
    /// crate can assert; `test_the_harvest_scenarios_root_system_tracks_the_crop_maximum`
    /// is the same shape. The seven other `open` rows carry an earlier slice's reasoning.
    #[test]
    fn the_unbacked_gaps_are_counted_and_there_are_three() {
        let open_s6 = rows()
            .iter()
            .filter(|r| r.kind == "P" && r.tag == "open" && r.rest[0] == "S6")
            .count();
        assert_eq!(open_s6, 3, "`open` rows with no earlier slice behind them");
        let open_total = rows()
            .iter()
            .filter(|r| r.kind == "P" && r.tag == "open")
            .count();
        assert_eq!(open_total, 10, "gaps carried forward past this slice");
    }

    /// The scanner sees both declaration shapes.
    ///
    /// ⚠ Not a tautology, and it is here because the first draft failed it: the sixteen
    /// science gates are `gate <name> { … }` macro arms, and a `#[test] fn` scanner finds
    /// none of them. One name of each shape, checked against the surface it lives in.
    #[test]
    fn the_scanner_finds_both_declaration_shapes() {
        let live = live_inventory();
        assert_eq!(
            live.get("the_depth_quadrature_conserves_photons_against_beer_lambert")
                .map(String::as_str),
            Some("science.rs"),
            "the plain `#[test] fn` shape"
        );
        assert_eq!(
            live.get("open_season_peaks_below_the_greenwood_crossing")
                .map(String::as_str),
            Some("science_gates.rs"),
            "the `gate <name> {{` macro shape"
        );
    }

    /// The census is a census OF a directory, and the directory is read rather than
    /// assumed — the same rule, and the same reason, as
    /// `params::tests::the_census_matches_the_directory_on_disk`.
    #[test]
    fn the_surfaces_scanned_are_the_directory_minus_the_one_exclusion() {
        let scanned = surface_files();
        assert!(
            !scanned.contains(&EXCLUDED_SURFACE.to_string()),
            "the census does not contain rows about its own gate"
        );
        assert!(
            std::path::Path::new(SURFACES_DIR)
                .join(EXCLUDED_SURFACE)
                .is_file(),
            "the exclusion names a file that exists — an exclusion for a file that is not \
             there is how a roster silently loses a member"
        );
        for required in REQUIRED_SURFACES {
            assert!(
                scanned.contains(&required.to_string()),
                "{required} is one of the four surfaces the strong rule applies to"
            );
        }
    }
}
