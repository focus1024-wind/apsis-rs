//! Errors reported by the propagator.

use core::fmt;

/// The category of a propagation failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PropagationErrorKind {
    /// The mean elements do not describe a satellite the model can
    /// propagate. This includes an eccentricity at or above one, which is
    /// not a closed orbit, and a mean motion that is not positive.
    InvalidElements,
    /// The satellite has decayed, meaning the drag term has brought it
    /// below the surface of the Earth.
    Decayed,
    /// The propagation left the region where the model is defined.
    OutOfRange,
}

impl fmt::Display for PropagationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::InvalidElements => "mean elements are not propagatable",
            Self::Decayed => "satellite has decayed",
            Self::OutOfRange => "propagation is out of range",
        };
        f.write_str(text)
    }
}

/// A failure while initialising or propagating a satellite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropagationError {
    kind: PropagationErrorKind,
    detail: Option<String>,
}

impl PropagationError {
    /// Creates an error with the given category.
    pub fn new(kind: PropagationErrorKind) -> Self {
        Self { kind, detail: None }
    }

    /// Attaches a description of the value that caused the failure.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Returns the category of this failure.
    pub fn kind(&self) -> PropagationErrorKind {
        self.kind
    }

    /// Returns the detail text attached to this error, if any.
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for PropagationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{}: {detail}", self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl core::error::Error for PropagationError {}
