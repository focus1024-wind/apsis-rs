//! Tests for the simulation engine.
//!
//! 仿真引擎的测试。

use apsis::apsis_sim_engine::{Engine, StopReason};

#[test]
fn a_new_engine_is_idle_at_time_zero() {
    let engine: Engine<u32> = Engine::new();
    assert_eq!(engine.now(), 0.0);
    assert_eq!(engine.executed(), 0);
    assert!(engine.is_idle());
    assert_eq!(engine.pending(), 0);
    assert_eq!(engine.next_time(), None);
    let state = engine.state();
    assert_eq!(state.now, 0.0);
    assert_eq!(state.executed, 0);
    assert!(state.is_idle());
    assert_eq!(state.next_time, None);
}

#[test]
fn a_run_executes_every_event_in_order() {
    let mut engine = Engine::new();
    engine.schedule(30.0, 3);
    engine.schedule(10.0, 1);
    engine.schedule(20.0, 2);

    let mut seen = Vec::new();
    let outcome = engine.run(|_, event| seen.push(event.payload));

    assert_eq!(seen, vec![1, 2, 3]);
    assert_eq!(outcome.executed, 3);
    assert_eq!(outcome.reason, StopReason::QueueEmpty);
    assert!(outcome.is_natural());
    assert_eq!(outcome.started_at, 0.0);
    assert_eq!(outcome.ended_at, 30.0);
    assert_eq!(outcome.remaining, 0);
    assert_eq!(engine.now(), 30.0);
    assert_eq!(engine.executed(), 3);
    assert!(engine.is_idle());
}

#[test]
fn a_handler_can_schedule_follow_up_events() {
    // A periodic process that reschedules itself, which is the pattern a
    // discrete-event model uses for a clock driven process.
    // 一个重新调度自身的周期过程，这是离散事件模型用于时钟驱动过程的模式。
    let mut engine = Engine::new();
    engine.schedule(0.0, 0_u32);

    let mut seen = Vec::new();
    let outcome = engine.run(|engine, event| {
        seen.push(event.time);
        if event.payload < 3 {
            engine
                .schedule_after(10.0, event.payload + 1)
                .expect("the delay is usable");
        }
    });

    assert_eq!(seen, vec![0.0, 10.0, 20.0, 30.0]);
    assert_eq!(outcome.executed, 4);
    assert_eq!(outcome.reason, StopReason::QueueEmpty);
    // The clock ends at the last event, not at the delay of the last
    // follow-up, which was never scheduled.
    // 时钟结束于最后一个事件，而非最后一个后续事件的延时，后者从未被调度。
    assert_eq!(outcome.ended_at, 30.0);
}

#[test]
fn events_at_the_same_instant_are_delivered_in_schedule_order() {
    let mut engine = Engine::new();
    engine.schedule(5.0, 'a');
    engine.schedule(5.0, 'b');
    engine.schedule(5.0, 'c');

    let mut seen = Vec::new();
    engine.run(|_, event| seen.push(event.payload));
    assert_eq!(seen, vec!['a', 'b', 'c']);
}

#[test]
fn a_follow_up_scheduled_for_the_same_instant_is_delivered_after_the_pending_ones() {
    // This is what makes a cascade at one instant well defined: events
    // scheduled by a handler go behind the events already queued.
    // 这正是使某一瞬时的级联有明确含义的东西：处理器所调度的事件排在已排队的事件
    // 之后。
    let mut engine = Engine::new();
    engine.schedule(5.0, "first");
    engine.schedule(5.0, "second");

    let mut seen = Vec::new();
    engine.run(|engine, event| {
        seen.push(event.payload);
        if event.payload == "first" {
            engine
                .schedule(5.0, "inserted")
                .expect("the time is usable");
        }
    });
    assert_eq!(seen, vec!["first", "second", "inserted"]);
}

#[test]
fn a_run_is_reproducible() {
    // The same inputs must give the same output, which the tie-break on the
    // schedule order guarantees.
    // 相同输入必须给出相同输出，这由调度次序上的并列打破规则保证。
    let build = || {
        let mut engine = Engine::new();
        for index in 0..20 {
            engine
                .schedule(f64::from(index % 3), index)
                .expect("the time is usable");
        }
        let mut seen = Vec::new();
        engine.run(|_, event| seen.push(event.payload));
        seen
    };
    assert_eq!(build(), build());
}

