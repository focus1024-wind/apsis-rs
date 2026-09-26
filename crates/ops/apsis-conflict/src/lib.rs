//! Resource and time conflict detection.
//!
//! A schedule can be wrong in several ways, and this crate finds them. It
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
//! use apsis_resource::{Resource, ResourceId, ResourceSet, Window};
//! use apsis_schedule::Schedule;
//! use apsis_task::{Task, TaskId};
//! use apsis_conflict::{ConflictKind, detect};
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

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod conflict;
mod detect;

pub use conflict::{Conflict, ConflictKind, ConflictReport};
pub use detect::{Precedence, detect, detect_precedence, placements_of};

pub use apsis_resource::{ResourceId, ResourceSet};
pub use apsis_schedule::Schedule;
pub use apsis_task::{Task, TaskId};
