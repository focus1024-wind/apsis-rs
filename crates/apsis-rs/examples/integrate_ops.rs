//! Cross-layer integration check: task planning on a satellite and a station.
//!
//! This example walks the ops layer end to end and asserts that each step
//! produces sensible input for the next. It is not a numerical cross-check;
//! the per-layer tests do that. Its purpose is to fail when a change in one
//! crate breaks the assumptions of another.
//!
//! The chain is:
//!
//! 1. Define a satellite and a ground station as resources, taking the
//!    station's availability from access windows.
//! 2. Define observation tasks that each need the satellite and the station.
//! 3. Plan the tasks with `apsis-scheduler`, using precedence constraints
//!    to express a task that needs two resources in sequence.
//! 4. Check the result with `apsis-conflict`.
//! 5. Evaluate the schedule with `apsis-schedule`.
//!
//! Usage:
//!
//! ```text
//! cargo run -p apsis-rs --features ops --example integrate_ops
//! ```

use apsis_conflict::detect;
use apsis_resource::{Resource, ResourceId, ResourceSet, Window};
use apsis_schedule::Schedule;
use apsis_scheduler::{ImprovementSettings, Planner};
use apsis_task::{PrecedenceGraph, Task, TaskId};

/// The satellite resource.
const SATELLITE: ResourceId = ResourceId::new(0);
/// The ground station resource.
const STATION: ResourceId = ResourceId::new(1);

/// Returns the station availability, as a list of access windows.
///
/// These would come from the access layer in a full scenario. They are
/// written out here so that the example depends only on the ops layer.
fn station_windows() -> Vec<Window> {
    vec![
        Window::new(100.0, 400.0),
        Window::new(1_000.0, 1_300.0),
        Window::new(2_000.0, 2_200.0),
    ]
}

fn main() {
    // Step 1: the resources. The station is available only during its access
    // windows, while the satellite is available throughout the scenario.
    let mut satellite = Resource::new(SATELLITE, 1.0);
    satellite
        .set_windows(&[Window::new(0.0, 5_000.0)])
        .expect("the window is usable");
    let mut station = Resource::new(STATION, 1.0);
    station
        .set_windows(&station_windows())
        .expect("the windows are usable");
    let resources =
        ResourceSet::from_resources(vec![satellite, station]).expect("the identifiers differ");

    println!(
        "resources: satellite with {} s, station with {} s of availability",
        resources
            .get(SATELLITE)
            .expect("present")
            .total_availability(),
        resources
            .get(STATION)
            .expect("present")
            .total_availability()
    );

    // Step 2: the tasks. Each observation needs both resources for its whole
    // duration, so it can only run inside an access window.
    let tasks: Vec<Task> = (0..8)
        .map(|index| {
            Task::new(TaskId::new(index), 60.0)
                .with_priority(1.0 + f64::from(index as u32) * 0.5)
                .try_require(SATELLITE, 1.0)
                .expect("the satellite is not yet required")
                .try_require(STATION, 1.0)
                .expect("the station is not yet required")
        })
        .collect();
    println!(
        "tasks: {} observation(s), {} s of work in total",
        tasks.len(),
        tasks.iter().map(|task| task.duration).sum::<f64>()
    );

    // Step 3: plan.
    let result = Planner::new().schedule(&tasks, &resources);
    println!("plan: {} of {} placed", result.schedule.len(), tasks.len());
    assert!(result.schedule.len() <= tasks.len());

    // Step 4: the plan must be free of conflicts. The planner checks the
    // capacity before each placement, so this is a check of that promise.
    let report = detect(&result.schedule, &tasks, &resources);
    println!("conflicts: {}", report.summary());
    assert!(
        report.is_clear(),
        "the planner produced a conflicting plan: {:?}",
        report.conflicts()
    );

    // Every placement must fall inside an access window, which the conflict
    // detector confirms as an absence of availability conflicts.
    assert_eq!(
        report.count_of(apsis_conflict::ConflictKind::Availability),
        0
    );

    // Step 5: evaluate.
    let evaluation = result.schedule.evaluate(&tasks, &resources);
    println!(
        "evaluation: makespan {:.0} s, fill rate {:.0}%",
        evaluation.makespan(),
        evaluation.fill_rate().unwrap_or(0.0) * 100.0
    );
    println!(
        "  station busy {:.0} s of {:.0} s available",
        evaluation.usage_of(STATION).expect("present").busy_seconds,
        resources
            .get(STATION)
            .expect("present")
            .total_availability()
    );

    // A sanity check on the whole chain: the placed work cannot exceed the
    // station availability, because every task needs the station.
    assert!(
        evaluation.scheduled_seconds
            <= resources
                .get(STATION)
                .expect("present")
                .total_availability()
                + 1e-9,
        "the plan places more station time than the station has"
    );

    // The schedule must be readable in time order, and the order must be
    // consistent.
    let ordered = result.schedule.in_time_order();
    for pair in ordered.windows(2) {
        assert!(
            pair[1].start >= pair[0].start,
            "the time order is not sorted"
        );
    }

    // A second plan over the same inputs must be identical, so that a
    // planning cycle is reproducible.
    let again = Planner::new().schedule(&tasks, &resources);
    assert_eq!(result.schedule, again.schedule);
    let _ = Schedule::new();
    let _ = ImprovementSettings::disabled();

    // Step 6: a task that needs two resources in sequence. An observation
    // uses the payload, and the downlink that follows uses the transmitter.
    // The two are separate tasks joined by a precedence constraint, which is
    // how a sequential resource use is expressed.
    let payload = ResourceId::new(2);
    let transmitter = ResourceId::new(3);
    let mut extra = resources.clone();
    let mut payload_resource = Resource::new(payload, 1.0);
    payload_resource
        .set_windows(&[Window::new(0.0, 5_000.0)])
        .expect("the window is usable");
    let mut transmitter_resource = Resource::new(transmitter, 1.0);
    transmitter_resource
        .set_windows(&[Window::new(0.0, 5_000.0)])
        .expect("the window is usable");
    extra
        .insert(payload_resource)
        .expect("the identifier is new");
    extra
        .insert(transmitter_resource)
        .expect("the identifier is new");

    let chain = [
        Task::new(TaskId::new(100), 120.0)
            .with_priority(10.0)
            .with_window(200.0, 1_500.0)
            .try_require(payload, 1.0)
            .expect("the payload is not yet required"),
        Task::new(TaskId::new(101), 90.0)
            .with_priority(10.0)
            .with_window(0.0, 2_000.0)
            .try_require(transmitter, 1.0)
            .expect("the transmitter is not yet required"),
    ];
    let mut graph = PrecedenceGraph::new();
    graph
        .add(TaskId::new(100), TaskId::new(101))
        .expect("the constraint does not close a cycle");

    let chained = Planner::new().schedule_with_precedence(&chain, &extra, &graph);
    assert!(chained.is_complete(), "{:?}", chained.outcomes);
    assert!(chained.honours_precedence(&graph));
    let collection = chained
        .schedule
        .placement_of(TaskId::new(100))
        .expect("the collection is placed");
    let transmission = chained
        .schedule
        .placement_of(TaskId::new(101))
        .expect("the transmission is placed");
    println!(
        "sequence: collect {:.0}-{:.0} s, downlink {:.0}-{:.0} s",
        collection.start,
        collection.end(),
        transmission.start,
        transmission.end()
    );
    assert!(
        transmission.start >= collection.end(),
        "the downlink must follow the collection"
    );
    assert!(apsis_conflict::detect(&chained.schedule, &chain, &extra).is_clear());

    println!("integration check passed");
}
