//! Tests for the ephemeris table and its interpolation.
//!
//! 星历表及其插值的测试。

use apsis_ephemeris::{
    Ephemeris, EphemerisErrorKind, Sample, Sampler, hermite_position, hermite_state,
    hermite_velocity,
};
use apsis_kepler::{GravityParameter, KeplerianElements};
use apsis_math::Vec3;

/// The circular orbit the interpolation is checked against.
///
/// A circular orbit is the useful case: the state is analytic, so the error
/// of the interpolation is exactly the difference from a known value, and
/// the radius is constant, so a bad reconstruction shows up immediately.
///
/// 插值所对照检查的圆轨道。
///
/// 圆轨道是有用的情形：状态是解析的，故插值误差恰是与已知值之差；半径恒定，故坏的
/// 重构会立即显现。
fn circular() -> KeplerianElements {
    KeplerianElements {
        semi_major_axis: 7_000.0,
        eccentricity: 0.0,
        inclination: 0.9,
        right_ascension: 0.5,
        argument_of_perigee: 0.0,
        true_anomaly: 0.0,
    }
}

/// Returns the exact state of the circular orbit at a time.
///
/// 返回该圆轨道在某时刻的精确状态。
fn exact_state(seconds: f64) -> Sample {
    let elements = circular();
    let mean = 2.0 * core::f64::consts::PI * seconds / elements.period(GravityParameter::WGS84);
    let state = elements
        .with_mean_anomaly(mean)
        .to_state(GravityParameter::WGS84)
        .expect("the circular orbit is valid");
    Sample::new(seconds, state.position, state.velocity)
}

fn sample_table(step: f64, span: f64) -> Ephemeris {
    Sampler::new(0.0, span, step)
        .sample(exact_state)
        .expect("the grid is usable")
}

#[test]
fn a_table_can_be_built_from_samples() {
    let samples = vec![
        Sample::new(0.0, Vec3::new(7_000.0, 0.0, 0.0), Vec3::new(0.0, 7.5, 0.0)),
        Sample::new(
            60.0,
            Vec3::new(0.0, 7_000.0, 0.0),
            Vec3::new(-7.5, 0.0, 0.0),
        ),
    ];
    let ephemeris = Ephemeris::from_samples(samples).expect("two ordered samples");
    assert_eq!(ephemeris.len(), 2);
    assert!(!ephemeris.is_empty());
    assert_eq!(ephemeris.start_time(), 0.0);
    assert_eq!(ephemeris.end_time(), 60.0);
    assert_eq!(ephemeris.duration(), 60.0);
}

#[test]
fn a_table_needs_two_samples() {
    let error = Ephemeris::from_samples(Vec::new()).expect_err("an empty table is refused");
    assert_eq!(error.kind(), EphemerisErrorKind::TooFewSamples);
    let one = vec![Sample::new(0.0, Vec3::ZERO, Vec3::ZERO)];
    let error = Ephemeris::from_samples(one).expect_err("one sample is refused");
    assert_eq!(error.kind(), EphemerisErrorKind::TooFewSamples);
}

#[test]
fn samples_must_be_in_increasing_time_order() {
    let samples = vec![
        Sample::new(10.0, Vec3::ZERO, Vec3::ZERO),
        Sample::new(5.0, Vec3::ZERO, Vec3::ZERO),
    ];
    let error = Ephemeris::from_samples(samples).expect_err("the order is wrong");
    assert_eq!(error.kind(), EphemerisErrorKind::NotMonotonic);

    // Equal times are also refused, because a zero width interval has no
    // interpolation to do.
    // 相等的时间也被拒绝，因为零宽区间没有插值可做。
    let samples = vec![
        Sample::new(5.0, Vec3::ZERO, Vec3::ZERO),
        Sample::new(5.0, Vec3::ZERO, Vec3::ZERO),
    ];
    let error = Ephemeris::from_samples(samples).expect_err("equal times are refused");
    assert_eq!(error.kind(), EphemerisErrorKind::NotMonotonic);
}

