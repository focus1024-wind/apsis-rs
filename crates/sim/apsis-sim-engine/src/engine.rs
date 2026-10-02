//! The engine itself.
//!
//! 引擎本身。

use crate::outcome::{EngineOutcome, EngineState, StopReason};
use apsis_event::{Event, EventId, EventQueue};

/// A discrete-event simulation engine.
///
/// The engine owns the event queue and the clock. A run repeatedly takes the
/// earliest event and passes it to a handler, which may schedule more
/// events. The engine stops when the queue empties or when one of the limits
/// is reached.
///
/// 离散事件仿真引擎。
///
/// 引擎拥有事件队列与时钟。一次运行反复取出最早的事件并交给处理器，
/// 处理器可再调度更多事件。队列排空或达到某个限制时引擎停止。
#[derive(Debug, Clone)]
pub struct Engine<T> {
    queue: EventQueue<T>,
    now: f64,
    executed: u64,
    stop_requested: bool,
    time_limit: Option<f64>,
    event_limit: Option<u64>,
}

impl<T> Default for Engine<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Engine<T> {
    /// Creates an engine with the clock at zero and no limits.
    ///
    /// An engine with no limits runs until the queue empties, which is the
    /// right default for a model that terminates. A model that schedules
    /// without bound needs a limit, which is set with
    /// [`Engine::with_time_limit`] or [`Engine::with_event_limit`].
    ///
    /// 创建时钟为零且无限制的引擎。
    ///
    /// 无限制的引擎一直运行到队列排空，这是会终止的模型的恰当缺省。
    /// 无界地调度事件的模型需要限制，可用 [`Engine::with_time_limit`]
    /// 或 [`Engine::with_event_limit`] 设置。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            queue: EventQueue::new(),
            now: 0.0,
            executed: 0,
            stop_requested: false,
            time_limit: None,
            event_limit: None,
        }
    }

    /// Sets the latest simulation time the engine will process.
    ///
    /// An event scheduled beyond the limit is left in the queue rather than
    /// executed, so the caller can examine what remained. A limit that is
    /// negative or not finite disables the limit.
    ///
    /// 设置引擎将处理的最晚仿真时刻。
    ///
    /// 调度到超出该限制的事件被留在队列中而不执行，使调用方能检查
    /// 剩余了哪些。限制为负或非有限时该限制被禁用。
    #[must_use]
    pub const fn with_time_limit(mut self, limit: f64) -> Self {
        self.time_limit = if limit.is_finite() && limit >= 0.0 {
            Some(limit)
        } else {
            None
        };
        self
    }

    /// Sets the greatest number of events the engine will execute.
    ///
    /// A limit of zero disables the rule, because a run that executes no
    /// events is never useful.
    ///
    /// 设置引擎将执行的事件数上限。
    ///
    /// 限制为零时该规则被禁用，因为不执行任何事件的运行从无用处。
    #[must_use]
    pub const fn with_event_limit(mut self, limit: u64) -> Self {
        self.event_limit = if limit == 0 { None } else { Some(limit) };
        self
    }

    /// Returns the current simulation time, in seconds.
    ///
    /// 返回当前仿真时刻，单位为秒。
    #[must_use]
    pub const fn now(&self) -> f64 {
        self.now
    }

    /// Sets the current simulation time.
    ///
    /// This is used to start a run at an offset, for example at a scenario
    /// epoch rather than at zero. It does not reschedule pending events, so a
    /// caller that moves the clock forward past a pending event will execute
    /// that event late. The engine has no way to detect the mistake, so the
    /// caller is responsible for keeping the clock consistent with the
    /// queue.
    ///
    /// 设置当前仿真时刻。
    ///
    /// 用于从某个偏移开始运行，例如从场景历元而非从零开始。它不重排
    /// 待处理事件，故把时钟前移越过某待处理事件的调用方会迟执行该
    /// 事件。引擎无法检测这一错误，故保持时钟与队列一致是调用方的
    /// 责任。
    pub fn set_now(&mut self, time: f64) {
        if time.is_finite() && time >= 0.0 {
            self.now = time;
        }
    }

    /// Returns the number of events executed so far.
    ///
    /// 返回迄今已执行的事件数。
    #[must_use]
    pub const fn executed(&self) -> u64 {
        self.executed
    }

    /// Returns the number of pending events.
    ///
    /// 返回待处理事件的数量。
    #[must_use]
    pub fn pending(&self) -> usize {
        self.queue.len()
    }

    /// Returns `true` when no events are pending.
    ///
    /// 当没有待处理事件时返回 `true`。
    #[must_use]
    pub fn is_idle(&self) -> bool {
        self.queue.is_empty()
    }

    /// Returns the time of the next pending event, if there is one.
    ///
    /// 返回下一待处理事件的时间（若存在）。
    #[must_use]
    pub fn next_time(&self) -> Option<f64> {
        self.queue.next_time()
    }

    /// Returns what the engine currently holds.
    ///
    /// 返回引擎当前所持有的状态。
    #[must_use]
    pub fn state(&self) -> EngineState {
        EngineState {
            now: self.now,
            pending: self.queue.len(),
            executed: self.executed,
            next_time: self.queue.next_time(),
        }
    }

    /// Schedules an event at a time.
    ///
    /// Returns the identifier of the new event, or `None` when the time is
    /// not usable.
    ///
    /// 在某个时刻调度一个事件。
    ///
    /// 返回新事件的标识符；该时刻不可用时返回 `None`。
    pub fn schedule(&mut self, time: f64, payload: T) -> Option<EventId> {
        self.queue.schedule(time, payload)
    }

    /// Schedules an event at a delay from the current simulation time.
    ///
    /// Returns the identifier of the new event, or `None` when the delay is
    /// not usable.
    ///
    /// 在当前仿真时刻之后某个延时处调度一个事件。
    ///
    /// 返回新事件的标识符；该延时不可用时返回 `None`。
    pub fn schedule_after(&mut self, delay: f64, payload: T) -> Option<EventId> {
        self.queue.schedule_after(self.now, delay, payload)
    }

    /// Asks the engine to stop after the current event.
    ///
    /// The handler calls this when its own model says the run is over, for
    /// example when a stopping condition that the queue cannot express is
    /// met. The pending events are left in the queue.
    ///
    /// 请求引擎在当前事件之后停止。
    ///
    /// 当处理器自身的模型表明运行已结束时调用它，例如当某个队列无法
    /// 表达的停止条件被满足时。待处理事件被留在队列中。
    pub fn request_stop(&mut self) {
        self.stop_requested = true;
    }

    /// Clears the stop request, so that a subsequent run continues.
    ///
    /// 清除停止请求，使后续运行得以继续。
    pub fn clear_stop(&mut self) {
        self.stop_requested = false;
    }

    /// Removes every pending event and returns how many were discarded.
    ///
    /// 移除所有待处理事件并返回被丢弃的数量。
    pub fn clear(&mut self) -> usize {
        self.queue.clear()
    }

    /// Returns the queue, for a caller that needs to inspect or manipulate it
    /// directly.
    ///
    /// 返回事件队列，供需要直接查看或操作它的调用方使用。
    #[must_use]
    pub const fn queue(&self) -> &EventQueue<T> {
        &self.queue
    }

    /// Returns the queue for modification.
    ///
    /// 返回事件队列以供修改。
    pub const fn queue_mut(&mut self) -> &mut EventQueue<T> {
        &mut self.queue
    }

    /// Runs the engine until it stops.
    ///
    /// Each iteration takes the earliest event, advances the clock to its
    /// time, and passes it to the handler. The handler may schedule further
    /// events, including at the same instant, which are delivered after the
    /// events already scheduled for that instant.
    ///
    /// Returns the outcome of the run. The clock is left at the time of the
    /// last event executed, not at a limit, so a caller that records the
    /// clock sees the last instant the model actually reached.
    ///
    /// 运行引擎直到其停止。
    ///
    /// 每次迭代取出最早的事件，把时钟推进到其时刻，并交给处理器。处理器
    /// 可再调度更多事件，包括在同一瞬时的，它们在该瞬时已调度的事件
    /// 之后投递。
    ///
    /// 返回该运行的结果。时钟留在最后执行事件的时间，而非某个限制处，
    /// 故记录时钟的调用方看到的是模型实际到达的最后瞬时。
    pub fn run<F>(&mut self, mut handler: F) -> EngineOutcome
    where
        F: FnMut(&mut Engine<T>, Event<T>),
    {
        let started_at = self.now;
        let mut executed_this_run = 0_u64;
        self.stop_requested = false;

        let reason = loop {
            // The event limit is checked before taking an event, so a run
            // that reaches the limit leaves the queue untouched for the next
            // call.
            // 事件限制在取出事件之前检查，故达到限制的运行会把队列原样
            // 留待下次调用。
            if let Some(limit) = self.event_limit {
                if self.executed >= limit {
                    break StopReason::EventLimit;
                }
            }

            let Some(event) = self.queue.pop() else {
                break StopReason::QueueEmpty;
            };

            // The time limit leaves the event in the queue rather than
            // discarding it, so the caller can inspect what remained. The
            // event was already popped, so it is put back.
            // 时限把事件留在队列中而非丢弃，使调用方能检查剩余了哪些。
            // 该事件已被弹出，故将其放回。
            if let Some(limit) = self.time_limit {
                if event.time > limit {
                    self.queue.schedule(event.time, event.payload);
                    break StopReason::TimeLimit;
                }
            }

            self.now = event.time;
            self.executed = self.executed.wrapping_add(1);
            executed_this_run += 1;

            handler(self, event);

            if self.stop_requested {
                break StopReason::Requested;
            }
        };

        EngineOutcome {
            reason,
            executed: executed_this_run,
            started_at,
            ended_at: self.now,
            remaining: self.queue.len(),
        }
    }
}
