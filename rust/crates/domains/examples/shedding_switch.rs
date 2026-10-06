//! The **shedding form** measured across the frozen biosphere scenarios —
//! `docs/plans/post-roadmap-leaf-shedding.md` §7. LAB-ONLY; writes nothing, decides nothing.
//!
//! ```text
//! cargo run --release -q -p domains --example shedding_switch            # the short roster
//! cargo run --release -q -p domains --example shedding_switch -- --long  # + the long horizons
//! ```
//!
//! Columns: `frozen`; `FLAT` (the lab flows at the frozen rates — the control, which must equal
//! `frozen` in every cell); `L` (leaf on Teh's form); `LR` (+ root); `LRS` (+ stem); `LRS-PdV`
//! (`LRS` with Penning de Vries' milder leaf table after anthesis).
//!
//! ⚠ **Two compositions per column, merged cell by cell.** The sealed scenarios carry the
//! nitrogen twin of `Senescence`, which sheds on the same rates, so there both flows are
//! replaced ([`composition`]). The open field has no twin, so the paired composition is n/a
//! there — and those cells, and only those, are filled from the carbon-only composition
//! ([`carbon_only`]). The carbon-only run's chamber cells are discarded: it would shed carbon on
//! the new law and nitrogen on the old.

use domains::lab::report::{measure, measure_composed, render, Column};
use domains::lab::shedding::{carbon_only, composition, SheddingForm};

fn column(label: &str, form: SheddingForm, long: bool) -> Column {
    let p = domains::lab::biosphere_with(&[]).expect("frozen params");
    let mut paired = measure_composed(label, &p, long, Some(&composition(form))).expect("paired");
    let carbon = measure_composed(label, &p, long, Some(&carbon_only(form))).expect("carbon only");
    let open: Vec<usize> = paired.not_applicable.iter().map(|(i, _)| *i).collect();
    paired.not_applicable.clear();
    paired
        .values
        .extend(carbon.values.iter().filter(|(i, _)| open.contains(i)));
    paired.values.sort_by_key(|(i, _)| *i);
    paired.failed.extend(
        carbon
            .failed
            .iter()
            .filter(|(i, _)| open.contains(i))
            .cloned(),
    );
    paired
        .constant
        .extend(carbon.constant.iter().filter(|i| open.contains(i)));
    paired.not_applicable.extend(
        carbon
            .not_applicable
            .iter()
            .filter(|(i, _)| open.contains(i))
            .cloned(),
    );
    // ⚠ rationed / events are SUMS over runs: the carbon-only column's chamber runs must not
    // be counted. The open field's own counts are not separable from the sum here, so the
    // paired column's totals stand, and the open field's are printed separately below.
    eprintln!(
        "{label}: open-field-only run (carbon-only composition) rationed {} events {} over all \
         its scenarios; the paired totals below exclude it",
        carbon.rationed, carbon.events
    );
    paired
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let long = args.iter().any(|a| a == "--long");
    if let Some(bad) = args.iter().find(|a| *a != "--long") {
        eprintln!("unknown argument {bad:?} (the only flag is --long)");
        std::process::exit(2);
    }
    let p = domains::lab::biosphere_with(&[]).expect("frozen params");
    let columns = vec![
        measure("frozen", &p, long),
        column("FLAT (control)", SheddingForm::FLAT, long),
        column("L", SheddingForm::LEAF, long),
        column("LR", SheddingForm::LEAF_ROOT, long),
        column("LRS", SheddingForm::ALL, long),
        column("LRS-PdV", SheddingForm::ALL_PDV, long),
    ];
    for c in &columns {
        println!("{}: rationed {} events {}", c.label, c.rationed, c.events);
    }
    println!("{}", render(&columns, long));
}