#[test]
fn a_non_finite_sample_is_refused() {
    let samples = vec![
        Sample::new(0.0, Vec3::ZERO, Vec3::ZERO),
        Sample::new(60.0, Vec3::new(f64::NAN, 0.0, 0.0), Vec3::ZERO),
    ];
    let error = Ephemeris::from_samples(samples).expect_err("the state is not finite");
    assert_eq!(error.kind(), EphemerisErrorKind::InvalidState);
}

#[test]
fn a_query_at_a_sample_returns_it_exactly() {
    let ephemeris = sample_table(60.0, 600.0);
    for sample in ephemeris.samples() {
        let queried = ephemeris
            .state_at(sample.seconds)
            .expect("the sample time is covered");
        // The interpolation must not perturb a sample, so the returned
        // state is bit for bit the stored one.
        // 插值不得扰动样本，故所返回的状态与所存者逐位相同。
        assert_eq!(queried, *sample, "at {} s", sample.seconds);
    }
}

#[test]
fn a_query_between_samples_is_interpolated() {
    let ephemeris = sample_table(60.0, 600.0);
    let midpoint = ephemeris.state_at(30.0).expect("the time is covered");
    assert_eq!(midpoint.seconds, 30.0);
    // The interpolated position differs from both endpoints.
    // 插值所得位置不同于两个端点。
    assert_ne!(midpoint.position, ephemeris.samples()[0].position);
    assert_ne!(midpoint.position, ephemeris.samples()[1].position);
    // And it is a plausible radius for the orbit.
    // 且它是该轨道的合理半径。
    assert!((midpoint.radius() - 7_000.0).abs() < 1.0);
}

#[test]
fn the_interpolation_is_accurate_on_a_circular_orbit() {
    // Hermite cubic on a smooth orbit: the position error over a step of
    // sixty seconds should be far below a kilometer.
    // 光滑轨道上的 Hermite 三次插值：六十秒步长上的位置误差应远低于一千米。
    let ephemeris = sample_table(60.0, 600.0);
    let mut worst = 0.0_f64;
    for index in 0..600 {
        let time = f64::from(index) + 0.5;
        let interpolated = ephemeris.state_at(time).expect("covered");
        let truth = exact_state(time);
        worst = worst.max(interpolated.position.distance(truth.position));
    }
    assert!(
        worst < 1e-3,
        "the worst position error is {worst} km, which is too large"
    );
}

#[test]
fn halving_the_step_cuts_the_error_by_about_sixteen() {
    // A quartic method: the error scales as the fourth power of the step.
    // This is the property that makes the interpolation worth using, and it
    // fails if the velocity is dropped or mis-scaled.
    // 四阶方法：误差按步长的四次方缩放。这正是使该插值值得使用的性质，若丢掉速度或
    // 缩放错误它就会失败。
    let error_for = |step: f64| {
        let ephemeris = sample_table(step, 600.0);
        let mut worst = 0.0_f64;
        // Query at the midpoints between samples, where the error is
        // largest.
        // 在样本之间的中点查询，那里误差最大。
        let mut time = step / 2.0;
        while time < 600.0 {
            let interpolated = ephemeris.state_at(time).expect("covered");
            let truth = exact_state(time);
            worst = worst.max(interpolated.position.distance(truth.position));
            time += step;
        }
        worst
    };
    let coarse = error_for(120.0);
    let fine = error_for(60.0);
    assert!(coarse > 0.0, "the coarse error should be non-zero");
    let ratio = coarse / fine;
    assert!(
        (8.0..32.0).contains(&ratio),
        "the error ratio is {ratio}, expected about sixteen"
    );
}

#[test]
fn the_interpolated_velocity_matches_the_position_derivative() {
    // The velocity is the analytic derivative of the position cubic, so a
    // numerical derivative of the position must agree with it.
    // 速度是位置三次式的解析导数，故位置的数值导数必须与它一致。
    let ephemeris = sample_table(60.0, 600.0);
    let time = 137.0;
    let h = 1e-4;
    let before = ephemeris.state_at(time - h).expect("covered").position;
    let after = ephemeris.state_at(time + h).expect("covered").position;
    let numeric = (after - before) / (2.0 * h);
    let analytic = ephemeris.state_at(time).expect("covered").velocity;
    assert!(
        numeric.distance(analytic) < 1e-6,
        "numeric {numeric} against analytic {analytic}"
    );
}

