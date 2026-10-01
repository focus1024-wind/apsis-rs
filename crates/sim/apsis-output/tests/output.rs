//! Tests for the output trace and recorder.
//!
//! 输出轨迹与记录器的测试。

use apsis_output::{Record, RecordKind, Recorder, Trace};

#[test]
fn an_empty_trace_has_nothing() {
    let trace = Trace::new();
    assert!(trace.is_empty());
    assert_eq!(trace.len(), 0);
    assert!(trace.is_stable());
    assert!(trace.records().is_empty());
    assert_eq!(trace.labels(), Vec::<&str>::new());
    assert_eq!(trace.value_of("anything"), None);
    assert_eq!(trace.summarize("anything"), None);
    assert_eq!(trace.warning_count(), 0);
    assert_eq!(trace.longest_series(), 0);
}

#[test]
fn a_trace_keeps_insertion_order() {
    // The order a model recorded in is part of what it decided, so the trace
    // does not sort.
    // 模型记录所用的次序是其决策的一部分，故轨迹不排序。
    let mut trace = Trace::new();
    trace.push(Record::observation(10.0, "b", 1.0));
    trace.push(Record::observation(5.0, "a", 2.0));
    let labels: Vec<&str> = trace
        .records()
        .iter()
        .map(|record| record.label.as_str())
        .collect();
    assert_eq!(labels, vec!["b", "a"]);
}

#[test]
fn time_order_is_available_on_request() {
    let mut trace = Trace::new();
    trace.push(Record::observation(10.0, "b", 1.0));
    trace.push(Record::observation(5.0, "a", 2.0));
    trace.push(Record::observation(7.0, "c", 3.0));
    let ordered = trace.in_time_order();
    let times: Vec<f64> = ordered.iter().map(|record| record.time).collect();
    assert_eq!(times, vec![5.0, 7.0, 10.0]);
    // The original is unchanged.
    // 原件不变。
    assert_eq!(trace.records()[0].time, 10.0);
}

#[test]
fn time_order_is_stable_for_equal_times() {
    let mut trace = Trace::new();
    trace.push(Record::observation(5.0, "first", 1.0));
    trace.push(Record::observation(5.0, "second", 2.0));
    trace.push(Record::observation(5.0, "third", 3.0));
    let ordered = trace.in_time_order();
    let labels: Vec<&str> = ordered.iter().map(|record| record.label.as_str()).collect();
    assert_eq!(labels, vec!["first", "second", "third"]);
}

#[test]
fn records_can_be_filtered_by_kind() {
    let mut trace = Trace::new();
    trace.push(Record::observation(1.0, "a", 1.0));
    trace.push(Record::warning(2.0, "b", 2.0));
    trace.push(Record::state(3.0, "c", 3.0));
    trace.push(Record::warning(4.0, "d", 4.0));

    assert_eq!(trace.of_kind(RecordKind::Observation).len(), 1);
    assert_eq!(trace.of_kind(RecordKind::Warning).len(), 2);
    assert_eq!(trace.of_kind(RecordKind::State).len(), 1);
    assert_eq!(trace.warning_count(), 2);
}

#[test]
fn records_can_be_filtered_by_label() {
    let mut trace = Trace::new();
    trace.push(Record::observation(1.0, "range", 1_000.0));
    trace.push(Record::observation(2.0, "range", 900.0));
    trace.push(Record::observation(3.0, "rate", 5.0));
    assert_eq!(trace.with_label("range").len(), 2);
    assert_eq!(trace.with_label("rate").len(), 1);
    assert_eq!(trace.with_label("absent").len(), 0);
}

#[test]
fn the_last_value_of_a_label_is_reported() {
    let mut trace = Trace::new();
    trace.push(Record::observation(1.0, "range", 1_000.0));
    trace.push(Record::observation(5.0, "range", 900.0));
    trace.push(Record::observation(9.0, "range", 950.0));
    assert_eq!(trace.value_of("range"), Some(950.0));
    assert_eq!(trace.last_time_of("range"), Some(9.0));
    assert_eq!(trace.value_of("absent"), None);
}

#[test]
fn the_labels_are_unique_and_sorted() {
    let mut trace = Trace::new();
    trace.push(Record::observation(1.0, "zeta", 1.0));
    trace.push(Record::observation(2.0, "alpha", 2.0));
    trace.push(Record::observation(3.0, "zeta", 3.0));
    assert_eq!(trace.labels(), vec!["alpha", "zeta"]);
}

#[test]
fn a_series_summarizes_its_values() {
    let mut trace = Trace::new();
    for (time, value) in [(0.0, 10.0), (1.0, 20.0), (2.0, 30.0), (3.0, 5.0)] {
        trace.push(Record::observation(time, "x", value));
    }
    let summary = trace.summarize("x").expect("the series exists");
    assert_eq!(summary.label, "x");
    assert_eq!(summary.count, 4);
    assert_eq!(summary.min, 5.0);
    assert_eq!(summary.max, 30.0);
    assert_eq!(summary.first, 10.0);
    assert_eq!(summary.last, 5.0);
    assert_eq!(summary.change, -5.0);
    assert_eq!(summary.mean, 65.0 / 4.0);
    assert_eq!(summary.range(), 25.0);
    assert!(!summary.is_constant());
}

#[test]
fn a_constant_series_is_recognized() {
    let mut trace = Trace::new();
    for time in 0..3 {
        trace.push(Record::observation(f64::from(time), "x", 7.0));
    }
    let summary = trace.summarize("x").expect("the series exists");
    assert!(summary.is_constant());
    assert_eq!(summary.range(), 0.0);
    assert_eq!(summary.change, 0.0);
}

