//! The event queue.
//!
//! 事件队列。

use crate::event::{Event, EventId};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// A heap entry ordered so that the earliest event is the greatest.
///
/// The standard library heap is a maximum heap, so the comparison is
/// reversed on time. Ties on time are broken by the sequence number, and
/// that comparison is also reversed so that the earlier sequence is the
/// greater element.
///
/// 一个堆元素，其排序使最早的事件为最大者。
///
/// 标准库的堆是最大堆，故时间上的比较被反转。时间上的并列由序号打破，
/// 而该比较也被反转，使较早的序号成为较大的元素。
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
        // 反转时间，使最小的时间成为最大的堆元素。时间相等时，较小的序号
        // 是较大的元素，故较早调度的事件先被投递。
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
///
/// 队列当前所持有的状态。
///
/// 该状态被公开，使仿真循环无需先弹出一个事件即可决定是推进时钟还是
/// 停止。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QueueState {
    /// The number of pending events.
    ///
    /// 待处理事件的数量。
    pub pending: usize,
    /// The time of the earliest pending event, if there is one.
    ///
    /// 最早待处理事件的时间（若存在）。
    pub next_time: Option<f64>,
    /// The number of events delivered so far.
    ///
    /// 迄今已投递的事件数。
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
///
/// 按时间、再按调度次序排序的事件优先队列。
///
/// 序号由队列分配，故调用方无法意外创建两个具有相同标识符的事件。
/// 序号从零开始，每个被调度的事件（包括在另一事件处理过程中调度的
/// 事件）都使之加一。
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
    ///
    /// 创建一个空队列。
    pub const fn new() -> Self {
        Self {
            pending: BinaryHeap::new(),
            payloads: Vec::new(),
            next_sequence: 0,
            delivered: 0,
        }
    }

    /// Returns the number of pending events.
    ///
    /// 返回待处理事件的数量。
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
    ///
    /// 返回迄今已投递的事件数。
    pub const fn delivered(&self) -> u64 {
        self.delivered
    }

    /// Returns what the queue currently holds.
    ///
    /// 返回队列当前所持有的状态。
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
    ///
    /// 返回最早待处理事件的时间。
    ///
    /// 队列为空时返回 `None`。
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
    ///
    /// 在某个时刻调度一个事件。
    ///
    /// 序号由队列分配，故同一瞬时各事件之间的投递次序即它们被调度的
    /// 次序。
    ///
    /// 返回新事件的标识符；时刻不可用时返回 `None`：仿真时钟从零向前
    /// 运行，故负或非有限的时刻被拒绝而非存储。
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
        // 载荷槽位按序号索引，故当调用方弹出事件而未取其载荷时，空档以
        // `None` 填充。
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
    ///
    /// 在相对于某基准瞬时的时刻调度一个事件。
    ///
    /// 这是处理器调度后续事件时所用的形式：延时相对于当前仿真时刻。
    ///
    /// 所得时刻不可用时返回 `None`。
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
    ///
    /// 取出并返回最早的事件。
    ///
    /// 队列为空时返回 `None`。载荷被取离队列，故该槽位被留空；调度
    /// 后续事件会复用该槽位。
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
    ///
    /// 返回最早事件载荷的引用而不移除它。
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
    ///
    /// 移除所有待处理事件并返回被丢弃的数量。
    ///
    /// 已投递计数保持不变，因为它记录的是历史而非队列内容。
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
    ///
    /// 按投递次序返回所有待处理事件，并使队列为空。
    ///
    /// 该次序即各事件本会被投递的次序，这正是调用方在运行结束时记录
    /// 剩余调度所需的。
    pub fn drain(&mut self) -> Vec<Event<T>> {
        let mut events = Vec::with_capacity(self.pending.len());
        while let Some(event) = self.pop() {
            events.push(event);
        }
        events
    }
}
