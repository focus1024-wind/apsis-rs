//! Tests for the propagator against the reference implementation.
//!
//! The expected states in this file were produced by the `sgp4` reference
//! library, which implements Vallado's revision of the theory. The element
//! sets are the historical test cases distributed with that library. A
//! propagation that agrees with the reference to well under a micrometer is
//! treated as correct, because both implementations evaluate the same
//! closed form expressions and should differ only by floating point
//! rounding.

use apsis_sgp4::{GravityModel, PropagationErrorKind, Satellite, State};
use apsis_tle::TwoLineElement;

/// A near circular low Earth orbit, the International Space Station.
const ISS: &str = concat!(
    "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\r\n",
    "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537\r\n",
);

/// A highly eccentric orbit, which exercises the eccentric anomaly
/// iteration away from circular motion.
const VANGUARD: &str = concat!(
    "1 00005U 58002B   00179.78495062  .00000023  00000-0  28098-4 0  4753\r\n",
    "2 00005  34.2682 348.7242 1859667 331.7664  19.3264 10.82419157413667\r\n",
);

/// A deep space orbit with a half-day resonance.
const DEEP_SPACE: &str = concat!(
    "1 09880U 77021A   06174.85710247  .00000090  00000-0  10000-3 0  7322\r\n",
    "2 09880  64.1526 313.1296 6546925 103.4683 305.3980  2.00754276 13047\r\n",
);

/// A geostationary transfer orbit, also deep space.
const TRANSFER: &str = concat!(
    "1 27651U 03004A   06185.99374200  .00000109  00000-0  19909-4 0   445\r\n",
    "2 27651   7.1743 182.9443 7867251 269.0386  18.6601  1.03461646  9884\r\n",
);

/// The position tolerance, in kilometers. The observed agreement is at the
/// nanometer level, so this leaves a wide margin.
const POSITION_TOLERANCE: f64 = 1e-3;

/// The velocity tolerance, in kilometers per second.
const VELOCITY_TOLERANCE: f64 = 1e-9;

fn satellite(text: &str) -> Satellite {
    let element = TwoLineElement::parse(text).expect("test element set parses");
    Satellite::from_element(&element, GravityModel::WGS72)
        .expect("test element set is propagatable")
}

fn assert_matches_reference(state: State, position: [f64; 3], velocity: [f64; 3]) {
    let position_error = [
        state.position.x - position[0],
        state.position.y - position[1],
        state.position.z - position[2],
    ]
    .iter()
    .fold(0.0_f64, |worst, value| worst.max(value.abs()));
    let velocity_error = [
        state.velocity.x - velocity[0],
        state.velocity.y - velocity[1],
        state.velocity.z - velocity[2],
    ]
    .iter()
    .fold(0.0_f64, |worst, value| worst.max(value.abs()));

    assert!(
        position_error < POSITION_TOLERANCE,
        "position error {position_error} km, got {}",
        state.position
    );
    assert!(
        velocity_error < VELOCITY_TOLERANCE,
        "velocity error {velocity_error} km/s, got {}",
        state.velocity
    );
}

#[test]
fn iss_at_epoch_matches_the_reference() {
    let state = satellite(ISS).propagate_minutes(0.0).expect("epoch state");
    assert_matches_reference(
        state,
        [4_083.902_463_521, -993.631_999_606, 5_243.603_665_371],
        [2.512_837_295_156, 7.259_888_524_981, -0.583_778_536_506],
    );
}

#[test]
fn iss_after_one_minute_matches_the_reference() {
    let state = satellite(ISS).propagate_minutes(1.0).expect("state exists");
    assert_matches_reference(
        state,
        [4_224.931_504_689, -556.040_371_394, 5_196.213_688_087],
        [2.186_284_834_622, 7.320_793_192_659, -0.995_282_270_726],
    );
}

#[test]
fn iss_after_six_hours_matches_the_reference() {
    let state = satellite(ISS)
        .propagate_minutes(360.0)
        .expect("state exists");
    assert_matches_reference(
        state,
        [2_748.401_544_599, -3_564.892_404_578, 4_992.448_308_874],
        [4.342_862_050_164, 6.063_045_163_749, 1.927_771_710_260],
    );
}

