//! The **potato** — the second species, as authored habitat content the reference can run.
//!
//! Stage 2 of `docs/plans/post-roadmap-potato-crop.md`. Stage 1 built the crop in Python and
//! measured it against an offline WOFOST oracle; that half died with the checker (S6), so
//! `params::potato` + `potato_scenario` are the crop's only live form and this file is its
//! only gate.
//!
//! # What this file claims, and what it deliberately does not
//!
//! **Authored ≠ validated.** Nothing here endorses a potato number. The crop is wired into no
//! golden and named in no manifest; what is asserted is that it *resolves as a set*, that the
//! set *reaches the run*, and that the run *conserves mass, is deterministic and is well fed*.
//! The measured disagreements with WOFOST — a tuber that starts filling 39 days early, a
//! canopy 2.79x low — are the record in `docs/log/potato-crop.md` and are not a target.
//!
//! # ⚠ Why the override pin is field-level and not file-level
//!
//! The obvious shape — *overridden file ⇒ every value differs, shared file ⇒ every value
//! identical* — is **red or vacuous** on this crop, and reading the files is what shows it.
//! Potato's `canopy.yaml` overrides the file but carries wheat's `extinction_coef` (0.6) and
//! `carbon_fraction` (0.45) unchanged, each labelled *"SHARED, not potato"* in its own
//! `source:` string. Only `specific_leaf_area` moves. A file-level assertion would either
//! fail on those two or be weakened until it asserted nothing.

use std::collections::BTreeSet;

use config::{with_override, ParamFile};
use domains::biosphere::params::{self, BiosphereParams, POTATO_OVERRIDES};
use domains::biosphere::readouts::trajectory;
use domains::biosphere::stocks::LEAF_C;
use domains::biosphere::{
    build_season, build_season_with, potato_scenario, run_season, sealed_chamber_scenario,
    season_setup_with, steps_for_years, SeasonScenario, BIO_DT, DEFAULT_SCENARIO,
    SEALED_CHAMBER_YEARS,
};
use simcore::conservation::compute_ledger;
use simcore::quantities::Quantity;
use simcore::state::State;

// --------------------------------------------------------------------------------- //
// Helpers                                                                             //
// --------------------------------------------------------------------------------- //

/// One override file's embedded text, by its loader name.
fn overlay(name: &str) -> &'static str {
    POTATO_OVERRIDES
        .iter()
        .find(|(n, _)| *n == name)
        .unwrap_or_else(|| panic!("{name} is one of the four overrides"))
        .1
}

/// One frozen file's embedded text, by its census name.
fn frozen(name: &str) -> &'static str {
    params::param_files()
        .into_iter()
        .find(|(n, _)| *n == name)
        .unwrap_or_else(|| panic!("{name} is in the frozen census"))
        .1
}

/// A `{value, unit, source}` entry's magnitude, straight off the file.
///
/// ⚠ Read from the **text**, not from a loaded struct, because one claim below is about a
/// value the structs no longer carry: `canopy.yaml`'s `carbon_fraction` is folded away into
/// `sla_per_mol_c` (`sla * M_C / carbon_fraction`), so a struct-level comparison cannot see
/// it at all.
fn field(text: &str, name: &str, where_: &str) -> f64 {
    ParamFile::parse(text, where_)
        .unwrap_or_else(|e| panic!("{where_} parses: {e}"))
        .entry(name, where_)
        .unwrap_or_else(|e| panic!("{where_} has {name}: {e}"))
        .value
}

/// The leaf-carbon series of one season, against `p`.
fn leaf_series(scenario: &SeasonScenario, p: &BiosphereParams) -> Vec<f64> {
    let (state, integrator, resolver) = season_setup_with(scenario, 1, p).expect("setup");
    let steps = steps_for_years(1);
    let mut series = Vec::with_capacity(steps + 1);
    {
        let mut observe = |s: &State| series.push(s.stocks[LEAF_C].amount);
        let (_final, rationed, events) = run_season(
            &integrator,
            state,
            &resolver,
            BIO_DT,
            steps,
            None,
            &mut observe,
        )
        .expect("season");
        assert_eq!(rationed, 0, "an A/B run must be well fed to be comparable");
        assert!(events.is_empty(), "unexpected extinction: {events:?}");
    }
    series
}

