//! Tests for the schedule representation and its evaluation.

use apsis_resource::{Resource, ResourceId, ResourceSet, Window};
use apsis_schedule::{Evaluation, Placement, Schedule, ScheduleErrorKind};
use apsis_task::{Task, TaskId};

/// A resource with the given identifier available from 0 to 1 000 seconds.
fn resource(id: usize, capacity: f64) -> Resource {
    let mut resource = Resource::new(ResourceId::new(id), capacity);
    resource
        .set_windows(&[Window::new(0.0, 1_000.0)])
        .expect("the window is usable");
    resource
}

/// A task requiring one unit of the given resources.
fn task(id: usize, duration: f64, resources: &[usize]) -> Task {
    let mut task = Task::new(TaskId::new(id), duration);
    for resource in resources {
        task = task
            .try_require(ResourceId::new(*resource), 1.0)
            .expect("the resource is new");
    }
    task
}

#[test]
fn a_new_schedule_is_empty() {
    let schedule = Schedule::new();
    assert!(schedule.is_empty());
    assert_eq!(schedule.len(), 0);
    assert!(schedule.placements().is_empty());
    assert_eq!(schedule.earliest_start(), None);
    assert_eq!(schedule.latest_end(), None);
    assert_eq!(schedule.scheduled_seconds(), 0.0);
}

#[test]
fn placing_a_task_records_it() {
    let mut schedule = Schedule::new();
    let task = task(0, 60.0, &[0]);
    schedule.place(&task, 100.0).expect("the task fits");

    assert_eq!(schedule.len(), 1);
    assert!(schedule.is_placed(TaskId::new(0)));
    let placement = schedule.placement_of(TaskId::new(0)).expect("placed");
    assert_eq!(placement.task, TaskId::new(0));
    assert_eq!(placement.start, 100.0);
    assert_eq!(placement.duration, 60.0);
    assert_eq!(placement.end(), 160.0);
    assert_eq!(schedule.scheduled_seconds(), 60.0);
    assert_eq!(schedule.earliest_start(), Some(100.0));
    assert_eq!(schedule.latest_end(), Some(160.0));
}

#[test]
fn a_task_can_be_placed_only_once() {
    let mut schedule = Schedule::new();
    let task = task(0, 60.0, &[0]);
    schedule.place(&task, 100.0).expect("the task fits");
    let error = schedule
        .place(&task, 200.0)
        .expect_err("the task is already placed");
    assert_eq!(error.kind(), ScheduleErrorKind::AlreadyPlaced);
    assert_eq!(schedule.len(), 1);
}

#[test]
fn a_placement_before_the_window_is_rejected() {
    let mut schedule = Schedule::new();
    let task = task(0, 60.0, &[0]).with_window(100.0, 1_000.0);
    let error = schedule
        .place(&task, 50.0)
        .expect_err("the placement is too early");
    assert_eq!(error.kind(), ScheduleErrorKind::BeforeWindow);
    assert!(schedule.is_empty());
}

#[test]
fn a_placement_after_the_window_is_rejected() {
    let mut schedule = Schedule::new();
    let task = task(0, 60.0, &[0]).with_window(0.0, 100.0);
    // Starting at fifty would end at one hundred and ten, past the window.
    let error = schedule
        .place(&task, 50.0)
        .expect_err("the placement is too late");
    assert_eq!(error.kind(), ScheduleErrorKind::AfterWindow);
    // Starting at forty ends exactly at the window close, which is allowed.
    assert!(schedule.place(&task, 40.0).is_ok());
}

#[test]
fn a_non_finite_start_is_rejected() {
    let mut schedule = Schedule::new();
    let task = task(0, 60.0, &[0]);
    for start in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let error = schedule
            .place(&task, start)
            .expect_err("the start is not finite");
        assert_eq!(
            error.kind(),
            ScheduleErrorKind::InvalidStart,
            "start {start}"
        );
    }
    assert!(schedule.is_empty());
}

#[test]
fn a_task_can_be_unplaced() {
    let mut schedule = Schedule::new();
    let first = task(0, 60.0, &[0]);
    let second = task(1, 30.0, &[0]);
    schedule.place(&first, 0.0).expect("fits");
    schedule.place(&second, 100.0).expect("fits");
    assert_eq!(schedule.len(), 2);

    let removed = schedule.unplace(TaskId::new(0)).expect("placed");
    assert_eq!(removed.task, TaskId::new(0));
    assert_eq!(schedule.len(), 1);
    assert!(!schedule.is_placed(TaskId::new(0)));
    // The remaining placement is still found, so the index was rebuilt.
    assert!(schedule.is_placed(TaskId::new(1)));
    assert_eq!(
        schedule
            .placement_of(TaskId::new(1))
            .expect("still placed")
            .start,
        100.0
    );

    assert_eq!(
        schedule.unplace(TaskId::new(0)).expect_err("gone").kind(),
        ScheduleErrorKind::NotPlaced
    );
}

