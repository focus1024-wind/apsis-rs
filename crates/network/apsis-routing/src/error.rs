//! Errors reported while routing.

use core::fmt;

/// The category of a routing error.
///
/// 路由错误的类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RoutingErrorKind {
    /// A node index is outside the range of the graph.
    ///
    /// 节点下标超出该图的取值范围。
    NoSuchNode,
    /// No route exists between the two nodes.
    ///
    /// 两节点之间不存在路由。
    NoRoute,
    /// The two endpoints are the same node.
    ///
    /// 两个端点是同一节点。
    SameEndpoint,
    /// The interval is empty or not ordered.
    ///
    /// 区间为空或次序不对。
    InvalidInterval,
    /// The topology series does not cover the requested interval.
    ///
    /// 拓扑序列未覆盖所请求的区间。
    SeriesTooShort,
}

impl fmt::Display for RoutingErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::NoSuchNode => "no such node",
            Self::NoRoute => "no route exists between these nodes",
            Self::SameEndpoint => "the source and destination are the same node",
            Self::InvalidInterval => "the interval is not usable",
            Self::SeriesTooShort => "the topology series does not cover the interval",
        };
        f.write_str(text)
    }
}

/// A failure while routing.
///
/// 路由时发生的失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingError {
    kind: RoutingErrorKind,
    detail: Option<String>,
}

impl RoutingError {
    /// Creates an error with the given category.
    ///
    /// 用给定类别创建一个错误。
    ///
    /// 用给定类别创建一个错误。
    pub fn new(kind: RoutingErrorKind) -> Self {
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
    pub fn kind(&self) -> RoutingErrorKind {
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

impl fmt::Display for RoutingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{}: {detail}", self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl core::error::Error for RoutingError {}
