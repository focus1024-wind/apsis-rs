//! The event type and its identifier.

use core::fmt;

/// An identifier for a scheduled event.
///
/// The identifier combines the time at which the event is due with the order
/// in which it was scheduled. Two events that fall at the same instant are
/// therefore distinguishable, which is what makes the delivery order
/// repeatable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EventId {
    /// The scheduled time, in seconds from the start of the simulation.
    ///
    /// The value is stored as its bit pattern so that the derived ordering on
    /// the identifier matches the ordering on time for non-negative values,
    /// which is the only range a simulation uses.
    time_bits: u64,
    /// The order in which the event was scheduled.
    sequence: u64,
}

impl EventId {
    /// Creates an identifier from a time and a sequence number.
    ///
    /// The time must be finite and non-negative. A caller that has a time
    /// outside that range cannot use this type, which is deliberate: a
    /// simulation clock that goes backwards or becomes infinite has no
    /// meaningful order.
    pub fn new(time: f64, sequence: u64) -> Option<Self> {
        if !time.is_finite() || time < 0.0 {
            return None;
        }
        Some(Self {
            time_bits: time.to_bits(),
            sequence,
        })
    }

    /// Creates an identifier from a time already validated as usable.
    ///
    /// This is the crate internal constructor used by the queue, which has
    /// already checked the time before it stored the entry.
    pub(crate) fn from_valid(time: f64, sequence: u64) -> Self {
        Self {
            time_bits: time.to_bits(),
            sequence,
        }
    }

    /// Returns the scheduled time, in seconds.
    pub fn time(self) -> f64 {
        f64::from_bits(self.time_bits)
    }

    /// Returns the order in which the event was scheduled.
    pub const fn sequence(self) -> u64 {
        self.sequence
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "event {} at {} s", self.sequence, self.time())
    }
}

/// One scheduled event.
///
/// The payload is the caller's: it names what should happen. The queue does
/// not interpret it, so the same queue serves any model.
#[derive(Debug, Clone, PartialEq)]
pub struct Event<T> {
    /// The identifier of this event.
    pub id: EventId,
    /// The scheduled time, in seconds from the start of the simulation.
    pub time: f64,
    /// The payload describing what should happen.
    pub payload: T,
}

impl<T> Event<T> {
    /// Returns `true` when the event is due at or before a time.
    pub fn is_due(&self, now: f64) -> bool {
        self.time <= now
    }

    /// Returns a copy of this event with a different payload.
    pub fn with_payload<U>(self, payload: U) -> Event<U> {
        Event {
            id: self.id,
            time: self.time,
            payload,
        }
    }

    /// Returns a copy of this event scheduled at a different time.
    ///
    /// Returns `None` when the new time is not usable, which leaves the
    /// original event available for the caller to keep using.
    pub fn reschedule(&self, time: f64, sequence: u64) -> Option<Event<T>>
    where
        T: Clone,
    {
        Some(Event {
            id: EventId::new(time, sequence)?,
            time,
            payload: self.payload.clone(),
        })
    }
}
