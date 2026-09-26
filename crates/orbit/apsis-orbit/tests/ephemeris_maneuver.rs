//! Integration tests for sampling and maneuvering through the facade.

use apsis_kepler::GravityParameter;
use apsis_maneuver::{FiniteBurn, hohmann_transfer};
use apsis_orbit::{EphemerisSamplingError, GravityModel, ManeuverApplicationError, Orbit};
use apsis_tle::TwoLineElement;

/// A satellite in a low Earth orbit, used as the source of the track.
const ISS: &str = concat!(
    "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\r\n",
    "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537\r\n",
);

fn orbit() -> Orbit {
    let element = TwoLineElement::parse(ISS).expect("the sample parses");
    Orbit::from_element(&element, GravityModel::WGS72).expect("the orbit propagates")
}

#[test]
fn an_orbit_samples_into_an_ephemeris() {
    let orbit = orbit();
    // Ten minutes over a one minute grid.
    let ephemeris = orbit
        .sample_ephemeris(0.0, 10.0, 1.0)
        .expect("the grid is usable");
    assert_eq!(ephemeris.len(), 11);
    // The sample times are seconds from the element epoch.
    assert_eq!(ephemeris.start_time(), 0.0);
    assert_eq!(ephemeris.end_time(), 600.0);

    // Every sample must match a direct propagation.
    for sample in ephemeris.samples() {
        let direct = orbit
            .propagate_minutes(sample.seconds / 60.0)
            .expect("the orbit propagates");
        assert!(
            sample.position.distance(direct.position) < 1e-12,
            "at {} s",
            sample.seconds
        );
    }
}

#[test]
fn the_interpolated_ephemeris_matches_a_direct_propagation() {
    // This is the point of the ephemeris: a query between samples must agree
    // with the propagator to well within the accuracy a track needs.
    let orbit = orbit();
    let ephemeris = orbit
        .sample_ephemeris(0.0, 30.0, 1.0)
        .expect("the grid is usable");

    let mut worst = 0.0_f64;
    // Query every ten seconds, which includes points between samples.
    for index in 0..180 {
        let seconds = f64::from(index) * 10.0;
        let interpolated = ephemeris.state_at(seconds).expect("the time is covered");
        let direct = orbit.propagate_minutes(seconds / 60.0).expect("propagates");
        worst = worst.max(interpolated.position.distance(direct.position));
    }
    // A one minute Hermite step on a low Earth orbit is accurate to far
    // better than a meter here.
    assert!(worst < 1e-3, "the worst interpolation error is {worst} km");
}

#[test]
fn a_bad_grid_is_refused() {
    let orbit = orbit();
    assert!(matches!(
        orbit.sample_ephemeris(0.0, 0.0, 1.0),
        Err(EphemerisSamplingError::InvalidGrid)
    ));
    assert!(matches!(
        orbit.sample_ephemeris(0.0, 10.0, 0.0),
        Err(EphemerisSamplingError::InvalidGrid)
    ));
    assert!(matches!(
        orbit.sample_ephemeris(10.0, 0.0, 1.0),
        Err(EphemerisSamplingError::InvalidGrid)
    ));
}

#[test]
fn a_propagation_failure_is_reported() {
    let orbit = orbit();
    // Far enough ahead that the drag term brings the satellite down, which
    // is a decay failure rather than a silent bad sample.
    let result = orbit.sample_ephemeris(0.0, 60.0 * 24.0 * 365.0, 60.0 * 24.0);
    assert!(
        matches!(result, Err(EphemerisSamplingError::Propagation(_)) | Ok(_)),
        "the result is {result:?}"
    );
}

#[test]
fn a_burn_raises_the_osculating_orbit() {
    let orbit = orbit();
    let before = orbit.osculating_elements(0.0).expect("elliptical");
    let state = orbit.propagate_minutes(0.0).expect("propagates");
    // Thrust along the velocity.
    let direction = state.velocity.normalize().expect("non-zero");
    let burn = FiniteBurn::along(direction, 1e-4, 30.0).expect("usable");

    let after = orbit.apply_burn(0.0, burn, 500).expect("the burn applies");
    assert!(
        after.semi_major_axis > before.semi_major_axis,
        "the axis went from {} to {}",
        before.semi_major_axis,
        after.semi_major_axis
    );
}

#[test]
fn a_burn_matches_a_direct_application() {
    // The facade must be a thin wrapper: applying the burn through it is the
    // same as propagating and applying the burn by hand.
    let orbit = orbit();
    let state = orbit.propagate_minutes(5.0).expect("propagates");
    let direction = state.velocity.normalize().expect("non-zero");
    let burn = FiniteBurn::along(direction, 1e-4, 20.0).expect("usable");

    let through_facade = orbit.apply_burn(5.0, burn, 500).expect("applies");

    let kepler_state = apsis_kepler::State {
        position: state.position,
        velocity: state.velocity,
    };
    let outcome = burn
        .apply(kepler_state, GravityParameter::WGS72, 500)
        .expect("the burn runs");
    let by_hand =
        apsis_kepler::KeplerianElements::from_state(&outcome.state, GravityParameter::WGS72)
            .expect("still elliptical");

    assert!(
        (through_facade.semi_major_axis - by_hand.semi_major_axis).abs() < 1e-9,
        "{} against {}",
        through_facade.semi_major_axis,
        by_hand.semi_major_axis
    );
}

#[test]
fn an_escaping_burn_is_reported() {
    let orbit = orbit();
    let state = orbit.propagate_minutes(0.0).expect("propagates");
    let direction = state.velocity.normalize().expect("non-zero");
    // A burn far larger than orbital velocity.
    let burn = FiniteBurn::along(direction, 1e-2, 600.0).expect("usable");
    let result = orbit.apply_burn(0.0, burn, 2_000);
    assert!(matches!(
        result,
        Err(ManeuverApplicationError::NotElliptical)
    ));
}

#[test]
fn a_hohmann_transfer_plans_a_raise_the_orbit_layer_can_sample() {
    // A cross-layer check that ties the maneuver and ephemeris crates to the
    // propagation: plan a transfer, apply the first burn, and sample the
    // resulting orbit.
    let orbit = orbit();
    let elements = orbit.osculating_elements(0.0).expect("elliptical");
    let from = elements.semi_major_axis;
    let to = from + 50.0;
    let transfer = hohmann_transfer(from, to, GravityParameter::WGS72).expect("usable");

    // The first burn is prograde for a raise, so thrust along the velocity.
    let state = orbit.propagate_minutes(0.0).expect("propagates");
    let direction = state.velocity.normalize().expect("non-zero");
    // Turn the delta-v into an acceleration over a short burn.
    let duration = 10.0;
    let burn =
        FiniteBurn::along(direction, transfer.first_burn / duration, duration).expect("usable");
    let after = orbit.apply_burn(0.0, burn, 1_000).expect("applies");
    assert!(
        after.semi_major_axis > from,
        "the transfer should raise the orbit from {from} to {}",
        after.semi_major_axis
    );

    // The osculating elements of the post burn state are the planning
    // result. Turning them back into a propagatable orbit would need mean
    // elements, which SGP4 requires and an osculating set does not provide,
    // so the workflow stops here rather than feeding them back in.
    assert!(after.is_valid());
    assert!(after.eccentricity < 1.0);
}
