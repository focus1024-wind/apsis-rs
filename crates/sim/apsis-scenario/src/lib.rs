//! Scenario definition and loading.
//!
//! 场景的定义与加载。
//!
//! A scenario is a simulation's initial conditions: the resources a plan may
//! use, the tasks it must place, the precedence constraints between those
//! tasks, and the limits the run should respect. It is the input a run is
//! built from, and keeping it in one place means a run can be reproduced from
//! a single description rather than reconstructed from scattered arguments.
//!
//! The crate offers two ways to build one. The builder methods are what Rust
//! code uses, and they check each addition as it is made. The text format is
//! what a stored scenario uses, and [`Scenario::parse`] reads it back. The
//! two agree: a scenario written with [`Scenario::to_text`] parses to an equal
//! scenario, so a scenario can be stored, edited, and reloaded without drift.
//!
//! The text format is line oriented and carries no third party dependency. A
//! blank line and a line whose first non-space character is `#` are ignored.
//! Every other line is a keyword followed by its arguments:
//!
//! ```text
//! scenario leo downlink
//! epoch 0
//! time_limit 3600
//! event_limit 1000000
//!
//! resource 0 capacity 1
//! window 0 0 600
//! window 0 1200 1800
//!
//! task 0 duration 60
//! task_window 0 100 1000
//! task_priority 0 5
//! task_mandatory 0 true
//! require 0 0 1
//!
//! precedence 0 1
//! ```
//!
//! `epoch` is the time the simulation clock starts at, in seconds.
//! `time_limit` and `event_limit` are the two run limits; either may be
//! omitted, which leaves that limit unset.
//!
//! ```
//! use apsis_scenario::Scenario;
//!
//! let text = "\
//! scenario demo
//! resource 0 capacity 1
//! window 0 0 600
//! task 0 duration 60
//! require 0 0 1
//! ";
//!
//! let scenario = Scenario::parse(text).expect("the text is well formed");
//! assert_eq!(scenario.name(), "demo");
//! assert_eq!(scenario.resources().len(), 1);
//! assert_eq!(scenario.tasks().len(), 1);
//! assert_eq!(scenario.validate(), Ok(()));
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod parse;
mod scenario;

pub use error::{ScenarioError, ScenarioErrorKind};
pub use scenario::{Scenario, ScenarioSummary};

pub use apsis_resource::{Resource, ResourceId, ResourceSet, Window};
pub use apsis_task::{PrecedenceGraph, Task, TaskId};
