//! The error type and result alias for element parsing.

use core::fmt;

/// The category of a parse failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ElementErrorKind {
    /// A line is shorter than the format requires.
    TooShort,
    /// A fixed width field does not contain a well formed value.
    InvalidField,
    /// The checksum at the end of a line does not match the line contents.
    Checksum,
    /// The two lines carry different element set numbers.
    MismatchedElementSet,
    /// A character where a digit or a specific symbol was expected.
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
    pub fn new(kind: ElementErrorKind, line: usize, columns: Option<(usize, usize)>) -> Self {
        Self {
            kind,
            line,
            columns,
            detail: None,
        }
    }

    /// Attaches a description of the offending text.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Returns the category of this failure.
    pub fn kind(&self) -> ElementErrorKind {
        self.kind
    }

    /// Returns the one based number of the line that failed.
    pub fn line(&self) -> usize {
        self.line
    }

    /// Returns the one based inclusive column range that failed, if any.
    pub fn columns(&self) -> Option<(usize, usize)> {
        self.columns
    }

    /// Returns the detail text attached to this error, if any.
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
pub type Result<T> = core::result::Result<T, ElementError>;
