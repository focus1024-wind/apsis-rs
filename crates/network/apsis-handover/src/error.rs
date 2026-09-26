//! Errors reported while making a handover decision.

use core::fmt;

/// The category of a handover error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum HandoverErrorKind {
    /// The candidate list is empty.
    NoCandidates,
    /// The same satellite appears more than once in the candidate list.
    DuplicateSatellite,
    /// A quality value is outside its range or not finite.
    InvalidQuality,
    /// A policy parameter is not usable.
    InvalidPolicy,
}

impl fmt::Display for HandoverErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::NoCandidates => "there are no candidates to choose from",
            Self::DuplicateSatellite => "a satellite appears more than once",
            Self::InvalidQuality => "a quality value is not usable",
            Self::InvalidPolicy => "a policy parameter is not usable",
        };
        f.write_str(text)
    }
}

/// A failure while making a handover decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoverError {
    kind: HandoverErrorKind,
    detail: Option<String>,
}

impl HandoverError {
    /// Creates an error with the given category.
    pub fn new(kind: HandoverErrorKind) -> Self {
        Self { kind, detail: None }
    }

    /// Attaches a description of the values that caused the failure.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Returns the category of this failure.
    pub fn kind(&self) -> HandoverErrorKind {
        self.kind
    }

    /// Returns the detail text attached to this error, if any.
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for HandoverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{}: {detail}", self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl core::error::Error for HandoverError {}