/// The potato params with one field of one override file rewritten in memory.
///
/// The rewrite goes through `config::with_override`, so the mutated value passes the *same*
/// schema, unit guard and frozen bounds as a committed one — a mutation the loader would
/// reject is an `Err` here rather than a run.
fn potato_with(file: &str, field_name: &str, value: f64) -> BiosphereParams {
    let text = with_override(overlay(file), field_name, value, file)
        .unwrap_or_else(|e| panic!("{file}:{field_name} := {value}: {e}"));
    // `*_from` takes a `&'static str` NAME; the TEXT is what varies here and it must outlive
    // the loader call. Leaking one short string per mutation is the price of reusing the real
    // loaders rather than hand-building a struct — which is exactly what would bypass the
    // schema, the unit guard and the bounds this seam exists to keep in the path.
    let text: &'static str = Box::leak(text.into_boxed_str());
    let mut p = params::potato();
    match file {
        "crops/potato/phenology.yaml" => {
            p.pheno = params::phenology_from(text, "crops/potato/phenology.yaml");
            p.vern = params::vernalization_from(text, "crops/potato/phenology.yaml");
            p.photoperiod = params::photoperiod_from(text, "crops/potato/phenology.yaml");
        }
        "crops/potato/canopy.yaml" => {
            p.canopy = params::canopy_from(text, "crops/potato/canopy.yaml");
        }
        "crops/potato/root_depth.yaml" => {
            p.rootd = params::root_depth_from(text, "crops/potato/root_depth.yaml");
        }
        other => panic!("no mutation seam for {other}"),
    }
    p
}

/// The flow + aux type names of one build.
fn type_set(scenario: &SeasonScenario, p: &BiosphereParams) -> BTreeSet<&'static str> {
    let (_, registry) = build_season_with(scenario, p).expect("build");
    registry
        .flows()
        .iter()
        .map(|f| f.type_name())
        .chain(registry.aux_processes().iter().map(|a| a.type_name()))
        .collect()
}

// --------------------------------------------------------------------------------- //
// The crop resolves as a SET                                                          //
// --------------------------------------------------------------------------------- //

/// The overridden files, field by field: what really moved and what deliberately did not.
///
/// ⚠ The `identical` half is the load-bearing one. *"An override differs"* is the claim a
/// reader assumes; *"this override carries the reference value on purpose"* is the claim that
/// rots silently, because someone later "fixes" a shared placeholder into a fabricated
/// species-specific number and nothing notices.
#[test]
fn the_overrides_are_a_field_level_partition_not_a_file_level_one() {
    let canopy = ("crops/potato/canopy.yaml", "canopy.yaml");
    let pheno = ("crops/potato/phenology.yaml", "phenology.yaml");
    let root = ("crops/potato/root_depth.yaml", "root_depth.yaml");

    // Moved: the crop really owns these.
    for (file, wheat_file, name) in [
        (canopy.0, canopy.1, "specific_leaf_area"),
        (pheno.0, pheno.1, "t_base"),
        (pheno.0, pheno.1, "t_cap"),
        (pheno.0, pheno.1, "tsum_anthesis"),
        (pheno.0, pheno.1, "tsum_maturity"),
        (root.0, root.1, "max_extension_rate"),
        (root.0, root.1, "max_rooted_depth"),
    ] {
        assert_ne!(
            field(overlay(file), name, file),
            field(frozen(wheat_file), name, wheat_file),
            "{file}:{name} is the reference value — the override asserts nothing"
        );
    }

    // Carried on purpose: the file is overridden, these fields are not.
    for (file, wheat_file, name) in [
        (canopy.0, canopy.1, "extinction_coef"),
        (canopy.0, canopy.1, "carbon_fraction"),
        (pheno.0, pheno.1, "t_base_v"),
        (pheno.0, pheno.1, "t_opt_lower_v"),
        (pheno.0, pheno.1, "t_opt_upper_v"),
        (pheno.0, pheno.1, "t_ceiling_v"),
        (pheno.0, pheno.1, "vsen"),
        (pheno.0, pheno.1, "vdsat"),
        (pheno.0, pheno.1, "cpp"),
        (pheno.0, pheno.1, "ppsen"),
    ] {
        assert_eq!(
            field(overlay(file), name, file).to_bits(),
            field(frozen(wheat_file), name, wheat_file).to_bits(),
            "{file}:{name} diverged from the reference — if that is intended it needs a \
             citation and this roster needs editing, not the assertion relaxing"
        );
    }

    // The partition table is the one field with no scalar entry; a different SHAPE is the
    // strongest statement of "overridden" available for it.
    let wheat = params::allocation();
    let potato = params::potato().alloc;
    assert_ne!(
        wheat.table.len(),
        potato.table.len(),
        "the potato partition table has collapsed onto wheat's shape"
    );
    assert!(potato.table.len() >= 8, "{}", potato.table.len());
}

