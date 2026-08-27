//! The **basis** under the Tier-2 bands — the `domains` half.
//!
//! Ported from `tests/crossport/test_crossport.py`'s three `*_sits_above_measured_sensitivity`
//! tests and the instrument they call, `tests/crossport/measure_tier2_bands.py` (reference
//! flip, Stage 3, the half D4 deferred). The station's two live in
//! `station/tests/tier_sensitivity.rs`.
//!
//! # What this adds that `tier_contract.rs` cannot
//!
//! That file asserts every golden sits **inside** its band. This one asserts the band is
//! **justified**: that it sits above the freshly measured ±1-ULP transcendental sensitivity,
//! which is the "measured, never derived" clause of `docs/native-port-reference.md`. Until it
//! existed the numbers in `rust/data/tiers.json` were asserted by the reference and justified
//! only by a Python program built out of the tree S6 deletes.
//!
//! ⚠ A band test is only as good as its probe, and this repo has a probe that measured
//! **exactly 0.0** for weeks after the code moved out from under it. So the gates below come
//! in three layers, and the last one is the one Python never had: the harness must reproduce
//! the golden with the nudge off, every sensitivity must be non-zero, and every sensitivity
//! must land within an order of magnitude of the figure the Python instrument measured for
//! the same scenario.
//!
//! ⚠ **The third layer is a reach check, not a subject check, and the battery measured how
//! loose it is.** A 16-ULP nudge reddens only one of the four band tests, and pointing the
//! biosphere seam at a *different* forcing (`vpd` instead of `par`) reddens nothing at all —
//! because a one-ULP perturbation of ANY input to these trajectories lands in the same order
//! of magnitude. It earned its place anyway: chasing an exact factor of two *inside* it is
//! what produced the compensated-`sum()` finding. The seam's identity is defended by the
//! grep-checked argument below, not by a magnitude comparison.

use domains::goldens;
use domains::tiers;
use domains::ulp_probe::{self, Nudge};

/// The Python instrument's readings, re-run 2026-08-27 and identical to `rust/data/tiers.json`'s
/// `_reference_flip.measured_2026_08_16` block — what this is a *re*-measurement of.
///
/// ⚠ The reference does **not** reproduce all four exactly, and the two reasons are measured
/// rather than assumed (both written up in `tiers.json`'s `measured_2026_08_27_by_the_reference`
/// block). Thermal matches to every digit. The two power runs read **half**, because CPython's
/// builtin `sum()` has been Neumaier-compensated since 3.12 while
/// [`domains::power::daily_solar_energy`] accumulates naively — compensating that one sum in a
/// throwaway probe reproduced both Python figures bit for bit. The biosphere reads 0.70×,
/// because a one-ULP step is a relative perturbation of between `2⁻⁵³` and `2⁻⁵²` depending on
/// where the value sits in its binade, and the `par` seam and the `exp` it stands in for sit in
/// different binades. So the window below is an order of magnitude on purpose: it is wide
/// enough for a granularity difference and far too narrow for a probe that missed its subject.
const PYTHON_POWER_BOUNDED_SOC: f64 = 5.215406e-15;
const PYTHON_POWER_SELF_DISCHARGE: f64 = 4.146325e-15;
const PYTHON_THERMAL_EQUILIBRIUM: f64 = 1.909423e-16;
const PYTHON_BIOSPHERE_PERENNIAL_15YR: f64 = 3.519726e-15;

/// The seven biosphere `tiers.json` keys that share one band.
const BIOSPHERE_KEYS: [&str; 7] = [
    "open_season",
    "sealed_chamber",
    "perennial_chamber",
    "perennial_long_horizon",
    "consumer_chamber",
    "consumer_long_horizon",
    "drift_summary",
];

fn entry(key: &str) -> tiers::TierEntry {
    tiers::entries()
        .into_iter()
        .find(|e| e.key == key)
        .unwrap_or_else(|| panic!("no tiers.json entry keyed {key:?}"))
}

fn band_of(key: &str) -> f64 {
    entry(key)
        .band
        .unwrap_or_else(|| panic!("{key}: Tier-2 band unmeasured"))
}

