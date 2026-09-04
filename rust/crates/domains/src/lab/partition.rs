//! The **partition-table switch** — the one frozen biosphere param the value switch cannot
//! address, made measurable without giving it a second copy of the loader's rules.
//!
//! # Why this module exists at all
//!
//! `docs/plans/post-roadmap-direction-2026-09.md` §2.1 item 3 lists the DVS-keyed partition
//! table as an *unmeasured* suspect in the too-small-canopy problem, and §4 item 4 prices the
//! measurement as *"value switch, minutes"* with a ⚠ adding that *"a `+` column can perturb
//! the whole partition table at once rather than one share at a time"*.
//!
//! ⚠ **Both are false, and the second was written by the 2026-09-02 re-read itself.**
//! [`config::with_override`] rejects a table-shaped field *before* it rewrites anything — its
//! own comment names `allocation.yaml`'s partition rows as the case it is refusing, and
//! `config::params::tests` pins it. The `+` form of [`super::parse_variants`] joins several
//! **scalar** substitutions into one column; the partition table has no scalar entry for a
//! substitution to name, so `+` cannot reach it either. The plan's re-read gate fired, the
//! re-read happened, and it still added a new claim that the harness was cheaper than it is.
//!
//! # The route, and why it is not the obvious one
//!
//! The obvious route is to mutate [`BiosphereParams::alloc`] in memory after loading. That
//! **skips every rule the table has**: `allocation_from` enforces, per row, `fl+fs+fr+fo == 1`
//! within tolerance, each fraction in `[0, 1]`, and strictly increasing `dvs` knots — all
//! inline in the loader, all unreachable from a struct that is already built. Re-stating them
//! here would put a second copy of a rule in the tree, which is the failure this repo has
//! already paid for more than once.
//!
//! So instead: parse the frozen rows with the frozen loader, perturb the parsed rows, **re-emit
//! the `rows:` block into the frozen file's own text**, and load that. The validation is then
//! *the* validation — not a paraphrase of it — and a perturbation that produces an impossible
//! table fails exactly where a committed one would.
//!
//! # ⚠ It writes nothing, and it takes no decision
//!
//! Same contract as [`super::biosphere_with`]: the rewrite lives in a `String` for the length
//! of one run, so no file, golden, manifest digest or gate bound can move. `allocation.yaml`'s
//! `TODO(cite)` is **not** discharged by anything here — this measures whether the table is a
//! suspect, which is a different question from where its numbers came from.

use crate::biosphere::params::{self, AllocationParams, BiosphereParams, PartitionRow};
use config::ConfigError;

/// The four organ shares of one partition row, in the file's column order.
pub const ORGANS: [&str; 4] = ["fl", "fs", "fr", "fo"];

/// The indent of a row's leading `- dvs:` line, and of the three lines under it.
const ROW_INDENT: &str = "      ";
const FIELD_INDENT: &str = "        ";
/// The `rows:` key as it appears in `allocation.yaml`, at its own indent.
const ROWS_KEY: &str = "    rows:";

/// The frozen `allocation.yaml` text, from the census rather than a second `include_str!`.
fn frozen_text() -> &'static str {
    params::param_files()
        .into_iter()
        .find(|(name, _)| *name == "allocation.yaml")
        .expect("allocation.yaml is a frozen biosphere param file")
        .1
}

/// One organ's share of `row`.
fn share(row: &PartitionRow, organ: &str) -> f64 {
    match organ {
        "fl" => row.fl,
        "fs" => row.fs,
        "fr" => row.fr,
        "fo" => row.fo,
        _ => unreachable!("organ names are checked before this point"),
    }
}

/// `row` with `organ` scaled by `factor` and the other three shares **compensated
/// proportionally** so the row still sums to 1.
///
/// # ⚠ Proportional compensation is a CHOICE, not the neutral option
///
/// The freed (or borrowed) carbon does not go anywhere in particular — it is split among the
/// remaining organs in their existing ratio, which is a different experiment at every knot.
/// At `dvs = 0.0` the other three are leaf 0.55 / stem 0.10 / storage 0.00, so shrinking root
/// sends ~85 % of what it frees to leaf; at `dvs = 1.0` they are 0.30 / 0.50 / 0.00, so the
/// same shrink sends only ~37.5 % there. A reader handed a single "the partition table's
/// sensitivity" number cannot see that, which is why [`render_header`] prints the scheme above
/// the table rather than leaving it in this docstring.
///
/// ⚠ `factor == 1.0` is **bit-identical** to the frozen row by construction: `x * 1.0 == x`
/// and `(1 - x) / (1 - x) == 1.0` for every finite `x < 1`. That is what makes the ×1.0
/// control in [`tests`] a test of the plumbing rather than of the arithmetic.
fn scale_row(row: &PartitionRow, organ: &str, factor: f64) -> Result<PartitionRow, ConfigError> {
    let x = share(row, organ);
    let scaled = x * factor;
    if scaled > 1.0 {
        return Err(ConfigError::new(format!(
            "partition: {organ} = {x} at dvs {} scaled by {factor} is {scaled}, above 1",
            row.dvs
        )));
    }
    let rest = 1.0 - x;
    if rest == 0.0 {
        // The other three are already 0, so there is nothing to compensate WITH. Only the
        // identity is representable; anything else is a request this scheme cannot express.
        if scaled == x {
            return Ok(*row);
        }
        return Err(ConfigError::new(format!(
            "partition: {organ} is the whole row at dvs {}, so scaling it by {factor} has \
             no other share to compensate against",
            row.dvs
        )));
    }
    let k = (1.0 - scaled) / rest;
    let mut out = *row;
    for other in ORGANS.iter().filter(|o| **o != organ) {
        let v = share(row, other) * k;
        match *other {
            "fl" => out.fl = v,
            "fs" => out.fs = v,
            "fr" => out.fr = v,
            "fo" => out.fo = v,
            _ => unreachable!(),
        }
    }
    match organ {
        "fl" => out.fl = scaled,
        "fs" => out.fs = scaled,
        "fr" => out.fr = scaled,
        "fo" => out.fo = scaled,
        _ => unreachable!(),
    }
    Ok(out)
}

