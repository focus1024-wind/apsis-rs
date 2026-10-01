//! Simulation event types and event queue.
//!
//! A discrete-event simulation advances by jumping from one event to the
//! next rather than by stepping through every instant. This crate provides
//! the queue that makes that possible: a priority queue ordered by time,
//! with a well defined order among events that fall at the same instant.
//!
//! The tie-break matters more than it first appears. Two events scheduled
//! for the same instant must be delivered in a repeatable order, or the
//! simulation produces a different result on each run and a cross-check
//! against another implementation is impossible. Each event therefore
//! carries a sequence number assigned when it is scheduled, and events at
//! equal times are ordered by that number, so the order is the order in
//! which they were scheduled. That is the convention used by every
//! discrete-event simulator.
//!
//! Times are in seconds from the start of the simulation, so the queue holds
//! no knowledge of calendars or time scales.
//!
//! ```
//! use apsis_event::EventQueue;
//!
//! let mut queue = EventQueue::new();
//! queue.schedule(10.0, "second");
//! queue.schedule(5.0, "first");
//!
//! // The earliest event comes out first.
//! let event = queue.pop().expect("the queue is not empty");
//! assert_eq!(event.time, 5.0);
//! assert_eq!(event.payload, "first");
//! ```
//!
//! 仿真事件类型与事件队列。
//!
//! 离散事件仿真通过从一个事件跳到下一个来推进，而不是逐步走过每个瞬时。本 crate
//! 提供使之成为可能的队列：一个按时间排序的优先队列，并在落在同一瞬时的各事件之间
//! 有明确定义的次序。
//!
//! 该并列打破规则比它初看更重要。为同一瞬时调度的两个事件必须以可重复的次序投递，
//! 否则仿真每次运行都会产生不同结果，与另一实现的交叉核对也不可能。故每个事件携带
//! 一个在调度时赋予的序号，时间相等的事件按该序号排序，故该次序即它们被调度的次序。
//! 这是每个离散事件仿真器所用的约定。
//!
//! 时间以距仿真起点的秒数计，故队列对日历与时标一无所知。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod event;
mod queue;

pub use event::{Event, EventId};
pub use queue::{EventQueue, QueueState};
