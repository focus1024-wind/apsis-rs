//! Validation tests for the conjunction assessment.
//!
//! 交会分析的验证测试。

use apsis::apsis_cat::{StateSample, closest_approach, isotropic_collision_probability, relative_state};
use apsis::apsis_kepler::{GravityParameter, KeplerianElements};
use apsis::apsis_math::Vec3;

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

#[test]
fn a_linear_relative_motion_reaches_its_closest_at_the_expected_time() {
    let primary_state = circular(7_000.0)
        .to_state(GravityParameter::WGS84)
        .expect("state");
    let primary = StateSample::new(0.0, primary_state.position, primary_state.velocity);
    let mut first = Vec::new();
    let mut second = Vec::new();
    for index in 0..=10 {
        let seconds = 10.0 * index as f64;
        let relative_position = Vec3::new(10.0 - 0.2 * seconds, 5.0, 0.0);
        let relative_velocity = Vec3::new(-0.2, 0.0, 0.0);
        first.push(StateSample::new(
            seconds,
            primary.position_km,
            primary.velocity_km_s,
        ));
        second.push(StateSample::new(
            seconds,
            primary.position_km + relative_position,
            primary.velocity_km_s + relative_velocity,
        ));
    }
    let result = closest_approach(&first, &second).expect("valid grid");
    assert!(
        (result.tca_seconds - 50.0).abs() < 1e-9,
        "{}",
        result.tca_seconds
    );
    assert!(
        (result.distance_km - 5.0).abs() < 1e-9,
        "{}",
        result.distance_km
    );
    assert!((result.relative_speed_km_s - 0.2).abs() < 1e-12);
}

#[test]
fn crossing_circular_orbits_meet_at_the_node() {
    let base = circular(7_000.0);
    let inclined = KeplerianElements {
        inclination: core::f64::consts::FRAC_PI_2,
        ..base
    };
    let mut first = Vec::new();
    let mut second = Vec::new();
    let period = base.period(GravityParameter::WGS84);
    let steps = 120;
    for index in 0..=steps {
        let seconds = period * index as f64 / steps as f64;
        let mean = core::f64::consts::TAU * index as f64 / steps as f64;
        let a = base
            .with_mean_anomaly(mean)
            .to_state(GravityParameter::WGS84)
            .expect("state");
        let b = inclined
            .with_mean_anomaly(mean)
            .to_state(GravityParameter::WGS84)
            .expect("state");
        first.push(StateSample::new(seconds, a.position, a.velocity));
        second.push(StateSample::new(seconds, b.position, b.velocity));
    }
    let result = closest_approach(&first, &second).expect("valid grid");
    // At the node both positions coincide.
    assert!(result.tca_seconds < 1e-6 || (period - result.tca_seconds).abs() < 1.0);
    assert!(result.distance_km < 1e-6, "{}", result.distance_km);
    // The two circular velocities are perpendicular at the node.
    let speed = (398_600.5 / 7_000.0_f64).sqrt();
    assert!((result.relative_speed_km_s - speed * 2.0_f64.sqrt()).abs() < 1e-6);
}

#[test]
fn separated_phases_on_one_circular_orbit_keep_a_constant_chord() {
    let base = circular(7_000.0);
    let separation = 10.0_f64.to_radians();
    let mut first = Vec::new();
    let mut second = Vec::new();
    let period = base.period(GravityParameter::WGS84);
    let steps = 90;
    for index in 0..=steps {
        let seconds = period * index as f64 / steps as f64;
        let mean = core::f64::consts::TAU * index as f64 / steps as f64;
        let a = base
            .with_true_anomaly(mean)
            .to_state(GravityParameter::WGS84)
            .expect("state");
        let b = base
            .with_true_anomaly(mean + separation)
            .to_state(GravityParameter::WGS84)
            .expect("state");
        first.push(StateSample::new(seconds, a.position, a.velocity));
        second.push(StateSample::new(seconds, b.position, b.velocity));
    }
    let result = closest_approach(&first, &second).expect("valid grid");
    let expected = 2.0 * 7_000.0 * (separation / 2.0).sin();
    assert!(
        (result.distance_km - expected).abs() < 0.01,
        "{} against {expected}",
        result.distance_km
    );
}

