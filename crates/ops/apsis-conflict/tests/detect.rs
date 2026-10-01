//! Tests for conflict detection.
//!
//! 冲突检测的测试。

use apsis_conflict::{Conflict, ConflictKind, Precedence, Schedule, detect, detect_precedence};
use apsis_resource::{Resource, ResourceId, ResourceSet, Window};
use apsis_task::{Task, TaskId};

/// A resource available from 0 to 1 000 seconds.
///
/// 一个自 0 到 1 000 秒可用的资源。
fn resource(id: usize, capacity: f64) -> Resource {
    let mut resource = Resource::new(ResourceId::new(id), capacity);
    resource
        .set_windows(&[Window::new(0.0, 1_000.0)])
        .expect("the window is usable");
    resource
}

/// A task requiring one unit of each named resource.
///
/// 一个需要每个指名资源一单位的任务。
fn task(id: usize, duration: f64, resources: &[usize]) -> Task {
    let mut task = Task::new(TaskId::new(id), duration);
    for resource in resources {
        task = task
            .try_require(ResourceId::new(*resource), 1.0)
            .expect("the resource is new");
    }
    task
}

/// The standard resource set used by the tests.
///
/// 测试所用的标准资源集。
fn resources() -> ResourceSet {
    ResourceSet::from_resources(vec![resource(0, 1.0), resource(1, 1.0)])
        .expect("the identifiers are distinct")
}

#[test]
fn a_sound_schedule_has_no_conflicts() {
    let resources = resources();
    let tasks = vec![task(0, 60.0, &[0]), task(1, 30.0, &[1])];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 0.0).expect("fits");
    schedule.place(&tasks[1], 100.0).expect("fits");

    let report = detect(&schedule, &tasks, &resources);
    assert!(report.is_clear());
    assert!(report.is_empty());
    assert!(!report.has_conflicts());
    assert_eq!(report.len(), 0);
    assert_eq!(report.placed_tasks(), 2);
    assert_eq!(report.offered_tasks(), 2);
    assert!(report.conflicting_tasks().is_empty());
    assert!(report.summary().contains("0 conflict"));
}

#[test]
fn a_capacity_conflict_is_found() {
    let resources = resources();
    let tasks = vec![task(0, 60.0, &[0]), task(1, 60.0, &[0])];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 0.0).expect("fits");
    schedule.place(&tasks[1], 30.0).expect("fits");

    let report = detect(&schedule, &tasks, &resources);
    assert!(report.has_conflicts());
    assert_eq!(report.count_of(ConflictKind::Capacity), 1);
    let conflict = report.of_kind(ConflictKind::Capacity)[0];
    assert_eq!(conflict.resource, Some(ResourceId::new(0)));
    // The conflict names both tasks, so a repair pass knows the pair.
    // 该冲突指名两个任务，故修复遍可知道这一对。
    assert!(conflict.other.is_some());
    // The instant reported is the start of the second placement.
    // 所报告的瞬时是第二处放置的开始。
    assert_eq!(conflict.at, 30.0);
    assert!(conflict.is_movable());
}

#[test]
fn back_to_back_placements_do_not_conflict() {
    let resources = resources();
    let tasks = vec![task(0, 60.0, &[0]), task(1, 60.0, &[0])];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 0.0).expect("fits");
    schedule.place(&tasks[1], 60.0).expect("fits");
    assert!(detect(&schedule, &tasks, &resources).is_clear());
}

#[test]
fn a_window_conflict_is_found_at_the_start() {
    let resources = resources();
    let tasks = [task(0, 60.0, &[0]).with_window(100.0, 1_000.0)];
    let mut schedule = Schedule::new();
    // The schedule refuses a placement before the window, so the conflict is
    // constructed directly to test the detector.
    // 调度拒绝窗口之前的放置，故直接构建该冲突以测试检测器。
    schedule.place(&tasks[0], 100.0).expect("fits");
    // A task list whose window has moved since the schedule was built.
    // 一个自调度构建以来窗口已移动的任务列表。
    let moved = vec![task(0, 60.0, &[0]).with_window(200.0, 1_000.0)];

    let report = detect(&schedule, &moved, &resources);
    assert_eq!(report.count_of(ConflictKind::Window), 1);
    let conflict = report.of_kind(ConflictKind::Window)[0];
    assert_eq!(conflict.at, 100.0);
    assert!(!conflict.is_movable());
}

