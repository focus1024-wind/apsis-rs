//! Tests for line checksums.

use apsis_tle::{line_checksum, verify_line_checksum};

const ISS_LINE_1: &str = "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927";
const ISS_LINE_2: &str = "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537";

#[test]
fn checksum_of_the_sample_lines_matches_the_final_digit() {
    assert_eq!(line_checksum(ISS_LINE_1), 7);
    assert_eq!(line_checksum(ISS_LINE_2), 7);
}

#[test]
fn verification_accepts_a_well_formed_line() {
    assert!(verify_line_checksum(ISS_LINE_1, 1).is_ok());
    assert!(verify_line_checksum(ISS_LINE_2, 2).is_ok());
}

#[test]
fn verification_rejects_a_corrupted_digit() {
    // Changing a digit in the body changes the sum.
    let corrupted = "1 35544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927";
    let error = verify_line_checksum(corrupted, 1).expect_err("checksum should fail");
    assert_eq!(error.kind(), apsis_tle::ElementErrorKind::Checksum);
    assert_eq!(error.line(), 1);
}

#[test]
fn verification_rejects_a_corrupted_checksum_digit() {
    let corrupted = "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2928";
    assert!(verify_line_checksum(corrupted, 1).is_err());
}

#[test]
fn minus_signs_contribute_one_each() {
    // A line of only minus signs, padded to 68 columns, sums to eight
    // modulo ten.
    let line = format!("{}{}", "-".repeat(68), "8");
    assert_eq!(line_checksum(&line), 8);
}

#[test]
fn decimal_points_and_spaces_contribute_nothing() {
    let line = format!("{}{}{}", " ".repeat(34), ".".repeat(34), "0");
    assert_eq!(line_checksum(&line), 0);
}

#[test]
fn only_the_first_68_columns_contribute() {
    let mut line = format!("{}0", "1".repeat(68));
    assert_eq!(line_checksum(&line), 8);
    // Appending digits beyond column 68 must not change the sum.
    line.push('9');
    assert_eq!(line_checksum(&line), 8);
}

#[test]
fn short_lines_are_rejected_with_the_column_range() {
    let error = verify_line_checksum("1 25544", 2).expect_err("line is too short");
    assert_eq!(error.kind(), apsis_tle::ElementErrorKind::TooShort);
    assert_eq!(error.line(), 2);
    assert_eq!(error.columns(), Some((69, 69)));
}

#[test]
fn a_non_numeric_checksum_is_rejected() {
    let line = format!("{}X", "1".repeat(68));
    let error = verify_line_checksum(&line, 3).expect_err("checksum is not a digit");
    assert_eq!(error.kind(), apsis_tle::ElementErrorKind::InvalidField);
    assert_eq!(error.line(), 3);
}
