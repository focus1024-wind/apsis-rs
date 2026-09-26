//! The workspace error type and result alias.
//!
//! The workspace does not use an external error crate. Errors are modelled
//! as a single [`Error`] struct carrying an [`ErrorKind`], an optional
//! message, and an optional source. `ErrorKind` is a fieldless enum so that
//! callers can match on the failure category without allocating.

use core::error::Error as StdError;
use core::fmt;

/// The category of an [`Error`].
///
/// The variants are grouped by the layer that most often produces them, but
/// any layer may return any variant. Callers should match on the variant
/// that is relevant to their own recovery logic and treat the rest as
/// opaque.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ErrorKind {
    /// An input value is outside the range the function accepts.
    OutOfRange,
    /// An input does not satisfy a documented precondition.
    InvalidInput,
    /// An operation is not defined for the given inputs, such as a
    /// singularity in a coordinate transformation.
    Undefined,
    /// A requested item does not exist.
    NotFound,
    /// Two inputs are inconsistent with each other.
    Inconsistent,
    /// An operation did not converge within the allowed number of steps.
    NotConverged,
    /// Text could not be parsed into the expected structure.
    Parse,
    /// A numeric operation produced a non-finite result.
    NotFinite,
    /// A feature or capability is not implemented.
    Unsupported,
    /// A failure that does not fit any other category.
    Other,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::OutOfRange => "value out of range",
            Self::InvalidInput => "invalid input",
            Self::Undefined => "operation undefined for these inputs",
            Self::NotFound => "not found",
            Self::Inconsistent => "inconsistent inputs",
            Self::NotConverged => "did not converge",
            Self::Parse => "parse error",
            Self::NotFinite => "non-finite result",
            Self::Unsupported => "unsupported",
            Self::Other => "error",
        };
        f.write_str(text)
    }
}

/// The error type used across the workspace.
///
/// `Error` carries a machine matchable [`ErrorKind`] plus a human readable
/// message. An optional source records the lower level error that caused
/// this one.
pub struct Error {
    kind: ErrorKind,
    message: Option<String>,
    source: Option<Box<dyn StdError + Send + Sync + 'static>>,
}

impl Error {
    /// Creates an error with the given kind and no message.
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            message: None,
            source: None,
        }
    }

    /// Creates an error with the given kind and message.
    ///
    /// The message should describe the concrete cause of this specific
    /// failure. It is meant for humans reading logs, not for matching.
    pub fn with_message(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: Some(message.into()),
            source: None,
        }
    }

    /// Creates an error with the given kind and a source error.
    pub fn with_source<E>(kind: ErrorKind, source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        Self {
            kind,
            message: None,
            source: Some(Box::new(source)),
        }
    }

    /// Attaches a message to this error, replacing any existing message.
    pub fn with_context(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Attaches a source error, replacing any existing source.
    pub fn caused_by<E>(mut self, source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        self.source = Some(Box::new(source));
        self
    }

    /// Returns the category of this error.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Returns the message attached to this error, if any.
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Error");
        debug.field("kind", &self.kind);
        if let Some(message) = &self.message {
            debug.field("message", message);
        }
        if let Some(source) = &self.source {
            debug.field("source", source);
        }
        debug.finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.message {
            Some(message) => write!(f, "{}: {message}", self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn StdError + 'static))
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self::new(kind)
    }
}

/// A [`Result`](core::result::Result) whose error type is [`Error`].
pub type Result<T> = core::result::Result<T, Error>;
