//! Tests for building and reading a scenario.

use apsis_resource::{Resource, ResourceId, Window};
use apsis_scenario::{Scenario, ScenarioErrorKind};
use apsis_task::{Task, TaskId};

fn station(capacity: f64, windows: &[(f64, f64)]) -> Resource {
    let mut resource = Resource::new(ResourceId::new(0), capacity);
    let windows: Vec<Window> = windows
        .iter()
        .map(|&(start, end)| Window::new(start, end))
        .collect();
    resource
        .set_windows(&windows)
        .expect("the windows are usable");
    resource
}

#[test]
fn a_new_scenario_is_empty() {
    let scenario = Scenario::new("demo");
    assert_eq!(scenario.name(), "demo");
    assert_eq!(scenario.epoch(), 0.0);
    assert_eq!(scenario.time_limit(), None);
    assert_eq!(scenario.event_limit(), None);
    assert!(scenario.resources().is_empty());
    assert!(scenario.tasks().is_empty());
    assert!(scenario.precedence().is_empty());
    assert_eq!(scenario.validate(), Ok(()));
}

#[test]
fn the_header_setters_take_effect() {
    let scenario = Scenario::new("demo")
        .with_epoch(100.0)
        .with_time_limit(3_600.0)
        .with_event_limit(1_000_000);
    assert_eq!(scenario.epoch(), 100.0);
    assert_eq!(scenario.time_limit(), Some(3_600.0));
    assert_eq!(scenario.event_limit(), Some(1_000_000));
}

#[test]
fn a_limit_the_run_cannot_use_is_cleared() {
    // The setters match the engine, which ignores a limit it cannot use, so
    // a scenario never carries one that would surprise a run.
    let scenario = Scenario::new("demo")
        .with_time_limit(-1.0)
        .with_event_limit(0);
    assert_eq!(scenario.time_limit(), None);
    assert_eq!(scenario.event_limit(), None);

    let scenario = Scenario::new("demo").with_time_limit(f64::NAN);
    assert_eq!(scenario.time_limit(), None);
}

#[test]
fn the_checked_setters_report_a_bad_value() {
    let mut scenario = Scenario::new("demo");
    let error = scenario.set_epoch(-1.0).expect_err("the epoch is bad");
    assert_eq!(error.kind(), ScenarioErrorKind::InvalidEpoch);
    assert!(scenario.set_epoch(f64::NAN).is_err());
    assert!(scenario.set_time_limit(f64::INFINITY).is_err());
    assert!(scenario.set_time_limit(-5.0).is_err());
    // A good value is accepted and the scenario keeps its old one on failure.
    scenario.set_epoch(10.0).expect("the epoch is usable");
    assert_eq!(scenario.epoch(), 10.0);
    scenario.set_time_limit(60.0).expect("the limit is usable");
    assert_eq!(scenario.time_limit(), Some(60.0));
}

#[test]
fn a_resource_is_added_and_checked() {
    let mut scenario = Scenario::new("demo");
    scenario
        .add_resource(station(1.0, &[(0.0, 600.0)]))
        .expect("the resource is new");
    assert_eq!(scenario.resources().len(), 1);

    // A second resource with the same identifier is refused.
    let error = scenario
        .add_resource(station(2.0, &[]))
        .expect_err("the identifier is taken");
    assert_eq!(error.kind(), ScenarioErrorKind::DuplicateResource);
}

#[test]
fn an_unusable_resource_is_refused() {
    let mut scenario = Scenario::new("demo");
    let error = scenario
        .add_resource(Resource::new(ResourceId::new(0), 0.0))
        .expect_err("a zero capacity is not usable");
    assert_eq!(error.kind(), ScenarioErrorKind::InvalidResource);
    assert!(scenario.resources().is_empty());
}

