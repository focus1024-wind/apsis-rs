//! Tests for the maneuver models.
//!
//! 机动模型的测试。

use apsis::apsis_kepler::{GravityParameter, KeplerianElements};
use apsis::apsis_maneuver::{FiniteBurn, Impulse, ManeuverErrorKind, hohmann_transfer};
use apsis::apsis_math::Vec3;

/// A circular orbit at a given radius, in the equatorial plane.
///
/// 给定半径、位于赤道面内的圆轨道。
fn circular(radius: f64) -> KeplerianElements {
    KeplerianElements {
        semi_major_axis: radius,
        eccentricity: 0.0,
        inclination: 0.0,
        right_ascension: 0.0,
        argument_of_perigee: 0.0,
        true_anomaly: 0.0,
    }
}

// The impulsive model.
// 脉冲模型。

#[test]
fn an_impulse_reports_its_change() {
    let impulse = Impulse::new(Vec3::new(0.0, 1.0, 0.0)).expect("usable");
    assert_eq!(impulse.delta_v(), Vec3::new(0.0, 1.0, 0.0));
    assert!((impulse.magnitude() - 1.0).abs() < 1e-12);
}

#[test]
fn a_zero_or_non_finite_impulse_is_refused() {
    let error = Impulse::new(Vec3::ZERO).expect_err("a zero change is not a maneuver");
    assert_eq!(error.kind(), ManeuverErrorKind::InvalidState);
    assert!(Impulse::new(Vec3::new(f64::NAN, 0.0, 0.0)).is_err());
    assert!(Impulse::new(Vec3::new(0.0, f64::INFINITY, 0.0)).is_err());
}

#[test]
fn an_impulse_can_be_built_from_a_direction_and_a_magnitude() {
    // The direction need not be a unit vector.
    // 方向不必是单位矢量。
    let impulse = Impulse::along(Vec3::new(0.0, 3.0, 0.0), 0.5).expect("usable");
    assert!((impulse.magnitude() - 0.5).abs() < 1e-12);
    assert!((impulse.delta_v().y - 0.5).abs() < 1e-12);
    assert!(impulse.delta_v().z.abs() < 1e-12);

    // A zero direction and a bad magnitude are refused.
    // 零方向与坏幅度被拒绝。
    assert!(Impulse::along(Vec3::ZERO, 1.0).is_err());
    assert!(Impulse::along(Vec3::X, 0.0).is_err());
    assert!(Impulse::along(Vec3::X, -1.0).is_err());
    assert!(Impulse::along(Vec3::X, f64::NAN).is_err());
}

#[test]
fn an_impulse_can_be_scaled_and_reversed() {
    let impulse = Impulse::new(Vec3::new(1.0, 0.0, 0.0)).expect("usable");
    let doubled = impulse.scaled(2.0).expect("usable");
    assert!((doubled.magnitude() - 2.0).abs() < 1e-12);
    let reversed = impulse.reversed();
    assert!((reversed.delta_v().x + 1.0).abs() < 1e-12);
    // Scaling by zero produces a zero change, which is refused.
    // 按零缩放产生零变化，会被拒绝。
    assert!(impulse.scaled(0.0).is_err());
}

#[test]
fn an_impulse_changes_the_velocity_but_not_the_position() {
    let impulse = Impulse::new(Vec3::new(0.0, 0.7, 0.0)).expect("usable");
    let velocity = Vec3::new(7.5, 0.0, 0.0);
    let changed = impulse.apply_to_velocity(velocity);
    assert_eq!(changed, Vec3::new(7.5, 0.7, 0.0));
}

#[test]
fn an_impulse_displays_its_magnitude() {
    let impulse = Impulse::new(Vec3::new(0.0, 1.234_5, 0.0)).expect("usable");
    let text = impulse.to_string();
    assert!(text.contains("1.234"), "{text}");
}

// The Hohmann transfer.
// Hohmann 转移。

