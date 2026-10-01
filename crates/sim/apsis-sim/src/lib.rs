//! Simulation facade that integrates the simulation layer.
//!
//! This crate is the single entry point to the simulation layer. It ties the
//! four pieces together: a [`Scenario`] describes the initial conditions, the
//! ops layer plans it, the engine runs the resulting timeline, and a
//! [`Recorder`] collects what the run produced.
//!
//! The sequence a caller follows is short:
//!
//! 1. [`Simulation::prepare`] checks a scenario and plans it, producing a
//!    schedule.
//! 2. [`Simulation::start`] turns that schedule into a timeline of start and
//!    end events and hands back a [`Session`].
//! 3. [`Session::run`] drives the engine, calling a handler at each event,
//!    and returns a [`RunReport`] holding the engine outcome and the trace.
//!
//! The plan is a greedy heuristic, described in `apsis-scheduler`, so a run
//! may leave tasks unplaced. The facade reports that rather than hiding it:
//! [`Simulation::plan`] exposes the full planning result, and
//! [`Simulation::has_full_plan`] answers the common question directly.
//!
//! The timeline is the schedule, not a physical model. A start event fires
//! when a task begins and an end event when it ends, which is what a caller
//! needs to record state over time; the handler supplies whatever physics or
//! bookkeeping the run requires.
//!
//! ```
//! use apsis_sim::{Activity, Scenario, Simulation};
//!
//! let text = "\
//! scenario demo
//! resource 0 capacity 1
//! window 0 0 600
//! task 0 duration 60
//! require 0 0 1
//! ";
//! let scenario = Scenario::parse(text).expect("the text is well formed");
//! let simulation = Simulation::prepare(scenario).expect("the scenario is usable");
//!
//! let mut session = simulation.start();
//! let mut starts = 0_u32;
//! let report = session.run(|_engine, _recorder, event| {
//!     if matches!(event.payload, Activity::Start { .. }) {
//!         starts += 1;
//!     }
//! });
//!
//! assert!(simulation.has_full_plan());
//! assert_eq!(starts, 1);
//! assert_eq!(report.outcome.executed, 2);
//! ```
//!
//! 整合仿真层的仿真门面。
//!
//! 本 crate 是仿真层的单一入口。它把四部分联系起来：[`Scenario`] 描述初始条件，
//! ops 层为其作计划，引擎运行所得时间线，[`Recorder`] 收集该运行所产生的。
//!
//! 调用方所遵循的序列很短：
//!
//! 1. [`Simulation::prepare`] 检查场景并为其作计划，产生一个调度。
//! 2. [`Simulation::start`] 把该调度转为由开始与结束事件构成的时间线，并交回一个
//!    [`Session`]。
//! 3. [`Session::run`] 驱动引擎，在每个事件处调用一个处理器，并返回一个
//!    [`RunReport`]，其中含有引擎结果与轨迹。
//!
//! 该计划是贪心启发式，在 `apsis-scheduler` 中有述，故一次运行可能留下未放置的任务。
//! 门面报告这一点而非隐藏它：[`Simulation::plan`] 公开完整规划结果，而
//! [`Simulation::has_full_plan`] 直接回答那个常见的问题。
//!
//! 时间线是调度而非物理模型。任务开始时触发一个开始事件，结束时触发一个结束事件，
//! 这正是调用方随时间记录状态所需的；处理器提供该运行所需的任何物理或簿记。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod activity;
mod error;
mod session;
mod simulation;

pub use activity::Activity;
pub use error::{SimulationError, SimulationErrorKind};
pub use session::{RunReport, Session};
pub use simulation::Simulation;

pub use apsis_event::{Event, EventId, EventQueue};
pub use apsis_output::{Record, RecordKind, Recorder, SeriesSummary, Trace};
pub use apsis_scenario::{Scenario, ScenarioError, ScenarioErrorKind, ScenarioSummary};
pub use apsis_sim_engine::{Engine, EngineOutcome, EngineState, StopReason};