#[test]
fn a_query_outside_the_interval_is_refused() {
    let ephemeris = sample_table(60.0, 600.0);
    assert!(ephemeris.state_at(-1.0).is_none());
    assert!(ephemeris.state_at(600.1).is_none());
    assert!(ephemeris.state_at(f64::NAN).is_none());
    assert!(ephemeris.state_at(f64::INFINITY).is_none());
    // The endpoints are covered.
    // 端点被覆盖。
    assert!(ephemeris.covers(0.0));
    assert!(ephemeris.covers(600.0));
    assert!(!ephemeris.covers(-1e-9));
}

#[test]
fn a_clamped_query_returns_the_nearest_endpoint() {
    let ephemeris = sample_table(60.0, 600.0);
    let before = ephemeris.clamped_state_at(-100.0).expect("clamped");
    assert_eq!(before, ephemeris.samples()[0]);
    let after = ephemeris.clamped_state_at(10_000.0).expect("clamped");
    assert_eq!(after, ephemeris.samples()[ephemeris.len() - 1]);
    // Inside the interval the clamped query is the ordinary one.
    // 在区间内，夹逼查询即普通查询。
    let inside = ephemeris.clamped_state_at(30.0).expect("covered");
    assert_eq!(inside, ephemeris.state_at(30.0).expect("covered"));
    assert!(ephemeris.clamped_state_at(f64::NAN).is_none());
}

#[test]
fn the_nearest_sample_is_found() {
    let ephemeris = sample_table(60.0, 600.0);
    assert_eq!(ephemeris.nearest_sample(0.0).expect("found").seconds, 0.0);
    assert_eq!(ephemeris.nearest_sample(29.0).expect("found").seconds, 0.0);
    assert_eq!(ephemeris.nearest_sample(31.0).expect("found").seconds, 60.0);
    // A midpoint ties toward the earlier sample.
    // 中点并列时取较早的样本。
    assert_eq!(ephemeris.nearest_sample(30.0).expect("found").seconds, 0.0);
    // Outside the interval the nearest end is returned.
    // 在区间外返回最近的一端。
    assert_eq!(
        ephemeris.nearest_sample(-100.0).expect("found").seconds,
        0.0
    );
    assert_eq!(
        ephemeris.nearest_sample(10_000.0).expect("found").seconds,
        600.0
    );
    assert!(ephemeris.nearest_sample(f64::NAN).is_none());
}

#[test]
fn a_uniform_grid_is_recognized() {
    let uniform = sample_table(60.0, 600.0);
    assert!(uniform.is_uniform());
    assert_eq!(uniform.uniform_step(), Some(60.0));

    let uneven = Ephemeris::from_samples(vec![
        Sample::new(0.0, Vec3::ZERO, Vec3::ZERO),
        Sample::new(60.0, Vec3::ZERO, Vec3::ZERO),
        Sample::new(150.0, Vec3::ZERO, Vec3::ZERO),
    ])
    .expect("ordered");
    assert!(!uneven.is_uniform());
    assert_eq!(uneven.uniform_step(), None);
}

#[test]
fn the_times_of_the_samples_are_listed() {
    let ephemeris = sample_table(60.0, 300.0);
    assert_eq!(
        ephemeris.times(),
        vec![0.0, 60.0, 120.0, 180.0, 240.0, 300.0]
    );
}

#[test]
fn a_property_of_every_sample_can_be_checked() {
    let ephemeris = sample_table(60.0, 600.0);
    assert!(ephemeris.all_samples(|sample| sample.radius() > 6_000.0));
    assert!(!ephemeris.all_samples(|sample| sample.radius() > 8_000.0));
}

// The sampler's grid.
// 采样器的网格。

