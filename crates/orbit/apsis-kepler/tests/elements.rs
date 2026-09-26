//! Tests for the Keplerian elements and their state vector conversions.
//!
//! The expected values were computed independently from the standard
//! perifocal frame expressions in extended precision. Only the point mass
//! gravity of the central body is involved, so the two should agree to
//! floating point rounding.

use apsis_kepler::{GravityParameter, KeplerianElements, State, wrap_to_pi};
use apsis_math::Vec3;
use core::f64::consts::{PI, TAU};

const POSITION_TOLERANCE: f64 = 1e-6;
const VELOCITY_TOLERANCE: f64 = 1e-9;

fn assert_state_close(state: State, position: [f64; 3], velocity: [f64; 3]) {
    let position_error = (state.position - Vec3::new(position[0], position[1], position[2])).norm();
    let velocity_error = (state.velocity - Vec3::new(velocity[0], velocity[1], velocity[2])).norm();
    assert!(
        position_error < POSITION_TOLERANCE,
        "position error {position_error}, got {}",
        state.position
    );
    assert!(
        velocity_error < VELOCITY_TOLERANCE,
        "velocity error {velocity_error}, got {}",
        state.velocity
    );
}

fn low_orbit() -> KeplerianElements {
    KeplerianElements {
        semi_major_axis: 7_000.0,
        eccentricity: 0.01,
        inclination: 0.9,
        right_ascension: 1.2,
        argument_of_perigee: 0.4,
        true_anomaly: 0.7,
    }
}

fn elliptical_orbit() -> KeplerianElements {
    KeplerianElements {
        semi_major_axis: 26_600.0,
        eccentricity: 0.74,
        inclination: 0.1,
        right_ascension: 2.0,
        argument_of_perigee: 5.0,
        true_anomaly: 1.1,
    }
}

#[test]
fn conversion_matches_an_independent_computation() {
    let state = low_orbit()
        .to_state(GravityParameter::WGS84)
        .expect("elements are valid");
    assert_state_close(
        state,
        [-2444.845_044_819, 4331.004_606_023, 4849.169_779_458],
        [-4.471_116_396_765, -5.509_080_648_689, 2.735_799_894_571],
    );
}

#[test]
fn conversion_matches_for_a_highly_elliptical_orbit() {
    let state = elliptical_orbit()
        .to_state(GravityParameter::WGS84)
        .expect("elements are valid");
    assert_state_close(
        state,
        [-2201.703_284_413, 8734.956_144_841, -163.848_650_213],
        [-8.348_843_969_446, 1.823_413_050_670, 0.685_564_215_543],
    );
}

#[test]
fn a_circular_equatorial_orbit_has_the_circular_speed() {
    // At zero eccentricity, zero inclination, and zero orientation, the
    // satellite sits on the first axis and moves along the second, with
    // speed sqrt(mu / a).
    let elements = KeplerianElements {
        semi_major_axis: 6_378.137,
        eccentricity: 0.0,
        inclination: 0.0,
        right_ascension: 0.0,
        argument_of_perigee: 0.0,
        true_anomaly: 0.0,
    };
    let state = elements
        .to_state(GravityParameter::WGS84)
        .expect("elements are valid");
    assert!((state.position.x - 6_378.137).abs() < 1e-9);
    assert!(state.position.y.abs() < 1e-9);
    assert!(state.position.z.abs() < 1e-9);

    let expected_speed = (GravityParameter::WGS84.value() / 6_378.137).sqrt();
    assert!((state.velocity.norm() - expected_speed).abs() < 1e-9);
    assert!(state.velocity.y > 0.0);
}

#[test]
fn the_state_vector_lies_in_the_orbit_plane() {
    let elements = low_orbit();
    let state = elements
        .to_state(GravityParameter::WGS84)
        .expect("elements are valid");
    // The position and velocity span the orbit plane, so the normal to that
    // plane is perpendicular to both.
    let normal = state
        .position
        .cross(state.velocity)
        .normalize()
        .expect("non-zero");
    assert!(normal.dot(state.position).abs() < 1e-6);
    assert!(normal.dot(state.velocity).abs() < 1e-9);
}

