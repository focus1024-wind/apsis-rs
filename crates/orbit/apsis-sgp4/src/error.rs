//! Errors reported by the propagator.
//!
//! 传播器报告的错误。

use core::fmt;

/// The category of a propagation failure.
///
/// 传播失败的类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PropagationErrorKind {
    /// The mean elements do not describe a satellite the model can
    /// propagate. This includes an eccentricity at or above one, which is
    /// not a closed orbit, and a mean motion that is not positive.
    ///
    /// 平均根数不描述模型可传播的卫星。这包括偏心率不小于一（非闭合轨道）
    /// 与平运动不为正的情形。
    InvalidElements,
    /// The satellite has decayed, meaning the drag term has brought it
    /// below the surface of the Earth.
    ///
    /// 卫星已衰减，即阻力项把它带到地表以下。
    Decayed,
    /// The propagation left the region where the model is defined.
    ///
    /// 传播超出了模型定义的范围。
    OutOfRange,
}

impl fmt::Display for PropagationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::InvalidElements => "mean elements are not propagatable",
            Self::Decayed => "satellite has decayed",
            Self::OutOfRange => "propagation is out of range",
        };
        f.write_str(text)
    }
}

/// A failure while initialising or propagating a satellite.
///
/// 初始化或传播卫星时发生的失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropagationError {
    kind: PropagationErrorKind,
    detail: Option<String>,
}

impl PropagationError {
    /// Creates an error with the given category.
    ///
    /// 用给定类别创建一个错误。
    pub fn new(kind: PropagationErrorKind) -> Self {
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
    pub fn kind(&self) -> PropagationErrorKind {
        self.kind
    }

    /// Returns the detail text attached to this error, if any.
    ///
    /// 返回附加在该错误上的说明文本，若有。
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for PropagationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{}: {detail}", self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl core::error::Error for PropagationError {}
