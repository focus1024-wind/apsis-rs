//! Validation tests for chain access interval algebra.
//!
//! 链访问区间代数的验证测试。

use apsis::apsis_access::{AccessInterval, chain_passes, intersect_intervals};

fn interval(start: f64, end: f64) -> AccessInterval {
    AccessInterval::new(start, end)
}

#[test]
fn the_intersection_of_two_overlapping_intervals_is_their_overlap() {
    let links = vec![vec![interval(0.0, 10.0)], vec![interval(5.0, 15.0)]];
    let chain = intersect_intervals(&links).expect("valid");
    assert_eq!(chain, vec![interval(5.0, 10.0)]);
}

#[test]
fn three_links_intersect_to_the_common_window() {
    let links = vec![
        vec![interval(0.0, 10.0)],
        vec![interval(2.0, 8.0)],
        vec![interval(5.0, 12.0)],
    ];
    let chain = intersect_intervals(&links).expect("valid");
    assert_eq!(chain, vec![interval(5.0, 8.0)]);
    assert!((chain[0].duration() - 3.0).abs() < 1e-12);
}

#[test]
fn multiple_intervals_produce_multiple_chain_windows() {
    let links = vec![
        vec![interval(0.0, 10.0), interval(20.0, 30.0)],
        vec![interval(5.0, 25.0)],
    ];
    let chain = intersect_intervals(&links).expect("valid");
    assert_eq!(chain, vec![interval(5.0, 10.0), interval(20.0, 25.0)]);
}

#[test]
fn a_link_without_availability_empties_the_chain() {
    let links = vec![vec![interval(0.0, 10.0)], Vec::new()];
    assert!(intersect_intervals(&links).expect("valid").is_empty());
}

#[test]
fn passes_convert_to_the_same_intervals() {
    let pass = apsis::apsis_access::Pass {
        rise_seconds: 100.0,
        set_seconds: 200.0,
        peak_seconds: 150.0,
        peak_elevation: 0.5,
        mask_elevation: 0.1,
        peak_position: apsis::apsis_math::Vec3::ZERO,
    };
    let chain = chain_passes(&[vec![pass]]).expect("valid");
    assert_eq!(chain, vec![interval(100.0, 200.0)]);
}

#[test]
fn invalid_interval_lists_are_rejected() {
    assert!(intersect_intervals(&[]).is_err());
    assert!(intersect_intervals(&[vec![interval(10.0, 10.0)]]).is_err());
    assert!(intersect_intervals(&[vec![interval(10.0, 20.0), interval(15.0, 30.0)]]).is_err());
}
