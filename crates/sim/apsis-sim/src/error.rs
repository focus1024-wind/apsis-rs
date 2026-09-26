//! Errors reported while preparing or starting a simulation.

use core::fmt;

/// The category of a simulation error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SimulationErrorKind {
    /// The scenario is not usable.
    InvalidScenario,
    /// The scheduled resources do not form a usable set.
    InvalidResources,
    /// A scheduled placement does not match the task it names.
    InconsistentPlan,
}

impl fmt::Display for SimulationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::InvalidScenario => "the scenario is not usable",
            Self::InvalidResources => "the resources are not a usable set",
            Self::InconsistentPlan => "the plan does not match the scenario",
        };
        f.write_str(text)
    }
}

/// A failure while preparing or starting a simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationError {
    kind: SimulationErrorKind,
    detail: Option<String>,
}

impl SimulationError {
    /// Creates an error with the given category.
    pub fn new(kind: SimulationErrorKind) -> Self {
        Self { kind, detail: None }
    }

    /// Attaches a description of what caused the failure.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Returns the category of this failure.
    pub const fn kind(&self) -> SimulationErrorKind {
        self.kind
    }

    /// Returns the detail text attached to this error, if any.
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for SimulationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{}: {detail}", self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl core::error::Error for SimulationError {}
