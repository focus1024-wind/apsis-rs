//! Tests for UTC and TAI conversions on the Modified Julian Date scale.

use apsis_leap::{
    MJD_AT_UNIX_EPOCH, calendar_mjd, tai_minus_utc_at_mjd, tai_mjd_from_utc_mjd,
    utc_mjd_from_tai_mjd,
};

const SECONDS_PER_DAY: f64 = 86_400.0;

#[test]
fn unix_epoch_constant_matches_the_calendar_conversion() {
    assert_eq!(MJD_AT_UNIX_EPOCH, calendar_mjd(1970, 1, 1));
}

#[test]
fn offset_is_rejected_before_the_table_begins() {
    let before = calendar_mjd(1971, 12, 31);
    assert_eq!(tai_minus_utc_at_mjd(before), None);
    assert_eq!(tai_mjd_from_utc_mjd(before), None);
    assert_eq!(tai_minus_utc_at_mjd(f64::NAN), None);
    assert_eq!(tai_minus_utc_at_mjd(f64::INFINITY), None);
}

#[test]
fn first_entry_supplies_the_initial_offset() {
    let start = calendar_mjd(1972, 1, 1);
    assert_eq!(tai_minus_utc_at_mjd(start), Some(10.0));
    // Just before midnight one day earlier the table does not apply.
    assert_eq!(tai_minus_utc_at_mjd(start - 1.0), None);
}

#[test]
fn offset_changes_exactly_on_the_effective_date() {
    // The 2017-01-01 leap second raised the offset from 36 to 37.
    let effective = calendar_mjd(2017, 1, 1);
    assert_eq!(tai_minus_utc_at_mjd(effective - 1.0), Some(36.0));
    assert_eq!(tai_minus_utc_at_mjd(effective), Some(37.0));
    assert_eq!(tai_minus_utc_at_mjd(effective + 0.5), Some(37.0));
}

#[test]
fn every_entry_is_reachable_at_its_effective_instant() {
    for entry in apsis_leap::LEAP_SECONDS {
        let at_entry = tai_minus_utc_at_mjd(entry.effective_mjd());
        assert_eq!(at_entry, Some(entry.tai_minus_utc), "entry {entry:?}");
    }
}

#[test]
fn offset_is_constant_between_entries() {
    // Mid 2016 sits between the 2015-07-01 and 2017-01-01 entries.
    let instant = calendar_mjd(2016, 6, 1);
    assert_eq!(tai_minus_utc_at_mjd(instant), Some(36.0));
    assert_eq!(tai_minus_utc_at_mjd(instant + 0.25), Some(36.0));
}

#[test]
fn future_instants_keep_the_last_offset() {
    let far_future = calendar_mjd(2100, 1, 1);
    assert_eq!(tai_minus_utc_at_mjd(far_future), Some(37.0));
}

#[test]
fn utc_to_tai_adds_the_offset() {
    let utc = calendar_mjd(2017, 6, 1) + 0.5;
    let tai = tai_mjd_from_utc_mjd(utc).expect("instant is inside the table");
    // An MJD near 58000 has a resolution of about 7e-12 days, so the sum
    // can only be compared to well below a microsecond.
    assert!((tai - utc - 37.0 / SECONDS_PER_DAY).abs() < 1e-11);
}

#[test]
fn utc_to_tai_round_trips() {
    for (year, month, day) in [
        (1972, 1, 1),
        (1980, 1, 6),
        (1999, 1, 1),
        (2015, 7, 1),
        (2017, 1, 1),
        (2024, 6, 15),
    ] {
        let utc = calendar_mjd(year, month, day) + 0.375;
        let tai = tai_mjd_from_utc_mjd(utc).expect("instant is inside the table");
        let back = utc_mjd_from_tai_mjd(tai).expect("instant is inside the table");
        assert!(
            (back - utc).abs() < 1e-10,
            "round trip failed at {year}-{month}-{day}: {utc} vs {back}"
        );
    }
}

#[test]
fn tai_to_utc_recovers_the_offset_on_each_side_of_a_step() {
    let effective = calendar_mjd(2017, 1, 1);
    // One hour before the step the offset is 36 s, one hour after it is 37 s.
    let before_utc = effective - 1.0 / 24.0;
    let after_utc = effective + 1.0 / 24.0;
    let before_tai = tai_mjd_from_utc_mjd(before_utc).expect("inside the table");
    let after_tai = tai_mjd_from_utc_mjd(after_utc).expect("inside the table");
    assert!((utc_mjd_from_tai_mjd(before_tai).expect("inside") - before_utc).abs() < 1e-10);
    assert!((utc_mjd_from_tai_mjd(after_tai).expect("inside") - after_utc).abs() < 1e-10);
}

#[test]
fn tai_to_utc_rejects_non_finite_input() {
    assert_eq!(utc_mjd_from_tai_mjd(f64::NAN), None);
    assert_eq!(utc_mjd_from_tai_mjd(f64::NEG_INFINITY), None);
}

#[test]
fn tai_to_utc_at_the_boundary_stays_inside_the_table() {
    // The first TAI instant that the table covers is 1972-01-01 00:00:10
    // TAI, because the offset there is ten seconds.
    let boundary = calendar_mjd(1972, 1, 1) + 10.0 / SECONDS_PER_DAY;
    let utc = utc_mjd_from_tai_mjd(boundary).expect("boundary is covered");
    assert!((utc - calendar_mjd(1972, 1, 1)).abs() < 1e-10);
}
