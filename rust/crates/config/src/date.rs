//! ISO calendar-date parsing — `YYYY-MM-DD` to day-of-year (reference flip, slice C9).
//!
//! The weather fixture dates its rows `"2006-10-01"`; the forcing conversions want the
//! day-of-year. Python did this with `date.fromisoformat(...).timetuple().tm_yday` and
//! precomputed the answer into the generated table, on the (correct) ground that a
//! calendar computation is **not** a libm op and so is not cross-port-sensitive. With
//! the generator retired, the reference does it itself.
//!
//! ⚠⚠ **This module is the one piece of C9 that no golden can check, and the reason is
//! the data.** The winter-wheat fixture runs 2006-10-01 → 2007-08-01: neither year is a
//! leap year and the span contains no 29 February. A wrong leap rule — `year % 4 == 0`
//! alone, the common one — therefore produces **byte-identical output on every row of
//! the fixture and every golden in the tree**. That is exactly *"a control with no test
//! to redden IS the finding"* (`docs/log/authoring-manifest-reanchored.md`), so the
//! tests below are **hand-computed against the calendar** rather than against the
//! fixture, and they carry the century cases (1900, 2000) the fixture cannot reach.

use crate::errors::ConfigError;

/// Days in each month of a non-leap year.
const MONTH_LENGTHS: [i64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// The proleptic Gregorian leap rule: divisible by 4, except centuries, except
/// multiples of 400.
pub fn is_leap_year(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// Days in `month` (1-based) of `year`.
fn days_in_month(year: i64, month: i64) -> i64 {
    let base = MONTH_LENGTHS[(month - 1) as usize];
    if month == 2 && is_leap_year(year) {
        base + 1
    } else {
        base
    }
}

/// Parse a strict `YYYY-MM-DD` date and return its 1-based day of the year.
///
/// Strict means exactly ten characters in exactly that shape: no `T` suffix, no
/// single-digit month or day, no other separator. Python's `date.fromisoformat` accepted
/// a couple of extra spellings; the fixture uses only this one, and a reader that
/// quietly takes more is a reader whose accepted language nobody has written down.
pub fn iso_day_of_year(text: &str) -> Result<i64, ConfigError> {
    let bytes = text.as_bytes();
    let malformed = || ConfigError::new(format!("'{text}' is not a YYYY-MM-DD date"));
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return Err(malformed());
    }
    let field = |start: usize, end: usize| -> Result<i64, ConfigError> {
        let slice = &text[start..end];
        if !slice.bytes().all(|b| b.is_ascii_digit()) {
            return Err(malformed());
        }
        slice.parse::<i64>().map_err(|_| malformed())
    };
    let year = field(0, 4)?;
    let month = field(5, 7)?;
    let day = field(8, 10)?;
    if !(1..=12).contains(&month) {
        return Err(ConfigError::new(format!("'{text}': month out of range")));
    }
    if day < 1 || day > days_in_month(year, month) {
        return Err(ConfigError::new(format!("'{text}': day out of range")));
    }
    let mut day_of_year = day;
    for earlier in 1..month {
        day_of_year += days_in_month(year, earlier);
    }
    Ok(day_of_year)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_ends_of_the_committed_season() {
        // Hand-counted: Jan..Sep 2006 = 31+28+31+30+31+30+31+31+30 = 273, so 1 Oct is
        // 274; Jan..Jul 2007 = 212, so 1 Aug is 213.
        assert_eq!(iso_day_of_year("2006-10-01").unwrap(), 274);
        assert_eq!(iso_day_of_year("2007-08-01").unwrap(), 213);
    }

    #[test]
    fn the_leap_rule_at_the_two_century_cases_the_fixture_cannot_reach() {
        // 1900 is divisible by 4 and by 100 but not by 400 -> NOT a leap year. 2000 is
        // divisible by 400 -> leap. These are the two the naive `% 4` rule gets wrong,
        // and no row of the fixture (2006-10 .. 2007-08) can tell the rules apart.
        assert!(!is_leap_year(1900));
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2023));
        assert_eq!(iso_day_of_year("1900-03-01").unwrap(), 60);
        assert_eq!(iso_day_of_year("2000-03-01").unwrap(), 61);
        assert_eq!(iso_day_of_year("1900-12-31").unwrap(), 365);
        assert_eq!(iso_day_of_year("2000-12-31").unwrap(), 366);
    }

    #[test]
    fn the_february_boundary_in_both_kinds_of_year() {
        assert_eq!(iso_day_of_year("2024-02-28").unwrap(), 59);
        assert_eq!(iso_day_of_year("2024-02-29").unwrap(), 60);
        assert_eq!(iso_day_of_year("2024-03-01").unwrap(), 61);
        assert_eq!(iso_day_of_year("2023-02-28").unwrap(), 59);
        assert_eq!(iso_day_of_year("2023-03-01").unwrap(), 60);
        assert_eq!(iso_day_of_year("2024-12-31").unwrap(), 366);
        assert_eq!(iso_day_of_year("2023-12-31").unwrap(), 365);
        assert_eq!(iso_day_of_year("2023-01-01").unwrap(), 1);
    }

    #[test]
    fn every_month_start_of_a_common_year() {
        // The running-sum loop, checked end to end rather than at two points.
        let expected = [1, 32, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335];
        for (index, day) in expected.iter().enumerate() {
            let month = index + 1;
            let text = format!("2023-{month:02}-01");
            assert_eq!(iso_day_of_year(&text).unwrap(), *day, "{text}");
        }
    }

    #[test]
    fn out_of_range_and_malformed_dates_are_errors() {
        for bad in [
            "2023-02-29", // not a leap year
            "1900-02-29", // the century trap, from the other side
            "2023-13-01",
            "2023-00-10",
            "2023-01-32",
            "2023-04-31",
            "2023-1-01",  // not zero-padded
            "20230101",   // no separators
            "2023/01/01", // wrong separator
            "2023-01-01T00:00:00",
            "abcd-01-01",
            "2023-ab-01",
            "",
        ] {
            assert!(
                iso_day_of_year(bad).is_err(),
                "'{bad}' should not parse as a date"
            );
        }
        assert_eq!(iso_day_of_year("2024-02-29").unwrap(), 60);
    }
}
