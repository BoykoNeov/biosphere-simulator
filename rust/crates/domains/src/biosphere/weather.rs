//! Weather → flow-driver conversions — the Rust port of `domains.biosphere.weather`
//! (+ the SVP primitive Python locates in `transpiration`) (Phase-7 P7.4).
//!
//! The **heaviest libm-audit surface** of the port (plan Step 4): `daylength_seconds`
//! (sin/tan/acos), `saturation_vapor_pressure`/`vapor_pressure_deficit` (exp),
//! `incident_par`, `net_radiation`. The raw NASAPower facts arrive from the committed
//! weather fixture; these conversions run **in Rust**, so the transcendentals are
//! exercised cross-port. Every op mirrors the Python source character-for-character
//! (`math.radians`→`.to_radians()`, `math.sin`→`.sin()`, …).
//!
//! # ⚠ Slice C9 — this module stopped reading a Python-generated copy
//!
//! Until C9 the facts arrived as `weather_facts.txt`, a hex-float table that
//! `tests/crossport/gen_biosphere_weather.py` lowered out of the JSON fixture. That was
//! the last generator in the reference flip with **no named successor**: the reference's
//! own forcing data reached it through a Python script. It now reads the fixture
//! directly, via the closed-subset JSON reader and the ISO-date calendar computation in
//! the `config` crate (`config::json`, `config::date`).
//!
//! **Nothing moved.** All 916 values (latitude + 3 × 305 observations) were measured
//! before the change to parse to the same bits through `f64::from_str` as they did
//! through the generated hex-float table — both conversions are correctly rounded — so
//! C9 re-anchors *who reads the data*, not what the data is.

use std::f64::consts::PI;

use config::{iso_day_of_year, parse_json, ConfigError};

/// The committed raw NASAPower weather fixture — the reference's forcing input.
///
/// ⚠ **The reach-out is gone as of Stage-3 slice S1** (2026-08-18). C9 embedded this file
/// from `tests/oracle/`, five directories up in the tree scheduled for deletion — the same
/// ugliness the param `include_str!`s carried, recorded rather than overlooked, with the
/// relocation slice named as its successor. S1 is that slice: the file now sits in this
/// crate's own `data/`, and the *surviving* oracle carve-out reads it from here instead.
/// The discriminator for moving this one and not `tests/oracle/`'s other two weather series
/// is that the reference compiles **this** one in. The fixture is *raw observational facts*,
/// license-clean per `docs/reuse-and-licenses.md` — it is not PCSE output.
///
/// Public so the inventory dump can hash **what the reference compiled in** against what
/// the checker finds on disk — see that program's `weather_sha256` note.
pub const WEATHER_FIXTURE: &str = include_str!("../../data/winter_wheat_weather.json");

const SECONDS_PER_DAY: f64 = 86400.0;
/// PAR is ~50% of global shortwave by energy (McCree 1972; FAO uses 0.5).
const PAR_ENERGY_FRACTION: f64 = 0.5;
/// PAR photon flux per unit PAR energy: ~4.57 µmol photons per J (McCree 1972).
pub const PAR_UMOL_PER_J: f64 = 4.57;
/// FAO-56 reference-crop albedo (net shortwave Rns = (1 − α)·Rs).
pub const ALBEDO: f64 = 0.23;

// Saturation-vapour-pressure constants (Tetens / FAO-56); Python locates these in
// `transpiration`. e_s(T) = SVP_A · exp(SVP_B · T / (T + SVP_C))  [Pa], T in °C.
/// Tetens/FAO-56 SVP pre-factor (Pa).
pub const SVP_A: f64 = 610.8;
/// Tetens/FAO-56 SVP exponent numerator coefficient (dimensionless).
pub const SVP_B: f64 = 17.27;
/// Tetens/FAO-56 SVP exponent denominator offset (°C).
pub const SVP_C: f64 = 237.3;

/// One raw daily weather fact (day-of-year + the NASAPower observations).
#[derive(Debug, Clone, Copy)]
pub struct WeatherRow {
    pub day_of_year: i64,
    pub temp_c: f64,
    pub irrad_j_m2_day: f64,
    pub vap_hpa: f64,
    /// The day's minimum and maximum air temperature (°C), added 2026-10-17 for the air's
    /// within-day cycle (`docs/plans/post-roadmap-open-field-night.md` §7a). `temp_c` is POWER's
    /// 24-h mean, not their midpoint.
    pub tmin_c: f64,
    pub tmax_c: f64,
}