#[test]
fn a_task_is_added_only_when_its_resources_exist() {
    let mut scenario = Scenario::new("demo");
    let task = Task::new(TaskId::new(0), 60.0)
        .try_require(ResourceId::new(0), 1.0)
        .expect("the resource is new");

    // The resource is not in the scenario yet.
    let error = scenario
        .add_task(task.clone())
        .expect_err("unknown resource");
    assert_eq!(error.kind(), ScenarioErrorKind::UnknownResource);

    scenario
        .add_resource(station(1.0, &[(0.0, 600.0)]))
        .expect("the resource is new");
    scenario.add_task(task).expect("the task is usable");
    assert_eq!(scenario.tasks().len(), 1);

    // A second task with the same identifier is refused.
    let error = scenario
        .add_task(Task::new(TaskId::new(0), 30.0))
        .expect_err("the identifier is taken");
    assert_eq!(error.kind(), ScenarioErrorKind::DuplicateTask);
}

#[test]
fn an_unusable_task_is_refused() {
    let mut scenario = Scenario::new("demo");
    // A task with a zero duration is not usable.
    let error = scenario
        .add_task(Task::new(TaskId::new(0), 0.0))
        .expect_err("the task is not usable");
    assert_eq!(error.kind(), ScenarioErrorKind::InvalidTask);
}

#[test]
fn precedence_requires_both_tasks_to_exist() {
    let mut scenario = Scenario::new("demo");
    scenario
        .add_resource(station(1.0, &[(0.0, 600.0)]))
        .expect("the resource is new");
    for id in 0..2 {
        scenario
            .add_task(Task::new(TaskId::new(id), 60.0))
            .expect("the task is usable");
    }

    scenario
        .add_precedence(TaskId::new(0), TaskId::new(1))
        .expect("both tasks exist");
    assert!(
        scenario
            .precedence()
            .has_constraint(TaskId::new(0), TaskId::new(1))
    );

    let error = scenario
        .add_precedence(TaskId::new(1), TaskId::new(0))
        .expect_err("the constraint closes a cycle");
    assert_eq!(error.kind(), ScenarioErrorKind::CyclicPrecedence);

    let error = scenario
        .add_precedence(TaskId::new(0), TaskId::new(9))
        .expect_err("the task is missing");
    assert_eq!(error.kind(), ScenarioErrorKind::UnknownTask);
}

#[test]
fn the_resource_set_is_built_from_the_scenario() {
    let mut scenario = Scenario::new("demo");
    scenario
        .add_resource(station(1.0, &[(0.0, 600.0)]))
        .expect("the resource is new");
    let set = scenario.resource_set().expect("the resources are distinct");
    assert_eq!(set.len(), 1);
    assert!(set.get(ResourceId::new(0)).is_some());
}

#[test]
fn the_summary_counts_what_the_scenario_holds() {
    let mut scenario = Scenario::new("demo");
    scenario
        .add_resource(station(1.0, &[(0.0, 600.0), (1_200.0, 1_800.0)]))
        .expect("the resource is new");
    scenario
        .add_task(Task::new(TaskId::new(0), 60.0))
        .expect("the task is usable");
    scenario
        .add_task(Task::new(TaskId::new(1), 90.0))
        .expect("the task is usable");
    scenario
        .add_precedence(TaskId::new(0), TaskId::new(1))
        .expect("both tasks exist");

    let summary = scenario.summary();
    assert_eq!(summary.resources, 1);
    assert_eq!(summary.tasks, 2);
    assert_eq!(summary.total_task_duration, 150.0);
    assert_eq!(summary.precedence_edges, 1);
    assert_eq!(summary.total_availability, 1_200.0);
}

