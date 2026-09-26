//! Errors reported while building or querying a task.

use core::fmt;

/// The category of a task error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TaskErrorKind {
    /// The duration is not finite, or is negative.
    InvalidDuration,
    /// A time window is not finite or ends before it starts.
    InvalidWindow,
    /// The window is shorter than the task duration.
    WindowTooShort,
    /// A resource demand is not positive and finite.
    InvalidDemand,
    /// The same resource is required more than once.
    DuplicateRequirement,
    /// A priority or value is not finite.
    InvalidWeight,
    /// No task has the requested identifier.
    NoSuchTask,
    /// A task with the identifier already exists.
    DuplicateTask,
    /// A precedence constraint would close a cycle.
    PrecedenceCycle,
}

impl fmt::Display for TaskErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::InvalidDuration => "the duration is not usable",
            Self::InvalidWindow => "the time window is not usable",
            Self::WindowTooShort => "the window is shorter than the task",
            Self::InvalidDemand => "a resource demand is not usable",
            Self::DuplicateRequirement => "a resource is required more than once",
            Self::InvalidWeight => "a priority or value is not finite",
            Self::NoSuchTask => "no such task",
            Self::DuplicateTask => "a task with that identifier exists",
            Self::PrecedenceCycle => "the precedence constraints contain a cycle",
        };
        f.write_str(text)
    }
}

/// A failure while building or querying a task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskError {
    kind: TaskErrorKind,
    detail: Option<String>,
}

impl TaskError {
    /// Creates an error with the given category.
    pub fn new(kind: TaskErrorKind) -> Self {
        Self { kind, detail: None }
    }

    /// Attaches a description of the values that caused the failure.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Returns the category of this failure.
    pub fn kind(&self) -> TaskErrorKind {
        self.kind
    }

    /// Returns the detail text attached to this error, if any.
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{}: {detail}", self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl core::error::Error for TaskError {}
