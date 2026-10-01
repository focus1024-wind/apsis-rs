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
//!
//! 卫星任务规划的调度算法。
//!
//! 调度器接收任务列表与资源集合，产生一个调度。本 crate 提供一个贪心调度器以及
//! 构建其他调度器所需的构件。
//!
//! 带资源约束的调度一般是难解的：判定是否每个任务都放得下是一个装箱问题，它是
//! NP 难的。卫星规划问题规模小，有数百个任务而非数百万个，但对例行的规划周期而言，
//! 精确搜索仍不切实际。故此处的调度器是贪心启发式。
//!
//! 该算法把任务排序，并把每个放到其全部资源同时空闲的最早时间。任务按加权价值递减
//! 的次序考虑，这是一个好的启发式：最有价值的工作先被放置，最不有价值的填充剩余
//! 空隙。强制任务不论其价值都先放置，因为一个必须运行的任务比任何可以运行的任务
//! 都值更多。
//!
//! 结果不保证最优，且本 crate 报告它无法放置的内容而非无声丢弃，使操作员能看到该
//! 计划的代价。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod greedy;
mod improve;
mod planner;
mod sweep;

pub use greedy::{
    demand_at, feasible_at, infeasible_tasks, resources_free_at, resources_have_capacity,
};
pub use improve::{ImprovementOutcome, ImprovementSettings, improve};
pub use planner::{PlanOutcome, Planner, SchedulingResult};
pub use sweep::earliest_feasible_start;

pub use apsis_resource::{ResourceId, ResourceSet};
pub use apsis_schedule::Schedule;
pub use apsis_task::{PrecedenceGraph, Task, TaskId};