/// Parse the embedded weather fixture into `(latitude, rows)`.
///
/// Panics if the committed fixture is unreadable — it is compiled into the binary, so a
/// failure here is a corrupt reference, not a runtime condition a caller could handle.
/// [`read_weather_facts`] is the fallible form, and is what the tests exercise.
pub fn weather_facts() -> (f64, Vec<WeatherRow>) {
    read_weather_facts(WEATHER_FIXTURE).expect("the committed weather fixture parses")
}

/// The fallible reader behind [`weather_facts`]: raw NASAPower facts out of the JSON.
///
/// Row order is the fixture's array order — the season is driven day by day, so this is
/// load-bearing and is asserted in the tests rather than assumed from the parser.
pub fn read_weather_facts(text: &str) -> Result<(f64, Vec<WeatherRow>), ConfigError> {
    const DOC: &str = "the weather fixture";
    let document = parse_json(text)?;
    let latitude = document
        .get("provenance", DOC)?
        .get("latitude", "provenance")?
        .as_f64("latitude")?;
    let mut rows: Vec<WeatherRow> = Vec::new();
    for (index, row) in document
        .get("weather", DOC)?
        .as_array("weather")?
        .iter()
        .enumerate()
    {
        let at = format!("weather[{index}]");
        rows.push(WeatherRow {
            // A calendar computation, not a libm op — see `config::date`, which carries
            // the leap-year cases this fixture's own dates cannot reach.
            day_of_year: iso_day_of_year(row.get("day", &at)?.as_str("day")?)?,
            temp_c: row.get("TEMP", &at)?.as_f64("TEMP")?,
            irrad_j_m2_day: row.get("IRRAD", &at)?.as_f64("IRRAD")?,
            vap_hpa: row.get("VAP", &at)?.as_f64("VAP")?,
            tmin_c: row.get("TMIN", &at)?.as_f64("TMIN")?,
            tmax_c: row.get("TMAX", &at)?.as_f64("TMAX")?,
        });
        let last = &rows[index];
        if !(last.tmin_c <= last.temp_c && last.temp_c <= last.tmax_c) {
            return Err(ConfigError::new(format!(
                "{at}: the day's mean {} °C lies outside its range [{}, {}] °C",
                last.temp_c, last.tmin_c, last.tmax_c
            )));
        }
    }
    if rows.is_empty() {
        return Err(ConfigError::new("the weather fixture has no rows"));
    }
    Ok((latitude, rows))
}

/// Astronomical daylight duration (s) from latitude + day-of-year (FAO-56). Mirrors
/// `weather.daylength_seconds` op-for-op — the sin/tan/acos libm surface.
pub fn daylength_seconds(latitude_deg: f64, day_of_year: i64) -> f64 {
    let phi = latitude_deg.to_radians();
    let decl = 0.409 * (2.0 * PI * (day_of_year as f64) / 365.0 - 1.39).sin();
    let arg = -phi.tan() * decl.tan();
    // Clamp for polar latitudes — Python `max(-1.0, min(1.0, arg))`; `.clamp` is
    // bit-identical for the finite args here (it selects arg / 1.0 / -1.0, no arithmetic).
    let arg = arg.clamp(-1.0, 1.0);
    let sunset_hour_angle = arg.acos();
    let daylight_hours = 24.0 / PI * sunset_hour_angle;
    daylight_hours * 3600.0
}

/// Daytime-mean incident PAR photon flux (µmol m⁻² s⁻¹) from daily IRRAD.
pub fn incident_par(irrad_j_m2_day: f64, daylength_s: f64) -> f64 {
    let mean_par_irradiance = PAR_ENERGY_FRACTION * irrad_j_m2_day / daylength_s;
    mean_par_irradiance * PAR_UMOL_PER_J
}

/// Daily-mean net radiation (W m⁻²) ≈ net shortwave (1 − α)·Rs (FAO-56).
pub fn net_radiation(irrad_j_m2_day: f64) -> f64 {
    let shortwave = irrad_j_m2_day / SECONDS_PER_DAY;
    net_shortwave(shortwave)
}

