//! Errors reported while constructing a constellation.
//!
//! 构建星座时报告的错误。

use core::fmt;

/// The category of a constellation error.
///
/// 星座错误的类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ConstellationErrorKind {
    /// The number of satellites is not positive.
    ///
    /// 卫星数不为正。
    NoSatellites,
    /// The number of planes is not positive.
    ///
    /// 轨道面数不为正。
    NoPlanes,
    /// The number of satellites is not a multiple of the number of planes,
    /// so the satellites cannot be spread evenly.
    ///
    /// 卫星数不是轨道面数的整数倍，无法均匀分布。
    UnevenPlanes,
    /// An orbital parameter is outside the range the orbit layer accepts.
    ///
    /// 某轨道参数超出 orbit 层接受的范围。
    InvalidOrbit,
    /// A shell or satellite index refers to something that does not exist.
    ///
    /// 某个壳层或卫星索引指向不存在的东西。
    NotFound,
}

impl fmt::Display for ConstellationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::NoSatellites => "the constellation has no satellites",
            Self::NoPlanes => "the constellation has no orbital planes",
            Self::UnevenPlanes => "the satellite count is not a multiple of the plane count",
            Self::InvalidOrbit => "the orbital parameters are not usable",
            Self::NotFound => "the requested shell or satellite does not exist",
        };
        f.write_str(text)
    }
}

/// A failure while building a constellation.
///
/// 构建星座时的失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstellationError {
    kind: ConstellationErrorKind,
    detail: Option<String>,
}

impl ConstellationError {
    /// Creates an error with the given category.
    ///
    /// 用给定类别创建一个错误。
    pub fn new(kind: ConstellationErrorKind) -> Self {
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
    pub fn kind(&self) -> ConstellationErrorKind {
        self.kind
    }

    /// Returns the detail text attached to this error, if any.
    ///
    /// 返回附加在该错误上的说明文本，若有。
    ///
    /// 返回附加在该错误上的说明文本，若无则返回 `None`。
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for ConstellationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{}: {detail}", self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl core::error::Error for ConstellationError {}
