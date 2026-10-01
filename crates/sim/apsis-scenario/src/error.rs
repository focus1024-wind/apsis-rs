//! Errors reported while building, loading, or validating a scenario.

use core::fmt;

/// The category of a scenario error.
///
/// 场景错误的类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ScenarioErrorKind {
    /// The scenario name is empty.
    ///
    /// 场景名称为空。
    EmptyName,
    /// A resource with the identifier already exists.
    ///
    /// 已存在具有该标识符的资源。
    DuplicateResource,
    /// A task with the identifier already exists.
    ///
    /// 已存在具有该标识符的任务。
    DuplicateTask,
    /// A requirement names a resource that is not in the scenario.
    ///
    /// 某需求指名的资源不在场景中。
    UnknownResource,
    /// A precedence constraint names a task that is not in the scenario.
    ///
    /// 某先后约束指名的任务不在场景中。
    UnknownTask,
    /// A resource in the scenario is itself not usable.
    ///
    /// 场景中的某资源自身不可用。
    InvalidResource,
    /// A task in the scenario is itself not usable.
    ///
    /// 场景中的某任务自身不可用。
    InvalidTask,
    /// A precedence constraint would close a cycle.
    ///
    /// 某先后约束将形成环。
    CyclicPrecedence,
    /// A time limit is negative or not finite.
    ///
    /// 某时限为负或非有限。
    InvalidTimeLimit,
    /// An epoch is negative or not finite.
    ///
    /// 某历元为负或非有限。
    InvalidEpoch,
    /// A line has too few arguments for its keyword.
    ///
    /// 某行的参数对其关键字而言太少。
    MissingArgument,
    /// A line has an argument that cannot be read as the expected type.
    ///
    /// 某行的某参数无法按预期类型读取。
    BadArgument,
    /// A line does not begin with a known keyword.
    ///
    /// 某行未以已知关键字开头。
    UnknownKeyword,
    /// A keyword appears before the scenario header, or a required header is
    /// missing.
    ///
    /// 某关键字出现在场景头部之前，或缺少必需的头部。
    MissingHeader,
    /// A field appears more than once where only one is allowed.
    ///
    /// 某字段在只允许出现一次处出现了多次。
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
///
/// 构建、加载或校验场景时发生的失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioError {
    kind: ScenarioErrorKind,
    detail: Option<String>,
    line: Option<usize>,
}

impl ScenarioError {
    /// Creates an error with the given category.
    ///
    /// 用给定类别创建一个错误。
    pub fn new(kind: ScenarioErrorKind) -> Self {
        Self {
            kind,
            detail: None,
            line: None,
        }
    }

    /// Attaches a description of the values that caused the failure.
    ///
    /// 附加对该失败所涉数值的说明。
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Attaches the one based line number a load failure came from.
    ///
    /// 附加加载失败所来自的、以 1 为基的行号。
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// Returns the category of this failure.
    ///
    /// 返回该失败的类别。
    pub fn kind(&self) -> ScenarioErrorKind {
        self.kind
    }

    /// Returns the detail text attached to this error, if any.
    ///
    /// 返回附加在该错误上的说明文本，若有。
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    /// Returns the line number this failure came from, if any.
    ///
    /// 返回该失败所来自的行号（若有）。
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