#[test]
fn a_time_limit_stops_the_run_and_leaves_the_event_pending() {
    let mut engine = Engine::new().with_time_limit(15.0);
    engine.schedule(10.0, 1);
    engine.schedule(20.0, 2);
    engine.schedule(30.0, 3);

    let mut seen = Vec::new();
    let outcome = engine.run(|_, event| seen.push(event.payload));

    assert_eq!(seen, vec![1]);
    assert_eq!(outcome.reason, StopReason::TimeLimit);
    assert!(!outcome.is_natural());
    // The event beyond the limit is still queued.
    // 超限的事件仍在队列中。
    assert_eq!(outcome.remaining, 2);
    assert_eq!(engine.next_time(), Some(20.0));
    // The clock is at the last event executed, not at the limit.
    // 时钟在最后执行的事件处，而非在限制处。
    assert_eq!(outcome.ended_at, 10.0);
}

#[test]
fn an_event_exactly_at_the_time_limit_is_executed() {
    let mut engine = Engine::new().with_time_limit(10.0);
    engine.schedule(10.0, "at limit");
    let outcome = engine.run(|_, _| {});
    assert_eq!(outcome.executed, 1);
    assert_eq!(outcome.reason, StopReason::QueueEmpty);
}

#[test]
fn an_event_limit_stops_the_run_without_discarding_events() {
    let mut engine = Engine::new().with_event_limit(2);
    for index in 0..5 {
        engine
            .schedule(f64::from(index), index)
            .expect("the time is usable");
    }

    let mut seen = Vec::new();
    let outcome = engine.run(|_, event| seen.push(event.payload));

    assert_eq!(seen, vec![0, 1]);
    assert_eq!(outcome.reason, StopReason::EventLimit);
    assert_eq!(outcome.executed, 2);
    assert_eq!(outcome.remaining, 3);
    assert_eq!(engine.executed(), 2);

    // The limit is a lifetime cap rather than a per run one, so a second
    // run at once stops without executing anything. Raising the limit lets
    // the remaining events run.
    // 该限制是生命期上限而非每次运行的上限，故紧接着的第二次运行不执行任何东西即
    // 停止。提高限制使剩余事件运行。
    let outcome = engine.run(|_, event| seen.push(event.payload));
    assert_eq!(outcome.executed, 0);
    assert_eq!(outcome.reason, StopReason::EventLimit);

    let mut continued = engine.with_event_limit(10);
    let outcome = continued.run(|_, event| seen.push(event.payload));
    assert_eq!(seen, vec![0, 1, 2, 3, 4]);
    assert_eq!(outcome.reason, StopReason::QueueEmpty);
    assert_eq!(outcome.executed, 3);
}

#[test]
fn the_event_limit_counts_across_runs() {
    // The limit is on the total number of events, so a run that already
    // reached it does nothing.
    // 该限制针对事件总数，故已达到它的运行什么也不做。
    let mut engine = Engine::new().with_event_limit(1);
    engine.schedule(1.0, "a").expect("usable");
    assert_eq!(engine.run(|_, _| {}).executed, 1);
    assert_eq!(engine.run(|_, _| {}).reason, StopReason::EventLimit);
    assert_eq!(engine.executed(), 1);
}

#[test]
fn a_stop_request_ends_the_run_after_the_current_event() {
    let mut engine = Engine::new();
    for index in 0..5 {
        engine
            .schedule(f64::from(index), index)
            .expect("the time is usable");
    }

    let mut seen = Vec::new();
    let outcome = engine.run(|engine, event| {
        seen.push(event.payload);
        if event.payload == 2 {
            engine.request_stop();
        }
    });

    // The event that requested the stop is the last one executed.
    // 请求停止的事件是最后执行的那个。
    assert_eq!(seen, vec![0, 1, 2]);
    assert_eq!(outcome.reason, StopReason::Requested);
    assert_eq!(outcome.remaining, 2);

    // The request is cleared by the run, so the next run continues.
    // 该请求由运行清除，故下次运行继续。
    let outcome = engine.run(|_, event| seen.push(event.payload));
    assert_eq!(seen, vec![0, 1, 2, 3, 4]);
    assert_eq!(outcome.reason, StopReason::QueueEmpty);
}

#[test]
fn a_stop_request_can_be_cleared_by_hand() {
    let mut engine: Engine<u8> = Engine::new();
    engine.schedule(1.0, 1).expect("usable");
    engine.request_stop();
    engine.clear_stop();
    // The run clears the flag itself, so a clear beforehand is harmless.
    // 运行自身会清除该标志，故事先清除是无害的。
    assert_eq!(engine.run(|_, _| {}).executed, 1);
}

#[test]
fn an_empty_run_does_nothing() {
    let mut engine: Engine<u8> = Engine::new();
    let outcome = engine.run(|_, _| panic!("no handler call is expected"));
    assert_eq!(outcome.executed, 0);
    assert_eq!(outcome.reason, StopReason::QueueEmpty);
    assert_eq!(outcome.duration(), 0.0);
    assert_eq!(outcome.event_rate(), None);
}