#[test]
fn a_window_conflict_is_found_at_the_end() {
    let resources = resources();
    let tasks = [task(0, 60.0, &[0]).with_window(0.0, 1_000.0)];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 100.0).expect("fits");
    // A task list whose window closes earlier than the placement ends.
    // 一个窗口比放置结束更早关闭的任务列表。
    let shortened = vec![task(0, 60.0, &[0]).with_window(0.0, 120.0)];

    let report = detect(&schedule, &shortened, &resources);
    assert_eq!(report.count_of(ConflictKind::Window), 1);
    let conflict = report.of_kind(ConflictKind::Window)[0];
    // The instant reported is the end of the placement.
    // 所报告的瞬时是该放置的结束。
    assert_eq!(conflict.at, 160.0);
}

#[test]
fn a_requirement_conflict_is_found_for_a_missing_task() {
    let resources = resources();
    let tasks = [task(0, 60.0, &[0])];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 0.0).expect("fits");

    // The task list is empty, so the placement has no task.
    // 任务列表为空，故该放置没有任务。
    let report = detect(&schedule, &[], &resources);
    assert_eq!(report.count_of(ConflictKind::Requirement), 1);
    assert_eq!(report.placed_tasks(), 1);
    assert_eq!(report.offered_tasks(), 0);
}

#[test]
fn a_requirement_conflict_is_found_for_a_duration_mismatch() {
    let resources = resources();
    let original = task(0, 60.0, &[0]);
    let mut schedule = Schedule::new();
    schedule.place(&original, 0.0).expect("fits");

    let changed = vec![task(0, 90.0, &[0])];
    let report = detect(&schedule, &changed, &resources);
    assert_eq!(report.count_of(ConflictKind::Requirement), 1);
}

#[test]
fn a_requirement_conflict_is_found_for_a_missing_resource() {
    let resources = resources();
    let tasks = vec![task(0, 60.0, &[9])];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 0.0).expect("fits");

    let report = detect(&schedule, &tasks, &resources);
    assert_eq!(report.count_of(ConflictKind::Requirement), 1);
    let conflict = report.of_kind(ConflictKind::Requirement)[0];
    assert_eq!(conflict.resource, Some(ResourceId::new(9)));
}

#[test]
fn a_requirement_conflict_is_found_for_an_impossible_demand() {
    let resources = resources();
    // A demand of two on a resource of capacity one can never be met.
    // 容量为一的资源上需求为二永远无法满足。
    let tasks = vec![
        Task::new(TaskId::new(0), 60.0)
            .try_require(ResourceId::new(0), 2.0)
            .expect("the resource is new"),
    ];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 0.0).expect("fits");

    let report = detect(&schedule, &tasks, &resources);
    assert_eq!(report.count_of(ConflictKind::Requirement), 1);
    let conflict = report.of_kind(ConflictKind::Requirement)[0];
    assert!(conflict.detail.contains("exceeds the capacity"));
}

#[test]
fn an_availability_conflict_is_found_outside_the_windows() {
    // The resource is available only from 200 to 300 seconds.
    // 该资源只在 200 至 300 秒可用。
    let mut limited = Resource::new(ResourceId::new(0), 1.0);
    limited
        .set_windows(&[Window::new(200.0, 300.0)])
        .expect("the window is usable");
    let resources = ResourceSet::from_resources(vec![limited]).expect("distinct");

    let tasks = vec![task(0, 60.0, &[0])];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 0.0).expect("fits");

    let report = detect(&schedule, &tasks, &resources);
    assert_eq!(report.count_of(ConflictKind::Availability), 1);
    let conflict = report.of_kind(ConflictKind::Availability)[0];
    assert_eq!(conflict.resource, Some(ResourceId::new(0)));
    assert!(conflict.is_movable());
}