/// `text` with its `rows:` block replaced by `rows`, everything else byte-for-byte.
///
/// ⚠ The `source:` string, the header and the schema keys are **kept**, so the re-emitted file
/// is the frozen one with different numbers — not a minimal file that happens to parse. The
/// provenance guard `allocation_from` runs (`source` must be a non-empty scalar, `parameters`
/// must hold exactly `partition_table`) therefore still has something to check.
fn with_rows(text: &str, rows: &[PartitionRow]) -> Result<String, ConfigError> {
    let mut out: Vec<String> = Vec::with_capacity(text.lines().count());
    let mut keys_seen = 0usize;
    let mut dropping = false;
    for line in text.lines() {
        if line == ROWS_KEY {
            keys_seen += 1;
            out.push(line.to_string());
            for r in rows {
                out.push(format!("{ROW_INDENT}- dvs: {:?}", r.dvs));
                out.push(format!("{FIELD_INDENT}fl: {:?}", r.fl));
                out.push(format!("{FIELD_INDENT}fs: {:?}", r.fs));
                out.push(format!("{FIELD_INDENT}fr: {:?}", r.fr));
                out.push(format!("{FIELD_INDENT}fo: {:?}", r.fo));
            }
            dropping = true;
            continue;
        }
        if dropping {
            if line.starts_with(ROW_INDENT) {
                continue; // one of the rows being replaced
            }
            dropping = false;
        }
        out.push(line.to_string());
    }
    // ⚠ Not a formality. If the key ever moved indent, the loop above would emit nothing and
    // this function would return the frozen text — a perturbation harness quietly reporting
    // the baseline, which is §7 of the value-switch plan's named failure.
    if keys_seen != 1 {
        return Err(ConfigError::new(format!(
            "allocation.yaml: found {keys_seen} {ROWS_KEY:?} lines, expected exactly 1"
        )));
    }
    let mut result = out.join("\n");
    if text.ends_with('\n') {
        result.push('\n');
    }
    Ok(result)
}

/// The frozen partition table with `organ`'s share scaled by `factor` at **every** knot,
/// re-loaded through [`params::allocation_from`].
///
/// ⚠ A table the perturbation makes impossible fails in the loader, by panic, exactly as a
/// committed one would — this function returns `Err` only for a request it can reject
/// *arithmetically* (an unknown organ, a non-finite or negative factor, a share driven above
/// 1, a row with nothing to compensate against). The split is deliberate: a bad *request* is
/// this module's fault, a bad *table* is the contract's verdict and belongs to the contract.
pub fn scaled_share(organ: &str, factor: f64) -> Result<AllocationParams, ConfigError> {
    if !ORGANS.contains(&organ) {
        return Err(ConfigError::new(format!(
            "{organ:?} is not a partition organ (have {ORGANS:?})"
        )));
    }
    if !factor.is_finite() || factor < 0.0 {
        return Err(ConfigError::new(format!(
            "partition: factor {factor} is not a finite, non-negative number"
        )));
    }
    let text = frozen_text();
    let frozen = params::allocation_from(text, "allocation.yaml");
    let mut rows = Vec::with_capacity(frozen.table.len());
    for row in &frozen.table {
        rows.push(scale_row(row, organ, factor)?);
    }
    Ok(params::allocation_from(
        &with_rows(text, &rows)?,
        "allocation.yaml",
    ))
}

/// The frozen biosphere params with only the partition table perturbed.
pub fn biosphere_with_share(organ: &str, factor: f64) -> Result<BiosphereParams, ConfigError> {
    let mut p = super::biosphere_with(&[])?;
    p.alloc = scaled_share(organ, factor)?;
    Ok(p)
}

/// The column heading for one `(organ, factor)` — the same shape the value switch prints.
pub fn label_of(organ: &str, factor: f64) -> String {
    format!("allocation.yaml:{organ}×{factor}")
}

