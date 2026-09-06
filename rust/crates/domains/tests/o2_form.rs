//! The **oxygen form** of the FvCB kinetics — the lab's second alternative form, and the
//! controls that say what its columns mean.
//!
//! Plan: `docs/plans/post-roadmap-o2-form.md`. The frozen tree reads O₂ from a constant, the
//! atmosphere's 210 mmol/mol. Three frozen scenarios are sealed chambers carrying O₂ as a
//! live stock, and one of them ends its golden at 0.033 mmol/mol — a factor of 6329.
//! `O2Form` selects between the constant and the stock over the **same frozen numbers**.
//!
//! # ⚠ Why this file's central assertion is that something MOVED
//!
//! [`O2Form`] is only half a switch. The form rides the params object; the VALUE it reads is
//! a stock, supplied per step by `CarbonContext::o2_pool_var`. A scenario with no O₂ pool
//! falls through to the frozen constant — right for the open field, which breathes the
//! atmosphere, and indistinguishable from *forgetting to wire the field at all*. That is
//! precisely the failure `lab::biosphere_with`'s header names: patch the wrong symbol, get a
//! clean run, read the baseline back, report "no effect" as a finding.
//!
//! No arrangement of the code can tell those apart, so the guard is a test:
//! [`the_sealed_chamber_moves_under_the_live_form`]. If it ever goes green by reading the
//! baseline, everything else here passes too.
//!
//! # ⚠ Which halves can be seen, and which cannot
//!
//! O₂ enters FvCB twice — the Rubisco denominator `Kc·(1 + O/Ko)` and `Γ* ∝ O`. The
//! denominator **saturates** by about 2 mmol/mol, which is *above* the jar's charge, so a
//! form that coupled only the denominator would be nearly invisible on the gated rows while
//! a form that dropped Γ* would be loud. Both are pinned here at the arithmetic level
//! ([`gamma_star_is_linear_in_the_oxygen_fraction`],
//! [`the_rubisco_denominator_saturates_above_the_jars_charge`]) rather than left to a
//! whole-run mutation that the saturation would swallow.

use domains::biosphere::params;
use domains::biosphere::science::{
    kinetics_at, o2_coupled, o2_mole_fraction, oxygen_at, rubisco_limited_rate, KineticsForm,
    O2Form, MMOL_PER_MOL,
};
use domains::lab::report::{compare_changes, measure, render, Change, Column, SPECS};
use domains::lab::biosphere_with_o2_form;

/// Every measured quantity under `form`, at the long horizon so the decade rows are included.
fn column(form: O2Form) -> Column {
    let p = biosphere_with_o2_form(&[], form).expect("the frozen files parse");
    measure("probe", &p, true)
}

/// The spec index of one scenario's quantity, resolved by name rather than assumed.
fn spec(scenario: &str, quantity_prefix: &str) -> usize {
    SPECS
        .iter()
        .position(|s| s.scenario == scenario && s.quantity.starts_with(quantity_prefix))
        .unwrap_or_else(|| panic!("the report measures {scenario} / {quantity_prefix}"))
}

fn value(c: &Column, i: usize) -> f64 {
    c.values
        .iter()
        .find(|(j, _)| *j == i)
        .map(|(_, v)| *v)
        .expect("every spec is measured on a run that survives")
}

fn rel_change(a: f64, b: f64) -> f64 {
    ((b - a) / a).abs()
}

// --- the seam adds nothing -------------------------------------------------

#[test]
fn the_constant_form_is_the_frozen_run_on_every_measured_quantity() {
    let by_loader = measure("frozen", &params::biosphere(), true);
    let by_form = column(O2Form::Constant);

    // Bit for bit. A tolerance would let the seam introduce a reordering and still pass, and
    // the whole claim is that it introduces nothing.
    assert_eq!(by_loader.values, by_form.values);
    assert!(
        by_form.failed.is_empty() && by_form.not_applicable.is_empty(),
        "the Constant column did not measure cleanly: {:?} / {:?}",
        by_form.failed,
        by_form.not_applicable
    );
    // The default matters as much as the branch: params built by the loader must already BE
    // `Constant`, or every golden in the tree is running an unlabelled form.
    assert_eq!(params::biosphere().photo.o2_form, O2Form::Constant);
}

// --- THE guard: the live form must MOVE the one scenario it can reach ------

#[test]
fn the_sealed_chamber_moves_under_the_live_form() {
    let base = column(O2Form::Constant);
    let live = column(O2Form::LivePool);
    let jar = spec("sealed_chamber", "season-low chamber CO2");

    let (before, after) = (value(&base, jar), value(&live, jar));
    // ⚠ Not "differs by more than epsilon" — the jar's O₂ ends 6329× below the constant, so
    // a correctly wired form moves this row by a large fraction. A tiny difference here
    // would mean the form reached the run but not the rate law.
    assert!(
        rel_change(before, after) > 0.5,
        "sealed_chamber season-low CO2 barely moved under the live form: {before} -> {after}. \
         Either `o2_pool_var` is not wired for this scenario (in which case `photo_at` falls \
         through to the frozen constant and this column IS the baseline), or the form is not \
         reaching `canopy_assimilation`."
    );
}

