//! The **temperature form** of the FvCB kinetics — the lab's first alternative form of a
//! biosphere process, and the controls that say what its columns mean.
//!
//! Plan: `docs/plans/post-roadmap-temperature-kinetics.md`. The tree scales the whole
//! assimilation rate by one cardinal-temperature multiplier over constants held at 25 °C;
//! [D] Teh ch. 6 gives each kinetic constant its own Q10 and the light branch its own
//! response instead. Both are cited. `KineticsForm` selects between them over the **same
//! frozen numbers**, so a difference here is attributable to the form.
//!
//! # ⚠ Everything here is measured through `lab::report::measure`
//!
//! The first draft of this file carried its own list of the six frozen runs and its own
//! folds. Both are already in `report::SPECS`, and a second copy is the defect this repo
//! keeps finding: a roster that is hand-listed goes stale silently (`report` grew a
//! fixed-point row this file did not have, and the count assertion is what caught it), and a
//! duplicated fold is a control that can drift from the thing it controls. So the roster is
//! derived and the folds are the report's.
//!
//! # What each test is for
//!
//! * [`the_cardinal_form_is_the_frozen_run_on_every_measured_quantity`] — the seam adds
//!   nothing. ⚠ **Not a round trip**: `lab::biosphere_with_form(&[], Cardinal)` reaches its
//!   params by parsing the frozen file text, `params::biosphere()` through the ordinary
//!   loader — two routes to one object, which is what the value harness's empty-substitution
//!   control has and what the mechanism half's empty-drop control lacks.
//! * [`the_q10_form_moves_every_measured_quantity`] — the mis-target guard, and the plan's
//!   own falsification criterion: a form reaching no rate law would leave `open_season`'s
//!   peak LAI at 6.022837 and the column would read "the alternative is inert".
//! * [`the_mutual_shading_step_is_what_caps_peak_lai_under_the_q10_form`] — the finding, run
//!   as a 2×2 rather than asserted.

use domains::biosphere::params;
use domains::biosphere::science::KineticsForm;
use domains::lab::report::{measure, Column, SPECS};
use domains::lab::{biosphere_with_form, Substitution};

/// Every measured quantity under `form`, at the long horizon so the decade rows are included.
fn column(form: KineticsForm, subs: &[Substitution]) -> Column {
    let p = biosphere_with_form(subs, form).expect("the frozen files parse");
    measure("probe", &p, true)
}

/// The spec index of `open_season`'s peak LAI, resolved by name rather than assumed to be 0.
fn peak_lai_spec() -> usize {
    SPECS
        .iter()
        .position(|s| s.scenario == "open_season" && s.quantity.starts_with("peak LAI"))
        .expect("the report measures open_season's peak LAI")
}

fn value(c: &Column, spec: usize) -> f64 {
    c.values
        .iter()
        .find(|(i, _)| *i == spec)
        .map(|(_, v)| *v)
        .expect("every spec is measured on a run that survives")
}

#[test]
fn the_cardinal_form_is_the_frozen_run_on_every_measured_quantity() {
    let by_loader = measure("frozen", &params::biosphere(), true);
    let by_form = column(KineticsForm::Cardinal, &[]);

    // Bit for bit. A tolerance would let the seam introduce a reordering and still pass, and
    // the whole claim is that it introduces nothing.
    assert_eq!(by_loader.values, by_form.values);
    assert!(
        by_form.failed.is_empty() && by_form.not_applicable.is_empty(),
        "the Cardinal column did not measure cleanly: {:?} / {:?}",
        by_form.failed,
        by_form.not_applicable
    );
    // The default matters as much as the branch: params built by the loader must already BE
    // `Cardinal`, or every golden in the tree is running an unlabelled form.
    assert_eq!(params::biosphere().photo.kinetics, KineticsForm::Cardinal);
}

#[test]
fn the_q10_form_moves_every_measured_quantity() {
    let base = column(KineticsForm::Cardinal, &[]);
    let q10 = column(KineticsForm::Q10Teh, &[]);
    assert_eq!(
        base.values.len(),
        SPECS.len(),
        "a spec went unmeasured, so 'every quantity moved' would be a claim about a subset"
    );
    for (spec, b) in &base.values {
        let q = value(&q10, *spec);
        assert!(
            (q - b).abs() > 1e-9,
            "{}/{}: the Q10 form left it at {b} — the form is not reaching the rate law, \
             which is a mis-target and not a finding",
            SPECS[*spec].scenario,
            SPECS[*spec].quantity
        );
    }
    // The plan's falsification criterion, spelled as the number it names.
    let lai = value(&q10, peak_lai_spec());
    assert!(
        (lai - 6.022837).abs() > 1e-3,
        "open_season peak LAI came back at the frozen value {lai}"
    );
}

/// **The finding.** The Q10 form raises leaf-level season-integrated assimilation ~56 % (the
/// plan's §3 table) and above-ground biomass 37 %, yet peak LAI moves only +3.5 %. The claim
/// is that the 5 %/day mutual-shading loss above LAI 6 absorbs the rest.
///
/// ⚠ **A causal claim earns the experiment that removes the cause**, so this is a 2×2: only
/// disabling the loss under BOTH forms separates "the step caps this form" from "the step
/// caps everything". Measured 2026-09-04 — frozen 6.022837 → 6.022837 (the loss is **exactly**
/// inert there: the tree's peak sits a hair over the threshold and is reached before the term
/// can bite), Q10 6.232730 → 13.544978.
///
/// ⚠ The frozen release being **exactly zero** is why this asserts two absolute facts rather
/// than a ratio between them. A ratio test against a zero denominator passes for a reason
/// that has nothing to do with the form — the degenerate-comparison trap this repo has
/// shipped before.
///
/// `shade_rate = 0.0` is inside the frozen bound (`require_non_negative`), so this is an
/// ordinary substitution and not an unfreeze.
#[test]
fn the_mutual_shading_step_is_what_caps_peak_lai_under_the_q10_form() {
    let off = [Substitution::resolve("shade_rate", 0.0).expect("one owner")];
    let spec = peak_lai_spec();
    let lai = |form, subs: &[Substitution]| value(&column(form, subs), spec);

    let cardinal_on = lai(KineticsForm::Cardinal, &[]);
    let cardinal_off = lai(KineticsForm::Cardinal, &off);
    let q10_on = lai(KineticsForm::Q10Teh, &[]);
    let q10_off = lai(KineticsForm::Q10Teh, &off);
    println!(
        "peak LAI 2x2 — cardinal {cardinal_on:.6} -> {cardinal_off:.6} (loss off), \
         q10 {q10_on:.6} -> {q10_off:.6} (loss off)"
    );

    assert!(
        (cardinal_off - cardinal_on).abs() < 1e-9,
        "the mutual-shading loss is no longer inert in the frozen tree ({cardinal_on} -> \
         {cardinal_off}) — then it is not the FORM that made the term load-bearing, and the \
         finding below is about the tree instead"
    );
    assert!(
        q10_off - q10_on > 5.0,
        "the loss releases only {} of LAI under the Q10 form — it is not what absorbs the \
         form's extra carbon",
        q10_off - q10_on
    );
    // And the released canopy leaves the recorded band, which the capped one does not: the
    // step is the only reason the Q10 column reads as band-compliant.
    assert!(
        q10_off > 8.0,
        "with the loss removed the Q10 form peaks at {q10_off}, still inside the recorded \
         5.0 < peak < 8.0 band"
    );
    assert!(
        q10_on < 8.0,
        "the capped Q10 run peaks at {q10_on}, outside the band it is reported as clearing"
    );
}
