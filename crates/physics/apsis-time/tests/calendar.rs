//! Tests for the calendar conversions.

use apsis_time::{Calendar, days_in_month, is_leap_year, mjd_from_ymd, ymd_from_mjd};

#[test]
fn modified_julian_date_zero_is_the_documented_day() {
    assert_eq!(mjd_from_ymd(1858, 11, 17), 0.0);
    assert_eq!(ymd_from_mjd(0.0), (1858, 11, 17));
}

#[test]
fn known_dates_map_to_known_modified_julian_dates() {
    assert_eq!(mjd_from_ymd(1970, 1, 1), 40_587.0);
    assert_eq!(mjd_from_ymd(2000, 1, 1), 51_544.0);
    assert_eq!(mjd_from_ymd(2017, 1, 1), 57_754.0);
    assert_eq!(mjd_from_ymd(2024, 1, 1), 60_310.0);
}

#[test]
fn calendar_conversion_round_trips_over_a_long_span() {
    // Walk day by day over a span that crosses every leap year rule.
    let mut mjd = mjd_from_ymd(1899, 12, 20);
    let end = mjd_from_ymd(2101, 1, 10);
    while mjd < end {
        let (year, month, day) = ymd_from_mjd(mjd);
        assert_eq!(
            mjd_from_ymd(year, month, day),
            mjd,
            "round trip failed at MJD {mjd}"
        );
        mjd += 1.0;
    }
}

#[test]
fn fractional_modified_julian_dates_map_to_the_containing_day() {
    let day = mjd_from_ymd(2024, 6, 15);
    assert_eq!(ymd_from_mjd(day + 0.25), (2024, 6, 15));
    assert_eq!(ymd_from_mjd(day + 0.75), (2024, 6, 15));
    // One day later, but still the same instant of the following day.
    assert_eq!(ymd_from_mjd(day + 1.25), (2024, 6, 16));
}

#[test]
fn leap_years_follow_the_gregorian_rule() {
    assert!(is_leap_year(2000));
    assert!(is_leap_year(2024));
    assert!(!is_leap_year(1900));
    assert!(!is_leap_year(2100));
    assert!(!is_leap_year(2023));
    // Year zero is a leap year in the proleptic calendar.
    assert!(is_leap_year(0));
}

#[test]
fn february_length_follows_the_leap_year_rule() {
    assert_eq!(days_in_month(2024, 2), 29);
    assert_eq!(days_in_month(2023, 2), 28);
    assert_eq!(days_in_month(1900, 2), 28);
    assert_eq!(days_in_month(2000, 2), 29);
}

#[test]
fn days_in_month_rejects_invalid_months() {
    assert_eq!(days_in_month(2024, 0), 0);
    assert_eq!(days_in_month(2024, 13), 0);
}

#[test]
fn every_month_has_the_expected_length() {
    let expected = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for (index, days) in expected.iter().enumerate() {
        let month = index as u32 + 1;
        assert_eq!(days_in_month(2023, month), *days, "month {month}");
    }
}

#[test]
fn validation_accepts_real_dates() {
    assert!(Calendar::from_ymd(2024, 2, 29).is_valid());
    assert!(Calendar::new(2016, 12, 31, 23, 59, 59.5).is_valid());
}

#[test]
fn validation_rejects_impossible_dates_and_the_leap_second_label() {
    assert!(!Calendar::from_ymd(2023, 2, 29).is_valid());
    assert!(!Calendar::from_ymd(2024, 13, 1).is_valid());
    assert!(!Calendar::from_ymd(2024, 1, 0).is_valid());
    assert!(!Calendar::from_ymd(2024, 1, 32).is_valid());
    assert!(!Calendar::new(2024, 1, 1, 24, 0, 0.0).is_valid());
    assert!(!Calendar::new(2024, 1, 1, 0, 60, 0.0).is_valid());
    // Second 60 is the leap second label, which is not a representable
    // instant on the continuous Modified Julian Date scale.
    assert!(!Calendar::new(2016, 12, 31, 23, 59, 60.0).is_valid());
    assert!(!Calendar::new(2024, 1, 1, 0, 0, 61.0).is_valid());
}

#[test]
fn seconds_of_day_accumulates_the_time_fields() {
    let calendar = Calendar::new(2024, 1, 1, 1, 2, 3.5);
    assert!((calendar.seconds_of_day() - 3_723.5).abs() < 1e-12);
    assert_eq!(Calendar::from_ymd(2024, 1, 1).seconds_of_day(), 0.0);
}

#[test]
fn display_uses_an_iso_style_layout() {
    let calendar = Calendar::new(2024, 6, 15, 12, 30, 45.25);
    assert_eq!(calendar.to_string(), "2024-06-15T12:30:45.250");
}
