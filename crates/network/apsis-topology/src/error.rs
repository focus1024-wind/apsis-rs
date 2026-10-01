//! Errors reported while building or querying a topology.

use core::fmt;

/// The category of a topology error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TopologyErrorKind {
    /// A node index is outside the range of the graph.
    NoSuchNode,
    /// An edge already exists between the two nodes.
    DuplicateEdge,
    /// No edge exists between the two nodes.
    NoSuchEdge,
    /// A cost is not finite or not positive.
    InvalidCost,
    /// The series is empty.
    EmptySeries,
    /// The times in a series are not strictly increasing.
    UnorderedTimes,
    /// A time is not finite.
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopologyError {
    kind: TopologyErrorKind,
    detail: Option<String>,
}

impl TopologyError {
    /// Creates an error with the given category.
    ///
    /// 用给定类别创建一个错误。
    pub fn new(kind: TopologyErrorKind) -> Self {
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
    pub fn kind(&self) -> TopologyErrorKind {
        self.kind
    }

    /// Returns the detail text attached to this error, if any.
    ///
    /// 返回附加在该错误上的说明文本，若有。
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
