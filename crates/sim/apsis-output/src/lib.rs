//! Simulation output recording.
//!
//! A simulation produces more output than a caller can act on at once: event
//! times, the values a model computes as it runs, and the warnings it raises.
//! This crate collects those into one trace, so that a run can be examined
//! after it finishes rather than only as it happens.
//!
//! A record has a time, a kind, a label, and a value. The time is the
//! simulation instant the record was made at, which is what makes a trace
//! orderable and joinable against the event log. The kind separates the three
//! things a caller usually wants to tell apart: an observation the model made,
//! a state change it decided on, and a warning it wants to surface. The label
//! names what the value is about, and the value is a single number, which
//! covers the great majority of what a simulation records without imposing a
//! schema the caller has to design.
//!
//! The trace keeps the records in the order they were made, not in time order.
//! That distinction matters: a handler may record a value for an event it is
//! handling after recording one for a follow-up it just scheduled, and the
//! order the model reasoned in is worth keeping. A caller that wants time
//! order asks for it with [`Trace::in_time_order`].
//!
//! ```
//! use apsis_output::{Recorder, RecordKind};
//!
//! let mut recorder = Recorder::new();
//! recorder.observe(10.0, "altitude", 550.0);
//! recorder.note(10.0, "eclipse", 1.0);
//!
//! assert_eq!(recorder.len(), 2);
//! assert!(recorder.trace().is_stable());
//! assert_eq!(recorder.trace().value_of("altitude"), Some(550.0));
//! let _ = RecordKind::Warning;
//! ```
//!
//! 仿真输出记录。
//!
//! 一次仿真产生的输出多到调用方无法一次全部处理：事件时刻、模型运行时计算的值，以及
//! 它提出的警告。本 crate 把这些收集进一条轨迹，使一次运行可以在结束后被检视，而不
//! 只在发生时。
//!
//! 一条记录有一个时刻、一个类别、一个标签与一个值。时刻是作出该记录时的仿真瞬时，
//! 这正是使轨迹可排序、可与事件日志连接的原因。类别区分了调用方通常想分辨的三件事：
//! 模型作出的观测、它决定的变更，以及它想揭示的警告。标签指明该值所关于的内容，而
//! 值是一个单数，这覆盖了一次仿真所记录的绝大多数内容，而不强加调用方必须设计的
//! 模式。
//!
//! 轨迹保持记录在它们被作出的次序，而非时间次序。该区别要紧：处理器可能先为它刚
//! 调度的后续事件记录了一个值，再为它正在处理的事件记录一个值，而模型推理所用的
//! 次序值得保留。想要时间次序的调用方用 [`Trace::in_time_order`] 索取。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod record;
mod recorder;
mod trace;

pub use record::{Record, RecordKind};
pub use recorder::Recorder;
pub use trace::{SeriesSummary, Trace};