#[test]
fn the_open_field_is_unreachable_by_the_live_form_by_construction() {
    let base = column(O2Form::Constant);
    let live = column(O2Form::LivePool);
    for quantity in ["peak LAI", "peak W"] {
        let i = spec("open_season", quantity);
        // Bit-identical, and this is a statement about the WIRING: `open_season` has no O₂
        // stock, so `photo_at` hands over the frozen params whatever form is selected.
        assert_eq!(
            value(&base, i),
            value(&live, i),
            "open_season / {quantity} moved under a form that cannot reach it"
        );
    }
    // The same claim at the arithmetic level, so the row above cannot pass for the wrong
    // reason (e.g. a form that is inert everywhere).
    let p = params::biosphere().photo;
    let untouched = oxygen_at(&p, None, O2Form::LivePool);
    assert_eq!(untouched.o2, p.o2);
    assert_eq!(untouched.gamma_star, p.gamma_star);
}

#[test]
fn the_two_big_chambers_are_the_controls_and_barely_move() {
    let base = column(O2Form::Constant);
    let live = column(O2Form::LivePool);
    // ⚠ These are the real controls. Both sit at ~210.2 mmol/mol all season, so the live
    // form must agree with the constant to about a tenth of a percent. A form wired
    // GLOBALLY — the failure mode the 2026-09-02 value-switch columns had by construction —
    // would move them as much as it moves the jar.
    for scenario in ["perennial_chamber", "consumer_chamber"] {
        let i = spec(scenario, "season-low chamber CO2");
        let (before, after) = (value(&base, i), value(&live, i));
        assert!(
            rel_change(before, after) < 0.005,
            "{scenario} moved {before} -> {after} under the live O2 form; it sits at ~210 \
             mmol/mol all season and must be a control, not a result"
        );
    }
}

// --- the arithmetic, pinned where a whole-run mutation would be swallowed ---

#[test]
fn the_live_form_at_the_atmospheric_fraction_is_the_constant_form_bit_for_bit() {
    let p = params::biosphere().photo;
    // The identity control, and it is exact rather than close: the ratio is `p.o2 / p.o2`,
    // which is 1.0 in binary floating point, and `x * 1.0 == x`.
    let same = o2_coupled(&p, p.o2);
    assert_eq!(same.o2, p.o2);
    assert_eq!(same.gamma_star, p.gamma_star);
    assert_eq!(same.kc, p.kc);
    assert_eq!(same.ko, p.ko);
    assert_eq!(same.vcmax, p.vcmax);
}

#[test]
fn gamma_star_is_linear_in_the_oxygen_fraction() {
    let p = params::biosphere().photo;
    // ⚠ THE half a mutation must be aimed at. Dropping this scaling leaves the denominator
    // half intact, and the denominator saturates above the jar's charge — so a form missing
    // Γ* would look almost right on the gated rows. Pinned as a shape, not at a point.
    let one = o2_coupled(&p, p.o2).gamma_star;
    for factor in [0.5, 0.25, 2.0, 1.0e-3] {
        let got = o2_coupled(&p, p.o2 * factor).gamma_star;
        let want = one * factor;
        assert!(
            (got - want).abs() <= 4.0 * f64::EPSILON * want.abs(),
            "Γ*({factor}·O) = {got}, linearity wants {want}"
        );
    }
    // ...and zero oxygen is a zero compensation point, which is the limb the jar's tail
    // approaches.
    assert_eq!(o2_coupled(&p, 0.0).gamma_star, 0.0);
}

