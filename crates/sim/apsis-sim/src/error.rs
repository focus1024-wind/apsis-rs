//! Errors reported while preparing or starting a simulation.
//!
//! 准备或启动仿真时报告的错误。

use core::fmt;

/// The category of a simulation error.
///
/// 仿真错误的类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SimulationErrorKind {
    /// The scenario is not usable.
    ///
    /// 该场景不可用。
    InvalidScenario,
    /// The scheduled resources do not form a usable set.
    ///
    /// 所调度的资源未构成可用的集合。
    InvalidResources,
    /// A scheduled placement does not match the task it names.
    ///
    /// 某处放置与其所指名的任务不匹配。
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
///
/// 准备或启动仿真时发生的失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationError {
    kind: SimulationErrorKind,
    detail: Option<String>,
}

impl SimulationError {
    /// Creates an error with the given category.
    ///
    /// 用给定类别创建一个错误。
    pub fn new(kind: SimulationErrorKind) -> Self {
        Self { kind, detail: None }
    }

    /// Attaches a description of what caused the failure.
    ///
    /// 附加对导致该失败之事的说明。
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Returns the category of this failure.
    ///
    /// 返回该失败的类别。
    pub const fn kind(&self) -> SimulationErrorKind {
        self.kind
    }

    /// Returns the detail text attached to this error, if any.
    ///
    /// 返回附加在该错误上的说明文本，若有。
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
