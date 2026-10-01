//! Errors reported while building or querying a task.

use core::fmt;

/// The category of a task error.
///
/// 任务错误的类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TaskErrorKind {
    /// The duration is not finite, or is negative.
    ///
    /// 时长非有限或为负。
    InvalidDuration,
    /// A time window is not finite or ends before it starts.
    ///
    /// 时间窗口非有限或终点早于起点。
    InvalidWindow,
    /// The window is shorter than the task duration.
    ///
    /// 窗口短于任务时长。
    WindowTooShort,
    /// A resource demand is not positive and finite.
    ///
    /// 资源需求非正或非有限。
    InvalidDemand,
    /// The same resource is required more than once.
    ///
    /// 同一资源被重复要求。
    DuplicateRequirement,
    /// A priority or value is not finite.
    ///
    /// 优先级或某个数值非有限。
    InvalidWeight,
    /// No task has the requested identifier.
    ///
    /// 没有任务具有所请求的标识符。
    NoSuchTask,
    /// A task with the identifier already exists.
    ///
    /// 具有该标识符的任务已存在。
    DuplicateTask,
    /// A precedence constraint would close a cycle.
    ///
    /// 某优先约束会闭成环。
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
///
/// 构建或查询任务时发生的失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskError {
    kind: TaskErrorKind,
    detail: Option<String>,
}

impl TaskError {
    /// Creates an error with the given category.
    ///
    /// 用给定类别创建一个错误。
    ///
    /// 用给定类别创建一个错误。
    pub fn new(kind: TaskErrorKind) -> Self {
        Self { kind, detail: None }
    }

    /// Attaches a description of the values that caused the failure.
    ///
    /// 附加对该失败所涉数值的说明。
    ///
    /// 附加对该失败所涉数值的说明。
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Returns the category of this failure.
    ///
    /// 返回该失败的类别。
    ///
    /// 返回该失败的类别。
    pub fn kind(&self) -> TaskErrorKind {
        self.kind
    }

    /// Returns the detail text attached to this error, if any.
    ///
    /// 返回附加在该错误上的说明文本，若有。
    ///
    /// 返回附加在该错误上的说明文本，若有。
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
