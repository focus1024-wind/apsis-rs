//! Tests for the leap second table.

use apsis_leap::{
    LEAP_SECONDS, TABLE_COVERS_FROM_MJD, TAI_MINUS_UTC_AFTER_LAST_ENTRY, mjd_from_ymd,
};

#[test]
fn table_is_ordered_and_monotonic() {
    let mut previous = f64::NEG_INFINITY;
    for entry in LEAP_SECONDS {
        assert!(
            (1..=12).contains(&entry.month),
            "month out of range: {entry:?}"
        );
        assert!((1..=31).contains(&entry.day), "day out of range: {entry:?}");
        let mjd = entry.effective_mjd();
        assert!(mjd > previous, "table is not ordered at {entry:?}");
        previous = mjd;
    }
}

#[test]
fn every_entry_increases_the_offset_by_one_second() {
    // The table starts at the inherited offset of ten seconds and every
    // later announcement inserted exactly one second.
    assert_eq!(LEAP_SECONDS[0].tai_minus_utc, 10.0);
    for pair in LEAP_SECONDS.windows(2) {
        let (earlier, later) = (pair[0], pair[1]);
        assert!(
            (later.tai_minus_utc - earlier.tai_minus_utc - 1.0).abs() < f64::EPSILON,
            "offset step is not one second between {earlier:?} and {later:?}"
        );
    }
}

#[test]
fn table_extremes_match_the_announced_values() {
    assert_eq!(LEAP_SECONDS[0].effective_mjd(), 41_317.0);
    let last = LEAP_SECONDS[LEAP_SECONDS.len() - 1];
    assert_eq!(last.year, 2017);
    assert_eq!(last.month, 1);
    assert_eq!(last.day, 1);
    assert_eq!(last.tai_minus_utc, 37.0);
    assert_eq!(last.effective_mjd(), 57_754.0);
}

#[test]
fn table_contains_every_announcement_since_1972() {
    // Twenty seven leap seconds were introduced between 1972 and 2017
    // inclusive of the initial 1972-01-01 entry.
    assert_eq!(LEAP_SECONDS.len(), 28);
}

#[test]
fn coverage_constants_match_the_table_boundaries() {
    assert_eq!(TABLE_COVERS_FROM_MJD, LEAP_SECONDS[0].effective_mjd());
    assert_eq!(TAI_MINUS_UTC_AFTER_LAST_ENTRY, 37.0);
}

#[test]
fn known_calendar_dates_map_to_known_modified_julian_dates() {
    // MJD 0 is the start of 1858-11-17.
    assert_eq!(mjd_from_ymd(1858, 11, 17), 0.0);
    // The calendar day of J2000.0 is MJD 51544.
    assert_eq!(mjd_from_ymd(2000, 1, 1), 51_544.0);
    // The Unix epoch is MJD 40587.
    assert_eq!(mjd_from_ymd(1970, 1, 1), 40_587.0);
    // The last leap second.
    assert_eq!(mjd_from_ymd(2017, 1, 1), 57_754.0);
    assert_eq!(mjd_from_ymd(2024, 1, 1), 60_310.0);
}

#[test]
fn calendar_conversion_handles_leap_days_and_month_ends() {
    // 2000 is a leap year, 1900 is not.
    assert_eq!(mjd_from_ymd(2000, 3, 1) - mjd_from_ymd(2000, 2, 1), 29.0);
    assert_eq!(mjd_from_ymd(1900, 3, 1) - mjd_from_ymd(1900, 2, 1), 28.0);
    // Consecutive month ends are one day apart.
    assert_eq!(mjd_from_ymd(2024, 3, 1) - mjd_from_ymd(2024, 2, 29), 1.0);
}
