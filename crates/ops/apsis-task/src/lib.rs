//! Task modeling for satellite operations.
//!
//! A task is a unit of work that a scheduler places on a set of resources. It
//! carries everything a scheduler needs to decide whether and where to run
//! it: how long it takes, when it may run, which resources it needs and in
//! what amount, and what it is worth.
//!
//! A task requires all of its resources at the same time, for its whole
//! duration. That is the model an observation has: the satellite and the
//! ground station must both be available while the data is taken. A task that
//! needs resources in sequence is expressed as two tasks with a
//! precedence constraint, which the scheduling layer provides.
//!
//! Tasks are immutable once built. The state a scheduler assigns to one is
//! kept separately, in the schedule, so that the same task can be considered
//! for several placements without being modified.
//!
//! ```
//! use apsis_resource::ResourceId;
//! use apsis_task::{Requirement, Task, TaskId};
//!
//! // An observation of sixty seconds that needs a satellite and a station.
//! let task = Task::new(TaskId::new(0), 60.0)
//!     .with_window(100.0, 1_000.0)
//!     .with_priority(5.0)
//!     .try_require(ResourceId::new(0), 1.0)
//!     .expect("the satellite is not yet required")
//!     .try_require(ResourceId::new(1), 1.0)
//!     .expect("the station is not yet required");
//!
//! assert_eq!(task.duration, 60.0);
//! assert_eq!(task.requirements.len(), 2);
//! assert!(task.is_valid());
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod task;

pub use error::{TaskError, TaskErrorKind};
pub use task::{Requirement, Task, TaskId, TaskSet, TaskState, task_order};

pub use apsis_resource::ResourceId;