/// The three claims every measured sensitivity must satisfy, in the order they can fail.
fn assert_justifies(key: &str, measured: f64, python: f64, leaf: &str) {
    // Captured unless `-- --nocapture`, so it costs nothing and the instrument can be read
    // back without editing it — the same reason `tiers::compare_at_tier` returns its measurement.
    eprintln!("MEASURED {key} = {measured:.6e} (python {python:.3e}) leaf={leaf}");
    // 1. The probe reached its subject at all. A re-measurement that reads zero is the
    //    failure mode, not the result — `measure_tier2_bands.py`'s own comments record both
    //    biosphere probes doing exactly this for weeks while the gate kept passing.
    assert!(
        measured > 0.0,
        "{key}: the ±1-ULP probe moved nothing — it is shimming something the run does not \
         reach (worst leaf {leaf:?})"
    );
    // 2. It reached the RIGHT subject. Non-zero is not enough: 1e-30 is non-zero and wrong,
    //    and only the number this re-measures can see that.
    assert!(
        measured > python / 10.0 && measured < python * 10.0,
        "{key}: measured ±1-ULP sensitivity {measured:.3e} is more than an order of magnitude \
         from the Python instrument's {python:.3e} — the two ports' bands are sized against \
         the same dynamics, so this is a finding about the probe or about the port, not a \
         number to write down (worst leaf {leaf:?})"
    );
    // 3. The band is justified, and still tight enough to catch a real port defect.
    let band = band_of(key);
    assert!(
        measured < band,
        "{key}: measured ±1-ULP sensitivity {measured:.3e} is not below the tiers.json band \
         {band:.3e}"
    );
    assert!(
        band <= 1e-9,
        "{key}: band {band:.3e} is too loose — a Tier-2 band must still catch a port defect"
    );
}

// --------------------------------------------------------------------------- //
// Layer 1 — the harness is the run it claims to perturb                        //
// --------------------------------------------------------------------------- //

/// ⚠⚠ **The control that makes every number below mean something.** Each probe run
/// assembles the scenario itself (it must, to reach inside for the seam), so it could drift
/// from `goldens::*` and go on measuring the sensitivity of a scenario nobody froze. With
/// the nudge off it must emit the golden emitter's **exact bytes**.
#[test]
fn the_probe_harness_reproduces_its_golden_with_the_nudge_off() {
    assert_eq!(
        ulp_probe::power_snapshot(false, Nudge::Off),
        goldens::power(),
        "the power probe harness has drifted from goldens::power"
    );
    assert_eq!(
        ulp_probe::power_snapshot(true, Nudge::Off),
        goldens::power_self_discharge(),
        "the self-discharge probe harness has drifted from goldens::power_self_discharge"
    );
    assert_eq!(
        ulp_probe::thermal_snapshot(Nudge::Off),
        goldens::thermal(),
        "the thermal probe harness (which SWAPS the radiator flow) has drifted from \
         goldens::thermal — with the nudge off the mirrored flow must be the frozen one"
    );
}

/// The biosphere half of the same control, at the horizon the band is measured on.
#[test]
fn the_biosphere_probe_harness_reproduces_its_golden_with_the_nudge_off() {
    assert_eq!(
        ulp_probe::perennial_snapshot(
            &domains::biosphere::perennial_chamber_scenario(),
            domains::biosphere::LONG_HORIZON_YEARS,
            Nudge::Off,
        ),
        goldens::perennial_chamber(domains::biosphere::LONG_HORIZON_YEARS),
        "the perennial probe harness has drifted from goldens::perennial_chamber"
    );
    assert_eq!(
        ulp_probe::perennial_snapshot(
            &domains::biosphere::consumer_chamber_scenario(),
            domains::biosphere::LONG_HORIZON_YEARS,
            Nudge::Off,
        ),
        goldens::consumer_chamber(domains::biosphere::LONG_HORIZON_YEARS),
        "the consumer probe harness has drifted from goldens::consumer_chamber"
    );
}

// --------------------------------------------------------------------------- //
// Layer 2 — the nudge is one ULP, and a missing seam is an error               //
// --------------------------------------------------------------------------- //

#[test]
fn a_nudge_moves_exactly_one_ulp_in_the_named_direction() {
    for x in [1.0_f64, 1234.5678, 1e-8, f64::MIN_POSITIVE] {
        for value in [x, -x] {
            let up = Nudge::Up.apply(value);
            let down = Nudge::Down.apply(value);
            assert!(up > value, "{value:e}: Up must increase");
            assert!(down < value, "{value:e}: Down must decrease");
            // One ULP and no more: stepping back the other way returns the original bits.
            assert_eq!(Nudge::Down.apply(up).to_bits(), value.to_bits());
            assert_eq!(Nudge::Up.apply(down).to_bits(), value.to_bits());
            assert_eq!(Nudge::Off.apply(value).to_bits(), value.to_bits());
        }
    }
}

/// ⚠ Zero is an identity by design, and this pins the design rather than the code: a
/// `nextafter(0, ±∞)` is the smallest subnormal, a relative change of infinity that no libm
/// disagreement can produce. Left in, it would put light in the canopy at midnight — the
/// solar and PAR schedules both return a literal `0.0` outside their window.
#[test]
fn a_nudge_leaves_zero_alone() {
    for nudge in [Nudge::Up, Nudge::Down, Nudge::Off] {
        assert_eq!(nudge.apply(0.0).to_bits(), 0.0_f64.to_bits());
    }
}

