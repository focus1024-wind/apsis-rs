//! Tests for the greedy planner.
//!
//! 贪心规划器的测试。

use apsis_resource::{Resource, ResourceId, ResourceSet, Window};
use apsis_schedule::Schedule;
use apsis_scheduler::{
    Planner, demand_at, earliest_feasible_start, feasible_at, infeasible_tasks, resources_free_at,
    resources_have_capacity,
};
use apsis_task::{Task, TaskId, TaskState};

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

#[test]
fn a_single_task_is_placed_at_the_opening_of_its_window() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let tasks = [task(0, 60.0, &[0])];

    let result = Planner::new().schedule(&tasks, &resources);
    assert_eq!(result.schedule.len(), 1);
    assert!(result.is_complete());
    let placement = result
        .schedule
        .placement_of(TaskId::new(0))
        .expect("placed");
    // The resource is available from zero, so the task starts there.
    // 资源自零起可用，故任务从那里开始。
    assert_eq!(placement.start, 0.0);
}

#[test]
fn tasks_on_one_exclusive_resource_are_placed_back_to_back() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    // Equal value, so the order falls back to the earlier deadline and then
    // the identifier.
    // 价值相等，故次序回退到更早的截止时间，然后回退到标识符。
    let tasks = [task(0, 60.0, &[0]), task(1, 30.0, &[0])];

    let result = Planner::new().schedule(&tasks, &resources);
    assert!(result.is_complete());
    let first = result
        .schedule
        .placement_of(TaskId::new(0))
        .expect("placed");
    let second = result
        .schedule
        .placement_of(TaskId::new(1))
        .expect("placed");
    // The two must not overlap: the second starts at or after the end of the
    // first.
    // 二者不得重叠：第二个在第一个结束处或之后开始。
    assert!(second.start >= first.end(), "{second:?} against {first:?}");
    assert!(!result.schedule.has_overlapping_placements());
}

#[test]
fn a_task_that_does_not_fit_is_deferred() {
    // The resource is available for only sixty seconds, so a task of ninety
    // cannot be placed.
    // 资源只可用六十秒，故九十秒的任务无法放置。
    let mut limited = Resource::new(ResourceId::new(0), 1.0);
    limited
        .set_windows(&[Window::new(0.0, 60.0)])
        .expect("the window is usable");
    let resources = ResourceSet::from_resources(vec![limited]).expect("one");
    let tasks = [task(0, 90.0, &[0])];

    let result = Planner::new().schedule(&tasks, &resources);
    assert!(!result.is_complete());
    assert_eq!(result.deferred(), vec![TaskId::new(0)]);
    assert_eq!(
        result.outcome_of(TaskId::new(0)).expect("reported").state,
        TaskState::Deferred
    );
    assert!(result.schedule.is_empty());
}

#[test]
fn a_task_demanding_more_than_the_capacity_is_infeasible() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let tasks = [Task::new(TaskId::new(0), 60.0)
        .try_require(ResourceId::new(0), 2.0)
        .expect("the resource is new")];

    let result = Planner::new().schedule(&tasks, &resources);
    assert_eq!(result.infeasible(), vec![TaskId::new(0)]);
    assert!(result.schedule.is_empty());
    assert!(!result.is_complete());
    assert!(
        result
            .outcome_of(TaskId::new(0))
            .expect("reported")
            .is_infeasible()
    );
}

#[test]
fn a_task_requiring_a_missing_resource_is_infeasible() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let tasks = [task(0, 60.0, &[9])];

    let result = Planner::new().schedule(&tasks, &resources);
    assert_eq!(result.infeasible(), vec![TaskId::new(0)]);
    assert!(result.schedule.is_empty());
}

#[test]
fn a_task_with_a_window_too_short_is_infeasible() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let tasks = [task(0, 90.0, &[0]).with_window(0.0, 60.0)];

    let result = Planner::new().schedule(&tasks, &resources);
    assert_eq!(result.infeasible(), vec![TaskId::new(0)]);
}