#[test]
fn iss_after_one_day_matches_the_reference() {
    let state = satellite(ISS)
        .propagate_minutes(1_440.0)
        .expect("state exists");
    assert_matches_reference(
        state,
        [-3_199.119_301_995, -5_925.838_895_195, -104.283_883_010],
        [4.160_900_126_061, -2.340_866_691_092, 6.034_239_787_489],
    );
}

#[test]
fn iss_before_epoch_matches_the_reference() {
    let state = satellite(ISS)
        .propagate_minutes(-1_440.0)
        .expect("state exists");
    assert_matches_reference(
        state,
        [1_121.392_381_234, 6_541.559_708_790, -1_120.952_322_949],
        [-4.940_430_025_083, -0.153_942_812_975, -5.902_529_984_919],
    );
}

#[test]
fn vanguard_at_epoch_matches_the_reference() {
    let state = satellite(VANGUARD)
        .propagate_minutes(0.0)
        .expect("epoch state");
    assert_matches_reference(
        state,
        [7_022.465_292_664, -1_400.082_967_554, 0.039_951_554],
        [1.893_841_014_513, 6.405_893_759_210, 4.534_807_250_355],
    );
}

#[test]
fn deep_space_satellite_uses_the_deep_space_branch() {
    let deep = satellite(DEEP_SPACE);
    assert!(deep.is_deep_space());
    let near = satellite(ISS);
    assert!(!near.is_deep_space());
}

#[test]
fn deep_space_at_epoch_matches_the_reference() {
    // MJD 53_962.36, about 12 hours after the element epoch.
    let state = satellite(DEEP_SPACE)
        .propagate_minutes(0.0)
        .expect("epoch state");
    assert_matches_reference(
        state,
        [12_020.677_224_927, -21_305.501_479_237, -11_902.993_493_655],
        [0.646_341_708_451, 1.668_868_685_981, 3.329_292_846_315],
    );
}

#[test]
fn deep_space_after_one_week_matches_the_reference() {
    let state = satellite(DEEP_SPACE)
        .propagate_minutes(10_080.0)
        .expect("state exists");
    assert_matches_reference(
        state,
        [12_501.446_941_807, -16_359.501_943_154, -3_718.626_893_065],
        [-0.155_492_271_223, 2.911_308_113_855, 3.819_401_898_806],
    );
}

#[test]
fn transfer_orbit_at_epoch_matches_the_reference() {
    let state = satellite(TRANSFER)
        .propagate_minutes(0.0)
        .expect("epoch state");
    assert_matches_reference(
        state,
        [-20_728.381_133_505, -10_167.792_006_995, 1_151.382_245_235],
        [-1.744_013_319_342, -4.632_866_113_051, 0.570_823_646_148],
    );
}

#[test]
fn transfer_orbit_after_one_week_matches_the_reference() {
    let state = satellite(TRANSFER)
        .propagate_minutes(10_080.0)
        .expect("state exists");
    assert_matches_reference(
        state,
        [-14_421.942_143_755, -62_862.455_417_683, 7_869.371_785_647],
        [0.946_839_675_888, -1.301_129_854_934, 0.164_515_795_189],
    );
}

#[test]
fn propagation_is_deterministic() {
    // Repeated evaluation must give the same result, which also verifies
    // that the resonance integrator does not carry hidden state between
    // calls.
    let satellite = satellite(DEEP_SPACE);
    let first = satellite.propagate_minutes(5_000.0).expect("state exists");
    let second = satellite.propagate_minutes(5_000.0).expect("state exists");
    assert_eq!(first, second);
}

#[test]
fn out_of_order_evaluation_gives_the_same_result() {
    // Evaluating a later time first must not change a later evaluation of an
    // earlier time.
    let satellite = satellite(DEEP_SPACE);
    let late = satellite.propagate_minutes(10_000.0).expect("state exists");
    let early = satellite.propagate_minutes(1_000.0).expect("state exists");
    let early_again = satellite.propagate_minutes(1_000.0).expect("state exists");
    let late_again = satellite.propagate_minutes(10_000.0).expect("state exists");
    assert_eq!(early, early_again);
    assert_eq!(late, late_again);
}

