//! Errors reported while building or evaluating a schedule.
//!
//! 构建或评估调度时报告的错误。

use core::fmt;

/// The category of a schedule error.
///
/// 计划错误的类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ScheduleErrorKind {
    /// The start time is not finite.
    ///
    /// 起始时刻非有限。
    InvalidStart,
    /// The task is already placed.
    ///
    /// 该任务已被放置。
    AlreadyPlaced,
    /// The task is not placed.
    ///
    /// 该任务未被放置。
    NotPlaced,
    /// The placement starts before the task's window opens.
    ///
    /// 放置早于该任务窗口的开启时刻。
    BeforeWindow,
    /// The placement ends after the task's window closes.
    ///
    /// 放置晚于该任务窗口的关闭时刻。
    AfterWindow,
    /// The placement does not use the resources the task requires.
    ///
    /// 放置未使用该任务所要求的资源。
    RequirementMismatch,
    /// A resource is used beyond its capacity.
    ///
    /// 某资源的使用超出其容量。
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
///
/// 构建或评估计划时发生的失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleError {
    kind: ScheduleErrorKind,
    detail: Option<String>,
}

impl ScheduleError {
    /// Creates an error with the given category.
    ///
    /// 用给定类别创建一个错误。
    pub fn new(kind: ScheduleErrorKind) -> Self {
        Self { kind, detail: None }
    }

    /// Attaches a description of the values that caused the failure.
    ///
    /// 附加对该失败所涉数值的说明。
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Returns the category of this failure.
    ///
    /// 返回该失败的类别。
    pub fn kind(&self) -> ScheduleErrorKind {
        self.kind
    }

    /// Returns the detail text attached to this error, if any.
    ///
    /// 返回附加在该错误上的说明文本，若有。
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
