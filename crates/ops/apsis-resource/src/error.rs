//! Errors reported while building or querying a resource.

use core::fmt;

/// The category of a resource error.
///
/// 资源错误的类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ResourceErrorKind {
    /// The end of an interval precedes its start.
    ///
    /// 区间的终点早于起点。
    ReversedInterval,
    /// A time or duration is not finite.
    ///
    /// 某时刻或时长非有限。
    NotFinite,
    /// A duration is negative.
    ///
    /// 时长为负。
    NegativeDuration,
    /// The capacity is not positive and finite.
    ///
    /// 容量非正或非有限。
    InvalidCapacity,
    /// No resource has the requested identifier.
    ///
    /// 没有资源具有所请求的标识符。
    NoSuchResource,
    /// A resource with the identifier already exists.
    ///
    /// 具有该标识符的资源已存在。
    DuplicateResource,
}

impl fmt::Display for ResourceErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::ReversedInterval => "the interval ends before it starts",
            Self::NotFinite => "a time is not finite",
            Self::NegativeDuration => "the duration is negative",
            Self::InvalidCapacity => "the capacity is not usable",
            Self::NoSuchResource => "no such resource",
            Self::DuplicateResource => "a resource with that identifier exists",
        };
        f.write_str(text)
    }
}

/// A failure while building or querying a resource.
///
/// 构建或查询资源时发生的失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceError {
    kind: ResourceErrorKind,
    detail: Option<String>,
}

impl ResourceError {
    /// Creates an error with the given category.
    ///
    /// 用给定类别创建一个错误。
    ///
    /// 用给定类别创建一个错误。
    pub fn new(kind: ResourceErrorKind) -> Self {
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
    pub fn kind(&self) -> ResourceErrorKind {
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

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{}: {detail}", self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl core::error::Error for ResourceError {}
