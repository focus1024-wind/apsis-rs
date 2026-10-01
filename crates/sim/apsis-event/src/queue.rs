//! The event queue.

use crate::event::{Event, EventId};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// A heap entry ordered so that the earliest event is the greatest.
///
/// The standard library heap is a maximum heap, so the comparison is
/// reversed on time. Ties on time are broken by the sequence number, and
/// that comparison is also reversed so that the earlier sequence is the
/// greater element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Scheduled {
    time_bits: u64,
    sequence: u64,
}

impl Scheduled {
    fn time(self) -> f64 {
        f64::from_bits(self.time_bits)
    }
}

impl Ord for Scheduled {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse the time so that the smallest time is the greatest heap
        // element. For equal times, the smaller sequence is the greater
        // element, so an earlier scheduled event is delivered first.
        other
            .time_bits
            .cmp(&self.time_bits)
            .then_with(|| other.sequence.cmp(&self.sequence))
    }
}

impl PartialOrd for Scheduled {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// What the queue currently holds.
///
/// The state is exposed so that a simulation loop can decide whether to
/// advance the clock or to stop without popping an event first.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QueueState {
    /// The number of pending events.
    pub pending: usize,
    /// The time of the earliest pending event, if there is one.
    pub next_time: Option<f64>,
    /// The number of events delivered so far.
    pub delivered: u64,
}

impl QueueState {
    /// Returns `true` when no events are pending.
    ///
    /// 当没有待处理事件时返回 `true`。
    pub const fn is_idle(&self) -> bool {
        self.pending == 0
    }
}

/// A priority queue of events ordered by time and then by schedule order.
///
/// The queue assigns the sequence numbers, so a caller cannot accidentally
/// create two events with the same identifier. Sequence numbers start at
/// zero and increase by one with every event scheduled, including events
/// scheduled from inside the handling of another event.
#[derive(Debug, Clone)]
pub struct EventQueue<T> {
    pending: BinaryHeap<Scheduled>,
    payloads: Vec<Option<T>>,
    next_sequence: u64,
    delivered: u64,
}

impl<T> Default for EventQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> EventQueue<T> {
    /// Creates an empty queue.
    pub const fn new() -> Self {
        Self {
            pending: BinaryHeap::new(),
            payloads: Vec::new(),
            next_sequence: 0,
            delivered: 0,
        }
    }

    /// Returns the number of pending events.
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    /// Returns `true` when no events are pending.
    ///
    /// 当没有待处理事件时返回 `true`。
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// Returns the number of events delivered so far.
    pub const fn delivered(&self) -> u64 {
        self.delivered
    }

    /// Returns what the queue currently holds.
    pub fn state(&self) -> QueueState {
        QueueState {
            pending: self.pending.len(),
            next_time: self.pending.peek().map(|entry| entry.time()),
            delivered: self.delivered,
        }
    }

    /// Returns the time of the earliest pending event.
    ///
    /// Returns `None` when the queue is empty.
    pub fn next_time(&self) -> Option<f64> {
        self.pending.peek().map(|entry| entry.time())
    }

    /// Schedules an event at a time.
    ///
    /// The sequence number is assigned by the queue, so the delivery order
    /// among events at the same instant is the order they were scheduled.
    ///
    /// Returns the identifier of the new event, or `None` when the time is
    /// not usable: a simulation clock runs forward from zero, so a negative
    /// or non-finite time is refused rather than stored.
    pub fn schedule(&mut self, time: f64, payload: T) -> Option<EventId> {
        let sequence = self.next_sequence;
        let id = EventId::new(time, sequence)?;
        self.next_sequence = self.next_sequence.wrapping_add(1);
        self.pending.push(Scheduled {
            time_bits: time.to_bits(),
            sequence,
        });
        // The payload slots are indexed by sequence, so gaps are filled with
        // `None` when a caller has popped an event without taking its
        // payload.
        let index = sequence as usize;
        if index < self.payloads.len() {
            self.payloads[index] = Some(payload);
        } else {
            while self.payloads.len() < index {
                self.payloads.push(None);
            }
            self.payloads.push(Some(payload));
        }
        Some(id)
    }

    /// Schedules an event at a time relative to a base instant.
    ///
    /// This is the form a handler uses when it schedules a follow-up: the
    /// delay is relative to the current simulation time.
    ///
    /// Returns `None` when the resulting time is not usable.
    pub fn schedule_after(&mut self, now: f64, delay: f64, payload: T) -> Option<EventId> {
        if !delay.is_finite() || delay < 0.0 {
            return None;
        }
        self.schedule(now + delay, payload)
    }

    /// Removes and returns the earliest event.
    ///
    /// Returns `None` when the queue is empty. The payload is taken from the
    /// queue, so the slot is left empty; scheduling a later event reuses the
    /// slot.
    pub fn pop(&mut self) -> Option<Event<T>> {
        let entry = self.pending.pop()?;
        let index = entry.sequence as usize;
        let payload = self
            .payloads
            .get_mut(index)
            .and_then(Option::take)
            .expect("a pending entry always has its payload");
        self.delivered = self.delivered.wrapping_add(1);
        Some(Event {
            id: EventId::from_valid(entry.time(), entry.sequence),
            time: entry.time(),
            payload,
        })
    }

    /// Returns a reference to the payload of the earliest event without
    /// removing it.
    pub fn peek(&self) -> Option<&T> {
        let entry = self.pending.peek()?;
        self.payloads
            .get(entry.sequence as usize)
            .and_then(Option::as_ref)
    }

    /// Removes every pending event and returns how many were discarded.
    ///
    /// The delivered count is left alone, because it records history rather
    /// than queue contents.
    pub fn clear(&mut self) -> usize {
        let count = self.pending.len();
        self.pending.clear();
        for slot in &mut self.payloads {
            *slot = None;
        }
        count
    }

    /// Returns every pending event in delivery order, leaving the queue
    /// empty.
    ///
    /// The order is the order the events would have been delivered in, which
    /// is what a caller wants when it records the remaining schedule at the
    /// end of a run.
    pub fn drain(&mut self) -> Vec<Event<T>> {
        let mut events = Vec::with_capacity(self.pending.len());
        while let Some(event) = self.pop() {
            events.push(event);
        }
        events
    }
}