#[test]
fn the_grid_includes_both_ends() {
    // The end falls on the grid.
    // 终点落在网格上。
    let sampler = Sampler::new(0.0, 300.0, 60.0);
    assert_eq!(sampler.sample_count(), Some(6));
    assert_eq!(
        sampler.times().expect("usable"),
        vec![0.0, 60.0, 120.0, 180.0, 240.0, 300.0]
    );

    // The end does not fall on the grid, so it is appended.
    // 终点不落在网格上，故追加它。
    let sampler = Sampler::new(0.0, 250.0, 60.0);
    assert_eq!(sampler.sample_count(), Some(6));
    assert_eq!(
        sampler.times().expect("usable"),
        vec![0.0, 60.0, 120.0, 180.0, 240.0, 250.0]
    );
}

#[test]
fn a_single_step_grid_has_two_samples() {
    let sampler = Sampler::new(10.0, 70.0, 60.0);
    assert_eq!(sampler.sample_count(), Some(2));
    assert_eq!(sampler.times().expect("usable"), vec![10.0, 70.0]);
}

#[test]
fn a_zero_length_interval_is_refused() {
    // A table needs at least two distinct times, so a zero length grid has
    // no interval to interpolate over and is rejected outright rather than
    // producing a table that cannot answer a query.
    // 表至少需要两个不同的时刻，故零长度网格没有可插值的区间，会被直接拒绝而非产生
    // 一张无法回答查询的表。
    let sampler = Sampler::new(5.0, 5.0, 60.0);
    assert!(!sampler.is_valid());
    assert_eq!(sampler.sample_count(), None);
    assert!(sampler.times().is_none());
    assert!(sampler.sample(exact_state).is_err());
}

#[test]
fn an_unusable_sampler_is_refused() {
    for sampler in [
        Sampler::new(0.0, 600.0, 0.0),
        Sampler::new(0.0, 600.0, -60.0),
        Sampler::new(0.0, 600.0, f64::NAN),
        Sampler::new(f64::NAN, 600.0, 60.0),
        Sampler::new(0.0, f64::INFINITY, 60.0),
        Sampler::new(600.0, 0.0, 60.0),
    ] {
        assert!(!sampler.is_valid(), "{sampler:?}");
        assert_eq!(sampler.sample_count(), None);
        assert!(sampler.times().is_none());
        assert!(sampler.sample(exact_state).is_err());
    }
}

#[test]
fn a_bad_state_from_the_function_is_reported() {
    let sampler = Sampler::new(0.0, 120.0, 60.0);
    let error = sampler
        .sample(|seconds| {
            if seconds > 0.0 {
                Sample::new(seconds, Vec3::new(f64::NAN, 0.0, 0.0), Vec3::ZERO)
            } else {
                Sample::new(seconds, Vec3::ZERO, Vec3::ZERO)
            }
        })
        .expect_err("the state is not finite");
    assert_eq!(error.kind(), EphemerisErrorKind::InvalidState);
}

#[test]
fn the_sampler_overrides_a_bad_time_from_the_function() {
    // A function that returns a wrong time must not corrupt the table: the
    // grid time is used instead.
    // 返回错误时刻的函数不得污染该表：改用网格时刻。
    let sampler = Sampler::new(0.0, 120.0, 60.0);
    let ephemeris = sampler
        .sample(|_| Sample::new(999.0, Vec3::new(7_000.0, 0.0, 0.0), Vec3::ZERO))
        .expect("the states are finite");
    assert_eq!(ephemeris.times(), vec![0.0, 60.0, 120.0]);
}

