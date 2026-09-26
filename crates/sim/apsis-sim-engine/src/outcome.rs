//! What a run produced and what the engine currently holds.

use core::fmt;

/// Why a run stopped.
///
/// The reason is reported rather than inferred from the event count, because
/// the same count can arise from an empty queue or from a limit, and a
/// caller that records the outcome of a run needs to distinguish them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StopReason {
    /// No events were left when the engine looked for the next one.
    QueueEmpty,
    /// The time of the next event is beyond the simulation time limit.
    TimeLimit,
    /// The handler set a flag asking the engine to stop.
    Requested,
    /// The event count reached the configured limit.
    EventLimit,
}

impl StopReason {
    /// Returns `true` when the run ended because the model had no more work
    /// to do rather than because a limit intervened.
    pub const fn is_natural(self) -> bool {
        matches!(self, Self::QueueEmpty)
    }

    /// Returns a short name for the reason.
    pub const fn name(self) -> &'static str {
        match self {
            Self::QueueEmpty => "queue empty",
            Self::TimeLimit => "time limit",
            Self::Requested => "requested",
            Self::EventLimit => "event limit",
        }
    }
}

impl fmt::Display for StopReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// What a run produced.
///
/// The outcome records the counts and the clock, so a caller can record the
/// result of a run without reaching back into the engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineOutcome {
    /// Why the run stopped.
    pub reason: StopReason,
    /// The number of events the run executed.
    pub executed: u64,
    /// The simulation time the run started from, in seconds.
    pub started_at: f64,
    /// The simulation time the run reached, in seconds.
    pub ended_at: f64,
    /// The number of events still pending when the run stopped.
    pub remaining: usize,
}

impl EngineOutcome {
    /// Returns the time the run covered, in seconds.
    pub fn duration(&self) -> f64 {
        self.ended_at - self.started_at
    }

    /// Returns the mean number of events per second of simulation time.
    ///
    /// This measures how busy the model is rather than how long the run took
    /// on the wall clock. Returns `None` when the run covered no time.
    pub fn event_rate(&self) -> Option<f64> {
        let duration = self.duration();
        if duration > 0.0 {
            Some(self.executed as f64 / duration)
        } else {
            None
        }
    }

    /// Returns `true` when the run finished because the model had no more
    /// work rather than because a limit stopped it.
    pub fn is_natural(&self) -> bool {
        self.reason.is_natural()
    }
}

impl fmt::Display for EngineOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} event(s) from {} s to {} s, stopped by {}",
            self.executed, self.started_at, self.ended_at, self.reason
        )
    }
}

/// What the engine currently holds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineState {
    /// The current simulation time, in seconds.
    pub now: f64,
    /// The number of pending events.
    pub pending: usize,
    /// The number of events executed so far.
    pub executed: u64,
    /// The time of the next pending event, if there is one.
    pub next_time: Option<f64>,
}

impl EngineState {
    /// Returns `true` when no events are pending.
    pub const fn is_idle(&self) -> bool {
        self.pending == 0
    }
}