#[test]
fn an_availability_conflict_is_found_for_a_placement_that_overruns_a_window() {
    // The resource is available only from 0 to 120 seconds, so a placement
    // from 100 for 60 seconds overruns it.
    // 该资源只在 0 至 120 秒可用，故自 100 起持续 60 秒的放置会超出它。
    let mut limited = Resource::new(ResourceId::new(0), 1.0);
    limited
        .set_windows(&[Window::new(0.0, 120.0)])
        .expect("the window is usable");
    let resources = ResourceSet::from_resources(vec![limited]).expect("distinct");

    let tasks = vec![task(0, 60.0, &[0])];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 100.0).expect("fits");
    assert_eq!(
        detect(&schedule, &tasks, &resources).count_of(ConflictKind::Availability),
        1
    );
}

#[test]
fn a_placement_inside_the_availability_has_no_conflict() {
    let resources = resources();
    let tasks = vec![task(0, 60.0, &[0])];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 200.0).expect("fits");
    assert!(detect(&schedule, &tasks, &resources).is_clear());
}

#[test]
fn several_conflicts_are_reported_together() {
    // The detector reports every problem, not just the first.
    // 检测器报告每个问题，而不只是第一个。
    let resources = resources();
    let tasks = [task(0, 60.0, &[0]), task(1, 60.0, &[0])];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 0.0).expect("fits");
    schedule.place(&tasks[1], 30.0).expect("fits");

    // A task list that also moves the window of the first task.
    // 一个还移动了第一个任务窗口的任务列表。
    let changed = vec![
        task(0, 60.0, &[0]).with_window(100.0, 1_000.0),
        task(1, 60.0, &[0]),
    ];
    let report = detect(&schedule, &changed, &resources);
    assert!(report.count_of(ConflictKind::Capacity) >= 1);
    assert!(report.count_of(ConflictKind::Window) >= 1);
    assert!(report.len() >= 2);
}

#[test]
fn the_report_is_ordered_by_time() {
    let resources = resources();
    // The placements are made inside a wide window, then the task list is
    // changed so that both placements fall before the new window opens. The
    // schedule refuses such a placement, which is why the changed list is
    // used for the check.
    // 各放置在一个宽窗口内作出，然后任务列表被改变，使两处放置都落在新窗口打开之前。
    // 调度拒绝这样的放置，故检查使用改变后的列表。
    let placed = [task(0, 60.0, &[0]), task(1, 60.0, &[1])];
    let mut schedule = Schedule::new();
    schedule.place(&placed[0], 100.0).expect("fits");
    schedule.place(&placed[1], 300.0).expect("fits");

    let changed = vec![
        task(0, 60.0, &[0]).with_window(500.0, 1_000.0),
        task(1, 60.0, &[1]).with_window(500.0, 1_000.0),
    ];
    let report = detect(&schedule, &changed, &resources);
    assert!(report.len() >= 2);
    let times: Vec<f64> = report.conflicts().iter().map(|c| c.at).collect();
    let mut sorted = times.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    assert_eq!(times, sorted);
}

#[test]
fn the_report_lists_the_conflicting_tasks_once_each() {
    let resources = resources();
    let tasks = vec![task(0, 60.0, &[0]), task(1, 60.0, &[0])];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 0.0).expect("fits");
    schedule.place(&tasks[1], 30.0).expect("fits");

    let report = detect(&schedule, &tasks, &resources);
    let tasks_involved = report.conflicting_tasks();
    // Both tasks are involved, and each appears once.
    // 两个任务都涉及，且各自出现一次。
    assert!(tasks_involved.contains(&TaskId::new(0)));
    assert!(tasks_involved.contains(&TaskId::new(1)));
    assert_eq!(tasks_involved.len(), 2);
}

#[test]
fn the_counts_by_kind_sum_to_the_total() {
    let resources = resources();
    let tasks = vec![task(0, 60.0, &[0]), task(1, 60.0, &[0])];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 0.0).expect("fits");
    schedule.place(&tasks[1], 30.0).expect("fits");

    let report = detect(&schedule, &tasks, &resources);
    let summed = report.count_of(ConflictKind::Capacity)
        + report.count_of(ConflictKind::Window)
        + report.count_of(ConflictKind::Requirement)
        + report.count_of(ConflictKind::Availability)
        + report.count_of(ConflictKind::Precedence);
    assert_eq!(summed, report.len());
}

