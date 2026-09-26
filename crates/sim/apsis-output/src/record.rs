//! One recorded value.

use core::fmt;

/// What a record is about.
///
/// The three kinds separate the things a caller usually wants to tell apart
/// when it reads a trace back: a value the model measured, a decision it
/// made, and a problem it wants to surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RecordKind {
    /// A value the model measured, such as a range or a rate.
    Observation,
    /// A change the model decided on, such as entering eclipse.
    State,
    /// A condition worth surfacing, such as a value outside its bounds.
    Warning,
}

impl RecordKind {
    /// Returns a short name for the kind.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Observation => "observation",
            Self::State => "state",
            Self::Warning => "warning",
        }
    }

    /// Returns `true` when the kind is a warning.
    pub const fn is_warning(self) -> bool {
        matches!(self, Self::Warning)
    }
}

impl fmt::Display for RecordKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// One recorded value.
///
/// A record is small and copied by value, so a trace holds a flat list rather
/// than a tree of references.
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    /// The simulation time the record was made at, in seconds.
    pub time: f64,
    /// What the record is about.
    pub kind: RecordKind,
    /// What the value is about.
    pub label: String,
    /// The value recorded.
    pub value: f64,
}

impl Record {
    /// Creates a record.
    pub fn new(time: f64, kind: RecordKind, label: impl Into<String>, value: f64) -> Self {
        Self {
            time,
            kind,
            label: label.into(),
            value,
        }
    }

    /// Creates an observation record.
    pub fn observation(time: f64, label: impl Into<String>, value: f64) -> Self {
        Self::new(time, RecordKind::Observation, label, value)
    }

    /// Creates a state record.
    pub fn state(time: f64, label: impl Into<String>, value: f64) -> Self {
        Self::new(time, RecordKind::State, label, value)
    }

    /// Creates a warning record.
    pub fn warning(time: f64, label: impl Into<String>, value: f64) -> Self {
        Self::new(time, RecordKind::Warning, label, value)
    }

    /// Returns `true` when the record is usable.
    ///
    /// A record whose time or value is not finite has no place in a trace,
    /// because it cannot be ordered or summarized.
    pub fn is_valid(&self) -> bool {
        self.time.is_finite() && self.value.is_finite() && !self.label.is_empty()
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} = {}",
            self.time, self.kind, self.label, self.value
        )
    }
}
