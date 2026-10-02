//! Running a prepared simulation.
//!
//! 运行已准备的仿真。

use crate::activity::Activity;
use crate::simulation::Simulation;
use apsis_output::{RecordKind, Recorder, Trace};
use apsis_sim_engine::{Engine, EngineOutcome};

/// What a run produced.
///
/// The report pairs the engine's own outcome with the output trace, so a
/// caller has everything a run produced without reaching back into the
/// session.
///
/// 一次运行所产生的结果。
///
/// 该报告把引擎自身的结果与输出轨迹配对，使调用方无需回访会话即可得到
/// 一次运行所产生的全部内容。
#[derive(Debug, Clone, PartialEq)]
pub struct RunReport {
    /// What the engine did.
    ///
    /// 引擎所做的。
    pub outcome: EngineOutcome,
    /// The records the run made.
    ///
    /// 该运行所作出的记录。
    pub trace: Trace,
    /// The number of records the recorder refused because they were not
    /// usable.
    ///
    /// 记录器因不可用而拒绝的记录数。
    pub refused: u64,
}

impl RunReport {
    /// Returns the number of activities the run executed.
    ///
    /// 返回该运行所执行的活动数。
    #[must_use]
    pub const fn executed(&self) -> u64 {
        self.outcome.executed
    }

    /// Returns `true` when the run finished because the timeline was
    /// exhausted rather than because a limit stopped it.
    ///
    /// 当运行因时间线耗尽而结束、而非因某个限制而停止时返回 `true`。
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.outcome.is_natural()
    }

    /// Returns the number of warnings recorded.
    ///
    /// 返回所记录的警告数。
    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.trace.warning_count()
    }
}

/// One run of a prepared simulation.
///
/// A session owns the engine and the recorder for a single run, so it can be
/// driven from one place without the caller holding either. It is created by
/// [`Simulation::start`].
///
/// 一次已准备仿真的运行。
///
/// 会话为单次运行拥有引擎与记录器，故它可在一处被驱动而无需调用方持有
/// 二者中的任何一个。它由 [`Simulation::start`] 创建。
#[derive(Debug, Clone)]
pub struct Session {
    engine: Engine<Activity>,
    recorder: Recorder,
}

impl Session {
    /// Builds a session from a prepared simulation.
    ///
    /// 由已准备的仿真构建会话。
    pub(crate) fn new(simulation: &Simulation) -> Self {
        let scenario = simulation.scenario();
        let mut engine = Engine::new()
            .with_time_limit(scenario.time_limit().unwrap_or(f64::INFINITY))
            .with_event_limit(scenario.event_limit().unwrap_or(0));
        engine.set_now(scenario.epoch());
        // The timeline is relative to the scenario origin, so the epoch is
        // added to every event time. Without the offset the engine would
        // move the clock backwards when it delivered the first event.
        // 时间线相对于场景原点，故历元被加到每个事件时间上。若无该偏移，
        // 引擎在投递第一个事件时会把时钟往回移。
        let epoch = scenario.epoch();
        for (time, activity) in simulation.timeline() {
            engine.schedule(epoch + time, *activity);
        }
        Self {
            engine,
            recorder: Recorder::new(),
        }
    }

    /// Returns the engine, for a caller that needs its state between calls.
    ///
    /// 返回引擎，供需要在两次调用之间获取其状态的调用方使用。
    #[must_use]
    pub const fn engine(&self) -> &Engine<Activity> {
        &self.engine
    }

    /// Returns the engine for modification.
    ///
    /// 返回引擎以供修改。
    pub const fn engine_mut(&mut self) -> &mut Engine<Activity> {
        &mut self.engine
    }

    /// Returns the recorder, for a caller that reads the trace as it grows.
    ///
    /// 返回记录器，供在轨迹增长时读取它的调用方使用。
    #[must_use]
    pub const fn recorder(&self) -> &Recorder {
        &self.recorder
    }

    /// Returns the recorder for modification.
    ///
    /// 返回记录器以供修改。
    pub const fn recorder_mut(&mut self) -> &mut Recorder {
        &mut self.recorder
    }

    /// Returns the current simulation time, in seconds.
    ///
    /// 返回当前仿真时刻，单位为秒。
    #[must_use]
    pub const fn now(&self) -> f64 {
        self.engine.now()
    }

    /// Runs the engine, calling the handler at each activity.
    ///
    /// The handler receives the engine, the recorder, and the activity. It
    /// normally records something for the activity and may schedule further
    /// events; the engine delivers them in time order alongside the timeline.
    ///
    /// Returns a report holding the engine outcome and the trace made during
    /// this call. The recorder is left empty for the next call.
    ///
    /// 运行引擎，在每个活动处调用处理器。
    ///
    /// 处理器接收引擎、记录器与活动。它通常为该活动记录些什么，并可
    /// 再调度更多事件；引擎把它们与时间线一起按时间次序投递。
    ///
    /// 返回一个报告，其中含有引擎结果与本次调用期间所作出的轨迹。记录器
    /// 被留空以备下次调用。
    pub fn run<F>(&mut self, mut handler: F) -> RunReport
    where
        F: FnMut(&mut Engine<Activity>, &mut Recorder, apsis_event::Event<Activity>),
    {
        let outcome = self.engine.run(|engine, event| {
            handler(engine, &mut self.recorder, event);
        });
        RunReport {
            outcome,
            trace: self.recorder.take_trace(),
            refused: self.recorder.refused(),
        }
    }

    /// Records an activity as a state change without running anything.
    ///
    /// This is for a caller that drives the engine itself and only wants the
    /// recorder.
    ///
    /// 把某个活动记录为状态变更而不运行任何东西。
    ///
    /// 这适用于自行驱动引擎而只想要记录器的调用方。
    pub fn note(&mut self, time: f64, label: impl Into<String>, value: f64) -> bool {
        self.recorder.note(time, label, value)
    }

    /// Records a value of a given kind without running anything.
    ///
    /// 记录给定类别的一个值而不运行任何东西。
    pub fn record(
        &mut self,
        time: f64,
        kind: RecordKind,
        label: impl Into<String>,
        value: f64,
    ) -> bool {
        self.recorder.record(time, kind, label, value)
    }
}
