//! Tests for line of sight and visibility from a ground station.
//!
//! 自地面站的视线与可见性的测试。

use apsis_coverage::{
    GroundPoint, Sensor, central_angle_to_station, elevation_angle, line_of_sight, local_up,
    sub_satellite_point, visible_from, visible_from_geodetic,
};
use apsis_geodesy::{Geodetic, WGS84};
use apsis_math::Vec3;
use core::f64::consts::{FRAC_PI_2, PI};

/// The Earth equatorial radius, in kilometers.
///
/// 地球赤道半径，单位为千米。
const BODY_RADIUS: f64 = 6_378.137;

/// A station at the equator on the prime meridian, at sea level.
///
/// 赤道上本初子午线处、海平面的一座站。
fn equatorial_station() -> GroundPoint {
    GroundPoint::new(Geodetic::new(0.0, 0.0, 0.0), 0.0)
}

#[test]
fn a_station_on_the_equator_is_at_the_equatorial_radius() {
    let station = equatorial_station();
    let position = station.ecef();
    assert!((position.x - BODY_RADIUS).abs() < 1e-6);
    assert!(position.y.abs() < 1e-6);
    assert!(position.z.abs() < 1e-6);
    assert!((station.radius() - BODY_RADIUS).abs() < 1e-6);
}

#[test]
fn a_station_at_the_pole_is_at_the_polar_radius() {
    // The polar radius is the semi-minor axis of the ellipsoid, which is
    // about 21 km less than the equatorial radius.
    // 极半径是椭球的半短轴，比赤道半径小约 21 km。
    let station = GroundPoint::new(Geodetic::new(FRAC_PI_2, 0.0, 0.0), 0.0);
    let radius = station.radius();
    assert!((radius - 6_356.752).abs() < 0.01, "radius {radius}");
    assert!(radius < BODY_RADIUS);
}

#[test]
fn the_local_up_equals_the_position_direction_at_the_equator() {
    let station = equatorial_station();
    let up = local_up(&station);
    assert!((up.x - 1.0).abs() < 1e-12);
    assert!(up.y.abs() < 1e-12);
    assert!(up.z.abs() < 1e-12);
}

#[test]
fn the_local_up_at_the_pole_is_the_third_axis() {
    let station = GroundPoint::new(Geodetic::new(FRAC_PI_2, 0.0, 0.0), 0.0);
    let up = local_up(&station);
    assert!(up.x.abs() < 1e-12);
    assert!(up.y.abs() < 1e-12);
    assert!((up.z - 1.0).abs() < 1e-12);
}

#[test]
fn a_satellite_overhead_is_at_the_zenith() {
    let station = equatorial_station();
    // Directly above the station along the local normal.
    // 沿当地法线位于该站正上方。
    let satellite = Vec3::new(BODY_RADIUS + 550.0, 0.0, 0.0);
    let elevation = elevation_angle(satellite, &station).expect("valid");
    assert!(
        (elevation - FRAC_PI_2).abs() < 1e-9,
        "elevation {elevation}"
    );
}

#[test]
fn a_satellite_at_the_horizon_has_zero_elevation() {
    let station = equatorial_station();
    // The local horizon is where the line of sight is tangent to the
    // sphere, at a central angle of about 22.98 degrees for a satellite at
    // 550 km. At that geometry the elevation is zero and the satellite is
    // at its greatest distance along the surface.
    // 当地地平是视线与球面相切之处，对 550 km 的卫星中心角约为 22.98 度。在该几何
    // 处仰角为零，卫星沿地表距站最远。
    let central = (BODY_RADIUS / (BODY_RADIUS + 550.0)).acos();
    let satellite = Vec3::new(
        (BODY_RADIUS + 550.0) * central.cos(),
        (BODY_RADIUS + 550.0) * central.sin(),
        0.0,
    );
    let elevation = elevation_angle(satellite, &station).expect("valid");
    assert!(elevation.abs() < 1e-9, "elevation {elevation}");
}

#[test]
fn a_satellite_below_the_horizon_has_negative_elevation() {
    let station = equatorial_station();
    // On the far side of the Earth, the elevation is negative.
    // 在地球远侧，仰角为负。
    let satellite = Vec3::new(-(BODY_RADIUS + 550.0), 0.0, 0.0);
    let elevation = elevation_angle(satellite, &station).expect("valid");
    assert!(elevation < -1.0, "elevation {elevation}");
}

