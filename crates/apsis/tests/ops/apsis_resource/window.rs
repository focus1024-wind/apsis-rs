//! Tests for availability windows and the interval algebra.
//!
//! 可用性窗口以及区间代数的测试。

use apsis::apsis_resource::{Interval, ResourceErrorKind, Window, merge_windows, subtract_window};

#[test]
fn an_interval_reports_its_duration() {
    assert_eq!(Interval::new(10.0, 30.0).duration(), 20.0);
    // An empty or reversed interval has no duration.
    // 空或反向的区间没有时长。
    assert_eq!(Interval::new(30.0, 30.0).duration(), 0.0);
    assert_eq!(Interval::new(30.0, 10.0).duration(), 0.0);
}

#[test]
fn emptiness_and_validity_are_reported() {
    assert!(Interval::new(5.0, 5.0).is_empty());
    assert!(Interval::new(5.0, 4.0).is_empty());
    assert!(!Interval::new(4.0, 5.0).is_empty());
    assert!(Interval::new(4.0, 5.0).is_valid());
    // A reversed interval is not valid, although it is empty.
    // 反向区间无效，尽管它是空的。
    assert!(!Interval::new(5.0, 4.0).is_valid());
    assert!(!Interval::new(f64::NAN, 5.0).is_valid());
    assert!(!Interval::new(4.0, f64::INFINITY).is_valid());
}

#[test]
fn containment_is_half_open() {
    let interval = Interval::new(10.0, 20.0);
    // The start is inside and the end is not.
    // 起点在内而终点不在。
    assert!(!interval.contains(9.999));
    assert!(interval.contains(10.0));
    assert!(interval.contains(15.0));
    assert!(interval.contains(19.999));
    assert!(!interval.contains(20.0));
}

#[test]
fn containing_an_interval_includes_the_boundaries() {
    let outer = Interval::new(0.0, 100.0);
    assert!(outer.contains_interval(&Interval::new(10.0, 20.0)));
    // The boundaries are included, so an interval that shares an end is
    // contained.
    // 各边界被包含，故共享一个端点的区间被包含。
    assert!(outer.contains_interval(&Interval::new(0.0, 100.0)));
    assert!(outer.contains_interval(&Interval::new(0.0, 50.0)));
    assert!(outer.contains_interval(&Interval::new(50.0, 100.0)));
    // An interval that reaches past the end is not.
    // 越过终点的区间则不是。
    assert!(!outer.contains_interval(&Interval::new(50.0, 100.1)));
    assert!(!outer.contains_interval(&Interval::new(-0.1, 50.0)));
}

#[test]
fn an_empty_interval_is_contained_when_its_instant_is() {
    let outer = Interval::new(0.0, 100.0);
    assert!(outer.contains_interval(&Interval::new(50.0, 50.0)));
    assert!(!outer.contains_interval(&Interval::new(200.0, 200.0)));
}

#[test]
fn intervals_that_meet_do_not_overlap() {
    // The half-open convention means the boundary instant belongs to the
    // second interval only.
    // 半开约定意味着边界瞬时只属于第二个区间。
    let first = Interval::new(0.0, 10.0);
    let second = Interval::new(10.0, 20.0);
    assert!(!first.overlaps(&second));
    assert!(!second.overlaps(&first));
    assert!(first.is_adjacent_to(&second));
}

#[test]
fn overlapping_intervals_report_their_intersection() {
    let first = Interval::new(0.0, 10.0);
    let second = Interval::new(5.0, 15.0);
    assert!(first.overlaps(&second));
    assert_eq!(first.intersection(&second), Some(Interval::new(5.0, 10.0)));
    assert_eq!(second.intersection(&first), first.intersection(&second));
    // Disjoint intervals have no intersection.
    // 不相交区间没有交集。
    assert_eq!(
        Interval::new(0.0, 1.0).intersection(&Interval::new(2.0, 3.0)),
        None
    );
}

#[test]
fn the_hull_covers_both_intervals() {
    let first = Interval::new(0.0, 10.0);
    let second = Interval::new(5.0, 15.0);
    assert_eq!(first.hull(&second), Interval::new(0.0, 15.0));
    assert_eq!(first.hull(&second), second.hull(&first));
}

#[test]
fn merging_combines_overlapping_windows() {
    let windows = [
        Window::new(0.0, 10.0),
        Window::new(5.0, 15.0),
        Window::new(30.0, 40.0),
    ];
    let merged = merge_windows(&windows).expect("the windows are usable");
    assert_eq!(merged.len(), 2);
    assert_eq!(merged[0], Window::new(0.0, 15.0));
    assert_eq!(merged[1], Window::new(30.0, 40.0));
}

#[test]
fn merging_combines_windows_that_touch() {
    // Two windows that meet share a boundary instant, which belongs to the
    // second, so the union is one continuous window.
    // 相接的两个窗口共享一个边界瞬时，它属于第二个，故并集是一个连续窗口。
    let merged = merge_windows(&[Window::new(0.0, 10.0), Window::new(10.0, 20.0)])
        .expect("the windows are usable");
    assert_eq!(merged, vec![Window::new(0.0, 20.0)]);
}

#[test]
fn merging_drops_empty_windows() {
    // A zero length window is valid but contributes no availability, so it
    // is dropped. A reversed window is a caller error and is rejected.
    // 零长窗口有效但不贡献可用性，故被丢弃。反向窗口是调用方错误并被拒绝。
    let merged = merge_windows(&[
        Window::new(5.0, 5.0),
        Window::new(0.0, 10.0),
        Window::new(50.0, 50.0),
    ])
    .expect("the windows are usable");
    assert_eq!(merged, vec![Window::new(0.0, 10.0)]);
}