/// ⚠ The two directions are **not the same measurement**, and thermal is where it shows:
/// nudging `t⁴` up moves the run by `1.909e-16` and nudging it down moves it by **exactly
/// zero**. So a probe that measured one direction only could read `0.0` and trip the
/// non-zero gate for a reason that has nothing to do with its seam being wired up.
///
/// ⚠ Worth stating because the battery measured it: taking the worse of the two is **inert on
/// today's roster** — the `Up` reading dominates in all six groups, so a `Up`-only probe
/// reports the same six numbers and no test notices. That is a property of these trajectories,
/// not of the design; a libm disagreement has no preferred sign. This test is what stops the
/// rule from being deleted as dead weight.
#[test]
fn the_two_nudge_directions_are_not_the_same_measurement() {
    let base = ulp_probe::thermal_snapshot(Nudge::Off);
    let up = ulp_probe::worst_relative_deviation(
        &base,
        &ulp_probe::thermal_snapshot(Nudge::Up),
        ulp_probe::FLOOR,
    )
    .0;
    let down = ulp_probe::worst_relative_deviation(
        &base,
        &ulp_probe::thermal_snapshot(Nudge::Down),
        ulp_probe::FLOOR,
    )
    .0;
    assert_ne!(
        up, down,
        "the two nudge directions read the same on the thermal run — either the perturbation          has become sign-symmetric or one direction is not being applied"
    );
}

/// A probe that shims a variable the run does not read measures 0.0 and passes vacuously.
/// Both seams refuse rather than silently perturb nothing.
#[test]
fn a_seam_that_finds_no_subject_is_an_error() {
    let scenario = domains::power::BOUNDED_SOC_SCENARIO;
    let charge = domains::params::charge();
    let resolver = domains::power::power_resolver(&charge, &scenario).expect("power_resolver");
    assert!(
        ulp_probe::nudge_forcing(resolver, "no_such_forcing", Nudge::Up).is_err(),
        "nudging an absent forcing must be an error, not a no-op"
    );

    // A registry with no RadiatorReject in it — the standalone Power one.
    let (state, registry) =
        domains::power::build_power(&charge, &scenario, None).expect("build_power");
    let params = domains::params::thermal();
    assert!(
        ulp_probe::nudge_radiator(registry, &state.stocks, &params, Nudge::Up).is_err(),
        "swapping an absent radiator flow must be an error, not a no-op"
    );
}

// --------------------------------------------------------------------------- //
// Layer 3 — the bands are justified                                            //
// --------------------------------------------------------------------------- //

/// The three Step-3 sibling bands (power / power-self-discharge / thermal).
#[test]
fn the_sibling_bands_sit_above_the_measured_sensitivity() {
    let (measured, leaf) = ulp_probe::power_sensitivity(false);
    assert_justifies(
        "power_bounded_soc",
        measured,
        PYTHON_POWER_BOUNDED_SOC,
        &leaf,
    );

    let (measured, leaf) = ulp_probe::power_sensitivity(true);
    assert_justifies(
        "power_self_discharge",
        measured,
        PYTHON_POWER_SELF_DISCHARGE,
        &leaf,
    );

    let (measured, leaf) = ulp_probe::thermal_sensitivity();
    assert_justifies(
        "thermal_equilibrium",
        measured,
        PYTHON_THERMAL_EQUILIBRIUM,
        &leaf,
    );
}

/// The one band all seven biosphere goldens share, measured on the worse of the two 15-year
/// sealed runs — the Python instrument's representative pair.
#[test]
fn the_biosphere_band_sits_above_the_measured_sensitivity() {
    let (measured, leaf) = ulp_probe::biosphere_sensitivity(domains::biosphere::LONG_HORIZON_YEARS);
    for key in BIOSPHERE_KEYS {
        assert_eq!(
            band_of(key),
            1e-11,
            "{key}: the seven biosphere goldens must share one band"
        );
        assert_justifies(key, measured, PYTHON_BIOSPHERE_PERENNIAL_15YR, &leaf);
    }
}

/// ⚠ The metric walks every hex-float leaf of the snapshot, where the Python instrument
/// compared **stock amounts only**. That is a superset, so it can only raise the maximum —
/// but "conservative" is a claim, and this checks it: the leaf that actually produces the
/// worst deviation is a stock amount, so the two instruments are reading the same thing.
#[test]
fn the_worst_leaf_is_a_stock_amount_as_the_python_instrument_compared() {
    for (name, leaf) in [
        ("power", ulp_probe::power_sensitivity(false).1),
        ("thermal", ulp_probe::thermal_sensitivity().1),
    ] {
        assert!(
            leaf.contains("stocks"),
            "{name}: worst leaf {leaf:?} is not a stock — the snapshot superset has changed \
             what this measures relative to the Python instrument"
        );
    }
}
