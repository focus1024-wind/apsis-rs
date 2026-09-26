//! Tests for tasks and the set that holds them.

use apsis_resource::ResourceId;
use apsis_task::{Requirement, Task, TaskErrorKind, TaskId, TaskSet, TaskState, task_order};

/// A sixty second task with no window and no requirements.
fn simple() -> Task {
    Task::new(TaskId::new(0), 60.0)
}

/// A task requiring two resources for sixty seconds, in a window from 100 to
/// 1 000 seconds.
fn observation() -> Task {
    simple()
        .with_window(100.0, 1_000.0)
        .with_priority(5.0)
        .try_require(ResourceId::new(0), 1.0)
        .expect("the resource is new")
        .try_require(ResourceId::new(1), 1.0)
        .expect("the resource is new")
}

#[test]
fn a_new_task_has_the_given_duration_and_nothing_else() {
    let task = simple();
    assert_eq!(task.duration, 60.0);
    assert_eq!(task.priority, 0.0);
    assert!(!task.mandatory);
    assert!(task.requirements.is_empty());
    assert_eq!(task.requirement_count(), 0);
    // A task with no window is unbounded at both ends.
    assert!(task.earliest_start.is_infinite() && task.earliest_start < 0.0);
    assert!(task.latest_end.is_infinite() && task.latest_end > 0.0);
    assert!(task.window_is_too_short() == false);
}

#[test]
fn a_window_restricts_the_task() {
    let task = observation();
    assert_eq!(task.earliest_start, 100.0);
    assert_eq!(task.latest_end, 1_000.0);
    assert_eq!(task.window_length(), 900.0);
    assert!(task.fits_window());
    assert_eq!(task.earliest_possible_end(), 160.0);
}

#[test]
fn a_window_shorter_than_the_task_is_detected() {
    let task = simple().with_window(0.0, 30.0);
    assert!(task.window_is_too_short());
    assert!(!task.fits_window());
    assert_eq!(
        task.validate().expect_err("the window is too short").kind(),
        TaskErrorKind::WindowTooShort
    );
    assert!(!task.is_valid());
}

#[test]
fn a_window_exactly_the_length_of_the_task_is_accepted() {
    // The task must fit entirely inside the window, and a window of exactly
    // the duration holds it.
    let task = simple().with_window(0.0, 60.0);
    assert!(!task.window_is_too_short());
    assert!(task.is_valid());
}

#[test]
fn requiring_a_resource_records_the_demand() {
    let task = observation();
    assert_eq!(task.requirement_count(), 2);
    assert_eq!(task.demand_for(ResourceId::new(0)), Some(1.0));
    assert_eq!(task.demand_for(ResourceId::new(1)), Some(1.0));
    assert_eq!(task.demand_for(ResourceId::new(9)), None);
    assert!(task.requires(ResourceId::new(0)));
    assert!(!task.requires(ResourceId::new(9)));
    assert_eq!(
        task.resources(),
        vec![ResourceId::new(0), ResourceId::new(1)]
    );
}

#[test]
fn a_duplicate_requirement_is_reported_immediately() {
    let task = simple()
        .try_require(ResourceId::new(0), 1.0)
        .expect("the resource is new");
    let error = task
        .try_require(ResourceId::new(0), 2.0)
        .expect_err("the resource is already required");
    assert_eq!(error.kind(), TaskErrorKind::DuplicateRequirement);
}

#[test]
fn a_duplicate_requirement_built_by_chaining_is_rejected_on_validation() {
    // The plain builder cannot report an error, so the duplicate survives
    // until validation.
    let task = simple()
        .require(ResourceId::new(0), 1.0)
        .require(ResourceId::new(0), 1.0);
    assert!(task.has_duplicate_requirements());
    assert_eq!(
        task.validate().expect_err("duplicate").kind(),
        TaskErrorKind::DuplicateRequirement
    );
}

#[test]
fn an_invalid_demand_is_rejected() {
    for demand in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let error = simple()
            .try_require(ResourceId::new(0), demand)
            .expect_err("the demand is not usable");
        assert_eq!(
            error.kind(),
            TaskErrorKind::InvalidDemand,
            "demand {demand}"
        );
    }
}

