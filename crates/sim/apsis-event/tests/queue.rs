//! Tests for the event queue.
//!
//! 事件队列的测试。

use apsis_event::{Event, EventId, EventQueue};

#[test]
fn an_empty_queue_reports_itself_idle() {
    let queue: EventQueue<u32> = EventQueue::new();
    assert!(queue.is_empty());
    assert_eq!(queue.len(), 0);
    assert_eq!(queue.next_time(), None);
    assert_eq!(queue.peek(), None);
    assert_eq!(queue.delivered(), 0);
    assert!(queue.state().is_idle());
    assert_eq!(queue.state().next_time, None);
}

#[test]
fn events_are_delivered_in_time_order() {
    let mut queue = EventQueue::new();
    queue.schedule(30.0, "third");
    queue.schedule(10.0, "first");
    queue.schedule(20.0, "second");

    assert_eq!(queue.len(), 3);
    assert_eq!(queue.next_time(), Some(10.0));
    assert_eq!(queue.peek(), Some(&"first"));

    let mut order = Vec::new();
    while let Some(event) = queue.pop() {
        order.push(event.payload);
    }
    assert_eq!(order, vec!["first", "second", "third"]);
    assert!(queue.is_empty());
    assert_eq!(queue.delivered(), 3);
}

#[test]
fn events_at_the_same_time_are_delivered_in_schedule_order() {
    // The tie-break is what makes a run reproducible.
    // 并列打破规则正是使运行可复现的东西。
    let mut queue = EventQueue::new();
    queue.schedule(10.0, 'a');
    queue.schedule(10.0, 'b');
    queue.schedule(10.0, 'c');

    let order: Vec<char> = queue
        .drain()
        .into_iter()
        .map(|event| event.payload)
        .collect();
    assert_eq!(order, vec!['a', 'b', 'c']);
}

#[test]
fn the_tie_break_holds_across_a_heap_rebuild() {
    // Scheduling in a different order must still deliver in schedule order,
    // because the sequence number, not the insertion position, decides.
    // 以不同次序调度仍必须按调度次序投递，因为决定的是序号而非插入位置。
    let mut queue = EventQueue::new();
    let mut ids = Vec::new();
    for index in 0..20 {
        ids.push(queue.schedule(5.0, index).expect("the time is usable"));
    }
    let delivered: Vec<u64> = queue
        .drain()
        .into_iter()
        .map(|event| event.payload)
        .collect();
    assert_eq!(delivered, (0..20).collect::<Vec<u64>>());

    // The identifiers are also in schedule order.
    // 标识符也按调度次序排列。
    for (position, id) in ids.iter().enumerate() {
        assert_eq!(id.sequence(), position as u64);
    }
}

#[test]
fn an_event_carries_its_identifier_and_time() {
    let mut queue = EventQueue::new();
    let id = queue.schedule(12.5, "payload").expect("the time is usable");
    assert_eq!(id.time(), 12.5);
    assert_eq!(id.sequence(), 0);

    let event = queue.pop().expect("the queue is not empty");
    assert_eq!(event.id, id);
    assert_eq!(event.time, 12.5);
    assert_eq!(event.payload, "payload");
}

#[test]
fn scheduling_a_negative_or_non_finite_time_is_refused() {
    let mut queue = EventQueue::new();
    for time in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(queue.schedule(time, "x"), None, "time {time}");
    }
    // The queue is unchanged and the sequence number was not consumed.
    // 队列不变，且序号未被消耗。
    assert!(queue.is_empty());
    let id = queue.schedule(0.0, "zero").expect("zero is usable");
    assert_eq!(id.sequence(), 0);
}

#[test]
fn a_follow_up_event_is_scheduled_relative_to_now() {
    let mut queue = EventQueue::new();
    queue
        .schedule_after(100.0, 30.0, "follow up")
        .expect("the delay is usable");
    let event = queue.pop().expect("the queue is not empty");
    assert_eq!(event.time, 130.0);
}

#[test]
fn a_follow_up_with_a_bad_delay_is_refused() {
    let mut queue = EventQueue::new();
    assert_eq!(queue.schedule_after(0.0, -1.0, "x"), None);
    assert_eq!(queue.schedule_after(0.0, f64::NAN, "x"), None);
    assert_eq!(queue.schedule_after(0.0, f64::INFINITY, "x"), None);
    assert!(queue.is_empty());
    // A zero delay schedules at the current instant.
    // 零延时在当前瞬时调度。
    let id = queue.schedule_after(5.0, 0.0, "now").expect("usable");
    assert_eq!(id.time(), 5.0);
}

#[test]
fn events_can_be_scheduled_from_a_handler() {
    // A handler that schedules its successor, which is the pattern a
    // discrete-event simulation uses for a periodic process.
    // 一个调度其后继的处理器，这是离散事件仿真用于周期过程的模式。
    let mut queue: EventQueue<u32> = EventQueue::new();
    queue.schedule(0.0, 0).expect("usable");
    let mut ticks = 0;
    while let Some(event) = queue.pop() {
        ticks += 1;
        if event.payload < 3 {
            queue
                .schedule_after(event.time, 10.0, event.payload + 1)
                .expect("the delay is usable");
        }
    }
    assert_eq!(ticks, 4);
    assert_eq!(queue.delivered(), 4);
}

