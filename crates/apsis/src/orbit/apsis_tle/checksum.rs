//! Line checksums for the two-line element format.
//!
//! Each line ends with a single digit that is the checksum of the preceding
//! columns. The checksum is the sum of the decimal digits on the line, where
//! a minus sign contributes one and every other character contributes
//! nothing, reduced modulo ten.
//!
//! The checksum digit itself is the last character of the line and is not
//! part of the sum.
//!
//! 两行根数格式的行校验和。
//!
//! 每行以一个数字结尾，它是前面各列的校验和。该校验和是该行十进制数字之和
//! 模十，其中负号贡献一，其余每个字符贡献零。
//!
//! 校验和数字本身是该行的最后一个字符，不属于该和。

use crate::apsis_tle::field::{ElementError, ElementErrorKind, Result};

/// Returns the checksum of the first 68 columns of a line.
///
/// The input may include the trailing checksum digit, which is ignored
/// because only the first 68 columns contribute to the sum. A line shorter
/// than 68 characters contributes only the columns it has.
///
/// A minus sign contributes one, a decimal digit contributes its value, and
/// every other character contributes nothing.
///
/// 返回某行前 68 列的校验和。
///
/// 输入可以包含末尾的校验和数字，该位被忽略，因为只有前 68 列参与求和。短于
/// 68 字符的行只贡献其实际具有的列。
///
/// 负号贡献 1，十进制数字贡献其数值，其余字符不贡献。
#[must_use]
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
///
/// 校验某行末位字符是否为其校验和。
///
/// 该行至少须有 69 个字符，因为校验和位于第 69 列。期望值与同一行的
/// [`line_checksum`] 比较。
///
/// `line_number` 是返回错误时使用的从一开始的行号，使解析多行输入的调用方
/// 能指出是哪一行失败。
pub fn verify_line_checksum(line: &str, line_number: usize) -> Result<()> {
    let checksum_columns = (69, 69);
    // The line must contain at least 69 characters. Counting characters
    // rather than bytes is deliberate: the format is ASCII, but a caller
    // may hand in text with a non-ASCII character by mistake, and counting
    // characters gives a clearer failure than slicing through one.
    // 该行至少须含 69 个字符。刻意按字符而非字节计数：格式虽为 ASCII，但
    // 调用方可能误传含非 ASCII 字符的文本，按字符计数给出的失败信息比
    // 从多字节字符中间切片更清晰。
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