/// Net shortwave (W m⁻²) from incident shortwave (W m⁻²): `(1 − α)·Rs`, FAO-56's reference-crop
/// albedo. Shared by the outdoor form ([`net_radiation`]) and the lamp-lit chamber's (the lamp's
/// radiant PAR, `station::lighting`). ⚠ For a lamp, α = 0.23 is a class citation: it is
/// broadband sunlight on grass, and leaves reflect less of an all-PAR lamp's light — the user's
/// choice of 2026-10-05 (`docs/plans/post-roadmap-room-temperature.md` §21).
pub fn net_shortwave(shortwave_w_m2: f64) -> f64 {
    (1.0 - ALBEDO) * shortwave_w_m2
}

/// Saturation vapour pressure e_s = A·exp(B·T/(T+C)) (Pa; Tetens/FAO-56). Python locates
/// this in `transpiration`; the port keeps it here (its first consumer, the VPD build).
pub fn saturation_vapor_pressure(temp_c: f64) -> f64 {
    SVP_A * (SVP_B * temp_c / (temp_c + SVP_C)).exp()
}

/// Vapour-pressure deficit (Pa) from air temperature + actual vapour pressure.
pub fn vapor_pressure_deficit(temp_c: f64, vap_hpa: f64) -> f64 {
    let e_a = vap_hpa * 100.0; // hPa -> Pa
    (saturation_vapor_pressure(temp_c) - e_a).max(0.0)
}

/// The per-day forcing tables the season resolver reads, tiled over `years`. Each Vec has
/// length `rows.len()·years` (the Python `weather = _weather()*years` tiling), so a
/// clamping `_table`-style index reproduces the reference exactly.
#[derive(Debug, Clone)]
pub struct ForcingTables {
    pub temp: Vec<f64>,
    /// The day's minimum / maximum air temperature (°C) and its vapour pressure (Pa) — the air's
    /// within-day cycle reads them (`air_temperature_window_mean`); nothing else does.
    pub tmin: Vec<f64>,
    pub tmax: Vec<f64>,
    pub vapour_pa: Vec<f64>,
    pub par: Vec<f64>,
    pub daylength: Vec<f64>,
    pub net_radiation: Vec<f64>,
    pub vpd: Vec<f64>,
}

/// Build the tiled forcing tables from the raw facts (the `weather_resolver` precompute).
pub fn season_forcing(latitude: f64, rows: &[WeatherRow], years: usize) -> ForcingTables {
    let mut temp = Vec::with_capacity(rows.len() * years);
    let mut par = Vec::with_capacity(rows.len() * years);
    let mut daylength = Vec::with_capacity(rows.len() * years);
    let mut rn = Vec::with_capacity(rows.len() * years);
    let mut vpd = Vec::with_capacity(rows.len() * years);
    let mut tmin = Vec::with_capacity(rows.len() * years);
    let mut tmax = Vec::with_capacity(rows.len() * years);
    let mut vapour_pa = Vec::with_capacity(rows.len() * years);
    // Convert once per row, then tile — identical to Python's convert-of-tiled-rows since
    // each conversion is a pure function of the row (and the shared latitude).
    let mut base_temp = Vec::with_capacity(rows.len());
    let mut base_par = Vec::with_capacity(rows.len());
    let mut base_dl = Vec::with_capacity(rows.len());
    let mut base_rn = Vec::with_capacity(rows.len());
    let mut base_vpd = Vec::with_capacity(rows.len());
    for row in rows {
        let dl = daylength_seconds(latitude, row.day_of_year);
        base_temp.push(row.temp_c);
        base_dl.push(dl);
        base_par.push(incident_par(row.irrad_j_m2_day, dl));
        base_rn.push(net_radiation(row.irrad_j_m2_day));
        base_vpd.push(vapor_pressure_deficit(row.temp_c, row.vap_hpa));
    }
    for _ in 0..years {
        temp.extend_from_slice(&base_temp);
        par.extend_from_slice(&base_par);
        daylength.extend_from_slice(&base_dl);
        rn.extend_from_slice(&base_rn);
        vpd.extend_from_slice(&base_vpd);
        for row in rows {
            tmin.push(row.tmin_c);
            tmax.push(row.tmax_c);
            vapour_pa.push(row.vap_hpa * 100.0);
        }
    }
    ForcingTables {
        temp,
        par,
        daylength,
        net_radiation: rn,
        vpd,
        tmin,
        tmax,
        vapour_pa,
    }
}

