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
//!
//! # The second axis: which KNOT (added 2026-09-04)
//!
//! The first ladder scaled an organ's share at **every** DVS knot, and stopped at `fl×1.8`
//! because `fl×2.0` drives the `dvs 0` share (0.55) above 1. That ceiling is set by the
//! **binding** knot, not the **effective** one: `1/0.55 = 1.818` at emergence against
//! `1/0.30 = 3.333` at anthesis, so the uniform ladder could never spend the anthesis knot's
//! headroom. Peak LAI in `open_season` falls at **DVS 1.306** — *past* anthesis, where the
//! interpolated share is governed by the `dvs 1.0` and `dvs 2.0` knots — which is why the
//! knot is an axis and not a detail. [`Knot::At`] moves one row and leaves the others alone.
//!
//! ⚠ **A per-knot column is not a continuation of the every-knot ladder**, for the same
//! reason the compensation scheme is not a nuisance parameter: it is a *different*
//! perturbation, and reading the two as one curve would be reading two experiments as one.
//! [`render_header`] and [`label_of`] both carry the knot for that reason.
//!
//! ⚠ And the table is **interpolated**, so moving one knot moves the fractions across the
//! whole span between its neighbours. `fl@dvs1` is not "leaf allocation at anthesis"; it is
//! the whole `0 < dvs < 2` interior, weighted by distance from the knot.

use crate::biosphere::params::{self, AllocationParams, BiosphereParams, PartitionRow};
use config::ConfigError;

/// The four organ shares of one partition row, in the file's column order.
pub const ORGANS: [&str; 4] = ["fl", "fs", "fr", "fo"];

/// Which DVS knots a perturbation applies to — the axis added 2026-09-04.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Knot {
    /// Every row of the table: the uniform ladder of `docs/log/partition-sensitivity.md`.
    Every,
    /// One row, named by its own `dvs` value.
    ///
    /// ⚠ Matched by **exact** equality, and a value that is not a knot is refused by name
    /// rather than snapped to the nearest one — a nearest-match would silently measure a
    /// perturbation the caller did not ask for. The frozen knots are 0.0 / 1.0 / 2.0, all
    /// exactly representable, so equality is the right test here and would stop being so
    /// only if a knot ever acquired a fractional value (in which case this must be revisited,
    /// not loosened to a tolerance — see [`knot_row`]).
    At(f64),
}

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
pub fn scaled_share(organ: &str, knot: Knot, factor: f64) -> Result<AllocationParams, ConfigError> {
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
    let target = match knot {
        Knot::Every => None,
        Knot::At(dvs) => Some(knot_row(&frozen.table, dvs)?),
    };
    let mut rows = Vec::with_capacity(frozen.table.len());
    for (i, row) in frozen.table.iter().enumerate() {
        match target {
            Some(t) if t != i => rows.push(*row),
            _ => rows.push(scale_row(row, organ, factor)?),
        }
    }
    refuse_if_inert(organ, knot, factor, &rows, &frozen.table)?;
    Ok(params::allocation_from(
        &with_rows(text, &rows)?,
        "allocation.yaml",
    ))
}

/// The index of the row whose `dvs` is exactly `dvs` — see [`Knot::At`] on why exactly.
fn knot_row(table: &[PartitionRow], dvs: f64) -> Result<usize, ConfigError> {
    table.iter().position(|r| r.dvs == dvs).ok_or_else(|| {
        let knots: Vec<f64> = table.iter().map(|r| r.dvs).collect();
        ConfigError::new(format!(
            "partition: {dvs} is not a DVS knot of the frozen table (have {knots:?}); a \
             perturbation between knots is not expressible — the table's rows ARE its \
             degrees of freedom"
        ))
    })
}