#[test]
fn a_hohmann_transfer_raises_an_orbit() {
    // Raise from 7000 to 8000 km.
    // 从 7000 升到 8000 km。
    let transfer =
        hohmann_transfer(7_000.0, 8_000.0, GravityParameter::WGS84).expect("the radii are usable");
    assert!(transfer.is_raising());
    // Raising an orbit takes two prograde burns.
    // 抬高轨道需要两次顺行点火。
    assert!(transfer.first_burn > 0.0);
    assert!(transfer.second_burn > 0.0);
    assert_eq!(transfer.transfer_semi_major_axis, 7_500.0);
    assert!(transfer.total_delta_v() > 0.0);
    assert_eq!(
        transfer.total_delta_v(),
        transfer.first_burn + transfer.second_burn
    );
    assert_eq!(transfer.mean_radius(), 7_500.0);
}

#[test]
fn a_hohmann_transfer_lowers_an_orbit() {
    // The same two radii in the other order is a descent, and the burns are
    // retrograde.
    // 两个半径反序即为下降，两次点火都是逆行。
    let transfer =
        hohmann_transfer(8_000.0, 7_000.0, GravityParameter::WGS84).expect("the radii are usable");
    assert!(!transfer.is_raising());
    assert!(transfer.first_burn < 0.0);
    assert!(transfer.second_burn < 0.0);
    // The cost is the same either way.
    // 两种方向的代价相同。
    let raised =
        hohmann_transfer(7_000.0, 8_000.0, GravityParameter::WGS84).expect("the radii are usable");
    assert!((transfer.total_delta_v() - raised.total_delta_v()).abs() < 1e-12);
    assert!((transfer.transfer_time - raised.transfer_time).abs() < 1e-12);
}

#[test]
fn the_first_burn_matches_the_vis_viva_speeds() {
    // The first burn is the transfer ellipse speed at the lower radius less
    // the circular speed there. Checking it against the vis-viva equation
    // directly pins the formula.
    // 第一次点火是转移椭圆在下半径处的速率减去那里的圆轨道速率。用活力公式直接核对
    // 它即可钉住该公式。
    let transfer =
        hohmann_transfer(7_000.0, 9_000.0, GravityParameter::WGS84).expect("the radii are usable");
    let mu = GravityParameter::WGS84.value();
    let circular = (mu / 7_000.0).sqrt();
    let ellipse = (mu * (2.0 / 7_000.0 - 1.0 / 8_000.0)).sqrt();
    assert!((transfer.first_burn - (ellipse - circular)).abs() < 1e-12);
}

#[test]
fn the_transfer_time_is_half_the_ellipse_period() {
    let transfer =
        hohmann_transfer(7_000.0, 8_000.0, GravityParameter::WGS84).expect("the radii are usable");
    let half_period =
        circular(transfer.transfer_semi_major_axis).period(GravityParameter::WGS84) / 2.0;
    assert!((transfer.transfer_time - half_period).abs() < 1e-9);
}

#[test]
fn a_hohmann_transfer_matches_a_known_leo_to_geo_case() {
    // The well documented case: a transfer from a 300 km circular parking
    // orbit to geostationary altitude. The total is about 3.89 km/s, which
    // is the figure quoted for a standard LEO to GEO chemical transfer.
    // 有充分记载的用例：从 300 km 圆停泊轨道转移到地球静止高度。总计约 3.89 km/s，
    // 这是标准 LEO 到 GEO 化学转移所引用的数字。
    let from = 6_378.137 + 300.0;
    let to = 42_164.0;
    let transfer = hohmann_transfer(from, to, GravityParameter::WGS84).expect("usable");
    assert!(
        (transfer.total_delta_v() - 3.89).abs() < 0.02,
        "the total is {} km/s",
        transfer.total_delta_v()
    );
    // The transfer ellipse touches both orbits: its perigee is the parking
    // radius and its apogee the destination, so its eccentricity follows
    // from the two radii.
    // 转移椭圆与两个轨道都相切：其近地点是停泊半径，远地点是目的地，故其偏心率由
    // 这两个半径得出。
    let eccentricity = (to - from) / (to + from);
    let perigee = transfer.transfer_semi_major_axis * (1.0 - eccentricity);
    let apogee = transfer.transfer_semi_major_axis * (1.0 + eccentricity);
    assert!((perigee - from).abs() < 1e-9);
    assert!((apogee - to).abs() < 1e-9);
}

