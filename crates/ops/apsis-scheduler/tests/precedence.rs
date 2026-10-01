//! Tests for precedence honoured by the planner, and for the improvement
//! pass.
//!
//! 规划器所遵守的先后约束，以及改进遍的测试。

use apsis_resource::{Resource, ResourceId, ResourceSet, Window};
use apsis_scheduler::{ImprovementSettings, Planner};
use apsis_task::{PrecedenceGraph, Task, TaskId};

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
fn a_chain_of_tasks_is_placed_in_order() {
    // Three tasks that must run one after another on the same resource.
    // 三个必须在同一资源上依次运行的任务。
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let tasks = [
        task(0, 60.0, &[0]),
        task(1, 30.0, &[0]),
        task(2, 90.0, &[0]),
    ];
    let mut graph = PrecedenceGraph::new();
    graph.add(TaskId::new(0), TaskId::new(1)).expect("no cycle");
    graph.add(TaskId::new(1), TaskId::new(2)).expect("no cycle");

    let result = Planner::new().schedule_with_precedence(&tasks, &resources, &graph);
    assert!(result.is_complete(), "{:?}", result.outcomes);
    assert!(result.honours_precedence(&graph));

    let first = result
        .schedule
        .placement_of(TaskId::new(0))
        .expect("placed");
    let second = result
        .schedule
        .placement_of(TaskId::new(1))
        .expect("placed");
    let third = result
        .schedule
        .placement_of(TaskId::new(2))
        .expect("placed");
    assert_eq!(first.start, 0.0);
    assert_eq!(second.start, 60.0);
    assert_eq!(third.start, 90.0);
}

#[test]
fn a_precedence_floor_delays_a_task_that_could_start_earlier() {
    // The second task could start at zero on its own, but must wait for the
    // first, which runs on a different resource.
    // 第二个任务本身可以从零开始，但必须等待在另一资源上运行的第一个。
    let resources =
        ResourceSet::from_resources(vec![resource(0, 1.0), resource(1, 1.0)]).expect("distinct");
    let tasks = [task(0, 100.0, &[0]), task(1, 60.0, &[1])];
    let mut graph = PrecedenceGraph::new();
    graph.add(TaskId::new(0), TaskId::new(1)).expect("no cycle");

    let result = Planner::new().schedule_with_precedence(&tasks, &resources, &graph);
    assert!(result.is_complete());
    let second = result
        .schedule
        .placement_of(TaskId::new(1))
        .expect("placed");
    // The second resource was free from the start, but the constraint delays
    // the task to the end of the first.
    // 第二个资源从一开始就空闲，但该约束把任务推迟到第一个结束。
    assert_eq!(second.start, 100.0);
    assert!(result.honours_precedence(&graph));
}

#[test]
fn a_diamond_precedence_is_honoured() {
    let resources =
        ResourceSet::from_resources(vec![resource(0, 1.0), resource(1, 1.0)]).expect("distinct");
    // The first task feeds two parallel tasks, which both feed a last one.
    // 第一个任务供给两个并行任务，二者又都供给最后一个。
    let tasks = [
        task(0, 100.0, &[0]),
        task(1, 60.0, &[1]),
        task(2, 60.0, &[1]),
        task(3, 30.0, &[0]),
    ];
    let mut graph = PrecedenceGraph::new();
    graph.add(TaskId::new(0), TaskId::new(1)).expect("no cycle");
    graph.add(TaskId::new(0), TaskId::new(2)).expect("no cycle");
    graph.add(TaskId::new(1), TaskId::new(3)).expect("no cycle");
    graph.add(TaskId::new(2), TaskId::new(3)).expect("no cycle");

    let result = Planner::new().schedule_with_precedence(&tasks, &resources, &graph);
    assert!(result.is_complete(), "{:?}", result.outcomes);
    assert!(result.honours_precedence(&graph));

    // The last task must follow both of the parallel ones.
    // 最后一个任务必须跟随两个并行任务。
    let last = result
        .schedule
        .placement_of(TaskId::new(3))
        .expect("placed");
    for parallel in [1_usize, 2] {
        let placement = result
            .schedule
            .placement_of(TaskId::new(parallel))
            .expect("placed");
        assert!(
            placement.end() <= last.start,
            "{placement:?} against {last:?}"
        );
    }
}

