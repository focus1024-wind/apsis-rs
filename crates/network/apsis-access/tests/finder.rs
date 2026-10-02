//! Tests for the access window finder.
//!
//! The reference case is a circular equatorial orbit at 550 km altitude seen
//! from a station on the equator with a ten degree mask. For that geometry
//! the elevation is a function of the orbital position alone, so the rise,
//! peak, and set times have a closed form:
//!
//! <div>
//! $$
//! \begin{aligned}
//! \theta_c &= 90^\circ - \text{mask} - \arcsin\!\left(\frac{\text{Re} \cos(\text{mask})}{\text{Rs}}\right) \\
//! t &= \frac{\theta - \theta_0}{n}
//! \end{aligned}
//! $$
//! </div>
//!
//! with `theta` the orbital position measured from the station and `n` the
//! mean motion. The values used below are computed from those expressions.
//!
//! 可见性窗口查找器的测试。
//!
//! 参考用例是自赤道上、遮蔽十度的一座站看 550 km 高度的圆赤道轨道。对该几何，
//! 仰角只依赖于轨道位置，故升起、峰值与降落时刻有闭式：
//!
//! <div>
//! $$
//! \begin{aligned}
//! \theta_c &= 90^\circ - \text{mask} - \arcsin\!\left(\frac{\text{Re} \cos(\text{mask})}{\text{Rs}}\right) \\
//! t &= \frac{\theta - \theta_0}{n}
//! \end{aligned}
//! $$
//! </div>
//!
//! 其中 `theta` 是自站量起的轨道位置，`n` 是平运动。下面所用的值即由这些表达式
//! 计算。

use apsis_access::{AccessErrorKind, AccessFinder, OrbitSample, Pass, PassSummary};
use apsis_coverage::GroundPoint;
use apsis_geodesy::Geodetic;
use apsis_math::Vec3;
use core::f64::consts::{FRAC_PI_2, PI};

/// The Earth equatorial radius, in kilometers.
///
/// 地球赤道半径，单位为千米。
const BODY_RADIUS: f64 = 6_378.137;

/// The orbit radius for a 550 km circular orbit, in kilometers.
///
/// 550 km 圆轨道的轨道半径，单位为千米。
const ORBIT_RADIUS: f64 = BODY_RADIUS + 550.0;

/// The mean motion of the reference orbit, in radians per second.
///
/// This is `sqrt(mu / a^3)` with the WGS 84 gravitational parameter.
///
/// 参考轨道的平运动，单位为弧度每秒。
///
/// 这是使用 WGS 84 引力参数的 `sqrt(mu / a^3)`。
const MEAN_MOTION: f64 = 0.001_094_823_692_885_802;

/// A station on the equator with a ten degree mask.
///
/// 赤道上、遮蔽十度的一座站。
fn station() -> GroundPoint {
    GroundPoint::new(Geodetic::new(0.0, 0.0, 0.0), 10.0_f64.to_radians())
}

/// A finder with a one second step and a millisecond tolerance.
///
/// 一秒步长、毫秒容差的查找器。
fn finder() -> AccessFinder {
    AccessFinder {
        station: station(),
        sample_step_seconds: 1.0,
        crossing_tolerance_seconds: 0.001,
    }
}

/// The orbital position measured from the station at the start of the
/// search, in radians.
///
/// The reference closed form solves for the crossing times relative to an
/// initial position of minus one radian, so the track starts there.
///
/// 搜索开始时自站量起的轨道位置，单位为弧度。
///
/// 参考闭式是相对初始位置负一弧度求解穿越时刻，故轨迹从那里开始。
const INITIAL_ANGLE: f64 = -1.0;

/// Returns the satellite position for a reference orbit that passes over the
/// station.
///
/// The orbit is circular and equatorial, so the position depends only on the
/// angular position at the given time. The angle is measured from the station
/// and increases at the mean motion, so the elevation is a function of that
/// angle alone.
///
/// 返回一条自该站正上方经过的参考轨道的卫星位置。
///
/// 轨道是圆形的且位于赤道面，故位置只依赖于给定时刻的角位置。该角自站量起并以平
/// 运动增大，故仰角只是该角的函数。
fn reference_position(seconds: f64) -> Vec3 {
    let angle = INITIAL_ANGLE + MEAN_MOTION * seconds;
    Vec3::new(ORBIT_RADIUS * angle.cos(), ORBIT_RADIUS * angle.sin(), 0.0)
}

