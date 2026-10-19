//! Validation tests for mission sequences and the Hohmann plan.
//!
//! 任务序列与霍曼方案的验证测试。

use apsis::apsis_kepler::{GravityParameter, KeplerianElements};
use apsis::apsis_math::Vec3;
use apsis::apsis_mission::{EventKind, MissionSequence, plan_hohmann};

const MU: f64 = 398_600.5;
const TOLERANCE: f64 = 1e-9;

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
fn the_hohmann_plan_matches_the_vis_viva_result() {
    let plan = plan_hohmann(6_678.0, 42_164.0).expect("valid radii");
    let semi_major = f64::midpoint(6_678.0, 42_164.0);
    let first = (MU * (2.0 / 6_678.0 - 1.0 / semi_major)).sqrt() - (MU / 6_678.0).sqrt();
    let second = (MU / 42_164.0).sqrt() - (MU * (2.0 / 42_164.0 - 1.0 / semi_major)).sqrt();
    let time = core::f64::consts::PI * (semi_major.powi(3) / MU).sqrt();
    assert!((plan.first_burn_km_s - first).abs() < TOLERANCE);
    assert!((plan.second_burn_km_s - second).abs() < TOLERANCE);
    assert!((plan.total_delta_v_km_s - first.abs() - second.abs()).abs() < TOLERANCE);
    assert!((plan.transfer_time_seconds - time).abs() < 1e-6);
    assert!(plan.raises);
}

#[test]
fn a_half_period_propagation_reverses_the_position() {
    let elements = circular(7_000.0);
    let period = core::f64::consts::TAU * (7_000.0f64.powi(3) / MU).sqrt();
    let initial = elements.to_state(GravityParameter::WGS84).expect("state");
    let sequence = MissionSequence::new(elements, GravityParameter::WGS84)
        .expect("valid sequence")
        .propagate(period / 2.0)
        .expect("valid segment");
    let result = sequence.run().expect("runs");
    assert!((result.final_state.position + initial.position).norm() < 1e-6);
    assert_eq!(result.events.len(), 1);
    assert_eq!(result.events[0].kind, EventKind::Propagate);
}

#[test]
fn a_hohmann_sequence_reaches_the_target_circular_orbit() {
    let from = 6_678.0;
    let to = 42_164.0;
    let plan = plan_hohmann(from, to).expect("valid radii");
    let initial = circular(from);
    let initial_state = initial.to_state(GravityParameter::WGS84).expect("state");
    let first_direction = initial_state.velocity.normalize().expect("direction");
    let first = MissionSequence::new(initial, GravityParameter::WGS84)
        .expect("valid sequence")
        .impulse(first_direction * plan.first_burn_km_s)
        .expect("valid impulse")
        .propagate(plan.transfer_time_seconds)
        .expect("valid propagation")
        .run()
        .expect("runs");
    // At apoapsis the radius is the target radius.
    assert!((first.final_state.position.norm() - to).abs() < 1e-6);
    let second_direction = first.final_state.velocity.normalize().expect("direction");
    let second = MissionSequence::new(first.final_elements, GravityParameter::WGS84)
        .expect("valid sequence")
        .impulse(second_direction * plan.second_burn_km_s)
        .expect("valid impulse")
        .run()
        .expect("runs");
    assert!((second.final_elements.semi_major_axis - to).abs() < 1e-6);
    assert!(second.final_elements.eccentricity < 1e-9);
    assert!((second.final_state.position.norm() - to).abs() < 1e-6);
    assert!(
        (first.total_delta_v_km_s + second.total_delta_v_km_s - plan.total_delta_v_km_s).abs()
            < 1e-12
    );
}

#[test]
fn the_sequence_records_events_in_order() {
    let elements = circular(7_000.0);
    let sequence = MissionSequence::new(elements, GravityParameter::WGS84)
        .expect("valid")
        .propagate(60.0)
        .expect("valid")
        .impulse(Vec3::new(0.0, 0.01, 0.0))
        .expect("valid")
        .propagate(60.0)
        .expect("valid");
    let result = sequence.run().expect("runs");
    assert_eq!(result.events.len(), 3);
    assert_eq!(result.events[0].kind, EventKind::Propagate);
    assert_eq!(result.events[1].kind, EventKind::Impulse);
    assert_eq!(result.events[2].kind, EventKind::Propagate);
    assert!((result.elapsed_seconds - 120.0).abs() < 1e-12);
    assert!((result.total_delta_v_km_s - 0.01).abs() < 1e-12);
}

#[test]
fn invalid_segments_are_rejected() {
    let elements = circular(7_000.0);
    assert!(
        MissionSequence::new(elements, GravityParameter::WGS84)
            .expect("valid")
            .propagate(0.0)
            .is_err()
    );
    assert!(
        MissionSequence::new(elements, GravityParameter::WGS84)
            .expect("valid")
            .impulse(Vec3::ZERO)
            .is_err()
    );
    assert!(plan_hohmann(7_000.0, 7_000.0).is_err());
}