#[test]
fn the_index_survives_several_removals() {
    let mut schedule = Schedule::new();
    let tasks: Vec<Task> = (0..5).map(|index| task(index, 10.0, &[index])).collect();
    for (index, task) in tasks.iter().enumerate() {
        schedule
            .place(task, index as f64 * 100.0)
            .expect("the task fits");
    }
    // Remove from the middle, which shifts every later placement.
    schedule.unplace(TaskId::new(2)).expect("placed");
    for index in [0_usize, 1, 3, 4] {
        let placement = schedule
            .placement_of(TaskId::new(index))
            .unwrap_or_else(|| panic!("task {index} should still be placed"));
        assert_eq!(placement.start, index as f64 * 100.0);
    }
    assert_eq!(schedule.len(), 4);
}

#[test]
fn placements_on_a_resource_are_found_in_time_order() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0), resource(1, 1.0)])
        .expect("the identifiers are distinct");
    let tasks = vec![
        task(0, 60.0, &[0]),
        task(1, 30.0, &[1]),
        task(2, 20.0, &[0]),
    ];

    let mut schedule = Schedule::new();
    // Placed out of time order.
    schedule.place(&tasks[2], 200.0).expect("fits");
    schedule.place(&tasks[0], 0.0).expect("fits");
    schedule.place(&tasks[1], 100.0).expect("fits");

    let on_zero = schedule.placements_on(ResourceId::new(0), &tasks);
    assert_eq!(on_zero.len(), 2);
    assert_eq!(on_zero[0].task, TaskId::new(0));
    assert_eq!(on_zero[1].task, TaskId::new(2));

    let on_one = schedule.placements_on(ResourceId::new(1), &tasks);
    assert_eq!(on_one.len(), 1);
    assert_eq!(on_one[0].task, TaskId::new(1));

    assert!(
        schedule
            .placements_on(ResourceId::new(9), &tasks)
            .is_empty()
    );
    let _ = resources;
}

#[test]
fn overlapping_placements_are_detected() {
    let mut schedule = Schedule::new();
    let first = task(0, 60.0, &[0]);
    let second = task(1, 60.0, &[1]);
    schedule.place(&first, 0.0).expect("fits");
    schedule.place(&second, 30.0).expect("fits");
    // The two overlap in time, although they use different resources, so the
    // schedule reports an overlap and the capacity check allows it.
    assert!(schedule.has_overlapping_placements());
}

#[test]
fn back_to_back_placements_do_not_overlap() {
    let mut schedule = Schedule::new();
    let first = task(0, 60.0, &[0]);
    let second = task(1, 60.0, &[0]);
    schedule.place(&first, 0.0).expect("fits");
    schedule.place(&second, 60.0).expect("fits");
    assert!(!schedule.has_overlapping_placements());
}

#[test]
fn the_capacity_check_accepts_an_exclusive_resource_used_once() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let tasks = vec![task(0, 60.0, &[0]), task(1, 60.0, &[0])];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 0.0).expect("fits");
    schedule.place(&tasks[1], 60.0).expect("fits");
    assert!(schedule.validate_capacity(&tasks, &resources).is_ok());
}

#[test]
fn the_capacity_check_rejects_an_exclusive_resource_used_twice() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let tasks = vec![task(0, 60.0, &[0]), task(1, 60.0, &[0])];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 0.0).expect("fits");
    schedule.place(&tasks[1], 30.0).expect("fits");
    let error = schedule
        .validate_capacity(&tasks, &resources)
        .expect_err("the resource is overbooked");
    assert_eq!(error.kind(), ScheduleErrorKind::CapacityExceeded);
}

#[test]
fn a_shared_resource_allows_two_tasks_within_the_capacity() {
    // A resource of capacity two holds two unit tasks at once.
    let resources = ResourceSet::from_resources(vec![resource(0, 2.0)]).expect("one");
    let tasks = vec![task(0, 60.0, &[0]), task(1, 60.0, &[0])];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 0.0).expect("fits");
    schedule.place(&tasks[1], 30.0).expect("fits");
    assert!(schedule.validate_capacity(&tasks, &resources).is_ok());

    // A third would exceed it.
    let third = task(2, 60.0, &[0]);
    schedule.place(&third, 45.0).expect("fits");
    let mut all = tasks.clone();
    all.push(third);
    assert_eq!(
        schedule
            .validate_capacity(&all, &resources)
            .expect_err("three exceeds two")
            .kind(),
        ScheduleErrorKind::CapacityExceeded
    );
}

