//! Tests for the simulation facade.
//!
//! 仿真门面的测试。

use apsis_sim::{
    Activity, RecordKind, RunReport, Scenario, Simulation, SimulationErrorKind, StopReason,
};

/// Builds a scenario with one station and a set of tasks.
///
/// 构建一个含一座站与一组任务的场景。
fn scenario_with(tasks: &[(usize, f64, f64)]) -> Scenario {
    let mut text = String::from("scenario test\nresource 0 capacity 1\nwindow 0 0 1000\n");
    for &(id, duration, priority) in tasks {
        text.push_str(&format!("task {id} duration {duration}\n"));
        text.push_str(&format!("require {id} 0 1\n"));
        text.push_str(&format!("task_priority {id} {priority}\n"));
    }
    Scenario::parse(&text).expect("the text is well formed")
}

#[test]
fn a_scenario_is_prepared_and_planned() {
    let simulation =
        Simulation::prepare(scenario_with(&[(0, 60.0, 1.0), (1, 90.0, 2.0)])).expect("usable");
    assert!(simulation.has_full_plan());
    assert_eq!(simulation.schedule().len(), 2);
    assert_eq!(simulation.activity_count(), 4);
    assert_eq!(simulation.timeline().len(), 4);
    assert!(simulation.deferred().is_empty());
    assert!(simulation.infeasible().is_empty());
    assert!(simulation.honours_mandatory());
    assert!(simulation.honours_precedence());
}

#[test]
fn the_timeline_has_a_start_and_an_end_for_each_task() {
    let simulation = Simulation::prepare(scenario_with(&[(0, 60.0, 1.0)])).expect("usable");
    let timeline = simulation.timeline();
    // In time order: the start precedes the end.
    // 按时间次序：开始在结束之前。
    assert_eq!(timeline.len(), 2);
    assert!(timeline[0].1.is_start());
    assert_eq!(timeline[0].1.task(), apsis_task::TaskId::new(0));
    assert!(timeline[1].1.is_end());
    assert!(timeline[1].0 > timeline[0].0);
}

#[test]
fn a_run_executes_every_activity() {
    let simulation =
        Simulation::prepare(scenario_with(&[(0, 60.0, 1.0), (1, 90.0, 2.0)])).expect("usable");
    let mut session = simulation.start();
    let report = session.run(|_, _, _| {});
    assert_eq!(report.executed(), 4);
    assert!(report.is_complete());
    assert_eq!(report.outcome.reason, StopReason::QueueEmpty);
    assert_eq!(report.trace.len(), 0);
}

#[test]
fn a_handler_can_record_each_activity() {
    let simulation =
        Simulation::prepare(scenario_with(&[(0, 60.0, 1.0), (1, 90.0, 2.0)])).expect("usable");
    let mut session = simulation.start();
    let mut starts = 0_u32;
    let mut ends = 0_u32;
    let report = session.run(|_engine, recorder, event| {
        if event.payload.is_start() {
            starts += 1;
            recorder.note(event.time, "started", event.payload.task().index() as f64);
        } else {
            ends += 1;
        }
    });
    assert_eq!(starts, 2);
    assert_eq!(ends, 2);
    assert_eq!(report.trace.len(), 2);
    assert_eq!(report.trace.of_kind(RecordKind::State).len(), 2);
}

#[test]
fn the_run_starts_at_the_scenario_epoch() {
    let mut scenario = scenario_with(&[(0, 60.0, 1.0)]);
    scenario.set_epoch(1_000.0).expect("the epoch is usable");
    let simulation = Simulation::prepare(scenario).expect("usable");
    let mut session = simulation.start();
    assert_eq!(session.now(), 1_000.0);
    let report = session.run(|_, _, _| {});
    // The timeline events are absolute, so the run reaches the end event.
    // 时间线事件是绝对的，故运行到达结束事件。
    assert!(report.outcome.ended_at > 1_000.0);
}

#[test]
fn a_time_limit_stops_the_run_early() {
    let mut scenario = scenario_with(&[(0, 60.0, 1.0)]);
    scenario.set_time_limit(1.0).expect("the limit is usable");
    let simulation = Simulation::prepare(scenario).expect("usable");
    let mut session = simulation.start();
    let report = session.run(|_, _, _| {});
    assert_eq!(report.outcome.reason, StopReason::TimeLimit);
    assert!(!report.is_complete());
    assert_eq!(report.trace.len(), 0);
}