/// Returns the samples of the reference orbit over a window around one pass.
///
/// 返回参考轨道在一次过境附近窗口上的样本。
fn reference_samples(step: f64, start: f64, end: f64) -> Vec<OrbitSample> {
    let mut samples = Vec::new();
    let mut seconds = start;
    while seconds <= end {
        samples.push(OrbitSample::new(seconds, reference_position(seconds)));
        seconds += step;
    }
    samples
}

/// The half angle of the footprint for the reference geometry, in radians.
///
/// This is `90 - mask - asin(Re cos(mask) / Rs)`, which for the reference
/// case is 14.9566 degrees.
///
/// 参考几何的覆盖区半角，单位为弧度。
///
/// 这是 `90 - mask - asin(Re cos(mask) / Rs)`，对参考用例为 14.9566 度。
const FOOTPRINT_HALF_ANGLE: f64 = 0.261_042_364_648_894_9;

/// The rise time of the reference pass from the start of the search, in
/// seconds.
///
/// 参考过境自搜索起点起的升起时刻，单位为秒。
const REFERENCE_RISE: f64 = 674.955_681_770_3;

/// The peak time of the reference pass, in seconds.
///
/// 参考过境的峰值时刻，单位为秒。
const REFERENCE_PEAK: f64 = 913.389_074_878_5;

/// The set time of the reference pass, in seconds.
///
/// 参考过境的降落时刻，单位为秒。
const REFERENCE_SET: f64 = 1_151.822_467_986_6;

#[test]
fn the_finder_settings_are_validated() {
    assert!(finder().is_valid());

    let coarse = AccessFinder {
        sample_step_seconds: 0.0,
        ..finder()
    };
    assert!(!coarse.is_valid());
    assert_eq!(
        coarse
            .find(&reference_samples(1.0, 0.0, 10.0))
            .expect_err("zero step")
            .kind(),
        AccessErrorKind::InvalidSampleStep
    );

    let loose = AccessFinder {
        crossing_tolerance_seconds: 10.0,
        ..finder()
    };
    assert!(!loose.is_valid());
    assert_eq!(
        loose
            .find(&reference_samples(1.0, 0.0, 10.0))
            .expect_err("tolerance exceeds the step")
            .kind(),
        AccessErrorKind::InvalidTolerance
    );

    let bad_station = AccessFinder {
        station: GroundPoint::new(Geodetic::new(0.0, 0.0, 0.0), PI),
        ..finder()
    };
    assert!(!bad_station.is_valid());
    assert_eq!(
        bad_station
            .find(&reference_samples(1.0, 0.0, 10.0))
            .expect_err("invalid station")
            .kind(),
        AccessErrorKind::InvalidStation
    );
}

#[test]
fn the_reference_pass_matches_the_closed_form() {
    let samples = reference_samples(1.0, 0.0, 2_000.0);
    let summary = finder().find(&samples).expect("valid");
    assert_eq!(summary.count(), 1, "expected exactly one pass");

    let pass = &summary.passes[0];
    assert!(
        (pass.rise_seconds - REFERENCE_RISE).abs() < 0.5,
        "rise {} against {REFERENCE_RISE}",
        pass.rise_seconds
    );
    assert!(
        (pass.peak_seconds - REFERENCE_PEAK).abs() < 1.0,
        "peak {} against {REFERENCE_PEAK}",
        pass.peak_seconds
    );
    assert!(
        (pass.set_seconds - REFERENCE_SET).abs() < 0.5,
        "set {} against {REFERENCE_SET}",
        pass.set_seconds
    );
}

