//! Tests for the Julian Date conversions.

use apsis_time::{JULIAN_DATE_OFFSET, julian_date_from_mjd, mjd_from_julian_date};

#[test]
fn the_offset_matches_the_defining_values() {
    // MJD 0 is Julian Date 2400000.5.
    assert_eq!(JULIAN_DATE_OFFSET, 2_400_000.5);
    assert_eq!(julian_date_from_mjd(0.0), 2_400_000.5);
    assert_eq!(mjd_from_julian_date(2_400_000.5), 0.0);
}

#[test]
fn j2000_maps_to_the_expected_julian_date() {
    // J2000.0 is MJD 51544.5 and Julian Date 2451545.0.
    assert_eq!(julian_date_from_mjd(51_544.5), 2_451_545.0);
    assert_eq!(mjd_from_julian_date(2_451_545.0), 51_544.5);
}

#[test]
fn conversion_round_trips() {
    for mjd in [0.0, 40_587.0, 51_544.5, 60_000.25, 60_310.0] {
        assert_eq!(mjd_from_julian_date(julian_date_from_mjd(mjd)), mjd);
    }
}