#[test]
fn the_relative_state_uses_the_primary_ric_frame() {
    let primary = StateSample::new(0.0, Vec3::new(7_000.0, 0.0, 0.0), Vec3::new(0.0, 7.5, 0.0));
    let secondary = StateSample::new(0.0, Vec3::new(7_010.0, 5.0, -2.0), Vec3::new(0.0, 7.6, 0.0));
    let relative = relative_state(primary, secondary).expect("valid");
    assert!((relative.radial_km - 10.0).abs() < 1e-12);
    assert!((relative.along_track_km - 5.0).abs() < 1e-12);
    assert!((relative.cross_track_km + 2.0).abs() < 1e-12);
    assert!((relative.relative_speed_km_s - 0.1).abs() < 1e-12);
}

#[test]
fn mismatched_or_non_uniform_grids_are_rejected() {
    let sample = StateSample::new(0.0, Vec3::new(7_000.0, 0.0, 0.0), Vec3::new(0.0, 7.5, 0.0));
    assert!(closest_approach(&[sample], &[sample]).is_err());
    assert!(closest_approach(&[sample, sample], &[sample, sample]).is_err());
    let later = StateSample::new(10.0, sample.position_km, sample.velocity_km_s);
    let end = StateSample::new(30.0, sample.position_km, sample.velocity_km_s);
    let grid = [sample, later, end];
    assert!(closest_approach(&grid, &grid).is_err());
}


#[test]
fn isotropic_collision_probability_is_bounded_and_decreases_with_miss() {
    let head_on = isotropic_collision_probability(0.0, 0.01, 1.0).expect("probability");
    let offset = isotropic_collision_probability(1.0, 0.01, 1.0).expect("probability");
    assert!((0.0..=1.0).contains(&head_on));
    assert!((0.0..=1.0).contains(&offset));
    assert!(head_on > offset);
    assert_eq!(isotropic_collision_probability(0.0, 0.0, 1.0).expect("probability"), 0.0);
    assert!(isotropic_collision_probability(0.0, 0.01, 0.0).is_err());
    assert!(isotropic_collision_probability(-1.0, 0.01, 1.0).is_err());
}

#[test]
fn anisotropic_probability_matches_the_isotropic_case() {
    let miss = Vec3::new(0.0, 0.0, 0.0);
    let sigma = Vec3::new(1.0, 1.0, 1.0);
    let velocity = Vec3::new(0.0, 7.5, 0.0);
    let anisotropic = apsis::apsis_cat::anisotropic_collision_probability(miss, sigma, velocity, 0.01).expect("anisotropic");
    let isotropic = isotropic_collision_probability(0.0, 0.01, 1.0).expect("isotropic");
    assert!((anisotropic - isotropic).abs() / isotropic < 0.01);
}

#[test]
fn anisotropic_probability_responds_to_the_large_axis() {
    let miss = Vec3::new(0.0, 1.0, 0.0);
    let velocity = Vec3::new(0.0, 7.5, 0.0);
    let tight = apsis::apsis_cat::anisotropic_collision_probability(miss, Vec3::new(0.5, 0.5, 0.5), velocity, 0.01).expect("tight");
    let wide = apsis::apsis_cat::anisotropic_collision_probability(miss, Vec3::new(5.0, 5.0, 5.0), velocity, 0.01).expect("wide");
    assert!((wide - tight).abs() > 1e-9);
    assert!(apsis::apsis_cat::anisotropic_collision_probability(miss, Vec3::ZERO, velocity, 0.01).is_err());
    assert!(apsis::apsis_cat::anisotropic_collision_probability(miss, Vec3::new(1.0, 1.0, 1.0), Vec3::ZERO, 0.01).is_err());
}