#[test]
fn a_mandatory_task_is_placed_before_a_more_valuable_optional_one() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    // The optional task is worth far more, but it can use whatever time is
    // left, while the mandatory task has only one window.
    // 可选任务价值高得多，但它可以使用剩余的任何时间，而强制任务只有一个窗口。
    let optional = task(0, 60.0, &[0])
        .with_priority(100.0)
        .with_window(0.0, 1_000.0);
    let mandatory = task(1, 60.0, &[0])
        .with_priority(1.0)
        .with_window(0.0, 100.0)
        .with_mandatory(true);
    let tasks = [optional, mandatory];

    let result = Planner::new().schedule(&tasks, &resources);
    assert!(result.honours_mandatory(&tasks));
    let mandatory_placement = result
        .schedule
        .placement_of(TaskId::new(1))
        .expect("the mandatory task is placed");
    assert_eq!(mandatory_placement.start, 0.0);

    // With the rule turned off, and the improvement pass disabled so that
    // only the ordering is under test, the optional task goes first and the
    // mandatory one is placed after it, using the rest of its window.
    // 关掉该规则、并禁用改进遍（使只有排序受检），可选任务先走，强制任务紧随其后放置，
    // 用它窗口的剩余部分。
    let without = Planner::new()
        .with_mandatory_first(false)
        .with_improvement(apsis_scheduler::ImprovementSettings::disabled())
        .schedule(&tasks, &resources);
    let optional_placement = without
        .schedule
        .placement_of(TaskId::new(0))
        .expect("the optional task is placed");
    assert_eq!(optional_placement.start, 0.0);
}

#[test]
fn the_improvement_pass_recovers_a_plan_that_the_greedy_order_missed() {
    // The optional task is worth more, so it is placed first and occupies
    // the resource from zero to sixty. The mandatory task has a window from
    // zero to a hundred, so it cannot fit after the optional one and would
    // be lost. The improvement pass moves the optional task aside and places
    // both.
    // 可选任务价值更高，故先被放置并占用资源自 0 到 60。强制任务的窗口自 0 到 100，
    // 故它放不进可选任务之后，会丢失。改进遍把可选任务挪开并放置二者。
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let optional = task(0, 60.0, &[0])
        .with_priority(100.0)
        .with_window(0.0, 1_000.0);
    let mandatory = task(1, 60.0, &[0])
        .with_priority(1.0)
        .with_window(0.0, 100.0)
        .with_mandatory(true);
    let tasks = [optional, mandatory];

    let greedy = Planner::new()
        .with_mandatory_first(false)
        .with_improvement(apsis_scheduler::ImprovementSettings::disabled())
        .schedule(&tasks, &resources);
    // Without the pass the mandatory task is lost.
    // 没有该遍时强制任务丢失。
    assert!(!greedy.honours_mandatory(&tasks));
    assert_eq!(greedy.schedule.len(), 1);

    let improved = Planner::new()
        .with_mandatory_first(false)
        .schedule(&tasks, &resources);
    // With the pass both are placed.
    // 有该遍时二者都被放置。
    assert!(improved.is_complete(), "{:?}", improved.outcomes);
    assert!(improved.honours_mandatory(&tasks));
    assert!(!improved.improved.is_empty());
}

#[test]
fn the_value_ordering_places_the_most_valuable_task_first() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let tasks = [
        task(0, 10.0, &[0]).with_priority(1.0),
        task(1, 100.0, &[0]).with_priority(5.0),
        task(2, 50.0, &[0]).with_priority(2.0),
    ];

    let result = Planner::new().schedule(&tasks, &resources);
    assert!(result.is_complete());
    // The weighted values are 10, 500, and 100, so task 1 goes first.
    // 加权值为 10、500 与 100，故任务 1 先走。
    assert_eq!(
        result
            .schedule
            .placement_of(TaskId::new(1))
            .expect("placed")
            .start,
        0.0
    );
}

#[test]
fn two_tasks_on_a_shared_resource_can_overlap() {
    let resources = ResourceSet::from_resources(vec![resource(0, 2.0)]).expect("one");
    let tasks = [task(0, 60.0, &[0]), task(1, 60.0, &[0])];

    let result = Planner::new().schedule(&tasks, &resources);
    assert!(result.is_complete());
    let first = result
        .schedule
        .placement_of(TaskId::new(0))
        .expect("placed");
    let second = result
        .schedule
        .placement_of(TaskId::new(1))
        .expect("placed");
    // Both start as early as possible, because the capacity holds them both.
    // 二者都尽早开始，因为容量能容纳二者。
    assert_eq!(first.start, 0.0);
    assert_eq!(second.start, 0.0);
    assert!(result.schedule.has_overlapping_placements());
}