#[test]
fn positions_can_be_sampled_without_velocities() {
    // The velocity is reconstructed by finite differences, so a
    // position-only source still produces a usable table.
    // 速度由有限差分重构，故仅提供位置的来源仍能产生可用的表。
    let ephemeris = Sampler::new(0.0, 600.0, 60.0)
        .sample_positions(|seconds| exact_state(seconds).position)
        .expect("the positions are finite");
    assert_eq!(ephemeris.len(), 11);

    // The reconstructed velocity is close to the true one.
    // 所重构的速度接近真值。
    let sample = ephemeris.state_at(300.0).expect("covered");
    let truth = exact_state(300.0);
    assert!(
        sample.velocity.distance(truth.velocity) < 1e-2,
        "the reconstructed velocity {} is too far from {}",
        sample.velocity,
        truth.velocity
    );

    // The interpolation is still usable, though far less accurate than with
    // exact velocities: the finite difference velocity is only second order
    // accurate, so its error feeds straight into the Hermite reconstruction.
    // The error is a few kilometers over a one minute step, against
    // millimeters when the velocity is known.
    // 插值仍可用，尽管远不如有精确速度时准确：有限差分速度只有二阶精度，故其误差
    // 直接输入 Hermite 重构。误差在一分钟步长上为几千米，而有已知速度时为毫米级。
    let mut worst = 0.0_f64;
    for index in 0..600 {
        let time = f64::from(index) + 0.5;
        let interpolated = ephemeris.state_at(time).expect("covered");
        worst = worst.max(interpolated.position.distance(exact_state(time).position));
    }
    assert!(worst < 5.0, "the worst position error is {worst} km");
}

#[test]
fn position_sampling_reports_a_bad_position() {
    let sampler = Sampler::new(0.0, 120.0, 60.0);
    let error = sampler
        .sample_positions(|seconds| {
            if seconds > 0.0 {
                Vec3::new(f64::NAN, 0.0, 0.0)
            } else {
                Vec3::ZERO
            }
        })
        .expect_err("the position is not finite");
    assert_eq!(error.kind(), EphemerisErrorKind::InvalidState);
}

// The Hermite basis functions directly.
// 直接检验 Hermite 基函数。

#[test]
fn the_hermite_basis_reproduces_its_endpoints() {
    let p0 = Vec3::new(1.0, 2.0, 3.0);
    let p1 = Vec3::new(4.0, 5.0, 6.0);
    let v0 = Vec3::new(0.1, 0.2, 0.3);
    let v1 = Vec3::new(0.4, 0.5, 0.6);
    let dt = 60.0;

    // At s = 0 the result is p0 and the velocity is v0.
    // 在 s = 0 处结果为 p0，速度为 v0。
    assert_eq!(hermite_position(p0, v0, p1, v1, dt, 0.0), p0);
    assert_eq!(hermite_velocity(p0, v0, p1, v1, dt, 0.0), v0);
    // At s = 1 the result is p1 and the velocity is v1.
    // 在 s = 1 处结果为 p1，速度为 v1。
    assert_eq!(hermite_position(p0, v0, p1, v1, dt, 1.0), p1);
    assert_eq!(hermite_velocity(p0, v0, p1, v1, dt, 1.0), v1);
}

#[test]
fn the_hermite_state_returns_both_parts() {
    let p0 = Vec3::new(1.0, 0.0, 0.0);
    let p1 = Vec3::new(2.0, 0.0, 0.0);
    let (position, velocity) = hermite_state(p0, Vec3::ZERO, p1, Vec3::ZERO, 10.0, 0.5);
    // Both endpoints are stationary, so the cubic is the S-curve that
    // passes through the midpoint with the slope the endpoints imply.
    // 两个端点都静止，故该三次式是经由中点、斜率为端点所蕴含值的 S 形曲线。
    assert_eq!(position, Vec3::new(1.5, 0.0, 0.0));
    // The slope at the midpoint is 1.5 / dt, not zero: the curve rises from
    // zero slope at one end to zero slope at the other.
    // 中点处的斜率是 1.5 / dt 而非零：曲线从一端的零斜率升到另一端的零斜率。
    assert_eq!(velocity, Vec3::new(0.15, 0.0, 0.0));
}

#[test]
fn a_sample_reports_its_radius_and_speed() {
    let sample = Sample::new(0.0, Vec3::new(3.0, 4.0, 0.0), Vec3::new(0.0, 0.0, 12.0));
    assert!((sample.radius() - 5.0).abs() < 1e-12);
    assert!((sample.speed() - 12.0).abs() < 1e-12);
}

#[test]
fn an_error_displays_its_kind() {
    let error = Ephemeris::from_samples(Vec::new()).expect_err("empty");
    assert!(error.to_string().contains("two samples"), "{error}");
}