#[test]
fn the_summaries_cover_every_label_in_order() {
    let mut trace = Trace::new();
    trace.push(Record::observation(1.0, "b", 1.0));
    trace.push(Record::observation(1.0, "a", 2.0));
    trace.push(Record::observation(2.0, "b", 3.0));
    let summaries = trace.summaries();
    assert_eq!(summaries.len(), 2);
    assert_eq!(summaries[0].label, "a");
    assert_eq!(summaries[1].label, "b");
    assert_eq!(summaries[1].count, 2);
}

#[test]
fn the_longest_series_is_reported() {
    let mut trace = Trace::new();
    for index in 0..5 {
        trace.push(Record::observation(f64::from(index), "x", 1.0));
    }
    for index in 0..2 {
        trace.push(Record::observation(f64::from(index), "y", 1.0));
    }
    assert_eq!(trace.longest_series(), 5);
}

#[test]
fn a_non_finite_record_is_refused() {
    let mut trace = Trace::new();
    assert!(!trace.push(Record::observation(f64::NAN, "x", 1.0)));
    assert!(!trace.push(Record::observation(1.0, "x", f64::INFINITY)));
    assert!(!trace.push(Record::observation(1.0, "", 1.0)));
    assert!(trace.is_empty());

    // A finite record is accepted.
    // 有限的记录被接受。
    assert!(trace.push(Record::observation(1.0, "x", 1.0)));
    assert_eq!(trace.len(), 1);
    assert!(trace.is_stable());
}

#[test]
fn clearing_empties_the_trace() {
    let mut trace = Trace::new();
    trace.push(Record::observation(1.0, "x", 1.0));
    trace.push(Record::observation(2.0, "x", 2.0));
    assert_eq!(trace.clear(), 2);
    assert!(trace.is_empty());
}

#[test]
fn the_recorder_collects_records_of_each_kind() {
    let mut recorder = Recorder::new();
    assert!(recorder.observe(1.0, "range", 100.0));
    assert!(recorder.note(2.0, "eclipse", 1.0));
    assert!(recorder.warn(3.0, "temperature", 90.0));

    assert_eq!(recorder.len(), 3);
    assert!(!recorder.is_empty());
    let trace = recorder.trace();
    assert_eq!(trace.of_kind(RecordKind::Observation).len(), 1);
    assert_eq!(trace.of_kind(RecordKind::State).len(), 1);
    assert_eq!(trace.of_kind(RecordKind::Warning).len(), 1);
}

#[test]
fn the_recorder_counts_what_it_refused() {
    let mut recorder = Recorder::new();
    assert!(!recorder.observe(f64::NAN, "x", 1.0));
    assert!(!recorder.observe(1.0, "x", f64::NAN));
    assert!(recorder.observe(1.0, "x", 1.0));
    assert_eq!(recorder.refused(), 2);
    assert_eq!(recorder.len(), 1);
}

#[test]
fn taking_the_trace_leaves_the_recorder_empty() {
    let mut recorder = Recorder::new();
    recorder.observe(1.0, "x", 1.0);
    recorder.observe(2.0, "x", 2.0);

    let trace = recorder.take_trace();
    assert_eq!(trace.len(), 2);
    assert!(recorder.is_empty());
    // The refused count belongs to the recorder and is not reset.
    // 被拒绝计数属于记录器且不重置。
    recorder.observe(f64::NAN, "x", 1.0);
    assert_eq!(recorder.refused(), 1);
}

#[test]
fn the_recorder_can_be_cleared_without_taking() {
    let mut recorder = Recorder::new();
    recorder.observe(1.0, "x", 1.0);
    assert_eq!(recorder.clear(), 1);
    assert!(recorder.is_empty());
    assert_eq!(recorder.trace().len(), 0);
}

#[test]
fn a_trace_can_be_built_from_records() {
    let records = vec![
        Record::observation(1.0, "a", 1.0),
        Record::observation(2.0, "b", 2.0),
    ];
    let trace = Trace::from_records(records);
    assert_eq!(trace.len(), 2);
    assert_eq!(trace.value_of("b"), Some(2.0));
}

#[test]
fn a_record_displays_its_shape() {
    let record = Record::warning(3.5, "temperature", 90.0);
    let text = record.to_string();
    assert!(text.contains("warning"), "{text}");
    assert!(text.contains("temperature"), "{text}");
    assert!(text.contains("90"), "{text}");
}

#[test]
fn the_record_kinds_report_themselves() {
    assert_eq!(RecordKind::Observation.name(), "observation");
    assert_eq!(RecordKind::State.name(), "state");
    assert_eq!(RecordKind::Warning.name(), "warning");
    assert!(RecordKind::Warning.is_warning());
    assert!(!RecordKind::State.is_warning());
    assert_eq!(RecordKind::State.to_string(), "state");
}

#[test]
fn a_record_reports_whether_it_is_usable() {
    assert!(Record::observation(1.0, "x", 1.0).is_valid());
    assert!(!Record::observation(f64::NAN, "x", 1.0).is_valid());
    assert!(!Record::observation(1.0, "x", f64::NAN).is_valid());
    assert!(!Record::observation(1.0, "", 1.0).is_valid());
}

#[test]
fn the_trace_default_is_empty() {
    let trace = Trace::default();
    assert!(trace.is_empty());
}

#[test]
fn the_recorder_default_is_empty() {
    let recorder = Recorder::default();
    assert!(recorder.is_empty());
    assert_eq!(recorder.refused(), 0);
}