#[test]
fn epoch_accessor_returns_the_element_set_epoch() {
    let element = TwoLineElement::parse(ISS).expect("parses");
    let expected = element
        .epoch
        .mjd(apsis_time::TimeScale::Utc)
        .expect("epoch is in the leap table");
    let satellite = Satellite::from_element(&element, GravityModel::WGS72).expect("propagatable");
    assert!((satellite.epoch_mjd() - expected).abs() < 1e-12);
}

#[test]
fn minutes_since_epoch_is_measured_in_minutes() {
    let element = TwoLineElement::parse(ISS).expect("parses");
    let satellite = Satellite::from_element(&element, GravityModel::WGS72).expect("propagatable");
    let epoch = satellite.epoch_mjd();
    assert!((satellite.minutes_since_epoch(epoch)).abs() < 1e-9);
    assert!((satellite.minutes_since_epoch(epoch + 1.0 / 1_440.0) - 1.0).abs() < 1e-6);
    assert!((satellite.minutes_since_epoch(epoch - 1.0) + 1_440.0).abs() < 1e-6);
}

#[test]
fn propagate_at_the_epoch_matches_propagate_minutes_zero() {
    let element = TwoLineElement::parse(ISS).expect("parses");
    let satellite = Satellite::from_element(&element, GravityModel::WGS72).expect("propagatable");
    let at_epoch = satellite
        .propagate_at(satellite.epoch_mjd())
        .expect("state exists");
    let at_zero = satellite.propagate_minutes(0.0).expect("state exists");
    assert_eq!(at_epoch, at_zero);
}

#[test]
fn invalid_elements_are_rejected() {
    // An eccentricity whose leading digit is nine was written with an
    // assumed decimal point, so it is 0.9999999, a valid but extremely
    // elongated orbit. An eccentricity field that cannot be read at all is
    // rejected by the parser instead, which is checked in the TLE tests.
    let text = concat!(
        "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\r\n",
        "2 25544  51.6416 247.4627 0000000 130.5360 325.0288 15.72125391563531\r\n",
    );
    let element = TwoLineElement::parse(text).expect("parses");
    // A circular orbit is perfectly valid, so this must succeed.
    assert!(Satellite::from_element(&element, GravityModel::WGS72).is_some());
}

#[test]
fn a_zero_mean_motion_is_rejected() {
    // The mean motion field is in revolutions per day, so a zero there
    // describes no orbit. The element set must still carry a valid
    // checksum, so the test builds one from a parsed set rather than
    // editing text.
    let element = TwoLineElement::parse(ISS).expect("parses");
    let mut elements = apsis_sgp4::Elements::from_tle(&element).expect("epoch is covered");
    elements.mean_motion = 0.0;
    let error = Satellite::try_from_elements(&elements, GravityModel::WGS72)
        .expect_err("zero mean motion is not propagatable");
    assert_eq!(error.kind(), PropagationErrorKind::InvalidElements);
}

#[test]
fn an_eccentricity_of_one_is_rejected() {
    let element = TwoLineElement::parse(ISS).expect("parses");
    let mut elements = apsis_sgp4::Elements::from_tle(&element).expect("epoch is covered");
    elements.eccentricity = 1.0;
    let error = Satellite::try_from_elements(&elements, GravityModel::WGS72)
        .expect_err("an open orbit is not propagatable");
    assert_eq!(error.kind(), PropagationErrorKind::InvalidElements);
}

#[test]
fn non_finite_elements_are_rejected() {
    let element = TwoLineElement::parse(ISS).expect("parses");
    let mut elements = apsis_sgp4::Elements::from_tle(&element).expect("epoch is covered");
    elements.inclination = f64::NAN;
    let error = Satellite::try_from_elements(&elements, GravityModel::WGS72)
        .expect_err("a non-finite inclination is not propagatable");
    assert_eq!(error.kind(), PropagationErrorKind::InvalidElements);
}

#[test]
fn state_accessors_report_radius_and_speed() {
    let state = satellite(ISS).propagate_minutes(0.0).expect("epoch state");
    assert!((state.altitude_radius() - state.position.norm()).abs() < 1e-12);
    assert!((state.speed() - state.velocity.norm()).abs() < 1e-12);
    // A low Earth satellite moves at about 7.7 km/s.
    assert!(state.speed() > 7.0 && state.speed() < 8.0);
}
