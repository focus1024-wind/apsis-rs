//! Errors reported while building or querying an ephemeris.

use core::fmt;

/// The category of an ephemeris error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EphemerisErrorKind {
    /// The sampling grid has fewer than two samples.
    TooFewSamples,
    /// A time or a step is not finite.
    NotFinite,
    /// A step is zero or negative.
    NonPositiveStep,
    /// The samples are not in increasing time order.
    NotMonotonic,
    /// A state produced by the sampler is not finite.
    InvalidState,
    /// A query time falls outside the covered interval.
    OutOfRange,
}

impl fmt::Display for EphemerisErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::TooFewSamples => "fewer than two samples",
            Self::NotFinite => "a time is not finite",
            Self::NonPositiveStep => "the step is not positive",
            Self::NotMonotonic => "the samples are not in increasing time order",
            Self::InvalidState => "the sampled state is not finite",
            Self::OutOfRange => "the time is outside the covered interval",
        };
        f.write_str(text)
    }
}

/// A failure while building or querying an ephemeris.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EphemerisError {
    kind: EphemerisErrorKind,
    detail: Option<String>,
}

impl EphemerisError {
    /// Creates an error with the given category.
    pub fn new(kind: EphemerisErrorKind) -> Self {
        Self { kind, detail: None }
    }

    /// Attaches a description of the value that caused the failure.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Returns the category of this failure.
    pub const fn kind(&self) -> EphemerisErrorKind {
        self.kind
    }

    /// Returns the detail text attached to this error, if any.
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for EphemerisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{}: {detail}", self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl core::error::Error for EphemerisError {}