/// ⚠ **A perturbation that leaves the table bit-identical is refused by name.**
///
/// Not a formality: `fl` is **0.00** at the `dvs 2.0` knot, so `fl@dvs2 ×1.5` scales 0 to 0,
/// compensates the others by `k = 1`, and returns the frozen table. That column would print
/// `<- UNCHANGED` on all eight readouts, which a reader takes as *"the harness is broken"* or
/// — worse — as *"the partition table does not matter here"*. Both readings are wrong: the
/// share is simply not there to scale. The distinction the module already draws for a bad
/// request applies to a **structurally empty** one too, so it says which organ, which knot,
/// and that the share is zero.
///
/// `factor == 1.0` is exempt because it is the plumbing control, whose whole job is to be
/// bit-identical.
fn refuse_if_inert(
    organ: &str,
    knot: Knot,
    factor: f64,
    rows: &[PartitionRow],
    frozen: &[PartitionRow],
) -> Result<(), ConfigError> {
    if factor == 1.0 || !same_table(rows, frozen) {
        return Ok(());
    }
    let zeros: Vec<f64> = frozen
        .iter()
        .filter(|r| share(r, organ) == 0.0)
        .map(|r| r.dvs)
        .collect();
    Err(ConfigError::new(format!(
        "partition: {} by {factor} leaves the table bit-identical — {organ} is 0 at the dvs \
         {zeros:?} knot(s), and a multiplicative perturbation of 0 is the identity. This is a \
         structural null, not a measurement: a column of UNCHANGED readouts would read as a \
         broken harness",
        match knot {
            Knot::Every => format!("scaling {organ} at every knot"),
            Knot::At(dvs) => format!("scaling {organ} at the dvs {dvs} knot"),
        }
    )))
}

/// Two tables equal **bit for bit**, which is the only comparison this module trusts.
fn same_table(a: &[PartitionRow], b: &[PartitionRow]) -> bool {
    a.len() == b.len()
        && a.iter().zip(b.iter()).all(|(x, y)| {
            x.dvs.to_bits() == y.dvs.to_bits()
                && ORGANS
                    .iter()
                    .all(|o| share(x, o).to_bits() == share(y, o).to_bits())
        })
}

/// The frozen biosphere params with only the partition table perturbed.
pub fn biosphere_with_share(
    organ: &str,
    knot: Knot,
    factor: f64,
) -> Result<BiosphereParams, ConfigError> {
    let mut p = super::biosphere_with(&[])?;
    p.alloc = scaled_share(organ, knot, factor)?;
    Ok(p)
}

/// The column heading for one `(organ, knot, factor)` — the same shape the value switch
/// prints, with the knot **in the label** so a per-knot column can never be quoted as a rung
/// of the every-knot ladder.
pub fn label_of(organ: &str, knot: Knot, factor: f64) -> String {
    match knot {
        Knot::Every => format!("allocation.yaml:{organ}×{factor}"),
        Knot::At(dvs) => format!("allocation.yaml:{organ}@dvs{dvs}×{factor}"),
    }
}