/// The **shared** half of the partition, at the struct level: everything the crop does not
/// own is bit-identical to the reference.
///
/// This is what makes "potato shares wheat's photosynthesis" a pinned claim rather than a
/// comment — the failure it exists for is a second species quietly acquiring its own copy of
/// a parameter nobody cited for it.
#[test]
fn every_shared_parameter_is_bit_identical_to_the_reference_crop() {
    let wheat = params::biosphere();
    let potato = params::potato();
    // Compared through the debug form: every field of every shared struct, with no roster of
    // field names here to go stale when one is added.
    for (name, a, b) in [
        (
            "photo",
            format!("{:?}", wheat.photo),
            format!("{:?}", potato.photo),
        ),
        (
            "resp",
            format!("{:?}", wheat.resp),
            format!("{:?}", potato.resp),
        ),
        (
            "transp",
            format!("{:?}", wheat.transp),
            format!("{:?}", potato.transp),
        ),
        (
            "senesc",
            format!("{:?}", wheat.senesc),
            format!("{:?}", potato.senesc),
        ),
        (
            "stem_reserve",
            format!("{:?}", wheat.stem_reserve),
            format!("{:?}", potato.stem_reserve),
        ),
        (
            "nitro",
            format!("{:?}", wheat.nitro),
            format!("{:?}", potato.nitro),
        ),
        (
            "decomp",
            format!("{:?}", wheat.decomp),
            format!("{:?}", potato.decomp),
        ),
        (
            "micro",
            format!("{:?}", wheat.micro),
            format!("{:?}", potato.micro),
        ),
        (
            "humi",
            format!("{:?}", wheat.humi),
            format!("{:?}", potato.humi),
        ),
        (
            "water",
            format!("{:?}", wheat.water),
            format!("{:?}", potato.water),
        ),
        (
            "herb",
            format!("{:?}", wheat.herb),
            format!("{:?}", potato.herb),
        ),
        // ⚠ These two are the awkward pair and they belong here for exactly that reason:
        // `potato()` loads them from the POTATO file (so the "INERT for potato" source lines
        // are falsifiable), yet every one of their eight fields must still equal the
        // reference crop's. Nothing else in this file pins them at the STRUCT level — the
        // field-level test compares YAML text — so without these entries a diverging fold, or
        // an edited value with the other test's roster updated to match, would pass.
        (
            "vern",
            format!("{:?}", wheat.vern),
            format!("{:?}", potato.vern),
        ),
        (
            "photoperiod",
            format!("{:?}", wheat.photoperiod),
            format!("{:?}", potato.photoperiod),
        ),
    ] {
        assert_eq!(a, b, "{name} is not shared with the reference crop");
    }
    // ...and the anti-vacuity half: the four it DOES own are not shared.
    assert_ne!(
        format!("{:?}", wheat.canopy),
        format!("{:?}", potato.canopy)
    );
    assert_ne!(format!("{:?}", wheat.pheno), format!("{:?}", potato.pheno));
    assert_ne!(format!("{:?}", wheat.rootd), format!("{:?}", potato.rootd));
    assert_ne!(format!("{:?}", wheat.alloc), format!("{:?}", potato.alloc));
}