#[test]
fn a_task_is_placed_after_the_other_frees_its_resource() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    // The first task occupies the resource from 100 to 200, so the second,
    // which can start no earlier than 100, must wait.
    // 第一个任务占用资源自 100 到 200，故不得早于 100 开始的第二个必须等待。
    let first = task(0, 100.0, &[0]).with_window(100.0, 1_000.0);
    let second = task(1, 50.0, &[0]).with_window(100.0, 1_000.0);
    let tasks = [first, second];

    let result = Planner::new().schedule(&tasks, &resources);
    assert!(result.is_complete());
    let first_placement = result
        .schedule
        .placement_of(TaskId::new(0))
        .expect("placed");
    let second_placement = result
        .schedule
        .placement_of(TaskId::new(1))
        .expect("placed");
    assert!(second_placement.start >= first_placement.end());
}

#[test]
fn the_earliest_start_is_the_false_of_the_window_when_nothing_blocks() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let schedule = Schedule::new();
    let tasks = [task(0, 60.0, &[0]).with_window(300.0, 1_000.0)];

    let start = earliest_feasible_start(&tasks[0], &tasks, &schedule, &resources)
        .expect("a placement exists");
    assert_eq!(start, 300.0);
}

#[test]
fn the_earliest_start_is_none_when_the_window_is_too_short() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let schedule = Schedule::new();
    let tasks = [task(0, 90.0, &[0]).with_window(0.0, 60.0)];
    assert_eq!(
        earliest_feasible_start(&tasks[0], &tasks, &schedule, &resources),
        None
    );
}

#[test]
fn the_earliest_start_accounts_for_a_blocking_placement() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let blocker = task(0, 100.0, &[0]);
    let mut schedule = Schedule::new();
    schedule.place(&blocker, 0.0).expect("fits");

    let candidate = task(1, 50.0, &[0]);
    let tasks = [blocker, candidate.clone()];
    let start = earliest_feasible_start(&candidate, &tasks, &schedule, &resources)
        .expect("a placement exists");
    // The resource is held until a hundred, so the earliest start is there.
    // 资源被占至百秒，故最早开始在那里。
    assert_eq!(start, 100.0);
}

#[test]
fn feasibility_reports_the_capacity_limit() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let existing = task(0, 100.0, &[0]);
    let mut schedule = Schedule::new();
    schedule.place(&existing, 0.0).expect("fits");

    let candidate = task(1, 50.0, &[0]);
    let tasks = [existing, candidate.clone()];
    // Starting at fifty would overlap the existing placement, which the
    // capacity of one does not allow.
    // 自五十开始会与已有放置重叠，容量为一不允许。
    assert!(!feasible_at(
        &candidate, &tasks, &schedule, &resources, 50.0
    ));
    // Starting at a hundred is free.
    // 自百秒开始是空闲的。
    assert!(feasible_at(
        &candidate, &tasks, &schedule, &resources, 100.0
    ));
}

#[test]
fn feasibility_reports_the_availability_limit() {
    let mut limited = Resource::new(ResourceId::new(0), 1.0);
    limited
        .set_windows(&[Window::new(0.0, 100.0)])
        .expect("the window is usable");
    let resources = ResourceSet::from_resources(vec![limited]).expect("one");
    let schedule = Schedule::new();
    let tasks = [task(0, 60.0, &[0])];

    assert!(feasible_at(&tasks[0], &tasks, &schedule, &resources, 0.0));
    assert!(feasible_at(&tasks[0], &tasks, &schedule, &resources, 40.0));
    // Starting at fifty would end at one hundred and ten, past the window.
    // 自五十开始会在百一十结束，越过窗口。
    assert!(!feasible_at(&tasks[0], &tasks, &schedule, &resources, 50.0));
}

#[test]
fn the_capacity_check_covers_a_multiple_resource_task() {
    let resources =
        ResourceSet::from_resources(vec![resource(0, 1.0), resource(1, 1.0)]).expect("distinct");
    // The task needs both resources at once.
    // 该任务同时需要两种资源。
    let tasks = [task(0, 60.0, &[0, 1])];
    assert!(resources_have_capacity(&tasks[0], &resources));

    // The same task with one resource missing is infeasible.
    // 同一任务缺少一个资源即不可行。
    let missing = [task(0, 60.0, &[0, 9])];
    assert!(!resources_have_capacity(&missing[0], &resources));
    assert_eq!(infeasible_tasks(&missing, &resources), vec![TaskId::new(0)]);
}

