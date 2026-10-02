//! What happens at an instant in a simulated timeline.
//!
//! 仿真时间线中某瞬时发生的事。

use apsis_task::TaskId;

/// One thing that happens in the timeline.
///
/// The timeline is derived from a schedule: a placed task contributes a start
/// and an end, and the handler is called at each. The two are separate events
/// rather than one interval so that the engine orders them with everything
/// else on the clock, which is what lets two tasks overlap correctly.
///
/// 时间线上发生的一件事。
///
/// 时间线由一个调度导出：一处被放置的任务贡献一个开始与一个结束，
/// 处理器在每个处被调用。二者是分开的事件而非一个区间，使引擎把它们
/// 与时钟上其余一切一起排序，这正是让两个任务正确重叠的原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Activity {
    /// A task begins.
    ///
    /// 某任务开始。
    Start {
        /// The task beginning.
        ///
        /// 正在开始的任务。
        task: TaskId,
    },
    /// A task ends.
    ///
    /// 某任务结束。
    End {
        /// The task ending.
        ///
        /// 正在结束的任务。
        task: TaskId,
    },
}

impl Activity {
    /// Creates a start activity.
    ///
    /// 创建一个开始活动。
    #[must_use]
    pub const fn start(task: TaskId) -> Self {
        Self::Start { task }
    }

    /// Creates an end activity.
    ///
    /// 创建一个结束活动。
    #[must_use]
    pub const fn end(task: TaskId) -> Self {
        Self::End { task }
    }

    /// Returns the task the activity is about.
    ///
    /// 返回该活动所关于的任务。
    #[must_use]
    pub const fn task(self) -> TaskId {
        match self {
            Self::Start { task } | Self::End { task } => task,
        }
    }

    /// Returns `true` when the activity is a start.
    ///
    /// 当该活动为开始时返回 `true`。
    #[must_use]
    pub const fn is_start(self) -> bool {
        matches!(self, Self::Start { .. })
    }

    /// Returns `true` when the activity is an end.
    ///
    /// 当该活动为结束时返回 `true`。
    #[must_use]
    pub const fn is_end(self) -> bool {
        matches!(self, Self::End { .. })
    }
}

impl core::fmt::Display for Activity {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Start { task } => write!(f, "start {task}"),
            Self::End { task } => write!(f, "end {task}"),
        }
    }
}