/// `carbon_fraction` agreement now spans a **crop boundary**.
///
/// The kg DM ↔ mol C bridge appears in two files and the loaders fold with it in both
/// (`canopy.yaml` divides by it, `nitrogen.yaml` multiplies). Potato overrides `canopy.yaml`
/// and does **not** override `nitrogen.yaml`, so a different value in the override would model
/// a plant whose leaves and whose nitrogen demand disagree about what dry matter is —
/// silently, because the two folds sit on opposite sides of the same run.
///
/// ⚠ Unreadable from the structs: `CanopyParams` keeps only `sla_per_mol_c`, with
/// `carbon_fraction` already folded into it. That is why this reads the files.
#[test]
fn carbon_fraction_agrees_across_the_crop_boundary() {
    let potato_canopy = field(
        overlay("crops/potato/canopy.yaml"),
        "carbon_fraction",
        "crops/potato/canopy.yaml",
    );
    let frozen_nitrogen = field(frozen("nitrogen.yaml"), "carbon_fraction", "nitrogen.yaml");
    let frozen_canopy = field(frozen("canopy.yaml"), "carbon_fraction", "canopy.yaml");
    assert_eq!(
        potato_canopy.to_bits(),
        frozen_nitrogen.to_bits(),
        "the potato canopy's carbon fraction ({potato_canopy}) disagrees with the nitrogen \
         params it does NOT override ({frozen_nitrogen})"
    );
    assert_eq!(frozen_canopy.to_bits(), frozen_nitrogen.to_bits());
}

// --------------------------------------------------------------------------------- //
// "INERT for potato" is a MEASUREMENT, not a source string                            //
// --------------------------------------------------------------------------------- //

/// The vernalization/photoperiod fields really are unread — and the control proves the
/// instrument can see a field that is not.
///
/// # ⚠ Why this is not obviously true
///
/// Those fields carry wheat's values under the source line *"INERT for potato — never read"*.
/// That is a claim about the **scenario** (`potato_scenario` sets both gates false), not about
/// the file, and it is exactly the shape this repo has been wrong about before: a claim of
/// inertness needs a ladder, and a pin evaluated at its subject's symmetry point is not a pin.
/// So the mutations are absurd rather than marginal — a vernalization sensitivity 100x the
/// reference, a critical photoperiod outside any real day — and the assertion is on **bits**.
///
/// The control (`t_base`, which the same file's thermal-time loader does read) is what makes
/// the silence mean something: without it, a seam that quietly dropped every mutation would
/// pass this test perfectly.
#[test]
fn the_inert_phenology_fields_are_inert_and_a_live_one_is_not() {
    let scenario = potato_scenario();
    let base = leaf_series(&scenario, &params::potato());

    for (name, absurd) in [("vsen", 3.3), ("cpp", 23.5), ("ppsen", 9.0), ("vdsat", 5.0)] {
        let p = potato_with("crops/potato/phenology.yaml", name, absurd);
        // ⚠ The half that separates "inert in the run" from "never loaded". Without this the
        // test would pass just as happily if `potato_with` had forgotten to rebuild `vern` and
        // `photoperiod` from the mutated text — and it would then be measuring nothing at all.
        let reference = params::potato();
        assert_ne!(
            (
                format!("{:?}", reference.vern),
                format!("{:?}", reference.photoperiod)
            ),
            (format!("{:?}", p.vern), format!("{:?}", p.photoperiod)),
            "{name} := {absurd} never reached the params object"
        );
        let mutated = leaf_series(&scenario, &p);
        assert_eq!(base.len(), mutated.len());
        for (i, (a, b)) in base.iter().zip(&mutated).enumerate() {
            assert_eq!(
                a.to_bits(),
                b.to_bits(),
                "{name} := {absurd} moved the run at step {i} ({a} vs {b}) — the file's \
                 INERT-for-potato source line is false and the value needs a citation"
            );
        }
    }

    // The control. `t_base` is read by the thermal-time accumulator on every step of every
    // run, so a mutation MUST show.
    let live = leaf_series(
        &scenario,
        &potato_with("crops/potato/phenology.yaml", "t_base", 12.0),
    );
    assert!(
        base.iter()
            .zip(&live)
            .any(|(a, b)| a.to_bits() != b.to_bits()),
        "t_base := 12 changed nothing — the mutation seam is dropping its argument, and the \
         silences above are therefore evidence of nothing"
    );
}