#[test]
fn merging_sorts_windows_into_time_order() {
    let merged = merge_windows(&[
        Window::new(30.0, 40.0),
        Window::new(0.0, 10.0),
        Window::new(15.0, 20.0),
    ])
    .expect("the windows are usable");
    assert_eq!(merged[0].start(), 0.0);
    assert_eq!(merged[1].start(), 15.0);
    assert_eq!(merged[2].start(), 30.0);
}

#[test]
fn merging_an_empty_list_gives_an_empty_list() {
    assert_eq!(merge_windows(&[]).expect("nothing to check"), &[]);
}

#[test]
fn merging_is_idempotent() {
    // The canonical form must be a fixed point, so that comparing two sets
    // is a direct comparison.
    // 规范形式必须是自身的固定点，使比较两个集合成为直接比较。
    let windows = [
        Window::new(0.0, 10.0),
        Window::new(5.0, 15.0),
        Window::new(30.0, 40.0),
    ];
    let once = merge_windows(&windows).expect("usable");
    let twice = merge_windows(&once).expect("usable");
    assert_eq!(once, twice);
}

#[test]
fn a_reversed_window_is_rejected() {
    let error = merge_windows(&[Window::new(10.0, 0.0)]).expect_err("reversed");
    assert_eq!(error.kind(), ResourceErrorKind::ReversedInterval);

    let error = Window::new(10.0, 0.0).validate().expect_err("reversed");
    assert_eq!(error.kind(), ResourceErrorKind::ReversedInterval);
}

#[test]
fn a_non_finite_window_is_rejected() {
    assert_eq!(
        Window::new(f64::NAN, 1.0)
            .validate()
            .expect_err("not finite")
            .kind(),
        ResourceErrorKind::NotFinite
    );
    assert_eq!(
        merge_windows(&[Window::new(0.0, f64::INFINITY)])
            .expect_err("not finite")
            .kind(),
        ResourceErrorKind::NotFinite
    );
}

#[test]
fn subtracting_removes_the_middle_of_a_window() {
    let windows = [Window::new(0.0, 100.0)];
    let result = subtract_window(&windows, Window::new(30.0, 40.0)).expect("usable");
    assert_eq!(
        result,
        vec![Window::new(0.0, 30.0), Window::new(40.0, 100.0)]
    );
}

#[test]
fn subtracting_the_whole_of_a_window_leaves_nothing() {
    let result =
        subtract_window(&[Window::new(0.0, 10.0)], Window::new(0.0, 10.0)).expect("usable");
    assert_eq!(result, &[]);

    // A window that covers more than the stored one also removes it.
    // 覆盖范围超过所存者的窗口也会移除它。
    let result =
        subtract_window(&[Window::new(0.0, 10.0)], Window::new(-5.0, 15.0)).expect("usable");
    assert_eq!(result, &[]);
}

#[test]
fn subtracting_the_start_trims_the_window() {
    let result = subtract_window(&[Window::new(0.0, 10.0)], Window::new(0.0, 4.0)).expect("usable");
    assert_eq!(result, vec![Window::new(4.0, 10.0)]);
}

#[test]
fn subtracting_the_end_trims_the_window() {
    let result =
        subtract_window(&[Window::new(0.0, 10.0)], Window::new(6.0, 10.0)).expect("usable");
    assert_eq!(result, vec![Window::new(0.0, 6.0)]);
}

#[test]
fn subtracting_a_disjoint_window_changes_nothing() {
    let windows = [Window::new(0.0, 10.0), Window::new(20.0, 30.0)];
    let result = subtract_window(&windows, Window::new(12.0, 18.0)).expect("usable");
    assert_eq!(result, windows.to_vec());
}

#[test]
fn subtracting_an_empty_window_changes_nothing() {
    let windows = [Window::new(0.0, 10.0)];
    let result = subtract_window(&windows, Window::new(5.0, 5.0)).expect("usable");
    assert_eq!(result, windows.to_vec());
}

#[test]
fn subtracting_from_several_windows_affects_each() {
    let windows = [Window::new(0.0, 10.0), Window::new(20.0, 30.0)];
    let result = subtract_window(&windows, Window::new(5.0, 25.0)).expect("usable");
    assert_eq!(result, vec![Window::new(0.0, 5.0), Window::new(25.0, 30.0)]);
}

#[test]
fn subtracting_merges_the_windows_first() {
    // The two overlapping windows form one, which the subtraction then
    // splits. Working on the unmerged list would give a different answer.
    // 两个重叠窗口合为一个，减法随后又把它分开。在未合并的列表上工作会给出不同的
    // 答案。
    let windows = [Window::new(0.0, 10.0), Window::new(5.0, 15.0)];
    let result = subtract_window(&windows, Window::new(8.0, 12.0)).expect("usable");
    assert_eq!(result, vec![Window::new(0.0, 8.0), Window::new(12.0, 15.0)]);
}

#[test]
fn a_window_displays_its_interval() {
    assert_eq!(Window::new(1.5, 2.5).to_string(), "[1.5, 2.5)");
    assert_eq!(Interval::new(0.0, 1.0).to_string(), "[0, 1)");
}

#[test]
fn a_window_can_be_built_from_an_interval() {
    let interval = Interval::new(3.0, 7.0);
    let window = Window::from_interval(interval);
    assert_eq!(window.interval, interval);
    assert_eq!(window.duration(), 4.0);
}