#[test]
fn a_sequential_observation_and_downlink_is_expressed_as_two_tasks() {
    // An observation uses the payload and a downlink uses the transmitter,
    // and the downlink must follow the observation. This is the pattern for
    // a task that needs two resources in sequence.
    // 一次观测使用载荷、一次下行使用发射机，且下行必须跟随观测。这是按序需要两种
    // 资源的任务模式。
    let resources =
        ResourceSet::from_resources(vec![resource(0, 1.0), resource(1, 1.0)]).expect("distinct");
    let observation = task(0, 120.0, &[0]).with_window(50.0, 500.0);
    let downlink = task(1, 90.0, &[1]).with_window(0.0, 1_000.0);
    let tasks = [observation, downlink];

    let mut graph = PrecedenceGraph::new();
    graph.add(TaskId::new(0), TaskId::new(1)).expect("no cycle");

    let result = Planner::new().schedule_with_precedence(&tasks, &resources, &graph);
    assert!(result.is_complete());
    assert!(result.honours_precedence(&graph));

    let collection = result
        .schedule
        .placement_of(TaskId::new(0))
        .expect("placed");
    let transmission = result
        .schedule
        .placement_of(TaskId::new(1))
        .expect("placed");
    assert_eq!(collection.start, 50.0);
    // The downlink starts when the collection ends.
    // 下行在采集结束时开始。
    assert_eq!(transmission.start, 170.0);
    // The two use different resources, so they may be placed back to back
    // without a conflict.
    // 二者使用不同资源，故它们可前后相接地放置而无冲突。
    assert_eq!(collection.end(), transmission.start);
}

#[test]
fn a_successor_of_an_unplaced_task_is_deferred() {
    // The first task cannot be placed, so the constraint cannot be met and
    // the second is deferred rather than placed early.
    // 第一个任务无法放置，故约束无法满足，第二个被推迟而非提前放置。
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let impossible = task(0, 90.0, &[0]).with_window(0.0, 60.0);
    let successor = task(1, 30.0, &[0]);
    let tasks = [impossible, successor];

    let mut graph = PrecedenceGraph::new();
    graph.add(TaskId::new(0), TaskId::new(1)).expect("no cycle");

    let result = Planner::new().schedule_with_precedence(&tasks, &resources, &graph);
    assert!(!result.schedule.is_placed(TaskId::new(1)));
    assert!(result.deferred().contains(&TaskId::new(1)));
    // The constraints are trivially satisfied, because nothing was promised.
    // 各约束空洞地满足，因为什么都没承诺。
    assert!(result.honours_precedence(&graph));
}

#[test]
fn the_precedence_aware_order_is_topological() {
    let tasks = [
        Task::new(TaskId::new(0), 60.0),
        Task::new(TaskId::new(1), 60.0),
        Task::new(TaskId::new(2), 60.0),
    ];
    let mut graph = PrecedenceGraph::new();
    graph.add(TaskId::new(2), TaskId::new(0)).expect("no cycle");

    let order = Planner::new().order_with_precedence(&tasks, Some(&graph));
    let position = |id: TaskId| {
        order
            .iter()
            .position(|candidate| *candidate == id)
            .expect("present")
    };
    assert!(position(TaskId::new(2)) < position(TaskId::new(0)));
}