// --------------------------------------------------------------------------------- //
// The crop reaches the RUN                                                            //
// --------------------------------------------------------------------------------- //

/// A potato season conserves every quantity, every step, and is never rationed.
///
/// The three engine invariants a piece of authored content is actually held to. Conservation
/// is the ledger form (`Inputs = Outputs + ΔStored` per quantity), asserted on every
/// consecutive pair rather than at the endpoints, because a pair of cancelling errors is
/// invisible to an endpoint check.
#[test]
fn a_potato_season_conserves_mass_and_runs_well_fed() {
    let scenario = potato_scenario();
    let p = params::potato();
    let (state, integrator, resolver) = season_setup_with(&scenario, 1, &p).expect("setup");
    let steps = steps_for_years(1);
    let mut states: Vec<State> = Vec::with_capacity(steps + 1);
    {
        let mut observe = |s: &State| states.push(s.clone());
        let (_final, rationed, events) = run_season(
            &integrator,
            state,
            &resolver,
            BIO_DT,
            steps,
            None,
            &mut observe,
        )
        .expect("the potato season runs");
        assert_eq!(
            rationed, 0,
            "the arbitration backstop fired — a rationed run is not the model's answer"
        );
        assert!(events.is_empty(), "unexpected extinction: {events:?}");
    }
    assert_eq!(states.len(), steps + 1);

    for (i, pair) in states.windows(2).enumerate() {
        let ledger = compute_ledger(&pair[0], &pair[1]).expect("ledger");
        assert!(
            !ledger.is_empty(),
            "step {i}: an empty ledger checks nothing"
        );
        for entry in &ledger {
            assert!(
                entry.residual.abs() <= 1e-9,
                "step {i}: {:?} residual {}",
                entry.quantity,
                entry.residual
            );
        }
        // The quantities a crop must actually be moving, named so a one-quantity ledger
        // cannot pass this by carrying only the easy one.
        let present: BTreeSet<&str> = ledger.iter().map(|e| e.quantity.name()).collect();
        for q in [Quantity::Carbon, Quantity::Water, Quantity::Nitrogen] {
            assert!(
                present.contains(q.name()),
                "step {i}: {q:?} absent from {present:?}"
            );
        }
    }
}

/// Two runs of the same potato season are bit-identical.
#[test]
fn the_potato_season_is_deterministic() {
    let scenario = potato_scenario();
    let a = leaf_series(&scenario, &params::potato());
    let b = leaf_series(&scenario, &params::potato());
    assert_eq!(a.len(), b.len());
    for (i, (x, y)) in a.iter().zip(&b).enumerate() {
        assert_eq!(x.to_bits(), y.to_bits(), "step {i}");
    }
}

/// The overlay reaches the run — the anti-vacuity half of everything above.
///
/// ⚠ Both runs are the same plot, so the only difference between them is the crop. If they
/// matched, every assertion in this file about "the potato" would be an assertion about the
/// winter wheat.
#[test]
fn the_potato_run_differs_from_the_reference_crop_on_the_same_plot() {
    let wheat = leaf_series(&DEFAULT_SCENARIO, &params::biosphere());
    let potato = leaf_series(&potato_scenario(), &params::potato());
    assert!(
        wheat
            .iter()
            .zip(&potato)
            .any(|(a, b)| a.to_bits() != b.to_bits()),
        "the potato run is bit-identical to the winter wheat — the overlay never arrived"
    );
}