#[test]
fn a_scenario_parses_from_text() {
    let text = "\
# a small scenario
scenario demo
epoch 100
time_limit 3600
event_limit 500000

resource 0 capacity 1
window 0 0 600
window 0 1200 1800

resource 1 capacity 2
window 1 0 3600

task 0 duration 60
task_window 0 100 1000
task_priority 0 5
task_mandatory 0 true
require 0 0 1
require 0 1 1

task 1 duration 90
require 1 0 1
";
    let scenario = Scenario::parse(text).expect("the text is well formed");
    assert_eq!(scenario.name(), "demo");
    assert_eq!(scenario.epoch(), 100.0);
    assert_eq!(scenario.time_limit(), Some(3_600.0));
    assert_eq!(scenario.event_limit(), Some(500_000));
    assert_eq!(scenario.resources().len(), 2);
    assert_eq!(scenario.resources()[0].window_count(), 2);
    assert_eq!(scenario.resources()[1].capacity(), 2.0);
    assert_eq!(scenario.tasks().len(), 2);
    let first = &scenario.tasks()[0];
    assert_eq!(first.duration, 60.0);
    assert_eq!(first.earliest_start, 100.0);
    assert_eq!(first.latest_end, 1_000.0);
    assert_eq!(first.priority, 5.0);
    assert!(first.mandatory);
    assert_eq!(first.requirement_count(), 2);
    assert_eq!(scenario.validate(), Ok(()));
}

#[test]
fn a_scenario_round_trips_through_text() {
    let mut original = Scenario::new("round trip")
        .with_epoch(50.0)
        .with_time_limit(7_200.0)
        .with_event_limit(9_999);
    original
        .add_resource(station(1.5, &[(0.0, 600.0), (900.0, 1_500.0)]))
        .expect("the resource is new");
    let mut second = Resource::new(ResourceId::new(1), 3.0);
    second
        .set_windows(&[Window::new(0.0, 3_600.0)])
        .expect("usable");
    original.add_resource(second).expect("the resource is new");

    original
        .add_task(
            Task::new(TaskId::new(0), 60.0)
                .with_window(0.0, 1_000.0)
                .with_priority(5.0)
                .with_mandatory(true)
                .try_require(ResourceId::new(0), 1.0)
                .expect("the resource is new")
                .try_require(ResourceId::new(1), 0.5)
                .expect("the resource is new"),
        )
        .expect("the task is usable");
    original
        .add_task(
            Task::new(TaskId::new(1), 90.0)
                .try_require(ResourceId::new(0), 1.0)
                .expect("the resource is new"),
        )
        .expect("the task is usable");
    original
        .add_precedence(TaskId::new(0), TaskId::new(1))
        .expect("both tasks exist");

    let text = original.to_text();
    let reloaded = Scenario::parse(&text).expect("the rendered text parses");
    assert_eq!(reloaded, original);
    // A second round trip is stable, so the text form is a fixed point.
    assert_eq!(reloaded.to_text(), text);
}

#[test]
fn a_task_with_no_window_round_trips() {
    // A task with no window leaves both ends infinite, which the text form
    // cannot write; the round trip must still recover an equivalent task.
    let mut original = Scenario::new("open");
    original
        .add_resource(station(1.0, &[(0.0, 600.0)]))
        .expect("the resource is new");
    original
        .add_task(
            Task::new(TaskId::new(0), 60.0)
                .try_require(ResourceId::new(0), 1.0)
                .expect("the resource is new"),
        )
        .expect("the task is usable");

    let reloaded = Scenario::parse(&original.to_text()).expect("the text parses");
    let task = &reloaded.tasks()[0];
    // The window is written as a finite interval that holds the task, which
    // is the best a text form with no infinity can do. The task is still
    // usable.
    assert!(task.is_valid());
    assert!(task.fits_window());
}

#[test]
fn a_missing_header_is_reported() {
    let error = Scenario::parse("resource 0 capacity 1").expect_err("no header");
    assert_eq!(error.kind(), ScenarioErrorKind::MissingHeader);
    assert_eq!(error.line(), Some(1));
}

#[test]
fn a_duplicate_header_field_is_reported() {
    let text = "scenario demo\nepoch 0\nepoch 1\n";
    let error = Scenario::parse(text).expect_err("the epoch repeats");
    assert_eq!(error.kind(), ScenarioErrorKind::DuplicateField);
    assert_eq!(error.line(), Some(3));
}

#[test]
fn an_unknown_keyword_is_reported_with_its_line() {
    let text = "scenario demo\nresource 0 capacity 1\norbit 42\n";
    let error = Scenario::parse(text).expect_err("the keyword is unknown");
    assert_eq!(error.kind(), ScenarioErrorKind::UnknownKeyword);
    assert_eq!(error.line(), Some(3));
}

