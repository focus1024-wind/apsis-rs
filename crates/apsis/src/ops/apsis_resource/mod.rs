//! Resource modeling for satellite operations.
//!
//! A resource is anything a task can occupy: a satellite, a ground station, a
//! transmitter, an onboard recorder. This module models the two things a
//! scheduler needs to know about one: when it is available, and how much of
//! it a task consumes.
//!
//! Availability is expressed as a set of disjoint intervals on a single time
//! axis, called windows. A resource is usable at an instant when that instant
//! falls inside one of its windows. The windows may come from anywhere: the
//! access analysis in `apsis_access` produces them for a ground station, and
//! an operator enters maintenance periods by hand.
//!
//! Capacity is a positive number whose meaning is the caller's. A satellite
//! that can run one observation at a time has a capacity of one; a recorder
//! has a capacity in megabits; a power bus has one in watts. Two tasks
//! conflict only when their demands together exceed the capacity, which lets
//! the same type express exclusive and shared resources.
//!
//! Times are in seconds from an origin chosen by the caller, so the same
//! model serves a scenario clock, an elapsed time, and a Julian date offset.
//!
//! ```
//! use apsis::apsis_resource::{Resource, ResourceId, Window};
//!
//! // A station available from 100 to 200 seconds.
//! let mut resource = Resource::new(ResourceId::new(0), 1.0);
//! resource.add_window(Window::new(100.0, 200.0)).expect("the window is usable");
//!
//! assert!(resource.is_available_at(150.0));
//! assert!(!resource.is_available_at(250.0));
//! assert!(resource.is_available_over(120.0, 80.0));
//! assert!(!resource.is_available_over(150.0, 100.0));
//! ```
//!
//! 卫星运行的资源建模。
//!
//! 资源是任务可以占用的任何东西：一颗卫星、一座地面站、一台发射机、一台星载
//! 记录器。本模块为调度器需要了解的关于它的两件事建模：它何时可用，以及一个
//! 任务消耗它多少。
//!
//! 可用性表达为单一时间轴上互不相交的区间集合，称为窗口。在某瞬时有窗口包含该
//! 瞬时，资源即可用。这些窗口可来自任何地方：`apsis_access` 中的可见性分析为
//! 地面站产生它们，操作员则手工录入维护时段。
//!
//! 容量是一个正数，其含义由调用方定义。一次只能运行一项观测的卫星容量为一；记录
//! 器的容量以兆比特计；功率母线的以瓦特计。两个任务只有在它们的需求合起来超过
//! 容量时才冲突，这使同一个类型既能表达独占资源也能表达共享资源。
//!
//! 时间以距调用方所选原点的秒数计，故同一模型可服务于场景时钟、流逝时间与儒略日
//! 偏移。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod resource;
mod window;

pub use error::{ResourceError, ResourceErrorKind};
pub use resource::{Resource, ResourceId, ResourceSet};
pub use window::{Interval, Window, merge_windows, subtract_window};
