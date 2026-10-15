//! Resource and time conflict detection.
//!
//! A schedule can be wrong in several ways, and this module finds them. It
//! reports conflicts rather than refusing them, so that a plan loaded from a
//! file can be checked in full and every problem listed at once instead of
//! one per attempt.
//!
//! Five kinds of conflict are detected:
//!
//! - **Capacity**: two placements need more of a resource at the same instant
//!   than the resource has.
//! - **Window**: a placement starts before its task may, or ends after it
//!   must.
//! - **Requirement**: a placement's task is not known, or its duration does
//!   not match the schedule.
//! - **Availability**: a placement falls outside the availability windows of
//!   a resource it needs, so the resource would not actually be there.
//! - **Precedence**: a task that must follow another starts before it ends.
//!
//! The distinction between a capacity conflict and an availability conflict
//! matters in practice: the first says the plan is over committed, the second
//! says the plan asks for a resource at a time when it is switched off or out
//! of view, which is a different repair.
//!
//! ```
//! use apsis::apsis_resource::{Resource, ResourceId, ResourceSet, Window};
//! use apsis::apsis_schedule::Schedule;
//! use apsis::apsis_task::{Task, TaskId};
//! use apsis::apsis_conflict::{ConflictKind, detect};
//!
//! let mut station = Resource::new(ResourceId::new(0), 1.0);
//! station.set_windows(&[Window::new(0.0, 1_000.0)]).expect("usable");
//! let resources = ResourceSet::from_resources(vec![station]).expect("distinct");
//!
//! let task = Task::new(TaskId::new(0), 60.0)
//!     .try_require(ResourceId::new(0), 1.0)
//!     .expect("the resource is new");
//! let mut schedule = Schedule::new();
//! schedule.place(&task, 100.0).expect("the task fits");
//!
//! let report = detect(&schedule, &[task], &resources);
//! assert!(report.is_clear());
//! assert_eq!(report.conflicts().len(), 0);
//! let _ = ConflictKind::Capacity;
//! ```
//!
//! 资源与时间冲突检测。
//!
//! 一份调度可能以若干方式出错，本模块找出它们。它报告冲突而非拒绝它们，使从
//! 文件加载的计划能被完整检查，并一次列出每个问题，而不是每次尝试报一个。
//!
//! 检测五类冲突：
//!
//! - **容量**：两处放置在同一瞬时需要的某资源超过该资源所有。
//! - **窗口**：一处放置在任务可开始之前开始，或在它必须结束之后结束。
//! - **需求**：一处放置的任务未知，或其时长与调度不符。
//! - **可用性**：一处放置落在它所需某资源的可用性窗口之外，故该资源实际上不在场。
//! - **先后**：一个必须跟随另一任务的任务在其结束之前开始。
//!
//! 容量冲突与可用性冲突之别在实践中要紧：前者说明计划超额承诺，后者说明计划在资源
//! 被关闭或不可见之时索求它，二者是不同的修复。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod conflict;
mod detect;

pub use conflict::{Conflict, ConflictKind, ConflictReport};
pub use detect::{Precedence, detect, detect_precedence, placements_of};

pub use crate::apsis_resource::{ResourceId, ResourceSet};
pub use crate::apsis_schedule::Schedule;
pub use crate::apsis_task::{Task, TaskId};
