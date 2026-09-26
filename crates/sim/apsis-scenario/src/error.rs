//! Errors reported while building, loading, or validating a scenario.

use core::fmt;

/// The category of a scenario error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ScenarioErrorKind {
    /// The scenario name is empty.
    EmptyName,
    /// A resource with the identifier already exists.
    DuplicateResource,
    /// A task with the identifier already exists.
    DuplicateTask,
    /// A requirement names a resource that is not in the scenario.
    UnknownResource,
    /// A precedence constraint names a task that is not in the scenario.
    UnknownTask,
    /// A resource in the scenario is itself not usable.
    InvalidResource,
    /// A task in the scenario is itself not usable.
    InvalidTask,
    /// A precedence constraint would close a cycle.
    CyclicPrecedence,
    /// A time limit is negative or not finite.
    InvalidTimeLimit,
    /// An epoch is negative or not finite.
    InvalidEpoch,
    /// A line has too few arguments for its keyword.
    MissingArgument,
    /// A line has an argument that cannot be read as the expected type.
    BadArgument,
    /// A line does not begin with a known keyword.
    UnknownKeyword,
    /// A keyword appears before the scenario header, or a required header is
    /// missing.
    MissingHeader,
    /// A field appears more than once where only one is allowed.
    DuplicateField,
}

impl fmt::Display for ScenarioErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::EmptyName => "the scenario name is empty",
            Self::DuplicateResource => "a resource with that identifier exists",
            Self::DuplicateTask => "a task with that identifier exists",
            Self::UnknownResource => "the resource is not in the scenario",
            Self::UnknownTask => "the task is not in the scenario",
            Self::InvalidResource => "the resource is not usable",
            Self::InvalidTask => "the task is not usable",
            Self::CyclicPrecedence => "the precedence constraint closes a cycle",
            Self::InvalidTimeLimit => "the time limit is not usable",
            Self::InvalidEpoch => "the epoch is not usable",
            Self::MissingArgument => "a line is missing an argument",
            Self::BadArgument => "an argument cannot be read as the expected type",
            Self::UnknownKeyword => "the line begins with an unknown keyword",
            Self::MissingHeader => "the scenario header is missing or out of place",
            Self::DuplicateField => "the field appears more than once",
        };
        f.write_str(text)
    }
}

/// A failure while building, loading, or validating a scenario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioError {
    kind: ScenarioErrorKind,
    detail: Option<String>,
    line: Option<usize>,
}

impl ScenarioError {
    /// Creates an error with the given category.
    pub fn new(kind: ScenarioErrorKind) -> Self {
        Self {
            kind,
            detail: None,
            line: None,
        }
    }

    /// Attaches a description of the values that caused the failure.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Attaches the one based line number a load failure came from.
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// Returns the category of this failure.
    pub fn kind(&self) -> ScenarioErrorKind {
        self.kind
    }

    /// Returns the detail text attached to this error, if any.
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    /// Returns the line number this failure came from, if any.
    pub fn line(&self) -> Option<usize> {
        self.line
    }
}

impl fmt::Display for ScenarioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.line, &self.detail) {
            (Some(line), Some(detail)) => write!(f, "line {line}: {}: {detail}", self.kind),
            (Some(line), None) => write!(f, "line {line}: {}", self.kind),
            (None, Some(detail)) => write!(f, "{}: {detail}", self.kind),
            (None, None) => write!(f, "{}", self.kind),
        }
    }
}

impl core::error::Error for ScenarioError {}