#[test]
fn an_invalid_duration_is_rejected() {
    for duration in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let task = Task::new(TaskId::new(0), duration);
        assert!(!task.is_valid(), "duration {duration}");
        assert_eq!(
            task.validate().expect_err("invalid duration").kind(),
            TaskErrorKind::InvalidDuration
        );
    }
}

#[test]
fn a_reversed_window_is_rejected() {
    let task = simple().with_window(100.0, 50.0);
    assert_eq!(
        task.validate().expect_err("reversed").kind(),
        TaskErrorKind::InvalidWindow
    );
}

#[test]
fn a_non_finite_window_is_rejected() {
    assert_eq!(
        simple()
            .with_window(f64::NAN, 100.0)
            .validate()
            .expect_err("nan")
            .kind(),
        TaskErrorKind::InvalidWindow
    );
    assert_eq!(
        simple()
            .with_window(0.0, f64::NAN)
            .validate()
            .expect_err("nan")
            .kind(),
        TaskErrorKind::InvalidWindow
    );
}

#[test]
fn a_non_finite_priority_is_rejected() {
    let task = simple().with_priority(f64::NAN);
    assert_eq!(
        task.validate().expect_err("nan").kind(),
        TaskErrorKind::InvalidWeight
    );
}

#[test]
fn the_weighted_value_combines_priority_and_duration() {
    let task = simple().with_priority(5.0);
    assert_eq!(task.weighted_value(), 300.0);
    // A task with no priority is worth nothing.
    assert_eq!(simple().weighted_value(), 0.0);
}

#[test]
fn the_mandatory_flag_is_recorded() {
    assert!(!simple().mandatory);
    assert!(simple().with_mandatory(true).mandatory);
}

#[test]
fn the_builder_does_not_modify_the_original() {
    // A builder chain returns a new value, so a base task can be shared.
    let base = simple();
    let first = base.clone().with_priority(1.0);
    let second = base.clone().with_priority(2.0);
    assert_eq!(base.priority, 0.0);
    assert_eq!(first.priority, 1.0);
    assert_eq!(second.priority, 2.0);
}

#[test]
fn a_task_displays_its_shape() {
    let text = observation().to_string();
    assert!(text.contains("task 0"), "{text}");
    assert!(text.contains("60 s"), "{text}");
    assert!(text.contains("2 resource"), "{text}");
}

#[test]
fn task_states_report_their_placement() {
    let pending = TaskState::Pending;
    assert!(!pending.is_scheduled());
    assert_eq!(pending.start(), None);
    assert_eq!(pending.name(), "pending");

    let scheduled = TaskState::Scheduled { start: 100.0 };
    assert!(scheduled.is_scheduled());
    assert_eq!(scheduled.start(), Some(100.0));
    assert_eq!(scheduled.end(60.0), Some(160.0));
    assert_eq!(scheduled.name(), "scheduled");

    let deferred = TaskState::Deferred;
    assert!(!deferred.is_scheduled());
    assert_eq!(deferred.name(), "deferred");

    let infeasible = TaskState::Infeasible {
        reason: "window too short".to_string(),
    };
    assert!(!infeasible.is_scheduled());
    assert_eq!(infeasible.name(), "infeasible");
}

#[test]
fn a_set_holds_tasks_by_identifier() {
    let mut set = TaskSet::new();
    assert!(set.is_empty());
    set.insert(simple()).expect("the identifier is new");
    set.insert(Task::new(TaskId::new(1), 30.0))
        .expect("the identifier is new");
    assert_eq!(set.len(), 2);
    assert_eq!(set.total_duration(), 90.0);

    assert!(set.get(TaskId::new(0)).is_some());
    assert!(set.get(TaskId::new(9)).is_none());
}

#[test]
fn a_duplicate_task_identifier_is_rejected() {
    let mut set = TaskSet::new();
    set.insert(simple()).expect("the identifier is new");
    let error = set
        .insert(Task::new(TaskId::new(0), 30.0))
        .expect_err("the identifier is taken");
    assert_eq!(error.kind(), TaskErrorKind::DuplicateTask);
    assert_eq!(set.len(), 1);
}

#[test]
fn an_invalid_task_is_rejected_by_the_set() {
    let mut set = TaskSet::new();
    let error = set
        .insert(Task::new(TaskId::new(0), -1.0))
        .expect_err("the duration is invalid");
    assert_eq!(error.kind(), TaskErrorKind::InvalidDuration);
    assert!(set.is_empty());
}