#[test]
fn fractional_demands_share_a_resource() {
    // Four tasks of a quarter unit each fill a resource of capacity one.
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let tasks: Vec<Task> = (0..4)
        .map(|index| {
            Task::new(TaskId::new(index), 60.0)
                .try_require(ResourceId::new(0), 0.25)
                .expect("the resource is new")
        })
        .collect();
    let mut schedule = Schedule::new();
    for task in &tasks {
        schedule.place(task, 0.0).expect("fits");
    }
    assert!(schedule.validate_capacity(&tasks, &resources).is_ok());

    let fifth = Task::new(TaskId::new(9), 60.0)
        .try_require(ResourceId::new(0), 0.25)
        .expect("the resource is new");
    schedule.place(&fifth, 0.0).expect("fits");
    let mut all = tasks.clone();
    all.push(fifth);
    assert_eq!(
        schedule
            .validate_capacity(&all, &resources)
            .expect_err("five quarters exceeds one")
            .kind(),
        ScheduleErrorKind::CapacityExceeded
    );
}

#[test]
fn the_capacity_check_rejects_a_placement_without_its_task() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let tasks = [task(0, 60.0, &[0])];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 0.0).expect("fits");
    // The task list does not contain the placed task.
    assert_eq!(
        schedule
            .validate_capacity(&[], &resources)
            .expect_err("the task is missing")
            .kind(),
        ScheduleErrorKind::RequirementMismatch
    );
}

#[test]
fn the_capacity_check_rejects_a_duration_mismatch() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let original = task(0, 60.0, &[0]);
    let mut schedule = Schedule::new();
    schedule.place(&original, 0.0).expect("fits");
    // A task with the same identifier but a different duration.
    let different = task(0, 90.0, &[0]);
    assert_eq!(
        schedule
            .validate_capacity(&[different], &resources)
            .expect_err("the durations differ")
            .kind(),
        ScheduleErrorKind::RequirementMismatch
    );
}

#[test]
fn the_evaluation_reports_the_counts_and_the_makespan() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let tasks = vec![task(0, 60.0, &[0]), task(1, 30.0, &[0])];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 0.0).expect("fits");
    schedule.place(&tasks[1], 100.0).expect("fits");

    let evaluation = schedule.evaluate(&tasks, &resources);
    assert_eq!(evaluation.placed_count, 2);
    assert_eq!(evaluation.offered_count, 2);
    assert_eq!(evaluation.scheduled_seconds, 90.0);
    assert_eq!(evaluation.makespan_start, 0.0);
    assert_eq!(evaluation.makespan_end, 130.0);
    assert_eq!(evaluation.makespan(), 130.0);
    assert_eq!(evaluation.fill_rate(), Some(1.0));
    assert!(evaluation.is_complete());
}

#[test]
fn the_evaluation_reports_resource_usage() {
    let resources =
        ResourceSet::from_resources(vec![resource(0, 2.0), resource(1, 1.0)]).expect("distinct");
    let tasks = vec![
        task(0, 60.0, &[0]),
        task(1, 60.0, &[0]),
        task(2, 30.0, &[1]),
    ];
    let mut schedule = Schedule::new();
    // The two on resource zero overlap, so the peak demand is two.
    schedule.place(&tasks[0], 0.0).expect("fits");
    schedule.place(&tasks[1], 30.0).expect("fits");
    schedule.place(&tasks[2], 0.0).expect("fits");

    let evaluation = schedule.evaluate(&tasks, &resources);
    let zero = evaluation
        .usage_of(ResourceId::new(0))
        .expect("the resource is used");
    assert_eq!(zero.peak_demand, 2.0);
    // The two overlap for thirty seconds, so the busy time is ninety.
    assert_eq!(zero.busy_seconds, 90.0);
    assert_eq!(zero.placement_count, 2);
    assert_eq!(zero.utilisation(180.0), Some(0.5));

    let one = evaluation
        .usage_of(ResourceId::new(1))
        .expect("the resource is used");
    assert_eq!(one.peak_demand, 1.0);
    assert_eq!(one.busy_seconds, 30.0);
    assert_eq!(one.placement_count, 1);

    assert!(evaluation.usage_of(ResourceId::new(9)).is_none());
}

