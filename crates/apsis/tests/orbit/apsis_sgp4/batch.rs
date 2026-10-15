//! Tests for the batch propagation entry points.
//!
//! The property that matters is that the parallel forms produce exactly the
//! values the sequential forms do, in the same order. Every test compares
//! against the sequential answer rather than against a constant, so a
//! mismatch in either the propagation or the ordering is caught.
//!
//! 批量传播入口的测试。
//!
//! 要紧的性质是并行形式产生与顺序形式完全相同的值，且次序相同。每个测试都与顺序
//! 答案而非常量比较，故传播或排序中的任何不一致都会被捕获。

use apsis::apsis_sgp4::{
    GravityModel, Satellite, TwoLineElement, propagate_at, propagate_at_parallel,
    propagate_minutes_at, propagate_minutes_at_parallel,
};

const SAMPLE: &str = concat!(
    "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\r\n",
    "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537\r\n",
);

fn satellite() -> Satellite {
    let element = TwoLineElement::parse(SAMPLE).expect("the sample parses");
    Satellite::from_element(&element, GravityModel::WGS72).expect("the orbit propagates")
}

#[test]
fn a_short_track_matches_the_sequential_form() {
    let satellite = satellite();
    let minutes: Vec<f64> = (0..100).map(|index| f64::from(index) * 1.0).collect();
    let sequential = propagate_minutes_at(&satellite, &minutes);
    let parallel = propagate_minutes_at_parallel(&satellite, &minutes);
    assert_eq!(parallel, sequential);
}

#[test]
fn a_long_track_matches_the_sequential_form() {
    // A day at a one minute step, which is long enough that the parallel path
    // takes the multi thread branch.
    // 一分钟步长的一天，长到足以使并行路径走多线程分支。
    let satellite = satellite();
    let minutes: Vec<f64> = (0..1_441).map(f64::from).collect();
    let sequential = propagate_minutes_at(&satellite, &minutes);
    let parallel = propagate_minutes_at_parallel(&satellite, &minutes);
    assert_eq!(parallel, sequential);
    assert_eq!(parallel.len(), 1_441);
}

#[test]
fn a_backwards_track_matches() {
    // Negative minutes propagate backwards and must behave the same way in
    // both forms.
    // 负分钟数向后传播，且在两种形式中必须表现相同。
    let satellite = satellite();
    let minutes: Vec<f64> = (0..2_000).map(|index| -f64::from(index) * 0.5).collect();
    assert_eq!(
        propagate_minutes_at_parallel(&satellite, &minutes),
        propagate_minutes_at(&satellite, &minutes)
    );
}

#[test]
fn an_empty_track_is_empty() {
    let satellite = satellite();
    assert_eq!(propagate_minutes_at_parallel(&satellite, &[]), &[]);
}

#[test]
fn a_decayed_instant_is_reported_in_both_forms() {
    // Far enough ahead that the drag term brings the satellite down, both
    // forms must agree that the instant is unusable rather than one of them
    // stopping early.
    // 足够远，使阻力项把卫星带下来，两种形式必须一致认为该瞬时不可用，而非其中之
    // 一提前停止。
    let satellite = satellite();
    let minutes = vec![0.0, 1.0, 60.0 * 24.0 * 365.0 * 20.0];
    let sequential = propagate_minutes_at(&satellite, &minutes);
    let parallel = propagate_minutes_at_parallel(&satellite, &minutes);
    assert_eq!(parallel, sequential);
}

#[test]
fn a_shell_at_one_instant_matches() {
    let satellite = satellite();
    let shell = vec![satellite; 500];
    let instant = 54_723.5;
    assert_eq!(
        propagate_at_parallel(&shell, instant),
        propagate_at(&shell, instant)
    );
}

#[test]
fn a_shell_result_is_in_input_order() {
    // The states of a repeated satellite are identical, so ordering is checked
    // through the minutes form, where each value differs.
    // 重复卫星的状态相同，故通过分钟形式检查次序，那里每个值都不同。
    let satellite = satellite();
    let minutes: Vec<f64> = (0..3_000).map(|index| f64::from(index) * 0.25).collect();
    let parallel = propagate_minutes_at_parallel(&satellite, &minutes);
    for (index, state) in parallel.iter().enumerate() {
        let state = state.as_ref().expect("the orbit has not decayed here");
        let direct = satellite
            .propagate_minutes(minutes[index])
            .expect("the orbit propagates");
        assert_eq!(state.position, direct.position, "at index {index}");
    }
}