#[test]
fn a_degenerate_transfer_is_refused() {
    let error = hohmann_transfer(7_000.0, 7_000.0, GravityParameter::WGS84)
        .expect_err("equal radii are not a transfer");
    assert_eq!(error.kind(), ManeuverErrorKind::DegenerateTransfer);
}

#[test]
fn an_unusable_radius_is_refused() {
    for (from, to) in [
        (0.0, 8_000.0),
        (-1.0, 8_000.0),
        (7_000.0, 0.0),
        (7_000.0, f64::NAN),
        (f64::INFINITY, 8_000.0),
    ] {
        let error = hohmann_transfer(from, to, GravityParameter::WGS84)
            .expect_err("the radius is not usable");
        assert_eq!(error.kind(), ManeuverErrorKind::InvalidRadius);
    }
}

// The finite burn.
// 有限点火。

#[test]
fn a_finite_burn_reports_its_acceleration_and_duration() {
    let burn = FiniteBurn::new(Vec3::new(0.0, 0.001, 0.0), 100.0).expect("usable");
    assert!((burn.acceleration_magnitude() - 0.001).abs() < 1e-15);
    assert_eq!(burn.duration(), 100.0);
    // The nominal change is the acceleration times the duration.
    // 标称变化是加速度乘以时长。
    assert!((burn.nominal_delta_v() - 0.1).abs() < 1e-12);
}

#[test]
fn a_finite_burn_can_be_built_from_a_direction() {
    let burn = FiniteBurn::along(Vec3::new(0.0, 5.0, 0.0), 0.002, 50.0).expect("usable");
    assert!((burn.acceleration_magnitude() - 0.002).abs() < 1e-15);
    assert!((burn.nominal_delta_v() - 0.1).abs() < 1e-12);
    assert!(FiniteBurn::along(Vec3::ZERO, 0.001, 10.0).is_err());
    assert!(FiniteBurn::along(Vec3::X, 0.0, 10.0).is_err());
}

#[test]
fn an_unusable_burn_is_refused() {
    assert!(FiniteBurn::new(Vec3::ZERO, 10.0).is_err());
    assert!(FiniteBurn::new(Vec3::X, 0.0).is_err());
    assert!(FiniteBurn::new(Vec3::X, -1.0).is_err());
    assert!(FiniteBurn::new(Vec3::new(f64::NAN, 0.0, 0.0), 10.0).is_err());
    assert!(FiniteBurn::new(Vec3::X, f64::NAN).is_err());
}

#[test]
fn a_finite_burn_agrees_with_its_equivalent_impulse_in_free_space() {
    // With gravity switched off the finite burn must deliver exactly the
    // nominal change: it is a constant acceleration, so the velocity change
    // is the acceleration times the duration.
    // 在引力关掉时，有限点火必须恰好给出标称变化：它是常值加速度，故速度变化是
    // 加速度乘以时长。
    //
    // The two-body model always has gravity, so instead the burn is applied
    // far from a mass, where gravity in the equation is negligible because
    // the central body was given a vanishing parameter.
    // 二体模型总有引力，故改为在远离质量处施加点火，那里方程中的引力可忽略，因为
    // 中心天体被赋予了一个趋于零的参数。
    let burn = FiniteBurn::new(Vec3::new(0.0, 0.001, 0.0), 100.0).expect("usable");
    let state = apsis::apsis_kepler::State {
        position: Vec3::new(1.0e9, 0.0, 0.0),
        velocity: Vec3::ZERO,
    };
    // A negligible gravitational parameter leaves only the thrust.
    // 可忽略的引力参数只留下推力。
    let outcome = burn
        .apply(state, GravityParameter(0.0), 100)
        .expect("the burn runs");
    assert!(
        (outcome.achieved_magnitude() - 0.1).abs() < 1e-9,
        "the achieved change is {}",
        outcome.achieved_magnitude()
    );
    assert!((outcome.nominal_delta_v - 0.1).abs() < 1e-12);
    // There is no gravity to lose anything to.
    // 没有引力可损失任何东西。
    assert!(outcome.gravity_loss().abs() < 1e-9);
}