/// The caption the table is unreadable without — see [`scale_row`]'s ⚠.
///
/// ⚠ It takes the `knot` rather than appending to a fixed *"at every DVS knot"* string. A
/// caption that describes a different experiment than the one that ran is this tree's most
/// expensive recurring defect, and the two schemes here differ in what is held fixed.
pub fn render_header(organ: &str, knot: Knot, factors: &[f64]) -> String {
    match knot {
        Knot::Every => format!(
            "partition switch: {organ} scaled by {factors:?} at every DVS knot; the other \
             three shares are compensated PROPORTIONALLY (each keeps its share of what is \
             left), so the destination of the moved carbon differs by knot and is not a \
             controlled variable.\n"
        ),
        Knot::At(dvs) => format!(
            "partition switch: {organ} scaled by {factors:?} at the dvs {dvs} knot ONLY; the \
             other three shares AT THAT KNOT are compensated PROPORTIONALLY and every other \
             row is left frozen. ⚠ The table is INTERPOLATED, so this moves the fractions \
             across the whole span between {dvs}'s neighbouring knots — it is not confined to \
             one development stage. ⚠ These columns are NOT rungs of the every-knot ladder: \
             they are a different perturbation and must not be read as one curve with it.\n"
        ),
    }
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
    ///
    /// ⚠ Extended over the **knot** axis on 2026-09-04. A control that only ever ran
    /// `Knot::Every` would leave the new axis with no plumbing check at all, and
    /// `Knot::At` has its own way to be wrong — a mis-resolved row index rewrites the
    /// wrong row, which `Every` cannot express.
    #[test]
    fn a_factor_of_one_reproduces_the_frozen_table_bit_for_bit() {
        let frozen = params::allocation();
        let knots: Vec<Knot> = std::iter::once(Knot::Every)
            .chain(frozen.table.iter().map(|r| Knot::At(r.dvs)))
            .collect();
        for organ in ORGANS {
            for knot in &knots {
                let got = scaled_share(organ, *knot, 1.0).expect("×1.0 is always representable");
                assert_eq!(
                    got.table.len(),
                    frozen.table.len(),
                    "{organ} {knot:?}: row count"
                );
                assert!(
                    same_table(&got.table, &frozen.table),
                    "{organ} {knot:?}: ×1.0 did not reproduce the frozen table bit for bit \
                     ({:?} against {:?})",
                    got.table,
                    frozen.table
                );
            }
        }
    }

    /// ⚠⚠ **The control the knot axis is worthless without.** `Knot::At` must move exactly
    /// one row: if the row lookup were ignored — or resolved to the wrong index — the column
    /// would still print plausible numbers, and they would be the *uniform* ladder's under a
    /// per-knot label. That is the failure the label and header changes exist to prevent, and
    /// a caption cannot detect it. Bit-identity on the untouched rows is what can.
    #[test]
    fn a_per_knot_perturbation_moves_exactly_its_own_row() {
        let frozen = params::allocation();
        for (i, row) in frozen.table.iter().enumerate() {
            for organ in ORGANS {
                // A zero share has no representable perturbation; that is its own test.
                let x = share(row, organ);
                if x == 0.0 {
                    continue;
                }
                // Halfway to the share's own ceiling, so every organ gets a real move and
                // none is refused for driving its row above 1 (`fo` is 0.8 at dvs 2).
                let factor = 1.0 + (1.0 - x) / (2.0 * x);
                let got = scaled_share(organ, Knot::At(row.dvs), factor)
                    .expect("halfway to the ceiling is representable by construction");
                for (j, (a, b)) in got.table.iter().zip(frozen.table.iter()).enumerate() {
                    let moved = !same_table(std::slice::from_ref(a), std::slice::from_ref(b));
                    assert_eq!(
                        moved,
                        i == j,
                        "{organ}@dvs{} ×{factor}: row {j} (dvs {}) {} — only row {i} may move",
                        row.dvs,
                        b.dvs,
                        if moved { "moved" } else { "did not move" }
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
    ///
    /// ⚠ Run on **both** axes since 2026-09-04. The uniform case says the run reads the table
    /// at all; it says nothing about whether a single-row rewrite survives the re-emitter and
    /// reaches the science — and a per-knot column that quietly measured the frozen table is
    /// exactly the reading this ladder was built to produce.
    #[test]
    fn a_large_perturbation_moves_the_science_readout() {
        let frozen = report::measure("frozen", &params::biosphere(), false);
        assert!(!frozen.values.is_empty(), "the baseline measured nothing");
        for (label, knot) in [("fl×1.5", Knot::Every), ("fl@dvs1×1.5", Knot::At(1.0))] {
            let moved = report::measure(
                label,
                &biosphere_with_share("fl", knot, 1.5).expect("fl ×1.5 is representable"),
                false,
            );
            assert!(
                frozen
                    .values
                    .iter()
                    .zip(moved.values.iter())
                    .any(|((_, a), (_, b))| a.to_bits() != b.to_bits()),
                "{label} moved no readout — the run does not read the partition table, or the \
                 perturbation never reached it"
            );
        }
    }

    /// A request this scheme cannot express is refused by name, not clamped into a nearby one.
    #[test]
    fn impossible_requests_are_refused() {
        assert!(
            scaled_share("fx", Knot::Every, 1.0).is_err(),
            "unknown organ"
        );
        assert!(
            scaled_share("fl", Knot::Every, -1.0).is_err(),
            "negative factor"
        );
        assert!(
            scaled_share("fl", Knot::Every, f64::NAN).is_err(),
            "non-finite factor"
        );
        // fl is 0.55 at dvs 0.0, so ×2 drives it to 1.1 — the ceiling the every-knot ladder
        // stopped at, and the reason the knot became an axis.
        assert!(
            scaled_share("fl", Knot::Every, 2.0).is_err(),
            "share driven above 1"
        );
        // The anthesis knot's own ceiling is 1/0.30, so the same ×2 is fine there. If this
        // ever failed, the per-knot axis would have no headroom over the uniform one and the
        // whole 2026-09-04 ladder would be pointless.
        assert!(
            scaled_share("fl", Knot::At(1.0), 2.0).is_ok(),
            "fl@dvs1 ×2 is 0.60, well inside the row"
        );
        assert!(
            scaled_share("fl", Knot::At(3.4), 1.5).is_err(),
            "a value that is not a knot is refused, not snapped to the nearest row"
        );
    }

    /// ⚠ The **structural null**, refused by name rather than printed as a column of
    /// UNCHANGED readouts. `fl` is 0.00 at `dvs 2.0`, so `fl@dvs2` cannot be scaled at all;
    /// a reader handed that column would read it as a broken harness — or as evidence the
    /// table does not matter — and both are wrong.
    #[test]
    fn a_perturbation_that_cannot_move_the_table_is_refused_by_name() {
        let err = scaled_share("fl", Knot::At(2.0), 1.5)
            .expect_err("fl is 0 at dvs 2.0, so ×1.5 is the identity");
        let text = err.to_string();
        for want in ["fl", "dvs 2", "identity", "structural null"] {
            assert!(
                text.contains(want),
                "the refusal never says {want:?}: {text}"
            );
        }
        // ⚠ Its two neighbours: ×1.0 is the plumbing control and must stay allowed, and a
        // knot where the share IS present must not be caught by the same guard.
        assert!(
            scaled_share("fl", Knot::At(2.0), 1.0).is_ok(),
            "the ×1.0 control is exempt — being bit-identical is its whole job"
        );
        assert!(
            scaled_share("fs", Knot::At(2.0), 1.5).is_ok(),
            "fs is 0.10 at dvs 2.0 and is a real rung"
        );
    }

    /// Every row still sums to 1 after a perturbation — asserted here as a property over the
    /// ladder rather than trusted, because the loader's own check is the thing that would
    /// fire, and a test that only ever runs representable factors never sees it.
    #[test]
    fn the_ladder_stays_on_the_simplex() {
        let knots: Vec<Knot> = std::iter::once(Knot::Every)
            .chain(params::allocation().table.iter().map(|r| Knot::At(r.dvs)))
            .collect();
        for organ in ORGANS {
            for knot in &knots {
                for factor in [0.5, 0.75, 1.0, 1.25, 2.0, 3.0] {
                    let Ok(a) = scaled_share(organ, *knot, factor) else {
                        continue; // refused arithmetically; that is its own test
                    };
                    for (i, r) in a.table.iter().enumerate() {
                        let total = r.fl + r.fs + r.fr + r.fo;
                        assert!(
                            (total - 1.0).abs() <= 1e-12,
                            "{}: row {i} sums to {total}",
                            label_of(organ, *knot, factor)
                        );
                    }
                }
            }
        }
    }
}