#[test]
fn the_elevation_is_symmetric_about_the_overhead_direction() {
    let station = equatorial_station();
    // Two satellites at the same angular distance from the overhead point,
    // one to the east and one to the west, have the same elevation.
    // 距过顶点的角距离相同、一个在东一个在西的两颗卫星具有相同的仰角。
    let angle = 0.3_f64;
    let east = Vec3::new(
        (BODY_RADIUS + 550.0) * angle.cos(),
        (BODY_RADIUS + 550.0) * angle.sin(),
        0.0,
    );
    let west = Vec3::new(
        (BODY_RADIUS + 550.0) * angle.cos(),
        -(BODY_RADIUS + 550.0) * angle.sin(),
        0.0,
    );
    let east_elevation = elevation_angle(east, &station).expect("valid");
    let west_elevation = elevation_angle(west, &station).expect("valid");
    assert!((east_elevation - west_elevation).abs() < 1e-12);
}

#[test]
fn visibility_respects_the_elevation_mask() {
    let station = GroundPoint::new(Geodetic::new(0.0, 0.0, 0.0), 0.2);
    let sensor = Sensor::omnidirectional(station);

    // Overhead is well above the mask.
    // 过顶处远高于遮蔽。
    assert_eq!(
        visible_from(Vec3::new(BODY_RADIUS + 550.0, 0.0, 0.0), &sensor),
        Some(true)
    );
    // On the horizon the satellite is below a 0.2 radian mask, because the
    // horizon direction is a right angle from the local up vector.
    // 在地平处卫星低于 0.2 弧度的遮蔽，因为地平方向与当地朝上矢量成直角。
    assert_eq!(
        visible_from(Vec3::new(0.0, BODY_RADIUS + 550.0, 0.0), &sensor),
        Some(false)
    );
    // On the far side is well below it.
    // 在远侧远低于它。
    assert_eq!(
        visible_from(Vec3::new(-(BODY_RADIUS + 550.0), 0.0, 0.0), &sensor),
        Some(false)
    );
}

#[test]
fn a_zero_mask_admits_a_satellite_at_the_horizon() {
    let sensor = Sensor::omnidirectional(equatorial_station());
    // A satellite just inside the horizon direction has a small positive
    // elevation and is visible with a zero mask.
    // 恰在地平方向之内的卫星有小正仰角，在零遮蔽下可见。
    let central = (BODY_RADIUS / (BODY_RADIUS + 550.0)).acos() * 0.999;
    let satellite = Vec3::new(
        (BODY_RADIUS + 550.0) * central.cos(),
        (BODY_RADIUS + 550.0) * central.sin(),
        0.0,
    );
    assert_eq!(visible_from(satellite, &sensor), Some(true));
}

#[test]
fn a_directional_sensor_only_sees_its_forward_hemisphere() {
    // The station points at the zenith, so it sees a satellite overhead but
    // not one on the horizon behind it.
    // 该站指向天顶，故它看到过顶的卫星，但看不到其后方地平的卫星。
    let station = equatorial_station();
    let sensor = Sensor::directional(station, Vec3::new(1.0, 0.0, 0.0));

    assert_eq!(
        visible_from(Vec3::new(BODY_RADIUS + 550.0, 0.0, 0.0), &sensor),
        Some(true)
    );
    // A satellite to the east is at a right angle to the pointing, so it is
    // not in the forward hemisphere.
    // 东侧的卫星与指向成直角，故不在前半球。
    assert_eq!(
        visible_from(Vec3::new(0.0, BODY_RADIUS + 550.0, 0.0), &sensor),
        Some(false)
    );
}

#[test]
fn a_directional_sensor_normalizes_its_pointing() {
    // An unnormalized pointing vector must behave the same as a unit one.
    // 未归一化的指向矢量必须与单位矢量表现相同。
    let station = equatorial_station();
    let unit = Sensor::directional(station, Vec3::new(1.0, 0.0, 0.0));
    let scaled = Sensor::directional(station, Vec3::new(100.0, 0.0, 0.0));
    let satellite = Vec3::new(BODY_RADIUS + 550.0, 0.0, 0.0);
    assert_eq!(
        visible_from(satellite, &unit),
        visible_from(satellite, &scaled)
    );
}

#[test]
fn a_degenerate_pointing_is_rejected() {
    let station = equatorial_station();
    let sensor = Sensor::directional(station, Vec3::ZERO);
    let satellite = Vec3::new(BODY_RADIUS + 550.0, 0.0, 0.0);
    assert_eq!(visible_from(satellite, &sensor), None);
}

