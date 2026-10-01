//! Errors reported while finding access windows.

use core::fmt;

/// The category of an access error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AccessErrorKind {
    /// The station or its mask is not usable.
    InvalidStation,
    /// The sampling step is not positive and finite.
    InvalidSampleStep,
    /// The crossing tolerance is not positive and finite.
    InvalidTolerance,
    /// The search interval is empty or not ordered.
    InvalidInterval,
    /// The sample sequence is not in increasing time order.
    UnorderedSamples,
    /// A supplied satellite position is not usable.
    InvalidPosition,
}

impl fmt::Display for AccessErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::InvalidStation => "the ground station is not usable",
            Self::InvalidSampleStep => "the sampling step is not usable",
            Self::InvalidTolerance => "the crossing tolerance is not usable",
            Self::InvalidInterval => "the search interval is not usable",
            Self::UnorderedSamples => "the sample sequence is not ordered",
            Self::InvalidPosition => "a satellite position is not usable",
        };
        f.write_str(text)
    }
}

/// A failure while finding access windows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessError {
    kind: AccessErrorKind,
    detail: Option<String>,
}

impl AccessError {
    /// Creates an error with the given category.
    pub fn new(kind: AccessErrorKind) -> Self {
        Self { kind, detail: None }
    }

    /// Attaches a description of the values that caused the failure.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Returns the category of this failure.
    pub fn kind(&self) -> AccessErrorKind {
        self.kind
    }

    /// Returns the detail text attached to this error, if any.
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for AccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{}: {detail}", self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl core::error::Error for AccessError {}