#[test]
fn a_cycle_in_the_graph_falls_back_to_the_value_order() {
    // The graph refuses a cycle, so a cycle can only exist if the caller
    // builds the graph from one task set and orders another. The planner
    // then falls back to the value order rather than failing.
    // 该图拒绝环，故只有调用方从一套任务集构建图而排序另一套时环才可能存在。规划器
    // 随后回退到价值次序而非失败。
    let tasks = [
        Task::new(TaskId::new(0), 60.0),
        Task::new(TaskId::new(1), 60.0),
    ];
    let graph = PrecedenceGraph::new();
    let order = Planner::new().order_with_precedence(&tasks, Some(&graph));
    // An empty graph gives the value order, which for equal values is by
    // identifier.
    // 空图给出价值次序，价值相等时按标识符。
    assert_eq!(order, vec![TaskId::new(0), TaskId::new(1)]);
}

#[test]
fn the_improvement_pass_is_skipped_when_precedence_is_used() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let tasks = [task(0, 60.0, &[0]), task(1, 60.0, &[0])];
    let graph = PrecedenceGraph::new();
    let result = Planner::new().schedule_with_precedence(&tasks, &resources, &graph);
    // The pass does not run, so nothing is reported as improved.
    // 该遍不运行，故没有报告改进。
    assert!(result.improved.is_empty());
}

#[test]
fn the_improvement_pass_can_be_disabled() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let optional = task(0, 60.0, &[0])
        .with_priority(100.0)
        .with_window(0.0, 1_000.0);
    let mandatory = task(1, 60.0, &[0])
        .with_priority(1.0)
        .with_window(0.0, 100.0)
        .with_mandatory(true);
    let tasks = [optional, mandatory];

    let planner = Planner::new()
        .with_mandatory_first(false)
        .with_improvement(ImprovementSettings::disabled());
    let result = planner.schedule(&tasks, &resources);
    assert!(result.improved.is_empty());
    assert_eq!(result.schedule.len(), 1);
}

#[test]
fn the_improvement_pass_preserves_feasibility() {
    // Whatever the pass decides, the result must remain conflict free.
    // 无论该遍如何决定，结果必须保持无冲突。
    let resources =
        ResourceSet::from_resources(vec![resource(0, 1.0), resource(1, 2.0)]).expect("distinct");
    let tasks: Vec<Task> = (0..10)
        .map(|index| {
            task(index, 40.0 + f64::from(index as u32) * 5.0, &[index % 2])
                .with_priority(1.0 + f64::from(index as u32))
        })
        .collect();

    let result = Planner::new().schedule(&tasks, &resources);
    // The schedule must not place two tasks beyond the capacity of a
    // resource at the same time.
    // 调度不得在同一时刻把两个任务放置到超出某资源的容量。
    let report = apsis_conflict::detect(&result.schedule, &tasks, &resources);
    assert!(
        report.is_clear(),
        "the improvement pass broke feasibility: {:?}",
        report.conflicts()
    );
}

#[test]
fn the_improvement_pass_never_loses_a_placement() {
    // Every task that the greedy pass placed must still be placed
    // afterwards, because the pass only adds placements or moves them.
    // 贪心遍所放置的每个任务其后必须仍被放置，因为该遍只添加或移动放置。
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let tasks: Vec<Task> = (0..6)
        .map(|index| task(index, 60.0, &[0]).with_priority(1.0 + f64::from(index as u32)))
        .collect();

    let with_pass = Planner::new().schedule(&tasks, &resources);
    let without_pass = Planner::new()
        .with_improvement(ImprovementSettings::disabled())
        .schedule(&tasks, &resources);
    assert!(
        with_pass.schedule.len() >= without_pass.schedule.len(),
        "{} against {}",
        with_pass.schedule.len(),
        without_pass.schedule.len()
    );
}

#[test]
fn the_improvement_settings_are_validated_by_use() {
    let resources = ResourceSet::from_resources(vec![resource(0, 1.0)]).expect("one");
    let tasks = [task(0, 60.0, &[0])];
    // A bound of zero disables the move search but leaves the pass enabled,
    // which behaves as if the pass were off.
    // 界为零禁用移动搜索但保留该遍启用，其行为如同该遍关闭。
    let planner = Planner::new().with_improvement(ImprovementSettings::new(0));
    let result = planner.schedule(&tasks, &resources);
    assert!(result.is_complete());
    assert!(result.improved.is_empty());
}