#[test]
fn an_event_limit_stops_the_run_early() {
    let mut scenario = scenario_with(&[(0, 60.0, 1.0)]);
    scenario = scenario.with_event_limit(1);
    let simulation = Simulation::prepare(scenario).expect("usable");
    let mut session = simulation.start();
    let report = session.run(|_, _, _| {});
    assert_eq!(report.outcome.reason, StopReason::EventLimit);
    assert_eq!(report.executed(), 1);
}

#[test]
fn a_run_is_reproducible_from_one_prepared_simulation() {
    let simulation =
        Simulation::prepare(scenario_with(&[(0, 60.0, 1.0), (1, 90.0, 2.0)])).expect("usable");
    let collect = || {
        let mut session = simulation.start();
        let mut seen = Vec::new();
        session.run(|_, _, event| seen.push((event.time, event.payload)));
        seen
    };
    assert_eq!(collect(), collect());
}

#[test]
fn a_session_leaves_the_prepared_simulation_alone() {
    let simulation = Simulation::prepare(scenario_with(&[(0, 60.0, 1.0)])).expect("usable");
    let mut first = simulation.start();
    first.run(|_, _, _| {});
    // A second session starts from the full timeline again.
    // 第二个会话又从完整的时间线开始。
    let mut second = simulation.start();
    assert_eq!(second.run(|_, _, _| {}).executed(), 2);
}

#[test]
fn a_handler_may_schedule_further_activity() {
    // The engine interleaves events the handler schedules with the timeline.
    // 引擎把处理器所调度的事件与时间线交织。
    let simulation = Simulation::prepare(scenario_with(&[(0, 60.0, 1.0)])).expect("usable");
    let mut session = simulation.start();
    let mut extra = 0_u32;
    let report = session.run(|engine, _, event| {
        if matches!(event.payload, Activity::Start { .. }) {
            engine
                .schedule(event.time + 30.0, Activity::end(event.payload.task()))
                .expect("the time is usable");
            extra += 1;
        }
    });
    assert_eq!(extra, 1);
    // Two timeline events plus the one the handler scheduled.
    // 两个时间线事件加上处理器所调度的那个。
    assert_eq!(report.executed(), 3);
}

#[test]
fn deferred_tasks_are_reported() {
    // Two tasks of 60 s in a 100 s window: only one fits.
    // 一百秒窗口中的两个六十秒任务：只有一个放得下。
    let scenario = Scenario::parse(
        "scenario tight\n\
         resource 0 capacity 1\n\
         window 0 0 100\n\
         task 0 duration 60\n\
         require 0 0 1\n\
         task 1 duration 60\n\
         require 1 0 1\n",
    )
    .expect("the text is well formed");
    let simulation = Simulation::prepare(scenario).expect("usable");
    assert!(!simulation.has_full_plan());
    assert_eq!(simulation.schedule().len(), 1);
    assert_eq!(simulation.deferred().len(), 1);
}

#[test]
fn precedence_is_honoured_in_the_timeline() {
    let scenario = Scenario::parse(
        "scenario chain\n\
         resource 0 capacity 1\n\
         window 0 0 1000\n\
         task 0 duration 60\n\
         require 0 0 1\n\
         task 1 duration 60\n\
         require 1 0 1\n\
         precedence 0 1\n",
    )
    .expect("the text is well formed");
    let simulation = Simulation::prepare(scenario).expect("usable");
    assert!(simulation.honours_precedence());
    let schedule = simulation.schedule();
    let first = schedule
        .placement_of(apsis_task::TaskId::new(0))
        .expect("placed");
    let second = schedule
        .placement_of(apsis_task::TaskId::new(1))
        .expect("placed");
    assert!(second.start >= first.end());
}

#[test]
fn the_schedule_evaluates() {
    let simulation =
        Simulation::prepare(scenario_with(&[(0, 60.0, 1.0), (1, 90.0, 2.0)])).expect("usable");
    let evaluation = simulation.evaluate();
    assert_eq!(evaluation.placed_count, 2);
    assert_eq!(evaluation.offered_count, 2);
    assert_eq!(evaluation.scheduled_seconds, 150.0);
}

