//! Schedule representation and evaluation.
//!
//! A schedule is the answer a scheduler produces: for each task, when it runs
//! and on which resources. This crate holds that answer and evaluates it.
//! It does not decide the placements, which is the job of `apsis-scheduler`;
//! keeping the two apart lets a schedule be read from a file, produced by a
//! scheduler, or written by hand, and evaluated the same way.
//!
//! A placement records the task, its start time, and the resources it holds.
//! The resources are checked against the task's requirements when the
//! placement is made, so a schedule cannot describe a task running on a
//! resource it did not ask for.
//!
//! The evaluation reports what an operator asks of a finished plan: how many
//! tasks were placed, how much of each resource is used at its busiest, and
//! how much of the available time was actually spent. Those figures are what
//! a planner compares between candidate schedules.
//!
//! ```
//! use apsis_resource::ResourceId;
//! use apsis_schedule::Schedule;
//! use apsis_task::{Task, TaskId};
//!
//! let task = Task::new(TaskId::new(0), 60.0)
//!     .try_require(ResourceId::new(0), 1.0)
//!     .expect("the resource is not yet required");
//!
//! let mut schedule = Schedule::new();
//! schedule.place(&task, 100.0).expect("the task fits");
//! assert_eq!(schedule.len(), 1);
//! assert!(schedule.is_placed(TaskId::new(0)));
//! ```
//!
//! 调度的表示与评估。
//!
//! 调度是调度器所产生的答案：对每个任务，它何时运行以及在哪些资源上运行。本
//! crate 保存该答案并评估它。它不决定放置，那是 `apsis-scheduler` 的事；把二者
//! 分开，使一个调度可以从文件读取、由调度器产生，或手工书写，并以相同方式评估。
//!
//! 一处放置记录任务、其开始时间，以及它所持有的资源。放置时各资源会针对任务的需求
//! 检查，故调度无法描述一个运行在它并未请求的资源上的任务。
//!
//! 评估报告操作员对已完成计划所问的：放置了多少任务、每种资源在其最繁忙时被用去
//! 多少，以及可用时间实际花去多少。这些数字正是规划器在候选调度之间比较的。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod placement;
mod schedule;

pub use error::{ScheduleError, ScheduleErrorKind};
pub use placement::{Evaluation, Placement, ResourceUsage};
pub use schedule::Schedule;

pub use apsis_resource::ResourceId;
pub use apsis_task::{Task, TaskId};
