//! Cross-layer integration check: running a scenario as a simulation.
//!
//! This example walks the simulation layer end to end and asserts that each
//! step produces sensible input for the next. It is not a numerical
//! cross-check; the per-layer tests do that. Its purpose is to fail when a
//! change in one crate breaks the assumptions of another.
//!
//! The chain is:
//!
//! 1. Load a scenario from text, which describes the resources, the tasks,
//!    and the precedence between them.
//! 2. Prepare it with `apsis-sim`, which plans the tasks and builds a
//!    timeline.
//! 3. Start a session and run it, recording state at each activity.
//! 4. Read the run report back, which holds the engine outcome and the
//!    trace.
//!
//! Usage:
//!
//! ```text
//! cargo run -p apsis-rs --features sim --example integrate_sim
//! ```

use apsis_sim::{RecordKind, Scenario, Simulation, StopReason};

/// The scenario the example runs.
///
/// The station is available in two passes. An observation uses the payload,
/// and the downlink that follows uses the transmitter, so the two are joined
/// by a precedence constraint. A third task of low priority may or may not
/// fit, which is what makes the plan interesting to report.
const SCENARIO: &str = "\
# A short downlink scenario.
scenario leo downlink
epoch 0
time_limit 4000

resource 0 capacity 1
window 0 0 4000

task 0 duration 120
task_window 0 200 1500
task_priority 0 10
require 0 0 1

task 1 duration 90
task_window 1 0 2000
task_priority 1 9
require 1 0 1

task 2 duration 60
task_window 2 2500 3000
task_priority 2 1
require 2 0 1

precedence 0 1
";

fn main() {
    // Step 1: load the scenario. A failure here is a problem with the text,
    // so the example reports it rather than unwrapping.
    let scenario = match Scenario::parse(SCENARIO) {
        Ok(scenario) => scenario,
        Err(error) => {
            eprintln!("the scenario does not load: {error}");
            return;
        }
    };
    let summary = scenario.summary();
    println!(
        "scenario {}: {} resource(s), {} task(s), {} precedence edge(s)",
        scenario.name(),
        summary.resources,
        summary.tasks,
        summary.precedence_edges
    );

    // Step 2: prepare the simulation. This plans the tasks and builds the
    // timeline that the plan implies.
    let simulation = match Simulation::prepare(scenario) {
        Ok(simulation) => simulation,
        Err(error) => {
            eprintln!("the simulation does not prepare: {error}");
            return;
        }
    };
    let evaluation = simulation.evaluate();
    println!(
        "plan: {} of {} placed, makespan {:.0} s",
        evaluation.placed_count,
        evaluation.offered_count,
        evaluation.makespan()
    );
    println!(
        "plan: {} start and end event(s), full plan: {}",
        simulation.activity_count(),
        simulation.has_full_plan()
    );
    if !simulation.deferred().is_empty() {
        println!("plan: deferred {:?}", simulation.deferred());
    }
    assert!(simulation.honours_precedence());

    // Step 3: run the timeline. The handler records what it sees, which is
    // what a real model would do with the physics it has.
    let mut session = simulation.start();
    let mut starts = 0_u32;
    let mut ends = 0_u32;
    let report = session.run(|_engine, recorder, event| {
        if event.payload.is_start() {
            starts += 1;
            recorder.note(event.time, "started", event.payload.task().index() as f64);
        } else {
            ends += 1;
            recorder.note(event.time, "finished", event.payload.task().index() as f64);
        }
    });

    // Step 4: read the report back.
    println!(
        "run: {} activity(s) executed, {} start(s), {} end(s)",
        report.executed(),
        starts,
        ends
    );
    println!(
        "run: stopped by {} ({})",
        report.outcome.reason, report.outcome
    );
    assert_eq!(report.outcome.reason, StopReason::QueueEmpty);
    assert_eq!(report.executed(), u64::from(starts + ends));
    assert_eq!(
        report.trace.of_kind(RecordKind::State).len(),
        starts as usize + ends as usize
    );

    // Two tasks are joined by precedence, so their ends and starts must line
    // up in the timeline the handler saw.
    let trace = &report.trace;
    let collection_end = trace
        .with_label("finished")
        .iter()
        .filter(|record| record.value == 0.0)
        .map(|record| record.time)
        .next();
    let downlink_start = trace
        .with_label("started")
        .iter()
        .filter(|record| record.value == 1.0)
        .map(|record| record.time)
        .next();
    if let (Some(collection_end), Some(downlink_start)) = (collection_end, downlink_start) {
        println!(
            "sequence: collection ends at {collection_end} s, downlink starts at {downlink_start} s"
        );
        assert!(downlink_start >= collection_end);
    }

    // A run of the same prepared simulation must reproduce the trace.
    let mut again = simulation.start();
    let repeated = again.run(|_engine, recorder, event| {
        if event.payload.is_start() {
            recorder.note(event.time, "started", event.payload.task().index() as f64);
        } else {
            recorder.note(event.time, "finished", event.payload.task().index() as f64);
        }
    });
    assert_eq!(repeated.executed(), report.executed());
    assert_eq!(repeated.trace.records(), report.trace.records());

    println!("integration check passed");
}