#[test]
fn the_geodetic_helper_agrees_with_the_sensor_path() {
    let position = Geodetic::new(0.9, -1.2, 100.0);
    let mask = 0.15;
    let satellite = {
        // A point above the station.
        // 该站上方的一个点。
        let station = GroundPoint::new(position, mask);
        station.ecef() * 1.09
    };
    let via_sensor = visible_from_geodetic(satellite, position, mask).expect("valid");
    let via_sensor_type = visible_from(
        satellite,
        &Sensor::omnidirectional(GroundPoint::new(position, mask)),
    )
    .expect("valid");
    assert_eq!(via_sensor, via_sensor_type);
    assert!(via_sensor);
}

#[test]
fn the_sub_satellite_point_is_at_zero_height() {
    let satellite = Vec3::new(BODY_RADIUS + 550.0, 0.0, 0.0);
    let point = sub_satellite_point(satellite, WGS84).expect("valid");
    assert_eq!(point.position.height, 0.0);
    assert!(point.position.latitude.abs() < 1e-9);
    assert!(point.position.longitude.abs() < 1e-9);
}

#[test]
fn the_sub_satellite_point_keeps_the_longitude() {
    let satellite = Vec3::new(0.0, 4_000.0, 4_500.0);
    let point = sub_satellite_point(satellite, WGS84).expect("valid");
    assert!((point.position.longitude - FRAC_PI_2).abs() < 1e-9);
    assert!(point.position.latitude > 0.0);
}

#[test]
fn a_satellite_on_the_polar_axis_has_no_sub_satellite_point() {
    let satellite = Vec3::new(0.0, 0.0, BODY_RADIUS + 550.0);
    assert_eq!(sub_satellite_point(satellite, WGS84), None);
    assert_eq!(sub_satellite_point(Vec3::splat(f64::NAN), WGS84), None);
}

#[test]
fn the_central_angle_to_a_station_follows_the_spherical_relation() {
    // A satellite at the zenith of the station has a central angle of zero.
    // 位于该站天顶的卫星中心角为零。
    let station = equatorial_station();
    let satellite = Vec3::new(BODY_RADIUS + 550.0, 0.0, 0.0);
    let central = central_angle_to_station(satellite, &station, FRAC_PI_2).expect("valid");
    assert!(central.abs() < 1e-9);

    // At a ten degree elevation the central angle is the footprint angle.
    // 十度仰角处中心角即覆盖区角。
    let central =
        central_angle_to_station(satellite, &station, 10.0_f64.to_radians()).expect("valid");
    assert!((central.to_degrees() - 14.956_635_124_957_29).abs() < 1e-6);
}

#[test]
fn invalid_inputs_are_rejected() {
    let station = equatorial_station();
    assert_eq!(elevation_angle(Vec3::splat(f64::NAN), &station), None);
    // A station with an out of range mask is not usable.
    // 遮蔽越界的站不可用。
    let bad = GroundPoint::new(Geodetic::new(0.0, 0.0, 0.0), PI);
    assert!(!bad.is_valid());
    assert_eq!(
        visible_from(
            Vec3::new(BODY_RADIUS + 550.0, 0.0, 0.0),
            &Sensor::omnidirectional(bad)
        ),
        None
    );
    // A satellite at the same place as the station has no line of sight.
    // 与站同处的卫星没有视线。
    let coincident = station.ecef();
    assert_eq!(line_of_sight(coincident, &station), None);
}

#[test]
fn a_station_above_sea_level_has_a_larger_radius() {
    let sea_level = GroundPoint::new(Geodetic::new(0.0, 0.0, 0.0), 0.0);
    let high = GroundPoint::new(Geodetic::new(0.0, 0.0, 3_000.0), 0.0);
    assert!((high.radius() - sea_level.radius() - 3.0).abs() < 1e-6);
}

#[test]
fn the_sensor_and_ground_point_validity_is_reported() {
    assert!(equatorial_station().is_valid());
    assert!(GroundPoint::new(Geodetic::new(0.9, 0.9, 0.0), 0.1).is_valid());
    assert!(!GroundPoint::new(Geodetic::new(0.0, 0.0, 0.0), 2.0).is_valid());
    assert!(!GroundPoint::new(Geodetic::new(0.0, 0.0, f64::NAN), 0.1).is_valid());
}
