//! Building a trace as a simulation runs.
//!
//! 在仿真运行过程中构建轨迹。

use crate::apsis_output::record::{Record, RecordKind};
use crate::apsis_output::trace::Trace;

/// Collects records as a simulation runs.
///
/// The recorder is the write side of a trace: a handler calls one of its
/// methods for each value it wants to keep, and asks for the trace when the
/// run finishes. The two are separate so that a handler can hold a recorder
/// while the trace is read elsewhere.
///
/// ```
/// use apsis::apsis_output::Recorder;
///
/// let mut recorder = Recorder::new();
/// recorder.observe(0.0, "range", 1_000.0);
/// recorder.observe(10.0, "range", 950.0);
///
/// let summary = recorder.trace().summarize("range").expect("the series exists");
/// assert_eq!(summary.count, 2);
/// assert_eq!(summary.change, -50.0);
/// ```
///
/// 在仿真运行过程中收集记录。
///
/// 记录器是轨迹的写入侧：处理器为其想保留的每个值调用它的某个方法，
/// 并在运行结束时索取轨迹。二者分开，使处理器可以持有记录器而轨迹
/// 在别处被读取。
///
/// ```
/// use apsis::apsis_output::Recorder;
///
/// let mut recorder = Recorder::new();
/// recorder.observe(0.0, "range", 1_000.0);
/// recorder.observe(10.0, "range", 950.0);
///
/// let summary = recorder.trace().summarize("range").expect("the series exists");
/// assert_eq!(summary.count, 2);
/// assert_eq!(summary.change, -50.0);
/// ```
#[derive(Debug, Clone, Default)]
pub struct Recorder {
    trace: Trace,
    refused: u64,
}

impl Recorder {
    /// Creates an empty recorder.
    ///
    /// 创建一个空记录器。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            trace: Trace::new(),
            refused: 0,
        }
    }

    /// Records a value of a given kind.
    ///
    /// Returns `false` when the record is not usable, which a caller can
    /// ignore when the value is known to be finite.
    ///
    /// 记录给定类别的一个值。
    ///
    /// 该记录不可用时返回 `false`；当值已知为有限时调用方可忽略。
    pub fn record(
        &mut self,
        time: f64,
        kind: RecordKind,
        label: impl Into<String>,
        value: f64,
    ) -> bool {
        let accepted = self.trace.push(Record::new(time, kind, label, value));
        if !accepted {
            self.refused = self.refused.wrapping_add(1);
        }
        accepted
    }

    /// Records an observation.
    ///
    /// 记录一次观测。
    pub fn observe(&mut self, time: f64, label: impl Into<String>, value: f64) -> bool {
        self.record(time, RecordKind::Observation, label, value)
    }

    /// Records a state change.
    ///
    /// 记录一次状态变更。
    pub fn note(&mut self, time: f64, label: impl Into<String>, value: f64) -> bool {
        self.record(time, RecordKind::State, label, value)
    }

    /// Records a warning.
    ///
    /// 记录一次警告。
    pub fn warn(&mut self, time: f64, label: impl Into<String>, value: f64) -> bool {
        self.record(time, RecordKind::Warning, label, value)
    }

    /// Returns the number of records made.
    ///
    /// 返回已作出的记录数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.trace.len()
    }

    /// Returns `true` when no records have been made.
    ///
    /// 当尚无记录时返回 `true`。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.trace.is_empty()
    }

    /// Returns the number of records refused because they were not usable.
    ///
    /// A caller that records values from a model checks this to see whether
    /// any value was dropped, rather than comparing counts by hand.
    ///
    /// 返回因不可用而被拒绝的记录数。
    ///
    /// 从模型记录值的调用方检查它即可看出是否有值被丢弃，而无需手工
    /// 比较计数。
    #[must_use]
    pub const fn refused(&self) -> u64 {
        self.refused
    }

    /// Returns the trace recorded so far.
    ///
    /// 返回迄今所记录的轨迹。
    #[must_use]
    pub const fn trace(&self) -> &Trace {
        &self.trace
    }

    /// Returns the trace, leaving an empty one behind.
    ///
    /// This is what a caller does between runs: the finished trace goes to
    /// whoever reads it, and the recorder starts fresh. The refused count is
    /// not reset, because it records the whole life of the recorder.
    ///
    /// 返回轨迹并留下一个空轨迹。
    ///
    /// 这是调用方在两次运行之间所做的：已完成的轨迹交给读取它的一方，
    /// 记录器重新开始。被拒绝的计数不重置，因为它记录记录器的整个
    /// 生命期。
    pub fn take_trace(&mut self) -> Trace {
        core::mem::take(&mut self.trace)
    }

    /// Removes every record without changing the refused count.
    ///
    /// 移除所有记录而不改变被拒绝的计数。
    pub fn clear(&mut self) -> usize {
        self.trace.clear()
    }
}
