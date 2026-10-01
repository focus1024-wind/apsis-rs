//! The event type and its identifier.
//!
//! 事件类型及其标识符。

use core::fmt;

/// An identifier for a scheduled event.
///
/// The identifier combines the time at which the event is due with the order
/// in which it was scheduled. Two events that fall at the same instant are
/// therefore distinguishable, which is what makes the delivery order
/// repeatable.
///
/// 已调度事件的标识符。
///
/// 该标识符把事件到期的时刻与其被调度的次序结合起来。因此落在同一
/// 瞬时的两个事件可区分，这正是投递次序可重复的原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EventId {
    /// The scheduled time, in seconds from the start of the simulation.
    ///
    /// The value is stored as its bit pattern so that the derived ordering on
    /// the identifier matches the ordering on time for non-negative values,
    /// which is the only range a simulation uses.
    ///
    /// 调度时刻，以距仿真起点的秒数表示。
    ///
    /// 该值以其位模式存储，使标识符上派生的次序对非负值与时间上的次序
    /// 一致；非负是仿真唯一使用的范围。
    time_bits: u64,
    /// The order in which the event was scheduled.
    ///
    /// 该事件被调度的次序。
    sequence: u64,
}

impl EventId {
    /// Creates an identifier from a time and a sequence number.
    ///
    /// The time must be finite and non-negative. A caller that has a time
    /// outside that range cannot use this type, which is deliberate: a
    /// simulation clock that goes backwards or becomes infinite has no
    /// meaningful order.
    ///
    /// 由时刻与序号创建标识符。
    ///
    /// 该时刻必须有限且非负。时刻超出该范围的调用方无法使用本类型，
    /// 这是有意为之：倒退或趋于无限的仿真时钟没有有意义的次序。
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
    ///
    /// 由一个已校验为可用的时刻创建标识符。
    ///
    /// 这是队列所用的 crate 内部构造函数，它在存储该项之前已检查过
    /// 该时刻。
    pub(crate) fn from_valid(time: f64, sequence: u64) -> Self {
        Self {
            time_bits: time.to_bits(),
            sequence,
        }
    }

    /// Returns the scheduled time, in seconds.
    ///
    /// 返回调度时刻，单位为秒。
    pub fn time(self) -> f64 {
        f64::from_bits(self.time_bits)
    }

    /// Returns the order in which the event was scheduled.
    ///
    /// 返回该事件被调度的次序。
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
///
/// 一个已调度的事件。
///
/// 载荷属于调用方：它指明应发生什么。队列不解释它，故同一队列可服务
/// 任何模型。
#[derive(Debug, Clone, PartialEq)]
pub struct Event<T> {
    /// The identifier of this event.
    ///
    /// 本事件的标识符。
    pub id: EventId,
    /// The scheduled time, in seconds from the start of the simulation.
    ///
    /// 调度时刻，以距仿真起点的秒数表示。
    pub time: f64,
    /// The payload describing what should happen.
    ///
    /// 描述应发生什么的载荷。
    pub payload: T,
}

impl<T> Event<T> {
    /// Returns `true` when the event is due at or before a time.
    ///
    /// 当该事件在某时刻或之前到期时返回 `true`。
    pub fn is_due(&self, now: f64) -> bool {
        self.time <= now
    }

    /// Returns a copy of this event with a different payload.
    ///
    /// 返回本事件的一个具有不同载荷的副本。
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
    ///
    /// 返回本事件的一个调度在不同时刻的副本。
    ///
    /// 新时刻不可用时返回 `None`，此时原事件仍可供调用方继续使用。
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