#[test]
fn a_set_can_be_built_from_a_list() {
    let set = TaskSet::from_tasks(vec![simple(), Task::new(TaskId::new(1), 30.0)])
        .expect("the tasks are usable");
    assert_eq!(set.len(), 2);

    assert_eq!(
        TaskSet::from_tasks(vec![simple(), simple()])
            .expect_err("the identifiers collide")
            .kind(),
        TaskErrorKind::DuplicateTask
    );
}

#[test]
fn a_task_can_be_removed() {
    let mut set = TaskSet::new();
    set.insert(simple()).expect("new");
    let removed = set.remove(TaskId::new(0)).expect("the task exists");
    assert_eq!(removed.id, TaskId::new(0));
    assert!(set.is_empty());
    assert_eq!(
        set.remove(TaskId::new(0)).expect_err("gone").kind(),
        TaskErrorKind::NoSuchTask
    );
}

#[test]
fn the_set_reports_queries() {
    let set = TaskSet::from_tasks(vec![
        observation(),
        Task::new(TaskId::new(1), 30.0).with_mandatory(true),
        Task::new(TaskId::new(2), 10.0)
            .with_priority(2.0)
            .try_require(ResourceId::new(0), 0.5)
            .expect("the resource is new"),
    ])
    .expect("the tasks are usable");

    assert_eq!(
        set.tasks_requiring(ResourceId::new(0)),
        vec![TaskId::new(0), TaskId::new(2)]
    );
    assert_eq!(
        set.tasks_requiring(ResourceId::new(9)),
        Vec::<TaskId>::new()
    );
    assert_eq!(set.mandatory(), vec![TaskId::new(1)]);
    // The weighted value is the priority times the duration.
    assert_eq!(set.total_weighted_value(), 300.0 + 0.0 + 20.0);
}

#[test]
fn the_task_order_places_the_most_valuable_first() {
    let set = TaskSet::from_tasks(vec![
        Task::new(TaskId::new(0), 10.0).with_priority(1.0),
        Task::new(TaskId::new(1), 100.0).with_priority(5.0),
        Task::new(TaskId::new(2), 50.0).with_priority(2.0),
    ])
    .expect("the tasks are usable");

    let order = task_order(set.tasks());
    // The weighted values are 10, 500, and 100, so the order is 1, 2, 0.
    assert_eq!(order, vec![TaskId::new(1), TaskId::new(2), TaskId::new(0)]);
}

#[test]
fn the_task_order_breaks_ties_by_deadline_then_identifier() {
    let set = TaskSet::from_tasks(vec![
        // The same value and no deadline.
        Task::new(TaskId::new(0), 10.0).with_priority(1.0),
        Task::new(TaskId::new(1), 10.0).with_priority(1.0),
        // The same value but an earlier deadline.
        Task::new(TaskId::new(2), 10.0)
            .with_priority(1.0)
            .with_window(0.0, 100.0),
        Task::new(TaskId::new(3), 10.0).with_priority(1.0),
    ])
    .expect("the tasks are usable");

    let order = task_order(set.tasks());
    // The task with the deadline comes first, then the others by identifier.
    assert_eq!(
        order,
        vec![
            TaskId::new(2),
            TaskId::new(0),
            TaskId::new(1),
            TaskId::new(3)
        ]
    );
}

#[test]
fn the_task_order_of_an_empty_set_is_empty() {
    assert!(task_order(&[]).is_empty());
}

#[test]
fn a_requirement_reports_validity() {
    assert!(Requirement::new(ResourceId::new(0), 1.0).is_valid());
    assert!(!Requirement::new(ResourceId::new(0), 0.0).is_valid());
    assert!(!Requirement::new(ResourceId::new(0), f64::NAN).is_valid());
}

#[test]
fn task_identifiers_are_ordered() {
    let mut ids = [TaskId::new(2), TaskId::new(0), TaskId::new(1)];
    ids.sort();
    assert_eq!(ids, [TaskId::new(0), TaskId::new(1), TaskId::new(2)]);
    assert_eq!(TaskId::new(3).to_string(), "task 3");
    assert_eq!(TaskId::new(3).index(), 3);
}
