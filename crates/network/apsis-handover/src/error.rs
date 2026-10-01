//! Errors reported while making a handover decision.
//!
//! 作出切换决策时报告的错误。

use core::fmt;

/// The category of a handover error.
///
/// 切换错误的类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum HandoverErrorKind {
    /// The candidate list is empty.
    ///
    /// 候选列表为空。
    NoCandidates,
    /// The same satellite appears more than once in the candidate list.
    ///
    /// 同一卫星在候选列表中出现多次。
    DuplicateSatellite,
    /// A quality value is outside its range or not finite.
    ///
    /// 某质量值超出取值范围或非有限。
    InvalidQuality,
    /// A policy parameter is not usable.
    ///
    /// 某策略参数不可用。
    InvalidPolicy,
}

impl fmt::Display for HandoverErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::NoCandidates => "there are no candidates to choose from",
            Self::DuplicateSatellite => "a satellite appears more than once",
            Self::InvalidQuality => "a quality value is not usable",
            Self::InvalidPolicy => "a policy parameter is not usable",
        };
        f.write_str(text)
    }
}

/// A failure while making a handover decision.
///
/// 作出切换决策时发生的失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoverError {
    kind: HandoverErrorKind,
    detail: Option<String>,
}

impl HandoverError {
    /// Creates an error with the given category.
    ///
    /// 用给定类别创建一个错误。
    pub fn new(kind: HandoverErrorKind) -> Self {
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
    pub fn kind(&self) -> HandoverErrorKind {
        self.kind
    }

    /// Returns the detail text attached to this error, if any.
    ///
    /// 返回附加在该错误上的说明文本，若有。
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for HandoverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{}: {detail}", self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl core::error::Error for HandoverError {}
