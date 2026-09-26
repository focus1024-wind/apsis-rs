//! Errors reported while building or querying a resource.

use core::fmt;

/// The category of a resource error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ResourceErrorKind {
    /// The end of an interval precedes its start.
    ReversedInterval,
    /// A time or duration is not finite.
    NotFinite,
    /// A duration is negative.
    NegativeDuration,
    /// The capacity is not positive and finite.
    InvalidCapacity,
    /// No resource has the requested identifier.
    NoSuchResource,
    /// A resource with the identifier already exists.
    DuplicateResource,
}

impl fmt::Display for ResourceErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::ReversedInterval => "the interval ends before it starts",
            Self::NotFinite => "a time is not finite",
            Self::NegativeDuration => "the duration is negative",
            Self::InvalidCapacity => "the capacity is not usable",
            Self::NoSuchResource => "no such resource",
            Self::DuplicateResource => "a resource with that identifier exists",
        };
        f.write_str(text)
    }
}

/// A failure while building or querying a resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceError {
    kind: ResourceErrorKind,
    detail: Option<String>,
}

impl ResourceError {
    /// Creates an error with the given category.
    pub fn new(kind: ResourceErrorKind) -> Self {
        Self { kind, detail: None }
    }

    /// Attaches a description of the values that caused the failure.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Returns the category of this failure.
    pub fn kind(&self) -> ResourceErrorKind {
        self.kind
    }

    /// Returns the detail text attached to this error, if any.
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{}: {detail}", self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl core::error::Error for ResourceError {}
