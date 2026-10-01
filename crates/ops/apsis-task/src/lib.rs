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
//! needs resources in sequence is expressed as two tasks joined by a
//! precedence constraint in a [`PrecedenceGraph`], which says that one task
//! must finish before the other starts.
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
//!
//! 卫星运行的任务建模。
//!
//! 任务是调度器放置到一组资源上的工作单元。它携带调度器决定是否运行它、在哪里
//! 运行它所需的一切：耗时多久、何时可以运行、需要哪些资源及多少量，以及它值多少。
//!
//! 一个任务在其整个时长内同时需要它的全部资源。这是观测所具有的模型：取数期间
//! 卫星与地面站必须都可用。需要按序使用资源的任务，表达为两个由
//! [`PrecedenceGraph`] 中先后约束连接的任务，该约束说一个任务必须在另一个开始
//! 之前完成。
//!
//! 任务一旦构建便不可变。调度器赋给它的状态分开保存在调度中，使同一个任务可以
//! 被考虑用于多处放置而不被修改。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod precedence;
mod task;

pub use error::{TaskError, TaskErrorKind};
pub use precedence::PrecedenceGraph;
pub use task::{Requirement, Task, TaskId, TaskSet, TaskState, task_order};

pub use apsis_resource::ResourceId;