/// The declined modifiers leave their machinery **out of the registry**, and the potato build
/// only ever subtracts from the reference crop's type set.
///
/// ⚠ Read the assertion, not the flag: `stem_reserves: false` is a scenario field, and the
/// claim worth pinning is what the assembly does with it. Potato is the first scenario in the
/// Rust tree to take that branch (measured 2026-09-06: `stem_reserves: false` appeared nowhere
/// in `rust/crates/`), so this is also that branch's first exercise by a run.
#[test]
fn the_declined_modifiers_are_absent_from_the_potato_build() {
    let wheat = type_set(&DEFAULT_SCENARIO, &params::biosphere());
    let potato = type_set(&potato_scenario(), &params::potato());
    assert!(
        wheat.contains("StemRemobilization"),
        "the reference crop stopped wiring StemRemobilization — this test's control is gone"
    );
    assert!(
        !potato.contains("StemRemobilization"),
        "potato wired StemRemobilization despite declining stem reserves: {potato:?}"
    );
    assert!(
        potato.is_subset(&wheat),
        "the potato build ADDED a type the reference crop does not have: {:?}",
        potato.difference(&wheat).collect::<Vec<_>>()
    );

    // ⚠ The comparison above confounds two differences — different params AND a different
    // scenario — so on its own it cannot say the FLAGS are what subtracted. These two hold
    // the params fixed and vary only the scenario, which is the discriminator.
    let potato_params_default_plot = type_set(&DEFAULT_SCENARIO, &params::potato());
    assert_eq!(
        potato_params_default_plot, wheat,
        "the potato PARAMS alone changed the type set — the subset above is not about the flags"
    );
    let with_reserves = type_set(
        &SeasonScenario {
            stem_reserves: true,
            ..potato_scenario()
        },
        &params::potato(),
    );
    assert!(with_reserves.contains("StemRemobilization"));
}

/// The sealed chamber does not over-draw on the second species.
///
/// The trap stage 1 flagged: a crop with a different canopy and a different partition table
/// runs in a jar sized for the reference crop, and the arbitration backstop is the only thing
/// between that and a negative pool. `rationed == 0` says it never had to be.
#[test]
fn the_sealed_chamber_does_not_over_draw_on_potato() {
    let scenario = SeasonScenario {
        vernalization: false,
        photoperiod: false,
        wssd: None,
        stem_reserves: false,
        ..sealed_chamber_scenario()
    };
    let t = trajectory(scenario, SEALED_CHAMBER_YEARS, false, &params::potato());
    assert_eq!(t.rationed, 0, "the backstop fired {} times", t.rationed);
    assert!(!t.carbon_pool.is_empty(), "a sealed run has a chamber pool");
    assert!(
        t.carbon_pool.iter().all(|c| *c > 0.0),
        "the chamber CO2 pool went non-positive"
    );
}

// --------------------------------------------------------------------------------- //
// ...and it is frozen by nothing                                                      //
// --------------------------------------------------------------------------------- //

/// The overlay is **loaded by the reference and named in no manifest** — the census stays at
/// fifteen.
///
/// ⚠ This is what would go red if someone "tidied" the four overrides into `param_files`,
/// which would freeze a set the project calls unvalidated. It compares the embedded TEXTS
/// rather than the basenames on purpose: all four basenames collide with frozen ones, so a
/// name check would pass while looking at the wrong file.
#[test]
fn the_overlay_is_loaded_by_the_reference_and_frozen_by_nothing() {
    let census = params::param_files();
    assert_eq!(census.len(), 15, "the frozen census moved");
    for (name, text) in POTATO_OVERRIDES {
        assert!(
            !census.iter().any(|(_, t)| *t == text),
            "{name} entered the frozen census — an authored overlay must not be frozen"
        );
    }
    // ...and it IS loaded: the params object differs from the reference crop's.
    assert_ne!(
        format!("{:?}", params::biosphere().rootd),
        format!("{:?}", params::potato().rootd)
    );
    // The ordinary entry point, exercised once so a break in it cannot hide behind
    // `build_season_with`.
    assert!(build_season(&potato_scenario()).is_ok());
}