#[test]
fn the_peak_of_an_overhead_pass_is_a_right_angle() {
    let samples = reference_samples(1.0, 0.0, 2_000.0);
    let summary = finder().find(&samples).expect("valid");
    let pass = &summary.passes[0];
    // The reference geometry passes directly over the station, so the
    // greatest elevation is a right angle.
    // 参考几何自该站正上方经过，故最大仰角是直角。
    assert!(
        (pass.peak_elevation - FRAC_PI_2).abs() < 0.01,
        "peak elevation {}",
        pass.peak_elevation
    );
}

#[test]
fn the_pass_is_symmetric_about_the_peak() {
    let samples = reference_samples(1.0, 0.0, 2_000.0);
    let summary = finder().find(&samples).expect("valid");
    let pass = &summary.passes[0];
    let rise_to_peak = pass.rise_to_peak();
    let peak_to_set = pass.set_seconds - pass.peak_seconds;
    assert!(
        (rise_to_peak - peak_to_set).abs() < 1.0,
        "{rise_to_peak} against {peak_to_set}"
    );
}

#[test]
fn the_duration_matches_the_geometry() {
    let samples = reference_samples(1.0, 0.0, 2_000.0);
    let summary = finder().find(&samples).expect("valid");
    let pass = &summary.passes[0];
    // The orbital position spans twice the footprint half angle, so the
    // duration is 2 theta_c / n.
    // 轨道位置跨越覆盖区半角的两倍，故时长为 2 theta_c / n。
    let expected = 2.0 * FOOTPRINT_HALF_ANGLE / MEAN_MOTION;
    assert!(
        (pass.duration() - expected).abs() < 1.0,
        "{} against {expected}",
        pass.duration()
    );
}

#[test]
fn the_refinement_is_independent_of_the_sample_step() {
    // The crossing times are found by interpolating the satellite position
    // between samples, so the result approaches the true crossing as the
    // step falls. The residual difference between two steps is second order
    // in the step, because the chord between two samples departs from the
    // arc by that order.
    // 穿越时刻通过样本之间插值卫星位置求得，故结果随步长减小而趋近真穿越。两个步长
    // 之间的残余差是步长的二阶量，因为两样本之间的弦与弧的偏差为该阶。
    let coarse = finder()
        .find(&reference_samples(10.0, 0.0, 2_000.0))
        .expect("valid");
    let fine = finder()
        .find(&reference_samples(1.0, 0.0, 2_000.0))
        .expect("valid");
    assert_eq!(coarse.count(), 1);
    assert_eq!(fine.count(), 1);
    let coarse_pass = &coarse.passes[0];
    let fine_pass = &fine.passes[0];

    // A ten second step gives a crossing within about a tenth of a second
    // of the one found with a one second step, because the chord error is
    // second order.
    // 十秒步长给出的穿越与一秒步长所得相差约十分之一秒，因为弦误差是二阶的。
    assert!(
        (coarse_pass.rise_seconds - fine_pass.rise_seconds).abs() < 0.1,
        "rise {} against {}",
        coarse_pass.rise_seconds,
        fine_pass.rise_seconds
    );
    assert!(
        (coarse_pass.set_seconds - fine_pass.set_seconds).abs() < 0.1,
        "set {} against {}",
        coarse_pass.set_seconds,
        fine_pass.set_seconds
    );
}

#[test]
fn the_tolerance_controls_the_crossing_accuracy() {
    let loose = AccessFinder {
        crossing_tolerance_seconds: 1.0,
        ..finder()
    };
    let tight = AccessFinder {
        crossing_tolerance_seconds: 1e-6,
        ..finder()
    };
    let samples = reference_samples(1.0, 0.0, 2_000.0);
    let loose_pass = loose.find(&samples).expect("valid").passes[0];
    let tight_pass = tight.find(&samples).expect("valid").passes[0];
    // The two agree to within the looser tolerance.
    // 二者在较宽松的容差内一致。
    assert!((loose_pass.rise_seconds - tight_pass.rise_seconds).abs() <= 1.0);
    assert!((loose_pass.set_seconds - tight_pass.set_seconds).abs() <= 1.0);
}

