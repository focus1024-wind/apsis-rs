//! Line checksums for the two-line element format.
//!
//! Each line ends with a single digit that is the checksum of the preceding
//! columns. The checksum is the sum of the decimal digits on the line, where
//! a minus sign contributes one and every other character contributes
//! nothing, reduced modulo ten.
//!
//! The checksum digit itself is the last character of the line and is not
//! part of the sum.

use crate::field::{ElementError, ElementErrorKind, Result};

/// Returns the checksum of the first 68 columns of a line.
///
/// The input may include the trailing checksum digit, which is ignored
/// because only the first 68 columns contribute to the sum. A line shorter
/// than 68 characters contributes only the columns it has.
///
/// A minus sign contributes one, a decimal digit contributes its value, and
/// every other character contributes nothing.
pub fn line_checksum(line: &str) -> u32 {
    line.chars()
        .take(68)
        .map(|character| match character {
            '-' => 1,
            '0'..='9' => character as u32 - '0' as u32,
            _ => 0,
        })
        .sum::<u32>()
        % 10
}

/// Verifies that the last character of a line is its checksum.
///
/// The line must be at least 69 characters long, because the checksum
/// occupies column 69. The expected value is compared against
/// [`line_checksum`] of the same line.
///
/// `line_number` is the one based number used in any error that is
/// returned, so that a caller parsing a multi-line input can report which
/// line failed.
pub fn verify_line_checksum(line: &str, line_number: usize) -> Result<()> {
    let checksum_columns = (69, 69);
    // The line must contain at least 69 characters. Counting characters
    // rather than bytes is deliberate: the format is ASCII, but a caller
    // may hand in text with a non-ASCII character by mistake, and counting
    // characters gives a clearer failure than slicing through one.
    let characters: Vec<char> = line.chars().collect();
    if characters.len() < 69 {
        return Err(ElementError::new(
            ElementErrorKind::TooShort,
            line_number,
            Some(checksum_columns),
        )
        .with_detail(format!("found {} characters, need 69", characters.len())));
    }
    let expected = characters[68];
    let Some(expected) = expected.to_digit(10) else {
        return Err(ElementError::new(
            ElementErrorKind::InvalidField,
            line_number,
            Some(checksum_columns),
        )
        .with_detail(format!("checksum character is {expected:?}, not a digit")));
    };
    let actual = line_checksum(line);
    if actual != expected {
        return Err(ElementError::new(
            ElementErrorKind::Checksum,
            line_number,
            Some(checksum_columns),
        )
        .with_detail(format!("computed {actual}, line states {expected}")));
    }
    Ok(())
}