#[test]
fn the_evaluation_of_an_empty_schedule_is_zero() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let schedule = Schedule::new();
    let evaluation = schedule.evaluate(&[], &resources);
    assert_eq!(evaluation.placed_count, 0);
    assert_eq!(evaluation.offered_count, 0);
    assert_eq!(evaluation.makespan(), 0.0);
    // With nothing offered and nothing placed, the schedule is vacuously
    // complete, but there is no fill rate to report.
    assert_eq!(evaluation.fill_rate(), None);
    assert!(evaluation.is_complete());
    let zero = evaluation.usage_of(ResourceId::new(0)).expect("present");
    assert_eq!(zero.peak_demand, 0.0);
    assert_eq!(zero.busy_seconds, 0.0);
    assert_eq!(zero.utilisation(0.0), None);
}

#[test]
fn the_fill_rate_reports_unplaced_tasks() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let tasks = vec![task(0, 60.0, &[0]), task(1, 60.0, &[0])];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 0.0).expect("fits");

    let evaluation = schedule.evaluate(&tasks, &resources);
    assert_eq!(evaluation.placed_count, 1);
    assert_eq!(evaluation.offered_count, 2);
    assert_eq!(evaluation.fill_rate(), Some(0.5));
    assert!(!evaluation.is_complete());
}

#[test]
fn the_time_order_differs_from_the_insertion_order() {
    let mut schedule = Schedule::new();
    let tasks = [task(0, 60.0, &[0]), task(1, 60.0, &[0])];
    schedule.place(&tasks[0], 200.0).expect("fits");
    schedule.place(&tasks[1], 0.0).expect("fits");

    // The insertion order is 0 then 1.
    assert_eq!(schedule.placements()[0].task, TaskId::new(0));
    // The time order is 1 then 0.
    let ordered = schedule.in_time_order();
    assert_eq!(ordered[0].task, TaskId::new(1));
    assert_eq!(ordered[1].task, TaskId::new(0));
    // The original is unchanged.
    assert_eq!(schedule.placements()[0].task, TaskId::new(0));
}

#[test]
fn the_resource_busy_until_reports_the_next_free_time() {
    let tasks = vec![task(0, 60.0, &[0]), task(1, 60.0, &[1])];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 0.0).expect("fits");
    schedule.place(&tasks[1], 30.0).expect("fits");

    assert_eq!(
        schedule.resource_busy_until(ResourceId::new(0), &tasks, 30.0),
        Some(60.0)
    );
    // Resource zero is idle at a hundred seconds.
    assert_eq!(
        schedule.resource_busy_until(ResourceId::new(0), &tasks, 100.0),
        None
    );
    // Resource one is busy at thirty seconds and free at ninety.
    assert_eq!(
        schedule.resource_busy_until(ResourceId::new(1), &tasks, 50.0),
        Some(90.0)
    );
}

#[test]
fn a_placement_reports_what_it_covers() {
    let placement = Placement::new(TaskId::new(0), 100.0, 60.0);
    assert_eq!(placement.end(), 160.0);
    assert!(placement.covers(100.0));
    assert!(placement.covers(159.999));
    // The end is not covered, so back to back placements do not overlap.
    assert!(!placement.covers(160.0));
    assert!(!placement.covers(99.999));
    assert!(placement.is_valid());
}

#[test]
fn placements_overlap_only_when_they_share_time() {
    let first = Placement::new(TaskId::new(0), 0.0, 60.0);
    let second = Placement::new(TaskId::new(1), 60.0, 60.0);
    assert!(!first.overlaps(&second));
    let overlapping = Placement::new(TaskId::new(2), 30.0, 60.0);
    assert!(first.overlaps(&overlapping));
    assert!(overlapping.overlaps(&first));
}

#[test]
fn an_invalid_placement_is_reported() {
    assert!(!Placement::new(TaskId::new(0), f64::NAN, 60.0).is_valid());
    assert!(!Placement::new(TaskId::new(0), 0.0, -1.0).is_valid());
    // A zero duration placement is valid, for an instantaneous task.
    assert!(Placement::new(TaskId::new(0), 0.0, 0.0).is_valid());
}

#[test]
fn the_evaluation_type_is_nameable() {
    // The type must be usable by a caller that stores the results, and an
    // empty schedule against one resource must report itself as empty.
    fn takes(_evaluation: &Evaluation) {}
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let schedule = Schedule::new();
    let evaluation = schedule.evaluate(&[], &resources);
    takes(&evaluation);
    assert_eq!(evaluation.placed_count, 0);
    assert_eq!(evaluation.offered_count, 0);
    assert_eq!(evaluation.scheduled_seconds, 0.0);
    assert_eq!(evaluation.resource_usage.len(), 1);
}