#[test]
fn a_full_orbit_finds_exactly_one_pass() {
    // The reference geometry gives one overhead pass per revolution, so a
    // window of one period contains one pass.
    // 参考几何每圈给出一次过顶，故一个周期的窗口含一次过境。
    let period = 2.0 * PI / MEAN_MOTION;
    let samples = reference_samples(1.0, 0.0, period);
    let summary = finder().find(&samples).expect("valid");
    assert_eq!(summary.count(), 1);
}

#[test]
fn the_summary_reports_the_access_fraction() {
    let samples = reference_samples(1.0, 0.0, 2_000.0);
    let summary = finder().find(&samples).expect("valid");
    let fraction = summary.access_fraction().expect("a positive interval");
    let expected = summary.total_access_seconds() / 2_000.0;
    assert!((fraction - expected).abs() < 1e-12);
    // One overhead pass in a two thousand second window is about a quarter
    // of the time.
    // 两千秒窗口中的一次过顶约占四分之一时间。
    assert!(fraction > 0.2 && fraction < 0.3, "fraction {fraction}");
}

#[test]
fn the_summary_finds_the_longest_and_highest_pass() {
    let samples = reference_samples(1.0, 0.0, 2_000.0);
    let summary = finder().find(&samples).expect("valid");
    assert!(summary.longest().is_some());
    assert!(summary.highest().is_some());
    assert!(summary.mean_duration_seconds().is_some());
}

#[test]
fn a_pass_that_starts_before_the_window_is_clipped() {
    // Starting the search after the rise means the pass is reported from
    // the first sample rather than extrapolated backwards.
    // 在升起之后开始搜索意味着过境自第一个样本起报告，而不向后外推。
    let samples = reference_samples(1.0, 800.0, 1_400.0);
    let summary = finder().find(&samples).expect("valid");
    assert_eq!(summary.count(), 1);
    let pass = &summary.passes[0];
    assert!((pass.rise_seconds - 800.0).abs() < 1e-9);
    assert!((pass.set_seconds - REFERENCE_SET).abs() < 1.0);
}

#[test]
fn a_pass_that_ends_after_the_window_is_clipped() {
    let samples = reference_samples(1.0, 0.0, 1_000.0);
    let summary = finder().find(&samples).expect("valid");
    assert_eq!(summary.count(), 1);
    let pass = &summary.passes[0];
    assert!((pass.rise_seconds - REFERENCE_RISE).abs() < 1.0);
    assert!((pass.set_seconds - 1_000.0).abs() < 1e-9);
}

#[test]
fn a_window_with_no_pass_is_empty() {
    // A window before the satellite comes into view contains no pass.
    // 在卫星进入视野之前的窗口不含过境。
    let samples = reference_samples(1.0, -2_000.0, -1_000.0);
    let summary = finder().find(&samples).expect("valid");
    assert_eq!(summary.count(), 0);
    assert_eq!(summary.total_access_seconds(), 0.0);
    assert_eq!(summary.longest(), None);
    assert_eq!(summary.mean_duration_seconds(), None);
    assert_eq!(summary.longest_gap_seconds(), None);
}

#[test]
fn the_gap_between_passes_is_reported() {
    // Two orbits give two passes, and the gap between them is the period
    // less the pass duration.
    // 两个轨道给出两次过境，其间的间隙是周期减去过境时长。
    let period = 2.0 * PI / MEAN_MOTION;
    let samples = reference_samples(1.0, 0.0, 2.0 * period);
    let summary = finder().find(&samples).expect("valid");
    assert!(summary.count() >= 2, "count {}", summary.count());

    let gap = summary.gap_seconds(0, 1).expect("consecutive passes");
    let expected = period - summary.passes[0].duration();
    assert!((gap - expected).abs() < 2.0, "{gap} against {expected}");
    // The largest gap is at least that one.
    // 最大间隙至少是该值。
    let longest = summary.longest_gap_seconds().expect("at least two passes");
    assert!(longest >= gap - 1e-9);
}