#[test]
fn radius_follows_the_conic_equation() {
    let elements = low_orbit();
    for true_anomaly in [-2.0, -1.0, 0.0, 1.0, 2.0, 3.0] {
        let state = elements
            .with_true_anomaly(true_anomaly)
            .to_state(GravityParameter::WGS84)
            .expect("elements are valid");
        let p = elements.semi_latus_rectum();
        let expected = p / (1.0 + elements.eccentricity * true_anomaly.cos());
        assert!((state.position.norm() - expected).abs() < 1e-6);
    }
}

#[test]
fn perigee_and_apogee_are_the_extremes_of_the_radius() {
    let elements = elliptical_orbit();
    let perigee = elements
        .with_true_anomaly(0.0)
        .to_state(GravityParameter::WGS84)
        .expect("valid");
    let apogee = elements
        .with_true_anomaly(PI)
        .to_state(GravityParameter::WGS84)
        .expect("valid");
    assert!((perigee.position.norm() - elements.perigee_radius()).abs() < 1e-6);
    assert!((apogee.position.norm() - elements.apogee_radius()).abs() < 1e-6);
    assert!(perigee.position.norm() < apogee.position.norm());
}

#[test]
fn the_speed_is_highest_at_perigee() {
    let elements = elliptical_orbit();
    let perigee = elements
        .with_true_anomaly(0.0)
        .to_state(GravityParameter::WGS84)
        .expect("valid");
    let apogee = elements
        .with_true_anomaly(PI)
        .to_state(GravityParameter::WGS84)
        .expect("valid");
    assert!(perigee.velocity.norm() > apogee.velocity.norm());
}

#[test]
fn orbital_geometry_is_consistent() {
    let elements = elliptical_orbit();
    let p = elements.semi_latus_rectum();
    assert!((p - elements.semi_major_axis * (1.0 - elements.eccentricity.powi(2))).abs() < 1e-9);
    assert!((elements.semi_minor_axis().powi(2) - elements.semi_major_axis * p).abs() < 1e-6);
    assert!(
        (elements.apogee_radius()
            - elements.perigee_radius()
            - 2.0 * elements.semi_major_axis * elements.eccentricity)
            .abs()
            < 1e-9
    );
}

#[test]
fn the_period_follows_keplers_third_law() {
    let elements = low_orbit();
    let period = elements.period(GravityParameter::WGS84);
    let expected =
        2.0 * PI * (elements.semi_major_axis.powi(3) / GravityParameter::WGS84.value()).sqrt();
    assert!((period - expected).abs() < 1e-9);
    // A 7000 km orbit has a period of about 97 minutes.
    assert!((period / 60.0 - 97.0).abs() < 2.0);
}

#[test]
fn state_round_trips_through_the_elements() {
    for elements in [low_orbit(), elliptical_orbit()] {
        let state = elements
            .to_state(GravityParameter::WGS84)
            .expect("elements are valid");
        let recovered = KeplerianElements::from_state(&state, GravityParameter::WGS84)
            .expect("state describes an ellipse");

        assert!(
            (recovered.semi_major_axis - elements.semi_major_axis).abs() < 1e-6,
            "semi-major axis {} vs {}",
            recovered.semi_major_axis,
            elements.semi_major_axis
        );
        assert!(
            (recovered.eccentricity - elements.eccentricity).abs() < 1e-9,
            "eccentricity {} vs {}",
            recovered.eccentricity,
            elements.eccentricity
        );
        assert!((recovered.inclination - elements.inclination).abs() < 1e-9);
        // The angles are compared modulo a full turn, because a value such
        // as 5.0 radians and one reduced to -1.28 radians describe the same
        // orientation.
        assert!((wrap_to_pi(recovered.right_ascension - elements.right_ascension)).abs() < 1e-9);
        assert!(
            (wrap_to_pi(recovered.argument_of_perigee - elements.argument_of_perigee)).abs() < 1e-9
        );
        assert!((wrap_to_pi(recovered.true_anomaly - elements.true_anomaly)).abs() < 1e-7);

        // The round trip must reproduce the state as well as the elements,
        // which is the stronger check.
        let again = recovered.to_state(GravityParameter::WGS84).expect("valid");
        assert!((again.position - state.position).norm() < 1e-6);
        assert!((again.velocity - state.velocity).norm() < 1e-9);
    }
}

