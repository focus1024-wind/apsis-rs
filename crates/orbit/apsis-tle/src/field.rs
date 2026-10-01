//! The error type and result alias for element parsing.

use core::fmt;

/// The category of a parse failure.
///
/// 解析失败的类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ElementErrorKind {
    /// A line is shorter than the format requires.
    ///
    /// 某行短于格式要求的长度。
    TooShort,
    /// A fixed width field does not contain a well formed value.
    ///
    /// 某定宽字段不含良构的数值。
    InvalidField,
    /// The checksum at the end of a line does not match the line contents.
    ///
    /// 行末校验和与该行内容不符。
    Checksum,
    /// The two lines carry different element set numbers.
    ///
    /// 两行携带不同的根数集编号。
    MismatchedElementSet,
    /// A character where a digit or a specific symbol was expected.
    ///
    /// 期望数字或特定符号处出现了其他字符。
    UnexpectedCharacter,
}

impl fmt::Display for ElementErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::TooShort => "line is too short",
            Self::InvalidField => "field is not well formed",
            Self::Checksum => "line checksum does not match",
            Self::MismatchedElementSet => "element set numbers differ between lines",
            Self::UnexpectedCharacter => "unexpected character",
        };
        f.write_str(text)
    }
}

/// A failure while parsing a two-line element set.
///
/// The error records which line failed and, where applicable, which columns
/// were being read, so that a caller can point at the offending text. Line
/// numbers are one based and column ranges are the one based inclusive
/// range from the format definition.
///
/// 解析两线根数集时的失败。
///
/// 该错误记录哪一行失败，并在适用时记录当时正在读取的列，使调用方能指出有问题的
/// 文本。行号从一开始，列区间为格式定义中从一开始的含两端区间。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementError {
    kind: ElementErrorKind,
    line: usize,
    columns: Option<(usize, usize)>,
    detail: Option<String>,
}

impl ElementError {
    /// Creates an error with the given kind, line number, and columns.
    ///
    /// The line number is one based. The column range is the one based
    /// inclusive range that was being read, if the failure is tied to a
    /// field.
    ///
    /// 用给定类别、行号与列区间创建错误。
    ///
    /// 行号从一开始。列区间为当时正在读取的从一开始的含两端区间，当失败与某个
    /// 字段相关时给出。
    pub fn new(kind: ElementErrorKind, line: usize, columns: Option<(usize, usize)>) -> Self {
        Self {
            kind,
            line,
            columns,
            detail: None,
        }
    }

    /// Attaches a description of the offending text.
    ///
    /// 附加对有问题的文本的说明。
    ///
    /// 附加对有问题的文本的说明。
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Returns the category of this failure.
    ///
    /// 返回该失败的类别。
    ///
    /// 返回该失败的类别。
    pub fn kind(&self) -> ElementErrorKind {
        self.kind
    }

    /// Returns the one based number of the line that failed.
    ///
    /// 返回失败行的从一而行号。
    pub fn line(&self) -> usize {
        self.line
    }

    /// Returns the one based inclusive column range that failed, if any.
    ///
    /// 返回失败的从一开始的含两端列区间，若有。
    pub fn columns(&self) -> Option<(usize, usize)> {
        self.columns
    }

    /// Returns the detail text attached to this error, if any.
    ///
    /// 返回附加在该错误上的说明文本，若有。
    ///
    /// 返回附加在该错误上的说明文本，若有。
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for ElementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.kind)?;
        if let Some((first, last)) = self.columns {
            write!(f, " (columns {first}-{last})")?;
        }
        if let Some(detail) = &self.detail {
            write!(f, ": {detail}")?;
        }
        Ok(())
    }
}

impl core::error::Error for ElementError {}

/// A [`Result`](core::result::Result) whose error type is [`ElementError`].
///
/// 错误类型为 [`ElementError`] 的 [`Result`](core::result::Result)。
pub type Result<T> = core::result::Result<T, ElementError>;
