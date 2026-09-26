//! Errors reported while building or evaluating a schedule.

use core::fmt;

/// The category of a schedule error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ScheduleErrorKind {
    /// The start time is not finite.
    InvalidStart,
    /// The task is already placed.
    AlreadyPlaced,
    /// The task is not placed.
    NotPlaced,
    /// The placement starts before the task's window opens.
    BeforeWindow,
    /// The placement ends after the task's window closes.
    AfterWindow,
    /// The placement does not use the resources the task requires.
    RequirementMismatch,
    /// A resource is used beyond its capacity.
    CapacityExceeded,
}

impl fmt::Display for ScheduleErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::InvalidStart => "the start time is not usable",
            Self::AlreadyPlaced => "the task is already placed",
            Self::NotPlaced => "the task is not placed",
            Self::BeforeWindow => "the placement starts before the task window",
            Self::AfterWindow => "the placement ends after the task window",
            Self::RequirementMismatch => "the placement does not match the task requirements",
            Self::CapacityExceeded => "a resource is used beyond its capacity",
        };
        f.write_str(text)
    }
}

/// A failure while building or evaluating a schedule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleError {
    kind: ScheduleErrorKind,
    detail: Option<String>,
}

impl ScheduleError {
    /// Creates an error with the given category.
    pub fn new(kind: ScheduleErrorKind) -> Self {
        Self { kind, detail: None }
    }

    /// Attaches a description of the values that caused the failure.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Returns the category of this failure.
    pub fn kind(&self) -> ScheduleErrorKind {
        self.kind
    }

    /// Returns the detail text attached to this error, if any.
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{}: {detail}", self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl core::error::Error for ScheduleError {}
