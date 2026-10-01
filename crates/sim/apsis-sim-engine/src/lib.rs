//! Discrete-event simulation engine.
//!
//! 离散事件仿真引擎。
//!
//! The engine advances a clock by jumping from one event to the next. It
//! holds the event queue, the current simulation time, and the statistics a
//! caller needs to judge a run. What an event means is the caller's: it
//! supplies a handler that receives the event and may schedule more events,
//! and the engine only sequences the calls.
//!
//! The engine keeps the simulation deterministic. Events at the same instant
//! are delivered in the order they were scheduled, which the event queue
//! guarantees, so a run with the same inputs produces the same output. That
//! is what makes a result reproducible and a cross-check against another
//! implementation meaningful.
//!
//! The engine stops for one of four reasons, reported as a
//! [`StopReason`]:
//! the queue empties, the simulation reaches its time limit, the caller asks
//! it to stop, or an event limit is reached. The last two exist so that a
//! model that schedules events forever cannot run without bound.
//!
//! ```
//! use apsis_sim_engine::Engine;
//!
//! let mut engine = Engine::new();
//! engine.schedule(10.0, "hello");
//!
//! let mut seen = Vec::new();
//! let outcome = engine.run(|engine, event| {
//!     seen.push(event.payload);
//!     // The handler schedules the next event, which the engine delivers
//!     // after this one. It schedules it only once, so the queue drains.
//!     if event.payload == "hello" {
//!         engine.schedule(20.0, "world");
//!     }
//! });
//!
//! assert_eq!(seen, vec!["hello", "world"]);
//! assert_eq!(outcome.executed, 2);
//! assert_eq!(engine.now(), 20.0);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod engine;
mod outcome;

pub use engine::Engine;
pub use outcome::{EngineOutcome, EngineState, StopReason};

pub use apsis_event::{Event, EventId, EventQueue, QueueState};