/// The caption the table is unreadable without — see [`scale_row`]'s ⚠.
pub fn render_header(organ: &str, factors: &[f64]) -> String {
    format!(
        "partition switch: {organ} scaled by {factors:?} at every DVS knot; the other three \
         shares are compensated PROPORTIONALLY (each keeps its share of what is left), so the \
         destination of the moved carbon differs by knot and is not a controlled variable.\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lab::report;

    /// ⚠⚠ **The control that makes every other number this module prints mean something.**
    /// A ×1.0 perturbation must reproduce the frozen table *bit for bit* — through the
    /// re-emitter and the loader, not around them. If it did not, every reported difference
    /// would be partly this module's own YAML round-trip, and the ladder would be measuring
    /// the harness. Same discipline as `lab::tests::no_substitutions_reproduces_the_frozen…`.
    #[test]
    fn a_factor_of_one_reproduces_the_frozen_table_bit_for_bit() {
        let frozen = params::allocation();
        for organ in ORGANS {
            let got = scaled_share(organ, 1.0).expect("×1.0 is always representable");
            assert_eq!(got.table.len(), frozen.table.len(), "{organ}: row count");
            for (i, (a, b)) in got.table.iter().zip(frozen.table.iter()).enumerate() {
                for (label, x, y) in [
                    ("dvs", a.dvs, b.dvs),
                    ("fl", a.fl, b.fl),
                    ("fs", a.fs, b.fs),
                    ("fr", a.fr, b.fr),
                    ("fo", a.fo, b.fo),
                ] {
                    assert_eq!(
                        x.to_bits(),
                        y.to_bits(),
                        "{organ}: row {i} {label} re-read as {x:?}, frozen is {y:?}"
                    );
                }
            }
        }
    }

    /// The re-emitted file must differ from the frozen one **only** inside the rows block —
    /// the header, the schema keys and the `source:` string are what keep the loader's
    /// provenance guard in play, and a rewrite that dropped them would still parse.
    #[test]
    fn the_re_emitted_file_keeps_everything_but_the_rows() {
        let text = frozen_text();
        let rows = params::allocation_from(text, "allocation.yaml").table;
        let out = with_rows(text, &rows).expect("the frozen file has one rows: key");
        for kept in [
            "name: winter_wheat",
            "process: allocation",
            "  partition_table:",
            "    source:",
            ROWS_KEY,
        ] {
            assert!(out.contains(kept), "the re-emitted file lost {kept:?}");
        }
        assert!(
            out.lines()
                .filter(|l| l.starts_with("      - dvs:"))
                .count()
                == rows.len(),
            "the re-emitted file has the wrong number of rows"
        );
    }

    /// ⚠ The liveness half: a large perturbation must actually MOVE the science readout. A
    /// ×1.0 control that passes says the plumbing is faithful; it says nothing about whether
    /// the run reads the table at all. Without this, an inert harness and an inert parameter
    /// are indistinguishable — and "the partition table is not a suspect" is precisely the
    /// finding that would be wrong.
    #[test]
    fn a_large_perturbation_moves_the_science_readout() {
        let frozen = report::measure("frozen", &params::biosphere(), false);
        let moved = report::measure(
            "fl×1.5",
            &biosphere_with_share("fl", 1.5).expect("fl×1.5 is representable"),
            false,
        );
        assert!(!frozen.values.is_empty(), "the baseline measured nothing");
        assert!(
            frozen
                .values
                .iter()
                .zip(moved.values.iter())
                .any(|((_, a), (_, b))| a.to_bits() != b.to_bits()),
            "fl×1.5 moved no readout — the run does not read the partition table, or the \
             perturbation never reached it"
        );
    }

    /// A request this scheme cannot express is refused by name, not clamped into a nearby one.
    #[test]
    fn impossible_requests_are_refused() {
        assert!(scaled_share("fx", 1.0).is_err(), "unknown organ");
        assert!(scaled_share("fl", -1.0).is_err(), "negative factor");
        assert!(scaled_share("fl", f64::NAN).is_err(), "non-finite factor");
        // fl is 0.55 at dvs 0.0, so ×2 drives it to 1.1.
        assert!(scaled_share("fl", 2.0).is_err(), "share driven above 1");
    }

    /// Every row still sums to 1 after a perturbation — asserted here as a property over the
    /// ladder rather than trusted, because the loader's own check is the thing that would
    /// fire, and a test that only ever runs representable factors never sees it.
    #[test]
    fn the_ladder_stays_on_the_simplex() {
        for organ in ORGANS {
            for factor in [0.5, 0.75, 1.0, 1.25] {
                let Ok(a) = scaled_share(organ, factor) else {
                    continue; // refused arithmetically; that is its own test
                };
                for (i, r) in a.table.iter().enumerate() {
                    let total = r.fl + r.fs + r.fr + r.fo;
                    assert!(
                        (total - 1.0).abs() <= 1e-12,
                        "{organ}×{factor} row {i} sums to {total}"
                    );
                }
            }
        }
    }
}
