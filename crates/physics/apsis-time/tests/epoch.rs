//! Tests for the instant type and its scale conversions.

use apsis_time::{Calendar, Epoch, SECONDS_PER_DAY, TAI_MINUS_GPS, TT_MINUS_TAI, TimeScale};

/// Tolerance for comparisons of Modified Julian Dates near the present day.
///
/// A double holds a Modified Julian Date near 60 000 with a resolution of
/// about 1e-11 days, roughly one microsecond. Conversions that add or
/// remove a scale offset each introduce one rounding at that resolution, so
/// comparisons are made at ten microseconds to leave room for a few steps.
const MJD_TOL: f64 = 1e-9;

fn assert_mjd_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < MJD_TOL,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn j2000_constant_lands_on_the_defining_instant() {
    // 2000-01-01T12:00:00 TT is Julian Date 2451545.0, or MJD 51544.5.
    let tt = Epoch::J2000
        .mjd(TimeScale::Tt)
        .expect("TT is always available");
    assert_mjd_close(tt, 51_544.5);
}

#[test]
fn uniform_scales_differ_by_their_defining_offsets() {
    let epoch = Epoch::from_mjd(60_000.0, TimeScale::Tai).expect("TAI is always available");
    assert_mjd_close(
        epoch.mjd(TimeScale::Tt).expect("TT is available"),
        60_000.0 + TT_MINUS_TAI / SECONDS_PER_DAY,
    );
    assert_mjd_close(
        epoch.mjd(TimeScale::Gps).expect("GPS is available"),
        60_000.0 - TAI_MINUS_GPS / SECONDS_PER_DAY,
    );
}

#[test]
fn construction_from_each_scale_round_trips() {
    // 60 000.5 is a mid-day instant, which keeps the fractional part of
    // every intermediate value exactly representable.
    let input = 60_000.5;
    for scale in [
        TimeScale::Tai,
        TimeScale::Tt,
        TimeScale::Gps,
        TimeScale::Utc,
    ] {
        let epoch = Epoch::from_mjd(input, scale).expect("instant is inside the leap table");
        // Reading the instant back on the same scale recovers the number
        // that was supplied.
        assert_mjd_close(epoch.mjd(scale).expect("scale is available"), input);
    }
}

#[test]
fn utc_construction_applies_the_leap_second_offset() {
    // In 2017 the offset TAI - UTC is 37 seconds.
    let utc = 60_000.0;
    let epoch = Epoch::from_mjd(utc, TimeScale::Utc).expect("covered by the table");
    assert_mjd_close(epoch.tai_mjd(), utc + 37.0 / SECONDS_PER_DAY);
}

#[test]
fn non_finite_input_is_rejected() {
    assert_eq!(Epoch::from_mjd(f64::NAN, TimeScale::Tai), None);
    assert_eq!(Epoch::from_mjd(f64::INFINITY, TimeScale::Tai), None);
}

#[test]
fn utc_conversion_is_rejected_before_the_table_begins() {
    let before = 40_000.0;
    assert_eq!(Epoch::from_mjd(before, TimeScale::Utc), None);
    // The same instant is representable in TAI.
    let epoch = Epoch::from_mjd(before, TimeScale::Tai).expect("TAI is always available");
    assert_eq!(epoch.mjd(TimeScale::Utc), None);
}

#[test]
fn calendar_construction_and_read_back_agree() {
    let calendar = Calendar::new(2024, 6, 15, 12, 30, 45.0);
    let epoch = Epoch::from_calendar(calendar, TimeScale::Utc).expect("covered by the table");
    let read_back = epoch
        .calendar(TimeScale::Utc)
        .expect("covered by the table");
    assert_eq!(read_back.year, 2024);
    assert_eq!(read_back.month, 6);
    assert_eq!(read_back.day, 15);
    assert_eq!(read_back.hour, 12);
    assert_eq!(read_back.minute, 30);
    assert!((read_back.second - 45.0).abs() < 1e-4);
}

#[test]
fn calendar_construction_rejects_impossible_dates() {
    let calendar = Calendar::from_ymd(2023, 2, 29);
    assert_eq!(Epoch::from_calendar(calendar, TimeScale::Tai), None);
}

