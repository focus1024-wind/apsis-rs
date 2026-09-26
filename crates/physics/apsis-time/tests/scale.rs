//! Tests for the time scale definitions.

use apsis_time::{TAI_MINUS_GPS, TT_MINUS_TAI, TimeScale};

#[test]
fn only_utc_is_not_uniform() {
    assert!(!TimeScale::Utc.is_uniform());
    assert!(TimeScale::Tai.is_uniform());
    assert!(TimeScale::Tt.is_uniform());
    assert!(TimeScale::Gps.is_uniform());
}

#[test]
fn scales_report_their_names() {
    assert_eq!(TimeScale::Utc.name(), "UTC");
    assert_eq!(TimeScale::Tai.name(), "TAI");
    assert_eq!(TimeScale::Tt.name(), "TT");
    assert_eq!(TimeScale::Gps.name(), "GPS");
    assert_eq!(TimeScale::Tai.to_string(), "TAI");
}

#[test]
fn offsets_match_their_definitions() {
    // Terrestrial Time is ahead of TAI by the defining constant.
    assert_eq!(TT_MINUS_TAI, 32.184);
    // GPS time lags TAI by the defining integer number of seconds.
    assert_eq!(TAI_MINUS_GPS, 19.0);
}