#[test]
fn the_demand_at_an_instant_is_reported_per_resource() {
    let resources =
        ResourceSet::from_resources(vec![resource(0, 2.0), resource(1, 1.0)]).expect("distinct");
    let tasks = [task(0, 100.0, &[0]), task(1, 100.0, &[1])];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 0.0).expect("fits");
    schedule.place(&tasks[1], 50.0).expect("fits");

    let demand = demand_at(&schedule, &tasks, &resources, 75.0);
    assert_eq!(demand[0], (ResourceId::new(0), 1.0));
    assert_eq!(demand[1], (ResourceId::new(1), 1.0));
    // Before either placement the demand is zero.
    // 在两处放置之前需求为零。
    let demand = demand_at(&schedule, &tasks, &resources, -1.0);
    assert_eq!(demand[0].1, 0.0);
}

#[test]
fn the_free_resources_at_an_instant_exclude_the_busy_ones() {
    let resources =
        ResourceSet::from_resources(vec![resource(0, 1.0), resource(1, 1.0)]).expect("distinct");
    let tasks = [task(0, 100.0, &[0])];
    let mut schedule = Schedule::new();
    schedule.place(&tasks[0], 0.0).expect("fits");

    // At fifty seconds resource zero is full and resource one is free.
    // 五十秒处资源零已满而资源一空闲。
    assert_eq!(
        resources_free_at(&schedule, &tasks, &resources, 50.0),
        vec![ResourceId::new(1)]
    );
    // At a hundred and fifty both are free.
    // 百五十秒处二者都空闲。
    assert_eq!(
        resources_free_at(&schedule, &tasks, &resources, 150.0),
        vec![ResourceId::new(0), ResourceId::new(1)]
    );
}

#[test]
fn the_planner_order_puts_mandatory_tasks_first() {
    let tasks = [
        task(0, 10.0, &[0]).with_priority(10.0),
        task(1, 10.0, &[0]).with_priority(1.0).with_mandatory(true),
    ];
    let order = Planner::new().order(&tasks);
    assert_eq!(order[0], TaskId::new(1));

    let without = Planner::new().with_mandatory_first(false).order(&tasks);
    assert_eq!(without[0], TaskId::new(0));
}

#[test]
fn the_result_reports_every_task() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let tasks = [
        task(0, 60.0, &[0]),
        task(1, 90.0, &[0]).with_window(0.0, 60.0),
        Task::new(TaskId::new(2), 60.0)
            .try_require(ResourceId::new(0), 5.0)
            .expect("the resource is new"),
    ];

    let result = Planner::new().schedule(&tasks, &resources);
    assert_eq!(result.outcomes.len(), 3);
    for task in &tasks {
        assert!(result.outcome_of(task.id).is_some());
    }
    // Only the first can be placed.
    // 只有第一个能被放置。
    assert_eq!(result.scheduled(), vec![TaskId::new(0)]);
    assert!(!result.is_complete());
    assert!(result.honours_mandatory(&tasks));
}

#[test]
fn a_mandatory_task_that_cannot_be_placed_is_reported() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let tasks = [task(0, 90.0, &[0])
        .with_window(0.0, 60.0)
        .with_mandatory(true)];
    let result = Planner::new().schedule(&tasks, &resources);
    // The plan does not honour the mandatory task, which the caller can
    // detect.
    // 该计划未兑现强制任务，调用方可检测到。
    assert!(!result.honours_mandatory(&tasks));
    assert!(!result.is_complete());
}

#[test]
fn planning_an_empty_task_list_gives_an_empty_schedule() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let result = Planner::new().schedule(&[], &resources);
    assert!(result.schedule.is_empty());
    assert!(result.outcomes.is_empty());
    assert!(result.is_complete());
    assert!(result.honours_mandatory(&[]));
}

#[test]
fn the_planner_default_places_mandatory_tasks_first() {
    assert_eq!(Planner::default(), Planner::new());
    assert!(Planner::new().mandatory_first);
}

#[test]
fn the_infeasible_task_list_excludes_the_merely_deferred() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let tasks = [
        // Can be placed.
        // 可以放置。
        task(0, 60.0, &[0]),
        // Demands too much.
        // 需求过多。
        Task::new(TaskId::new(1), 60.0)
            .try_require(ResourceId::new(0), 5.0)
            .expect("the resource is new"),
        // Names a resource that does not exist.
        // 指名了不存在的资源。
        task(2, 60.0, &[9]),
    ];
    let infeasible = infeasible_tasks(&tasks, &resources);
    assert_eq!(infeasible, vec![TaskId::new(1), TaskId::new(2)]);
}