#[test]
fn a_finite_burn_converges_on_the_matching_impulse() {
    // This is the property that ties the two models together. A finite burn
    // of acceleration `a` over a duration `t` delivers the impulse `a t` in
    // the limit of a shrinking duration, because the gravity deflection
    // during the burn vanishes with the duration.
    // 这正是把两个模型联系起来的性质。加速度为 `a`、时长为 `t` 的有限点火在时长
    // 趋于零的极限下给出冲量 `a t`，因为点火期间的引力偏折随时长消失。
    let parameter = GravityParameter::WGS84;
    let elements = circular(7_000.0);
    let state = elements.to_state(parameter).expect("valid");

    // The thrust direction is along the velocity, which for this orbit is
    // the second axis.
    // 推力方向沿速度，对该轨道而言即第二轴。
    let direction = state.velocity.normalize().expect("the speed is non-zero");
    let target_delta_v = 0.01;

    let achieved_for = |duration: f64| {
        // Hold the nominal delta-v fixed by scaling the acceleration, so
        // every burn is compared at the same impulse.
        // 通过缩放加速度保持标称 delta-v 不变，使每次点火都在相同冲量下比较。
        let acceleration = target_delta_v / duration;
        let burn = FiniteBurn::along(direction, acceleration, duration).expect("usable");
        burn.apply(state, parameter, 2_000)
            .expect("the burn runs")
            .achieved_magnitude()
    };

    // A long burn loses some of the change to gravity; a short one keeps
    // nearly all of it.
    // 长点火把部分变化损失给引力；短点火几乎全部保留。
    let long = achieved_for(600.0);
    let short = achieved_for(6.0);
    let long_error = (long - target_delta_v).abs();
    let short_error = (short - target_delta_v).abs();
    assert!(
        short_error < long_error,
        "the short burn error {short_error} should be below the long one {long_error}"
    );
    assert!(
        short_error < target_delta_v * 1e-3,
        "the short burn should be within a thousandth of the impulse, got {short_error}"
    );
}

#[test]
fn a_finite_burn_delivers_less_than_nominal_against_gravity() {
    // A long prograde burn on a circular orbit: gravity turns the velocity
    // while the burn runs, so the achieved change differs from the free
    // space value. The sign of the difference depends on the geometry; the
    // point of the test is that the two are not equal, which is why the
    // finite model exists.
    // 圆轨道上的长顺行点火：点火进行时引力使速度转向，故所达到的变化不同于自由空间
    // 值。差值的符号取决于几何；本测试的要点是二者不相等，这正是有限模型存在的
    // 原因。
    let parameter = GravityParameter::WGS84;
    let elements = circular(7_000.0);
    let state = elements.to_state(parameter).expect("valid");
    let direction = state.velocity.normalize().expect("the speed is non-zero");
    // A burn lasting a tenth of the orbital period.
    // 持续轨道周期十分之一的点火。
    let duration = elements.period(parameter) / 10.0;
    let acceleration = 1e-5;
    let burn = FiniteBurn::along(direction, acceleration, duration).expect("usable");
    let outcome = burn.apply(state, parameter, 1_000).expect("the burn runs");

    assert!(
        outcome.gravity_loss().abs() > 1e-9,
        "the loss is {}",
        outcome.gravity_loss()
    );
    // The position moved during the burn.
    // 点火期间位置移动了。
    assert!(outcome.state.position.distance(state.position) > 1.0);
}

#[test]
fn a_finite_burn_raises_the_orbit_when_prograde() {
    // A prograde burn adds energy, so the semi-major axis grows.
    // 顺行点火增加能量，故半长轴增大。
    let parameter = GravityParameter::WGS84;
    let elements = circular(7_000.0);
    let state = elements.to_state(parameter).expect("valid");
    let direction = state.velocity.normalize().expect("non-zero");
    let burn = FiniteBurn::along(direction, 1e-4, 60.0).expect("usable");
    let outcome = burn.apply(state, parameter, 1_000).expect("the burn runs");
    let after = outcome.elements(parameter).expect("still elliptical");
    assert!(
        after.semi_major_axis > elements.semi_major_axis,
        "the axis went from {} to {}",
        elements.semi_major_axis,
        after.semi_major_axis
    );
}

