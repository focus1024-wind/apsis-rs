//! Simulation event types and event queue.
//!
//! 仿真事件类型与事件队列。
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

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod event;
mod queue;

pub use event::{Event, EventId};
pub use queue::{EventQueue, QueueState};