/// The shortest day the within-day air curve takes (s): its maximum, 1.5 h after solar noon, must
/// fall before sunset. Unreachable at the fixture's 52° N (its shortest day is ~7.7 h).
pub const AIR_CURVE_MIN_DAYLENGTH_S: f64 = 3.0 * 3600.0;

/// The hours after sunrise and after solar noon at which the air reaches its minimum and maximum —
/// Teh (2006) §2.7's assumption ("1.5 hours after sunrise and solar noon, respectively").
const AIR_CURVE_LAG_H: f64 = 1.5;

/// The air temperature's integral (°C·h) from midnight to hour `t` ∈ [0, 24] of a day with minimum
/// `tmin`, maximum `tmax` and day length `daylength_h` — Teh (2006), *Introduction to Mathematical
/// Modeling of Crop Growth*, §2.7, Eqs. 2.41–2.47 as printed:
///
/// * from `sunrise + 1.5 h` to sunset, a half sine (Eq. 2.41): `tmin + (tmax − tmin)·sin(π·x / D)`,
///   `x = t − sunrise − 1.5`, so `tmin` at `sunrise + 1.5` and `tmax` at solar noon + 1.5;
/// * after sunset, a straight line to the minimum at the next `sunrise + 1.5` (Eq. 2.43), its slope
///   from the same day's sunrise and minimum (Eq. 2.45, Teh's approximation for "the next day");
/// * before `sunrise + 1.5`, the same line continued (Eq. 2.46), so the day closes on itself at
///   midnight.
///
/// Sunrise and sunset are `12 ∓ D/2`, solar noon at noon — the span PAR is lit over.
fn air_temperature_integral(t: f64, tmin: f64, tmax: f64, daylength_h: f64) -> f64 {
    let sunrise = 12.0 - 0.5 * daylength_h;
    let sunset = 12.0 + 0.5 * daylength_h;
    let coldest = sunrise + AIR_CURVE_LAG_H;
    let amplitude = tmax - tmin;
    let at_sunset = tmin + amplitude * (PI * (sunset - coldest) / daylength_h).sin();
    let slope = (tmin - at_sunset) / (coldest + 24.0 - sunset);
    // Section I, [0, coldest): T = at_sunset + slope·(24 + t − sunset).
    let night_before = |u: f64| at_sunset * u + slope * ((24.0 - sunset) * u + 0.5 * u * u);
    if t <= coldest {
        return night_before(t);
    }
    // Section II, [coldest, sunset]: the half sine.
    let day = |u: f64| {
        let x = u - coldest;
        tmin * x + amplitude * daylength_h / PI * (1.0 - (PI * x / daylength_h).cos())
    };
    if t <= sunset {
        return night_before(coldest) + day(t);
    }
    // Section III, (sunset, 24]: T = at_sunset + slope·(t − sunset).
    let x = t - sunset;
    night_before(coldest) + day(sunset) + at_sunset * x + 0.5 * slope * x * x
}

/// The mean air temperature (°C) over the window `[t0, t0 + dt)` of one day (both in days, `t0` the
/// fraction of the day elapsed) on Teh's curve ([`air_temperature_integral`]), exact from its
/// integral. `None` for a window that leaves the day or a day shorter than
/// [`AIR_CURVE_MIN_DAYLENGTH_S`].
pub fn air_temperature_window_mean(
    t0: f64,
    dt: f64,
    tmin: f64,
    tmax: f64,
    daylength_s: f64,
) -> Option<f64> {
    // A NaN day length is refused as a short one.
    let short_day = daylength_s.is_nan() || daylength_s < AIR_CURVE_MIN_DAYLENGTH_S;
    if short_day || dt <= 0.0 || t0 < 0.0 || t0 + dt > 1.0 + 1e-12 {
        return None;
    }
    let dl_h = daylength_s / 3600.0;
    let (a, b) = (24.0 * t0, (24.0 * (t0 + dt)).min(24.0));
    Some(
        (air_temperature_integral(b, tmin, tmax, dl_h)
            - air_temperature_integral(a, tmin, tmax, dl_h))
            / (b - a),
    )
}

