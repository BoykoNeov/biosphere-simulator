//! The within-day light path — the hand mirror of `domains/biosphere/light_path.py`.
//!
//! PAR varies inside the day: the cited sinusoidal path ([E] Penning de Vries et al.,
//! *"The path of radiation intensity during the day is assumed to be sinusoidal"*) for the
//! sun, a top-hat for a grow lamp, both delivered to the forcing seam as the **analytic
//! mean over one step's window**. The day's photon dose is conserved exactly at any step
//! size, so this redistributes radiation rather than adding any.
//!
//! ⚠ **The port has no reference authority.** The reference for the form, the refusal of
//! the instantaneous-sampling alternative, and the measured consequences (including the
//! peak-LAI deviation) is the Python module and
//! `docs/plans/post-roadmap-gross-net-gas-exchange.md`. What is mirrored here is the
//! arithmetic, op for op: `cos` to `.cos()`, the same `max`/`min` clipping order, the same
//! division by the **full** window length rather than by the daylight overlap.
//!
//! Errors mirror Python's `ValueError`s: they are build bugs (a window crossing midnight
//! cannot happen while the step divides the day), so they surface rather than answering
//! from the wrong day.

use simcore::error::SimError;

/// Seconds in one day — the integration window the daily carbon budget passes.
pub const SECONDS_PER_DAY: f64 = 86400.0;

/// `(sunrise, sunset)` as fractions of the day, centred on solar noon (½).
fn daylight_span(daylength_s: f64) -> Result<(f64, f64), SimError> {
    if !(0.0..=SECONDS_PER_DAY).contains(&daylength_s) {
        return Err(SimError::Validation(format!(
            "daylength_s must be within [0, {SECONDS_PER_DAY}] s, got {daylength_s:?}"
        )));
    }
    let half = daylength_s / SECONDS_PER_DAY / 2.0;
    Ok((0.5 - half, 0.5 + half))
}

/// The window must lie inside one day — the schedule's own precondition.
fn check_window(t0: f64, dt: f64) -> Result<(), SimError> {
    if dt <= 0.0 {
        return Err(SimError::Validation(format!("dt must be > 0 days, got {dt:?}")));
    }
    if t0 < 0.0 || t0 + dt > 1.0 + 1e-12 {
        return Err(SimError::Validation(format!(
            "light-path window [{t0:?}, {:?}) must lie within one day; the step must \
             divide the day",
            t0 + dt
        )));
    }
    Ok(())
}

/// Mean PAR over `[t0, t0+dt)` of the sinusoidal day (µmol m⁻² s⁻¹).
///
/// `peak = (π/2)·daytime_mean_par` is the value that conserves the day's dose (a half-sine
/// integrates to `(2/π)·peak·D`). Dividing the window integral by the **full** window
/// length is what makes a step that is half night carry half the light.
pub fn half_sine_window_mean(
    t0: f64,
    dt: f64,
    daytime_mean_par: f64,
    daylength_s: f64,
) -> Result<f64, SimError> {
    check_window(t0, dt)?;
    if daytime_mean_par < 0.0 {
        return Err(SimError::Validation(format!(
            "daytime_mean_par must be >= 0, got {daytime_mean_par:?}"
        )));
    }
    let (sunrise, sunset) = daylight_span(daylength_s)?;
    let lo = t0.max(sunrise);
    let hi = (t0 + dt).min(sunset);
    if hi <= lo || daytime_mean_par == 0.0 {
        return Ok(0.0);
    }
    let span = sunset - sunrise;
    let peak = (std::f64::consts::PI / 2.0) * daytime_mean_par;
    let integral = (span / std::f64::consts::PI)
        * ((std::f64::consts::PI * (lo - sunrise) / span).cos()
            - (std::f64::consts::PI * (hi - sunrise) / span).cos());
    Ok(peak * integral / dt)
}

/// Mean PAR over `[t0, t0+dt)` of a lamp's on/off day (µmol m⁻² s⁻¹).
///
/// The photoperiod is the **shape** of the day rather than a multiplier on its total, so
/// the lamp's dark hours are hours the crop respires through. Dose conserved exactly.
pub fn top_hat_window_mean(
    t0: f64,
    dt: f64,
    on_par: f64,
    photoperiod_s: f64,
) -> Result<f64, SimError> {
    check_window(t0, dt)?;
    if on_par < 0.0 {
        return Err(SimError::Validation(format!("on_par must be >= 0, got {on_par:?}")));
    }
    let (sunrise, sunset) = daylight_span(photoperiod_s)?;
    let lo = t0.max(sunrise);
    let hi = (t0 + dt).min(sunset);
    if hi <= lo {
        return Ok(0.0);
    }
    Ok(on_par * (hi - lo) / dt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_day_dose_is_conserved_at_every_step_size() {
        // The port's own copy of the property the Python side pins: the window means are
        // an exact partition of one integral, so summing them reproduces the flat
        // daytime mean's dose whatever the step.
        for steps_per_day in [1usize, 2, 4, 8, 32, 96] {
            for daylength_h in [0.0f64, 4.0, 9.5, 12.0, 16.5, 24.0] {
                let dt = 1.0 / steps_per_day as f64;
                let dose: f64 = (0..steps_per_day)
                    .map(|k| {
                        half_sine_window_mean(k as f64 * dt, dt, 400.0, daylength_h * 3600.0)
                            .unwrap()
                            * dt
                    })
                    .sum();
                let expected = 400.0 * daylength_h * 3600.0 / SECONDS_PER_DAY;
                assert!(
                    (dose - expected).abs() < 1e-9,
                    "{steps_per_day} steps, {daylength_h} h: {dose} vs {expected}"
                );
            }
        }
    }

    #[test]
    fn a_window_in_the_dark_is_exactly_zero() {
        let daylength_s = 12.0 * 3600.0;
        assert_eq!(
            half_sine_window_mean(0.0, 0.125, 400.0, daylength_s).unwrap(),
            0.0
        );
        assert_eq!(
            top_hat_window_mean(0.875, 0.125, 400.0, daylength_s).unwrap(),
            0.0
        );
        assert!(half_sine_window_mean(0.375, 0.125, 400.0, daylength_s).unwrap() > 0.0);
    }

    #[test]
    fn a_window_crossing_midnight_is_an_error_not_a_wrong_day() {
        assert!(half_sine_window_mean(0.9, 0.25, 400.0, 43200.0).is_err());
        assert!(half_sine_window_mean(0.0, 0.0, 400.0, 43200.0).is_err());
    }
}