#[test]
fn the_clock_can_be_set_to_start_at_an_offset() {
    let mut engine = Engine::new();
    engine.set_now(1_000.0);
    assert_eq!(engine.now(), 1_000.0);
    // A relative schedule is measured from the offset.
    // 相对调度自该偏移量起度量。
    engine.schedule_after(5.0, "x").expect("usable");
    assert_eq!(engine.next_time(), Some(1_005.0));

    let outcome = engine.run(|_, _| {});
    assert_eq!(outcome.started_at, 1_000.0);
    assert_eq!(outcome.ended_at, 1_005.0);
    assert_eq!(outcome.duration(), 5.0);

    // An unusable time is ignored, leaving the clock alone.
    // 不可用的时间被忽略，时钟不动。
    engine.set_now(-1.0);
    assert_eq!(engine.now(), 1_005.0);
    engine.set_now(f64::NAN);
    assert_eq!(engine.now(), 1_005.0);
}

#[test]
fn the_outcome_reports_the_event_rate() {
    let mut engine = Engine::new();
    for index in 0..10 {
        engine
            .schedule(f64::from(index) * 10.0, index)
            .expect("usable");
    }
    let outcome = engine.run(|_, _| {});
    // Ten events over ninety seconds, so the rate is a ninth per second.
    // 九十秒内十个事件，故速率为每秒九分之一。
    assert_eq!(outcome.duration(), 90.0);
    assert!((outcome.event_rate().expect("a positive duration") - 10.0 / 90.0).abs() < 1e-12);
}

#[test]
fn clearing_discards_the_pending_events() {
    let mut engine = Engine::new();
    for index in 0..4 {
        engine.schedule(f64::from(index), index).expect("usable");
    }
    assert_eq!(engine.clear(), 4);
    assert!(engine.is_idle());
    assert_eq!(engine.run(|_, _| {}).executed, 0);
}

#[test]
fn the_queue_is_reachable_from_the_engine() {
    let mut engine = Engine::new();
    engine.schedule(1.0, "a").expect("usable");
    assert_eq!(engine.queue().len(), 1);
    assert_eq!(engine.queue().peek(), Some(&"a"));
    // The queue may be modified directly.
    // 队列可被直接修改。
    engine.queue_mut().clear();
    assert!(engine.is_idle());
}

#[test]
fn a_bad_time_limit_is_ignored() {
    let mut engine = Engine::new().with_time_limit(-1.0);
    engine.schedule(100.0, "x").expect("usable");
    // The run reaches the event, because no limit applies.
    // 该运行到达该事件，因为没有限制适用。
    assert_eq!(engine.run(|_, _| {}).executed, 1);

    let mut engine = Engine::new().with_time_limit(f64::NAN);
    engine.schedule(100.0, "x").expect("usable");
    assert_eq!(engine.run(|_, _| {}).executed, 1);
}

#[test]
fn an_event_limit_of_zero_is_ignored() {
    let mut engine = Engine::new().with_event_limit(0);
    engine.schedule(1.0, "x").expect("usable");
    assert_eq!(engine.run(|_, _| {}).executed, 1);
}

#[test]
fn a_negative_time_cannot_be_scheduled() {
    let mut engine = Engine::new();
    assert_eq!(engine.schedule(-1.0, "x"), None);
    assert_eq!(engine.schedule(f64::NAN, "x"), None);
    assert!(engine.is_idle());
}

#[test]
fn the_stop_reasons_report_their_names() {
    assert!(StopReason::QueueEmpty.is_natural());
    assert!(!StopReason::TimeLimit.is_natural());
    assert!(!StopReason::Requested.is_natural());
    assert!(!StopReason::EventLimit.is_natural());
    assert_eq!(StopReason::QueueEmpty.name(), "queue empty");
    assert_eq!(StopReason::TimeLimit.name(), "time limit");
    assert_eq!(StopReason::Requested.name(), "requested");
    assert_eq!(StopReason::EventLimit.name(), "event limit");
    assert_eq!(StopReason::TimeLimit.to_string(), "time limit");
}

#[test]
fn the_outcome_displays_its_shape() {
    let mut engine = Engine::new();
    engine.schedule(5.0, "x").expect("usable");
    let outcome = engine.run(|_, _| {});
    let text = outcome.to_string();
    assert!(text.contains("1 event"), "{text}");
    assert!(text.contains("queue empty"), "{text}");
}

#[test]
fn the_engine_default_is_empty() {
    let engine: Engine<u8> = Engine::default();
    assert!(engine.is_idle());
    assert_eq!(engine.now(), 0.0);
}