/// The 24-h mean (°C) of Teh's curve for a day — **not** the day's observed mean: on the fixture's
/// days it runs 0.44 °C below it on average (plan §7b), which is why the shifted variant exists.
pub fn air_temperature_day_mean(tmin: f64, tmax: f64, daylength_s: f64) -> Option<f64> {
    air_temperature_window_mean(0.0, 1.0, tmin, tmax, daylength_s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facts_parse_305_rows() {
        let (lat, rows) = weather_facts();
        assert_eq!(lat, 52.0);
        assert_eq!(rows.len(), 305);
        assert_eq!(rows[0].day_of_year, 274); // 2006-10-01
        assert_eq!(rows[304].day_of_year, 213); // 2007-08-01
    }

    #[test]
    fn the_rows_arrive_as_consecutive_calendar_days_in_source_order() {
        // The fixture is a time series and the season is driven row by row, so ORDER is
        // as load-bearing as the values. A reader that shuffled the array would leave
        // every number correct and the run wrong. 2006 is not a leap year, so the wrap
        // is 365 -> 1.
        let (_, rows) = weather_facts();
        for pair in rows.windows(2) {
            let (previous, next) = (pair[0].day_of_year, pair[1].day_of_year);
            let expected = if previous == 365 { 1 } else { previous + 1 };
            assert_eq!(next, expected, "rows are out of order after doy {previous}");
        }
    }

    #[test]
    fn a_malformed_fixture_is_an_error_rather_than_a_partial_season() {
        // The failure modes that would otherwise reach the season as missing or
        // silently-defaulted forcing.
        for bad in [
            r#"{"provenance": {}, "weather": [{"day": "2006-10-01", "TEMP": 1.0, "IRRAD": 2.0, "VAP": 3.0}]}"#,
            r#"{"provenance": {"latitude": 52.0}, "weather": []}"#,
            r#"{"provenance": {"latitude": 52.0}}"#,
            r#"{"provenance": {"latitude": 52.0}, "weather": [{"day": "2006-10-01", "TEMP": 1.0, "IRRAD": 2.0}]}"#,
            r#"{"provenance": {"latitude": 52.0}, "weather": [{"day": "2006-13-01", "TEMP": 1.0, "IRRAD": 2.0, "VAP": 3.0}]}"#,
            r#"{"provenance": {"latitude": "52.0"}, "weather": [{"day": "2006-10-01", "TEMP": 1.0, "IRRAD": 2.0, "VAP": 3.0, "TMIN": 0.0, "TMAX": 2.0}]}"#,
            // no maximum; a mean outside its own range
            r#"{"provenance": {"latitude": 52.0}, "weather": [{"day": "2006-10-01", "TEMP": 1.0, "IRRAD": 2.0, "VAP": 3.0, "TMIN": 0.0}]}"#,
            r#"{"provenance": {"latitude": 52.0}, "weather": [{"day": "2006-10-01", "TEMP": 3.0, "IRRAD": 2.0, "VAP": 3.0, "TMIN": 0.0, "TMAX": 2.0}]}"#,
        ] {
            assert!(read_weather_facts(bad).is_err(), "should not parse: {bad}");
        }
        let good = r#"{"provenance": {"latitude": 52.0},
                       "weather": [{"IRRAD": 2.0, "TEMP": 1.0, "TMAX": 2.0, "TMIN": 0.5, "VAP": 3.0, "day": "2006-10-01"}]}"#;
        let (lat, rows) = read_weather_facts(good).expect("parses");
        assert_eq!(lat, 52.0);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].day_of_year, 274);
        assert_eq!(rows[0].temp_c, 1.0);
        assert_eq!(rows[0].irrad_j_m2_day, 2.0);
        assert_eq!(rows[0].vap_hpa, 3.0);
        assert_eq!((rows[0].tmin_c, rows[0].tmax_c), (0.5, 2.0));
    }

    #[test]
    fn equator_daylength_is_12h() {
        // ωs = π/2 at the equator ⇒ exactly 12 h = 43200 s for every day.
        assert!((daylength_seconds(0.0, 100) - 43200.0).abs() < 1e-6);
    }
}
