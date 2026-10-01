//! One recorded value.

use core::fmt;

/// What a record is about.
///
/// The three kinds separate the things a caller usually wants to tell apart
/// when it reads a trace back: a value the model measured, a decision it
/// made, and a problem it wants to surface.
///
/// 一条记录所关于的内容。
///
/// 三种类别区分了调用方回读轨迹时通常想分辨的事物：模型测得的值、模型
/// 作出的决策，以及它想揭示的问题。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RecordKind {
    /// A value the model measured, such as a range or a rate.
    ///
    /// 模型测得的值，如距离或速率。
    Observation,
    /// A change the model decided on, such as entering eclipse.
    ///
    /// 模型决定的变更，如进入地影。
    State,
    /// A condition worth surfacing, such as a value outside its bounds.
    ///
    /// 值得揭示的情形，如某值超出其范围。
    Warning,
}

impl RecordKind {
    /// Returns a short name for the kind.
    ///
    /// 返回该类别的简称。
    pub const fn name(self) -> &'static str {
        match self {
            Self::Observation => "observation",
            Self::State => "state",
            Self::Warning => "warning",
        }
    }

    /// Returns `true` when the kind is a warning.
    ///
    /// 当该类别为警告时返回 `true`。
    pub const fn is_warning(self) -> bool {
        matches!(self, Self::Warning)
    }
}

impl fmt::Display for RecordKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// One recorded value.
///
/// A record is small and copied by value, so a trace holds a flat list rather
/// than a tree of references.
///
/// 一个被记录的值。
///
/// 记录很小且按值复制，故轨迹保存的是扁平列表而非引用树。
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    /// The simulation time the record was made at, in seconds.
    ///
    /// 作出该记录时的仿真时刻，单位为秒。
    pub time: f64,
    /// What the record is about.
    ///
    /// 该记录所关于的内容。
    pub kind: RecordKind,
    /// What the value is about.
    ///
    /// 该值所关于的内容。
    pub label: String,
    /// The value recorded.
    ///
    /// 所记录的值。
    pub value: f64,
}

impl Record {
    /// Creates a record.
    ///
    /// 创建一条记录。
    pub fn new(time: f64, kind: RecordKind, label: impl Into<String>, value: f64) -> Self {
        Self {
            time,
            kind,
            label: label.into(),
            value,
        }
    }

    /// Creates an observation record.
    ///
    /// 创建一条观测记录。
    pub fn observation(time: f64, label: impl Into<String>, value: f64) -> Self {
        Self::new(time, RecordKind::Observation, label, value)
    }

    /// Creates a state record.
    ///
    /// 创建一条状态记录。
    pub fn state(time: f64, label: impl Into<String>, value: f64) -> Self {
        Self::new(time, RecordKind::State, label, value)
    }

    /// Creates a warning record.
    ///
    /// 创建一条警告记录。
    pub fn warning(time: f64, label: impl Into<String>, value: f64) -> Self {
        Self::new(time, RecordKind::Warning, label, value)
    }

    /// Returns `true` when the record is usable.
    ///
    /// A record whose time or value is not finite has no place in a trace,
    /// because it cannot be ordered or summarized.
    ///
    /// 当该记录可用时返回 `true`。
    ///
    /// 时间或值非有限的记录在轨迹中没有位置，因为它无法被排序或汇总。
    pub fn is_valid(&self) -> bool {
        self.time.is_finite() && self.value.is_finite() && !self.label.is_empty()
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} = {}",
            self.time, self.kind, self.label, self.value
        )
    }
}
