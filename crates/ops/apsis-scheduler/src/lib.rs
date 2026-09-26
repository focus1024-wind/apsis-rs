//! Scheduling algorithms for satellite task planning.
//!
//! A scheduler takes a list of tasks and a set of resources and produces a
//! schedule. This crate provides a greedy scheduler and the pieces needed to
//! build others.
//!
//! Scheduling with resource constraints is intractable in general: deciding
//! whether every task fits is a bin packing problem, which is NP-hard. A
//! satellite planning problem is small, with hundreds of tasks rather than
//! millions, but an exact search is still impractical for a routine planning
//! cycle. The scheduler here is therefore a greedy heuristic.
//!
//! The algorithm sorts the tasks and places each in the earliest time at
//! which all of its resources are free together. Tasks are considered in
//! decreasing order of weighted value, which is a good heuristic: the most
//! valuable work is placed first and the least valuable fills the remaining
//! gaps. Mandatory tasks are placed first regardless of their value, because
//! a task that must run is worth more than any task that may.
//!
//! The result is not guaranteed to be optimal, and the crate reports what it
//! could not place rather than dropping it silently, so that an operator can
//! see the cost of the plan.
//!
//! ```
//! use apsis_resource::{Resource, ResourceId, ResourceSet, Window};
//! use apsis_scheduler::Planner;
//! use apsis_task::{Task, TaskId};
//!
//! let mut station = Resource::new(ResourceId::new(0), 1.0);
//! station.set_windows(&[Window::new(0.0, 1_000.0)]).expect("usable");
//! let resources = ResourceSet::from_resources(vec![station]).expect("distinct");
//!
//! let tasks = vec![Task::new(TaskId::new(0), 60.0)
//!     .try_require(ResourceId::new(0), 1.0)
//!     .expect("the resource is new")];
//!
//! let planner = Planner::new();
//! let result = planner.schedule(&tasks, &resources);
//! assert_eq!(result.schedule.len(), 1);
//! assert!(result.is_complete());
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod greedy;
mod planner;

pub use greedy::{
    demand_at, earliest_feasible_start, feasible_at, infeasible_tasks, resources_free_at,
    resources_have_capacity,
};
pub use planner::{PlanOutcome, Planner, SchedulingResult};

pub use apsis_resource::{ResourceId, ResourceSet};
pub use apsis_schedule::Schedule;
pub use apsis_task::{Task, TaskId};
