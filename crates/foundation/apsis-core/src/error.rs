//! The workspace error type and result alias.
//!
//! The workspace does not use an external error crate. Errors are modelled
//! as a single [`Error`] struct carrying an [`ErrorKind`], an optional
//! message, and an optional source. `ErrorKind` is a fieldless enum so that
//! callers can match on the failure category without allocating.
//!
//! 工作区不使用外部错误 crate。错误统一建模为单个 [`Error`] 结构体，携带
//! [`ErrorKind`]、可选消息与可选来源。`ErrorKind` 是无字段枚举，因此调用方可以
//! 在不分配内存的前提下匹配失败类别。

use core::error::Error as StdError;
use core::fmt;

/// The category of an [`Error`].
///
/// The variants are grouped by the layer that most often produces them, but
/// any layer may return any variant. Callers should match on the variant
/// that is relevant to their own recovery logic and treat the rest as
/// opaque.
///
/// [`Error`] 的类别。
///
/// 各变体按最常产生它们的层分组，但任何层都可以返回任何变体。调用方应只匹配
/// 与自身恢复逻辑相关的变体，其余视为不透明。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ErrorKind {
    /// An input value is outside the range the function accepts.
    ///
    /// 输入值超出该函数接受的范围。
    OutOfRange,
    /// An input does not satisfy a documented precondition.
    ///
    /// 输入不满足文档中说明的前置条件。
    InvalidInput,
    /// An operation is not defined for the given inputs, such as a
    /// singularity in a coordinate transformation.
    ///
    /// 对给定输入该运算未定义，例如坐标变换中的奇点。
    Undefined,
    /// A requested item does not exist.
    ///
    /// 请求的项不存在。
    NotFound,
    /// Two inputs are inconsistent with each other.
    ///
    /// 两个输入彼此矛盾。
    Inconsistent,
    /// An operation did not converge within the allowed number of steps
    ///
    /// 该运算未在允许的步数内收敛.
    NotConverged,
    /// Text could not be parsed into the expected structure.
    ///
    /// 文本无法解析为预期的结构。
    Parse,
    /// A numeric operation produced a non-finite result.
    ///
    /// 某个数值运算产生了非有限结果。
    NotFinite,
    /// A feature or capability is not implemented.
    ///
    /// 某项特性或能力未实现。
    Unsupported,
    /// A failure that does not fit any other category.
    ///
    /// 不属于任何其他类别的失败。
    Other,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::OutOfRange => "value out of range",
            Self::InvalidInput => "invalid input",
            Self::Undefined => "operation undefined for these inputs",
            Self::NotFound => "not found",
            Self::Inconsistent => "inconsistent inputs",
            Self::NotConverged => "did not converge",
            Self::Parse => "parse error",
            Self::NotFinite => "non-finite result",
            Self::Unsupported => "unsupported",
            Self::Other => "error",
        };
        f.write_str(text)
    }
}

/// The error type used across the workspace.
///
/// `Error` carries a machine matchable [`ErrorKind`] plus a human readable
/// message. An optional source records the lower level error that caused
/// this one.
///
/// 工作区使用的错误类型。
///
/// `Error` 携带可供机器匹配的 [`ErrorKind`] 与一条供人阅读的消息。可选的
/// source 记录导致该错误的底层错误。
pub struct Error {
    kind: ErrorKind,
    message: Option<String>,
    source: Option<Box<dyn StdError + Send + Sync + 'static>>,
}

impl Error {
    /// Creates an error with the given kind and no message.
    ///
    /// 用给定类别创建一个不带消息的错误。
    #[must_use]
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            message: None,
            source: None,
        }
    }

    /// Creates an error with the given kind and message.
    ///
    /// The message should describe the concrete cause of this specific
    /// failure. It is meant for humans reading logs, not for matching.
    ///
    /// 用给定类别与消息创建一个错误。
    ///
    /// 该消息应描述这一具体失败的具体原因。它意在供人阅读日志，而非供匹配。
    pub fn with_message(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: Some(message.into()),
            source: None,
        }
    }

    /// Creates an error with the given kind and a source error.
    ///
    /// 用给定类别与来源错误创建一个错误。
    pub fn with_source<E>(kind: ErrorKind, source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        Self {
            kind,
            message: None,
            source: Some(Box::new(source)),
        }
    }

    /// Attaches a message to this error, replacing any existing message.
    ///
    /// 为该错误附加消息，替换已有消息。
    #[must_use]
    pub fn with_context(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Attaches a source error, replacing any existing source.
    ///
    /// 为该错误附加来源错误，替换已有来源。
    #[must_use]
    pub fn caused_by<E>(mut self, source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        self.source = Some(Box::new(source));
        self
    }

    /// Returns the category of this error.
    ///
    /// 返回该错误的类别。
    #[must_use]
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Returns the message attached to this error, if any.
    ///
    /// 返回附加在该错误上的消息，若无则返回 `None`。
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Error");
        debug.field("kind", &self.kind);
        if let Some(message) = &self.message {
            debug.field("message", message);
        }
        if let Some(source) = &self.source {
            debug.field("source", source);
        }
        debug.finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.message {
            Some(message) => write!(f, "{}: {message}", self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn StdError + 'static))
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self::new(kind)
    }
}

/// A [`Result`](core::result::Result) whose error type is [`Error`]
///
/// 错误类型为 [`Error`] 的 [`Result`](core::result::Result).
pub type Result<T> = core::result::Result<T, Error>;