#[test]
fn a_missing_argument_is_reported() {
    let error = Scenario::parse("scenario demo\nresource 0 capacity\n").expect_err("short line");
    assert_eq!(error.kind(), ScenarioErrorKind::MissingArgument);
    assert_eq!(error.line(), Some(2));
}

#[test]
fn a_bad_number_is_reported() {
    let error =
        Scenario::parse("scenario demo\nresource 0 capacity many\n").expect_err("not a number");
    assert_eq!(error.kind(), ScenarioErrorKind::BadArgument);
}

#[test]
fn an_unknown_resource_in_a_window_is_reported() {
    let error = Scenario::parse("scenario demo\nwindow 3 0 10\n").expect_err("no such resource");
    assert_eq!(error.kind(), ScenarioErrorKind::UnknownResource);
    assert_eq!(error.line(), Some(2));
}

#[test]
fn a_requirement_for_a_missing_resource_is_reported() {
    let text = "scenario demo\ntask 0 duration 60\nrequire 0 9 1\n";
    let error = Scenario::parse(text).expect_err("no such resource");
    assert_eq!(error.kind(), ScenarioErrorKind::UnknownResource);
}

#[test]
fn a_qualifier_before_any_task_is_reported() {
    let error = Scenario::parse("scenario demo\ntask_priority 0 5\n").expect_err("no task yet");
    assert_eq!(error.kind(), ScenarioErrorKind::MissingArgument);
    assert_eq!(error.line(), Some(2));
}

#[test]
fn precedence_may_name_tasks_in_either_order_in_the_file() {
    // The three passes let a constraint name a task defined later.
    let text = "\
scenario demo
resource 0 capacity 1
window 0 0 600
task 1 duration 90
precedence 0 1
task 0 duration 60
";
    let scenario = Scenario::parse(text).expect("the text parses");
    assert!(
        scenario
            .precedence()
            .has_constraint(TaskId::new(0), TaskId::new(1))
    );
}

#[test]
fn a_requirement_may_name_a_resource_defined_later() {
    let text = "\
scenario demo
task 0 duration 60
require 0 0 1
resource 0 capacity 1
window 0 0 600
";
    let scenario = Scenario::parse(text).expect("the text parses");
    assert_eq!(scenario.tasks()[0].requirement_count(), 1);
}

#[test]
fn comments_and_blank_lines_are_ignored() {
    let text = "\
# leading comment
scenario demo

   # indented comment
resource 0 capacity 1

window 0 0 600
";
    let scenario = Scenario::parse(text).expect("the text parses");
    assert_eq!(scenario.resources().len(), 1);
}

#[test]
fn a_cyclic_precedence_is_reported() {
    let text = "\
scenario demo
task 0 duration 60
task 1 duration 60
precedence 0 1
precedence 1 0
";
    let error = Scenario::parse(text).expect_err("the cycle is refused");
    assert_eq!(error.kind(), ScenarioErrorKind::CyclicPrecedence);
    // The second constraint, on the fifth line, is the one that closes the
    // cycle.
    assert_eq!(error.line(), Some(5));
}

#[test]
fn a_scenario_displays_its_shape() {
    let mut scenario = Scenario::new("demo");
    scenario
        .add_resource(station(1.0, &[]))
        .expect("the resource is new");
    scenario
        .add_task(Task::new(TaskId::new(0), 60.0))
        .expect("the task is usable");
    let text = scenario.to_string();
    assert!(text.contains("demo"), "{text}");
    assert!(text.contains('1'), "{text}");
}

#[test]
fn an_error_displays_its_line_and_kind() {
    let error = Scenario::parse("scenario demo\norbit 1\n").expect_err("unknown keyword");
    let text = error.to_string();
    assert!(text.contains("line 2"), "{text}");
    assert!(text.contains("keyword"), "{text}");
}

#[test]
fn an_empty_name_is_refused_by_validation() {
    let scenario = Scenario::new("");
    let error = scenario.validate().expect_err("the name is empty");
    assert_eq!(error.kind(), ScenarioErrorKind::EmptyName);
}
