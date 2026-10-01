//! Resource modeling for satellite operations.
//!
//! 卫星运行的资源建模。
//!
//! A resource is anything a task can occupy: a satellite, a ground station, a
//! transmitter, an onboard recorder. This crate models the two things a
//! scheduler needs to know about one: when it is available, and how much of
//! it a task consumes.
//!
//! Availability is expressed as a set of disjoint intervals on a single time
//! axis, called windows. A resource is usable at an instant when that instant
//! falls inside one of its windows. The windows may come from anywhere: the
//! access analysis in `apsis-access` produces them for a ground station, and
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
//! use apsis_resource::{Resource, ResourceId, Window};
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

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod resource;
mod window;

pub use error::{ResourceError, ResourceErrorKind};
pub use resource::{Resource, ResourceId, ResourceSet};
pub use window::{Interval, Window, merge_windows, subtract_window};