#[test]
fn clearing_discards_the_pending_events() {
    let mut queue = EventQueue::new();
    for index in 0..5 {
        queue.schedule(f64::from(index), index).expect("usable");
    }
    queue.pop().expect("the queue is not empty");
    assert_eq!(queue.len(), 4);
    assert_eq!(queue.delivered(), 1);

    assert_eq!(queue.clear(), 4);
    assert!(queue.is_empty());
    // The delivered count records history and is not reset.
    // 已投递计数记录历史且不重置。
    assert_eq!(queue.delivered(), 1);
    assert_eq!(queue.pop(), None);
}

#[test]
fn draining_returns_the_events_in_delivery_order() {
    let mut queue = EventQueue::new();
    queue.schedule(3.0, 'c');
    queue.schedule(1.0, 'a');
    queue.schedule(2.0, 'b');

    let events = queue.drain();
    let payloads: Vec<char> = events.iter().map(|event| event.payload).collect();
    assert_eq!(payloads, vec!['a', 'b', 'c']);
    assert!(queue.is_empty());
    assert_eq!(queue.delivered(), 3);
}

#[test]
fn the_queue_state_reports_what_is_pending() {
    let mut queue = EventQueue::new();
    queue.schedule(10.0, "a").expect("usable");
    queue.schedule(20.0, "b").expect("usable");
    let state = queue.state();
    assert_eq!(state.pending, 2);
    assert_eq!(state.next_time, Some(10.0));
    assert_eq!(state.delivered, 0);
    assert!(!state.is_idle());

    queue.pop();
    let state = queue.state();
    assert_eq!(state.pending, 1);
    assert_eq!(state.next_time, Some(20.0));
    assert_eq!(state.delivered, 1);
}

#[test]
fn a_payload_slot_is_reused_after_a_pop() {
    // The queue indexes payloads by sequence, so a popped slot must be
    // reusable without disturbing a later event.
    // 队列按序号索引载荷，故被弹出的槽位必须可复用而不扰动更晚的事件。
    let mut queue = EventQueue::new();
    queue.schedule(1.0, "first").expect("usable");
    queue.schedule(2.0, "second").expect("usable");
    let first = queue.pop().expect("non-empty");
    assert_eq!(first.payload, "first");
    let second = queue.pop().expect("non-empty");
    assert_eq!(second.payload, "second");
    assert_eq!(queue.pop(), None);
}

#[test]
fn the_identifier_orders_by_time_then_sequence() {
    // The derived ordering on the identifier is the delivery order, which is
    // what lets a caller sort identifiers to reproduce a run.
    // 标识符上派生的次序即投递次序，这正是使调用方可对标识符排序以复现一次运行的
    // 原因。
    let earlier = EventId::new(1.0, 5).expect("usable");
    let later_time = EventId::new(2.0, 0).expect("usable");
    let later_sequence = EventId::new(1.0, 6).expect("usable");
    assert!(earlier < later_time);
    assert!(earlier < later_sequence);
    assert!(EventId::new(1.0, 0).expect("usable") < earlier);
}

#[test]
fn an_identifier_reports_itself() {
    let id = EventId::new(3.5, 7).expect("usable");
    assert_eq!(id.time(), 3.5);
    assert_eq!(id.sequence(), 7);
    assert!(id.to_string().contains('7'));
    assert!(id.to_string().contains("3.5"));
}

#[test]
fn an_event_reports_whether_it_is_due() {
    let event = Event {
        id: EventId::new(5.0, 0).expect("usable"),
        time: 5.0,
        payload: "x",
    };
    assert!(!event.is_due(4.999));
    assert!(event.is_due(5.0));
    assert!(event.is_due(6.0));
}

#[test]
fn an_event_can_be_rescheduled() {
    let event = Event {
        id: EventId::new(5.0, 0).expect("usable"),
        time: 5.0,
        payload: "x",
    };
    let moved = event.reschedule(9.0, 1).expect("the new time is usable");
    assert_eq!(moved.time, 9.0);
    assert_eq!(moved.id.sequence(), 1);
    assert_eq!(moved.payload, "x");
    // The original is unchanged.
    // 原件不变。
    assert_eq!(event.time, 5.0);

    assert!(event.reschedule(-1.0, 2).is_none());
    assert!(event.reschedule(f64::NAN, 3).is_none());
}

#[test]
fn an_event_can_change_its_payload() {
    let event = Event {
        id: EventId::new(5.0, 0).expect("usable"),
        time: 5.0,
        payload: 1_u32,
    };
    let changed = event.with_payload("text");
    assert_eq!(changed.payload, "text");
    assert_eq!(changed.time, 5.0);
}

#[test]
fn a_zero_time_event_is_the_earliest_possible() {
    let mut queue = EventQueue::new();
    queue.schedule(1.0, "later").expect("usable");
    queue.schedule(0.0, "first").expect("usable");
    assert_eq!(queue.pop().expect("non-empty").payload, "first");
}

#[test]
fn the_queue_default_is_empty() {
    let queue: EventQueue<u8> = EventQueue::default();
    assert!(queue.is_empty());
}
