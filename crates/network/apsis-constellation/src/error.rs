//! Errors reported while constructing a constellation.

use core::fmt;

/// The category of a constellation error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ConstellationErrorKind {
    /// The number of satellites is not positive.
    NoSatellites,
    /// The number of planes is not positive.
    NoPlanes,
    /// The number of satellites is not a multiple of the number of planes,
    /// so the satellites cannot be spread evenly.
    UnevenPlanes,
    /// An orbital parameter is outside the range the orbit layer accepts.
    InvalidOrbit,
    /// A shell or satellite index refers to something that does not exist.
    NotFound,
}

impl fmt::Display for ConstellationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::NoSatellites => "the constellation has no satellites",
            Self::NoPlanes => "the constellation has no orbital planes",
            Self::UnevenPlanes => "the satellite count is not a multiple of the plane count",
            Self::InvalidOrbit => "the orbital parameters are not usable",
            Self::NotFound => "the requested shell or satellite does not exist",
        };
        f.write_str(text)
    }
}

/// A failure while building a constellation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstellationError {
    kind: ConstellationErrorKind,
    detail: Option<String>,
}

impl ConstellationError {
    /// Creates an error with the given category.
    pub fn new(kind: ConstellationErrorKind) -> Self {
        Self { kind, detail: None }
    }

    /// Attaches a description of the values that caused the failure.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Returns the category of this failure.
    pub fn kind(&self) -> ConstellationErrorKind {
        self.kind
    }

    /// Returns the detail text attached to this error, if any.
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for ConstellationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{}: {detail}", self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl core::error::Error for ConstellationError {}