#[test]
fn seconds_until_is_independent_of_the_scale_used() {
    let start = Epoch::from_calendar(Calendar::from_ymd(2024, 1, 1), TimeScale::Utc)
        .expect("covered by the table");
    let end = Epoch::from_calendar(Calendar::from_ymd(2024, 1, 2), TimeScale::Utc)
        .expect("covered by the table");
    // One calendar day in early 2024 contains no leap second, so the
    // elapsed time is exactly 86 400 seconds.
    assert!((start.seconds_until(end) - 86_400.0).abs() < 1e-6);
    assert!((end.seconds_until(start) + 86_400.0).abs() < 1e-6);
}

#[test]
fn add_seconds_moves_along_the_uniform_scale() {
    let epoch = Epoch::from_mjd(60_000.5, TimeScale::Tai).expect("TAI is always available");
    let later = epoch.add_seconds(3_600.0);
    assert!((epoch.seconds_until(later) - 3_600.0).abs() < 1e-3);
    let earlier = epoch.add_seconds(-3_600.0);
    assert!((earlier.seconds_until(epoch) - 3_600.0).abs() < 1e-3);
}

#[test]
fn add_seconds_crosses_a_leap_second_as_elapsed_time() {
    // 2016-12-31T23:59:59.5 UTC, half a second before the inserted leap
    // second. The leap second makes the next UTC midnight 1.5 SI seconds
    // away, because the interval labelled 23:59:60 is a whole second long.
    let before = Epoch::from_calendar(Calendar::new(2016, 12, 31, 23, 59, 59.5), TimeScale::Utc)
        .expect("covered by the table");

    // One and a half seconds of elapsed time reaches the next midnight.
    let after = before.add_seconds(1.5);
    let utc = after
        .calendar(TimeScale::Utc)
        .expect("covered by the table");
    assert_eq!((utc.year, utc.month, utc.day), (2017, 1, 1));
    assert!(utc.second.abs() < 1e-4, "got {}", utc.second);

    // An instant strictly inside the leap second has no UTC label on the
    // continuous Modified Julian Date scale, because it would be written
    // 23:59:60 and that value does not exist there.
    assert_eq!(before.add_seconds(1.0).mjd(TimeScale::Utc), None);

    // Reading the interval on the UTC clock shows half a second of clock
    // advance over one and a half seconds of elapsed time.
    let utc_before = before.mjd(TimeScale::Utc).expect("covered by the table");
    let utc_after = after.mjd(TimeScale::Utc).expect("covered by the table");
    let clock_advance = (utc_after - utc_before) * SECONDS_PER_DAY;
    assert!((clock_advance - 0.5).abs() < 1e-3, "got {clock_advance}");
}

#[test]
fn ordering_follows_the_instant() {
    let earlier = Epoch::from_mjd(60_000.0, TimeScale::Tai).expect("TAI is always available");
    let later = earlier.add_seconds(1.0);
    assert!(earlier < later);
    assert!(later > earlier);
    assert!(earlier <= earlier);
}

#[test]
fn equality_is_scale_independent() {
    // The same physical instant described in two scales yields equal
    // epochs.
    let in_tai = Epoch::from_mjd(60_000.0, TimeScale::Tai).expect("TAI is always available");
    let in_tt = Epoch::from_mjd(60_000.0 + TT_MINUS_TAI / SECONDS_PER_DAY, TimeScale::Tt)
        .expect("TT is always available");
    assert_eq!(in_tai, in_tt);
    assert!(in_tai.seconds_until(in_tt).abs() < 1e-6);
}

#[test]
fn finiteness_is_reported() {
    assert!(
        Epoch::from_mjd(60_000.0, TimeScale::Tai)
            .expect("always available")
            .is_finite()
    );
    assert!(!Epoch::from_tai_mjd(f64::NAN).is_finite());
    assert!(!Epoch::from_tai_mjd(f64::INFINITY).is_finite());
}

#[test]
fn display_reports_the_stored_scale() {
    let epoch = Epoch::from_tai_mjd(60_000.5);
    assert_eq!(epoch.to_string(), "60000.5 TAI");
}
