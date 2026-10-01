//! Tests for the frame carrying position types.
//!
//! 携带坐标架的位置类型的测试。

use apsis_coord::{EarthRotationAngle, Ecef, Eci, Geodetic, WGS84};
use apsis_math::Vec3;

const EPS: f64 = 1e-9;

#[test]
fn constructors_round_trip_the_vector() {
    let vector = Vec3::new(1.0, 2.0, 3.0);
    assert_eq!(Eci::new(vector).vector(), vector);
    assert_eq!(Ecef::new(vector).vector(), vector);
}

#[test]
fn geodetic_fields_round_trip() {
    let position = Geodetic::new(0.5, -1.0, 250.0);
    assert_eq!(position.latitude(), 0.5);
    assert_eq!(position.longitude(), -1.0);
    assert_eq!(position.height(), 250.0);
    assert_eq!(position.ellipsoidal().latitude, 0.5);
}

#[test]
fn geodetic_to_ecef_uses_wgs84_by_default() {
    let position = Geodetic::new(0.35, -1.2, 123.0);
    assert_eq!(position.to_ecef(), position.to_ecef_on(WGS84));
}

#[test]
fn geodetic_to_ecef_and_back_round_trips() {
    let original = Geodetic::new(0.921, -1.234, 12_345.0);
    let recovered = original
        .to_ecef()
        .to_geodetic(WGS84)
        .expect("off-axis points have a longitude");
    assert!((recovered.latitude() - original.latitude()).abs() < 1e-11);
    assert!((recovered.longitude() - original.longitude()).abs() < 1e-11);
    assert!((recovered.height() - original.height()).abs() < 1e-4);
}

#[test]
fn polar_axis_has_no_defined_longitude() {
    let on_axis = Ecef::new(Vec3::new(0.0, 0.0, WGS84.semi_minor_axis()));
    assert_eq!(on_axis.to_geodetic(WGS84), None);
    assert_eq!(Ecef::new(Vec3::ZERO).to_geodetic(WGS84), None);
}

#[test]
fn frame_conversions_round_trip() {
    let angle = EarthRotationAngle::from_ut1_days(9.25);
    let original = Eci::new(Vec3::new(7_000.0, -1_500.0, 3_200.0));
    let ecef = original.to_ecef(angle);
    let back = ecef.to_eci(angle);
    assert!((back.vector() - original.vector()).norm() < EPS);
}

#[test]
fn frame_conversion_preserves_length() {
    let angle = EarthRotationAngle::from_ut1_days(-3.5);
    let vector = Vec3::new(6_878_137.0, 0.0, 0.0);
    let converted = Eci::new(vector).to_ecef(angle);
    assert!((converted.vector().norm() - vector.norm()).abs() < 1e-6);
}

#[test]
fn a_ground_point_rotates_with_the_earth() {
    // A point on the equator at zero longitude in the Earth-fixed frame is
    // rotated by the Earth Rotation Angle in the inertial frame.
    // 地固系中赤道上零经度的一个点，在惯性系中按地球自转角被转动。
    let angle = EarthRotationAngle::from_ut1_days(1.5);
    let ground = Ecef::new(Vec3::new(6_378_137.0, 0.0, 0.0));
    let inertial = ground.to_eci(angle);
    let radians = angle.radians_positive();
    let expected = Vec3::new(
        6_378_137.0 * radians.cos(),
        6_378_137.0 * radians.sin(),
        0.0,
    );
    assert!((inertial.vector() - expected).norm() < 1e-6);
}

#[test]
fn a_geodetic_station_round_trips_through_both_frames() {
    // A station is fixed to the ground, so converting it to the inertial
    // frame and back at the same angle returns the original position.
    // 地面站固定于地面，故把它转入惯性系再以相同角度转回，得到原位置。
    let station = Geodetic::new(0.662, 0.175, 80.0);
    let angle = EarthRotationAngle::from_ut1_julian_date(2_460_000.5);
    let inertial = station.to_ecef().to_eci(angle);
    let back = inertial.to_ecef(angle).to_geodetic(WGS84).expect("station");
    assert!((back.latitude() - station.latitude()).abs() < 1e-11);
    assert!((back.longitude() - station.longitude()).abs() < 1e-11);
    assert!((back.height() - station.height()).abs() < 1e-4);
}
