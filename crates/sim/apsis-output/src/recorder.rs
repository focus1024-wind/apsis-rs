//! Building a trace as a simulation runs.

use crate::record::{Record, RecordKind};
use crate::trace::Trace;

/// Collects records as a simulation runs.
///
/// The recorder is the write side of a trace: a handler calls one of its
/// methods for each value it wants to keep, and asks for the trace when the
/// run finishes. The two are separate so that a handler can hold a recorder
/// while the trace is read elsewhere.
///
/// ```
/// use apsis_output::Recorder;
///
/// let mut recorder = Recorder::new();
/// recorder.observe(0.0, "range", 1_000.0);
/// recorder.observe(10.0, "range", 950.0);
///
/// let summary = recorder.trace().summarize("range").expect("the series exists");
/// assert_eq!(summary.count, 2);
/// assert_eq!(summary.change, -50.0);
/// ```
#[derive(Debug, Clone, Default)]
pub struct Recorder {
    trace: Trace,
    refused: u64,
}

impl Recorder {
    /// Creates an empty recorder.
    pub const fn new() -> Self {
        Self {
            trace: Trace::new(),
            refused: 0,
        }
    }

    /// Records a value of a given kind.
    ///
    /// Returns `false` when the record is not usable, which a caller can
    /// ignore when the value is known to be finite.
    pub fn record(
        &mut self,
        time: f64,
        kind: RecordKind,
        label: impl Into<String>,
        value: f64,
    ) -> bool {
        let accepted = self.trace.push(Record::new(time, kind, label, value));
        if !accepted {
            self.refused = self.refused.wrapping_add(1);
        }
        accepted
    }

    /// Records an observation.
    pub fn observe(&mut self, time: f64, label: impl Into<String>, value: f64) -> bool {
        self.record(time, RecordKind::Observation, label, value)
    }

    /// Records a state change.
    pub fn note(&mut self, time: f64, label: impl Into<String>, value: f64) -> bool {
        self.record(time, RecordKind::State, label, value)
    }

    /// Records a warning.
    pub fn warn(&mut self, time: f64, label: impl Into<String>, value: f64) -> bool {
        self.record(time, RecordKind::Warning, label, value)
    }

    /// Returns the number of records made.
    pub fn len(&self) -> usize {
        self.trace.len()
    }

    /// Returns `true` when no records have been made.
    pub fn is_empty(&self) -> bool {
        self.trace.is_empty()
    }

    /// Returns the number of records refused because they were not usable.
    ///
    /// A caller that records values from a model checks this to see whether
    /// any value was dropped, rather than comparing counts by hand.
    pub const fn refused(&self) -> u64 {
        self.refused
    }

    /// Returns the trace recorded so far.
    pub const fn trace(&self) -> &Trace {
        &self.trace
    }

    /// Returns the trace, leaving an empty one behind.
    ///
    /// This is what a caller does between runs: the finished trace goes to
    /// whoever reads it, and the recorder starts fresh. The refused count is
    /// not reset, because it records the whole life of the recorder.
    pub fn take_trace(&mut self) -> Trace {
        core::mem::take(&mut self.trace)
    }

    /// Removes every record without changing the refused count.
    pub fn clear(&mut self) -> usize {
        self.trace.clear()
    }
}