#[test]
fn a_non_consecutive_gap_query_is_rejected() {
    let samples = reference_samples(1.0, 0.0, 2_000.0);
    let summary = finder().find(&samples).expect("valid");
    assert_eq!(summary.gap_seconds(0, 2), None);
    assert_eq!(summary.gap_seconds(1, 0), None);
    assert_eq!(summary.gap_seconds(0, 99), None);
}

#[test]
fn the_pass_reports_containment() {
    let pass = Pass {
        rise_seconds: 100.0,
        set_seconds: 200.0,
        peak_seconds: 150.0,
        peak_elevation: 1.0,
        mask_elevation: 0.1,
        peak_position: Vec3::new(ORBIT_RADIUS, 0.0, 0.0),
    };
    assert!(pass.contains(100.0));
    assert!(pass.contains(150.0));
    assert!(pass.contains(200.0));
    assert!(!pass.contains(99.9));
    assert!(!pass.contains(200.1));
    assert!((pass.duration() - 100.0).abs() < 1e-12);
    assert!((pass.rise_to_peak() - 50.0).abs() < 1e-12);
    assert!((pass.maximum_elevation() - 1.0).abs() < 1e-12);
}

#[test]
fn too_few_samples_are_rejected() {
    let error = finder()
        .find(&[OrbitSample::new(0.0, Vec3::new(ORBIT_RADIUS, 0.0, 0.0))])
        .expect_err("one sample is not enough");
    assert_eq!(error.kind(), AccessErrorKind::InvalidInterval);
    assert_eq!(
        finder().find(&[]).expect_err("no samples").kind(),
        AccessErrorKind::InvalidInterval
    );
}

#[test]
fn an_unordered_sample_sequence_is_rejected() {
    let samples = reference_samples(1.0, 1_000.0, 0.0);
    assert!(samples.is_empty());
    // The ends run backwards.
    // 两端是倒序的。
    let backwards = [
        OrbitSample::new(100.0, reference_position(100.0)),
        OrbitSample::new(50.0, reference_position(50.0)),
    ];
    let error = finder().find(&backwards).expect_err("time runs backwards");
    assert_eq!(error.kind(), AccessErrorKind::UnorderedSamples);

    // The ends are in order but the middle is not. This is the case that a
    // check of the ends alone misses: the interpolation binary searches the
    // list, and a binary search over an unordered list returns a wrong index
    // without failing, so the search would refine a crossing against the
    // wrong pair of positions and report a pass that never happened.
    // 两端有序但中间无序。这正是只检查两端会漏掉的情形：插值对该列表做二分查找，而
    // 对无序列表的二分查找会返回错误下标而不报错，故搜索会针对错误的位置对细化穿越，
    // 并报告一个从未发生过的过境。
    let mut shuffled = reference_samples(60.0, 0.0, 1_200.0);
    assert!(shuffled.len() > 6, "the reference track is long enough");
    shuffled.swap(2, 4);
    let error = finder()
        .find(&shuffled)
        .expect_err("the middle is out of order");
    assert_eq!(error.kind(), AccessErrorKind::UnorderedSamples);
}

#[test]
fn an_invalid_position_is_rejected() {
    let samples = [
        OrbitSample::new(0.0, reference_position(0.0)),
        OrbitSample::new(1.0, Vec3::splat(f64::NAN)),
    ];
    let error = finder()
        .find(&samples)
        .expect_err("the position is not finite");
    assert_eq!(error.kind(), AccessErrorKind::InvalidPosition);
}

#[test]
fn the_finder_can_drive_a_propagator_directly() {
    // The convenience entry point must agree with building the samples by
    // hand.
    // 该便利入口必须与手工构建样本一致。
    let via_samples = finder()
        .find(&reference_samples(1.0, 0.0, 2_000.0))
        .expect("valid");
    let via_propagator = finder()
        .find_with(0.0, 2_000.0, |seconds| Some(reference_position(seconds)))
        .expect("valid");
    assert_eq!(via_samples.count(), via_propagator.count());
    let first = &via_samples.passes[0];
    let second = &via_propagator.passes[0];
    assert!((first.rise_seconds - second.rise_seconds).abs() < 1e-6);
    assert!((first.set_seconds - second.set_seconds).abs() < 1e-6);
}