#[test]
fn a_circular_orbit_round_trips() {
    let elements = KeplerianElements {
        semi_major_axis: 7_000.0,
        eccentricity: 0.0,
        inclination: 1.0,
        right_ascension: 0.5,
        argument_of_perigee: 0.0,
        true_anomaly: 0.0,
    };
    let state = elements
        .to_state(GravityParameter::WGS72)
        .expect("elements are valid");
    let recovered = KeplerianElements::from_state(&state, GravityParameter::WGS72)
        .expect("state describes an ellipse");
    assert!((recovered.semi_major_axis - 7_000.0).abs() < 1e-6);
    assert!(recovered.eccentricity < 1e-9);
    assert!((recovered.inclination - 1.0).abs() < 1e-9);
    assert!((recovered.right_ascension - 0.5).abs() < 1e-9);
}

#[test]
fn mean_anomaly_round_trips_through_the_elements() {
    let elements = elliptical_orbit();
    let mean = elements.to_mean_anomaly();
    let restored = elements.with_mean_anomaly(mean);
    assert!((restored.true_anomaly - elements.true_anomaly).abs() < 1e-8);
}

#[test]
fn invalid_elements_are_rejected() {
    let mut elements = low_orbit();
    elements.eccentricity = 1.0;
    assert!(!elements.is_valid());
    assert!(elements.to_state(GravityParameter::WGS84).is_none());

    let mut elements = low_orbit();
    elements.semi_major_axis = -1.0;
    assert!(!elements.is_valid());
    assert!(elements.to_state(GravityParameter::WGS84).is_none());

    let mut elements = low_orbit();
    elements.inclination = PI + 0.1;
    assert!(!elements.is_valid());

    let mut elements = low_orbit();
    elements.true_anomaly = f64::NAN;
    assert!(!elements.is_valid());
}

#[test]
fn degenerate_states_are_rejected() {
    assert_eq!(
        KeplerianElements::from_state(
            &State {
                position: Vec3::ZERO,
                velocity: Vec3::ZERO
            },
            GravityParameter::WGS84
        ),
        None
    );
    // A purely radial state has no defined orbit plane.
    assert_eq!(
        KeplerianElements::from_state(
            &State {
                position: Vec3::new(7_000.0, 0.0, 0.0),
                velocity: Vec3::new(1.0, 0.0, 0.0)
            },
            GravityParameter::WGS84
        ),
        None
    );
    // An unbound state has a non-positive semi-major axis.
    assert_eq!(
        KeplerianElements::from_state(
            &State {
                position: Vec3::new(7_000.0, 0.0, 0.0),
                velocity: Vec3::new(0.0, 20.0, 0.0)
            },
            GravityParameter::WGS84
        ),
        None
    );
}

#[test]
fn gravity_parameter_values_match_the_standards() {
    assert_eq!(GravityParameter::WGS72.value(), 398_600.8);
    assert_eq!(GravityParameter::WGS84.value(), 398_600.5);
    assert!((GravityParameter::WGS84.sqrt() - 398_600.5_f64.sqrt()).abs() < 1e-9);
}

#[test]
fn a_full_revolution_returns_to_the_same_point() {
    // Advancing the mean anomaly by one full turn must return the same
    // position, which verifies the anomaly conversions agree.
    let elements = elliptical_orbit();
    let start = elements
        .with_mean_anomaly(1.0)
        .to_state(GravityParameter::WGS84)
        .expect("valid");
    let end = elements
        .with_mean_anomaly(1.0 + TAU)
        .to_state(GravityParameter::WGS84)
        .expect("valid");
    assert!((start.position - end.position).norm() < 1e-6);
    assert!((start.velocity - end.velocity).norm() < 1e-9);
}
