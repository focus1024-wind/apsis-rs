//! Tests for parsing a complete two-line element set.

use apsis_time::TimeScale;
use apsis_tle::{Classification, ElementErrorKind, TwoLineElement};

/// The sample element set for the International Space Station used by the
/// `sgp4` reference implementation.
const ISS: &str = concat!(
    "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\r\n",
    "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537\r\n",
);

#[test]
fn sample_fields_match_the_reference_parser() {
    let element = TwoLineElement::parse(ISS).expect("sample is well formed");
    assert_eq!(element.name, None);
    assert_eq!(element.satellite_number, 25544);
    assert_eq!(element.classification, Classification::Unclassified);
    assert_eq!(element.international_designator, "98067A");
    assert_eq!(element.element_set_number, 292);
    assert_eq!(element.revolution_number, 56353);
    assert!((element.inclination - 51.6416).abs() < 1e-12);
    assert!((element.right_ascension - 247.4627).abs() < 1e-12);
    assert!((element.eccentricity - 0.0006703).abs() < 1e-15);
    assert!((element.argument_of_perigee - 130.5360).abs() < 1e-12);
    assert!((element.mean_anomaly - 325.0288).abs() < 1e-12);
    assert!((element.mean_motion - 15.72125391).abs() < 1e-9);
}

#[test]
fn exponent_fields_use_an_assumed_decimal_point() {
    let element = TwoLineElement::parse(ISS).expect("sample is well formed");
    // `-11606-4` is -0.11606e-4.
    assert!((element.bstar - -1.1606e-5).abs() < 1e-18);
    // `00000-0` is zero.
    assert_eq!(element.mean_motion_double_dot, 0.0);
    // `-.00002182` is written with an explicit decimal point and no leading
    // zero.
    assert!((element.mean_motion_dot - -2.182e-5).abs() < 1e-18);
}

#[test]
fn epoch_resolves_to_the_expected_day() {
    let element = TwoLineElement::parse(ISS).expect("sample is well formed");
    // The reference parser reports Julian Date 2454730.01782528 for the
    // epoch, which is Modified Julian Date 54729.51782528 on the UTC scale.
    // Day 264 of 2008 is 2008-09-20, and the fraction is 12:25:40.
    let mjd = element
        .epoch
        .mjd(TimeScale::Utc)
        .expect("epoch is covered by the leap table");
    assert!((mjd - 54_729.517_825_28).abs() < 1e-9, "got {mjd}");

    let calendar = element
        .epoch
        .calendar(TimeScale::Utc)
        .expect("epoch is covered by the leap table");
    assert_eq!((calendar.year, calendar.month, calendar.day), (2008, 9, 20));
    assert_eq!(calendar.hour, 12);
    assert_eq!(calendar.minute, 25);
    assert!(
        (calendar.second - 40.104).abs() < 0.01,
        "got {}",
        calendar.second
    );
}

#[test]
fn three_line_form_captures_the_name() {
    let text = format!("ISS (ZARYA)\r\n{ISS}");
    let element = TwoLineElement::parse(&text).expect("three line form is valid");
    assert_eq!(element.name.as_deref(), Some("ISS (ZARYA)"));
    assert_eq!(element.satellite_number, 25544);
}

#[test]
fn line_endings_and_blank_lines_do_not_matter() {
    for text in [
        ISS.replace("\r\n", "\n"),
        format!("\n{ISS}\n"),
        format!("  \n{ISS}   \n"),
        ISS.replace("\r\n", "\r\n\n"),
    ] {
        assert!(TwoLineElement::parse(&text).is_ok(), "failed for {text:?}");
    }
}

#[test]
fn classification_is_read_from_the_sample() {
    let element = TwoLineElement::parse(ISS).expect("sample is well formed");
    assert_eq!(element.classification.as_char(), 'U');

    let secret = mutate(ISS, 0, 7, 'S');
    let element = TwoLineElement::parse(&secret).expect("valid after checksum fix");
    assert_eq!(element.classification, Classification::Secret);
}

/// Replaces the character at the given zero based column of the given zero
/// based line and recomputes that line's checksum.
fn mutate(text: &str, line_index: usize, column: usize, replacement: char) -> String {
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let mut characters: Vec<char> = lines[line_index].chars().collect();
    characters[column] = replacement;
    let body: String = characters[..68].iter().collect();
    let checksum = apsis_tle::line_checksum(&body);
    lines[line_index] = format!("{body}{checksum}");
    format!("{}\r\n{}\r\n", lines[0], lines[1])
}

#[test]
fn mismatched_satellite_numbers_are_rejected() {
    // Change a digit of the second line's satellite number and repair the
    // checksum, so that only the mismatch remains. The satellite number
    // occupies one based columns 3 to 7, which are zero based 2 to 6.
    let text = mutate(ISS, 1, 2, '3');
    let error = TwoLineElement::parse(&text).expect_err("numbers differ");
    assert_eq!(error.kind(), ElementErrorKind::MismatchedElementSet);
    assert_eq!(error.line(), 2);
}

#[test]
fn a_wrong_leading_digit_is_rejected() {
    let text = mutate(ISS, 0, 0, '3');
    let error = TwoLineElement::parse(&text).expect_err("first line must start with 1");
    assert_eq!(error.kind(), ElementErrorKind::UnexpectedCharacter);
    assert_eq!(error.line(), 1);
}

#[test]
fn a_corrupted_checksum_is_reported_with_its_line() {
    let text = ISS.replace("15.72125391563537", "15.72125391563538");
    let error = TwoLineElement::parse(&text).expect_err("checksum fails");
    assert_eq!(error.kind(), ElementErrorKind::Checksum);
    assert_eq!(error.line(), 2);
}

#[test]
fn too_few_lines_are_rejected() {
    let error = TwoLineElement::parse("1 25544U 98067A\n").expect_err("only one line");
    assert_eq!(error.kind(), ElementErrorKind::TooShort);
}

#[test]
fn an_empty_input_is_rejected() {
    assert!(TwoLineElement::parse("").is_err());
    assert!(TwoLineElement::parse("\n\n").is_err());
}

#[test]
fn a_non_numeric_field_is_reported_with_its_columns() {
    // Replace the first inclination digit with a letter, so that the
    // checksum still matches but the field cannot be read.
    let text = mutate(ISS, 1, 8, 'X');

    let error = TwoLineElement::parse(&text).expect_err("inclination is not numeric");
    assert_eq!(error.kind(), ElementErrorKind::InvalidField);
    assert_eq!(error.line(), 2);
    assert_eq!(error.columns(), Some((9, 16)));
}

#[test]
fn a_negative_satellite_number_is_rejected() {
    let text = mutate(ISS, 0, 2, '-');

    let error = TwoLineElement::parse(&text).expect_err("satellite number is not unsigned");
    assert_eq!(error.kind(), ElementErrorKind::InvalidField);
    assert_eq!(error.columns(), Some((3, 7)));
}
