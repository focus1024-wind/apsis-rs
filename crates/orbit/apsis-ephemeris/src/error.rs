//! Errors reported while building or querying an ephemeris.

use core::fmt;

/// The category of an ephemeris error.
///
/// 星历错误的类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EphemerisErrorKind {
    /// The sampling grid has fewer than two samples.
    ///
    /// 采样网格少于两个样本。
    TooFewSamples,
    /// A time or a step is not finite.
    ///
    /// 某时刻或步长非有限。
    NotFinite,
    /// A step is zero or negative.
    ///
    /// 步长为零或为负。
    NonPositiveStep,
    /// The samples are not in increasing time order.
    ///
    /// 样本未按时间递增排列。
    NotMonotonic,
    /// A state produced by the sampler is not finite.
    ///
    /// 采样器产生的状态非有限。
    InvalidState,
    /// A query time falls outside the covered interval.
    ///
    /// 查询时刻落在覆盖区间之外。
    OutOfRange,
}

impl fmt::Display for EphemerisErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::TooFewSamples => "fewer than two samples",
            Self::NotFinite => "a time is not finite",
            Self::NonPositiveStep => "the step is not positive",
            Self::NotMonotonic => "the samples are not in increasing time order",
            Self::InvalidState => "the sampled state is not finite",
            Self::OutOfRange => "the time is outside the covered interval",
        };
        f.write_str(text)
    }
}

/// A failure while building or querying an ephemeris.
///
/// 构建或查询星历时发生的失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EphemerisError {
    kind: EphemerisErrorKind,
    detail: Option<String>,
}

impl EphemerisError {
    /// Creates an error with the given category.
    ///
    /// 用给定类别创建一个错误。
    pub fn new(kind: EphemerisErrorKind) -> Self {
        Self { kind, detail: None }
    }

    /// Attaches a description of the value that caused the failure.
    ///
    /// 附加对该失败所涉数值的说明。
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Returns the category of this failure.
    ///
    /// 返回该失败的类别。
    pub const fn kind(&self) -> EphemerisErrorKind {
        self.kind
    }

    /// Returns the detail text attached to this error, if any.
    ///
    /// 返回附加在该错误上的说明文本，若有。
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for EphemerisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{}: {detail}", self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl core::error::Error for EphemerisError {}