#[test]
fn a_precedence_violation_is_found() {
    let tasks = [task(0, 60.0, &[0]), task(1, 60.0, &[1])];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 100.0).expect("fits");
    // The second task must follow the first, but starts inside it.
    // 第二个任务必须跟随第一个，但起始于其中。
    schedule.place(&tasks[1], 120.0).expect("fits");

    let conflicts = detect_precedence(
        &schedule,
        &[Precedence::new(TaskId::new(0), TaskId::new(1))],
    );
    assert_eq!(conflicts.len(), 1);
    let conflict = &conflicts[0];
    assert_eq!(conflict.kind, ConflictKind::Precedence);
    assert_eq!(conflict.task, TaskId::new(1));
    assert_eq!(conflict.other, Some(TaskId::new(0)));
    assert_eq!(conflict.at, 120.0);
}

#[test]
fn a_satisfied_precedence_has_no_conflict() {
    let tasks = [task(0, 60.0, &[0]), task(1, 60.0, &[1])];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 100.0).expect("fits");
    // Starting exactly at the end of the first is allowed, because the
    // placement interval is half-open.
    // 恰在第一个结束处开始是允许的，因为放置区间是半开的。
    schedule.place(&tasks[1], 160.0).expect("fits");
    assert!(
        detect_precedence(
            &schedule,
            &[Precedence::new(TaskId::new(0), TaskId::new(1))]
        )
        .is_empty()
    );
}

#[test]
fn a_precedence_over_an_unplaced_task_is_ignored() {
    let tasks = [task(0, 60.0, &[0])];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 100.0).expect("fits");
    // The second task is not placed, so nothing has been promised.
    // 第二个任务未被放置，故没有承诺任何东西。
    assert!(
        detect_precedence(
            &schedule,
            &[Precedence::new(TaskId::new(0), TaskId::new(1))]
        )
        .is_empty()
    );
    assert!(
        detect_precedence(
            &schedule,
            &[Precedence::new(TaskId::new(1), TaskId::new(0))]
        )
        .is_empty()
    );
}

#[test]
fn a_conflict_displays_its_parts() {
    let conflict = Conflict::new(
        ConflictKind::Capacity,
        TaskId::new(0),
        30.0,
        "over committed",
    )
    .with_other(TaskId::new(1))
    .with_resource(ResourceId::new(2));
    let text = conflict.to_string();
    assert!(text.contains("capacity"), "{text}");
    assert!(text.contains("task 0"), "{text}");
    assert!(text.contains("task 1"), "{text}");
    assert!(text.contains("resource 2"), "{text}");
    assert!(text.contains("30"), "{text}");
}

#[test]
fn the_conflict_kinds_display_their_names() {
    assert_eq!(ConflictKind::Capacity.to_string(), "capacity");
    assert_eq!(ConflictKind::Window.to_string(), "window");
    assert_eq!(ConflictKind::Requirement.to_string(), "requirement");
    assert_eq!(ConflictKind::Availability.to_string(), "availability");
    assert_eq!(ConflictKind::Precedence.to_string(), "precedence");
}

#[test]
fn movability_is_reported_by_kind() {
    assert!(Conflict::new(ConflictKind::Capacity, TaskId::new(0), 0.0, "").is_movable());
    assert!(Conflict::new(ConflictKind::Availability, TaskId::new(0), 0.0, "").is_movable());
    assert!(!Conflict::new(ConflictKind::Window, TaskId::new(0), 0.0, "").is_movable());
    assert!(!Conflict::new(ConflictKind::Requirement, TaskId::new(0), 0.0, "").is_movable());
    assert!(!Conflict::new(ConflictKind::Precedence, TaskId::new(0), 0.0, "").is_movable());
}

#[test]
fn the_report_can_be_built_directly() {
    use apsis_conflict::ConflictReport;
    let report = ConflictReport::new(
        vec![Conflict::new(
            ConflictKind::Window,
            TaskId::new(0),
            5.0,
            "late",
        )],
        3,
        2,
    );
    assert_eq!(report.len(), 1);
    assert_eq!(report.offered_tasks(), 3);
    assert_eq!(report.placed_tasks(), 2);
    assert!(report.summary().contains("1 conflict"));
}
