//! Errors reported while building or querying a topology.
//!
//! 构建或查询拓扑时报告的错误。

use core::fmt;

/// The category of a topology error.
///
/// 拓扑错误的类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TopologyErrorKind {
    /// A node index is outside the range of the graph.
    ///
    /// 节点下标超出该图的取值范围。
    NoSuchNode,
    /// An edge already exists between the two nodes.
    ///
    /// 两节点之间已存在该边。
    DuplicateEdge,
    /// No edge exists between the two nodes.
    ///
    /// 两节点之间不存在该边。
    NoSuchEdge,
    /// A cost is not finite or not positive.
    ///
    /// 代价非有限或非正。
    InvalidCost,
    /// The series is empty.
    ///
    /// 该序列为空。
    EmptySeries,
    /// The times in a series are not strictly increasing.
    ///
    /// 序列中的时刻非严格递增。
    UnorderedTimes,
    /// A time is not finite.
    ///
    /// 某时刻非有限。
    InvalidTime,
}

impl fmt::Display for TopologyErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::NoSuchNode => "no such node",
            Self::DuplicateEdge => "an edge already exists between these nodes",
            Self::NoSuchEdge => "no edge exists between these nodes",
            Self::InvalidCost => "the edge cost is not usable",
            Self::EmptySeries => "the topology series is empty",
            Self::UnorderedTimes => "the series times are not strictly increasing",
            Self::InvalidTime => "the time is not finite",
        };
        f.write_str(text)
    }
}

/// A failure while building or querying a topology.
///
/// 构建或查询拓扑时发生的失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopologyError {
    kind: TopologyErrorKind,
    detail: Option<String>,
}

impl TopologyError {
    /// Creates an error with the given category.
    ///
    /// 用给定类别创建一个错误。
    #[must_use]
    pub fn new(kind: TopologyErrorKind) -> Self {
        Self { kind, detail: None }
    }

    /// Attaches a description of the values that caused the failure.
    ///
    /// 附加对该失败所涉数值的说明。
    #[must_use]
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Returns the category of this failure.
    ///
    /// 返回该失败的类别。
    #[must_use]
    pub fn kind(&self) -> TopologyErrorKind {
        self.kind
    }

    /// Returns the detail text attached to this error, if any.
    ///
    /// 返回附加在该错误上的说明文本，若有。
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for TopologyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{}: {detail}", self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl core::error::Error for TopologyError {}