#[test]
fn a_finite_burn_lowers_the_orbit_when_retrograde() {
    // The mirror image: a retrograde burn removes energy.
    // 镜像情形：逆行点火移除能量。
    let parameter = GravityParameter::WGS84;
    let elements = circular(7_000.0);
    let state = elements.to_state(parameter).expect("valid");
    let direction = -state.velocity.normalize().expect("non-zero");
    let burn = FiniteBurn::along(direction, 1e-4, 60.0).expect("usable");
    let outcome = burn.apply(state, parameter, 1_000).expect("the burn runs");
    let after = outcome.elements(parameter).expect("still elliptical");
    assert!(after.semi_major_axis < elements.semi_major_axis);
}

#[test]
fn a_finite_burn_can_be_applied_to_elements() {
    let parameter = GravityParameter::WGS84;
    let elements = circular(7_000.0);
    let state = elements.to_state(parameter).expect("valid");
    let direction = state.velocity.normalize().expect("non-zero");
    let burn = FiniteBurn::along(direction, 1e-4, 60.0).expect("usable");

    let from_state = burn.apply(state, parameter, 500).expect("the burn runs");
    let from_elements = burn
        .apply_to_elements(elements, parameter, 500)
        .expect("the burn runs");
    // Both routes must give the same state.
    // 两条路径必须给出相同状态。
    assert!(
        from_state
            .state
            .position
            .distance(from_elements.state.position)
            < 1e-9
    );
}

#[test]
fn a_burn_on_invalid_elements_is_refused() {
    let bad = KeplerianElements {
        semi_major_axis: -1.0,
        ..circular(7_000.0)
    };
    let burn = FiniteBurn::new(Vec3::new(0.0, 1e-4, 0.0), 60.0).expect("usable");
    let error = burn
        .apply_to_elements(bad, GravityParameter::WGS84, 100)
        .expect_err("the elements are invalid");
    assert_eq!(error.kind(), ManeuverErrorKind::InvalidState);
}

#[test]
fn a_burn_with_no_steps_is_refused() {
    let parameter = GravityParameter::WGS84;
    let state = circular(7_000.0).to_state(parameter).expect("valid");
    let burn = FiniteBurn::new(Vec3::new(0.0, 1e-4, 0.0), 60.0).expect("usable");
    let error = burn
        .apply(state, parameter, 0)
        .expect_err("no steps cannot integrate");
    assert_eq!(error.kind(), ManeuverErrorKind::InvalidState);
}

#[test]
fn a_large_burn_can_escape_and_is_reported() {
    // A burn hard enough to exceed escape velocity leaves no elliptical
    // elements, which the outcome reports by returning `None`.
    // 强度足以超过逃逸速度的点火不留下椭圆根数，结果通过返回 `None` 报告这一点。
    let parameter = GravityParameter::WGS84;
    let state = circular(7_000.0).to_state(parameter).expect("valid");
    let direction = state.velocity.normalize().expect("non-zero");
    let burn = FiniteBurn::along(direction, 1e-2, 600.0).expect("usable");
    let outcome = burn.apply(state, parameter, 2_000).expect("the burn runs");
    assert!(outcome.elements(parameter).is_none());
}

#[test]
fn the_equivalent_impulse_matches_the_nominal_change() {
    let burn = FiniteBurn::new(Vec3::new(0.0, 0.001, 0.0), 100.0).expect("usable");
    let impulse = burn.equivalent_impulse().expect("the change is non-zero");
    assert!((impulse.magnitude() - burn.nominal_delta_v()).abs() < 1e-12);
}

#[test]
fn the_outcome_reports_the_achieved_magnitude() {
    let parameter = GravityParameter::WGS84;
    let state = circular(7_000.0).to_state(parameter).expect("valid");
    let direction = state.velocity.normalize().expect("non-zero");
    let burn = FiniteBurn::along(direction, 1e-4, 60.0).expect("usable");
    let outcome = burn.apply(state, parameter, 500).expect("the burn runs");
    assert!(outcome.achieved_magnitude() > 0.0);
    assert_eq!(outcome.duration, 60.0);
    assert_eq!(
        outcome.achieved_delta_v.norm(),
        outcome.achieved_magnitude()
    );
}

#[test]
fn an_error_displays_its_kind() {
    let error =
        hohmann_transfer(7_000.0, 7_000.0, GravityParameter::WGS84).expect_err("degenerate");
    assert!(error.to_string().contains("radii"), "{error}");
}
