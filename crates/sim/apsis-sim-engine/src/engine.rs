//! The engine itself.

use crate::outcome::{EngineOutcome, EngineState, StopReason};
use apsis_event::{Event, EventId, EventQueue};

/// A discrete-event simulation engine.
///
/// The engine owns the event queue and the clock. A run repeatedly takes the
/// earliest event and passes it to a handler, which may schedule more
/// events. The engine stops when the queue empties or when one of the limits
/// is reached.
#[derive(Debug, Clone)]
pub struct Engine<T> {
    queue: EventQueue<T>,
    now: f64,
    executed: u64,
    stop_requested: bool,
    time_limit: Option<f64>,
    event_limit: Option<u64>,
}

impl<T> Default for Engine<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Engine<T> {
    /// Creates an engine with the clock at zero and no limits.
    ///
    /// An engine with no limits runs until the queue empties, which is the
    /// right default for a model that terminates. A model that schedules
    /// without bound needs a limit, which is set with
    /// [`Engine::with_time_limit`] or [`Engine::with_event_limit`].
    pub const fn new() -> Self {
        Self {
            queue: EventQueue::new(),
            now: 0.0,
            executed: 0,
            stop_requested: false,
            time_limit: None,
            event_limit: None,
        }
    }

    /// Sets the latest simulation time the engine will process.
    ///
    /// An event scheduled beyond the limit is left in the queue rather than
    /// executed, so the caller can examine what remained. A limit that is
    /// negative or not finite disables the limit.
    pub const fn with_time_limit(mut self, limit: f64) -> Self {
        self.time_limit = if limit.is_finite() && limit >= 0.0 {
            Some(limit)
        } else {
            None
        };
        self
    }

    /// Sets the greatest number of events the engine will execute.
    ///
    /// A limit of zero disables the rule, because a run that executes no
    /// events is never useful.
    pub const fn with_event_limit(mut self, limit: u64) -> Self {
        self.event_limit = if limit == 0 { None } else { Some(limit) };
        self
    }

    /// Returns the current simulation time, in seconds.
    pub const fn now(&self) -> f64 {
        self.now
    }

    /// Sets the current simulation time.
    ///
    /// This is used to start a run at an offset, for example at a scenario
    /// epoch rather than at zero. It does not reschedule pending events, so a
    /// caller that moves the clock forward past a pending event will execute
    /// that event late. The engine has no way to detect the mistake, so the
    /// caller is responsible for keeping the clock consistent with the
    /// queue.
    pub fn set_now(&mut self, time: f64) {
        if time.is_finite() && time >= 0.0 {
            self.now = time;
        }
    }

    /// Returns the number of events executed so far.
    pub const fn executed(&self) -> u64 {
        self.executed
    }

    /// Returns the number of pending events.
    pub fn pending(&self) -> usize {
        self.queue.len()
    }

    /// Returns `true` when no events are pending.
    pub fn is_idle(&self) -> bool {
        self.queue.is_empty()
    }

    /// Returns the time of the next pending event, if there is one.
    pub fn next_time(&self) -> Option<f64> {
        self.queue.next_time()
    }

    /// Returns what the engine currently holds.
    pub fn state(&self) -> EngineState {
        EngineState {
            now: self.now,
            pending: self.queue.len(),
            executed: self.executed,
            next_time: self.queue.next_time(),
        }
    }

    /// Schedules an event at a time.
    ///
    /// Returns the identifier of the new event, or `None` when the time is
    /// not usable.
    pub fn schedule(&mut self, time: f64, payload: T) -> Option<EventId> {
        self.queue.schedule(time, payload)
    }

    /// Schedules an event at a delay from the current simulation time.
    ///
    /// Returns the identifier of the new event, or `None` when the delay is
    /// not usable.
    pub fn schedule_after(&mut self, delay: f64, payload: T) -> Option<EventId> {
        self.queue.schedule_after(self.now, delay, payload)
    }

    /// Asks the engine to stop after the current event.
    ///
    /// The handler calls this when its own model says the run is over, for
    /// example when a stopping condition that the queue cannot express is
    /// met. The pending events are left in the queue.
    pub fn request_stop(&mut self) {
        self.stop_requested = true;
    }

    /// Clears the stop request, so that a subsequent run continues.
    pub fn clear_stop(&mut self) {
        self.stop_requested = false;
    }

    /// Removes every pending event and returns how many were discarded.
    pub fn clear(&mut self) -> usize {
        self.queue.clear()
    }

    /// Returns the queue, for a caller that needs to inspect or manipulate it
    /// directly.
    pub const fn queue(&self) -> &EventQueue<T> {
        &self.queue
    }

    /// Returns the queue for modification.
    pub const fn queue_mut(&mut self) -> &mut EventQueue<T> {
        &mut self.queue
    }

    /// Runs the engine until it stops.
    ///
    /// Each iteration takes the earliest event, advances the clock to its
    /// time, and passes it to the handler. The handler may schedule further
    /// events, including at the same instant, which are delivered after the
    /// events already scheduled for that instant.
    ///
    /// Returns the outcome of the run. The clock is left at the time of the
    /// last event executed, not at a limit, so a caller that records the
    /// clock sees the last instant the model actually reached.
    pub fn run<F>(&mut self, mut handler: F) -> EngineOutcome
    where
        F: FnMut(&mut Engine<T>, Event<T>),
    {
        let started_at = self.now;
        let mut executed_this_run = 0_u64;
        self.stop_requested = false;

        let reason = loop {
            // The event limit is checked before taking an event, so a run
            // that reaches the limit leaves the queue untouched for the next
            // call.
            if let Some(limit) = self.event_limit {
                if self.executed >= limit {
                    break StopReason::EventLimit;
                }
            }

            let Some(event) = self.queue.pop() else {
                break StopReason::QueueEmpty;
            };

            // The time limit leaves the event in the queue rather than
            // discarding it, so the caller can inspect what remained. The
            // event was already popped, so it is put back.
            if let Some(limit) = self.time_limit {
                if event.time > limit {
                    self.queue.schedule(event.time, event.payload);
                    break StopReason::TimeLimit;
                }
            }

            self.now = event.time;
            self.executed = self.executed.wrapping_add(1);
            executed_this_run += 1;

            handler(self, event);

            if self.stop_requested {
                break StopReason::Requested;
            }
        };

        EngineOutcome {
            reason,
            executed: executed_this_run,
            started_at,
            ended_at: self.now,
            remaining: self.queue.len(),
        }
    }
}
