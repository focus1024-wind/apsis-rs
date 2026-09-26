//! Errors reported while routing.

use core::fmt;

/// The category of a routing error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RoutingErrorKind {
    /// A node index is outside the range of the graph.
    NoSuchNode,
    /// No route exists between the two nodes.
    NoRoute,
    /// The two endpoints are the same node.
    SameEndpoint,
    /// The interval is empty or not ordered.
    InvalidInterval,
    /// The topology series does not cover the requested interval.
    SeriesTooShort,
}

impl fmt::Display for RoutingErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::NoSuchNode => "no such node",
            Self::NoRoute => "no route exists between these nodes",
            Self::SameEndpoint => "the source and destination are the same node",
            Self::InvalidInterval => "the interval is not usable",
            Self::SeriesTooShort => "the topology series does not cover the interval",
        };
        f.write_str(text)
    }
}

/// A failure while routing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingError {
    kind: RoutingErrorKind,
    detail: Option<String>,
}

impl RoutingError {
    /// Creates an error with the given category.
    pub fn new(kind: RoutingErrorKind) -> Self {
        Self { kind, detail: None }
    }

    /// Attaches a description of the values that caused the failure.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Returns the category of this failure.
    pub fn kind(&self) -> RoutingErrorKind {
        self.kind
    }

    /// Returns the detail text attached to this error, if any.
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for RoutingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{}: {detail}", self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl core::error::Error for RoutingError {}
