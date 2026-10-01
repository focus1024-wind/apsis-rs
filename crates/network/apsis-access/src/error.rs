//! Errors reported while finding access windows.

use core::fmt;

/// The category of an access error.
///
/// 接入错误的类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AccessErrorKind {
    /// The station or its mask is not usable.
    ///
    /// 地面站或其遮蔽角不可用。
    InvalidStation,
    /// The sampling step is not positive and finite.
    ///
    /// 采样步长非正或非有限。
    InvalidSampleStep,
    /// The crossing tolerance is not positive and finite.
    ///
    /// 交叉容差非正或非有限。
    InvalidTolerance,
    /// The search interval is empty or not ordered.
    ///
    /// 搜索区间为空或次序不对。
    InvalidInterval,
    /// The sample sequence is not in increasing time order.
    ///
    /// 样本序列未按时间递增排列。
    UnorderedSamples,
    /// A supplied satellite position is not usable.
    ///
    /// 所提供的卫星位置不可用。
    InvalidPosition,
}

impl fmt::Display for AccessErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::InvalidStation => "the ground station is not usable",
            Self::InvalidSampleStep => "the sampling step is not usable",
            Self::InvalidTolerance => "the crossing tolerance is not usable",
            Self::InvalidInterval => "the search interval is not usable",
            Self::UnorderedSamples => "the sample sequence is not ordered",
            Self::InvalidPosition => "a satellite position is not usable",
        };
        f.write_str(text)
    }
}

/// A failure while finding access windows.
///
/// 查找可见性窗口时发生的失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessError {
    kind: AccessErrorKind,
    detail: Option<String>,
}

impl AccessError {
    /// Creates an error with the given category.
    ///
    /// 用给定类别创建一个错误。
    pub fn new(kind: AccessErrorKind) -> Self {
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
    pub fn kind(&self) -> AccessErrorKind {
        self.kind
    }

    /// Returns the detail text attached to this error, if any.
    ///
    /// 返回附加在该错误上的说明文本，若有。
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for AccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{}: {detail}", self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl core::error::Error for AccessError {}