#[test]
fn an_unusable_scenario_is_refused() {
    let mut scenario = Scenario::new("");
    // An empty name is refused by validation.
    // 空名称被校验拒绝。
    scenario
        .add_resource(apsis_resource::Resource::new(
            apsis_resource::ResourceId::new(0),
            1.0,
        ))
        .expect("the resource is usable");
    let error = Simulation::prepare(scenario).expect_err("the scenario is unusable");
    assert_eq!(error.kind(), SimulationErrorKind::InvalidScenario);
}

#[test]
fn a_plan_naming_a_task_outside_the_scenario_is_refused() {
    // A scheduling result that places a task the scenario does not hold is
    // inconsistent and must be rejected rather than run.
    // 一个放置了场景所不持有任务的调度结果是不一致的，必须被拒绝而非运行。
    let mut scenario = Scenario::new("mismatch");
    scenario
        .add_resource(apsis_resource::Resource::new(
            apsis_resource::ResourceId::new(0),
            1.0,
        ))
        .expect("the resource is usable");
    scenario
        .add_task(apsis_task::Task::new(apsis_task::TaskId::new(0), 60.0))
        .expect("the task is usable");

    let mut schedule = apsis_schedule::Schedule::new();
    // Task 99 is not in the scenario. A placement records the duration, so
    // it is placed directly rather than through a task.
    // 任务 99 不在场景中。放置记录时长，故直接放置它而非经任务。
    schedule
        .place(
            &apsis_task::Task::new(apsis_task::TaskId::new(99), 60.0),
            0.0,
        )
        .expect("the placement fits");
    let result = apsis_scheduler::SchedulingResult {
        schedule,
        outcomes: Vec::new(),
        improved: Vec::new(),
    };
    let error = Simulation::from_plan(scenario, result).expect_err("the plan is inconsistent");
    assert_eq!(error.kind(), SimulationErrorKind::InconsistentPlan);
}

#[test]
fn an_empty_plan_is_accepted() {
    // A scenario whose tasks none can be placed still prepares; the
    // resulting simulation simply has an empty timeline.
    // 没有一个任务放得下的场景仍可准备；所得仿真只是有一条空时间线。
    let scenario = Scenario::new("empty");
    let result =
        apsis_scheduler::Planner::new().schedule(&[], &scenario.resource_set().expect("ok"));
    let simulation = Simulation::from_plan(scenario, result).expect("the plan is consistent");
    assert_eq!(simulation.activity_count(), 0);
    assert!(simulation.has_full_plan());
}

#[test]
fn the_activity_type_reports_itself() {
    let start = Activity::start(apsis_task::TaskId::new(3));
    let end = Activity::end(apsis_task::TaskId::new(3));
    assert!(start.is_start());
    assert!(!start.is_end());
    assert!(end.is_end());
    assert_eq!(start.task(), apsis_task::TaskId::new(3));
    assert_eq!(end.task(), apsis_task::TaskId::new(3));
    assert!(start.to_string().contains("start"));
    assert!(end.to_string().contains("end"));
}

#[test]
fn the_session_can_record_without_running() {
    let simulation = Simulation::prepare(scenario_with(&[(0, 60.0, 1.0)])).expect("usable");
    let mut session = simulation.start();
    assert!(session.note(1.0, "manual", 5.0));
    assert!(session.record(2.0, RecordKind::Warning, "manual", 6.0));
    assert_eq!(session.recorder().len(), 2);
    // A run takes the trace, so the manual records come out with it.
    // 一次运行取走轨迹，故手工记录随之而出。
    let report: RunReport = session.run(|_, _, _| {});
    assert_eq!(report.trace.len(), 2);
    assert_eq!(report.warning_count(), 1);
}

#[test]
fn the_engine_is_reachable_from_the_session() {
    let simulation = Simulation::prepare(scenario_with(&[(0, 60.0, 1.0)])).expect("usable");
    let mut session = simulation.start();
    assert!(!session.engine().is_idle());
    assert_eq!(session.engine().pending(), 2);
    session.engine_mut().clear();
    assert!(session.engine().is_idle());
}

#[test]
fn the_simulation_exposes_its_scenario_and_resources() {
    let simulation = Simulation::prepare(scenario_with(&[(0, 60.0, 1.0)])).expect("usable");
    assert_eq!(simulation.scenario().name(), "test");
    assert_eq!(simulation.resources().len(), 1);
    assert_eq!(simulation.tasks().len(), 1);
    assert_eq!(simulation.plan().outcomes.len(), 1);
}