#[test]
fn a_failing_propagator_is_reported() {
    let error = finder()
        .find_with(0.0, 100.0, |_| None)
        .expect_err("the propagator failed");
    assert_eq!(error.kind(), AccessErrorKind::InvalidPosition);
}

#[test]
fn an_invalid_search_interval_is_rejected() {
    let error = finder()
        .find_with(100.0, 0.0, |seconds| Some(reference_position(seconds)))
        .expect_err("the interval runs backwards");
    assert_eq!(error.kind(), AccessErrorKind::InvalidInterval);
    assert_eq!(
        finder()
            .find_with(0.0, f64::NAN, |seconds| Some(reference_position(seconds)))
            .expect_err("the interval is not finite")
            .kind(),
        AccessErrorKind::InvalidInterval
    );
}

#[test]
fn a_higher_mask_shortens_the_pass() {
    let low = AccessFinder {
        station: GroundPoint::new(Geodetic::new(0.0, 0.0, 0.0), 5.0_f64.to_radians()),
        ..finder()
    };
    let high = AccessFinder {
        station: GroundPoint::new(Geodetic::new(0.0, 0.0, 0.0), 30.0_f64.to_radians()),
        ..finder()
    };
    let samples = reference_samples(1.0, 0.0, 2_000.0);
    let low_duration = low.find(&samples).expect("valid").passes[0].duration();
    let high_duration = high.find(&samples).expect("valid").passes[0].duration();
    assert!(
        low_duration > high_duration,
        "{low_duration} against {high_duration}"
    );
}

#[test]
fn the_summary_is_constructible_directly() {
    let pass = Pass {
        rise_seconds: 0.0,
        set_seconds: 10.0,
        peak_seconds: 5.0,
        peak_elevation: 1.0,
        mask_elevation: 0.1,
        peak_position: Vec3::ZERO,
    };
    let summary = PassSummary::new(vec![pass], 10.0);
    assert_eq!(summary.count(), 1);
    assert!((summary.total_access_seconds() - 10.0).abs() < 1e-12);
    assert!((summary.access_fraction().expect("valid") - 1.0).abs() < 1e-12);
    assert_eq!(PassSummary::new(Vec::new(), 0.0).access_fraction(), None);
}

#[test]
fn the_propagator_entry_point_reaches_the_tolerance_regardless_of_step() {
    // When the propagator is available the refinement evaluates the true
    // position at each bisection point rather than interpolating, so the
    // crossing is accurate to the tolerance however coarse the sampling.
    // 当传播器可用时，细化在每个二分点求真实位置而非插值，故无论采样多粗，穿越都
    // 精确到容差。
    let coarse = AccessFinder {
        sample_step_seconds: 30.0,
        crossing_tolerance_seconds: 1e-6,
        ..finder()
    };
    let fine = AccessFinder {
        sample_step_seconds: 1.0,
        crossing_tolerance_seconds: 1e-6,
        ..finder()
    };
    let via_coarse = coarse
        .find_with(0.0, 2_000.0, |seconds| Some(reference_position(seconds)))
        .expect("valid");
    let via_fine = fine
        .find_with(0.0, 2_000.0, |seconds| Some(reference_position(seconds)))
        .expect("valid");

    assert_eq!(via_coarse.count(), 1);
    assert_eq!(via_fine.count(), 1);
    let first = &via_coarse.passes[0];
    let second = &via_fine.passes[0];
    assert!(
        (first.rise_seconds - second.rise_seconds).abs() < 1e-5,
        "rise {} against {}",
        first.rise_seconds,
        second.rise_seconds
    );
    assert!(
        (first.set_seconds - second.set_seconds).abs() < 1e-5,
        "set {} against {}",
        first.set_seconds,
        second.set_seconds
    );
    // The crossing matches the closed form to the tolerance.
    // 该穿越在容差内与闭式一致。
    assert!(
        (first.rise_seconds - REFERENCE_RISE).abs() < 1e-5,
        "rise {} against {REFERENCE_RISE}",
        first.rise_seconds
    );
}
