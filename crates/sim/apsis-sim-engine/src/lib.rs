//! Discrete-event simulation engine.
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
//!
//! 离散事件仿真引擎。
//!
//! 引擎通过从一个事件跳到下一个来推进时钟。它保存事件队列、当前仿真时刻，以及调用方
//! 判断一次运行所需的统计。一个事件意味着什么由调用方定义：它提供一个接收该事件并可
//! 再调度更多事件的处理器，而引擎只负责排序各次调用。
//!
//! 引擎保持仿真确定。同一瞬时的事件按它们被调度的次序投递，这是事件队列所保证的，故
//! 相同输入的一次运行产生相同输出。这正是使结果可复现、与另一实现的交叉核对有意义的
//! 原因。
//!
//! 引擎因四种原因之一停止，报告为一个 [`StopReason`]：队列排空、仿真达到其时限、
//! 调用方请求它停止，或达到事件上限。后两者存在，是为了让永远调度事件的模型无法无界
//! 运行。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod engine;
mod outcome;

pub use engine::Engine;
pub use outcome::{EngineOutcome, EngineState, StopReason};

pub use apsis_event::{Event, EventId, EventQueue, QueueState};
