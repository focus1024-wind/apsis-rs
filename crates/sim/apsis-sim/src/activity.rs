//! What happens at an instant in a simulated timeline.

use apsis_task::TaskId;

/// One thing that happens in the timeline.
///
/// The timeline is derived from a schedule: a placed task contributes a start
/// and an end, and the handler is called at each. The two are separate events
/// rather than one interval so that the engine orders them with everything
/// else on the clock, which is what lets two tasks overlap correctly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Activity {
    /// A task begins.
    Start {
        /// The task beginning.
        task: TaskId,
    },
    /// A task ends.
    End {
        /// The task ending.
        task: TaskId,
    },
}

impl Activity {
    /// Creates a start activity.
    pub const fn start(task: TaskId) -> Self {
        Self::Start { task }
    }

    /// Creates an end activity.
    pub const fn end(task: TaskId) -> Self {
        Self::End { task }
    }

    /// Returns the task the activity is about.
    pub const fn task(self) -> TaskId {
        match self {
            Self::Start { task } | Self::End { task } => task,
        }
    }

    /// Returns `true` when the activity is a start.
    pub const fn is_start(self) -> bool {
        matches!(self, Self::Start { .. })
    }

    /// Returns `true` when the activity is an end.
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
