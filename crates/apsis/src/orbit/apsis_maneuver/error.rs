//! Errors reported while building or applying a maneuver.
//!
//! 构建或施加机动时报告的错误。

use core::fmt;

/// The category of a maneuver error.
///
/// 机动错误的类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ManeuverErrorKind {
    /// A velocity change, acceleration, or duration is not finite.
    ///
    /// 速度变化、加速度或时长非有限。
    NotFinite,
    /// The duration of a burn is not positive.
    ///
    /// 点火时长不为正。
    NonPositiveDuration,
    /// The acceleration of a burn is not positive.
    ///
    /// 点火加速度不为正。
    NonPositiveAcceleration,
    /// A radius is not finite or not positive.
    ///
    /// 某半径非有限或不为正。
    InvalidRadius,
    /// The two radii of a transfer are equal, so there is nothing to do.
    ///
    /// 转移的两个半径相等，因而无事可做。
    DegenerateTransfer,
    /// A state is missing the data the operation needs.
    ///
    /// 某状态缺少该运算所需的数据。
    InvalidState,
}

impl fmt::Display for ManeuverErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::NotFinite => "a value is not finite",
            Self::NonPositiveDuration => "the duration is not positive",
            Self::NonPositiveAcceleration => "the acceleration is not positive",
            Self::InvalidRadius => "the radius is not usable",
            Self::DegenerateTransfer => "the two radii are equal",
            Self::InvalidState => "the state is not usable",
        };
        f.write_str(text)
    }
}

/// A failure while building or applying a maneuver.
///
/// 构建或应用机动时发生的失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManeuverError {
    kind: ManeuverErrorKind,
    detail: Option<String>,
}

impl ManeuverError {
    /// Creates an error with the given category.
    ///
    /// 用给定类别创建一个错误。
    #[must_use]
    pub fn new(kind: ManeuverErrorKind) -> Self {
        Self { kind, detail: None }
    }

    /// Attaches a description of the value that caused the failure.
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
    pub const fn kind(&self) -> ManeuverErrorKind {
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

impl fmt::Display for ManeuverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{}: {detail}", self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl core::error::Error for ManeuverError {}
