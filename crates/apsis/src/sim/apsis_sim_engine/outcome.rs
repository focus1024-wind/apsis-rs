//! What a run produced and what the engine currently holds.
//!
//! 一次运行所产生的结果以及引擎当前所持有的状态。

use core::fmt;

/// Why a run stopped.
///
/// The reason is reported rather than inferred from the event count, because
/// the same count can arise from an empty queue or from a limit, and a
/// caller that records the outcome of a run needs to distinguish them.
///
/// 一次运行停止的原因。
///
/// 该原因是被报告的，而不是由事件计数推断的，因为同样的计数可能来自
/// 空队列或某个限制，而记录运行结果的调用方需要区分二者。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StopReason {
    /// No events were left when the engine looked for the next one.
    ///
    /// 引擎寻找下一事件时已无事件剩余。
    QueueEmpty,
    /// The time of the next event is beyond the simulation time limit.
    ///
    /// 下一事件的时间超出仿真时限。
    TimeLimit,
    /// The handler set a flag asking the engine to stop.
    ///
    /// 处理器置位标志，要求引擎停止。
    Requested,
    /// The event count reached the configured limit.
    ///
    /// 事件计数达到所配置的上限。
    EventLimit,
}

impl StopReason {
    /// Returns `true` when the run ended because the model had no more work
    /// to do rather than because a limit intervened.
    ///
    /// 当运行因模型已无工作而结束、而非因某个限制介入时返回 `true`。
    #[must_use]
    pub const fn is_natural(self) -> bool {
        matches!(self, Self::QueueEmpty)
    }

    /// Returns a short name for the reason.
    ///
    /// 返回该原因的简称。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::QueueEmpty => "queue empty",
            Self::TimeLimit => "time limit",
            Self::Requested => "requested",
            Self::EventLimit => "event limit",
        }
    }
}

impl fmt::Display for StopReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// What a run produced.
///
/// The outcome records the counts and the clock, so a caller can record the
/// result of a run without reaching back into the engine.
///
/// 一次运行所产生的结果。
///
/// 结果记录各项计数与时钟，使调用方无需回访引擎即可记录一次运行的结果。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineOutcome {
    /// Why the run stopped.
    ///
    /// 该运行停止的原因。
    pub reason: StopReason,
    /// The number of events the run executed.
    ///
    /// 该运行所执行的事件数。
    pub executed: u64,
    /// The simulation time the run started from, in seconds.
    ///
    /// 该运行的起始仿真时刻，单位为秒。
    pub started_at: f64,
    /// The simulation time the run reached, in seconds.
    ///
    /// 该运行所到达的仿真时刻，单位为秒。
    pub ended_at: f64,
    /// The number of events still pending when the run stopped.
    ///
    /// 该运行停止时仍待处理的事件数。
    pub remaining: usize,
}

impl EngineOutcome {
    /// Returns the time the run covered, in seconds.
    ///
    /// 返回该运行所覆盖的时长，单位为秒。
    #[must_use]
    pub fn duration(&self) -> f64 {
        self.ended_at - self.started_at
    }

    /// Returns the mean number of events per second of simulation time.
    ///
    /// This measures how busy the model is rather than how long the run took
    /// on the wall clock. Returns `None` when the run covered no time.
    ///
    /// 返回每仿真秒的平均事件数。
    ///
    /// 它衡量模型的繁忙程度，而非该运行在挂钟上耗时多久。运行未覆盖
    /// 任何时间时返回 `None`。
    #[must_use]
    pub fn event_rate(&self) -> Option<f64> {
        let duration = self.duration();
        if duration > 0.0 {
            Some(self.executed as f64 / duration)
        } else {
            None
        }
    }

    /// Returns `true` when the run finished because the model had no more
    /// work rather than because a limit stopped it.
    ///
    /// 当运行因模型已无工作而结束、而非因某个限制而停止时返回 `true`。
    #[must_use]
    pub fn is_natural(&self) -> bool {
        self.reason.is_natural()
    }
}

impl fmt::Display for EngineOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} event(s) from {} s to {} s, stopped by {}",
            self.executed, self.started_at, self.ended_at, self.reason
        )
    }
}

/// What the engine currently holds.
///
/// 引擎当前所持有的状态。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineState {
    /// The current simulation time, in seconds.
    ///
    /// 当前仿真时刻，单位为秒。
    pub now: f64,
    /// The number of pending events.
    ///
    /// 待处理事件的数量。
    pub pending: usize,
    /// The number of events executed so far.
    ///
    /// 迄今已执行的事件数。
    pub executed: u64,
    /// The time of the next pending event, if there is one.
    ///
    /// 下一待处理事件的时间（若存在）。
    pub next_time: Option<f64>,
}

impl EngineState {
    /// Returns `true` when no events are pending.
    ///
    /// 当没有待处理事件时返回 `true`。
    #[must_use]
    pub const fn is_idle(&self) -> bool {
        self.pending == 0
    }
}
