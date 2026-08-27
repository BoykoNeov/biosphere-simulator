//! The **basis** under the Tier-2 bands — the station half.
//!
//! Ported from `tests/crossport/test_crossport.py`'s two station
//! `*_sits_above_measured_sensitivity` tests (reference flip, Stage 3, the half D4 deferred).
//! The three-layer discipline and the reason for it are in `domains/tests/tier_sensitivity.rs`.

use domains::tiers;
use domains::ulp_probe::Nudge;
use station::goldens;
use station::ulp_probe::{self, EnergySeam};

/// The Python instrument's readings, re-run 2026-08-27 and identical to `tiers.json`'s
/// `_reference_flip.measured_2026_08_16` block.
const PYTHON_STATION_ENERGY: f64 = 5.215406e-15;
const PYTHON_GREENHOUSE: f64 = 2.814887e-16;

/// Power→Thermal only, no biosphere.
const STATION_ENERGY_KEYS: [&str; 2] = ["station_heat_closure", "sealed_energy_drift"];
/// The four whose graph reaches an FvCB transcendental — they borrow the biosphere band.
const STATION_BIOSPHERE_KEYS: [&str; 4] = ["greenhouse", "lighting", "harvest", "sealed_station"];

fn band_of(key: &str) -> f64 {
    tiers::entries()
        .into_iter()
        .find(|e| e.key == key)
        .unwrap_or_else(|| panic!("no tiers.json entry keyed {key:?}"))
        .band
        .unwrap_or_else(|| panic!("{key}: Tier-2 band unmeasured"))
}

/// The same three claims the `domains` half asserts, in the order they can fail.
fn assert_justifies(key: &str, measured: f64, python: f64, leaf: &str, expected_band: f64) {
    // Captured unless `-- --nocapture`, so it costs nothing and the instrument can be read
    // back without editing it — the same reason `tiers::compare_at_tier` returns its measurement.
    eprintln!("MEASURED {key} = {measured:.6e} (python {python:.3e}) leaf={leaf}");
    assert!(
        measured > 0.0,
        "{key}: the ±1-ULP probe moved nothing — it is shimming something the run does not \
         reach (worst leaf {leaf:?})"
    );
    assert!(
        measured > python / 10.0 && measured < python * 10.0,
        "{key}: measured ±1-ULP sensitivity {measured:.3e} is more than an order of magnitude \
         from the Python instrument's {python:.3e} — a finding about the probe or the port, \
         not a number to write down (worst leaf {leaf:?})"
    );
    let band = band_of(key);
    assert_eq!(
        band, expected_band,
        "{key}: band {band:.3e} is not the {expected_band:.3e} its group shares"
    );
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

/// ⚠⚠ The control that makes the numbers mean something: with the nudge off, each probe run
/// must emit its golden emitter's **exact bytes**. The station's solar seam rebuilds the
/// resolver and its radiator seam swaps a flow out of the registry, so both have somewhere to
/// drift to.
#[test]
fn the_probe_harness_reproduces_its_golden_with_the_nudge_off() {
    for seam in [EnergySeam::Solar, EnergySeam::Radiator] {
        assert_eq!(
            ulp_probe::station_energy_snapshot(seam, Nudge::Off),
            goldens::station(),
            "the {seam:?} probe harness has drifted from goldens::station"
        );
    }
    assert_eq!(
        ulp_probe::greenhouse_snapshot(Nudge::Off),
        goldens::greenhouse(),
        "the greenhouse probe harness has drifted from goldens::greenhouse"
    );
}

/// The coupled Power→Thermal band, measured on the cheap 7-day run that bounds the 15-year one.
#[test]
fn the_station_energy_bands_sit_above_the_measured_sensitivity() {
    let (measured, leaf) = ulp_probe::station_energy_sensitivity();
    for key in STATION_ENERGY_KEYS {
        assert_justifies(key, measured, PYTHON_STATION_ENERGY, &leaf, 1e-12);
    }
}

/// The four biosphere-coupled station goldens, measured on the cheap 7-day greenhouse.
#[test]
fn the_station_biosphere_bands_sit_above_the_measured_sensitivity() {
    let (measured, leaf) = ulp_probe::greenhouse_sensitivity();
    for key in STATION_BIOSPHERE_KEYS {
        assert_justifies(key, measured, PYTHON_GREENHOUSE, &leaf, 1e-11);
    }
}

/// ⚠ Perturbing both of the energy run's transcendentals at once is the mistake this guards
/// against: the solar `sin` and the radiator `t⁴` push the node in opposite directions, so a
/// combined nudge can cancel and read *lower* than either alone. The measurement takes them
/// one at a time and this pins that it must — each seam alone has to move something.
#[test]
fn each_energy_seam_moves_the_run_on_its_own() {
    let base = ulp_probe::station_energy_snapshot(EnergySeam::Solar, Nudge::Off);
    for seam in [EnergySeam::Solar, EnergySeam::Radiator] {
        let mut moved = false;
        for nudge in Nudge::BOTH {
            moved |= ulp_probe::station_energy_snapshot(seam, nudge) != base;
        }
        assert!(
            moved,
            "the {seam:?} seam changes nothing in either direction — it is not on the run's path"
        );
    }
}