#[test]
fn the_rubisco_denominator_saturates_above_the_jars_charge() {
    let p = params::biosphere().photo;
    // Γ* held at the frozen value on purpose: this isolates the DENOMINATOR half, which is
    // the half the form's original wording named and the half that turns out to be nearly
    // blind. The term is `Kc·(1 + O/Ko)` with `Ko = 278.4 mmol/mol`, so it flattens once
    // `O ≪ Ko` — and the jar spends its whole season below 2.0.
    let denom = |o: f64| 1.0 + o / p.ko;
    let (atmos, charge, end) = (210.0, 2.0, 0.033_185_896_524_892);
    let atmos_to_charge = (denom(atmos) - denom(charge)).abs() / denom(charge);
    let charge_to_end = (denom(charge) - denom(end)).abs() / denom(charge);
    // The saturation, as a RATIO rather than an absolute tolerance: falling from the
    // atmosphere to the jar's charge moves the term two orders of magnitude more than the
    // jar's entire remaining season does.
    assert!(
        atmos_to_charge / charge_to_end > 50.0,
        "the denominator was expected to be far flatter below the jar's charge than above          it, but atmosphere->charge is {atmos_to_charge} and charge->end is {charge_to_end}"
    );

    // ⚠⚠ AND THE SCOPE OF THE 2026-09-02 "SATURATES" FINDING IS CORRECTED HERE. That record
    // reads *"the denominator saturates by ~2 mmol/mol (`o2=2` and `o2=0.033` agree to six
    // figures)"*. Six figures is true of the **whole-run gated observable** — the chamber's
    // season-low CO₂ — and NOT of the leaf-level rate it was phrased as being about: `Ac`
    // moves ~4e-3 between the two, which this pins so the stronger reading cannot be quoted
    // from here. The run-level agreement is the chamber being carbon-limited, not the
    // arithmetic being flat. *A finding is scoped to the observable it was measured on.*
    let mut at_charge = p;
    at_charge.o2 = charge;
    let mut at_end = p;
    at_end.o2 = end;
    let ci = 300.0;
    let (a, b) = (
        rubisco_limited_rate(ci, &at_charge),
        rubisco_limited_rate(ci, &at_end),
    );
    assert!(
        (1.0e-4..1.0e-2).contains(&rel_change(a, b)),
        "Ac between the jar's charge and its end moved {} ({a} -> {b}); measured 4.06e-3 on          2026-09-06, and the point of the bound is that it is neither six figures nor large",
        rel_change(a, b)
    );
    // ...and it is NOT flat against the atmospheric value, or the term would be inert
    // everywhere and the form would have only one half worth building.
    let atmospheric = rubisco_limited_rate(ci, &p);
    assert!(
        rel_change(atmospheric, a) > 0.05,
        "Ac at 210 mmol/mol ({atmospheric}) and at 2.0 ({a}) should differ substantially"
    );
}

#[test]
fn the_oxygen_and_temperature_forms_commute() {
    let p = params::biosphere().photo;
    let x = 2.0;
    let t = 12.0;

    // O₂ first, then temperature.
    let mut a = p;
    a.o2_form = O2Form::LivePool;
    let a = kinetics_at(t, &o2_coupled(&a, x), KineticsForm::Q10Teh).0;

    // Temperature first, then O₂.
    let b = kinetics_at(t, &p, KineticsForm::Q10Teh).0;
    let b = o2_coupled(&b, x);

    // ⚠ Both act on Γ* multiplicatively, so the composition is order-independent — a
    // property to pin rather than assume, because a table with both switches thrown means
    // nothing if it is false. Not bit-exact: the two orders associate the multiplications
    // differently and can land a few ULP apart.
    assert!(
        (a.gamma_star - b.gamma_star).abs() <= 8.0 * f64::EPSILON * b.gamma_star.abs(),
        "Γ* depends on the order the two forms are applied: {} vs {}",
        a.gamma_star,
        b.gamma_star
    );
    assert_eq!(a.o2, b.o2);
    assert_eq!(a.vcmax, b.vcmax);
}

#[test]
fn the_mole_fraction_is_millimoles_per_mole_and_clamps_a_negative_stock() {
    // The one place a stock in mol meets a param in mmol/mol; a silent factor of 1000 here
    // looks exactly like a form that does nothing.
    assert_eq!(o2_mole_fraction(2.0, 1000.0), 2.0);
    assert_eq!(o2_mole_fraction(1.0, MMOL_PER_MOL), 1.0);
    assert_eq!(o2_mole_fraction(-1.0e-9, 1000.0), 0.0);
}

// --- the report must refuse the floor it can no longer compute -------------

#[test]
fn the_constant_floor_is_refused_rather_than_printed_stale_under_the_live_form() {
    let base = column(O2Form::Constant);
    let live = column(O2Form::LivePool);
    assert!(
        base.floor_ppm.is_some(),
        "the frozen form has a single compensation floor and must print it"
    );
    // ⚠ Under the live form Γ* tracks a stock, so there is no one floor. The params object
    // still holds 42.75 — the substitution happens per step inside the flow — so a naive
    // `floor_ppm(p)` would print 61.071429 and invite a reader to divide by it.
    assert!(
        live.floor_ppm.is_none(),
        "the live-O2 column printed a constant floor of {:?}; Γ* is not constant under it",
        live.floor_ppm
    );

    let out = render(&[base, live], true);
    assert!(
        out.contains("Γ* tracks the live O2 stock under this form"),
        "the rendered report does not say why the live column has no floor:\n{out}"
    );
}

#[test]
fn a_live_oxygen_column_is_built_through_the_seam_and_not_by_poking_the_field() {
    // The `Change` route and the direct route must land on the same numbers. A reader sees
    // the report's column, and it must not be able to drift from the seam every other caller
    // uses — the discipline `Change::Form` already carries, applied to its sibling.
    let columns = compare_changes(
        &[("o2 form live".to_string(), Change::OxygenForm(O2Form::LivePool))],
        true,
    )
    .expect("the live-O2 column measures");
    assert_eq!(columns.len(), 2, "a baseline and one variant");
    assert_eq!(columns[1].values, column(O2Form::LivePool).values);
    assert!(columns[1].floor_ppm.is_none());
    // ...and the baseline column the table is read against still has its floor.
    assert!(columns[0].floor_ppm.is_some());
}
